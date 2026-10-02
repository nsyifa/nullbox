//! Secret access tokens for customers: the only "auth" in the MVP.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand::TryRng;
use rand::rngs::SysRng;
use sha2::{Digest, Sha256};

/// 32 random bytes from the OS RNG, base64url-encoded. Never stored.
pub fn generate() -> Result<String, rand::rngs::SysError> {
    let mut bytes = [0u8; 32];
    SysRng.try_fill_bytes(&mut bytes)?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

/// SHA-256 (hex) of a token. This is what the database stores and looks up.
pub fn hash(token: &str) -> String {
    Sha256::digest(token.as_bytes()).iter().map(|b| format!("{b:02x}")).collect()
}
