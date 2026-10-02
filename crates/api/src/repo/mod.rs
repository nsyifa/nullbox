//! Storage seam. Handlers and services only see `TicketRepository`, so a
//! Postgres implementation, multi-tenancy or backups don't touch them.

mod sqlite;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use nullbox_domain::{AnonIdentity, Message, Severity, Ticket, TicketStatus};
use uuid::Uuid;

pub use sqlite::SqliteTicketRepository;

#[derive(Debug, thiserror::Error)]
pub enum RepoError {
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error("corrupt row: {0}")]
    Corrupt(String),
}

/// Everything needed to store a new ticket. The repository allocates `number`.
#[derive(Debug, Clone)]
pub struct NewTicket {
    pub id: Uuid,
    pub subject: String,
    pub body: String,
    pub severity: Severity,
    pub customer_name: Option<String>,
    pub customer_email: Option<String>,
    pub anon: Option<AnonIdentity>,
    pub access_token_hash: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BacklogSnapshot {
    pub open_count: u64,
    pub oldest_open_created_at: Option<DateTime<Utc>>,
}

#[async_trait]
pub trait TicketRepository: Send + Sync {
    /// Insert a ticket, allocating the next sequential number.
    async fn create_ticket(&self, ticket: NewTicket) -> Result<Ticket, RepoError>;

    async fn get_ticket(&self, id: Uuid) -> Result<Option<Ticket>, RepoError>;

    /// The ticket a customer's secret token was issued for.
    async fn get_ticket_by_token(&self, token_hash: &str) -> Result<Option<Ticket>, RepoError>;

    /// Tickets with any of the given statuses, oldest first.
    async fn list_tickets(&self, statuses: &[TicketStatus]) -> Result<Vec<Ticket>, RepoError>;

    /// Persist the ticket's mutable fields and append messages, atomically.
    async fn save(&self, ticket: &Ticket, new_messages: &[Message]) -> Result<(), RepoError>;

    async fn list_messages(&self, ticket_id: Uuid, include_internal: bool) -> Result<Vec<Message>, RepoError>;

    async fn backlog(&self) -> Result<BacklogSnapshot, RepoError>;
}
