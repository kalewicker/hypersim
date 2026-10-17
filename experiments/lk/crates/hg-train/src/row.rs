//! `CampaignRow` — one row per rollout (`docs/campaign_output.md` §3.1).
//!
//! Written through `hg_output::CsvWriter<CampaignRow>`. No new output
//! infrastructure is needed: the generic writer already emits the
//! `#hypersim_csv_v=…` provenance comment and derives the header from this
//! type's field names, so the campaign CSV is self-identifying by exactly
//! the same convention as a run CSV.

use crate::fitness::{FitnessBreakdown, HeldOut, Reported, RolloutFlags};
use serde::{Deserialize, Serialize};

/// Version history. Every bump is strictly additive — an earlier CSV's
/// columns all survive with their meanings, so an older reader still parses
/// the prefix and no published number changes under a bump.
///
/// | v | phase | what it added |
/// |---|---|---|
/// | 2 | D1 | `arm`, F4/F5, eight engagement counters, three tripwires, `RolloutStatus::Unmeasured` |
/// | 3 | E0 | F4's drift columns (`f4_jsd_spread`, `f4_trend`, `f4_share_of_ceiling`) |
/// | 4 | E2 | `genome_l2` — the radial coordinate N-1 makes behaviourally inert |
pub const CAMPAIGN_CSV_SCHEMA_VERSION: u32 = 4;

/// Status of one rollout. A failure is a written row, not a dropped one: a
/// campaign where 30% of members died is a campaign with a problem, and that
/// fact must survive in the CSV rather than only in a log line (§3.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RolloutStatus {
    Ok,
    NoScoredWindows,
    SimError,
    /// v2. The rollout ran and every other column is valid, but the
    /// *selected* fitness function's instrument never engaged
    /// (`docs/instrument_registry.md` §1). Distinct from `NoScoredWindows`,
    /// which invalidates the whole rollout, and distinct from a score of
    /// `0.0`, which is a measurement.
    Unmeasured,
}

impl RolloutStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::NoScoredWindows => "no_scored_windows",
            Self::SimError => "sim_error",
            Self::Unmeasured => "unmeasured",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CampaignRow {
    // --- identity
    pub generation: u32,
    pub member_idx: u32,
    /// `+1` / `-1` for ES antithetic arms, `0` for CMA-ES.
    pub antithetic: i8,
    pub sim_seed: u64,
    pub rollout_idx: u32,
    pub policy_rng_seed: u64,
    pub genome_hash: u64,
    /// `‖θ‖₂` for the genome this row was produced by.
    ///
    /// Recorded because N-1 (`docs/policy_config.md` §4C) makes the **radial**
    /// component of θ behaviourally inert: standardized scores are exactly
    /// invariant under `θ → cθ`, so a genome that only grows in norm is the
    /// same policy. But `σ` and `α` are absolute, so a drifting norm silently
    /// rescales the search — the sampled angular width `atan(σ√n/‖θ‖)` and
    /// the angular step both shrink as ‖θ‖ grows. That is an unintended
    /// annealing schedule, it is invisible in the fitness column, and
    /// `genome_hash` cannot show it. E2 exists partly to measure it.
    pub genome_l2: f64,
    pub is_holdout: bool,
    /// `random` | `untrained` | `champion` — the three-arm protocol
    /// (`docs/campaign_output.md` §4.2). Every row carries one; there is no
    /// default and no unlabelled row, because the C0–C5 confound was
    /// precisely that the untrained arm had nowhere to live.
    pub arm: String,

    // --- fitness, always decomposed
    pub fitness: Option<f64>,
    pub fitness_fn: String,
    pub f1_mean_interestingness: Option<f64>,
    /// Target-regime fraction BEFORE the penalty. Recorded separately from
    /// `f2_penalty` and never only as their difference: penalty domination
    /// is a live risk (`docs/fitness_formulas.md` §3) and is invisible in
    /// the sum.
    pub f2_reward: Option<f64>,
    pub f2_penalty: Option<f64>,
    pub f3_constructor_yield: Option<f64>,
    pub n_confirmed_constructors: Option<usize>,
    pub mean_fidelity: Option<f64>,
    /// v2 §4A. Renders as the literal `unmeasured` when its instrument did
    /// not engage — never as an empty cell and never as `0.0`.
    pub f4_monotone_drift: Option<Reported>,
    /// v2 §4B. Same rendering contract.
    pub f5_constructor_reuse: Option<Reported>,

    // --- engagement counters (docs/instrument_registry.md §2)
    pub ei_intervals_emitted: usize,
    pub task_lattice_entries: usize,
    pub task_lattice_chains: usize,
    pub constructor_records_examined: usize,
    pub f5_records_examined: usize,
    pub f4_pairs: usize,
    pub f1_windows_scored: usize,
    pub probe_rows_collected: usize,
    /// Whether the causal layer ran. Gates F3 and F5 (v2.1) — the record
    /// counts above do not, because an empty registry with the layer live
    /// is a measured zero, not a non-measurement.
    pub causal_enabled: bool,

    // --- regime occupancy, in the docs/fitness_formulas.md §3.1 order
    pub occ_frozen: u64,
    pub occ_static_ordered: u64,
    pub occ_growth_treadmill: u64,
    pub occ_white_noise_soup: u64,
    pub occ_sterile_chaos: u64,
    pub occ_exploding: u64,
    pub occ_fragmented_isolation: u64,
    pub occ_monoculture: u64,
    pub occ_active_persistent_structure: u64,
    pub occ_generative_candidate: u64,
    pub occ_unknown: u64,

    // --- held out, never optimized
    pub growth_exponent_nodes: Option<f64>,
    pub growth_exponent_edges: Option<f64>,
    pub mean_type_entropy: Option<f64>,
    pub mean_type_jsd: Option<f64>,
    pub mean_mean_degree: Option<f64>,
    pub mean_active_persistent: Option<f64>,
    pub mean_static_persistent: Option<f64>,
    pub archive_admission_rate: Option<f64>,
    pub mean_n_nodes: Option<f64>,
    pub replicator_rate: Option<f64>,
    pub kernel_held: Option<f64>,
    pub f5_top_share: Option<f64>,
    pub f4_null_sd: Option<f64>,
    /// v2.1 §4A.2 / §4A.6 — F4's resolution and drift context. §4A.6 makes
    /// these mandatory alongside any published F4 value.
    pub f4_jsd_spread: Option<f64>,
    pub f4_trend: Option<f64>,
    pub f4_share_of_ceiling: Option<f64>,

    // --- tripwires
    pub flag_fitness_structure_divergence: bool,
    pub flag_seed_overfit: bool,
    pub flag_degenerate_occupancy: bool,
    pub flag_degenerate_reuse: bool,
    pub flag_f4_degenerate_alphabet: bool,
    pub flag_instrument_not_engaged: bool,

    // --- provenance and cost
    pub final_canonical_hash: Option<u64>,
    pub n_nodes_initial: Option<usize>,
    pub n_windows_scored: usize,
    pub status: String,
    pub wallclock_ms: u64,
    /// Recorded per row: the budget is the load-bearing experimental
    /// variable, not a footnote.
    pub budget: Option<usize>,
    pub cap_bound_steps: u64,
    pub pairings_cap_bound_steps: u64,
    pub error: Option<String>,
}

/// Identity fields, shared by the ok and failure constructors.
pub struct RowIdentity {
    pub generation: u32,
    pub member_idx: u32,
    pub antithetic: i8,
    pub sim_seed: u64,
    pub rollout_idx: u32,
    pub policy_rng_seed: u64,
    pub genome_hash: u64,
    pub genome_l2: f64,
    pub is_holdout: bool,
    pub fitness_fn: String,
    pub budget: Option<usize>,
    pub arm: Arm,
}

/// The three-arm protocol (`docs/campaign_output.md` §4.2).
///
/// v1 compared the champion only against budget-matched `RandomSubset`,
/// "nothing else". That exclusion is what let a policy-*class* effect read
/// as a learning effect across three stages. [`Arm::Untrained`] is the
/// control that separates them, and the primary reported statistic is
/// `champion − untrained`, never `champion − random`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Arm {
    /// Budget-matched `RandomSubset`: isolates the budget effect alone.
    Random,
    /// The policy class at initialization (θ=0 / class defaults): isolates
    /// the allocation-class effect.
    Untrained,
    /// The trained champion: the learning effect is what it adds over
    /// [`Arm::Untrained`].
    Champion,
}

impl Arm {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Random => "random",
            Self::Untrained => "untrained",
            Self::Champion => "champion",
        }
    }
}

impl CampaignRow {
    fn skeleton(id: RowIdentity, status: RolloutStatus) -> Self {
        Self {
            generation: id.generation,
            member_idx: id.member_idx,
            antithetic: id.antithetic,
            sim_seed: id.sim_seed,
            rollout_idx: id.rollout_idx,
            policy_rng_seed: id.policy_rng_seed,
            genome_hash: id.genome_hash,
            genome_l2: id.genome_l2,
            is_holdout: id.is_holdout,
            arm: id.arm.as_str().to_string(),
            fitness: None,
            fitness_fn: id.fitness_fn,
            f1_mean_interestingness: None,
            f2_reward: None,
            f2_penalty: None,
            f3_constructor_yield: None,
            n_confirmed_constructors: None,
            mean_fidelity: None,
            f4_monotone_drift: None,
            f5_constructor_reuse: None,
            ei_intervals_emitted: 0,
            task_lattice_entries: 0,
            task_lattice_chains: 0,
            constructor_records_examined: 0,
            f5_records_examined: 0,
            f4_pairs: 0,
            f1_windows_scored: 0,
            probe_rows_collected: 0,
            causal_enabled: false,
            occ_frozen: 0,
            occ_static_ordered: 0,
            occ_growth_treadmill: 0,
            occ_white_noise_soup: 0,
            occ_sterile_chaos: 0,
            occ_exploding: 0,
            occ_fragmented_isolation: 0,
            occ_monoculture: 0,
            occ_active_persistent_structure: 0,
            occ_generative_candidate: 0,
            occ_unknown: 0,
            growth_exponent_nodes: None,
            growth_exponent_edges: None,
            mean_type_entropy: None,
            mean_type_jsd: None,
            mean_mean_degree: None,
            mean_active_persistent: None,
            mean_static_persistent: None,
            archive_admission_rate: None,
            mean_n_nodes: None,
            replicator_rate: None,
            kernel_held: None,
            f5_top_share: None,
            f4_null_sd: None,
            f4_jsd_spread: None,
            f4_trend: None,
            f4_share_of_ceiling: None,
            flag_fitness_structure_divergence: false,
            flag_seed_overfit: false,
            flag_degenerate_occupancy: false,
            flag_degenerate_reuse: false,
            flag_f4_degenerate_alphabet: false,
            flag_instrument_not_engaged: false,
            final_canonical_hash: None,
            n_nodes_initial: None,
            n_windows_scored: 0,
            status: status.as_str().to_string(),
            wallclock_ms: 0,
            budget: id.budget,
            cap_bound_steps: 0,
            pairings_cap_bound_steps: 0,
            error: None,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn ok(
        id: RowIdentity,
        fitness: Result<f64, crate::fitness::FitnessError>,
        fit: &FitnessBreakdown,
        ho: &HeldOut,
        flags: RolloutFlags,
        occupancy: [u64; 11],
        final_hash: u64,
        n_nodes_initial: usize,
        wallclock_ms: u64,
        cap_bound_steps: u64,
        pairings_cap_bound_steps: u64,
    ) -> Self {
        // A rollout whose *selected* fitness is unmeasured still produced
        // every other column, so the row is written in full — but its
        // `fitness` cell stays empty and its status says why. Writing 0.0
        // here is bug 8 (`docs/instrument_registry.md` §2.2).
        let status = match fitness {
            Ok(_) => RolloutStatus::Ok,
            Err(_) => RolloutStatus::Unmeasured,
        };
        let mut r = Self::skeleton(id, status);
        r.fitness = fitness.ok();
        if let Err(e) = fitness {
            r.error = Some(e.to_string());
        }
        r.f4_monotone_drift = Some(fit.f4_monotone_drift);
        r.f5_constructor_reuse = Some(fit.f5_constructor_reuse);
        r.ei_intervals_emitted = fit.engagement.ei_intervals_emitted;
        r.task_lattice_entries = fit.engagement.task_lattice_entries;
        r.task_lattice_chains = fit.engagement.task_lattice_chains;
        r.constructor_records_examined = fit.engagement.constructor_records_examined;
        r.f5_records_examined = fit.engagement.f5_records_examined;
        r.f4_pairs = fit.engagement.f4_pairs;
        r.f1_windows_scored = fit.engagement.f1_windows_scored;
        r.probe_rows_collected = fit.engagement.probe_rows_collected;
        r.causal_enabled = fit.engagement.causal_enabled;
        r.f1_mean_interestingness = Some(fit.f1_mean_interestingness);
        r.f2_reward = Some(fit.f2_reward);
        r.f2_penalty = Some(fit.f2_penalty);
        r.f3_constructor_yield = Some(fit.f3_constructor_yield);
        r.n_confirmed_constructors = Some(fit.n_confirmed_constructors);
        r.mean_fidelity = Some(fit.mean_fidelity);
        r.set_occupancy(occupancy);
        r.growth_exponent_nodes = ho.growth_exponent_nodes;
        r.growth_exponent_edges = ho.growth_exponent_edges;
        r.mean_type_entropy = ho.mean_type_entropy;
        r.mean_type_jsd = ho.mean_type_jsd;
        r.mean_mean_degree = ho.mean_mean_degree;
        r.mean_active_persistent = ho.mean_active_persistent;
        r.mean_static_persistent = ho.mean_static_persistent;
        r.archive_admission_rate = ho.archive_admission_rate;
        r.mean_n_nodes = Some(ho.mean_n_nodes);
        r.replicator_rate = Some(ho.replicator_rate);
        r.kernel_held = ho.kernel_held;
        r.f5_top_share = Some(ho.f5_top_share);
        r.f4_null_sd = Some(ho.f4_null_sd);
        r.f4_jsd_spread = Some(ho.f4_jsd_spread);
        r.f4_trend = Some(ho.f4_trend);
        r.f4_share_of_ceiling = Some(ho.f4_share_of_ceiling);
        r.flag_degenerate_occupancy = flags.degenerate_occupancy;
        r.flag_degenerate_reuse = flags.degenerate_reuse;
        r.flag_f4_degenerate_alphabet = flags.f4_degenerate_alphabet;
        r.flag_instrument_not_engaged = flags.instrument_not_engaged;
        r.final_canonical_hash = Some(final_hash);
        r.n_nodes_initial = Some(n_nodes_initial);
        r.n_windows_scored = fit.n_windows_scored;
        r.wallclock_ms = wallclock_ms;
        r.cap_bound_steps = cap_bound_steps;
        r.pairings_cap_bound_steps = pairings_cap_bound_steps;
        r
    }

    pub fn failed(id: RowIdentity, status: RolloutStatus, error: String, wallclock_ms: u64) -> Self {
        let mut r = Self::skeleton(id, status);
        r.error = Some(error);
        r.wallclock_ms = wallclock_ms;
        r
    }

    fn set_occupancy(&mut self, o: [u64; 11]) {
        self.occ_frozen = o[0];
        self.occ_static_ordered = o[1];
        self.occ_growth_treadmill = o[2];
        self.occ_white_noise_soup = o[3];
        self.occ_sterile_chaos = o[4];
        self.occ_exploding = o[5];
        self.occ_fragmented_isolation = o[6];
        self.occ_monoculture = o[7];
        self.occ_active_persistent_structure = o[8];
        self.occ_generative_candidate = o[9];
        self.occ_unknown = o[10];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id() -> RowIdentity {
        RowIdentity {
            generation: 1,
            member_idx: 2,
            antithetic: -1,
            sim_seed: 99,
            rollout_idx: 0,
            policy_rng_seed: 1234,
            genome_hash: 0xdead,
            genome_l2: 7.0,
            is_holdout: false,
            fitness_fn: "f2_regime_occupancy".into(),
            budget: Some(16),
            arm: Arm::Champion,
        }
    }

    #[test]
    fn a_failure_row_carries_no_fitness_and_says_why() {
        let r = CampaignRow::failed(
            id(),
            RolloutStatus::NoScoredWindows,
            "burn-in consumed every window".into(),
            12,
        );
        assert_eq!(r.status, "no_scored_windows");
        assert_eq!(r.fitness, None, "a failure must not be scored 0.0");
        assert_eq!(r.n_windows_scored, 0);
        assert!(r.error.is_some());
        // Identity survives so the row is still attributable.
        assert_eq!(r.member_idx, 2);
        assert_eq!(r.genome_hash, 0xdead);
    }

    #[test]
    fn occupancy_columns_follow_the_documented_index_order() {
        let mut o = [0u64; 11];
        for (i, slot) in o.iter_mut().enumerate() {
            *slot = i as u64;
        }
        let mut r = CampaignRow::failed(id(), RolloutStatus::Ok, String::new(), 0);
        r.set_occupancy(o);
        assert_eq!(r.occ_frozen, 0);
        assert_eq!(r.occ_exploding, 5);
        assert_eq!(r.occ_active_persistent_structure, 8);
        assert_eq!(r.occ_generative_candidate, 9);
        assert_eq!(r.occ_unknown, 10);
    }

    #[test]
    fn f2_reward_and_penalty_survive_as_separate_columns() {
        let fit = FitnessBreakdown {
            f1_mean_interestingness: 0.3,
            f2_reward: 0.8,
            f2_penalty: 0.25,
            f2: 0.55,
            f3_constructor_yield: 0.0,
            n_confirmed_constructors: 0,
            mean_fidelity: 0.0,
            n_windows_scored: 10,
            f4_monotone_drift: Reported::Unmeasured,
            f4_null_sd: 0.0,
            f4_gap_joins: 0,
            f4_degenerate_alphabet: false,
            f4_jsd_spread: 0.0,
            f5_constructor_reuse: Reported::Unmeasured,
            f5_top_share: 0.0,
            f5_max_cycles: 0,
            engagement: crate::fitness::Engagement::default(),
        };
        let r = CampaignRow::ok(
            id(),
            Ok(fit.f2),
            &fit,
            &HeldOut::default(),
            RolloutFlags::default(),
            [0; 11],
            7,
            64,
            5,
            0,
            0,
        );
        assert_eq!(r.f2_reward, Some(0.8));
        assert_eq!(r.f2_penalty, Some(0.25));
        assert_eq!(r.fitness, Some(0.55));
    }

    #[test]
    fn every_row_serializes_with_a_stable_header() {
        let r = CampaignRow::failed(id(), RolloutStatus::Ok, String::new(), 0);
        let mut w = csv::Writer::from_writer(vec![]);
        w.serialize(&r).expect("serialize");
        let out = String::from_utf8(w.into_inner().expect("inner")).expect("utf8");
        let header = out.lines().next().expect("header");
        assert!(header.starts_with("generation,member_idx,antithetic,sim_seed"));
        assert!(header.contains("f2_reward,f2_penalty"));
        assert!(header.contains("kernel_held"));
        assert!(header.contains("pairings_cap_bound_steps"));
    }
}
