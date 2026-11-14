
use argon2::{
    password_hash::{
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
    },
    Argon2,
};
use axum::{
    extract::State,
    http::{header, HeaderMap, HeaderValue},
    Json,
};
use chrono::{Duration, Utc};
use rand::RngExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{ApiError, AppState};
use super::extract::Actor;

const COOKIE_NAME: &str = "nullbox_session";

#[derive(Deserialize)]
pub struct Credentials {
    username: String,
    password: String,
}

#[derive(Serialize)]
pub struct UserResponse {
    id: String,
    username: String,
    display_name: String,
}

#[derive(sqlx::FromRow)]
struct UserRow {
    id: String,
    username: String,
    display_name: String,
    password_hash: Option<String>,
}

fn hash_token(token: &str) -> String {
    Sha256::digest(token.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn validate_password(password: &str) -> Result<(), ApiError> {
    if password.chars().count() < 12 {
        return Err(ApiError::Validation(
            "Password must be at least 12 characters".into(),
        ));
    }

    if password.len() > 1024 {
        return Err(ApiError::Validation("Password is too long".into()));
    }

    // Basic local blocklist only. A breached-password lookup
    // is still required for the full hackathon criterion.
    let common = [
        "password123456",
        "123456789012",
        "qwerty123456",
        "admin12345678",
    ];

    if common.contains(&password.to_lowercase().as_str()) {
        return Err(ApiError::Validation(
            "Password is too common".into(),
        ));
    }

    Ok(())
}

async fn create_session(
    state: &AppState,
    user_id: &str,
) -> Result<HeaderMap, ApiError> {
    let mut bytes = [0u8; 32];
    rand::rng().fill(&mut bytes);

    let token = base64::Engine::encode(
        &base64::engine::general_purpose::URL_SAFE_NO_PAD,
        bytes,
    );

    let now = Utc::now();
    let expires = now + Duration::days(7);

    sqlx::query(
        "INSERT INTO sessions
         (id, user_id, token_hash, expires_at, created_at)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(Uuid::now_v7().to_string())
    .bind(user_id)
    .bind(hash_token(&token))
    .bind(expires.to_rfc3339())
    .bind(now.to_rfc3339())
    .execute(&state.pool)
    .await
    .map_err(|e| ApiError::WorksOnMyMachine(e.to_string()))?;

    let secure = "";

    let cookie = format!(
        "{COOKIE_NAME}={token}; HttpOnly; Path=/api; SameSite=Lax; Max-Age=604800{secure}"
    );

    let mut headers = HeaderMap::new();
    headers.insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie)
            .map_err(|e| ApiError::WorksOnMyMachine(e.to_string()))?,
    );

    Ok(headers)
}

pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<Credentials>,
) -> Result<(HeaderMap, Json<UserResponse>), ApiError> {
    let username = body.username.trim();

    if username.len() < 3
        || username.len() > 32
        || !username
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err(ApiError::Validation(
            "Username must be 3-32 characters (letters, numbers, _ or -)".into(),
        ));
    }

    validate_password(&body.password)?;

    let mut salt_bytes = [0u8; 16];
    rand::rng().fill(&mut salt_bytes);

    let salt = SaltString::encode_b64(&salt_bytes)
        .map_err(|e| ApiError::WorksOnMyMachine(e.to_string()))?;
    let password_hash = Argon2::default()
        .hash_password(body.password.as_bytes(), &salt)
        .map_err(|e| ApiError::WorksOnMyMachine(e.to_string()))?
        .to_string();

    let id = Uuid::now_v7().to_string();

    let result = sqlx::query(
        "INSERT INTO users
         (id, username, password_hash, display_name, created_at)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(username)
    .bind(password_hash)
    .bind(username)
    .bind(Utc::now().to_rfc3339())
    .execute(&state.pool)
    .await;

    match result {
        Ok(_) => {}
        Err(sqlx::Error::Database(e)) if e.is_unique_violation() => {
            return Err(ApiError::Validation(
                "Username already exists".into(),
            ));
        }
        Err(e) => return Err(ApiError::WorksOnMyMachine(e.to_string())),
    }

    let headers = create_session(&state, &id).await?;

    Ok((
        headers,
        Json(UserResponse {
            id,
            username: username.to_owned(),
            display_name: username.to_owned(),
        }),
    ))
}

pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<Credentials>,
) -> Result<(HeaderMap, Json<UserResponse>), ApiError> {
    let user = sqlx::query_as::<_, UserRow>(
        "SELECT id, username, display_name, password_hash
         FROM users WHERE username = ? COLLATE NOCASE",
    )
    .bind(body.username.trim())
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| ApiError::WorksOnMyMachine(e.to_string()))?
    .ok_or(ApiError::InvalidCredentials)?;

    let stored_hash = user
        .password_hash
        .as_deref()
        .ok_or(ApiError::InvalidCredentials)?;

    let parsed = PasswordHash::new(stored_hash)
        .map_err(|_| ApiError::InvalidCredentials)?;

    Argon2::default()
        .verify_password(body.password.as_bytes(), &parsed)
        .map_err(|_| ApiError::InvalidCredentials)?;

    let headers = create_session(&state, &user.id).await?;

    Ok((
        headers,
        Json(UserResponse {
            id: user.id,
            username: user.username,
            display_name: user.display_name,
        }),
    ))
}

pub async fn me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<UserResponse>, ApiError> {
    let user = authenticated_user(&state, &headers).await?;
    Ok(Json(user))
}

pub async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<HeaderMap, ApiError> {
    if let Some(token) = session_cookie(&headers) {
        sqlx::query("DELETE FROM sessions WHERE token_hash = ?")
            .bind(hash_token(&token))
            .execute(&state.pool)
            .await
            .map_err(|e| ApiError::WorksOnMyMachine(e.to_string()))?;
    }

    let mut response_headers = HeaderMap::new();
    response_headers.insert(
        header::SET_COOKIE,
        HeaderValue::from_static(
            "nullbox_session=; HttpOnly; Path=/api; SameSite=Lax; Max-Age=0",
        ),
    );

    Ok(response_headers)
}

fn session_cookie(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .filter_map(|part| part.trim().split_once('='))
        .find(|(name, _)| *name == COOKIE_NAME)
        .map(|(_, value)| value.to_owned())
}

async fn authenticated_user(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<UserResponse, ApiError> {
    let token = session_cookie(headers).ok_or(ApiError::Unauthorized)?;

    let user = sqlx::query_as::<_, UserRow>(
        "SELECT u.id, u.username, u.display_name, u.password_hash
         FROM users u
         JOIN sessions s ON s.user_id = u.id
         WHERE s.token_hash = ?
           AND s.expires_at > ?",
    )
    .bind(hash_token(&token))
    .bind(Utc::now().to_rfc3339())
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| ApiError::WorksOnMyMachine(e.to_string()))?
    .ok_or(ApiError::Unauthorized)?;

    Ok(UserResponse {
        id: user.id,
        username: user.username,
        display_name: user.display_name,
    })
}

pub async fn authenticated_actor(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<Actor, ApiError> {
    let user = authenticated_user(state, headers).await?;

    Ok(Actor {
        name: user.display_name,
    })
}
