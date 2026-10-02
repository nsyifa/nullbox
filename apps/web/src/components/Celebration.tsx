import { useEffect, useRef, useState } from "react";
import { useCopy } from "../copy/CopyProvider";
import styles from "./Celebration.module.css";

const CONFETTI = Array.from({ length: 40 }, (_, i) => i);

/** Fires once when the open count drops from > 0 to 0. */
export function Celebration({ openCount }: { openCount: number | undefined }) {
  const copy = useCopy();
  const previous = useRef(openCount);
  const [active, setActive] = useState(false);

  useEffect(() => {
    if (previous.current !== undefined && previous.current > 0 && openCount === 0) {
      setActive(true);
      const timer = setTimeout(() => setActive(false), 4000);
      previous.current = openCount;
      return () => clearTimeout(timer);
    }
    previous.current = openCount;
  }, [openCount]);

  if (!active) return null;
  return (
    <div className={styles.celebration} role="status">
      {CONFETTI.map((i) => (
        <span
          key={i}
          className={styles.confetti}
          style={{ "--i": i, "--x": `${(i * 37) % 100}vw` } as React.CSSProperties}
        />
      ))}
      <p className={styles.banner}>{copy.zero.celebration}</p>
    </div>
  );
}
