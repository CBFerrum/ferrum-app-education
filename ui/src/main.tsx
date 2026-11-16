// Education's own interface at /school/: today's classes, applicants waiting, fees owed, a class's
// roll call, with every record in the records frame's lists and forms.

import { useEffect } from "react";
import { Link, Outlet, useRouteError } from "react-router";
import { bootstrap, shell, t } from "@ferrum/client";
import { EmptyState, ErrorState } from "@ferrum/kit";
import { mountApp, RequireAuth, setFrameMode } from "@ferrum/records/frame";
import "./styles.css";

const page = (name: "FormView" | "ListView" | "ReportView") => () => import("./records").then((m) => ({ Component: m[name] }));

function RouteError() {
  return <ErrorState error={useRouteError()} onRetry={() => window.location.reload()} />;
}

function NotFound() {
  return (
    <EmptyState title={t("Page not found")}>
      <Link to="/">{t("Go to the school office")}</Link>
    </EmptyState>
  );
}

function Started() {
  useEffect(() => shell().ready(), []);
  return null;
}

/** A page outside the frame: started, and drawn below. */
function Bare() {
  useEffect(() => shell().ready(), []);
  return <Outlet />;
}

/** The family's page of the portal, when the platform serves this release there. */
const FAMILY = "portal/family";
const onPortal = bootstrap()?.app?.entry === FAMILY;

if (!onPortal) setFrameMode("app", ["education"]);

const familyRoutes = [
  {
    path: "/",
    element: (
      <main>
        <Bare />
      </main>
    ),
    errorElement: <RouteError />,
    children: [{ index: true, lazy: () => import("./family/Family").then((m) => ({ Component: m.FamilyPage })) }],
  },
];

const staffRoutes = [
  // For families applying: no sign-in, no frame.
  { path: "/apply/:tenant", element: <Bare />, errorElement: <RouteError />, children: [{ index: true, lazy: () => import("./family/Apply").then((m) => ({ Component: m.ApplyPage })) }] },
  {
    path: "/apply/:tenant/status/:token",
    element: <Bare />,
    errorElement: <RouteError />,
    children: [{ index: true, lazy: () => import("./family/Apply").then((m) => ({ Component: m.StatusPage })) }],
  },
  { path: "/apply/*", element: <EmptyState title={t("Page not found")} icon="graduation-cap" /> },
    {
      path: "/",
      element: (
        <>
          <Started />
          <RequireAuth />
        </>
      ),
      errorElement: <RouteError />,
      children: [
        { index: true, lazy: () => import("./Home").then((m) => ({ Component: m.Home })) },
        { path: "class/:id", lazy: () => import("./Roll").then((m) => ({ Component: m.Roll })) },
        { path: "e/:entity", lazy: page("ListView") },
        { path: "e/:entity/:id", lazy: page("FormView") },
        { path: "r/:name", lazy: page("ReportView") },
        { path: "*", element: <NotFound /> },
      ],
    },
];

mountApp({
  catalogs: { ar: () => import("./locales/ar.json"), hi: () => import("./locales/hi.json") },
  ...(onPortal ? { route: FAMILY, routes: familyRoutes } : { route: "school", routes: staffRoutes }),
});
