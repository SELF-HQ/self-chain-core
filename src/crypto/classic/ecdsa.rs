use secp256k1::{Secp256k1, SecretKey, PublicKey, Message, ecdsa};
use bitcoin_hashes::sha256;
use hex::{encode, decode};
// Need to use StdRng for compatibility with secp256k1
use rand::rngs::StdRng;
use rand::SeedableRng;
use rand_0_8::rngs::OsRng;
use rand_0_8::RngCore;

use crate::crypto::{CryptoResult, CryptoError, CryptoAlgorithm};
use crate::crypto::common::traits::{KeyPair, Signer, Verifier};

/// ECDSA key pair using secp256k1 curve (compatible with original blockchain implementation)
#[derive(Debug, Clone)]
pub struct ECDSAKeys {
    public_key: Vec<u8>,
    secret_key: Option<Vec<u8>>,
    pub_key_obj: PublicKey,
    context: Secp256k1<secp256k1::All>,
}

#[derive(Debug, Clone)]
pub struct ECDSASignature {
    signature_bytes: Vec<u8>,
    public_key: Vec<u8>,
}

impl KeyPair for ECDSAKeys {
    fn new() -> CryptoResult<Self> {
        let context = Secp256k1::new();
        // Create a StdRng from OsRng to be compatible with secp256k1 0.31.0
        let mut seed = [0u8; 32];
        OsRng.fill_bytes(&mut seed);
        let mut rng = StdRng::from_seed(seed);
        
        // Generate key pair
        let (secret_key, pub_key) = context.generate_keypair(&mut rng);
        let pub_key_ser = pub_key.serialize().to_vec();
        let secret_key_ser = secret_key.secret_bytes().to_vec();

        Ok(Self {
            public_key: pub_key_ser,
            secret_key: Some(secret_key_ser),
            pub_key_obj: pub_key,
            context,
        })
    }
    
    fn from_private_key(private_key: &[u8]) -> CryptoResult<Self> {
        let context = Secp256k1::new();

        // Recreate keys from private key
        let key_array: [u8; 32] = private_key.try_into()
            .map_err(|_| CryptoError::KeyGenerationError("Invalid key length".to_string()))?;
        let secret_key = SecretKey::from_byte_array(key_array)
            .map_err(|e| CryptoError::KeyGenerationError(e.to_string()))?;
        
        let pub_key = PublicKey::from_secret_key(&context, &secret_key);
        let pub_key_ser = pub_key.serialize().to_vec();
        
        Ok(Self {
            public_key: pub_key_ser,
            secret_key: Some(private_key.to_vec()),
            pub_key_obj: pub_key,
            context,
        })
    }
    
    fn public_key(&self) -> &[u8] {
        &self.public_key
    }
    
    fn private_key(&self) -> Option<&[u8]> {
        self.secret_key.as_deref()
    }
    
    fn algorithm_id(&self) -> u8 {
        1 // ECDSA using Secp256k1
    }
    
    fn to_bytes(&self) -> CryptoResult<Vec<u8>> {
        // Format: [version_byte][pubkey_len: 1 byte][pubkey][optional secret key]
        let mut result = Vec::new();
        
        // Version byte (algorithm and version info)
        result.push(CryptoAlgorithm::ECDSAsecp256k1 as u8);
        
        // Public key length and data
        result.push(self.public_key.len() as u8);
        result.extend_from_slice(&self.public_key);
        
        // Optional private key
        if let Some(sk) = &self.secret_key {
            result.push(1); // Has private key flag
            result.push(sk.len() as u8);
            result.extend_from_slice(sk);
        } else {
            result.push(0); // No private key flag
        }
        
        Ok(result)
    }
    
    fn from_bytes(bytes: &[u8]) -> CryptoResult<Self> {
        if bytes.len() < 3 {
            return Err(CryptoError::SerializationError(
                "Invalid ECDSA key format: too short".into()
            ));
        }
        
        // Check algorithm version
        if bytes[0] != CryptoAlgorithm::ECDSAsecp256k1 as u8 {
            return Err(CryptoError::InvalidAlgorithm(
                format!("Expected ECDSA algorithm, got {}", bytes[0])
            ));
        }
        
        let mut pos = 1;
        
        // Read public key
        let pubkey_len = bytes[pos] as usize;
        pos += 1;
        
        if pos + pubkey_len > bytes.len() {
            return Err(CryptoError::SerializationError("Invalid public key length".into()));
        }
        
        let pubkey_bytes = &bytes[pos..pos + pubkey_len];
        pos += pubkey_len;
        
        // Parse the public key
        let context = Secp256k1::new();
        let pub_key_obj = PublicKey::from_slice(pubkey_bytes)
            .map_err(|e| CryptoError::SerializationError(e.to_string()))?;
        
        // Check for private key
        let secret_key = if pos < bytes.len() && bytes[pos] == 1 {
            pos += 1;
            let sk_len = bytes[pos] as usize;
            pos += 1;
            
            if pos + sk_len > bytes.len() {
                return Err(CryptoError::SerializationError("Invalid private key length".into()));
            }
            
            Some(bytes[pos..pos + sk_len].to_vec())
        } else {
            None
        };
        
        Ok(Self {
            public_key: pubkey_bytes.to_vec(),
            secret_key,
            pub_key_obj,
            context,
        })
    }
}

impl Signer for ECDSAKeys {
    fn sign(&self, message: &[u8]) -> CryptoResult<Vec<u8>> {
        // Need private key to sign
        let secret_bytes = self.private_key().ok_or_else(|| {
            CryptoError::SigningError("Cannot sign without private key".into())
        })?;

        let key_array: [u8; 32] = secret_bytes.try_into()
            .map_err(|_| CryptoError::SigningError("Invalid key length".to_string()))?;
        let secret_key = SecretKey::from_byte_array(key_array)
            .map_err(|e| CryptoError::SigningError(e.to_string()))?;
        
        // Hash the message with SHA-256 (compatible with original implementation)
        let digest = sha256::Hash::hash(message);

        // Create a Message object from the digest
        let message = Message::from_digest(*digest.as_byte_array());
        
        // Sign the message
        // For secp256k1 0.31.0, message needs to be converted to Message, not passed by reference
        let signature = self.context.sign_ecdsa(message, &secret_key);
        
        // Return the serialized signature
        Ok(signature.serialize_compact().to_vec())
    }
    
    fn algorithm_id(&self) -> u8 {
        CryptoAlgorithm::ECDSAsecp256k1 as u8
    }
}

impl Verifier for ECDSAKeys {
    fn verify(&self, message: &[u8], signature: &[u8]) -> CryptoResult<bool> {
        // Hash the message with SHA-256 (compatible with original implementation)
        let digest = sha256::Hash::hash(message);

        // Create a Message object from the digest
        let message = Message::from_digest(*digest.as_byte_array());
        
        // Parse the signature
        let sig = ecdsa::Signature::from_compact(signature)
            .map_err(|e| CryptoError::VerificationError(e.to_string()))?;
        
        // Verify 
        // For secp256k1 0.31.0, message needs to be converted to Message, not passed by reference
        match self.context.verify_ecdsa(message, &sig, &self.pub_key_obj) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
    
    fn algorithm_id(&self) -> u8 {
        CryptoAlgorithm::ECDSAsecp256k1 as u8
    }
}

impl ECDSASignature {
    pub fn new(signature: Vec<u8>, public_key: Vec<u8>) -> Self {
        Self {
            signature_bytes: signature,
            public_key,
        }
    }
    
    pub fn signature(&self) -> &Vec<u8> {
        &self.signature_bytes
    }
    
    pub fn public_key(&self) -> &Vec<u8> {
        &self.public_key
    }
    
    pub fn verify(&self, message: &[u8]) -> CryptoResult<bool> {
        let context = Secp256k1::new();
        
        // Hash the message with SHA-256
        let digest = sha256::Hash::hash(message);

        // Create Message object
        let msg = Message::from_digest(*digest.as_byte_array());
        
        // Parse public key
        let pub_key = PublicKey::from_slice(&self.public_key)
            .map_err(|e| CryptoError::VerificationError(e.to_string()))?;
        
        // Parse signature
        let sig = ecdsa::Signature::from_compact(&self.signature_bytes)
            .map_err(|e| CryptoError::VerificationError(e.to_string()))?;
        
        // Verify
        // For secp256k1 0.31.0, message needs to be converted to Message, not passed by reference
        match context.verify_ecdsa(msg, &sig, &pub_key) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
    
    pub fn to_hex(&self) -> String {
        encode(&self.signature_bytes)
    }

    pub fn from_hex(signature_hex: &str, public_key: Vec<u8>) -> CryptoResult<Self> {
        let signature_bytes = decode(signature_hex)
            .map_err(|e| CryptoError::SerializationError(e.to_string()))?;
            
        Ok(Self { 
            signature_bytes,
            public_key,
        })
    }
}
