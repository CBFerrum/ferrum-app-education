# Changelog

## 0.4.2

Rebuilt with Ferrum-Frontend 45dad7f, the design system as tuned after the landed build was reviewed. The left menu's icons are chosen so that none repeats on more than three rows and each fits its row; their drawings load beside the first page instead of inside it, so the page starts lighter; a row that scrolls sideways fades at the edge where there is more, in every browser and with reduced motion; a printed record leaves out empty fields and keeps its totals with the table; menus and popovers sit inside the page's landmarks for assistive technology; a selection bar folds what does not fit into More on a narrow screen. Nothing else changes.

## 0.4.1

The interface follows the upgraded design system, rebuilt with Ferrum-Frontend 0a47157. Text uses the kit's type roles (nothing under 12px) and controls its height tokens; loading shows skeletons shaped like what is coming; empty, no matches, denied, offline and failed are told apart; status colours come from stored state ids. The interface check now runs the UI lint against an empty baseline. The home is rebuilt on the kit's work-home pattern: one page title, the main action as a button, queues with their counts, status at its natural width; its other own pages follow the same anatomy.

## 0.4.0

The family's portal as configuration too (Ferrum's builder-first gate BF1, docs/builder/portal.md).

- **Family**, a portal role the package ships: a guardian's children, reached through each child's guardians by the address they sign in with (`via: guardians.email`, `via: student.guardians.email`), with their enrollments, attendance, fees and published report cards, on the portal's generic Your records page.
- **Give or Withdraw a Consent**, a student's action: the family's consent given or withdrawn as on the family page, recorded with who did it. The Family role names it, so a guardian runs it without editing the student; the office runs it too.
- Needs Ferrum with portal roles (BF1).

## 0.3.1

Its summary within the 200 characters the platform takes (0.3.0's was longer, and Ferrum's compiled apps refused it). Nothing else changes.

## 0.3.0

Admission, family consent, fee schedules and academic reporting with guardians' access (FEO-047).

- **Applications online.** Program's Applications Open Online puts it on the public page (/school/apply/{workspace}, `shell.public`): a family applies for a year not yet over, with a guardian's name and email and their consent to the school keeping the details (Guardian's Consent and Consent Given On are kept; Source is Online). The family gets the application's own link (only its SHA-256 kept) showing its status. Writes are limited per address; a hidden field catches programs. A guardian's relation reaches the student.
- **The family's portal** (`shell.portal` family, /portal/family/): for a guardian signed in with an address the school lists, each child's enrollment, this year's attendance, fees (issued invoices open on ERP's "Your invoices"), published report cards, and consents (Photos and Media, School Trips, Sharing With Learn), given and withdrawn there with who and when kept on the student; the office does not rewrite them.
- **Fee Schedule**: a structure due on a day for every enrolled active student (or one group of them); Make the Fees makes each student's fees once, Invoice Them invoices those still drafts.
- **Report Card**: a student's year, each course's results averaged and graded on its latest assessment's scale, attendance counted (present or late, of the classes taken), a comment; worked out again while a draft, kept once published, one a year.
- Names and words in Arabic and Hindi for the new fields and pages.

## 0.2.8

Rebuilt with Ferrum-Frontend c59c9a2: the names of its models, fields and manifest in Arabic and Hindi (127 each in ui/src/locales/names, which the platform serves to every page of the workspace, so a record's labels, the launcher and the bar read in the person's language), and the sentences its pages still showed in English (split around a link or a figure, built in code from a template or word by word) whole and translated (23 words each in ui/src/locales). Translated by the model, to be reviewed by native speakers.

## 0.2.7

The assistant knows what its records mean in its pages (contributes.shell.assistant): their states, what counts and where figures come from, with questions it suggests asking there. Personal and health details are in field groups: StudentApplicant.date_of_birth (personal), Student.date_of_birth (personal). Every role that read or wrote them is granted the group, so people see what they saw, and an AI provider gets them only where the workspace's egress policy allows the group.

## 0.2.6

Rebuilt with Ferrum-Frontend 5b22e7b: its own words carry built-in Arabic and Hindi (25 words each, ui/src/locales, translated by the model and to be reviewed by native speakers), fetched for the page's language only. The labels it keeps in objects, its dialogs, its page titles and the words it keeps in variables go through t() as well, and the names and sizes it shows as they are keep their order right to left. Its pages are drawn once their words are in, and its entry carries only the parts of the kit it uses.

## 0.2.5

Rebuilt with Ferrum-Frontend 6537d65: an action's link, code or token shown once is kept in a dialog to copy, and on a page that reads right to left, English left untranslated keeps its own order. Its own words go through the kit's t() (21 places), so a workspace's own words for its language reach them, and Admin, Interface words lists them under it (dist/words.json).

## 0.2.4

Rebuilt with Ferrum-Frontend d5d98f2: a workspace's own words for its interface (Admin, Interface words) reach its pages as they load; an attachment field that holds a picture shows it; and a value another record already has is said so, where the form said the record had been changed by someone else.

## 0.2.3

Rebuilt with Ferrum-Frontend 63280d3: right to left, its pages mirror as they read (icons that point along the line, margins and alignment by the line's start and end, arrow keys along it), and the records pages' words are in Arabic and Hindi.

## 0.2.2

Rebuilt with Ferrum-Frontend a64416a: a new record's box whose default is 1 shows ticked, as it is saved.

## 0.2.1

Rebuilt with Ferrum-Frontend 922fcfb: an action's dialog says every refusal, the errors on what the action made (another record's rows) among them, and an action with parameters and a confirmation asks once.

## 0.2.0

What walking a school day through the pages found:
- Attendance was one record per student, made one at a time. Each class now has a roll call, at /school/class/{class}, opened from today's classes. It lists the class's group, each student present unless marked otherwise, and saves the new and changed marks at once.
- Today's classes were listed by their times alone. Each now shows its course, group, room and instructor.
- Today was the browser's day, not the workspace's.
- The counts of applicants waiting and fees unpaid stopped at 200. They are now counted.
- An office with nothing set up showed empty lists. It now says to set the company, then a year and a program.
- A score over the maximum was refused with "is between 0 and the assessment's maximum, 50.000000000". It now reads "must be between 0 and the assessment's maximum, 50".

## 0.1.8

Rebuilt with Ferrum-Frontend a76295e: its pages show times on a person's own clock when they chose one on their Account page, while today and due dates stay on the workspace's day.

## 0.1.7

Rebuilt with Ferrum-Frontend 24c3951: its pages show times on the workspace's clock, naming the zone when it is not the browser's, a draft numbered on submit is headed as a draft, and a record of an app that is not installed says so.

## 0.1.6

Rebuilt with Ferrum-Frontend bb60115: a record page reads an empty number or check as the server does, so an action whose condition needs one shows, and a workspace's settings open as their one record.

## 0.1.5

An applicant's Reject asks for the reason it keeps in their notes: it read one and never asked, so from the applicant's page it was never kept.

## 0.1.4

Rebuilt with Ferrum-Frontend 00dea07: in a workflow state that names what may change, the other fields show read-only, and the fields the state needs show required.

## 0.1.3

Personal-data requests list the student, student applicant records holding the person's name apart, for an administrator to judge (never erased for the name alone) (Ferrum's FERRUM-PRIVACY-002).

## 0.1.2

Rebuilt with Ferrum-Frontend daeadda: after an action runs, a record page shows what the action said it did rather than only that it is done.

## 0.1.1

Rebuilt with Ferrum-Frontend 80ff179: a field's label names its control for assistive technology, and a record whose title is a link shows that record's title, not its id.

## 0.1.0

A school's office (FERRUM-INDUSTRY-004).

- Academic years, courses and programs; applicants approved, rejected or admitted into students
  with their guardians and enrollments.
- Fee structures and fees invoiced through ERP to the guardian; upkeep marks them paid.
- Instructors, rooms, student groups of enrolled students, and a timetable with no room, instructor
  or group in two classes at once.
- Attendance for a class's own students; assessments graded on a scale.
- Its own interface at `/school/`: today's classes, applicants waiting and fees owed.
