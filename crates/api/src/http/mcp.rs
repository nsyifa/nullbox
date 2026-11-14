//! MCP server (Streamable HTTP, JSON responses) at `/mcp`, so an AI agent can work
//! the help desk: read the inbox, look up a ticket, reply, change status.
//!
//! Auth: `Authorization: Bearer <MCP_TOKEN>`. Without `MCP_TOKEN` configured, every
//! request is refused with 401. Tools call `TicketService`, like the worker handlers.

use axum::Json;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde_json::{Value, json};
use uuid::Uuid;

use super::extract::Actor;
use crate::dto::UpdateTicketRequest;
use crate::policy::{self, Action, Resource};
use crate::{ApiError, AppState, token};

const SESSION_HEADER: &str = "mcp-session-id";
const SUPPORTED_VERSIONS: [&str; 4] = ["2026-07-28", "2025-11-25", "2025-06-18", "2025-03-26"];
const LATEST_VERSION: &str = "2026-07-28";
const AGENT_NAME: &str = "MCP agent";

/// `POST /mcp`: one JSON-RPC message per request.
pub async fn post(State(state): State<AppState>, headers: HeaderMap, body: Bytes) -> Response {
    if let Err(refused) = authorize(&state, &headers) {
        return refused;
    }
    let msg: Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(e) => return rpc_error(Value::Null, -32700, &format!("parse error: {e}")).into_response(),
    };
    let Some(method) = msg.get("method").and_then(Value::as_str) else {
        // A response or garbage from the client: nothing to answer.
        return StatusCode::ACCEPTED.into_response();
    };
    let Some(id) = msg.get("id").cloned() else {
        // Notification (e.g. notifications/initialized).
        return StatusCode::ACCEPTED.into_response();
    };
    let params = msg.get("params").cloned().unwrap_or(Value::Null);

    match method {
        "initialize" => {
            let requested = params.get("protocolVersion").and_then(Value::as_str).unwrap_or(LATEST_VERSION);
            let version = if SUPPORTED_VERSIONS.contains(&requested) { requested } else { LATEST_VERSION };
            let session = token::generate().unwrap_or_else(|_| Uuid::now_v7().to_string());
            let mut res = rpc_result(id, server_info(version)).into_response();
            if let Ok(value) = HeaderValue::from_str(&session) {
                res.headers_mut().insert(SESSION_HEADER, value);
            }
            res
        }
        "server/discover" => rpc_result(id, server_info(LATEST_VERSION)).into_response(),
        "ping" => rpc_result(id, json!({})).into_response(),
        "tools/list" => rpc_result(id, json!({ "tools": tools() })).into_response(),
        "tools/call" => {
            let name = params.get("name").and_then(Value::as_str).unwrap_or_default();
            let args = params.get("arguments").cloned().unwrap_or_else(|| json!({}));
            let result = match call_tool(&state, name, &args).await {
                Ok(value) => json!({
                    "content": [{ "type": "text", "text": value.to_string() }],
                    "structuredContent": value,
                    "isError": false,
                }),
                Err(message) => json!({
                    "content": [{ "type": "text", "text": message }],
                    "isError": true,
                }),
            };
            rpc_result(id, result).into_response()
        }
        other => rpc_error(id, -32601, &format!("method not found: {other}")).into_response(),
    }
}

/// `GET /mcp`: we don't offer a server-initiated SSE stream.
pub async fn get(State(state): State<AppState>, headers: HeaderMap) -> Response {
    authorize(&state, &headers).err().unwrap_or_else(|| StatusCode::METHOD_NOT_ALLOWED.into_response())
}

/// `DELETE /mcp`: end a session. Sessions hold no state, so there is nothing to drop.
pub async fn delete(State(state): State<AppState>, headers: HeaderMap) -> Response {
    authorize(&state, &headers).err().unwrap_or_else(|| StatusCode::NO_CONTENT.into_response())
}

fn authorize(state: &AppState, headers: &HeaderMap) -> Result<(), Response> {
    let presented = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer ").or_else(|| v.strip_prefix("bearer ")))
        .map(str::trim)
        .filter(|v| !v.is_empty());
    let ok = match (presented, state.config.mcp_token.as_deref()) {
        (Some(presented), Some(expected)) => constant_time_eq(&token::hash(presented), &token::hash(expected)),
        _ => false,
    };
    if ok {
        Ok(())
    } else {
        tracing::warn!("refused MCP request without a valid bearer token");
        let mut res = (StatusCode::UNAUTHORIZED, Json(json!({ "code": "unauthorized", "message": "bearer token required" })))
            .into_response();
        res.headers_mut().insert(header::WWW_AUTHENTICATE, HeaderValue::from_static("Bearer realm=\"nullbox-mcp\""));
        Err(res)
    }
}

fn constant_time_eq(a: &str, b: &str) -> bool {
    a.len() == b.len() && a.bytes().zip(b.bytes()).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

fn rpc_result(id: Value, result: Value) -> Json<Value> {
    Json(json!({ "jsonrpc": "2.0", "id": id, "result": result }))
}

fn rpc_error(id: Value, code: i64, message: &str) -> Json<Value> {
    Json(json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } }))
}

fn server_info(version: &str) -> Value {
    json!({
        "protocolVersion": version,
        "capabilities": { "tools": { "listChanged": false } },
        "serverInfo": { "name": "nullbox", "title": "Nullbox help desk", "version": env!("CARGO_PKG_VERSION") },
        "instructions": "Nullbox help desk. List the inbox, read a ticket, reply to the customer or leave an internal note, change status, or yeet (resolve) a ticket.",
    })
}

fn tools() -> Value {
    let status = json!({
        "type": "string",
        "enum": ["open", "in_progress", "waiting_on_customer", "resolved", "closed"],
    });
    let id = json!({ "type": "string", "description": "Ticket UUID" });
    json!([
        {
            "name": "list_tickets",
            "description": "List tickets in the inbox. Without a status, all active tickets (open, in progress, waiting on customer).",
            "inputSchema": { "type": "object", "properties": { "status": status } },
            "annotations": { "readOnlyHint": true },
        },
        {
            "name": "get_ticket",
            "description": "Get a ticket with its full message thread, internal notes included.",
            "inputSchema": { "type": "object", "properties": { "id": id }, "required": ["id"] },
            "annotations": { "readOnlyHint": true },
        },
        {
            "name": "reply_to_ticket",
            "description": "Reply to the customer on a ticket, or add an internal note (internal: true) the customer never sees.",
            "inputSchema": {
                "type": "object",
                "properties": { "id": id, "body": { "type": "string" }, "internal": { "type": "boolean" } },
                "required": ["id", "body"],
            },
        },
        {
            "name": "update_ticket",
            "description": "Change a ticket's status, severity or assignee (empty assignee_name unassigns).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": id,
                    "status": status,
                    "severity": { "type": "string" },
                    "assignee_name": { "type": "string" },
                },
                "required": ["id"],
            },
        },
        {
            "name": "yeet_ticket",
            "description": "Resolve a ticket. A customer reply reopens it.",
            "inputSchema": { "type": "object", "properties": { "id": id }, "required": ["id"] },
        },
        {
            "name": "backlog_stats",
            "description": "Count of open tickets, the backlog beast's mood, and the oldest open ticket's age.",
            "inputSchema": { "type": "object", "properties": {} },
            "annotations": { "readOnlyHint": true },
        },
    ])
}

async fn call_tool(state: &AppState, name: &str, args: &Value) -> Result<Value, String> {
    let actor = Actor { name: AGENT_NAME.to_owned() };
    let tickets = &state.tickets;
    let to_json = |v: Result<Value, serde_json::Error>| v.map_err(|e| e.to_string());
    let id = || -> Result<Uuid, String> {
        args.get("id").and_then(Value::as_str).and_then(|s| s.parse().ok()).ok_or_else(|| "id: a ticket UUID is required".to_owned())
    };

    match name {
        "list_tickets" => {
            policy::ensure(&actor, Action::ListTickets, Resource::Inbox).map_err(err)?;
            let list = tickets.list(field(args, "status")?).await.map_err(err)?;
            to_json(serde_json::to_value(list).map(|t| json!({ "tickets": t })))
        }
        "get_ticket" => {
            let id = id()?;
            policy::ensure(&actor, Action::ViewTicket, Resource::Ticket(id)).map_err(err)?;
            to_json(serde_json::to_value(tickets.worker_thread(id).await.map_err(err)?))
        }
        "reply_to_ticket" => {
            let id = id()?;
            let body: String = field(args, "body")?.ok_or_else(|| "body: required".to_owned())?;
            let internal: bool = field(args, "internal")?.unwrap_or(false);
            let action = if internal { Action::AddInternalNote } else { Action::ReplyToTicket };
            policy::ensure(&actor, action, Resource::Ticket(id)).map_err(err)?;
            to_json(serde_json::to_value(tickets.worker_message(&actor.name, id, &body, internal).await.map_err(err)?))
        }
        "update_ticket" => {
            let id = id()?;
            policy::ensure(&actor, Action::UpdateTicket, Resource::Ticket(id)).map_err(err)?;
            let req = UpdateTicketRequest {
                status: field(args, "status")?,
                severity: field(args, "severity")?,
                assignee_name: field(args, "assignee_name")?,
            };
            to_json(serde_json::to_value(tickets.update(&actor.name, id, req).await.map_err(err)?))
        }
        "yeet_ticket" => {
            let id = id()?;
            policy::ensure(&actor, Action::YeetTicket, Resource::Ticket(id)).map_err(err)?;
            to_json(serde_json::to_value(tickets.yeet_ticket(&actor.name, id).await.map_err(err)?))
        }
        "backlog_stats" => {
            policy::ensure(&actor, Action::ViewBacklogStats, Resource::Inbox).map_err(err)?;
            to_json(serde_json::to_value(tickets.backlog_stats().await.map_err(err)?))
        }
        other => Err(format!("unknown tool: {other}")),
    }
}

fn err(e: ApiError) -> String {
    match e {
        // Never leak internals to the agent.
        ApiError::ProbablyDns(_) | ApiError::WorksOnMyMachine(_) => "internal error".to_owned(),
        other => other.to_string(),
    }
}

fn field<T: serde::de::DeserializeOwned>(args: &Value, name: &str) -> Result<Option<T>, String> {
    match args.get(name) {
        None | Some(Value::Null) => Ok(None),
        Some(v) => serde_json::from_value(v.clone()).map(Some).map_err(|e| format!("{name}: {e}")),
    }
}
