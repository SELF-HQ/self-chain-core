use crate::crypto::CryptoResult;

/// Common trait for key pair generation and management
pub trait KeyPair {
    /// Create a new key pair
    fn new() -> CryptoResult<Self> where Self: Sized;
    /// Optional convenience alias – default to `new()`
    #[inline]
    fn generate() -> CryptoResult<Self> where Self: Sized { Self::new() }
    
    /// Create a key pair from an existing private key
    fn from_private_key(private_key: &[u8]) -> CryptoResult<Self> where Self: Sized;
    
    /// Get the public key
    fn public_key(&self) -> &[u8];
    
    /// Get the private key if available
    fn private_key(&self) -> Option<&[u8]>;
    
    /// Check if the key pair has a private key component
    fn has_private_key(&self) -> bool {
        self.private_key().is_some()
    }
    
    /// Convert the key pair to bytes for storage/transmission
    fn to_bytes(&self) -> CryptoResult<Vec<u8>>;
    
    /// Load a key pair from bytes
    fn from_bytes(bytes: &[u8]) -> CryptoResult<Self> where Self: Sized;
    
    /// Get the algorithm identifier associated with this key pair
    fn algorithm_id(&self) -> u8;
}

/// Trait for signing operations
pub trait Signer {
    /// Sign a message using the private key
    fn sign(&self, message: &[u8]) -> CryptoResult<Vec<u8>>;
    
    /// Get the algorithm identifier associated with this signer
    fn algorithm_id(&self) -> u8;
}

/// Trait for signature verification
pub trait Verifier {
    /// Verify a signature against a message using the public key
    fn verify(&self, message: &[u8], signature: &[u8]) -> CryptoResult<bool>;
    
    /// Get the algorithm identifier associated with this verifier
    fn algorithm_id(&self) -> u8;
}

/// Trait for key encapsulation mechanism (KEM)
pub trait KeyEncapsulation {
    /// Encapsulate a shared secret using a public key
    fn encapsulate(&self) -> CryptoResult<(Vec<u8>, Vec<u8>)>;
    
    /// Decapsulate a shared secret using a private key
    fn decapsulate(&self, ciphertext: &[u8]) -> CryptoResult<Vec<u8>>;
    
    /// Get the algorithm identifier associated with this KEM
    fn algorithm_id(&self) -> u8;
}

/// Trait for serialization and deserialization
pub trait CryptoSerialize {
    /// Serialize to bytes
    fn serialize(&self) -> CryptoResult<Vec<u8>>;
    
    /// Deserialize from bytes
    fn deserialize(bytes: &[u8]) -> CryptoResult<Self> where Self: Sized;
}
