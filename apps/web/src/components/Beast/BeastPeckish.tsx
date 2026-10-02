import styles from "./Beast.module.css";

export function BeastPeckish() {
  return (
    <svg viewBox="0 0 200 160" className={`${styles.svg} ${styles.bob}`} aria-hidden="true">
      <ellipse cx="100" cy="150" rx="70" ry="8" className={styles.shadow} />
      <path d="M30 140 Q30 60 100 60 Q170 60 170 140 Z" className={styles.body} />
      <circle cx="78" cy="98" r="11" className={styles.eyeWhite} />
      <circle cx="122" cy="98" r="11" className={styles.eyeWhite} />
      <circle cx="81" cy="100" r="5" className={`${styles.pupil} ${styles.look}`} />
      <circle cx="125" cy="100" r="5" className={`${styles.pupil} ${styles.look}`} />
      <path d="M82 122 q18 14 36 0" className={styles.line} />
      <path d="M108 128 q4 10 10 0" className={styles.tongue} />
    </svg>
  );
}
