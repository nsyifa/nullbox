//! Request extractors. `Actor` is the auth seam.

use axum::Json;
use axum::extract::{FromRequest, FromRequestParts, Path, Query};
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use sha2::{Digest, Sha256};

use crate::{ApiError, AppState};

/// No longer read for identity. Kept only so the CORS allow-list in `http/mod.rs`
/// still compiles; a client sending it gains nothing.
pub const ACTOR_HEADER: &str = "x-actor-name";
pub const ADMIN_TOKEN_HEADER: &str = "x-admin-token";
pub const TICKET_TOKEN_HEADER: &str = "x-ticket-token";

/// Actor name recorded for requests that present `WORKER_TOKEN`.
pub const WORKER_ACTOR_NAME: &str = "worker";
/// Actor name recorded for requests that present `HOT_WORKER_TOKEN`.
pub const HOT_ACTOR_NAME: &str = "HOT";

/// The worker making the request, verified by `Authorization: Bearer <token>`.
///
/// The token is matched against `WORKER_TOKEN` and `HOT_WORKER_TOKEN` (see `config.rs`)
/// and the name comes from whichever one matched, never from the caller. No header,
/// an unknown token, or no tokens configured at all: 401 `missing_actor`.
/// To add SSO/JWT later, replace only this extractor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Actor {
    pub name: String,
}

impl FromRequestParts<AppState> for Actor {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let presented = bearer_token(parts).ok_or(ApiError::MissingActor)?;
        let presented = Sha256::digest(presented.as_bytes());

        // Check both identities every time, so timing does not reveal which one is configured.
        let worker = digest_matches(&presented, state.config.worker_token.as_deref());
        let hot = digest_matches(&presented, state.config.hot_worker_token.as_deref());

        let name = match (worker, hot) {
            (true, false) => WORKER_ACTOR_NAME,
            (false, true) => HOT_ACTOR_NAME,
            // Config refuses identical tokens, so (true, true) only means misconfiguration.
            _ => return Err(ApiError::MissingActor),
        };
        Ok(Actor { name: name.to_owned() })
    }
}

/// The token from `Authorization: Bearer <token>`. The scheme is case-insensitive.
fn bearer_token(parts: &Parts) -> Option<&str> {
    let value = parts.headers.get(AUTHORIZATION)?.to_str().ok()?;
    let (scheme, token) = value.split_once(' ')?;
    let token = token.trim();
    (scheme.eq_ignore_ascii_case("bearer") && !token.is_empty()).then_some(token)
}

/// Compares SHA-256 digests (fixed 32 bytes) without an early exit, so the time taken
/// does not depend on how much of the token was right. An unset token never matches.
fn digest_matches(presented: &[u8], expected: Option<&str>) -> bool {
    let Some(expected) = expected else {
        return false;
    };
    let expected = Sha256::digest(expected.as_bytes());
    let diff = presented.iter().zip(expected.iter()).fold(0u8, |acc, (a, b)| acc | (a ^ b));
    std::hint::black_box(diff) == 0 && presented.len() == expected.len()
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

#[cfg(test)]
mod tests {
    use super::*;

    fn parts_with(auth: Option<&str>) -> Parts {
        let mut builder = axum::http::Request::builder();
        if let Some(value) = auth {
            builder = builder.header(AUTHORIZATION, value);
        }
        builder.body(()).unwrap().into_parts().0
    }

    #[test]
    fn bearer_token_parsing() {
        assert_eq!(bearer_token(&parts_with(Some("Bearer abc"))), Some("abc"));
        assert_eq!(bearer_token(&parts_with(Some("bearer abc"))), Some("abc"));
        assert_eq!(bearer_token(&parts_with(Some("Bearer "))), None);
        assert_eq!(bearer_token(&parts_with(Some("Basic abc"))), None);
        assert_eq!(bearer_token(&parts_with(Some("abc"))), None);
        assert_eq!(bearer_token(&parts_with(None)), None);
    }

    #[test]
    fn digest_matching() {
        let presented = Sha256::digest(b"synthetic");
        assert!(digest_matches(&presented, Some("synthetic")));
        assert!(!digest_matches(&presented, Some("synthetiC")));
        assert!(!digest_matches(&presented, None));
    }
}
