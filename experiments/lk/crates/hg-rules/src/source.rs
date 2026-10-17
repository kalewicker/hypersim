//! The `RuleSource` seam (plan §2.2) — in hg-rules per the plan's dependency
//! argument: it produces a `RuleSet` (hg-rules' own type family) from a
//! snapshot, and hg-engine already depends on hg-rules, so the engine can
//! hold one with no new crate edges. hg-observe stays untouched.

use crate::rule::Rule;
use hg_core::GraphSnapshot;

/// When the source wants its consultation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EpochPolicy {
    /// Never consult — the static default's behavior.
    Never,
    /// Consult every n steps at the epoch boundary (before step k*n).
    Fixed(u64),
}

impl EpochPolicy {
    /// Is this step an epoch boundary?
    pub fn due(&self, step: u64) -> bool {
        match self {
            EpochPolicy::Never => false,
            EpochPolicy::Fixed(n) => *n > 0 && step.is_multiple_of(*n),
        }
    }
}

/// Result of one consultation: which rules exist after this boundary.
/// Installation order is the Vec order; callers must derive rule ids from
/// THIS order (determinism hazard D1 lives in the source's enumeration).
#[derive(Clone, Debug, Default)]
pub struct RuleSetDelta {
    pub rules: Vec<Rule>,
    /// Counters for G-R4 / metrics rows.
    pub quarantined: usize,
    pub read_rejected: usize,
}

/// The seam. Called only at epoch boundaries; MUST be deterministic and
/// MUST NOT consume any engine RNG stream (plan §2.4 hazard D2).
pub trait RuleSource: Send + 'static {
    fn rules(&mut self, snapshot: &GraphSnapshot<'_>, step: u64) -> RuleSetDelta;
    fn epoch_len(&self) -> EpochPolicy;
}

/// The default: a fixed table, never consulted.
#[derive(Clone)]
pub struct StaticRuleSource(pub Vec<Rule>);

impl RuleSource for StaticRuleSource {
    fn rules(&mut self, _snapshot: &GraphSnapshot<'_>, _step: u64) -> RuleSetDelta {
        RuleSetDelta {
            rules: self.0.clone(),
            quarantined: 0,
            read_rejected: 0,
        }
    }
    fn epoch_len(&self) -> EpochPolicy {
        EpochPolicy::Never
    }
}
