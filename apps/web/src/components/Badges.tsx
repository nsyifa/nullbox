import type { Severity } from "../api/generated/Severity";
import type { Ticket } from "../api/generated/Ticket";
import type { TicketStatus } from "../api/generated/TicketStatus";
import { useCopy } from "../copy/CopyProvider";
import styles from "./ui.module.css";

export function StatusBadge({ status }: { status: TicketStatus }) {
  const copy = useCopy();
  return <span className={`${styles.badge} ${styles[status]}`}>{copy.status[status]}</span>;
}

export function SeverityBadge({ severity }: { severity: Severity }) {
  const copy = useCopy();
  return <span className={`${styles.badge} ${styles[severity]}`}>{copy.severity[severity]}</span>;
}

/** "#12". Ticket #404 is, of course, "Not Found". */
export function TicketNumber({ number }: { number: number }) {
  const copy = useCopy();
  if (number === 404) {
    return (
      <span title={copy.ticket.notFoundHint}>
        <span className={styles.notFound}>#404</span> {copy.ticket.notFound}
      </span>
    );
  }
  return <span>{copy.ticket.number(number)}</span>;
}

/** Named customers by name, anonymous ones as e.g. "Anonymous Platypus #4471". */
export function CustomerName({ ticket }: { ticket: Pick<Ticket, "customer_name" | "anon"> }) {
  const copy = useCopy();
  if (ticket.customer_name) return <>{ticket.customer_name}</>;
  if (ticket.anon) {
    const animal = copy.animals[ticket.anon.animal] ?? ticket.anon.animal;
    return <>{copy.anonName(animal, ticket.anon.number)}</>;
  }
  return <>{copy.time.unknown}</>;
}
