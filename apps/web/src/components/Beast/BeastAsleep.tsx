import styles from "./Beast.module.css";

export function BeastAsleep() {
  return (
    <svg viewBox="0 0 200 160" className={`${styles.svg} ${styles.breathe}`} aria-hidden="true">
      <ellipse cx="100" cy="150" rx="70" ry="8" className={styles.shadow} />
      <path d="M30 140 Q30 70 100 70 Q170 70 170 140 Z" className={styles.body} />
      <path d="M70 105 q10 8 20 0" className={styles.line} />
      <path d="M110 105 q10 8 20 0" className={styles.line} />
      <ellipse cx="100" cy="122" rx="6" ry="4" className={styles.mouth} />
      <text x="150" y="60" className={`${styles.zzz} ${styles.z1}`}>
        z
      </text>
      <text x="162" y="42" className={`${styles.zzz} ${styles.z2}`}>
        z
      </text>
      <text x="176" y="24" className={`${styles.zzz} ${styles.z3}`}>
        Z
      </text>
    </svg>
  );
}
