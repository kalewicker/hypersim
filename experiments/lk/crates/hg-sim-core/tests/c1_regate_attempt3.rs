//! v9 C1 §3.1 + §3.2: the corrected-mask re-gate and EXP-L7R attempt 3.
//!
//! **Zero new compute.** Everything here reads the persisted E₀ = 2e6
//! runs from v8 B3 (`runs/v8_l7r_passive/`), which carry `leakage.csv`,
//! `metrics.csv` and `closure_samples.json`.
//!
//! ## §3.1 — the corrected mask (executing the v8 pre-registration)
//!
//! Attempt 2's stratum was gated on `wormhole_count_est > 0`, a sampled,
//! threshold-limited census that Q7-1 had already retired in favour of
//! the gen>0 standing count. The stratum ended when that instrument
//! crossed its own detection floor, while production continued and the
//! phase stayed Sparse — so the variance test failed on the gate, not on
//! the dynamics.
//!
//! The corrected census is recoverable from persisted data with no
//! snapshots: the gen-0 lattice is **non-removable** under the age gate
//! (lk_fuse's generation guard; pinned by
//! `bg_lattice_stays_non_removable_though_it_passes_the_age_guard`), so
//! every node beyond the initial 32² is a gen>0 filament node and
//!
//!   census(w) = n_nodes(w) − 1024, exactly.
//!
//! Sanity-checked here rather than assumed: `n_nodes` must never dip
//! below 1024 in any run.
//!
//! PRE-REGISTERED IN v8, NOT AMENDED HERE:
//!   * prediction — the quartile-4 variance inflation disappears (it was
//!     the sampled census growing noisy at its detection floor) and the
//!     variance ratio enters [0.5, 2.0] on the full stratum;
//!   * exclusion — if it STILL fails, the destabilization is real, the
//!     stationarity milestone shrinks to quartiles 1–3, and attempt 3
//!     runs on the Q1–Q3 sub-stratum only if effective N is independently
//!     re-cleared there.
//!
//! ## §3.2 — attempt 3, under the C0-frozen configuration
//!
//! Single-trajectory CD. Uncertainty is a moving-block bootstrap
//! (b = 20 windows headline, sensitivity at 10/40, B = 1000, percentile
//! 2.5/97.5) plus `sampler_spread` across the five runs. A claim requires
//! BOTH: the interval excludes 0, AND the point estimate exceeds
//! `sampler_spread`. See the C0 memo in `docs/leakage_closure_spec.md`.
//!
//! Run with --ignored --nocapture.

use hg_causal::closure::{estimate_stratum, ClosureSettings, WindowSample};
use std::path::{Path, PathBuf};

/// 32² TriLattice: the non-removable gen-0 population.
const LATTICE_N: usize = 32 * 32;

/// C0-frozen bootstrap parameters. Changing any of these after an
/// attempt-3 number has been seen is a protocol violation (v9 hazard 1).
const BOOT_BLOCKS: [usize; 3] = [10, 20, 40];
const BOOT_HEADLINE: usize = 20;
const BOOT_REPLICATES: usize = 1000;
const BOOT_SEED: u64 = 0x9E37_79B9_7F4A_7C15;

struct Run {
    name: String,
    /// gen>0 standing census per window (the instrument of record).
    census: Vec<f64>,
    /// Sampled wormhole census — attempt 2's gate, kept for the contrast.
    sampled_census: Vec<f64>,
    producing: Vec<bool>,
    sparse: Vec<bool>,
    samples: Vec<WindowSample>,
    audit_ok: bool,
    min_nodes: usize,
}

fn load(dir: &Path) -> Run {
    let leak = std::fs::read_to_string(dir.join("leakage.csv")).expect("leakage.csv");
    let lrows: Vec<Vec<String>> = leak
        .lines()
        .skip(2)
        .map(|l| l.split(',').map(str::to_string).collect())
        .collect();
    let met = std::fs::read_to_string(dir.join("metrics.csv")).expect("metrics.csv");
    let mrows: Vec<Vec<String>> = met
        .lines()
        .skip(2)
        .map(|l| l.split(',').map(str::to_string).collect())
        .collect();
    assert_eq!(
        lrows.len(),
        mrows.len(),
        "{}: leakage and metrics must be window-aligned",
        dir.display()
    );

    let audit_ok = lrows.iter().all(|r| r[18].parse::<f64>().unwrap_or(1.0) == 0.0);
    let nsub: Vec<f64> = lrows.iter().map(|r| r[15].parse().unwrap_or(0.0)).collect();
    let sampled_census: Vec<f64> = lrows.iter().map(|r| r[10].parse().unwrap_or(0.0)).collect();
    let sparse: Vec<bool> = lrows.iter().map(|r| r[12] == "sparse").collect();
    let n_nodes: Vec<usize> = mrows.iter().map(|r| r[3].parse().unwrap_or(0)).collect();
    let min_nodes = n_nodes.iter().copied().min().unwrap_or(0);
    let census: Vec<f64> = n_nodes
        .iter()
        .map(|&n| n.saturating_sub(LATTICE_N) as f64)
        .collect();
    let producing: Vec<bool> = (0..lrows.len())
        .map(|i| i > 0 && nsub[i] > nsub[i - 1])
        .collect();

    let samples: Vec<WindowSample> = std::fs::read_to_string(dir.join("closure_samples.json"))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    assert_eq!(
        samples.len(),
        lrows.len(),
        "{}: closure samples must align 1:1 with leakage rows",
        dir.display()
    );

    Run {
        name: dir.file_name().unwrap().to_string_lossy().into_owned(),
        census,
        sampled_census,
        producing,
        sparse,
        samples,
        audit_ok,
        min_nodes,
    }
}

/// Longest consecutive run of windows satisfying the mask.
fn longest_stratum(mask: &[bool]) -> Vec<usize> {
    let (mut best, mut cur): (Vec<usize>, Vec<usize>) = (vec![], vec![]);
    for (i, &g) in mask.iter().enumerate() {
        if g {
            cur.push(i);
            if cur.len() > best.len() {
                best = cur.clone();
            }
        } else {
            cur.clear();
        }
    }
    best
}

fn mean(v: &[f64]) -> f64 {
    v.iter().sum::<f64>() / v.len().max(1) as f64
}
fn var(v: &[f64]) -> f64 {
    let m = mean(v);
    v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / v.len().max(1) as f64
}

/// P2, pre-registered and unchanged since v7: mean drift < 25% and
/// variance ratio within 2×.
fn stationary(v: &[f64]) -> (bool, f64, f64) {
    let h = v.len() / 2;
    let (a, b) = (&v[..h], &v[h..]);
    let overall = mean(v);
    let drift = if overall > 0.0 {
        (mean(a) - mean(b)).abs() / overall
    } else {
        f64::INFINITY
    };
    let (va, vb) = (var(a), var(b));
    let ratio = if vb > 0.0 { va / vb } else { f64::INFINITY };
    (drift < 0.25 && (0.5..=2.0).contains(&ratio), drift, ratio)
}

/// The A3-restated floor: effective, not raw, sample size.
fn effective_n(samples: &[WindowSample], stratum: &[usize]) -> (f64, f64) {
    if stratum.len() < 5 {
        return (f64::NAN, 0.0);
    }
    let dims = samples[stratum[0]].micro_v.len();
    let mut rhos = Vec::new();
    for j in 0..dims {
        let x: Vec<f64> = stratum.iter().map(|&i| samples[i].micro_v[j]).collect();
        let m = mean(&x);
        let v: f64 = x.iter().map(|y| (y - m).powi(2)).sum();
        if v < 1e-12 {
            continue;
        }
        let cov: f64 = (1..x.len()).map(|t| (x[t] - m) * (x[t - 1] - m)).sum();
        rhos.push(cov / v);
    }
    if rhos.is_empty() {
        return (f64::NAN, 0.0);
    }
    let rho = mean(&rhos);
    let n = stratum.len() as f64;
    (rho, n * (1.0 - rho) / (1.0 + rho))
}

fn run_dirs() -> Vec<PathBuf> {
    let root = std::env::var("C1_RUNS").unwrap_or_else(|_| {
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../runs/v8_l7r_passive").into()
    });
    let mut v: Vec<PathBuf> = std::fs::read_dir(&root)
        .unwrap_or_else(|e| panic!("{root}: {e}"))
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    v.sort();
    v
}

/// §3.1 — the re-gate. Reports the corrected mask against attempt 2's,
/// and evaluates the pre-registered prediction and its exclusion.
#[test]
#[ignore]
fn c1_31_corrected_mask_regate() {
    println!(
        "§3.1 corrected-mask re-gate. Stratum census = n_nodes − {LATTICE_N} \
         (gen>0, exact) instead of the sampled wormhole count.\n"
    );
    println!(
        "run | stratum old→new | steps covered (new) | drift | var ratio | P2 | ρ | eff N"
    );
    let mut verdicts = Vec::new();
    for d in run_dirs() {
        let r = load(&d);
        assert!(r.audit_ok, "{}: audit drift halts interpretation", r.name);
        assert!(
            r.min_nodes >= LATTICE_N,
            "{}: n_nodes dipped to {} below the {LATTICE_N}-node lattice — the \
             census identity census = n_nodes − {LATTICE_N} would be invalid",
            r.name,
            r.min_nodes
        );

        let old_mask: Vec<bool> = (0..r.census.len())
            .map(|i| r.producing[i] && r.sparse[i] && r.sampled_census[i] > 0.0)
            .collect();
        let new_mask: Vec<bool> = (0..r.census.len())
            .map(|i| r.producing[i] && r.sparse[i] && r.census[i] > 0.0)
            .collect();
        let old = longest_stratum(&old_mask);
        let new = longest_stratum(&new_mask);
        let v: Vec<f64> = new.iter().map(|&i| r.census[i]).collect();
        let (p2, drift, ratio) = stationary(&v);
        let (rho, eff) = effective_n(&r.samples, &new);
        println!(
            "{} | {} → {} | {}..{} | {drift:.3} | {ratio:.2} | {} | {rho:.3} | {eff:.0}",
            r.name,
            old.len(),
            new.len(),
            new.first().map(|&i| (i + 1) * 100).unwrap_or(0),
            new.last().map(|&i| (i + 1) * 100).unwrap_or(0),
            if p2 { "PASS" } else { "FAIL" },
        );
        verdicts.push((r.name.clone(), p2, eff, new.len(), v));
    }

    // Quartile diagnosis — the shape the prediction is about.
    println!("\nquartile means / sds on the corrected stratum:");
    for (name, _, _, _, v) in &verdicts {
        let q = v.len() / 4;
        let cells: Vec<String> = (0..4)
            .map(|j| {
                let seg = &v[j * q..((j + 1) * q).min(v.len())];
                format!("{:.1}±{:.1}", mean(seg), var(seg).sqrt())
            })
            .collect();
        println!("  {name}: {}", cells.join(" | "));
    }

    let passing = verdicts.iter().filter(|(_, p2, _, _, _)| *p2).count();
    println!(
        "\nPRE-REGISTERED PREDICTION (v8): quartile-4 inflation disappears and the \
         variance ratio enters [0.5, 2.0] on the full stratum.\n\
         RESULT: {passing}/{} runs pass P2 on the corrected mask.\n\
         PRE-REGISTERED EXCLUSION: if it still fails, the destabilization is REAL, \
         the stationarity milestone shrinks to quartiles 1–3, and attempt 3 runs on \
         the Q1–Q3 sub-stratum only if effective N is independently re-cleared there.",
        verdicts.len()
    );
    if passing == 0 {
        println!("\nEXCLUSION BRANCH TAKEN — Q1–Q3 sub-stratum effective N:");
        for d in run_dirs() {
            let r = load(&d);
            let new_mask: Vec<bool> = (0..r.census.len())
                .map(|i| r.producing[i] && r.sparse[i] && r.census[i] > 0.0)
                .collect();
            let new = longest_stratum(&new_mask);
            let q3 = &new[..new.len() * 3 / 4];
            let (rho, eff) = effective_n(&r.samples, q3);
            let v: Vec<f64> = q3.iter().map(|&i| r.census[i]).collect();
            let (p2, drift, ratio) = stationary(&v);
            println!(
                "  {} | Q1–Q3 n={} | ρ={rho:.3} | eff N={eff:.0} ({}) | drift {drift:.3} \
                 var ratio {ratio:.2} ({})",
                r.name,
                q3.len(),
                if eff >= 200.0 { "clears" } else { "SHORT" },
                if p2 { "P2 pass" } else { "P2 fail" },
            );
        }
    }
}

/// Deterministic LCG — the bootstrap is the one place this track draws
/// RNG on purpose, so the draw is reproducible from `BOOT_SEED`.
struct Lcg(u64);
impl Lcg {
    fn next_usize(&mut self, n: usize) -> usize {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((self.0 >> 33) as usize) % n.max(1)
    }
}

/// One moving-block bootstrap replicate of CD_clock over `stratum`.
///
/// Blocks of `b` consecutive transitions are drawn with replacement and
/// concatenated, each followed by a **separator window carrying
/// `mask = false`**. Since `estimate_stratum` retains a transition only
/// when `mask[i] && mask[i+1]`, the separator deletes the join
/// transition, so a replicate contains exactly the genuine transitions of
/// its blocks and no spurious ones (C0 memo).
fn bootstrap_replicate(
    samples: &[WindowSample],
    stratum: &[usize],
    b: usize,
    rng: &mut Lcg,
    settings: &ClosureSettings,
) -> Option<f64> {
    let n = stratum.len();
    if n < b + 2 {
        return None;
    }
    let starts = n - b; // a block needs b+1 windows: [s, s+b]
    let n_blocks = n / b;
    let mut resampled: Vec<WindowSample> = Vec::with_capacity(n_blocks * (b + 2));
    let mut mask: Vec<bool> = Vec::with_capacity(n_blocks * (b + 2));
    for _ in 0..n_blocks {
        let s = rng.next_usize(starts);
        for j in 0..=b {
            resampled.push(samples[stratum[s + j]].clone());
            mask.push(true);
        }
        // Separator: present so indices stay contiguous, masked out so no
        // transition can straddle the join.
        resampled.push(WindowSample::default());
        mask.push(false);
    }
    let rep = estimate_stratum(&resampled, &mask, "bootstrap", settings);
    rep.cd_clock_bits
}

/// §3.2 — attempt 3. Gated on §3.1: this test asserts the gate verdict it
/// depends on rather than assuming it, and prints the branch it took.
#[test]
#[ignore]
fn c1_32_exp_l7r_attempt3() {
    // P3 (structural, before anything runs): no micro feature is a
    // property, clock, age or birth-step channel.
    for f in hg_causal::closure::features::MICRO_FIELDS {
        let l = f.to_lowercase();
        assert!(
            !l.contains("countdown")
                && !l.contains("propert")
                && !l.contains("age")
                && !l.contains("birth"),
            "P3: micro feature '{f}' must not carry a clock channel"
        );
    }

    let settings = ClosureSettings::default();
    println!(
        "EXP-L7R attempt 3 — single-trajectory CD, C0-frozen config.\n\
         bootstrap: moving-block, b ∈ {BOOT_BLOCKS:?} (headline {BOOT_HEADLINE}), \
         B = {BOOT_REPLICATES}, percentile 2.5/97.5, seed {BOOT_SEED:#x}\n"
    );

    let mut clocks: Vec<f64> = Vec::new();
    let mut rows: Vec<String> = Vec::new();
    let mut headline: Option<(f64, f64, f64)> = None; // (cd, lo, hi) of the first run
    let mut authorized = 0usize;

    for d in run_dirs() {
        let r = load(&d);
        assert!(r.audit_ok, "{}: audit drift halts interpretation", r.name);
        let mask: Vec<bool> = (0..r.census.len())
            .map(|i| r.producing[i] && r.sparse[i] && r.census[i] > 0.0)
            .collect();
        let stratum = longest_stratum(&mask);
        let v: Vec<f64> = stratum.iter().map(|&i| r.census[i]).collect();
        let (p2, drift, ratio) = stationary(&v);
        let (rho, eff) = effective_n(&r.samples, &stratum);
        let p1 = eff >= 200.0;

        if !(p1 && p2) {
            rows.push(format!(
                "{} | n={} | ρ={rho:.3} eff={eff:.0} ({}) | drift {drift:.3} ratio {ratio:.2} ({}) \
                 | NOT AUTHORIZED",
                r.name,
                stratum.len(),
                if p1 { "P1 pass" } else { "P1 FAIL" },
                if p2 { "P2 pass" } else { "P2 FAIL" },
            ));
            continue;
        }
        authorized += 1;

        // Point estimate on the stratum, in the original window order.
        let mut point_mask = vec![false; r.samples.len()];
        for &i in &stratum {
            point_mask[i] = true;
        }
        let rep = estimate_stratum(&r.samples, &point_mask, "coexistence_steady", &settings);
        let cd = rep.cd_clock_bits.unwrap_or(f64::NAN);
        if cd.is_finite() {
            clocks.push(cd);
        }

        // Moving-block bootstrap at all three pre-registered block lengths.
        let mut cis = Vec::new();
        for &b in &BOOT_BLOCKS {
            let mut rng = Lcg(BOOT_SEED ^ (b as u64));
            let mut draws: Vec<f64> = Vec::with_capacity(BOOT_REPLICATES);
            for _ in 0..BOOT_REPLICATES {
                if let Some(x) = bootstrap_replicate(&r.samples, &stratum, b, &mut rng, &settings) {
                    if x.is_finite() {
                        draws.push(x);
                    }
                }
            }
            draws.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let (lo, hi) = if draws.len() >= 20 {
                (
                    draws[(draws.len() as f64 * 0.025) as usize],
                    draws[((draws.len() as f64 * 0.975) as usize).min(draws.len() - 1)],
                )
            } else {
                (f64::NAN, f64::NAN)
            };
            if b == BOOT_HEADLINE && headline.is_none() {
                headline = Some((cd, lo, hi));
            }
            cis.push(format!("b={b}: [{lo:.4}, {hi:.4}] (n={})", draws.len()));
        }
        rows.push(format!(
            "{} | n={} | ρ={rho:.3} eff={eff:.0} | drift {drift:.3} ratio {ratio:.2} | \
             CD_raw {:.4} CD_clock {cd:.4} | {}",
            r.name,
            stratum.len(),
            rep.cd_raw_bits,
            cis.join("  "),
        ));
    }

    for l in &rows {
        println!("{l}");
    }

    let sampler_spread = if clocks.len() >= 2 {
        clocks.iter().copied().fold(f64::MIN, f64::max)
            - clocks.iter().copied().fold(f64::MAX, f64::min)
    } else {
        f64::NAN
    };
    println!(
        "\nauthorized runs: {authorized}/5 | sampler_spread (max−min of CD_clock, \
         ENSEMBLE = sampler noise, dynamical component identically zero) = {sampler_spread:.4}"
    );
    match headline {
        Some((cd, lo, hi)) if authorized > 0 => {
            let excludes_zero = lo > 0.0 || hi < 0.0;
            let beats_spread = cd.abs() > sampler_spread;
            println!(
                "HEADLINE (b={BOOT_HEADLINE}): CD_clock = {cd:.4} bits/window, \
                 95% block-bootstrap CI [{lo:.4}, {hi:.4}]"
            );
            println!(
                "DECISION RULE (C0, both required): CI excludes 0 = {excludes_zero}; \
                 |CD| > sampler_spread = {beats_spread} ⇒ {}",
                if excludes_zero && beats_spread {
                    "CLAIM AUTHORIZED — this trajectory's closure defect"
                } else {
                    "NO CLAIM — the fourth quantified obstacle (estimator variance floor)"
                }
            );
        }
        _ => println!("HEADLINE: not computed — no run cleared §3.1's gate."),
    }
    println!(
        "\nCLAIM LANGUAGE (C0): any number above is THIS TRAJECTORY's closure defect. \
         The family-level claim is out of scope and requires the deferred IC ensemble."
    );
}
