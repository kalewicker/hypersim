//! Phase C5 — Stages 4 and 5 (NN plan §11). Measurement passes, no training.
//!
//! **Stage 4.** Do policy-shaped dynamics populate the task lattice at a
//! higher rate than random dynamics? Metrics: lattice depth, task count,
//! and composition rate — "A→C entries appearing after A→B and B→C are
//! both present".
//!
//! **Stage 5.** Do shaped dynamics increase the macro-over-micro EI
//! advantage? Metrics: `ei_micro`, `ei_macro` and the structural CE
//! coefficient, which the evaluator already writes to every `MetricsRow`.
//!
//! Both run off the same rollouts, because both are read from one causal
//! layer and paying for it twice would be silly.
//!
//! ## What "champion" means here, and why the untrained arm is present
//!
//! Stage 2 measured that its champion is **not better than an untrained
//! `StaticBias`** (`docs/cognition_stage2_findings.md` §3): champion minus
//! untrained is +0.0000 on this substrate. So a champion-vs-random contrast
//! here is a **policy-class** contrast — group-uniform allocation versus
//! binding-uniform sampling — and not a contrast between learned and
//! unlearned dynamics.
//!
//! That is still a legitimate and interesting question for Stages 4 and 5,
//! whose hypotheses are about *shaped* dynamics rather than *learned* ones.
//! But it must not be reported as "learned shaping", so the untrained arm
//! runs alongside: if champion and untrained agree, the effect belongs to
//! the policy class, and the plan's pre-registered direction has been
//! tested against the wrong null.
//!
//! **`[causal]` is ON.** The task lattice and EI both come from the causal
//! layer, which the Stage 0 diagnostics disable. With it off every number
//! below would be 0 and would read as "shaping has no effect" rather than
//! "nothing was computed".

use hg_cognition::{derive_policy_seed, Parameterized, RandomSubset, StaticBias};
use hg_observe::RuleSelectionPolicy;
use hg_sim_core::{run_rollout, RolloutRequest, TaskSummary};
use hg_train::diagnostics::{Fixture, RuleSource};
use std::collections::HashSet;

const SEEDS: usize = 10;
const STEPS: u64 = 384;
const WINDOW: u64 = 24;
const BURN_IN: usize = 5;
const BUDGET: usize = 16;

/// The σ = 2.0 `wn_three` champion (`gen_0039.policy.toml`, genome hash
/// `0x405a3aa651eeac0f`). Inlined rather than loaded from `runs_stage2/`
/// so this test does not depend on a campaign directory that a clean
/// checkout will not have.
const CHAMPION: [f32; 3] = [0.033_061_95, 1.331_307_9, -2.882_334_2];

fn wn_three() -> Fixture {
    Fixture {
        name: "wn_three",
        rules: RuleSource::File("rules/wn_three.toml"),
        ic: "WhiteNoise",
        width: 8,
        node_count: 24,
        steps: STEPS,
        window: WINDOW,
        step_mode: "MaxParallel",
    }
}

fn seeds() -> Vec<u64> {
    (0..SEEDS as u64).map(|i| 7_400_000 + i * 101).collect()
}

/// Stage 4's composition rate.
///
/// Over the set of distinct (input, output) task pairs: for every
/// composable chain A→B, B→C, does the composed task A→C also appear? The
/// rate is realized / composable. Self-loops (A == C) are excluded, since
/// A→A is not evidence of composition.
///
/// Returns (realized, composable, rate). A rate is only meaningful when
/// `composable > 0`; with no chains at all there is nothing to compose and
/// the answer is undefined rather than zero.
fn composition_rate(tasks: &[TaskSummary]) -> (usize, usize, Option<f64>) {
    let set: HashSet<(u64, u64)> = tasks.iter().map(|t| (t.input_wl, t.output_wl)).collect();
    let (mut realized, mut composable) = (0usize, 0usize);
    for a in &set {
        for b in &set {
            if a.1 == b.0 && a.0 != b.1 {
                composable += 1;
                if set.contains(&(a.0, b.1)) {
                    realized += 1;
                }
            }
        }
    }
    let rate = (composable > 0).then(|| realized as f64 / composable as f64);
    (realized, composable, rate)
}

#[derive(Default)]
struct Arm {
    depth: Vec<f64>,
    task_count: Vec<f64>,
    comp_rate: Vec<f64>,
    composable: Vec<f64>,
    ei_micro: Vec<f64>,
    ei_macro: Vec<f64>,
    ce_structural: Vec<f64>,
    n_constructors: Vec<f64>,
}

fn mean(v: &[f64]) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    v.iter().sum::<f64>() / v.len() as f64
}
fn sd(v: &[f64]) -> f64 {
    if v.len() < 2 {
        return 0.0;
    }
    let m = mean(v);
    (v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (v.len() - 1) as f64).sqrt()
}

/// Paired z of `a` over `b`. Same denominator convention as the Stage 0a
/// classifier: pooled when both vary, the one that varies when only one
/// does, `None` when neither.
fn z(a: &[f64], b: &[f64]) -> Option<f64> {
    let (am, bm) = (mean(a), mean(b));
    let (asd, bsd) = (sd(a), sd(b));
    let floor = |s: f64, m: f64| s > 1e-9 * (m.abs() + 1.0);
    let d = match (floor(asd, am), floor(bsd, bm)) {
        (true, true) => Some(((asd * asd + bsd * bsd) / 2.0).sqrt()),
        (true, false) => Some(asd),
        (false, true) => Some(bsd),
        (false, false) => None,
    }?;
    Some((am - bm) / d)
}

fn run_arm(make: &(dyn Fn(u64) -> Box<dyn RuleSelectionPolicy> + Sync)) -> Arm {
    let dir = tempfile::tempdir().expect("tempdir");
    let f = wn_three();
    let ss = seeds();
    let one = |s: u64| {
        // `[causal]` ON — the whole point of these two stages.
        let cfg = f.sim_config_causal(dir.path(), s);
        let m = run_rollout(RolloutRequest {
            config: &cfg,
            seed: s,
            policy: make(derive_policy_seed(s, 0, 0, 0, 0)),
            horizon_steps: STEPS,
            probe: None,
        })
        .expect("rollout");
        let (_, composable, rate) = composition_rate(&m.tasks);
        // EI is per-window; average over the scored windows so it is on the
        // same burn-in footing as every fitness number in this programme.
        let scored = m.windows.get(BURN_IN..).unwrap_or(&[]);
        let avg = |f: fn(&hg_schema::MetricsRow) -> Option<f64>| -> f64 {
            let v: Vec<f64> = scored.iter().filter_map(f).collect();
            mean(&v)
        };
        (
            m.task_lattice_depth as f64,
            m.tasks.len() as f64,
            rate.unwrap_or(f64::NAN),
            composable as f64,
            avg(|r| r.ei_micro),
            avg(|r| r.ei_macro),
            avg(|r| r.ce_structural),
            m.constructors.len() as f64,
        )
    };

    // Scoped OS threads, not rayon — the runner installs its step loop into
    // its own pool and nesting them serializes completely.
    let n = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
    let chunk = ss.len().div_ceil(n.max(1));
    let mut out = Vec::new();
    std::thread::scope(|scope| {
        let handles: Vec<_> = ss
            .chunks(chunk.max(1))
            .map(|b| scope.spawn(|| b.iter().map(|&s| one(s)).collect::<Vec<_>>()))
            .collect();
        for h in handles {
            out.extend(h.join().expect("worker"));
        }
    });

    Arm {
        depth: out.iter().map(|r| r.0).collect(),
        task_count: out.iter().map(|r| r.1).collect(),
        comp_rate: out.iter().map(|r| r.2).filter(|x| x.is_finite()).collect(),
        composable: out.iter().map(|r| r.3).collect(),
        ei_micro: out.iter().map(|r| r.4).filter(|x| x.is_finite()).collect(),
        ei_macro: out.iter().map(|r| r.5).filter(|x| x.is_finite()).collect(),
        ce_structural: out.iter().map(|r| r.6).filter(|x| x.is_finite()).collect(),
        n_constructors: out.iter().map(|r| r.7).collect(),
    }
}

#[test]
#[ignore = "long-running; run explicitly for the Stage 4/5 tables"]
fn stage45_measurement_passes() {
    println!("\n=== Stages 4 & 5 — shaped vs random dynamics ===");
    println!("(wn_three, k = {BUDGET}, {STEPS} steps, {SEEDS} seeds, causal ON)\n");

    let arms: Vec<(&str, Arm)> = vec![
        (
            "random",
            run_arm(&|seed| Box::new(RandomSubset::new(Some(BUDGET), seed))),
        ),
        (
            "untrained",
            run_arm(&|seed| {
                let mut p = StaticBias::new(3, Some(BUDGET), seed);
                p.set_params(&[0.0, 0.0, 0.0]);
                Box::new(p)
            }),
        ),
        (
            "champion",
            run_arm(&|seed| {
                let mut p = StaticBias::new(3, Some(BUDGET), seed);
                p.set_params(&CHAMPION);
                Box::new(p)
            }),
        ),
    ];

    println!("## Stage 4 — task lattice");
    println!(
        "{:<12} {:>8} {:>8} {:>10} {:>8} {:>12} {:>8} {:>9}",
        "arm", "depth", "sd", "tasks", "sd", "comp rate", "sd", "chains"
    );
    for (n, a) in &arms {
        println!(
            "{:<12} {:>8.2} {:>8.3} {:>10.1} {:>8.2} {:>12.4} {:>8.4} {:>9.0}",
            n,
            mean(&a.depth),
            sd(&a.depth),
            mean(&a.task_count),
            sd(&a.task_count),
            mean(&a.comp_rate),
            sd(&a.comp_rate),
            mean(&a.composable),
        );
    }

    println!("\n## Stage 5 — effective information");
    println!(
        "{:<12} {:>10} {:>8} {:>10} {:>8} {:>12} {:>8}",
        "arm", "ei_micro", "sd", "ei_macro", "sd", "ce_struct", "sd"
    );
    for (n, a) in &arms {
        println!(
            "{:<12} {:>10.4} {:>8.4} {:>10.4} {:>8.4} {:>12.4} {:>8.4}",
            n,
            mean(&a.ei_micro),
            sd(&a.ei_micro),
            mean(&a.ei_macro),
            sd(&a.ei_macro),
            mean(&a.ce_structural),
            sd(&a.ce_structural),
        );
    }

    // Pre-registered directions: Stage 4 composition rate higher under the
    // shaped arm; Stage 5 shaped > random at the macro/attribute level.
    let base = &arms[0].1;
    println!("\n## Contrasts (pre-registered direction: shaped > random)");
    for (n, a) in arms.iter().skip(1) {
        for (label, x, y) in [
            ("lattice depth", &a.depth, &base.depth),
            ("task count", &a.task_count, &base.task_count),
            ("composition rate", &a.comp_rate, &base.comp_rate),
            ("ei_macro", &a.ei_macro, &base.ei_macro),
            ("ce_structural", &a.ce_structural, &base.ce_structural),
        ] {
            match z(x, y) {
                Some(v) => println!(
                    "  {n:<10} {label:<18} {:+.4} vs {:+.4}  = {v:+.2} sigma{}",
                    mean(x),
                    mean(y),
                    if v.abs() > 2.0 { "  <== resolved" } else { "" }
                ),
                None => println!(
                    "  {n:<10} {label:<18} {:+.4} vs {:+.4}  = no spread on either arm",
                    mean(x),
                    mean(y)
                ),
            }
        }
    }

    // The claim that decides how the whole table reads.
    println!("\n## Is this learned shaping, or a policy class?");
    for (label, x, y) in [
        ("composition rate", &arms[2].1.comp_rate, &arms[1].1.comp_rate),
        ("ei_macro", &arms[2].1.ei_macro, &arms[1].1.ei_macro),
    ] {
        println!(
            "  champion vs untrained  {label:<18} {:+.4} vs {:+.4}  = {}",
            mean(x),
            mean(y),
            z(x, y).map(|v| format!("{v:+.2} sigma")).unwrap_or("no spread".into())
        );
    }

    // Machinery assertions only — the science may come out either way
    // (plan §11: "Report either way").
    for (n, a) in &arms {
        assert_eq!(a.depth.len(), SEEDS, "{n}: not every rollout produced a lattice depth");
        assert!(
            mean(&a.task_count) > 0.0,
            "{n}: the task lattice is empty — [causal] is off or nothing was witnessed, \
             and every Stage 4 number would be a measurement of that rather than of shaping"
        );
    }
}
