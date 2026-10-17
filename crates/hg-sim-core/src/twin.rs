//! Twin-run counterfactual harness (LK3, plan §6): branch A (unperturbed)
//! vs branch B (one added or removed hyperedge) from a common state at
//! step T, run deterministically, damage measured with the
//! label-invariant ball-WL-profile metric from `hg_eval::twin`.
//!
//! Preconditions (v1 scope): MaxParallel with jitter-free rulesets only —
//! the RNG zero-draw guarantee is what makes two fresh engines
//! stream-equivalent to a continued run. Stochastic step modes are out of
//! scope (plan §11.4 / the F2 caveat); the harness refuses them.
//!
//! Placement deviation from the LK plan (documented): the plan put the
//! harness in hg-eval, but v4 §2.3 makes hg-sim-core the ONLY importer of
//! hg-engine, so the engine-driving part lives here and the engine-free
//! metric stays in hg-eval.

use crate::config::SimConfig;
use crate::runner::{build_initial_store, load_rules, SimError};
use hg_core::{HyperEdge, Label, NodeId};
use hg_engine::Engine;
use hg_eval::twin::{anchor_profile, distances_from, sample_anchors};
use hg_rules::StepMode;
use hg_schema::{ClaimType, TwinRow};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use smallvec::SmallVec;

#[derive(Clone, Copy, Debug)]
pub struct TwinParams {
    /// Anchors sampled from nodes alive at branch time (default 64).
    pub anchors: usize,
    /// Ball radius of the damage profile (default 3).
    pub anchor_radius: usize,
    /// Windows to run after branching.
    pub windows: u64,
    /// Steps per window.
    pub window_len: u64,
    /// Seed for anchor/perturbation sampling (independent of the run seed).
    pub sample_seed: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TwinPerturbation {
    /// Remove one sampled hyperedge.
    RemoveSampledEdge,
    /// Add one label-0 2-edge between two sampled distinct nodes.
    AddRandomEdge,
    /// No perturbation — the validity control; damage must be ≡ 0.
    Control,
}

#[derive(Clone, Debug)]
pub struct TwinOutcome {
    pub rows: Vec<TwinRow>,
    /// Perturbation site node(s) (empty for Control).
    pub site: Vec<NodeId>,
    /// Total anchors dropped over the run (deleted in either branch).
    pub total_attrition: u32,
    /// Damage was zero in every window (the Control expectation).
    pub damage_always_zero: bool,
    /// Final running slope of front_radius vs step (NaN if never fit).
    pub v_damage: f64,
}

/// Run a twin experiment defined by `config`, branching at `branch_step`.
pub fn run_twin(
    config: &SimConfig,
    branch_step: u64,
    params: &TwinParams,
    perturbation: TwinPerturbation,
) -> Result<TwinOutcome, SimError> {
    if config.engine.step_mode != StepMode::MaxParallel {
        return Err(SimError::Config(
            "twin harness requires MaxParallel (RNG zero-draw precondition; \
             stochastic step modes are out of scope in v1)"
                .into(),
        ));
    }

    // Base run to the branch point.
    let rules = load_rules(config)?;
    let mut base = build_initial_store(config);
    {
        let mut engine = Engine::new(
            rules.clone(),
            config.engine.step_mode,
            config.seed,
            config.engine.max_pairings_per_step,
        );
        for _ in 0..branch_step {
            engine.step(&mut base);
        }
    }

    // Branch B: apply the single perturbation.
    let mut store_b = base.clone();
    let mut rng = ChaCha8Rng::seed_from_u64(params.sample_seed);
    let site: Vec<NodeId> = match perturbation {
        TwinPerturbation::Control => Vec::new(),
        TwinPerturbation::RemoveSampledEdge => {
            let edges: Vec<_> = store_b.edges().map(|(id, e)| (id, e.nodes.to_vec())).collect();
            if edges.is_empty() {
                return Err(SimError::Config("no edge to remove at branch step".into()));
            }
            let (edge_id, endpoints) = edges[rng.gen_range(0..edges.len())].clone();
            store_b.remove_edge(edge_id);
            endpoints
        }
        TwinPerturbation::AddRandomEdge => {
            let ids: Vec<NodeId> = store_b.nodes().map(|(id, _)| id).collect();
            if ids.len() < 2 {
                return Err(SimError::Config("too few nodes to add an edge".into()));
            }
            let a = ids[rng.gen_range(0..ids.len())];
            let mut b = a;
            while b == a {
                b = ids[rng.gen_range(0..ids.len())];
            }
            store_b.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[a, b])));
            vec![a, b]
        }
    };
    let mut store_a = base;

    // Anchors + pre-perturbation distances, both in the branch-A state
    // at T (pre-existing NodeIds are valid in both branches).
    let mut anchors = sample_anchors(&store_a.snapshot(), params.anchors, params.sample_seed ^ 0xA0C4);
    let pre_dist = distances_from(&store_a.snapshot(), &site);

    // Fresh engines, identical seeds: stream-equivalent under the
    // zero-draw precondition checked above.
    let mut engine_a = Engine::new(
        rules.clone(),
        config.engine.step_mode,
        config.seed,
        config.engine.max_pairings_per_step,
    );
    let mut engine_b = Engine::new(
        rules,
        config.engine.step_mode,
        config.seed,
        config.engine.max_pairings_per_step,
    );

    let mut rows = Vec::with_capacity(params.windows as usize);
    let mut total_attrition = 0u32;
    let mut damage_always_zero = true;
    // (step, front_radius) points for the running slope.
    let mut front_pts: Vec<(f64, f64)> = Vec::new();
    let mut v_damage = f64::NAN;

    for w in 0..params.windows {
        for _ in 0..params.window_len {
            engine_a.step(&mut store_a);
            engine_b.step(&mut store_b);
        }
        let snap_a = store_a.snapshot();
        let snap_b = store_b.snapshot();

        // Attrition first: anchors deleted in EITHER branch drop out.
        let mut window_attrition = 0u32;
        anchors.retain(|&a| {
            let alive = snap_a.get_node(a).is_some() && snap_b.get_node(a).is_some();
            if !alive {
                window_attrition += 1;
            }
            alive
        });
        total_attrition += window_attrition;

        let mut damaged_dists: Vec<f64> = Vec::new();
        let mut damaged = 0usize;
        for &a in &anchors {
            let pa = anchor_profile(&snap_a, a, params.anchor_radius);
            let pb = anchor_profile(&snap_b, a, params.anchor_radius);
            if pa != pb {
                damaged += 1;
                if let Some(&d) = pre_dist.get(&a) {
                    damaged_dists.push(d as f64);
                }
            }
        }
        let damage_frac = if anchors.is_empty() {
            0.0
        } else {
            damaged as f32 / anchors.len() as f32
        };
        if damage_frac > 0.0 {
            damage_always_zero = false;
        }

        let step = branch_step + (w + 1) * params.window_len;
        let front_radius = percentile(&mut damaged_dists, 0.95);
        // Expansion-phase points only: on a CONVERGENT dynamic (e.g. the
        // excitation reference, where both branches end all-excited)
        // damage is transient and front_radius recedes after the front
        // passes; the damage-front speed is defined by the windows that
        // EXTEND the front. Windows after the peak measure recovery, not
        // propagation. (Measured: including them halves v_damage on the
        // reference dynamic — docs/fixture_calibration.md.)
        let running_max = front_pts.iter().map(|(_, y)| *y).fold(f64::MIN, f64::max);
        if front_radius.is_finite() && (front_pts.is_empty() || front_radius > running_max) {
            front_pts.push(((step - branch_step) as f64, front_radius));
        }
        // Running slope of front_radius vs step (through origin: damage
        // starts at the site at branch time).
        v_damage = if front_pts.is_empty() {
            f64::NAN
        } else {
            let sxy: f64 = front_pts.iter().map(|(x, y)| x * y).sum();
            let sxx: f64 = front_pts.iter().map(|(x, _)| x * x).sum();
            sxy / sxx
        };

        rows.push(TwinRow {
            window: w,
            step,
            damage_frac,
            front_radius: front_radius as f32,
            v_damage_running: v_damage as f32,
            anchor_attrition: window_attrition,
            claim_type: ClaimType::Estimated,
        });
    }

    Ok(TwinOutcome {
        rows,
        site,
        total_attrition,
        damage_always_zero,
        v_damage,
    })
}

fn percentile(vals: &mut [f64], q: f64) -> f64 {
    if vals.is_empty() {
        return f64::NAN;
    }
    vals.sort_by(|a, b| a.total_cmp(b));
    let rank = ((q * vals.len() as f64).ceil() as usize).clamp(1, vals.len());
    vals[rank - 1]
}
