<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/assets/banner-dark.svg">
  <img alt="Ferrum Education: admissions, students, fees, timetable and report cards for a school" src="docs/assets/banner-light.svg" width="100%">
</picture>

<br>

[![Version](https://img.shields.io/badge/version-0.4.2-1d4ed8?style=flat-square)](CHANGELOG.md)
[![Licence: AGPL-3.0](https://img.shields.io/badge/licence-AGPL--3.0-blue?style=flat-square)](LICENSE)
[![Ferrum platform](https://img.shields.io/badge/runs%20on-Ferrum-2f5bea?style=flat-square)](https://github.com/CBFerrum/Ferrum)
![Rust](https://img.shields.io/badge/Rust-server-b7410e?style=flat-square&logo=rust)
![PostgreSQL](https://img.shields.io/badge/PostgreSQL-data-336791?style=flat-square&logo=postgresql&logoColor=white)
![React](https://img.shields.io/badge/React%20%2B%20TypeScript-interface-087ea4?style=flat-square&logo=react&logoColor=white)
![Tests](https://img.shields.io/badge/tests-2%20Rust%20%2B%20UI%20suite-16a34a?style=flat-square)

### A school's office in one place: admissions online, fees in the books, a timetable that cannot clash, and a family portal.

Ferrum Education is for schools and small colleges. Families apply online. The office admits students, bills fees through
Ferrum ERP, builds a timetable and takes attendance. Teachers enter results, the school publishes report cards, and each
guardian sees their children on the workspace's portal.

[💡 Why](#-why-education) · [📸 Screenshots](#-screenshots) · [✨ Features](#-features) · [📊 Compare](#-how-it-compares) · [📍 Roadmap](#-roadmap) · [🚀 Install](#-install) · [🔧 Develop](#-develop)

</div>

## 💡 Why Education

- 📝 **Admissions from the first click.** A public page takes applications for the programs that are open, with a guardian to
  reach and their consent to the school keeping the details. The family gets a private link to see where it stands.
- 🎓 **Admit in one step.** Admitting an applicant makes the student, with their guardian, and their enrollment in the program and
  year, once.
- 💳 **Fees in the school's own books.** Fees are invoiced to the first guardian through ERP as a draft Sales Invoice, and marked
  paid once the invoice is.
- 🗓️ **A timetable that cannot double-book.** No room, instructor or student group is in two classes at once.
- 👨‍👩‍👧 **A family portal.** Guardians see each child's enrollment, attendance, fees and published report cards, and give or withdraw
  the family's consents. Nobody else's children.
- 🔏 **Children's data treated with care.** Dates of birth are in a restricted field group, and a personal-data request erases names,
  contacts and birth dates while marks, fees and attendance stay.
- 🔓 **Open source and self-hosted.** AGPL-3.0, on your own PostgreSQL.
- 🌍 **Arabic and Hindi** words and names, with right-to-left layout.

## 🎯 At a glance

| | |
|---|---|
| **Version** | 0.4.2 |
| **Kind** | Its own interface at `/school/` (today's classes, a class roll call, what waits on the office); the public application page; the family's portal page; every record in Ferrum's lists and forms |
| **What it adds** | 26 record types (18 documents, 7 child tables, 1 settings record), 4 roles (one for the portal), 1 menu |
| **Depends on** | Ferrum platform `>=0.3.2, <0.4`; ERP `>=0.1` (installing Education installs ERP) |
| **Works with** | ERP (customers, items, companies, sales invoices); Learn (consent to share is recorded, nothing is sent yet) |
| **Languages** | English, Arabic, Hindi (words and record names); right-to-left layout |
| **Tests** | 2 Rust tests (one walks a term on PostgreSQL) and a UI suite |
| **Licence** | AGPL-3.0-only |

## 📸 Screenshots

<div align="center">
<picture><source media="(prefers-color-scheme: dark)" srcset="docs/assets/screens/education-home-dark.webp"><img alt="The school day" src="docs/assets/screens/education-home-light.webp" width="100%"></picture>
<p><b>🏫 The school day</b><br><sub>Today&#x27;s classes, applicants waiting for a decision and fees invoiced, with the records behind them.</sub></p>
</div>

<table>
<tr>
<td width="50%" valign="top">
<picture><source media="(prefers-color-scheme: dark)" srcset="docs/assets/screens/education-student-list-dark.webp"><img alt="Students" src="docs/assets/screens/education-student-list-light.webp"></picture>
<p><b>🧒 Students</b><br><sub>Browse, search and filter students.</sub></p>
</td>
<td width="50%" valign="top">
<picture><source media="(prefers-color-scheme: dark)" srcset="docs/assets/screens/education-program-list-dark.webp"><img alt="Programs" src="docs/assets/screens/education-program-list-light.webp"></picture>
<p><b>📚 Programs</b><br><sub>Each program with its code and whether families can apply online.</sub></p>
</td>
</tr>
<tr>
<td width="50%" valign="top">
<picture><source media="(prefers-color-scheme: dark)" srcset="docs/assets/screens/education-course-list-dark.webp"><img alt="Courses" src="docs/assets/screens/education-course-list-light.webp"></picture>
<p><b>📖 Courses</b><br><sub>The courses the school teaches, by code.</sub></p>
</td>
<td width="50%" valign="top">
<picture><source media="(prefers-color-scheme: dark)" srcset="docs/assets/screens/education-student-detail-dark.webp"><img alt="One student" src="docs/assets/screens/education-student-detail-light.webp"></picture>
<p><b>🗂️ One student</b><br><sub>Read one student: guardians, status, billing customer and the family&#x27;s consents.</sub></p>
</td>
</tr>
<tr>
<td width="50%" valign="top">
<picture><source media="(prefers-color-scheme: dark)" srcset="docs/assets/screens/education-class-dark.webp"><img alt="A class&#x27;s roll call" src="docs/assets/screens/education-class-light.webp"></picture>
<p><b>✅ A class&#x27;s roll call</b><br><sub>Take attendance for today&#x27;s class: present unless marked otherwise, saved at once.</sub></p>
</td>
<td width="50%" valign="top">
<picture><source media="(prefers-color-scheme: dark)" srcset="docs/assets/screens/education-fees-list-dark.webp"><img alt="Fees" src="docs/assets/screens/education-fees-list-light.webp"></picture>
<p><b>💳 Fees</b><br><sub>The fees invoiced through ERP and which are still to be paid.</sub></p>
</td>
</tr>
<tr>
<td width="50%" valign="top">
<picture><source media="(prefers-color-scheme: dark)" srcset="docs/assets/screens/education-timetable-dark.webp"><img alt="The timetable" src="docs/assets/screens/education-timetable-light.webp"></picture>
<p><b>🗓️ The timetable</b><br><sub>Every class with its date, time, course, group, instructor and room.</sub></p>
</td>
<td width="50%" valign="top">
<picture><source media="(prefers-color-scheme: dark)" srcset="docs/assets/screens/education-applicants-dark.webp"><img alt="Applicants" src="docs/assets/screens/education-applicants-light.webp"></picture>
<p><b>📝 Applicants</b><br><sub>Review applicants, approve or reject them, and admit them.</sub></p>
</td>
</tr>
</table>

### 📱 On a phone

<table>
<tr>
<td align="center" valign="top">
<picture><source media="(prefers-color-scheme: dark)" srcset="docs/assets/screens/education-home-phone-dark.webp"><img alt="The school day on a phone" src="docs/assets/screens/education-home-phone-light.webp" width="260"></picture>
<p><b>🏫 The school day on a phone</b><br><sub>The same page on a narrow screen.</sub></p>
</td>
<td align="center" valign="top">
<picture><source media="(prefers-color-scheme: dark)" srcset="docs/assets/screens/education-student-list-phone-dark.webp"><img alt="Students on a phone" src="docs/assets/screens/education-student-list-phone-light.webp" width="260"></picture>
<p><b>🧒 Students on a phone</b><br><sub>Look up a student from the corridor.</sub></p>
</td>
</tr>
</table>


## ✨ Features

### 📝 Admissions

- **Programs and years.** A program has a code, courses (each required or not) and a switch, *Applications Open Online*. An
  academic year has a name and start and end dates.
- **Applicants.** Name, email, date of birth (a restricted field), program, academic year, guardian name, email, phone and
  relation (*Mother, Father, Guardian, Other*), notes, and where it came from (*Office* or *Online*).
- **Four states.** *Applied, Approved, Rejected, Admitted.* Approve, Reject and Admit show only when they apply. Reject asks for a
  reason, which is kept in the applicant's notes.
- **Admit.** Admitting makes the student, with the applicant's guardian, and their enrollment in the program and year, once, and
  links the student to the applicant.
- **Numbered** `APP-2026-00001`.

### 🌐 The public application page

- **At `/school/apply/{workspace}`,** without signing in, for programs open to applications and academic years not yet over.
- **What a family gives.** The applicant's name and date of birth, the guardian's name, relation, email and phone, and the guardian's
  agreement to the school keeping and using these details. Without that agreement the application is refused. When and that it
  was given are kept.
- **A link to follow it.** The family gets the application's own link to see where it stands. Only a hash of it is kept, and writes
  are limited per address.
- **A page that explains itself** when the school is not taking applications.

### 🧒 Students, guardians and enrollment

- **Students.** Name, email, date of birth, status (*Active, Graduated, Left*), the customer fees are invoiced to (made from the first
  guardian, or from the student when there is none), and guardians with relation, email and phone.
- **Enrollments.** A student in a program for an academic year, with the date enrolled.
- **Family consents.** The consents *Photos and Media*, *School Trips* and *Sharing With Learn*. A guardian gives or withdraws each on
  the portal, or the office does it as an action. Who and when are kept, and the office does not rewrite them. Consent to share with
  Learn is recorded; nothing is sent there yet.
- **Numbered** `STU-2026-00001`. Students can be imported and exported.

### 💳 Fees

- **Fee structures.** A structure belongs to a program and year and totals its components, each an item and an amount.
- **A student's fees.** One student's fees take the structure's total, only for a program and year they are enrolled in, with a due date.
- **Four states.** *Draft, Invoiced, Paid, Cancelled.* Invoice It makes a draft Sales Invoice through ERP, one line per component,
  to the student's customer. An invoiced fee does not change. Upkeep marks fees paid once the invoice is submitted and settled.
- **Fee schedules.** A structure due on a day for every student enrolled in its program and year, or in one group of them. Make the
  Fees makes each student's fees once; Invoice Them invoices those still drafts. A group must belong to the structure's program and year.
- **Settings.** The company that fees are invoiced from is set in Education Settings.
- **Numbered** `FEE-2026-00001`.

### 🗓️ Timetable and groups

- **Groups.** A student group holds students enrolled in its program and year, each once.
- **Classes.** A class is a group, a course, an instructor and a room at a time on a date, with times in `HH:MM`.
- **No clashes.** No room, instructor or student group is in two classes at once. Back to back is fine, and the end must be after the start.
- **Instructors and rooms.** Instructors have a name and email; rooms have a name and seats.

### ✅ Attendance

- **Roll call.** Open a class from the home page: its group's students are listed, present unless marked otherwise, and saved at once.
  The page says how many were saved and which are not taken yet.
- **Statuses.** *Present, Absent, Late, Excused.*
- **Only the class's students, once each.** Attendance for a student outside the class's group, or twice for the same class, is refused.

### 🏅 Assessments and grades

- **Grading scales** are bands, each with a grade and the lowest percentage that earns it.
- **Assessments.** A name, a group, a course, a date, a maximum score and a grading scale.
- **Results.** One result for each student and assessment. A score within the maximum becomes a percentage and the highest band the
  percentage reaches. Regraded when the score is corrected, with an optional comment.

### 📄 Report cards

- **A student's year.** Each course's results averaged and graded on the scale of its latest assessment, attendance (present or late,
  of the classes taken, with counts of present, absent, late and excused), and a comment. One report card a student a year.
- **Draft, then published.** A draft is worked out again whenever it is saved. Publish It keeps the card as it was and shows it to the family
  on the portal. A published report card is not changed. A card with no results and no attendance cannot be published.

### 🧑‍🧒 The family's portal

- **At the workspace's portal.** A guardian signed in with the address the school lists sees each of their children: enrollment, this
  year's attendance, fees (an issued invoice opens on ERP's "Your invoices"), published report cards, and the family's consents.
- **Give or withdraw consent** there; who and when are kept.
- **Nobody else's children.** What is theirs is found through the guardian addresses on each student, set up by the Family role.

### 🏠 The school's home

- **Today's classes,** in time order, each opening its roll call.
- **What waits on the office.** Applicants waiting for a decision and fees invoiced and not yet paid.
- **First run.** An empty office says where to start.

### 🔐 Roles and permissions

| Who | Can |
|---|---|
| 🎓 Education Manager | Everything: settings, years, courses, programs, applicants, students (including the personal group), enrollments, fees and structures, fee schedules, report cards, instructors, rooms, groups, the timetable, attendance, grading scales, assessments and results; reads items and companies; creates customers and invoices |
| 🧑‍🏫 Teacher | Sees the timetable, groups, courses and students; takes attendance; enters results; works on report cards |
| 📋 Admissions Officer | Takes applications and approves, rejects or admits them; reads programs, years, students and enrollments |
| 👨‍👩‍👧 Family (portal) | A guardian's own children: enrollments, attendance, fees, published report cards, and the consent action |

### 🔏 Personal data

- **Found by email.** A personal-data request finds applicants and students by email and erases names, contacts, guardian details and
  dates of birth. Marks, fees and attendance stay, as the school's records need them.
- **Sensitive details held back.** Dates of birth are a restricted field group, and the assistant sees them only where the workspace
  allows it.

### 🌍 Interface

- **Its own pages.** The school's home, a class's roll call, the public application page and the family's portal; every record uses the
  shared list and form pages.
- **Languages.** English, Arabic and Hindi words, and Arabic and Hindi names for every record and field. A workspace can add its own words in Admin.
- **Right to left.** Pages mirror as they read.
- **Assistant.** It knows the states and treats children's details carefully. It suggests "Which students have unpaid fees?", "Who was absent
  today?", "Which applicants are waiting for a decision?" and "What is on the timetable tomorrow?".
- **Accessibility basics.** Skip links, labelled controls and a roll call built as a list of grouped choices.

## 🧭 How it works

```mermaid
flowchart LR
  F["Family applies online or at the office"] --> AP["Applicant: Applied"]
  AP --> OK["Approved"]
  AP --> NO["Rejected, with a reason"]
  OK --> AD["Admitted"]
  AD --> ST["Student and enrollment"]
  ST --> FE["Fees from a fee structure"]
  FE --> INV["Draft sales invoice in ERP"]
  INV --> PD["Fees paid"]
```

Admitting makes the student and the enrollment in one step. Fees are invoiced to the first guardian's customer, and upkeep marks them paid
once ERP shows the invoice settled.

```mermaid
flowchart TD
  G["Student group"] --> CL["Class: course, instructor, room, time"]
  CL --> AT["Attendance, roll call"]
  AS["Assessment on a grading scale"] --> RS["Result: percent and grade"]
  AT --> RC["Report card (draft)"]
  RS --> RC
  RC -->|publish| PUB["Published: the family sees it"]
```

A class has a group, a course, an instructor and a room, with no double-booking. Attendance and results feed the year's report card.

## 🧱 Data model

| Entity | What it is | Key fields or states |
|---|---|---|
| Academic Year, Course, Program | The school's calendar and curriculum | Year dates; course code; program code, courses, applications open online |
| Applicant | Someone applying to a program | Program, year, guardian, consent, source; *Applied, Approved, Rejected, Admitted* |
| Student | A student | Email, date of birth, customer, guardians, consents; *Active, Graduated, Left* |
| Enrollment | A student in a program for a year | Student, program, year, date |
| Fee Structure, Fee Schedule | What a program charges and when | Components and total; structure, group, due date, fees made and invoiced |
| Fees | What one student owes | Student, structure, due date, amount, invoice; *Draft, Invoiced, Paid, Cancelled* |
| Instructor, Room | Who teaches and where | Name, email; name, seats |
| Student Group | A set of enrolled students | Program, year, students |
| Class (`CourseSchedule`) | One class in the timetable | Group, course, instructor, room, date, from, to |
| Attendance | A student at a class | Class, student; *Present, Absent, Late, Excused* |
| Grading Scale, Assessment, Result | Marks | Bands; group, course, date, maximum; score, percent, grade |
| Report Card | A student's year | Courses, attendance counts and percent, comment; *Draft, Published* |
| Education Settings | Where fees are invoiced from | Company |

**Roles:** Education Manager, Teacher, Admissions Officer; Family on the portal.

**Reports and print formats:** none yet. The lists are the school's figures.

**Scheduled upkeep:** fees are marked paid once their invoices are settled.

## 🔌 Integrations

| With | How | Needs |
|---|---|---|
| **Ferrum ERP** | Customers, items and companies; fees become Sales Invoices; paid status follows the invoice | ERP installed with the app; a company in Education Settings |
| **Ferrum portal** | The Family role and the children's page | Portal enabled; guardians made portal people |
| **Learn** | A family's consent to share is recorded | Nothing is sent to Learn yet |
| **Personal-data requests** | Applicants and students declared with what is erased | Nothing |

Not built in: online fee payment, SMS, a state or national reporting format, and payment gateways.

## 📊 How it compares

Education is compared with **PowerSchool**, **Ellucian Banner**, **Blackbaud** (its school products), **openSIS** and **Frappe Education** (the
ERPNext-family module this app's design started from; Fedena is another open-source school system). The big three are
large vendor-hosted or enterprise products; this one is the office side of a school, on the same books as the rest of the business.

| Capability | Ferrum Education | PowerSchool | Ellucian Banner | Blackbaud | openSIS | Frappe Education |
|---|:-:|:-:|:-:|:-:|:-:|:-:|
| Public online application with a guardian's consent | ✅ | ✅ | 🟡 | ✅ | 🟡 | ✅ |
| Admit an applicant into a student and enrollment in one step | ✅ | ✅ | ✅ | ✅ | 🟡 | ✅ |
| Students with several guardians | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Programs and courses | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Fee structures and fee schedules for a program and year | ✅ | 🟡 | ✅ | ✅ | 🟡 | ✅ |
| Fees invoiced in the same ledger as the school's accounting | ✅ | ❌ | ✅ | 🟡 | ❌ | ✅ |
| Timetable that refuses room, instructor and group clashes | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Attendance by class with a roll call | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Assessments graded on a scale | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Report cards published to the family | 🟡 | ✅ | 🟡 | ✅ | ✅ | 🟡 |
| Guardian portal: attendance, fees, report cards | ✅ | ✅ | 🟡 | ✅ | ✅ | 🟡 |
| Online fee payment | ❌ | ✅ | ✅ | ✅ | ❌ | 🟡 |
| Academic terms and semesters within a year | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Timetable generated from rules and constraints | ❌ | ✅ | 🟡 | ✅ | 🟡 | ❌ |
| Room capacities enforced | ❌ | ✅ | ✅ | ✅ | 🟡 | ❌ |
| Absence alerts to guardians | ❌ | ✅ | 🟡 | ✅ | 🟡 | ❌ |
| Gradebook with weighted categories and standards-based grading | ❌ | ✅ | 🟡 | ✅ | 🟡 | 🟡 |
| Transcripts, credits and GPA | ❌ | ✅ | ✅ | ✅ | ✅ | ❌ |
| Enquiries and recruitment before an application | ❌ | ✅ | 🟡 | ✅ | ❌ | ❌ |
| Learning management: course content and quizzes | 🟡 | ✅ | ❌ | ✅ | 🟡 | ❌ |
| Discipline, health and special-education records | ❌ | ✅ | 🟡 | 🟡 | ✅ | ❌ |
| State or national reporting formats | ❌ | ✅ | ✅ | 🟡 | 🟡 | ❌ |
| Higher-education registration, degree audit, financial aid | ❌ | 🟡 | ✅ | ➖ | ➖ | ❌ |
| Mobile apps for staff and families | ❌ | ✅ | ✅ | ✅ | 🟡 | ❌ |
| Self-hosted on your own servers | ✅ | 🟡 | 🟡 | ❌ | ✅ | ✅ |
| Open source | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ |
| No licence fee per student | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ |

Notes. **Ferrum 🟡:** report cards are published as records to the portal and are not yet printed documents; Learn is a separate Ferrum
app, and only the family's consent to share is recorded. **Leaders:** 🟡 means a lighter feature, an add-on, a different edition or a
feature reached through an integration. Ellucian Banner is aimed at higher education; PowerSchool and Blackbaud serve schools mostly
at the K-12 level, with differences by product.

### 🏆 Where Education stands out

- 💳 **The fee is an invoice in the school's books.** No separate finance system, no export.
- 🗓️ **Clash-free timetable.** Enforced for room, instructor and group.
- 👨‍👩‍👧 **Consent that is kept as given.** Families give or withdraw consents on the portal and the office cannot rewrite them.
- 🔏 **Children's data by rule.** Restricted fields and a personal-data erasure that keeps marks, fees and attendance.
- 🔓 **Open and owned.** AGPL, self-hosted, no per-student price.

### 🚧 Where the leaders are still ahead

- 📆 **Academic terms,** transcripts, credits and GPA.
- 🧠 **Scheduling and grading depth.** Rule-based timetables, room capacities, weighted gradebooks and standards-based grading.
- 💳 **Online payment** and absence alerts.
- 🏛️ **Reporting formats,** discipline and special-education records.
- 📱 **Mobile apps.**

> [!NOTE]
> Leader columns are from public product information as of November 2026 and may change. Product names belong to
> their owners.

## 📍 Roadmap

### ✅ Recently shipped

- [x] **0.4.2** Tuned design system: better menu icons, lighter first page, cleaner print.
- [x] **0.4.1** The redesigned interface with shaped loading and told-apart states.
- [x] **0.4.0** The Family role: the family's portal as configuration; a consent given or withdrawn as an action.
- [x] **0.3.0** Applications online with a guardian's consent; the family's portal; fee schedules; report cards.
- [x] **0.2.8** Names of records and fields in Arabic and Hindi.
- [x] **0.2.7** The assistant knows the records; dates of birth in a field group.

### 🔜 Next up

**Teaching**
- [ ] Academic terms within a year, with report cards and comments by term and course.
- [ ] A gradebook with weighted categories.
- [ ] Transcripts, credits and GPA.

**Scheduling**
- [ ] Room capacities enforced.
- [ ] Timetables made from rules and constraints.

**Families**
- [ ] Pay fees online on the portal.
- [ ] Alerts to guardians when a child is absent.
- [ ] Report cards as printed documents.

### 🔭 Later

**Records and reporting**
- [ ] Discipline, health and special-education records.
- [ ] State or national reporting formats as country add-ons.

**Learning**
- [ ] Send a student to Learn when the family has consented, with its courses and quizzes.

**Reach**
- [ ] Enquiries and recruitment before an application.
- [ ] SMS to families.
- [ ] Mobile use for teachers.
- [ ] Higher-education registration, degree audit and financial aid.

## 🚀 Install

Education is a native Ferrum app: the Ferrum server includes it (`runtime: native`) and a workspace turns it on from **App Manager**. It needs platform
`>=0.3.2, <0.4`. Installing it installs **ERP**, which holds customers, items and invoices.

After installing:

1. Give people the roles **Education Manager**, **Teacher** and **Admissions Officer**. Give the **Family** role to guardians who sign in to the portal.
2. Open **Education Settings** and set the **company** fees are invoiced from.
3. Create an **academic year**, **courses**, a **program** (tick *Applications Open Online* to take applications), a **grading scale**, **instructors** and **rooms**.
4. Create a **fee structure** for a program and year, then a **fee schedule** to make and invoice each student's fees.
5. Share the school's application page, `/school/apply/{workspace}`, with families.

## 🔧 Develop

```bash
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
FERRUM_EDUCATION_TEST_DATABASE_URL=postgres://ferrum:ferrum@localhost:5432/ferrum_education_test cargo test
```

The database must exist, be owned by a role that is not a superuser (superusers bypass row-level security) and be reset before the run.
`tests/education_it.rs` walks a term: an admission and fees paid through ERP; groups and a timetable with each kind of clash refused;
attendance and a graded assessment; and who may do what. Rust dependencies come from the Ferrum platform and Ferrum ERP repositories on `main`.

The interface lives in `ui/` and is built with Ferrum-Frontend. Check the Frontend out beside the repository as `../ferrum/web`, install its
packages, then:

```bash
cd ui
npm run build       # builds into ui/dist, which the server embeds
npm run check       # interface lint
npm run typecheck
npm test
```

CI (`.github/workflows/ci.yml`) runs Rust on PostgreSQL 17 and the interface checks.

## 📂 Project layout

```text
app.yaml                    manifest: id, roles, shell entries, public and portal pages, assistant notes
packages/education/         entities, roles, field groups and personal-data rules as YAML
src/admissions.rs           applicants: approve, reject, admit
src/fees.rs                 fee structures, fees, invoices, paid status
src/records.rs              fee schedules and report cards
src/timetable.rs            groups, classes, clashes, attendance
src/marks.rs                results, percent and grade
src/site.rs                 public applications and the family's portal endpoints
ui/                         the home, roll call, application page and family page; words in Arabic and Hindi
tests/education_it.rs       a term on PostgreSQL
docs/PROVENANCE.md          where the design comes from
```

## 📜 Changelog

Every release is described in [CHANGELOG.md](CHANGELOG.md).

## 🤝 Contributing

Issues and pull requests are welcome at [github.com/CBFerrum/ferrum-app-education](https://github.com/CBFerrum/ferrum-app-education/issues).
Keep the checks above green. The design is adapted from ERPNext 13's Education module; see [docs/PROVENANCE.md](docs/PROVENANCE.md).

## 📄 License

AGPL-3.0-only. See [LICENSE](LICENSE).

<div align="center">

<sub>Part of <a href="https://github.com/CBFerrum/Ferrum">Ferrum</a>, the open business platform · Made with 🦀 Rust and 🐘 PostgreSQL</sub>

</div>
