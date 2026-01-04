//! Transaction Selection Algorithm (20/20/50/10)
//!
//! Implements the PoAI transaction selection algorithm for block builders.
//!
//! Per PoAI specification:
//! - 20% highest PointPrice transactions
//! - 20% lowest PointPrice transactions (to compensate/balance)
//! - 50% average PointPrice transactions
//! - 10% oldest transactions
//!
//! This creates efficient, fair blocks optimized for affordability.
use crate::blockchain::Transaction;
use anyhow::Result;

/// PoAI Point system constants
const POINT_TO_COIN_RATIO: f64 = 0.001; // 1 point = 0.001 coins
const FIRST_HALVING_THRESHOLD: u64 = 30_000_000_000; // 30 billion points
const SECOND_HALVING_THRESHOLD: u64 = 60_000_000_000; // 60 billion points

/// Configuration for transaction selection
#[derive(Debug, Clone)]
pub struct TransactionSelectorConfig {
    /// Maximum number of transactions per block
    pub max_transactions_per_block: usize,
    
    /// Target block size in bytes
    pub target_block_size: u64,
    
    /// Current total PointPrice spent in blockchain history
    pub total_points_spent: u64,
    
    /// Minimum transaction fee (in points)
    pub min_transaction_fee: u64,
}

impl Default for TransactionSelectorConfig {
    fn default() -> Self {
        Self {
            max_transactions_per_block: 1000,
            target_block_size: 1_000_000, // 1MB
            total_points_spent: 0,
            min_transaction_fee: 1,
        }
    }
}

/// Transaction with PoAI-specific metadata
#[derive(Debug, Clone)]
pub struct TransactionWithMetadata {
    pub transaction: Transaction,
    pub point_price: u64,      // Fee in points
    pub point_data: u64,       // Size/data volume in points
    pub timestamp: u64,        // When transaction was created
    pub priority_score: f64,   // Calculated priority for sorting
}

impl TransactionWithMetadata {
    /// Create from a base transaction
    pub fn from_transaction(tx: Transaction) -> Self {
        let point_data = tx.calculate_size(); // Size in bytes = points
        let point_price = calculate_point_price(&tx);
        
        Self {
            timestamp: tx.timestamp,
            transaction: tx,
            point_price,
            point_data,
            priority_score: 0.0,
        }
    }
    
    /// Calculate efficiency (PointData per PointPrice)
    pub fn efficiency(&self) -> f64 {
        if self.point_price == 0 {
            return 0.0;
        }
        self.point_data as f64 / self.point_price as f64
    }
}

/// Calculate PointPrice from transaction
/// In a real implementation, this would come from the transaction fee field
fn calculate_point_price(tx: &Transaction) -> u64 {
    // TODO: Once Transaction has a fee field, use that
    // For now, estimate based on transaction size and amount
    let base_fee = (tx.calculate_size() / 100).max(1); // Minimum 1 point per 100 bytes
    let amount_fee = tx.amount / 1000000; // 1 point per million in amount
    base_fee + amount_fee
}

/// Transaction selector implementing the PoAI 20/20/50/10 algorithm
pub struct TransactionSelector {
    config: TransactionSelectorConfig,
}

impl TransactionSelector {
    pub fn new(config: TransactionSelectorConfig) -> Self {
        Self { config }
    }
    
    /// Select transactions for a block using the PoAI 20/20/50/10 algorithm
    ///
    /// # Arguments
    /// * `mempool` - Available transactions from the mempool
    ///
    /// # Returns
    /// * Selected transactions organized by priority category
    pub fn select_transactions(
        &self,
        mempool: Vec<Transaction>,
    ) -> Result<SelectedTransactions> {
        if mempool.is_empty() {
            return Ok(SelectedTransactions::empty());
        }
        
        // Convert to metadata format
        let tx_with_meta: Vec<TransactionWithMetadata> = mempool
            .into_iter()
            .map(TransactionWithMetadata::from_transaction)
            .collect();
        
        // Calculate target counts for each category
        let total_count = self.config.max_transactions_per_block.min(tx_with_meta.len());
        let high_price_count = (total_count as f64 * 0.20).ceil() as usize;
        let low_price_count = (total_count as f64 * 0.20).ceil() as usize;
        let avg_price_count = (total_count as f64 * 0.50).ceil() as usize;
        let oldest_count = (total_count as f64 * 0.10).ceil() as usize;
        
        // Calculate average PointPrice
        let avg_point_price = if !tx_with_meta.is_empty() {
            tx_with_meta.iter().map(|t| t.point_price).sum::<u64>() / tx_with_meta.len() as u64
        } else {
            0
        };
        
        // Select unique transactions per category (no overlap) in this order:
        // high (20%), low (20%), avg (50%), oldest (10%)
        use std::collections::HashSet;
        let mut selected_ids: HashSet<String> = HashSet::new();

        // Helper: take up to `count` items from `candidates` that aren't already selected.
        let mut take_unique = |mut candidates: Vec<TransactionWithMetadata>, count: usize| {
            let mut out = Vec::new();
            for tx in candidates.drain(..) {
                if out.len() >= count {
                    break;
                }
                let id = tx.transaction.id.clone();
                if selected_ids.insert(id) {
                    out.push(tx);
                }
            }
            out
        };

        // 1) High price: sort by PointPrice desc
        let mut high_candidates = tx_with_meta.clone();
        high_candidates.sort_by(|a, b| b.point_price.cmp(&a.point_price));
        let high_price = take_unique(high_candidates, high_price_count);

        // 2) Low price: sort by PointPrice asc
        let mut low_candidates = tx_with_meta.clone();
        low_candidates.sort_by(|a, b| a.point_price.cmp(&b.point_price));
        let low_price = take_unique(low_candidates, low_price_count);

        // 3) Avg price: sort by absolute diff to avg_point_price (closest first)
        let mut avg_candidates = tx_with_meta.clone();
        avg_candidates.sort_by(|a, b| {
            let da = (a.point_price as i64 - avg_point_price as i64).abs();
            let db = (b.point_price as i64 - avg_point_price as i64).abs();
            da.cmp(&db)
                .then_with(|| a.point_price.cmp(&b.point_price))
        });
        let avg_price = take_unique(avg_candidates, avg_price_count);

        // 4) Oldest: sort by timestamp asc
        let mut oldest_candidates = tx_with_meta;
        oldest_candidates.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        let oldest = take_unique(oldest_candidates, oldest_count);

        let total_selected = high_price.len() + low_price.len() + avg_price.len() + oldest.len();
        
        Ok(SelectedTransactions {
            high_price,
            low_price,
            avg_price,
            oldest,
            avg_point_price,
            total_selected,
        })
    }
    
    /// Calculate block efficiency
    ///
    /// Efficiency is measured as:
    /// - How close the average PointPrice is to the target
    /// - How much PointData (useful information) is included
    pub fn calculate_block_efficiency(
        &self,
        selected: &SelectedTransactions,
    ) -> Result<BlockEfficiency> {
        let all_tx = selected.all_transactions();
        
        if all_tx.is_empty() {
            return Ok(BlockEfficiency::default());
        }
        
        // Calculate total PointData (useful information)
        let total_point_data: u64 = all_tx.iter().map(|t| t.point_data).sum();
        
        // Calculate total PointPrice (fees)
        let total_point_price: u64 = all_tx.iter().map(|t| t.point_price).sum();
        
        // Calculate average PointPrice
        let avg_point_price = total_point_price / all_tx.len() as u64;
        
        // Calculate fill percentage (how full the block is)
        let fill_percentage = (total_point_data as f64 / self.config.target_block_size as f64)
            .min(1.0);
        
        // Calculate price stability (how close average is to median)
        let price_stability = self.calculate_price_stability(&all_tx);
        
        // Calculate overall efficiency score (0-100)
        let efficiency_score = (fill_percentage * 40.0) + ((price_stability / 100.0) * 60.0);
        
        Ok(BlockEfficiency {
            total_point_data,
            total_point_price,
            avg_point_price,
            fill_percentage,
            price_stability,
            efficiency_score,
            transaction_count: all_tx.len(),
        })
    }
    
    /// Calculate price stability score
    fn calculate_price_stability(&self, transactions: &[&TransactionWithMetadata]) -> f64 {
        if transactions.is_empty() {
            return 0.0;
        }
        
        let mut prices: Vec<u64> = transactions.iter().map(|t| t.point_price).collect();
        prices.sort();
        
        let median = if prices.len() % 2 == 0 {
            let mid = prices.len() / 2;
            (prices[mid - 1] + prices[mid]) / 2
        } else {
            prices[prices.len() / 2]
        };
        
        let avg: u64 = prices.iter().sum::<u64>() / prices.len() as u64;
        
        // Stability is higher when median and average are close
        let diff = (avg as i64 - median as i64).abs();
        let max_expected_diff = avg.max(1);
        
        let stability = 1.0 - (diff as f64 / max_expected_diff as f64).min(1.0);
        stability * 100.0
    }
    
    /// Get current point-to-coin ratio based on total points spent
    pub fn get_point_to_coin_ratio(&self) -> f64 {
        if self.config.total_points_spent >= SECOND_HALVING_THRESHOLD {
            POINT_TO_COIN_RATIO / 4.0 // 0.00025 coins per point
        } else if self.config.total_points_spent >= FIRST_HALVING_THRESHOLD {
            POINT_TO_COIN_RATIO / 2.0 // 0.0005 coins per point
        } else {
            POINT_TO_COIN_RATIO // 0.001 coins per point
        }
    }
}

/// Selected transactions organized by priority category
#[derive(Debug, Clone)]
pub struct SelectedTransactions {
    pub high_price: Vec<TransactionWithMetadata>,
    pub low_price: Vec<TransactionWithMetadata>,
    pub avg_price: Vec<TransactionWithMetadata>,
    pub oldest: Vec<TransactionWithMetadata>,
    pub avg_point_price: u64,
    pub total_selected: usize,
}

impl SelectedTransactions {
    pub fn empty() -> Self {
        Self {
            high_price: vec![],
            low_price: vec![],
            avg_price: vec![],
            oldest: vec![],
            avg_point_price: 0,
            total_selected: 0,
        }
    }
    
    /// Get all selected transactions as a flat list
    pub fn all_transactions(&self) -> Vec<&TransactionWithMetadata> {
        let mut all = Vec::new();
        all.extend(self.high_price.iter());
        all.extend(self.low_price.iter());
        all.extend(self.avg_price.iter());
        all.extend(self.oldest.iter());
        all
    }
    
    /// Get all transactions as owned values
    pub fn into_transactions(self) -> Vec<Transaction> {
        let mut all = Vec::new();
        all.extend(self.high_price.into_iter().map(|t| t.transaction));
        all.extend(self.low_price.into_iter().map(|t| t.transaction));
        all.extend(self.avg_price.into_iter().map(|t| t.transaction));
        all.extend(self.oldest.into_iter().map(|t| t.transaction));
        all
    }
}

/// Block efficiency metrics
#[derive(Debug, Clone, Default)]
pub struct BlockEfficiency {
    pub total_point_data: u64,     // Total useful information (bytes)
    pub total_point_price: u64,    // Total fees collected
    pub avg_point_price: u64,      // Average fee per transaction
    pub fill_percentage: f64,      // How full the block is (0.0-1.0)
    pub price_stability: f64,      // Price stability score (0-100)
    pub efficiency_score: f64,     // Overall efficiency (0-100)
    pub transaction_count: usize,  // Number of transactions
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_transaction(id: &str, amount: u64, timestamp: u64) -> Transaction {
        Transaction {
            id: id.to_string(),
            sender: format!("sender_{}", id),
            receiver: format!("receiver_{}", id),
            amount,
            signature: format!("sig_{}", id),
            timestamp,
            data: None,
        }
    }
    
    #[test]
    fn test_transaction_selector_basic() {
        let config = TransactionSelectorConfig::default();
        let selector = TransactionSelector::new(config);
        
        // Create test transactions with varying amounts (which affects PointPrice)
        let mut mempool = vec![];
        for i in 0..100 {
            let amount = (i + 1) as u64 * 1000000; // Varying amounts
            let tx = create_test_transaction(&format!("tx_{}", i), amount, i as u64);
            mempool.push(tx);
        }
        
        let result = selector.select_transactions(mempool).unwrap();
        
        // Should have selected transactions
        assert!(result.total_selected > 0);
        
        // Should have all categories
        assert!(!result.high_price.is_empty());
        assert!(!result.low_price.is_empty());
        assert!(!result.avg_price.is_empty());
        assert!(!result.oldest.is_empty());
    }
    
    #[test]
    fn test_20_20_50_10_distribution() {
        let config = TransactionSelectorConfig {
            max_transactions_per_block: 100,
            ..Default::default()
        };
        let selector = TransactionSelector::new(config);
        
        // Create 100 transactions
        let mut mempool = vec![];
        for i in 0..100 {
            let amount = (i + 1) as u64 * 1000000;
            let tx = create_test_transaction(&format!("tx_{}", i), amount, i as u64);
            mempool.push(tx);
        }
        
        let result = selector.select_transactions(mempool).unwrap();
        
        // Check distribution is approximately 20/20/50/10
        let total = result.total_selected as f64;
        let high_pct = result.high_price.len() as f64 / total;
        let low_pct = result.low_price.len() as f64 / total;
        let avg_pct = result.avg_price.len() as f64 / total;
        let old_pct = result.oldest.len() as f64 / total;
        
        assert!((high_pct - 0.20).abs() < 0.05); // Within 5%
        assert!((low_pct - 0.20).abs() < 0.05);
        assert!((avg_pct - 0.50).abs() < 0.10); // Within 10% (more variance for avg)
        assert!((old_pct - 0.10).abs() < 0.05);
    }
    
    #[test]
    fn test_block_efficiency_calculation() {
        let config = TransactionSelectorConfig::default();
        let selector = TransactionSelector::new(config);
        
        let mut mempool = vec![];
        for i in 0..50 {
            let tx = create_test_transaction(&format!("tx_{}", i), (i + 1) * 1000000, i);
            mempool.push(tx);
        }
        
        let selected = selector.select_transactions(mempool).unwrap();
        let efficiency = selector.calculate_block_efficiency(&selected).unwrap();
        
        assert!(efficiency.efficiency_score >= 0.0);
        assert!(efficiency.efficiency_score <= 100.0);
        assert!(efficiency.transaction_count > 0);
        assert!(efficiency.total_point_data > 0);
        assert!(efficiency.total_point_price > 0);
    }
    
    #[test]
    fn test_oldest_transactions_selected() {
        let config = TransactionSelectorConfig {
            max_transactions_per_block: 100,
            ..Default::default()
        };
        let selector = TransactionSelector::new(config);
        
        let mut mempool = vec![];
        // Create transactions with decreasing timestamps (newer to older)
        for i in 0..100 {
            let timestamp = 1000000 - i; // Older timestamps are smaller
            let tx = create_test_transaction(&format!("tx_{}", i), 1000000, timestamp);
            mempool.push(tx);
        }
        
        let result = selector.select_transactions(mempool).unwrap();
        
        // Oldest category should have transactions with smallest timestamps
        assert!(!result.oldest.is_empty());
        let oldest_timestamp = result.oldest[0].timestamp;
        
        // Should be one of the oldest (smallest timestamps)
        assert!(oldest_timestamp < 1000000 - 90);
    }
    
    #[test]
    fn test_point_to_coin_ratio_halving() {
        let mut config = TransactionSelectorConfig::default();
        
        // Before first halving
        config.total_points_spent = 0;
        let selector = TransactionSelector::new(config.clone());
        assert_eq!(selector.get_point_to_coin_ratio(), 0.001);
        
        // After first halving
        config.total_points_spent = FIRST_HALVING_THRESHOLD;
        let selector = TransactionSelector::new(config.clone());
        assert_eq!(selector.get_point_to_coin_ratio(), 0.0005);
        
        // After second halving
        config.total_points_spent = SECOND_HALVING_THRESHOLD;
        let selector = TransactionSelector::new(config);
        assert_eq!(selector.get_point_to_coin_ratio(), 0.00025);
    }
    
    #[test]
    fn test_empty_mempool() {
        let config = TransactionSelectorConfig::default();
        let selector = TransactionSelector::new(config);
        
        let result = selector.select_transactions(vec![]).unwrap();
        assert_eq!(result.total_selected, 0);
        assert!(result.all_transactions().is_empty());
    }
    
    #[test]
    fn test_price_stability_calculation() {
        let config = TransactionSelectorConfig::default();
        let selector = TransactionSelector::new(config);
        
        // Create transactions with very similar prices (high stability)
        let mut mempool = vec![];
        for i in 0..20 {
            let amount = 1000000; // Same amount = similar PointPrice
            let tx = create_test_transaction(&format!("tx_{}", i), amount, i);
            mempool.push(tx);
        }
        
        let selected = selector.select_transactions(mempool).unwrap();
        let efficiency = selector.calculate_block_efficiency(&selected).unwrap();
        
        // High stability (prices are similar)
        assert!(efficiency.price_stability > 80.0);
    }
}

