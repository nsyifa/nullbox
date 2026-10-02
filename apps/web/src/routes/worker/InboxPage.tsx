import { useState } from "react";
import { Link } from "react-router";
import type { TicketStatus } from "../../api/generated/TicketStatus";
import { useBacklogStats, useInbox } from "../../api/queries";
import { agingLevel } from "../../components/aging";
import { CustomerName, SeverityBadge, StatusBadge, TicketNumber } from "../../components/Badges";
import { Beast } from "../../components/Beast/Beast";
import { Celebration } from "../../components/Celebration";
import { ErrorMessage, Loading, Toast } from "../../components/Feedback";
import { useCopy, useFormat } from "../../copy/CopyProvider";
import styles from "../pages.module.css";

const STATUSES: TicketStatus[] = [
  "open",
  "in_progress",
  "waiting_on_customer",
  "resolved",
  "closed",
];

export function InboxPage() {
  const copy = useCopy();
  const format = useFormat();
  const [status, setStatus] = useState<TicketStatus | undefined>(undefined);
  const [search, setSearch] = useState("");
  const [toast, setToast] = useState<string | null>(null);
  const inbox = useInbox(status);
  const stats = useBacklogStats();

  const tickets = (inbox.data ?? []).filter((ticket) =>
    ticket.subject.toLowerCase().includes(search.trim().toLowerCase()),
  );

  return (
    <section className={styles.inboxLayout}>
      <div className={styles.page}>
        <h1>{copy.worker.inboxTitle}</h1>
        <div className={styles.filters}>
          <label>
            {copy.worker.filter}
            <select
              value={status ?? ""}
              onChange={(event) =>
                setStatus((event.target.value || undefined) as TicketStatus | undefined)
              }
            >
              <option value="">{copy.worker.filterActive}</option>
              {STATUSES.map((s) => (
                <option key={s} value={s}>
                  {copy.status[s]}
                </option>
              ))}
            </select>
          </label>
          <label>
            {copy.worker.search}
            <input
              type="search"
              value={search}
              placeholder={copy.worker.searchPlaceholder}
              onChange={(event) => {
                setSearch(event.target.value);
                // IME-safe: only react to committed text.
                const native = event.nativeEvent as InputEvent;
                if (!native.isComposing && /\bsudo\b/i.test(event.target.value))
                  setToast(copy.eggs.sudo);
              }}
            />
          </label>
        </div>

        {inbox.isPending && <Loading />}
        {inbox.error && <ErrorMessage error={inbox.error} />}
        {inbox.data && inbox.data.length === 0 && !status && (
          <div className={styles.zero}>
            <p>{copy.zero.void}</p>
          </div>
        )}
        {inbox.data && inbox.data.length > 0 && tickets.length === 0 && (
          <p className={styles.muted}>{copy.worker.noMatches}</p>
        )}
        <ul className={styles.list}>
          {tickets.map((ticket) => {
            const aging =
              ticket.status === "resolved" || ticket.status === "closed"
                ? "fresh"
                : agingLevel(ticket.created_at);
            return (
              <li key={ticket.id}>
                <Link
                  to={`/inbox/${ticket.id}`}
                  className={`${styles.ticketRow} ${styles[aging] ?? ""}`}
                  title={copy.ticket.aging[aging]}
                >
                  <span className={styles.muted}>
                    <TicketNumber number={ticket.number} />
                  </span>
                  <span className={styles.ticketSubject}>{ticket.subject}</span>
                  <StatusBadge status={ticket.status} />
                  <span className={styles.ticketMeta}>
                    <SeverityBadge severity={ticket.severity} />
                    <CustomerName ticket={ticket} />
                    <span>· {copy.ticket.opened(format.ago(ticket.created_at))}</span>
                    {ticket.assignee_name && <span>· {ticket.assignee_name}</span>}
                  </span>
                </Link>
              </li>
            );
          })}
        </ul>
      </div>

      <aside className={`${styles.card} ${styles.beastPanel}`} aria-label={copy.beast.title}>
        <h2>{copy.beast.title}</h2>
        {stats.data ? (
          <>
            <Beast mood={stats.data.beast_mood} />
            <p className={styles.stat}>
              {copy.beast.openCount(format.number(stats.data.open_count))}
            </p>
            {stats.data.oldest_open_age_secs !== null && (
              <p className={styles.muted}>
                {copy.beast.oldest(format.age(stats.data.oldest_open_age_secs))}
              </p>
            )}
          </>
        ) : stats.error ? (
          <ErrorMessage error={stats.error} />
        ) : (
          <Loading />
        )}
      </aside>

      <Celebration openCount={stats.data?.open_count} />
      <Toast message={toast} onDone={() => setToast(null)} />
    </section>
  );
}
