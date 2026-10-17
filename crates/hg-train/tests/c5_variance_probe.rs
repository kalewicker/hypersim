//! C5 gate — where does hold-out variance actually come from?
//!
//! `docs/cognition_stage0_findings.md` §0.4 recorded that four of the five
//! Stage 0a fixtures return bit-identical results on every simulation seed,
//! and §5.3 concluded from that that Stage 2 champion comparisons cannot use
//! the plan's 2σ criterion. That conclusion is only correct if the *campaign*
//! has no other source of variance, and it has one: a campaign derives a
//! fresh `policy_rng_seed` per rollout, while Stage 0a's arms did not — its
//! `random` arm pins `RandomSubset`'s seed to 1 and both `degree_*` arms are
//! deterministic by construction.
//!
//! So there are three candidate variance sources and Stage 0a could only
//! ever see one of them. This test separates them by holding each fixed in
//! turn, before any campaign is sized around the answer.

use hg_cognition::{derive_policy_seed, Parameterized, StaticBias};
use hg_train::diagnostics::{Fixture, RuleSource};
use hg_sim_core::{run_rollout, RolloutRequest};

const N: usize = 8;
const BUDGET: usize = 4;

fn probe_fixtures() -> Vec<Fixture> {
    vec![
        // Deterministic IC (GliderRing is constructed, not sampled).
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
        // Seed-sampled IC — the one fixture Stage 0b called Viable.
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
    ]
}

struct Arm {
    label: &'static str,
    /// Held constant across the N rollouts, so a zero sd here is evidence
    /// that the *other* source carries no information.
    vary_sim: bool,
    vary_policy: bool,
}

const ARMS: [Arm; 3] = [
    Arm { label: "both vary", vary_sim: true, vary_policy: true },
    Arm { label: "sim seed only", vary_sim: true, vary_policy: false },
    Arm { label: "policy seed only", vary_sim: false, vary_policy: true },
];

fn stats(v: &[f64]) -> (f64, f64) {
    let m = v.iter().sum::<f64>() / v.len() as f64;
    let sd = (v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (v.len() - 1) as f64).sqrt();
    (m, sd)
}

/// Same relative floor the Stage 0a classifier uses: with N identical
/// samples the sample sd lands near 1e-16 rather than exactly zero, because
/// the mean is not bit-identical to the samples.
fn has_spread(sd: f64, mean: f64) -> bool {
    sd > 1e-9 * (mean.abs() + 1.0)
}

#[test]
fn variance_sources_are_separated() {
    let dir = tempfile::tempdir().expect("tempdir");
    println!("\n=== C5 probe — variance decomposition (StaticBias, k = {BUDGET}, {N} rollouts) ===");
    println!(
        "{:<20} {:<18} {:>9} {:>9} {:>10} {:>9}",
        "fixture", "arm", "F2", "F2 sd", "interest", "sd"
    );

    let mut any_policy_variance = false;

    for f in probe_fixtures() {
        let cfg0 = f.sim_config(dir.path(), 1);
        let n_rules = hg_sim_core::load_rules(&cfg0).expect("rules load").len();
        // A fixed, arbitrary but non-uniform genome: a uniform one would
        // make StaticBias's ranking degenerate and hide policy-RNG effects
        // behind a tie.
        let theta: Vec<f32> = (0..n_rules)
            .map(|i| (i as f32 * 0.7).sin())
            .collect();

        for arm in &ARMS {
            let mut f2 = Vec::with_capacity(N);
            let mut interest = Vec::with_capacity(N);
            for i in 0..N {
                let sim_seed = if arm.vary_sim { 7_000_000 + i as u64 * 101 } else { 7_000_000 };
                let policy_seed = if arm.vary_policy {
                    derive_policy_seed(4242, 0, 0, i as u32, 0)
                } else {
                    derive_policy_seed(4242, 0, 0, 0, 0)
                };
                let cfg = f.sim_config(dir.path(), sim_seed);
                let mut p = StaticBias::new(n_rules, Some(BUDGET), policy_seed);
                p.set_params(&theta);
                let m = run_rollout(RolloutRequest {
                    config: &cfg,
                    seed: sim_seed,
                    policy: Box::new(p),
                    horizon_steps: f.steps,
                    probe: None,
                })
                .expect("rollout");
                let fit = hg_train::fitness::evaluate(&m, 1).expect("scored windows");
                f2.push(fit.f2);
                interest.push(fit.f1_mean_interestingness);
            }
            let (f2_m, f2_sd) = stats(&f2);
            let (i_m, i_sd) = stats(&interest);
            println!(
                "{:<20} {:<18} {:>9.4} {:>9.5} {:>10.4} {:>9.5}",
                f.name, arm.label, f2_m, f2_sd, i_m, i_sd
            );
            if arm.label == "policy seed only" && (has_spread(f2_sd, f2_m) || has_spread(i_sd, i_m))
            {
                any_policy_variance = true;
            }
        }
    }

    println!(
        "\npolicy-RNG variance present on at least one fixture: {}",
        any_policy_variance
    );
    // No assertion on the science: either answer decides how Stage 2 is
    // scored, and both are legitimate. The test fails only if a rollout or
    // the fitness evaluation breaks.
}
