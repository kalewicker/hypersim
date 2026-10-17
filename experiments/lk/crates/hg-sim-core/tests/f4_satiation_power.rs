//! F-track F4 — criterion 5: is `satiation` underpowered, or genuinely null?
//!
//! V2.1 concluded criterion 5 is met only via the degenerate route. For
//! `replication_threshold` that is certain: node-funded replication means nothing
//! opposes it, it moved 100 % down, and its effect size sits INSIDE the drift band
//! (1.22× against a 4.47 envelope).
//!
//! For `satiation` the conclusion is less safe. It showed mean Δ +4.28, |Δ| 7.56 —
//! **1.69× the drift envelope, larger than `replication_threshold`'s** — and 83 %
//! directional. It is the one trait carrying a genuine trade-off (more buffer means
//! longer survival but costs more energy to fill), it moved in the pre-registered
//! direction, and all of that rested on just **6 established seeds**.
//!
//! F4.1 replicates at high seed count targeting ≥30 established runs. The
//! measurement is byte-for-byte the same as `alife_criterion5_selection_vs_drift.rs`
//! (same config, same window displacement, same matched in-run drift null from the
//! two inert traits) — only n changes, so this is a power increase and not a new
//! test that could be shopped for a different answer.
//!
//! **F4.2 decision rule, pre-committed in `docs/validation_v2_fix_plan.md` §F4.2
//! before these runs existed:**
//!   * sign test on Δ direction significant AND |Δ|/drift-envelope > 1.5
//!     ⟹ criterion 5 has a NON-DEGENERATE component; the verdict changes.
//!   * either fails ⟹ the V1 verdict stands, now properly powered.
#![cfg(feature = "alife")]

use hg_alife::AlifeObserver;
use hg_core::GraphSnapshot;
use hg_engine::Engine;
use hg_observe::{Observer, RewriteEvent, StepStats};
use hg_rules::StepMode;
use hg_sim_core::SimConfig;
use std::sync::{Arc, Mutex};

const INERT: [usize; 2] = [0, 2]; // harvest_rate, maintenance_cost — read by no rule
const REPL_THR: usize = 1;
const SATIATION: usize = 3;

/// Target from the plan: ≥30 established runs. The plan budgeted ~45–55 attempts on
/// the basis of a 57–71 % establishment rate, but that figure came from small curated
/// seed sets (n=7 in `alife_band_reality`, n=10 in `alife_criterion5_selection_vs_drift`).
/// An unbiased sequential sweep establishes far less often (measured below), so the
/// attempt budget is raised to reach the pre-registered target. Raising `n` toward a
/// pre-committed target is a power increase, not a stopping-rule choice: the F4.2
/// decision rule is fixed and is applied to whatever this returns.
const TARGET_ESTABLISHED: usize = 30;
const MAX_ATTEMPTS: u64 = 160;
const STEPS: u64 = 10_000;
/// Pre-committed thresholds (plan §F4.2).
const ALPHA: f64 = 0.05;
const EFFECT_RATIO_MIN: f64 = 1.5;

fn exp5_config(seed: u64, steps: u64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "f4"
run_id = "f4"
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
/// `None` if the seed never established a sustained population. Identical to the
/// V2.1 harness so the two results are directly comparable.
fn trait_displacements(seed: u64, steps: u64) -> Option<[f64; 4]> {
    let config = exp5_config(seed, steps);
    let rules = hg_sim_core::runner::load_rules(&config).unwrap();
    let mut store = hg_sim_core::runner::build_initial_store(&config);
    let mut engine = Engine::new(rules, StepMode::MaxParallel, config.seed, 100_000);
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
        return None;
    }
    let mut disp = [0.0; 4];
    for t in 0..4 {
        disp[t] = win[last_idx].trait_mean[t] - win[first_idx].trait_mean[t];
    }
    Some(disp)
}

/// Exact two-sided binomial sign-test p-value for `k` of `n` under p = 0.5.
fn sign_test_p(k: usize, n: usize) -> f64 {
    if n == 0 {
        return 1.0;
    }
    // log-space binomial coefficients keep this exact enough at n ~ 60.
    let ln_c = |n: usize, k: usize| -> f64 {
        (1..=k).map(|i| ((n - k + i) as f64).ln() - (i as f64).ln()).sum()
    };
    let ln_half_n = (n as f64) * 0.5f64.ln();
    let tail = |from: usize| -> f64 {
        (from..=n).map(|i| (ln_c(n, i) + ln_half_n).exp()).sum::<f64>()
    };
    let extreme = k.max(n - k);
    (2.0 * tail(extreme)).min(1.0)
}

#[test]
fn f4_satiation_selection_power() {
    let mut established: Vec<(u64, [f64; 4])> = Vec::new();
    let mut attempts = 0u64;
    for seed in 1..=MAX_ATTEMPTS {
        attempts += 1;
        if let Some(d) = trait_displacements(seed, STEPS) {
            established.push((seed, d));
        }
        if established.len() >= TARGET_ESTABLISHED {
            break;
        }
    }

    let n = established.len();
    let est_rate = 100.0 * n as f64 / attempts as f64;
    eprintln!("\n========== F4 — criterion-5 power test on `satiation` ==========");
    eprintln!("turnover regime (influx 0.34, bootstrap thr 3), {STEPS} steps/seed");
    eprintln!("attempts {attempts}, established {n} ({est_rate:.0}% establishment rate)");
    eprintln!("V2.1 ran this at n=6; the plan targets n>={TARGET_ESTABLISHED}.\n");

    let col = |t: usize| established.iter().map(|(_, d)| d[t]).collect::<Vec<_>>();
    let absmean = |v: &[f64]| v.iter().map(|x| x.abs()).sum::<f64>() / v.len().max(1) as f64;
    let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len().max(1) as f64;

    // The matched in-run drift null: two traits set/copied/jittered identically but
    // read by NO rule, so they experience the same lineage sampling with zero
    // fitness coupling.
    let drift_env = (absmean(&col(INERT[0])) + absmean(&col(INERT[1]))) / 2.0;
    let sat = col(SATIATION);
    let repl = col(REPL_THR);

    let sat_up = sat.iter().filter(|x| **x > 0.0).count();
    let repl_down = repl.iter().filter(|x| **x < 0.0).count();
    let sat_p = sign_test_p(sat_up, n);
    let repl_p = sign_test_p(repl_down, n);
    let sat_ratio = absmean(&sat) / drift_env;
    let repl_ratio = absmean(&repl) / drift_env;

    // Inert-trait sign balance is the null's own sanity check: a proper null should
    // be sign-MIXED. If an inert trait were itself directional, the null is broken.
    for (name, ix) in [("harvest_rate", INERT[0]), ("maintenance_cost", INERT[1])] {
        let c = col(ix);
        let up = c.iter().filter(|x| **x > 0.0).count();
        eprintln!("  null trait {name:<17}: {up}/{n} up, sign-test p={:.3}, |Δ| {:.2}", sign_test_p(up, n), absmean(&c));
    }
    eprintln!("  drift envelope (mean inert |Δ|)   : {drift_env:.2}\n");

    eprintln!("  trait                  mean Δ    |Δ|     directional      sign-test p   |Δ|/drift");
    eprintln!("  ---------------------+---------+-------+----------------+-------------+---------");
    eprintln!("  replication_threshold  {:+7.2}  {:6.2}  {:>3}/{n} down     {repl_p:11.2e}  {repl_ratio:8.2}",
        mean(&repl), absmean(&repl), repl_down);
    eprintln!("  satiation (trade-off)  {:+7.2}  {:6.2}  {:>3}/{n} up       {sat_p:11.2e}  {sat_ratio:8.2}",
        mean(&sat), absmean(&sat), sat_up);

    // ---- F4.2 pre-committed decision rule ----
    let sign_ok = sat_p < ALPHA;
    let effect_ok = sat_ratio > EFFECT_RATIO_MIN;
    eprintln!("\n  F4.2 decision rule (pre-committed before these runs):");
    eprintln!("    (i)  sign test significant      : p={sat_p:.2e} < {ALPHA}  ⟹ {}", if sign_ok { "PASS" } else { "FAIL" });
    eprintln!("    (ii) |Δ|/drift-envelope > {EFFECT_RATIO_MIN}     : {sat_ratio:.2}          ⟹ {}", if effect_ok { "PASS" } else { "FAIL" });
    eprintln!();
    if sign_ok && effect_ok {
        eprintln!("  ⟹ BOTH PASS: criterion 5 has a NON-DEGENERATE COMPONENT. `satiation` — the one");
        eprintln!("    trait carrying a real trade-off — moves reproducibly in the pre-registered");
        eprintln!("    direction with an effect size outside the matched drift null. The V1/V2.1");
        eprintln!("    verdict ('degenerate route only') CHANGES: it was underpowered at n=6.");
    } else {
        eprintln!("  ⟹ NOT BOTH: the V1 verdict STANDS, now properly powered at n={n} rather than 6.");
        if !sign_ok {
            eprintln!("    The direction is not reproducible across seeds (sign test p={sat_p:.2e}).");
        }
        if !effect_ok {
            eprintln!("    The effect size {sat_ratio:.2}× does not clear {EFFECT_RATIO_MIN}× of the drift envelope,");
            eprintln!("    i.e. it remains inside what an unselected trait does by random walk.");
        }
        eprintln!("    Criterion 5 is met via the degenerate route only. Reported, not repaired.");
    }

    // F4.3 — the structural reason, recorded beside the result.
    eprintln!("\n  F4.3 STRUCTURAL CAUSE (a pre-committed-deviation consequence, not a discovery):");
    eprintln!("    Deviation #4 kept harvest/maintenance as FIXED QUANTA rather than genome-scaled,");
    eprintln!("    so `harvest_rate` and `maintenance_cost` are read by no rule — 2 of 4 genome");
    eprintln!("    traits can only drift. Of the remaining two, `replication_threshold` is unopposed");
    eprintln!("    under node-funded replication (nothing pays for lowering it). That leaves");
    eprintln!("    `satiation` as the ONLY trait able to express a trade-off at all.");
    eprintln!("    ⟹ The deviation that made the economy tractable is what made selection");
    eprintln!("      near-undetectable. Criterion 5 was close to unsatisfiable BY CONSTRUCTION.");
    eprintln!("================================================================\n");

    if n < TARGET_ESTABLISHED {
        eprintln!("  NOTE: reached n={n} of the target {TARGET_ESTABLISHED} within {attempts} attempts");
        eprintln!("  ({est_rate:.0}% establishment). The shortfall is itself reported, per plan rule 4.");
    }
    // Guard against a vacuous run only; n is a measured quantity, not a gate.
    assert!(n >= 10, "F4 vacuous: only {n} established runs from {attempts} attempts");
}
