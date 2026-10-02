//! Authorization seam. Every worker handler asks `can` before doing anything.

use uuid::Uuid;

use crate::ApiError;
use crate::http::extract::Actor;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    ListTickets,
    ViewTicket,
    ReplyToTicket,
    AddInternalNote,
    UpdateTicket,
    YeetTicket,
    ViewBacklogStats,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resource {
    Inbox,
    Ticket(Uuid),
}

/// MVP: everyone may do everything. Replace the body with real rules (roles, teams, tenants).
pub fn can(_actor: &Actor, _action: Action, _resource: Resource) -> bool {
    true
}

pub fn ensure(actor: &Actor, action: Action, resource: Resource) -> Result<(), ApiError> {
    if can(actor, action, resource) { Ok(()) } else { Err(ApiError::Forbidden) }
}
