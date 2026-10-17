//! ERE rule-table replacement seam. Compilation never receives engine RNG.
use crate::CompiledRule;
use hg_core::GraphSnapshot;
use serde::{Deserialize, Serialize};

pub const META_BASE: u32 = 0xFFF0_0000;
pub type RuleSet = Vec<CompiledRule>;

/// Resource allocation only; completed matching/selection semantics are identical.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatchBudgetPolicy {
    #[default]
    Split,
    Pooled,
}
impl MatchBudgetPolicy {
    pub fn is_split(&self) -> bool {
        *self == Self::Split
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum EpochPolicy {
    #[default]
    Never,
    Fixed(std::num::NonZeroU64),
}
impl EpochPolicy {
    pub fn due(self, step: u64) -> bool {
        step == 0 || matches!(self, Self::Fixed(n) if step.is_multiple_of(n.get()))
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RuleCompileReport {
    pub step: u64,
    pub installed: usize,
    pub retired: usize,
    pub quarantined: usize,
    /// Valid genotype regions, independent of expanded executable rule count.
    pub validated_regions: usize,
    pub quarantine_by_validator: [usize; 9],
    /// RCF hashes, in installed rule-id order; empty on the static path.
    pub classes: Vec<u64>,
    /// Finite vocabulary suffix, after world classes and before static editors.
    pub encoded_meta_operations: Vec<String>,
    pub changed: bool,
}
pub enum RuleSetDelta {
    Unchanged,
    Replace {
        rules: RuleSet,
        report: RuleCompileReport,
    },
}
pub trait RuleSource: Send + 'static {
    fn rules(&mut self, snapshot: &GraphSnapshot<'_>, step: u64) -> RuleSetDelta;
    fn epoch_len(&self) -> EpochPolicy;
    /// Opt-in diagnostic persistence; unsupported sources must fail closed.
    fn checkpoint(&self) -> Result<RuleSourceCheckpoint, String> {
        Err("rule source does not support diagnostic checkpoints".into())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleSourceCheckpoint {
    pub kind: String,
    pub payload: Vec<u8>,
}
/// The table is supplied to Engine::new; static sources never replace it.
pub struct StaticRuleSource(pub RuleSet);
impl RuleSource for StaticRuleSource {
    fn rules(&mut self, _: &GraphSnapshot<'_>, _: u64) -> RuleSetDelta {
        RuleSetDelta::Unchanged
    }
    fn epoch_len(&self) -> EpochPolicy {
        EpochPolicy::Never
    }
}
