//! Step 1 — does the energy-band partition stay RESOLVED (k_carrier >= 2, adequate
//! null support) and DETECT the boundary on the BOUNDED CHURN substrate?
//!
//! J2 showed the (label, energy-band) partition recovers F8 detectability on STATIC
//! data and re-clears the TWTF gate under (unbounded) motion. The one thing the
//! unbounded proxies could not show was whether it stays RESOLVED under real
//! *bounded* churn (triangle's explosion collapsed it to M=2 as the biology died).
//! Now the H1 churn substrate exists, so this is directly testable: run the F8
//! biology WITH the churn rule and score the energy-band carrier/substrate boundary.
//!
//! Rules = F8 biology (alife_base + overrides, incl. its own influx) + the churn
//! rule only (energy>=cap, unoccupied; no duplicate influx). Scoring is F8's.
#![cfg(feature = "alife")]

use hg_alife::CARRIER_LABEL;
use hg_causal::{coarse_grain, ei_from_tpm, TpmAccumulator};
use hg_core::{NodeId, PropertyValue, WlHash};
use hg_engine::Engine;
use hg_rules::{compile_rules, parse_rules_toml, StepMode};
use hg_sim_core::runner::apply_alife_overrides;
use hg_sim_core::SimConfig;
use std::collections::HashMap;

const VACUUM: WlHash = WlHash(0xE6_0000_0000_0000);
const STEPS: u64 = 6_000;
const N_PERM: usize = 200;
const ENERGY: u32 = 1;
const OCCUPIED: u32 = 22;

fn config(seed: u64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "s1"
run_id = "s1"
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
seed_carrier_marker = true
initial_substrate_energy = 5.0
influx_rate = 0.34
bootstrap_replication_threshold = 3.0
"#
    );
    SimConfig::from_toml(&toml).unwrap()
}

/// The churn rule ONLY (no influx — the biology supplies influx). Path-preserving,
/// energy>=cap, unoccupied.
fn churn_rule_toml() -> &'static str {
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

fn build_rules(cfg: &SimConfig, with_churn: bool) -> Vec<hg_rules::CompiledRule> {
    let mut biology = parse_rules_toml(hg_alife::alife_base_toml()).unwrap();
    apply_alife_overrides(&mut biology, &cfg.alife);
    if with_churn {
        biology.extend(parse_rules_toml(churn_rule_toml()).unwrap());
    }
    compile_rules(&biology).unwrap()
}

fn splitmix(s: &mut u64) -> u64 {
    *s = s.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *s;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}
fn null_support(m: usize, k: usize) -> f64 {
    if k == 0 || k >= m { return 0.0; }
    let k = k.min(m - k);
    let mut acc = 1.0f64;
    for i in 0..k { acc *= (m - i) as f64 / (i + 1) as f64; }
    acc
}
fn ce_for(tpm: &TpmAccumulator, g: &[usize], k: usize) -> f64 {
    let rows = tpm.normalized_rows();
    let micro = ei_from_tpm(&rows.iter().map(|(_, r)| r.clone()).collect::<Vec<_>>());
    let mac = ei_from_tpm(&coarse_grain(&rows, g, k));
    if micro.ei > 0.0 { mac.ei / micro.ei } else { 0.0 }
}
fn energy_band_class(label: i64, e: f64) -> u64 {
    let band = e.floor().clamp(-1.0, 64.0) as i64;
    ((label as u64) << 32) ^ (band as u64 & 0xFFFF_FFFF) ^ 0x9E37_79B9
}

struct Res { m: usize, k_carrier: usize, real_ce: f64, null_mean: f64, null_ge: usize, support: f64, final_carriers: usize, last_alive: u64 }

fn run(seed: u64, with_churn: bool) -> Res {
    let cfg = config(seed);
    let rules = build_rules(&cfg, with_churn);
    let mut store = hg_sim_core::runner::build_initial_store(&cfg);
    let mut engine = Engine::new(rules, StepMode::MaxParallel, cfg.seed, 100_000);

    let mut tpm = TpmAccumulator::new();
    let mut is_c: HashMap<u64, bool> = HashMap::new();
    is_c.insert(VACUUM.0, false);
    let mut prev: Option<HashMap<NodeId, u64>> = None;
    let mut final_carriers = 0usize;
    let mut last_alive = 0u64;

    for step in 1..=STEPS {
        engine.step(&mut store);
        let snap = store.snapshot();
        let mut now: HashMap<NodeId, u64> = HashMap::new();
        final_carriers = 0;
        for (id, d) in snap.nodes() {
            let is_carrier = d.label == CARRIER_LABEL;
            if is_carrier { final_carriers += 1; }
            let e = match d.get_property(ENERGY) { Some(PropertyValue::Float(v)) => v, _ => 0.0 };
            let cls = energy_band_class(d.label.0 as i64, e);
            now.insert(id, cls);
            is_c.entry(cls).or_insert(is_carrier);
        }
        if final_carriers > 0 { last_alive = step; }
        if let Some(p) = &prev {
            for (id, &hp) in p { tpm.record(WlHash(hp), WlHash(now.get(id).copied().unwrap_or(VACUUM.0))); }
            for (id, &hn) in &now { if !p.contains_key(id) { tpm.record(VACUUM, WlHash(hn)); } }
        }
        prev = Some(now);
    }

    let mut compact: HashMap<usize, usize> = HashMap::new();
    let group_of: Vec<usize> = tpm.hashes().iter().map(|h| {
        let raw = *is_c.get(&h.0).unwrap_or(&false) as usize;
        let n = compact.len();
        *compact.entry(raw).or_insert(n)
    }).collect();
    let k = compact.len().max(1);
    let m = tpm.hashes().len();
    let k_carrier = tpm.hashes().iter().filter(|h| *is_c.get(&h.0).unwrap_or(&false)).count();
    let real_ce = if k >= 2 { ce_for(&tpm, &group_of, k) } else { 0.0 };
    let canon = tpm.canonical_order();
    let mut rng = 0x5EED_F7 ^ seed;
    let (mut ge, mut acc) = (0usize, 0.0);
    if k >= 2 {
        for _ in 0..N_PERM {
            let mut labels: Vec<usize> = canon.iter().map(|&i| group_of[i]).collect();
            for i in (1..labels.len()).rev() { let j = (splitmix(&mut rng) % (i as u64 + 1)) as usize; labels.swap(i, j); }
            let mut perm = vec![0usize; group_of.len()];
            for (pos, &i) in canon.iter().enumerate() { perm[i] = labels[pos]; }
            let c = ce_for(&tpm, &perm, k);
            acc += c;
            if c >= real_ce { ge += 1; }
        }
    }
    Res { m, k_carrier, real_ce, null_mean: acc / N_PERM as f64, null_ge: ge, support: null_support(m, k_carrier), final_carriers, last_alive }
}

#[test]
fn step1_energyband_resolved_under_churn() {
    let seed = 12345;
    let stat = run(seed, false);
    let churn = run(seed, true);
    eprintln!("\n================ STEP 1 — energy-band partition on STATIC vs BOUNDED CHURN (seed {seed}) ================");
    eprintln!("  substrate | M   | k_carrier | real CE | null mean | null>=real | p       | support  | final carriers | last-alive");
    eprintln!("  ----------+-----+-----------+---------+-----------+------------+---------+----------+----------------+-----------");
    for (name, r) in [("static", &stat), ("churn", &churn)] {
        let p = if r.null_ge == 0 { "<.005".into() } else { format!("{:.3}", r.null_ge as f64 / N_PERM as f64) };
        eprintln!("  {:<9} | {:>3} | {:>9} | {:>7.4} | {:>9.4} | {:>4}/{:<4}  | {:>7} | {:.2e} | {:>14} | {:>10}",
            name, r.m, r.k_carrier, r.real_ce, r.null_mean, r.null_ge, N_PERM, p, r.support, r.final_carriers, r.last_alive);
    }
    eprintln!("==================================================================================================================\n");
    let detects = churn.k_carrier >= 2 && churn.k_carrier < churn.m && (churn.null_ge as f64 / N_PERM as f64) < 0.05 && churn.support > 100.0;
    if detects {
        eprintln!("  STEP 1 PASSES: the energy-band partition stays RESOLVED (k_carrier={}, support {:.1e}) and DETECTS the",
            churn.k_carrier, churn.support);
        eprintln!("  boundary (p<0.05) on the REAL bounded churn substrate. The J2 instrument survives bounded churn, not");
        eprintln!("  just static + the unbounded proxies. The one thing triangle's explosion could not show is now shown.");
    } else {
        eprintln!("  STEP 1 CAVEAT: energy-band under-resolved/undetecting under churn (k={}, support {:.1e}, null>=real {}/{}).",
            churn.k_carrier, churn.support, churn.null_ge, N_PERM);
    }
    assert!(churn.k_carrier >= 2, "energy-band collapsed to <2 carrier classes under churn (k={})", churn.k_carrier);
}
