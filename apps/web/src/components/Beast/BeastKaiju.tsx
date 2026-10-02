import styles from "./Beast.module.css";

export function BeastKaiju() {
  return (
    <svg viewBox="0 0 200 160" className={`${styles.svg} ${styles.stomp}`} aria-hidden="true">
      <ellipse cx="100" cy="152" rx="90" ry="7" className={styles.shadow} />
      <path
        d="M14 145 Q14 40 100 34 Q186 40 186 145 Z M40 50 l10 -24 l12 20 l14 -26 l12 24 l14 -26 l12 26 l14 -24 l12 22 l12 -18 l4 30 Z"
        className={`${styles.body} ${styles.kaiju}`}
      />
      <path d="M58 76 l34 14" className={styles.brow} />
      <path d="M142 76 l-34 14" className={styles.brow} />
      <circle cx="76" cy="98" r="10" className={styles.glow} />
      <circle cx="124" cy="98" r="10" className={styles.glow} />
      <path d="M58 118 Q100 150 142 118 Z" className={styles.maw} />
      <path
        d="M66 121 l6 10 l6 -8 l7 10 l7 -9 l8 10 l8 -10 l7 9 l7 -10 l6 8 l6 -10"
        className={styles.teeth}
      />
    </svg>
  );
}
