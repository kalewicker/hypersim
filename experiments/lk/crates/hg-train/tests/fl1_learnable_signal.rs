//! F-L1 — the learnable-signal fixture (NN plan §9).
//!
//! End-to-end validation of the entire loop: seam, features, fitness,
//! optimizer, reproducibility. Two rules, one right and one wrong, with the
//! correct behaviour expressible by `StaticBias` and nothing more. If
//! CMA-ES cannot solve this, the training loop is broken — not the
//! hypothesis.
//!
//! ## Calibration (measured, not assumed) — and one trap in measuring it
//!
//! `rules/fl1_learnable_signal.toml` on a 31-node `GliderRing`, 192 steps,
//! 32-step windows, burn-in 2. F2 of each arm, **measured with the policy
//! that is actually being trained**:
//!
//! | budget | StaticBias→glider | StaticBias→bloat | budget-matched random |
//! |---|---|---|---|
//! | 5 | 0.25 | 1.00 | 0.25 |
//! | 6 | 1.00 | 1.00 | 0.50 |
//! | 7 | 1.00 | 1.00 | 0.50 |
//! | **8** | **1.00** | **−0.50** | **0.25** |
//! | 16 | −0.50 | −0.50 | 0.25 |
//!
//! The learnable band is the single value `k = 8`, and it is narrow on both
//! sides for two *different* reasons:
//!
//! - **Below it** (`k ≤ 7`) the growth rule is slow enough to read as
//!   `GenerativeCandidate` rather than `Exploding`, so preferring it scores
//!   just as well and there is nothing to learn.
//! - **Above it** (`k ≥ 16`) the correct answer becomes *unreachable*. The
//!   glider group holds only ~6–7 bindings per step, and `StaticBias`
//!   redistributes budget away from exhausted rules — it always spends the
//!   full `k`. So at `k = 16` every genome, including a maximally
//!   glider-preferring one, must spend the remaining slots on bloat and
//!   scores −0.50. A campaign run at `k = 16` converges to the worst
//!   possible fitness while behaving perfectly correctly.
//!
//! **`StaticBias` cannot express "use less than the budget."** That is a
//! real expressiveness limit, not a bug — `RuleBias` *can* (a zero-weighted
//! rule is dropped and the budget is left unspent), which is exactly why
//! calibrating this fixture with `RuleBias` gave a clean 0.75 margin at
//! `k = 16` that `StaticBias` then could not collect. A fixture must be
//! calibrated with the policy class under test, or it measures an arm the
//! trainee cannot reach.
//!
//! `k = 8` is used here.

use hg_cognition::{CognitionConfig, PolicyKind, RandomSubset};
use hg_sim_core::{run_rollout, RolloutRequest, SimConfig};
use hg_train::campaign::{run_campaign, CampaignSpec};
use hg_train::config::{Algo, TrainingConfig};
use hg_train::fitness::FitnessFn;

const STEPS: u64 = 192;
const WINDOW: u64 = 32;
const BUDGET: usize = 8;
const BURN_IN: usize = 2;
const HOLDOUTS: [u64; 5] = [900_001, 900_002, 900_003, 900_004, 900_005];

fn sim_config(dir: &std::path::Path) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "fl1"
seed = 4242
out_dir = "{}"
[rules]
source = "File"
file_path = "{}/../../rules/fl1_learnable_signal.toml"
[initial]
condition_type = "GliderRing"
node_count = 31
[engine]
max_steps = {STEPS}
metrics_window_interval = {WINDOW}
thread_count = 1
[evaluator]
enabled = true
[causal]
enabled = false
[output]
fixed_provenance = true
"#,
        dir.display(),
        env!("CARGO_MANIFEST_DIR")
    );
    toml::from_str(&toml).expect("config parses")
}

fn cognition() -> CognitionConfig {
    CognitionConfig {
        policy: PolicyKind::StaticBias,
        budget: Some(BUDGET),
        ..Default::default()
    }
}

fn mean_sd(v: &[f64]) -> (f64, f64) {
    let m = v.iter().sum::<f64>() / v.len() as f64;
    let sd = if v.len() < 2 {
        0.0
    } else {
        (v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (v.len() - 1) as f64).sqrt()
    };
    (m, sd)
}

/// Budget-matched `RandomSubset` on the frozen hold-out seeds — the only
/// baseline a champion may be compared against.
fn random_baseline(cfg: &SimConfig) -> Vec<f64> {
    HOLDOUTS
        .iter()
        .map(|&s| {
            let m = run_rollout(RolloutRequest {
                config: cfg,
                seed: s,
                policy: Box::new(RandomSubset::new(Some(BUDGET), 1)),
                horizon_steps: STEPS,
                probe: None,
            })
            .expect("rollout");
            hg_train::fitness::evaluate(&m, BURN_IN).expect("fitness").f2
        })
        .collect()
}

/// The C3 exit criterion: CMA-ES solves F-L1 within 30 generations, and the
/// champion beats budget-matched random on the frozen hold-out seeds.
#[test]
fn cma_es_solves_fl1_within_thirty_generations() {
    let dir = tempfile::tempdir().expect("tempdir");
    let cfg = sim_config(dir.path());

    let baseline = random_baseline(&cfg);
    let (base_mean, base_sd) = mean_sd(&baseline);

    let training = TrainingConfig {
        algo: Algo::CmaEs,
        generations: 30,
        seeds_per_member: 3,
        sigma: 1.0,
        fitness: FitnessFn::F2RegimeOccupancy,
        horizon_steps: STEPS,
        burn_in_windows: BURN_IN,
        holdout_seeds: HOLDOUTS.to_vec(),
        campaign_seed: 20260725,
        ..Default::default()
    };

    let spec = CampaignSpec {
        campaign_id: "fl1_cma".into(),
        base_dir: dir.path().to_path_buf(),
        sim_config: &cfg,
        cognition: cognition(),
        training,
    };
    let out = run_campaign(&spec).expect("campaign");

    let solved_at = out
        .generations
        .iter()
        .position(|g| g.champion_holdout_mean >= 0.9)
        .map(|i| out.generations[i].generation);

    let last = out.generations.last().expect("generations");
    assert!(
        solved_at.is_some(),
        "CMA-ES did not reach the optimum within 30 generations; best hold-out was \
         {:.3}. Baseline (budget-matched random) is {base_mean:.3}.",
        last.champion_holdout_mean
    );
    assert!(
        last.champion_holdout_mean > base_mean,
        "champion {:.3} did not beat budget-matched random {base_mean:.3}",
        last.champion_holdout_mean
    );

    // The champion is the "prefer the glider" genome: logit 0 above logit 1.
    let p = &out.champion.params;
    assert_eq!(p.len(), 2, "StaticBias on a 2-rule fixture");
    assert!(
        p[0] > p[1],
        "champion logits {p:?} do not prefer rule 0 (glider) — it solved the fixture \
         by some other route, which means the fixture is not measuring what it claims"
    );

    // ---- On the 2-sigma form of the criterion.
    //
    // The plan's exit is "F2 margin > 2σ over budget-matched random on
    // hold-out seeds". On THIS fixture the hold-out standard deviation is
    // exactly zero: the ring is symmetric and the arms are effectively
    // deterministic, so every hold-out seed returns the same F2. A "> 2σ"
    // test against σ = 0 is arithmetic, not measurement — the same
    // degeneracy the null-spread guard exists to catch elsewhere in this
    // project. The absolute margin is asserted instead, and the 2σ form is
    // left to Stage 2 on real rule sets, where seed variance is nonzero.
    assert_eq!(
        base_sd, 0.0,
        "hold-out seed variance became nonzero on F-L1 — the 2-sigma criterion is now \
         meaningful here and this test should start using it"
    );
    assert!(
        last.champion_holdout_mean - base_mean > 0.5,
        "margin {:.3} over random {base_mean:.3} is smaller than the calibrated 0.75",
        last.champion_holdout_mean - base_mean
    );
}

/// The fixture only measures anything if the pure arms differ as calibrated.
/// If a rule set or an evaluator constant drifts, this fails first and
/// explains why, rather than the campaign test failing mysteriously.
///
/// Measured with `StaticBias` — the policy actually under test — because
/// the arms a different policy class can reach are not the arms this
/// campaign is choosing between (see the module docs).
#[test]
fn the_pure_arms_still_match_the_calibration() {
    use hg_cognition::{Parameterized, StaticBias};
    let dir = tempfile::tempdir().expect("tempdir");
    let cfg = sim_config(dir.path());

    let arm = |logits: [f32; 2]| -> f64 {
        let mut p = StaticBias::new(2, Some(BUDGET), 1);
        p.set_params(&logits);
        let m = run_rollout(RolloutRequest {
            config: &cfg,
            seed: HOLDOUTS[0],
            policy: Box::new(p),
            horizon_steps: STEPS,
            probe: None,
        })
        .expect("rollout");
        hg_train::fitness::evaluate(&m, BURN_IN).expect("fitness").f2
    };

    let glider = arm([10.0, -10.0]);
    let bloat = arm([-10.0, 10.0]);
    let (random_mean, _) = mean_sd(&random_baseline(&cfg));

    assert!(
        glider >= 0.9,
        "glider arm scored {glider}, calibrated 1.00 — if this dropped to -0.50 the \
         budget has outgrown the glider group and the correct answer is no longer \
         REACHABLE by StaticBias, not merely unlearned"
    );
    assert!(bloat <= -0.4, "bloat arm scored {bloat}, calibrated -0.50");
    assert!(
        random_mean < glider - 0.5,
        "budget-matched random scored {random_mean} against glider {glider} — the \
         margin a learned policy needs has closed, and F-L1 no longer has room to \
         measure anything"
    );
}
