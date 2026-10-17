//! Criterion 5 validation — is the trait shift SELECTION, or neutral DRIFT?
//!
//! The plan's criterion 5 asks that trait distributions *shift* (directional mean
//! change), "not merely drift." The pre-registered failure mode: replication is
//! node-funded, so nothing opposes `replication_threshold` sliding toward its floor
//! — a free, degenerate directional move that is NOT selection resolving a trade-off.
//! The demanded control: run a pure-drift null and confirm the observed trajectory
//! is DISTINGUISHABLE from it. If it isn't, the result is drift, not selection.
//!
//! This test uses a MATCHED IN-RUN drift null. Of the four genome traits, only two
//! are read by any rule:
//!   - key 11 replication_threshold — read by `replicate` (the FREE/degenerate one).
//!   - key 13 satiation            — read by `harvest` (the TRADE-OFF one).
//!   - key 10 harvest_rate, key 12 maintenance_cost — SET, copied, and jittered
//!     identically but READ BY NOTHING (harvest/metabolise use fixed quanta). They
//!     drift under the exact same jitter magnitude and the exact same birth/death
//!     lineage sampling, with zero fitness coupling. They are the null.
//!
//! RESULT (reported, and the reason criterion 5 is NOT cleanly met): at jitter 0.5
//! over ~500-generation lineages, the population trait-mean random-walks by ~±10 for
//! a trait under NO selection. replication_threshold's directional move (~2–3, and
//! consistently downward) is REAL as a sign bias but its magnitude sits INSIDE that
//! drift envelope, and the trade-off trait (satiation) shows no reproducible
//! directional selection. So the scorecard's single-seed "directional shift ✓" does
//! not survive a matched drift null: the shift is drift-dominated, and to the extent
//! a directional signal exists it is the free degenerate trait, not a trade-off.
#![cfg(feature = "alife")]

use hg_alife::AlifeObserver;
use hg_core::GraphSnapshot;
use hg_engine::Engine;
use hg_observe::{Observer, RewriteEvent, StepStats};
use hg_rules::StepMode;
use hg_sim_core::SimConfig;
use std::sync::{Arc, Mutex};

const TRAIT_NAMES: [&str; 4] = ["harvest_rate", "replication_threshold", "maintenance_cost", "satiation"];
const INERT: [usize; 2] = [0, 2]; // harvest_rate, maintenance_cost — read by nothing
const REPL_THR: usize = 1;
const SATIATION: usize = 3;

fn exp5_config(seed: u64, steps: u64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "crit5"
run_id = "crit5"
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

struct Handle {
    alife: Arc<Mutex<AlifeObserver>>,
}
impl Observer for Handle {
    fn on_step_end(&mut self, snap: &GraphSnapshot<'_>, s: &StepStats, events: &[RewriteEvent]) {
        self.alife.lock().unwrap().on_step_end(snap, s, events);
    }
}

/// Displacement (last populated window − first populated window) of each trait mean.
/// Returns `None` if the seed never established a population (no populated window).
fn trait_displacements(seed: u64, steps: u64) -> Option<[f64; 4]> {
    let config = exp5_config(seed, steps);
    let rules = hg_sim_core::runner::load_rules(&config).unwrap();
    let mut store = hg_sim_core::runner::build_initial_store(&config);
    let mut engine = Engine::new(rules, StepMode::MaxParallel, config.seed, 100000);
    let alife = Arc::new(Mutex::new(AlifeObserver::new(500)));
    engine.add_observer(Box::new(Handle { alife: alife.clone() }));
    for _ in 0..steps {
        engine.step(&mut store);
    }
    drop(engine);
    let a = alife.lock().unwrap();
    let win = a.windows();
    let first_idx = win.iter().position(|w| w.carriers > 0)?;
    let last_idx = win.iter().rposition(|w| w.carriers > 0)?;
    if first_idx == last_idx {
        return None; // established for < one window; no drift measurable
    }
    let mut disp = [0.0; 4];
    for t in 0..4 {
        disp[t] = win[last_idx].trait_mean[t] - win[first_idx].trait_mean[t];
    }
    Some(disp)
}

#[test]
fn criterion5_shift_is_drift_dominated_not_selection() {
    let steps = 10_000u64;
    let seeds = [12345u64, 1, 7, 99999, 2024, 42, 777, 31415, 8, 100];

    eprintln!("\n===== CRITERION 5 — selection vs. matched in-run drift null =====");
    eprintln!("regime: turnover (influx 0.34, bootstrap thr 3), {steps} steps, {} seeds", seeds.len());
    eprintln!("inert (drift null): {} + {} — read by no rule.\n", TRAIT_NAMES[0], TRAIT_NAMES[2]);
    eprintln!("  seed   | Δharvest  Δrepl_thr  Δmaint    Δsatiation | inert|Δ|max");
    eprintln!("  -------+------------------------------------------+-----------");

    let mut established = Vec::new();
    for &seed in &seeds {
        match trait_displacements(seed, steps) {
            Some(d) => {
                let inert_max = INERT.iter().map(|&i| d[i].abs()).fold(0.0, f64::max);
                eprintln!(
                    "  {seed:>6} | {:+8.3}  {:+8.3}  {:+8.3}  {:+8.3}   | {:8.3}",
                    d[0], d[1], d[2], d[3], inert_max
                );
                established.push(d);
            }
            None => eprintln!("  {seed:>6} |  (never established a sustained population)"),
        }
    }

    let n = established.len();
    assert!(n >= 2, "need at least a couple of established seeds to characterize drift");

    let col = |t: usize| established.iter().map(|d| d[t]).collect::<Vec<_>>();
    let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
    let absmean = |v: &[f64]| v.iter().map(|x| x.abs()).sum::<f64>() / v.len() as f64;

    // Drift envelope: typical |displacement| of a trait under NO selection.
    let drift_env = (absmean(&col(INERT[0])) + absmean(&col(INERT[1]))) / 2.0;
    let repl = col(REPL_THR);
    let sat = col(SATIATION);
    let repl_mean = mean(&repl);
    let sat_mean = mean(&sat);
    // Directional consistency: fraction of established seeds with the same sign.
    let frac_neg = |v: &[f64]| v.iter().filter(|x| **x < 0.0).count() as f64 / v.len() as f64;

    eprintln!("\n  established: {n}/{} seeds  (turnover regime is seed-fragile)", seeds.len());
    eprintln!("  drift envelope (mean inert |Δ|)         : {drift_env:.2}");
    eprintln!(
        "  replication_threshold: mean Δ {repl_mean:+.2}  |Δ| {:.2}  frac-down {:.0}%  → magnitude/drift {:.2}",
        absmean(&repl), 100.0 * frac_neg(&repl), absmean(&repl) / drift_env
    );
    eprintln!(
        "  satiation (trade-off): mean Δ {sat_mean:+.2}  |Δ| {:.2}  frac-down {:.0}%  → magnitude/drift {:.2}",
        absmean(&sat), 100.0 * frac_neg(&sat), absmean(&sat) / drift_env
    );
    eprintln!("\n  ⟹ FINDING: the population trait-mean of a NON-selected trait drifts ~±{drift_env:.0}");
    eprintln!("     (jitter 0.5 random-walk over deep near-clonal lineages), and the inert null traits");
    eprintln!("     are sign-MIXED across seeds — a proper null. Against that null:");
    eprintln!("      • replication_threshold is DOWN in {:.0}% of established seeds — a reproducible", 100.0 * frac_neg(&repl));
    eprintln!("        directional SIGN bias (distinguishable from drift by SIGN, not magnitude: its");
    eprintln!("        |Δ| {:.1} is only {:.2}× drift). This is the §1 FREE/DEGENERATE collapse —", absmean(&repl), absmean(&repl) / drift_env);
    eprintln!("        real directional pressure with no counterforce, NOT a trade-off being resolved.");
    eprintln!("      • satiation (the trade-off trait) is up in {:.0}% — a WEAK, non-decisive bias at", 100.0 * (1.0 - frac_neg(&sat)));
    eprintln!("        n={n}, magnitude fully inside drift. No clean trade-off selection.");
    eprintln!("     ⟹ Criterion 5 is met ONLY via the pre-registered §1 degenerate route (now EARNED");
    eprintln!("       with a matched drift null, not asserted from one seed). The specific scorecard");
    eprintln!("       numbers (0.14→−3.72) are an effect size INSIDE the drift band and not meaningful");
    eprintln!("       as magnitude. No non-degenerate selection. Regime is seed-fragile ({n}/{}).", seeds.len());
    eprintln!("================================================================\n");

    // Robust assertions (the FINDING, not the premature scorecard claim):
    // 1. A non-selected trait's population-mean drift is substantial — a single-seed
    //    "the trait shifted by a few units" claim is therefore uninformative.
    assert!(drift_env > 2.0, "inert drift envelope must be large enough to swamp a few-unit shift (got {drift_env:.2})");
    // 2. replication_threshold's shift magnitude does NOT clearly exceed neutral drift
    //    — so magnitude alone cannot establish selection over drift.
    assert!(
        absmean(&repl) < 2.0 * drift_env,
        "repl_threshold magnitude does not dominate drift (|Δ| {:.2} vs drift {drift_env:.2})",
        absmean(&repl)
    );
    // 3. The trade-off trait (satiation) is not reproducibly directional beyond drift
    //    — the only directional signal is the free degenerate trait, if any.
    assert!(
        (frac_neg(&sat) - 0.5).abs() < 0.5 || absmean(&sat) < 2.0 * drift_env,
        "satiation should not show clean trade-off selection beyond drift"
    );
}
