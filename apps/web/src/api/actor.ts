/**
 * Worker session state.
 *
 * - The worker bearer token: the only worker credential. Kept in `sessionStorage`
 *   (gone when the tab/browser session ends) with an in-memory fallback. Never in
 *   the URL, never in `localStorage`, never bundled into the build.
 * - A display label for "Assign to me". This is NOT identity: the API ignores it and
 *   derives the actor from the token. It is only a convenience default for assignee.
 */
const TOKEN_KEY = "nullbox.workerToken";
const LABEL_KEY = "nullbox.actor";
const listeners = new Set<() => void>();

function notify(): void {
  for (const listener of listeners) listener();
}

let token: string | null = (() => {
  try {
    return sessionStorage.getItem(TOKEN_KEY) || null;
  } catch {
    return null;
  }
})();

export function getWorkerToken(): string | null {
  return token;
}

export function setWorkerToken(value: string): void {
  token = value.trim() || null;
  try {
    if (token) sessionStorage.setItem(TOKEN_KEY, token);
    else sessionStorage.removeItem(TOKEN_KEY);
  } catch {
    // keep it in memory only
  }
  notify();
}

export function clearWorkerToken(): void {
  setWorkerToken("");
}

function defaultLabel(): string {
  return `Agent ${Math.floor(100 + Math.random() * 900)}`;
}

let label: string = (() => {
  try {
    const stored = localStorage.getItem(LABEL_KEY);
    if (stored) return stored;
    const name = defaultLabel();
    localStorage.setItem(LABEL_KEY, name);
    return name;
  } catch {
    return defaultLabel();
  }
})();

/** Display label used by "Assign to me". Not sent as identity. */
export function getActorName(): string {
  return label;
}

export function setActorName(name: string): void {
  label = name;
  try {
    localStorage.setItem(LABEL_KEY, name);
  } catch {
    // keep it in memory only
  }
  notify();
}

/** Fires when the token or the label changes. */
export function subscribeActor(listener: () => void): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}
