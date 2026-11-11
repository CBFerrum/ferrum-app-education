// The family's page of the workspace's portal, at /portal/family/: each child whose guardians list
// the address the person signed in with. Their enrollment, this year's attendance, their fees (an
// issued invoice opens on ERP's "Your invoices"), their published report cards, and the consents
// the family gives or withdraws here.

import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { refusal, t, isolate } from "@ferrum/client";
import { GraduationCap } from "lucide-react";
import { HomeSkeleton, LoadFailed, PageHead, StatePanel, StatusBadge, toast } from "@ferrum/kit";
import { attended, type Child, family, setConsent } from "./api";

// Student.status and Fees.status as stored (packages/education/entities).
const STUDENT_TONES = { Active: "green", Graduated: "blue", Left: "neutral" } as const;
const FEE_TONES = { Draft: "neutral", Invoiced: "amber", Paid: "green", Cancelled: "neutral" } as const;

export function FamilyPage() {
  const q = useQuery({ queryKey: ["school-family"], queryFn: ({ signal }) => family(signal) });
  const head = <PageHead title={t("Your children's school")} description={t("Attendance, fees, report cards and what the family agreed to.")} />;
  if (q.error)
    return (
      <div className="ap-portal">
        {head}
        <LoadFailed error={q.error} onRetry={() => void q.refetch()} />
      </div>
    );
  if (!q.data)
    return (
      <div className="ap-portal">
        <PageHead title={t("Your children's school")} />
        <HomeSkeleton queues={1} />
      </div>
    );
  return (
    <div className="stack ap-portal">
      {head}
      {q.data.children.length === 0 ? (
        <StatePanel kind="empty" title={t("No child here yet")} icon={GraduationCap}>
          {t("Once the school lists you as a guardian with this address, your children are here.")}
        </StatePanel>
      ) : (
        q.data.children.map((c) => <ChildCard key={c.id} child={c} />)
      )}
    </div>
  );
}

function ChildCard({ child }: { child: Child }) {
  const qc = useQueryClient();
  const change = useMutation({
    mutationFn: ({ consent, given }: { consent: string; given: boolean }) => setConsent(child.id, consent, given),
    onSuccess: (r) => {
      toast(r.message);
      void qc.invalidateQueries({ queryKey: ["school-family"] });
    },
    onError: (e) => toast(refusal(e), "error"),
  });
  const percent = attended(child.attendance);
  return (
    <section className="card card-pad stack" aria-label={child.name}>
      <header className="ap-child">
        <h2>{isolate(child.name)}</h2>
        {child.status && <StatusBadge value={child.status} tones={STUDENT_TONES} />}
      </header>
      {child.enrollments[0] && <p className="faint">{isolate(`${child.enrollments[0].program}, ${child.enrollments[0].academic_year}`)}</p>}
      <div>
        <h3>{t("Attendance this year")}</h3>
        {percent === null ? (
          <p className="muted">{t("No classes taken yet.")}</p>
        ) : (
          <p>{t("{percent}% of {classes} classes: {present} present, {late} late, {absent} absent, {excused} excused", {
            percent,
            classes: child.attendance!.present + child.attendance!.absent + child.attendance!.late + child.attendance!.excused,
            present: child.attendance!.present,
            late: child.attendance!.late,
            absent: child.attendance!.absent,
            excused: child.attendance!.excused,
          })}</p>
        )}
      </div>
      <div>
        <h3>{t("Fees")}</h3>
        {child.fees.length === 0 ? (
          <p className="muted">{t("No fees yet.")}</p>
        ) : (
          <ul className="ap-list">
            {child.fees.map((f, i) => (
              <li key={i} className="ap-fee">
                <span className="ap-fee-what">{f.invoice ? <a className="ap-link" href={`/portal/invoices/${f.invoice}`}>{isolate(f.structure)}</a> : isolate(f.structure)}</span>
                <span className="num-inline">{isolate(f.amount ?? "")}</span>
                <span className="muted">{t("due {due}", { due: f.due_on })}</span>
                <StatusBadge value={f.status} tones={FEE_TONES} />
              </li>
            ))}
          </ul>
        )}
      </div>
      <div>
        <h3>{t("Report cards")}</h3>
        {child.report_cards.length === 0 ? (
          <p className="muted">{t("None published yet.")}</p>
        ) : (
          child.report_cards.map((r, i) => (
            <div key={i} className="stack">
              <strong>{isolate(r.academic_year ?? "")}</strong>
              <table className="table">
                <thead>
                  <tr>
                    <th scope="col">{t("Course")}</th>
                    <th scope="col">{t("Average")}</th>
                    <th scope="col">{t("Grade")}</th>
                  </tr>
                </thead>
                <tbody>
                  {r.courses.map((c, j) => (
                    <tr key={j}>
                      <td>{isolate(c.course ?? "")}</td>
                      <td>{isolate(`${c.percent ?? ""}%`)}</td>
                      <td>{isolate(c.grade ?? "")}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
              <p className="faint">{t("Attendance {percent}%", { percent: r.attendance_percent ?? "0" })}</p>
              {r.comment && <p>{isolate(r.comment)}</p>}
            </div>
          ))
        )}
      </div>
      <fieldset className="stack">
        <legend>{t("What the family agrees to")}</legend>
        {child.consents.map((c) => (
          <label key={c.consent} className="ap-consent">
            <input type="checkbox" checked={c.given} disabled={change.isPending} onChange={(e) => change.mutate({ consent: c.consent, given: e.target.checked })} />
            {isolate(c.consent)}
            {c.by && <span className="faint">{t("({by}, {when})", { by: c.by, when: (c.given_on ?? "").slice(0, 10) })}</span>}
          </label>
        ))}
      </fieldset>
    </section>
  );
}
