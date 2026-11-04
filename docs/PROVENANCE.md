# Provenance

## ERPNext 13's Education module

- Source: [frappe/erpnext](https://github.com/frappe/erpnext), branch `version-13`, commit 37e00a6
  (13.55.2), `erpnext/education/doctype/`. That module later became frappe/education.
- Licence: GNU General Public License v3. This app is under the GNU Affero General Public License v3,
  which may be combined with GPL-3.0 code (section 13 of each licence).
- What was taken, as designs rather than code:
  - its doctypes Academic Year, Course, Program (with Program Course), Student Applicant, Student (with
    guardians), Program Enrollment, Fee Structure (with Fee Component), Fees, Instructor, Room,
    Student Group, Course Schedule, Student Attendance, Grading Scale, Assessment Plan and Assessment
    Result, trimmed to what a school's office needs first;
  - two rules: Course Schedule's check that a room, an instructor and a group are not double-booked,
    and the grading scale's highest-threshold band.
- Not taken: academic terms, assessment criteria, the creation tools, the LMS parts (Learn has its own),
  the website, and fee schedules.
- Its own: admitting into a student and an enrollment in one step, invoicing fees to the guardian
  through ERP, upkeep's payment check, and all the code.

The copy read is in `ERP/referance/industry/erpnext-v13` on the machine this was written on (sparse:
`erpnext/non_profit` and `erpnext/education`).
