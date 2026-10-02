import { useEffect } from "react";

const SEQUENCE = [
  "ArrowUp",
  "ArrowUp",
  "ArrowDown",
  "ArrowDown",
  "ArrowLeft",
  "ArrowRight",
  "ArrowLeft",
  "ArrowRight",
  "b",
  "a",
];

/** Calls `onUnlock` when ↑↑↓↓←→←→BA is typed (ignored while an IME is composing). */
export function useKonami(onUnlock: () => void) {
  useEffect(() => {
    let position = 0;
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.isComposing) return;
      const key = event.key.length === 1 ? event.key.toLowerCase() : event.key;
      position = key === SEQUENCE[position] ? position + 1 : key === SEQUENCE[0] ? 1 : 0;
      if (position === SEQUENCE.length) {
        position = 0;
        onUnlock();
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [onUnlock]);
}
