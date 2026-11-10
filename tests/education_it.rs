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
        "must be between 0 and the assessment's maximum, 50",
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

    // ───────────── families ─────────────
    // An application online: the program opens to them, and a guardian consents to the school
    // keeping the details.
    let tenant_id = t.tenant;
    let call = |who: Option<Ctx>, method: &'static str, path: String, body: Value| {
        let k = k.clone();
        async move {
            let site = k.hooks.site("education").expect("education's endpoints").clone();
            let req = ferrum_kernel::SiteRequest {
                tenant: tenant_id,
                user: who,
                method: method.into(),
                path: format!("/{path}"),
                query: Default::default(),
                headers: Default::default(),
                body: if body.is_null() { vec![] } else { body.to_string().into_bytes() },
                client_ip: "203.0.113.7".into(),
            };
            let r = site.handle(&k, req).await?;
            Ok::<_, Error>((r.status, serde_json::from_slice::<Value>(&r.body).unwrap()))
        }
    };
    refused(call(None, "GET", "apply/programs".into(), Value::Null).await, "not found", "");
    t.k.update(&manager, "Program", grade5, obj(json!({"admissions_open": true})), None).await.expect("open to applications");
    let grade6 = uid(&t.insert(&manager, "Program", json!({"program_name": "Grade 6", "code": "G6"})).await["id"]);
    let open = call(None, "GET", "apply/programs".into(), Value::Null).await.unwrap().1;
    assert_eq!((open["programs"][0]["code"].as_str(), open["years"][0]["name"].as_str()), (Some("G5"), Some("2026-27")));
    let form = |consent: bool| {
        json!({"applicant_name": "Dev Iyer", "date_of_birth": "2015-04-02", "program": "G5", "academic_year": "2026-27", "guardian_name": "Lata Iyer",
               "guardian_email": "lata@family.test", "guardian_relation": "Mother", "consent": consent})
    };
    refused(call(None, "POST", "apply".into(), form(false)).await, "invalid", "consent");
    let mut closed = form(true);
    closed["program"] = json!("G6");
    refused(call(None, "POST", "apply".into(), closed).await, "invalid", "is not open to applications");
    assert_eq!(call(None, "POST", "apply".into(), json!({"website": "x"})).await.unwrap(), (201, json!({"status": "received"})));
    let (code, applied) = call(None, "POST", "apply".into(), form(true)).await.expect("applied online");
    assert_eq!((code, applied["application"]["status"].as_str(), applied["application"]["program"].as_str()), (201, Some("Applied"), Some("Grade 5")));
    let token = applied["token"].as_str().unwrap().to_string();
    let dev_app = t.find_one("StudentApplicant", json!({"applicant_name": "Dev Iyer"})).await;
    assert_eq!((dev_app["source"].as_str(), dev_app["consent_given"].as_bool()), (Some("Online"), Some(true)));
    assert!(dev_app["consent_on"].is_string() && dev_app["token_hash"].as_str().is_some_and(|h| h.len() == 64 && h != token), "{dev_app:?}");
    let dev_app = uid(&dev_app["id"]);
    t.k.run_action(&officer, "StudentApplicant", dev_app, "approve", Value::Null).await.unwrap();
    let dev = uid(&t.k.run_action(&officer, "StudentApplicant", dev_app, "admit", Value::Null).await.unwrap()["student"]["id"]);
    assert_eq!(call(None, "GET", format!("apply/status/{token}"), Value::Null).await.unwrap().1["application"]["status"], "Admitted");
    refused(call(None, "GET", format!("apply/status/{}", "A".repeat(43)), Value::Null).await, "not found", "");
    assert_eq!(t.get("Student", dev).await["guardians"][0]["relation"], "Mother");

    // A fee schedule: the term's fees for everyone enrolled, once each, then invoiced together.
    let six_a =
        uid(&t.insert(&manager, "StudentGroup", json!({"group_name": "6A", "program": grade6.to_string(), "academic_year": year.to_string()})).await["id"]);
    refused(
        t.k.insert(&manager, "FeeSchedule", obj(json!({"fee_structure": sid.to_string(), "student_group": six_a.to_string(), "due_on": "2026-12-01"}))).await,
        "invalid",
        "a group of another program or year",
    );
    let term = uid(&t.insert(&manager, "FeeSchedule", json!({"fee_structure": sid.to_string(), "due_on": "2026-12-01"})).await["id"]);
    refused(t.k.run_action(&manager, "FeeSchedule", term, "invoice_fees", Value::Null).await, "invalid", "make them first");
    let made = t.k.run_action(&manager, "FeeSchedule", term, "make_fees", Value::Null).await.expect("made");
    assert_eq!((made["fees_made"].as_i64(), made["message"].as_str()), (Some(2), Some("Made 2 students' fees; 2 in all")), "Ana and Dev, not Cy");
    assert_eq!(t.k.run_action(&manager, "FeeSchedule", term, "make_fees", Value::Null).await.unwrap()["fees_made"], 2, "once each");
    let invoiced = t.k.run_action(&manager, "FeeSchedule", term, "invoice_fees", Value::Null).await.expect("invoiced");
    assert_eq!(invoiced["invoiced"], 2);
    let devs = t.find_one("Fees", json!({"student": dev.to_string(), "fee_schedule": term.to_string()})).await;
    let dev_invoice = t.get("SalesInvoice", uid(&devs["sales_invoice"])).await;
    assert_eq!(t.get("Customer", uid(&dev_invoice["customer"])).await["customer_name"], "Lata Iyer", "his mother is invoiced");
    let group_term = uid(&t
        .insert(&manager, "FeeSchedule", json!({"fee_structure": sid.to_string(), "student_group": five_a.to_string(), "due_on": "2027-01-05"}))
        .await["id"]);
    assert_eq!(t.k.run_action(&manager, "FeeSchedule", group_term, "make_fees", Value::Null).await.unwrap()["fees_made"], 1, "5A is Ana alone");

    // Ana's report card: each course's results averaged and graded, and her attendance.
    let quiz = uid(&t
        .insert(&manager, "AssessmentPlan", json!({"assessment_name": "Quiz", "student_group": five_a.to_string(), "course": science.to_string(), "date": "2026-11-01", "maximum_score": "20", "grading_scale": scale.to_string()}))
        .await["id"]);
    t.insert(&teacher, "AssessmentResult", json!({"assessment": quiz.to_string(), "student": ana.to_string(), "score": "18"})).await;
    let unit_test = uid(&t
        .insert(&manager, "AssessmentPlan", json!({"assessment_name": "Unit test", "student_group": five_a.to_string(), "course": math.to_string(), "date": "2026-11-05", "maximum_score": "100", "grading_scale": scale.to_string()}))
        .await["id"]);
    t.insert(&teacher, "AssessmentResult", json!({"assessment": unit_test.to_string(), "student": ana.to_string(), "score": "80"})).await;
    let mut next_day = class(five_a, math, ira, r1, "09:00", "10:00");
    next_day.insert("date".into(), json!("2026-09-02"));
    let next_day = uid(&Value::String(t.k.insert(&manager, "CourseSchedule", next_day).await.unwrap()["id"].as_str().unwrap().into()));
    t.insert(&teacher, "StudentAttendance", json!({"course_schedule": next_day.to_string(), "student": ana.to_string(), "status": "Absent"})).await;
    let mut third_day = class(five_a, math, ira, r1, "09:00", "10:00");
    third_day.insert("date".into(), json!("2026-09-03"));
    let third_day = uid(&Value::String(t.k.insert(&manager, "CourseSchedule", third_day).await.unwrap()["id"].as_str().unwrap().into()));
    t.insert(&teacher, "StudentAttendance", json!({"course_schedule": third_day.to_string(), "student": ana.to_string(), "status": "Late"})).await;
    let card = t.insert(&teacher, "ReportCard", json!({"student": ana.to_string(), "academic_year": year.to_string(), "comment": "A good term."})).await;
    let courses: Vec<(String, i64, f64, String)> = card["courses"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| (c["course"].as_str().unwrap().to_string(), c["assessments"].as_i64().unwrap(), dec(&c["percent"]), c["grade"].as_str().unwrap().to_string()))
        .collect();
    let mut want = vec![(math.to_string(), 2, 75.0, "B".to_string()), (science.to_string(), 1, 90.0, "A".to_string())];
    want.sort_by(|x, y| x.0.cmp(&y.0));
    let mut got = courses.clone();
    got.sort_by(|x, y| x.0.cmp(&y.0));
    assert_eq!(got, want, "Mathematics: 70% and 80%; Science: 90%");
    assert_eq!(
        (card["classes"].as_i64(), card["present"].as_i64(), card["absent"].as_i64(), dec(&card["attendance_percent"]), card["program"].as_str()),
        (Some(3), Some(1), Some(1), 66.67, Some(grade5.to_string().as_str()))
    );
    refused(
        t.k.insert(&teacher, "ReportCard", obj(json!({"student": ana.to_string(), "academic_year": year.to_string()}))).await,
        "invalid",
        "has its report card already",
    );
    let card = uid(&card["id"]);
    let draft_dev = uid(&t.insert(&teacher, "ReportCard", json!({"student": dev.to_string(), "academic_year": year.to_string()})).await["id"]);
    refused(t.k.run_action(&teacher, "ReportCard", draft_dev, "publish", Value::Null).await, "invalid", "no results or attendance");
    t.k.run_action(&teacher, "ReportCard", card, "publish", Value::Null).await.expect("published");
    refused(t.k.update(&teacher, "ReportCard", card, obj(json!({"comment": "Changed"})), None).await, "invalid", "kept as it was published");

    // The family's portal: a guardian sees their own children, and nobody else's.
    let portal = |email: &'static str| {
        let k = k.clone();
        let a = a.clone();
        async move {
            let id = k.create_user(&a, email, "Family", None, &[], false).await.unwrap();
            k.set_user_kind(&a, id, "portal").await.unwrap();
            k.load_ctx(a.tenant, id, None, "it".into()).await.unwrap()
        }
    };
    assert!(matches!(call(None, "GET", "portal/family".into(), Value::Null).await, Err(Error::Unauthorized(_))), "signed in only");
    let mira = portal("Mira@Family.test").await;
    let mira_user = mira.user;
    let mine = call(Some(mira.clone()), "GET", "portal/family".into(), Value::Null).await.unwrap().1;
    let kids = mine["children"].as_array().unwrap();
    assert_eq!(kids.len(), 1, "Ana only: {mine}");
    let kid = &kids[0];
    assert_eq!((kid["name"].as_str(), kid["enrollments"][0]["program"].as_str()), (Some("Ana Rao"), Some("Grade 5")));
    assert_eq!((kid["attendance"]["present"].as_i64(), kid["attendance"]["absent"].as_i64(), kid["attendance"]["late"].as_i64()), (Some(1), Some(1), Some(1)));
    let fees: Vec<(String, bool)> =
        kid["fees"].as_array().unwrap().iter().map(|f| (f["status"].as_str().unwrap().to_string(), f["invoice"].is_string())).collect();
    assert_eq!(fees.len(), 3, "{fees:?}");
    assert!(fees.contains(&("Paid".to_string(), true)) && fees.contains(&("Invoiced".to_string(), false)), "a draft invoice is not the family's yet: {fees:?}");
    let report = &kid["report_cards"][0];
    assert_eq!(
        (report["academic_year"].as_str(), dec(&report["attendance_percent"]), report["comment"].as_str()),
        (Some("2026-27"), 66.67, Some("A good term."))
    );
    assert!(report["courses"].as_array().unwrap().iter().any(|c| c["course"] == "Mathematics" && c["grade"] == "B"), "{report}");
    assert!(kid["consents"].as_array().unwrap().iter().all(|c| c["given"] == false));
    let lata = portal("lata@family.test").await;
    let lata_user = lata.user;
    let hers = call(Some(lata.clone()), "GET", "portal/family".into(), Value::Null).await.unwrap().1;
    assert_eq!(hers["children"].as_array().unwrap().iter().map(|c| c["name"].as_str().unwrap()).collect::<Vec<_>>(), ["Dev Iyer"]);
    assert!(hers["children"][0]["report_cards"].as_array().unwrap().is_empty(), "a draft report card is not shown");
    let stranger = portal("someone@else.test").await;
    assert!(call(Some(stranger), "GET", "portal/family".into(), Value::Null).await.unwrap().1["children"].as_array().unwrap().is_empty());
    // A family gives and withdraws its consents; who and when are kept.
    let ask = |student: Uuid, what: &str, given: bool| json!({"student": student.to_string(), "consent": what, "given": given});
    let gave = call(Some(mira.clone()), "POST", "portal/consents".into(), ask(ana, "Photos and Media", true)).await.expect("given").1;
    assert_eq!(gave["message"], "Photos and Media: consent given");
    let ana_now = t.get("Student", ana).await;
    let row = ana_now["consents"].as_array().unwrap().iter().find(|c| c["consent"] == "Photos and Media").cloned().unwrap();
    assert_eq!((row["given"].as_bool(), row["by"].as_str(), row["given_on"].is_string()), (Some(true), Some("mira@family.test"), true));
    refused(call(Some(mira.clone()), "POST", "portal/consents".into(), ask(dev, "School Trips", true)).await, "not found", "");
    refused(call(Some(mira.clone()), "POST", "portal/consents".into(), ask(ana, "Anything", true)).await, "invalid", "not one the school asks for");
    call(Some(mira.clone()), "POST", "portal/consents".into(), ask(ana, "Photos and Media", false)).await.expect("withdrawn");
    let again = call(Some(mira), "GET", "portal/family".into(), Value::Null).await.unwrap().1;
    let photos = again["children"][0]["consents"].as_array().unwrap().iter().find(|c| c["consent"] == "Photos and Media").cloned().unwrap();
    assert_eq!(photos["given"], false);
    let _ = t.k.update(&manager, "Student", ana, obj(json!({"consents": []})), None).await;
    assert_eq!(t.get("Student", ana).await["consents"].as_array().unwrap().len(), 1, "the office does not rewrite what the family said");

    // ───────────── the same portal as configuration (Ferrum BF-PORTAL-006) ─────────────
    // The package's Family role, given to each guardian: no app code between them and their children's
    // records, which must be exactly what the app's page shows them.
    let family = |user: Uuid| {
        let k = k.clone();
        let a = a.clone();
        async move {
            k.set_user_roles_until(&a, user, &["Family".to_string()], &Default::default()).await.expect("a portal role for a portal person");
            k.load_ctx(a.tenant, user, None, "it".into()).await.unwrap()
        }
    };
    let (mira_g, lata_g) = (family(mira_user).await, family(lata_user).await);
    let all = |ctx: Ctx, entity: &'static str| {
        let k = k.clone();
        async move { k.list(&ctx, entity, &ListQuery { limit: Some(500), ..Default::default() }).await.unwrap().rows }
    };
    let ids = |rows: &[Doc]| rows.iter().map(|r| r["id"].as_str().unwrap().to_string()).collect::<Vec<_>>();
    let page = call(Some(mira_g.clone()), "GET", "portal/family".into(), Value::Null).await.unwrap().1;
    let kid = &page["children"][0];
    assert_eq!(ids(&all(mira_g.clone(), "Student").await), [ana.to_string()], "her children, as the page lists them");
    assert_eq!(ids(&all(lata_g.clone(), "Student").await), [dev.to_string()]);
    let cards = all(mira_g.clone(), "ReportCard").await;
    assert_eq!((cards.len(), kid["report_cards"].as_array().unwrap().len()), (1, 1), "published report cards");
    assert_eq!(ids(&cards), [card.to_string()]);
    assert!(all(lata_g.clone(), "ReportCard").await.is_empty(), "a draft report card is not the family's, as on the page");
    assert_eq!(all(mira_g.clone(), "ProgramEnrollment").await.len(), kid["enrollments"].as_array().unwrap().len());
    let marks = all(mira_g.clone(), "StudentAttendance").await;
    for status in ["Present", "Absent", "Late"] {
        assert_eq!(marks.iter().filter(|m| m["status"] == status).count() as i64, kid["attendance"][status.to_lowercase()].as_i64().unwrap(), "{status}");
    }
    let open_fees = all(mira_g.clone(), "Fees").await.into_iter().filter(|f| f["status"] != "Cancelled").count();
    assert_eq!(open_fees, kid["fees"].as_array().unwrap().len(), "fees");
    for (entity, other) in [("Student", dev), ("ReportCard", draft_dev)] {
        assert!(t.k.get(&mira_g, entity, other, false).await.is_err(), "another family's {entity}");
    }
    let her_child = t.k.get(&mira_g, "Student", ana, false).await.unwrap();
    assert!(her_child.get("date_of_birth").is_none_or(Value::is_null), "a personal field reached the portal");
    refused(t.k.update(&mira_g, "Student", ana, obj(json!({"student_name": "Renamed"})), None).await, "forbidden", "");
    // Consents through the role's one action, recorded as the page records them.
    let consent = |what: &str, given: &str| json!({"consent": what, "given": given});
    t.k.run_action(&mira_g, "Student", ana, "set_consent", consent("School Trips", "Given")).await.expect("given through the portal role");
    let trips = t.get("Student", ana).await["consents"].as_array().unwrap().iter().find(|c| c["consent"] == "School Trips").cloned().unwrap();
    assert_eq!((trips["given"].as_bool(), trips["by"].as_str()), (Some(true), Some("mira@family.test")));
    refused(t.k.run_action(&mira_g, "Student", dev, "set_consent", consent("School Trips", "Given")).await, "not found", "");
    refused(t.k.run_action(&mira_g, "Student", ana, "set_consent", consent("Anything", "Given")).await, "invalid", "not one the school asks for");
}
