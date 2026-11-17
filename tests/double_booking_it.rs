//! A clash in the timetable under concurrent requests, against a real PostgreSQL
//! (`FERRUM_EDUCATION_TEST_DATABASE_URL`, default `postgres://ferrum:ferrum@localhost:5432/ferrum_education_test`;
//! reset first): fifty classes saved at once in one room at one time leave one, and so do fifty for
//! one instructor and fifty for one group. Each class is its own request on its own connection, all
//! let go together.

use ferrum_kernel::tenant::NewTenant;
use ferrum_kernel::{Db, DbConfig, Doc, Error, HookRegistry, Kernel, KernelSettings};
use ferrum_meta::Catalog;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::sync::Barrier;
use uuid::Uuid;

const N: usize = 50;
const DAY: &str = "2026-09-01";

fn obj(v: Value) -> Doc {
    v.as_object().cloned().unwrap()
}

fn uid(v: &Value) -> Uuid {
    v.as_str().unwrap_or_else(|| panic!("not an id: {v}")).parse().unwrap()
}

/// How many requests ended each way: `saved`, or the refusal's code and words (numbers masked).
fn tally<T>(outcomes: &[Result<T, Error>]) -> BTreeMap<String, usize> {
    let mut seen = BTreeMap::new();
    for o in outcomes {
        let said = match o {
            Ok(_) => "saved".to_string(),
            Err(e) => format!("{}: {}", e.code(), e.to_string().replace(|c: char| c.is_ascii_digit(), "#")),
        };
        *seen.entry(said).or_insert(0) += 1;
    }
    seen
}

/// Runs `n` calls of `f` at once, each on its own task, let go together; their outcomes in order.
async fn at_once<T, F, Fut>(n: usize, f: F) -> Vec<Result<T, Error>>
where
    T: Send + 'static,
    F: Fn(usize) -> Fut,
    Fut: std::future::Future<Output = Result<T, Error>> + Send + 'static,
{
    let gate = Arc::new(Barrier::new(n));
    let tasks: Vec<_> = (0..n)
        .map(|i| {
            let (gate, call) = (gate.clone(), f(i));
            tokio::spawn(async move {
                gate.wait().await;
                call.await
            })
        })
        .collect();
    let mut out = Vec::with_capacity(n);
    for t in tasks {
        out.push(t.await.expect("a request's task"));
    }
    out
}

#[test]
fn fifty_classes_in_one_slot_yield_one() {
    let rt = tokio::runtime::Builder::new_multi_thread().enable_all().thread_stack_size(16 << 20).build().unwrap();
    rt.block_on(async { tokio::spawn(races()).await.unwrap() });
}

async fn races() {
    let url = std::env::var("FERRUM_EDUCATION_TEST_DATABASE_URL").unwrap_or_else(|_| "postgres://ferrum:ferrum@localhost:5432/ferrum_education_test".into());
    // A connection for each request, and some to spare.
    let db = Db::connect(DbConfig { url, max_connections: (N + 10) as u32, ..Default::default() }).await.expect("connect");
    sqlx::raw_sql("drop schema if exists public cascade; create schema public;").execute(&db.pool).await.expect("reset");
    let mut packages = ferrum_domain::packages().expect("platform packages");
    packages.extend(ferrum_app_erp::packages().expect("ERP's packages"));
    packages.extend(ferrum_app_education::packages().expect("app packages"));
    let catalog = Catalog::from_packages(packages).expect("catalog");
    let mut hooks = HookRegistry::default();
    ferrum_domain::register(&mut hooks);
    hooks.app("erp", &[], ferrum_app_erp::register);
    ferrum_app_education::register(&mut hooks);
    let k = Kernel::boot(db, catalog, hooks, KernelSettings::default(), None).await.expect("boot");
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
            settings: Some(json!({"timezone": "Asia/Kolkata"})),
        })
        .await
        .expect("tenant");
    let session = k.login("school", "admin@school.test", "Correct-Horse-9").await.expect("login");
    let a = k.load_ctx(tenant, session.user, Some(session.token_version), "it".into()).await.expect("ctx");
    let id = |entity: &'static str, v: Value| {
        let (k, a) = (k.clone(), a.clone());
        async move { uid(&k.insert(&a, entity, obj(v)).await.unwrap_or_else(|e| panic!("insert {entity}: {e:?}"))["id"]) }
    };
    // The office makes the timetable.
    let office_id = k.create_user(&a, "office@school.test", "Office", Some("Correct-Horse-9"), &["Education Manager".to_string()], false).await.expect("user");
    let s = k.login("school", "office@school.test", "Correct-Horse-9").await.expect("login");
    let office = k.load_ctx(tenant, office_id, Some(s.token_version), "it".into()).await.expect("ctx");

    let year = id("AcademicYear", json!({"year_name": "2026-27", "starts_on": "2026-06-01", "ends_on": "2027-03-31"})).await;
    let math = id("Course", json!({"course_name": "Mathematics", "code": "MATH"})).await;
    let grade5 = id("Program", json!({"program_name": "Grade 5", "code": "G5"})).await;
    let (mut groups, mut instructors, mut rooms) = (vec![], vec![], vec![]);
    for i in 0..N {
        groups.push(id("StudentGroup", json!({"group_name": format!("5-{i}"), "program": grade5.to_string(), "academic_year": year.to_string()})).await);
        instructors.push(id("Instructor", json!({"instructor_name": format!("Teacher {i}")})).await);
        rooms.push(id("Classroom", json!({"room_name": format!("R{i}")})).await);
    }
    let (groups, instructors, rooms) = (Arc::new(groups), Arc::new(instructors), Arc::new(rooms));
    // Fifty classes at `from`-`to`, the i-th in the group, with the instructor and in the room `pick` gives it.
    let race = |pick: fn(usize) -> (usize, usize, usize), from: &'static str, to: &'static str| {
        let (k, office, groups, instructors, rooms) = (k.clone(), office.clone(), groups.clone(), instructors.clone(), rooms.clone());
        async move {
            let outcomes = at_once(N, |i| {
                let (k, office) = (k.clone(), office.clone());
                let (g, t, r) = pick(i);
                let class = json!({"student_group": groups[g].to_string(), "course": math.to_string(), "instructor": instructors[t].to_string(),
                                   "room": rooms[r].to_string(), "date": DAY, "from_time": from, "to_time": to});
                async move { k.insert(&office, "CourseSchedule", obj(class)).await }
            })
            .await;
            tally(&outcomes)
        }
    };
    let room = race(|i| (i, i, 0), "09:00", "10:00").await;
    let instructor = race(|i| (i, 0, i), "11:00", "12:00").await;
    let group = race(|i| (0, i, i), "14:00", "15:00").await;

    let saved = |t: &BTreeMap<String, usize>| t.get("saved").copied();
    assert_eq!(
        (saved(&room), saved(&instructor), saved(&group)),
        (Some(1), Some(1), Some(1)),
        "one class each: one room {room:#?}, one instructor {instructor:#?}, one group {group:#?}"
    );
    let mut tx = k.db.tenant_tx(tenant).await.unwrap();
    let kept: (i64, i64, i64) = sqlx::query_as(
        "select (select count(*) from e_course_schedule where tenant_id = $1 and \"date\" = $5::date and room = $2 and from_time = '09:00'), \
                (select count(*) from e_course_schedule where tenant_id = $1 and \"date\" = $5::date and instructor = $3 and from_time = '11:00'), \
                (select count(*) from e_course_schedule where tenant_id = $1 and \"date\" = $5::date and student_group = $4 and from_time = '14:00')",
    )
    .bind(tenant)
    .bind(rooms[0])
    .bind(instructors[0])
    .bind(groups[0])
    .bind(DAY)
    .fetch_one(&mut *tx)
    .await
    .unwrap();
    tx.commit().await.unwrap();
    assert_eq!(kept, (1, 1, 1), "one class kept each");
    // Everyone else is told of the clash, in the hook's words (with the other class's times) or the database's.
    for (what, field, seen) in [("room", "room", &room), ("instructor", "instructor", &instructor), ("group", "student_group", &group)] {
        let told = [
            format!("conflict: the {what} has another class then"),
            format!("validation_failed: validation failed: {field}: the {what} has another class then (##:##-##:##)"),
        ];
        assert!(seen.keys().all(|s| s == "saved" || told.contains(s)), "{seen:#?}");
    }
}
