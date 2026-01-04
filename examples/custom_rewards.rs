//! Custom Rewards Example
//!
//! This example demonstrates how to implement a custom reward distribution
//! mechanism for your SELF Chain constellation.
//!
//! The PoAI specification defines a default distribution (90/8/1/1), but
//! constellations can implement alternative mechanisms such as:
//! - Prize pools (random selection from participants)
//! - Staking rewards (based on stake amount)
//! - Tiered rewards (based on participation level)
//! - Hybrid models

use async_trait::async_trait;
use std::collections::HashMap;

/// Completed voting round with results
pub struct CompletedRound {
    pub round_id: u64,
    pub winning_builder_id: String,
    pub winning_block_hash: String,
    pub voters: Vec<VoterInfo>,
    pub color_validator_id: String,
    pub block_reward: u64,
}

/// Information about a voter in the round
pub struct VoterInfo {
    pub validator_id: String,
    pub voted_for_winner: bool,
    pub vote_timestamp: u64,
}

/// Trait for implementing custom reward distribution
#[async_trait]
pub trait RewardDistributor: Send + Sync {
    /// Calculate and distribute rewards for a completed round
    async fn distribute_rewards(&self, round: &CompletedRound) -> anyhow::Result<RewardDistribution>;
    
    /// Get the name of this reward mechanism
    fn name(&self) -> &str;
}

/// Result of reward distribution
pub struct RewardDistribution {
    pub round_id: u64,
    pub distributions: HashMap<String, u64>,  // validator_id -> reward amount
    pub total_distributed: u64,
}

// =============================================================================
// Example 1: Default PoAI Rewards (90/8/1/1)
// =============================================================================

pub struct DefaultPoAIRewards;

#[async_trait]
impl RewardDistributor for DefaultPoAIRewards {
    async fn distribute_rewards(&self, round: &CompletedRound) -> anyhow::Result<RewardDistribution> {
        let mut distributions = HashMap::new();
        let reward = round.block_reward;
        
        // 90% to block builder
        let builder_reward = (reward as f64 * 0.90) as u64;
        distributions.insert(round.winning_builder_id.clone(), builder_reward);
        
        // 8% split among voters who voted for the winner
        let voter_pool = (reward as f64 * 0.08) as u64;
        let winning_voters: Vec<_> = round.voters.iter()
            .filter(|v| v.voted_for_winner)
            .collect();
        
        if !winning_voters.is_empty() {
            let per_voter = voter_pool / winning_voters.len() as u64;
            for voter in winning_voters {
                *distributions.entry(voter.validator_id.clone()).or_insert(0) += per_voter;
            }
        }
        
        // 1% to color validator
        let color_reward = (reward as f64 * 0.01) as u64;
        *distributions.entry(round.color_validator_id.clone()).or_insert(0) += color_reward;
        
        // 1% to network (could go to treasury, burned, etc.)
        let network_reward = (reward as f64 * 0.01) as u64;
        distributions.insert("network_treasury".to_string(), network_reward);
        
        Ok(RewardDistribution {
            round_id: round.round_id,
            distributions,
            total_distributed: reward,
        })
    }
    
    fn name(&self) -> &str {
        "Default PoAI (90/8/1/1)"
    }
}

// =============================================================================
// Example 2: Prize Pool Rewards
// =============================================================================

use rand::seq::SliceRandom;

pub struct PrizePoolRewards {
    pub daily_pool: u64,
    pub weekly_pool: u64,
    pub monthly_pool: u64,
}

impl PrizePoolRewards {
    /// Select a random winner from eligible participants
    fn select_winner<'a>(&self, participants: &'a [VoterInfo]) -> Option<&'a VoterInfo> {
        let mut rng = rand::thread_rng();
        participants.choose(&mut rng)
    }
}

#[async_trait]
impl RewardDistributor for PrizePoolRewards {
    async fn distribute_rewards(&self, round: &CompletedRound) -> anyhow::Result<RewardDistribution> {
        let mut distributions = HashMap::new();
        
        // Each voter gets one "entry" per vote
        // Winner is selected randomly from all voters
        if let Some(winner) = self.select_winner(&round.voters) {
            // For this example, we give the daily pool reward
            distributions.insert(winner.validator_id.clone(), self.daily_pool);
        }
        
        let total = distributions.values().sum();
        
        Ok(RewardDistribution {
            round_id: round.round_id,
            distributions,
            total_distributed: total,
        })
    }
    
    fn name(&self) -> &str {
        "Prize Pool"
    }
}

// =============================================================================
// Example 3: Tiered Staking Rewards
// =============================================================================

pub struct StakingRewards {
    pub min_stake: u64,
    pub base_rate: f64,  // Annual percentage rate
}

impl StakingRewards {
    /// In a real implementation, this would look up actual staked amounts
    fn get_stake(&self, _validator_id: &str) -> u64 {
        // Placeholder - would query staking contract
        10000
    }
}

#[async_trait]
impl RewardDistributor for StakingRewards {
    async fn distribute_rewards(&self, round: &CompletedRound) -> anyhow::Result<RewardDistribution> {
        let mut distributions = HashMap::new();
        let reward = round.block_reward;
        
        // Calculate total eligible stake
        let stakes: Vec<(String, u64)> = round.voters.iter()
            .map(|v| (v.validator_id.clone(), self.get_stake(&v.validator_id)))
            .filter(|(_, stake)| *stake >= self.min_stake)
            .collect();
        
        let total_stake: u64 = stakes.iter().map(|(_, s)| s).sum();
        
        if total_stake > 0 {
            for (validator_id, stake) in stakes {
                let share = (stake as f64 / total_stake as f64) * reward as f64;
                distributions.insert(validator_id, share as u64);
            }
        }
        
        let total = distributions.values().sum();
        
        Ok(RewardDistribution {
            round_id: round.round_id,
            distributions,
            total_distributed: total,
        })
    }
    
    fn name(&self) -> &str {
        "Staking Rewards"
    }
}

// =============================================================================
// Main: Demonstrate different reward mechanisms
// =============================================================================

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create a sample completed round
    let round = CompletedRound {
        round_id: 1,
        winning_builder_id: "builder-001".to_string(),
        winning_block_hash: "abc123...".to_string(),
        voters: vec![
            VoterInfo {
                validator_id: "validator-001".to_string(),
                voted_for_winner: true,
                vote_timestamp: 1704067200,
            },
            VoterInfo {
                validator_id: "validator-002".to_string(),
                voted_for_winner: true,
                vote_timestamp: 1704067201,
            },
            VoterInfo {
                validator_id: "validator-003".to_string(),
                voted_for_winner: false,
                vote_timestamp: 1704067202,
            },
        ],
        color_validator_id: "validator-003".to_string(),
        block_reward: 1000,
    };

    // Demonstrate different reward mechanisms
    let mechanisms: Vec<Box<dyn RewardDistributor>> = vec![
        Box::new(DefaultPoAIRewards),
        Box::new(PrizePoolRewards {
            daily_pool: 5000,
            weekly_pool: 50000,
            monthly_pool: 200000,
        }),
        Box::new(StakingRewards {
            min_stake: 1000,
            base_rate: 0.05,
        }),
    ];

    for mechanism in mechanisms {
        println!("\n=== {} ===", mechanism.name());
        let distribution = mechanism.distribute_rewards(&round).await?;
        
        println!("Round {}: Distributed {} tokens", distribution.round_id, distribution.total_distributed);
        for (validator, amount) in &distribution.distributions {
            println!("  {} -> {} tokens", validator, amount);
        }
    }

    Ok(())
}

