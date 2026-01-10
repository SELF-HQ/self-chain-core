# Getting Started with SELF Chain Core

> **⚠️ IMPORTANT: This repository is for EVALUATION ONLY**
> 
> This codebase is provided for **technical evaluation** by prospective constellation clients. You **cannot** deploy your own constellation without proper licensing from SELF Technology.
> 
> **To deploy a constellation:** Contact [info@theselfchain.com](mailto:info@theselfchain.com) for licensing and deployment support.

This guide will help you understand the codebase, run examples, and evaluate PoAI consensus technology for potential constellation deployment.

---

## Prerequisites

- **Rust 1.70+** (install from [rustup.rs](https://rustup.rs/))
- **Cargo** (comes with Rust)
- **Basic understanding of blockchain concepts** (blocks, transactions, consensus)

---

## Quick Start

### 1. Clone and Build

```bash
git clone https://github.com/self-technology/self-chain-core.git
cd self-chain-core

# Build the project
cargo build

# Run tests
cargo test

# Run examples
cargo run --example custom_rewards
```

### 2. Explore the Codebase

**Key Modules:**

```
src/
├── consensus/              # PoAI consensus implementation
│   ├── validator.rs        # Color marker validation
│   ├── transaction_selector.rs  # 20/20/50/10 algorithm
│   ├── voting.rs           # Voting system
│   └── metrics.rs          # Prometheus metrics
├── crypto/                 # Cryptographic primitives
│   ├── delegated_keys.rs   # Master/validator key hierarchy
│   ├── classic/            # ECDSA, X25519, hashing
│   ├── quantum/            # Kyber, SPHINCS+
│   └── hybrid/             # Combined schemes
├── blockchain/             # Core types (Block, Transaction)
└── node/                   # Node type implementations
    └── node_types.rs       # Validator, Builder, Coordinator
```

---

## Core Concepts

### 1. Transaction Selection (20/20/50/10)

The PoAI algorithm selects transactions fairly:

```rust
use self_chain_core::consensus::{TransactionSelector, TransactionSelectorConfig};

let config = TransactionSelectorConfig {
    max_transactions_per_block: 1000,
    target_block_size: 1_000_000,
    total_points_spent: 0,
    min_transaction_fee: 1,
};

let selector = TransactionSelector::new(config);
let mempool: Vec<Transaction> = vec![/* ... */];

// Select transactions using 20/20/50/10 algorithm
let selected = selector.select_transactions(mempool)?;

// Calculate block efficiency
let efficiency = selector.calculate_block_efficiency(&selected)?;
println!("Efficiency score: {}", efficiency.efficiency_score);
```

**Distribution:**
- 20% highest PointPrice transactions
- 20% lowest PointPrice transactions (fairness)
- 50% average PointPrice transactions
- 10% oldest transactions (prevents starvation)

### 2. Color Marker Validation

Validators verify transactions using HEX wallet colors:

```rust
use self_chain_core::consensus::validator::Validator;

let validator = Validator::new(metrics, cache);

// Validate a transaction
validator.validate_transaction(&tx).await?;

// Calculate HEX transaction
let hex_tx = validator.calculate_hex_transaction(&tx)?;

// Calculate new wallet color
let new_color = validator.calculate_new_color(&current_color, &hex_tx)?;
```

**How it works:**
1. Each wallet has a HEX color (6-character hex string)
2. Transaction hash is converted to HEX transaction (6 characters)
3. New color = (current_color + hex_tx) mod 0x1000000
4. Validators verify color transitions match expected values

### 3. Delegated Keys

Separate master keys (funds) from validator keys (voting):

```rust
use self_chain_core::crypto::{MasterKey, ValidatorKey, KeyManager};

let key_manager = KeyManager::new();

// Import master key from recovery phrase
let master_key = key_manager.import_master_key(seed)?;

// Derive validator key (cannot move funds)
let validator_key = master_key.derive_validator_key(&nonce)?;

// Sign a vote with validator key
let signature = validator_key.sign_vote(block_hash, approve)?;
```

**Security:**
- Master key controls funds (never leaves device)
- Validator key only votes (cannot move funds)
- Keys can be revoked by changing recovery phrase

### 4. Node Types

Three node types in PoAI:

```rust
use self_chain_core::node::{NodeType, NodeConfig, ValidatorNode, BlockBuilderNode};

// Validator Node (Lite)
let validator_config = NodeConfig {
    node_id: "validator-001".to_string(),
    node_type: NodeType::Validator,
    listen_addr: "127.0.0.1:9000".to_string(),
    bootstrap_peers: vec![],
};
let mut validator = ValidatorNode::new(validator_config)?;
validator.initialize_with_master_key(master_key)?;

// Block Builder Node (Full)
let builder_config = NodeConfig {
    node_id: "builder-001".to_string(),
    node_type: NodeType::BlockBuilder,
    listen_addr: "127.0.0.1:9001".to_string(),
    bootstrap_peers: vec![],
};
let mut builder = BlockBuilderNode::new(builder_config);
```

---

## Custom Reward Mechanisms

Constellations can implement custom reward distributions:

```rust
use async_trait::async_trait;
use self_chain_core::examples::custom_rewards::{RewardDistributor, CompletedRound, RewardDistribution};

pub struct MyCustomRewards {
    // Your configuration
}

#[async_trait]
impl RewardDistributor for MyCustomRewards {
    async fn distribute_rewards(&self, round: &CompletedRound) -> anyhow::Result<RewardDistribution> {
        // Your custom logic here
        // - Prize pools
        // - Staking rewards
        // - Tiered distributions
        // - Hybrid models
        todo!()
    }
    
    fn name(&self) -> &str {
        "My Custom Rewards"
    }
}
```

See `examples/custom_rewards.rs` for complete examples including:
- Default PoAI rewards (90/8/1/1)
- Prize pool system
- Staking-based rewards

---

## Evaluation Checklist

**For evaluating SELF Chain technology:**

- [ ] **Review Architecture**: Understand PoAI consensus mechanisms
- [ ] **Run Examples**: Test transaction selection, color markers, voting
- [ ] **Review Code**: Examine implementation quality and security
- [ ] **Test Integration**: Understand how components work together
- [ ] **Assess Fit**: Determine if PoAI fits your use case

**⚠️ For actual constellation deployment, you must:**
- Contact SELF Technology for licensing
- Work with our team on customization
- Deploy through our supported infrastructure
- Complete security review and testing

**You cannot deploy a constellation without proper licensing.**

---

## Testing

Run the test suite:

```bash
# Run all tests
cargo test

# Run specific test module
cargo test consensus::validator::tests

# Run with output
cargo test -- --nocapture
```

**Test Coverage:**
- Transaction selection (20/20/50/10 algorithm)
- Color marker validation (HEX calculation)
- Delegated key derivation
- Node type implementations
- Block efficiency calculations

---

## Production Considerations

**⚠️ These are considerations for SELF Technology's production deployment, not for you to deploy independently.**

SELF Technology manages production infrastructure for licensed constellations:

1. **Security Audit**: Cryptographic implementations reviewed by third-party auditors
2. **Key Storage**: Keys never leave user devices (browser-based architecture)
3. **Network Security**: TLS-secured WebSocket connections
4. **Monitoring**: Prometheus metrics and alerting (see `consensus/metrics.rs`)
5. **Error Handling**: Graceful degradation and recovery
6. **Load Testing**: Tested at scale with production validators

**For constellation clients:** SELF Technology provides managed infrastructure. You focus on your application layer (UI, user accounts, token distribution) while we handle consensus coordination.

---

## Next Steps

**For Evaluation:**
1. **Review Architecture**: Read [Constellation Overview](CONSTELLATION_OVERVIEW.md)
2. **Understand PoAI**: Read [PoAI Specification](POAI_SPECIFICATION.md)
3. **Study Examples**: Review `examples/custom_rewards.rs` to understand reward mechanisms
4. **Run Tests**: Execute test suite to verify implementation quality

**For Constellation Deployment:**
- **Contact SELF Technology** for licensing and deployment support
- Schedule technical deep-dive with our engineering team
- Discuss your use case, tokenomics, and customization requirements
- Work with us on integration and deployment

**⚠️ You cannot deploy a constellation without proper licensing from SELF Technology.**

**Contact:** [info@theselfchain.com](mailto:info@theselfchain.com)

---

## Resources

- **PoAI Specification**: [proofofai.com](https://proofofai.com)
- **Production Demo**: [self.app](https://self.app) (live since Jan 2026)
- **Documentation**: See `docs/` directory
- **Examples**: See `examples/` directory

---

## Legal Notice

**This software is proprietary and confidential.** 

- Provided for **evaluation purposes only**
- **Cannot be used** for production deployment without licensing
- **Cannot be redistributed** or modified without permission
- **Patent pending** technology — unauthorized use prohibited

**For licensing inquiries:** [info@theselfchain.com](mailto:info@theselfchain.com)

*© 2026 SELF Technology. All rights reserved. Patent pending.*
