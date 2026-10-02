//! Worker endpoints. UNPROTECTED in the MVP: `Actor` is an untrusted header and
//! `policy::can` allows everything. Both are the seams to replace.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use nullbox_domain::{Message, Ticket};
use uuid::Uuid;

use super::extract::{Actor, ApiJson, ApiPath, ApiQuery};
use crate::dto::{BacklogStats, ListTicketsQuery, TicketThread, UpdateTicketRequest, WorkerMessageRequest};
use crate::policy::{self, Action, Resource};
use crate::{ApiError, AppState};

pub async fn list_tickets(
    State(state): State<AppState>,
    actor: Actor,
    ApiQuery(query): ApiQuery<ListTicketsQuery>,
) -> Result<Json<Vec<Ticket>>, ApiError> {
    policy::ensure(&actor, Action::ListTickets, Resource::Inbox)?;
    Ok(Json(state.tickets.list(query.status).await?))
}

pub async fn get_ticket(
    State(state): State<AppState>,
    actor: Actor,
    ApiPath(id): ApiPath<Uuid>,
) -> Result<Json<TicketThread>, ApiError> {
    policy::ensure(&actor, Action::ViewTicket, Resource::Ticket(id))?;
    Ok(Json(state.tickets.worker_thread(id).await?))
}

pub async fn add_message(
    State(state): State<AppState>,
    actor: Actor,
    ApiPath(id): ApiPath<Uuid>,
    ApiJson(req): ApiJson<WorkerMessageRequest>,
) -> Result<(StatusCode, Json<Message>), ApiError> {
    let internal = req.internal.unwrap_or(false);
    let action = if internal { Action::AddInternalNote } else { Action::ReplyToTicket };
    policy::ensure(&actor, action, Resource::Ticket(id))?;
    let message = state.tickets.worker_message(&actor.name, id, &req.body, internal).await?;
    Ok((StatusCode::CREATED, Json(message)))
}

pub async fn update_ticket(
    State(state): State<AppState>,
    actor: Actor,
    ApiPath(id): ApiPath<Uuid>,
    ApiJson(req): ApiJson<UpdateTicketRequest>,
) -> Result<Json<Ticket>, ApiError> {
    policy::ensure(&actor, Action::UpdateTicket, Resource::Ticket(id))?;
    Ok(Json(state.tickets.update(&actor.name, id, req).await?))
}

/// Yeet a ticket, i.e. close it. Documented alias for `PATCH { status: "resolved" }`
/// that also emits `TicketYeeted`.
pub async fn yeet_ticket(
    State(state): State<AppState>,
    actor: Actor,
    ApiPath(id): ApiPath<Uuid>,
) -> Result<Json<Ticket>, ApiError> {
    policy::ensure(&actor, Action::YeetTicket, Resource::Ticket(id))?;
    Ok(Json(state.tickets.yeet_ticket(&actor.name, id).await?))
}

pub async fn backlog_stats(State(state): State<AppState>, actor: Actor) -> Result<Json<BacklogStats>, ApiError> {
    policy::ensure(&actor, Action::ViewBacklogStats, Resource::Inbox)?;
    Ok(Json(state.tickets.backlog_stats().await?))
}
