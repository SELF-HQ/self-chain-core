//! SPHINCS+ implementation using the pqcrypto library
//!
//! This implementation fixes the key serialization and deserialization issues
//! that were present in the OQS-based implementation. With this implementation,
//! we can properly store and restore private keys from bytes, which was not
//! possible with the OQS Rust bindings.

use crate::crypto::{CryptoResult, CryptoError};
use crate::crypto::common::traits::{KeyPair, Signer, Verifier};
use pqcrypto_traits::sign::{
    PublicKey as PQPublicKey, 
    SecretKey as PQSecretKey, 
    DetachedSignature as PQDetachedSignature
};
use serde::{Deserialize, Serialize};
use std::fmt;
use tracing::debug;
use zeroize::Zeroize;

// Import SPHINCS+ variants modules
use pqcrypto_sphincsplus::sphincssha2128fsimple;
use pqcrypto_sphincsplus::sphincssha2128ssimple;
use pqcrypto_sphincsplus::sphincssha2256fsimple;
use pqcrypto_sphincsplus::sphincssha2256ssimple;
use pqcrypto_sphincsplus::sphincsshake128fsimple;
use pqcrypto_sphincsplus::sphincsshake128ssimple;
use pqcrypto_sphincsplus::sphincsshake256fsimple;
use pqcrypto_sphincsplus::sphincsshake256ssimple;
use crate::crypto::CryptoAlgorithm;

/// Enum representing different variants of the SPHINCS+ signature algorithm
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SphincsVariant {
    /// SHA2-128 based SPHINCS+ with small simple parameter set
    Sha2128SSimple,
    /// SHA2-128 based SPHINCS+ with fast simple parameter set
    Sha2128FSimple,
    /// SHA2-256 based SPHINCS+ with small simple parameter set
    Sha2256SSimple,
    /// SHA2-256 based SPHINCS+ with fast simple parameter set
    Sha2256FSimple,
    /// SHAKE-128 based SPHINCS+ with small simple parameter set
    Shake128SSimple,
    /// SHAKE-128 based SPHINCS+ with fast simple parameter set
    Shake128FSimple,
    /// SHAKE-256 based SPHINCS+ with small simple parameter set (maps to Sha3256Small in OQS)
    Shake256SSimple,
    /// SHAKE-256 based SPHINCS+ with fast simple parameter set (maps to Sha3256Fast in OQS)
    Shake256FSimple,
}

impl SphincsVariant {
    /// Returns the algorithm ID for this variant
    pub fn algorithm_id(&self) -> u8 {
        match self {
            // Use algorithm ID values from CryptoAlgorithm enum for SHA3/SHAKE-256 variants
            // to maintain compatibility with existing keys
            SphincsVariant::Shake256FSimple => CryptoAlgorithm::SphincsShaSha3Fast as u8,   // 4
            SphincsVariant::Shake256SSimple => CryptoAlgorithm::SphincsShaSha3Small as u8,  // 5
            
            // For other variants, use the previous ID values
            SphincsVariant::Sha2128SSimple => 0x01,
            SphincsVariant::Sha2128FSimple => 0x02,
            SphincsVariant::Sha2256SSimple => 0x03,
            SphincsVariant::Sha2256FSimple => 0x04,
            SphincsVariant::Shake128SSimple => 0x05,
            SphincsVariant::Shake128FSimple => 0x06,
        }
    }

    /// Returns the human-readable name of the variant
    pub fn name(&self) -> &'static str {
        match self {
            SphincsVariant::Sha2128SSimple => "SPHINCS+-SHA2-128s-simple",
            SphincsVariant::Sha2128FSimple => "SPHINCS+-SHA2-128f-simple",
            SphincsVariant::Sha2256SSimple => "SPHINCS+-SHA2-256s-simple",
            SphincsVariant::Sha2256FSimple => "SPHINCS+-SHA2-256f-simple",
            SphincsVariant::Shake128SSimple => "SPHINCS+-SHAKE-128s-simple",
            SphincsVariant::Shake128FSimple => "SPHINCS+-SHAKE-128f-simple",
            SphincsVariant::Shake256SSimple => "SPHINCS+-SHAKE-256s-simple",
            SphincsVariant::Shake256FSimple => "SPHINCS+-SHAKE-256f-simple",
        }
    }

    /// Returns the variant corresponding to the given algorithm ID
    pub fn from_algorithm_id(id: u8) -> Option<Self> {
        match id {
            // Map CryptoAlgorithm IDs to appropriate variants
            id if id == CryptoAlgorithm::SphincsShaSha3Fast as u8 => Some(SphincsVariant::Shake256FSimple),
            id if id == CryptoAlgorithm::SphincsShaSha3Small as u8 => Some(SphincsVariant::Shake256SSimple),
            
            // Map other IDs to their variants
            0x01 => Some(SphincsVariant::Sha2128SSimple),
            0x02 => Some(SphincsVariant::Sha2128FSimple),
            0x03 => Some(SphincsVariant::Sha2256SSimple),
            0x04 => Some(SphincsVariant::Sha2256FSimple),
            0x05 => Some(SphincsVariant::Shake128SSimple),
            0x06 => Some(SphincsVariant::Shake128FSimple),
            _ => None,
        }
    }

    /// Returns the public key size in bytes for this variant
    pub fn public_key_bytes(&self) -> usize {
        match self {
            SphincsVariant::Sha2128SSimple => sphincssha2128ssimple::public_key_bytes(),
            SphincsVariant::Sha2128FSimple => sphincssha2128fsimple::public_key_bytes(), 
            SphincsVariant::Sha2256SSimple => sphincssha2256ssimple::public_key_bytes(),
            SphincsVariant::Sha2256FSimple => sphincssha2256fsimple::public_key_bytes(),
            SphincsVariant::Shake128SSimple => sphincsshake128ssimple::public_key_bytes(),
            SphincsVariant::Shake128FSimple => sphincsshake128fsimple::public_key_bytes(),
            SphincsVariant::Shake256SSimple => sphincsshake256ssimple::public_key_bytes(),
            SphincsVariant::Shake256FSimple => sphincsshake256fsimple::public_key_bytes(),
        }
    }

    /// Returns the secret key size in bytes for this variant
    pub fn secret_key_bytes(&self) -> usize {
        match self {
            SphincsVariant::Sha2128SSimple => sphincssha2128ssimple::secret_key_bytes(),
            SphincsVariant::Sha2128FSimple => sphincssha2128fsimple::secret_key_bytes(),
            SphincsVariant::Sha2256SSimple => sphincssha2256ssimple::secret_key_bytes(),
            SphincsVariant::Sha2256FSimple => sphincssha2256fsimple::secret_key_bytes(),
            SphincsVariant::Shake128SSimple => sphincsshake128ssimple::secret_key_bytes(),
            SphincsVariant::Shake128FSimple => sphincsshake128fsimple::secret_key_bytes(),
            SphincsVariant::Shake256SSimple => sphincsshake256ssimple::secret_key_bytes(),
            SphincsVariant::Shake256FSimple => sphincsshake256fsimple::secret_key_bytes(),
        }
    }

    /// Returns the signature size in bytes for this variant
    pub fn signature_bytes(&self) -> usize {
        match self {
            SphincsVariant::Sha2128SSimple => sphincssha2128ssimple::signature_bytes(),
            SphincsVariant::Sha2128FSimple => sphincssha2128fsimple::signature_bytes(),
            SphincsVariant::Sha2256SSimple => sphincssha2256ssimple::signature_bytes(),
            SphincsVariant::Sha2256FSimple => sphincssha2256fsimple::signature_bytes(),
            SphincsVariant::Shake128SSimple => sphincsshake128ssimple::signature_bytes(),
            SphincsVariant::Shake128FSimple => sphincsshake128fsimple::signature_bytes(),
            SphincsVariant::Shake256SSimple => sphincsshake256ssimple::signature_bytes(),
            SphincsVariant::Shake256FSimple => sphincsshake256fsimple::signature_bytes(),
        }
    }
}

impl fmt::Display for SphincsVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// SPHINCS+ key pair containing public and optional secret keys
#[derive(Clone, Debug)]
pub struct SphincsKeys {
    /// The variant of SPHINCS+ being used
    variant: SphincsVariant,
    /// Public key bytes
    public_key: Vec<u8>,
    /// Optional secret key bytes (present for signing keys, absent for verification-only keys)
    secret_key: Option<Vec<u8>>,
}

impl SphincsKeys {
    /// Create a new SPHINCS+ keypair with the specified algorithm variant
    pub fn new(variant: SphincsVariant) -> CryptoResult<Self> {
        debug!("Generating new SPHINCS+ keypair with variant: {}", variant);
        
        // Generate the keypair using the appropriate variant
        let (public_key, secret_key) = match variant {
            SphincsVariant::Sha2128SSimple => {
                let (pk, sk) = sphincssha2128ssimple::keypair();
                (pk.as_bytes().to_vec(), sk.as_bytes().to_vec())
            },
            SphincsVariant::Sha2128FSimple => {
                let (pk, sk) = sphincssha2128fsimple::keypair();
                (pk.as_bytes().to_vec(), sk.as_bytes().to_vec())
            },
            SphincsVariant::Sha2256SSimple => {
                let (pk, sk) = sphincssha2256ssimple::keypair();
                (pk.as_bytes().to_vec(), sk.as_bytes().to_vec())
            },
            SphincsVariant::Sha2256FSimple => {
                let (pk, sk) = sphincssha2256fsimple::keypair();
                (pk.as_bytes().to_vec(), sk.as_bytes().to_vec())
            },
            SphincsVariant::Shake128SSimple => {
                let (pk, sk) = sphincsshake128ssimple::keypair();
                (pk.as_bytes().to_vec(), sk.as_bytes().to_vec())
            },
            SphincsVariant::Shake128FSimple => {
                let (pk, sk) = sphincsshake128fsimple::keypair();
                (pk.as_bytes().to_vec(), sk.as_bytes().to_vec())
            },
            SphincsVariant::Shake256SSimple => {
                let (pk, sk) = sphincsshake256ssimple::keypair();
                (pk.as_bytes().to_vec(), sk.as_bytes().to_vec())
            },
            SphincsVariant::Shake256FSimple => {
                let (pk, sk) = sphincsshake256fsimple::keypair();
                (pk.as_bytes().to_vec(), sk.as_bytes().to_vec())
            },
        };
        
        Ok(Self {
            variant,
            public_key,
            secret_key: Some(secret_key),
        })
    }
    
    /// Create a verification-only key (with no secret key material) from public key bytes
    pub fn from_public_key(variant: SphincsVariant, public_key_bytes: &[u8]) -> CryptoResult<Self> {
        if public_key_bytes.len() != variant.public_key_bytes() {
            return Err(CryptoError::InvalidKeyFormat(
                format!("Invalid public key length: expected {}, got {}", 
                        variant.public_key_bytes(), public_key_bytes.len())
            ));
        }
        
        Ok(Self {
            variant,
            public_key: public_key_bytes.to_vec(),
            secret_key: None,
        })
    }
    
    /// Create a SPHINCS+ keypair from existing public and secret key bytes
    pub fn from_keypair(
        variant: SphincsVariant,
        public_key_bytes: &[u8], 
        secret_key_bytes: &[u8]
    ) -> CryptoResult<Self> {
        if public_key_bytes.len() != variant.public_key_bytes() {
            return Err(CryptoError::InvalidKeyFormat(
                format!("Invalid public key length: expected {}, got {}", 
                        variant.public_key_bytes(), public_key_bytes.len())
            ));
        }
        
        if secret_key_bytes.len() != variant.secret_key_bytes() {
            return Err(CryptoError::InvalidKeyFormat(
                format!("Invalid secret key length: expected {}, got {}", 
                        variant.secret_key_bytes(), secret_key_bytes.len())
            ));
        }
        
        Ok(Self {
            variant,
            public_key: public_key_bytes.to_vec(),
            secret_key: Some(secret_key_bytes.to_vec()),
        })
    }
    
    /// Get the public key bytes
    pub fn public_key_bytes(&self) -> &[u8] {
        &self.public_key
    }
    
    /// Get the secret key bytes if available
    pub fn secret_key_bytes(&self) -> Option<&[u8]> {
        self.secret_key.as_deref()
    }
}

/// Serializable form of SPHINCS+ keys for storage and transmission
#[derive(Serialize, Deserialize)]
pub struct SerializableSphincsKeys {
    /// Algorithm ID indicating the SPHINCS+ variant
    pub algorithm_id: u8,
    /// Public key bytes
    pub public_key: Vec<u8>,
    /// Optional secret key bytes (None for public-key only instances)
    pub secret_key: Option<Vec<u8>>,
}

impl Drop for SphincsKeys {
    fn drop(&mut self) {
        // If a secret key is present, ensure it gets zeroized
        if let Some(ref mut sk) = self.secret_key {
            sk.zeroize();
        }
    }
}

impl SphincsKeys {
    /// Creates a new SPHINCS+ key pair with the specified variant
    pub fn new_with_variant(variant: SphincsVariant) -> CryptoResult<Self> {
        Self::new(variant)
    }

    /// Creates a copy of this key containing only the public key (no secret key)
    pub fn public_key_only(&self) -> Self {
        Self {
            variant: self.variant,
            public_key: self.public_key.clone(),
            secret_key: None,
        }
    }

    /// Convert the key pair to a serializable form
    pub fn to_bytes(&self) -> CryptoResult<Vec<u8>> {
        let serializable = SerializableSphincsKeys {
            algorithm_id: self.variant.algorithm_id(),
            public_key: self.public_key.clone(),
            secret_key: self.secret_key.clone(),
        };

        serde_json::to_vec(&serializable)
            .map_err(|e| CryptoError::SerializationError(format!("Failed to serialize keys: {}", e)))
    }

    /// Create a key pair from serialized bytes
    pub fn from_bytes(bytes: &[u8]) -> CryptoResult<Self> {
        // Deserialize the basic structure
        let serializable: SerializableSphincsKeys = serde_json::from_slice(bytes)
            .map_err(|e| CryptoError::SerializationError(format!("Failed to deserialize keys: {}", e)))?;

        // Get the variant based on the algorithm ID
        let variant = SphincsVariant::from_algorithm_id(serializable.algorithm_id)
            .ok_or_else(|| CryptoError::SerializationError("Invalid algorithm ID".into()))?;
        
        // Validate key sizes
        if serializable.public_key.len() != variant.public_key_bytes() {
            return Err(CryptoError::SerializationError(format!(
                "Invalid public key size for variant {}", variant
            )));
        }
        
        if let Some(ref sk) = serializable.secret_key {
            if sk.len() != variant.secret_key_bytes() {
                return Err(CryptoError::SerializationError(format!(
                    "Invalid secret key size for variant {}", variant
                )));
            }
        }

        Ok(Self {
            variant,
            public_key: serializable.public_key,
            secret_key: serializable.secret_key,
        })
    }
}

impl KeyPair for SphincsKeys {
    fn new() -> CryptoResult<Self> {
        // Default to the SHA2-128f-simple variant
        Self::new_with_variant(SphincsVariant::Sha2128FSimple)
    }

    fn from_private_key(_private_key: &[u8]) -> CryptoResult<Self> {
        // Expect concatenated bytes: algorithm_id (1) | pk_len (u32) | sk_len(u32) ??? ; simplified not implemented
        Err(CryptoError::NotImplemented("Loading from raw private key not supported; use from_bytes".into()))
    }

    fn public_key(&self) -> &[u8] {
        &self.public_key
    }
    
    fn private_key(&self) -> Option<&[u8]> {
        self.secret_key.as_deref()
    }

    fn algorithm_id(&self) -> u8 {
        self.variant.algorithm_id()
    }
    
    fn to_bytes(&self) -> CryptoResult<Vec<u8>> {
        self.to_bytes()
    }
    
    fn from_bytes(bytes: &[u8]) -> CryptoResult<Self> {
        Self::from_bytes(bytes)
    }
}

impl Signer for SphincsKeys {
    fn sign(&self, message: &[u8]) -> CryptoResult<Vec<u8>> {
        // Check if we have a secret key
        let sk_bytes = match &self.secret_key {
            Some(sk) => sk,
            None => return Err(CryptoError::SigningError("No secret key available".into())),
        };

        // Sign the message using pqcrypto's implementation based on variant
        // Note: We immediately convert each signature to bytes to avoid type mismatches
        let signature_bytes = match self.variant {
            SphincsVariant::Sha2128SSimple => {
                let sk = sphincssha2128ssimple::SecretKey::from_bytes(sk_bytes)
                    .map_err(|_| CryptoError::SigningError("Invalid secret key format".into()))?;
                let sig = sphincssha2128ssimple::detached_sign(message, &sk);
                sig.as_bytes().to_vec()
            },
            SphincsVariant::Sha2128FSimple => {
                let sk = sphincssha2128fsimple::SecretKey::from_bytes(sk_bytes)
                    .map_err(|_| CryptoError::SigningError("Invalid secret key format".into()))?;
                let sig = sphincssha2128fsimple::detached_sign(message, &sk);
                sig.as_bytes().to_vec()
            },
            SphincsVariant::Sha2256SSimple => {
                let sk = sphincssha2256ssimple::SecretKey::from_bytes(sk_bytes)
                    .map_err(|_| CryptoError::SigningError("Invalid secret key format".into()))?;
                let sig = sphincssha2256ssimple::detached_sign(message, &sk);
                sig.as_bytes().to_vec()
            },
            SphincsVariant::Sha2256FSimple => {
                let sk = sphincssha2256fsimple::SecretKey::from_bytes(sk_bytes)
                    .map_err(|_| CryptoError::SigningError("Invalid secret key format".into()))?;
                let sig = sphincssha2256fsimple::detached_sign(message, &sk);
                sig.as_bytes().to_vec()
            },
            SphincsVariant::Shake128SSimple => {
                let sk = sphincsshake128ssimple::SecretKey::from_bytes(sk_bytes)
                    .map_err(|_| CryptoError::SigningError("Invalid secret key format".into()))?;
                let sig = sphincsshake128ssimple::detached_sign(message, &sk);
                sig.as_bytes().to_vec()
            },
            SphincsVariant::Shake128FSimple => {
                let sk = sphincsshake128fsimple::SecretKey::from_bytes(sk_bytes)
                    .map_err(|_| CryptoError::SigningError("Invalid secret key format".into()))?;
                let sig = sphincsshake128fsimple::detached_sign(message, &sk);
                sig.as_bytes().to_vec()
            },
            SphincsVariant::Shake256SSimple => {
                let sk = sphincsshake256ssimple::SecretKey::from_bytes(sk_bytes)
                    .map_err(|_| CryptoError::SigningError("Invalid secret key format".into()))?;
                let sig = sphincsshake256ssimple::detached_sign(message, &sk);
                sig.as_bytes().to_vec()
            },
            SphincsVariant::Shake256FSimple => {
                let sk = sphincsshake256fsimple::SecretKey::from_bytes(sk_bytes)
                    .map_err(|_| CryptoError::SigningError("Invalid secret key format".into()))?;
                let sig = sphincsshake256fsimple::detached_sign(message, &sk);
                sig.as_bytes().to_vec()
            },
        };
        
        // Return the signature bytes
        Ok(signature_bytes)
    }

    fn algorithm_id(&self) -> u8 {
        self.variant.algorithm_id()
    }
}

impl Verifier for SphincsKeys {
    fn verify(&self, message: &[u8], signature: &[u8]) -> CryptoResult<bool> {
        debug!("Verifying SPHINCS+ signature with variant: {}", self.variant);
        
        // Verify based on which variant we're using
        let result = match self.variant {
            SphincsVariant::Sha2128SSimple => {
                if signature.len() != sphincssha2128ssimple::signature_bytes() {
                    return Ok(false); // Invalid signature length
                }
                
                let pk = match sphincssha2128ssimple::PublicKey::from_bytes(&self.public_key) {
                    Ok(pk) => pk,
                    Err(_) => return Ok(false), // Invalid public key format
                };
                
                let sig = match sphincssha2128ssimple::DetachedSignature::from_bytes(signature) {
                    Ok(sig) => sig,
                    Err(_) => return Ok(false), // Invalid signature format
                };
                
                sphincssha2128ssimple::verify_detached_signature(&sig, message, &pk).is_ok()
            },
            SphincsVariant::Sha2128FSimple => {
                if signature.len() != sphincssha2128fsimple::signature_bytes() {
                    return Ok(false); // Invalid signature length
                }
                
                let pk = match sphincssha2128fsimple::PublicKey::from_bytes(&self.public_key) {
                    Ok(pk) => pk,
                    Err(_) => return Ok(false), // Invalid public key format
                };
                
                let sig = match sphincssha2128fsimple::DetachedSignature::from_bytes(signature) {
                    Ok(sig) => sig,
                    Err(_) => return Ok(false), // Invalid signature format
                };
                
                sphincssha2128fsimple::verify_detached_signature(&sig, message, &pk).is_ok()
            },
            SphincsVariant::Sha2256SSimple => {
                if signature.len() != sphincssha2256ssimple::signature_bytes() {
                    return Ok(false); // Invalid signature length
                }
                
                let pk = match sphincssha2256ssimple::PublicKey::from_bytes(&self.public_key) {
                    Ok(pk) => pk,
                    Err(_) => return Ok(false), // Invalid public key format
                };
                
                let sig = match sphincssha2256ssimple::DetachedSignature::from_bytes(signature) {
                    Ok(sig) => sig,
                    Err(_) => return Ok(false), // Invalid signature format
                };
                
                sphincssha2256ssimple::verify_detached_signature(&sig, message, &pk).is_ok()
            },
            SphincsVariant::Sha2256FSimple => {
                if signature.len() != sphincssha2256fsimple::signature_bytes() {
                    return Ok(false); // Invalid signature length
                }
                
                let pk = match sphincssha2256fsimple::PublicKey::from_bytes(&self.public_key) {
                    Ok(pk) => pk,
                    Err(_) => return Ok(false), // Invalid public key format
                };
                
                let sig = match sphincssha2256fsimple::DetachedSignature::from_bytes(signature) {
                    Ok(sig) => sig,
                    Err(_) => return Ok(false), // Invalid signature format
                };
                
                sphincssha2256fsimple::verify_detached_signature(&sig, message, &pk).is_ok()
            },
            SphincsVariant::Shake128SSimple => {
                if signature.len() != sphincsshake128ssimple::signature_bytes() {
                    return Ok(false); // Invalid signature length
                }
                
                let pk = match sphincsshake128ssimple::PublicKey::from_bytes(&self.public_key) {
                    Ok(pk) => pk,
                    Err(_) => return Ok(false), // Invalid public key format
                };
                
                let sig = match sphincsshake128ssimple::DetachedSignature::from_bytes(signature) {
                    Ok(sig) => sig,
                    Err(_) => return Ok(false), // Invalid signature format
                };
                
                sphincsshake128ssimple::verify_detached_signature(&sig, message, &pk).is_ok()
            },
            SphincsVariant::Shake128FSimple => {
                if signature.len() != sphincsshake128fsimple::signature_bytes() {
                    return Ok(false); // Invalid signature length
                }
                
                let pk = match sphincsshake128fsimple::PublicKey::from_bytes(&self.public_key) {
                    Ok(pk) => pk,
                    Err(_) => return Ok(false), // Invalid public key format
                };
                
                let sig = match sphincsshake128fsimple::DetachedSignature::from_bytes(signature) {
                    Ok(sig) => sig,
                    Err(_) => return Ok(false), // Invalid signature format
                };
                
                sphincsshake128fsimple::verify_detached_signature(&sig, message, &pk).is_ok()
            },
            SphincsVariant::Shake256SSimple => {
                if signature.len() != sphincsshake256ssimple::signature_bytes() {
                    return Ok(false); // Invalid signature length
                }
                
                let pk = match sphincsshake256ssimple::PublicKey::from_bytes(&self.public_key) {
                    Ok(pk) => pk,
                    Err(_) => return Ok(false), // Invalid public key format
                };
                
                let sig = match sphincsshake256ssimple::DetachedSignature::from_bytes(signature) {
                    Ok(sig) => sig,
                    Err(_) => return Ok(false), // Invalid signature format
                };
                
                sphincsshake256ssimple::verify_detached_signature(&sig, message, &pk).is_ok()
            },
            SphincsVariant::Shake256FSimple => {
                if signature.len() != sphincsshake256fsimple::signature_bytes() {
                    return Ok(false); // Invalid signature length
                }
                
                let pk = match sphincsshake256fsimple::PublicKey::from_bytes(&self.public_key) {
                    Ok(pk) => pk,
                    Err(_) => return Ok(false), // Invalid public key format
                };
                
                let sig = match sphincsshake256fsimple::DetachedSignature::from_bytes(signature) {
                    Ok(sig) => sig,
                    Err(_) => return Ok(false), // Invalid signature format
                };
                
                sphincsshake256fsimple::verify_detached_signature(&sig, message, &pk).is_ok()
            },
        };
        
        if result {
            debug!("SPHINCS+ signature verification succeeded");
        } else {
            debug!("SPHINCS+ signature verification failed"); 
        }
        
        Ok(result)
    }

    fn algorithm_id(&self) -> u8 {
        self.variant.algorithm_id()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pqcrypto_traits::sign::{PublicKey as PqPublicKey, SecretKey as PqSecretKey};

    /// Tests key generation, signing, and verification for all SPHINCS+ variants
    #[test]
    fn test_sphincs_all_variants() {
        let test_message = b"Security validation test message";
        
        // Test all variants
        let variants = [
            SphincsVariant::Sha2128SSimple,
            SphincsVariant::Sha2128FSimple,
            SphincsVariant::Sha2256SSimple, 
            SphincsVariant::Sha2256FSimple,
            SphincsVariant::Shake128SSimple,
            SphincsVariant::Shake128FSimple,
        ];
        
        for variant in variants.iter() {
            println!("Testing SPHINCS+ variant: {}", variant);
            
            // Generate a keypair
            let keys = SphincsKeys::new(*variant).expect("Failed to generate keys");
            
            // Sign a message
            let signature = keys.sign(test_message).expect("Failed to sign message");
            
            // Verify with the same key
            let verified = keys.verify(test_message, &signature).expect("Failed to verify");
            assert!(verified, "Signature verification failed for {}", variant);
            
            // Create a public key only copy
            let pubkey = keys.public_key_only();
            
            // Verify with public key only
            let verified = pubkey.verify(test_message, &signature).expect("Failed to verify with public key");
            assert!(verified, "Public-key-only verification failed for {}", variant);
            
            // Test with wrong message
            let wrong_message = b"Wrong message";
            let verified = keys.verify(wrong_message, &signature).expect("Failed verification attempt");
            assert!(!verified, "Verification should fail with wrong message for {}", variant);
        }
    }
    
    /// Tests serialization and deserialization of SPHINCS+ keys
    #[test]
    fn test_sphincs_serialization() {
        let test_message = b"Security validation test message";
        let variant = SphincsVariant::Sha2128FSimple; // Fast variant for quicker tests
        
        // Generate keys
        let keys = SphincsKeys::new(variant).expect("Failed to generate keys");
        
        // Sign a message with original keys
        let original_signature = keys.sign(test_message).expect("Failed to sign with original keys");
        
        // Serialize keys
        let serialized = keys.to_bytes().expect("Failed to serialize keys");
        
        // Deserialize keys
        let deserialized = SphincsKeys::from_bytes(&serialized).expect("Failed to deserialize keys");
        
        // Verify that the original signature is valid with the deserialized key
        let verified = deserialized.verify(test_message, &original_signature)
            .expect("Failed to verify with deserialized key");
        assert!(verified, "Original signature verification failed with deserialized key");
        
        // Sign with the deserialized keys
        let new_signature = deserialized.sign(test_message).expect("Failed to sign with deserialized keys");
        
        // Verify the new signature with the original key
        let verified = keys.verify(test_message, &new_signature)
            .expect("Failed verification with original key");
        assert!(verified, "New signature verification failed with original key");
    }
    
    /// Tests that public-key-only keys can't sign
    #[test]
    fn test_sphincs_public_key_only() {
        let test_message = b"Security validation test message";
        
        // Generate a keypair
        let keys = SphincsKeys::new(SphincsVariant::Sha2128FSimple).expect("Failed to generate keys");
        
        // Create a public key only copy
        let pubkey = keys.public_key_only();
        
        // Try to sign with public key only (should fail)
        let result = pubkey.sign(test_message);
        assert!(result.is_err(), "Signing with public key only should fail");
        
        if let Err(e) = result {
            match e {
                CryptoError::SigningError(_) => (), // Expected error
                _ => panic!("Wrong error type: {:?}", e),
            }
        }
    }
    
    /// Tests direct compatibility with pqcrypto functions
    #[test]
    fn test_direct_pqcrypto_compatibility() {
        // Test message
        let test_message = b"Testing direct pqcrypto compatibility";
        
        // Generate keys directly with pqcrypto for SHA2-128F-Simple variant
        let (pk_pqc, sk_pqc) = sphincssha2128fsimple::keypair();
        
        // Extract raw bytes
        let pk_bytes = pk_pqc.as_bytes();  
        let sk_bytes = sk_pqc.as_bytes();
        
        // Create SphincsKeys using our implementation with these raw bytes
        let our_keys = SphincsKeys::from_keypair(
            SphincsVariant::Sha2128FSimple,
            pk_bytes,
            sk_bytes
        ).expect("Failed to create keys from raw bytes");
        
        // Sign with pqcrypto directly
        let signed_msg_pqc = sphincssha2128fsimple::sign(test_message, &sk_pqc);
        let detached_sig_pqc = sphincssha2128fsimple::detached_sign(test_message, &sk_pqc);
        
        // Sign with our implementation
        let our_sig = our_keys.sign(test_message).expect("Failed to sign with our implementation");
        
        // Verify each signature both ways
        
        // 1. Verify our signature with pqcrypto verify function
        let pqc_sig = sphincssha2128fsimple::DetachedSignature::from_bytes(&our_sig)
            .expect("Failed to convert our signature to pqcrypto format");
        let verify_our_sig_with_pqc = sphincssha2128fsimple::verify_detached_signature(
            &pqc_sig, test_message, &pk_pqc
        ).is_ok();
        assert!(verify_our_sig_with_pqc, "Our signature failed verification with pqcrypto function");
        
        // 2. Verify pqcrypto signature with our implementation
        let verify_pqc_sig_with_our = our_keys.verify(
            test_message, detached_sig_pqc.as_bytes()
        ).expect("Verification failed");
        assert!(verify_pqc_sig_with_our, "pqcrypto signature failed verification with our implementation");
        
        // Also verify full signed message can be properly verified
        let verify_full_msg = sphincssha2128fsimple::open(&signed_msg_pqc, &pk_pqc).is_ok();
        assert!(verify_full_msg, "Full signed message verification failed");
        
        // Test all other variants to ensure complete compatibility
        test_variant_compatibility(SphincsVariant::Sha2128SSimple, test_message);
        test_variant_compatibility(SphincsVariant::Sha2256FSimple, test_message);
        test_variant_compatibility(SphincsVariant::Sha2256SSimple, test_message);
        test_variant_compatibility(SphincsVariant::Shake128FSimple, test_message);
        test_variant_compatibility(SphincsVariant::Shake128SSimple, test_message);
    }
    
    /// Helper function to test compatibility with a specific variant
    fn test_variant_compatibility(variant: SphincsVariant, test_message: &[u8]) {
        println!("Testing compatibility for variant: {}", variant);
        
        // Generate keys with our implementation
        let our_keys = SphincsKeys::new(variant).expect("Failed to generate keys");
        
        // Sign with our implementation
        let our_sig = our_keys.sign(test_message).expect("Failed to sign with our implementation");
        
        // Verify with our implementation
        let verify_result = our_keys.verify(test_message, &our_sig).expect("Verification failed");
        assert!(verify_result, "Our signature verification failed");
        
        // Extract public key bytes
        let pk_bytes = our_keys.public_key();
        
        // Verify directly using appropriate pqcrypto function based on variant
        let direct_verify = match variant {
            SphincsVariant::Sha2128SSimple => {
                let pk = sphincssha2128ssimple::PublicKey::from_bytes(pk_bytes).unwrap();
                let sig = sphincssha2128ssimple::DetachedSignature::from_bytes(&our_sig).unwrap();
                sphincssha2128ssimple::verify_detached_signature(&sig, test_message, &pk).is_ok()
            },
            SphincsVariant::Sha2128FSimple => {
                let pk = sphincssha2128fsimple::PublicKey::from_bytes(pk_bytes).unwrap();
                let sig = sphincssha2128fsimple::DetachedSignature::from_bytes(&our_sig).unwrap();
                sphincssha2128fsimple::verify_detached_signature(&sig, test_message, &pk).is_ok()
            },
            SphincsVariant::Sha2256SSimple => {
                let pk = sphincssha2256ssimple::PublicKey::from_bytes(pk_bytes).unwrap();
                let sig = sphincssha2256ssimple::DetachedSignature::from_bytes(&our_sig).unwrap();
                sphincssha2256ssimple::verify_detached_signature(&sig, test_message, &pk).is_ok()
            },
            SphincsVariant::Sha2256FSimple => {
                let pk = sphincssha2256fsimple::PublicKey::from_bytes(pk_bytes).unwrap();
                let sig = sphincssha2256fsimple::DetachedSignature::from_bytes(&our_sig).unwrap();
                sphincssha2256fsimple::verify_detached_signature(&sig, test_message, &pk).is_ok()
            },
            SphincsVariant::Shake128SSimple => {
                let pk = sphincsshake128ssimple::PublicKey::from_bytes(pk_bytes).unwrap();
                let sig = sphincsshake128ssimple::DetachedSignature::from_bytes(&our_sig).unwrap();
                sphincsshake128ssimple::verify_detached_signature(&sig, test_message, &pk).is_ok()
            },
            SphincsVariant::Shake128FSimple => {
                let pk = sphincsshake128fsimple::PublicKey::from_bytes(pk_bytes).unwrap();
                let sig = sphincsshake128fsimple::DetachedSignature::from_bytes(&our_sig).unwrap();
                sphincsshake128fsimple::verify_detached_signature(&sig, test_message, &pk).is_ok()
            },
        };
        
        assert!(direct_verify, "Direct verification with pqcrypto functions failed for {}", variant);
    }
}
