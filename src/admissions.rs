//! Admissions: an applicant is approved or rejected, and an approved one admitted. Admitting makes
//! the student, with the applicant's guardian, and their enrollment in the program and year, once.

use crate::obj;
use async_trait::async_trait;
use chrono::Utc;
use ferrum_domain::flow::created;
use ferrum_domain::util::{elevated, id, s};
use ferrum_kernel::{Doc, DocHook, Error, HookCtx, Result};
use serde_json::{Value, json};

pub struct ApplicantHook;

#[async_trait]
impl DocHook for ApplicantHook {
    fn name(&self) -> &'static str {
        "education.applicant"
    }

    async fn action(&self, hc: &mut HookCtx<'_>, name: &str, doc: &Doc, params: &Value) -> Result<Option<Value>> {
        let applicant = id(doc, "id").ok_or_else(|| Error::internal("an applicant without an id"))?;
        let system = elevated(hc.ctx);
        let set = |status: &str| obj(json!({"status": status}));
        match name {
            "approve" => {
                hc.kernel.set_values_tx(&system, hc.tx, "StudentApplicant", applicant, set("Approved")).await?;
                Ok(Some(json!({"status": "Approved"})))
            }
            "reject" => {
                let mut values = set("Rejected");
                if let Some(reason) = params.get("reason").and_then(Value::as_str).filter(|r| !r.trim().is_empty()) {
                    values.insert("notes".into(), json!(format!("{}\nRejected: {reason}", s(doc, "notes").unwrap_or_default()).trim().to_string()));
                }
                hc.kernel.set_values_tx(&system, hc.tx, "StudentApplicant", applicant, values).await?;
                Ok(Some(json!({"status": "Rejected"})))
            }
            "admit" => {
                let guardians: Vec<Value> = s(doc, "guardian_name")
                    .map(|g| vec![json!({"guardian_name": g, "relation": s(doc, "guardian_relation").unwrap_or("Guardian"), "email": s(doc, "guardian_email"), "phone": s(doc, "guardian_phone")})])
                    .unwrap_or_default();
                let student = hc
                    .kernel
                    .insert_tx(
                        &system,
                        hc.tx,
                        "Student",
                        obj(json!({"student_name": s(doc, "applicant_name"), "email": s(doc, "email"), "date_of_birth": s(doc, "date_of_birth"), "guardians": guardians})),
                    )
                    .await?;
                let sid = student.get("id").cloned().unwrap_or(Value::Null);
                hc.kernel
                    .insert_tx(
                        &system,
                        hc.tx,
                        "ProgramEnrollment",
                        obj(json!({"student": sid, "program": s(doc, "program"), "academic_year": s(doc, "academic_year"), "enrolled_on": Utc::now().date_naive().to_string()})),
                    )
                    .await?;
                hc.kernel.set_values_tx(&system, hc.tx, "StudentApplicant", applicant, obj(json!({"status": "Admitted", "student": sid}))).await?;
                Ok(Some(json!({"status": "Admitted", "student": created("Student", &student)})))
            }
            _ => Ok(None),
        }
    }
}
