//! `[cognition]` config types (`docs/policy_config.md` §1).
//!
//! Lives here rather than in `hg-sim-core` so that `hg-train` can read a
//! campaign config without depending on the simulator: `hg-sim-core` embeds
//! `Option<CognitionConfig>` behind its `cognition` feature.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PolicyKind {
    /// Equivalent to omitting `[cognition]` entirely.
    #[default]
    None,
    PassThrough,
    RandomSubset,
    DegreeGreedy,
    RuleBias,
    StaticBias,
    MlpScorer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DegreeDirection {
    #[default]
    High,
    Low,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct MlpConfig {
    #[serde(default = "d_hidden")]
    pub hidden: usize,
}

fn d_hidden() -> usize {
    16
}

impl Default for MlpConfig {
    fn default() -> Self {
        Self { hidden: d_hidden() }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CognitionConfig {
    #[serde(default)]
    pub policy: PolicyKind,
    /// Max pattern-match bindings selected per step. `None` (key omitted)
    /// means no truncation — NOT `budget = "inf"`, which is not a spelling
    /// this config accepts.
    #[serde(default)]
    pub budget: Option<usize>,
    #[serde(default = "d_wl_buckets")]
    pub wl_buckets: usize,
    #[serde(default = "d_feature_set")]
    pub feature_set: String,
    /// N-1 score normalization (`docs/policy_config.md` §4C, plan v3 §4).
    /// `"pool_standard"` (default, adopted) or `"raw"` (pre-E1, retained
    /// only for the regression that demonstrates the basin it removes).
    #[serde(default = "d_score_norm")]
    pub score_norm: String,
    #[serde(default)]
    pub rng_seed_salt: u64,
    #[serde(default)]
    pub mlp: MlpConfig,
    /// `[cognition.probe]` — the **readout**, not the policy features
    /// (`docs/policy_config.md` §4B). Versioned separately from
    /// `feature_set` because a policy feature must be cheap per candidate
    /// and a probe feature must be complete per window; v1 conflated them
    /// under one key and Stage 0b/Stage 1 paid for it.
    #[serde(default)]
    pub probe: crate::readout::ProbeConfig,
    #[serde(default)]
    pub degree_direction: DegreeDirection,
    /// Hand-set per-rule weights for `RuleBias`; length must equal the
    /// loaded rule count.
    #[serde(default)]
    pub rule_bias_weights: Vec<f32>,
}

fn d_wl_buckets() -> usize {
    16
}

fn d_feature_set() -> String {
    crate::features::FEATURE_SET_V1.to_string()
}

fn d_score_norm() -> String {
    "pool_standard".to_string()
}

impl Default for CognitionConfig {
    fn default() -> Self {
        Self {
            policy: PolicyKind::None,
            budget: None,
            wl_buckets: d_wl_buckets(),
            feature_set: d_feature_set(),
            score_norm: d_score_norm(),
            rng_seed_salt: 0,
            mlp: MlpConfig::default(),
            probe: crate::readout::ProbeConfig::default(),
            degree_direction: DegreeDirection::default(),
            rule_bias_weights: Vec::new(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ConfigError {
    ZeroBudget,
    ZeroWlBuckets,
    UnknownFeatureSet(String),
    UnknownScoreNorm(String),
    RuleBiasLengthMismatch { got: usize, want: usize },
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ZeroBudget => write!(
                f,
                "[cognition].budget = 0 selects nothing every step; omit the key for an \
                 unbounded budget"
            ),
            Self::ZeroWlBuckets => write!(f, "[cognition].wl_buckets must be >= 1"),
            Self::UnknownFeatureSet(s) => {
                write!(f, "unknown feature_set {s:?}; known sets are \"v1\" and \"v2\"")
            }
            Self::UnknownScoreNorm(s) => write!(
                f,
                "unknown score_norm {s:?}; known values are \"pool_standard\" (N-1, \
                 adopted) and \"raw\" (pre-E1)"
            ),
            Self::RuleBiasLengthMismatch { got, want } => write!(
                f,
                "[cognition].rule_bias_weights has {got} entries but {want} rules are loaded"
            ),
        }
    }
}

impl std::error::Error for ConfigError {}

impl CognitionConfig {
    /// Validate against the loaded rule count.
    ///
    /// `budget = 0` is rejected rather than silently read as "unbounded":
    /// zero is a plausible typo for "no budget" and would instead starve
    /// every pattern match for the whole run, producing a Frozen result
    /// that looks like a finding.
    pub fn validate(&self, n_rules: usize) -> Result<(), ConfigError> {
        if self.budget == Some(0) {
            return Err(ConfigError::ZeroBudget);
        }
        if self.wl_buckets == 0 {
            return Err(ConfigError::ZeroWlBuckets);
        }
        if crate::features::FeatureSet::parse(&self.feature_set).is_none() {
            return Err(ConfigError::UnknownFeatureSet(self.feature_set.clone()));
        }
        if self.score_norm_parsed().is_none() {
            return Err(ConfigError::UnknownScoreNorm(self.score_norm.clone()));
        }
        if self.policy == PolicyKind::RuleBias && self.rule_bias_weights.len() != n_rules {
            return Err(ConfigError::RuleBiasLengthMismatch {
                got: self.rule_bias_weights.len(),
                want: n_rules,
            });
        }
        Ok(())
    }

    /// Feature dimension for this config: `n_rules + wl_buckets + 7`
    /// (`docs/policy_config.md` §4).
    /// `None` when the configured string is not a known value.
    pub fn score_norm_parsed(&self) -> Option<crate::ScoreNorm> {
        match self.score_norm.as_str() {
            "pool_standard" => Some(crate::ScoreNorm::PoolStandard),
            "raw" => Some(crate::ScoreNorm::Raw),
            _ => None,
        }
    }

    pub fn feat_dim(&self, n_rules: usize) -> usize {
        crate::features::feat_dim(n_rules, self.wl_buckets)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absent_section_is_the_none_policy() {
        let cfg = CognitionConfig::default();
        assert_eq!(cfg.policy, PolicyKind::None);
        assert_eq!(cfg.budget, None);
    }

    #[test]
    fn zero_budget_is_rejected_not_read_as_unbounded() {
        let cfg = CognitionConfig {
            budget: Some(0),
            ..Default::default()
        };
        assert_eq!(cfg.validate(4), Err(ConfigError::ZeroBudget));
    }

    #[test]
    fn rule_bias_weight_count_must_match_rules() {
        let cfg = CognitionConfig {
            policy: PolicyKind::RuleBias,
            rule_bias_weights: vec![1.0, 2.0],
            ..Default::default()
        };
        assert_eq!(
            cfg.validate(4),
            Err(ConfigError::RuleBiasLengthMismatch { got: 2, want: 4 })
        );
        assert!(cfg.validate(2).is_ok());
    }

    #[test]
    fn known_feature_sets_validate_and_unknown_ones_do_not() {
        // v2 is E0's scale-free encoding of the two features v1 saturates
        // (`docs/cognition_e0_quantizer_audit.md`). v1 stays the default so
        // no existing config changes meaning.
        for ok in [crate::features::FEATURE_SET_V1, crate::features::FEATURE_SET_V2] {
            let cfg = CognitionConfig {
                feature_set: ok.into(),
                ..Default::default()
            };
            assert!(cfg.validate(1).is_ok(), "{ok} must validate");
        }
        assert_eq!(
            CognitionConfig::default().feature_set,
            crate::features::FEATURE_SET_V1,
            "the default must stay v1: a champion genome is meaningless under \
             a different normalization"
        );
        let bad = CognitionConfig {
            feature_set: "v3".into(),
            ..Default::default()
        };
        assert!(bad.validate(1).is_err());
    }

    #[test]
    fn feat_dim_matches_the_frozen_formula() {
        let cfg = CognitionConfig::default();
        assert_eq!(cfg.feat_dim(6), 6 + 16 + 7);
    }
}
