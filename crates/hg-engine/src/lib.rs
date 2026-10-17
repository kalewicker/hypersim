//! hg-engine: the rewriting engine (v4 §2.3).
//!
//! Only `hg-sim-core` imports this crate directly; everything else plugs
//! into the step loop through `hg-observe`'s traits.

pub mod matcher;
pub mod conflict;
pub mod engine;

pub use matcher::{find_bindings, find_bindings_counted, find_bindings_counted_feasible,
    find_bindings_counted_legacy, GuardStats, MATCHER_WORK_MODEL};
pub use conflict::{conflict_set, resolve_conflicts, ConflictSet};
pub use engine::{apply_binding, Engine, MatchBudgetUsage, RuleMatchUsage};

/// Explicit, fail-closed graph patches; ordinary engine effects are unchanged.
#[cfg(feature = "checked-transitions")]
pub mod checked;
