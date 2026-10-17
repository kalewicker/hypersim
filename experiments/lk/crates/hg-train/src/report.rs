//! The campaign report (`docs/campaign_output.md` §4).
//!
//! Eight sections, in order, the first four mirroring the run report's
//! structure so a reader who knows one knows the other. Section 5,
//! **Suspicious Metric Flags**, is mandatory *and mandatory when empty* —
//! "No flags fired." is a finding, and a section that vanishes when clean
//! trains readers to stop looking for it.

use crate::campaign::{ArmScores, ArmTable, CampaignOutcome, CampaignSpec, TrainError};
use crate::fitness::FITNESS_FORMULAS_VERSION;
use std::fmt::Write as _;
use std::path::Path;

pub fn write_campaign_report(
    spec: &CampaignSpec<'_>,
    out: &CampaignOutcome,
    path: &Path,
) -> Result<(), TrainError> {
    let mut s = String::new();
    let t = &spec.training;
    let c = &spec.cognition;

    writeln!(s, "# Campaign report — `{}`", out.campaign_id).ok();
    writeln!(s).ok();

    // 1 -----------------------------------------------------------------
    writeln!(s, "## 1. Campaign Parameters").ok();
    writeln!(s).ok();
    writeln!(s, "| Parameter | Value |").ok();
    writeln!(s, "|---|---|").ok();
    let mut p = |k: &str, v: String| {
        writeln!(s, "| `{k}` | {v} |").ok();
    };
    p("algo", format!("{:?}", t.algo));
    p("population_pairs", t.population_pairs.to_string());
    p("population_lambda", format!("{:?}", t.population_lambda));
    p("sigma", t.sigma.to_string());
    p("alpha", t.alpha.to_string());
    p("generations", t.generations.to_string());
    p("seeds_per_member", t.seeds_per_member.to_string());
    p("seed_rotation", format!("{:?}", t.seed_rotation));
    p("fitness", t.fitness.as_str().to_string());
    p("horizon_steps", t.horizon_steps.to_string());
    p("burn_in_windows", t.burn_in_windows.to_string());
    p("campaign_seed", t.campaign_seed.to_string());
    p("holdout_seeds", format!("{:?}", t.holdout_seeds));
    p("policy", format!("{:?}", c.policy));
    p("budget", format!("{:?}", c.budget));
    p("wl_buckets", c.wl_buckets.to_string());
    p("feature_set", c.feature_set.clone());
    p("fitness_formulas", FITNESS_FORMULAS_VERSION.to_string());
    p("rules", format!("{:?}", spec.sim_config.rules.source));
    p("config_hash", spec.sim_config.config_hash());
    p("n_rules", out.champion.n_rules.to_string());
    writeln!(s).ok();

    // 2 -----------------------------------------------------------------
    writeln!(s, "## 2. Fitness Trajectory").ok();
    writeln!(s).ok();
    writeln!(
        s,
        "Rotation and hold-out columns side by side: the gap between them *is* the \
         `SEED_OVERFIT` signal, and it should be readable without arithmetic."
    )
    .ok();
    writeln!(s).ok();
    writeln!(
        s,
        "| gen | best (rotation) | mean (rotation) | champion hold-out | hold-out sd |"
    )
    .ok();
    writeln!(s, "|---|---|---|---|---|").ok();
    for g in &out.generations {
        writeln!(
            s,
            "| {} | {:.4} | {:.4} | {:.4} | {:.4} |",
            g.generation,
            g.best_fitness,
            g.champion_rotation_mean,
            g.champion_holdout_mean,
            g.champion_holdout_sd
        )
        .ok();
    }
    writeln!(s).ok();

    // 3 -----------------------------------------------------------------
    writeln!(s, "## 3. Arm Table").ok();
    writeln!(s).ok();
    write_arm_table(&mut s, out, t.holdout_seeds.len());

    // 4 -----------------------------------------------------------------
    writeln!(s, "## 4. Held-out Metric Trajectories").ok();
    writeln!(s).ok();
    writeln!(
        s,
        "Optimized by nothing. This is where structure-vs-fitness divergence becomes \
         visible before the flag fires."
    )
    .ok();
    writeln!(s).ok();
    writeln!(
        s,
        "| gen | type entropy | type JSD | active persistent | static persistent | \
         archive rate | mean n_nodes | kernel_held |"
    )
    .ok();
    writeln!(s, "|---|---|---|---|---|---|---|---|").ok();
    let f = |v: Option<f64>| v.map(|x| format!("{x:.4}")).unwrap_or_else(|| "—".into());
    for g in &out.generations {
        let h = &g.champion_held_out;
        writeln!(
            s,
            "| {} | {} | {} | {} | {} | {} | {:.1} | {} |",
            g.generation,
            f(h.mean_type_entropy),
            f(h.mean_type_jsd),
            f(h.mean_active_persistent),
            f(h.mean_static_persistent),
            f(h.archive_admission_rate),
            h.mean_n_nodes,
            f(h.kernel_held)
        )
        .ok();
    }
    writeln!(s).ok();

    // 5 -----------------------------------------------------------------
    writeln!(s, "## 5. Suspicious Metric Flags").ok();
    writeln!(s).ok();
    let degenerate = out
        .failure_census
        .iter()
        .any(|(k, _)| k == "degenerate_occupancy");
    let any = out.flags.fitness_structure_divergence || out.flags.seed_overfit || degenerate;
    if !any {
        writeln!(s, "No flags fired.").ok();
    }
    if out.flags.fitness_structure_divergence {
        writeln!(s, "### `FITNESS_STRUCTURE_DIVERGENCE`").ok();
        writeln!(s).ok();
        writeln!(
            s,
            "Champion fitness rose more than 25% over the last 10 generations while every \
             held-out structure metric was flat or degrading. The policy is finding \
             something the fitness function rewards and the structure metrics do not see."
        )
        .ok();
        writeln!(s).ok();
    }
    if out.flags.seed_overfit {
        writeln!(s, "### `SEED_OVERFIT`").ok();
        writeln!(s).ok();
        writeln!(
            s,
            "Champion fitness on the rotation seeds exceeded its hold-out fitness by more \
             than 2 hold-out standard deviations. Champion claims should be made on the \
             hold-out column only."
        )
        .ok();
        writeln!(s).ok();
    }
    // The probe is optional, and a column of blanks must never be left to
    // inference (`docs/fitness_formulas.md` §5.1).
    if out
        .generations
        .iter()
        .all(|g| g.champion_held_out.kernel_held.is_none())
    {
        writeln!(
            s,
            "**No probe was configured for this campaign**, so `kernel_held` is empty for \
             every rollout. That is an absent measurement, not a zero."
        )
        .ok();
        writeln!(s).ok();
    }
    writeln!(
        s,
        "_A fired flag is a finding. These experiments deliberately optimize against the \
         evaluator; measuring how the evaluator gets gamed is the point, not a side \
         effect. Retuning a fitness constant because a flag fired — without saying so — \
         is the one move `docs/fitness_formulas.md` §6.4 forbids outright._"
    )
    .ok();
    writeln!(s).ok();

    // 6 -----------------------------------------------------------------
    writeln!(s, "## 6. Failure Census").ok();
    writeln!(s).ok();
    writeln!(s, "| status | rollouts |").ok();
    writeln!(s, "|---|---|").ok();
    for (k, n) in &out.failure_census {
        writeln!(s, "| `{k}` | {n} |").ok();
    }
    writeln!(s).ok();

    // 7 -----------------------------------------------------------------
    writeln!(s, "## 7. Cost").ok();
    writeln!(s).ok();
    writeln!(s, "| Quantity | Value |").ok();
    writeln!(s, "|---|---|").ok();
    writeln!(s, "| total rollouts | {} |", out.total_rollouts).ok();
    writeln!(s, "| wallclock | {:.1} s |", out.wallclock_ms as f64 / 1000.0).ok();
    let steps = out.total_rollouts as f64 * t.horizon_steps as f64;
    let secs = (out.wallclock_ms as f64 / 1000.0).max(1e-9);
    writeln!(s, "| total steps | {steps:.0} |").ok();
    writeln!(s, "| steps/sec (all threads) | {:.0} |", steps / secs).ok();
    writeln!(s).ok();
    writeln!(
        s,
        "`campaign_hours = pairs × 2 × seeds_per_member × generations × horizon_steps / \
         steps_per_sec / cores / 3600`. The measured steps/sec above replaces the NN \
         plan §12 illustration for any campaign scheduled on this rule set."
    )
    .ok();
    writeln!(s).ok();

    // 8 -----------------------------------------------------------------
    writeln!(s, "## 8. Precheck Record").ok();
    writeln!(s).ok();
    if out.prechecks.is_empty() {
        writeln!(s, "No prechecks ran. **This campaign is not citable.**").ok();
    } else {
        writeln!(s, "| Precheck | Measured | Bar | Detail |").ok();
        writeln!(s, "|---|---|---|---|").ok();
        for p in &out.prechecks {
            let num = |x: f64| if x.is_nan() { "n/a".to_string() } else { format!("{x:.4}") };
            writeln!(
                s,
                "| {} | {} | {} | {} |",
                p.name,
                num(p.measured),
                num(p.threshold),
                p.detail
            )
            .ok();
        }
        writeln!(s).ok();
        writeln!(
            s,
            "Measured values rather than a pass/fail column: a marginal pass should be \
             visible before it becomes a marginal result \
             (`hypersim_nn_experiments_plan_v2.md` §4)."
        )
        .ok();
    }
    writeln!(s).ok();

    std::fs::write(path, s)?;
    Ok(())
}

/// The Arm Table (`docs/campaign_output.md` §4.2).
///
/// Per-seed values, then both margins with both σ figures, with
/// `champion − untrained` marked as the primary. A report missing the
/// untrained arm is malformed and says so in its own text rather than
/// leaving the reader to notice the absence.
fn write_arm_table(s: &mut String, out: &CampaignOutcome, n_holdout: usize) {
    let a = &out.arm_table;

    if !a.is_complete() {
        writeln!(
            s,
            "> **MALFORMED REPORT.** The three-arm protocol requires `random`, \
             `untrained` and `champion` at matched budget and matched seeds \
             (`docs/campaign_output.md` §4.2). This campaign produced \
             random={}, untrained={}, champion={} measurable hold-out scores. \
             **No learning claim may be made from this table.**",
            a.random.per_seed.len(),
            a.untrained.per_seed.len(),
            a.champion.per_seed.len()
        )
        .ok();
        writeln!(s).ok();
    }

    writeln!(s, "Per-seed values on the {n_holdout} frozen hold-out seeds:").ok();
    writeln!(s).ok();
    writeln!(s, "| arm | isolates | per-seed | mean ± sd | unmeasured |").ok();
    writeln!(s, "|---|---|---|---|---|").ok();
    let row = |s: &mut String, name: &str, what: &str, x: &ArmScores| {
        let per: Vec<String> = x.per_seed.iter().map(|v| format!("{v:.4}")).collect();
        writeln!(
            s,
            "| `{name}` | {what} | {} | {:.4} ± {:.4} | {} |",
            if per.is_empty() { "—".into() } else { per.join(", ") },
            x.mean(),
            x.sd(),
            x.unmeasured
        )
        .ok();
    };
    row(s, "random", "the budget effect alone", &a.random);
    row(s, "untrained", "the allocation-class effect", &a.untrained);
    row(s, "champion", "**the learning effect**", &a.champion);
    writeln!(s).ok();

    writeln!(s, "| margin | Δ | σ | status |").ok();
    writeln!(s, "|---|---|---|---|").ok();
    let m = |s: &mut String, label: &str, v: Option<(f64, f64)>, note: &str| {
        match v {
            Some((d, sig)) => writeln!(s, "| {label} | {d:+.4} | {sig:+.2}σ | {note} |").ok(),
            None => writeln!(s, "| {label} | — | — | not measurable: an arm has no score |").ok(),
        };
    };
    m(
        s,
        "**champion − untrained**",
        a.champion_vs_untrained(),
        "**PRIMARY** — the learning claim rests on this and nothing else",
    );
    m(
        s,
        "champion − random",
        a.champion_vs_random(),
        "reported alongside; never an exit criterion for a learning claim",
    );
    m(
        s,
        "untrained − random",
        a.untrained_vs_random(),
        "the allocation-class effect: a real finding, about allocation",
    );
    writeln!(s).ok();

    // The verdict is written by the report, not left to the reader, because
    // the C0-C5 failure was precisely that a large champion-random margin
    // read as learning for three stages.
    match a.champion_vs_untrained() {
        Some((_, sig)) if sig.abs() >= 2.0 => {
            writeln!(
                s,
                "**Learning effect: {sig:+.2}σ against the untrained arm.** This is the \
                 statistic a learning claim may cite."
            )
            .ok();
        }
        Some((d, sig)) => {
            writeln!(
                s,
                "**No learning effect.** `champion − untrained` = {d:+.4} ({sig:+.2}σ) is \
                 within seed noise. Whatever margin this campaign shows against \
                 budget-matched random is an **allocation-class** effect, not a learned \
                 one, and must be reported as such."
            )
            .ok();
        }
        None => {
            writeln!(
                s,
                "**No learning claim is possible from this campaign**: the primary margin \
                 could not be computed."
            )
            .ok();
        }
    }
    writeln!(s).ok();
}

/// Unused-import guard: `ArmTable`'s methods are exercised above through
/// `out.arm_table`, and naming the type here keeps the import honest.
#[allow(dead_code)]
fn _arm_table_type_is_used(_: &ArmTable) {}
