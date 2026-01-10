# SELF Chain

### Production-Proven Proof-of-AI Consensus

> Launch your own Layer-1 blockchain with Proof-of-AI consensus, quantum-resistant security, and enterprise-grade infrastructure.

SELF Chain is an AI-native blockchain implementing the patent-pending Proof-of-AI consensus mechanism. This repository contains the core technology powering SELF Chain constellations.

**Status:** Production-proven — SELF App (first constellation) has been live since January 1, 2026 with real users validating blocks from their browsers.

---

## The Constellation Model

A **Constellation** is a licensed deployment of SELF Chain technology, customized for your use case:

- **Your network** — Independent chain with your branding and governance
- **Your economics** — Custom token model and reward mechanisms  
- **Your users** — Browser-based validators mean zero infrastructure per user
- **Our consensus** — Battle-tested PoAI core, production-proven

**SELF App** is the first constellation, live in production at `https://self.app` since January 1, 2026, demonstrating PoAI consensus at scale with browser-based validators.

---

## Technology Overview

### Proof-of-AI Consensus

PoAI replaces computational waste (PoW) and wealth concentration (PoS) with efficient, democratic validation:

| Node Type | Role | Footprint |
|-----------|------|-----------|
| **Validator** | Vote on blocks, validate color markers | Browser-based, ~256MB RAM |
| **Block Builder** | Assemble blocks, compete on efficiency | Server, ~2GB RAM |
| **Coordinator** | Organize rounds, generate reference blocks | Managed service (WebSocket) |

**Key differentiators:**
- **Browser-based validators** — Keys never leave user's device (zero-knowledge architecture)
- **Zero per-user infrastructure** — Scales to millions with fixed coordination layer
- **60-second consensus rounds** — Fast finality, browser-friendly timing
- **Color marker validation** — Lightweight cryptographic validation (~10MB storage per validator)
- **Production-proven** — Live since January 2026 with real users

### Transaction Selection (20/20/50/10)

Fair block building that prevents fee manipulation:

```
┌─────────────────────────────────────────────┐
│              BLOCK COMPOSITION              │
├─────────────────────────────────────────────┤
│  20%  │ Highest fee transactions            │
│  20%  │ Lowest fee transactions (fairness)  │
│  50%  │ Average fee transactions            │
│  10%  │ Oldest transactions (no starvation) │
└─────────────────────────────────────────────┘
```

### Color Marker Validation

Lightweight cryptographic validation without full blockchain storage:

- Each wallet has a HEX color derived from transaction history
- Validators verify color transitions: `new_color = (current + tx_hex) mod 0x1000000`
- Enables browser-based validation with cryptographic security
- ~10MB storage per validator vs gigabytes for full nodes

### Delegated Key Architecture

User sovereignty with operational flexibility:

```
Master Key (User Device)          Validator Key (Derived)
├─ Controls funds                 ├─ Votes on blocks
├─ Signs transactions             ├─ Validates colors
├─ Never transmitted              ├─ Cannot move funds
└─ Can revoke validators          └─ Destroyed on revocation
```

### Hybrid Cryptography

Production-ready with post-quantum preparation:

- **Classic**: ECDSA (secp256k1), X25519, SHA3-256
- **Post-Quantum**: Kyber-1024, SPHINCS+
- **Hybrid Mode**: Combined schemes for transition period

---

## Configurable Reward Mechanisms

The PoAI specification defines a default distribution (90% builder / 8% voters / 1% validator / 1% network), but constellations implement their own economics:

```rust
/// Constellations implement this trait to define their reward model
pub trait RewardDistributor: Send + Sync {
    async fn distribute_rewards(&self, round: &CompletedRound) -> Result<RewardDistribution>;
}
```

**Example mechanisms:**
- Per-round blockchain rewards (default PoAI)
- Prize pool drawings (time-weighted entries)
- Staking-based distributions
- Tiered participation rewards
- Hybrid models

See `examples/custom_rewards.rs` for implementation patterns.

For full configuration options, performance data, and integration details, see **[Constellation Overview](docs/CONSTELLATION_OVERVIEW.md)**.

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        CONSTELLATION                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   ┌──────────────────┐              ┌──────────────────┐        │
│   │   User Browsers  │◄────────────►│   Coordinator    │        │
│   │   (Validators)   │  WebSocket   │    (Managed)     │        │
│   └──────────────────┘              └────────┬─────────┘        │
│            │                                 │                   │
│            │ Vote                            │ Proposals         │
│            ▼                                 ▼                   │
│   ┌──────────────────────────────────────────────────────┐      │
│   │                    PoAI Core                          │      │
│   │  ┌────────────┐  ┌────────────┐  ┌────────────────┐  │      │
│   │  │ Transaction│  │   Color    │  │    Voting      │  │      │
│   │  │  Selector  │  │  Markers   │  │    System      │  │      │
│   │  └────────────┘  └────────────┘  └────────────────┘  │      │
│   └──────────────────────────────────────────────────────┘      │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Repository Structure

```
src/
├── consensus/              # PoAI consensus implementation
│   ├── validator.rs        # Color marker validation
│   ├── transaction_selector.rs  # 20/20/50/10 algorithm
│   ├── voting.rs           # Decentralized voting
│   ├── metrics.rs          # Prometheus instrumentation
│   └── v1/                 # Spec-compliant consensus types
│       └── types.rs        # ConsensusConfig, RoundState, ValidatorInfo
├── crypto/                 # Cryptographic primitives
│   ├── delegated_keys.rs   # Master/validator key hierarchy
│   ├── classic/            # ECDSA, X25519, hashing
│   ├── quantum/            # Kyber, SPHINCS+
│   └── hybrid/             # Combined schemes
├── blockchain/             # Core types (Block, Transaction)
│   ├── mod.rs              # Production types (WebSocket coordinator)
│   └── v1/                 # Spec-compliant wire formats
│       ├── block.rs        # BlockHeader with [u8;32] hashes
│       ├── transaction.rs  # Transaction with point_price, nonce
│       ├── vote.rs         # Vote, RankedVote types
│       └── proposal.rs     # BlockProposal for competition model
├── node/                   # Node type implementations
│   └── node_types.rs       # Validator, Builder, Coordinator
└── examples/               # Reference implementations
    └── custom_rewards.rs   # Reward mechanism patterns
```

### Production vs v1 Types

| Aspect | Production (`blockchain/mod.rs`) | v1 Spec (`blockchain/v1/`) |
|--------|----------------------------------|---------------------------|
| `previous_hash` | `String` | `[u8; 32]` |
| `point_price` | Not present | `u64` |
| `chain_id` | Not present | `String` |
| `nonce` | Not present | `u64` |
| Hash algorithm | `DefaultHasher` | SHA-256 + domain separation |
| Signature | `String` | `[u8; 64]` Ed25519 |

The production types work with the current WebSocket-based coordinator. The v1 types match `docs/POAI_SPECIFICATION.md` and are used in the decentralized P2P consensus implementation.

---

## Documentation

| Document | Description |
|----------|-------------|
| **[Getting Started](docs/GETTING_STARTED.md)** | Developer guide: setup, examples, integration checklist |
| **[Constellation Overview](docs/CONSTELLATION_OVERVIEW.md)** | Configuration options, performance data, integration model, what we provide |
| **[PoAI Specification](docs/POAI_SPECIFICATION.md)** | Complete consensus mechanism specification |
| **[Browser Validator Architecture](docs/BROWSER_VALIDATOR_ARCHITECTURE.md)** | Zero-knowledge browser-based validator design |
| **[Network Architecture](docs/NETWORK_ARCHITECTURE.md)** | Peer validation system and network layer |

---

## Technical Evaluation

This repository contains working implementations of:

| Component | Status | Notes |
|-----------|--------|-------|
| Transaction Selector | ✅ Complete | Full 20/20/50/10 with tests |
| Color Marker Validation | ✅ Complete | HEX calculation per spec |
| Delegated Keys | ✅ Complete | HMAC-SHA3 derivation |
| Voting System | ✅ Complete | Round management, tallying |
| Hybrid Crypto | ✅ Complete | Classic + post-quantum |
| Node Types | ✅ Complete | Validator, Builder, Coordinator |
| Metrics | ✅ Complete | Prometheus integration |

**Not included** (proprietary to SELF Technology):
- AI validation rules and thresholds
- Pattern analysis algorithms
- Production coordinator implementation
- Infrastructure automation

---

## Production Validation

SELF App (first constellation - https://self.app) has been live since January 1, 2026:

| Metric | Value |
|--------|-------|
| **Uptime** | 100% since launch |
| **Consensus** | 1 round per minute, continuous |
| **Validators** | Browser-based, real users |
| **Key Architecture** | Zero-knowledge (keys never leave device) |
| **Reward Mechanism** | Prize pool (daily/weekly/monthly drawings) |

**Production Endpoints:**
- Coordinator: ``
- Orchestrator: ``

---

## Security

**This is NOT an open-source project.** This repository contains core PoAI consensus technology for evaluation by prospective constellation clients.

**Security Properties:**
- **Quantum-Resistant**: Hybrid cryptography with Kyber-1024 and SPHINCS+
- **Key Isolation**: Master keys never leave user devices
- **Zero-Knowledge Architecture**: Validator keys held in browser memory only
- **Production-Proven**: Live in production since January 2026

**Not Included** (proprietary to SELF Technology):
- AI validation rules and thresholds
- Pattern analysis algorithms  
- Production coordinator implementation details
- Infrastructure automation and deployment configs

**Threat Model**: Available under NDA for serious evaluators considering constellation deployment.

---

## Getting Started

### For Developers (Evaluation Only)

**⚠️ This repository is for EVALUATION ONLY. You cannot deploy a constellation without proper licensing from SELF Technology.**

This repository contains working implementations of core PoAI mechanisms for technical evaluation:

```bash
# Clone the repository
git clone https://github.com/self-technology/self-chain-core.git
cd self-chain-core

# Build the project
cargo build

# Run tests
cargo test

# Run examples
cargo run --example custom_rewards
```

**Key Components:**
- `src/consensus/` — PoAI consensus (transaction selection, color markers, voting)
- `src/crypto/` — Hybrid cryptography (classic + post-quantum)
- `src/blockchain/` — Core blockchain types (Block, Transaction)
- `src/node/` — Node type implementations (Validator, Builder, Coordinator)
- `examples/` — Reference implementations for custom reward mechanisms

See the [Getting Started Guide](docs/GETTING_STARTED.md) for evaluation instructions and the [Constellation Overview](docs/CONSTELLATION_OVERVIEW.md) for integration details.

**For constellation deployment:** Contact [info@theselfchain.com](mailto:info@theselfchain.com) for licensing and deployment support.

---

## FAQ

**What's the licensing model?**

- Development cost based on your requirements and customization scope
- Annual license fee (determined by constellation specification)
- 10% of native token generation + 10% of any generated tokens

**How long to launch a constellation?**

Timeline depends entirely on your specification — from weeks for standard configurations to months for heavily customized implementations.

**Can we see a demo?**

Yes. Contact us for a technical demonstration and deep-dive with our engineering team. SELF App is live at https://self.app — you can see PoAI consensus in action if you create an account.

**Has this been audited?**

Cryptographic audit is in progress. Preliminary results and methodology available under NDA. Production deployment demonstrates real-world security and reliability.

---

## Next Steps

Interested in launching a constellation?

**Contact:** [info@self.chain](mailto:info@self.chain)

We provide:
- Full technology licensing
- Customization for your reward model and tokenomics
- Infrastructure guidance and deployment support
- Ongoing technical partnership

---

**Patent-pending technology.** See [proofofai.com](https://proofofai.com) for the public PoAI specification.

*© 2026 SELF Technology Ltd. All rights reserved.*
