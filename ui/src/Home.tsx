// Today at the school office: the day's classes in time order, each named and with its roll call,
// applicants waiting for a decision, and fees invoiced and not yet paid. The day is the
// workspace's, as the server counts it. An office with nothing set up says where to start.

import { useQuery } from "@tanstack/react-query";
import { GraduationCap } from "lucide-react";
import { Link } from "react-router";
import { api, docPath, formatValue, todayIso, t as tr, isolate } from "@ferrum/client";
import { ButtonLink, HomeSkeleton, LoadFailed, PageHead, placed, Queue, StatePanel, Tile, TileGrid } from "@ferrum/kit";

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

/** The class's own row: its time range and course (the link to the class), then its group, room and instructor, and the
 *  one step the office takes from it. */
export function classRow(c: Row): { title: string; meta: string } {
  const t = (c._titles ?? {}) as Record<string, string>;
  return { title: `${String(c.from_time)}–${String(c.to_time)} · ${t.course ?? ""}`.replace(/ · $/, ""), meta: [t.student_group, t.room, t.instructor].filter(Boolean).join(" · ") };
}

/** A count as a figure: tabular, and bold only when there is something to do (a zero is quiet). */
const figure = (n: number) => (n ? <strong className="num-inline">{n}</strong> : <span className="num-inline">{n}</span>);

export function Home() {
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
  const head = (description?: string) => <PageHead title={tr("School")} description={description} />;
  if (q.isPending)
    return (
      <>
        {head()}
        <HomeSkeleton queues={1} />
      </>
    );
  if (q.isError)
    return (
      <>
        {head()}
        <LoadFailed error={q.error} onRetry={() => void q.refetch()} />
      </>
    );
  const { classes, applicants, fees, programs } = q.data;
  if (!programs)
    return (
      <>
        {head()}
        <StatePanel kind="empty" title={tr("Nothing set up yet")} icon={GraduationCap}>
          {placed(tr("Set the company fees are invoiced from in {p1}, then add an {p2} and a {p3} with its courses. Applicants, students, fees and the timetable follow from them."), { p1: <Link to="/e/EducationSettings">{tr("Education Settings")}</Link>, p2: <Link to="/e/AcademicYear/new">{tr("academic year")}</Link>, p3: <Link to="/e/Program/new">{tr("program")}</Link> })}
        </StatePanel>
      </>
    );
  return (
    <>
      {head(tr("{day} · {n} classes today", { day: formatValue({ type: "date" }, today), n: classes.length }))}
      <TileGrid className="edu-figures" label={tr("Waiting for the office")}>
        <Tile to="/e/StudentApplicant" title={tr("Applicants waiting")} meta={figure(applicants)} />
        <Tile to="/e/Fees" title={tr("Fees invoiced, not yet paid")} meta={figure(fees)} />
      </TileGrid>
      <div className="edu-today">
      <Queue title={tr("Today's classes")} count={classes.length} empty={tr("None today.")}>
        {classes.map((c) => {
          const row = classRow(c);
          return (
            <li key={c.id} className="queue-item">
              <div className="queue-text">
                <Link className="queue-name edu-time" to={`/e/CourseSchedule/${c.id}`} title={row.title}>
                  {isolate(row.title)}
                </Link>
                {row.meta && <span className="queue-meta">{isolate(row.meta)}</span>}
              </div>
              <span className="edu-act">
                <ButtonLink to={`/class/${c.id}`}>{tr("Take attendance")}</ButtonLink>
              </span>
            </li>
          );
        })}
      </Queue>
      </div>
    </>
  );
}
