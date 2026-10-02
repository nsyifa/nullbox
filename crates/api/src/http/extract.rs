//! Request extractors. `Actor` is the auth seam.

use axum::Json;
use axum::extract::{FromRequest, FromRequestParts, Path, Query};
use axum::http::request::Parts;
use percent_encoding::percent_decode_str;

use crate::{ApiError, AppState};

pub const ACTOR_HEADER: &str = "x-actor-name";
pub const ADMIN_TOKEN_HEADER: &str = "x-admin-token";
pub const TICKET_TOKEN_HEADER: &str = "x-ticket-token";

/// The worker making the request.
///
/// MVP: trusts the `X-Actor-Name` header (percent-encoded, so Japanese names
/// survive HTTP headers), which the frontend fills from a name picker.
/// Anyone can claim to be anyone. To add SSO/JWT, replace only this extractor.
///
/// It gets the `AppState`, so a real login can look up a session or a user here,
/// and a refused request can be written to an audit log before the 401 goes out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Actor {
    pub name: String,
}

impl FromRequestParts<AppState> for Actor {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &AppState) -> Result<Self, Self::Rejection> {
        let raw = parts.headers.get(ACTOR_HEADER).and_then(|v| v.to_str().ok()).ok_or(ApiError::MissingActor)?;
        let name = percent_decode_str(raw).decode_utf8().map_err(|_| ApiError::MissingActor)?;
        let name = name.trim();
        if name.is_empty() || name.chars().count() > 100 {
            return Err(ApiError::MissingActor);
        }
        Ok(Actor { name: name.to_owned() })
    }
}

/// The customer's secret, from `X-Ticket-Token`. The frontend reads it from the URL fragment.
pub struct TicketToken(pub String);

impl<S: Send + Sync> FromRequestParts<S> for TicketToken {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .headers
            .get(TICKET_TOKEN_HEADER)
            .and_then(|v| v.to_str().ok())
            .filter(|v| !v.is_empty())
            .map(|v| TicketToken(v.to_owned()))
            .ok_or(ApiError::MissingTicketToken)
    }
}

/// `Json`, `Path` and `Query` with rejections in our `{ code, message }` shape.
#[derive(FromRequest)]
#[from_request(via(Json), rejection(ApiError))]
pub struct ApiJson<T>(pub T);

#[derive(FromRequestParts)]
#[from_request(via(Path), rejection(ApiError))]
pub struct ApiPath<T>(pub T);

#[derive(FromRequestParts)]
#[from_request(via(Query), rejection(ApiError))]
pub struct ApiQuery<T>(pub T);
