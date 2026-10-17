//! G1 — structural probe for the sterile-token control (V3 / G-track).
//!
//! Plan: `validation_v3_sterile_token_plan.md`. This harness builds the sterile
//! token and reports ONLY structural quantities — WL class count, group sizes,
//! transition count — and computes NO CE and NO permutation null. That separation
//! is deliberate: G1.2/G1.3 permit choosing the bootstrap-bloom size to reach
//! (a) >1 carrier WL class and (b) enough transitions, but the choice must NOT be
//! driven by the token's CE or p-value (rule 1). Keeping this file CE-free makes
//! that guarantee structural: there is no coefficient here to tune toward.
//!
//! The token is "the real carrier's full rule set MINUS the replicate rule"
//! (G1.1), built purely by in-test setup — the SCORING pipeline (WL depth, TPM,
//! null) is untouched and lives in the G3 harness, reused verbatim from F8 (rule 3).
//!  - replicate removed: filter it out of the compiled rule set.
//!  - bootstrap bloom: plant N carrier_markers with the SAME deterministic picker
//!    the production single-marker path uses, so N=1 reproduces F8's marker exactly.
#![cfg(feature = "alife")]

use hg_alife::CARRIER_LABEL;
use hg_causal::TpmAccumulator;
use hg_core::{compute_wl_class_map_snapshot, HypergraphStore, NodeId, PropertyValue, WlHash};
use hg_engine::Engine;
use hg_rules::StepMode;
use hg_sim_core::SimConfig;
use std::collections::HashMap;

const VACUUM: WlHash = WlHash(0xE6_0000_0000_0000);
const STEPS: u64 = 6_000; // identical to F8
const WL_DEPTH: u8 = 2; // F1 pre-registration; re-selecting is forbidden

/// F8's config verbatim, but marker-seeding is disabled here: this harness plants
/// its own markers so the carrier arm (n=1) and the token arm (n=N) differ ONLY in
/// marker count and the presence of the replicate rule.
fn config(seed: u64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "g_probe"
run_id = "g_probe"
seed = {seed}
[rules]
source = "Inline"
inline_toml = "rules = []"
[initial]
condition_type = "PureCrystal"
width = 6
height = 6
arity = 3
[engine]
step_mode = "MaxParallel"
max_steps = {STEPS}
metrics_window_interval = 128
max_pairings_per_step = 100000
[evaluator]
enabled = false
[causal]
enabled = true
[alife]
enabled = true
seed_carrier_marker = false
initial_substrate_energy = 5.0
influx_rate = 0.34
bootstrap_replication_threshold = 3.0
"#
    );
    SimConfig::from_toml(&toml).unwrap()
}

/// Plant `n` distinct carrier_markers, deterministic from `seed`. The FIRST pick is
/// byte-identical to `runner::seed_one_carrier_marker` (same constant, same id
/// order, same ChaCha8 stream) so n=1 reproduces the production single marker.
fn plant_markers(store: &mut HypergraphStore, seed: u64, n: usize) {
    use rand::{Rng, SeedableRng};
    let ids: Vec<NodeId> = store.nodes().map(|(id, _)| id).collect();
    if ids.is_empty() {
        return;
    }
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed ^ 0x00CA_111E);
    let mut planted = 0usize;
    let mut used = std::collections::HashSet::new();
    let cap = n.min(ids.len());
    while planted < cap {
        let pick = ids[rng.gen_range(0..ids.len())];
        if used.insert(pick) {
            if let Some(d) = store.get_node_mut(pick) {
                d.set_property(hg_alife::keys::CARRIER_MARKER, PropertyValue::Bool(true));
                planted += 1;
            }
        }
    }
}

struct Structural {
    m: usize,
    k_carrier: usize,
    transitions: u64,
    /// live carrier count sampled at each of a few checkpoints (bloom shape).
    trajectory: Vec<(u64, usize)>,
    peak_carriers: usize,
    /// occupant node-count per carrier WL class, at the step of peak population.
    peak_class_sizes: Vec<usize>,
}

/// Build the streaming vacuum-augmented TPM at depth 2 EXACTLY as F8 does, plus
/// record structural trajectory. Computes no CE and no null.
fn run_structural(seed: u64, sterile: bool, n_markers: usize) -> Structural {
    let cfg = config(seed);
    let mut rules = hg_sim_core::runner::load_rules(&cfg).unwrap();
    if sterile {
        rules.retain(|r| r.name != "replicate");
    }
    let mut store = hg_sim_core::runner::build_initial_store(&cfg);
    plant_markers(&mut store, seed, n_markers);
    let mut engine = Engine::new(rules, StepMode::MaxParallel, cfg.seed, 100_000);

    let mut tpm = TpmAccumulator::new();
    let mut is_carrier_class: HashMap<u64, bool> = HashMap::new();
    is_carrier_class.insert(VACUUM.0, false);
    let mut prev: Option<HashMap<NodeId, WlHash>> = None;

    let mut trajectory = Vec::new();
    let mut peak_carriers = 0usize;
    let mut peak_classes: HashMap<u64, usize> = HashMap::new();

    for step in 1..=STEPS {
        engine.step(&mut store);
        let snap = store.snapshot();
        let classes = compute_wl_class_map_snapshot(&snap, WL_DEPTH);
        let mut now: HashMap<NodeId, WlHash> = HashMap::new();
        let mut live_carriers = 0usize;
        let mut this_classes: HashMap<u64, usize> = HashMap::new();
        for (id, data) in snap.nodes() {
            let h = classes.get(&id).copied().unwrap_or(WlHash(0));
            let is_c = data.label == CARRIER_LABEL;
            if is_c {
                live_carriers += 1;
                *this_classes.entry(h.0).or_insert(0) += 1;
            }
            now.insert(id, h);
            is_carrier_class.entry(h.0).or_insert(is_c);
        }
        if live_carriers > peak_carriers {
            peak_carriers = live_carriers;
            peak_classes = this_classes;
        }
        if let Some(p) = &prev {
            for (id, &hp) in p {
                match now.get(id) {
                    Some(&hn) => tpm.record(hp, hn),
                    None => tpm.record(hp, VACUUM),
                }
            }
            for (id, &hn) in &now {
                if !p.contains_key(id) {
                    tpm.record(VACUUM, hn);
                }
            }
        }
        prev = Some(now);
        if step == 1 || step == 5 || step == 20 || step == 50 || step == 200 || step == STEPS {
            trajectory.push((step, live_carriers));
        }
    }

    let m = tpm.hashes().len();
    let k_carrier = tpm
        .hashes()
        .iter()
        .filter(|h| *is_carrier_class.get(&h.0).unwrap_or(&false))
        .count();

    let mut peak_class_sizes: Vec<usize> = peak_classes.values().copied().collect();
    peak_class_sizes.sort_unstable_by(|a, b| b.cmp(a));

    Structural {
        m,
        k_carrier,
        transitions: tpm.total_transitions(),
        trajectory,
        peak_carriers,
        peak_class_sizes,
    }
}

#[test]
fn g1_structural_probe() {
    // FIDELITY ANCHOR (rule 3 / G0): the carrier arm through THIS harness's setup —
    // planted single marker + full rule set — must reproduce F8's structural facts
    // at seed 12345 (M=200, k_carrier=4). If planting-1 diverged from the production
    // auto-marker, nothing the token arm reports would be comparable.
    let anchor = run_structural(12345, false, 1);
    eprintln!(
        "\nFIDELITY ANCHOR seed 12345 carrier (planted 1 marker): M={} k_carrier={} transitions={}  (F8: M=200, k=4)",
        anchor.m, anchor.k_carrier, anchor.transitions
    );
    assert_eq!(anchor.m, 200, "planted-1 carrier arm diverged from F8 M");
    assert_eq!(anchor.k_carrier, 4, "planted-1 carrier arm diverged from F8 k_carrier");

    // Token structure across candidate bootstrap-bloom sizes. This is a MATCHING
    // sweep (G1.2/G1.3): find the smallest bloom that clears >1 carrier class and
    // the ~220k transition power bar. No CE is computed here, so the pick cannot be
    // outcome-driven. Reported for the G2 pre-registration.
    eprintln!("\n================ G1: STERILE-TOKEN STRUCTURE (seed 12345, WL depth {WL_DEPTH}) ================");
    eprintln!("(carrier arm, single marker + replication, for reference:)");
    eprintln!("   carrier: M={} k_carrier={} transitions={} peak_carriers={}",
        anchor.m, anchor.k_carrier, anchor.transitions, anchor.peak_carriers);
    eprintln!("\n  n_markers | M   | k_tok | transitions | peak_pop | peak carrier-class sizes | trajectory (step:pop)");
    eprintln!("  ----------+-----+-------+-------------+----------+--------------------------+----------------------");
    for &n in &[1usize, 4, 9, 16, 25, 36] {
        let s = run_structural(12345, true, n);
        let traj: String = s
            .trajectory
            .iter()
            .map(|(st, p)| format!("{st}:{p}"))
            .collect::<Vec<_>>()
            .join(" ");
        eprintln!(
            "  {:>9} | {:>3} | {:>5} | {:>11} | {:>8} | {:<24} | {}",
            n,
            s.m,
            s.k_carrier,
            s.transitions,
            s.peak_carriers,
            format!("{:?}", s.peak_class_sizes),
            traj
        );
    }
    eprintln!("======================================================================================\n");
    eprintln!("Power bar (F8 extinct-seed floor): ~220k carrier/substrate transitions.");
    eprintln!("Gate G1.2: token carrier-class count > 1.  Gate G1.3: transitions in F8-power range.");
}
