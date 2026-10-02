//! Request/response bodies. Exported to TypeScript by `cargo test`.

use nullbox_domain::{BeastMood, Message, Severity, Ticket, TicketStatus};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct CreateTicketRequest {
    pub subject: String,
    pub body: String,
    pub severity: Severity,
    /// Leave both name and email out to stay anonymous.
    #[serde(default)]
    #[ts(optional)]
    pub customer_name: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub customer_email: Option<String>,
}

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct CreateTicketResponse {
    pub ticket: Ticket,
    /// Shown once. Only its hash is stored.
    pub access_token: String,
    /// `{PUBLIC_WEB_URL}/t/{id}#{token}`: the token sits in the fragment so it never reaches server logs.
    pub access_link: String,
}

/// A ticket with its messages. The customer view never contains internal notes.
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct TicketThread {
    pub ticket: Ticket,
    pub messages: Vec<Message>,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct CustomerReplyRequest {
    pub body: String,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct WorkerMessageRequest {
    pub body: String,
    /// Internal notes are hidden from the customer.
    #[serde(default)]
    #[ts(optional)]
    pub internal: Option<bool>,
}

#[derive(Debug, Default, Deserialize, TS)]
#[ts(export)]
pub struct UpdateTicketRequest {
    #[serde(default)]
    #[ts(optional)]
    pub status: Option<TicketStatus>,
    #[serde(default)]
    #[ts(optional)]
    pub severity: Option<Severity>,
    /// An empty string unassigns.
    #[serde(default)]
    #[ts(optional)]
    pub assignee_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListTicketsQuery {
    /// Without a status, all active tickets (open, in progress, waiting) are listed.
    pub status: Option<TicketStatus>,
}

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct BacklogStats {
    #[ts(type = "number")]
    pub open_count: u64,
    pub beast_mood: BeastMood,
    #[ts(type = "number | null")]
    pub oldest_open_age_secs: Option<i64>,
}
