import styles from "./Beast.module.css";

export function BeastGrumpy() {
  return (
    <svg viewBox="0 0 200 160" className={`${styles.svg} ${styles.shake}`} aria-hidden="true">
      <ellipse cx="100" cy="150" rx="75" ry="8" className={styles.shadow} />
      <path
        d="M25 140 Q25 50 100 50 Q175 50 175 140 Z"
        className={`${styles.body} ${styles.angry}`}
      />
      <path d="M62 82 l28 10" className={styles.brow} />
      <path d="M138 82 l-28 10" className={styles.brow} />
      <circle cx="78" cy="100" r="9" className={styles.eyeWhite} />
      <circle cx="122" cy="100" r="9" className={styles.eyeWhite} />
      <circle cx="78" cy="102" r="4" className={styles.pupil} />
      <circle cx="122" cy="102" r="4" className={styles.pupil} />
      <path d="M78 130 q22 -14 44 0" className={styles.line} />
      <path d="M40 48 q-6 -12 2 -22" className={`${styles.steam} ${styles.s1}`} />
      <path d="M160 48 q6 -12 -2 -22" className={`${styles.steam} ${styles.s2}`} />
    </svg>
  );
}
