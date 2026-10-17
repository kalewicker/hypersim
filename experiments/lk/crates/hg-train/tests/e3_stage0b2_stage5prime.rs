//! Phase E3 — Stage 0b″ and Stage 5′
//! (`hypersim_nn_experiments_plan_v3.md` §9.1, §9.2, §10).
//!
//! > **Exit:** separation classes with spreads; the emergence prediction
//! > adjudicated; F6 go/no-go.
//!
//! Stage 0b″ rebuilds the statistic D4 §2.5 specified and left unbuilt.
//! Stage 5′ adjudicates plan §7.2's **pre-registered** prediction:
//!
//! > the champion shows higher `ei_micro` and lower `ce_structural` than
//! > budget-matched random at matched budget, with `ei_macro` ambiguous in
//! > sign.

use hg_cognition::readout::ReadoutSet;
use hg_cognition::{CognitionConfig, MlpScorer, Parameterized, PolicyKind, RandomSubset, ScoreNorm};
use hg_cognition::features::FeatureSet;
use hg_observe::RuleSelectionPolicy;
use hg_train::campaign::n_params_for;
use hg_train::diagnostics::{self, Fixture, RuleSource, Viability};

const STAGE0B_BUDGET: usize = 4;
const STEPS: u64 = 384;
/// **6, matching E2 and E4 exactly.** With `burn_in_windows = 5`, a window
/// of 48 would leave 3 of 8 windows scored and the campaign would not be
/// the one whose champion Stage 5′ is supposed to be reading. The Stage 0b″
/// battery above uses its own per-fixture windows and is unaffected.
const WINDOW: u64 = 6;
const HOLDOUT: [u64; 5] = [9_600_001, 9_600_002, 9_600_003, 9_600_004, 9_600_005];

/// Stage 5′ runs on `tri_two` — the substrate plan §7.2 names — plus the
/// three E4 substrates, so the prediction is adjudicated on four rather
/// than on the one it was written for.
fn stage5_cells() -> Vec<(&'static str, &'static str, &'static str, usize)> {
    vec![
        ("tri_two", "rules/tri_two.toml", "TriLattice", 2),
        ("relay_tri", "rules/relay_pair.toml", "TriLattice", 2),
        ("fan_tri", "rules/fan_merge.toml", "TriLattice", 2),
        ("fan_pure", "rules/fan_merge.toml", "PureCrystal", 2),
    ]
}
/// Eight base seeds — above plan §5.2's floor of five, and the same count
/// D4 §2.2 used, so the pooled statistic and the sweep that condemned the
/// old one are read at equal power.
const BASE_SEEDS: [u64; 8] = [4242, 4243, 4244, 4245, 4246, 4247, 4248, 4249];

fn fx(
    name: &'static str,
    rules: RuleSource,
    ic: &'static str,
    node_count: u64,
    steps: u64,
    window: u64,
) -> Fixture {
    Fixture {
        name,
        rules,
        ic,
        width: 8,
        node_count,
        steps,
        window,
        step_mode: "MaxParallel",
    }
}

/// D4's five, unchanged, plus the four substrates the programme actually
/// runs campaigns on. The first five make the pooled statistic directly
/// comparable to the sweep that condemned the old one; the last four are
/// the ones whose class anyone would want.
fn battery() -> Vec<Fixture> {
    vec![
        fx("glider", RuleSource::Builtin("glider"), "GliderRing", 31, 512, 64),
        fx("growth_treadmill", RuleSource::Builtin("growth_treadmill"), "WhiteNoise", 24, 512, 64),
        fx("triangle_subdivide", RuleSource::Builtin("triangle_subdivide"), "TriLattice", 24, 96, 24),
        fx("binary_edge", RuleSource::Builtin("binary_edge"), "PureCrystal", 24, 96, 24),
        fx("fl1_two_rule", RuleSource::File("rules/fl1_learnable_signal.toml"), "GliderRing", 31, 192, 32),
        fx("tri_two", RuleSource::File("rules/tri_two.toml"), "TriLattice", 24, 384, 48),
        fx("relay_tri", RuleSource::File("rules/relay_pair.toml"), "TriLattice", 24, 384, 48),
        fx("fan_tri", RuleSource::File("rules/fan_merge.toml"), "TriLattice", 24, 384, 48),
        fx("fan_pure", RuleSource::File("rules/fan_merge.toml"), "PureCrystal", 24, 384, 48),
    ]
}

// =====================================================================
// Stage 0b" — the rebuilt statistic
// =====================================================================

/// Plan §9.1.
///
/// > **Exit:** every rule set has a class with a reported spread, and no
/// > class rests on a single draw. Classes that remain distribution-spanning
/// > across two categories are reported as `Undetermined` with the
/// > distribution, not resolved by fiat.
///
/// Note what the exit does *not* say: it does not require the classes to be
/// decisive. `Undetermined` with a distribution attached **is** a satisfied
/// exit, and D4 §2.2's finding predicts several of them. The failure mode
/// this replaces is a confident class computed from one triple.
#[test]
#[ignore = "E3 Stage 0b\": pooled separation over 8 draws x 2 readouts; run explicitly"]
fn stage0b2_pooled_separation() {
    println!("\n=== E3 — Stage 0b\", pooled over {} draws ===", BASE_SEEDS.len());
    println!(
        "   reference averaged over {} DISJOINT independent pairs per draw\n",
        diagnostics::REFERENCE_PAIRS
    );

    for f in battery() {
        println!("## {}", f.name);
        for readout in [ReadoutSet::R1, ReadoutSet::R2] {
            let dir = tempfile::tempdir().expect("tempdir");
            let t0 = std::time::Instant::now();
            let policy = |s: u64| {
                Box::new(RandomSubset::new(Some(STAGE0B_BUDGET), s))
                    as Box<dyn RuleSelectionPolicy>
            };
            match diagnostics::stage0b_pooled(&f, dir.path(), &BASE_SEEDS, &policy, readout) {
                Ok(p) => {
                    println!(
                        "   {:<3} {}  [{:.1}s]",
                        match readout {
                            ReadoutSet::R1 => "r1",
                            ReadoutSet::R2 => "r2",
                        },
                        p.detail(),
                        t0.elapsed().as_secs_f64()
                    );
                    // The reference's OWN spread, which is the quantity D4
                    // §2.3 blamed. Printed for the first draw only; if the
                    // reference is stable this is boring, and its being
                    // boring is the fix working.
                    if let Some(d) = p.draws.first() {
                        let s = &d.reference_samples;
                        let mn = s.iter().cloned().fold(f64::INFINITY, f64::min);
                        let mx = s.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                        println!(
                            "       draw0 reference {:.5} over {} pairs (min {:.5}, max {:.5}, \
                             spread/mean {:.2}); plateau {:.5}",
                            d.reference,
                            s.len(),
                            mn,
                            mx,
                            if d.reference > 0.0 { (mx - mn) / d.reference } else { f64::NAN },
                            d.plateau
                        );
                    }
                }
                Err(e) => println!("   {readout:?}  FAILED {e}"),
            }
        }
    }
}

/// The exit criterion, as an assertion rather than a reading.
///
/// Every rule set must return a verdict carrying its distribution, and no
/// verdict may rest on fewer than `MIN_SEPARATION_DRAWS` draws. A
/// unanimous class and an `Undetermined`-with-distribution both satisfy
/// this; a class from one draw does not, and cannot be constructed through
/// this API.
#[test]
#[ignore = "E3: the Stage 0b\" exit criterion; run explicitly"]
fn no_separation_class_rests_on_a_single_draw() {
    let f = battery().into_iter().next().expect("battery");
    let dir = tempfile::tempdir().expect("tempdir");
    let policy =
        |s: u64| Box::new(RandomSubset::new(Some(STAGE0B_BUDGET), s)) as Box<dyn RuleSelectionPolicy>;

    let one = diagnostics::stage0b_pooled(&f, dir.path(), &BASE_SEEDS[..1], &policy, ReadoutSet::R1)
        .expect("pooled");
    let many =
        diagnostics::stage0b_pooled(&f, dir.path(), &BASE_SEEDS, &policy, ReadoutSet::R1)
            .expect("pooled");

    assert!(
        one.draws.len() < diagnostics::MIN_SEPARATION_DRAWS,
        "the single-draw call must be recognizable as under-drawn"
    );
    assert!(
        many.draws.len() >= diagnostics::MIN_SEPARATION_DRAWS,
        "the pooled call must clear the floor: {}",
        many.detail()
    );
    // And the distribution travels with the verdict in both cases, so a
    // reader can always see what the class was computed from.
    assert!(many.detail().contains("draws ["), "{}", many.detail());
    assert!(
        many.detail().contains("hard-bound violations"),
        "the hard bound is reported whether or not it fired: {}",
        many.detail()
    );
}

/// D4 §2.5 clause 1, pinned: the reference is an average over disjoint
/// pairs, not a single sample.
///
/// This is the clause that fixes the defect, so it is asserted rather than
/// left to the report. A reference built from one pair is what produced
/// `binary_edge`'s ratio of 15.818.
#[test]
#[ignore = "E3: the reference is pooled; run explicitly"]
fn the_reference_is_an_average_over_disjoint_pairs() {
    let f = battery()
        .into_iter()
        .find(|f| f.name == "growth_treadmill")
        .expect("fixture");
    let dir = tempfile::tempdir().expect("tempdir");
    let policy =
        |s: u64| Box::new(RandomSubset::new(Some(STAGE0B_BUDGET), s)) as Box<dyn RuleSelectionPolicy>;
    let p = diagnostics::stage0b_pooled(&f, dir.path(), &BASE_SEEDS[..2], &policy, ReadoutSet::R1)
        .expect("pooled");
    for d in &p.draws {
        assert_eq!(
            d.reference_samples.len(),
            diagnostics::REFERENCE_PAIRS,
            "each draw's reference must average {} pairs, got {:?}",
            diagnostics::REFERENCE_PAIRS,
            d.reference_samples
        );
        let mean = d.reference_samples.iter().sum::<f64>() / d.reference_samples.len() as f64;
        assert!(
            (mean - d.reference).abs() < 1e-12,
            "the reported reference must BE that average: {} vs {}",
            d.reference,
            mean
        );
    }
}

// =====================================================================
// Stage 5' — the pre-registered emergence prediction
// =====================================================================

/// Plan §9.2 / §7.2.
///
/// Three arms at **matched budget** — random, untrained, champion — with
/// the `ei_micro` / `ce_structural` / `ei_macro` triple reported for each.
/// Budget being fixed across arms is what makes the prediction derivable
/// rather than direction-guessed: C0–C5's Stage 5 confounded the
/// intervention with the variable that moves both terms.
///
/// **All three outcomes are exits.** As predicted → the F5/emergence
/// tension is real and F6 becomes live. `ce_structural` rises instead →
/// the micro-slop mechanism is incomplete and C-4 reopens. Both flat →
/// F5's reuse pressure is too weak at this budget to move either term.
///
/// E4 §5 supplies a *second* pre-registered expectation, recorded before
/// this ran: on all three authored substrates the champion carries less
/// persistent structure than random while running the smallest graph, so if
/// §7.2's mechanism is right the same substrates should show the
/// `ei_micro` up / `ce_structural` down signature most clearly.
#[test]
#[ignore = "E3 Stage 5': the emergence triple across arms; hours, run explicitly"]
fn stage5_prime_emergence_triple() {
    println!("\n=== E3 — Stage 5′, the emergence prediction ===");
    println!("PRE-REGISTERED (plan §7.2): champion has HIGHER ei_micro and");
    println!("LOWER ce_structural than budget-matched random; ei_macro ambiguous.");
    println!("PRE-REGISTERED (E4 §5): the effect should be clearest on the three");
    println!("authored substrates, where the champion already carries the least");
    println!("persistent structure and the smallest graph.\n");

    for (name, rules, ic, n_rules) in stage5_cells() {
        let f = fx(name, RuleSource::File(rules), ic, 24, STEPS, WINDOW);
        let dir = tempfile::tempdir().expect("tempdir");
        let cfg = f.sim_config_causal(dir.path(), 9_600_000);

        let mut t = hg_train::config::TrainingConfig::default();
        t.fitness = hg_train::fitness::FitnessFn::F5ConstructorReuse;
        t.horizon_steps = STEPS;
        t.burn_in_windows = 5;
        t.holdout_seeds = HOLDOUT.to_vec();
        t.population_pairs = 16;
        t.seeds_per_member = 2;
        t.generations = 30;
        t.sigma = 0.10;
        t.alpha = 0.032;
        t.campaign_seed = 9_600_000;

        let mut cog = CognitionConfig {
            policy: PolicyKind::MlpScorer,
            budget: Some(STAGE0B_BUDGET),
            ..Default::default()
        };
        cog.mlp.hidden = 16;

        println!("\n## {name} ({ic})");
        let spec = hg_train::campaign::CampaignSpec {
            campaign_id: format!("e3_{name}"),
            base_dir: dir.path().to_path_buf(),
            sim_config: &cfg,
            cognition: cog.clone(),
            training: t.clone(),
        };
        // The champion genome comes back in-process on `CampaignOutcome`, so
        // Stage 5′ measures the arms of a campaign it ran rather than
        // reloading artifacts. Re-deriving a champion from a fresh campaign
        // would be a different policy and the comparison would not be the
        // one plan §9.2 asks for.
        let champion = match hg_train::campaign::run_campaign(&spec) {
            Ok(o) => {
                println!(
                    "   campaign champion {:.4} vs untrained {:.4} vs random {:.4}",
                    o.arm_table.champion.mean(),
                    o.arm_table.untrained.mean(),
                    o.arm_table.random.mean()
                );
                o.champion.params.clone()
            }
            Err(e) => {
                println!("   campaign refused: {e}");
                continue;
            }
        };

        let n = n_params_for(&cog, n_rules);
        let untrained =
            hg_train::prechecks::class_init(PolicyKind::MlpScorer, n, 9_600_000u64 ^ 0x1_c001_1u64);

        println!(
            "   {:<12} {:>12} {:>14} {:>12} {:>10}",
            "arm", "ei_micro", "ce_structural", "ei_macro", "intervals"
        );
        let mut rows: Vec<(&str, [Vec<f64>; 3])> = Vec::new();
        for (label, genome) in [
            ("random", None),
            ("untrained", Some(untrained.clone())),
            ("champion", Some(champion.clone())),
        ] {
            // PER SEED, not per interval. The ~42 emission windows inside one
            // rollout are correlated — they are one trajectory — so pooling
            // 5x42 into a single mean and quoting it without a spread would
            // treat 210 dependent observations as 210 independent ones. The
            // independent unit is the hold-out seed, and n = 5.
            let mut per_seed: [Vec<f64>; 3] = [Vec::new(), Vec::new(), Vec::new()];
            for &s in &HOLDOUT {
                let policy: Box<dyn RuleSelectionPolicy> = match &genome {
                    None => Box::new(RandomSubset::new(Some(STAGE0B_BUDGET), s)),
                    Some(g) => {
                        let mut p = MlpScorer::with_features_and_norm(
                            n_rules,
                            cog.wl_buckets,
                            16,
                            Some(STAGE0B_BUDGET),
                            s,
                            FeatureSet::V1,
                            ScoreNorm::PoolStandard,
                        );
                        p.set_params(g);
                        Box::new(p)
                    }
                };
                let Ok(m) = hg_sim_core::run_rollout(hg_sim_core::RolloutRequest {
                    config: &cfg,
                    seed: s,
                    policy,
                    horizon_steps: STEPS,
                    probe: None,
                }) else {
                    continue;
                };
                // Only windows where the causal layer actually emitted.
                // Averaging over `None` as zero is the exact failure
                // `instrument_registry.md` §2 exists to prevent.
                let (mut mi, mut cs, mut ma, mut k) = (0.0f64, 0.0f64, 0.0f64, 0usize);
                for r in &m.windows {
                    if let (Some(a), Some(b), Some(c)) = (r.ei_micro, r.ce_structural, r.ei_macro) {
                        mi += a;
                        cs += b;
                        ma += c;
                        k += 1;
                    }
                }
                if k > 0 {
                    per_seed[0].push(mi / k as f64);
                    per_seed[1].push(cs / k as f64);
                    per_seed[2].push(ma / k as f64);
                }
            }
            if per_seed[0].is_empty() {
                println!("   {label:<12}  unmeasured — the causal layer emitted no interval");
                continue;
            }
            let stat = |v: &Vec<f64>| {
                let m = v.iter().sum::<f64>() / v.len() as f64;
                let sd = if v.len() < 2 {
                    0.0
                } else {
                    (v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (v.len() - 1) as f64).sqrt()
                };
                (m, sd)
            };
            let (a, asd) = stat(&per_seed[0]);
            let (b, bsd) = stat(&per_seed[1]);
            let (c, csd) = stat(&per_seed[2]);
            println!(
                "   {:<12} {:>8.4}+-{:<6.4} {:>8.4}+-{:<6.4} {:>8.4}+-{:<6.4}  n={}",
                label,
                a,
                asd,
                b,
                bsd,
                c,
                csd,
                per_seed[0].len()
            );
            rows.push((label, per_seed));
        }

        // The adjudication, in units of the pooled seed sd. A delta whose
        // magnitude is under 2 pooled sd is not a direction, and the
        // prediction is about directions.
        let get = |w: &str| rows.iter().find(|(l, _)| *l == w).map(|(_, v)| v);
        if let (Some(r), Some(ch)) = (get("random"), get("champion")) {
            let cmp = |i: usize| -> (f64, f64) {
                let mn = |v: &Vec<f64>| v.iter().sum::<f64>() / v.len() as f64;
                let sd = |v: &Vec<f64>| {
                    let m = mn(v);
                    if v.len() < 2 {
                        0.0
                    } else {
                        (v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (v.len() - 1) as f64)
                            .sqrt()
                    }
                };
                let d = mn(&ch[i]) - mn(&r[i]);
                let pooled = ((sd(&ch[i]).powi(2) + sd(&r[i]).powi(2)) / 2.0).sqrt();
                (d, d / pooled.max(1e-12))
            };
            let (dm, zm) = cmp(0);
            let (dc, zc) = cmp(1);
            let (da, za) = cmp(2);
            println!(
                "   champion - random:  ei_micro {dm:+.5} ({zm:+.2}sd)   \
                 ce_structural {dc:+.5} ({zc:+.2}sd)   ei_macro {da:+.5} ({za:+.2}sd)"
            );
            let micro_up = zm > 2.0;
            let micro_down = zm < -2.0;
            let ce_down = zc < -2.0;
            let ce_up = zc > 2.0;
            println!(
                "   => prediction {}",
                match (micro_up, ce_down) {
                    (true, true) => "HELD (ei_micro up, ce_structural down)".to_string(),
                    _ => format!(
                        "NOT HELD — ei_micro {}, ce_structural {}",
                        if micro_up {
                            "up"
                        } else if micro_down {
                            "DOWN (opposite)"
                        } else {
                            "flat"
                        },
                        if ce_down {
                            "down"
                        } else if ce_up {
                            "UP (opposite)"
                        } else {
                            "flat"
                        }
                    ),
                }
            );
        }
    }
}

// =====================================================================
// Stage 0b" — is there a perturbation at all?
// =====================================================================

/// The pooled statistic did not remove the hard-bound violations
/// (`triangle_subdivide` r1 goes from 5/8 under D4's single-pair reference
/// to **8/8** here, minimum ratio 1.003). D4 §2.5's fix is implemented
/// faithfully and does not work, which puts its §2.3 diagnosis — "the fault
/// is in the normalizer" — in question rather than the implementation.
///
/// # The hypothesis this test decides
///
/// Stage 0b's numerator is the JSD between runs at `seed` and `seed + 1`,
/// called "the perturbed pair"; its denominator is the JSD between two
/// far-apart seeds, called "fully diverged". But the seed reaches the run
/// only through the policy RNG, and `ChaCha8Rng` seeded at 4242 and 4243
/// produces **completely uncorrelated** streams. So `seed + 1` is not a
/// small perturbation of a trajectory — it is an independent draw, and the
/// numerator and the denominator are estimating *the same quantity*.
///
/// If that is right, the ratio's expected value is ~1, roughly half of all
/// draws exceed the hard bound by chance, and the classifier's thresholds
/// at 0.05 and 0.8 are cutting noise. It also predicts every symptom D4
/// §2.2 recorded, including `binary_edge`'s ratio of 15.818 when the
/// denominator happens to land near zero.
///
/// **The test.** Draw many `(s, s+1)` pairs and many far-apart pairs from
/// the same substrate and compare the two *distributions*. Under the
/// hypothesis they are the same distribution; under Stage 0b's premise the
/// adjacent-seed pairs are systematically smaller.
///
/// Reported as a mean ratio and a Mann–Whitney-style dominance count,
/// because with skewed bounded quantities a difference in means is easier
/// to misread than the fraction of cross-pair comparisons that go one way.
#[test]
#[ignore = "E3: does Stage 0b have a perturbation at all; run explicitly"]
fn adjacent_seeds_are_not_a_perturbation() {
    println!("\n=== E3 — is `seed + 1` a perturbation? ===");
    println!("H0: JSD(s, s+1) and JSD(far, far') are the SAME distribution,");
    println!("    i.e. Stage 0b's numerator and denominator measure one thing.\n");

    for f in battery().into_iter().filter(|f| {
        matches!(f.name, "triangle_subdivide" | "growth_treadmill" | "fl1_two_rule")
    }) {
        let dir = tempfile::tempdir().expect("tempdir");
        let policy = |s: u64| {
            Box::new(RandomSubset::new(Some(STAGE0B_BUDGET), s)) as Box<dyn RuleSelectionPolicy>
        };
        let probe_of = |seed: u64| -> Option<Vec<Vec<f32>>> {
            let cfg = f.sim_config(dir.path(), seed);
            hg_sim_core::run_rollout(hg_sim_core::RolloutRequest {
                config: &cfg,
                seed,
                policy: policy(seed),
                horizon_steps: f.steps,
                probe: Some(Box::new(hg_cognition::readout::probe(
                    ReadoutSet::R1,
                    diagnostics::SEPARATION_BUCKETS,
                    diagnostics::R2_TOP_N_WL,
                    diagnostics::R2_LABEL_SLOTS,
                ))),
            })
            .ok()
            .map(|m| m.probe_rows)
        };
        let tail = |x: &[Vec<f32>], y: &[Vec<f32>]| -> Option<f64> {
            let n = x.len().min(y.len());
            if n < 4 {
                return None;
            }
            let t = n - n / 3;
            Some(
                (t..n).map(|i| diagnostics::jsd_pub(&x[i], &y[i])).sum::<f64>()
                    / (n - t) as f64,
            )
        };

        // 12 adjacent pairs (s, s+1) and 12 far pairs, disjoint seeds
        // throughout so no run is reused between the two groups.
        let mut adjacent = Vec::new();
        for k in 0..12u64 {
            let s = 50_000 + 2 * k;
            if let (Some(a), Some(b)) = (probe_of(s), probe_of(s + 1)) {
                if let Some(v) = tail(&a, &b) {
                    adjacent.push(v);
                }
            }
        }
        let mut far = Vec::new();
        for k in 0..12u64 {
            let s1 = 80_000 + 6007 * k;
            let s2 = s1 + 3001;
            if let (Some(a), Some(b)) = (probe_of(s1), probe_of(s2)) {
                if let Some(v) = tail(&a, &b) {
                    far.push(v);
                }
            }
        }
        if adjacent.is_empty() || far.is_empty() {
            println!("## {}  — no measurable pairs", f.name);
            continue;
        }
        let mean = |v: &Vec<f64>| v.iter().sum::<f64>() / v.len() as f64;
        // Fraction of all (adjacent, far) comparisons where adjacent < far.
        // 0.5 is "same distribution"; Stage 0b's premise needs this near 1.
        let mut wins = 0usize;
        for a in &adjacent {
            for b in &far {
                if a < b {
                    wins += 1;
                }
            }
        }
        let (na, nf) = (adjacent.len() as f64, far.len() as f64);
        let dominance = wins as f64 / (na * nf);
        // The 144 comparisons come from 12 + 12 independent runs, so the
        // effective n is 12 per group and not 144. Mann-Whitney's null SE
        // for U is sqrt(n*m*(n+m+1)/12); divided by n*m it is the SE of the
        // dominance itself. Without this, 0.67 reads as a finding and it is
        // barely one standard error from a coin flip.
        let se = ((na * nf * (na + nf + 1.0)) / 12.0).sqrt() / (na * nf);
        let z = (dominance - 0.5) / se;
        println!("## {}", f.name);
        println!(
            "   adjacent (s, s+1): {:.5} over {} pairs   far: {:.5} over {} pairs",
            mean(&adjacent),
            adjacent.len(),
            mean(&far),
            far.len()
        );
        println!(
            "   mean ratio adjacent/far = {:.3};  P(adjacent < far) = {:.2} +- {:.2} \
             ({:+.2} sd from 0.5)",
            mean(&adjacent) / mean(&far).max(1e-12),
            dominance,
            se,
            z
        );
        println!(
            "   => {}",
            if dominance - 2.0 * se > 0.75 {
                "a real perturbation — adjacent pairs stay closer"
            } else if dominance + 2.0 * se < 0.25 {
                "adjacent pairs diverge MORE — inverted, and unexplained"
            } else if z.abs() < 2.0 {
                "NO PERTURBATION — indistinguishable from one distribution"
            } else {
                "a WEAK effect: separable from 0.5, but far short of a perturbation"
            }
        );
    }
}
