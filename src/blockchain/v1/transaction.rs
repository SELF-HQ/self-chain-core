//! PoAI v1 Transaction Type
//!
//! Spec-compliant transaction structure as defined in `docs/POAI_SPECIFICATION.md`.
//!
//! ## Wire Format
//!
//! Transaction hash calculation uses SHA-256 with domain separation:
//! ```text
//! Hash = SHA256("self-chain-transaction-v1" || bincode(tx_without_sig))
//! ```
//!
//! The signature and public_key fields are excluded from the hash to allow
//! signature verification.

/// PoAI v1 Transaction (spec-compliant)
///
/// This is the canonical transaction format for the v1 protocol.
///
/// ## Canonical Encoding Order
///
/// 1. `nonce` (u64, little-endian)
/// 2. `chain_id` (string, UTF-8, length-prefixed)
/// 3. `sender` (string, hex, length-prefixed)
/// 4. `recipient` (Option<string>, hex, length-prefixed, None = empty)
/// 5. `data` (length-prefixed bytes)
/// 6. `point_price` (u64, little-endian)
/// 7. `timestamp` (u64, little-endian)
/// 8. `public_key` (32 bytes)
/// 9. `signature` (64 bytes)
///
/// ## Key Differences from Production
///
/// | Field | Production | v1 Spec |
/// |-------|-----------|---------|
/// | `nonce` | Not present | `u64` (replay protection) |
/// | `chain_id` | Not present | `String` (replay protection) |
/// | `point_price` | Not present | `u64` (fee in points) |
/// | `public_key` | Not present | `[u8; 32]` (Ed25519) |
/// | `signature` | `String` | `[u8; 64]` (Ed25519) |
///
/// ## Signature Verification
///
/// ```text
/// message = "self-chain-transaction-v1" || tx_hash
/// valid = Ed25519.verify(public_key, message, signature)
/// ```
///
/// ## Serialization
///
/// Production uses `bincode` with `#[serde(with = "serde_bytes")]` for byte arrays.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    /// Account nonce (prevents replay attacks)
    ///
    /// Each account has a sequential nonce starting from 0.
    /// Transactions must include nonce = account.nonce + 1.
    pub nonce: u64,
    
    /// Chain identifier (prevents cross-chain replay)
    ///
    /// Must match the target chain's `CHAIN_ID`.
    pub chain_id: String,
    
    /// Sender account address (hex-encoded)
    pub sender: String,
    
    /// Recipient address (hex-encoded), or None for contract deployment
    pub recipient: Option<String>,
    
    /// Transaction payload (arbitrary bytes)
    pub data: Vec<u8>,
    
    /// PointPrice for this transaction (fee in points)
    ///
    /// Higher PointPrice increases selection priority in the
    /// 20/20/50/10 algorithm.
    pub point_price: u64,
    
    /// Transaction timestamp (Unix seconds)
    pub timestamp: u64,
    
    /// Ed25519 public key (32 bytes)
    pub public_key: [u8; 32],
    
    /// Ed25519 signature (64 bytes)
    pub signature: [u8; 64],
}

impl Transaction {
    /// Domain separation prefix for transaction signatures
    pub const DOMAIN_PREFIX: &'static [u8] = b"self-chain-transaction-v1";
    
    /// Create a new unsigned transaction
    pub fn new(
        nonce: u64,
        chain_id: String,
        sender: String,
        recipient: Option<String>,
        data: Vec<u8>,
        point_price: u64,
        timestamp: u64,
    ) -> Self {
        Self {
            nonce,
            chain_id,
            sender,
            recipient,
            data,
            point_price,
            timestamp,
            public_key: [0u8; 32],
            signature: [0u8; 64],
        }
    }
    
    /// Check if transaction has a recipient (transfer) or not (deployment)
    pub fn is_transfer(&self) -> bool {
        self.recipient.is_some()
    }
    
    /// Get estimated size in bytes
    pub fn estimated_size(&self) -> usize {
        // Fixed fields: nonce(8) + timestamp(8) + point_price(8) + pubkey(32) + sig(64) = 120
        // Variable fields: chain_id + sender + recipient + data
        120 + self.chain_id.len() 
            + self.sender.len() 
            + self.recipient.as_ref().map(|r| r.len()).unwrap_or(0)
            + self.data.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_transaction_structure() {
        let tx = Transaction {
            nonce: 1,
            chain_id: "self-chain-mainnet".to_string(),
            sender: "a1b2c3d4e5f6".to_string(),
            recipient: Some("f6e5d4c3b2a1".to_string()),
            data: vec![],
            point_price: 1000,
            timestamp: 1704067200,
            public_key: [0u8; 32],
            signature: [0u8; 64],
        };
        
        assert_eq!(tx.nonce, 1);
        assert_eq!(tx.chain_id, "self-chain-mainnet");
        assert_eq!(tx.point_price, 1000);
        assert!(tx.is_transfer());
    }
    
    #[test]
    fn test_transaction_new() {
        let tx = Transaction::new(
            5,
            "test-chain".to_string(),
            "sender123".to_string(),
            Some("recipient456".to_string()),
            vec![1, 2, 3],
            500,
            1704067200,
        );
        
        assert_eq!(tx.nonce, 5);
        assert_eq!(tx.point_price, 500);
        assert_eq!(tx.data, vec![1, 2, 3]);
        // Unsigned transaction has zero signature
        assert_eq!(tx.signature, [0u8; 64]);
    }
    
    #[test]
    fn test_deployment_transaction() {
        let tx = Transaction::new(
            0,
            "test-chain".to_string(),
            "deployer".to_string(),
            None, // No recipient = deployment
            vec![0xDE, 0xAD, 0xBE, 0xEF],
            100,
            1704067200,
        );
        
        assert!(!tx.is_transfer());
        assert!(tx.recipient.is_none());
    }
    
    #[test]
    fn test_estimated_size() {
        let tx = Transaction::new(
            0,
            "chain".to_string(),  // 5 bytes
            "sender".to_string(), // 6 bytes
            Some("recipient".to_string()), // 9 bytes
            vec![1, 2, 3, 4, 5],  // 5 bytes
            100,
            1704067200,
        );
        
        // 120 (fixed) + 5 + 6 + 9 + 5 = 145
        assert_eq!(tx.estimated_size(), 145);
    }
}
