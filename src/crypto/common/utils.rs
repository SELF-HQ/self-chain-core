use sha3::{Digest, Sha3_256};
// Import OsRng and RngCore from rand v0.8 which is compatible with x25519-dalek
use rand_0_8::rngs::OsRng;
use rand_0_8::RngCore;

use crate::crypto::CryptoError;

/// Generate a secure random byte array of specified length
pub fn random_bytes(length: usize) -> Vec<u8> {
    let mut bytes = vec![0u8; length];
    OsRng.fill_bytes(&mut bytes);
    bytes
}

/// Generate a deterministic seed from an input seed and context
/// using SHA3-256 for domain separation
pub fn derive_seed(seed: &[u8], context: &str) -> Vec<u8> {
    let mut hasher = Sha3_256::new();
    hasher.update(context.as_bytes());
    hasher.update(seed);
    hasher.finalize().to_vec()
}

/// Hash data using SHA3-256
pub fn hash_sha3_256(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha3_256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// Constant-time comparison to avoid timing attacks (critical for crypto code)
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }

    result == 0
}

/// Version byte layout: [algorithm_id: 5 bits][version: 3 bits]
/// This allows for 32 different algorithms and 8 versions per algorithm
pub fn create_version_byte(algorithm_id: u8, version: u8) -> u8 {
    ((algorithm_id & 0x1F) << 3) | (version & 0x07)
}

/// Extract algorithm ID from a version byte
pub fn extract_algorithm_id(version_byte: u8) -> u8 {
    (version_byte >> 3) & 0x1F
}

/// Extract version from a version byte
pub fn extract_version(version_byte: u8) -> u8 {
    version_byte & 0x07
}

/// Securely wipe sensitive data from memory
pub fn secure_wipe(data: &mut [u8]) {
    // Simple implementation - in production, use a specialized crate for this
    for byte in data.iter_mut() {
        *byte = 0;
    }
}

/// Safely handle cryptographic errors with consistent logging and wrapping
pub fn handle_crypto_error<T>(result: Result<T, impl std::fmt::Display>, context: &str) -> Result<T, CryptoError> {
    result.map_err(|e| {
        // Log error details (with appropriate log level in production)
        tracing::error!("Crypto operation failed in {}: {}", context, e);
        
        // Return a generic error without exposing internal details
        CryptoError::SigningError(context.to_string())
    })
}
