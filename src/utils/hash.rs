use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use sha2::{Digest, Sha256};

/// Computes the SHA-256 hash of raw bytes and returns a lowercase hex string.
pub fn hash_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Computes the SHA-256 hash of base64-encoded content.
/// Returns `Ok(hex_hash)` on success, or `Err` if the base64 input is invalid.
pub fn hash_base64(encoded: &str) -> Result<String, base64::DecodeError> {
    let bytes = BASE64.decode(encoded.trim())?;
    Ok(hash_bytes(&bytes))
}
