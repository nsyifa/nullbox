use async_trait::async_trait;
use chrono::{DateTime, Utc};
use nullbox_domain::{AnonIdentity, Message, Ticket, TicketStatus};
use sqlx::SqlitePool;
use uuid::Uuid;

use super::{BacklogSnapshot, NewTicket, RepoError, TicketRepository};

pub struct SqliteTicketRepository {
    pool: SqlitePool,
}

impl SqliteTicketRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

struct TicketRow {
    id: String,
    number: i64,
    subject: String,
    body: String,
    severity: String,
    status: String,
    customer_name: Option<String>,
    customer_email: Option<String>,
    anon_animal: Option<String>,
    anon_number: Option<i64>,
    assignee_name: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    closed_at: Option<DateTime<Utc>>,
}

impl TryFrom<TicketRow> for Ticket {
    type Error = RepoError;

    fn try_from(row: TicketRow) -> Result<Self, Self::Error> {
        let anon = match (row.anon_animal, row.anon_number) {
            (Some(animal), Some(number)) => {
                Some(AnonIdentity { animal, number: u16::try_from(number).map_err(corrupt)? })
            }
            _ => None,
        };
        Ok(Ticket {
            id: Uuid::parse_str(&row.id).map_err(corrupt)?,
            number: row.number,
            subject: row.subject,
            body: row.body,
            severity: row.severity.parse().map_err(corrupt)?,
            status: row.status.parse().map_err(corrupt)?,
            customer_name: row.customer_name,
            customer_email: row.customer_email,
            anon,
            assignee_name: row.assignee_name,
            created_at: row.created_at,
            updated_at: row.updated_at,
            closed_at: row.closed_at,
        })
    }
}

struct MessageRow {
    id: String,
    ticket_id: String,
    author_kind: String,
    author_name: Option<String>,
    body: String,
    internal: bool,
    system_event: Option<String>,
    created_at: DateTime<Utc>,
}

impl TryFrom<MessageRow> for Message {
    type Error = RepoError;

    fn try_from(row: MessageRow) -> Result<Self, Self::Error> {
        Ok(Message {
            id: Uuid::parse_str(&row.id).map_err(corrupt)?,
            ticket_id: Uuid::parse_str(&row.ticket_id).map_err(corrupt)?,
            author_kind: row.author_kind.parse().map_err(corrupt)?,
            author_name: row.author_name,
            body: row.body,
            internal: row.internal,
            system_event: row.system_event.as_deref().map(serde_json::from_str).transpose().map_err(corrupt)?,
            created_at: row.created_at,
        })
    }
}

fn corrupt(err: impl std::fmt::Display) -> RepoError {
    RepoError::Corrupt(err.to_string())
}

#[async_trait]
impl TicketRepository for SqliteTicketRepository {
    async fn create_ticket(&self, new: NewTicket) -> Result<Ticket, RepoError> {
        let id = new.id.to_string();
        let severity = new.severity.as_str();
        let status = TicketStatus::Open.as_str();
        let (anon_animal, anon_number) = match &new.anon {
            Some(anon) => (Some(anon.animal.as_str()), Some(i64::from(anon.number))),
            None => (None, None),
        };
        // Number allocation is SQLite-specific: a single INSERT holds the write
        // lock, so MAX + 1 can't race. On Postgres, use a sequence instead.
        let number = sqlx::query_scalar!(
            r#"INSERT INTO tickets (id, number, subject, body, severity, status, customer_name, customer_email,
                                   anon_animal, anon_number, access_token_hash, created_at, updated_at)
               VALUES (?, (SELECT COALESCE(MAX(number), 0) + 1 FROM tickets), ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
               RETURNING number"#,
            id,
            new.subject,
            new.body,
            severity,
            status,
            new.customer_name,
            new.customer_email,
            anon_animal,
            anon_number,
            new.access_token_hash,
            new.created_at,
            new.created_at,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(Ticket {
            id: new.id,
            number,
            subject: new.subject,
            body: new.body,
            severity: new.severity,
            status: TicketStatus::Open,
            customer_name: new.customer_name,
            customer_email: new.customer_email,
            anon: new.anon,
            assignee_name: None,
            created_at: new.created_at,
            updated_at: new.created_at,
            closed_at: None,
        })
    }

    async fn get_ticket(&self, id: Uuid) -> Result<Option<Ticket>, RepoError> {
        let id = id.to_string();
        sqlx::query_as!(
            TicketRow,
            r#"SELECT id, number, subject, body, severity, status, customer_name, customer_email, anon_animal,
                      anon_number, assignee_name, created_at AS "created_at: DateTime<Utc>",
                      updated_at AS "updated_at: DateTime<Utc>", closed_at AS "closed_at: DateTime<Utc>"
               FROM tickets WHERE id = ?"#,
            id
        )
        .fetch_optional(&self.pool)
        .await?
        .map(Ticket::try_from)
        .transpose()
    }

    async fn get_ticket_by_token(&self, token_hash: &str) -> Result<Option<Ticket>, RepoError> {
        sqlx::query_as!(
            TicketRow,
            r#"SELECT id, number, subject, body, severity, status, customer_name, customer_email, anon_animal,
                      anon_number, assignee_name, created_at AS "created_at: DateTime<Utc>",
                      updated_at AS "updated_at: DateTime<Utc>", closed_at AS "closed_at: DateTime<Utc>"
               FROM tickets WHERE access_token_hash = ?"#,
            token_hash
        )
        .fetch_optional(&self.pool)
        .await?
        .map(Ticket::try_from)
        .transpose()
    }

    async fn list_tickets(&self, statuses: &[TicketStatus]) -> Result<Vec<Ticket>, RepoError> {
        // SQLite has no array binds; json_each does the job (Postgres: `= ANY($1)`).
        let statuses = serde_json::to_string(statuses).map_err(corrupt)?;
        sqlx::query_as!(
            TicketRow,
            r#"SELECT id, number, subject, body, severity, status, customer_name, customer_email, anon_animal,
                      anon_number, assignee_name, created_at AS "created_at: DateTime<Utc>",
                      updated_at AS "updated_at: DateTime<Utc>", closed_at AS "closed_at: DateTime<Utc>"
               FROM tickets WHERE status IN (SELECT value FROM json_each(?))
               ORDER BY number"#,
            statuses
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(Ticket::try_from)
        .collect()
    }

    async fn save(&self, ticket: &Ticket, new_messages: &[Message]) -> Result<(), RepoError> {
        let mut tx = self.pool.begin().await?;

        let id = ticket.id.to_string();
        let severity = ticket.severity.as_str();
        let status = ticket.status.as_str();
        sqlx::query!(
            "UPDATE tickets SET severity = ?, status = ?, assignee_name = ?, updated_at = ?, closed_at = ? WHERE id = ?",
            severity,
            status,
            ticket.assignee_name,
            ticket.updated_at,
            ticket.closed_at,
            id,
        )
        .execute(&mut *tx)
        .await?;

        for message in new_messages {
            let message_id = message.id.to_string();
            let ticket_id = message.ticket_id.to_string();
            let author_kind = message.author_kind.as_str();
            let system_event = message.system_event.as_ref().map(serde_json::to_string).transpose().map_err(corrupt)?;
            sqlx::query!(
                "INSERT INTO messages (id, ticket_id, author_kind, author_name, body, internal, system_event, created_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                message_id,
                ticket_id,
                author_kind,
                message.author_name,
                message.body,
                message.internal,
                system_event,
                message.created_at,
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    async fn list_messages(&self, ticket_id: Uuid, include_internal: bool) -> Result<Vec<Message>, RepoError> {
        let ticket_id = ticket_id.to_string();
        sqlx::query_as!(
            MessageRow,
            r#"SELECT id, ticket_id, author_kind, author_name, body, internal AS "internal: bool", system_event,
                      created_at AS "created_at: DateTime<Utc>"
               FROM messages WHERE ticket_id = ? AND (? OR internal = FALSE)
               ORDER BY created_at, id"#,
            ticket_id,
            include_internal,
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(Message::try_from)
        .collect()
    }

    async fn backlog(&self) -> Result<BacklogSnapshot, RepoError> {
        let active = serde_json::to_string(&TicketStatus::ACTIVE).map_err(corrupt)?;
        let row = sqlx::query!(
            r#"SELECT COUNT(*) AS "open_count!: i64", MIN(created_at) AS "oldest: DateTime<Utc>"
               FROM tickets WHERE status IN (SELECT value FROM json_each(?))"#,
            active
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(BacklogSnapshot {
            open_count: u64::try_from(row.open_count).map_err(corrupt)?,
            oldest_open_created_at: row.oldest,
        })
    }
}
