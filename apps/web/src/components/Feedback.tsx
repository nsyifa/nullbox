import { useEffect, useState } from "react";
import { ApiError } from "../api/client";
import { useCopy } from "../copy/CopyProvider";
import styles from "./ui.module.css";

/** A random funny loading line, e.g. "Bribing the database…". */
export function Loading() {
  const copy = useCopy();
  const [index] = useState(() => Math.floor(Math.random() * copy.loading.length));
  return (
    <p className={styles.loading} role="status">
      {copy.loading[index % copy.loading.length]}
    </p>
  );
}

/** Maps API error codes to copy. The server's `message` is for developers only. */
export function ErrorMessage({ error }: { error: unknown }) {
  const copy = useCopy();
  const code = error instanceof ApiError ? error.code : "internal";
  return (
    <p className={styles.error} role="alert">
      {copy.errors[code]}
    </p>
  );
}

/** A short-lived message at the bottom of the screen. */
export function Toast({ message, onDone }: { message: string | null; onDone: () => void }) {
  useEffect(() => {
    if (!message) return;
    const timer = setTimeout(onDone, 3000);
    return () => clearTimeout(timer);
  }, [message, onDone]);
  if (!message) return null;
  return (
    <div className={styles.toast} role="status">
      {message}
    </div>
  );
}
