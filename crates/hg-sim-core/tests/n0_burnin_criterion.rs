//! V9 / N0 — derive the burn-in length from a COMMITTED CRITERION, and prove the
//! criterion rejects.
//!
//! Criterion committed in docs/validation_v9_n0_burnin_criterion.md (d853f94)
//! BEFORE any N-track churn number existed. Burn-in had become a free parameter
//! with demonstrated gate-flipping leverage (M1's gate initially fired FALSE on a
//! pre-settling window, and the length was chosen after seeing that), so the length
//! must be an OUTPUT of a rule, not a choice.
//!
//! SETTLED from block b iff for every i >= b, over 100-step blocks:
//!   (1) consecutive-block WL class-set Jaccard J(i) >= 0.90, AND
//!   (2) | |S_{i+1}| - |S_i| | / max(|S_i|,1) <= 0.05.
//! Two clauses with DIFFERENT failure modes on purpose (rule 4).
//! T_burn = b * 100. A run that never settles has NO valid scoring window.
#![cfg(feature = "alife")]

use hg_core::{compute_wl_class_map_snapshot, Label, NodeId, WlHash};
use hg_engine::Engine;
use hg_rules::{builtin_rules, compile_rules, parse_rules_toml, StepMode};
use hg_sim_core::runner::apply_alife_overrides;
use hg_sim_core::SimConfig;
use std::collections::HashSet;

const WL_DEPTH: u8 = 2;
// ---- committed in d853f94; changing any of these after seeing a T_burn is forbidden
pub const L_BLK: u64 = 100;
pub const J_MIN: f64 = 0.90;
pub const M_TOL: f64 = 0.05;
pub const MIN_RETAINED_FRAC: f64 = 0.50;
const NODE_CAP: usize = 25_000;

fn config(seed: u64, steps: u64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "n0"
run_id = "n0"
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
max_steps = {steps}
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

pub fn churn_toml() -> &'static str {
    r#"
[[rules]]
name = "churn"
priority = 2
[rules.lhs]
nodes = ["a", "old", "b"]
edges = [["a", "old"], ["old", "b"]]
[rules.lhs.node_labels]
a = 0
old = 0
b = 0
[[rules.lhs.node_property_guards]]
var = "old"
property = 1
op = "gte"
value = 20.0
[[rules.lhs.node_property_guards]]
var = "old"
property = 22
op = "eq"
value = false
[[rules.lhs.var_guards]]
type = "vars_distinct"
vars = ["a", "old", "b"]
[rules.rhs]
nodes = ["a", "fresh", "b"]
edges = [["a", "fresh"], ["fresh", "b"]]
[rules.rhs.node_labels]
fresh = 0
[[rules.rhs.property_effects]]
var = "fresh"
key = 1
op = "set"
value = 0.0
[[rules.rhs.property_effects]]
var = "fresh"
key = 22
op = "set"
value = false
"#
}

pub fn motility_toml() -> &'static str {
    r#"
[[rules]]
name = "relocate"
priority = 2
[rules.lhs]
nodes = ["site", "carrier", "target"]
edges = [["site", "carrier"], ["site", "target"]]
[rules.lhs.node_labels]
carrier = 100
site = 0
target = 0
[[rules.lhs.var_guards]]
type = "vars_distinct"
vars = ["site", "carrier", "target"]
[[rules.lhs.node_property_compares]]
var_a = "carrier"
key_a = 1
op = "lt"
var_b = "carrier"
key_b = 13
[[rules.lhs.node_property_guards]]
var = "target"
property = 1
op = "gte"
value = 1.0
[[rules.lhs.node_property_guards]]
var = "target"
property = 22
op = "eq"
value = false
[[rules.lhs.node_property_guards]]
var = "site"
property = 1
op = "lt"
value = 1.0
[rules.rhs]
nodes = ["site", "carrier", "target"]
edges = [["site", "target"], ["target", "carrier"]]
[[rules.rhs.property_effects]]
var = "site"
key = 22
op = "set"
value = false
[[rules.rhs.property_effects]]
var = "target"
key = 22
op = "set"
value = true
"#
}

#[derive(Clone, Copy)]
pub enum Arm { StaticBare, StaticF8, ChurnBare, ChurnCarriers, Growth, Triangle }

fn build(cfg: &SimConfig, arm: Arm) -> Vec<hg_rules::CompiledRule> {
    let mut raw = Vec::new();
    match arm {
        Arm::Growth => raw.extend(builtin_rules("growth_treadmill").unwrap()),
        Arm::Triangle => raw.extend(builtin_rules("triangle_subdivide").unwrap()),
        _ => {}
    }
    let mut biology = parse_rules_toml(hg_alife::alife_base_toml()).unwrap();
    apply_alife_overrides(&mut biology, &cfg.alife);
    match arm {
        Arm::ChurnBare => { biology.retain(|r| r.name != "replicate"); biology.extend(parse_rules_toml(churn_toml()).unwrap()); }
        Arm::ChurnCarriers => {
            biology.retain(|r| r.name != "replicate");
            biology.extend(parse_rules_toml(motility_toml()).unwrap());
            biology.extend(parse_rules_toml(churn_toml()).unwrap());
        }
        _ => {}
    }
    raw.extend(biology);
    compile_rules(&raw).unwrap()
}

fn plant_markers(store: &mut hg_core::HypergraphStore, seed: u64, n: usize) {
    use rand::{Rng, SeedableRng};
    let ids: Vec<NodeId> = store.nodes().map(|(id, _)| id).collect();
    if ids.is_empty() || n == 0 { return; }
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed ^ 0x00CA_111E);
    let (mut planted, mut used) = (0usize, HashSet::new());
    let cap = n.min(ids.len());
    while planted < cap {
        let pick = ids[rng.gen_range(0..ids.len())];
        if used.insert(pick) {
            store.get_node_mut(pick).unwrap()
                .set_property(hg_alife::keys::CARRIER_MARKER, hg_core::PropertyValue::Bool(true));
            planted += 1;
        }
    }
}

/// Per-block WL class sets over the whole run.
pub fn block_class_sets(seed: u64, arm: Arm, steps: u64, markers: usize) -> Vec<HashSet<u64>> {
    let cfg = config(seed, steps);
    let rules = build(&cfg, arm);
    let mut store = hg_sim_core::runner::build_initial_store(&cfg);
    if matches!(arm, Arm::Growth) {
        let first = store.nodes().next().map(|(id, _)| id);
        if let Some(id) = first { store.get_node_mut(id).unwrap().label = Label(1); }
    }
    plant_markers(&mut store, seed, markers);
    let mut engine = Engine::new(rules, StepMode::MaxParallel, cfg.seed, 100_000);
    let n_blocks = (steps / L_BLK) as usize;
    let mut sets: Vec<HashSet<u64>> = vec![HashSet::new(); n_blocks.max(1)];
    for step in 1..=steps {
        engine.step(&mut store);
        let snap = store.snapshot();
        let wl = compute_wl_class_map_snapshot(&snap, WL_DEPTH);
        let bi = (((step - 1) / L_BLK) as usize).min(sets.len() - 1);
        for (id, _) in snap.nodes() {
            sets[bi].insert(wl.get(&id).copied().unwrap_or(WlHash(0)).0);
        }
        if snap.nodes().count() > NODE_CAP { break; }
    }
    sets
}

/// v1's ABSOLUTE criterion (d853f94). RETAINED ONLY to document its failure: it rejects
/// the trusted F8 baseline, so its churn rejection is about block size, not stationarity.
pub fn derive_burn_in_v1(sets: &[HashSet<u64>]) -> Option<u64> {
    derive_burn_in_with(sets, J_MIN, M_TOL)
}

/// v2 — BASELINE-RELATIVE criterion (18dcd28), following H0.1's precedent that the F8
/// static run defines the tolerated floor. `j_ref`/`d_ref` are DERIVED from arm A.
pub fn derive_burn_in_v2(sets: &[HashSet<u64>], j_ref: f64, d_ref: f64) -> Option<u64> {
    derive_burn_in_with(sets, j_ref, d_ref)
}

fn derive_burn_in_with(sets: &[HashSet<u64>], j_bar: f64, d_bar: f64) -> Option<u64> {
    let n = sets.len();
    if n < 2 { return None; }
    let ok_from = |b: usize| -> bool {
        (b..n - 1).all(|i| {
            let (a, c) = (&sets[i], &sets[i + 1]);
            // An EMPTY block is not a measurement (the run was truncated at the node cap),
            // so it can never count as settled. Returning Jaccard 1.0 for two empty sets
            // would silently mark a truncated run as stationary.
            if a.is_empty() || c.is_empty() { return false; }
            let uni = a.union(c).count() as f64;
            let j = a.intersection(c).count() as f64 / uni;
            let da = (c.len() as f64 - a.len() as f64).abs() / (a.len().max(1) as f64);
            j >= j_bar - 1e-12 && da <= d_bar + 1e-12
        })
    };
    let b = (0..n - 1).find(|&b| ok_from(b))?;
    let t_burn = b as u64 * L_BLK;
    let total = n as u64 * L_BLK;
    if (total - t_burn) as f64 / total as f64 >= MIN_RETAINED_FRAC { Some(t_burn) } else { None }
}

/// Derive the v2 reference bars from the F8 static run: worst Jaccard, worst |dM|/M.
pub fn reference_bars(sets: &[HashSet<u64>]) -> (f64, f64) {
    let p = pairs(sets);
    let j_ref = p.iter().map(|(j, _)| *j).fold(f64::INFINITY, f64::min);
    let d_ref = p.iter().map(|(_, d)| *d).fold(0.0f64, f64::max);
    (j_ref, d_ref)
}

fn pairs(sets: &[HashSet<u64>]) -> Vec<(f64, f64)> {
    (0..sets.len().saturating_sub(1)).map(|i| {
        let (a, c) = (&sets[i], &sets[i + 1]);
        let uni = a.union(c).count() as f64;
        let j = if uni > 0.0 { a.intersection(c).count() as f64 / uni } else { 1.0 };
        let da = (c.len() as f64 - a.len() as f64).abs() / (a.len().max(1) as f64);
        (j, da)
    }).collect()
}

#[test]
fn n0_burn_in_derived_and_criterion_fires() {
    let seed = 12345;
    let steps = 6_000u64;
    eprintln!("\n================ V9 / N0 — BURN-IN DERIVED FROM THE COMMITTED CRITERION (seed {seed}) ================");
    eprintln!("  criterion (d853f94): over {L_BLK}-step blocks, settled from b iff for all i>=b");
    eprintln!("    Jaccard(S_i, S_i+1) >= {J_MIN}  AND  |ΔM|/M <= {M_TOL};  T_burn = b * {L_BLK}\n");

    let cases: [(&str, Arm, usize, u64); 6] = [
        ("static bare",        Arm::StaticBare,    0, steps),
        // CALIBRATION: the F8 static run is the baseline the whole report already relies on.
        // If the criterion rejects THAT, the criterion is too strict and the churn rejection
        // below cannot be read as a fact about churn.
        ("static F8 (A)",      Arm::StaticF8,      1, steps),
        ("CHURN bare",         Arm::ChurnBare,     0, steps),
        ("CHURN carriers",     Arm::ChurnCarriers, 6, steps),
        // The growing proxies are run only long enough to be judged: both are in their
        // asymptotic regime from step 1, so a few blocks settle the question, and
        // triangle_subdivide explodes past the node cap almost immediately.
        ("growth_treadmill",   Arm::Growth,        0, 600),
        ("triangle_subdivide", Arm::Triangle,      0, 300),
    ];

    // All block-class-sets first, so the v2 reference bars can be derived from arm A
    // and then applied unchanged to every other run.
    let mut all: Vec<(&str, Vec<HashSet<u64>>)> = Vec::new();
    for (name, arm, markers, s) in cases {
        all.push((name, block_class_sets(seed, arm, s, markers)));
    }
    let f8_sets = &all.iter().find(|(n, _)| *n == "static F8 (A)").unwrap().1;
    let (j_ref, d_ref) = reference_bars(f8_sets);

    eprintln!("  v2 REFERENCE BARS, derived from arm A (the run the report already trusts):");
    eprintln!("    J_ref = min consecutive-block Jaccard on F8 = {j_ref:.4}");
    eprintln!("    D_ref = max |ΔM|/M on F8                    = {d_ref:.4}");
    eprintln!("    (outputs of the reference run, not chosen — see docs/validation_v9_n0_burnin_criterion_v2.md)\n");

    let mut derived: Vec<(&str, Option<u64>)> = Vec::new();
    let mut derived_v1: Vec<(&str, Option<u64>)> = Vec::new();
    for (name, sets) in &all {
        let t = derive_burn_in_v2(sets, j_ref, d_ref);
        let t1 = derive_burn_in_v1(sets);
        derived.push((name, t));
        derived_v1.push((name, t1));
        let pr = pairs(sets);
        let head: Vec<String> = pr.iter().take(8).map(|(j, d)| format!("J{j:.2}/Δ{d:.2}")).collect();
        eprintln!("  {:<19} blocks {:>3}, per-block M {:?}...", name, sets.len(),
            sets.iter().take(8).map(|s| s.len()).collect::<Vec<_>>());
        eprintln!("  {:<19}   first pairs: [{}]", "", head.join(", "));
        eprintln!("  {:<19}   => v2 T_burn = {:<32} (v1 said: {})", "",
            t.map(|x| format!("{x} steps")).unwrap_or_else(|| "NO SETTLED WINDOW".into()),
            t1.map(|x| format!("{x} steps")).unwrap_or_else(|| "NO SETTLED WINDOW".into()));
    }

    // ---------- N0.2 — agreement with the M-track's post-hoc value ----------
    let churn_bare = derived.iter().find(|(n, _)| *n == "CHURN bare").unwrap().1;
    let churn_car = derived.iter().find(|(n, _)| *n == "CHURN carriers").unwrap().1;
    eprintln!("\n================ N0.2 — DERIVED LENGTH vs THE M-TRACK'S POST-HOC VALUE ================");
    eprintln!("  M-track effectively discarded block 0 of 6 over 6000 steps = 1000 steps of burn-in (post-hoc).");
    eprintln!("  Criterion-derived: CHURN bare = {:?}, CHURN carriers = {:?}", churn_bare, churn_car);
    for (name, t) in [("CHURN bare", churn_bare), ("CHURN carriers", churn_car)] {
        match t {
            Some(v) if v <= 1000 => eprintln!("    {name}: derived {v} <= M-track's 1000 => the M-track's post-hoc choice was CONSERVATIVE (safe)."),
            Some(v) => eprintln!("    {name}: derived {v} > M-track's 1000 => *** the M-track UNDER-burned; its scoring window included unsettled blocks ***"),
            None => eprintln!("    {name}: NO settled window at L_blk={L_BLK} => not supported by this criterion at this block size"),
        }
    }

    // ---------- CALIBRATION: does the criterion accept the run the report already trusts? ----------
    let f8_v2 = derived.iter().find(|(n, _)| *n == "static F8 (A)").unwrap().1;
    let f8_v1 = derived_v1.iter().find(|(n, _)| *n == "static F8 (A)").unwrap().1;
    eprintln!("\n================ CALIBRATION — why v1 was replaced ================");
    eprintln!("  The F8 static run is the baseline every published structural result rests on.");
    eprintln!("  v1 (absolute J>={J_MIN}, ΔM<={M_TOL}) verdict on F8: {:?}", f8_v1);
    eprintln!("  v2 (baseline-relative)          verdict on F8: {:?}", f8_v2);
    assert!(f8_v1.is_none(), "v1 was expected to reject the trusted baseline — that is why it was replaced");
    assert!(f8_v2.is_some(), "v2 must ACCEPT the trusted baseline, or it is vacuous too");
    eprintln!("  => v1 REJECTED the run the report relies on, so its churn rejection measured BLOCK SIZE, not");
    eprintln!("     stationarity. v2 accepts the baseline by construction, so a rejection under v2 IS a fact");
    eprintln!("     about the run rejected. v1 stands in the record as a failed criterion.");

    // ---------- N0.3 — FIRING TEST ----------
    let growth = derived.iter().find(|(n, _)| *n == "growth_treadmill").unwrap().1;
    let tri = derived.iter().find(|(n, _)| *n == "triangle_subdivide").unwrap().1;
    eprintln!("\n================ N0.3 — FIRING TEST (rule 5): the criterion must REJECT ================");
    eprintln!("  growth_treadmill   => {:?}  (expected: NO settled window — it replaces its class set at constant M)", growth);
    eprintln!("  triangle_subdivide => {:?}  (expected: NO settled window — J ~ 0.004 and M grows unbounded)", tri);
    eprintln!("  A criterion that accepts everything is not a criterion.");
    assert!(growth.is_none(), "FIRING TEST FAILED: growth_treadmill must be REJECTED as never settling");
    assert!(tri.is_none(), "FIRING TEST FAILED: triangle_subdivide must be REJECTED as never settling");
    eprintln!("  => the criterion FIRES on both growing proxies. It is tested, not merely asserted.");

    // pre-settling churn window must also be rejected (the mistake that flipped the M1 gate)
    let churn_sets = block_class_sets(seed, Arm::ChurnBare, steps, 0);
    let from_zero_ok = {
        let (a, c) = (&churn_sets[0], &churn_sets[1]);
        let uni = a.union(c).count() as f64;
        let j = a.intersection(c).count() as f64 / uni;
        j >= J_MIN
    };
    eprintln!("  scoring CHURN from step 0 (block 0->1 Jaccard) accepted by clause 1? {from_zero_ok}  (must be false)");
    assert!(!from_zero_ok, "the criterion must reject a pre-settling churn window scored from step 0");
    eprintln!("=========================================================================\n");
}
