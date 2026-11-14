use std::net::SocketAddr;

/// Every setting, read from the environment (plus `.env` in dev). See `.env.example`.
///
/// `Debug` is written by hand so secrets never reach a log line.
#[derive(Clone)]
pub struct Config {
    /// `BIND_ADDR`, e.g. `0.0.0.0:3000` in a container.
    pub bind_addr: SocketAddr,
    /// `DATABASE_URL`, e.g. `sqlite://nullbox.db`.
    pub database_url: String,
    /// `CORS_ALLOWED_ORIGINS`, comma-separated. The web app runs on another origin (Vite, Tauri).
    pub cors_allowed_origins: Vec<String>,
    /// `PUBLIC_WEB_URL`: where the web app lives; used to build secret access links.
    pub public_web_url: String,
    /// `ADMIN_TOKEN`: unlocks `GET /api/admin/export`, the CEO's spreadsheet.
    pub admin_token: String,
    /// `MCP_TOKEN`: bearer token for the MCP server at `/mcp`. Unset means `/mcp` refuses everyone.
    pub mcp_token: Option<String>,
    /// `WORKER_TOKEN`: bearer token for the normal worker identity. Optional, no default:
    /// unset means no one can sign in as a worker (fail closed), but the server still starts.
    pub worker_token: Option<String>,
    /// `HOT_WORKER_TOKEN`: bearer token for the separate HOT worker identity. Same rules.
    pub hot_worker_token: Option<String>,
}

impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let redact = |set: bool| if set { "<redacted>" } else { "<unset>" };
        f.debug_struct("Config")
            .field("bind_addr", &self.bind_addr)
            .field("database_url", &self.database_url)
            .field("cors_allowed_origins", &self.cors_allowed_origins)
            .field("public_web_url", &self.public_web_url)
            .field("admin_token", &redact(true))
            .field("mcp_token", &redact(self.mcp_token.is_some()))
            .field("worker_token", &redact(self.worker_token.is_some()))
            .field("hot_worker_token", &redact(self.hot_worker_token.is_some()))
            .finish()
    }
}

#[derive(Debug, thiserror::Error)]
#[error("invalid config value for {name}: {reason}")]
pub struct ConfigError {
    pub name: &'static str,
    pub reason: String,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let var = |name: &str, default: &str| std::env::var(name).unwrap_or_else(|_| default.to_owned());

        let bind_addr = var("BIND_ADDR", "127.0.0.1:3000")
            .parse()
            .map_err(|e: std::net::AddrParseError| ConfigError { name: "BIND_ADDR", reason: e.to_string() })?;

        let worker_token = optional_token("WORKER_TOKEN", std::env::var("WORKER_TOKEN").ok())?;
        let hot_worker_token = optional_token("HOT_WORKER_TOKEN", std::env::var("HOT_WORKER_TOKEN").ok())?;
        ensure_distinct_worker_tokens(worker_token.as_deref(), hot_worker_token.as_deref())?;

        Ok(Self {
            bind_addr,
            database_url: var("DATABASE_URL", "sqlite://nullbox.db"),
            cors_allowed_origins: var("CORS_ALLOWED_ORIGINS", "http://localhost:5173")
                .split(',')
                .map(|origin| origin.trim().to_owned())
                .filter(|origin| !origin.is_empty())
                .collect(),
            public_web_url: var("PUBLIC_WEB_URL", "http://localhost:5173").trim_end_matches('/').to_owned(),
            admin_token: require_admin_token(std::env::var("ADMIN_TOKEN").ok())?,
            mcp_token: std::env::var("MCP_TOKEN").ok().map(|t| t.trim().to_owned()).filter(|t| !t.is_empty()),
            worker_token,
            hot_worker_token,
        })
    }
}

/// `ADMIN_TOKEN` has no default: a missing or blank value is a startup error.
fn require_admin_token(value: Option<String>) -> Result<String, ConfigError> {
    match value {
        Some(token) if !token.trim().is_empty() => Ok(token),
        Some(_) => Err(ConfigError { name: "ADMIN_TOKEN", reason: "must not be blank".to_owned() }),
        None => Err(ConfigError {
            name: "ADMIN_TOKEN",
            reason: "is required; set it in the server environment".to_owned(),
        }),
    }
}

/// A worker token is optional, but if it is set it must not be blank.
fn optional_token(name: &'static str, value: Option<String>) -> Result<Option<String>, ConfigError> {
    match value {
        None => Ok(None),
        Some(token) if token.trim().is_empty() => {
            Err(ConfigError { name, reason: "must not be blank when set; unset it to disable".to_owned() })
        }
        Some(token) => Ok(Some(token)),
    }
}

/// The normal worker and HOT are separate identities, so one token cannot be both.
fn ensure_distinct_worker_tokens(worker: Option<&str>, hot: Option<&str>) -> Result<(), ConfigError> {
    match (worker, hot) {
        (Some(w), Some(h)) if w == h => {
            Err(ConfigError { name: "HOT_WORKER_TOKEN", reason: "must differ from WORKER_TOKEN".to_owned() })
        }
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admin_token_missing_is_rejected() {
        let err = require_admin_token(None).unwrap_err();
        assert_eq!(err.name, "ADMIN_TOKEN");
        assert!(err.reason.contains("required"));
    }

    #[test]
    fn admin_token_blank_is_rejected() {
        for blank in ["", "   ", "\t\n"] {
            let err = require_admin_token(Some(blank.to_owned())).unwrap_err();
            assert_eq!(err.name, "ADMIN_TOKEN");
            assert!(err.reason.contains("blank"));
        }
    }

    #[test]
    fn admin_token_valid_is_accepted() {
        assert_eq!(require_admin_token(Some("test-token".to_owned())).unwrap(), "test-token");
    }

    #[test]
    fn worker_token_absent_is_allowed() {
        assert_eq!(optional_token("WORKER_TOKEN", None).unwrap(), None);
    }

    #[test]
    fn worker_token_blank_is_rejected() {
        for blank in ["", "   ", "\t\n"] {
            let err = optional_token("WORKER_TOKEN", Some(blank.to_owned())).unwrap_err();
            assert_eq!(err.name, "WORKER_TOKEN");
            assert!(err.reason.contains("blank"));
        }
    }

    #[test]
    fn worker_and_hot_tokens_must_differ() {
        assert!(ensure_distinct_worker_tokens(Some("a"), Some("b")).is_ok());
        assert!(ensure_distinct_worker_tokens(Some("a"), None).is_ok());
        let err = ensure_distinct_worker_tokens(Some("same"), Some("same")).unwrap_err();
        assert_eq!(err.name, "HOT_WORKER_TOKEN");
    }

    #[test]
    fn debug_never_prints_tokens() {
        let config = Config {
            bind_addr: "127.0.0.1:0".parse().unwrap(),
            database_url: "sqlite::memory:".into(),
            cors_allowed_origins: vec![],
            public_web_url: String::new(),
            admin_token: "synthetic-admin".into(),
            mcp_token: Some("synthetic-mcp".into()),
            worker_token: Some("synthetic-worker".into()),
            hot_worker_token: None,
        };
        let printed = format!("{config:?}");
        assert!(!printed.contains("synthetic-admin"));
        assert!(!printed.contains("synthetic-worker"));
        assert!(!printed.contains("synthetic-mcp"));
        assert!(printed.contains("<unset>"));
    }
}
