//! Phase C5 — Stage 2: evolved rule-selection bias (NN plan §11).
//!
//! **Hypothesis:** a per-rule preference vector can reliably steer the
//! substrate into target regimes.
//! **Exit:** champion beats budget-matched random by > 2σ on the five
//! frozen hold-out seeds, no Goodhart flags, on ≥ 1 non-fixture rule set.
//!
//! ## Why the rule sets here are not the rule sets Stage 0a swept
//!
//! Every builtin rule set ships exactly **one** rule, and `StaticBias` has
//! one parameter per rule. A masked softmax over one logit is 1.0 whatever
//! the logit is, so on a single-rule set the genome cannot change the
//! selection at all and the policy is identical to budget-matched random
//! (pinned by `hg-cognition`'s
//! `a_single_rule_genome_cannot_change_selection`; `run_campaign` refuses
//! the configuration outright). Stage 2's hypothesis is not false on
//! `glider` or `binary_edge` — it is unstateable there.
//!
//! `rules/wn_three.toml` and `rules/tri_two.toml` are compositions of
//! rules the §2.5 liveness matrix measured co-live on a shared initial
//! condition. They are substrates, not fixtures: no rule was planted as
//! the right answer.
//!
//! Stage 0a's response curve was measured on the single-rule sets, so it
//! does not transfer — the budget probe below re-measures it on the
//! composed sets before any campaign is sized.

use hg_train::diagnostics::{
    stage0a_with_ladder, EffectClass, Fixture, RuleSource, BUDGET_LADDER,
};
use hg_train::stage2::{run_case, Stage2Case};

fn wn_three(steps: u64, window: u64) -> Fixture {
    Fixture {
        name: "wn_three",
        rules: RuleSource::File("rules/wn_three.toml"),
        ic: "WhiteNoise",
        width: 8,
        node_count: 24,
        steps,
        window,
        step_mode: "MaxParallel",
    }
}

fn tri_two(steps: u64, window: u64) -> Fixture {
    Fixture {
        name: "tri_two",
        rules: RuleSource::File("rules/tri_two.toml"),
        ic: "TriLattice",
        width: 8,
        node_count: 24,
        steps,
        window,
        step_mode: "MaxParallel",
    }
}

fn fmt_budget(b: Option<usize>) -> String {
    b.map(|k| k.to_string()).unwrap_or_else(|| "inf".into())
}

/// Stage 0a, re-run on the composed substrates. Picks the budget the
/// campaigns use, and is a gate in its own right: a composed rule set with
/// no measurable selection effect is not worth a campaign.
#[test]
#[ignore = "long-running; run explicitly for the Stage 2 budget table"]
fn stage2_budget_probe() {
    let dir = tempfile::tempdir().expect("tempdir");
    let seeds: Vec<u64> = (0..12u64).map(|i| 7_100_000 + i * 101).collect();

    println!("\n=== Stage 2 budget probe — composed multi-rule substrates ===");
    for f in [wn_three(192, 24), tri_two(192, 24)] {
        println!("\n## {} ({} / {} steps, window {})", f.name, f.ic, f.steps, f.window);
        println!(
            "{:>5}  {:<12} {:>8} {:>7} {:>8} {:>7} {:>9} {:>6}",
            "k", "arm", "F2", "F2 sd", "interest", "sd", "final n", "capW"
        );
        // No `k = ∞` rung: see `stage0a_with_ladder`. Stage 2 refuses an
        // unbounded budget, and on a composed set containing `binary_edge`
        // that arm is the one that reaches five figures of nodes.
        for cell in stage0a_with_ladder(&f, dir.path(), &seeds, 1, &BUDGET_LADDER[..4]) {
            for (name, a) in &cell.arms {
                println!(
                    "{:>5}  {:<12} {:>8.3} {:>7.3} {:>8.3} {:>7.3} {:>9.0} {:>6}",
                    fmt_budget(cell.budget),
                    name,
                    a.f2_mean,
                    a.f2_sd,
                    a.interestingness_mean,
                    a.interestingness_sd,
                    a.final_nodes_mean,
                    a.pairings_cap_bound_windows
                );
            }
            println!("       -> {}", cell.classify_effect().label());
            if let EffectClass::Measurable { sigma, .. } = cell.classify_effect() {
                if sigma > 2.0 {
                    println!("          (budget candidate)");
                }
            }
        }
    }
}

/// Softmax of the champion logits — what the genome actually says, in the
/// units the policy uses it in. Raw logits are shift-invariant and so are
/// not comparable across campaigns; the allocation probabilities are.
fn allocation(logits: &[f32]) -> Vec<f32> {
    let m = logits.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let e: Vec<f32> = logits.iter().map(|l| (l - m).exp()).collect();
    let s: f32 = e.iter().sum();
    e.iter().map(|x| x / s).collect()
}

/// CMA-ES step size, in logit units.
///
/// **Not the `TrainingConfig` default of 0.02**, which is wrong for this
/// policy class. `StaticBias`'s parameters are softmax logits and the
/// genome starts at all-zeros; separating two rules takes O(1)-O(10) of
/// logit movement, and a 40-generation campaign at sigma = 0.02 explores a
/// neighbourhood over which the softmax is nearly uniform and every
/// population member scores identically. The landscape sweep measured the
/// consequence: on `tri_two` at k=16 an untrained genome scores 0.9455
/// while a reachable one-hot genome scores 1.0000, and the sigma = 0.02
/// campaign converged to uniform and missed it.
const STAGE2_SIGMA: f32 = 2.0;

/// The Stage 2 campaigns.
#[test]
#[ignore = "long-running; run explicitly for the Stage 2 results table"]
fn stage2_campaigns() {
    let base = std::path::PathBuf::from(
        std::env::var("HG_STAGE2_DIR").unwrap_or_else(|_| "runs_stage2".into()),
    );
    std::fs::create_dir_all(&base).expect("base dir");

    // Budgets come from `stage2_budget_probe`, not from Stage 0a: Stage 0a
    // swept the single-rule sets, and candidate volume — which is what a
    // budget is relative to — changes when rules are composed.
    //
    // `k = 16` on both, chosen on two criteria the exit criterion actually
    // needs. **Headroom**: budget-matched random scores F2 0.321 on
    // `wn_three` and 0.607 on `tri_two` there, so a champion has somewhere
    // to go — at `k = 1` random already scores 0.786 / 0.929 and the
    // measurable ceiling is a rounding error away. **Baseline variance**:
    // random's F2 sd is 0.273 / 0.083 at `k = 16` against 0.000 at `k = 4`
    // on `tri_two`, and a σ needs a denominator. Every rung was measurable,
    // so this is a choice among live options, not the only live one.
    let cases = vec![
        Stage2Case {
            fixture: wn_three(384, 24),
            budget: Some(16),
            generations: 40,
            seeds_per_member: 3,
            campaign_seed: 20260726,
            burn_in_windows: 5,
            sigma: STAGE2_SIGMA,
            stage0a_verdict: "composed substrate; k=16 measurable at 3.49 sigma",
        },
        Stage2Case {
            fixture: tri_two(384, 24),
            budget: Some(16),
            generations: 40,
            seeds_per_member: 3,
            campaign_seed: 20260726,
            burn_in_windows: 5,
            sigma: STAGE2_SIGMA,
            stage0a_verdict: "composed substrate; k=16 measurable at 20.78 sigma",
        },
    ];

    println!("\n=== Stage 2 — CMA-ES over StaticBias, F2, 5 frozen hold-out seeds ===");
    let mut any_exit = false;
    let mut results = Vec::new();

    for case in &cases {
        let r = run_case(case, &base).expect("stage 2 case");
        println!(
            "\n## {} (k = {:?}, {} rules, {} steps, {} generations, {:.0}s)",
            r.name, r.budget, r.n_rules, r.horizon, r.generations, r.wallclock_s
        );
        println!("  champion  {:>8.4} +/- {:.4}   {:?}", r.champion.mean(), r.champion.sd(), r.champion.per_seed);
        println!("  random    {:>8.4} +/- {:.4}   {:?}", r.baseline.mean(), r.baseline.sd(), r.baseline.per_seed);
        println!(
            "  untrained {:>8.4} +/- {:.4}   {:?}   <- StaticBias, theta = 0",
            r.untrained.mean(),
            r.untrained.sd(),
            r.untrained.per_seed
        );
        println!(
            "  margin vs untrained {:+.4}   <- what CMA-ES actually contributed",
            r.margin_vs_untrained
        );
        println!(
            "  margin {:+.4}   paired sigma {:.2}   pooled sigma {:.2}   wins {}/{}",
            r.margin,
            r.sigma_paired,
            r.sigma_pooled,
            r.wins,
            r.champion.per_seed.len()
        );
        println!(
            "  flags: divergence={} seed_overfit={} degenerate_occupancy={}",
            r.flag_fitness_structure_divergence, r.flag_seed_overfit, r.flag_degenerate_occupancy
        );
        println!(
            "  champion allocation {:?}  (hash {})",
            allocation(&r.champion_genome)
                .iter()
                .map(|p| (p * 1000.0).round() / 1000.0)
                .collect::<Vec<_>>(),
            r.genome_hash
        );
        println!("  -> exit met: {}", r.meets_exit());
        any_exit |= r.meets_exit();
        results.push(r);
    }

    println!(
        "\nStage 2 exit (>2 paired sigma over budget-matched random, no Goodhart flags, \
         on >=1 non-fixture rule set): {}",
        if any_exit { "MET" } else { "NOT MET" }
    );

    // A null is a pre-committed finding (plan §11): failure here means rule
    // identity alone carries no exploitable signal on real rule sets even
    // though the loop demonstrably works (F-L1), which sharpens Stage 3's
    // question rather than killing it. So the assertion is on the
    // machinery: every case must have produced a real comparison.
    for r in &results {
        assert_eq!(
            r.champion.per_seed.len(),
            5,
            "{}: champion was not scored on all five frozen hold-out seeds",
            r.name
        );
        assert_eq!(
            r.baseline.per_seed.len(),
            5,
            "{}: baseline was not scored on all five frozen hold-out seeds",
            r.name
        );
    }
}

/// A campaign on a single-rule set must be refused, not run.
///
/// This is the F-L0 discipline applied to Stage 2: the loop would run to
/// completion on `glider`, produce a champion, and report a hold-out margin
/// that is entirely policy-RNG noise, because the genome provably cannot
/// change what gets selected.
#[test]
fn a_single_rule_stage2_campaign_is_refused() {
    let dir = tempfile::tempdir().expect("tempdir");
    let case = Stage2Case {
        fixture: Fixture {
            name: "glider",
            rules: RuleSource::Builtin("glider"),
            ic: "GliderRing",
            width: 8,
            node_count: 31,
            steps: 512,
            window: 64,
            step_mode: "MaxParallel",
        },
        budget: Some(4),
        generations: 1,
        seeds_per_member: 1,
        campaign_seed: 4242,
        burn_in_windows: 1,
        sigma: STAGE2_SIGMA,
        stage0a_verdict: "single-rule: StaticBias inexpressible",
    };
    let err = run_case(&case, dir.path()).expect_err("a one-rule StaticBias campaign must fail");
    let msg = err.to_string();
    // Under plan v2 this refusal comes from precheck P-1 rather than from
    // the ad-hoc guard C5 shipped, so the assertion is on the abort *code*
    // — which is the stable identifier — rather than on wording that now
    // belongs to `hg_train::prechecks`.
    assert!(
        msg.contains("CLASS_INEXPRESSIBLE"),
        "expected the P-1 single-rule refusal, got: {msg}"
    );
    assert!(
        msg.contains("softmax over one logit"),
        "the refusal must still say WHY, not just that it refused: {msg}"
    );
}

/// `k = ∞` must be refused too: Stage 0a measured that at unbounded budget
/// every arm is identical on every rule set tested, so the campaign cannot
/// produce a result whatever the optimizer does.
#[test]
fn an_unbounded_budget_stage2_campaign_is_refused() {
    let dir = tempfile::tempdir().expect("tempdir");
    let case = Stage2Case {
        fixture: wn_three(192, 24),
        budget: None,
        generations: 1,
        seeds_per_member: 1,
        campaign_seed: 4242,
        burn_in_windows: 1,
        sigma: STAGE2_SIGMA,
        stage0a_verdict: "n/a",
    };
    let err = run_case(&case, dir.path()).expect_err("k = inf must be refused");
    assert!(err.to_string().contains("unbounded budget"), "got: {err}");
}
