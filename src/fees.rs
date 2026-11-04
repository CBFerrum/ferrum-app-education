//! Fees: a fee structure's total from its components; a student's fees take it as their amount and
//! are invoiced through ERP to the student's customer (made from their first guardian, or from them);
//! upkeep marks fees paid once their invoice is submitted and settled.

use crate::obj;
use async_trait::async_trait;
use chrono::NaiveDate;
use ferrum_domain::flow::{create_for_user, created};
use ferrum_domain::util::{d, elevated, id, money, rows, s, set, sys};
use ferrum_kernel::{Ctx, Doc, DocHook, Error, HookCtx, Kernel, Result};
use ferrum_meta::effective::Schema;
use rust_decimal::Decimal;
use serde_json::{Value, json};
use std::sync::Arc;
use uuid::Uuid;

pub struct StructureHook;

#[async_trait]
impl DocHook for StructureHook {
    fn name(&self) -> &'static str {
        "education.fee_structure"
    }

    fn fills(&self) -> &'static [&'static str] {
        &["total"]
    }

    async fn before_validate(&self, _hc: &mut HookCtx<'_>, doc: &mut Doc) -> Result<()> {
        let total: Decimal = rows(doc, "components").iter().map(|c| d(c, "amount")).sum();
        set(doc, "total", money(total));
        Ok(())
    }
}

pub struct FeesHook;

#[async_trait]
impl DocHook for FeesHook {
    fn name(&self) -> &'static str {
        "education.fees"
    }

    fn fills(&self) -> &'static [&'static str] {
        &["amount"]
    }

    async fn before_validate(&self, hc: &mut HookCtx<'_>, doc: &mut Doc) -> Result<()> {
        if s(doc, "status").is_some_and(|st| st != "Draft") {
            return Ok(());
        }
        if let Some(structure) = id(doc, "fee_structure") {
            let total: Option<Decimal> = sqlx::query_scalar("select total from e_fee_structure where tenant_id = $1 and id = $2")
                .bind(hc.ctx.tenant)
                .bind(structure)
                .fetch_optional(&mut **hc.tx)
                .await?
                .flatten();
            set(doc, "amount", money(total.ok_or_else(|| Error::field("fee_structure", "is not a fee structure here"))?));
        }
        Ok(())
    }

    async fn validate_change(&self, hc: &mut HookCtx<'_>, old: Option<&Doc>, doc: &Doc) -> Result<()> {
        if hc.ctx.is_system {
            return Ok(());
        }
        if let Some(o) = old
            && s(o, "status").is_some_and(|st| st != "Draft")
            && ["student", "fee_structure"].iter().any(|k| o.get(*k) != doc.get(*k))
        {
            return Err(Error::invalid("invoiced fees do not change"));
        }
        // The student is enrolled in the structure's program and year.
        if let (Some(student), Some(structure)) = (id(doc, "student"), id(doc, "fee_structure")) {
            let enrolled: bool = sqlx::query_scalar(
                "select exists(select 1 from e_program_enrollment e join e_fee_structure f on f.tenant_id = e.tenant_id and f.program = e.program and f.academic_year = e.academic_year \
                 where e.tenant_id = $1 and e.student = $2 and f.id = $3)",
            )
            .bind(hc.ctx.tenant)
            .bind(student)
            .bind(structure)
            .fetch_one(&mut **hc.tx)
            .await?;
            if !enrolled {
                return Err(Error::field("fee_structure", "is for a program and year the student is not enrolled in"));
            }
        }
        Ok(())
    }

    async fn action(&self, hc: &mut HookCtx<'_>, name: &str, doc: &Doc, _params: &Value) -> Result<Option<Value>> {
        if name != "make_invoice" {
            return Ok(None);
        }
        Ok(Some(invoice(hc, doc).await?))
    }
}

/// The student's customer: theirs, or one made from their first guardian (or them), and kept.
async fn customer_for(hc: &mut HookCtx<'_>, student: &Doc) -> Result<Uuid> {
    if let Some(c) = id(student, "customer") {
        return Ok(c);
    }
    let sid = id(student, "id").ok_or_else(|| Error::internal("a student without an id"))?;
    let guardian = rows(student, "guardians").into_iter().next();
    let (name, email) = match &guardian {
        Some(g) => (s(g, "guardian_name").map(str::to_string), s(g, "email").map(str::to_string)),
        None => (s(student, "student_name").map(str::to_string), s(student, "email").map(str::to_string)),
    };
    let customer = create_for_user(hc, "Customer", obj(json!({"customer_name": name, "customer_type": "Individual", "email": email}))).await?;
    let cid = id(&customer, "id").ok_or_else(|| Error::internal("a customer without an id"))?;
    hc.kernel.set_values_tx(&elevated(hc.ctx), hc.tx, "Student", sid, obj(json!({"customer": cid.to_string()}))).await?;
    Ok(cid)
}

/// `make_invoice`: the fees as a draft Sales Invoice, one line per component of the structure.
async fn invoice(hc: &mut HookCtx<'_>, doc: &Doc) -> Result<Value> {
    let tenant = hc.ctx.tenant;
    let fees = id(doc, "id").ok_or_else(|| Error::internal("fees without an id"))?;
    let system = sys(hc.ctx);
    let company: Option<Uuid> =
        sqlx::query_scalar("select company from e_education_settings where tenant_id = $1 limit 1").bind(tenant).fetch_optional(&mut **hc.tx).await?.flatten();
    let company = company.ok_or_else(|| Error::invalid("set the company in Education Settings first"))?;
    let student = hc.kernel.get_tx(&system, hc.tx, "Student", id(doc, "student").ok_or_else(|| Error::field("student", "is required"))?, false).await?;
    let structure =
        hc.kernel.get_tx(&system, hc.tx, "FeeStructure", id(doc, "fee_structure").ok_or_else(|| Error::field("fee_structure", "is required"))?, false).await?;
    let currency: Option<Uuid> = sqlx::query_scalar("select default_currency from e_company where tenant_id = $1 and id = $2")
        .bind(tenant)
        .bind(company)
        .fetch_optional(&mut **hc.tx)
        .await?
        .flatten();
    let items: Vec<Value> = rows(&structure, "components")
        .iter()
        .map(|c| {
            json!({"item": s(c, "item"), "qty": "1", "price_list_rate": money(d(c, "amount")), "discount_percent": "0",
                   "description": format!("{} for {}", s(c, "component").unwrap_or("Fees"), s(&student, "student_name").unwrap_or("a student"))})
        })
        .collect();
    if items.is_empty() {
        return Err(Error::invalid("the fee structure has no components to invoice"));
    }
    let customer = customer_for(hc, &student).await?;
    // Fees invoiced after their due date are due at once: an invoice is not due before it is posted.
    let due = crate::date(doc, "due_on").filter(|d| *d >= chrono::Utc::now().date_naive()).map(|d| d.to_string());
    let inv = create_for_user(
        hc,
        "SalesInvoice",
        obj(json!({"company": company.to_string(), "customer": customer.to_string(), "currency": currency.map(|c| c.to_string()), "items": items,
                   "due_date": due, "remarks": format!("Fees {}", s(doc, "number").unwrap_or_default())})),
    )
    .await?;
    hc.kernel
        .set_values_tx(
            &elevated(hc.ctx),
            hc.tx,
            "Fees",
            fees,
            obj(json!({"status": "Invoiced", "sales_invoice": inv.get("id").cloned().unwrap_or(Value::Null)})),
        )
        .await?;
    Ok(json!({"status": "Invoiced", "sales_invoice": created("SalesInvoice", &inv)}))
}

/// One workspace's upkeep: fees whose invoices are submitted and settled are paid.
pub(crate) async fn upkeep(k: &Kernel, tenant: Uuid, schema: Arc<Schema>, _today: NaiveDate) -> Result<()> {
    let ctx = Ctx::system(tenant, schema);
    let mut tx = k.db.tenant_tx(tenant).await?;
    let paid: Vec<Uuid> = sqlx::query_scalar(
        "select f.id from e_fees f join e_sales_invoice i on i.tenant_id = f.tenant_id and i.id = f.sales_invoice \
         where f.tenant_id = $1 and f.status = 'Invoiced' and i.docstatus = 1 and coalesce(i.outstanding_amount, 0) <= 0",
    )
    .bind(tenant)
    .fetch_all(&mut *tx)
    .await?;
    for fees in &paid {
        k.set_values_tx(&ctx, &mut tx, "Fees", *fees, obj(json!({"status": "Paid"}))).await?;
    }
    tx.commit().await?;
    Ok(())
}
