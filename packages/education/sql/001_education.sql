-- Education: the database's own guard against clashes in the timetable. No room, instructor or
-- group is in two classes at once on a day: from_time to to_time, half-open, so 09:00-10:00 and
-- 10:00-11:00 do not clash. The times are kept as HH:MM, read here as the number HHMM, which
-- orders as the times do. The schedule hook refuses a clash first, naming the other class's
-- times; these hold when two are saved at once (src/lib.rs registers their words).
alter table e_course_schedule add constraint education_room_free
  exclude using gist (tenant_id with =, room with =, "date" with =,
                      int4range(replace(from_time, ':', '')::int, replace(to_time, ':', '')::int) with &&);

alter table e_course_schedule add constraint education_instructor_free
  exclude using gist (tenant_id with =, instructor with =, "date" with =,
                      int4range(replace(from_time, ':', '')::int, replace(to_time, ':', '')::int) with &&);

alter table e_course_schedule add constraint education_group_free
  exclude using gist (tenant_id with =, student_group with =, "date" with =,
                      int4range(replace(from_time, ':', '')::int, replace(to_time, ':', '')::int) with &&);
