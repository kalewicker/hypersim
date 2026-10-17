//! V-track closeout / C3 — the sorted-order gate for the CLOSURE-DEFECT estimator.
//!
//! N1.3 established that the metabolic instrument is reproducible *because it records in
//! sorted NodeId order*, not because the statistic is order-invariant, and measured
//! max |ΔEI| = 4.441e-16 = 2^-51 across processes on raw-order recording — exactly the
//! last-ulp perturbation theory predicts for an order-one sum.
//!
//! The closure-defect estimator has a SECOND order hazard that N1.3's fix does not touch,
//! and this file is where it is found and closed. Sorted recording fixes the TPM REGISTRY
//! order. It does not fix the order in which the ENTROPY SUM runs: every entropy in the
//! V-track's estimators is
//!
//!     -m.values().map(|&c| p * p.log2()).sum::<f64>()          // m: HashMap
//!
//! and `HashMap` iteration order is seeded per PROCESS by `RandomState`. Floating-point
//! addition is non-associative, so the same counts summed in two orders differ in the last
//! ulps. A run recorded in perfectly sorted order can still produce a different closure
//! defect in a different process.
//!
//! C3.1 — the gate is asserted on SORTED summation order (keys sorted), not raw order.
//! C3.2 — checked at n = 3 processes from the start. A single agreeing run is not a
//!        reproducibility test; that is §6.1's fifth instance, and its remedy is
//!        REPETITION, not a second witness.
//! C3.3 — the raw (HashMap-order) divergence is reported as a DIAGNOSTIC, so the guarantee
//!        reads as the conditional it is: reproducible *because* we sum in sorted order.
//!
//! Both live forms of the estimator are gated:
//!   - WITHIN-channel closure defect  (Phase 1's form; degenerate as a boundary test but
//!     still reported, so it still has to be reproducible)
//!   - CROSS-channel retention        (the M3/N4.3 form — the one that survives, and the
//!     one the write-up quotes)
#![cfg(feature = "alife")]

use hg_alife::CARRIER_LABEL;
use hg_causal::TpmAccumulator;
use hg_core::{compute_wl_class_map_snapshot, HypergraphStore, NodeId, PropertyValue, WlHash};
use hg_engine::Engine;
use hg_rules::{compile_rules, parse_rules_toml, StepMode};
use hg_sim_core::runner::apply_alife_overrides;
use hg_sim_core::SimConfig;
use std::collections::{HashMap, HashSet};
use std::process::Command;

const VACUUM: WlHash = WlHash(0xE6_0000_0000_0000);
const ENERGY: u32 = 1;
const EPS: f64 = 1e-9;
const DEPTH: usize = 2;
const WL_DEPTH: u8 = 2;
const STEPS: u64 = 1_500;
/// C3.2: n >= 3 from the start, not two.
const N_PROC: usize = 3;

fn config(seed: u64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "c3"
run_id = "c3"
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

fn diffuse_toml() -> &'static str { include_str!("n_shared/diffuse.toml") }

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

fn bucket(d: f64) -> u64 {
    if d.abs() < EPS { return 0; }
    let mag = d.abs();
    let m = if mag < 0.25 { 1 } else if mag < 0.75 { 2 } else if mag < 1.5 { 3 }
            else if mag < 4.0 { 4 } else if mag < 9.0 { 5 } else { 6 };
    if d < 0.0 { m } else { m + 8 }
}
fn micro_class(label: i64, hist: &[u64]) -> u64 {
    let mut h = 0xCBF2_9CE4_8422_2325u64 ^ ((label as u64) << 40);
    for b in hist { h = (h ^ b).wrapping_mul(0x1000_0000_01B3); }
    h
}

// ------------------------------------------------------------------ the two summation orders
//
// Same counts, same arithmetic, two orders. `h_sorted` is the gate; `h_raw` is the
// diagnostic that shows the gate is doing work.

/// C3.1 — deterministic: terms summed in ascending KEY order, which is process-independent
/// because the keys are deterministic class hashes / registry indices.
fn h_sorted<K: Ord + Copy + std::hash::Hash>(m: &HashMap<K, f64>, tot: f64) -> f64 {
    if tot <= 0.0 { return 0.0; }
    let mut ks: Vec<K> = m.keys().copied().collect();
    ks.sort_unstable();
    -ks.iter().map(|k| { let p = m[k] / tot; if p > 0.0 { p * p.log2() } else { 0.0 } }).sum::<f64>()
}

/// C3.3 — the status quo throughout the V-track: terms summed in HashMap iteration order,
/// which `RandomState` seeds per PROCESS.
fn h_raw<K: Ord + Copy + std::hash::Hash>(m: &HashMap<K, f64>, tot: f64) -> f64 {
    if tot <= 0.0 { return 0.0; }
    -m.values().map(|&c| { let p = c / tot; if p > 0.0 { p * p.log2() } else { 0.0 } }).sum::<f64>()
}

// ------------------------------------------------------------------------------ the estimators

/// WITHIN-channel closure defect on the metabolic TPM, under the TRUE carrier partition.
/// Returns (sorted-order value, raw-order value) from ONE set of counts.
fn cd_within(tpm: &TpmAccumulator, is_c: &HashMap<u64, bool>) -> (f64, f64) {
    let mut compact: HashMap<usize, usize> = HashMap::new();
    let g: Vec<usize> = tpm.hashes().iter().map(|h| {
        let raw = *is_c.get(&h.0).unwrap_or(&false) as usize;
        let n = compact.len();
        *compact.entry(raw).or_insert(n)
    }).collect();
    let counts = tpm.counts_dense();
    let mut joint: HashMap<(usize, usize, usize, usize), f64> = HashMap::new();
    let mut macro_c: HashMap<(usize, usize), f64> = HashMap::new();
    for (i, rowv) in counts.iter().enumerate() {
        for (j, &c) in rowv.iter().enumerate() {
            if c == 0 { continue; }
            let cf = c as f64;
            *joint.entry((g[i], g[j], i, j)).or_insert(0.0) += cf;
            *macro_c.entry((g[i], g[j])).or_insert(0.0) += cf;
        }
    }
    let tot: f64 = macro_c.values().sum(); // u64-derived integers: exact in f64, order-safe
    let s = (h_sorted(&joint, tot) - h_sorted(&macro_c, tot)).max(0.0);
    let r = (h_raw(&joint, tot) - h_raw(&macro_c, tot)).max(0.0);
    (s, r)
}

struct Digest { cd_s: f64, cd_r: f64, ret_s: f64, ret_r: f64, cond_s: f64, cond_r: f64, m: usize, t: u64 }

/// One run, both estimators, both summation orders.
fn measure(seed: u64, sterile: bool, diffuse: bool, markers: usize) -> Digest {
    let cfg = config(seed);
    let mut biology = parse_rules_toml(hg_alife::alife_base_toml()).unwrap();
    apply_alife_overrides(&mut biology, &cfg.alife);
    if sterile { biology.retain(|r| r.name != "replicate"); }
    if diffuse { biology.extend(parse_rules_toml(diffuse_toml()).unwrap()); }
    let rules = compile_rules(&biology).unwrap();
    let mut store = hg_sim_core::runner::build_initial_store(&cfg);
    plant_markers(&mut store, seed, markers);
    let mut engine = Engine::new(rules, StepMode::MaxParallel, cfg.seed, 100_000);

    let mut tpm = TpmAccumulator::new();
    let mut is_c: HashMap<u64, bool> = HashMap::new();
    is_c.insert(VACUUM.0, false);
    let mut prev_e: HashMap<NodeId, f64> = HashMap::new();
    let mut prev_wl: HashMap<NodeId, u64> = HashMap::new();
    let mut hist: HashMap<NodeId, Vec<u64>> = HashMap::new();
    let mut prev_cls: Option<HashMap<NodeId, u64>> = None;
    // cross-channel (M3 / N4.3 form): macro = WL transition, micro = energy sign-of-change
    let mut x_joint: HashMap<(u64, u64), f64> = HashMap::new();
    let mut x_macro: HashMap<u64, f64> = HashMap::new();
    let mut x_micro: HashMap<u64, f64> = HashMap::new();

    for _ in 1..=STEPS {
        engine.step(&mut store);
        let snap = store.snapshot();
        let wl = compute_wl_class_map_snapshot(&snap, WL_DEPTH);
        let mut now: HashMap<NodeId, u64> = HashMap::new();
        let mut now_wl: HashMap<NodeId, u64> = HashMap::new();
        let mut xrows: Vec<(NodeId, u64, u64)> = Vec::new();
        for (id, d) in snap.nodes() {
            let is_carrier = d.label == CARRIER_LABEL;
            let wh = wl.get(&id).copied().unwrap_or(WlHash(0)).0;
            now_wl.insert(id, wh);
            let e = match d.get_property(ENERGY) { Some(PropertyValue::Float(v)) => v, _ => 0.0 };
            let delta = e - prev_e.get(&id).copied().unwrap_or(e);
            let hv = hist.entry(id).or_default();
            hv.insert(0, bucket(delta));
            hv.truncate(DEPTH);
            if hv.len() == DEPTH {
                let cls = micro_class(d.label.0 as i64, hv);
                now.insert(id, cls);
                is_c.entry(cls).or_insert(is_carrier);
            }
            if is_carrier {
                let s = if delta.abs() < EPS { 1 } else if delta < 0.0 { 0 } else { 2 };
                if let Some(&pw) = prev_wl.get(&id) { xrows.push((id, pw ^ wh.rotate_left(1), s)); }
            }
            prev_e.insert(id, e);
        }
        // SORTED-order recording (the N1.3 fix) — fixes the REGISTRY order only
        if let Some(p) = &prev_cls {
            let mut carry: Vec<(NodeId, u64)> = p.iter().map(|(k, v)| (*k, *v)).collect();
            let mut born: Vec<(NodeId, u64)> = now.iter().filter(|(id, _)| !p.contains_key(id)).map(|(k, v)| (*k, *v)).collect();
            carry.sort_unstable();
            born.sort_unstable();
            for (id, hp) in carry { tpm.record(WlHash(hp), WlHash(now.get(&id).copied().unwrap_or(VACUUM.0))); }
            for (_, hn) in born { tpm.record(VACUUM, WlHash(hn)); }
        }
        xrows.sort_unstable();
        for (_, mt, s) in xrows {
            *x_joint.entry((mt, s)).or_insert(0.0) += 1.0;
            *x_macro.entry(mt).or_insert(0.0) += 1.0;
            *x_micro.entry(s).or_insert(0.0) += 1.0;
        }
        prev_cls = Some(now);
        prev_wl = now_wl;
    }

    let (cd_s, cd_r) = cd_within(&tpm, &is_c);
    let xtot: f64 = x_micro.values().sum();
    let hm_s = h_sorted(&x_micro, xtot);
    let hm_r = h_raw(&x_micro, xtot);
    let cond_s = (h_sorted(&x_joint, xtot) - h_sorted(&x_macro, xtot)).max(0.0);
    let cond_r = (h_raw(&x_joint, xtot) - h_raw(&x_macro, xtot)).max(0.0);
    Digest {
        cd_s, cd_r, cond_s, cond_r,
        ret_s: if hm_s > 1e-12 { 1.0 - cond_s / hm_s } else { f64::NAN },
        ret_r: if hm_r > 1e-12 { 1.0 - cond_r / hm_r } else { f64::NAN },
        m: tpm.hashes().len(), t: tpm.total_transitions(),
    }
}

const ARMS: [(&str, bool, bool, usize); 3] = [("A", false, false, 1), ("C", true, false, 6), ("D", true, true, 6)];

fn all_digests() -> String {
    let mut s = String::new();
    for (name, sterile, diffuse, markers) in ARMS {
        let d = measure(12345, sterile, diffuse, markers);
        s.push_str(&format!(
            "{name} M={} T={} cd_s={:.17e} cd_r={:.17e} cond_s={:.17e} cond_r={:.17e} ret_s={:.17e} ret_r={:.17e}\n",
            d.m, d.t, d.cd_s, d.cd_r, d.cond_s, d.cond_r, d.ret_s, d.ret_r));
    }
    s
}

/// Pull one named field out of every line of a digest.
fn field(d: &str, key: &str) -> Vec<f64> {
    d.lines().filter_map(|l| {
        l.split_whitespace().find(|t| t.starts_with(&format!("{key}=")))
            .and_then(|t| t[key.len() + 1..].parse::<f64>().ok())
    }).collect()
}

/// Largest absolute difference in one field across ALL pairs of the n process digests.
fn max_delta(ds: &[String], key: &str) -> f64 {
    let cols: Vec<Vec<f64>> = ds.iter().map(|d| field(d, key)).collect();
    let mut worst = 0.0f64;
    for i in 0..cols.len() {
        for j in (i + 1)..cols.len() {
            for (a, b) in cols[i].iter().zip(cols[j].iter()) { worst = worst.max((a - b).abs()); }
        }
    }
    worst
}

/// The gate: one field bit-identical across every pair of processes.
fn all_equal(ds: &[String], key: &str) -> bool {
    let cols: Vec<Vec<f64>> = ds.iter().map(|d| field(d, key)).collect();
    cols.windows(2).all(|w| w[0].iter().zip(w[1].iter()).all(|(a, b)| a.to_bits() == b.to_bits()))
}

#[test]
fn c3_closure_defect_sorted_order_gate() {
    // CHILD
    if let Ok(path) = std::env::var("C3_PROBE_OUT") {
        std::fs::write(&path, all_digests()).expect("child could not write digest");
        return;
    }
    // PARENT: C3.2 — n = 3 separate OS processes, from the start.
    let exe = std::env::current_exe().expect("current_exe");
    let dir = std::env::temp_dir();
    let paths: Vec<std::path::PathBuf> = (0..N_PROC).map(|i| dir.join(format!("c3_probe_{i}.txt"))).collect();
    for p in &paths {
        let st = Command::new(&exe)
            .args(["c3_closure_defect_sorted_order_gate", "--exact", "--nocapture"])
            .env("C3_PROBE_OUT", p)
            .status()
            .expect("failed to spawn child process");
        assert!(st.success(), "child process failed");
    }
    let ds: Vec<String> = paths.iter()
        .map(|p| std::fs::read_to_string(p).expect("child wrote no digest")).collect();
    assert!(ds.iter().all(|d| !d.trim().is_empty()), "a child digest is empty");

    eprintln!("\n================ V-TRACK CLOSEOUT / C3 — SORTED-ORDER GATE FOR THE CLOSURE DEFECT ================");
    eprintln!("  n = {N_PROC} separate OS processes (C3.2: a single agreeing run is not a reproducibility test).");
    eprintln!("  N1.3's sorted RECORDING fixes the TPM registry order. It does NOT fix the order of the ENTROPY");
    eprintln!("  SUM, which iterates a HashMap and is therefore seeded per PROCESS. That is the second hazard.\n");
    for (i, d) in ds.iter().enumerate() {
        eprintln!("  --- process {i} ---");
        for l in d.lines() { eprintln!("    {l}"); }
    }

    let keys = [
        ("cd    (within-channel closure defect)", "cd_s", "cd_r"),
        ("cond  (cross-channel H(micro|macro))", "cond_s", "cond_r"),
        ("ret   (cross-channel retention)", "ret_s", "ret_r"),
    ];
    eprintln!("\n  estimator                              | SORTED sum (the gate)      | RAW HashMap-order sum (diagnostic)");
    eprintln!("  ---------------------------------------+----------------------------+-----------------------------------");
    for (name, ks, kr) in keys {
        eprintln!("  {:<38} | bit-identical: {:<11} | bit-identical: {:<6}  max |Δ| = {:.3e}",
            name, all_equal(&ds, ks), all_equal(&ds, kr), max_delta(&ds, kr));
    }

    // C3.1 — THE GATE, on sorted summation order.
    for (name, ks, _) in keys {
        assert!(all_equal(&ds, ks),
            "{name}: sorted-order estimator differs across processes — the gate is not holding");
    }

    // C3.2, made EMPIRICAL rather than asserted: is there a field on which the first two
    // processes agree bit-for-bit and only the third diverges? If so, an n = 2 check would
    // have PASSED it, and the n >= 3 requirement is load-bearing rather than ceremonial.
    let mut n2_would_pass: Vec<String> = Vec::new();
    for (name, _, kr) in keys {
        let cols: Vec<Vec<f64>> = ds.iter().map(|d| field(d, kr)).collect();
        for row in 0..cols[0].len() {
            let v: Vec<f64> = cols.iter().map(|c| c[row]).collect();
            let first_two_agree = v[0].to_bits() == v[1].to_bits();
            let any_later_differs = v[2..].iter().any(|x| x.to_bits() != v[0].to_bits());
            if first_two_agree && any_later_differs {
                n2_would_pass.push(format!("{} arm {}", name.split_whitespace().next().unwrap(), ARMS[row].0));
            }
        }
    }
    eprintln!("\n  C3.2 — would n = 2 have caught this?");
    if n2_would_pass.is_empty() {
        eprintln!("    On this run every divergence was already visible in the first two processes.");
        eprintln!("    n >= 3 stays required regardless: agreement between two samples of a per-process");
        eprintln!("    random seed is not evidence of invariance.");
    } else {
        eprintln!("    NO — on {} field(s) processes 0 and 1 agree BIT-FOR-BIT and only process 2 diverges:",
            n2_would_pass.len());
        for f in &n2_would_pass { eprintln!("      {f}"); }
        eprintln!("    An n = 2 check would have reported those fields as reproducible. This is §6.1's");
        eprintln!("    fifth instance caught in the act, and it is why the remedy is REPETITION.");
    }

    // C3.3 — the raw divergence, reported as the diagnostic that makes the gate conditional.
    let raw_worst = keys.iter().map(|(_, _, kr)| max_delta(&ds, *kr)).fold(0.0, f64::max);
    eprintln!("\n  worst raw-order divergence across all estimators: {raw_worst:.3e}");
    if raw_worst > 0.0 {
        eprintln!("  *** THE DIAGNOSTIC FIRES. ***");
        eprintln!("  The closure-defect estimator is NOT order-invariant. Identical counts summed in two");
        eprintln!("  HashMap orders give different last ulps, exactly as §6.2 predicts for a non-associative");
        eprintln!("  sum. N1.3's sorted RECORDING does not cover this: it fixes which index a class lands on,");
        eprintln!("  not the order the entropy terms are added in.");
        eprintln!("  SCOPE: {raw_worst:.3e} cannot move any published verdict (permutation counts here compare");
        eprintln!("  values separated by ~1e-2), so nothing is retracted. What changes is the WORDING of the");
        eprintln!("  guarantee: the estimator is reproducible BECAUSE it sums in sorted order, not because the");
        eprintln!("  statistic is order-invariant. Asserted above, at n = {N_PROC}.");
    } else {
        eprintln!("  Raw order happened to agree in this environment across {N_PROC} processes. That is NOT a");
        eprintln!("  guarantee — RandomState is seeded per process and the sum is non-associative — so the gate");
        eprintln!("  stays on sorted order regardless.");
    }
    eprintln!("=========================================================================\n");
    for p in &paths { let _ = std::fs::remove_file(p); }
}
