import type { BeastMood } from "../../api/generated/BeastMood";
import { useCopy } from "../../copy/CopyProvider";
import styles from "./Beast.module.css";
import { BeastAsleep } from "./BeastAsleep";
import { BeastGrumpy } from "./BeastGrumpy";
import { BeastKaiju } from "./BeastKaiju";
import { BeastPeckish } from "./BeastPeckish";

const BY_MOOD = {
  asleep: BeastAsleep,
  peckish: BeastPeckish,
  grumpy: BeastGrumpy,
  kaiju: BeastKaiju,
} satisfies Record<BeastMood, () => React.JSX.Element>;

/** The Backlog Beast. Its mood is computed by the backend (`nullbox_domain::BeastMood`). */
export function Beast({ mood }: { mood: BeastMood }) {
  const copy = useCopy();
  const Creature = BY_MOOD[mood];
  return (
    <figure className={`${styles.beast} ${styles[mood]}`} data-mood={mood}>
      <Creature />
      <figcaption className={styles.caption}>{copy.beast.moods[mood]}</figcaption>
    </figure>
  );
}
