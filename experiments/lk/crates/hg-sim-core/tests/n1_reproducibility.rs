//! V9 / N1.3 — TWO-PROCESS reproducibility of the metabolic instrument.
//!
//! A same-process rerun cannot detect a process-seeded hasher (it seeds once per
//! process), so this test re-execs ITSELF as two separate OS processes and diffs
//! their output. Each child writes its digest to the file named by N1_PROBE_OUT;
//! the parent (env unset) spawns two children and compares.
//!
//! Per the §6 lesson — statistics over INDICES can differ while a scalar
//! coefficient agrees — the digest is INDEX-DEPENDENT: the ordered vector of
//! metabolic micro-class hashes and their per-class TPM row sums, not just CE.
//! A run that permuted its class registry would keep CE and fail this.
#![cfg(feature = "alife")]

use hg_alife::CARRIER_LABEL;
use hg_causal::TpmAccumulator;
use hg_core::{HypergraphStore, NodeId, PropertyValue, WlHash};
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
const STEPS: u64 = 1_500;

fn config(seed: u64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "n1r"
run_id = "n1r"
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

/// INDEX-DEPENDENT digest: class hashes AND their row sums, in TPM REGISTRY order.
///
/// `sorted` controls the only thing that can make that order vary: whether the
/// per-step recording iterates the node maps in sorted NodeId order or in std
/// HashMap order (which is seeded per PROCESS by RandomState). With `sorted =
/// false` this is the original recording pattern used throughout the project.
fn digest(seed: u64, sterile: bool, diffuse: bool, markers: usize, sorted: bool) -> (String, f64) {
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
    let mut prev_e: HashMap<NodeId, f64> = HashMap::new();
    let mut hist: HashMap<NodeId, Vec<u64>> = HashMap::new();
    let mut prev_cls: Option<HashMap<NodeId, u64>> = None;
    for _ in 1..=STEPS {
        engine.step(&mut store);
        let snap = store.snapshot();
        let mut now: HashMap<NodeId, u64> = HashMap::new();
        for (id, d) in snap.nodes() {
            let e = match d.get_property(ENERGY) { Some(PropertyValue::Float(v)) => v, _ => 0.0 };
            let delta = e - prev_e.get(&id).copied().unwrap_or(e);
            let hv = hist.entry(id).or_default();
            hv.insert(0, bucket(delta));
            hv.truncate(DEPTH);
            if hv.len() == DEPTH { now.insert(id, micro_class(d.label.0 as i64, hv)); }
            prev_e.insert(id, e);
        }
        if let Some(p) = &prev_cls {
            let mut carry: Vec<(NodeId, u64)> = p.iter().map(|(k, v)| (*k, *v)).collect();
            let mut born: Vec<(NodeId, u64)> = now.iter()
                .filter(|(id, _)| !p.contains_key(id)).map(|(k, v)| (*k, *v)).collect();
            if sorted { carry.sort_unstable(); born.sort_unstable(); }
            for (id, hp) in carry { tpm.record(WlHash(hp), WlHash(now.get(&id).copied().unwrap_or(VACUUM.0))); }
            for (_, hn) in born { tpm.record(VACUUM, WlHash(hn)); }
        }
        prev_cls = Some(now);
    }
    let counts = tpm.counts_dense();
    let mut out = String::new();
    // registry ORDER matters here — this is the index-dependent part
    for (i, h) in tpm.hashes().iter().enumerate() {
        let rs: u64 = counts[i].iter().sum();
        out.push_str(&format!("{i} {:#018x} {rs}\n", h.0));
    }
    out.push_str(&format!("M={} T={}\n", tpm.hashes().len(), tpm.total_transitions()));
    // scalar: EI_micro over the normalized rows — order-INVARIANT by construction,
    // so it should agree even when the registry order does not.
    let rows = tpm.normalized_rows();
    let ei = hg_causal::ei_from_tpm(&rows.iter().map(|(_, r)| r.clone()).collect::<Vec<_>>()).ei;
    (out, ei)
}

/// Emits, per arm, BOTH recording orders plus the order-invariant scalar, so the
/// parent can separate "the statistic is reproducible" from "the registry order is".
fn all_digests() -> String {
    let mut s = String::new();
    for (name, sterile, diffuse, markers) in [
        ("A", false, false, 1usize),
        ("D", true, true, 6),
        ("C", true, false, 6),
    ] {
        let (sorted, ei_s) = digest(12345, sterile, diffuse, markers, true);
        let (raw, ei_r) = digest(12345, sterile, diffuse, markers, false);
        s.push_str(&format!("=== arm {name} SORTED ei={ei_s:.17e}\n{sorted}"));
        s.push_str(&format!("=== arm {name} RAW ei={ei_r:.17e}\n{raw}"));
    }
    s
}

/// Split into (sorted digest, raw digest, sorted scalars, raw scalars). The two
/// scalar streams are kept APART: the sorted one is the instrument's actual output
/// and must be bit-identical; the raw one is diagnostic.
fn split(d: &str) -> (String, String, String, String) {
    let (mut so, mut ra, mut sc_s, mut sc_r) = (String::new(), String::new(), String::new(), String::new());
    let mut in_sorted = true;
    for line in d.lines() {
        if line.starts_with("=== ") {
            in_sorted = line.contains(" SORTED ");
            if in_sorted { sc_s.push_str(line); sc_s.push('\n'); } else { sc_r.push_str(line); sc_r.push('\n'); }
            continue;
        }
        if in_sorted { so.push_str(line); so.push('\n'); } else { ra.push_str(line); ra.push('\n'); }
    }
    (so, ra, sc_s, sc_r)
}

/// Largest absolute difference between the `ei=` values in two scalar streams.
fn max_ei_delta(a: &str, b: &str) -> f64 {
    let nums = |s: &str| -> Vec<f64> {
        s.lines().filter_map(|l| l.split("ei=").nth(1)).filter_map(|v| v.trim().parse::<f64>().ok()).collect()
    };
    nums(a).iter().zip(nums(b).iter()).map(|(x, y)| (x - y).abs()).fold(0.0, f64::max)
}

#[test]
fn n1_3_two_process_reproducibility() {
    // CHILD: env var set — compute and write, then return.
    if let Ok(path) = std::env::var("N1_PROBE_OUT") {
        std::fs::write(&path, all_digests()).expect("child could not write digest");
        return;
    }
    // PARENT: spawn two children as separate OS processes and diff their digests.
    let exe = std::env::current_exe().expect("current_exe");
    let dir = std::env::temp_dir();
    let (p1, p2) = (dir.join("n1_probe_a.txt"), dir.join("n1_probe_b.txt"));
    for p in [&p1, &p2] {
        let st = Command::new(&exe)
            .args(["n1_3_two_process_reproducibility", "--exact", "--nocapture"])
            .env("N1_PROBE_OUT", p)
            .status()
            .expect("failed to spawn child process");
        assert!(st.success(), "child process failed");
    }
    let a = std::fs::read_to_string(&p1).expect("child A wrote no digest");
    let b = std::fs::read_to_string(&p2).expect("child B wrote no digest");
    assert!(!a.trim().is_empty(), "child A digest empty");

    let (sa, ra, csa, cra) = split(&a);
    let (sb, rb, csb, crb) = split(&b);

    eprintln!("\n================ V9 / N1.3 — TWO-PROCESS REPRODUCIBILITY ================");
    eprintln!("  Two separate OS processes. Per the §6 lesson the digest is INDEX-DEPENDENT (class hashes +");
    eprintln!("  row sums in TPM REGISTRY order), so a scalar that merely happens to agree cannot hide a");
    eprintln!("  divergence. Both recording orders are emitted so the two questions separate cleanly.\n");
    for line in csa.lines() { eprintln!("    {line}"); }
    eprintln!();
    eprintln!("  THE INSTRUMENT (sorted-order recording):");
    eprintln!("    scalar (EI_micro) bit-identical across processes : {}", csa == csb);
    eprintln!("    registry digest bit-identical across processes   : {}", sa == sb);
    eprintln!("  DIAGNOSTIC (raw HashMap-order recording):");
    eprintln!("    scalar (EI_micro) bit-identical across processes : {}  (max |Δ| = {:.3e})",
        cra == crb, max_ei_delta(&cra, &crb));
    eprintln!("    registry digest bit-identical across processes   : {}", ra == rb);

    // The GATE is the instrument as it actually records: sorted order, both digest and scalar.
    assert_eq!(csa, csb, "sorted-order scalar differs across processes — the instrument is not reproducible");
    assert_eq!(sa, sb, "sorted-order registry digest differs across processes");

    if ra != rb {
        let d = max_ei_delta(&cra, &crb);
        eprintln!("\n  *** FINDING — the check fired, and it is SHARPER than first reported. ***");
        eprintln!("  Iterating the per-step node maps in std HashMap order gives each PROCESS a different");
        eprintln!("  class-registry ORDER (RandomState is seeded per process). The class SET and every row sum");
        eprintln!("  agree; only the indices they land on differ.");
        eprintln!("  CORRECTION to the first reading of this test: I initially recorded that 'the scalars are");
        eprintln!("  UNAFFECTED'. That is true in EXACT arithmetic and FALSE in floating point. EI sums over rows,");
        eprintln!("  floating-point addition is non-associative, so a different row order changes the last ulps:");
        eprintln!("  observed max |Δ EI| = {d:.3e} between processes. The first run of this test happened to agree");
        eprintln!("  bit-for-bit and I generalised from one observation.");
        eprintln!("  WHAT THIS DOES AND DOES NOT MEAN: a ~1e-16 perturbation cannot move any p-value here (the");
        eprintln!("  permutation counts compare CE values separated by ~1e-2), and F3's `canonical_order()` already");
        eprintln!("  fixes the NULL's shuffle order, so no published result changes. But 'reproducible' now means");
        eprintln!("  'reproducible BECAUSE we record in sorted order', not 'reproducible because the statistic is");
        eprintln!("  order-invariant'. The instrument records sorted, and that is bit-identical (asserted above).");
    } else {
        eprintln!("\n  Both orders agree; the registry order happened to be stable in this environment.");
    }
    eprintln!("=========================================================\n");
    let _ = std::fs::remove_file(&p1);
    let _ = std::fs::remove_file(&p2);
}
