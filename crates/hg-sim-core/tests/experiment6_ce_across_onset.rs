//! Experiment 6 — causal emergence across the replication-onset boundary.
//!
//! Runs the pre-registered measurement (`docs/experiment6_preregistration.md`): the
//! carrier/substrate partition's CE, measured in a paired bare-vs-live pair at the
//! pinned establishing seed 12345. The micro state space is the WL-class TPM; the
//! macro partition groups classes by `motif_class` (substrate 0 / carrier ≠0). The
//! boundary is the presence/absence of the `replicate` process.
//!
//! Predictions under test (committed BEFORE this ran):
//!   P1  CE_bare = 0 (no carrier classes ⟹ K=1 degenerate) and CE_live > 0.
//!   P2  CE_live < 1 expected and NOT a failure (Finding F1: CE biased downward).
//!   F-B CE_bare > 0 would be an artifact — asserted against.
//!   F-C below the 1000-transition gate ⟹ no claim.
//! Nothing here is tuned to raise CE; a result is a result.
#![cfg(feature = "alife")]

use hg_causal::{CausalObserver, CoarseStrategy, EiResult, EiSettings};
use hg_core::compute_wl_class_map_snapshot;
use hg_engine::Engine;
use hg_rules::StepMode;
use hg_sim_core::SimConfig;

// Cadence = 1 step. This is a MEASUREMENT-DESIGN decision forced by the biology, not
// a tuning knob: carriers live and die on a ~tens-of-steps timescale in the turnover
// regime, so at the substrate window cadence (128) they never survive two consecutive
// observations and never register a node-class transition — the micro TPM degenerates
// to the 36 static substrate self-loops and the carrier/substrate partition can see no
// carrier class (macro_K collapses to 1). Per-step cadence resolves carrier lifetimes,
// so the carrier classes actually enter the micro dynamics. The cadence is part of the
// measurement's definition (every EiResult carries it).
const CADENCE: u64 = 1;
const MIN_TRANSITIONS: u64 = 1000;

fn config(alife: bool, steps: u64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "exp6"
run_id = "exp6"
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

fn ei_settings() -> EiSettings {
    EiSettings {
        interval_steps: CADENCE,
        min_transitions: MIN_TRANSITIONS,
        cadence_steps: CADENCE,
        try_structural: false,
        try_energetic: false,
        try_functional: false,
        try_carrier_substrate: true, // the Experiment-6 partition
        try_optimal: false,
    }
}

/// Drive the causal observer window-by-window (mirroring the runner's
/// `process_window_with_classes` cadence). Returns the final EiResults plus a
/// per-observation trace of the carrier/substrate CE and macro-K (for the P3
/// temporal view). The trace is sampled every `sample_every` observations to keep
/// it cheap.
fn run_arm(alife: bool, steps: u64, sample_every: u64) -> (Vec<EiResult>, Vec<(u64, f64, usize)>) {
    let cfg = config(alife, steps);
    let rules = hg_sim_core::runner::load_rules(&cfg).unwrap();
    let mut store = hg_sim_core::runner::build_initial_store(&cfg);
    let mut engine = Engine::new(rules, StepMode::MaxParallel, cfg.seed, 100000);

    let mut causal = CausalObserver::new()
        .with_classifier(Box::new(hg_alife::HypergraphMotifClassifier::new()))
        .with_ei_settings(ei_settings());

    let mut window_id = 0u64;
    let mut trace = Vec::new();
    for step in 1..=steps {
        engine.step(&mut store);
        if step % CADENCE == 0 {
            let snapshot = store.snapshot();
            let classes = compute_wl_class_map_snapshot(&snapshot, 1);
            causal.process_window_with_classes(&snapshot, window_id, &classes);
            if window_id % sample_every == 0 {
                if let Some(r) = carrier_substrate(&causal.latest_ei) {
                    trace.push((step, r.ce_coefficient, r.macro_classes));
                }
            }
            window_id += 1;
        }
    }
    (causal.latest_ei.clone(), trace)
}

fn carrier_substrate(results: &[EiResult]) -> Option<&EiResult> {
    results.iter().find(|r| r.strategy == CoarseStrategy::CarrierSubstrate)
}

#[test]
fn experiment6_ce_across_replication_onset() {
    let steps = 6_000u64;
    let (live, live_trace) = run_arm(true, steps, 400);
    let (bare, _) = run_arm(false, steps, 400);

    eprintln!("\n================ EXPERIMENT 6 — CE across replication onset ================");
    eprintln!("carrier/substrate partition, seed 12345, cadence {CADENCE}, {steps} steps, gate {MIN_TRANSITIONS} transitions\n");

    let report = |name: &str, r: Option<&EiResult>| match r {
        Some(r) => eprintln!(
            "  {name:<5}: CE={:.4}  EI_micro={:.4}  EI_macro={:.4}  macro_K={}  micro_M={}  transitions={}",
            r.ce_coefficient, r.ei_micro, r.ei_macro, r.macro_classes, r.micro_classes, r.transition_count
        ),
        None => eprintln!("  {name:<5}: (below the {MIN_TRANSITIONS}-transition gate — no CE claim)"),
    };
    report("bare", carrier_substrate(&bare));
    report("live", carrier_substrate(&live));
    eprintln!("===========================================================================\n");

    // F-C: the LIVE arm must clear the transition gate, else no claim is made.
    let live_ce = carrier_substrate(&live)
        .expect("live arm cleared the transition gate (F-C) — carrier/substrate CE is defined");

    // F-B: the bare arm has NO carrier classes ⟹ partition degenerate (K=1) ⟹ CE=0.
    // If the bare arm is below the gate it emits nothing (also fine: CE not defined,
    // certainly not > 0). If it IS present, it must be exactly the degenerate 0.
    if let Some(bare_ce) = carrier_substrate(&bare) {
        assert_eq!(
            bare_ce.macro_classes, 1,
            "bare arm must be degenerate (K=1, no carrier classes) — else F-B artifact"
        );
        assert_eq!(bare_ce.ce_coefficient, 0.0, "F-B: bare CE must be exactly 0 (degenerate baseline)");
    }

    // P1: with carriers present the partition is a genuine 2-group coarse-graining.
    assert_eq!(live_ce.macro_classes, 2, "live arm must have a real carrier+substrate 2-group partition");
    assert!(live_ce.ei_micro > 0.0, "P4: live micro dynamics are non-degenerate");
    assert!(live_ce.transition_count >= MIN_TRANSITIONS, "F-C: gate cleared");

    // CAVEAT (findings §1), asserted so it can't be forgotten: EI_macro is PINNED to
    // log2(K) = 1.0 because surviving nodes never cross the carrier/substrate boundary
    // (a substrate node never becomes a carrier; a carrier stays a carrier until it
    // dies), so the 2x2 macro TPM is exactly the identity. CE_live is therefore just
    // 1/EI_micro and does NOT quantify replicator turnover dynamics (which live in
    // birth/death, outside the surviving-node transition model). The 0→>0 bridge is
    // real; the CE magnitude is bounded by this. See docs/experiment6_findings.md.
    assert!((live_ce.ei_macro - 1.0).abs() < 1e-9, "EI_macro is the pinned identity value log2(2)=1");
    assert!((live_ce.ce_coefficient - 1.0 / live_ce.ei_micro).abs() < 1e-9, "CE reduces to 1/EI_micro");

    // The RESULT is reported, not asserted to a target (Finding F1: do not tune to CE>1).
    // The only committed structural assertion is P1's 0→>0 bridge vs the bare baseline:
    // the live partition is non-degenerate (K=2) where the bare one is not.
    eprintln!(
        "RESULT: CE_live = {:.4} (EI_macro {:.4} / EI_micro {:.4}); bare baseline degenerate (K=1, CE=0).",
        live_ce.ce_coefficient, live_ce.ei_macro, live_ce.ei_micro
    );
    if live_ce.ce_coefficient > 1.0 {
        eprintln!("  CE_live > 1 clears the conservative (downward-biased) bar — strong positive (F1).");
    } else if live_ce.ce_coefficient > 0.0 {
        eprintln!("  0 < CE_live < 1: replication onset yields macro causal structure below the");
        eprintln!("  conservative bar — the expected P2 outcome; CE<1 falsifies nothing (F1).");
    } else {
        eprintln!("  CE_live ≈ 0: HONEST NEGATIVE (F-A) — the carrier/substrate partition captures no");
        eprintln!("  macro causal emergence across the boundary. Reported, not repaired.");
    }

    // P3 (secondary, within-run temporal corroboration): CE rises from the degenerate
    // baseline to its live value as replication establishes inside a SINGLE run.
    eprintln!("\n  P3 temporal (live run): step → (CE, macro_K)");
    for (step, ce, k) in &live_trace {
        eprintln!("    step {step:>5}: CE {ce:.4}  K={k}");
    }
    // The earliest sampled observation should still be degenerate/near-zero (few or no
    // carriers surviving cadence-to-cadence yet), and the last should match the final.
    if let (Some(first), Some(last)) = (live_trace.first(), live_trace.last()) {
        assert!(first.1 <= last.1 + 1e-9 || first.2 < last.2,
            "CE should rise (or the partition should become non-degenerate) as replication establishes");
    }
}

/// F-D (hard gate): CE_live is reproducible. Same seed, two independent runs.
///
/// The measurement is deterministic where it counts: the `transition_count` and the
/// underlying TPM are BIT-identical (engine rewriting + WL hashing are deterministic;
/// WL cross-process stability is proven in `wl_cross_process`). CE itself is
/// reproducible to floating-point precision but NOT bit-identical, because
/// `TpmAccumulator` assigns dense class indices in hashmap-iteration order (documented
/// in `ei_dump`), and EI is permutation-invariant only *mathematically* — the summation
/// order in `coarse_grain`/`ei_from_tpm` differs by that permutation, jittering the last
/// ULP of `EI_macro` (≈1.0) and hence CE. The scientific value is reproducible to ~15
/// digits; the last ULP is FP associativity, not scientific nondeterminism.
#[test]
fn experiment6_ce_is_deterministic() {
    let steps = 3_000u64;
    let (a, _) = run_arm(true, steps, 100_000);
    let (b, _) = run_arm(true, steps, 100_000);
    let ca = carrier_substrate(&a).expect("run a defined");
    let cb = carrier_substrate(&b).expect("run b defined");
    // The real determinism gate: the TPM (transition count) is bit-identical.
    assert_eq!(ca.transition_count, cb.transition_count, "transition count must be bit-reproducible");
    assert_eq!(ca.macro_classes, cb.macro_classes, "partition K reproducible");
    // CE / EI reproducible to FP precision (last-ULP jitter from index-permutation only).
    let rel = |x: f64, y: f64| (x - y).abs() / x.abs().max(1e-300);
    assert!(rel(ca.ce_coefficient, cb.ce_coefficient) < 1e-12, "CE reproducible to FP precision");
    assert!(rel(ca.ei_micro, cb.ei_micro) < 1e-12, "EI_micro reproducible to FP precision");
    assert!(rel(ca.ei_macro, cb.ei_macro) < 1e-12, "EI_macro reproducible to FP precision");
}
