//! Stage 2 — is the `StaticBias` fitness landscape flat?
//!
//! The Stage 2 campaigns raised a question their own exit criterion cannot
//! answer. On `wn_three` the champion beat budget-matched random by 0.7636
//! at 3.11σ with 5/5 seeds and no Goodhart flags — the plan's exit as
//! written, met. But the champion's allocation was `[0.313, 0.336, 0.351]`,
//! i.e. uniform, and an **untrained** `StaticBias` at θ = 0 scored exactly
//! the same 1.0000 on all five hold-out seeds. CMA-ES contributed nothing;
//! the margin was the difference between two policy *classes*
//! (`StaticBias` allocates per rule group, `RandomSubset` samples per
//! binding, so they diverge whenever one rule dominates the pool) and it
//! was there before any training.
//!
//! Two explanations survive that, and they call for different follow-ups:
//!
//! 1. **The landscape is flat.** Every genome scores the same, so there is
//!    nothing for any optimizer to find and the negative is about the
//!    substrate, not about CMA-ES.
//! 2. **The landscape has structure CMA-ES missed** — bad step size, too
//!    few generations, a deceptive basin.
//!
//! A direct sweep separates them and costs a small fraction of a campaign:
//! score a spread of genomes, including the one-hot extremes that commit
//! the entire budget to a single rule, and look at the range. If the
//! extremes score identically to θ = 0 then (1) holds and no campaign
//! tuning can change it.
//!
//! This also re-measures the ceiling. `tri_two`'s campaign returned
//! champion **and** random both at exactly 1.0000 — no headroom at all, σ
//! undefined. Its budget was chosen for headroom measured at a 192-step
//! horizon and the campaign then ran at 384, where random is already at the
//! ceiling. Choosing a budget at one horizon and running at another is the
//! methodological error; the sweep below runs at the campaign's horizon.

use hg_cognition::{derive_policy_seed, Parameterized, RandomSubset, StaticBias};
use hg_observe::RuleSelectionPolicy;
use hg_sim_core::{run_rollout, RolloutRequest};
use hg_train::diagnostics::{Fixture, RuleSource};

/// The frozen hold-out seeds, so these numbers are directly comparable to
/// the campaign's own champion scoring.
const HOLDOUT: [u64; 5] = [900_001, 900_002, 900_003, 900_004, 900_005];
const BURN_IN: usize = 5;
const CAMPAIGN_SEED: u64 = 20260726;
/// The campaign's final generation index (`generations - 1`), which is the
/// generation its last hold-out rollouts were scored in.
const LAST_GEN: u32 = 39;

/// CMA-ES population size for `n` parameters — must match
/// `optim::CmaEs::default_lambda(n).max(4)`, which is what the campaign
/// uses as its hold-out `member_idx`.
fn lambda_for(n_rules: usize) -> usize {
    hg_train::optim::CmaEs::default_lambda(n_rules).max(4)
}

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

fn mean(v: &[f64]) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    v.iter().sum::<f64>() / v.len() as f64
}

/// Score one policy factory across the five frozen hold-out seeds.
fn score(
    f: &Fixture,
    n_rules: usize,
    make: &(dyn Fn(u64) -> Box<dyn RuleSelectionPolicy> + Sync),
) -> Vec<f64> {
    let dir = tempfile::tempdir().expect("tempdir");
    // The campaign scores its hold-outs at member_idx = <population size>,
    // which is CMA-ES's lambda and therefore depends on the PARAMETER COUNT
    // — 7 for a 3-rule set, 6 for a 2-rule set.
    //
    // This was hardcoded to 7. That silently put `tri_two`'s rows on a
    // different policy-RNG stream from its campaign's, and the two were
    // then compared: the sweep read theta=0 at 0.9455 where the campaign
    // read 1.0000, which looked like +0.055 of headroom the optimizer had
    // missed. It was not headroom, it was two different draws. Deriving
    // lambda from the rule count is the fix.
    let n_members = lambda_for(n_rules) as u32;
    let jobs: Vec<(usize, u64)> = HOLDOUT.iter().copied().enumerate().collect();
    let one = |(r, sim_seed): (usize, u64)| -> Option<f64> {
        let cfg = f.sim_config(dir.path(), sim_seed);
        // Same derivation the campaign uses for its hold-out rollouts, so
        // every arm here is paired on the policy RNG.
        let policy_seed = derive_policy_seed(CAMPAIGN_SEED, LAST_GEN, n_members, r as u32, 0);
        let m = run_rollout(RolloutRequest {
            config: &cfg,
            seed: sim_seed,
            policy: make(policy_seed),
            horizon_steps: f.steps,
            probe: None,
        })
        .ok()?;
        hg_train::fitness::evaluate(&m, BURN_IN).ok().map(|fit| fit.f2)
    };

    // Scoped OS threads, not rayon: the runner installs its step loop into
    // its own rayon pool and nesting the two serializes completely.
    let mut out = Vec::new();
    std::thread::scope(|scope| {
        let handles: Vec<_> = jobs
            .chunks(1)
            .map(|b| scope.spawn(move || b.iter().copied().filter_map(one).collect::<Vec<_>>()))
            .collect();
        for h in handles {
            out.extend(h.join().expect("worker"));
        }
    });
    out
}

/// The genomes to sweep. Includes the one-hot extremes: a genome that
/// commits essentially the whole budget to one rule is the strongest
/// statement `StaticBias` can make, and if that scores like θ = 0 then rule
/// identity has no purchase at this cell whatsoever.
fn genomes(n_rules: usize) -> Vec<(String, Vec<f32>)> {
    let mut out = vec![("uniform (theta=0)".to_string(), vec![0.0; n_rules])];
    for r in 0..n_rules {
        let mut g = vec![-6.0f32; n_rules];
        g[r] = 6.0;
        out.push((format!("all-in on rule {r}"), g));
        let mut g = vec![0.0f32; n_rules];
        g[r] = -6.0;
        out.push((format!("suppress rule {r}"), g));
    }
    out
}

#[test]
#[ignore = "long-running; run explicitly for the Stage 2 landscape table"]
fn stage2_genome_landscape() {
    println!("\n=== Stage 2 landscape — does any StaticBias genome matter? ===");
    println!("(five frozen hold-out seeds, campaign horizon, F2)\n");

    for (make_f, budgets) in [
        (wn_three as fn(u64, u64) -> Fixture, [4usize, 16, 64]),
        (tri_two as fn(u64, u64) -> Fixture, [4, 16, 64]),
    ] {
        for k in budgets {
            let f = make_f(384, 24);
            let cfg = f.sim_config(std::path::Path::new("."), 1);
            let n_rules = hg_sim_core::load_rules(&cfg).expect("rules").len();

            let rnd = score(&f, n_rules, &|seed| Box::new(RandomSubset::new(Some(k), seed)));
            println!(
                "## {} k={k}  ({n_rules} rules, {} steps)",
                f.name, f.steps
            );
            println!(
                "{:<24} {:>9}   {:?}",
                "budget-matched random",
                mean(&rnd),
                rnd.iter().map(|x| (x * 1000.0).round() / 1000.0).collect::<Vec<_>>()
            );

            let mut all = Vec::new();
            for (label, theta) in genomes(n_rules) {
                let t = theta.clone();
                let s = score(&f, n_rules, &|seed| {
                    let mut p = StaticBias::new(n_rules, Some(k), seed);
                    p.set_params(&t);
                    Box::new(p)
                });
                println!(
                    "{:<24} {:>9.4}   {:?}",
                    label,
                    mean(&s),
                    s.iter().map(|x| (x * 1000.0).round() / 1000.0).collect::<Vec<_>>()
                );
                all.push(mean(&s));
            }

            let lo = all.iter().cloned().fold(f64::INFINITY, f64::min);
            let hi = all.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            println!(
                "  genome spread: {:.4} .. {:.4}  (range {:.4}){}\n",
                lo,
                hi,
                hi - lo,
                if hi - lo < 1e-9 {
                    "   <== FLAT: no genome can beat any other"
                } else {
                    ""
                }
            );
        }
    }
}
