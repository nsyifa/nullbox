use axum::Json;
use axum::extract::rejection::{JsonRejection, PathRejection, QueryRejection};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use ts_rs::TS;

use crate::repo::RepoError;

/// Stable error codes. The frontend maps each one to copy; never rename a variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ErrorCode {
    NotFound,
    MissingTicketToken,
    MissingActor,
    Forbidden,
    ValidationFailed,
    InvalidStatusTransition,
    TicketClosed,
    ServiceUnavailable,
    Internal,
    ImATeapot,
}

/// The one error shape: `{ code, message }`. `message` is English and for developers only.
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct ErrorBody {
    pub code: ErrorCode,
    pub message: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("not found")]
    NotFound,
    #[error("missing X-Ticket-Token header")]
    MissingTicketToken,
    #[error("missing or invalid X-Actor-Name header")]
    MissingActor,
    #[error("forbidden")]
    Forbidden,
    #[error("validation failed: {0}")]
    Validation(String),
    #[error(transparent)]
    InvalidTransition(#[from] nullbox_domain::InvalidTransition),
    #[error("ticket is closed")]
    TicketClosed,
    /// The database is unreachable. It's not DNS. There's no way it's DNS. It was DNS.
    #[error("storage unavailable: {0}")]
    ProbablyDns(String),
    /// Anything unexpected. The details are logged, never sent.
    #[error("internal error: {0}")]
    WorksOnMyMachine(String),
    #[error("I'm a teapot")]
    Teapot,
}

impl ApiError {
    pub fn code(&self) -> ErrorCode {
        match self {
            Self::NotFound => ErrorCode::NotFound,
            Self::MissingTicketToken => ErrorCode::MissingTicketToken,
            Self::MissingActor => ErrorCode::MissingActor,
            Self::Forbidden => ErrorCode::Forbidden,
            Self::Validation(_) => ErrorCode::ValidationFailed,
            Self::InvalidTransition(_) => ErrorCode::InvalidStatusTransition,
            Self::TicketClosed => ErrorCode::TicketClosed,
            Self::ProbablyDns(_) => ErrorCode::ServiceUnavailable,
            Self::WorksOnMyMachine(_) => ErrorCode::Internal,
            Self::Teapot => ErrorCode::ImATeapot,
        }
    }

    pub fn status(&self) -> StatusCode {
        match self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::MissingTicketToken | Self::MissingActor => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::InvalidTransition(_) | Self::TicketClosed => StatusCode::CONFLICT,
            Self::ProbablyDns(_) => StatusCode::SERVICE_UNAVAILABLE,
            Self::WorksOnMyMachine(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Teapot => StatusCode::IM_A_TEAPOT,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        // Details of unexpected failures are logged, never sent.
        let message = match &self {
            Self::ProbablyDns(detail) | Self::WorksOnMyMachine(detail) => {
                tracing::error!(error = %detail, code = ?self.code(), "request failed");
                format!("{:?}", self.code())
            }
            other => other.to_string(),
        };
        (self.status(), Json(ErrorBody { code: self.code(), message })).into_response()
    }
}

impl From<RepoError> for ApiError {
    fn from(err: RepoError) -> Self {
        match err {
            RepoError::Database(e @ (sqlx::Error::PoolTimedOut | sqlx::Error::PoolClosed | sqlx::Error::Io(_))) => {
                Self::ProbablyDns(e.to_string())
            }
            other => Self::WorksOnMyMachine(other.to_string()),
        }
    }
}

impl From<JsonRejection> for ApiError {
    fn from(rejection: JsonRejection) -> Self {
        Self::Validation(rejection.body_text())
    }
}

impl From<PathRejection> for ApiError {
    fn from(_: PathRejection) -> Self {
        Self::NotFound
    }
}

impl From<QueryRejection> for ApiError {
    fn from(rejection: QueryRejection) -> Self {
        Self::Validation(rejection.body_text())
    }
}
