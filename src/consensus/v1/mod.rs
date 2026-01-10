//! PoAI v1 Consensus Types
//!
//! Spec-compliant consensus types and configuration as defined in
//! `docs/POAI_SPECIFICATION.md`.
//!
//! ## Protocol Constants
//!
//! | Parameter | Value | Description |
//! |-----------|-------|-------------|
//! | `CHAIN_ID` | Constellation-specific | Unique chain identifier |
//! | `BLOCK_TIME_TARGET` | 60 seconds | Target time between blocks |
//! | `QUORUM_THRESHOLD` | 2/3 | Fraction required for commit |
//! | `COMMITTEE_SIZE_MIN` | 10 | Minimum committee members |
//! | `COMMITTEE_SIZE_MAX` | 100 | Maximum committee members |
//!
//! ## Round Steps
//!
//! ```text
//! PoAI Competition Model:
//! ┌───────────────┐  ┌───────────────┐  ┌───────────────┐
//! │ ProposeWindow │─>│    Voting     │─>│   Finalize    │
//! │   (0-50s)     │  │  (50-58s)     │  │  (58-60s)     │
//! └───────────────┘  └───────────────┘  └───────────────┘
//! ```

pub mod types;

pub use types::{
    ConsensusConfig, RoundStep, RoundState, ValidatorInfo,
    ConsensusMessage, ConsensusError, ConsensusResult,
    constants,
};
