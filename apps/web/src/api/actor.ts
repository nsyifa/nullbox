/**
 * The worker's display name, from the name picker. This is NOT auth: the API
 * trusts whatever name we send. Replace with a real session/SSO later.
 */
const STORAGE_KEY = "nullbox.actor";
const listeners = new Set<() => void>();

function defaultName(): string {
  return `Agent ${Math.floor(100 + Math.random() * 900)}`;
}

let current: string = (() => {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) return stored;
    const name = defaultName();
    localStorage.setItem(STORAGE_KEY, name);
    return name;
  } catch {
    return defaultName();
  }
})();

export function getActorName(): string {
  return current;
}

export function setActorName(name: string): void {
  current = name;
  try {
    localStorage.setItem(STORAGE_KEY, name);
  } catch {
    // keep it in memory only
  }
  for (const listener of listeners) listener();
}

export function subscribeActor(listener: () => void): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}
