
import { useEffect, useState, type ReactNode } from "react";
import { Navigate } from "react-router";
import { api } from "../api/client";

export function ProtectedRoute({ children }: { children: ReactNode }) {
  const [authenticated, setAuthenticated] = useState<boolean | null>(null);

  useEffect(() => {
    let active = true;

    api.me()
      .then(() => {
        if (active) setAuthenticated(true);
      })
      .catch(() => {
        if (active) setAuthenticated(false);
      });

    return () => {
      active = false;
    };
  }, []);

  if (authenticated === null) return <p>Checking session...</p>;

  if (!authenticated) return <Navigate to="/login" replace />;

  return <>{children}</>;
}
