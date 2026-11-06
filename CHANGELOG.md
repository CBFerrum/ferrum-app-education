# Changelog

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
