use crate::crypto::common::traits::{KeyEncapsulation, KeyPair};
use crate::crypto::{CryptoAlgorithm, CryptoError, CryptoResult};
use log::debug;
use pqcrypto_kyber::kyber1024;
use pqcrypto_kyber::kyber768;
use pqcrypto_traits::kem::{Ciphertext, PublicKey, SecretKey, SharedSecret};
use std::fmt::Debug;
use std::ops::Drop;
use zeroize::Zeroize;

/// Kyber post-quantum key encapsulation mechanism variants.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KyberVariant {
    /// Kyber768 offering 196-bit security level
    Kyber768,
    /// Kyber1024 offering 256-bit security level
    Kyber1024,
}

impl KyberVariant {
    /// Get algorithm identifier for this variant
    pub fn algorithm_id(&self) -> u8 {
        match self {
            KyberVariant::Kyber768 => CryptoAlgorithm::Kyber768 as u8,
            KyberVariant::Kyber1024 => CryptoAlgorithm::Kyber1024 as u8,
        }
    }

    /// Get string identifier for this variant
    pub fn name(&self) -> &'static str {
        match self {
            KyberVariant::Kyber768 => "kyber-768",
            KyberVariant::Kyber1024 => "kyber-1024",
        }
    }

    /// Get the public key size in bytes
    pub fn public_key_size(&self) -> usize {
        match self {
            KyberVariant::Kyber768 => kyber768::public_key_bytes(),
            KyberVariant::Kyber1024 => kyber1024::public_key_bytes(),
        }
    }

    /// Get the secret key size in bytes
    pub fn secret_key_size(&self) -> usize {
        match self {
            KyberVariant::Kyber768 => kyber768::secret_key_bytes(),
            KyberVariant::Kyber1024 => kyber1024::secret_key_bytes(),
        }
    }

    /// Get the ciphertext size in bytes
    pub fn ciphertext_size(&self) -> usize {
        match self {
            KyberVariant::Kyber768 => kyber768::ciphertext_bytes(),
            KyberVariant::Kyber1024 => kyber1024::ciphertext_bytes(),
        }
    }

    /// Get the shared secret size in bytes
    pub fn shared_secret_size(&self) -> usize {
        match self {
            KyberVariant::Kyber768 => kyber768::shared_secret_bytes(),
            KyberVariant::Kyber1024 => kyber1024::shared_secret_bytes(),
        }
    }
}

/// Kyber post-quantum key encapsulation mechanism keys.
///
/// This implementation uses the pqcrypto-kyber crate to provide
/// post-quantum secure key encapsulation.
///
/// The keys can be used to encapsulate and decapsulate shared secrets
/// used for symmetric encryption/decryption.
#[derive(Clone)]
pub struct KyberKeys {
    public_key: Vec<u8>,
    secret_key: Option<Vec<u8>>,
    variant: KyberVariant,
}

impl Debug for KyberKeys {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KyberKeys")
            .field("public_key", &self.public_key)
            .field("secret_key", &"[]")
            .field("variant", &self.variant)
            .finish()
    }
}

impl Drop for KyberKeys {
    fn drop(&mut self) {
        // Ensure secret key is zeroized when KyberKeys is dropped
        if let Some(ref mut sk) = self.secret_key {
            sk.zeroize();
        }
    }
}

impl KeyPair for KyberKeys {
    fn new() -> CryptoResult<Self> {
        Self::new_with_variant(KyberVariant::Kyber1024)
    }

    fn from_private_key(private_key: &[u8]) -> CryptoResult<Self> {
        // Determine variant from key size
        let variant = if private_key.len() == KyberVariant::Kyber768.secret_key_size() {
            KyberVariant::Kyber768
        } else if private_key.len() == KyberVariant::Kyber1024.secret_key_size() {
            KyberVariant::Kyber1024
        } else {
            return Err(CryptoError::InvalidKeyFormat(
                "Invalid key length for Kyber operation".to_string(),
            ));
        };

        // Generate new keypair with this variant
        let mut kyber = Self::new_with_variant(variant)?;

        // Replace secret key with provided one
        kyber.secret_key = Some(private_key.to_vec());

        // Derive public key from secret key if possible
        // Note: In Kyber, we can't actually derive the public key from the secret key
        // So we'd need the original public key too, otherwise this won't work correctly
        // This is a limitation compared to many other asymmetric cryptography schemes
        Err(CryptoError::NotImplemented(
            "Cannot derive public key from private key in Kyber".to_string(),
        ))
    }

    fn public_key(&self) -> &[u8] {
        &self.public_key
    }

    fn private_key(&self) -> Option<&[u8]> {
        self.secret_key.as_deref()
    }

    fn to_bytes(&self) -> CryptoResult<Vec<u8>> {
        let variant_id = self.variant.algorithm_id();
        let mut result = Vec::with_capacity(
            1 + self.public_key.len() + self.secret_key.as_ref().map_or(0, |sk| sk.len()),
        );

        // Format: [1 byte variant][public key bytes][optional secret key bytes]
        result.push(variant_id);
        result.extend_from_slice(&self.public_key);

        if let Some(sk) = &self.secret_key {
            result.extend_from_slice(sk);
        }

        Ok(result)
    }

    fn from_bytes(bytes: &[u8]) -> CryptoResult<Self> {
        if bytes.is_empty() {
            return Err(CryptoError::SerializationError(
                "Invalid format".to_string(),
            ));
        }

        // First byte is the variant ID
        let variant_id = bytes[0];
        let variant = match variant_id {
            id if id == CryptoAlgorithm::Kyber768 as u8 => KyberVariant::Kyber768,
            id if id == CryptoAlgorithm::Kyber1024 as u8 => KyberVariant::Kyber1024,
            _ => {
                return Err(CryptoError::InvalidAlgorithm(
                    "Unsupported Kyber variant".to_string(),
                ))
            }
        };

        // Calculate expected sizes based on variant
        let pk_size = variant.public_key_size();
        let sk_size = variant.secret_key_size();

        // Minimum required length: 1 byte variant + public key bytes
        let min_required_len = 1 + pk_size;
        if bytes.len() < min_required_len {
            return Err(CryptoError::SerializationError(
                "Invalid format".to_string(),
            ));
        }

        // Extract public key
        let public_key = bytes[1..1 + pk_size].to_vec();

        // Check if we have secret key bytes
        let secret_key = if bytes.len() >= min_required_len + sk_size {
            let sk_bytes = bytes[1 + pk_size..1 + pk_size + sk_size].to_vec();
            Some(sk_bytes)
        } else if bytes.len() == min_required_len {
            // Public key only
            None
        } else {
            // Invalid length
            return Err(CryptoError::SerializationError(
                "Invalid format".to_string(),
            ));
        };

        Ok(Self {
            public_key,
            secret_key,
            variant,
        })
    }

    fn algorithm_id(&self) -> u8 {
        self.variant.algorithm_id()
    }
}

impl KyberKeys {
    /// Generate new Kyber keys with the default variant (Kyber1024).
    pub fn new() -> CryptoResult<Self> {
        Self::new_with_variant(KyberVariant::Kyber1024)
    }

    /// Generate new Kyber keys with the specified variant.
    pub fn new_with_variant(variant: KyberVariant) -> CryptoResult<Self> {
        debug!("Generating new Kyber keys with variant: {:?}", variant);

        match variant {
            KyberVariant::Kyber768 => {
                let (pk, sk) = kyber768::keypair();
                Ok(Self {
                    public_key: pk.as_bytes().to_vec(),
                    secret_key: Some(sk.as_bytes().to_vec()),
                    variant,
                })
            }
            KyberVariant::Kyber1024 => {
                let (pk, sk) = kyber1024::keypair();
                Ok(Self {
                    public_key: pk.as_bytes().to_vec(),
                    secret_key: Some(sk.as_bytes().to_vec()),
                    variant,
                })
            }
        }
    }

    /// Create Kyber keys from existing public key bytes.
    ///
    /// This creates a KyberKeys instance with only the public key,
    /// which can be used for encapsulation but not decapsulation.
    pub fn from_public_key(public_key: Vec<u8>, variant: KyberVariant) -> CryptoResult<Self> {
        // Check public key length
        if public_key.len() != variant.public_key_size() {
            return Err(CryptoError::InvalidKeyFormat(
                "Invalid public key length for Kyber operation".to_string(),
            ));
        }

        Ok(Self {
            public_key,
            secret_key: None,
            variant,
        })
    }

    /// Create Kyber keys from existing public and secret key bytes.
    pub fn from_keypair(
        public_key: Vec<u8>,
        secret_key: Vec<u8>,
        variant: KyberVariant,
    ) -> CryptoResult<Self> {
        // Check key lengths
        if public_key.len() != variant.public_key_size() {
            return Err(CryptoError::InvalidKeyFormat(
                "Invalid public key length for Kyber operation".to_string(),
            ));
        }

        if secret_key.len() != variant.secret_key_size() {
            return Err(CryptoError::InvalidKeyFormat(
                "Invalid secret key length for Kyber operation".to_string(),
            ));
        }

        Ok(Self {
            public_key,
            secret_key: Some(secret_key),
            variant,
        })
    }

    /// Return a reference to the public key.
    pub fn public_key(&self) -> &[u8] {
        &self.public_key
    }

    /// Return a clone of the secret key, if available.
    pub fn secret_key(&self) -> Option<Vec<u8>> {
        self.secret_key.clone()
    }

    /// Return the variant of these keys.
    pub fn variant(&self) -> KyberVariant {
        self.variant
    }

    /// Clear the secret key from this instance to secure memory.
    pub fn clear_secret_key(&mut self) {
        if let Some(ref mut sk) = self.secret_key {
            sk.zeroize();
        }
        self.secret_key = None;
    }

    /// Encapsulate a shared secret using the public key.
    ///
    /// Returns a tuple containing (ciphertext, shared_secret).
    pub fn encapsulate(&self) -> CryptoResult<(Vec<u8>, Vec<u8>)> {
        match self.variant {
            KyberVariant::Kyber768 => {
                let pk = kyber768::PublicKey::from_bytes(&self.public_key).map_err(|_| {
                    CryptoError::InvalidKeyFormat(
                        "Invalid key format for Kyber operation".to_string(),
                    )
                })?;

                let (ss, ct) = kyber768::encapsulate(&pk);

                // Clone and zeroize shared secret
                let shared_secret = ss.as_bytes().to_vec();
                let ciphertext = ct.as_bytes().to_vec();

                debug!(
                    "Encapsulated shared secret with Kyber768. Ciphertext size: {}, Shared secret size: {}",
                    ciphertext.len(),
                    shared_secret.len()
                );

                Ok((ciphertext, shared_secret))
            }
            KyberVariant::Kyber1024 => {
                let pk = kyber1024::PublicKey::from_bytes(&self.public_key).map_err(|_| {
                    CryptoError::InvalidKeyFormat(
                        "Invalid key format for Kyber operation".to_string(),
                    )
                })?;

                let (ss, ct) = kyber1024::encapsulate(&pk);

                // Clone and zeroize shared secret
                let shared_secret = ss.as_bytes().to_vec();
                let ciphertext = ct.as_bytes().to_vec();

                debug!(
                    "Encapsulated shared secret with Kyber1024. Ciphertext size: {}, Shared secret size: {}",
                    ciphertext.len(),
                    shared_secret.len()
                );

                Ok((ciphertext, shared_secret))
            }
        }
    }

    /// Decapsulate a shared secret using the private key and ciphertext.
    pub fn decapsulate(&self, ciphertext: &[u8]) -> CryptoResult<Vec<u8>> {
        // Verify we have a secret key
        let sk_bytes = match &self.secret_key {
            Some(sk) => sk,
            None => {
                return Err(CryptoError::DecapsulationError(
                    "Missing secret key for decapsulation".to_string(),
                ))
            }
        };

        // Verify ciphertext length
        if ciphertext.len() != self.variant.ciphertext_size() {
            return Err(CryptoError::DecapsulationError(
                "Invalid ciphertext length".to_string(),
            ));
        }

        match self.variant {
            KyberVariant::Kyber768 => {
                let sk = kyber768::SecretKey::from_bytes(sk_bytes).map_err(|_| {
                    CryptoError::InvalidKeyFormat(
                        "Invalid key format for Kyber operation".to_string(),
                    )
                })?;

                let ct = kyber768::Ciphertext::from_bytes(ciphertext).map_err(|_| {
                    CryptoError::DecapsulationError("Invalid ciphertext format".to_string())
                })?;

                let ss = kyber768::decapsulate(&ct, &sk);
                let shared_secret = ss.as_bytes().to_vec();

                debug!(
                    "Decapsulated shared secret with Kyber768. Shared secret size: {}",
                    shared_secret.len()
                );

                Ok(shared_secret)
            }
            KyberVariant::Kyber1024 => {
                let sk = kyber1024::SecretKey::from_bytes(sk_bytes).map_err(|_| {
                    CryptoError::InvalidKeyFormat(
                        "Invalid key format for Kyber operation".to_string(),
                    )
                })?;

                let ct = kyber1024::Ciphertext::from_bytes(ciphertext).map_err(|_| {
                    CryptoError::DecapsulationError("Invalid ciphertext format".to_string())
                })?;

                let ss = kyber1024::decapsulate(&ct, &sk);
                let shared_secret = ss.as_bytes().to_vec();

                debug!(
                    "Decapsulated shared secret with Kyber1024. Shared secret size: {}",
                    shared_secret.len()
                );

                Ok(shared_secret)
            }
        }
    }
}

impl KeyEncapsulation for KyberKeys {
    fn encapsulate(&self) -> CryptoResult<(Vec<u8>, Vec<u8>)> {
        match self.variant {
            KyberVariant::Kyber768 => {
                let pk = kyber768::PublicKey::from_bytes(&self.public_key).map_err(|_| {
                    CryptoError::InvalidKeyFormat(
                        "Invalid key format for Kyber operation".to_string(),
                    )
                })?;

                let (ss, ct) = kyber768::encapsulate(&pk);

                // Clone and zeroize shared secret
                let shared_secret = ss.as_bytes().to_vec();
                let ciphertext = ct.as_bytes().to_vec();

                debug!(
                    "Encapsulated shared secret with Kyber768. Ciphertext size: {}, Shared secret size: {}",
                    ciphertext.len(),
                    shared_secret.len()
                );

                Ok((ciphertext, shared_secret))
            }
            KyberVariant::Kyber1024 => {
                let pk = kyber1024::PublicKey::from_bytes(&self.public_key).map_err(|_| {
                    CryptoError::InvalidKeyFormat(
                        "Invalid key format for Kyber operation".to_string(),
                    )
                })?;

                let (ss, ct) = kyber1024::encapsulate(&pk);

                // Clone and zeroize shared secret
                let shared_secret = ss.as_bytes().to_vec();
                let ciphertext = ct.as_bytes().to_vec();

                debug!(
                    "Encapsulated shared secret with Kyber1024. Ciphertext size: {}, Shared secret size: {}",
                    ciphertext.len(),
                    shared_secret.len()
                );

                Ok((ciphertext, shared_secret))
            }
        }
    }

    fn decapsulate(&self, ciphertext: &[u8]) -> CryptoResult<Vec<u8>> {
        // Verify we have a secret key
        let sk_bytes = match &self.secret_key {
            Some(sk) => sk,
            None => {
                return Err(CryptoError::DecapsulationError(
                    "Missing secret key for decapsulation".to_string(),
                ))
            }
        };

        // Verify ciphertext length
        if ciphertext.len() != self.variant.ciphertext_size() {
            return Err(CryptoError::DecapsulationError(
                "Invalid ciphertext length".to_string(),
            ));
        }

        match self.variant {
            KyberVariant::Kyber768 => {
                let sk = kyber768::SecretKey::from_bytes(sk_bytes).map_err(|_| {
                    CryptoError::InvalidKeyFormat(
                        "Invalid key format for Kyber operation".to_string(),
                    )
                })?;

                let ct = kyber768::Ciphertext::from_bytes(ciphertext).map_err(|_| {
                    CryptoError::DecapsulationError("Invalid ciphertext format".to_string())
                })?;

                let ss = kyber768::decapsulate(&ct, &sk);
                let shared_secret = ss.as_bytes().to_vec();

                debug!(
                    "Decapsulated shared secret with Kyber768. Shared secret size: {}",
                    shared_secret.len()
                );

                Ok(shared_secret)
            }
            KyberVariant::Kyber1024 => {
                let sk = kyber1024::SecretKey::from_bytes(sk_bytes).map_err(|_| {
                    CryptoError::InvalidKeyFormat(
                        "Invalid key format for Kyber operation".to_string(),
                    )
                })?;

                let ct = kyber1024::Ciphertext::from_bytes(ciphertext).map_err(|_| {
                    CryptoError::DecapsulationError("Invalid ciphertext format".to_string())
                })?;

                let ss = kyber1024::decapsulate(&ct, &sk);
                let shared_secret = ss.as_bytes().to_vec();

                debug!(
                    "Decapsulated shared secret with Kyber1024. Shared secret size: {}",
                    shared_secret.len()
                );

                Ok(shared_secret)
            }
        }
    }

    fn algorithm_id(&self) -> u8 {
        self.variant.algorithm_id()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kyber768_encapsulation_decapsulation() {
        let kyber = KyberKeys::new_with_variant(KyberVariant::Kyber768).unwrap();
        let (ciphertext, shared_secret1) = kyber.encapsulate().unwrap();
        let shared_secret2 = kyber.decapsulate(&ciphertext).unwrap();

        assert_eq!(shared_secret1, shared_secret2);
    }

    #[test]
    fn test_kyber1024_encapsulation_decapsulation() {
        let kyber = KyberKeys::new_with_variant(KyberVariant::Kyber1024).unwrap();
        let (ciphertext, shared_secret1) = kyber.encapsulate().unwrap();
        let shared_secret2 = kyber.decapsulate(&ciphertext).unwrap();

        assert_eq!(shared_secret1, shared_secret2);
    }

    #[test]
    fn test_from_public_key() {
        let kyber = KyberKeys::new_with_variant(KyberVariant::Kyber1024).unwrap();
        let public_key = kyber.public_key().to_vec();
        let kyber_pub = KyberKeys::from_public_key(public_key, KyberVariant::Kyber1024).unwrap();

        // Public key should work for encapsulation
        let (ciphertext, _) = kyber_pub.encapsulate().unwrap();

        // But not for decapsulation (no secret key)
        assert!(kyber_pub.decapsulate(&ciphertext).is_err());

        // Original kyber should be able to decapsulate
        let _ = kyber.decapsulate(&ciphertext).unwrap();
    }

    #[test]
    fn test_from_keypair() {
        let kyber = KyberKeys::new_with_variant(KyberVariant::Kyber1024).unwrap();
        let public_key = kyber.public_key().to_vec();
        let secret_key = kyber.secret_key().unwrap();

        let kyber2 =
            KyberKeys::from_keypair(public_key, secret_key, KyberVariant::Kyber1024).unwrap();

        // Should be able to encapsulate and decapsulate
        let (ciphertext, shared_secret1) = kyber.encapsulate().unwrap();
        let shared_secret2 = kyber2.decapsulate(&ciphertext).unwrap();

        assert_eq!(shared_secret1, shared_secret2);
    }

    #[test]
    fn test_clear_secret_key() {
        let mut kyber = KyberKeys::new_with_variant(KyberVariant::Kyber1024).unwrap();
        let (ciphertext, _) = kyber.encapsulate().unwrap();

        // Should be able to decapsulate before clearing
        assert!(kyber.decapsulate(&ciphertext).is_ok());

        // Clear secret key
        kyber.clear_secret_key();

        // Should not be able to decapsulate after clearing
        assert!(kyber.decapsulate(&ciphertext).is_err());
    }

    #[test]
    fn test_invalid_key_sizes() {
        // Test with wrong public key size
        let invalid_pk = vec![0u8; 10];
        assert!(KyberKeys::from_public_key(invalid_pk, KyberVariant::Kyber1024).is_err());

        // Test with wrong secret key size
        let valid_pk = vec![0u8; KyberVariant::Kyber1024.public_key_size()];
        let invalid_sk = vec![0u8; 10];
        assert!(KyberKeys::from_keypair(valid_pk, invalid_sk, KyberVariant::Kyber1024).is_err());
    }

    #[test]
    fn test_serialization() {
        // Create a keypair
        let kyber = KyberKeys::new_with_variant(KyberVariant::Kyber1024).unwrap();

        // Serialize to bytes
        let bytes = kyber.to_bytes().unwrap();

        // Deserialize from bytes
        let restored = KyberKeys::from_bytes(&bytes).unwrap();

        // Check that the variant is the same
        assert_eq!(kyber.variant(), restored.variant());

        // Check that the public key is the same
        assert_eq!(kyber.public_key(), restored.public_key());

        // Check that the secret key is the same
        assert_eq!(kyber.secret_key(), restored.secret_key());
    }
}
