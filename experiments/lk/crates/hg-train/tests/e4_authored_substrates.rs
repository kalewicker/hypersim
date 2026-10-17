//! Phase E4 — substrate authoring and Stage 8 replication
//! (`hypersim_nn_experiments_plan_v3.md` §8.3, §9.4, §10).
//!
//! > **Exit:** the F5 learning margin reproduces on ≥1 substrate other than
//! > `tri_two` at > 2σ champion − untrained.
//! > **Failure means:** the D6 margin is a property of `tri_two`'s specific
//! > rule pair, not of selection-learning on hypergraph rewriting.
//!
//! The authored substrates are `rules/relay_pair.toml` and
//! `rules/fan_merge.toml`; each file's header carries its authoring note and
//! its plan-risk-2 statement.

use hg_cognition::features::FeatureSet;
use hg_cognition::{CognitionConfig, MlpScorer, Parameterized, PolicyKind, RandomSubset, ScoreNorm};
use hg_observe::RuleSelectionPolicy;
use hg_train::campaign::{n_params_for, CampaignSpec};
use hg_train::config::TrainingConfig;
use hg_train::diagnostics::{Fixture, RuleSource};
use hg_train::fitness::FitnessFn;
use hg_train::prechecks::{self, class_extreme};
use hg_train::screen::rule_dependency_screen;

const STEPS: u64 = 384;
const WINDOW: u64 = 6;
const BURN_IN: usize = 5;
const BUDGET: usize = 4;
const HIDDEN: usize = 16;
const HOLDOUT: [u64; 5] = [9_600_001, 9_600_002, 9_600_003, 9_600_004, 9_600_005];

/// Candidate `(rule file, IC, n_rules)`. Both authored sets are run on more
/// than one initial condition, because D5's nine-pairing sweep found that a
/// rule set's liveness is a property of the *pairing*, not of the rules.
fn candidates() -> Vec<(&'static str, &'static str, &'static str, usize)> {
    vec![
        ("relay_pure", "rules/relay_pair.toml", "PureCrystal", 2),
        ("relay_wn", "rules/relay_pair.toml", "WhiteNoise", 2),
        ("relay_tri", "rules/relay_pair.toml", "TriLattice", 2),
        ("fan_pure", "rules/fan_merge.toml", "PureCrystal", 2),
        ("fan_wn", "rules/fan_merge.toml", "WhiteNoise", 2),
        ("fan_tri", "rules/fan_merge.toml", "TriLattice", 2),
    ]
}

fn fixture(name: &'static str, rules: &'static str, ic: &'static str) -> Fixture {
    Fixture {
        name,
        rules: RuleSource::File(rules),
        ic,
        width: 8,
        node_count: 24,
        steps: STEPS,
        window: WINDOW,
        step_mode: "MaxParallel",
    }
}

fn mlp_cognition(n_rules: usize) -> CognitionConfig {
    let _ = n_rules;
    let mut c = CognitionConfig {
        policy: PolicyKind::MlpScorer,
        budget: Some(BUDGET),
        ..Default::default()
    };
    c.mlp.hidden = HIDDEN;
    c
}

fn training() -> TrainingConfig {
    let mut t = TrainingConfig::default();
    t.fitness = FitnessFn::F5ConstructorReuse;
    t.horizon_steps = STEPS;
    t.burn_in_windows = BURN_IN;
    t.holdout_seeds = HOLDOUT.to_vec();
    t
}

fn load(path: &str) -> Vec<hg_rules::CompiledRule> {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path);
    let raw = hg_rules::load_rules_from_file(&p).expect("parse");
    hg_rules::compile_rules(&raw).expect("compile")
}

// =====================================================================
// Part 1 — the static screen, before any rollout
// =====================================================================

/// §8.2's whole point is that this costs nothing and runs first. An
/// authored rule set that cannot pass its own design criterion should never
/// reach a rollout, and finding that out here rather than after a campaign
/// is the difference the screen was built to make.
///
/// A pass is still only a **candidate** (plan risk 5) — part 2 decides.
#[test]
fn authored_substrates_pass_the_static_screen() {
    for path in ["rules/relay_pair.toml", "rules/fan_merge.toml"] {
        let r = rule_dependency_screen(&load(path));
        println!("\n## {path}\n    {}", r.detail());
        assert!(
            r.passes(),
            "{path} was authored against §8.2 and must pass its own screen: {}",
            r.detail()
        );
        assert!(
            r.sustaining_components.iter().any(|c| c.len() >= 2),
            "{path} needs a genuine two-rule cycle, not a self-loop: {:?} / self-loops {:?}",
            r.sustaining_components,
            r.self_loops
        );
    }
}

// =====================================================================
// Part 2 — P-1 across pairings, with plan §5.2's >= 5 draws
// =====================================================================

/// P-1 on every (rule set, IC) pairing, **five independent draws each**.
///
/// D5 ran P-1 once per pairing. E2 §5 is the standing argument against
/// that: the 8-draw class control gave a materially wrong answer that 30
/// draws corrected, and a gate resolved by whichever draw ran first is
/// exactly what plan §5.2 forbids. Co-liveness is a go/no-go, so it gets
/// the discipline.
///
/// Reported per pairing: the modal verdict, the fraction agreeing with it,
/// and the co-live step counts — so a marginal pass is visible as marginal
/// rather than as a pass.
#[test]
#[ignore = "E4 part 2: P-1 across pairings at 5 draws; run explicitly"]
fn p1_across_pairings_at_five_draws() {
    println!("\n=== E4 part 2 — P-1, five draws per pairing ===");
    println!(
        "{:<14} {:<12} {:>10}  {}",
        "pairing", "ic", "verdict", "detail"
    );

    for (name, rules, ic, n_rules) in candidates() {
        let f = fixture(name, rules, ic);
        let cog = mlp_cognition(n_rules);
        let mut draws = Vec::new();
        let mut details = Vec::new();
        for d in 0..5u64 {
            let dir = tempfile::tempdir().expect("tempdir");
            let cfg = f.sim_config_causal(dir.path(), 9_700_000 + d);
            match prechecks::p1_class_expressiveness(&cog, &cfg, n_rules, 64, 9_700_100 + d) {
                Ok(r) => {
                    draws.push(true);
                    details.push(r.detail);
                }
                Err(e) => {
                    draws.push(false);
                    details.push(format!("[{}] {}", e.code(), e.detail()));
                }
            }
        }
        let pass = draws.iter().filter(|b| **b).count();
        let verdict = if pass == draws.len() {
            "PASS 5/5"
        } else if pass == 0 {
            "abort 0/5"
        } else {
            "UNSTABLE"
        };
        println!(
            "{:<14} {:<12} {:>10}  {}",
            name,
            ic,
            verdict,
            details.last().cloned().unwrap_or_default()
        );
        if pass > 0 && pass < draws.len() {
            for (i, d) in details.iter().enumerate() {
                println!("      draw {i}: {d}");
            }
        }
    }
}

// =====================================================================
// Part 3 — P-3 / P-4 on whatever cleared P-1
// =====================================================================

/// The remaining prechecks, on the pairings part 2 passes. Same probe
/// construction as D6/E2 so a comparison across substrates is a comparison
/// of substrates.
#[test]
#[ignore = "E4 part 3: P-3/P-4 on the surviving pairings; run explicitly"]
fn p3_p4_on_surviving_pairings() {
    println!("\n=== E4 part 3 — P-3 / P-4 ===");

    for (name, rules, ic, n_rules) in candidates() {
        let f = fixture(name, rules, ic);
        let cog = mlp_cognition(n_rules);
        let n = n_params_for(&cog, n_rules);
        let dir = tempfile::tempdir().expect("tempdir");
        let cfg = f.sim_config_causal(dir.path(), 9_700_000);
        let t = training();

        println!("\n## {name} ({ic})  n_params={n}");

        let extreme = class_extreme(PolicyKind::MlpScorer, n, 4242);
        let init = prechecks::class_init(PolicyKind::MlpScorer, n, 9_600_000u64 ^ 0x1_c001_1u64);
        let mk = |g: Vec<f32>| {
            move |s: u64| {
                let mut p = MlpScorer::with_features_and_norm(
                    n_rules,
                    cog.wl_buckets,
                    HIDDEN,
                    Some(BUDGET),
                    s,
                    FeatureSet::V1,
                    ScoreNorm::PoolStandard,
                );
                p.set_params(&g);
                Box::new(p) as Box<dyn RuleSelectionPolicy>
            }
        };
        let arms = vec![
            prechecks::probe_arm("random", &cfg, &t, &HOLDOUT, |s| {
                Box::new(RandomSubset::new(Some(BUDGET), s)) as Box<dyn RuleSelectionPolicy>
            }),
            prechecks::probe_arm("campaign init", &cfg, &t, &HOLDOUT, mk(init)),
            prechecks::probe_arm("class extreme", &cfg, &t, &HOLDOUT, mk(extreme)),
        ];
        for a in &arms {
            println!(
                "    {:<15} {:>10.4} +- {:>8.4}  (n={}, unmeasured={})",
                a.label,
                a.mean(),
                a.sd(),
                a.scores.len(),
                a.unmeasured
            );
        }
        match prechecks::p3_objective_discrimination(&arms) {
            Ok(r) => println!("    P-3  PASS  {}", r.detail),
            Err(e) => println!("    P-3  ABORT [{}] {}", e.code(), e.detail()),
        }
        match prechecks::p4_signal_non_saturation(FitnessFn::F5ConstructorReuse, &arms[0]) {
            Ok(r) => println!("    P-4  PASS  {}", r.detail),
            Err(e) => println!("    P-4  ABORT [{}] {}", e.code(), e.detail()),
        }
    }
}

// =====================================================================
// Part 4 — Stage 8: the D6/E2 campaign design, on authored substrates
// =====================================================================

/// The pairings that cleared parts 2 and 3. `relay_wn` is absent: it aborts
/// P-3 `OBJECTIVE_BLIND`, and running it anyway would be running a campaign
/// the prechecks refused.
fn stage8_cells() -> Vec<(&'static str, &'static str, &'static str, usize)> {
    vec![
        // New rules on tri_two's OWN initial condition — so the only thing
        // that differs from the D6/E2 result is the rule pair itself. This
        // is the cell that most directly tests §9.4's failure clause.
        ("relay_tri", "rules/relay_pair.toml", "TriLattice", 2),
        ("fan_tri", "rules/fan_merge.toml", "TriLattice", 2),
        // And a different initial condition as well, so a reproduction is
        // not confined to one substrate geometry.
        ("fan_pure", "rules/fan_merge.toml", "PureCrystal", 2),
    ]
}

/// Stage 8 (plan §9.4).
///
/// **Identical campaign design to E2 arm A** — 16 pairs × 2 seeds × 30
/// generations, σ = 0.10, α = 0.032, N-1, `feature_set` v1, the same
/// `campaign_seed` and the same frozen hold-out. Nothing is retuned per
/// substrate, deliberately: a margin that only appears after per-substrate
/// tuning is not a replication, and E2 §2.4 already established that no
/// single σ is right everywhere, so re-tuning here would confound
/// "the result reproduces" with "the result reproduces once σ is fitted".
///
/// **Exit:** > 2σ `champion − untrained` on ≥ 1 substrate other than
/// `tri_two`.
#[test]
#[ignore = "E4 part 4: Stage 8 campaigns; hours, run explicitly"]
fn stage8_campaigns_on_authored_substrates() {
    println!("\n=== E4 part 4 — Stage 8 replication ===");

    for (name, rules, ic, n_rules) in stage8_cells() {
        let f = fixture(name, rules, ic);
        let dir = tempfile::tempdir().expect("tempdir");
        let cfg = f.sim_config_causal(dir.path(), 9_600_000);
        let mut t = training();
        t.population_pairs = 16;
        t.seeds_per_member = 2;
        t.generations = 30;
        t.sigma = 0.10;
        t.alpha = 0.032;
        t.campaign_seed = 9_600_000;

        let cog = mlp_cognition(n_rules);
        println!(
            "\n## {name}  ({ic}, {} rules, {} params)",
            n_rules,
            n_params_for(&cog, n_rules)
        );

        let spec = CampaignSpec {
            campaign_id: format!("e4_{name}"),
            base_dir: dir.path().to_path_buf(),
            sim_config: &cfg,
            cognition: cog,
            training: t,
        };
        let t0 = std::time::Instant::now();
        match hg_train::campaign::run_campaign(&spec) {
            Ok(o) => {
                for p in &o.prechecks {
                    println!(
                        "    {:<28} {:<5} {}",
                        p.name,
                        if p.passed { "PASS" } else { "fail" },
                        p.detail
                    );
                }
                let a = &o.arm_table;
                println!(
                    "    arms: random {:.4}+-{:.4}  untrained {:.4}+-{:.4}  champion {:.4}+-{:.4}",
                    a.random.mean(),
                    a.random.sd(),
                    a.untrained.mean(),
                    a.untrained.sd(),
                    a.champion.mean(),
                    a.champion.sd()
                );
                match a.champion_vs_untrained() {
                    Some((d, s)) => println!(
                        "    PRIMARY champion - untrained = {d:+.4} at {s:.2}sigma  => exit {}",
                        if s > 2.0 { "MET" } else { "NOT met" }
                    ),
                    None => println!("    PRIMARY  no margin (an arm was never measured)"),
                }
                match a.champion_vs_random() {
                    Some((d, s)) => {
                        println!("    (reported) champion - random = {d:+.4} at {s:.2}sigma")
                    }
                    None => println!("    (reported) champion - random = no margin"),
                }
                println!(
                    "    {:<12} {:>10} {:>14} {:>10} {:>8}",
                    "arm", "fitness", "active_persist", "n_nodes", "flags"
                );
                for want in ["random", "untrained", "champion"] {
                    let rows: Vec<_> =
                        o.final_holdout_rows.iter().filter(|r| r.arm == want).collect();
                    if rows.is_empty() {
                        continue;
                    }
                    let avg = |g: fn(&hg_train::row::CampaignRow) -> Option<f64>| {
                        let v: Vec<f64> = rows.iter().filter_map(|r| g(r)).collect();
                        if v.is_empty() {
                            f64::NAN
                        } else {
                            v.iter().sum::<f64>() / v.len() as f64
                        }
                    };
                    let flags = rows
                        .iter()
                        .filter(|r| {
                            r.flag_fitness_structure_divergence
                                || r.flag_degenerate_occupancy
                                || r.flag_degenerate_reuse
                        })
                        .count();
                    println!(
                        "    {:<12} {:>10.4} {:>14.2} {:>10.1} {:>8}",
                        want,
                        avg(|r| r.fitness),
                        avg(|r| r.mean_active_persistent),
                        avg(|r| r.mean_n_nodes),
                        flags
                    );
                }
                println!("    {:.1} min", t0.elapsed().as_secs_f64() / 60.0);
            }
            Err(e) => println!("    campaign refused: {e}"),
        }
    }
}

// =====================================================================
// Part 5 — the class control, per substrate
// =====================================================================

/// E2 §5's control, repeated per authored substrate.
///
/// E2 established that `champion − untrained` in **seed** sd answers "did
/// training beat this one draw", and that under N-1 `class_init` samples a
/// *direction* whose spread dwarfs the seed spread. It also established that
/// the direction distribution is badly skewed, so the parametric z measures
/// the wrong tail and the statistic that means something is the
/// nonparametric rank.
///
/// Champion values are pasted from part 4 rather than recomputed, for the
/// same reason E2's version hard-codes arm A's: this test adjudicates
/// published numbers and should fail loudly if they are ever re-derived
/// differently.
#[test]
#[ignore = "E4 part 5: 30-draw class control per substrate; run explicitly"]
fn class_control_on_authored_substrates() {
    println!("\n=== E4 part 5 — how good is a random direction, per substrate? ===");

    // (cell, champion from part 4). Filled in after part 4 reports.
    let champions: Vec<(&str, f64)> = vec![
        ("relay_tri", 601.1094),
        ("fan_tri", 677.3790),
        ("fan_pure", 655.9072),
    ];

    for (name, rules, ic, n_rules) in stage8_cells() {
        let champ = champions
            .iter()
            .find(|(c, _)| *c == name)
            .map(|(_, v)| *v)
            .unwrap_or(f64::NAN);
        let f = fixture(name, rules, ic);
        let cog = mlp_cognition(n_rules);
        let n = n_params_for(&cog, n_rules);
        let dir = tempfile::tempdir().expect("tempdir");
        let cfg = f.sim_config_causal(dir.path(), 9_600_000);
        let t = training();

        let mut means: Vec<f64> = Vec::new();
        for k in 0..30u64 {
            let g = prechecks::class_init(PolicyKind::MlpScorer, n, 0xE2_D00D + k);
            let a = prechecks::probe_arm("draw", &cfg, &t, &HOLDOUT, {
                let g = g.clone();
                move |s: u64| {
                    let mut p = MlpScorer::with_features_and_norm(
                        n_rules,
                        cog.wl_buckets,
                        HIDDEN,
                        Some(BUDGET),
                        s,
                        FeatureSet::V1,
                        ScoreNorm::PoolStandard,
                    );
                    p.set_params(&g);
                    Box::new(p) as Box<dyn RuleSelectionPolicy>
                }
            });
            means.push(a.mean());
        }
        means.retain(|v| v.is_finite());
        let m = means.iter().sum::<f64>() / means.len() as f64;
        let sd = (means.iter().map(|x| (x - m).powi(2)).sum::<f64>()
            / (means.len() - 1) as f64)
            .sqrt();
        let mut sorted = means.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        println!("\n## {name} ({ic})");
        println!(
            "   {} draws: {m:.4} +- {sd:.4}   (min {:.2}, max {:.2})",
            means.len(),
            sorted[0],
            sorted[sorted.len() - 1]
        );
        println!(
            "   deciles: {}",
            (0..=10)
                .map(|i| format!("{:.0}", sorted[(i * (sorted.len() - 1)) / 10]))
                .collect::<Vec<_>>()
                .join(" ")
        );
        if champ.is_finite() {
            println!(
                "   champion {champ:.4}:  parametric {:.2} direction-sd;  \
                 nonparametric {}/{} draws reach it (best {:.2})",
                (champ - m) / sd.max(1e-9),
                means.iter().filter(|v| **v >= champ).count(),
                means.len(),
                sorted[sorted.len() - 1]
            );
        }
    }
}
