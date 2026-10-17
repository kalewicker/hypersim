//! hg-cognition: the policy zoo and feature extractor for the learning
//! layer (`hypersim_nn_experiments_plan_v1.md` §5–6).
//!
//! Everything here implements `hg_observe::RuleSelectionPolicy`, the seam
//! v4 §6.1 reserved: a policy sits between candidate collection and
//! conflict resolution and returns the indices it wants considered. Two
//! properties follow and are worth stating where they cannot be missed:
//!
//! 1. **A policy can never violate the rewrite contract.** Node-disjointness
//!    is enforced downstream by the engine's conflict resolver, whatever a
//!    policy returns. The learned component is sandboxed by construction.
//! 2. **A policy can only choose among what physics offers.** It cannot
//!    invent rewrites, only bias which offered rewrites happen.
//!
//! The one thing a policy *can* wrongly do is starve a scheduled creation
//! rule — the engine runs the policy before its own creations/ordinary
//! split and does not restore dropped creation bindings. Every policy here
//! therefore passes creation bindings through unconditionally
//! (`docs/policy_config.md` §3), and `tests/creation_exempt.rs` asserts it
//! for the whole zoo. `StimulusDriver` is the single sanctioned exception
//! and is never a learned component.

pub mod config;
pub mod features;
pub mod genome;
pub mod policies;
pub mod readout;
pub mod select;
#[doc(hidden)]
pub mod testkit;

pub use config::{CognitionConfig, DegreeDirection, MlpConfig, PolicyKind};
pub use features::{FeatureExtractor, CAND_SCORE_CAP, DEG_SCALE, FEATURE_SET_V1};
pub use genome::Parameterized;
pub use readout::{ProbeConfig, ReadoutSet};
pub use policies::{
    pool_standardize, DegreeGreedy, MlpScorer, PassThrough, PolicyStack, RandomSubset, RuleBias,
    ScoreNorm, StaticBias, SCORE_NORM_EPS,
    StimulusDriver,
};
pub use select::{derive_policy_seed, gumbel_top_k, partition_candidates};
