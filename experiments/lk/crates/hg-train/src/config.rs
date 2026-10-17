//! `[training]` — read by `hg-train` only (`docs/policy_config.md` §6).
//!
//! `hg-sim-core` ignores this section entirely. It lives in the campaign
//! config so that one file fully determines a campaign.

use crate::fitness::FitnessFn;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Algo {
    #[default]
    OpenaiEs,
    CmaEs,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SeedRotation {
    /// Debugging only: flags `SEED_OVERFIT` by construction.
    Fixed,
    #[default]
    PerGeneration,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrainingConfig {
    #[serde(default)]
    pub algo: Algo,
    #[serde(default = "d_pairs")]
    pub population_pairs: usize,
    /// CMA-ES population. `None` uses `4 + ⌊3 ln n⌋`.
    #[serde(default)]
    pub population_lambda: Option<usize>,
    #[serde(default = "d_sigma")]
    pub sigma: f32,
    #[serde(default = "d_alpha")]
    pub alpha: f32,
    #[serde(default = "d_generations")]
    pub generations: u32,
    #[serde(default = "d_seeds_per_member")]
    pub seeds_per_member: usize,
    #[serde(default)]
    pub seed_rotation: SeedRotation,
    #[serde(default = "d_fitness")]
    pub fitness: FitnessFn,
    #[serde(default = "d_horizon")]
    pub horizon_steps: u64,
    #[serde(default = "d_burn_in")]
    pub burn_in_windows: usize,
    #[serde(default = "d_holdout")]
    pub holdout_seeds: Vec<u64>,
    /// Seeds the whole campaign: seed schedule, ES/CMA-ES draws, and every
    /// policy RNG derivation.
    #[serde(default)]
    pub campaign_seed: u64,
}

fn d_pairs() -> usize { 32 }
fn d_sigma() -> f32 { 0.02 }
fn d_alpha() -> f32 { 0.01 }
fn d_generations() -> u32 { 60 }
fn d_seeds_per_member() -> usize { 3 }
fn d_fitness() -> FitnessFn { FitnessFn::F2RegimeOccupancy }
fn d_horizon() -> u64 { 20_000 }
fn d_burn_in() -> usize { 5 }
fn d_holdout() -> Vec<u64> { vec![900_001, 900_002, 900_003, 900_004, 900_005] }

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            algo: Algo::default(),
            population_pairs: d_pairs(),
            population_lambda: None,
            sigma: d_sigma(),
            alpha: d_alpha(),
            generations: d_generations(),
            seeds_per_member: d_seeds_per_member(),
            seed_rotation: SeedRotation::default(),
            fitness: d_fitness(),
            horizon_steps: d_horizon(),
            burn_in_windows: d_burn_in(),
            holdout_seeds: d_holdout(),
            campaign_seed: 0,
        }
    }
}

/// Note there is **no output-directory key**. The campaign path is derived
/// as `runs/cognition/<campaign_id>/` and `runs/substrate/` is unreachable
/// from `[training]` by construction (`docs/campaign_output.md` §5.1).
impl TrainingConfig {
    /// Rotation seed set for generation `g`.
    pub fn seed_set(&self, g: u32) -> Vec<u64> {
        let g = match self.seed_rotation {
            SeedRotation::Fixed => 0,
            SeedRotation::PerGeneration => g,
        };
        (0..self.seeds_per_member)
            .map(|j| mix_seed(self.campaign_seed, g, j as u32))
            .collect()
    }

    /// Every rotation seed set is disjoint from the frozen hold-outs.
    ///
    /// Checked at campaign start rather than assumed: a collision would
    /// silently invalidate `SEED_OVERFIT` for the whole campaign, and it is
    /// cheap to rule out up front.
    pub fn check_seed_disjointness(&self) -> Result<(), String> {
        if self.holdout_seeds.is_empty() {
            return Err("[training].holdout_seeds must not be empty".into());
        }
        if self.seeds_per_member == 0 {
            return Err("[training].seeds_per_member must be >= 1".into());
        }
        for g in 0..self.generations {
            for s in self.seed_set(g) {
                if self.holdout_seeds.contains(&s) {
                    return Err(format!(
                        "rotation seed {s} (generation {g}) collides with a hold-out seed — \
                         SEED_OVERFIT would be invalid for the whole campaign"
                    ));
                }
            }
        }
        Ok(())
    }
}

/// Deterministic seed mixing, written out so a dependency bump can never
/// silently change which seeds a campaign ran on.
pub fn mix_seed(campaign_seed: u64, generation: u32, index: u32) -> u64 {
    const K: u64 = 0x9e37_79b9_7f4a_7c15;
    let mut h = campaign_seed ^ 0xa076_1d64_78bd_642f;
    for x in [generation as u64, index as u64] {
        h = (h ^ x).wrapping_mul(K);
        h ^= h >> 31;
    }
    // Keep clear of the 900_00x hold-out band by construction, so
    // check_seed_disjointness is a belt-and-braces assertion rather than
    // the only thing standing between a campaign and a silent collision.
    h | (1 << 62)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotation_seeds_change_per_generation() {
        let cfg = TrainingConfig { campaign_seed: 7, ..Default::default() };
        assert_ne!(cfg.seed_set(0), cfg.seed_set(1));
        assert_eq!(cfg.seed_set(3), cfg.seed_set(3));
    }

    #[test]
    fn fixed_rotation_pins_the_seed_set() {
        let cfg = TrainingConfig {
            campaign_seed: 7,
            seed_rotation: SeedRotation::Fixed,
            ..Default::default()
        };
        assert_eq!(cfg.seed_set(0), cfg.seed_set(9));
    }

    #[test]
    fn rotation_seeds_are_disjoint_from_holdouts() {
        let cfg = TrainingConfig { campaign_seed: 12345, ..Default::default() };
        cfg.check_seed_disjointness().expect("disjoint");
    }

    #[test]
    fn a_holdout_collision_is_rejected() {
        let cfg = TrainingConfig { campaign_seed: 1, ..Default::default() };
        let colliding = TrainingConfig {
            holdout_seeds: cfg.seed_set(0),
            ..cfg
        };
        assert!(colliding.check_seed_disjointness().is_err());
    }

    #[test]
    fn empty_holdouts_are_rejected() {
        let cfg = TrainingConfig { holdout_seeds: vec![], ..Default::default() };
        assert!(cfg.check_seed_disjointness().is_err());
    }

    #[test]
    fn defaults_match_the_documented_schema() {
        let cfg: TrainingConfig = toml::from_str("").expect("empty section is valid");
        assert_eq!(cfg.algo, Algo::OpenaiEs);
        assert_eq!(cfg.population_pairs, 32);
        assert_eq!(cfg.sigma, 0.02);
        assert_eq!(cfg.alpha, 0.01);
        assert_eq!(cfg.generations, 60);
        assert_eq!(cfg.seeds_per_member, 3);
        assert_eq!(cfg.fitness, FitnessFn::F2RegimeOccupancy);
        assert_eq!(cfg.horizon_steps, 20_000);
        assert_eq!(cfg.burn_in_windows, 5);
        assert_eq!(cfg.holdout_seeds, vec![900_001, 900_002, 900_003, 900_004, 900_005]);
    }
}
