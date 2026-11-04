//! Results: a score within its assessment's maximum, as a percentage, graded on the assessment's
//! scale (the band with the highest threshold the percentage reaches, as ERPNext 13's Grading Scale
//! did), once per student, for students of the assessment's group.

use async_trait::async_trait;
use ferrum_domain::util::{d, id, money, round, set};
use ferrum_kernel::{Doc, DocHook, Error, HookCtx, Result};
use rust_decimal::Decimal;
use serde_json::json;
use uuid::Uuid;

/// The grade of a percentage on a scale of (grade, lowest percentage that earns it).
pub(crate) fn grade_of(percent: Decimal, bands: &[(String, Decimal)]) -> Option<String> {
    bands.iter().filter(|(_, from)| percent >= *from).max_by_key(|(_, from)| *from).map(|(g, _)| g.clone())
}

pub struct ResultHook;

#[async_trait]
impl DocHook for ResultHook {
    fn name(&self) -> &'static str {
        "education.result"
    }

    fn fills(&self) -> &'static [&'static str] {
        &["percent", "grade"]
    }

    async fn before_validate(&self, hc: &mut HookCtx<'_>, doc: &mut Doc) -> Result<()> {
        let Some(plan) = id(doc, "assessment") else { return Ok(()) };
        let row: Option<(Decimal, Option<Uuid>)> =
            sqlx::query_as("select maximum_score, grading_scale from e_assessment_plan where tenant_id = $1 and id = $2")
                .bind(hc.ctx.tenant)
                .bind(plan)
                .fetch_optional(&mut **hc.tx)
                .await?;
        let (max, scale) = row.ok_or_else(|| Error::field("assessment", "is not an assessment here"))?;
        let score = d(doc, "score");
        if score < Decimal::ZERO || score > max {
            return Err(Error::field("score", format!("is between 0 and the assessment's maximum, {max}")));
        }
        let percent = if max > Decimal::ZERO { round(score * Decimal::from(100) / max, 2) } else { Decimal::ZERO };
        let bands: Vec<(String, Decimal)> = sqlx::query_as("select grade, from_percent from e_grading_scale__grades where tenant_id = $1 and parent_id = $2")
            .bind(hc.ctx.tenant)
            .bind(scale)
            .fetch_all(&mut **hc.tx)
            .await?;
        set(doc, "percent", money(percent));
        set(doc, "grade", json!(grade_of(percent, &bands)));
        Ok(())
    }

    async fn validate(&self, hc: &mut HookCtx<'_>, doc: &Doc) -> Result<()> {
        let (Some(plan), Some(student)) = (id(doc, "assessment"), id(doc, "student")) else { return Ok(()) };
        let in_group: bool = sqlx::query_scalar(
            "select exists(select 1 from e_assessment_plan p join e_student_group__students g on g.tenant_id = p.tenant_id and g.parent_id = p.student_group \
             where p.tenant_id = $1 and p.id = $2 and g.student = $3)",
        )
        .bind(hc.ctx.tenant)
        .bind(plan)
        .bind(student)
        .fetch_one(&mut **hc.tx)
        .await?;
        if !in_group {
            return Err(Error::field("student", "is not in the assessment's group"));
        }
        let me = id(doc, "id").unwrap_or_else(Uuid::nil);
        let again: bool =
            sqlx::query_scalar("select exists(select 1 from e_assessment_result where tenant_id = $1 and assessment = $2 and student = $3 and id <> $4)")
                .bind(hc.ctx.tenant)
                .bind(plan)
                .bind(student)
                .bind(me)
                .fetch_one(&mut **hc.tx)
                .await?;
        if again {
            return Err(Error::field("student", "has a result for this assessment already"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::grade_of;
    use rust_decimal::Decimal;

    #[test]
    fn a_grade_is_the_highest_band_reached() {
        let bands: Vec<(String, Decimal)> = [("A", 85), ("B", 70), ("C", 50), ("F", 0)].iter().map(|(g, p)| (g.to_string(), Decimal::from(*p))).collect();
        assert_eq!(grade_of(Decimal::from(92), &bands).as_deref(), Some("A"));
        assert_eq!(grade_of(Decimal::from(85), &bands).as_deref(), Some("A"), "the threshold itself earns it");
        assert_eq!(grade_of(Decimal::new(8499, 2), &bands).as_deref(), Some("B"));
        assert_eq!(grade_of(Decimal::ZERO, &bands).as_deref(), Some("F"));
        assert_eq!(grade_of(Decimal::from(40), &[]), None);
    }
}
