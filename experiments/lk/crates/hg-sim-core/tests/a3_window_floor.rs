//! A3 (v6 §5): window-floor resolution. Run one 26k-step cell at the
//! finest cadence (25), dump its per-window closure feature vectors, and
//! re-window offline by subsampling to effective cadence
//! {25,50,100,200,500}. At each cadence report the CD tuple, the lag-1
//! autocorrelation of a micro feature, and the cross-seed spread of
//! CD_clock. Verdict (written to leakage_closure_spec.md): was the ≥200
//! window floor statistics-limited (signal < spread at every cadence) or
//! only step-limited (spread holds as cadence shrinks)?
//!
//! NOTE this needs a dedicated fine-cadence run: no persisted artifact
//! carried sub-500-step closure feature vectors, and snapshots existed
//! only at cadence 500 — so "re-window a persisted run" was not literally
//! possible. The run's PURPOSE is estimator characterization, not
//! substrate science (no CD claim is made here), which is A3's remit.
//! Run with --ignored; HYPERSIM_SWEEP_OUT persists.

use hg_causal::closure::{estimate, ClosureSettings, WindowSample};
use hg_sim_core::{run_headless, SimConfig};
use std::path::Path;

fn rules_toml(q: f64, tau: f64) -> String {
    std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../rules/lk_pair_capped.toml"
    ))
    .unwrap()
    .replace(
        "property = 6\nop       = \"lt\"\nvalue    = 4.0",
        "property = 6\nop       = \"lt\"\nvalue    = 1.0e18",
    )
    .replace("value = 50.0\n", &format!("value = {q:.1}\n"))
    .replace("value = 60.0", &format!("value = {:.1}", q + 10.0))
    .replace("offset = 50.0", &format!("offset = {:.1}", 2.0 * tau))
    .replace("value = 25.0", &format!("value = {tau:.1}"))
}

fn run_fine(dir: &Path, seed: u64) -> Vec<WindowSample> {
    // Swap regime (transport_first), the long-diffusion-era cell (the
    // 26k-step archetype), cadence 25, sample dump on.
    let toml = format!(
        r#"
experiment_name = "a3_floor"
seed = {seed}
out_dir = "{}"
[rules]
source = "Inline"
[initial]
condition_type = "TriLattice"
width = 32
[engine]
step_mode = "MaxParallel"
max_steps = 26000
metrics_window_interval = 25
max_pairings_per_step = 100000
thread_count = 1
[lk_rules]
enabled = true
e_total = 400000.0
g_max = 1.0e18
priority = "transport_first"
q = 75.0
tau = 25.0
[leakage]
enabled = true
cadence_steps = 25
dump_closure_samples = true
[output]
fixed_provenance = true
"#,
        dir.display(),
    );
    let mut cfg = SimConfig::from_toml(&toml).unwrap();
    cfg.rules.inline_toml = Some(rules_toml(75.0, 25.0));
    let artifacts = run_headless(&cfg, None, None, None).expect("run");
    let json = std::fs::read_to_string(artifacts.run_dir.join("closure_samples.json"))
        .expect("closure_samples.json (dump_closure_samples must be on)");
    serde_json::from_str(&json).expect("parse samples")
}

/// Re-window: keep every `stride`-th sample, renumbering the clock
/// window index (clock_v[0]) so the macro+clock baseline stays
/// well-defined at the coarsened cadence.
fn subsample(samples: &[WindowSample], stride: usize) -> Vec<WindowSample> {
    samples
        .iter()
        .step_by(stride)
        .enumerate()
        .map(|(i, s)| {
            let mut s = s.clone();
            if !s.clock_v.is_empty() {
                s.clock_v[0] = i as f64;
            }
            s
        })
        .collect()
}

/// Lag-1 autocorrelation of micro slot `j` over a sample series.
fn autocorr1(samples: &[WindowSample], j: usize) -> f64 {
    let xs: Vec<f64> = samples.iter().filter_map(|s| s.micro_v.get(j).copied()).collect();
    let n = xs.len();
    if n < 3 {
        return f64::NAN;
    }
    let mean = xs.iter().sum::<f64>() / n as f64;
    let var = xs.iter().map(|x| (x - mean).powi(2)).sum::<f64>();
    if var < 1e-12 {
        return f64::NAN;
    }
    let cov: f64 = (1..n).map(|i| (xs[i] - mean) * (xs[i - 1] - mean)).sum();
    cov / var
}

#[test]
#[ignore]
fn a3_window_floor_resolution() {
    let root = std::env::var("HYPERSIM_SWEEP_OUT").ok();
    let tmp = tempfile::tempdir().unwrap();
    let seeds = [42u64, 43, 44];
    // Finest-cadence samples per seed.
    let per_seed: Vec<Vec<WindowSample>> = seeds
        .iter()
        .map(|&s| {
            let dir = root
                .as_ref()
                .map(|r| {
                    let p = Path::new(r).join(format!("a3_s{s}"));
                    std::fs::create_dir_all(&p).unwrap();
                    p
                })
                .unwrap_or_else(|| tmp.path().join(format!("a3_s{s}")));
            run_fine(&dir, s)
        })
        .collect();
    println!("finest windows/seed: {:?}", per_seed.iter().map(|v| v.len()).collect::<Vec<_>>());

    // Force estimation regardless of window count so we can READ the
    // tuple at every cadence (this phase characterizes the estimator; it
    // makes no CD claim, so the floor is not applied here).
    let settings = ClosureSettings { min_windows: 20, ..Default::default() };
    println!("eff_cadence | n_windows | CD_raw | CD_clock (mean±spread over seeds) | micro autocorr1");
    for (stride, eff) in [(1usize, 25), (2, 50), (4, 100), (8, 200), (20, 500)] {
        let mut clocks = Vec::new();
        let mut raws = Vec::new();
        let mut acs = Vec::new();
        let mut nwin = 0;
        for samples in &per_seed {
            let sub = subsample(samples, stride);
            nwin = sub.len();
            let rep = estimate(&sub, &settings);
            raws.push(rep.cd_raw_bits);
            if let Some(c) = rep.cd_clock_bits {
                clocks.push(c);
            }
            acs.push(autocorr1(&sub, 6)); // slot 6 = top WL-class frequency
        }
        let cmean = clocks.iter().sum::<f64>() / clocks.len().max(1) as f64;
        let spread = clocks.iter().copied().fold(f64::MIN, f64::max)
            - clocks.iter().copied().fold(f64::MAX, f64::min);
        let rmean = raws.iter().sum::<f64>() / raws.len() as f64;
        let acmean = acs.iter().filter(|a| a.is_finite()).sum::<f64>()
            / acs.iter().filter(|a| a.is_finite()).count().max(1) as f64;
        println!(
            "{eff} | {nwin} | {rmean:.4} | {cmean:.4} ± {spread:.4} | {acmean:.3}"
        );
    }
    println!(
        "VERDICT INPUTS: if |CD_clock| < spread at every cadence ⇒ statistics-limited \
         (EXP-L7R needs longer DYNAMICS, not finer windows); if spread stays below a \
         nonzero CD_clock as cadence shrinks ⇒ step-limited (re-windowing is a legitimate \
         route; floor restated as an effective-sample count)."
    );
}
