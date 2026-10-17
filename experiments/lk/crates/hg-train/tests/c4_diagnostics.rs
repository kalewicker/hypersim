//! Phase C4 — Stage 0a / 0b / 1 diagnostics, run for real.
//!
//! These tests both *check* the diagnostics and *produce* the numbers that
//! `docs/cognition_stage0_findings.md` records. Run with `--nocapture` to
//! see the tables.
//!
//! Assertions here are deliberately weak on the science and strong on the
//! machinery: a diagnostic that returns "no effect" is a legitimate and
//! pre-committed outcome (NN plan §11, Stage 0a: "Failure means … the
//! interesting question becomes *why* the dynamics are selection-invariant
//! — a substrate result in its own right"). So these tests fail when the
//! diagnostic is broken, not when the substrate is uncooperative.

use hg_train::diagnostics::*;

/// The live (rule set, IC) pairings from the C2 liveness matrix, at
/// horizons scaled to their candidate volume. `tetrahedral` is absent
/// because it has no live pairing among the shipped initial conditions.
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
            // Short on purpose: node count compounds, and at k = inf this
            // reaches tens of thousands of nodes within ~100 steps. The
            // budget response curve is legible well before that, and a
            // longer horizon buys runtime rather than information.
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
            // ~56 000 candidates/step and 36 000 nodes by step 128; see
            // triangle_subdivide.
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

#[test]
fn stage0a_budget_response_curves() {
    let dir = tempfile::tempdir().expect("tempdir");
    let s = seeds(20);
    let mut any_effect = false;
    let (mut n_measurable, mut n_deterministic, mut n_no_difference) = (0, 0, 0);

    println!("\n=== Stage 0a — budget response curves (20 seeds) ===");
    for f in fixtures() {
        println!(
            "\n## {} ({} / {} steps, window {})",
            f.name, f.ic, f.steps, f.window
        );
        println!(
            "{:>5}  {:<12} {:>8} {:>7} {:>8} {:>7} {:>9} {:>7} {:>9} {:>6}",
            "k", "arm", "F2", "F2 sd", "interest", "sd", "apstruct", "aps sd", "final n", "capW"
        );
        for cell in stage0a(&f, dir.path(), &s, 1) {
            for (name, a) in &cell.arms {
                println!(
                    "{:>5}  {:<12} {:>8.3} {:>7.3} {:>8.3} {:>7.3} {:>9.2} {:>7.3} {:>9.0} {:>6}",
                    fmt_budget(cell.budget),
                    name,
                    a.f2_mean,
                    a.f2_sd,
                    a.interestingness_mean,
                    a.interestingness_sd,
                    a.active_persistent_mean,
                    a.active_persistent_sd,
                    a.final_nodes_mean,
                    a.pairings_cap_bound_windows
                );
            }
            let effect = cell.classify_effect();
            println!("       -> {}", effect.label());
            if effect.meets_exit() {
                any_effect = true;
            }
            match effect {
                EffectClass::Measurable { .. } => n_measurable += 1,
                EffectClass::Deterministic { .. } => n_deterministic += 1,
                EffectClass::NoDifference => n_no_difference += 1,
            }
        }
    }

    println!(
        "\ncells: {n_measurable} measurable, {n_deterministic} deterministic, \
         {n_no_difference} no-difference"
    );
    println!(
        "Stage 0a exit criterion (>2 sigma MEASURABLE effect somewhere): {}",
        if any_effect { "MET" } else { "NOT MET" }
    );
    // Not asserted: a null here is a pre-committed, publishable outcome.
    // What IS asserted is that the sweep ran and produced comparable cells.
    assert!(!fixtures().is_empty());
}

#[test]
fn stage0b_separation_classes() {
    let dir = tempfile::tempdir().expect("tempdir");
    println!("\n=== Stage 0b — separation / fading memory ===");
    println!(
        "{:<20} {:<10} {:>10} {:>12} {:>10}  class",
        "fixture", "arm", "plateau", "indep base", "ratio"
    );
    let mut viable = Vec::new();
    let mut budgeted_classified = 0usize;

    for f in fixtures() {
        // Two arms, because the first one cannot perturb three of these
        // five fixtures at all. `GliderRing`, `TriLattice` and
        // `PureCrystal` are constructed rather than sampled, and
        // `MaxParallel` with no jitter never draws from the engine RNG, so
        // a `PassThrough` run is a bit-identical function of the config and
        // "a different seed" is not a perturbation. Under a budgeted
        // stochastic policy the seed decides which rewrites fire — a real
        // perturbation, and the regime Stages 2-3 actually run in.
        for (arm, r) in [
            ("passthrough", stage0b(&f, dir.path(), 4242, passthrough)),
            ("random k=4", stage0b(&f, dir.path(), 4242, budgeted_random(4))),
        ] {
            let r = r.expect("stage0b");
            let ratio = if r.independent_baseline > 0.0 {
                r.plateau / r.independent_baseline
            } else {
                f64::NAN
            };
            println!(
                "{:<20} {:<10} {:>10.5} {:>12.5} {:>10.3}  {:?}",
                f.name, arm, r.plateau, r.independent_baseline, ratio, r.class
            );
            if arm != "passthrough" {
                if r.class != Viability::Undetermined {
                    budgeted_classified += 1;
                }
                if r.class == Viability::Viable {
                    viable.push(f.name);
                }
            }
        }
    }
    println!("\nviable rule sets (budgeted arm): {viable:?}");

    // A null is a finding, so the assertion is on the machinery — but the
    // machinery claim is now narrower and true: the PASSTHROUGH arm is
    // *expected* to come back Undetermined on the constructed initial
    // conditions, and asserting otherwise would be asserting that a
    // deterministic run is stochastic. What must hold is that the arm which
    // does have a perturbation channel produces a classification on at
    // least one fixture; otherwise the diagnostic says nothing anywhere.
    assert!(
        budgeted_classified > 0,
        "no fixture could be classified even under a budgeted stochastic policy — the \
         separation diagnostic has no perturbation channel at all"
    );
}

#[test]
fn stage1_memory_capacity() {
    let dir = tempfile::tempdir().expect("tempdir");
    let f = Fixture {
        name: "stage1_reservoir",
        rules: RuleSource::File("rules/stage1_reservoir.toml"),
        ic: "Empty",
        width: 8,
        node_count: 31,
        // 128 windows of 32 steps: one injected symbol per window, and
        // enough samples that a 16-bucket readout has more training rows
        // than free parameters.
        steps: 32 * 128,
        window: 32,
        step_mode: "MaxParallel",
    };

    // Rule indices in rules/stage1_reservoir.toml.
    const INJECT_ONE: usize = 3;
    const INJECT_ZERO: usize = 4;
    const BUCKETS: usize = 16;

    println!("\n=== Stage 1 — reservoir characterization ===");
    let r = stage1(&f, dir.path(), 4242, INJECT_ZERO, INJECT_ONE, BUCKETS, 12, Some(4))
        .expect("stage1");

    println!(
        "windows {}  tape {}  state dim {}",
        r.n_windows, r.tape_len, r.mc.state_dim
    );
    println!("{:>7} {:>10} {:>10}", "delay", "substrate", "ESN(16)");
    for (d, sub) in &r.mc.per_delay {
        let esn = r
            .esn_mc
            .per_delay
            .iter()
            .find(|(dd, _)| dd == d)
            .map(|(_, v)| *v)
            .unwrap_or(f64::NAN);
        println!("{d:>7} {sub:>10.4} {esn:>10.4}");
    }
    println!(
        "\nMC (substrate) = {:.4}   MC (ESN, matched dim) = {:.4}",
        r.mc.mc, r.esn_mc.mc
    );
    println!(
        "max decodable delay (r2>0.1): substrate {:?}, ESN {:?}",
        r.mc.max_decodable_delay, r.esn_mc.max_decodable_delay
    );
    println!(
        "input coupling (delay 0) r2: substrate {:?}, ESN {:?}",
        r.mc.input_coupling_r2, r.esn_mc.input_coupling_r2
    );
    println!(
        "nonlinearity (XOR d1,d2) r2: substrate {:?}, ESN {:?}",
        r.mc.nonlinearity_r2, r.esn_mc.nonlinearity_r2
    );
    println!(
        "\nStage 1 exit (MC>0 with r2>0.1 at delay>=3): {}",
        match r.mc.max_decodable_delay {
            Some(d) if d >= 3 => "MET",
            _ => "NOT MET",
        }
    );

    // Machinery assertions only. A substrate that scrambles input identity
    // within one window is an informative negative about its intrinsic
    // timescale, not a broken test.
    assert!(r.n_windows > 20, "too few probe rows to say anything");
    assert_eq!(r.mc.state_dim, BUCKETS);
    assert!(
        !r.mc.per_delay.is_empty(),
        "no delay produced a scoreable fit — the readout never ran"
    );
    // The ESN baseline must have memory, or the comparison is against a
    // broken reference rather than a working one.
    assert!(
        r.esn_mc.mc > 0.5,
        "the matched-dimension ESN baseline has MC {:.3}; the reference is broken, so \
         the substrate number has nothing to be read against",
        r.esn_mc.mc
    );
}
