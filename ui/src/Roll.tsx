// A class's roll call: its group's students, each present unless marked otherwise, saved at once.
// The server keeps one attendance a student for each class, and only for the class's own students.

import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { Link, useParams } from "react-router";
import { api, docPath, refusal, t, isolate } from "@ferrum/client";
import { Button, EmptyState, ErrorState, ListSkeleton, toast, useShellTitle } from "@ferrum/kit";
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
  useShellTitle(t("Attendance"));
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
  if (q.isPending) return <ListSkeleton className="np-page" />;
  if (q.isError) return <ErrorState error={q.error} onRetry={() => void q.refetch()} />;
  const { cls, students, kept } = q.data;
  const statusOf = (s: string) => chosen.get(s) ?? kept.get(s)?.status ?? "Present";
  return (
    <div className="np-page stack">
      <header className="row" style={{ justifyContent: "space-between" }}>
        <h2>
          <Link to={`/e/CourseSchedule/${id}`}>{isolate(classLine(cls))}</Link>
        </h2>
        <Button variant="primary" loading={save.isPending} disabled={!students.length} onClick={() => save.mutate()}>
          {t("Save attendance")}
        </Button>
      </header>
      {students.length === 0 ? (
        <EmptyState title={t("No students in this class's group")}>
          <Link to={`/e/StudentGroup/${String(cls.student_group)}`}>{t("Add them to the group")}</Link>
        </EmptyState>
      ) : (
        <table className="table">
          <thead>
            <tr>
              <th>{t("Student")}</th>
              <th>{t("Attendance")}</th>
            </tr>
          </thead>
          <tbody>
            {students.map((s) => (
              <tr key={s.id}>
                <td>{isolate(s.name)}</td>
                <td>
                  <select
                    className="select"
                    aria-label={t("Attendance of {name}", { name: s.name })}
                    value={statusOf(s.id)}
                    onChange={(e) => setChosen((m) => new Map(m).set(s.id, e.target.value))}
                  >
                    {STATUSES.map((v) => (
                      <option key={v}>{isolate(v)}</option>
                    ))}
                  </select>
                  {kept.has(s.id) ? null : <span className="faint"> {t("not taken yet")}</span>}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}
