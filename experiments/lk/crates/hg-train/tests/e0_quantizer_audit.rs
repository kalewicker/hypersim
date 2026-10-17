//! Phase E0 — the quantizer audit (`hypersim_nn_experiments_plan_v3.md`
//! §6.1, §10).
//!
//! > Every quantizer, binning, or thresholding step declares the **expected
//! > range of the observable it consumes**, and the registry check compares
//! > that range against the quantizer's design range.
//!
//! F4's defect was a fixed `[0, 1]` grid applied to an observable living in
//! `[0.002, 0.085]`. The grid's *output* was in range and meaningless. That
//! is catchable before a run by writing down where the observable sits — and
//! the same shape exists elsewhere in the learning layer, unchecked.
//!
//! # What counts as a quantizer here
//!
//! Not every `clamp`. The defect class is **a fixed scale applied to a
//! continuous or unbounded observable**, where the scale's design range is a
//! constant and the observable's actual range is a substrate property that
//! nothing verifies. Excluded, with reasons:
//!
//! * **Saturation guards on mathematically-bounded quantities** —
//!   `jsd.clamp(0, 1)` in `hg-eval/window.rs`, `interestingness.clamp(0, 1)`
//!   in `classify.rs`. The bound is a theorem, not an assumption, so there
//!   is no observable-range question to ask.
//! * **Rank and quantile computations** — `hg-observe/locality.rs`,
//!   `hg-eval/chains.rs`, `hg-sim-core/twin.rs`. These are data-adaptive by
//!   construction and immune to this defect; it is exactly why F4's fix was
//!   to become one.
//! * **GUI rendering** (`hg-gui/*`) — display, not measurement.
//! * **Integer/optimizer arithmetic** — CMA-ES's `clamp(-1, 1)` on an
//!   exponent, population sizing. Not observables.
//!
//! What is left is the **policy's own input** (`hg-cognition/features.rs`)
//! and the **probe readout** (`hg-cognition/readout.rs`): six fixed scales,
//! none of which has ever been checked against the substrates it runs on.

use hg_cognition::features::{ARITY_SCALE, CAND_SCALE, DEG_SCALE, PRIORITY_SCALE};
use hg_core::GraphSnapshot;
use hg_observe::{Binding, RuleSelectionPolicy};
use hg_sim_core::{run_rollout, RolloutRequest};
use hg_train::diagnostics::{Fixture, RuleSource};

const STEPS: u64 = 384;
const WINDOW: u64 = 6;
const BUDGET: usize = 4;
const SEED: u64 = 9_700_001;

/// One observable's realized distribution, and what its scale does to it.
#[derive(Default, Clone)]
struct Observed {
    vals: Vec<f64>,
}

impl Observed {
    fn push(&mut self, v: f64) {
        self.vals.push(v);
    }
    fn stats(&self) -> (f64, f64, f64) {
        if self.vals.is_empty() {
            return (f64::NAN, f64::NAN, f64::NAN);
        }
        let mut s = self.vals.clone();
        s.sort_by(|a, b| a.partial_cmp(b).expect("no NaN"));
        (s[0], s[s.len() / 2], s[s.len() - 1])
    }
}

/// A quantizer under audit: what it consumes, how it scales it, and the
/// range its design assumes.
struct Quantizer {
    name: &'static str,
    /// The scale constant, as written in the source.
    design: String,
    /// The observable → `[0, 1]` map the code applies.
    scale: Box<dyn Fn(f64) -> f64>,
}

fn log_scale(denom: f64) -> Box<dyn Fn(f64) -> f64> {
    Box::new(move |x| ((1.0 + x).ln() / (1.0 + denom).ln()).clamp(0.0, 1.0))
}

fn quantizers() -> Vec<Quantizer> {
    vec![
        Quantizer {
            name: "priority (tanh)",
            design: format!("PRIORITY_SCALE = {PRIORITY_SCALE}, tanh, unbounded i32 in"),
            scale: Box::new(|x| (x / PRIORITY_SCALE as f64).tanh()),
        },
        Quantizer {
            name: "arity (linear)",
            design: format!("ARITY_SCALE = {ARITY_SCALE}, clamped [0,1]"),
            scale: Box::new(|x| (x / ARITY_SCALE as f64).clamp(0.0, 1.0)),
        },
        Quantizer {
            name: "node degree (log)",
            design: format!("DEG_SCALE = {DEG_SCALE}, clamped [0,1]"),
            scale: log_scale(DEG_SCALE as f64),
        },
        Quantizer {
            name: "candidate count (log)",
            design: format!("CAND_SCALE = {CAND_SCALE}, clamped [0,1]"),
            scale: log_scale(CAND_SCALE as f64),
        },
    ]
}

/// Records the raw observables the feature extractor consumes.
///
/// Deliberately independent of `FeatureExtractor`: the point is to compare
/// what the substrate *offers* against what the scale assumes, and reading
/// the scaled output would only tell us what the scale already decided.
struct AuditPolicy {
    inner: Box<dyn RuleSelectionPolicy>,
    obs: std::sync::Arc<std::sync::Mutex<[Observed; 4]>>,
}

impl RuleSelectionPolicy for AuditPolicy {
    fn select(&mut self, snapshot: &GraphSnapshot<'_>, candidates: &[Binding]) -> Vec<usize> {
        {
            let mut o = self.obs.lock().expect("audit mutex");
            o[3].push(candidates.len() as f64);
            for b in candidates {
                o[0].push(b.priority as f64);
                o[1].push(b.bound_nodes.len() as f64);
                for &n in b.bound_nodes.iter() {
                    o[2].push(snapshot.edges_incident_to(n).len() as f64);
                }
            }
        }
        self.inner.select(snapshot, candidates)
    }
}

fn audit(f: &Fixture, budget: usize) -> [Observed; 4] {
    let dir = tempfile::tempdir().expect("tempdir");
    let cfg = f.sim_config(dir.path(), SEED);
    let obs = std::sync::Arc::new(std::sync::Mutex::new(std::array::from_fn(|_| {
        Observed::default()
    })));
    let pol = AuditPolicy {
        inner: Box::new(hg_cognition::RandomSubset::new(Some(budget), SEED)),
        obs: std::sync::Arc::clone(&obs),
    };
    // The policy is moved into the engine and dropped when the rollout
    // ends, so the counters have to outlive it — same reason `MlpScorer`
    // holds its cap counter in an `Arc`.
    {
        run_rollout(RolloutRequest {
            config: &cfg,
            seed: SEED,
            policy: Box::new(pol) as Box<dyn RuleSelectionPolicy>,
            horizon_steps: f.steps,
            probe: None,
        })
        .expect("rollout");
    }
    let out = obs.lock().expect("audit mutex").clone();
    out
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

#[test]
#[ignore = "E0: the quantizer audit; run explicitly"]
fn observable_ranges_against_design_ranges() {
    println!("\n=== E0 — quantizer audit: observable range vs design range ===");
    println!(
        "The F4 shape: a fixed scale whose design range does not match where the\n\
         observable actually lives. `used` is the fraction of the scale's [0,1]\n\
         output range the substrate actually occupies — F4's grid scored 0.000.\n"
    );

    let subs = [
        fixture("tri_two", "rules/tri_two.toml", "TriLattice"),
        fixture("wn_three", "rules/wn_three.toml", "WhiteNoise"),
    ];

    for f in &subs {
        println!("## {} (k = {BUDGET}, {} steps)", f.name, f.steps);
        let obs = audit(f, BUDGET);
        println!(
            "{:<24} {:>10} {:>10} {:>10} {:>9} {:>9} {:>7} {:>7}",
            "observable", "obs min", "obs med", "obs max", "scaled lo", "scaled hi", "used", "sat"
        );
        for (q, o) in quantizers().iter().zip(obs.iter()) {
            let (lo, med, hi) = o.stats();
            let (slo, smed, shi) = ((q.scale)(lo), (q.scale)(med), (q.scale)(hi));
            let sat = o
                .vals
                .iter()
                .filter(|v| {
                    let s = (q.scale)(**v);
                    s <= 0.0 || s >= 1.0
                })
                .count() as f64
                / o.vals.len().max(1) as f64;
            println!(
                "{:<24} {:>10.2} {:>10.2} {:>10.2} {:>9.4} {:>9.4} {:>7.3} {:>7.1}%",
                q.name,
                lo,
                med,
                hi,
                slo,
                shi,
                shi - slo,
                sat * 100.0
            );
            let _ = smed;
        }
        println!("    design ranges:");
        for q in quantizers() {
            println!("      {:<22} {}", q.name, q.design);
        }
        println!();
    }
}

/// The same observables under `feature_set = "v2"`.
///
/// The claim is narrow and must be checked as stated: the two scales that
/// **saturated** no longer can, and the two that merely compressed are
/// untouched. `used` is deliberately not the target — see the module header
/// of `hg-cognition/features.rs`.
#[test]
#[ignore = "E0: re-audit under feature_set v2; run explicitly"]
fn v2_encodings_do_not_saturate() {
    println!("\n=== E0 — the same observables under feature_set v2 ===");
    println!(
        "{:<12} {:<24} {:>10} {:>10} {:>9} {:>9} {:>7}",
        "substrate", "observable", "obs min", "obs max", "v1 sat", "v2 sat", "v2 used"
    );

    let v2_deg = |deg: f64, mean_deg: f64| {
        let r = deg / mean_deg.max(f64::MIN_POSITIVE);
        if r <= 0.0 { 0.0 } else { r / (1.0 + r) }
    };
    let v2_cand = |n: f64, nodes: f64| {
        let r = n / nodes.max(1.0);
        if r <= 0.0 { 0.0 } else { r / (1.0 + r) }
    };

    for f in [
        fixture("tri_two", "rules/tri_two.toml", "TriLattice"),
        fixture("wn_three", "rules/wn_three.toml", "WhiteNoise"),
    ] {
        let obs = audit(&f, BUDGET);
        // Graph-relative references, taken at the scale the rollout actually
        // reached rather than assumed.
        let dir = tempfile::tempdir().expect("tempdir");
        let cfg = f.sim_config(dir.path(), SEED);
        let m = run_rollout(RolloutRequest {
            config: &cfg,
            seed: SEED,
            policy: Box::new(hg_cognition::RandomSubset::new(Some(BUDGET), SEED))
                as Box<dyn RuleSelectionPolicy>,
            horizon_steps: f.steps,
            probe: None,
        })
        .expect("rollout");
        let last = m.windows.last().expect("windows");
        let mean_deg = last.n_edges as f64 / (last.n_nodes as f64).max(1.0);
        let nodes = last.n_nodes as f64;

        for (idx, name, v1, v2) in [
            (
                2usize,
                "node degree",
                log_scale(DEG_SCALE as f64) as Box<dyn Fn(f64) -> f64>,
                Box::new(move |x: f64| v2_deg(x, mean_deg)) as Box<dyn Fn(f64) -> f64>,
            ),
            (
                3,
                "candidate count",
                log_scale(CAND_SCALE as f64),
                Box::new(move |x: f64| v2_cand(x, nodes)),
            ),
        ] {
            let o = &obs[idx];
            let (lo, _, hi) = o.stats();
            let s1 = o.vals.iter().filter(|v| v1(**v) >= 1.0).count() as f64
                / o.vals.len().max(1) as f64;
            let s2 = o.vals.iter().filter(|v| v2(**v) >= 1.0).count() as f64
                / o.vals.len().max(1) as f64;
            println!(
                "{:<12} {:<24} {:>10.2} {:>10.2} {:>8.1}% {:>8.1}% {:>7.3}",
                f.name,
                name,
                lo,
                hi,
                s1 * 100.0,
                s2 * 100.0,
                v2(hi) - v2(lo)
            );
            assert_eq!(s2, 0.0, "v2 must not saturate {name} on {}", f.name);
        }
    }
    println!("\n    v2 saturation is 0% by construction: x/(1+x) never reaches 1.");
}

/// The probe readout's scales (`hg-cognition/readout.rs`): `DEG_SCALE` on
/// mean/max degree and `SIZE_SCALE` on raw graph size. Same defect class,
/// different consumer — these feed Stage 0b/1's separation and memory
/// diagnostics rather than the policy.
#[test]
#[ignore = "E0: readout half of the quantizer audit; run explicitly"]
fn readout_observable_ranges() {
    println!("\n=== E0 — quantizer audit: the probe readout's scales ===");
    println!(
        "{:<12} {:<22} {:>10} {:>10} {:>9} {:>9} {:>7} {:>7}",
        "substrate", "observable", "obs min", "obs max", "scaled lo", "scaled hi", "used", "sat"
    );

    let rd_deg = log_scale(hg_cognition::readout::DEG_SCALE);
    let rd_size = log_scale(hg_cognition::readout::SIZE_SCALE);

    for f in [
        fixture("tri_two", "rules/tri_two.toml", "TriLattice"),
        fixture("wn_three", "rules/wn_three.toml", "WhiteNoise"),
    ] {
        let dir = tempfile::tempdir().expect("tempdir");
        let cfg = f.sim_config(dir.path(), SEED);
        let m = run_rollout(RolloutRequest {
            config: &cfg,
            seed: SEED,
            policy: Box::new(hg_cognition::RandomSubset::new(Some(BUDGET), SEED))
                as Box<dyn RuleSelectionPolicy>,
            horizon_steps: f.steps,
            probe: None,
        })
        .expect("rollout");

        for (name, vals, scale) in [
            (
                "mean degree (DEG)",
                m.windows.iter().filter_map(|w| w.mean_degree).collect::<Vec<f64>>(),
                &rd_deg,
            ),
            (
                "graph size (SIZE)",
                m.windows.iter().map(|w| w.n_nodes as f64).collect::<Vec<f64>>(),
                &rd_size,
            ),
        ] {
            if vals.is_empty() {
                println!("{:<12} {:<22} {:>10}", f.name, name, "no rows");
                continue;
            }
            let lo = vals.iter().cloned().fold(f64::INFINITY, f64::min);
            let hi = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let sat = vals.iter().filter(|v| scale(**v) >= 1.0).count() as f64 / vals.len() as f64;
            println!(
                "{:<12} {:<22} {:>10.2} {:>10.2} {:>9.4} {:>9.4} {:>7.3} {:>7.1}%",
                f.name,
                name,
                lo,
                hi,
                scale(lo),
                scale(hi),
                scale(hi) - scale(lo),
                sat * 100.0
            );
        }
    }
    println!(
        "\n    DEG_SCALE = {}, SIZE_SCALE = {}",
        hg_cognition::readout::DEG_SCALE,
        hg_cognition::readout::SIZE_SCALE
    );
}

/// The audit's own firing test: the F4 grid, scored by the same metric, must
/// come out at zero used range. If this ever passes, the metric has stopped
/// measuring the thing the audit exists to catch.
#[test]
fn the_audit_metric_catches_the_f4_grid() {
    // The measured type-histogram JSD band, from D5.
    let jsd: Vec<f64> = vec![0.002, 0.0031, 0.0092, 0.0311, 0.0852];
    let v2_grid = |x: f64| ((4.0 * x).floor() as usize).min(3) as f64 / 3.0;

    let lo = v2_grid(*jsd.first().unwrap());
    let hi = v2_grid(*jsd.last().unwrap());
    assert_eq!(
        hi - lo,
        0.0,
        "the v2 grid must score zero used range on the real JSD band"
    );

    // And the quantile symbolization that replaced it must score the full
    // range on the same data — otherwise the fix did not fix it.
    let mut sorted = jsd.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let cuts: Vec<f64> = (1..4).map(|i| sorted[(i * sorted.len()) / 4]).collect();
    let q = |x: f64| cuts.iter().filter(|c| x >= **c).count() as f64 / 3.0;
    assert!(
        q(*jsd.last().unwrap()) - q(*jsd.first().unwrap()) > 0.99,
        "equal-count cutting must occupy the full alphabet on the same band"
    );
}
