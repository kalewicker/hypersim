//! Phase D5, step 0 — resolving F4's horizon/window before Stage 2′ can
//! test it at all.
//!
//! Stage I measured `f4_pairs` at **2 to 10** across every fixture the
//! programme owns, against `PI_MIN_PAIRS = 50`, so F4 reports `unmeasured`
//! everywhere and P-3/P-4 cannot be run on it
//! (`docs/instrument_registry.md` §5.1).
//!
//! `f4_pairs = floor(horizon / window) - burn_in - 1`, so clearing 50 needs
//! `horizon / window >= 56` at `burn_in = 5`. There are two routes and they
//! are **not** equivalent:
//!
//! * **Longer horizon at fixed window** — the window is what a symbol
//!   *means* (one JSD between consecutive `window`-step type histograms).
//!   Holding it fixed keeps F4's alphabet comparable to every other window
//!   statistic the programme has published.
//! * **Shorter window at fixed horizon** — cheap, and it changes the
//!   symbol. Stage 1 measured input coupling falling 0.93 at `w = 32` to
//!   0.68 at `w = 4`: shrinking the window moves what a window resolves,
//!   so an F4 measured at `w = 4` is not the same quantity as one at
//!   `w = 24`.
//!
//! This probe measures both, so the choice is made on evidence and recorded
//! in F4's measurement definition rather than picked to clear a bar.

use hg_cognition::RandomSubset;
use hg_observe::RuleSelectionPolicy;
use hg_sim_core::{run_rollout, RolloutRequest};
use hg_train::diagnostics::{Fixture, RuleSource};
use hg_train::fitness::{self, PI_MIN_PAIRS};

const BURN_IN: usize = 5;

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

struct Probe {
    label: String,
    pairs: usize,
    distinct: usize,
    f4: String,
    secs: f64,
    nodes: usize,
    /// The underlying observable, before quantization. `distinct == 1` with
    /// a *varying* JSD is a quantizer failure; with a constant JSD it is a
    /// frozen substrate. Only these numbers separate the two.
    jsd_min: f64,
    jsd_max: f64,
    jsd_sd: f64,
    /// Spearman trend, monotone ceiling and share, from
    /// `fitness::f4_drift` — the shipped diagnostic (§4A.6), not a
    /// reimplementation. Equal-count symbolization reads a monotone drift
    /// as temporal structure, so the route choice cannot be made without
    /// these.
    drift: fitness::F4Drift,
}

fn probe(label: &str, f: &Fixture, budget: usize, causal: bool, seed: u64) -> Probe {
    let dir = tempfile::tempdir().expect("tempdir");
    let cfg = if causal {
        f.sim_config_causal(dir.path(), seed)
    } else {
        f.sim_config(dir.path(), seed)
    };
    let t0 = std::time::Instant::now();
    let m = run_rollout(RolloutRequest {
        config: &cfg,
        seed,
        policy: Box::new(RandomSubset::new(Some(budget), seed)) as Box<dyn RuleSelectionPolicy>,
        horizon_steps: f.steps,
        probe: None,
    })
    .expect("rollout");
    let secs = t0.elapsed().as_secs_f64();
    let fit = fitness::evaluate(&m, BURN_IN).expect("evaluate");

    // Distinct symbols actually realized, recomputed here so the probe
    // reports WHY a measured F4 is zero when it is.
    let mut seen = [false; fitness::PI_ALPHABET];
    let mut jsd = Vec::new();
    for r in m.windows.iter().skip(BURN_IN) {
        if let Some(j) = r.type_histogram_jsd {
            let q = (fitness::PI_ALPHABET as f64 * j).floor() as isize;
            seen[q.clamp(0, fitness::PI_ALPHABET as isize - 1) as usize] = true;
            jsd.push(j);
        }
    }
    let n = jsd.len().max(1) as f64;
    let mean = jsd.iter().sum::<f64>() / n;
    let sd = (jsd.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n).sqrt();
    Probe {
        label: label.to_string(),
        pairs: fit.engagement.f4_pairs,
        distinct: seen.iter().filter(|b| **b).count(),
        f4: match fit.f4_monotone_drift.value() {
            Some(v) => format!("{v:.4}"),
            None => "unmeasured".into(),
        },
        secs,
        nodes: m.windows.last().map(|w| w.n_nodes as usize).unwrap_or(0),
        jsd_min: jsd.iter().cloned().fold(f64::INFINITY, f64::min),
        jsd_max: jsd.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
        jsd_sd: sd,
        drift: fitness::f4_drift(&m, BURN_IN),
    }
}

#[test]
#[ignore = "cost probe: runs long-horizon rollouts; run explicitly for D5"]
fn f4_horizon_and_window_probe() {
    println!("\n=== D5 step 0 — what makes F4 measurable, and what it costs ===");
    println!("PI_MIN_PAIRS = {PI_MIN_PAIRS}; f4_pairs = windows - burn_in({BURN_IN}) - 1\n");
    println!(
        "{:<28} {:>5} {:>5} {:>4} {:>11} {:>9} {:>9} {:>7} {:>8} {:>6} {:>6}",
        "config", "wins", "pairs", "sym", "F4", "jsd_min", "jsd_max", "jsd_sd", "trend",
        "share", "secs"
    );

    let mut rows = Vec::new();

    // Route A — longer horizon, window frozen at the Stage 4/5 value of 24.
    for steps in [384u64, 768, 1536] {
        let f = wn_three(steps, 24);
        rows.push(probe(
            &format!("A: steps={steps} w=24 k=4"),
            &f,
            4,
            false,
            9_500_001,
        ));
    }
    // Route B — horizon frozen at 384, window shrunk.
    for window in [24u64, 12, 6] {
        let f = wn_three(384, window);
        rows.push(probe(
            &format!("B: steps=384 w={window} k=4"),
            &f,
            4,
            false,
            9_500_001,
        ));
    }
    // Does whatever route A and B show generalize off `wn_three`? The other
    // Stage 2′ substrate and one non-fixture rule set, at the cheap config.
    rows.push(probe(
        "tri_two steps=384 w=6 k=4",
        &Fixture {
            name: "tri_two",
            rules: RuleSource::File("rules/tri_two.toml"),
            ic: "TriLattice",
            width: 8,
            node_count: 24,
            steps: 384,
            window: 6,
            step_mode: "MaxParallel",
        },
        4,
        false,
        9_500_001,
    ));
    rows.push(probe(
        "growth_tread steps=384 w=6 k=4",
        &Fixture {
            name: "growth_treadmill",
            rules: RuleSource::Builtin("growth_treadmill"),
            ic: "WhiteNoise",
            width: 8,
            node_count: 24,
            steps: 384,
            window: 6,
            step_mode: "MaxParallel",
        },
        4,
        false,
        9_500_001,
    ));

    for r in &rows {
        println!(
            "{:<28} {:>5} {:>5} {:>4} {:>11} {:>9.6} {:>9.6} {:>7.4} {:>8.3} {:>6.3} {:>6.1}",
            r.label,
            r.pairs + BURN_IN + 1,
            r.pairs,
            r.distinct,
            r.f4,
            r.jsd_min,
            r.jsd_max,
            r.jsd_sd,
            r.drift.trend,
            r.drift.share_of_ceiling,
            r.secs
        );
    }
    println!("(final node counts: {:?})", rows.iter().map(|r| r.nodes).collect::<Vec<_>>());

    // The one number D5 is blocked on.
    let a1536 = rows
        .iter()
        .find(|r| r.label.starts_with("A: steps=1536"))
        .expect("route A at 1536");
    println!(
        "\nroute A at 1536/24 gives {} pairs (bar {PI_MIN_PAIRS}) in {:.1}s",
        a1536.pairs, a1536.secs
    );
}

/// Same probe with `[causal]` on, because F5 needs it and F3's history says
/// the causal layer is where the cost lives (bug 8's fix made rollouts an
/// order of magnitude slower).
#[test]
#[ignore = "cost probe: runs long-horizon causal rollouts; run explicitly for D5"]
fn f5_causal_cost_probe() {
    println!("\n=== D5 step 0b — causal-layer cost at the F4-measurable horizon ===");
    println!(
        "{:<34} {:>7} {:>6} {:>12} {:>8} {:>8}",
        "config", "windows", "pairs", "F4", "secs", "nodes"
    );
    for (steps, budget) in [(384u64, 4usize), (768, 4), (1536, 4)] {
        let f = wn_three(steps, 24);
        let r = probe(
            &format!("causal steps={steps} w=24 k={budget}"),
            &f,
            budget,
            true,
            9_500_001,
        );
        println!(
            "{:<34} {:>7} {:>6} {:>12} {:>8.1} {:>8}",
            r.label,
            r.pairs + BURN_IN + 1,
            r.pairs,
            r.f4,
            r.secs,
            r.nodes
        );
    }
}
