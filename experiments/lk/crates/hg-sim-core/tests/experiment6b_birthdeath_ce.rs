//! Experiment 6b — birth/death-aware CE, fixing the identity macro of 6a.
//!
//! Pre-registration: `docs/experiment6b_preregistration.md`. 6a's carrier/substrate macro
//! TPM is exactly the identity (surviving nodes never cross the boundary), so `EI_macro` is
//! pinned to log2(K) and CE = 1/EI_micro. 6b augments the micro dynamics with a VACUUM state
//! ∅: births are ∅→carrier and deaths are carrier→∅, so the macro TPM gains real off-diagonal
//! (birth/death) mass and EI_macro is no longer pinned. Everything is self-contained here
//! (own TpmAccumulator); the core CausalObserver is untouched (exit-criterion 7).
//!
//! Committed predictions: P1 EI_macro_live < log2(K); P2 EI_macro decreases with cadence;
//! P3 CE_bare=0 & CE_live>0; F1 no tuning to CE>1.
#![cfg(feature = "alife")]

use hg_alife::CARRIER_LABEL;
use hg_causal::{coarse_grain, ei_from_tpm, TpmAccumulator};
use hg_core::{compute_wl_class_map_snapshot, NodeId, WlHash};
use hg_engine::Engine;
use hg_rules::StepMode;
use hg_sim_core::SimConfig;
use std::collections::HashMap;

/// Reserved vacuum class ∅. Real WL hashes come from FxHasher; a collision with this
/// sentinel is astronomically unlikely and asserted against.
const VACUUM: WlHash = WlHash(0xE6_0000_0000_0000);
const MIN_TRANSITIONS: u64 = 1000;

fn config(alife: bool, steps: u64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "exp6b"
run_id = "exp6b"
seed = 12345
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
enabled = {alife}
seed_carrier_marker = {alife}
initial_substrate_energy = 5.0
influx_rate = 0.34
bootstrap_replication_threshold = 3.0
"#
    );
    SimConfig::from_toml(&toml).unwrap()
}

struct Outcome {
    ce: f64,
    ei_micro: f64,
    ei_macro: f64,
    macro_k: usize,
    micro_m: usize,
    transitions: u64,
    births: u64,
    deaths: u64,
}

/// Run one arm, sampling node classes every `cadence` steps, and build a VACUUM-augmented
/// class-transition TPM (surviving + born(∅→c) + died(c→∅)). Returns the carrier/substrate
/// CE where the carrier macro group is {classes whose witness is a carrier} and the
/// substrate group is {substrate classes ∪ ∅}.
fn run_arm(alife: bool, steps: u64, cadence: u64) -> Outcome {
    let cfg = config(alife, steps);
    let rules = hg_sim_core::runner::load_rules(&cfg).unwrap();
    let mut store = hg_sim_core::runner::build_initial_store(&cfg);
    let mut engine = Engine::new(rules, StepMode::MaxParallel, cfg.seed, 100000);

    let mut tpm = TpmAccumulator::new();
    // class hash -> is that class a carrier class? (learned from any witness node's label)
    let mut is_carrier_class: HashMap<u64, bool> = HashMap::new();
    is_carrier_class.insert(VACUUM.0, false); // ∅ is grouped with substrate
    let mut prev: Option<HashMap<NodeId, WlHash>> = None;
    let (mut births, mut deaths) = (0u64, 0u64);

    for step in 1..=steps {
        engine.step(&mut store);
        if step % cadence != 0 {
            continue;
        }
        let snapshot = store.snapshot();
        let classes = compute_wl_class_map_snapshot(&snapshot, 1);
        let mut now: HashMap<NodeId, WlHash> = HashMap::new();
        for (id, data) in snapshot.nodes() {
            let h = classes.get(&id).copied().unwrap_or(WlHash(0));
            assert_ne!(h, VACUUM, "real WL class collided with the ∅ sentinel");
            now.insert(id, h);
            is_carrier_class.entry(h.0).or_insert(data.label == CARRIER_LABEL);
        }
        if let Some(prev) = &prev {
            // Surviving + died.
            for (id, &hp) in prev {
                match now.get(id) {
                    Some(&hn) => tpm.record(hp, hn),           // survived
                    None => { tpm.record(hp, VACUUM); deaths += 1; } // died → ∅
                }
            }
            // Born (∅ → class).
            for (id, &hn) in &now {
                if !prev.contains_key(id) {
                    tpm.record(VACUUM, hn);
                    births += 1;
                }
            }
        }
        prev = Some(now);
    }

    // Micro EI over the full augmented TPM.
    let rows = tpm.normalized_rows();
    let bare: Vec<Vec<f64>> = rows.iter().map(|(_, r)| r.clone()).collect();
    let micro = ei_from_tpm(&bare);

    // Carrier/substrate partition with compact indices (∅ ∈ substrate group).
    let mut group_map: HashMap<usize, usize> = HashMap::new();
    let group_of: Vec<usize> = tpm
        .hashes()
        .iter()
        .map(|h| {
            let raw = *is_carrier_class.get(&h.0).unwrap_or(&false) as usize;
            let next = group_map.len();
            *group_map.entry(raw).or_insert(next)
        })
        .collect();
    let k = group_map.len().max(1);
    let macro_rows = coarse_grain(&rows, &group_of, k);
    let mac = ei_from_tpm(&macro_rows);
    let ce = if micro.ei > 0.0 { mac.ei / micro.ei } else { 0.0 };

    Outcome {
        ce,
        ei_micro: micro.ei,
        ei_macro: mac.ei,
        macro_k: k,
        micro_m: micro.m,
        transitions: tpm.total_transitions(),
        births,
        deaths,
    }
}

#[test]
fn experiment6b_birthdeath_ce_across_cadence() {
    let steps = 8_000u64;
    let cadences = [1u64, 4, 16, 64];

    eprintln!("\n============ EXPERIMENT 6b — birth/death-aware CE (identity-macro fix) ============");
    eprintln!("vacuum-augmented carrier/substrate partition, seed 12345, {steps} steps, gate {MIN_TRANSITIONS}\n");
    eprintln!("  Δ   | arm  | CE       EI_micro  EI_macro  log2K-EImac | K  M   births deaths  trans");
    eprintln!("  ----+------+---------------------------------------------------------------------");

    let mut live_ei_macro = Vec::new();
    for &cad in &cadences {
        let live = run_arm(true, steps, cad);
        let bare = run_arm(false, steps, cad);
        let log2k = (live.macro_k as f64).log2();
        let gap = log2k - live.ei_macro;
        eprintln!(
            "  {cad:>3} | live | {:.4}   {:.4}    {:.4}    {:+.4}     | {}  {:<3} {:>6} {:>6} {:>7}",
            live.ce, live.ei_micro, live.ei_macro, gap, live.macro_k, live.micro_m, live.births, live.deaths, live.transitions
        );
        eprintln!(
            "      | bare | {:.4}   {:.4}    {:.4}               | {}  {:<3} {:>6} {:>6} {:>7}",
            bare.ce, bare.ei_micro, bare.ei_macro, bare.macro_k, bare.micro_m, bare.births, bare.deaths, bare.transitions
        );

        // P3: bridge preserved — bare degenerate, live non-degenerate.
        assert_eq!(bare.macro_k, 1, "P3/F-B: bare arm has no carriers ⟹ K=1 degenerate");
        assert_eq!(bare.ce, 0.0, "P3/F-B: bare CE must be exactly 0");
        assert!(bare.births == 0 && bare.deaths == 0, "bare static substrate has no births/deaths");
        if live.transitions >= MIN_TRANSITIONS {
            assert_eq!(live.macro_k, 2, "P3: live arm has a real 2-group partition");
            assert!(live.births > 0 && live.deaths > 0, "live arm has turnover");
            live_ei_macro.push((cad, live.ei_macro, log2k));
        }
    }
    eprintln!("==================================================================================\n");

    // P1 (the headline fix): EI_macro is NO LONGER pinned to log2(K). At the largest cadence
    // the birth/death mass must pull EI_macro strictly below log2(K).
    let (cad_max, ei_macro_max, log2k_max) = *live_ei_macro.last().expect("some cadence cleared the gate");
    let gap_max = log2k_max - ei_macro_max;
    eprintln!("P1: at Δ={cad_max}, log2(K)-EI_macro = {gap_max:.4} (6a had this pinned at 0 exactly).");
    assert!(
        gap_max > 1e-6,
        "P1 FAILED (F-A honest negative): EI_macro still ≈ log2(K) even at Δ={cad_max} — node-class CE \
         cannot capture turnover; a population-level observable is needed. gap={gap_max:.2e}"
    );

    // P2 (turnover dependence): the gap grows with cadence (more birth/death per persistence).
    if live_ei_macro.len() >= 2 {
        let (_, ei_lo, log2k_lo) = live_ei_macro[0];
        let gap_lo = log2k_lo - ei_lo;
        eprintln!("P2: gap(Δ={}) = {gap_lo:.4}  →  gap(Δ={cad_max}) = {gap_max:.4}", live_ei_macro[0].0);
        assert!(gap_max > gap_lo, "P2: EI_macro departs further from log2(K) as cadence grows (encodes turnover)");
    }

    eprintln!("\n⟹ FIX CONFIRMED: including birth/death (∅) transitions un-pins EI_macro from log2(K);");
    eprintln!("  the macro now encodes replicator turnover. CE reported honestly (Finding F1: not tuned).");
}

/// F-D (pre-registered hard gate): birth/death counts and the resulting CE are
/// reproducible. Integer event counts are bit-identical (engine determinism); CE is
/// reproducible to FP precision (same index-permutation caveat as 6a).
#[test]
fn experiment6b_is_deterministic() {
    let (a, b) = (run_arm(true, 3_000, 4), run_arm(true, 3_000, 4));
    assert_eq!(a.births, b.births, "births bit-reproducible");
    assert_eq!(a.deaths, b.deaths, "deaths bit-reproducible");
    assert_eq!(a.transitions, b.transitions, "transition count bit-reproducible");
    assert_eq!(a.macro_k, b.macro_k, "partition K reproducible");
    let rel = |x: f64, y: f64| (x - y).abs() / x.abs().max(1e-300);
    assert!(rel(a.ce, b.ce) < 1e-12, "CE reproducible to FP precision");
}
