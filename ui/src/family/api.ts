// Education's endpoints for families. Without signing in (`/api/site/{workspace}/education/…`): the
// programs taking applications, applying, and an application's status. Signed in to the portal
// (`/api/app/education/portal/…`): a guardian's children and the family's consents. The server
// holds every rule; these only carry the question.

import { api } from "@ferrum/client";

export interface Programs {
  programs: { code: string; name: string }[];
  years: { name: string; starts_on: string; ends_on: string }[];
}

export interface Application {
  number: string;
  applicant_name: string;
  program: string | null;
  academic_year: string | null;
  status: "Applied" | "Approved" | "Rejected" | "Admitted";
}

export interface Child {
  id: string;
  number: string | null;
  name: string;
  status: string | null;
  enrollments: { program: string; academic_year: string }[];
  attendance: { present: number; absent: number; late: number; excused: number } | null;
  fees: { number: string | null; structure: string; due_on: string; amount: string | null; status: string; invoice: string | null }[];
  report_cards: {
    number: string | null;
    academic_year: string | null;
    attendance_percent: string | null;
    comment: string | null;
    published_on: string | null;
    courses: { course: string | null; assessments: number; percent: string | null; grade: string | null }[];
  }[];
  consents: { consent: string; given: boolean; by: string | null; given_on: string | null }[];
}

const site = (tenant: string) => `/api/site/${encodeURIComponent(tenant)}/education`;
const PORTAL = "/api/app/education/portal";

export const programs = (tenant: string, signal?: AbortSignal) => api.get<Programs>(`${site(tenant)}/apply/programs`, undefined, signal);

/** An application and its link, or `{status: "received"}` for the field only a program fills in. */
export type Applied = { application: Application; token: string } | { status: "received" };

export const apply = (tenant: string, body: unknown) => api.post<Applied>(`${site(tenant)}/apply`, body);

export const status = (tenant: string, token: string, signal?: AbortSignal) =>
  api.get<{ application: Application }>(`${site(tenant)}/apply/status/${encodeURIComponent(token)}`, undefined, signal);

export const family = (signal?: AbortSignal) => api.get<{ children: Child[] }>(`${PORTAL}/family`, undefined, signal);

export const setConsent = (student: string, consent: string, given: boolean) =>
  api.post<{ message: string }>(`${PORTAL}/consents`, { student, consent, given });

/** Where an application's status is, under the application page. */
export const statusLink = (tenant: string, token: string) => `/apply/${encodeURIComponent(tenant)}/status/${encodeURIComponent(token)}`;

/** Present or late, of the classes taken; null before any. */
export function attended(a: Child["attendance"]): number | null {
  if (!a) return null;
  const all = a.present + a.absent + a.late + a.excused;
  return all ? Math.round(((a.present + a.late) * 10000) / all) / 100 : null;
}
