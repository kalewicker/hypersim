//! G-R1 harness (plan §6): sample mutations, classify, threshold.
//!
//! Classification is EXCLUSIVE — every sample lands in exactly one bucket,
//! so `total()` is honest and rates need no denominators beyond it:
//!
//! - `Nothing`       — sampled parameters out of range for this rule;
//!   counted separately, never folded into anything.
//! - `Quarantined`   — child fails any of V1..V9.
//! - `Neutral`       — child valid AND RCF-identical to parent.
//! - `ValidDistinct` — child valid AND RCF-distinct from parent.
//!
//! Pre-committed thresholds read off these buckets (plan §6):
//!
//! - valid_rate  = (neutral + valid_distinct) / total   >= 0.40
//! - neutral_rate = neutral / total                      >= 0.10
//!
//! (catastrophic <= 0.25 is an ENGINE-side class; deferred to R3 and
//! recorded as unmeasured here, per plan §8's unmeasured-vs-zero rule.)

use crate::mutate::{mutate, Applied, MutationRng};
use crate::rcf::rcf_hash;
use crate::schema::{CandidateRule, RegionFootprint};
use crate::validate::{validate, PriorityObservation, Verdict};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Class {
    Nothing,
    Quarantined,
    Neutral,
    ValidDistinct,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ClassCounts {
    pub nothing: usize,
    pub quarantined: usize,
    pub neutral: usize,
    pub valid_distinct: usize,
}

impl ClassCounts {
    pub fn total(&self) -> usize {
        self.nothing + self.quarantined + self.neutral + self.valid_distinct
    }

    /// Installable fraction over all samples (plan threshold ≥ 0.40).
    pub fn valid_rate(&self) -> f64 {
        let t = self.total();
        if t == 0 {
            0.0
        } else {
            (self.neutral + self.valid_distinct) as f64 / t as f64
        }
    }

    /// Semantics-preserving fraction over all samples (threshold ≥ 0.10).
    pub fn neutral_rate(&self) -> f64 {
        let t = self.total();
        if t == 0 {
            0.0
        } else {
            self.neutral as f64 / t as f64
        }
    }

    /// Malformation rate (no ceiling pre-committed; reported alongside).
    pub fn quarantine_rate(&self) -> f64 {
        let t = self.total();
        if t == 0 {
            0.0
        } else {
            self.quarantined as f64 / t as f64
        }
    }

    /// Alias matching the plan's vocabulary.
    pub fn quarantined_rate(&self) -> f64 {
        self.quarantine_rate()
    }

    /// Out-of-range sampling fraction (kernel coverage diagnostic).
    pub fn nothing_rate(&self) -> f64 {
        let t = self.total();
        if t == 0 {
            0.0
        } else {
            self.nothing as f64 / t as f64
        }
    }
}

/// Structural footprint of a candidate without a snapshot (offline mode):
/// nodes = root+sides+PEs+vars+lits; edges = BECOMEs+MEMBERs+SLOTs+HASLABELs.
pub fn offline_footprint(rule: &CandidateRule) -> RegionFootprint {
    let n_pes = rule.lhs.len() + rule.rhs.len();
    let label_lits = rule.var_labels.iter().flatten().count();
    RegionFootprint {
        nodes: 3 + n_pes + rule.n_vars() + label_lits,
        edges: 2 * n_pes
            + rule
                .lhs
                .iter()
                .chain(rule.rhs.iter())
                .map(|e| e.slots.len())
                .sum::<usize>()
            + label_lits,
    }
}

/// Apply and classify ONE mutation of `parent`.
pub fn classify_one(parent: &CandidateRule, rng: &mut dyn MutationRng) -> (Class, ClassCounts) {
    let mut counts = ClassCounts::default();
    let (child, applied) = mutate(parent, rng);
    match applied {
        Applied::Nothing => {
            counts.nothing += 1;
            return (Class::Nothing, counts);
        }
        Applied::Edited => {}
    }

    match validate(
        &child,
        &offline_footprint(&child),
        &PriorityObservation { malformed: false },
    ) {
        Verdict::Quarantined(_) => {
            counts.quarantined += 1;
            (Class::Quarantined, counts)
        }
        Verdict::Valid => {
            if rcf_hash(&child) == rcf_hash(parent) {
                counts.neutral += 1;
                (Class::Neutral, counts)
            } else {
                counts.valid_distinct += 1;
                (Class::ValidDistinct, counts)
            }
        }
    }
}
