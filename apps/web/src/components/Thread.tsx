import type { Message } from "../api/generated/Message";
import type { SystemEvent } from "../api/generated/SystemEvent";
import type { Ticket } from "../api/generated/Ticket";
import { useCopy, useFormat } from "../copy/CopyProvider";
import type { Copy } from "../copy/en";
import { CustomerName } from "./Badges";
import styles from "./Thread.module.css";
import ui from "./ui.module.css";

/** The ticket's opening message followed by replies, notes and system events. */
export function Thread({ ticket, messages }: { ticket: Ticket; messages: Message[] }) {
  const copy = useCopy();
  const format = useFormat();
  return (
    <ol className={styles.thread}>
      <li className={`${styles.message} ${styles.customer}`}>
        <header className={styles.meta}>
          <strong>
            <CustomerName ticket={ticket} />
          </strong>
          <time dateTime={ticket.created_at} title={format.dateTime(ticket.created_at)}>
            {format.ago(ticket.created_at)}
          </time>
        </header>
        <p className={styles.body}>{ticket.body}</p>
      </li>
      {messages.map((message) =>
        message.system_event ? (
          <li key={message.id} className={styles.system}>
            {describeEvent(copy, message.system_event)} ·{" "}
            <time dateTime={message.created_at}>{format.ago(message.created_at)}</time>
          </li>
        ) : (
          <li
            key={message.id}
            className={`${styles.message} ${styles[message.author_kind]} ${message.internal ? styles.internal : ""}`}
          >
            <header className={styles.meta}>
              <strong>
                {message.author_kind === "customer" ? (
                  <CustomerName ticket={ticket} />
                ) : (
                  (message.author_name ?? copy.thread.system)
                )}
              </strong>
              {message.internal && <span className={ui.badge}>{copy.worker.internalBadge}</span>}
              <time dateTime={message.created_at} title={format.dateTime(message.created_at)}>
                {format.ago(message.created_at)}
              </time>
            </header>
            <p className={styles.body}>{message.body}</p>
          </li>
        ),
      )}
    </ol>
  );
}

function describeEvent(copy: Copy, event: SystemEvent): string {
  switch (event.kind) {
    case "status_changed":
      if (event.from === "resolved" && event.to === "open") return copy.thread.itsAlive;
      return copy.thread.statusChanged(copy.status[event.from], copy.status[event.to]);
    case "severity_changed":
      return copy.thread.severityChanged(copy.severity[event.from], copy.severity[event.to]);
    case "assignee_changed":
      return event.to ? copy.thread.assigned(event.to) : copy.thread.unassigned;
  }
}
