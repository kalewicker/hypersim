//! V9 / N4.3 — is the ~0% retention result a property of THIS substrate, or of
//! structural coarse-graining generally?
//!
//! M3 measured, on the STATIC substrate, that the structural (WL) description
//! retains ~0% of the metabolic channel's information:
//!   H(micro energy | macro WL) / H(micro energy) = 1.0000 for arms C and D,
//!   0.9986 for arm A.
//! That is the "shadow" result. It was never measured on a MOVING substrate, and
//! whether it survives there bears directly on how general the claim is.
//!
//! RULE 2 (binding): this is CLASS-LEVEL INFORMATION RETENTION, never causal
//! independence. The channels are causally coupled by construction — energy
//! crossing zero CAUSES death, a structural event. WL classes pool carriers at
//! different energies, so strong individual-level coupling washes out at class
//! level. No wording here implies independence.
//!
//! Burn-in on churn is the N0 v2 DERIVED value (18dcd28), not a chosen one.
#![cfg(feature = "alife")]

use hg_alife::CARRIER_LABEL;
use hg_core::{compute_wl_class_map_snapshot, HypergraphStore, NodeId, PropertyValue, WlHash};
use hg_engine::Engine;
use hg_rules::{compile_rules, parse_rules_toml, StepMode};
use hg_sim_core::runner::apply_alife_overrides;
use hg_sim_core::SimConfig;
use std::collections::{HashMap, HashSet};

const ENERGY: u32 = 1;
const EPS: f64 = 1e-9;
const WL_DEPTH: u8 = 2;
const STEPS: u64 = 6_000;
const T_BURN_CHURN: u64 = 900; // N0 v2 derived, churn + carriers

fn config(seed: u64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "n43"
run_id = "n43"
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

fn motility_toml() -> &'static str { include_str!("n_shared/relocate.toml") }
fn diffuse_toml() -> &'static str { include_str!("n_shared/diffuse.toml") }
fn churn_toml() -> &'static str { include_str!("n_shared/churn.toml") }

fn plant_markers(store: &mut HypergraphStore, seed: u64, n: usize) {
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
                .set_property(hg_alife::keys::CARRIER_MARKER, PropertyValue::Bool(true));
            planted += 1;
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Arm { A, B, D }

fn build(cfg: &SimConfig, arm: Arm, churn: bool) -> Vec<hg_rules::CompiledRule> {
    let mut b = parse_rules_toml(hg_alife::alife_base_toml()).unwrap();
    apply_alife_overrides(&mut b, &cfg.alife);
    match arm {
        Arm::A => {}
        Arm::B => { b.retain(|r| r.name != "replicate"); b.extend(parse_rules_toml(motility_toml()).unwrap()); }
        Arm::D => { b.retain(|r| r.name != "replicate"); b.extend(parse_rules_toml(diffuse_toml()).unwrap()); }
    }
    if churn { b.extend(parse_rules_toml(churn_toml()).unwrap()); }
    compile_rules(&b).unwrap()
}

/// C3.1: terms summed in ascending KEY order. `-m.values().map(..).sum()` iterates a
/// `HashMap`, whose order `RandomState` seeds per PROCESS; float addition is
/// non-associative, so that form differs in the last ulps between processes even when
/// recording is sorted. Measured at n = 3 in c3_closure_defect_order_gate.rs.
fn h<K: Ord + Copy + std::hash::Hash>(m: &HashMap<K, u64>) -> f64 {
    let tot: u64 = m.values().sum(); // integer sum: exact and order-independent
    if tot == 0 { return 0.0; }
    let mut ks: Vec<K> = m.keys().copied().collect();
    ks.sort_unstable();
    -ks.iter().map(|k| { let p = m[k] as f64 / tot as f64; if p > 0.0 { p * p.log2() } else { 0.0 } }).sum::<f64>()
}

struct Ret { h_micro: f64, h_macro: f64, cond: f64, retention: f64, carrier_steps: u64, macro_classes: usize }

/// H(micro energy transition | macro WL class transition), over CARRIER nodes.
fn retention(seed: u64, arm: Arm, churn: bool, burn_in: u64) -> Ret {
    let cfg = config(seed);
    let rules = build(&cfg, arm, churn);
    let mut store = hg_sim_core::runner::build_initial_store(&cfg);
    let markers = if arm == Arm::A { 1 } else { 6 };
    plant_markers(&mut store, seed, markers);
    let mut engine = Engine::new(rules, StepMode::MaxParallel, cfg.seed, 100_000);

    let mut prev_e: HashMap<NodeId, f64> = HashMap::new();
    let mut prev_wl: HashMap<NodeId, u64> = HashMap::new();
    let mut joint: HashMap<(u64, u64), u64> = HashMap::new();
    let mut macro_c: HashMap<u64, u64> = HashMap::new();
    let mut micro_c: HashMap<u64, u64> = HashMap::new();
    let mut carrier_steps = 0u64;

    for step in 1..=STEPS {
        engine.step(&mut store);
        let snap = store.snapshot();
        let wl = compute_wl_class_map_snapshot(&snap, WL_DEPTH);
        let mut now_wl: HashMap<NodeId, u64> = HashMap::new();
        let mut rows: Vec<(NodeId, u64, u64)> = Vec::new(); // sorted recording (N1.3)
        for (id, d) in snap.nodes() {
            let wh = wl.get(&id).copied().unwrap_or(WlHash(0)).0;
            now_wl.insert(id, wh);
            let e = match d.get_property(ENERGY) { Some(PropertyValue::Float(v)) => v, _ => 0.0 };
            let delta = e - prev_e.get(&id).copied().unwrap_or(e);
            if d.label == CARRIER_LABEL {
                let s = if delta.abs() < EPS { 1 } else if delta < 0.0 { 0 } else { 2 };
                if let Some(&pw) = prev_wl.get(&id) { rows.push((id, pw ^ wh.rotate_left(1), s)); }
            }
            prev_e.insert(id, e);
        }
        if step > burn_in {
            rows.sort_unstable();
            for (_, mt, s) in rows {
                carrier_steps += 1;
                *joint.entry((mt, s)).or_insert(0) += 1;
                *macro_c.entry(mt).or_insert(0) += 1;
                *micro_c.entry(s).or_insert(0) += 1;
            }
        }
        prev_wl = now_wl;
    }
    // C3.1: the previous form re-keyed by HashMap ENUMERATION POSITION, so the keys
    // themselves were per-process. Entropy over the real (macro, micro) keys is the same
    // number and is deterministically orderable.
    let (hm, hM) = (h(&micro_c), h(&macro_c));
    let cond = (h(&joint) - hM).max(0.0);
    Ret { h_micro: hm, h_macro: hM, cond,
        retention: if hm > 1e-12 { 1.0 - cond / hm } else { f64::NAN },
        carrier_steps, macro_classes: macro_c.len() }
}

#[test]
fn n4_3_retention_static_vs_churn() {
    let seed = 12345;
    eprintln!("\n================ V9 / N4.3 — RETENTION OF THE METABOLIC CHANNEL BY THE STRUCTURAL ONE ================");
    eprintln!("  retention = 1 - H(micro energy | macro WL) / H(micro energy), over carrier nodes.");
    eprintln!("  0% retention = the structural CLASS description carries ~none of the metabolic channel's");
    eprintln!("  information AT CLASS LEVEL. This is NOT causal independence (rule 2): the channels are coupled");
    eprintln!("  by construction — energy crossing zero CAUSES death, a structural event — but WL classes pool");
    eprintln!("  carriers at different energies, so individual-level coupling washes out at class level.\n");
    eprintln!("  churn burn-in is the N0 v2 DERIVED value ({T_BURN_CHURN}), not a chosen one.\n");
    eprintln!("  arm | substrate | carrier-steps | macro classes | H(micro) | H(macro) | H(micro|macro) | RETENTION");
    eprintln!("  ----+-----------+---------------+---------------+----------+----------+----------------+----------");
    let mut rows: Vec<(&str, &str, Ret)> = Vec::new();
    for (an, arm) in [("A", Arm::A), ("B", Arm::B), ("D", Arm::D)] {
        for (sn, churn, burn) in [("static", false, 0u64), ("CHURN ", true, T_BURN_CHURN)] {
            let r = retention(seed, arm, churn, burn);
            eprintln!("   {}  | {}    | {:>13} | {:>13} | {:>8.4} | {:>8.4} | {:>14.4} | {:>8.2}%",
                an, sn, r.carrier_steps, r.macro_classes, r.h_micro, r.h_macro, r.cond, 100.0 * r.retention);
            rows.push((an, sn, r));
        }
    }

    let stat: Vec<f64> = rows.iter().filter(|(_, s, _)| *s == "static").map(|(_, _, r)| r.retention).collect();
    let chn: Vec<f64> = rows.iter().filter(|(_, s, _)| *s == "CHURN ").map(|(_, _, r)| r.retention).collect();
    let mx_static = stat.iter().cloned().fold(f64::MIN, f64::max);
    let mx_churn = chn.iter().cloned().fold(f64::MIN, f64::max);
    eprintln!("\n  max retention across arms: static {:.2}%, churn {:.2}%", 100.0 * mx_static, 100.0 * mx_churn);
    if mx_churn <= 0.02 && mx_static <= 0.02 {
        eprintln!("  => THE SHADOW RESULT IS NOT SPECIFIC TO THE STATIC SUBSTRATE. The structural channel retains");
        eprintln!("     ~0% of the metabolic channel's class-level information on a MOVING substrate too, so this");
        eprintln!("     looks like a property of structural coarse-graining under the Q2 invariant rather than an");
        eprintln!("     artifact of a frozen crystal.");
    } else if mx_churn > mx_static + 0.02 {
        eprintln!("  => RETENTION RISES UNDER CHURN. The structural channel picks up more of the metabolic channel");
        eprintln!("     when the substrate moves, so the ~0% result is partly a property of the STATIC substrate.");
    } else {
        eprintln!("  => see the table; report the numbers rather than a slogan.");
    }
    eprintln!("=========================================================\n");
}
