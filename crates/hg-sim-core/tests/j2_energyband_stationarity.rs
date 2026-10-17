//! J2 — does the energy-band partition survive churn? (re-clearing the H0 TWTF
//! gate for the new partition, as the plan requires — the gate is partition-
//! specific and must NOT be assumed to transfer.)
//!
//! The (label, energy-band) partition recovered F8 detectability on static data
//! (`j2_churn_invariant_resolution.rs`: 0/200, support 2.2e9). The decisive
//! question for a churn-STABLE instrument: does it stay stationary under substrate
//! motion, where the WL partition failed (H0: TWTF 0.515 / 0.984)? Prediction:
//! energy bands are a BOUNDED recurrent value-set, so a fresh churned node falls
//! into an EXISTING band (band 0), not a new ephemeral class — unlike WL structural
//! hashes that proliferate. This measures WL-class TWTF and energy-band-class TWTF
//! head-to-head under the same motion builtins as H0.3.
#![cfg(feature = "alife")]

use hg_alife::CARRIER_LABEL;
use hg_core::{compute_wl_class_map_snapshot, Label, NodeId, PropertyValue, WlHash};
use hg_engine::Engine;
use hg_rules::{builtin_rules, compile_rules, parse_rules_toml, StepMode};
use hg_sim_core::runner::apply_alife_overrides;
use hg_sim_core::SimConfig;
use std::collections::HashMap;

const VACUUM: u64 = 0xE6_0000_0000_0000;
const WL_DEPTH: u8 = 2;
const R_PERSIST: u64 = 3; // same as H0.1
const NODE_CAP: usize = 25_000;

fn config(seed: u64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "j2eb"
run_id = "j2eb"
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
max_steps = 6000
metrics_window_interval = 128
max_pairings_per_step = 100000
[evaluator]
enabled = false
[causal]
enabled = true
[alife]
enabled = true
seed_carrier_marker = true
initial_substrate_energy = 5.0
influx_rate = 0.34
bootstrap_replication_threshold = 3.0
"#
    );
    SimConfig::from_toml(&toml).unwrap()
}

fn build_rules(cfg: &SimConfig, motion: Option<&str>) -> Vec<hg_rules::CompiledRule> {
    let mut raw = Vec::new();
    if let Some(name) = motion {
        raw.extend(builtin_rules(name).unwrap());
    }
    let mut biology = parse_rules_toml(hg_alife::alife_base_toml()).unwrap();
    apply_alife_overrides(&mut biology, &cfg.alife);
    raw.extend(biology);
    compile_rules(&raw).unwrap()
}

fn energy_band_class(label: i64, energy: f64) -> u64 {
    let band = energy.floor().clamp(-1.0, 64.0) as i64;
    ((label as u64) << 32) ^ (band as u64 & 0xFFFF_FFFF) ^ 0x9E37_79B9
}

/// TWTF over a chosen class function: fraction of transitions whose SOURCE class
/// is live in < R_PERSIST steps. Returns (M, TWTF, transitions).
struct Twtf { m: usize, twtf: f64, transitions: u64 }

fn twtf_for(
    class_live_steps: &HashMap<u64, u64>,
    src_trans: &HashMap<u64, u64>,
    m: usize,
) -> Twtf {
    let total: u64 = src_trans.values().sum();
    let transient: u64 = src_trans.iter()
        .filter(|(h, _)| *class_live_steps.get(*h).unwrap_or(&0) < R_PERSIST)
        .map(|(_, c)| *c).sum();
    Twtf { m, twtf: if total > 0 { transient as f64 / total as f64 } else { 0.0 }, transitions: total }
}

fn run(seed: u64, motion: Option<&str>, window: u64) -> (Twtf, Twtf, u64, usize) {
    let cfg = config(seed);
    let rules = build_rules(&cfg, motion);
    let mut store = hg_sim_core::runner::build_initial_store(&cfg);
    if motion == Some("growth_treadmill") {
        let first = store.nodes().next().map(|(id, _)| id);
        if let Some(id) = first { store.get_node_mut(id).unwrap().label = Label(1); }
    }
    let mut engine = Engine::new(rules, StepMode::MaxParallel, cfg.seed, 100_000);

    let mut wl_prev: Option<HashMap<NodeId, u64>> = None;
    let mut eb_prev: Option<HashMap<NodeId, u64>> = None;
    let mut wl_live: HashMap<u64, u64> = HashMap::new();
    let mut eb_live: HashMap<u64, u64> = HashMap::new();
    let mut wl_src: HashMap<u64, u64> = HashMap::new();
    let mut eb_src: HashMap<u64, u64> = HashMap::new();
    wl_live.insert(VACUUM, u64::MAX);
    eb_live.insert(VACUUM, u64::MAX);
    let mut steps = 0u64;
    let mut n_end = store.nodes().count();

    for step in 1..=window {
        engine.step(&mut store);
        let snap = store.snapshot();
        n_end = snap.nodes().count();
        let wl = compute_wl_class_map_snapshot(&snap, WL_DEPTH);
        let mut wl_now: HashMap<NodeId, u64> = HashMap::new();
        let mut eb_now: HashMap<NodeId, u64> = HashMap::new();
        // distinct classes present THIS step (liveness is per-step, not per-node —
        // increment each class's live-step count once, matching H0.1's definition).
        let mut wl_present: std::collections::HashSet<u64> = std::collections::HashSet::new();
        let mut eb_present: std::collections::HashSet<u64> = std::collections::HashSet::new();
        for (id, d) in snap.nodes() {
            let wh = wl.get(&id).copied().unwrap_or(WlHash(0)).0;
            wl_now.insert(id, wh);
            wl_present.insert(wh);
            let energy = match d.get_property(hg_alife::keys::ENERGY) {
                Some(PropertyValue::Float(v)) => v,
                _ => 0.0,
            };
            let eb = energy_band_class(d.label.0 as i64, energy);
            eb_now.insert(id, eb);
            eb_present.insert(eb);
        }
        for &h in &wl_present { *wl_live.entry(h).or_insert(0) += 1; }
        for &h in &eb_present { *eb_live.entry(h).or_insert(0) += 1; }
        if let Some(p) = &wl_prev {
            for (id, &hp) in p { *wl_src.entry(hp).or_insert(0) += 1; let _ = wl_now.get(id); }
            for (id, _) in &wl_now { if !p.contains_key(id) { *wl_src.entry(VACUUM).or_insert(0) += 1; } }
        }
        if let Some(p) = &eb_prev {
            for (id, &hp) in p { *eb_src.entry(hp).or_insert(0) += 1; let _ = eb_now.get(id); }
            for (id, _) in &eb_now { if !p.contains_key(id) { *eb_src.entry(VACUUM).or_insert(0) += 1; } }
        }
        wl_prev = Some(wl_now);
        eb_prev = Some(eb_now);
        steps = step;
        if n_end > NODE_CAP { break; }
    }

    let wl_m = wl_live.len().saturating_sub(1); // minus VACUUM sentinel
    let eb_m = eb_live.len().saturating_sub(1);
    (twtf_for(&wl_live, &wl_src, wl_m), twtf_for(&eb_live, &eb_src, eb_m), steps, n_end)
}

#[test]
fn j2_energyband_survives_churn() {
    let seed = 12345;
    let window = 120;
    eprintln!("\n================ J2 — ENERGY-BAND vs WL PARTITION STATIONARITY UNDER MOTION (seed {seed}, W={window}, R={R_PERSIST}) ================");
    eprintln!("  run                    | steps | n_end  | WL: M / TWTF        | energy-band: M / TWTF");
    eprintln!("  -----------------------+-------+--------+---------------------+----------------------");
    let mut eb_twtf_under_motion: Vec<f64> = Vec::new();
    let mut wl_twtf_under_motion: Vec<f64> = Vec::new();
    for (label, motion) in [
        ("static (baseline)", None),
        ("growth_treadmill", Some("growth_treadmill")),
        ("triangle_subdivide", Some("triangle_subdivide")),
    ] {
        let (wl, eb, steps, n_end) = run(seed, motion, window);
        eprintln!("  {:<22} | {:>5} | {:>6} | {:>3} / {:>6.3}        | {:>3} / {:>6.3}",
            label, steps, n_end, wl.m, wl.twtf, eb.m, eb.twtf);
        if motion.is_some() {
            wl_twtf_under_motion.push(wl.twtf);
            eb_twtf_under_motion.push(eb.twtf);
        }
    }
    eprintln!("=========================================================================================================\n");

    let eb_max = eb_twtf_under_motion.iter().cloned().fold(0.0f64, f64::max);
    let wl_max = wl_twtf_under_motion.iter().cloned().fold(0.0f64, f64::max);
    eprintln!("Re-clearing the H0 TWTF gate for the energy-band partition (STOP if TWTF majority >0.5, as committed in H0.1):");
    eprintln!("  under motion: WL TWTF max = {wl_max:.3} (H0: FAILED), energy-band TWTF max = {eb_max:.3}");
    if eb_max <= 0.5 {
        eprintln!("  => energy-band partition RE-CLEARS the TWTF gate under motion (bounded recurrent bands do not");
        eprintln!("     ephemeralise: a fresh churned node falls into an existing band). A churn-STABLE, F8-recovering");
        eprintln!("     instrument EXISTS. The sustained-turnover confound-split (H2) is instrumentable after all.");
    } else {
        eprintln!("  => energy-band ALSO ephemeralises under motion; no partition survives. STOP.");
    }
    eprintln!();
}
