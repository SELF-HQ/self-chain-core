/// SELF Chain Cryptography Module
///
/// This module provides cryptographic primitives for SELF Chain, including:
/// - Classic cryptography (ECDSA, SHA3-256)
/// - Post-quantum cryptography (Kyber, SPHINCS+)
/// - Hybrid cryptographic schemes combining both
///
/// The architecture is designed to maintain backward compatibility while
/// providing forward security against quantum computing threats.
pub mod classic;
pub mod quantum;
pub mod hybrid;
pub mod common;
pub mod delegated_keys;

// Re-exports for convenient usage
pub use classic::ecdsa::{ECDSAKeys, ECDSASignature};
pub use quantum::kyber::KyberKeys;
pub use quantum::sphincs::SphincsKeys;
pub use hybrid::{HybridKeys, HybridSignature};
pub use common::traits::{KeyPair, Signer, Verifier};
pub use delegated_keys::{MasterKey, ValidatorKey, KeyManager, KeyOperation, Revocation};

// Types used throughout the module
pub type PrivateKey = Vec<u8>;
pub type PublicKey = Vec<u8>;
pub type Signature = Vec<u8>;
pub type SharedSecret = Vec<u8>;
pub type Ciphertext = Vec<u8>;

/// Cryptographic algorithm version identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CryptoAlgorithm {
    /// Legacy ECDSA with secp256k1
    ECDSAsecp256k1 = 1,
    
    /// X25519 key exchange
    X25519 = 20,
    
    /// Kyber-768 post-quantum key encapsulation
    Kyber768 = 2,
    
    /// Kyber-1024 post-quantum key encapsulation (higher security)
    Kyber1024 = 3,
    
    /// SPHINCS+-SHA3-256f (fast variant)
    SphincsShaSha3Fast = 4,
    
    /// SPHINCS+-SHA3-256s (small variant)
    SphincsShaSha3Small = 5,
    
    /// Hybrid: ECDSA + SPHINCS+ (transition scheme)
    HybridSignature = 10,
    
    /// Hybrid: X25519 + Kyber-1024 (transition scheme)
    HybridX25519Kyber = 11,
}

/// Error types for cryptographic operations
#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("Key generation failed: {0}")]
    KeyGenerationError(String),
    
    #[error("Signing operation failed: {0}")]
    SigningError(String),
    
    #[error("Verification failed: {0}")]
    VerificationError(String),
    
    #[error("Encapsulation failed: {0}")]
    EncapsulationError(String),
    
    #[error("Decapsulation failed: {0}")]
    DecapsulationError(String),
    
    #[error("Invalid algorithm: {0}")]
    InvalidAlgorithm(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Invalid signature format: {0}")]
    InvalidSignatureFormat(String),
    
    #[error("Invalid key format: {0}")]
    InvalidKeyFormat(String),
    
    #[error("Not implemented: {0}")]
    NotImplemented(String),
}

/// Result type for cryptographic operations
pub type CryptoResult<T> = Result<T, CryptoError>;
