import { useState } from "react";
import { useCopy } from "../copy/CopyProvider";
import styles from "./EnterpriseMode.module.css";

/** Konami-code parody of enterprise software. Purely cosmetic. */
export function EnterpriseMode({ onExit }: { onExit: () => void }) {
  const copy = useCopy().eggs.enterprise;
  const [synergizing, setSynergizing] = useState(false);

  return (
    <div
      className={styles.backdrop}
      role="dialog"
      aria-modal="true"
      aria-labelledby="enterprise-title"
    >
      <div className={styles.dialog}>
        <h2 id="enterprise-title">{copy.title}</h2>
        {synergizing ? (
          <div className={styles.synergizing} role="status">
            <span className={styles.spinner} aria-hidden="true" />
            {copy.synergizing}
          </div>
        ) : (
          <form
            onSubmit={(event) => {
              event.preventDefault();
              setSynergizing(true);
            }}
          >
            <p>{copy.intro}</p>
            {copy.fields.map((field) => (
              <label key={field} className={styles.field}>
                {field} <span aria-hidden="true">*</span>
                <input required />
              </label>
            ))}
            <button type="submit">{copy.submit}</button>
          </form>
        )}
        <button type="button" className={styles.exit} onClick={onExit}>
          {copy.exit}
        </button>
      </div>
    </div>
  );
}
