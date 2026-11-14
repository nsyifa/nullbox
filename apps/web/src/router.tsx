import { createBrowserRouter } from "react-router";
import { AppShell } from "./components/AppShell";
import { CustomerTicketPage } from "./routes/customer/CustomerTicketPage";
import { NewTicketPage } from "./routes/customer/NewTicketPage";
import { NotFoundPage } from "./routes/NotFoundPage";
import { InboxPage } from "./routes/worker/InboxPage";
import { WorkerTicketPage } from "./routes/worker/WorkerTicketPage";
import { AuthPage } from "./routes/AuthPage";
import { ProtectedRoute } from "./routes/ProtectedRoute";

/**
 * All routes, in one place. For Tauri (no server to rewrite deep links),
 * swap `createBrowserRouter` for `createHashRouter`: that's the only change.
 */
export const router = createBrowserRouter([
  {
    element: <AppShell />,
    children: [
      { path: "/login", element: <AuthPage /> },
      // customer portal
      { path: "/", element: <NewTicketPage /> },
      { path: "/t/:id", element: <CustomerTicketPage /> },
      // worker inbox
      {
        path: "/inbox",
        element: (
          <ProtectedRoute>
            <InboxPage />
          </ProtectedRoute>
        ),
      },
      {
        path: "/inbox/:id",
        element: (
          <ProtectedRoute>
            <WorkerTicketPage />
          </ProtectedRoute>
        ),
      },
      { path: "*", element: <NotFoundPage /> },
    ],
  },
]);
