
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use std::num::NonZeroUsize;
use serde::{Serialize, Deserialize};
use crate::blockchain::{Block, Transaction};

use crate::consensus::metrics::ConsensusMetrics;
use anyhow::Result;
use tokio::sync::RwLock;
use lru::LruCache;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub block_cache_size: usize,
    pub transaction_cache_size: usize,
    pub color_cache_size: usize,
    pub validation_window: u64, // Seconds
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub value: bool,
    pub timestamp: u64,
    pub score: u64,
}

#[derive(Debug)]
pub struct ValidationCache {
    block_cache: Arc<RwLock<LruCache<String, CacheEntry>>>,
    transaction_cache: Arc<RwLock<LruCache<String, CacheEntry>>>,
    color_cache: Arc<RwLock<LruCache<String, CacheEntry>>>,
    config: CacheConfig,
    metrics: Arc<ConsensusMetrics>,
}

impl ValidationCache {
    pub fn new(metrics: Arc<ConsensusMetrics>) -> Self {
        Self {
            block_cache: Arc::new(RwLock::new(LruCache::new(
                NonZeroUsize::new(1000).unwrap()
            ))),
            transaction_cache: Arc::new(RwLock::new(LruCache::new(
                NonZeroUsize::new(10000).unwrap()
            ))),
            color_cache: Arc::new(RwLock::new(LruCache::new(
                NonZeroUsize::new(1000).unwrap()
            ))),
            config: CacheConfig {
                block_cache_size: 1000,
                transaction_cache_size: 10000,
                color_cache_size: 1000,
                validation_window: 3600, // 1 hour
            },
            metrics,
        }
    }

    pub async fn cache_block_validation(&self, block: &Block, is_valid: bool, score: u64) -> Result<()> {
        let mut cache = self.block_cache.write().await;
        let key = block.hash.clone();
        
        let entry = CacheEntry {
            value: is_valid,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            score,
        };
        
        cache.put(key, entry);
        self.metrics.observe_block_validation_time(0.0); // Cache hit
        Ok(())
    }

    pub async fn get_cached_block_validation(&self, block: &Block) -> Option<CacheEntry> {
        let mut cache = self.block_cache.write().await;
        cache.get(&block.hash).cloned()
    }

    pub async fn cache_transaction_validation(&self, tx: &Transaction, is_valid: bool, score: u64) -> Result<()> {
        let mut cache = self.transaction_cache.write().await;
        let key = tx.hash();
        
        let entry = CacheEntry {
            value: is_valid,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            score,
        };
        
        cache.put(key, entry);
        self.metrics.observe_transaction_validation_time(0.0); // Cache hit
        Ok(())
    }

    pub async fn get_cached_transaction_validation(&self, tx: &Transaction) -> Option<CacheEntry> {
        let mut cache = self.transaction_cache.write().await;
        cache.get(&tx.hash()).cloned()
    }

    pub async fn cache_color_validation(&self, color: &str, is_valid: bool, score: u64) -> Result<()> {
        let mut cache = self.color_cache.write().await;
        let key = color.to_string();
        
        let entry = CacheEntry {
            value: is_valid,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            score,
        };
        
        cache.put(key, entry);
        Ok(())
    }

    pub async fn get_cached_color_validation(&self, color: &str) -> Option<CacheEntry> {
        let mut cache = self.color_cache.write().await;
        cache.get(color).cloned()
    }

    pub async fn cleanup_cache(&self) -> Result<()> {
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        
        // Clean block cache
        let mut block_cache = self.block_cache.write().await;
        let mut keys_to_remove = Vec::new();
        for (key, entry) in block_cache.iter() {
            if entry.timestamp + self.config.validation_window < current_time {
                keys_to_remove.push(key.clone());
            }
        }
        for key in keys_to_remove {
            block_cache.pop(&key);
        }
        
        // Clean transaction cache
        let mut tx_cache = self.transaction_cache.write().await;
        let mut keys_to_remove = Vec::new();
        for (key, entry) in tx_cache.iter() {
            if entry.timestamp + self.config.validation_window < current_time {
                keys_to_remove.push(key.clone());
            }
        }
        for key in keys_to_remove {
            tx_cache.pop(&key);
        }
        
        // Clean color cache
        let mut color_cache = self.color_cache.write().await;
        let mut keys_to_remove = Vec::new();
        for (key, entry) in color_cache.iter() {
            if entry.timestamp + self.config.validation_window < current_time {
                keys_to_remove.push(key.clone());
            }
        }
        for key in keys_to_remove {
            color_cache.pop(&key);
        }
        
        Ok(())
    }

    pub async fn is_cache_valid(&self, entry: &CacheEntry) -> Result<bool, anyhow::Error> {
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        Ok(entry.timestamp + self.config.validation_window >= current_time)
    }
}
