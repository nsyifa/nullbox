//! Business operations. Handlers stay thin; this is where rules meet storage.

use std::sync::Arc;

use chrono::Utc;
use nullbox_domain::{AnonIdentity, AuthorKind, BeastMood, DomainEvent, Message, SystemEvent, Ticket, TicketStatus};
use uuid::Uuid;

use crate::dto::{BacklogStats, CreateTicketRequest, TicketThread, UpdateTicketRequest};
use crate::error::ApiError;
use crate::events::EventSink;
use crate::repo::{NewTicket, TicketRepository};
use crate::token;

const MAX_SUBJECT: usize = 200;
const MAX_BODY: usize = 20_000;
const MAX_NAME: usize = 100;
const MAX_EMAIL: usize = 254;

pub struct TicketService {
    repo: Arc<dyn TicketRepository>,
    events: Arc<dyn EventSink>,
}

pub struct CreatedTicket {
    pub ticket: Ticket,
    pub access_token: String,
}

impl TicketService {
    pub fn new(repo: Arc<dyn TicketRepository>, events: Arc<dyn EventSink>) -> Self {
        Self { repo, events }
    }

    pub async fn create(&self, req: CreateTicketRequest) -> Result<CreatedTicket, ApiError> {
        let subject = required(&req.subject, "subject", MAX_SUBJECT)?;
        let body = required(&req.body, "body", MAX_BODY)?;
        let customer_name = optional(req.customer_name.as_deref(), "customer_name", MAX_NAME)?;
        let customer_email = optional(req.customer_email.as_deref(), "customer_email", MAX_EMAIL)?;
        if customer_email.as_deref().is_some_and(|email| !email.contains('@')) {
            return Err(ApiError::Validation("customer_email: not an email address".into()));
        }

        // No name means anonymous: the frontend renders e.g. "Anonymous Platypus #4471".
        let anon = customer_name.is_none().then(|| AnonIdentity::from_random(rand::random()));
        let access_token = token::generate().map_err(|e| ApiError::WorksOnMyMachine(e.to_string()))?;

        let ticket = self
            .repo
            .create_ticket(NewTicket {
                id: Uuid::now_v7(),
                subject,
                body,
                severity: req.severity,
                customer_name,
                customer_email,
                anon,
                access_token_hash: token::hash(&access_token),
                created_at: Utc::now(),
            })
            .await?;

        self.events.emit(DomainEvent::TicketCreated {
            ticket_id: ticket.id,
            number: ticket.number,
            severity: ticket.severity,
        });
        Ok(CreatedTicket { ticket, access_token })
    }

    pub async fn customer_thread(&self, id: Uuid, access_token: &str) -> Result<TicketThread, ApiError> {
        let ticket = self.ticket_for_customer(id, access_token).await?;
        let messages = self.repo.list_messages(id, false).await?;
        Ok(TicketThread { ticket, messages })
    }

    /// A customer reply. Replying to a resolved ticket reopens it.
    pub async fn customer_reply(&self, id: Uuid, access_token: &str, body: &str) -> Result<Message, ApiError> {
        let mut ticket = self.ticket_for_customer(id, access_token).await?;
        if ticket.status == TicketStatus::Closed {
            return Err(ApiError::TicketClosed);
        }
        let body = required(body, "body", MAX_BODY)?;

        let mut changes = Changes::default();
        let message = changes.message(&ticket, AuthorKind::Customer, ticket.customer_name.clone(), body, false, None);
        let next = ticket.status.after_customer_reply();
        if next != ticket.status {
            changes.status(&mut ticket, next, None)?;
        }
        self.commit(&mut ticket, changes).await?;
        Ok(message)
    }

    /// Tickets with the given status, or all active ones.
    pub async fn list(&self, status: Option<TicketStatus>) -> Result<Vec<Ticket>, ApiError> {
        let statuses = match status {
            Some(status) => vec![status],
            None => TicketStatus::ACTIVE.to_vec(),
        };
        Ok(self.repo.list_tickets(&statuses).await?)
    }

    /// Every ticket, whatever its status. For the admin export.
    pub async fn export(&self) -> Result<Vec<Ticket>, ApiError> {
        Ok(self.repo.list_tickets(&TicketStatus::ALL).await?)
    }

    pub async fn worker_thread(&self, id: Uuid) -> Result<TicketThread, ApiError> {
        let ticket = self.ticket(id).await?;
        let messages = self.repo.list_messages(id, true).await?;
        Ok(TicketThread { ticket, messages })
    }

    pub async fn worker_message(&self, actor: &str, id: Uuid, body: &str, internal: bool) -> Result<Message, ApiError> {
        let mut ticket = self.ticket(id).await?;
        let body = required(body, "body", MAX_BODY)?;
        let mut changes = Changes::default();
        let message = changes.message(&ticket, AuthorKind::Worker, Some(actor.to_owned()), body, internal, Some(actor));
        self.commit(&mut ticket, changes).await?;
        Ok(message)
    }

    pub async fn update(&self, actor: &str, id: Uuid, req: UpdateTicketRequest) -> Result<Ticket, ApiError> {
        let mut ticket = self.ticket(id).await?;
        let mut changes = Changes::default();

        if let Some(to) = req.status
            && to != ticket.status
        {
            changes.status(&mut ticket, to, Some(actor))?;
        }
        if let Some(to) = req.severity
            && to != ticket.severity
        {
            let from = std::mem::replace(&mut ticket.severity, to);
            changes.system(&ticket, SystemEvent::SeverityChanged { from, to });
        }
        if let Some(name) = req.assignee_name {
            let to = optional(Some(&name), "assignee_name", MAX_NAME)?;
            if to != ticket.assignee_name {
                let from = std::mem::replace(&mut ticket.assignee_name, to.clone());
                changes.system(&ticket, SystemEvent::AssigneeChanged { from, to });
            }
        }

        self.commit(&mut ticket, changes).await?;
        Ok(ticket)
    }

    /// Yeet = close, from the worker's point of view: the ticket becomes `resolved`
    /// ("Yeeted" in the UI). A customer reply brings it back; `closed` is the final archive.
    pub async fn yeet_ticket(&self, actor: &str, id: Uuid) -> Result<Ticket, ApiError> {
        let mut ticket = self.ticket(id).await?;
        let mut changes = Changes::default();
        changes.status(&mut ticket, TicketStatus::Resolved, Some(actor))?;
        changes.events.push(DomainEvent::TicketYeeted { ticket_id: ticket.id, actor: actor.to_owned() });
        self.commit(&mut ticket, changes).await?;
        Ok(ticket)
    }

    pub async fn backlog_stats(&self) -> Result<BacklogStats, ApiError> {
        let snapshot = self.repo.backlog().await?;
        Ok(BacklogStats {
            open_count: snapshot.open_count,
            beast_mood: BeastMood::from_open_count(snapshot.open_count),
            oldest_open_age_secs: snapshot
                .oldest_open_created_at
                .map(|created| (Utc::now() - created).num_seconds().max(0)),
        })
    }

    async fn ticket(&self, id: Uuid) -> Result<Ticket, ApiError> {
        self.repo.get_ticket(id).await?.ok_or(ApiError::NotFound)
    }

    /// The secret token is the customer's login: without a valid one, every ticket looks missing.
    async fn ticket_for_customer(&self, id: Uuid, access_token: &str) -> Result<Ticket, ApiError> {
        self.repo.get_ticket_by_token(&token::hash(access_token)).await?.ok_or(ApiError::NotFound)?;
        self.ticket(id).await
    }

    /// Save the ticket with its new messages, then emit events (only once the write succeeded).
    async fn commit(&self, ticket: &mut Ticket, changes: Changes) -> Result<(), ApiError> {
        if changes.messages.is_empty() {
            return Ok(());
        }
        ticket.updated_at = Utc::now();
        self.repo.save(ticket, &changes.messages).await?;
        for event in changes.events {
            self.events.emit(event);
        }
        Ok(())
    }
}

/// Messages and events produced by one operation, persisted together by `commit`.
#[derive(Default)]
struct Changes {
    messages: Vec<Message>,
    events: Vec<DomainEvent>,
}

impl Changes {
    fn message(
        &mut self,
        ticket: &Ticket,
        kind: AuthorKind,
        author: Option<String>,
        body: String,
        internal: bool,
        actor: Option<&str>,
    ) -> Message {
        let message = new_message(ticket, kind, author, body, internal);
        self.events.push(DomainEvent::MessageAdded {
            ticket_id: ticket.id,
            message_id: message.id,
            internal,
            actor: actor.map(str::to_owned),
        });
        self.messages.push(message.clone());
        message
    }

    fn system(&mut self, ticket: &Ticket, event: SystemEvent) {
        self.messages.push(Message {
            system_event: Some(event),
            ..new_message(ticket, AuthorKind::System, None, String::new(), false)
        });
    }

    fn status(&mut self, ticket: &mut Ticket, to: TicketStatus, actor: Option<&str>) -> Result<(), ApiError> {
        let from = ticket.status;
        ticket.status = from.transition_to(to)?;
        ticket.closed_at = if to.is_active() { None } else { Some(Utc::now()) };
        self.system(ticket, SystemEvent::StatusChanged { from, to });
        self.events.push(DomainEvent::StatusChanged {
            ticket_id: ticket.id,
            from,
            to,
            actor: actor.map(str::to_owned),
        });
        Ok(())
    }
}

fn new_message(ticket: &Ticket, kind: AuthorKind, author: Option<String>, body: String, internal: bool) -> Message {
    Message {
        id: Uuid::now_v7(),
        ticket_id: ticket.id,
        author_kind: kind,
        author_name: author,
        body,
        internal,
        system_event: None,
        created_at: Utc::now(),
    }
}

fn required(value: &str, field: &str, max_chars: usize) -> Result<String, ApiError> {
    optional(Some(value), field, max_chars)?.ok_or_else(|| ApiError::Validation(format!("{field}: required")))
}

/// Trims; blank becomes `None`.
fn optional(value: Option<&str>, field: &str, max_chars: usize) -> Result<Option<String>, ApiError> {
    let Some(value) = value.map(str::trim).filter(|v| !v.is_empty()) else {
        return Ok(None);
    };
    if value.chars().count() > max_chars {
        return Err(ApiError::Validation(format!("{field}: longer than {max_chars} characters")));
    }
    Ok(Some(value.to_owned()))
}
