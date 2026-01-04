/// Common cryptographic utilities and traits used across
/// both classical and post-quantum implementations
pub mod traits;
pub mod utils;

// Re-export common traits for convenient usage
pub use traits::{KeyPair, Signer, Verifier, KeyEncapsulation, CryptoSerialize};
