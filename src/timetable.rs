//! The timetable: a class is a group, a course, an instructor and a room at a time on a date; no
//! room, instructor or group is in two classes at once (ERPNext 13's Course Schedule checked the same
//! three). A group holds students enrolled in its program and year, each once; attendance is taken
//! only for a class's own students, once each.

use async_trait::async_trait;
use chrono::NaiveTime;
use ferrum_domain::util::{id, rows, s, set};
use ferrum_kernel::{Doc, DocHook, Error, HookCtx, Result};
use serde_json::json;
use std::collections::BTreeSet;
use uuid::Uuid;

fn time(doc: &Doc, k: &str) -> Result<NaiveTime> {
    s(doc, k).and_then(|t| NaiveTime::parse_from_str(t, "%H:%M").ok()).ok_or_else(|| Error::field(k, "is a time of day, HH:MM"))
}

pub struct ScheduleHook;

#[async_trait]
impl DocHook for ScheduleHook {
    fn name(&self) -> &'static str {
        "education.schedule"
    }

    fn fills(&self) -> &'static [&'static str] {
        &["from_time", "to_time"]
    }

    /// Times are kept as HH:MM, so that they compare as times.
    async fn before_validate(&self, _hc: &mut HookCtx<'_>, doc: &mut Doc) -> Result<()> {
        for k in ["from_time", "to_time"] {
            if let Ok(t) = time(doc, k) {
                set(doc, k, json!(t.format("%H:%M").to_string()));
            }
        }
        Ok(())
    }

    async fn validate(&self, hc: &mut HookCtx<'_>, doc: &Doc) -> Result<()> {
        let (from, to) = (time(doc, "from_time")?, time(doc, "to_time")?);
        if from >= to {
            return Err(Error::field("to_time", "is after the start"));
        }
        let me = id(doc, "id").unwrap_or_else(Uuid::nil);
        for (field, what) in [("room", "room"), ("instructor", "instructor"), ("student_group", "group")] {
            let Some(target) = id(doc, field) else { continue };
            let sql = match field {
                "room" => {
                    "select from_time || '-' || to_time from e_course_schedule where tenant_id = $1 and room = $2 and date = $3 and id <> $4 and from_time < $6 and to_time > $5 limit 1"
                }
                "instructor" => {
                    "select from_time || '-' || to_time from e_course_schedule where tenant_id = $1 and instructor = $2 and date = $3 and id <> $4 and from_time < $6 and to_time > $5 limit 1"
                }
                _ => {
                    "select from_time || '-' || to_time from e_course_schedule where tenant_id = $1 and student_group = $2 and date = $3 and id <> $4 and from_time < $6 and to_time > $5 limit 1"
                }
            };
            let clash: Option<String> = sqlx::query_scalar(sql)
                .bind(hc.ctx.tenant)
                .bind(target)
                .bind(crate::date(doc, "date").ok_or_else(|| Error::field("date", "is required"))?)
                .bind(me)
                .bind(from.format("%H:%M").to_string())
                .bind(to.format("%H:%M").to_string())
                .fetch_optional(&mut **hc.tx)
                .await?;
            if let Some(at) = clash {
                return Err(Error::field(field, format!("the {what} has another class then ({at})")));
            }
        }
        Ok(())
    }
}

pub struct GroupHook;

#[async_trait]
impl DocHook for GroupHook {
    fn name(&self) -> &'static str {
        "education.group"
    }

    async fn validate(&self, hc: &mut HookCtx<'_>, doc: &Doc) -> Result<()> {
        let students: Vec<Uuid> = rows(doc, "students").iter().filter_map(|r| id(r, "student")).collect();
        let unique: BTreeSet<&Uuid> = students.iter().collect();
        if unique.len() != students.len() {
            return Err(Error::field("students", "a student is in a group once"));
        }
        let (Some(program), Some(year)) = (id(doc, "program"), id(doc, "academic_year")) else { return Ok(()) };
        let enrolled: Vec<Uuid> =
            sqlx::query_scalar("select student from e_program_enrollment where tenant_id = $1 and program = $2 and academic_year = $3 and student = any($4)")
                .bind(hc.ctx.tenant)
                .bind(program)
                .bind(year)
                .bind(&students)
                .fetch_all(&mut **hc.tx)
                .await?;
        if let Some(outside) = students.iter().find(|s| !enrolled.contains(s)) {
            let name: Option<String> = sqlx::query_scalar("select student_name from e_student where tenant_id = $1 and id = $2")
                .bind(hc.ctx.tenant)
                .bind(outside)
                .fetch_optional(&mut **hc.tx)
                .await?;
            return Err(Error::field("students", format!("{} is not enrolled in the group's program and year", name.unwrap_or_default())));
        }
        Ok(())
    }
}

pub struct AttendanceHook;

#[async_trait]
impl DocHook for AttendanceHook {
    fn name(&self) -> &'static str {
        "education.attendance"
    }

    async fn validate(&self, hc: &mut HookCtx<'_>, doc: &Doc) -> Result<()> {
        let (Some(class), Some(student)) = (id(doc, "course_schedule"), id(doc, "student")) else { return Ok(()) };
        let in_group: bool = sqlx::query_scalar(
            "select exists(select 1 from e_course_schedule c join e_student_group__students g on g.tenant_id = c.tenant_id and g.parent_id = c.student_group \
             where c.tenant_id = $1 and c.id = $2 and g.student = $3)",
        )
        .bind(hc.ctx.tenant)
        .bind(class)
        .bind(student)
        .fetch_one(&mut **hc.tx)
        .await?;
        if !in_group {
            return Err(Error::field("student", "is not in this class's group"));
        }
        let me = id(doc, "id").unwrap_or_else(Uuid::nil);
        let taken: bool =
            sqlx::query_scalar("select exists(select 1 from e_student_attendance where tenant_id = $1 and course_schedule = $2 and student = $3 and id <> $4)")
                .bind(hc.ctx.tenant)
                .bind(class)
                .bind(student)
                .bind(me)
                .fetch_one(&mut **hc.tx)
                .await?;
        if taken {
            return Err(Error::field("student", "has their attendance for this class already"));
        }
        Ok(())
    }
}
