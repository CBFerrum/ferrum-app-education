//! Fee schedules and report cards.
//!
//! * A fee schedule is a fee structure due on a day for every student enrolled in its program and
//!   year (or one group of them). Making it makes each student's fees once; invoicing it invoices
//!   those still drafts, each through ERP as one student's fees are.
//! * A report card is a student's year: each course's assessment results in the year averaged and
//!   graded (on the scale of the course's latest assessment), their attendance at classes in the
//!   year, and a comment. Worked out again whenever a draft is saved; published, it is kept as it
//!   was, and the family sees it on their portal.

use crate::fees::invoice;
use crate::marks::grade_of;
use crate::obj;
use async_trait::async_trait;
use chrono::Utc;
use ferrum_domain::util::{elevated, id, money, round, s, set, sys};
use ferrum_kernel::{Doc, DocHook, Error, HookCtx, Result};
use rust_decimal::Decimal;
use serde_json::{Value, json};
use uuid::Uuid;

pub struct ScheduleHook;

#[async_trait]
impl DocHook for ScheduleHook {
    fn name(&self) -> &'static str {
        "education.fee_schedule"
    }

    async fn validate(&self, hc: &mut HookCtx<'_>, doc: &Doc) -> Result<()> {
        // A group is one of the structure's program and year.
        if let (Some(group), Some(structure)) = (id(doc, "student_group"), id(doc, "fee_structure")) {
            let same: bool = sqlx::query_scalar(
                "select exists(select 1 from e_student_group g join e_fee_structure f on f.tenant_id = g.tenant_id and f.program = g.program \
                 and f.academic_year = g.academic_year where g.tenant_id = $1 and g.id = $2 and f.id = $3)",
            )
            .bind(hc.ctx.tenant)
            .bind(group)
            .bind(structure)
            .fetch_one(&mut **hc.tx)
            .await?;
            if !same {
                return Err(Error::field("student_group", "is a group of another program or year than the fee structure's"));
            }
        }
        Ok(())
    }

    async fn action(&self, hc: &mut HookCtx<'_>, name: &str, doc: &Doc, _params: &Value) -> Result<Option<Value>> {
        let me = id(doc, "id").ok_or_else(|| Error::internal("a schedule without an id"))?;
        let tenant = hc.ctx.tenant;
        let system = elevated(hc.ctx);
        match name {
            "make_fees" => {
                let structure = id(doc, "fee_structure").ok_or_else(|| Error::field("fee_structure", "is required"))?;
                let due = s(doc, "due_on").ok_or_else(|| Error::field("due_on", "is required"))?.to_string();
                // Its students: enrolled in the structure's program and year (and in its group),
                // active, without fees for this structure and day.
                let students: Vec<Uuid> = sqlx::query_scalar(
                    "select distinct e.student from e_program_enrollment e join e_fee_structure f on f.tenant_id = e.tenant_id and f.program = e.program \
                     and f.academic_year = e.academic_year join e_student st on st.tenant_id = e.tenant_id and st.id = e.student \
                     where e.tenant_id = $1 and f.id = $2 and st.status = 'Active' \
                     and ($3::uuid is null or exists (select 1 from e_student_group__students g where g.tenant_id = e.tenant_id and g.parent_id = $3 and g.student = e.student)) \
                     and not exists (select 1 from e_fees x where x.tenant_id = e.tenant_id and x.student = e.student and x.fee_structure = $2 \
                                     and x.due_on = $4::date and x.status <> 'Cancelled')",
                )
                .bind(tenant)
                .bind(structure)
                .bind(id(doc, "student_group"))
                .bind(&due)
                .fetch_all(&mut **hc.tx)
                .await?;
                for student in &students {
                    let input = json!({"student": student.to_string(), "fee_structure": structure.to_string(), "due_on": due, "fee_schedule": me.to_string()});
                    hc.kernel.insert_tx(&system, hc.tx, "Fees", obj(input)).await?;
                }
                let made: i64 = sqlx::query_scalar("select count(*) from e_fees where tenant_id = $1 and fee_schedule = $2")
                    .bind(tenant)
                    .bind(me)
                    .fetch_one(&mut **hc.tx)
                    .await?;
                hc.kernel.set_values_tx(&system, hc.tx, "FeeSchedule", me, obj(json!({"fees_made": made}))).await?;
                let n = students.len();
                Ok(Some(json!({"fees_made": made, "message": format!("Made {n} {} fees; {made} in all", if n == 1 { "student's" } else { "students'" })})))
            }
            "invoice_fees" => {
                let drafts: Vec<Uuid> =
                    sqlx::query_scalar("select id from e_fees where tenant_id = $1 and fee_schedule = $2 and status = 'Draft' order by created_at")
                        .bind(tenant)
                        .bind(me)
                        .fetch_all(&mut **hc.tx)
                        .await?;
                if drafts.is_empty() {
                    return Err(Error::invalid("no fees of this schedule are still to invoice: make them first"));
                }
                for fees in &drafts {
                    let doc = hc.kernel.get_tx(&sys(hc.ctx), hc.tx, "Fees", *fees, true).await?;
                    invoice(hc, &doc).await?;
                }
                let invoiced: i64 =
                    sqlx::query_scalar("select count(*) from e_fees where tenant_id = $1 and fee_schedule = $2 and status in ('Invoiced', 'Paid')")
                        .bind(tenant)
                        .bind(me)
                        .fetch_one(&mut **hc.tx)
                        .await?;
                hc.kernel.set_values_tx(&system, hc.tx, "FeeSchedule", me, obj(json!({"invoiced": invoiced}))).await?;
                Ok(Some(json!({"invoiced": invoiced, "message": format!("Invoiced {}: draft invoices in ERP", drafts.len())})))
            }
            _ => Ok(None),
        }
    }
}

pub struct ReportCardHook;

#[async_trait]
impl DocHook for ReportCardHook {
    fn name(&self) -> &'static str {
        "education.report_card"
    }

    fn fills(&self) -> &'static [&'static str] {
        &["program", "courses", "classes", "present", "absent", "late", "excused", "attendance_percent"]
    }

    async fn before_validate(&self, hc: &mut HookCtx<'_>, doc: &mut Doc) -> Result<()> {
        if s(doc, "status").is_some_and(|st| st != "Draft") {
            return Ok(());
        }
        let (Some(student), Some(year)) = (id(doc, "student"), id(doc, "academic_year")) else { return Ok(()) };
        let tenant = hc.ctx.tenant;
        let span: Option<(chrono::NaiveDate, chrono::NaiveDate)> =
            sqlx::query_as("select starts_on, ends_on from e_academic_year where tenant_id = $1 and id = $2")
                .bind(tenant)
                .bind(year)
                .fetch_optional(&mut **hc.tx)
                .await?;
        let (from, to) = span.ok_or_else(|| Error::field("academic_year", "is not a year here"))?;
        let program: Option<Uuid> = sqlx::query_scalar(
            "select program from e_program_enrollment where tenant_id = $1 and student = $2 and academic_year = $3 order by enrolled_on desc limit 1",
        )
        .bind(tenant)
        .bind(student)
        .bind(year)
        .fetch_optional(&mut **hc.tx)
        .await?;
        let program = program.ok_or_else(|| Error::field("academic_year", "is a year the student is not enrolled in"))?;
        // Each course's results in the year: how many, their average, and the latest scale.
        let courses: Vec<(Uuid, i64, Decimal, Option<Uuid>)> = sqlx::query_as(
            "select p.course, count(*), avg(r.percent), \
                    (array_agg(p.grading_scale order by p.date desc, p.created_at desc))[1] \
             from e_assessment_result r join e_assessment_plan p on p.tenant_id = r.tenant_id and p.id = r.assessment \
             where r.tenant_id = $1 and r.student = $2 and p.date between $3 and $4 group by p.course order by p.course",
        )
        .bind(tenant)
        .bind(student)
        .bind(from)
        .bind(to)
        .fetch_all(&mut **hc.tx)
        .await?;
        let mut lines = Vec::new();
        for (course, n, avg, scale) in courses {
            let percent = round(avg, 2);
            let bands: Vec<(String, Decimal)> = match scale {
                Some(scale) => {
                    sqlx::query_as("select grade, from_percent from e_grading_scale__grades where tenant_id = $1 and parent_id = $2")
                        .bind(tenant)
                        .bind(scale)
                        .fetch_all(&mut **hc.tx)
                        .await?
                }
                None => vec![],
            };
            lines.push(json!({"id": Uuid::now_v7().to_string(), "course": course.to_string(), "assessments": n, "percent": money(percent), "grade": grade_of(percent, &bands)}));
        }
        // Attendance at the year's classes.
        let (present, absent, late, excused): (i64, i64, i64, i64) = sqlx::query_as(
            "select count(*) filter (where a.status = 'Present'), count(*) filter (where a.status = 'Absent'), \
                    count(*) filter (where a.status = 'Late'), count(*) filter (where a.status = 'Excused') \
             from e_student_attendance a join e_course_schedule c on c.tenant_id = a.tenant_id and c.id = a.course_schedule \
             where a.tenant_id = $1 and a.student = $2 and c.date between $3 and $4",
        )
        .bind(tenant)
        .bind(student)
        .bind(from)
        .bind(to)
        .fetch_one(&mut **hc.tx)
        .await?;
        let classes = present + absent + late + excused;
        let attended = if classes > 0 { round(Decimal::from((present + late) * 100) / Decimal::from(classes), 2) } else { Decimal::ZERO };
        set(doc, "program", json!(program.to_string()));
        set(doc, "courses", Value::Array(lines));
        for (k, v) in [("classes", classes), ("present", present), ("absent", absent), ("late", late), ("excused", excused)] {
            set(doc, k, json!(v));
        }
        set(doc, "attendance_percent", money(attended));
        Ok(())
    }

    async fn validate(&self, hc: &mut HookCtx<'_>, doc: &Doc) -> Result<()> {
        let other: Option<String> = sqlx::query_scalar(
            "select coalesce(number, id::text) from e_report_card where tenant_id = $1 and student = $2 and academic_year = $3 and id is distinct from $4 limit 1",
        )
        .bind(hc.ctx.tenant)
        .bind(id(doc, "student"))
        .bind(id(doc, "academic_year"))
        .bind(id(doc, "id"))
        .fetch_optional(&mut **hc.tx)
        .await?;
        if let Some(n) = other {
            return Err(Error::field("academic_year", format!("has its report card already: {n}")));
        }
        Ok(())
    }

    async fn validate_change(&self, hc: &mut HookCtx<'_>, old: Option<&Doc>, _doc: &Doc) -> Result<()> {
        if !hc.ctx.is_system && old.is_some_and(|o| s(o, "status") == Some("Published")) {
            return Err(Error::invalid("a published report card is kept as it was published"));
        }
        Ok(())
    }

    async fn action(&self, hc: &mut HookCtx<'_>, name: &str, doc: &Doc, _params: &Value) -> Result<Option<Value>> {
        if name != "publish" {
            return Ok(None);
        }
        let me = id(doc, "id").ok_or_else(|| Error::internal("a report card without an id"))?;
        // Worked out as it stands now, then kept.
        let fresh = hc.kernel.update_tx(&elevated(hc.ctx), hc.tx, "ReportCard", me, obj(json!({"comment": doc.get("comment")})), None).await?;
        if fresh.get("courses").and_then(Value::as_array).is_none_or(Vec::is_empty) && fresh.get("classes").and_then(Value::as_i64).unwrap_or(0) == 0 {
            return Err(Error::invalid("it has no results or attendance to report yet"));
        }
        let today = Utc::now().date_naive().to_string();
        hc.kernel
            .set_values_tx(&elevated(hc.ctx), hc.tx, "ReportCard", me, obj(json!({"status": "Published", "published_by": hc.ctx.email, "published_on": today})))
            .await?;
        Ok(Some(json!({"status": "Published", "message": "Published: the family sees it on their portal"})))
    }
}
