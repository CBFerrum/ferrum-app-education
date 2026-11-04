# Education

A school's office for [Ferrum](https://github.com/CBFerrum/Ferrum):
- admissions, students and their guardians, and programs and courses;
- fees invoiced through ERP, and a timetable without clashes;
- assessments graded on a scale, and attendance.

| | |
|---|---|
| 🎓 App | `education`, native, in the server |
| 🧩 Package | `education` (needs ERP's selling and accounting) |
| 🖥️ Interface | its own, at `/school/`: today's classes, what waits on the office, and the records |
| 🔐 Roles | Education Manager, Teacher, Admissions Officer |
| 📜 Licence | AGPL-3.0-only |

## Highlights

- **Admissions.** An applicant is approved or rejected, and an approved one admitted. Admitting makes
  the student, with the applicant's guardian, and their enrollment in the program and year, once.
- **Fees through ERP.** A fee structure totals its components. A student's fees take that total,
  only for a program and year they are enrolled in. They are invoiced as a draft Sales Invoice, one
  line per component, to the first guardian, or to the student when there is no guardian. Upkeep
  marks them paid once the invoice is submitted and settled.
- **A timetable without clashes.** No room, instructor or student group is in two classes at once.
  Back to back is fine, and times are kept as HH:MM.
- **Groups** hold students enrolled in their program and year, each once. **Attendance** is taken only
  for a class's own students, once each.
- **Assessments** turn a score within the maximum into a percentage and a grade: the highest band
  of the scale the percentage reaches. Each student has one result per assessment, regraded when the
  score is corrected.
- **Personal data** requests find applicants and students by email and erase their names, contacts
  and dates of birth. Marks, fees and attendance stay, as the school's records need them.

## Develop

```bash
cargo test                       # needs FERRUM_EDUCATION_TEST_DATABASE_URL's database
cd ui && npm run build && npm run check
```

`tests/education_it.rs` walks a term:
- an admission, and fees paid through ERP;
- groups, and a timetable with each kind of clash refused;
- attendance, and a graded assessment;
- who may do what.

## Not yet

- Online applications, a parents' portal, and report cards as documents.
- Timetables made from rules, and room capacities enforced.
- Courses' content and quizzes (Learn has them), and fee schedules for many students at once.

## Changelog

| Version | Highlights |
|---|---|
| **0.1.0** | Admissions, students and guardians, fees through ERP, a timetable without clashes, assessments and attendance (FERRUM-INDUSTRY-004). |

## License

AGPL-3.0-only. See [LICENSE](LICENSE).
