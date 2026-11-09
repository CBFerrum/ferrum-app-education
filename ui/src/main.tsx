// Education's own interface at /school/: today's classes, applicants waiting, fees owed, a class's
// roll call, with every record in the records frame's lists and forms.

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { StrictMode, useEffect } from "react";
import { createRoot } from "react-dom/client";
import { createBrowserRouter, Link, Outlet, RouterProvider, useRouteError } from "react-router";
import { addCatalogs, bootstrap, routeBase, shell, t, wordsReady } from "@ferrum/client";
import { ConfirmHost, EmptyState, ErrorState, Toaster } from "@ferrum/kit";
import { RequireAuth, setFrameMode } from "@ferrum/records/frame";
import "./styles.css";

// Its own words in the page's language (src/locales), fetched as the page starts; it is drawn once they are in.
void addCatalogs({ ar: () => import("./locales/ar.json"), hi: () => import("./locales/hi.json") });

const page = (name: "FormView" | "ListView" | "ReportView" | "AppSettingsPage" | "ScreenPage") => () => import("./records").then((m) => ({ Component: m[name] }));

const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false, refetchOnWindowFocus: false, staleTime: 5_000 } } });

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
        { path: "apps/:app/settings", lazy: page("AppSettingsPage") },
        { path: "x/:app/:page", lazy: page("ScreenPage") },
        { path: "*", element: <NotFound /> },
      ],
    },
];

const router = onPortal ? createBrowserRouter(familyRoutes, { basename: routeBase(FAMILY) }) : createBrowserRouter(staffRoutes, { basename: routeBase("school") });

// Drawn once the page's words are in (its language's catalogs, fetched as the page starts).
void wordsReady().then(() =>
  createRoot(document.getElementById("root")!).render(
    <StrictMode>
      <QueryClientProvider client={queryClient}>
        <RouterProvider router={router} />
        <Toaster />
        <ConfirmHost />
      </QueryClientProvider>
    </StrictMode>,
  ),
);
