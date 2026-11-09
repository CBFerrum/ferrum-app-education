// The school's application page, /school/apply/{workspace}: a family applies to a program taking
// applications, for a year still to come, with a guardian to reach and their consent to the school
// keeping the details. They are given the application's own link. Every rule is the server's; a
// refusal says why.

import { useMutation, useQuery } from "@tanstack/react-query";
import { useState } from "react";
import { Link, useParams } from "react-router";
import { ApiError, bootstrap, refusal, t, isolate } from "@ferrum/client";
import { Alert, Badge, Button, EmptyState, ErrorState, Field, PageLoading } from "@ferrum/kit";
import { type Applied, apply, programs, status, statusLink } from "./api";

function Page({ children }: { children: React.ReactNode }) {
  return (
    <div className="ap-page">
      <a className="skip-link" href="#main">
        {t("Skip to content")}
      </a>
      <header className="ap-top">
        <span className="ap-brand">{isolate(bootstrap()?.workspace.name ?? t("Apply to the school"))}</span>
      </header>
      <main id="main" className="ap-main" tabIndex={-1}>
        {children}
      </main>
    </div>
  );
}

function Closed({ error, onRetry }: { error: unknown; onRetry?: () => void }) {
  if (error instanceof ApiError && error.status === 404)
    return (
      <EmptyState title={t("Applications are not open here")} icon="graduation-cap">
        <p className="muted">{t("The school is not taking applications online now, or the address is not quite right.")}</p>
      </EmptyState>
    );
  return <ErrorState error={error} onRetry={onRetry} />;
}

export function ApplyPage() {
  const { tenant = "" } = useParams();
  const q = useQuery({ queryKey: ["school-apply", tenant], queryFn: ({ signal }) => programs(tenant, signal) });
  const [v, setV] = useState({
    applicant_name: "",
    date_of_birth: "",
    email: "",
    program: "",
    academic_year: "",
    guardian_name: "",
    guardian_relation: "Guardian",
    guardian_email: "",
    guardian_phone: "",
    consent: false,
    website: "",
  });
  const [done, setDone] = useState<Applied | null>(null);
  const send = useMutation({ mutationFn: () => apply(tenant, v), onSuccess: setDone });
  if (q.isPending)
    return (
      <Page>
        <PageLoading />
      </Page>
    );
  if (q.isError)
    return (
      <Page>
        <Closed error={q.error} onRetry={() => void q.refetch()} />
      </Page>
    );
  if (done)
    return (
      <Page>
        {"application" in done ? (
          <section className="card card-pad stack" aria-label={t("Applied")}>
            <h1>{t("Applied: {number}", { number: done.application.number })}</h1>
            <p>{t("The school writes to the guardian about it. Keep its link: it shows where the application stands.")}</p>
            <Link to={statusLink(tenant, done.token)}>{t("Your application")}</Link>
          </section>
        ) : (
          <Alert tone="success">{t("Thank you: your application is in.")}</Alert>
        )}
      </Page>
    );
  const program = v.program || q.data.programs[0]?.code || "";
  const year = v.academic_year || q.data.years[0]?.name || "";
  return (
    <Page>
      <form
        className="stack"
        aria-label={t("Apply to the school")}
        onSubmit={(e) => {
          e.preventDefault();
          send.mutate();
        }}
      >
        <h1>{t("Apply to the school")}</h1>
        <section className="card card-pad stack" aria-label={t("The applicant")}>
          <Field label={t("Program")} required>
            <select className="select" value={program} onChange={(e) => setV({ ...v, program: e.target.value })}>
              {q.data.programs.map((p) => (
                <option key={p.code} value={p.code}>
                  {p.name}
                </option>
              ))}
            </select>
          </Field>
          <Field label={t("Academic year")} required>
            <select className="select" value={year} onChange={(e) => setV({ ...v, academic_year: e.target.value })}>
              {q.data.years.map((y) => (
                <option key={y.name} value={y.name}>
                  {y.name}
                </option>
              ))}
            </select>
          </Field>
          <Field label={t("Applicant's name")} required>
            <input className="input" required maxLength={200} value={v.applicant_name} onChange={(e) => setV({ ...v, applicant_name: e.target.value })} />
          </Field>
          <Field label={t("Date of birth")}>
            <input className="input" type="date" value={v.date_of_birth} onChange={(e) => setV({ ...v, date_of_birth: e.target.value })} />
          </Field>
        </section>
        <section className="card card-pad stack" aria-label={t("The guardian")}>
          <Field label={t("Guardian's name")} required>
            <input className="input" autoComplete="name" required maxLength={200} value={v.guardian_name} onChange={(e) => setV({ ...v, guardian_name: e.target.value })} />
          </Field>
          <Field label={t("Relation")}>
            <select className="select" value={v.guardian_relation} onChange={(e) => setV({ ...v, guardian_relation: e.target.value })}>
              <option value="Mother">{t("Mother")}</option>
              <option value="Father">{t("Father")}</option>
              <option value="Guardian">{t("Guardian")}</option>
              <option value="Other">{t("Other")}</option>
            </select>
          </Field>
          <Field label={t("Guardian's email")} required hint={t("Where the school writes about the application")}>
            <input className="input" type="email" autoComplete="email" required maxLength={254} value={v.guardian_email} onChange={(e) => setV({ ...v, guardian_email: e.target.value })} />
          </Field>
          <Field label={t("Guardian's phone")}>
            <input className="input" type="tel" autoComplete="tel" maxLength={40} value={v.guardian_phone} onChange={(e) => setV({ ...v, guardian_phone: e.target.value })} />
          </Field>
          <label className="row">
            <input type="checkbox" checked={v.consent} onChange={(e) => setV({ ...v, consent: e.target.checked })} />
            {t("I agree to the school keeping and using these details for the admission.")}
          </label>
        </section>
        <input className="ap-trap" tabIndex={-1} autoComplete="off" aria-hidden="true" name="website" value={v.website} onChange={(e) => setV({ ...v, website: e.target.value })} />
        {send.isError && <Alert tone="error">{refusal(send.error)}</Alert>}
        <div>
          <Button type="submit" variant="primary" loading={send.isPending} onClick={() => setV({ ...v, program, academic_year: year })}>
            {t("Apply")}
          </Button>
        </div>
      </form>
    </Page>
  );
}

export function StatusPage() {
  const { tenant = "", token = "" } = useParams();
  const q = useQuery({ queryKey: ["school-application", tenant, token], queryFn: ({ signal }) => status(tenant, token, signal) });
  if (q.isPending)
    return (
      <Page>
        <PageLoading />
      </Page>
    );
  if (q.isError)
    return (
      <Page>
        {q.error instanceof ApiError && q.error.status === 404 ? (
          <EmptyState title={t("This link does not work")} icon="graduation-cap">
            <p className="muted">{t("It opens only the application it was given for.")}</p>
          </EmptyState>
        ) : (
          <ErrorState error={q.error} onRetry={() => void q.refetch()} />
        )}
      </Page>
    );
  const a = q.data.application;
  return (
    <Page>
      <section className="card card-pad stack" aria-label={t("Your application")}>
        <div className="row" style={{ justifyContent: "space-between" }}>
          <h1>{isolate(a.number)}</h1>
          <Badge tone={a.status === "Admitted" ? "green" : a.status === "Rejected" ? "red" : "amber"}>{isolate(a.status)}</Badge>
        </div>
        <p>{t("{name}, {program}, {year}.", { name: a.applicant_name, program: a.program ?? "", year: a.academic_year ?? "" })}</p>
      </section>
    </Page>
  );
}
