//! SPHINCS+ implementation using the pqcrypto library
//!
//! This implementation replaces the previous OQS-based SPHINCS+ implementation with a pure Rust
//! implementation from pqcrypto-sphincsplus. This enables better key handling, memory security,
//! and removes the dependency on the OQS library.

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
    
    /// Creates a new SPHINCS+ key pair with the specified variant
    fn new(variant: SphincsVariant) -> CryptoResult<Self> {
        // Generate key pair based on the variant
        match variant {
            SphincsVariant::Sha2128SSimple => Self::generate_sha2_128s_simple(),
            SphincsVariant::Sha2128FSimple => Self::generate_sha2_128f_simple(),
            SphincsVariant::Sha2256SSimple => Self::generate_sha2_256s_simple(),
            SphincsVariant::Sha2256FSimple => Self::generate_sha2_256f_simple(),
            SphincsVariant::Shake128SSimple => Self::generate_shake_128s_simple(),
            SphincsVariant::Shake128FSimple => Self::generate_shake_128f_simple(),
            SphincsVariant::Shake256SSimple => Self::generate_shake_256s_simple(),
            SphincsVariant::Shake256FSimple => Self::generate_shake_256f_simple(),
        }
    }
    
    // Generate key pair implementations for each variant
    fn generate_sha2_128s_simple() -> CryptoResult<Self> {
        let (pk, sk) = sphincssha2128ssimple::keypair();
        Ok(Self {
            variant: SphincsVariant::Sha2128SSimple,
            public_key: pk.as_bytes().to_vec(),
            secret_key: Some(sk.as_bytes().to_vec()),
        })
    }
    
    fn generate_sha2_128f_simple() -> CryptoResult<Self> {
        let (pk, sk) = sphincssha2128fsimple::keypair();
        Ok(Self {
            variant: SphincsVariant::Sha2128FSimple,
            public_key: pk.as_bytes().to_vec(),
            secret_key: Some(sk.as_bytes().to_vec()),
        })
    }
    
    fn generate_sha2_256s_simple() -> CryptoResult<Self> {
        let (pk, sk) = sphincssha2256ssimple::keypair();
        Ok(Self {
            variant: SphincsVariant::Sha2256SSimple,
            public_key: pk.as_bytes().to_vec(),
            secret_key: Some(sk.as_bytes().to_vec()),
        })
    }
    
    fn generate_sha2_256f_simple() -> CryptoResult<Self> {
        let (pk, sk) = sphincssha2256fsimple::keypair();
        Ok(Self {
            variant: SphincsVariant::Sha2256FSimple,
            public_key: pk.as_bytes().to_vec(),
            secret_key: Some(sk.as_bytes().to_vec()),
        })
    }
    
    fn generate_shake_128s_simple() -> CryptoResult<Self> {
        let (pk, sk) = sphincsshake128ssimple::keypair();
        Ok(Self {
            variant: SphincsVariant::Shake128SSimple,
            public_key: pk.as_bytes().to_vec(),
            secret_key: Some(sk.as_bytes().to_vec()),
        })
    }
    
    fn generate_shake_128f_simple() -> CryptoResult<Self> {
        let (pk, sk) = sphincsshake128fsimple::keypair();
        Ok(Self {
            variant: SphincsVariant::Shake128FSimple,
            public_key: pk.as_bytes().to_vec(),
            secret_key: Some(sk.as_bytes().to_vec()),
        })
    }
    
    fn generate_shake_256s_simple() -> CryptoResult<Self> {
        let (pk, sk) = sphincsshake256ssimple::keypair();
        Ok(Self {
            variant: SphincsVariant::Shake256SSimple,
            public_key: pk.as_bytes().to_vec(),
            secret_key: Some(sk.as_bytes().to_vec()),
        })
    }
    
    fn generate_shake_256f_simple() -> CryptoResult<Self> {
        let (pk, sk) = sphincsshake256fsimple::keypair();
        Ok(Self {
            variant: SphincsVariant::Shake256FSimple,
            public_key: pk.as_bytes().to_vec(),
            secret_key: Some(sk.as_bytes().to_vec()),
        })
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
        // Default to the SHA2-256f-simple variant as a balance between size and speed
        Self::new_with_variant(SphincsVariant::Sha2256FSimple)
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
        // We already have a to_bytes implementation on the SphincsKeys struct
        // Use that implementation to avoid duplication
        SphincsKeys::to_bytes(self)
    }
    
    fn from_bytes(bytes: &[u8]) -> CryptoResult<Self> {
        // We already have a from_bytes implementation on the SphincsKeys struct
        // Use that implementation to avoid duplication
        SphincsKeys::from_bytes(bytes)
    }
}

impl SphincsKeys {
    /// Sign a message using the appropriate signature function based on the variant
    fn sign_message(&self, message: &[u8]) -> CryptoResult<Vec<u8>> {
        // Check if we have a secret key
        let secret_key_bytes = match &self.secret_key {
            Some(sk) => sk,
            None => return Err(CryptoError::SigningError("No secret key available for signing".into())),
        };
        
        // Choose the appropriate signing function based on the variant
        match self.variant {
            SphincsVariant::Sha2128SSimple => Self::sign_sha2_128s_simple(message, secret_key_bytes),
            SphincsVariant::Sha2128FSimple => Self::sign_sha2_128f_simple(message, secret_key_bytes),
            SphincsVariant::Sha2256SSimple => Self::sign_sha2_256s_simple(message, secret_key_bytes),
            SphincsVariant::Sha2256FSimple => Self::sign_sha2_256f_simple(message, secret_key_bytes),
            SphincsVariant::Shake128SSimple => Self::sign_shake_128s_simple(message, secret_key_bytes),
            SphincsVariant::Shake128FSimple => Self::sign_shake_128f_simple(message, secret_key_bytes),
            SphincsVariant::Shake256SSimple => Self::sign_shake_256s_simple(message, secret_key_bytes),
            SphincsVariant::Shake256FSimple => Self::sign_shake_256f_simple(message, secret_key_bytes),
        }
    }
    
    // Signature implementation for each variant
    fn sign_sha2_128s_simple(message: &[u8], secret_key_bytes: &[u8]) -> CryptoResult<Vec<u8>> {
        let sk = match sphincssha2128ssimple::SecretKey::from_bytes(secret_key_bytes) {
            Ok(sk) => sk,
            Err(_) => return Err(CryptoError::SigningError("Invalid secret key format".into())),
        };
        
        let signature = sphincssha2128ssimple::detached_sign(message, &sk);
        Ok(signature.as_bytes().to_vec())
    }
    
    fn sign_sha2_128f_simple(message: &[u8], secret_key_bytes: &[u8]) -> CryptoResult<Vec<u8>> {
        let sk = match sphincssha2128fsimple::SecretKey::from_bytes(secret_key_bytes) {
            Ok(sk) => sk,
            Err(_) => return Err(CryptoError::SigningError("Invalid secret key format".into())),
        };
        
        let signature = sphincssha2128fsimple::detached_sign(message, &sk);
        Ok(signature.as_bytes().to_vec())
    }
    
    fn sign_sha2_256s_simple(message: &[u8], secret_key_bytes: &[u8]) -> CryptoResult<Vec<u8>> {
        let sk = match sphincssha2256ssimple::SecretKey::from_bytes(secret_key_bytes) {
            Ok(sk) => sk,
            Err(_) => return Err(CryptoError::SigningError("Invalid secret key format".into())),
        };
        
        let signature = sphincssha2256ssimple::detached_sign(message, &sk);
        Ok(signature.as_bytes().to_vec())
    }
    
    fn sign_sha2_256f_simple(message: &[u8], secret_key_bytes: &[u8]) -> CryptoResult<Vec<u8>> {
        let sk = match sphincssha2256fsimple::SecretKey::from_bytes(secret_key_bytes) {
            Ok(sk) => sk,
            Err(_) => return Err(CryptoError::SigningError("Invalid secret key format".into())),
        };
        
        let signature = sphincssha2256fsimple::detached_sign(message, &sk);
        Ok(signature.as_bytes().to_vec())
    }
    
    fn sign_shake_128s_simple(message: &[u8], secret_key_bytes: &[u8]) -> CryptoResult<Vec<u8>> {
        let sk = match sphincsshake128ssimple::SecretKey::from_bytes(secret_key_bytes) {
            Ok(sk) => sk,
            Err(_) => return Err(CryptoError::SigningError("Invalid secret key format".into())),
        };
        
        let signature = sphincsshake128ssimple::detached_sign(message, &sk);
        Ok(signature.as_bytes().to_vec())
    }
    
    fn sign_shake_128f_simple(message: &[u8], secret_key_bytes: &[u8]) -> CryptoResult<Vec<u8>> {
        let sk = match sphincsshake128fsimple::SecretKey::from_bytes(secret_key_bytes) {
            Ok(sk) => sk,
            Err(_) => return Err(CryptoError::SigningError("Invalid secret key format".into())),
        };
        
        let signature = sphincsshake128fsimple::detached_sign(message, &sk);
        Ok(signature.as_bytes().to_vec())
    }
    
    fn sign_shake_256s_simple(message: &[u8], secret_key_bytes: &[u8]) -> CryptoResult<Vec<u8>> {
        let sk = match sphincsshake256ssimple::SecretKey::from_bytes(secret_key_bytes) {
            Ok(sk) => sk,
            Err(_) => return Err(CryptoError::SigningError("Invalid secret key format".into())),
        };
        
        let signature = sphincsshake256ssimple::detached_sign(message, &sk);
        Ok(signature.as_bytes().to_vec())
    }
    
    fn sign_shake_256f_simple(message: &[u8], secret_key_bytes: &[u8]) -> CryptoResult<Vec<u8>> {
        let sk = match sphincsshake256fsimple::SecretKey::from_bytes(secret_key_bytes) {
            Ok(sk) => sk,
            Err(_) => return Err(CryptoError::SigningError("Invalid secret key format".into())),
        };
        
        let signature = sphincsshake256fsimple::detached_sign(message, &sk);
        Ok(signature.as_bytes().to_vec())
    }
}

impl Signer for SphincsKeys {
    fn sign(&self, message: &[u8]) -> CryptoResult<Vec<u8>> {
        debug!("Signing message of length {} with SPHINCS+ key", message.len());
        
        // Perform the signing using our variant-specific implementation
        let signature_bytes = self.sign_message(message)?;
        
        debug!("Created SPHINCS+ signature of length {}", signature_bytes.len());
        
        Ok(signature_bytes)
    }
    
    fn algorithm_id(&self) -> u8 {
        self.variant.algorithm_id()
    }
}
impl SphincsKeys {
    /// Verify a signature using the appropriate verification function for the variant
    fn verify_message(&self, message: &[u8], signature: &[u8]) -> CryptoResult<bool> {
        // Choose the appropriate verification function based on the variant
        match self.variant {
            SphincsVariant::Sha2128SSimple => Self::verify_sha2_128s_simple(message, signature, &self.public_key),
            SphincsVariant::Sha2128FSimple => Self::verify_sha2_128f_simple(message, signature, &self.public_key),
            SphincsVariant::Sha2256SSimple => Self::verify_sha2_256s_simple(message, signature, &self.public_key),
            SphincsVariant::Sha2256FSimple => Self::verify_sha2_256f_simple(message, signature, &self.public_key),
            SphincsVariant::Shake128SSimple => Self::verify_shake_128s_simple(message, signature, &self.public_key),
            SphincsVariant::Shake128FSimple => Self::verify_shake_128f_simple(message, signature, &self.public_key),
            SphincsVariant::Shake256SSimple => Self::verify_shake_256s_simple(message, signature, &self.public_key),
            SphincsVariant::Shake256FSimple => Self::verify_shake_256f_simple(message, signature, &self.public_key),
        }
    }
    
    // Verification implementation for each variant
    fn verify_sha2_128s_simple(message: &[u8], signature: &[u8], public_key_bytes: &[u8]) -> CryptoResult<bool> {
        // Create public key object from bytes
        let pk = match sphincssha2128ssimple::PublicKey::from_bytes(public_key_bytes) {
            Ok(pk) => pk,
            Err(_) => return Err(CryptoError::VerificationError("Invalid public key format".into())),
        };
        
        // Create signature object from bytes
        let sig = match sphincssha2128ssimple::DetachedSignature::from_bytes(signature) {
            Ok(sig) => sig,
            Err(_) => return Ok(false), // Invalid signature format
        };
        
        // Verify the signature
        match sphincssha2128ssimple::verify_detached_signature(&sig, message, &pk) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
    
    fn verify_sha2_128f_simple(message: &[u8], signature: &[u8], public_key_bytes: &[u8]) -> CryptoResult<bool> {
        // Create public key object from bytes
        let pk = match sphincssha2128fsimple::PublicKey::from_bytes(public_key_bytes) {
            Ok(pk) => pk,
            Err(_) => return Err(CryptoError::VerificationError("Invalid public key format".into())),
        };
        
        // Create signature object from bytes
        let sig = match sphincssha2128fsimple::DetachedSignature::from_bytes(signature) {
            Ok(sig) => sig,
            Err(_) => return Ok(false), // Invalid signature format
        };
        
        // Verify the signature
        match sphincssha2128fsimple::verify_detached_signature(&sig, message, &pk) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
    
    fn verify_sha2_256s_simple(message: &[u8], signature: &[u8], public_key_bytes: &[u8]) -> CryptoResult<bool> {
        // Create public key object from bytes
        let pk = match sphincssha2256ssimple::PublicKey::from_bytes(public_key_bytes) {
            Ok(pk) => pk,
            Err(_) => return Err(CryptoError::VerificationError("Invalid public key format".into())),
        };
        
        // Create signature object from bytes
        let sig = match sphincssha2256ssimple::DetachedSignature::from_bytes(signature) {
            Ok(sig) => sig,
            Err(_) => return Ok(false), // Invalid signature format
        };
        
        // Verify the signature
        match sphincssha2256ssimple::verify_detached_signature(&sig, message, &pk) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
    
    fn verify_sha2_256f_simple(message: &[u8], signature: &[u8], public_key_bytes: &[u8]) -> CryptoResult<bool> {
        // Create public key object from bytes
        let pk = match sphincssha2256fsimple::PublicKey::from_bytes(public_key_bytes) {
            Ok(pk) => pk,
            Err(_) => return Err(CryptoError::VerificationError("Invalid public key format".into())),
        };
        
        // Create signature object from bytes
        let sig = match sphincssha2256fsimple::DetachedSignature::from_bytes(signature) {
            Ok(sig) => sig,
            Err(_) => return Ok(false), // Invalid signature format
        };
        
        // Verify the signature
        match sphincssha2256fsimple::verify_detached_signature(&sig, message, &pk) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
    
    fn verify_shake_128s_simple(message: &[u8], signature: &[u8], public_key_bytes: &[u8]) -> CryptoResult<bool> {
        // Create public key object from bytes
        let pk = match sphincsshake128ssimple::PublicKey::from_bytes(public_key_bytes) {
            Ok(pk) => pk,
            Err(_) => return Err(CryptoError::VerificationError("Invalid public key format".into())),
        };
        
        // Create signature object from bytes
        let sig = match sphincsshake128ssimple::DetachedSignature::from_bytes(signature) {
            Ok(sig) => sig,
            Err(_) => return Ok(false), // Invalid signature format
        };
        
        // Verify the signature
        match sphincsshake128ssimple::verify_detached_signature(&sig, message, &pk) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
    
    fn verify_shake_128f_simple(message: &[u8], signature: &[u8], public_key_bytes: &[u8]) -> CryptoResult<bool> {
        // Create public key object from bytes
        let pk = match sphincsshake128fsimple::PublicKey::from_bytes(public_key_bytes) {
            Ok(pk) => pk,
            Err(_) => return Err(CryptoError::VerificationError("Invalid public key format".into())),
        };
        
        // Create signature object from bytes
        let sig = match sphincsshake128fsimple::DetachedSignature::from_bytes(signature) {
            Ok(sig) => sig,
            Err(_) => return Ok(false), // Invalid signature format
        };
        
        // Verify the signature
        match sphincsshake128fsimple::verify_detached_signature(&sig, message, &pk) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
    
    fn verify_shake_256s_simple(message: &[u8], signature: &[u8], public_key_bytes: &[u8]) -> CryptoResult<bool> {
        // Create public key object from bytes
        let pk = match sphincsshake256ssimple::PublicKey::from_bytes(public_key_bytes) {
            Ok(pk) => pk,
            Err(_) => return Err(CryptoError::VerificationError("Invalid public key format".into())),
        };
        
        // Create signature object from bytes
        let sig = match sphincsshake256ssimple::DetachedSignature::from_bytes(signature) {
            Ok(sig) => sig,
            Err(_) => return Ok(false), // Invalid signature format
        };
        
        // Verify the signature
        match sphincsshake256ssimple::verify_detached_signature(&sig, message, &pk) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
    
    fn verify_shake_256f_simple(message: &[u8], signature: &[u8], public_key_bytes: &[u8]) -> CryptoResult<bool> {
        // Create public key object from bytes
        let pk = match sphincsshake256fsimple::PublicKey::from_bytes(public_key_bytes) {
            Ok(pk) => pk,
            Err(_) => return Err(CryptoError::VerificationError("Invalid public key format".into())),
        };
        
        // Create signature object from bytes
        let sig = match sphincsshake256fsimple::DetachedSignature::from_bytes(signature) {
            Ok(sig) => sig,
            Err(_) => return Ok(false), // Invalid signature format
        };
        
        // Verify the signature
        match sphincsshake256fsimple::verify_detached_signature(&sig, message, &pk) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

impl Verifier for SphincsKeys {
    fn verify(&self, message: &[u8], signature: &[u8]) -> CryptoResult<bool> {
        debug!("Verifying SPHINCS+ signature of length {} against message of length {}",
               signature.len(), message.len());
        
        // Perform the verification using our variant-specific implementation
        let result = self.verify_message(message, signature)?;
        
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

/// A SPHINCS+ signature with metadata for verification
#[derive(Clone)]
pub struct SphincsSignature {
    signature: Vec<u8>,
    public_key_bytes: Vec<u8>,
    variant: SphincsVariant,
}

impl fmt::Debug for SphincsSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SphincsSignature")
            .field("signature", &format!("[{} bytes]", self.signature.len()))
            .field("public_key_bytes", &format!("[{} bytes]", self.public_key_bytes.len()))
            .field("variant", &self.variant)
            .finish()
    }
}

impl SphincsSignature {
    /// Create a new SPHINCS+ signature object
    pub fn new(signature: Vec<u8>, public_key_bytes: Vec<u8>, variant: SphincsVariant) -> CryptoResult<Self> {
        // Validate signature size matches the variant
        if signature.len() != variant.signature_bytes() {
            return Err(CryptoError::InvalidSignatureFormat(
                format!("Signature size mismatch: expected {}, got {}", 
                    variant.signature_bytes(), signature.len())
            ));
        }
        
        // Validate public key size matches the variant
        if public_key_bytes.len() != variant.public_key_bytes() {
            return Err(CryptoError::InvalidKeyFormat(
                format!("Public key size mismatch: expected {}, got {}", 
                    variant.public_key_bytes(), public_key_bytes.len())
            ));
        }
        
        Ok(Self {
            signature,
            public_key_bytes,
            variant,
        })
    }
    
    /// Get the raw signature bytes
    pub fn signature(&self) -> &Vec<u8> {
        &self.signature
    }
    
    /// Get the public key bytes
    pub fn public_key(&self) -> &Vec<u8> {
        &self.public_key_bytes
    }
    
    /// Get the SPHINCS+ variant used for this signature
    pub fn variant(&self) -> SphincsVariant {
        self.variant
    }
    
    /// Verify this signature against a message
    pub fn verify(&self, message: &[u8]) -> CryptoResult<bool> {
        debug!("Verifying SPHINCS+ signature with public key only");
        
        // Create a verifier-only key from our public key bytes
        let mut verifier = SphincsKeys::new_with_variant(self.variant)
            .map_err(|e| CryptoError::VerificationError(
                format!("Failed to create verifier from public key: {}", e)
            ))?;
        
        // Set the verifier to use our public key
        verifier.public_key = self.public_key_bytes.clone();
        
        // Use the verifier implementation we already have
        verifier.verify(message, &self.signature)
    }
}

// Tests for the SPHINCS+ implementation
#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::common::traits::{KeyPair, Signer, Verifier};
    
    // ... (other tests)

    #[test]
    fn test_sphincs_raw_pqcrypto_compatibility() {
        use pqcrypto_sphincsplus::sphincsshake256ssimple;
        
        // Generate key pair using our wrapper implementation
        let keys = SphincsKeys::new_with_variant(SphincsVariant::Shake256SSimple)
            .expect("Failed to generate SPHINCS+ keys");
        
        // Extract public and secret keys
        let public_key_bytes = keys.public_key(); // No need for expect() on &[u8]
        let secret_key_bytes = keys.private_key().expect("Missing private key");
        
        // Convert to pqcrypto native types
        let pqc_pk = sphincsshake256ssimple::PublicKey::from_bytes(&public_key_bytes)
            .expect("Failed to convert public key to pqcrypto type");
        
        let pqc_sk = sphincsshake256ssimple::SecretKey::from_bytes(&secret_key_bytes)
            .expect("Failed to convert secret key to pqcrypto type");
        
        // Test message
        let test_message = b"Compatibility test message";
        
        // Sign directly with pqcrypto
        let pqc_sig = sphincsshake256ssimple::detached_sign(test_message, &pqc_sk);
        
        // Verify with our wrapper
        let wrapped_verify = keys.verify(test_message, &pqc_sig.as_bytes().to_vec())
            .expect("Verification with wrapper failed");
        
        assert!(wrapped_verify, "Wrapper should verify signature created with raw pqcrypto");
        
        // Sign with our wrapper
        let wrapped_signature = keys.sign(test_message)
            .expect("Failed to sign with wrapper");
        
        // Verify directly with pqcrypto
        let signature_obj = sphincsshake256ssimple::DetachedSignature::from_bytes(&wrapped_signature)
            .expect("Failed to convert wrapper signature to pqcrypto format");
        
        // This should not panic if signature is valid
        sphincsshake256ssimple::verify_detached_signature(&signature_obj, test_message, &pqc_pk)
            .expect("Raw pqcrypto failed to verify wrapper signature");
    }
    
    #[test]
    fn test_sphincs_error_handling() {
        // Generate a key pair
        let keys = SphincsKeys::new_with_variant(SphincsVariant::Shake256SSimple)
            .expect("Failed to generate SPHINCS+ keys");
        
        let message = b"Error handling test message";
        
        // Create public-key-only copy
        let public_only = keys.public_key_only();
        
        // Attempt to sign with public-key-only should fail
        let sign_result = public_only.sign(message);
        assert!(sign_result.is_err(), "Signing with public-key-only should fail");
        
        // Generate valid signature with full keys
        let valid_signature = keys.sign(message)
            .expect("Failed to sign message");
        
        // Verify with truncated (invalid) signature
        let truncated_sig = valid_signature[0..valid_signature.len()-10].to_vec();
        let truncated_result = keys.verify(message, &truncated_sig);
        
        // Should return Ok(false) for an invalid signature (not an error)
        assert!(truncated_result.is_ok(), "Verification with invalid signature should not error");
        assert!(!truncated_result.unwrap(), "Verification with invalid signature should fail");
        
        // Test with wrong variant
        let wrong_variant_keys = SphincsKeys::new_with_variant(SphincsVariant::Sha2256FSimple)
            .expect("Failed to generate different variant keys");
        
        let wrong_variant_sig = wrong_variant_keys.sign(message)
            .expect("Failed to sign with different variant");
        
        // Verify signature from one variant with keys from another should fail
        let cross_variant_result = keys.verify(message, &wrong_variant_sig);
        assert!(cross_variant_result.is_ok(), "Cross-variant verification should not error");
        assert!(!cross_variant_result.unwrap(), "Cross-variant verification should fail");
    }
}
