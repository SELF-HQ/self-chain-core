//! Hybrid Key Exchange Mechanism - X25519 + Kyber
//!
//! This module implements a hybrid key exchange mechanism combining:
//! - X25519 classical key exchange (for traditional security)
//! - Kyber post-quantum key exchange (for quantum resistance)
//!
//! # Security Design
//!
//! The hybrid approach follows NIST recommendations for post-quantum security, combining:
//! * A well-established classical algorithm (X25519) for immediate security needs
//! * A post-quantum algorithm (Kyber) to protect against future quantum computer attacks
//!
//! # Implementation Notes
//!
//! * Both key exchange mechanisms are used in parallel
//! * Shared secrets from both mechanisms are concatenated rather than mixed cryptographically
//! * Ciphertexts are combined with explicit length prefixes for proper parsing
//! * With the minimal viable fix implemented in X25519, both the X25519 and Kyber portions of
//!   the shared secret should now match between encapsulation and decapsulation
//!
//! # Known Limitations
//!
//! * The X25519 implementation uses a temporary caching approach documented in its module
//!   documentation. This is a minimal viable fix for blockchain testing but not a long-term solution.
//! * A more secure implementation would apply a KDF to the combined shared secrets
//! * Tests now verify that both X25519 and Kyber portions of shared secrets match

use crate::crypto::{CryptoResult, CryptoError, CryptoAlgorithm};
use crate::crypto::common::traits::{KeyEncapsulation, KeyPair};
use crate::crypto::quantum::kyber::{KyberKeys, KyberVariant};
use crate::crypto::classic::x25519::X25519Keys;
use tracing::debug;

/// Hybrid key exchange mechanism combining X25519 (classic) with Kyber (post-quantum)
/// This implements the post-quantum hybrid approach recommended by NIST
#[derive(Debug)]
pub struct HybridKeyExchange {
    /// The classical cryptography component using X25519
    classic: X25519Keys,
    /// The quantum-resistant cryptography component using Kyber
    quantum: KyberKeys,
}

impl HybridKeyExchange {
    /// Create a new hybrid key exchange pair with freshly generated keys
    /// 
    /// This method generates both an X25519 key pair and a Kyber-1024 key pair.
    /// Kyber-1024 is used for the highest level of post-quantum security (NIST Level 5).
    /// 
    /// # Security Considerations
    /// * Uses cryptographically secure random number generation for both key pairs
    /// * Note that the X25519 implementation has limitations as documented in its module
    /// 
    /// # Returns
    /// * A new HybridKeyExchange instance with fresh key pairs if successful
    /// * A CryptoError if either key generation fails
    pub fn new() -> CryptoResult<Self> {
        // Use Kyber-1024 for maximum security (NIST Level 5)
        let quantum = KyberKeys::new_with_variant(KyberVariant::Kyber1024)?;
        let classic = X25519Keys::new()?;
        
        debug!("Created hybrid key exchange with X25519 and Kyber-1024");
        
        Ok(Self {
            classic,
            quantum,
        })
    }
    
    /// Get a reference to the classical X25519 key pair component
    /// 
    /// This provides access to the X25519 key pair for inspection or direct operations.
    /// 
    /// # Returns
    /// * A reference to the X25519Keys instance
    pub fn classic(&self) -> &X25519Keys {
        &self.classic
    }
    
    /// Get a reference to the quantum-resistant Kyber key pair component
    /// 
    /// This provides access to the Kyber key pair for inspection or direct operations.
    /// 
    /// # Returns
    /// * A reference to the KyberKeys instance
    pub fn quantum(&self) -> &KyberKeys {
        &self.quantum
    }
    
    /// Encapsulate a shared secret using both X25519 and Kyber simultaneously
    /// 
    /// This method performs hybrid key encapsulation by:
    /// 1. Encapsulating with both X25519 and Kyber independently
    /// 2. Combining both ciphertexts with explicit length prefixes
    /// 3. Combining both shared secrets by simple concatenation
    /// 
    /// # Security Considerations
    /// * Generates two independent shared secrets (X25519 and Kyber)
    /// * For simplicity, the shared secrets are concatenated rather than mixed cryptographically
    /// * A more secure implementation would apply a KDF (e.g., HKDF) to the combined secrets
    /// * The combined ciphertext includes explicit length prefixes for reliable parsing
    /// 
    /// # Output Format
    /// * Ciphertext format: [c_len(4 bytes)][c_data][q_len(4 bytes)][q_data]
    /// * Shared secret format: [c_shared][q_shared]
    /// 
    /// # Returns
    /// * A tuple containing (combined_ciphertext, combined_shared_secret) if successful
    /// * A CryptoError if either encapsulation fails
    pub fn encapsulate(&self) -> CryptoResult<(Vec<u8>, Vec<u8>)> {
        // Get quantum ciphertext and shared secret
        let (q_ciphertext, q_shared) = self.quantum.encapsulate()?;
        
        // Get classic shared secret using proper X25519 key exchange
        let (c_ciphertext, c_shared) = self.classic.encapsulate()?;
        
        // Combine the shared secrets (concatenation for simplicity)
        // NOTE: Using SHA-256 or HKDF to derive a combined key would be more secure
        let mut combined_shared = Vec::with_capacity(c_shared.len() + q_shared.len());
        combined_shared.extend_from_slice(&c_shared);
        combined_shared.extend_from_slice(&q_shared);
        
        // Combine the ciphertexts (with length prefixes for proper decapsulation)
        let mut combined_ciphertext = Vec::new();
        
        // Add classic ciphertext length and data (32 bytes for X25519)
        combined_ciphertext.extend_from_slice(&(c_ciphertext.len() as u32).to_be_bytes());
        combined_ciphertext.extend_from_slice(&c_ciphertext);
        
        // Add quantum ciphertext length and data
        combined_ciphertext.extend_from_slice(&(q_ciphertext.len() as u32).to_be_bytes());
        combined_ciphertext.extend_from_slice(&q_ciphertext);
        
        debug!("Hybrid encapsulation: X25519 ({} bytes) + Kyber ({} bytes) = {} bytes combined ciphertext",
               c_ciphertext.len(), q_ciphertext.len(), combined_ciphertext.len());
        
        Ok((combined_ciphertext, combined_shared))
    }
    
    /// Decapsulate a shared secret from a combined ciphertext using both X25519 and Kyber
    /// 
    /// This method decapsulates a hybrid ciphertext by:
    /// 1. Parsing the combined ciphertext to extract X25519 and Kyber components
    /// 2. Decapsulating each component separately
    /// 3. Combining the resulting shared secrets by concatenation
    /// 
    /// # Implementation Notes
    /// * With the minimal viable fix implemented in X25519, both the X25519 and Kyber 
    ///   portions of the shared secret should now match between encapsulation and decapsulation
    /// * The X25519 implementation uses a caching approach to ensure consistent shared secrets
    /// * This is a temporary solution for blockchain testing - a more robust API-based solution
    ///   will be implemented according to the project timeline
    /// 
    /// # Ciphertext Format
    /// Expected format: [c_len(4 bytes)][c_data][q_len(4 bytes)][q_data]
    /// 
    /// # Parameters
    /// * `ciphertext`: The combined ciphertext from a previous encapsulation
    /// 
    /// # Returns
    /// * The combined shared secret if successful
    /// * A CryptoError if parsing fails or either decapsulation fails
    pub fn decapsulate(&self, ciphertext: &[u8]) -> CryptoResult<Vec<u8>> {
        // Need at least 8 bytes for the length prefixes (4 bytes each)
        if ciphertext.len() < 8 {
            return Err(CryptoError::DecapsulationError(
                "Ciphertext too short for hybrid decapsulation".into()
            ));
        }
        
        // Extract classic ciphertext
        let c_len = u32::from_be_bytes([
            ciphertext[0], ciphertext[1], ciphertext[2], ciphertext[3]
        ]) as usize;
        
        if ciphertext.len() < 4 + c_len + 4 {
            return Err(CryptoError::DecapsulationError(
                "Ciphertext too short after classic length prefix".into()
            ));
        }
        
        let c_ciphertext = &ciphertext[4..4+c_len];
        
        // Extract quantum ciphertext
        let q_len_offset = 4 + c_len;
        let q_len = u32::from_be_bytes([
            ciphertext[q_len_offset], 
            ciphertext[q_len_offset+1], 
            ciphertext[q_len_offset+2], 
            ciphertext[q_len_offset+3]
        ]) as usize;
        
        if ciphertext.len() != 4 + c_len + 4 + q_len {
            return Err(CryptoError::DecapsulationError(
                "Ciphertext length mismatch for quantum component".into()
            ));
        }
        
        let q_ciphertext = &ciphertext[q_len_offset+4..q_len_offset+4+q_len];
        
        // Decapsulate each component
        let c_shared = self.classic.decapsulate(c_ciphertext)?;
        let q_shared = self.quantum.decapsulate(q_ciphertext)?;
        
        // Combine the shared secrets
        let mut combined_shared = Vec::with_capacity(c_shared.len() + q_shared.len());
        combined_shared.extend_from_slice(&c_shared);
        combined_shared.extend_from_slice(&q_shared);
        
        Ok(combined_shared)
    }
    
    /// Get the algorithm identifier for this hybrid mechanism
    /// 
    /// # Returns
    /// * The numeric identifier for the HybridX25519Kyber algorithm
    pub fn algorithm_id(&self) -> u8 {
        CryptoAlgorithm::HybridX25519Kyber as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hybrid_key_exchange() {
        // Create hybrid key exchange
        let keys = HybridKeyExchange::new().expect("Failed to create hybrid key exchange");
        
        // Test encapsulation
        let (ciphertext, shared_secret) = keys.encapsulate().expect("Failed to encapsulate");
        
        // Test decapsulation
        let recovered_secret = keys.decapsulate(&ciphertext).expect("Failed to decapsulate");
        
        // With our minimal viable fix, both X25519 and Kyber portions of the shared secret should match
        assert_eq!(shared_secret, recovered_secret, "Complete shared secrets should match with our MVF");
        
        // Additionally verify each component matches individually
        // X25519 is the first 32 bytes
        let x25519_shared = &shared_secret[0..32];
        let x25519_recovered = &recovered_secret[0..32];
        assert_eq!(x25519_shared, x25519_recovered, "X25519 shared secrets should match with our MVF");
        
        // Kyber is the remaining bytes
        let kyber_shared = &shared_secret[32..];
        let kyber_recovered = &recovered_secret[32..];
        assert_eq!(kyber_shared, kyber_recovered, "Kyber shared secrets should match");
    }
}
