// Today at the school office: the day's classes in time order, applicants waiting for a decision,
// and fees invoiced and not yet paid. Each opens its record.

import { useQuery } from "@tanstack/react-query";
import { Link } from "react-router";
import { api, docPath } from "@ferrum/client";
import { ErrorState, PageLoading, useShellTitle } from "@ferrum/kit";

interface Row {
  id: string;
  [k: string]: unknown;
}

const rows = (entity: string, filters: unknown[], extra: Record<string, string | number> = {}) =>
  api.get<{ rows: Row[] }>(docPath(entity), { filters: JSON.stringify(filters), limit: 200, ...extra }).then((r) => r.rows);

/** Today in the browser's time zone, as YYYY-MM-DD. */
export const todayIso = (now = new Date()) => `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, "0")}-${String(now.getDate()).padStart(2, "0")}`;

export function Home() {
  useShellTitle("School");
  const today = todayIso();
  const q = useQuery({
    queryKey: ["school", "home", today],
    queryFn: async () => {
      const [classes, applicants, fees] = await Promise.all([
        rows("CourseSchedule", [["date", "eq", today]], { sort: "from_time asc" }),
        rows("StudentApplicant", [["status", "in", ["Applied", "Approved"]]]),
        rows("Fees", [["status", "eq", "Invoiced"]]),
      ]);
      return { classes, applicants, fees };
    },
  });
  if (q.isPending) return <PageLoading />;
  if (q.isError) return <ErrorState error={q.error} onRetry={() => void q.refetch()} />;
  const { classes, applicants, fees } = q.data;
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
                <Link to={`/e/CourseSchedule/${c.id}`}>
                  {String(c.from_time)} to {String(c.to_time)}
                </Link>
              </li>
            ))}
          </ul>
        )}
      </section>
      <div className="np-columns">
        <Link className="card card-pad stack" to="/e/StudentApplicant">
          <span className="muted">Applicants waiting</span>
          <strong style={{ fontSize: 28 }}>{applicants.length}</strong>
        </Link>
        <Link className="card card-pad stack" to="/e/Fees">
          <span className="muted">Fees invoiced, not yet paid</span>
          <strong style={{ fontSize: 28 }}>{fees.length}</strong>
        </Link>
      </div>
    </div>
  );
}
