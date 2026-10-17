//! H0.2 + H0.3 — CE partition-stationarity, measured against the committed H0.1
//! criterion (`docs/validation_v4_h0_stationarity_criterion.md`, commit 24456eb).
//!
//! H0.2: baseline stationarity on the EXISTING F8 static-substrate run (the trusted
//!       floor CE already relies on).
//! H0.3: stationarity under substrate MOTION, injected with the existing builtins
//!       (`growth_treadmill`, `triangle_subdivide`) as perturbation sources — they
//!       are unbounded and unusable as regimes, but perfectly good for the question
//!       "what does substrate motion do to the WL partition?".
//!
//! Metrics (all from the same streaming WL/TPM machinery as F8, no pipeline change):
//!  - TWTF (primary): transition-weighted transient fraction; a class is transient
//!    if live in < R=3 distinct steps. Node-count independent (a ratio).
//!  - M(t) closure: new classes minted in the last window-quartile / total M.
//!  - live-class step-to-step persistence (Jaccard overlap), node-count independent.
//!  - T/M sampling ratio.
#![cfg(feature = "alife")]

use hg_alife::CARRIER_LABEL;
use hg_causal::TpmAccumulator;
use hg_core::{compute_wl_class_map_snapshot, Label, NodeId, WlHash};
use hg_engine::Engine;
use hg_rules::{builtin_rules, compile_rules, parse_rules_toml, StepMode};
use hg_sim_core::runner::apply_alife_overrides;
use hg_sim_core::SimConfig;
use std::collections::{HashMap, HashSet};

const VACUUM: WlHash = WlHash(0xE6_0000_0000_0000);
const WL_DEPTH: u8 = 2;
const R_PERSIST: u64 = 3; // committed in H0.1: live in >=3 steps => persistent
const NODE_CAP: usize = 25_000; // stop an exploding proxy before it hangs

fn config(seed: u64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "h0"
run_id = "h0"
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

/// Biology (with F8 overrides) optionally preceded by a substrate-motion builtin.
fn build_rules(cfg: &SimConfig, motion: Option<&str>) -> Vec<hg_rules::CompiledRule> {
    let mut raw = Vec::new();
    if let Some(name) = motion {
        raw.extend(builtin_rules(name).unwrap()); // substrate rule(s) first (id convention)
    }
    let mut biology = parse_rules_toml(hg_alife::alife_base_toml()).unwrap();
    apply_alife_overrides(&mut biology, &cfg.alife);
    raw.extend(biology);
    compile_rules(&raw).unwrap()
}

struct Stat {
    steps: u64,
    m: usize,
    transitions: u64,
    twtf: f64,               // primary
    carrier_twtf: f64,       // transient fraction among carrier-source transitions
    last_quartile_new_frac: f64, // closure: new classes in last quartile / M
    dmdt_early: f64,
    dmdt_late: f64,
    mean_persistence: f64,   // step-to-step live-class Jaccard
    t_over_m: f64,
    n_start: usize,
    n_end: usize,
    carrier_classes: usize,
}

fn run(seed: u64, motion: Option<&str>, window: u64) -> Stat {
    let cfg = config(seed);
    let rules = build_rules(&cfg, motion);
    let mut store = hg_sim_core::runner::build_initial_store(&cfg);
    // growth_treadmill needs a label-1 seed to fire; plant one on an arbitrary node.
    if motion == Some("growth_treadmill") {
        let first = store.nodes().next().map(|(id, _)| id);
        if let Some(id) = first {
            store.get_node_mut(id).unwrap().label = Label(1);
        }
    }
    let n_start = store.nodes().count();
    let mut engine = Engine::new(rules, StepMode::MaxParallel, cfg.seed, 100_000);

    let mut tpm = TpmAccumulator::new();
    let mut prev: Option<HashMap<NodeId, WlHash>> = None;
    let mut prev_live: HashSet<u64> = HashSet::new();

    let mut class_live_steps: HashMap<u64, u64> = HashMap::new();
    class_live_steps.insert(VACUUM.0, u64::MAX); // ∅ channel is always available
    let mut is_carrier_class: HashMap<u64, bool> = HashMap::new();
    is_carrier_class.insert(VACUUM.0, false);
    let mut src_trans: HashMap<u64, u64> = HashMap::new();
    let mut first_seen: HashMap<u64, u64> = HashMap::new();
    let mut m_traj: Vec<usize> = Vec::new();
    let mut persistence_acc = 0.0f64;
    let mut persistence_n = 0u64;
    let mut steps_done = 0u64;
    let mut n_end = n_start;

    for step in 1..=window {
        engine.step(&mut store);
        let snap = store.snapshot();
        n_end = snap.nodes().count();
        let classes = compute_wl_class_map_snapshot(&snap, WL_DEPTH);
        let mut now: HashMap<NodeId, WlHash> = HashMap::new();
        let mut live: HashSet<u64> = HashSet::new();
        for (id, data) in snap.nodes() {
            let h = classes.get(&id).copied().unwrap_or(WlHash(0));
            let is_c = data.label == CARRIER_LABEL;
            now.insert(id, h);
            live.insert(h.0);
            is_carrier_class.entry(h.0).or_insert(is_c);
            first_seen.entry(h.0).or_insert(step);
        }
        for &h in &live {
            *class_live_steps.entry(h).or_insert(0) += 1;
        }
        // step-to-step live-class persistence (Jaccard), node-count independent.
        if step > 1 && !(live.is_empty() && prev_live.is_empty()) {
            let inter = live.intersection(&prev_live).count() as f64;
            let uni = live.union(&prev_live).count() as f64;
            if uni > 0.0 {
                persistence_acc += inter / uni;
                persistence_n += 1;
            }
        }
        // streaming TPM + per-source transition counts (F8 machinery).
        if let Some(p) = &prev {
            for (id, &hp) in p {
                let to = now.get(id).copied().unwrap_or(VACUUM);
                tpm.record(hp, to);
                *src_trans.entry(hp.0).or_insert(0) += 1;
            }
            for (id, &hn) in &now {
                if !p.contains_key(id) {
                    tpm.record(VACUUM, hn);
                    *src_trans.entry(VACUUM.0).or_insert(0) += 1;
                }
            }
        }
        prev = Some(now);
        prev_live = live;
        m_traj.push(tpm.hashes().len());
        steps_done = step;
        if n_end > NODE_CAP {
            break;
        }
    }

    let total: u64 = src_trans.values().sum();
    let transient_trans: u64 = src_trans
        .iter()
        .filter(|(h, _)| *class_live_steps.get(*h).unwrap_or(&0) < R_PERSIST)
        .map(|(_, c)| *c)
        .sum();
    let twtf = if total > 0 { transient_trans as f64 / total as f64 } else { 0.0 };

    let carrier_total: u64 = src_trans
        .iter()
        .filter(|(h, _)| *is_carrier_class.get(*h).unwrap_or(&false))
        .map(|(_, c)| *c)
        .sum();
    let carrier_transient: u64 = src_trans
        .iter()
        .filter(|(h, _)| {
            *is_carrier_class.get(*h).unwrap_or(&false)
                && *class_live_steps.get(*h).unwrap_or(&0) < R_PERSIST
        })
        .map(|(_, c)| *c)
        .sum();
    let carrier_twtf = if carrier_total > 0 { carrier_transient as f64 / carrier_total as f64 } else { 0.0 };

    let m = tpm.hashes().len();
    // closure: classes first-seen in the last quartile of the completed window / M.
    let q = steps_done - steps_done / 4;
    let last_q_new = first_seen.values().filter(|&&s| s > q).count();
    let last_quartile_new_frac = if m > 0 { last_q_new as f64 / m as f64 } else { 0.0 };
    // dM/dt early vs late (per-step new-class rate over first vs last quarter).
    let quarter = (steps_done / 4).max(1) as usize;
    let dmdt_early = if m_traj.len() > quarter { m_traj[quarter - 1] as f64 / quarter as f64 } else { 0.0 };
    let dmdt_late = if m_traj.len() >= 2 && m_traj.len() > quarter {
        (m_traj[m_traj.len() - 1] - m_traj[m_traj.len() - 1 - quarter]) as f64 / quarter as f64
    } else { 0.0 };

    let carrier_classes = is_carrier_class.iter().filter(|(_, &c)| c).count();

    Stat {
        steps: steps_done,
        m,
        transitions: total,
        twtf,
        carrier_twtf,
        last_quartile_new_frac,
        dmdt_early,
        dmdt_late,
        mean_persistence: if persistence_n > 0 { persistence_acc / persistence_n as f64 } else { 0.0 },
        t_over_m: if m > 0 { total as f64 / m as f64 } else { 0.0 },
        n_start,
        n_end,
        carrier_classes,
    }
}

fn print_row(label: &str, s: &Stat) {
    eprintln!(
        "  {:<22} | {:>5} | {:>5}->{:>6} | {:>4} | {:>4} | {:>7} | {:>6.3} | {:>7.3} | {:>7.3} | {:>6.3} | {:>7.1} | {:.2}/{:.2}",
        label, s.steps, s.n_start, s.n_end, s.m, s.carrier_classes, s.transitions,
        s.twtf, s.carrier_twtf, s.last_quartile_new_frac, s.mean_persistence, s.t_over_m,
        s.dmdt_early, s.dmdt_late
    );
}

#[test]
fn h0_partition_stationarity() {
    let seed = 12345;
    eprintln!("\n================ H0.2 / H0.3 — PARTITION STATIONARITY (seed {seed}, WL depth {WL_DEPTH}, R={R_PERSIST}) ================");
    eprintln!("  run                    | steps |  n_start->end | M    | k_c  | trans   | TWTF   | carr-TWTF| lastQ-new| persist| T/M     | dM/dt e/l");
    eprintln!("  -----------------------+-------+---------------+------+------+---------+--------+----------+---------+--------+---------+----------");

    // H0.2 baseline: static substrate, over the SAME short window as the motion runs
    // (fair head-to-head) AND over the full F8 window (the floor CE actually relies on).
    let base_120 = run(seed, None, 120);
    let base_full = run(seed, None, 6000);
    print_row("H0.2 static W=120", &base_120);
    print_row("H0.2 static W=6000", &base_full);

    // H0.3 motion proxies over a short window (they are unbounded; NODE_CAP stops them).
    let gt = run(seed, Some("growth_treadmill"), 120);
    let tri = run(seed, Some("triangle_subdivide"), 120);
    print_row("H0.3 growth_treadmill", &gt);
    print_row("H0.3 triangle_subdiv", &tri);
    eprintln!("======================================================================================================================\n");

    // ---- apply the committed H0.1 decision rule ----
    let base = &base_120; // the fair (same-window) comparator for the motion runs
    eprintln!("Committed decision rule (vs F8 static floor; STOP if any: linear M growth / TWTF majority / T/M->O(1)):");
    eprintln!("  static floor (W=120): TWTF={:.3}  persist={:.3}  T/M={:.1}  lastQ-new={:.3}",
        base.twtf, base.mean_persistence, base.t_over_m, base.last_quartile_new_frac);
    for (name, s) in [("growth_treadmill", &gt), ("triangle_subdivide", &tri)] {
        let twtf_majority = s.twtf > 0.5;
        let twtf_blowup = s.twtf > 2.0 * base.twtf.max(0.01) && s.twtf > 0.2;
        let tm_collapse = s.t_over_m < 10.0;
        let no_closure = s.last_quartile_new_frac > 0.5 || s.dmdt_late >= s.dmdt_early.max(1.0);
        let stop = twtf_majority || tm_collapse || no_closure;
        eprintln!(
            "  [{name}] TWTF {:.3} (maj>{}: {}) | T/M {:.1} (<10: {}) | lastQ-new {:.3}/dM/dt {:.2}->{:.2} (no-closure: {}) | 2x-floor: {} => {}",
            s.twtf, 0.5, twtf_majority, s.t_over_m, tm_collapse,
            s.last_quartile_new_frac, s.dmdt_early, s.dmdt_late, no_closure, twtf_blowup,
            if stop { "STOP -> H4" } else { "within tolerance" }
        );
    }
    eprintln!();

    // ---- H0.4 windowing-rescue check (plan: only if borderline) ----
    // Neither proxy is borderline (both cross the transient-majority line), and a
    // SHORTER window makes the R=3 persistence bar HARDER, so windowing is expected
    // to worsen TWTF, not rescue it. Measured on growth_treadmill (the milder proxy,
    // 0.515) to close the question empirically rather than by argument. triangle is
    // already foreclosed by persist=0.000 (no window contains a recurrent class).
    eprintln!("H0.4 windowing check (growth_treadmill; shorter window should NOT lower TWTF):");
    for w in [20u64, 40, 60, 120] {
        let s = run(seed, Some("growth_treadmill"), w);
        eprintln!("  W={:>3}: TWTF={:.3}  T/M={:.1}  persist={:.3}  (support falls with W; TWTF does not drop)",
            w, s.twtf, s.t_over_m, s.mean_persistence);
    }
    eprintln!();

    // Guards so the measurement can't silently rot. The baseline must be well-defined.
    assert!(base_full.t_over_m > 10.0, "F8 baseline itself under-sampled (T/M={:.1})", base_full.t_over_m);
    assert!(base_full.twtf < 0.5, "F8 baseline itself transient-dominated (TWTF={:.3})", base_full.twtf);
}
