use std::net::SocketAddr;

/// Every setting, read from the environment (plus `.env` in dev). See `.env.example`.
#[derive(Debug, Clone)]
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
}
