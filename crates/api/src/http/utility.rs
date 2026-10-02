use axum::Json;
use serde_json::{Value, json};

use crate::ApiError;

pub async fn healthz() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

/// RFC 2324 compliance is not optional.
pub async fn teapot() -> ApiError {
    ApiError::Teapot
}

pub async fn not_found() -> ApiError {
    ApiError::NotFound
}
