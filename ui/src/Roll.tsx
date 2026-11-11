// A class's roll call: its group's students, each present unless marked otherwise, saved at once.
// The server keeps one attendance a student for each class, and only for the class's own students.

import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { GraduationCap } from "lucide-react";
import { Link, useParams } from "react-router";
import { api, docPath, refusal, t, isolate } from "@ferrum/client";
import { Button, ButtonLink, HomeSkeleton, LoadFailed, PageHead, StatePanel, toast } from "@ferrum/kit";
import { classLine, type Row } from "./Home";

export const STATUSES = ["Present", "Absent", "Late", "Excused"] as const;

/** What to save: a new attendance for each student without one, and the ones whose status changed. */
export function rollChanges(students: string[], kept: Map<string, { id: string; status: string }>, chosen: Map<string, string>) {
  return students.flatMap((s) => {
    const status = chosen.get(s) ?? kept.get(s)?.status ?? "Present";
    const was = kept.get(s);
    if (was && was.status === status) return [];
    return [{ student: s, id: was?.id, status }];
  });
}

export function Roll() {
  const { id = "" } = useParams();
  const qc = useQueryClient();
  const [chosen, setChosen] = useState<Map<string, string>>(new Map());
  const q = useQuery({
    queryKey: ["school", "roll", id],
    queryFn: async () => {
      const cls = await api.get<Row>(docPath("CourseSchedule", id), { titles: true });
      const group = await api.get<Row>(docPath("StudentGroup", String(cls.student_group)), { titles: true });
      const marks = await api.get<{ rows: Row[] }>(docPath("StudentAttendance"), { filters: JSON.stringify([["course_schedule", "eq", id]]), limit: 500 });
      const students = ((group.students ?? []) as Row[]).map((r) => ({ id: String(r.student), name: String((r._titles as Record<string, string> | undefined)?.student ?? r.student) }));
      const kept = new Map(marks.rows.map((m) => [String(m.student), { id: m.id, status: String(m.status) }]));
      return { cls, students, kept };
    },
  });
  const save = useMutation({
    mutationFn: async () => {
      const d = q.data!;
      const changes = rollChanges(
        d.students.map((s) => s.id),
        d.kept,
        chosen,
      );
      for (const c of changes) {
        if (c.id) await api.put(docPath("StudentAttendance", c.id), { status: c.status });
        else await api.post(docPath("StudentAttendance"), { course_schedule: id, student: c.student, status: c.status });
      }
      return changes.length;
    },
    onSuccess: (n) => {
      toast(n ? (n === 1 ? t("Attendance saved for {n} student", { n }) : t("Attendance saved for {n} students", { n })) : t("Nothing changed"));
      setChosen(new Map());
      void qc.invalidateQueries({ queryKey: ["school"] });
    },
    onError: (e) => toast(refusal(e), "error"),
  });
  const back = { to: "/", label: t("School") };
  if (q.isPending)
    return (
      <>
        <PageHead title={t("Attendance")} back={back} />
        <HomeSkeleton queues={1} />
      </>
    );
  if (q.isError)
    return (
      <>
        <PageHead title={t("Attendance")} back={back} />
        <LoadFailed error={q.error} onRetry={() => void q.refetch()} />
      </>
    );
  const { cls, students, kept } = q.data;
  // What a row shows is what the person marked, else what the server has, else Present: a mark the server has not taken is told so.
  const statusOf = (s: string) => chosen.get(s) ?? kept.get(s)?.status ?? "Present";
  const unsaved = (s: string) => chosen.has(s) && chosen.get(s) !== (kept.get(s)?.status ?? "Present");
  const saveButton = (
    <Button variant="primary" loading={save.isPending} disabled={!students.length} onClick={() => save.mutate()}>
      {t("Save attendance")}
    </Button>
  );
  return (
    <>
      <PageHead
        title={t("Attendance")}
        description={
          <Link className="edu-roll-class" to={`/e/CourseSchedule/${id}`} title={classLine(cls)}>
            {isolate(classLine(cls))}
          </Link>
        }
        back={back}
        actions={saveButton}
      />
      {students.length === 0 ? (
        <StatePanel
          kind="empty"
          title={t("No students in this class's group")}
          icon={GraduationCap}
          action={
            <ButtonLink to={`/e/StudentGroup/${String(cls.student_group)}`} variant="primary">
              {t("Add them to the group")}
            </ButtonLink>
          }
        />
      ) : (
        <>
          <ul className="edu-roll" aria-label={t("Student")}>
            {students.map((s) => (
              <li key={s.id} className="edu-roll-row">
                <span className="edu-roll-name">
                  {isolate(s.name)}
                  {unsaved(s.id) ? <span className="edu-roll-note">{t("Not saved yet")}</span> : kept.has(s.id) ? null : <span className="edu-roll-note">{t("not taken yet")}</span>}
                </span>
                <span className="edu-marks" role="group" aria-label={t("Attendance of {name}", { name: s.name })}>
                  {STATUSES.map((v) => (
                    <Button key={v} variant={statusOf(s.id) === v ? "primary" : "default"} aria-pressed={statusOf(s.id) === v} onClick={() => setChosen((m) => new Map(m).set(s.id, v))}>
                      {isolate(v)}
                    </Button>
                  ))}
                </span>
              </li>
            ))}
          </ul>
          <div className="edu-roll-foot">{saveButton}</div>
        </>
      )}
    </>
  );
}
