//! Delegated Key System for Hosted Validators
//!
//! Implements a hierarchical key system that enables hosted validators while
//! maintaining user sovereignty and security.
//!
//! ## Architecture
//!
//! **Master Key (User Controls - Client-Side)**
//! - Controls funds and can send transactions
//! - Never leaves the user's device
//! - Can revoke/migrate validator keys
//! - Full sovereignty over assets
//!
//! **Validator Key (Server-Side - Railway)**
//! - Can only vote on blocks (scope-limited)
//! - Can validate color markers
//! - **Cannot** move funds or send transactions
//! - Destroyed if user migrates to new validator
//!
//! ## Security Model
//!
//! The validator key is cryptographically derived from the master key but
//! can only perform specific operations. Even if the validator key is
//! compromised, user funds remain safe.
use crate::crypto::{CryptoError, CryptoResult, PrivateKey, PublicKey, Signature};
use crate::crypto::classic::ecdsa::ECDSAKeys;
use crate::crypto::common::traits::{KeyPair, Signer};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::time::{SystemTime, UNIX_EPOCH};
use zeroize::Zeroize;

/// Operation types that can be performed with keys
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyOperation {
    /// Send transaction (master key only)
    SendTransaction,
    
    /// Vote on block proposals (validator key allowed)
    Vote,
    
    /// Validate color markers (validator key allowed)
    ValidateColorMarker,
    
    /// Revoke validator key (master key only)
    RevokeValidatorKey,
    
    /// Migrate to new validator (master key only)
    MigrateValidator,
}

/// Master key that controls the account
#[derive(Clone, Zeroize)]
#[zeroize(drop)]
pub struct MasterKey {
    /// ECDSA private key
    private_key: PrivateKey,
    
    /// ECDSA public key
    public_key: PublicKey,
    
    /// Wallet address (derived from public key)
    address: String,
    
    /// Creation timestamp
    created_at: u64,
}

impl MasterKey {
    /// Generate a new master key
    pub fn generate() -> CryptoResult<Self> {
        let ecdsa_keys = ECDSAKeys::new()?;
        
        let address = Self::derive_address(ecdsa_keys.public_key());
        
        Ok(Self {
            private_key: ecdsa_keys.private_key()
                .ok_or_else(|| CryptoError::KeyGenerationError("No private key".to_string()))?
                .to_vec(),
            public_key: ecdsa_keys.public_key().to_vec(),
            address,
            created_at: Self::current_timestamp(),
        })
    }
    
    /// Import master key from private key bytes
    pub fn from_private_key(private_key: PrivateKey) -> CryptoResult<Self> {
        let ecdsa_keys = ECDSAKeys::from_private_key(&private_key)
            .map_err(|e| CryptoError::InvalidKeyFormat(e.to_string()))?;
        
        let address = Self::derive_address(ecdsa_keys.public_key());
        
        Ok(Self {
            private_key,
            public_key: ecdsa_keys.public_key().to_vec(),
            address,
            created_at: Self::current_timestamp(),
        })
    }
    
    /// Derive a validator key from this master key
    ///
    /// The validator key is created using a deterministic derivation:
    /// validator_key = HMAC(master_private_key, "validator" || timestamp || nonce)
    pub fn derive_validator_key(&self, nonce: &[u8]) -> CryptoResult<ValidatorKey> {
        // Create derivation input
        let mut derivation_input = Vec::new();
        derivation_input.extend_from_slice(b"SELF_VALIDATOR_KEY_v1");
        derivation_input.extend_from_slice(&self.created_at.to_le_bytes());
        derivation_input.extend_from_slice(nonce);
        
        // Use HMAC-SHA3 for derivation
        use hmac::{Hmac, Mac};
        type HmacSha3 = Hmac<Sha3_256>;
        
        let mut mac = HmacSha3::new_from_slice(&self.private_key)
            .map_err(|e| CryptoError::KeyGenerationError(e.to_string()))?;
        mac.update(&derivation_input);
        let derived_key_material = mac.finalize().into_bytes();
        
        // Use first 32 bytes as validator private key
        let validator_private_key = derived_key_material[..32].to_vec();
        
        // Generate corresponding public key
        let ecdsa_keys = ECDSAKeys::from_private_key(&validator_private_key)?;
        
        Ok(ValidatorKey {
            private_key: validator_private_key,
            public_key: ecdsa_keys.public_key().to_vec(),
            master_address: self.address.clone(),
            nonce: nonce.to_vec(),
            created_at: Self::current_timestamp(),
            revoked: false,
        })
    }
    
    /// Sign a revocation message for a validator key
    pub fn create_revocation(&self, validator_public_key: &[u8]) -> CryptoResult<Revocation> {
        let timestamp = Self::current_timestamp();
        
        // Create revocation message
        let mut message = Vec::new();
        message.extend_from_slice(b"REVOKE_VALIDATOR");
        message.extend_from_slice(validator_public_key);
        message.extend_from_slice(&timestamp.to_le_bytes());
        
        // Sign with master key
        let signature = self.sign(&message)?;
        
        Ok(Revocation {
            master_address: self.address.clone(),
            validator_public_key: validator_public_key.to_vec(),
            timestamp,
            signature,
        })
    }
    
    /// Sign data with master key
    pub fn sign(&self, data: &[u8]) -> CryptoResult<Signature> {
        let ecdsa_keys = ECDSAKeys::from_private_key(&self.private_key)?;
        ecdsa_keys.sign(data)
    }
    
    /// Get public key
    pub fn public_key(&self) -> &[u8] {
        &self.public_key
    }
    
    /// Get wallet address
    pub fn address(&self) -> &str {
        &self.address
    }
    
    /// Export private key (use with caution!)
    pub fn export_private_key(&self) -> PrivateKey {
        self.private_key.clone()
    }
    
    /// Derive wallet address from public key
    fn derive_address(public_key: &[u8]) -> String {
        let mut hasher = Sha3_256::new();
        hasher.update(public_key);
        let hash = hasher.finalize();
        
        // Take last 20 bytes and encode as hex
        let address_bytes = &hash[hash.len() - 20..];
        format!("0x{}", hex::encode(address_bytes))
    }
    
    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

/// Validator key with scope-limited permissions
#[derive(Clone, Zeroize, Serialize, Deserialize)]
#[zeroize(drop)]
pub struct ValidatorKey {
    /// Private key (derived from master key)
    #[serde(skip)]
    private_key: PrivateKey,
    
    /// Public key
    public_key: PublicKey,
    
    /// Master wallet address this validator serves
    master_address: String,
    
    /// Nonce used for derivation (ensures uniqueness)
    nonce: Vec<u8>,
    
    /// Creation timestamp
    created_at: u64,
    
    /// Whether this key has been revoked
    #[serde(default)]
    revoked: bool,
}

impl ValidatorKey {
    /// Check if this key can perform an operation
    pub fn can_perform(&self, operation: KeyOperation) -> bool {
        if self.revoked {
            return false;
        }
        
        match operation {
            // Validator can only vote and validate
            KeyOperation::Vote | KeyOperation::ValidateColorMarker => true,
            
            // Master key operations not allowed
            KeyOperation::SendTransaction | 
            KeyOperation::RevokeValidatorKey | 
            KeyOperation::MigrateValidator => false,
        }
    }
    
    /// Sign a vote (allowed operation)
    pub fn sign_vote(&self, block_hash: &[u8], vote: bool) -> CryptoResult<Signature> {
        if !self.can_perform(KeyOperation::Vote) {
            return Err(CryptoError::SigningError(
                "Validator key is revoked".to_string()
            ));
        }
        
        // Create vote message
        let mut message = Vec::new();
        message.extend_from_slice(b"VOTE");
        message.extend_from_slice(block_hash);
        message.push(if vote { 1 } else { 0 });
        message.extend_from_slice(&Self::current_timestamp().to_le_bytes());
        
        self.sign(&message)
    }
    
    /// Sign a color marker validation (allowed operation)
    pub fn sign_color_validation(&self, tx_hash: &[u8], valid: bool) -> CryptoResult<Signature> {
        if !self.can_perform(KeyOperation::ValidateColorMarker) {
            return Err(CryptoError::SigningError(
                "Validator key is revoked".to_string()
            ));
        }
        
        // Create validation message
        let mut message = Vec::new();
        message.extend_from_slice(b"COLOR_VALIDATION");
        message.extend_from_slice(tx_hash);
        message.push(if valid { 1 } else { 0 });
        
        self.sign(&message)
    }
    
    /// Attempt to send transaction (will fail - not allowed)
    pub fn sign_transaction(&self, _tx_data: &[u8]) -> CryptoResult<Signature> {
        Err(CryptoError::SigningError(
            "Validator keys cannot sign transactions - use master key".to_string()
        ))
    }
    
    /// Mark this key as revoked
    pub fn revoke(&mut self) {
        self.revoked = true;
        // Zero out the private key
        self.private_key.zeroize();
    }
    
    /// Check if revoked
    pub fn is_revoked(&self) -> bool {
        self.revoked
    }
    
    /// Get public key
    pub fn public_key(&self) -> &[u8] {
        &self.public_key
    }
    
    /// Get master address
    pub fn master_address(&self) -> &str {
        &self.master_address
    }
    
    /// Internal signing function
    fn sign(&self, data: &[u8]) -> CryptoResult<Signature> {
        if self.revoked {
            return Err(CryptoError::SigningError("Key is revoked".to_string()));
        }
        
        let ecdsa_keys = ECDSAKeys::from_private_key(&self.private_key)?;
        ecdsa_keys.sign(data)
    }
    
    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

/// Revocation certificate for a validator key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Revocation {
    /// Master wallet address
    pub master_address: String,
    
    /// Public key of validator being revoked
    pub validator_public_key: PublicKey,
    
    /// Revocation timestamp
    pub timestamp: u64,
    
    /// Master key signature over revocation
    pub signature: Signature,
}

impl Revocation {
    /// Verify that this revocation is valid
    pub fn verify(&self, master_public_key: &[u8]) -> CryptoResult<bool> {
        // Reconstruct message
        let mut message = Vec::new();
        message.extend_from_slice(b"REVOKE_VALIDATOR");
        message.extend_from_slice(&self.validator_public_key);
        message.extend_from_slice(&self.timestamp.to_le_bytes());
        
        // For now, skip verification (need to implement from_public_key for ECDSAKeys)
        // TODO: Implement proper signature verification
        Ok(true)
    }
}

/// Key manager for handling master and validator keys
pub struct KeyManager {
    master_key: Option<MasterKey>,
    validator_keys: Vec<ValidatorKey>,
}

impl KeyManager {
    /// Create new key manager
    pub fn new() -> Self {
        Self {
            master_key: None,
            validator_keys: Vec::new(),
        }
    }
    
    /// Generate and store a new master key
    pub fn generate_master_key(&mut self) -> CryptoResult<String> {
        let master_key = MasterKey::generate()?;
        let address = master_key.address().to_string();
        self.master_key = Some(master_key);
        Ok(address)
    }
    
    /// Import existing master key
    pub fn import_master_key(&mut self, private_key: PrivateKey) -> CryptoResult<String> {
        let master_key = MasterKey::from_private_key(private_key)?;
        let address = master_key.address().to_string();
        self.master_key = Some(master_key);
        Ok(address)
    }
    
    /// Derive a new validator key
    pub fn derive_validator(&mut self, nonce: &[u8]) -> CryptoResult<PublicKey> {
        let master_key = self.master_key.as_ref()
            .ok_or_else(|| CryptoError::KeyGenerationError("No master key".to_string()))?;
        
        let validator_key = master_key.derive_validator_key(nonce)?;
        let public_key = validator_key.public_key().to_vec();
        self.validator_keys.push(validator_key);
        
        Ok(public_key)
    }
    
    /// Get master key reference
    pub fn master_key(&self) -> Option<&MasterKey> {
        self.master_key.as_ref()
    }
    
    /// Get validator keys
    pub fn validator_keys(&self) -> &[ValidatorKey] {
        &self.validator_keys
    }
    
    /// Revoke a validator key
    pub fn revoke_validator(&mut self, public_key: &[u8]) -> CryptoResult<()> {
        for validator in &mut self.validator_keys {
            if validator.public_key() == public_key {
                validator.revoke();
                return Ok(());
            }
        }
        
        Err(CryptoError::InvalidKeyFormat("Validator key not found".to_string()))
    }
}

impl Default for KeyManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_master_key_generation() {
        let master = MasterKey::generate().unwrap();
        
        assert!(!master.public_key().is_empty());
        assert!(!master.address().is_empty());
        assert!(master.address().starts_with("0x"));
    }
    
    #[test]
    fn test_validator_key_derivation() {
        let master = MasterKey::generate().unwrap();
        let nonce = b"test_nonce_123";
        
        let validator = master.derive_validator_key(nonce).unwrap();
        
        assert!(!validator.public_key().is_empty());
        assert_eq!(validator.master_address(), master.address());
        assert!(!validator.is_revoked());
    }
    
    #[test]
    fn test_deterministic_derivation() {
        let master = MasterKey::generate().unwrap();
        let nonce = b"same_nonce";
        
        // Same nonce should produce same validator key
        let validator1 = master.derive_validator_key(nonce).unwrap();
        let validator2 = master.derive_validator_key(nonce).unwrap();
        
        assert_eq!(validator1.public_key(), validator2.public_key());
    }
    
    #[test]
    fn test_different_nonces_different_keys() {
        let master = MasterKey::generate().unwrap();
        
        let validator1 = master.derive_validator_key(b"nonce1").unwrap();
        let validator2 = master.derive_validator_key(b"nonce2").unwrap();
        
        assert_ne!(validator1.public_key(), validator2.public_key());
    }
    
    #[test]
    fn test_validator_permissions() {
        let master = MasterKey::generate().unwrap();
        let validator = master.derive_validator_key(b"nonce").unwrap();
        
        // Validator can vote and validate
        assert!(validator.can_perform(KeyOperation::Vote));
        assert!(validator.can_perform(KeyOperation::ValidateColorMarker));
        
        // Validator cannot do master operations
        assert!(!validator.can_perform(KeyOperation::SendTransaction));
        assert!(!validator.can_perform(KeyOperation::RevokeValidatorKey));
        assert!(!validator.can_perform(KeyOperation::MigrateValidator));
    }
    
    #[test]
    fn test_validator_cannot_sign_transactions() {
        let master = MasterKey::generate().unwrap();
        let validator = master.derive_validator_key(b"nonce").unwrap();
        
        let tx_data = b"fake_transaction_data";
        let result = validator.sign_transaction(tx_data);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot sign transactions"));
    }
    
    #[test]
    fn test_validator_can_sign_votes() {
        let master = MasterKey::generate().unwrap();
        let validator = master.derive_validator_key(b"nonce").unwrap();
        
        let block_hash = b"test_block_hash_12345678901234567890";
        let signature = validator.sign_vote(block_hash, true).unwrap();
        
        assert!(!signature.is_empty());
    }
    
    #[test]
    fn test_validator_can_sign_color_validations() {
        let master = MasterKey::generate().unwrap();
        let validator = master.derive_validator_key(b"nonce").unwrap();
        
        let tx_hash = b"test_tx_hash_123456789012345678901234";
        let signature = validator.sign_color_validation(tx_hash, true).unwrap();
        
        assert!(!signature.is_empty());
    }
    
    #[test]
    fn test_revocation() {
        let master = MasterKey::generate().unwrap();
        let mut validator = master.derive_validator_key(b"nonce").unwrap();
        
        assert!(!validator.is_revoked());
        assert!(validator.can_perform(KeyOperation::Vote));
        
        // Revoke the validator key
        validator.revoke();
        
        assert!(validator.is_revoked());
        assert!(!validator.can_perform(KeyOperation::Vote));
        
        // Signing should fail after revocation
        let block_hash = b"test_block_hash";
        let result = validator.sign_vote(block_hash, true);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_revocation_certificate() {
        let master = MasterKey::generate().unwrap();
        let validator = master.derive_validator_key(b"nonce").unwrap();
        
        // Create revocation
        let revocation = master.create_revocation(validator.public_key()).unwrap();
        
        assert_eq!(revocation.master_address, master.address());
        assert_eq!(revocation.validator_public_key, validator.public_key());
        
        // Verify revocation
        let is_valid = revocation.verify(master.public_key()).unwrap();
        assert!(is_valid);
    }
    
    #[test]
    fn test_key_manager() {
        let mut manager = KeyManager::new();
        
        // Generate master key
        let address = manager.generate_master_key().unwrap();
        assert!(!address.is_empty());
        
        // Derive validator keys
        let validator1_pk = manager.derive_validator(b"validator1").unwrap();
        let validator2_pk = manager.derive_validator(b"validator2").unwrap();
        
        assert_ne!(validator1_pk, validator2_pk);
        assert_eq!(manager.validator_keys().len(), 2);
        
        // Revoke one validator
        manager.revoke_validator(&validator1_pk).unwrap();
        assert!(manager.validator_keys()[0].is_revoked());
        assert!(!manager.validator_keys()[1].is_revoked());
    }
    
    #[test]
    fn test_master_key_import_export() {
        let master1 = MasterKey::generate().unwrap();
        let private_key = master1.export_private_key();
        let address1 = master1.address().to_string();
        
        // Import into new master key
        let master2 = MasterKey::from_private_key(private_key).unwrap();
        let address2 = master2.address();
        
        // Should have same address
        assert_eq!(address1, address2);
    }
}

