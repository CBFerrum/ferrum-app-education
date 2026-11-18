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
mod records;
mod site;
mod timetable;

use chrono::NaiveDate;
use ferrum_domain::reports::ReportSource;
use ferrum_domain::util::s;
use ferrum_kernel::{Doc, HookRegistry, Kernel, Result};
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

/// Registers this app's document hooks, and the words for its timetable guard's refusals
/// (packages/education/sql/001_education.sql).
pub fn register(hooks: &mut HookRegistry) {
    ferrum_kernel::error::set_constraint_message("education_room_free", "the room has another class then");
    ferrum_kernel::error::set_constraint_message("education_instructor_free", "the instructor has another class then");
    ferrum_kernel::error::set_constraint_message("education_group_free", "the group has another class then");
    hooks
        .register("StudentApplicant", Arc::new(admissions::ApplicantHook))
        .register("Student", Arc::new(site::StudentHook))
        .register("FeeStructure", Arc::new(fees::StructureHook))
        .register("Fees", Arc::new(fees::FeesHook))
        .register("StudentGroup", Arc::new(timetable::GroupHook))
        .register("CourseSchedule", Arc::new(timetable::ScheduleHook))
        .register("StudentAttendance", Arc::new(timetable::AttendanceHook))
        .register("AssessmentResult", Arc::new(marks::ResultHook))
        .register("FeeSchedule", Arc::new(records::ScheduleHook))
        .register("ReportCard", Arc::new(records::ReportCardHook))
        .add_site("education", Arc::new(site::SchoolSite));
}

/// This app's reports: none of its own yet; its lists are its figures.
pub fn reports() -> Vec<Arc<dyn ReportSource>> {
    vec![]
}

/// One workspace's upkeep, which the platform runs about once a minute where Education is installed, on whichever
/// replica is free, and may run again: fees whose invoices were settled are paid.
pub async fn maintenance(k: &Kernel, tenant: Uuid) -> Result<()> {
    let schema = k.registry.schema(tenant).await?;
    fees::upkeep(k, tenant, schema).await
}

/// A date field.
pub(crate) fn date(doc: &Doc, k: &str) -> Option<NaiveDate> {
    s(doc, k).and_then(|v| NaiveDate::parse_from_str(v.get(..10).unwrap_or(v), "%Y-%m-%d").ok())
}

/// A document from JSON.
pub(crate) fn obj(v: Value) -> Doc {
    v.as_object().cloned().unwrap_or_default()
}
