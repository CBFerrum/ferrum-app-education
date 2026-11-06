//! A school term (FERRUM-INDUSTRY-004) against a real PostgreSQL
//! (`FERRUM_EDUCATION_TEST_DATABASE_URL`, default `postgres://ferrum:ferrum@localhost:5432/ferrum_education_test`;
//! reset first): an applicant approved and admitted into a student with their guardian and enrollment;
//! fees from a structure invoiced to the guardian and paid through ERP; groups of enrolled students; a
//! timetable that refuses a room, an instructor or a group in two classes at once; attendance and a
//! graded assessment; and who may do what.

use chrono::NaiveDate;
use ferrum_kernel::secret::SecretKey;
use ferrum_kernel::tenant::NewTenant;
use ferrum_kernel::{Ctx, Db, DbConfig, Doc, Error, HookRegistry, Kernel, KernelSettings, ListQuery};
use ferrum_meta::Catalog;
use serde_json::{Value, json};
use std::sync::Arc;
use uuid::Uuid;

fn obj(v: Value) -> Doc {
    v.as_object().cloned().unwrap()
}

fn uid(v: &Value) -> Uuid {
    v.as_str().unwrap_or_else(|| panic!("not an id: {v}")).parse().unwrap()
}

fn dec(v: &Value) -> f64 {
    v.as_str().map(|s| s.parse().unwrap()).or_else(|| v.as_f64()).unwrap_or_else(|| panic!("not a number: {v}"))
}

struct T {
    k: Arc<Kernel>,
    tenant: Uuid,
    admin: Ctx,
}

impl T {
    async fn insert(&self, ctx: &Ctx, entity: &str, v: Value) -> Doc {
        self.k.insert(ctx, entity, obj(v)).await.unwrap_or_else(|e| panic!("insert {entity}: {e:?}"))
    }
    async fn get(&self, entity: &str, id: Uuid) -> Doc {
        self.k.get(&self.admin, entity, id, false).await.unwrap_or_else(|e| panic!("get {entity}: {e:?}"))
    }
    async fn find_one(&self, entity: &str, filters: Value) -> Doc {
        let q = ListQuery { filters: ferrum_kernel::query::parse_filters(&filters).unwrap(), limit: Some(10), ..Default::default() };
        let rows = self.k.list(&self.admin, entity, &q).await.unwrap().rows;
        assert_eq!(rows.len(), 1, "{entity} {filters}");
        rows[0].clone()
    }
    async fn user(&self, email: &str, roles: &[&str]) -> Ctx {
        let roles: Vec<String> = roles.iter().map(|r| r.to_string()).collect();
        let id = self.k.create_user(&self.admin, email, email, Some("Correct-Horse-9"), &roles, false).await.expect("user");
        let s = self.k.login("school", email, "Correct-Horse-9").await.expect("login");
        self.k.load_ctx(self.tenant, id, Some(s.token_version), "it".into()).await.expect("ctx")
    }
}

fn refused<V: std::fmt::Debug>(r: Result<V, Error>, kind: &str, says: &str) {
    let msg = format!("{r:?}");
    let ok = matches!(
        (&r, kind),
        (Err(Error::Invalid(_) | Error::Validation(_)), "invalid") | (Err(Error::Forbidden(_)), "forbidden") | (Err(Error::NotFound(_)), "not found")
    );
    assert!(ok && msg.contains(says), "expected {kind} saying {says:?}, got {msg}");
}

#[test]
fn a_school_term() {
    let rt = tokio::runtime::Builder::new_multi_thread().enable_all().thread_stack_size(16 << 20).build().unwrap();
    rt.block_on(async { tokio::spawn(scenario()).await.unwrap() });
}

async fn scenario() {
    let url = std::env::var("FERRUM_EDUCATION_TEST_DATABASE_URL").unwrap_or_else(|_| "postgres://ferrum:ferrum@localhost:5432/ferrum_education_test".into());
    let db = Db::connect(DbConfig { url, max_connections: 16, ..Default::default() }).await.expect("connect");
    sqlx::raw_sql("drop schema if exists public cascade; create schema public;").execute(&db.pool).await.expect("reset");
    let mut packages = ferrum_domain::packages().expect("platform packages");
    packages.extend(ferrum_app_erp::packages().expect("ERP's packages"));
    packages.extend(ferrum_app_education::packages().expect("app packages"));
    let catalog = Catalog::from_packages(packages).expect("catalog");
    let mut hooks = HookRegistry::default();
    ferrum_domain::register(&mut hooks);
    ferrum_app_erp::register(&mut hooks);
    ferrum_app_education::register(&mut hooks);
    let settings = KernelSettings { secret_key: Some(SecretKey::new([7u8; 32])), ..Default::default() };
    let k = Kernel::boot(db, catalog, hooks, settings, None).await.expect("boot");
    ferrum_domain::maintenance(&k).await.expect("partitions");
    ferrum_app_erp::maintenance(&k).await.expect("partitions");
    let tenant = k
        .create_tenant(NewTenant {
            slug: "school".into(),
            name: "Hill School".into(),
            admin_email: "admin@school.test".into(),
            admin_password: "Correct-Horse-9".into(),
            admin_name: None,
            packages: vec!["education".into()],
            settings: Some(json!({"fiscal_year_start_month": 4, "timezone": "Asia/Kolkata"})),
        })
        .await
        .expect("tenant");
    let session = k.login("school", "admin@school.test", "Correct-Horse-9").await.expect("login");
    let admin = k.load_ctx(tenant, session.user, Some(session.token_version), "it".into()).await.expect("ctx");
    let t = T { k: k.clone(), tenant, admin };
    let a = t.admin.clone();

    // ───────────── the books, the school ─────────────
    let inr = t.find_one("Currency", json!({"code": "INR"})).await;
    let company = uid(&t.insert(&a, "Company", json!({"company_name": "Hill School", "abbr": "HS", "default_currency": inr["id"]})).await["id"]);
    let services = t.find_one("ItemGroup", json!({"group_name": "Services"})).await;
    let nos = t.find_one("Uom", json!({"uom_name": "Nos"})).await;
    let item = |code: &str| json!({"item_code": code, "item_name": code, "item_group": services["id"], "stock_uom": nos["id"], "is_stock_item": false});
    let tuition = uid(&t.insert(&a, "Item", item("TUITION")).await["id"]);
    let library = uid(&t.insert(&a, "Item", item("LIBRARY")).await["id"]);
    let manager = t.user("office@school.test", &["Education Manager"]).await;
    let teacher = t.user("ira@school.test", &["Teacher"]).await;
    let officer = t.user("admit@school.test", &["Admissions Officer"]).await;
    t.insert(&manager, "EducationSettings", json!({"company": company.to_string()})).await;
    let year = uid(&t.insert(&manager, "AcademicYear", json!({"year_name": "2026-27", "starts_on": "2026-06-01", "ends_on": "2027-03-31"})).await["id"]);
    let math = uid(&t.insert(&manager, "Course", json!({"course_name": "Mathematics", "code": "MATH"})).await["id"]);
    let science = uid(&t.insert(&manager, "Course", json!({"course_name": "Science", "code": "SCI"})).await["id"]);
    let grade5 = uid(&t
        .insert(
            &manager,
            "Program",
            json!({"program_name": "Grade 5", "code": "G5", "courses": [{"course": math.to_string()}, {"course": science.to_string()}]}),
        )
        .await["id"]);

    // ───────────── admissions ─────────────
    let apply = |name: &str, email: &str, guardian: Option<&str>| json!({"applicant_name": name, "email": email, "program": grade5.to_string(), "academic_year": year.to_string(), "guardian_name": guardian, "guardian_email": guardian.map(|_| "mira@family.test")});
    let ana_app = uid(&t.insert(&officer, "StudentApplicant", apply("Ana Rao", "ana@school.test", Some("Mira Rao"))).await["id"]);
    let bo_app = uid(&t.insert(&officer, "StudentApplicant", apply("Bo Chen", "bo@school.test", None)).await["id"]);
    refused(t.k.run_action(&officer, "StudentApplicant", ana_app, "admit", Value::Null).await, "invalid", "not available");
    t.k.run_action(&officer, "StudentApplicant", ana_app, "approve", Value::Null).await.expect("approved");
    let admitted = t.k.run_action(&officer, "StudentApplicant", ana_app, "admit", Value::Null).await.expect("admitted");
    let ana = uid(&admitted["student"]["id"]);
    refused(t.k.run_action(&officer, "StudentApplicant", ana_app, "admit", Value::Null).await, "invalid", "not available");
    let student = t.get("Student", ana).await;
    assert_eq!((student["student_name"].as_str(), student["guardians"][0]["guardian_name"].as_str()), (Some("Ana Rao"), Some("Mira Rao")));
    assert_eq!(t.find_one("ProgramEnrollment", json!({"student": ana.to_string()})).await["program"].as_str(), Some(grade5.to_string().as_str()));
    // The applicant's page asks for the reason Reject keeps.
    let reject = officer.schema.entity("StudentApplicant").unwrap().def.actions.iter().find(|a| a.name == "reject").unwrap().clone();
    assert_eq!(reject.params.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(), ["reason"]);
    t.k.run_action(&officer, "StudentApplicant", bo_app, "reject", json!({"reason": "the class is full"})).await.unwrap();
    let bo = t.get("StudentApplicant", bo_app).await;
    assert_eq!(bo["status"], "Rejected");
    assert!(bo["notes"].as_str().unwrap().contains("the class is full"));
    // Someone not enrolled in Grade 5.
    let cy = uid(&t.insert(&manager, "Student", json!({"student_name": "Cy Diaz", "email": "cy@school.test"})).await["id"]);

    // ───────────── fees ─────────────
    let structure = t
        .insert(
            &manager,
            "FeeStructure",
            json!({"structure_name": "Grade 5, 2026-27", "program": grade5.to_string(), "academic_year": year.to_string(),
                   "components": [{"component": "Tuition", "item": tuition.to_string(), "amount": "30000"}, {"component": "Library", "item": library.to_string(), "amount": "2000"}]}),
        )
        .await;
    assert_eq!(dec(&structure["total"]), 32000.0);
    let sid = uid(&structure["id"]);
    refused(
        t.k.insert(&manager, "Fees", obj(json!({"student": cy.to_string(), "fee_structure": sid.to_string(), "due_on": "2026-07-10"}))).await,
        "invalid",
        "not enrolled",
    );
    refused(
        t.k.insert(&teacher, "Fees", obj(json!({"student": ana.to_string(), "fee_structure": sid.to_string(), "due_on": "2026-07-10"}))).await,
        "forbidden",
        "",
    );
    let fees = t.insert(&manager, "Fees", json!({"student": ana.to_string(), "fee_structure": sid.to_string(), "due_on": "2026-07-10"})).await;
    assert_eq!(dec(&fees["amount"]), 32000.0);
    let fid = uid(&fees["id"]);
    let made = t.k.run_action(&manager, "Fees", fid, "make_invoice", Value::Null).await.expect("invoiced");
    let invoice = uid(&made["sales_invoice"]["id"]);
    let inv = t.get("SalesInvoice", invoice).await;
    let customer = t.get("Customer", uid(&inv["customer"])).await;
    assert_eq!(customer["customer_name"], "Mira Rao", "the guardian is invoiced");
    assert_eq!(inv["items"].as_array().unwrap().len(), 2, "one line a component");
    ferrum_app_education::maintenance_on(&k, NaiveDate::from_ymd_opt(2026, 7, 1).unwrap()).await.unwrap();
    assert_eq!(t.get("Fees", fid).await["status"], "Invoiced", "not paid yet");
    t.k.submit(&a, "SalesInvoice", invoice, None).await.expect("submitted");
    let payment = t.k.run_action(&a, "SalesInvoice", invoice, "make_payment", Value::Null).await.expect("a payment");
    t.k.submit(&a, "PaymentEntry", uid(&payment["id"]), None).await.expect("paid");
    ferrum_app_education::maintenance_on(&k, NaiveDate::from_ymd_opt(2026, 7, 2).unwrap()).await.unwrap();
    assert_eq!(t.get("Fees", fid).await["status"], "Paid");
    refused(t.k.update(&manager, "Fees", fid, obj(json!({"fee_structure": Value::Null})), None).await, "invalid", "");

    // ───────────── groups and the timetable ─────────────
    let five_a = uid(&t
        .insert(
            &manager,
            "StudentGroup",
            json!({"group_name": "5A", "program": grade5.to_string(), "academic_year": year.to_string(), "students": [{"student": ana.to_string()}]}),
        )
        .await["id"]);
    refused(
        t.k.insert(
            &manager,
            "StudentGroup",
            obj(json!({"group_name": "5X", "program": grade5.to_string(), "academic_year": year.to_string(), "students": [{"student": cy.to_string()}]})),
        )
        .await,
        "invalid",
        "Cy Diaz is not enrolled",
    );
    refused(
        t.k.insert(&manager, "StudentGroup", obj(json!({"group_name": "5Y", "program": grade5.to_string(), "academic_year": year.to_string(), "students": [{"student": ana.to_string()}, {"student": ana.to_string()}]}))).await,
        "invalid",
        "once",
    );
    let five_b =
        uid(&t.insert(&manager, "StudentGroup", json!({"group_name": "5B", "program": grade5.to_string(), "academic_year": year.to_string()})).await["id"]);
    let ira = uid(&t.insert(&manager, "Instructor", json!({"instructor_name": "Ira Sen", "email": "ira@school.test"})).await["id"]);
    let jo = uid(&t.insert(&manager, "Instructor", json!({"instructor_name": "Jo Park"})).await["id"]);
    let r1 = uid(&t.insert(&manager, "Classroom", json!({"room_name": "R1"})).await["id"]);
    let r2 = uid(&t.insert(&manager, "Classroom", json!({"room_name": "R2"})).await["id"]);
    let class = |group: Uuid, course: Uuid, who: Uuid, room: Uuid, from: &str, to: &str| {
        obj(
            json!({"student_group": group.to_string(), "course": course.to_string(), "instructor": who.to_string(), "room": room.to_string(), "date": "2026-09-01", "from_time": from, "to_time": to}),
        )
    };
    let first = uid(&Value::String(
        t.k.insert(&manager, "CourseSchedule", class(five_a, math, ira, r1, "09:00", "10:00")).await.unwrap()["id"].as_str().unwrap().into(),
    ));
    refused(
        t.k.insert(&manager, "CourseSchedule", class(five_b, science, jo, r1, "09:30", "10:30")).await,
        "invalid",
        "the room has another class then (09:00-10:00)",
    );
    refused(
        t.k.insert(&manager, "CourseSchedule", class(five_b, science, ira, r2, "09:30", "10:30")).await,
        "invalid",
        "the instructor has another class then",
    );
    refused(t.k.insert(&manager, "CourseSchedule", class(five_a, science, jo, r2, "09:30", "10:30")).await, "invalid", "the group has another class then");
    refused(t.k.insert(&manager, "CourseSchedule", class(five_b, science, jo, r2, "11:00", "10:00")).await, "invalid", "after the start");
    let second = t.k.insert(&manager, "CourseSchedule", class(five_b, science, jo, r2, "9:30", "10:30")).await.expect("no clash");
    assert_eq!(second["from_time"], "09:30", "kept as HH:MM");
    t.k.insert(&manager, "CourseSchedule", class(five_a, science, ira, r1, "10:00", "11:00")).await.expect("back to back is no clash");

    // ───────────── attendance and marks ─────────────
    t.insert(&teacher, "StudentAttendance", json!({"course_schedule": first.to_string(), "student": ana.to_string(), "status": "Present"})).await;
    refused(
        t.k.insert(&teacher, "StudentAttendance", obj(json!({"course_schedule": first.to_string(), "student": ana.to_string(), "status": "Late"}))).await,
        "invalid",
        "already",
    );
    refused(
        t.k.insert(&teacher, "StudentAttendance", obj(json!({"course_schedule": first.to_string(), "student": cy.to_string(), "status": "Present"}))).await,
        "invalid",
        "not in this class's group",
    );
    let scale = uid(
        &t.insert(
            &manager,
            "GradingScale",
            json!({"scale_name": "Letters", "grades": [{"grade": "A", "from_percent": "85"}, {"grade": "B", "from_percent": "70"}, {"grade": "C", "from_percent": "50"}, {"grade": "F", "from_percent": "0"}]}),
        )
        .await["id"],
    );
    let midterm = uid(
        &t.insert(&manager, "AssessmentPlan", json!({"assessment_name": "Midterm", "student_group": five_a.to_string(), "course": math.to_string(), "date": "2026-10-05", "maximum_score": "50", "grading_scale": scale.to_string()}))
            .await["id"],
    );
    refused(
        t.k.insert(&teacher, "AssessmentResult", obj(json!({"assessment": midterm.to_string(), "student": ana.to_string(), "score": "60"}))).await,
        "invalid",
        "maximum, 50",
    );
    let result = t.insert(&teacher, "AssessmentResult", json!({"assessment": midterm.to_string(), "student": ana.to_string(), "score": "43"})).await;
    assert_eq!((dec(&result["percent"]), result["grade"].as_str()), (86.0, Some("A")));
    refused(
        t.k.insert(&teacher, "AssessmentResult", obj(json!({"assessment": midterm.to_string(), "student": ana.to_string(), "score": "40"}))).await,
        "invalid",
        "already",
    );
    refused(
        t.k.insert(&teacher, "AssessmentResult", obj(json!({"assessment": midterm.to_string(), "student": cy.to_string(), "score": "40"}))).await,
        "invalid",
        "not in the assessment's group",
    );
    let lower = t.k.update(&teacher, "AssessmentResult", uid(&result["id"]), obj(json!({"score": "35"})), None).await.expect("corrected");
    assert_eq!((dec(&lower["percent"]), lower["grade"].as_str()), (70.0, Some("B")), "regraded");
}
