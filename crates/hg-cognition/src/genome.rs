//! `Parameterized` and champion serialization
//! (`docs/campaign_output.md` §6).
//!
//! The optimizer sees a flat `f32` vector and nothing else. That is the
//! whole point of the interface: swapping ES for something stronger later
//! touches `hg-train` only, and no optimizer ever learns a policy's
//! internal layout.

use serde::{Deserialize, Serialize};

pub trait Parameterized {
    fn n_params(&self) -> usize;
    /// Load a flat parameter vector. Implementations must accept exactly
    /// `n_params()` values and panic otherwise — a silently truncated or
    /// zero-padded genome would produce a policy that runs, scores, and
    /// means nothing.
    fn set_params(&mut self, theta: &[f32]);
    fn get_params(&self) -> Vec<f32>;
}

/// Serialized champion (`champions/gen_NNNN.policy.toml`).
///
/// `n_rules`, `wl_buckets`, and `feature_set` are recorded because the
/// feature dimension is `n_rules + wl_buckets + 7`: a genome trained under
/// one feature layout has no meaning under another, so loading it against a
/// mismatched rule set is rejected rather than reshaped.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChampionRecord {
    pub schema_version: u32,
    pub campaign_id: String,
    pub generation: u32,
    pub policy: crate::config::PolicyKind,
    pub feature_set: String,
    pub n_rules: usize,
    pub wl_buckets: usize,
    pub budget: Option<usize>,
    pub hidden: usize,
    /// Hex, as `"0x…"`. A string rather than an integer because TOML
    /// integers are **i64**: a `u64` hash with the high bit set is
    /// "out-of-range value for u64 type" at serialization time, which would
    /// make roughly half of all champions unsaveable — and only on the
    /// campaigns unlucky enough to produce one.
    pub genome_hash: String,
    pub fitness_fn: String,
    pub holdout_fitness: f64,
    pub params: Vec<f32>,
}

pub const CHAMPION_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, PartialEq, Eq)]
pub enum ChampionError {
    SchemaVersion { got: u32, want: u32 },
    FeatureSet { got: String, want: String },
    RuleCount { got: usize, want: usize },
    WlBuckets { got: usize, want: usize },
    ParamCount { got: usize, want: usize },
    GenomeHash { got: String, want: String },
}

impl std::fmt::Display for ChampionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SchemaVersion { got, want } => {
                write!(f, "champion schema_version {got}, expected {want}")
            }
            Self::FeatureSet { got, want } => write!(
                f,
                "champion was trained under feature_set {got:?}, this build uses {want:?}"
            ),
            Self::RuleCount { got, want } => write!(
                f,
                "champion was trained against {got} rules, {want} are loaded — the feature \
                 layout differs, so the genome has no meaning here"
            ),
            Self::WlBuckets { got, want } => {
                write!(f, "champion used wl_buckets = {got}, this config uses {want}")
            }
            Self::ParamCount { got, want } => {
                write!(f, "champion carries {got} params, the policy needs {want}")
            }
            Self::GenomeHash { got, want } => write!(
                f,
                "champion genome_hash {got} does not match its params ({want}) — the \
                 file was edited or truncated"
            ),
        }
    }
}

impl std::error::Error for ChampionError {}

impl ChampionRecord {
    /// Reject a champion that cannot mean what it says against this
    /// rule set and feature layout.
    pub fn check(&self, n_rules: usize, wl_buckets: usize, n_params: usize) -> Result<(), ChampionError> {
        if self.schema_version != CHAMPION_SCHEMA_VERSION {
            return Err(ChampionError::SchemaVersion {
                got: self.schema_version,
                want: CHAMPION_SCHEMA_VERSION,
            });
        }
        if self.feature_set != crate::features::FEATURE_SET_V1 {
            return Err(ChampionError::FeatureSet {
                got: self.feature_set.clone(),
                want: crate::features::FEATURE_SET_V1.to_string(),
            });
        }
        if self.n_rules != n_rules {
            return Err(ChampionError::RuleCount {
                got: self.n_rules,
                want: n_rules,
            });
        }
        if self.wl_buckets != wl_buckets {
            return Err(ChampionError::WlBuckets {
                got: self.wl_buckets,
                want: wl_buckets,
            });
        }
        if self.params.len() != n_params {
            return Err(ChampionError::ParamCount {
                got: self.params.len(),
                want: n_params,
            });
        }
        let actual = genome_hash_hex(&self.params);
        if self.genome_hash != actual {
            return Err(ChampionError::GenomeHash {
                got: self.genome_hash.clone(),
                want: actual,
            });
        }
        Ok(())
    }
}

/// Stable hash of a parameter vector, over the `f32` **bit patterns**.
///
/// Bits, not values: two genomes that differ only in a bit that `f32`
/// comparison calls equal (−0.0 vs 0.0) still reproduce differently once
/// they hit the arithmetic, and a provenance hash that cannot see the
/// difference is not doing its job. NaN is likewise distinguished from
/// itself only bitwise.
pub fn genome_hash_hex(theta: &[f32]) -> String {
    format!("{:#018x}", genome_hash(theta))
}

pub fn genome_hash(theta: &[f32]) -> u64 {
    const K: u64 = 0x517c_c1b7_2722_0a95;
    let mut h: u64 = 0xcbf2_9ce4_8422_2325 ^ theta.len() as u64;
    for v in theta {
        h = (h ^ v.to_bits() as u64).wrapping_mul(K);
        h ^= h >> 29;
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(params: Vec<f32>) -> ChampionRecord {
        ChampionRecord {
            schema_version: CHAMPION_SCHEMA_VERSION,
            campaign_id: "test_1".into(),
            generation: 42,
            policy: crate::config::PolicyKind::MlpScorer,
            feature_set: crate::features::FEATURE_SET_V1.into(),
            n_rules: 4,
            wl_buckets: 16,
            budget: Some(16),
            hidden: 16,
            genome_hash: genome_hash_hex(&params),
            fitness_fn: "f2_regime_occupancy".into(),
            holdout_fitness: 0.412,
            params,
        }
    }

    /// The Phase C1 exit criterion: exact round-trip. A champion that
    /// reloads to a slightly different vector reproduces a slightly
    /// different run — quietly breaking the determinism contract instead
    /// of loudly breaking a test.
    #[test]
    fn toml_round_trip_is_bit_exact() {
        let params: Vec<f32> = (0..129)
            .map(|i| (i as f32 * 0.017_231).sin() * 3.3)
            .chain([f32::MIN_POSITIVE, -0.0, 1.0 / 3.0])
            .collect();
        let rec = record(params.clone());

        let text = toml::to_string(&rec).expect("serialize");
        let back: ChampionRecord = toml::from_str(&text).expect("deserialize");

        assert_eq!(back.params.len(), params.len());
        for (a, b) in params.iter().zip(&back.params) {
            assert_eq!(a.to_bits(), b.to_bits(), "{a} round-tripped to {b}");
        }
        assert_eq!(back.genome_hash, rec.genome_hash);
        assert!(back.check(4, 16, params.len()).is_ok());
    }

    #[test]
    fn genome_hash_sees_signed_zero() {
        assert_ne!(genome_hash(&[0.0, 1.0]), genome_hash(&[-0.0, 1.0]));
    }

    #[test]
    fn genome_hash_sees_length_and_order() {
        assert_ne!(genome_hash(&[1.0, 2.0]), genome_hash(&[2.0, 1.0]));
        assert_ne!(genome_hash(&[1.0]), genome_hash(&[1.0, 0.0]));
    }

    #[test]
    fn mismatched_rule_count_is_rejected() {
        let rec = record(vec![0.5; 8]);
        assert_eq!(
            rec.check(7, 16, 8),
            Err(ChampionError::RuleCount { got: 4, want: 7 })
        );
    }

    /// The failure this hex encoding exists to prevent: a high-bit-set
    /// hash must serialize, not overflow TOML's i64 integers.
    #[test]
    fn a_high_bit_hash_still_serializes() {
        // Search for params whose hash has the top bit set — half of all
        // genomes do, and under the old u64 encoding every one of them
        // failed to save.
        let params = (0u32..)
            .map(|i| vec![i as f32, 0.5, -1.25])
            .find(|p| genome_hash(p) & (1 << 63) != 0)
            .expect("some genome hashes high");
        let rec = record(params.clone());
        let text = toml::to_string(&rec).expect("high-bit hash must serialize");
        let back: ChampionRecord = toml::from_str(&text).expect("deserialize");
        assert_eq!(back.genome_hash, rec.genome_hash);
        assert!(back.check(4, 16, params.len()).is_ok());
    }

    #[test]
    fn edited_params_fail_the_hash_check() {
        let mut rec = record(vec![0.5; 8]);
        rec.params[3] = 0.6;
        assert!(matches!(
            rec.check(4, 16, 8),
            Err(ChampionError::GenomeHash { .. })
        ));
    }
}
