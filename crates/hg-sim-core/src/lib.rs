//! hg-sim-core: the real SimController — orchestrates hg-engine, hg-eval,
//! hg-causal, hg-output, and hg-report behind the hg-app-api trait
//! boundary (v4 §13 Phase 10).

pub mod config;
pub mod runner;
pub mod controller;
pub mod twin;
/// Bounded development/lifecycle engineering fixtures; no optimizer admission.
#[cfg(feature = "evolving-minds")]
pub mod evolving_minds;
#[cfg(feature = "hg-meta")]
pub mod ere;
/// The learning layer's rollout API (NN plan §3). Feature-gated: without
/// `cognition` this module does not exist and no policy can be installed.
#[cfg(feature = "cognition")]
pub mod rollout;

pub use config::*;
pub use runner::{
    leakage_flags, leakage_flags_v2, load_rules, publish_graph, regime_index, run_controlled,
    run_headless, ConstructorSummary, RunArtifacts, SimError, TaskSummary,
};
#[cfg(feature = "cognition")]
pub use rollout::{build_policy, run_rollout, RolloutMetrics, RolloutRequest};
pub use controller::LiveSimController;
pub use twin::{run_twin, TwinOutcome, TwinParams, TwinPerturbation};

// Re-exported for the GUI's builtin-experiment dropdown: hg-gui depends
// on hg-sim-core (behind "live") but never on hg-rules directly.
pub use hg_rules::builtins::BUILTIN_NAMES;

/// Scripted atomic chemistry engineering profile (no kinetic clock).
#[cfg(feature = "constructive-chemistry")]
pub mod constructive_chemistry;
