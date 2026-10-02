import { useState } from "react";
import { Link, useNavigate, useParams } from "react-router";
import { getActorName } from "../../api/actor";
import type { Severity } from "../../api/generated/Severity";
import type { TicketStatus } from "../../api/generated/TicketStatus";
import { useAddMessage, useUpdateTicket, useWorkerTicket, useYeetTicket } from "../../api/queries";
import { CustomerName, SeverityBadge, StatusBadge, TicketNumber } from "../../components/Badges";
import { Composer } from "../../components/Composer";
import { ErrorMessage, Loading } from "../../components/Feedback";
import { Thread } from "../../components/Thread";
import { useCopy } from "../../copy/CopyProvider";
import styles from "../pages.module.css";

const STATUSES: TicketStatus[] = [
  "open",
  "in_progress",
  "waiting_on_customer",
  "resolved",
  "closed",
];
const SEVERITIES: Severity[] = ["p1", "p2", "p3", "p4"];
const YEET_MS = 700;

export function WorkerTicketPage() {
  const copy = useCopy();
  const navigate = useNavigate();
  const { id = "" } = useParams();
  const thread = useWorkerTicket(id);
  const addMessage = useAddMessage(id);
  const update = useUpdateTicket(id);
  const yeet = useYeetTicket();
  const [internal, setInternal] = useState(false);
  const [yeeting, setYeeting] = useState(false);

  if (thread.isPending) return <Loading />;
  if (thread.error) return <ErrorMessage error={thread.error} />;
  const { ticket, messages } = thread.data;

  const onYeet = () => {
    setYeeting(true);
    yeet.mutate(ticket.id, {
      onSuccess: () => setTimeout(() => navigate("/inbox"), YEET_MS),
      onError: () => setYeeting(false),
    });
  };

  const canYeet = ticket.status !== "resolved" && ticket.status !== "closed";
  const mutationError = addMessage.error ?? update.error ?? yeet.error;

  return (
    <section className={`${styles.page} ${yeeting ? styles.yeeting : ""}`}>
      <Link to="/inbox">← {copy.worker.back}</Link>
      <div className={styles.titleRow}>
        <span className={styles.muted}>
          <TicketNumber number={ticket.number} />
        </span>
        <h1>{ticket.subject}</h1>
        <StatusBadge status={ticket.status} />
        <SeverityBadge severity={ticket.severity} />
      </div>
      {mutationError && <ErrorMessage error={mutationError} />}

      <div className={styles.detailLayout}>
        <div className={styles.page}>
          <Thread ticket={ticket} messages={messages} />
          <div className={styles.card}>
            <div className={styles.tabs}>
              <button type="button" aria-pressed={!internal} onClick={() => setInternal(false)}>
                {copy.worker.reply}
              </button>
              <button type="button" aria-pressed={internal} onClick={() => setInternal(true)}>
                {copy.worker.internalNote}
              </button>
            </div>
            <Composer
              label={internal ? copy.worker.internalNote : copy.worker.reply}
              placeholder={internal ? copy.worker.notePlaceholder : copy.worker.replyPlaceholder}
              submitLabel={copy.worker.send}
              busy={addMessage.isPending}
              onSubmit={(body) => addMessage.mutateAsync({ body, internal })}
            />
          </div>
        </div>

        <aside className={`${styles.card} ${styles.side}`}>
          <p className={styles.muted}>
            {copy.ticket.customer}: <CustomerName ticket={ticket} />
            {ticket.customer_email && <> ({ticket.customer_email})</>}
          </p>
          <label className={styles.field}>
            {copy.ticket.status}
            <select
              value={ticket.status}
              onChange={(event) => update.mutate({ status: event.target.value as TicketStatus })}
            >
              {STATUSES.map((s) => (
                <option key={s} value={s}>
                  {copy.status[s]}
                </option>
              ))}
            </select>
          </label>
          <label className={styles.field}>
            {copy.ticket.severity}
            <select
              value={ticket.severity}
              onChange={(event) => update.mutate({ severity: event.target.value as Severity })}
            >
              {SEVERITIES.map((s) => (
                <option key={s} value={s}>
                  {copy.severity[s]}
                </option>
              ))}
            </select>
          </label>
          <AssigneeField
            key={ticket.assignee_name ?? ""}
            current={ticket.assignee_name}
            onAssign={(assignee_name) => update.mutate({ assignee_name })}
          />
          {canYeet && (
            <button
              type="button"
              className={styles.yeetButton}
              onClick={onYeet}
              disabled={yeeting}
              title={copy.worker.yeetHint}
            >
              🚀 {copy.worker.yeet}
            </button>
          )}
        </aside>
      </div>
    </section>
  );
}

function AssigneeField({
  current,
  onAssign,
}: {
  current: string | null;
  onAssign: (name: string) => void;
}) {
  const copy = useCopy();
  const [value, setValue] = useState(current ?? "");
  return (
    <form
      className={styles.field}
      onSubmit={(event) => {
        event.preventDefault();
        onAssign(value);
      }}
    >
      <label className={styles.field}>
        {copy.ticket.assignee}
        <input
          value={value}
          placeholder={copy.worker.assigneePlaceholder}
          maxLength={100}
          onChange={(e) => setValue(e.target.value)}
        />
      </label>
      <div className={styles.row}>
        <button type="submit">{copy.worker.assign}</button>
        <button type="button" onClick={() => onAssign(getActorName())}>
          {copy.worker.assignToMe}
        </button>
      </div>
    </form>
  );
}
