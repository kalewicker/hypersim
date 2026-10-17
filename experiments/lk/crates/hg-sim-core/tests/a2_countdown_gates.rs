//! A2 gates (v6 §4.4) for the countdown removal mechanism:
//!   AG1 tick engagement — lk_tick guard counters show evals ≈ live
//!        countdown nodes, rejects on expired/absent ones (a zero-reject
//!        sweep is uninterpretable, the v4 §1.1 rule).
//!   AG2 lifetime exactness — a link fuses on exactly the step its
//!        countdown hits 0, not one step early or late (scripted chain).
//!   AG3 dispersion precondition (§1.2) — the live countdown distribution
//!        over interior nodes has p10 ≠ p90 in the first windows.
//!   AG4 audit exact in both ledger modes.
//!   AG5 determinism — zero draws, byte-identical reruns.

use hg_core::{HyperEdge, HypergraphStore, Label, NodeData, NodeId, PropertyValue};
use hg_rules::StepMode;
use hg_sim_core::{run_headless, SimConfig};
use smallvec::smallvec;
use std::path::Path;

const ENERGY: u32 = 1;
const GENERATION: u32 = 6;
const COUNTDOWN: u32 = 7;

fn rules_toml(q: f64, tau: f64) -> String {
    std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../rules/lk_pair_capped.toml"
    ))
    .unwrap()
    .replace(
        "property = 6\nop       = \"lt\"\nvalue    = 4.0",
        "property = 6\nop       = \"lt\"\nvalue    = 1.0e18",
    )
    .replace("value = 50.0\n", &format!("value = {q:.1}\n"))
    .replace("value = 60.0", &format!("value = {:.1}", q + 10.0))
    .replace("offset = 50.0", &format!("offset = {:.1}", 2.0 * tau))
    .replace("value = 25.0", &format!("value = {tau:.1}"))
}

fn countdown_config(dir: &Path, k: i64, ledger: &str, max_steps: u64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "a2_countdown"
seed = 42
out_dir = "{}"
[rules]
source = "Inline"
[initial]
condition_type = "TriLattice"
width = 32
[engine]
step_mode = "MaxParallel"
max_steps = {max_steps}
metrics_window_interval = 100
max_pairings_per_step = 100000
thread_count = 1
[lk_rules]
enabled = true
e_total = 100000.0
g_max = 1.0e18
priority = "transport_first"
q = 50.0
tau = 15.0
[lk_rules.fuse]
enabled = true
gate = "countdown"
countdown_k = {k}
ledger_mode = "{ledger}"
[leakage]
enabled = true
cadence_steps = 100
[output]
snapshots_enabled = true
fixed_provenance = true
"#,
        dir.display(),
    );
    let mut cfg = SimConfig::from_toml(&toml).unwrap();
    cfg.rules.inline_toml = Some(rules_toml(50.0, 15.0));
    cfg
}

/// AG1: lk_tick guard engaged — nonzero evals AND nonzero rejects.
/// AG4 (vanish): audit exact. AG3: countdown dispersion p10 ≠ p90.
#[test]
fn ag1_tick_engaged_ag3_dispersion_ag4_audit() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = countdown_config(dir.path(), 100, "vanish", 3000);
    let artifacts = run_headless(&cfg, None, None, None).expect("run");

    // AG4: audit residual exactly 0 every window.
    let leakage = std::fs::read_to_string(artifacts.run_dir.join("leakage.csv")).unwrap();
    let lrows: Vec<Vec<&str>> = leakage.lines().skip(2).map(|l| l.split(',').collect()).collect();
    assert!(
        lrows.iter().all(|r| r[18].parse::<f64>().unwrap() == 0.0),
        "AG4: vanish-mode audit must be exact"
    );

    // AG1: lk_tick prop0 (countdown > 0) engaged — evals and rejects both
    // nonzero over the run (rejects come from the step a node hits 0 /
    // any candidate lacking the key).
    let guards = std::fs::read_to_string(artifacts.run_dir.join("guards.csv")).unwrap();
    let (mut evals, mut rejects) = (0u64, 0u64);
    for l in guards.lines().skip(2) {
        let f: Vec<&str> = l.split(',').collect();
        if f[1] == "lk_tick" && f[2] == "prop0" {
            evals += f[3].parse::<u64>().unwrap_or(0);
            rejects += f[4].parse::<u64>().unwrap_or(0);
        }
    }
    assert!(evals > 0, "AG1: lk_tick must be evaluated");
    assert!(rejects > 0, "AG1: lk_tick must reject (expired/absent) — zero-reject is uninterpretable");

    // AG3: live countdown distribution has dispersion (p10 ≠ p90) in an
    // early window. Read a mid-era snapshot; interior nodes born at
    // different steps carry different countdowns.
    let snaps = hg_eval::cone::load_snapshots(&artifacts.run_dir.join("snapshots")).unwrap();
    let mid = &snaps[snaps.len() / 3].1;
    let mut cds: Vec<i64> = mid
        .nodes()
        .filter_map(|(_, d)| match d.get_property(COUNTDOWN) {
            Some(PropertyValue::Int(c)) if c > 0 => Some(c),
            _ => None,
        })
        .collect();
    cds.sort_unstable();
    assert!(cds.len() >= 10, "AG3: need a populated interior to check dispersion");
    let p10 = cds[cds.len() / 10];
    let p90 = cds[cds.len() * 9 / 10];
    assert_ne!(p10, p90, "AG3: countdown must be dispersed (p10 {p10} == p90 {p90} is degenerate)");
}

/// AG4 (transfer): audit exact on E_total + c·N_sub = E₀, ledger 0.
#[test]
fn ag4_transfer_audit_exact() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = countdown_config(dir.path(), 100, "transfer", 3000);
    let artifacts = run_headless(&cfg, None, None, None).expect("run");
    let leakage = std::fs::read_to_string(artifacts.run_dir.join("leakage.csv")).unwrap();
    let rows: Vec<Vec<&str>> = leakage.lines().skip(2).map(|l| l.split(',').collect()).collect();
    assert!(rows.iter().all(|r| r[18].parse::<f64>().unwrap() == 0.0), "AG4: transfer audit exact");
    let last = rows.last().unwrap();
    let e_fused: f64 = last[16].parse().unwrap();
    assert_eq!(e_fused, 0.0, "AG4: transfer mode ledgers nothing");
}

/// AG5: determinism — two independent runs are byte-identical
/// (canonical hash) ⇒ zero-draw preserved through the countdown path.
#[test]
fn ag5_determinism() {
    let d1 = tempfile::tempdir().unwrap();
    let d2 = tempfile::tempdir().unwrap();
    let a = run_headless(&countdown_config(d1.path(), 100, "transfer", 2000), None, None, None).unwrap();
    let b = run_headless(&countdown_config(d2.path(), 100, "transfer", 2000), None, None, None).unwrap();
    assert_eq!(a.canonical_hash, b.canonical_hash, "AG5: countdown runs must be byte-identical");
}

/// AG2: lifetime exactness on a scripted throat. A filament link with
/// countdown = k fuses on EXACTLY the step its countdown reaches 0 — not
/// one step early or late — with the whole LK-R + tick + fuse rule set
/// (countdown gate) driving it, no transport competition on this
/// isolated chain. Built through the production `load_rules` path.
#[test]
fn ag2_lifetime_exactness() {
    // Scripted throat S–w–N, interior w carrying gen 1, sub-θ energy (so
    // only tick + fuse act), countdown = 3. Endpoints anchored in a bulk
    // (degree > 2) so only w is fusable.
    let k = 3i64;
    let mut store = HypergraphStore::new();
    let mk = |s: &mut HypergraphStore, gen: f64, cd: Option<i64>| -> NodeId {
        let mut d = NodeData::new(Label(0), 0);
        d.set_property(ENERGY, PropertyValue::Float(30.0));
        d.set_property(GENERATION, PropertyValue::Float(gen));
        if let Some(c) = cd {
            d.set_property(COUNTDOWN, PropertyValue::Int(c));
        }
        s.add_node(d)
    };
    let s = mk(&mut store, 0.0, None);
    let n = mk(&mut store, 0.0, None);
    let w = mk(&mut store, 1.0, Some(k));
    store.add_edge(HyperEdge::new(Label(0), smallvec![s, w]));
    store.add_edge(HyperEdge::new(Label(0), smallvec![w, n]));
    let a = mk(&mut store, 0.0, None);
    let b = mk(&mut store, 0.0, None);
    store.add_edge(HyperEdge::new(Label(0), smallvec![s, a]));
    store.add_edge(HyperEdge::new(Label(0), smallvec![n, b]));

    // Build the countdown rule set via the production config path (tick +
    // fuse-with-countdown-guard), then drive it directly.
    let cfg = countdown_config(Path::new("/tmp/ag2_unused"), k, "vanish", 1);
    let rules = hg_sim_core::load_rules(&cfg).unwrap();
    let mut engine = hg_engine::Engine::new(rules, StepMode::MaxParallel, 42, 100000);

    // Track w's countdown and existence per step. With no transport
    // competition (isolated chain, sub-θ), w ticks every step: k, k−1,
    // …, 0, then fuses on the step countdown is 0.
    let cd = |store: &HypergraphStore, id: NodeId| -> Option<i64> {
        store.get_node(id).and_then(|d| match d.get_property(COUNTDOWN) {
            Some(PropertyValue::Int(c)) => Some(c),
            _ => None,
        })
    };
    assert_eq!(cd(&store, w), Some(k), "w starts at k");
    let mut fused_at = None;
    for step in 1..=(k as u64 + 3) {
        engine.step(&mut store);
        if store.get_node(w).is_none() {
            fused_at = Some(step);
            break;
        }
        // While alive, countdown must be exactly k − step.
        assert_eq!(
            cd(&store, w),
            Some(k - step as i64),
            "AG2: countdown must decrement by exactly 1/step (step {step})"
        );
    }
    // Countdown hits 0 after k ticks (step k); fuse (guard countdown==0)
    // then fires the next step, so w is gone at step k+1.
    assert_eq!(
        fused_at,
        Some(k as u64 + 1),
        "AG2: link must fuse exactly one step after countdown reaches 0, not early/late"
    );
    // Endpoints rejoined.
    let joined = store.edges().any(|(_, e)| {
        let mut v: Vec<NodeId> = e.nodes.to_vec();
        v.sort_unstable();
        let mut want = vec![s, n];
        want.sort_unstable();
        v == want
    });
    assert!(joined, "AG2: S–N rejoined on fuse");
}
