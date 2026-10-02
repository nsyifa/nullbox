import { type FormEvent, useState } from "react";
import { Link } from "react-router";
import type { CreateTicketResponse } from "../../api/generated/CreateTicketResponse";
import type { Severity } from "../../api/generated/Severity";
import { useCreateTicket } from "../../api/queries";
import { ErrorMessage } from "../../components/Feedback";
import { useCopy } from "../../copy/CopyProvider";
import styles from "../pages.module.css";

const SEVERITIES: Severity[] = ["p4", "p3", "p2", "p1"];

export function NewTicketPage() {
  const copy = useCopy();
  const create = useCreateTicket();
  const [created, setCreated] = useState<CreateTicketResponse | null>(null);

  const onSubmit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const form = new FormData(event.currentTarget);
    const text = (name: string) => String(form.get(name) ?? "").trim();
    create.mutate(
      {
        subject: text("subject"),
        body: text("body"),
        severity: text("severity") as Severity,
        customer_name: text("customer_name") || undefined,
        customer_email: text("customer_email") || undefined,
      },
      { onSuccess: setCreated },
    );
  };

  if (created) return <TicketCreated created={created} />;

  return (
    <section className={styles.page}>
      <div>
        <h1>{copy.customer.title}</h1>
        <p className={styles.intro}>{copy.customer.intro}</p>
      </div>
      <form className={`${styles.card} ${styles.form}`} onSubmit={onSubmit}>
        <label className={styles.field}>
          {copy.customer.subject}
          <input
            name="subject"
            required
            maxLength={200}
            placeholder={copy.customer.subjectPlaceholder}
          />
        </label>
        <label className={styles.field}>
          {copy.customer.body}
          <textarea name="body" required rows={6} placeholder={copy.customer.bodyPlaceholder} />
        </label>
        <label className={styles.field}>
          {copy.customer.severity}
          <select name="severity" defaultValue="p3">
            {SEVERITIES.map((severity) => (
              <option key={severity} value={severity}>
                {copy.severity[severity]}
              </option>
            ))}
          </select>
        </label>
        <div className={styles.row}>
          <label className={styles.field}>
            {copy.customer.name}
            <input name="customer_name" maxLength={100} autoComplete="name" />
          </label>
          <label className={styles.field}>
            {copy.customer.email}
            <input name="customer_email" type="email" maxLength={254} autoComplete="email" />
          </label>
        </div>
        <p className={styles.hint}>{copy.customer.anonymousHint}</p>
        {create.error && <ErrorMessage error={create.error} />}
        <div>
          <button type="submit" disabled={create.isPending}>
            {create.isPending ? copy.customer.submitting : copy.customer.submit}
          </button>
        </div>
      </form>
    </section>
  );
}

function TicketCreated({ created }: { created: CreateTicketResponse }) {
  const copy = useCopy();
  const [copied, setCopied] = useState(false);
  const path = `/t/${created.ticket.id}#${created.access_token}`;

  return (
    <section className={`${styles.card} ${styles.page}`}>
      <h1>{copy.customer.createdTitle}</h1>
      <p>{copy.customer.createdBody(created.ticket.number)}</p>
      <h2>{copy.customer.secretLinkTitle}</h2>
      <div className={styles.linkBox}>
        <code>{created.access_link}</code>
        <button
          type="button"
          onClick={() =>
            navigator.clipboard.writeText(created.access_link).then(() => setCopied(true))
          }
        >
          {copied ? copy.customer.copied : copy.customer.copyLink}
        </button>
      </div>
      <p className={styles.warning}>{copy.customer.secretLinkWarning}</p>
      <div>
        <Link to={path}>{copy.customer.openTicket} →</Link>
      </div>
    </section>
  );
}
