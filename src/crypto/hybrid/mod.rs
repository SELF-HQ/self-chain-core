/// Hybrid cryptography implementations for SELF Chain
///
/// This module provides hybrid cryptographic algorithms that combine classical
/// and post-quantum security. These hybrid schemes are designed for the transition
/// period, ensuring security against both classical and quantum attacks.
pub mod signature;
pub mod key_exchange;

// Re-exports for convenient usage
pub use signature::{HybridKeys, HybridSignature};
pub use key_exchange::HybridKeyExchange;
