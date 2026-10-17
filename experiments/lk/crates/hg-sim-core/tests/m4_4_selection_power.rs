//! V8 / M4.4 — power the selection question to >= 30 ESTABLISHED seeds.
//!
//! The criterion-5 validation (alife_criterion5_selection_vs_drift.rs) ran 10 seeds
//! and found the repl-threshold shift is INSIDE the drift envelope (~1.2x), leaving
//! criterion 5 "met degenerate-only". This powers that to >= 30 established seeds so
//! the drift envelope and any selection signal are estimated with enough seeds to be
//! trusted rather than asserted.
//!
//! Pre-registration: docs/validation_v8_m4_4_preregistration.md (commit c9a31ee).
//! Drift null = pooled RMS displacement of the two INERT traits (harvest_rate,
//! maintenance_cost -- read by no rule). Reading fixed there:
//!   SELECTION iff magnitude ratio >= 2.0 AND sign fraction >= 0.80.
//!   DRIFT-DOMINATED iff ratio < 2.0 OR sign fraction < 0.65.
//!   null voided if the inert traits are themselves sign-biased beyond 40-60%.
#![cfg(feature = "alife")]

use hg_alife::AlifeObserver;
use hg_core::GraphSnapshot;
use hg_engine::Engine;
use hg_observe::{Observer, RewriteEvent, StepStats};
use hg_rules::StepMode;
use hg_sim_core::SimConfig;
use std::sync::{Arc, Mutex};

const TRAIT_NAMES: [&str; 4] = ["harvest_rate", "replication_threshold", "maintenance_cost", "satiation"];
const INERT: [usize; 2] = [0, 2];
const REPL_THR: usize = 1;
const SATIATION: usize = 3;
const STEPS: u64 = 15_000;
const TARGET_ESTABLISHED: usize = 30;
const MAX_ATTEMPTS: u64 = 200;

fn exp5_config(seed: u64, steps: u64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "m44"
run_id = "m44"
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
max_pairings_per_step = 100000
[evaluator]
enabled = false
[causal]
enabled = false
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

struct Handle { alife: Arc<Mutex<AlifeObserver>> }
impl Observer for Handle {
    fn on_step_end(&mut self, snap: &GraphSnapshot<'_>, s: &StepStats, events: &[RewriteEvent]) {
        self.alife.lock().unwrap().on_step_end(snap, s, events);
    }
}

fn trait_displacements(seed: u64, steps: u64) -> Option<[f64; 4]> {
    let config = exp5_config(seed, steps);
    let rules = hg_sim_core::runner::load_rules(&config).unwrap();
    let mut store = hg_sim_core::runner::build_initial_store(&config);
    let mut engine = Engine::new(rules, StepMode::MaxParallel, config.seed, 100000);
    let alife = Arc::new(Mutex::new(AlifeObserver::new(500)));
    engine.add_observer(Box::new(Handle { alife: alife.clone() }));
    for _ in 0..steps { engine.step(&mut store); }
    drop(engine);
    let a = alife.lock().unwrap();
    let win = a.windows();
    let first = win.iter().position(|w| w.carriers > 0)?;
    let last = win.iter().rposition(|w| w.carriers > 0)?;
    if first == last { return None; }
    let mut disp = [0.0; 4];
    for t in 0..4 { disp[t] = win[last].trait_mean[t] - win[first].trait_mean[t]; }
    Some(disp)
}

fn rms(xs: &[f64]) -> f64 {
    if xs.is_empty() { return 0.0; }
    (xs.iter().map(|x| x * x).sum::<f64>() / xs.len() as f64).sqrt()
}
fn frac_neg(xs: &[f64]) -> f64 {
    if xs.is_empty() { return 0.0; }
    xs.iter().filter(|x| **x < 0.0).count() as f64 / xs.len() as f64
}
/// The null-validity guard: the INERT traits must be ~50/50 in sign, or the drift null is
/// itself biased and no selection verdict may be read.
///
/// Extracted from the inline expression it used to be so that C4.2 can drive it into
/// REJECTION on constructed input. M4.3's guard audit logged it as gap 2 — the check passed
/// on the real run because the null happened to be clean (43%/53%), and by V8 rule 5 a guard
/// with no test in which it rejects is untested. That is exactly the §6.1 shape.
fn null_clean(inert_fdown: &[f64]) -> bool {
    inert_fdown.iter().all(|f| (0.40..=0.60).contains(f))
}

/// exact two-sided binomial sign test against p = 1/2
fn sign_test_p(k: usize, n: usize) -> f64 {
    if n == 0 { return 1.0; }
    let mut pmf = vec![0.0f64; n + 1];
    let mut c = 1.0f64;
    let denom = 2f64.powi(n as i32);
    for i in 0..=n { pmf[i] = c / denom; c = c * (n - i) as f64 / (i + 1) as f64; }
    let obs = pmf[k];
    pmf.iter().filter(|&&v| v <= obs * (1.0 + 1e-12)).sum::<f64>().min(1.0)
}

#[test]
fn m4_4_selection_power_30_seeds() {
    eprintln!("\n================ V8 / M4.4 — POWERED SELECTION TEST ({TARGET_ESTABLISHED} established seeds, {STEPS} steps) ================");
    let mut disp: Vec<[f64; 4]> = Vec::new();
    let mut attempts = 0u64;
    for seed in 1..=MAX_ATTEMPTS {
        attempts += 1;
        if let Some(d) = trait_displacements(seed, STEPS) { disp.push(d); }
        if disp.len() >= TARGET_ESTABLISHED { break; }
    }
    let n = disp.len();
    eprintln!("  attempts {attempts}, ESTABLISHED {n} ({:.0}%) — the turnover regime is seed-fragile (F6.2)",
        100.0 * n as f64 / attempts as f64);

    // per-trait displacement vectors
    let col = |t: usize| -> Vec<f64> { disp.iter().map(|d| d[t]).collect() };
    let inert_all: Vec<f64> = INERT.iter().flat_map(|&t| col(t)).collect();
    let drift_env = rms(&inert_all);
    let repl = col(REPL_THR);
    let sat = col(SATIATION);

    eprintln!("\n  per-trait displacement (last populated window - first), across {n} established seeds:");
    eprintln!("    trait                  | mean Δ   | RMS |Δ| | frac-down | ratio to drift | sign p");
    eprintln!("    -----------------------+----------+---------+-----------+----------------+--------");
    for (t, name) in TRAIT_NAMES.iter().enumerate() {
        let c = col(t);
        let mean = c.iter().sum::<f64>() / n as f64;
        let r = rms(&c);
        let fneg = frac_neg(&c);
        let k_dir = c.iter().filter(|x| (**x < 0.0) == (fneg >= 0.5)).count();
        let tag = if INERT.contains(&t) { " (NULL)" } else if t == REPL_THR { " (free)" } else { " (trade-off)" };
        eprintln!("    {:<22}{} | {:>+8.2} | {:>7.2} | {:>8.0}% | {:>14} | {:.3}",
            name, tag, mean, r, 100.0 * fneg,
            if INERT.contains(&t) { "  (is null)".to_string() } else { format!("{:.2}", r / drift_env) },
            sign_test_p(k_dir, n));
    }

    // null validity: the inert traits must be ~50/50 in sign
    let inert_fdown: Vec<f64> = INERT.iter().map(|&t| frac_neg(&col(t))).collect();
    let null_clean = null_clean(&inert_fdown);
    eprintln!("\n  DRIFT NULL: envelope (RMS of both inert traits pooled) = {drift_env:.2}");
    eprintln!("    inert-trait sign balance (frac-down): {:?}  => null {}",
        inert_fdown.iter().map(|f| format!("{:.0}%", 100.0 * f)).collect::<Vec<_>>(),
        if null_clean { "CLEAN" } else { "*** BIASED — verdict withheld ***" });

    // ---------- pre-committed reading, applied to satiation (the plan's target) ----------
    let sat_ratio = rms(&sat) / drift_env;
    let sat_fdir = frac_neg(&sat).max(1.0 - frac_neg(&sat));
    let sat_k = sat.iter().filter(|x| (**x < 0.0) == (frac_neg(&sat) >= 0.5)).count();
    let sat_signp = sign_test_p(sat_k, n);
    let repl_ratio = rms(&repl) / drift_env;
    let repl_fdown = frac_neg(&repl);

    eprintln!("\n================ M4.4 — PRE-COMMITTED READING (rule fixed in c9a31ee) ================");
    if !null_clean {
        eprintln!("  NULL BIASED: the inert traits are themselves sign-skewed, so the drift null is not clean and");
        eprintln!("  NO selection verdict is read (pre-committed). Reported as the finding.");
    } else {
        eprintln!("  TARGET satiation (trade-off): magnitude ratio {sat_ratio:.2} (needs >=2.0), sign fraction {sat_fdir:.2}");
        eprintln!("    (needs >=0.80), sign-test p {sat_signp:.3}");
        if sat_ratio >= 2.0 && sat_fdir >= 0.80 && sat_signp < 0.05 {
            eprintln!("    => SELECTION on satiation: a reproducible directional shift beyond drift. Criterion 5 is met");
            eprintln!("       on STRONGER footing than 'degenerate-only'.");
        } else if sat_ratio < 2.0 || sat_fdir < 0.65 {
            eprintln!("    => DRIFT-DOMINATED. satiation's shift is inside the drift envelope / not reproducibly");
            eprintln!("       directional. Criterion 5 stays MET DEGENERATE-ONLY, now confirmed on {n} seeds not 10.");
        } else {
            eprintln!("    => INTERMEDIATE: bounded by the numbers above, not forced to a branch.");
        }
        eprintln!("  (context) replication_threshold (free trait): ratio {repl_ratio:.2}, frac-down {:.0}% — the",
            100.0 * repl_fdown);
        eprintln!("    known sign bias; magnitude {} the drift envelope.",
            if repl_ratio >= 2.0 { "EXCEEDS" } else { "sits INSIDE" });
    }
    eprintln!("======================================================================\n");

    assert!(n >= 10, "too few established seeds ({n}) to characterise drift");
}

/// V-track closeout / C4.2 — GAP 2 CLOSED: a firing test for the null-validity guard.
///
/// Pre-registration: docs/validation_v9_c4_preregistration.md (4fd9df1).
///
/// M4.3's audit logged this guard as the last untested one in the V-track family: it passed
/// on the real run because the inert traits happened to be balanced, and no constructed case
/// existed in which it REJECTS. §6.1's pattern is precisely a guard whose accept condition is
/// never exercised from the failing side, so this drives it from both.
///
/// No simulation is run: the guard is a pure function of the displacement vectors, which is
/// what makes constructed input the right instrument.
#[test]
fn c4_2_null_validity_guard_firing_test() {
    eprintln!("\n================ V-TRACK CLOSEOUT / C4.2 — FIRING TEST FOR M4.4's NULL-VALIDITY GUARD ================");
    eprintln!("  guard: both INERT traits must have frac-down in [0.40, 0.60], else NO selection verdict is read.");
    eprintln!("  M4.3 gap 2: it had never been shown to REJECT. Driven from both sides here.\n");

    // 1. REJECTION on a sign-skewed null — the case the guard exists for.
    let skewed = [0.80, 0.50];
    let r1 = null_clean(&skewed);
    eprintln!("  1. skewed null      inert frac-down {skewed:?} => null_clean = {r1}  (must be false)");
    assert!(!r1, "FIRING TEST FAILED: the guard must REJECT an inert trait skewed to 80% down");

    // a skew in the OTHER direction must reject too — the interval is two-sided
    let skewed_up = [0.50, 0.12];
    let r1b = null_clean(&skewed_up);
    eprintln!("  1b. skewed the other way {skewed_up:?} => null_clean = {r1b}  (must be false)");
    assert!(!r1b, "FIRING TEST FAILED: the guard must reject skew in BOTH directions");

    // 2. The threshold pinned from both sides, so the test shows WHERE the guard sits and
    //    not merely that some extreme input fails it.
    for (v, want) in [([0.40, 0.60], true), ([0.60, 0.40], true),
                      ([0.39, 0.50], false), ([0.50, 0.61], false)] {
        let got = null_clean(&v);
        eprintln!("  2. boundary {v:?} => null_clean = {got}  (want {want})");
        assert_eq!(got, want, "guard threshold moved: {v:?} should give {want}");
    }

    // 3. The REAL run's balance must still be accepted, so closing the gap cannot have
    //    silently inverted the published reading.
    let real = [0.43, 0.53]; // M4.4's measured inert sign balance, 30 established seeds
    let r3 = null_clean(&real);
    eprintln!("  3. the REAL M4.4 run inert frac-down {real:?} => null_clean = {r3}  (must be true)");
    assert!(r3, "the real run's null must still be accepted — the guard's meaning must not have changed");

    eprintln!("\n  ⟹ GAP 2 CLOSED. The guard rejects a skewed null in both directions, sits exactly at the");
    eprintln!("     pre-registered [0.40, 0.60] interval, and still accepts the real run. Every V-track guard");
    eprintln!("     now has a test in which it fires.");
    eprintln!("======================================================================\n");
}
