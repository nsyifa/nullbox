use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

use crate::{Severity, TicketStatus};

/// Structured payload of a `system` message. The frontend turns it into prose.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[ts(export)]
pub enum SystemEvent {
    StatusChanged { from: TicketStatus, to: TicketStatus },
    SeverityChanged { from: Severity, to: Severity },
    AssigneeChanged { from: Option<String>, to: Option<String> },
}

/// Something that happened, emitted by services to an `EventSink`.
///
/// This is the seam for audit logs, webhooks, notifications and metrics.
/// Add variants freely; consumers should ignore what they don't understand.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DomainEvent {
    TicketCreated {
        ticket_id: Uuid,
        number: i64,
        severity: Severity,
    },
    StatusChanged {
        ticket_id: Uuid,
        from: TicketStatus,
        to: TicketStatus,
        actor: Option<String>,
    },
    MessageAdded {
        ticket_id: Uuid,
        message_id: Uuid,
        internal: bool,
        actor: Option<String>,
    },
    /// The ticket was yeeted (closed by a worker, status `resolved`).
    TicketYeeted {
        ticket_id: Uuid,
        actor: String,
    },
}
