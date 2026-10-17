//! Phase D3, Stage R — the re-analysis passes (`hypersim_nn_experiments_plan_v2.md` §8).
//!
//! **R-1** re-signs the Stage 0a table. The C0–C5 classifier compared
//! `|heuristic − random|`, which is correct for "does selection have causal
//! power" and wrong for everything downstream: several headline cells are
//! the heuristic being *worse*, including the exit cell
//! (`growth_treadmill` k=1: random 0.147, `degree_high` 0.091).
//!
//! R-1 is also **the first real test of the §5.1 diagnosis**. If
//! diversity-reward is the mechanism — if uniform selection is the optimum
//! *of the objective* rather than merely of `StaticBias` — then signed
//! differences should be overwhelmingly negative across every measurable
//! cell: every non-uniform heuristic worse than budget-matched random, not
//! merely different. **Any positive-signed cell is exploitable signal the
//! diagnosis does not account for, and is the highest-value lead in the
//! dataset** (plan v2 risk 4).
//!
//! ## The plan says "no new compute". That premise does not hold.
//!
//! §8 opens "These consume no new rollouts and should run first." That is
//! true of the *analysis*, not of the *data*: Stage 0a wrote nothing.
//! `runs/cognition/` is empty, the C0–C5 sweep printed its table to stdout,
//! and `docs/campaign_output.md` §1 is explicit that a rollout leaves no
//! artifact by design. So R-1 re-runs the sweep to recover the signs.
//!
//! That is a cost, not a compromise: the re-run happens under the v2
//! instrument (signed classifier, engagement counters, the fixed
//! `RandomSubset`), so the signed table is measured rather than
//! reconstructed. It also means R-1 is a **re-derivation**, which is what
//! plan v2 §12 asks for on any figure carried over from before bug 1.

use hg_train::diagnostics::*;

/// The C0–C5 fixture battery, unchanged so the signed table is comparable
/// to the unsigned one it refines.
fn fixtures() -> Vec<Fixture> {
    vec![
        Fixture {
            name: "glider",
            rules: RuleSource::Builtin("glider"),
            ic: "GliderRing",
            width: 8,
            node_count: 31,
            steps: 512,
            window: 64,
            step_mode: "MaxParallel",
        },
        Fixture {
            name: "growth_treadmill",
            rules: RuleSource::Builtin("growth_treadmill"),
            ic: "WhiteNoise",
            width: 8,
            node_count: 24,
            steps: 512,
            window: 64,
            step_mode: "MaxParallel",
        },
        Fixture {
            name: "triangle_subdivide",
            rules: RuleSource::Builtin("triangle_subdivide"),
            ic: "TriLattice",
            width: 8,
            node_count: 24,
            steps: 96,
            window: 24,
            step_mode: "MaxParallel",
        },
        Fixture {
            name: "binary_edge",
            rules: RuleSource::Builtin("binary_edge"),
            ic: "PureCrystal",
            width: 8,
            node_count: 24,
            steps: 96,
            window: 24,
            step_mode: "MaxParallel",
        },
        Fixture {
            name: "fl1_two_rule",
            rules: RuleSource::File("rules/fl1_learnable_signal.toml"),
            ic: "GliderRing",
            width: 8,
            node_count: 31,
            steps: 192,
            window: 32,
            step_mode: "MaxParallel",
        },
    ]
}

fn seeds(n: usize) -> Vec<u64> {
    (0..n as u64).map(|i| 7_000_000 + i * 101).collect()
}

fn fmt_budget(b: Option<usize>) -> String {
    b.map(|k| k.to_string()).unwrap_or_else(|| "inf".into())
}

/// The bar a signed effect must clear to count as a direction rather than
/// noise. Same 2σ the plan's exit criterion uses, applied one-sided.
const SIGMA_BAR: f64 = 2.0;

/// Above this, an effect size on 20 seeds is an instrument fault pending
/// proof, not a finding (`docs/instrument_registry.md` §2.6). A 100σ effect
/// and a collapsed denominator look identical in the output, and the
/// collapsed denominator is overwhelmingly more likely.
const SIGMA_FAULT_BAR: f64 = 100.0;

#[test]
#[ignore = "re-runs the full Stage 0a sweep (5 rule sets x 5 budgets x 4 arms \
            x 20 seeds); run explicitly for the R-1 signed table"]
fn r1_signed_stage0a_table() {
    let dir = tempfile::tempdir().expect("tempdir");
    let s = seeds(20);

    println!("\n=== R-1 — Stage 0a, SIGNED (20 seeds) ===");
    println!(
        "delta and sigma are `heuristic - random`. POSITIVE means the heuristic \
         BEAT budget-matched random.\n"
    );

    let mut all: Vec<(String, Option<usize>, SignedEffect)> = Vec::new();
    let mut cell_verdicts: Vec<(String, Option<usize>, EffectClass)> = Vec::new();

    for f in fixtures() {
        println!("\n## {} ({} / {} steps)", f.name, f.ic, f.steps);
        println!(
            "{:>5}  {:<12} {:<18} {:>10} {:>9} {:>10} {:>10}",
            "k", "arm", "metric", "delta", "sigma", "heuristic", "random"
        );
        for cell in stage0a(&f, dir.path(), &s, 1) {
            for e in cell.signed_effects() {
                println!(
                    "{:>5}  {:<12} {:<18} {:>+10.4} {:>9} {:>10.4} {:>10.4}",
                    fmt_budget(cell.budget),
                    e.arm,
                    e.metric,
                    e.delta,
                    e.sigma
                        .map(|v| format!("{v:+.2}"))
                        .unwrap_or_else(|| "  n/a".into()),
                    e.heuristic_mean,
                    e.random_mean,
                );
                all.push((f.name.to_string(), cell.budget, e));
            }
            let v = cell.classify_effect();
            println!("       -> {}", v.label());
            cell_verdicts.push((f.name.to_string(), cell.budget, v));
        }
    }

    // ---- Rule 2, applied mechanically ---------------------------------
    //
    // `docs/instrument_registry.md` §2.6: a cell reporting |sigma| > 100 on
    // 20 seeds is an instrument fault pending proof — the denominator
    // collapsed, it is not that the effect is enormous. Applying that by
    // eye after reading the table is exactly the discipline the registry
    // exists to replace, so it is applied here before anything is counted.
    let (credible, suspect): (Vec<_>, Vec<_>) = all
        .iter()
        .partition(|(_, _, e)| e.sigma.is_none_or(|s| s.abs() <= SIGMA_FAULT_BAR));
    if !suspect.is_empty() {
        println!("\n=== Rule 2: {} comparison(s) exceed |{SIGMA_FAULT_BAR}|σ ===", suspect.len());
        println!("Denominator collapse, not enormous effects. Excluded from the counts below.");
        for (set, k, e) in &suspect {
            println!(
                "    {set} k={} {} on {}: {:+.4} ({:+.2}σ, random sd ~ {:.2e})",
                fmt_budget(*k),
                e.arm,
                e.metric,
                e.delta,
                e.sigma.unwrap_or(f64::NAN),
                e.delta.abs() / e.sigma.unwrap_or(f64::NAN).abs()
            );
        }
    }

    // ---- degree-regularity check --------------------------------------
    //
    // On a degree-regular graph `degree_high` and `degree_low` see one
    // degree value, so both collapse to the same deterministic tie-break
    // order and produce identical numbers. When that happens the arm is
    // NOT a degree heuristic — it is "prefer whatever canonical order puts
    // first", and the finding has to be stated that way.
    let mut regular_cells = Vec::new();
    for (set, k, e) in &all {
        if e.arm != "degree_high" {
            continue;
        }
        if let Some(twin) = all.iter().find(|(s, b, o)| {
            s == set && b == k && o.arm == "degree_low" && o.metric == e.metric
        }) {
            if (twin.2.delta - e.delta).abs() < 1e-12 && e.delta.abs() > 1e-12 {
                regular_cells.push((set.clone(), *k, e.metric));
            }
        }
    }
    if !regular_cells.is_empty() {
        println!(
            "\n=== degree_high == degree_low on {} comparison(s) ===",
            regular_cells.len()
        );
        println!(
            "Two opposite heuristics returning identical numbers means the graph is \
             degree-REGULAR, so neither is selecting on degree: both reduce to the \
             canonical tie-break order. The effect is real but it is 'deterministic \
             non-uniform selection', not 'prefer high degree'."
        );
        for (set, k, m) in &regular_cells {
            println!("    {set} k={} on {m}", fmt_budget(*k));
        }
    }

    // ---- the §5.1 test -------------------------------------------------
    let beats: Vec<_> = credible.iter().filter(|(_, _, e)| e.beats_random(SIGMA_BAR)).collect();
    let loses: Vec<_> = credible.iter().filter(|(_, _, e)| e.loses_to_random(SIGMA_BAR)).collect();
    let flat = credible.len() - beats.len() - loses.len();

    println!("\n=== R-1 verdict ===");
    println!("comparisons: {} ({} credible after Rule 2)", all.len(), credible.len());
    println!("heuristic BEATS random by > {SIGMA_BAR}σ : {}", beats.len());
    println!("heuristic LOSES to random by > {SIGMA_BAR}σ: {}", loses.len());
    println!("within ±{SIGMA_BAR}σ or no denominator     : {flat}");

    if beats.is_empty() {
        println!(
            "\nNo heuristic beats budget-matched random anywhere in the battery. \
             This is what the §5.1 diversity-reward diagnosis predicts: uniform \
             selection is the optimum OF THE OBJECTIVE, so every non-uniform \
             heuristic should be worse rather than merely different."
        );
    } else {
        println!(
            "\n*** {} POSITIVE-SIGNED CELL(S) — plan v2 risk 4. The §5.1 \
             generalization is weakened: these are exploitable signal the \
             diversity-reward diagnosis does not account for, and they are the \
             highest-value lead in the dataset. ***",
            beats.len()
        );
        for (set, k, e) in &beats {
            println!(
                "    {set} k={} {} on {}: {:+.4} ({:+.2}σ)",
                fmt_budget(*k),
                e.arm,
                e.metric,
                e.delta,
                e.sigma.unwrap_or(f64::NAN)
            );
        }
    }

    // The exit cell v1 reported, re-read with its sign attached.
    for (set, k, v) in &cell_verdicts {
        if let Some(sig) = v.signed_sigma() {
            if sig.abs() > SIGMA_BAR {
                println!(
                    "exit-criterion cell: {set} k={} -> {:+.2}σ ({})",
                    fmt_budget(*k),
                    sig,
                    if sig > 0.0 { "heuristic wins" } else { "heuristic LOSES" }
                );
            }
        }
    }

    // Machinery assertions only. A table that is entirely negative and a
    // table with a positive cell are both legitimate scientific outcomes
    // and the test must not prefer one; what it must catch is a signed
    // table that disagrees with the unsigned one it refines.
    assert!(!all.is_empty(), "the sweep produced no comparisons at all");
    // Rule 2 must be *able* to fire and *able* not to: a filter that
    // excluded everything, or nothing, on every dataset would not be a
    // filter (`docs/instrument_registry.md` §1).
    assert!(!credible.is_empty(), "Rule 2 rejected every comparison");
    for (set, k, v) in &cell_verdicts {
        if let EffectClass::Measurable { arm, metric, sigma, .. } = v {
            let matching = all.iter().find(|(s, b, e)| {
                s == set && b == k && &e.arm == arm && e.metric == *metric
            });
            let e = matching.unwrap_or_else(|| {
                panic!("{set} k={k:?}: classify_effect named {arm}/{metric}, \
                        which signed_effects did not produce")
            });
            let se = e.2.sigma.expect("a Measurable cell has a denominator");
            assert!(
                (se - sigma).abs() < 1e-9,
                "{set} k={k:?}: signed table says {se:+.4}σ, classifier says \
                 {sigma:+.4}σ — the two views of one measurement disagree"
            );
        }
    }
}

/// The classifier's magnitude ranking must not have changed when the sign
/// was added, or the signed table is not a refinement of the C0–C5 one and
/// nothing published stays comparable.
#[test]
fn signing_the_classifier_preserves_which_effect_is_largest() {
    // Two arms against random. `hi` is a large NEGATIVE effect, `lo` a
    // small positive one. Under `|delta|` ranking `hi` wins; that must
    // still be true now that the value carries a sign.
    let cell = Stage0aCell {
        budget: Some(4),
        arms: vec![
            (
                "random".into(),
                ArmStats {
                    n_seeds: 20,
                    f2_mean: 0.50,
                    f2_sd: 0.10,
                    ..Default::default()
                },
            ),
            (
                "degree_high".into(),
                ArmStats {
                    n_seeds: 20,
                    f2_mean: 0.10,
                    f2_sd: 0.10,
                    ..Default::default()
                },
            ),
            (
                "degree_low".into(),
                ArmStats {
                    n_seeds: 20,
                    f2_mean: 0.60,
                    f2_sd: 0.10,
                    ..Default::default()
                },
            ),
        ],
    };

    match cell.classify_effect() {
        EffectClass::Measurable { arm, sigma, delta, .. } => {
            assert_eq!(arm, "degree_high", "the largest-magnitude effect still wins");
            assert!(sigma < 0.0, "and it is now reported as negative: {sigma}");
            assert!(delta < 0.0);
            assert!(cell.classify_effect().meets_exit(), "magnitude, not sign");
        }
        other => panic!("expected a measurable effect, got {other:?}"),
    }

    // The signed view sees both, including the one the classifier discards.
    let signed = cell.signed_effects();
    let f2: Vec<_> = signed.iter().filter(|e| e.metric == "f2").collect();
    assert_eq!(f2.len(), 2);
    assert!(f2.iter().any(|e| e.arm == "degree_low" && e.delta > 0.0));
    assert!(f2.iter().any(|e| e.arm == "degree_high" && e.delta < 0.0));
}

#[test]
fn a_heuristic_that_loses_still_demonstrates_causal_power() {
    // The distinction R-1 exists to draw, pinned. `growth_treadmill` k=1 is
    // the real C0–C5 case: random 0.147, degree_high 0.091 — the exit
    // criterion was met by a heuristic performing WORSE than random.
    // "Selection has causal power" is true; "this heuristic is good" is
    // false; v1 could only say the first and reported it in a way that read
    // as the second.
    let cell = Stage0aCell {
        budget: Some(1),
        arms: vec![
            (
                "random".into(),
                ArmStats {
                    n_seeds: 20,
                    interestingness_mean: 0.147,
                    interestingness_sd: 0.010,
                    ..Default::default()
                },
            ),
            (
                "degree_high".into(),
                ArmStats {
                    n_seeds: 20,
                    interestingness_mean: 0.091,
                    interestingness_sd: 0.012,
                    ..Default::default()
                },
            ),
        ],
    };

    let v = cell.classify_effect();
    assert!(v.meets_exit(), "selection demonstrably shifted the metric");
    let sigma = v.signed_sigma().expect("measurable");
    assert!(sigma < -SIGMA_BAR, "and the heuristic LOST: {sigma:+.2}σ");
    assert!(
        v.label().contains("WORSE than random"),
        "the label must say which way it went: {}",
        v.label()
    );

    // By metric name, not by index: `signed_effects` emits every metric in
    // `metric_rows` order and f2 is first, so indexing would silently test
    // the wrong comparison the moment a metric is added.
    let all = cell.signed_effects();
    let e = all
        .iter()
        .find(|e| e.metric == "interestingness")
        .expect("interestingness is compared");
    assert!(!e.beats_random(SIGMA_BAR));
    assert!(e.loses_to_random(SIGMA_BAR));
}

#[test]
fn the_signed_and_unsigned_views_cannot_drift_apart() {
    // Both are computed from one `metric_rows` list, so a metric added to
    // the classifier appears in the signed table automatically. This
    // asserts the shared list rather than the coincidence.
    let cell = Stage0aCell {
        budget: Some(4),
        arms: vec![
            (
                "random".into(),
                ArmStats {
                    n_seeds: 20,
                    f2_mean: 0.5,
                    f2_sd: 0.1,
                    interestingness_mean: 0.3,
                    interestingness_sd: 0.05,
                    active_persistent_mean: 2.0,
                    active_persistent_sd: 0.4,
                    ..Default::default()
                },
            ),
            (
                "degree_high".into(),
                ArmStats {
                    n_seeds: 20,
                    f2_mean: 0.2,
                    f2_sd: 0.1,
                    interestingness_mean: 0.31,
                    interestingness_sd: 0.05,
                    active_persistent_mean: 1.0,
                    active_persistent_sd: 0.4,
                    ..Default::default()
                },
            ),
        ],
    };
    let metrics: Vec<&str> = cell.signed_effects().iter().map(|e| e.metric).collect();
    assert_eq!(metrics, vec!["f2", "interestingness", "active_persistent"]);
}
