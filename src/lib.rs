//! Education (FERRUM-INDUSTRY-004): a school's office — admissions, students and guardians,
//! programs and courses, fees invoiced through ERP, a timetable without clashes, assessments graded
//! on a scale, and attendance.
//!
//! * `admissions`: an applicant approved, rejected or admitted; admitting makes the student, with
//!   their guardian, and their enrollment.
//! * `fees`: a fee structure's total, a student's fees from it, invoiced to their guardian's or their
//!   own customer; upkeep marks fees paid once their invoice is settled.
//! * `timetable`: no room, instructor or student group in two classes at once; groups hold students
//!   enrolled in their program and year; attendance only for a class's own students, once each.
//! * `marks`: a result's percentage and grade from its assessment's scale, once per student.
//!
//! Adapted from ERPNext 13's Education module (GPL-3.0; docs/PROVENANCE.md). Hooks are registered per
//! entity, so a workspace without the `education` package never runs them.

mod admissions;
mod fees;
mod marks;
mod timetable;

use chrono::{NaiveDate, Utc};
use ferrum_domain::reports::ReportSource;
use ferrum_domain::util::s;
use ferrum_kernel::{Doc, Error, HookRegistry, Kernel, Result};
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;

include!(concat!(env!("OUT_DIR"), "/packages.rs"));
include!(concat!(env!("OUT_DIR"), "/ui.rs"));

/// This app's manifest (`app.yaml`).
pub const MANIFEST: &str = include_str!("../app.yaml");

/// This app's metadata packages, embedded at build time.
pub fn packages() -> std::result::Result<Vec<ferrum_meta::Package>, ferrum_meta::MetaError> {
    ferrum_meta::catalog::load_embedded(EMBEDDED)
}

/// Registers this app's document hooks.
pub fn register(hooks: &mut HookRegistry) {
    hooks
        .register("StudentApplicant", Arc::new(admissions::ApplicantHook))
        .register("FeeStructure", Arc::new(fees::StructureHook))
        .register("Fees", Arc::new(fees::FeesHook))
        .register("StudentGroup", Arc::new(timetable::GroupHook))
        .register("CourseSchedule", Arc::new(timetable::ScheduleHook))
        .register("StudentAttendance", Arc::new(timetable::AttendanceHook))
        .register("AssessmentResult", Arc::new(marks::ResultHook));
}

/// This app's reports: none of its own yet; its lists are its figures.
pub fn reports() -> Vec<Arc<dyn ReportSource>> {
    vec![]
}

/// Upkeep: fees whose invoices were settled are paid.
pub async fn maintenance(k: &Kernel) -> Result<()> {
    maintenance_on(k, Utc::now().date_naive()).await
}

/// [`maintenance`] as of `today`.
pub async fn maintenance_on(k: &Kernel, today: NaiveDate) -> Result<()> {
    let mut tx = k.db.system_tx().await?;
    let tenants: Vec<Uuid> = sqlx::query_scalar("select id from tenant where status = 'active' order by id").fetch_all(&mut *tx).await?;
    tx.commit().await?;
    let mut first_error: Option<Error> = None;
    for tenant in tenants {
        match k.registry.schema(tenant).await {
            Ok(s) if s.packages.iter().any(|p| p == "education") => {
                if let Err(e) = fees::upkeep(k, tenant, s, today).await {
                    first_error.get_or_insert(e);
                }
            }
            Ok(_) => {}
            Err(e) => {
                first_error.get_or_insert(e);
            }
        }
    }
    first_error.map_or(Ok(()), Err)
}

/// A date field.
pub(crate) fn date(doc: &Doc, k: &str) -> Option<NaiveDate> {
    s(doc, k).and_then(|v| NaiveDate::parse_from_str(v.get(..10).unwrap_or(v), "%Y-%m-%d").ok())
}

/// A document from JSON.
pub(crate) fn obj(v: Value) -> Doc {
    v.as_object().cloned().unwrap_or_default()
}
