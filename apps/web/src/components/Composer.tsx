import { type FormEvent, type KeyboardEvent, useId, useState } from "react";
import { useCopy } from "../copy/CopyProvider";
import styles from "./Composer.module.css";
import ui from "./ui.module.css";

interface ComposerProps {
  label: string;
  placeholder: string;
  submitLabel: string;
  busy?: boolean;
  /** Return a promise; the text is cleared once it resolves. */
  onSubmit: (body: string) => Promise<unknown>;
  children?: React.ReactNode;
}

/**
 * Text box for replies. Ctrl/Cmd + Enter sends, but never while an IME is
 * composing (Japanese input uses Enter to confirm a conversion).
 */
export function Composer({
  label,
  placeholder,
  submitLabel,
  busy,
  onSubmit,
  children,
}: ComposerProps) {
  const copy = useCopy();
  const id = useId();
  const [body, setBody] = useState("");
  const [capsLock, setCapsLock] = useState(false);

  const submit = async () => {
    if (!body.trim() || busy) return;
    await onSubmit(body);
    setBody("");
  };

  const onKeyDown = (event: KeyboardEvent<HTMLTextAreaElement>) => {
    setCapsLock(event.getModifierState("CapsLock"));
    if (event.nativeEvent.isComposing) return;
    if (event.key === "Enter" && (event.ctrlKey || event.metaKey)) {
      event.preventDefault();
      void submit();
    }
  };

  return (
    <form
      className={styles.composer}
      onSubmit={(event: FormEvent) => {
        event.preventDefault();
        void submit();
      }}
    >
      <label htmlFor={id} className={styles.label}>
        {label}
        {capsLock && <span className={`${ui.badge} ${ui.loud}`}>{copy.eggs.capsLock}</span>}
      </label>
      <textarea
        id={id}
        value={body}
        rows={4}
        placeholder={placeholder}
        onChange={(event) => setBody(event.target.value)}
        onKeyDown={onKeyDown}
        onKeyUp={(event) => setCapsLock(event.getModifierState("CapsLock"))}
      />
      <div className={styles.actions}>
        {children}
        <span className={styles.hint}>{copy.customer.sendHint}</span>
        <button type="submit" disabled={busy || !body.trim()}>
          {submitLabel}
        </button>
      </div>
    </form>
  );
}
