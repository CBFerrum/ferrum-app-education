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
- **Applications online.** A program open to applications is on the school's public page,
  /school/apply/{workspace}: a family applies for a year still to come, with a guardian to reach and
  their consent to the school keeping the details (when and that it was given are kept). They get the
  application's own link to see where it stands; only its hash is kept, and writes are limited.
- **The family's portal.** A guardian signed in to the workspace's portal with the address the school
  lists sees each of their children: enrollment, this year's attendance, fees (an issued invoice opens
  on ERP's "Your invoices"), published report cards, and the family's consents (photos and media,
  school trips, sharing with Learn), which they give or withdraw there; who and when are kept, and the
  office does not rewrite them. Nobody else's children.
- **Fee schedules.** A fee structure due on a day for every student enrolled in its program and year,
  or one group of them: making it makes each student's fees once, and invoicing it invoices those
  still drafts.
- **Report cards.** A student's year: each course's results averaged and graded on the scale of its
  latest assessment, their attendance (present or late, of the classes taken), and a comment. Worked
  out again while a draft; published, kept as it was and shown to the family. One a year.
- **Fees through ERP.** A fee structure totals its components. A student's fees take that total,
  only for a program and year they are enrolled in. They are invoiced as a draft Sales Invoice, one
  line per component, to the first guardian, or to the student when there is no guardian. Upkeep
  marks them paid once the invoice is submitted and settled.
- **A timetable without clashes.** No room, instructor or student group is in two classes at once.
  Back to back is fine, and times are kept as HH:MM.
- **Groups** hold students enrolled in their program and year, each once. **Attendance** is taken only
  for a class's own students, once each, on the class's roll call: its group's students, present
  unless marked otherwise, saved at once.
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

- Report cards as printed documents, terms within a year, and comments by course.
- Timetables made from rules, and room capacities enforced.
- Courses' content and quizzes (Learn has them): consent to share with Learn is recorded, nothing is
  sent there yet. Paying fees online (ERP's invoices page shows what is due).

## Changelog

| Version | Highlights |
|---|---|
| **0.3.0** | Applications online with a guardian's consent; the family's portal (attendance, fees, report cards, consents); fee schedules; report cards (FEO-047). |
| **0.2.8** | Its names in Arabic and Hindi; the last English sentences translated. |
| **0.2.7** | The assistant knows its records; personal and health details in field groups. |
| **0.2.6** | Rebuilt with Ferrum-Frontend 5b22e7b: its own words in Arabic and Hindi. |
| **0.2.5** | Rebuilt with Ferrum-Frontend 6537d65: once-shown answers kept; English isolated right to left; its own words translatable by a workspace. |
| **0.2.4** | Rebuilt with Ferrum-Frontend d5d98f2: a workspace's own interface words; pictures in attachment fields; a duplicate value said so. |
| **0.2.3** | Rebuilt with Ferrum-Frontend 63280d3: right to left, pages mirror; the records pages' words in Arabic and Hindi. |
| **0.2.2** | Rebuilt with Ferrum-Frontend a64416a: a box whose default is 1 shows ticked. |
| **0.2.1** | Rebuilt with Ferrum-Frontend 922fcfb: action dialogs say every refusal and ask once. |
| **0.2.0** | What walking a school day found: a class's roll call; today's classes named and on the workspace's day; counts beyond 200; an empty office says where to start; a score's limit written plainly. |
| **0.1.8** | Rebuilt with Ferrum-Frontend a76295e: times on a person's own clock when they chose one. |
| **0.1.7** | Rebuilt with Ferrum-Frontend 24c3951: times on the workspace's clock; drafts headed as drafts. |
| **0.1.6** | Rebuilt with Ferrum-Frontend bb60115: empty numbers read as the server reads them; settings open as their record. |
| **0.1.5** | Reject asks for the reason it keeps. |
| **0.1.4** | Rebuilt with Ferrum-Frontend 00dea07: a workflow state's kept fields are read-only, its needed ones required. |
| **0.1.3** | Personal-data requests list the student, student applicant records holding the person's name apart, for an administrator to judge (never erased for the name … |
| **0.1.2** | Rebuilt with Ferrum-Frontend daeadda: actions say what they did. |
| **0.1.1** | Rebuilt with Ferrum-Frontend 80ff179: labels name their fields, and linked titles show their titles. |
| **0.1.0** | Admissions, students and guardians, fees through ERP, a timetable without clashes, assessments and attendance (FERRUM-INDUSTRY-004). |

## License

AGPL-3.0-only. See [LICENSE](LICENSE).
