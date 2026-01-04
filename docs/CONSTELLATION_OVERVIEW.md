# Constellation Overview

> **For prospective constellation partners evaluating SELF Chain technology**

This document outlines what's configurable, what's managed, and what you get when launching a SELF Chain constellation.

---

## What is a Constellation?

A constellation is your own blockchain network powered by PoAI consensus:

```
┌─────────────────────────────────────────────────────────────────┐
│                    YOUR CONSTELLATION                            │
│                                                                  │
│   Your Brand  ·  Your Token  ·  Your Economics  ·  Your Users   │
│                                                                  │
│   ┌─────────────────────────────────────────────────────────┐   │
│   │              PoAI Consensus Core (Licensed)              │   │
│   └─────────────────────────────────────────────────────────┘   │
│                                                                  │
│   Managed by SELF Technology  ·  Battle-tested  ·  Scalable     │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

You control the user experience. We provide the consensus infrastructure.

---

## Configuration Options

### Consensus Parameters

| Parameter | Description | Range | Default |
|-----------|-------------|-------|---------|
| **Round Duration** | Time between block proposals | 30s – 5min | 60s |
| **Min Participation** | Validator participation threshold | 30% – 80% | 50% |
| **AI Strictness** | Anomaly detection sensitivity | 1 – 10 | 5 |
| **Block Size** | Maximum transactions per block | 100 – 10,000 | 1,000 |

### Validator Eligibility

Configure who can validate:

```rust
pub struct ValidatorEligibility {
    /// Minimum token stake (0 = no minimum)
    pub min_stake: u64,
    
    /// Minimum account age in days
    pub min_account_age: u32,
    
    /// Minimum activity score (participation history)
    pub min_activity_score: u32,
    
    /// Tier restrictions (e.g., only paid tiers)
    pub allowed_tiers: Vec<String>,
    
    /// Geographic restrictions (optional)
    pub allowed_regions: Option<Vec<String>>,
}
```

**Examples:**
- Open to all users (maximize decentralization)
- Stake-weighted (align incentives)
- Tier-gated (premium feature)
- Geographic (regulatory compliance)

### Reward Mechanisms

Full flexibility over how value flows to participants:

| Mechanism | Description | Use Case |
|-----------|-------------|----------|
| **Per-Round Rewards** | Blockchain mints tokens each round | Inflationary tokenomics |
| **Prize Pools** | Periodic drawings from fixed pool | Gamification, engagement |
| **Staking Rewards** | Proportional to stake | DeFi-style economics |
| **Fee Distribution** | Transaction fees to validators | Deflationary model |
| **Hybrid** | Combination of above | Complex economics |

See `examples/custom_rewards.rs` for implementation patterns.

### Token Economics

| Parameter | Description | Configurable |
|-----------|-------------|--------------|
| **Initial Supply** | Tokens at genesis | ✅ |
| **Max Supply** | Hard cap (optional) | ✅ |
| **Inflation Rate** | New tokens per period | ✅ |
| **Halving Schedule** | Emission reduction over time | ✅ |
| **Burn Mechanism** | Deflationary pressure | ✅ |
| **Vesting Schedules** | Team/investor lockups | ✅ |

### AI Compute Infrastructure

The AI validation component is fully configurable:

| Option | Description | Trade-offs |
|--------|-------------|------------|
| **Self-Hosted** | Your own GPU infrastructure | Full control, higher ops burden |
| **Cloud GPU** | AWS/GCP/Azure GPU instances | Flexible scaling, pay-per-use |
| **Managed** | SELF Technology provides | Turnkey, included in license |

Constellations choose based on:
- Latency requirements
- Data residency / compliance
- Cost structure preferences
- Operational capability

### Governance

| Model | Description | Complexity |
|-------|-------------|------------|
| **Centralized** | You control all parameters | Simple |
| **Multisig** | N-of-M approval for changes | Medium |
| **On-Chain Voting** | Token-weighted governance | Complex |
| **Hybrid** | Different rules for different parameters | Flexible |

---

## Performance & Scaling

Based on SELF App production data (January 2026):

### User Experience

| Operation | Latency | Notes |
|-----------|---------|-------|
| **Vote Submission** | <100ms | Browser → Coordinator |
| **Vote Confirmation** | <500ms | Acknowledgment returned |
| **Round Finality** | 60 seconds | Configurable 30s–5min |

Users experience near-instant feedback. The 60-second round time is what enables browser-based participation — shorter rounds would require always-on infrastructure.

### Scaling Economics

Traditional blockchains require you to run infrastructure for validators. PoAI inverts this:

```
Traditional PoS:                    PoAI:
┌─────────────────────┐            ┌─────────────────────┐
│  You run servers    │            │  Users hold keys    │
│  for validators     │            │  in their browser   │
│  ($5-50/month each) │            │  and sign locally   │
└─────────────────────┘            └─────────────────────┘
        ↓                                   ↓
  Your costs scale                  No per-user servers
  with users                        
```

**Validators run in user browsers.** Each user's browser holds their validator key and signs votes locally. Keys never leave their device. The signing is lightweight (Ed25519) — the browser isn't doing heavy computation, it's providing **key custody and signature authority**.

The coordination layer (proposal broadcasting, vote collection) is lightweight. It doesn't hold keys, can't sign on behalf of users, and doesn't see unencrypted data.

### Coordination Layer Scaling

| Validators | Coordination Layer | Est. Cost/Month |
|------------|-------------------|-----------------|
| 1,000 | 1 coordinator | ~$50 |
| 10,000 | 1 coordinator | ~$100 |
| 100,000 | 2 coordinators + LB | ~$300 |
| 1,000,000 | 3 coordinators + LB | ~$1,000 |

The coordination layer handles:
- Broadcasting proposals to connected validators
- Collecting signed votes
- Tracking participation (public data only)

**It does not:**
- Hold or access private keys
- Sign transactions on behalf of users
- Store sensitive user data

**Not included above:** Your application infrastructure (auth, UI, APIs) — that's yours to build.

---

## Integration Model

### Your Application ↔ SELF Chain

```
┌─────────────────────────────────────────────────────────────────┐
│                      YOUR APPLICATION                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   ┌──────────────┐    ┌──────────────┐    ┌──────────────┐      │
│   │   Your UI    │    │  Your API    │    │ Your Users   │      │
│   └──────┬───────┘    └──────┬───────┘    └──────┬───────┘      │
│          │                   │                   │               │
└──────────┼───────────────────┼───────────────────┼───────────────┘
           │                   │                   │
           ▼                   ▼                   ▼
┌──────────────────────────────────────────────────────────────────┐
│                      INTEGRATION LAYER                           │
├──────────────────────────────────────────────────────────────────┤
│                                                                   │
│   WebSocket (Validators)     REST API (Queries)     SDK          │
│   ├─ Connect/auth            ├─ Balances            ├─ JS/TS     │
│   ├─ Receive proposals       ├─ History             ├─ Rust      │
│   ├─ Submit votes            ├─ Status              └─ (others)  │
│   └─ Participation tracking  └─ Rewards                          │
│                                                                   │
└───────────────────────────────┬──────────────────────────────────┘
                                │
                                ▼
┌──────────────────────────────────────────────────────────────────┐
│                    SELF CHAIN (Managed)                          │
├──────────────────────────────────────────────────────────────────┤
│   Coordinator  ·  Consensus  ·  AI Oracle  ·  Block Production   │
└──────────────────────────────────────────────────────────────────┘
```

### Key Generation (Client-Side)

Your app derives keys from user recovery phrase:

```
Recovery Phrase (12/24 words)
        │
        ▼ BIP39
   Master Seed
        │
        ├─► Wallet Key (m/44'/60'/0'/0/0)  → Funds control
        │
        └─► Validator Key (m/44'/60'/1'/0/0) → Voting
                    │
                    └─► Never transmitted. Signs locally.
```

### Validator Connection Flow

```
1. User opens your app
2. App derives validator keypair (client-side)
3. App connects WebSocket to coordinator
4. Coordinator sends challenge
5. App signs challenge with validator key
6. Coordinator verifies → authenticated
7. App receives proposals, submits signed votes
8. Coordinator tracks participation
```

All cryptography happens in the browser. Keys never leave the device.

---

## What SELF Technology Provides

### Managed Infrastructure

| Component | Description | SLA |
|-----------|-------------|-----|
| **Coordinator** | WebSocket server for validators | 99.9% |
| **AI Oracle** | Pattern analysis, anomaly detection | 99.5% |
| **Block Production** | Finalization, chain state | 99.9% |
| **Monitoring** | Prometheus, alerting, dashboards | Included |

### Technology License

| Included | Description |
|----------|-------------|
| PoAI Consensus Core | Transaction selection, color markers, voting |
| Cryptography Suite | Classic + post-quantum hybrid |
| Validator Protocol | WebSocket auth, vote signing |
| SDK Access | JavaScript/TypeScript, Rust |

### Support Tiers

| Tier | Response Time | Includes |
|------|---------------|----------|
| **Standard** | 24h | Email support, documentation |
| **Priority** | 4h | Slack channel, priority fixes |
| **Enterprise** | 1h | Dedicated engineer, custom SLAs |

### What You Manage

| Component | Your Responsibility |
|-----------|---------------------|
| **User Application** | UI, UX, user accounts |
| **User Authentication** | Login, recovery phrase handling |
| **Token Distribution** | Airdrops, sales, vesting |
| **Governance Decisions** | Parameter changes, upgrades |
| **Compliance** | KYC/AML, regulatory filings |

---

## Comparison: PoAI vs Alternatives

### vs Proof-of-Work

| Aspect | PoW (Bitcoin) | PoAI |
|--------|---------------|------|
| Energy | Massive | Minimal |
| Hardware | Specialized ASICs | Browser/phone |
| Centralization | Mining pools | Every user validates |
| Finality | ~60 minutes | ~60 seconds |

### vs Proof-of-Stake

| Aspect | PoS (Ethereum) | PoAI |
|--------|----------------|------|
| Barrier to Entry | 32 ETH (~$100K) | Browser + app |
| Infrastructure | Dedicated server | Zero per user |
| Wealth Concentration | Rich get richer | Equal participation |
| Slashing Risk | Yes | No |

### vs Delegated PoS

| Aspect | DPoS (Solana, etc.) | PoAI |
|--------|---------------------|------|
| Validators | 100-1000 operators | Millions of users |
| Trust Model | Delegate to strangers | Validate yourself |
| Decentralization | Limited | Maximum |
| User Sovereignty | Low | Complete |

---

## Typical Engagement

### Phase 1: Evaluation (This Repo)
- Review architecture and code
- Assess fit for your use case
- Technical Q&A

### Phase 2: Design
- Token economics modeling
- Reward mechanism design
- Integration architecture
- Timeline and milestones

### Phase 3: Development
- SDK integration
- Custom reward implementation
- Testing environment
- Security review

### Phase 4: Launch
- Mainnet deployment
- Monitoring setup
- Go-live support

### Phase 5: Operation
- Ongoing infrastructure management
- Performance optimization
- Feature updates

---

## Next Steps

Ready to explore further?

**Contact:** [constellation@self.technology](mailto:constellation@self.technology)

We'll schedule a technical deep-dive to discuss:
- Your use case and requirements
- Token economics and reward design
- Integration approach
- Timeline and pricing

---

*© 2025 SELF Technology. All rights reserved. Patent pending.*

