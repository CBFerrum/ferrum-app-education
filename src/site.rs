//! Education's endpoints for families. Without signing in, at `/api/site/{workspace}/education/…`:
//! the programs open to applications, applying (with a guardian's consent to the school keeping the
//! details), and an application's status from the link given (only its SHA-256 is kept). Signed in
//! to the workspace's portal, at `/api/app/education/portal/…`: a guardian's children (those whose
//! guardians list the address they signed in with): their enrollments, this year's attendance, fees,
//! published report cards and the consents the family gave, which they give or withdraw there.
//! Portal people hold no roles: everything is read for them after these checks, as the system.

use crate::obj;
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{NaiveDate, Utc};
use ferrum_domain::util::{id, rows, s, sys};
use ferrum_kernel::{Ctx, Error, FieldError, Kernel, Result, Site, SiteRequest, SiteResponse, Tx, with_retry};
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use std::time::Duration as StdDuration;
use uuid::Uuid;

const MAX_BODY: usize = 16 * 1024;
const WRITES_PER_WINDOW: u32 = 10;
const WINDOW: StdDuration = StdDuration::from_secs(600);
/// What a family may consent to on the portal.
pub(crate) const CONSENTS: [&str; 3] = ["Photos and Media", "School Trips", "Sharing With Learn"];

pub struct SchoolSite;

#[async_trait::async_trait]
impl Site for SchoolSite {
    fn max_body(&self) -> usize {
        MAX_BODY
    }

    async fn handle(&self, k: &Kernel, req: SiteRequest) -> Result<SiteResponse> {
        let segs: Vec<&str> = req.path.split('/').filter(|p| !p.is_empty()).collect();
        let (status, out) = match (req.method.as_str(), segs.as_slice()) {
            ("GET", ["apply", "programs"]) => (200, programs(k, &public(k, &req).await?).await?),
            ("POST", ["apply"]) => {
                k.rate_limit(
                    &format!("education|{}|{}", req.tenant, req.client_ip),
                    WINDOW,
                    WRITES_PER_WINDOW,
                    "too many applications from this address; try again later",
                )
                .await?;
                apply(k, &public(k, &req).await?, &body(&req)?).await?
            }
            ("GET", ["apply", "status", tok]) => (200, status_of(k, &public(k, &req).await?, tok).await?),
            ("GET", ["portal", "family"]) => (200, family(k, signed_in(&req)?).await?),
            ("POST", ["portal", "consents"]) => (200, consent(k, signed_in(&req)?, &body(&req)?).await?),
            _ => return Err(Error::NotFound("page".into())),
        };
        let mut r = SiteResponse::json(status, &out);
        r.headers.push(("cache-control".into(), "no-store".into()));
        Ok(r)
    }
}

/// The system, for a request from anyone.
async fn public(k: &Kernel, req: &SiteRequest) -> Result<Ctx> {
    let schema = match &req.user {
        Some(u) => u.schema.clone(),
        None => k.registry.schema(req.tenant).await?,
    };
    let mut ctx = Ctx::system(req.tenant, schema);
    ctx.request_id = format!("education-{}", Uuid::now_v7());
    Ok(ctx)
}

fn signed_in(req: &SiteRequest) -> Result<&Ctx> {
    req.user.as_ref().ok_or_else(|| Error::Unauthorized("sign in first".into()))
}

fn body(req: &SiteRequest) -> Result<Map<String, Value>> {
    match serde_json::from_slice::<Value>(&req.body) {
        Ok(Value::Object(m)) => Ok(m),
        _ => Err(Error::invalid("the request body is a JSON object")),
    }
}

/// A piece of text a family typed, trimmed, at most `max` characters.
fn text(body: &Map<String, Value>, key: &str, max: usize, errors: &mut Vec<FieldError>) -> Option<String> {
    let v = body.get(key).and_then(Value::as_str).map(str::trim).unwrap_or_default();
    if v.chars().count() > max {
        errors.push(FieldError::new(key, format!("is at most {max} characters")));
    }
    (!v.is_empty()).then(|| v.to_string())
}

fn hash(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}

/// The programs taking applications, each with the years not yet over.
async fn programs(k: &Kernel, ctx: &Ctx) -> Result<Value> {
    let mut tx = k.db.tenant_tx(ctx.tenant).await?;
    let open = open_programs(&mut tx, ctx).await?;
    tx.commit().await?;
    if open.is_empty() {
        return Err(Error::NotFound("page".into()));
    }
    let years = years(&mut k.db.tenant_tx(ctx.tenant).await?, ctx).await?;
    Ok(json!({
        "programs": open.into_iter().map(|(code, name)| json!({"code": code, "name": name})).collect::<Vec<_>>(),
        "years": years.into_iter().map(|(name, from, to)| json!({"name": name, "starts_on": from, "ends_on": to})).collect::<Vec<_>>(),
    }))
}

async fn open_programs(tx: &mut Tx, ctx: &Ctx) -> Result<Vec<(String, String)>> {
    Ok(sqlx::query_as("select code, program_name from e_program where tenant_id = $1 and coalesce(admissions_open, false) order by program_name")
        .bind(ctx.tenant)
        .fetch_all(&mut **tx)
        .await?)
}

/// The academic years not yet over, soonest first.
async fn years(tx: &mut Tx, ctx: &Ctx) -> Result<Vec<(String, NaiveDate, NaiveDate)>> {
    Ok(sqlx::query_as("select year_name, starts_on, ends_on from e_academic_year where tenant_id = $1 and ends_on >= $2 order by starts_on")
        .bind(ctx.tenant)
        .bind(ctx.today())
        .fetch_all(&mut **tx)
        .await?)
}

/// `POST /apply`: an application to a program open for a year not yet over, with a guardian to
/// reach and their consent to the school keeping the details. The family is given its link.
async fn apply(k: &Kernel, ctx: &Ctx, body: &Map<String, Value>) -> Result<(u16, Value)> {
    // A form field people never see: whatever fills it is a program, answered like a person.
    if body.get("website").is_some_and(|v| !v.is_null() && v.as_str() != Some("")) {
        return Ok((201, json!({"status": "received"})));
    }
    let mut errors = Vec::new();
    let name = text(body, "applicant_name", 200, &mut errors);
    let email = text(body, "email", 254, &mut errors);
    let born = text(body, "date_of_birth", 10, &mut errors);
    let guardian = text(body, "guardian_name", 200, &mut errors);
    let guardian_email = text(body, "guardian_email", 254, &mut errors);
    let guardian_phone = text(body, "guardian_phone", 40, &mut errors);
    let relation = text(body, "guardian_relation", 20, &mut errors).unwrap_or_else(|| "Guardian".into());
    let program = text(body, "program", 20, &mut errors);
    let year = text(body, "academic_year", 140, &mut errors);
    if name.is_none() {
        errors.push(FieldError::new("applicant_name", "is required"));
    }
    if guardian.is_none() {
        errors.push(FieldError::new("guardian_name", "is required"));
    }
    if !guardian_email.as_deref().is_some_and(|e| e.contains('@') && !e.contains(char::is_whitespace)) {
        errors.push(FieldError::new("guardian_email", "is an email address, where the school writes about the application"));
    }
    if born.as_deref().is_some_and(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").is_err()) {
        errors.push(FieldError::new("date_of_birth", "is a date, YYYY-MM-DD"));
    }
    if !["Mother", "Father", "Guardian", "Other"].contains(&relation.as_str()) {
        errors.push(FieldError::new("guardian_relation", "is Mother, Father, Guardian or Other"));
    }
    if body.get("consent").and_then(Value::as_bool) != Some(true) {
        errors.push(FieldError::new("consent", "the guardian's consent to the school keeping these details is needed to apply"));
    }
    if !errors.is_empty() {
        return Err(Error::Validation(errors));
    }
    with_retry(|| async {
        let mut tx = k.db.tenant_tx(ctx.tenant).await?;
        let found: Option<Uuid> = sqlx::query_scalar("select id from e_program where tenant_id = $1 and code = $2 and coalesce(admissions_open, false)")
            .bind(ctx.tenant)
            .bind(&program)
            .fetch_optional(&mut *tx)
            .await?;
        let program = found.ok_or_else(|| Error::Validation(vec![FieldError::new("program", "is not open to applications")]))?;
        let found: Option<Uuid> = sqlx::query_scalar("select id from e_academic_year where tenant_id = $1 and year_name = $2 and ends_on >= $3")
            .bind(ctx.tenant)
            .bind(&year)
            .bind(ctx.today())
            .fetch_optional(&mut *tx)
            .await?;
        let year = found.ok_or_else(|| Error::Validation(vec![FieldError::new("academic_year", "is not a year still to come")]))?;
        let token = URL_SAFE_NO_PAD.encode(rand::random::<[u8; 32]>());
        let input = json!({"applicant_name": name, "email": email, "date_of_birth": born, "program": program.to_string(), "academic_year": year.to_string(),
                           "guardian_name": guardian, "guardian_email": guardian_email, "guardian_phone": guardian_phone, "guardian_relation": relation});
        let applied = k.insert_tx(ctx, &mut tx, "StudentApplicant", obj(input)).await?;
        let aid = id(&applied, "id").ok_or_else(|| Error::internal("an applicant without an id"))?;
        let values = json!({"source": "Online", "consent_given": true, "consent_on": Utc::now().to_rfc3339(), "token_hash": hash(&token)});
        k.set_values_tx(ctx, &mut tx, "StudentApplicant", aid, obj(values)).await?;
        let view = application(k, ctx, &mut tx, aid).await?;
        tx.commit().await?;
        Ok((201, json!({"application": view, "token": token})))
    })
    .await
}

/// What a family sees of its application.
async fn application(k: &Kernel, ctx: &Ctx, tx: &mut Tx, aid: Uuid) -> Result<Value> {
    let a = k.get_tx(ctx, tx, "StudentApplicant", aid, false).await?;
    let names: Option<(Option<String>, Option<String>)> = sqlx::query_as(
        "select p.program_name, y.year_name from e_program p, e_academic_year y where p.tenant_id = $1 and p.id = $2 and y.tenant_id = $1 and y.id = $3",
    )
    .bind(ctx.tenant)
    .bind(id(&a, "program"))
    .bind(id(&a, "academic_year"))
    .fetch_optional(&mut **tx)
    .await?;
    let (program, year) = names.unwrap_or_default();
    Ok(json!({"number": s(&a, "number"), "applicant_name": s(&a, "applicant_name"), "program": program, "academic_year": year, "status": s(&a, "status")}))
}

/// `GET /apply/status/{token}`: the application the link was given for.
async fn status_of(k: &Kernel, ctx: &Ctx, tok: &str) -> Result<Value> {
    let shaped = tok.len() == 43 && tok.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_');
    let mut tx = k.db.tenant_tx(ctx.tenant).await?;
    let found: Option<Uuid> = if shaped {
        sqlx::query_scalar("select id from e_student_applicant where tenant_id = $1 and token_hash = $2")
            .bind(ctx.tenant)
            .bind(hash(tok))
            .fetch_optional(&mut *tx)
            .await?
    } else {
        None
    };
    let aid = found.ok_or_else(|| Error::NotFound("application".into()))?;
    let view = application(k, ctx, &mut tx, aid).await?;
    tx.commit().await?;
    Ok(json!({"application": view}))
}

/// The students whose guardians list `email`.
async fn children(tx: &mut Tx, tenant: Uuid, email: &str) -> Result<Vec<Uuid>> {
    Ok(sqlx::query_scalar(
        "select distinct g.parent_id from e_student__guardians g join e_student s on s.tenant_id = g.tenant_id and s.id = g.parent_id \
         where g.tenant_id = $1 and lower(g.email) = $2 order by g.parent_id limit 20",
    )
    .bind(tenant)
    .bind(email)
    .fetch_all(&mut **tx)
    .await?)
}

/// `GET /portal/family`: each of a guardian's children.
async fn family(k: &Kernel, user: &Ctx) -> Result<Value> {
    let email = user.email.trim().to_lowercase();
    if email.is_empty() {
        return Ok(json!({"children": []}));
    }
    let sctx = sys(user);
    let tenant = user.tenant;
    let mut tx = k.db.tenant_tx(tenant).await?;
    let mut out = Vec::new();
    for child in children(&mut tx, tenant, &email).await? {
        let student = k.get_tx(&sctx, &mut tx, "Student", child, false).await?;
        let enrollments: Vec<(String, String, NaiveDate, NaiveDate)> = sqlx::query_as(
            "select p.program_name, y.year_name, y.starts_on, y.ends_on from e_program_enrollment e join e_program p on p.tenant_id = e.tenant_id and p.id = e.program \
             join e_academic_year y on y.tenant_id = e.tenant_id and y.id = e.academic_year where e.tenant_id = $1 and e.student = $2 order by y.starts_on desc",
        )
        .bind(tenant)
        .bind(child)
        .fetch_all(&mut *tx)
        .await?;
        // This year's attendance: the latest year they are enrolled in.
        let attendance = match enrollments.first() {
            Some((_, _, from, to)) => {
                let (present, absent, late, excused): (i64, i64, i64, i64) = sqlx::query_as(
                    "select count(*) filter (where a.status = 'Present'), count(*) filter (where a.status = 'Absent'), \
                            count(*) filter (where a.status = 'Late'), count(*) filter (where a.status = 'Excused') \
                     from e_student_attendance a join e_course_schedule c on c.tenant_id = a.tenant_id and c.id = a.course_schedule \
                     where a.tenant_id = $1 and a.student = $2 and c.date between $3 and $4",
                )
                .bind(tenant)
                .bind(child)
                .bind(from)
                .bind(to)
                .fetch_one(&mut *tx)
                .await?;
                json!({"present": present, "absent": absent, "late": late, "excused": excused})
            }
            None => Value::Null,
        };
        let fees: Vec<(Option<String>, String, NaiveDate, Option<rust_decimal::Decimal>, String, Option<Uuid>, Option<i64>)> = sqlx::query_as(
            "select f.number, st.structure_name, f.due_on, f.amount, f.status, f.sales_invoice, i.docstatus::bigint from e_fees f \
             join e_fee_structure st on st.tenant_id = f.tenant_id and st.id = f.fee_structure \
             left join e_sales_invoice i on i.tenant_id = f.tenant_id and i.id = f.sales_invoice \
             where f.tenant_id = $1 and f.student = $2 and f.status <> 'Cancelled' order by f.due_on desc limit 50",
        )
        .bind(tenant)
        .bind(child)
        .fetch_all(&mut *tx)
        .await?;
        let cards: Vec<Value> = k
            .find_tx(
                &sctx,
                &mut tx,
                "ReportCard",
                &[ferrum_kernel::Filter::eq("student", child.to_string()), ferrum_kernel::Filter::eq("status", "Published")],
                Some("created_at desc"),
                20,
            )
            .await?
            .into_iter()
            .map(|c| json!({"id": s(&c, "id"), "number": s(&c, "number")}))
            .collect();
        let mut published = Vec::new();
        for c in cards {
            let rc: Uuid = c["id"].as_str().and_then(|v| v.parse().ok()).ok_or_else(|| Error::internal("a report card without an id"))?;
            let card = k.get_tx(&sctx, &mut tx, "ReportCard", rc, false).await?;
            let year: Option<String> = sqlx::query_scalar("select year_name from e_academic_year where tenant_id = $1 and id = $2")
                .bind(tenant)
                .bind(id(&card, "academic_year"))
                .fetch_optional(&mut *tx)
                .await?;
            let mut courses = Vec::new();
            for line in rows(&card, "courses") {
                let name: Option<String> = sqlx::query_scalar("select course_name from e_course where tenant_id = $1 and id = $2")
                    .bind(tenant)
                    .bind(id(&line, "course"))
                    .fetch_optional(&mut *tx)
                    .await?;
                courses.push(json!({"course": name, "assessments": line.get("assessments"), "percent": s(&line, "percent"), "grade": s(&line, "grade")}));
            }
            published
                .push(json!({"number": s(&card, "number"), "academic_year": year, "courses": courses, "attendance_percent": s(&card, "attendance_percent"),
                                  "comment": s(&card, "comment"), "published_on": s(&card, "published_on")}));
        }
        let given = rows(&student, "consents");
        let consents: Vec<Value> = CONSENTS
            .iter()
            .map(|c| {
                let row = given.iter().find(|r| s(r, "consent") == Some(*c));
                json!({"consent": c, "given": row.and_then(|r| r.get("given")).and_then(Value::as_bool).unwrap_or(false),
                       "by": row.and_then(|r| s(r, "by")), "given_on": row.and_then(|r| s(r, "given_on"))})
            })
            .collect();
        out.push(json!({
            "id": child.to_string(), "number": s(&student, "number"), "name": s(&student, "student_name"), "status": s(&student, "status"),
            "enrollments": enrollments.iter().map(|(p, y, _, _)| json!({"program": p, "academic_year": y})).collect::<Vec<_>>(),
            "attendance": attendance,
            "fees": fees.into_iter().map(|(n, what, due, amount, status, inv, doc)| json!({"number": n, "structure": what, "due_on": due,
                "amount": amount.map(|a| a.to_string()), "status": status, "invoice": if doc == Some(1) { inv.map(|i| i.to_string()) } else { None }})).collect::<Vec<_>>(),
            "report_cards": published,
            "consents": consents,
        }));
    }
    tx.commit().await?;
    Ok(json!({"children": out}))
}

/// `POST /portal/consents {student, consent, given}`: a guardian gives or withdraws one of the
/// family's consents for their child; who and when are kept.
async fn consent(k: &Kernel, user: &Ctx, body: &Map<String, Value>) -> Result<Value> {
    let email = user.email.trim().to_lowercase();
    let child: Uuid = body.get("student").and_then(Value::as_str).and_then(|v| v.parse().ok()).ok_or_else(|| Error::field("student", "is required"))?;
    let what = body
        .get("consent")
        .and_then(Value::as_str)
        .filter(|c| CONSENTS.contains(c))
        .ok_or_else(|| Error::field("consent", "is not one the school asks for"))?;
    let given = body.get("given").and_then(Value::as_bool).ok_or_else(|| Error::field("given", "is true or false"))?;
    let sctx = sys(user);
    with_retry(|| async {
        let mut tx = k.db.tenant_tx(user.tenant).await?;
        // Only a guardian of the child: anyone else is told it is not there.
        if email.is_empty() || !children(&mut tx, user.tenant, &email).await?.contains(&child) {
            return Err(Error::NotFound("student".into()));
        }
        let done = record_consent(k, &sctx, &mut tx, child, what, given, &email).await?;
        tx.commit().await?;
        Ok(done)
    })
    .await
}

/// Keeps a family's consent to `what` on student `child`, given or withdrawn by `by`, in place of
/// what it said before.
async fn record_consent(k: &Kernel, sctx: &Ctx, tx: &mut Tx, child: Uuid, what: &str, given: bool, by: &str) -> Result<Value> {
    let student = k.get_tx(sctx, tx, "Student", child, true).await?;
    let mut kept: Vec<ferrum_kernel::Doc> = rows(&student, "consents")
        .into_iter()
        .filter(|r| s(r, "consent") != Some(what))
        .map(|mut r| {
            r.retain(|k, _| ["id", "consent", "given", "by", "given_on"].contains(&k.as_str()));
            r
        })
        .collect();
    kept.push(obj(json!({"consent": what, "given": given, "by": by, "given_on": Utc::now().to_rfc3339()})));
    k.set_child_rows_tx(sctx, tx, "Student", "consents", child, kept).await?;
    Ok(json!({"consent": what, "given": given, "message": if given { format!("{what}: consent given") } else { format!("{what}: consent withdrawn") }}))
}

/// A student's `set_consent` action: the family's consent given or withdrawn, as on the portal's
/// page, by whoever runs it (a guardian through a portal role naming it, Ferrum
/// docs/builder/decisions.md BD-02, or the school's office), recorded with their address.
pub struct StudentHook;

#[async_trait::async_trait]
impl ferrum_kernel::DocHook for StudentHook {
    fn name(&self) -> &'static str {
        "education.student"
    }

    async fn action(&self, hc: &mut ferrum_kernel::HookCtx<'_>, name: &str, doc: &ferrum_kernel::Doc, params: &Value) -> Result<Option<Value>> {
        if name != "set_consent" {
            return Ok(None);
        }
        let what = params
            .get("consent")
            .and_then(Value::as_str)
            .filter(|c| CONSENTS.contains(c))
            .ok_or_else(|| Error::field("consent", "is not one the school asks for"))?;
        let given = match params.get("given").and_then(Value::as_str) {
            Some("Given") => true,
            Some("Withdrawn") => false,
            _ => return Err(Error::field("given", "is Given or Withdrawn")),
        };
        let child = id(doc, "id").ok_or_else(|| Error::internal("a student without an id"))?;
        let by = hc.ctx.email.trim().to_lowercase();
        Ok(Some(record_consent(hc.kernel, &sys(hc.ctx), hc.tx, child, what, given, &by).await?))
    }
}
