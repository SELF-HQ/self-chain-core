//! # PoAI Validator
//!
//! The Validator implements the core validation logic for the PoAI consensus mechanism.
//!
//! ## Color Marker System
//!
//! Each wallet in the system has a HEX color derived from transaction history:
//!
//! 1. When a transaction is signed, a HEX is calculated from the transaction hash
//! 2. The new wallet color = current wallet color + transaction HEX
//! 3. Validators store wallet colors to verify transactions without full blockchain
//!
//! ## Validation Process
//!
//! Blocks are validated through:
//! 1. Transaction structure verification
//! 2. Color marker validation
//! 3. Block efficiency calculation
//!
//! ## Usage
//!
//! ```rust,ignore
//! use self_chain_core::consensus::Validator;
//!
//! let validator = Validator::new();
//! let is_valid = validator.validate_transaction(&tx)?;
//! ```

use crate::blockchain::{Block, Transaction};
use crate::consensus::cache::ValidationCache;
use crate::consensus::error::ConsensusError;
use crate::consensus::metrics::ConsensusMetrics;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Configuration for the validator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorConfig {
    /// Minimum hours of activity required to validate
    pub min_active_hours: u64,
    /// Minimum token balance required to validate
    pub min_balance: u64,
    /// Time window for validation in seconds
    pub validation_window: u64,
}

impl Default for ValidatorConfig {
    fn default() -> Self {
        Self {
            min_active_hours: 24,
            min_balance: 1000000,    // 1000 tokens
            validation_window: 3600, // 1 hour
        }
    }
}

/// Wallet color state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletColor {
    /// Wallet address
    pub address: String,
    /// Current HEX color (6 characters, e.g., "a1b2c3")
    pub color: String,
    /// Unix timestamp of last update
    pub last_update: u64,
}

/// PoAI Validator for block and transaction validation
#[derive(Debug)]
pub struct Validator {
    config: ValidatorConfig,
    wallet_colors: Arc<tokio::sync::RwLock<HashMap<String, WalletColor>>>,
    metrics: Arc<ConsensusMetrics>,
    cache: Arc<ValidationCache>,
}

impl Validator {
    /// Create a new validator with default configuration
    pub fn new(metrics: Arc<ConsensusMetrics>, cache: Arc<ValidationCache>) -> Self {
        Self {
            config: ValidatorConfig::default(),
            wallet_colors: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            metrics,
            cache,
        }
    }

    /// Create a new validator with custom configuration
    pub fn with_config(
        config: ValidatorConfig,
        metrics: Arc<ConsensusMetrics>,
        cache: Arc<ValidationCache>,
    ) -> Self {
        Self {
            config,
            wallet_colors: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            metrics,
            cache,
        }
    }

    /// Validate a transaction using PoAI color marker rules
    pub async fn validate_transaction(&self, tx: &Transaction) -> Result<(), ConsensusError> {
        // Check cache first
        if let Some(result) = self.cache.get_cached_transaction_validation(tx).await {
            return if result.value {
                Ok(())
            } else {
                Err(ConsensusError::InvalidTransaction(
                    "Cached validation failed".to_string(),
                ))
            };
        }

        // 1. Basic transaction structure validation
        if !tx.verify() {
            self.metrics.increment_validation_failures("tx_structure");
            return Err(ConsensusError::TransactionValidationFailed(
                "Invalid transaction structure".to_string(),
            ));
        }

        // 2. Color marker validation
        let sender_color = self.get_wallet_color(&tx.sender).await?;
        let hex_tx = self.calculate_hex_transaction(tx)?;
        let new_color = self.calculate_new_color(&sender_color, &hex_tx)?;

        if !self.validate_color_transition(&sender_color, &new_color)? {
            self.metrics.increment_validation_failures("color_transition");
            return Err(ConsensusError::InvalidColorTransition);
        }

        // Cache result
        self.cache
            .cache_transaction_validation(tx, true, 100)
            .await?;
        self.metrics.increment_valid_transactions();
        Ok(())
    }

    /// Validate a block
    pub async fn validate_block(&self, block: &Block) -> Result<bool, ConsensusError> {
        // Check cache first
        if let Some(cached) = self.cache.get_cached_block_validation(block).await {
            if self.cache.is_cache_valid(&cached).await? {
                return Ok(cached.value);
            }
        }

        let start_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs_f64();

        // Calculate block efficiency
        let efficiency = self.calculate_block_efficiency(block).await?;
        self.metrics.set_block_efficiency(efficiency);

        // Validate all transactions
        for tx in &block.transactions {
            self.validate_transaction(tx).await?;
        }

        let duration = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs_f64() - start_time;
        self.metrics.observe_block_validation(duration);
        self.metrics.increment_blocks_validated();

        // Cache the result
        self.cache
            .cache_block_validation(block, true, efficiency as u64)
            .await?;

        Ok(true)
    }

    /// Get the current wallet color
    pub async fn get_wallet_color(&self, address: &str) -> Result<String> {
        let colors = self.wallet_colors.read().await;
        if let Some(color) = colors.get(address) {
            Ok(color.color.clone())
        } else {
            // Generate initial color for new wallet
            Ok(self.generate_initial_color())
        }
    }

    /// Update wallet color after transaction
    pub async fn update_wallet_color(&self, address: &str, color: &str) -> Result<()> {
        let mut colors = self.wallet_colors.write().await;
        colors.insert(
            address.to_string(),
            WalletColor {
                address: address.to_string(),
                color: color.to_string(),
                last_update: Self::current_timestamp(),
            },
        );
        Ok(())
    }

    /// Calculate HEX transaction per PoAI specification
    ///
    /// Per PoAI spec:
    /// 1. Divide transaction hash into 6 parts
    /// 2. For each part: recursively sum hex digits until single digit remains
    /// 3. Combine the 6 single digits into a 6-character HEX string
    pub fn calculate_hex_transaction(&self, tx: &Transaction) -> Result<String> {
        let hash = tx.hash();
        let hash_hex = hash;

        // Ensure we have enough data
        if hash_hex.len() < 6 {
            return Err(anyhow::anyhow!("Transaction hash too short for color calculation"));
        }

        // Divide hash into 6 parts
        let part_size = hash_hex.len() / 6;
        let mut hex_digits = Vec::new();

        for i in 0..6 {
            let start = i * part_size;
            let end = if i == 5 { hash_hex.len() } else { (i + 1) * part_size };
            let part = &hash_hex[start..end];
            let single_digit = Self::reduce_to_single_hex_digit(part)?;
            hex_digits.push(single_digit);
        }

        Ok(hex_digits.iter().collect())
    }

    /// Recursively sum hex digits until single digit remains
    fn reduce_to_single_hex_digit(hex_str: &str) -> Result<char> {
        if hex_str.is_empty() {
            return Ok('0');
        }

        let mut sum: u32 = 0;
        for c in hex_str.chars() {
            if let Some(digit) = c.to_digit(16) {
                sum += digit;
            }
        }

        // If sum is single hex digit (0-15), we're done
        if sum < 16 {
            return Ok(std::char::from_digit(sum, 16).unwrap().to_ascii_lowercase());
        }

        // Otherwise, convert to hex and recurse
        let hex_sum = format!("{:x}", sum);
        Self::reduce_to_single_hex_digit(&hex_sum)
    }

    /// Calculate new wallet color per PoAI specification
    ///
    /// new_color = (current_color + hex_tx) mod 0x1000000
    pub fn calculate_new_color(&self, current_color: &str, hex_tx: &str) -> Result<String> {
        if !self.is_valid_hex(current_color) {
            return Err(anyhow::anyhow!("Invalid current color format: {}", current_color));
        }
        if !self.is_valid_hex(hex_tx) {
            return Err(anyhow::anyhow!("Invalid hex transaction format: {}", hex_tx));
        }

        let current_num = u32::from_str_radix(current_color, 16)
            .map_err(|e| anyhow::anyhow!("Failed to parse current color: {}", e))?;
        let tx_num = u32::from_str_radix(hex_tx, 16)
            .map_err(|e| anyhow::anyhow!("Failed to parse hex transaction: {}", e))?;

        let new_num = (current_num + tx_num) % 0x1000000;
        Ok(format!("{:06x}", new_num))
    }

    /// Validate color transition
    pub fn validate_color_transition(&self, current: &str, new: &str) -> Result<bool> {
        Ok(self.is_valid_hex(current) && self.is_valid_hex(new))
    }

    /// Check if a string is a valid 6-character hex color
    fn is_valid_hex(&self, color: &str) -> bool {
        color.len() == 6 && color.chars().all(|c| c.is_ascii_hexdigit())
    }

    /// Generate initial color for new wallet
    fn generate_initial_color(&self) -> String {
        format!("{:06x}", rand::random::<u32>() % 0xFFFFFF)
    }

    /// Calculate block efficiency
    async fn calculate_block_efficiency(&self, block: &Block) -> Result<f64> {
        if block.transactions.is_empty() {
            return Ok(0.0);
        }

        // Simple efficiency: transaction count / target count
        let target = 100.0; // Target transactions per block
        let actual = block.transactions.len() as f64;
        Ok((actual / target).min(1.0) * 100.0)
    }

    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_validator() -> Validator {
        let registry = prometheus::Registry::new();
        let metrics = Arc::new(ConsensusMetrics::new(&registry).unwrap());
        let cache = Arc::new(ValidationCache::new(metrics.clone()));
        Validator::new(metrics, cache)
    }

    #[test]
    fn test_reduce_to_single_hex_digit() {
        // Single digit already
        assert_eq!(Validator::reduce_to_single_hex_digit("5").unwrap(), '5');
        assert_eq!(Validator::reduce_to_single_hex_digit("f").unwrap(), 'f');

        // Two digits
        assert_eq!(Validator::reduce_to_single_hex_digit("10").unwrap(), '1');
        assert_eq!(Validator::reduce_to_single_hex_digit("1e").unwrap(), 'f');

        // Multiple digits
        assert_eq!(Validator::reduce_to_single_hex_digit("a3f2").unwrap(), 'f');
    }

    #[test]
    fn test_calculate_new_color() {
        let validator = create_test_validator();

        // Simple addition
        let result = validator.calculate_new_color("000001", "000001").unwrap();
        assert_eq!(result, "000002");

        // Overflow wraps
        let result = validator.calculate_new_color("ffffff", "000002").unwrap();
        assert_eq!(result, "000001");
    }

    #[test]
    fn test_is_valid_hex() {
        let validator = create_test_validator();

        assert!(validator.is_valid_hex("000000"));
        assert!(validator.is_valid_hex("ffffff"));
        assert!(validator.is_valid_hex("abcdef"));
        assert!(validator.is_valid_hex("ABCDEF"));

        assert!(!validator.is_valid_hex(""));
        assert!(!validator.is_valid_hex("12345"));   // Too short
        assert!(!validator.is_valid_hex("1234567")); // Too long
        assert!(!validator.is_valid_hex("gggggg"));  // Invalid chars
    }
}

