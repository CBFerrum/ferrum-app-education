// Today at the school office: the day's classes in time order, each named and with its roll call,
// applicants waiting for a decision, and fees invoiced and not yet paid. The day is the
// workspace's, as the server counts it. An office with nothing set up says where to start.

import { useQuery } from "@tanstack/react-query";
import { Link } from "react-router";
import { api, docPath, todayIso } from "@ferrum/client";
import { EmptyState, ErrorState, PageLoading, useShellTitle } from "@ferrum/kit";

export interface Row {
  id: string;
  [k: string]: unknown;
}

interface List {
  rows: Row[];
  total?: number;
}

const list = (entity: string, filters: unknown[], extra: Record<string, string | number | boolean> = {}) =>
  api.get<List>(docPath(entity), { filters: JSON.stringify(filters), limit: 200, ...extra });

/** How many match: the list's total, not the rows of one page. */
const count = (entity: string, filters: unknown[]) => list(entity, filters, { limit: 1, count: true, fields: "id" }).then((r) => r.total ?? r.rows.length);

/** A class as the office reads it: its time, course, group, room and instructor. */
export function classLine(c: Row): string {
  const t = (c._titles ?? {}) as Record<string, string>;
  return [`${String(c.from_time)} to ${String(c.to_time)}`, t.course, t.student_group, t.room, t.instructor].filter(Boolean).join(" · ");
}

export function Home() {
  useShellTitle("School");
  const today = todayIso();
  const q = useQuery({
    queryKey: ["school", "home", today],
    queryFn: async () => {
      const [classes, applicants, fees, programs] = await Promise.all([
        list("CourseSchedule", [["date", "eq", today]], { sort: "from_time asc", titles: true }).then((r) => r.rows),
        count("StudentApplicant", [["status", "in", ["Applied", "Approved"]]]),
        count("Fees", [["status", "eq", "Invoiced"]]),
        count("Program", []),
      ]);
      return { classes, applicants, fees, programs };
    },
  });
  if (q.isPending) return <PageLoading />;
  if (q.isError) return <ErrorState error={q.error} onRetry={() => void q.refetch()} />;
  const { classes, applicants, fees, programs } = q.data;
  if (!programs)
    return (
      <div className="np-page">
        <EmptyState title="Nothing set up yet" icon="calendar">
          <p className="muted">
            Set the company fees are invoiced from in <Link to="/e/EducationSettings">Education Settings</Link>, then add an{" "}
            <Link to="/e/AcademicYear/new">academic year</Link> and a <Link to="/e/Program/new">program</Link> with its courses. Applicants, students,
            fees and the timetable follow from them.
          </p>
        </EmptyState>
      </div>
    );
  return (
    <div className="np-page">
      <section className="card card-pad stack" aria-label="Today's classes">
        <h3>Today's classes ({classes.length})</h3>
        {classes.length === 0 ? (
          <p className="muted">None today.</p>
        ) : (
          <ul className="stack">
            {classes.map((c) => (
              <li key={c.id}>
                <Link to={`/e/CourseSchedule/${c.id}`}>{classLine(c)}</Link> · <Link to={`/class/${c.id}`}>Take attendance</Link>
              </li>
            ))}
          </ul>
        )}
      </section>
      <div className="np-columns">
        <Link className="card card-pad stack" to="/e/StudentApplicant">
          <span className="muted">Applicants waiting</span>
          <strong style={{ fontSize: 28 }}>{applicants}</strong>
        </Link>
        <Link className="card card-pad stack" to="/e/Fees">
          <span className="muted">Fees invoiced, not yet paid</span>
          <strong style={{ fontSize: 28 }}>{fees}</strong>
        </Link>
      </div>
    </div>
  );
}
