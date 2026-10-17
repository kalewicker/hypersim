//! LK0 fixture-calibration suite (EXP-L1, the gate for the whole leakage
//! track). Static graphs with known answers; the pre-committed
//! interpretation of failure is "the instruments are not trustworthy" —
//! nothing downstream runs until these pass.

use hg_eval::fixtures::locality::*;
use hg_eval::geometry::{
    dimension_from_profiles, sample_sources, DEFF_MIN_FIT_POINTS, DEFF_R_MAX,
    DEFF_SATURATION_FRAC, DEFF_SOURCES,
};
use hg_core::HypergraphStore;
use hg_observe::balls::ball_profiles;
use hg_observe::locality::{cut_fit_from_profiles, detour_census, DetourCensus};

const SEED: u64 = 42;
const R_MAX_DETOUR: usize = 12;

fn census(store: &HypergraphStore, k_e: usize) -> DetourCensus {
    detour_census(&store.snapshot(), SEED, k_e, R_MAX_DETOUR)
}

/// cut_exponent, cut_fit_r2, d_eff, area_law_defect with the shared-BFS
/// protocol (exactly what the LocalityObserver computes live).
fn cut_and_dim(store: &HypergraphStore) -> (Option<(f64, f64)>, Option<f64>, Option<f64>) {
    let snapshot = store.snapshot();
    let sources = sample_sources(&snapshot, SEED, DEFF_SOURCES);
    let profiles = ball_profiles(&snapshot, &sources, DEFF_R_MAX);
    let cut = cut_fit_from_profiles(
        &profiles,
        snapshot.node_count(),
        DEFF_R_MAX,
        DEFF_SATURATION_FRAC,
        DEFF_MIN_FIT_POINTS,
    );
    let d_eff = dimension_from_profiles(&profiles, snapshot.node_count());
    let defect = match (cut, d_eff) {
        (Some((ce, _)), Some(d)) if d > 0.0 => Some(ce - (d - 1.0) / d),
        _ => None,
    };
    (cut, d_eff, defect)
}

#[test]
fn detour_lattice_2d_is_2() {
    let store = lattice_2d_tri(16);
    // Full census: every edge of the triangulated torus detours through
    // its shared triangle corner.
    let c = census(&store, store.edge_count());
    assert_eq!(c.p50, 2.0, "lattice detour p50");
    assert_eq!(c.p95, 2.0, "lattice detour p95");
    assert_eq!(c.wormhole_frac, 0.0);
    assert!(c.detours.iter().all(|&d| d == 2.0), "every edge detour = 2");
}

#[test]
fn cut_exponent_lattice_2d_half() {
    let store = lattice_2d_tri(48);
    let (cut, _, _) = cut_and_dim(&store);
    let (exp, r2) = cut.expect("fit must succeed on a clean lattice");
    assert!((exp - 0.5).abs() < 0.1, "cut_exponent {exp} not within 0.5 ± 0.1");
    assert!(r2 > 0.95, "cut_fit_r2 {r2} <= 0.95");
}

#[test]
fn cut_exponent_lattice_3d_two_thirds() {
    let store = lattice_3d(12);
    let (cut, _, _) = cut_and_dim(&store);
    let (exp, r2) = cut.expect("fit must succeed on a clean 3d lattice");
    assert!((exp - 0.67).abs() < 0.1, "cut_exponent {exp} not within 0.67 ± 0.1");
    assert!(r2 > 0.9, "cut_fit_r2 {r2} <= 0.9");
}

#[test]
fn expander_flagged_leaky() {
    // Calibrated instantiation (docs/fixture_calibration.md): detour of a
    // d-regular expander is ≈ log_{d−1}(n); exceeding detour_r_max = 12
    // needs log₂(16384) = 14 at d = 3 (d = 4 would need n > 3^12).
    let store = expander_checked(32768, 3, R_MAX_DETOUR, SEED);
    let c = census(&store, 256);
    assert!(
        c.wormhole_frac > 0.5,
        "expander wormhole_frac {} <= 0.5",
        c.wormhole_frac
    );
    let (cut, _, _) = cut_and_dim(&store);
    let (exp, _) = cut.expect("cut fit on expander");
    assert!(exp > 0.9, "expander cut_exponent {exp} <= 0.9 (volume scaling expected)");
}

#[test]
fn small_world_wormhole_unimodal_with_recorded_p_peak() {
    // v2 §3.3 (F-LK1 formalized): wormhole_frac(p) is UNIMODAL — it
    // rises through the sparse phase, peaks at p_peak, and collapses
    // past the small-world crossover where shortcuts become mutually
    // redundant. Recorded: p_peak(48², cap 12) ≈ 5e-3 (bracket
    // [2e-3, 1.5e-2]); mean curve over seeds {42, 1001, 1002}:
    // 1e-3→0.00067, 2e-3→0.00135, 5e-3→0.00302, 1e-2→0.00258,
    // 1.5e-2→0.00124, 3e-2→0.000 (docs/fixture_calibration.md).
    let mean_wf = |p: f64| -> f32 {
        let mut acc = 0.0f32;
        for seed in [42u64, 1001, 1002] {
            let store = small_world_checked(48, p, R_MAX_DETOUR, seed);
            let c = detour_census(&store.snapshot(), 7, store.edge_count(), R_MAX_DETOUR);
            acc += c.wormhole_frac;
        }
        acc / 3.0
    };
    let rise = [1e-3, 2e-3, 5e-3].map(mean_wf);
    let fall = [5e-3, 1.5e-2, 3e-2].map(mean_wf);
    println!("rise: {rise:?}, fall: {fall:?}");
    assert!(rise[0] < rise[1] && rise[1] < rise[2], "sparse arm must rise: {rise:?}");
    assert!(fall[0] > fall[1] && fall[1] > fall[2], "dense arm must fall: {fall:?}");
    assert!(fall[2] < 5e-4, "past the crossover the channel reads ~0: {}", fall[2]);
}

#[test]
fn small_world_cut_channel_monotone_or_demoted() {
    // v2 §1.2 pre-commitment: LK0R must check whether area_law_defect is
    // the dense-phase monotone. VERDICT (recorded): it is NOT — the cut
    // channel rises through the sparse phase then falls across the
    // crossover (0.093 → 0.134 → 0.176 → 0.074 at p = 0/1e-3/1e-2/5e-2,
    // seed 42), so it is DEMOTED to phase-plane coordinate only. The
    // pre-committed promotion of dim_mismatch to dense-channel scalar is
    // BLOCKED on the reserved spectral-dimension observer; until then the
    // phase plane is the only dense-phase readout
    // (docs/fixture_calibration.md).
    let defect = |p: f64| -> f64 {
        let store = small_world_checked(48, p, R_MAX_DETOUR, SEED);
        cut_and_dim(&store).2.expect("defect")
    };
    let d = [0.0, 1e-3, 1e-2, 5e-2].map(defect);
    println!("area_law_defect by p: {d:?}");
    assert!(d[0] < d[1] && d[1] < d[2], "sparse-regime rise: {d:?}");
    assert!(
        d[3] < d[2],
        "crossover fall — if this ever monotonizes, the demotion verdict \
         must be revisited (plan v2 §1.2): {d:?}"
    );
}

#[test]
#[should_panic(expected = "NOT wormhole-registrable")]
fn expander_precondition_fails_loudly_when_underranged() {
    // F-LK3: the v1 calibration fixture (16384, 3) is UNDER-RANGED at
    // cap 12 by the v2 formula (needs n > 4·3·2^11 = 24576) — its
    // measured wormhole_frac was 0.777, not ~1. The checked constructor
    // turns that silent under-ranging into a construction-time error.
    let _ = expander_checked(16384, 3, R_MAX_DETOUR, SEED);
}

#[test]
fn p_peak_shifts_with_r_max() {
    // v2 §3.2 validation: p_peak must move OPPOSITE to R_max (a smaller
    // cap needs more shortcuts before mutual redundancy reroutes within
    // it). Measured: p_peak(cap 8) ≈ 3e-2 vs p_peak(cap 12) ≈ 5e-3. A
    // p_peak that does not move with R_max falsifies the crossover
    // reading and reopens F-LK1.
    let mean_wf = |p: f64, cap: usize| -> f32 {
        let mut acc = 0.0f32;
        for seed in [42u64, 1001, 1002] {
            let store = small_world(48, p, seed);
            let c = detour_census(&store.snapshot(), 7, store.edge_count(), cap);
            acc += c.wormhole_frac;
        }
        acc / 3.0
    };
    // cap 12: past-peak at 3e-2; cap 8: still rising toward 3e-2.
    assert!(mean_wf(5e-3, 12) > mean_wf(3e-2, 12), "cap 12 peaks early");
    assert!(mean_wf(3e-2, 8) > mean_wf(5e-3, 8), "cap 8 peaks late");
}

#[test]
fn monotone_ranking_including_expander() {
    // Exit criterion §3: lattice < small_world(p1) < small_world(p2)
    // < expander on wormhole_frac and area_law_defect. p1 = 1e-3,
    // p2 = 1e-2 (the sparse regime; see small_world_monotone_in_p for
    // why 5e-2 is past the small-world transition).
    let lattice = small_world(48, 0.0, SEED);
    let sw1 = small_world(48, 1e-3, SEED);
    let sw2 = small_world(48, 1e-2, SEED);
    let exp = expander_checked(32768, 3, R_MAX_DETOUR, SEED);

    let w: Vec<f32> = [&lattice, &sw1, &sw2]
        .iter()
        .map(|s| census(s, s.edge_count()).wormhole_frac)
        .collect();
    let w_exp = census(&exp, 256).wormhole_frac;
    assert!(w[0] < w[1] && w[1] < w[2] && w[2] < w_exp,
        "wormhole ranking: {w:?} vs expander {w_exp}");

    let d: Vec<f64> = [&lattice, &sw1, &sw2, &exp]
        .iter()
        .map(|s| cut_and_dim(s).2.expect("defect"))
        .collect();
    println!("area_law_defect ranking: {d:?}");
    assert!(d[0] < d[1] && d[1] < d[2] && d[2] < d[3],
        "area_law_defect ranking violated: {d:?}");
}

#[test]
fn ring_degenerate_documented() {
    // Small ring, cap above n: the exact known value detour = n − 1.
    let store = ring(10);
    let c = census(&store, store.edge_count());
    assert!(c.detours.iter().all(|&d| d == 9.0), "ring(10) detour = n−1 = 9");

    // Production-sized ring under the default cap: every edge reads as a
    // wormhole (the DOCUMENTED degeneracy) while the cut instrument says
    // "not volume-scaling" — the two are interpreted jointly, never singly.
    let store = ring(200);
    let c = census(&store, store.edge_count());
    assert_eq!(c.wormhole_frac, 1.0, "ring under cap: all wormholes (documented)");
    let (cut, _, _) = cut_and_dim(&store);
    let (exp, _) = cut.expect("cut fit on ring");
    assert!(exp.abs() < 0.05, "ring cut_exponent {exp} should be ≈ 0 (C(r) = 2)");
}

#[test]
fn tree_nongeometric() {
    // Calibrated instantiation: branching 3. A BINARY tree asymptotes
    // at cut_exponent ≈ 0.82 under leaf-heavy source sampling (measured
    // 0.823 at depth 12 AND depth 14) — still far above every clean
    // lattice, but the "→ 1" claim is cleanest at b >= 3; recorded in
    // docs/fixture_calibration.md.
    let store = tree(3, 9);
    let (cut, _, _) = cut_and_dim(&store);
    let (exp, _) = cut.expect("cut fit on tree");
    assert!(exp > 0.9, "tree cut_exponent {exp} should approach 1 (volume scaling)");

    let binary = tree(2, 12);
    let (cut2, _, _) = cut_and_dim(&binary);
    let (exp2, _) = cut2.expect("cut fit on binary tree");
    assert!(
        exp2 > 0.75,
        "binary tree cut_exponent {exp2} still clearly non-geometric"
    );
}

#[test]
fn detour_sampler_deterministic_under_seed() {
    let store = small_world(32, 1e-2, 7);
    let a = detour_census(&store.snapshot(), 123, 256, R_MAX_DETOUR);
    let b = detour_census(&store.snapshot(), 123, 256, R_MAX_DETOUR);
    assert_eq!(a.detours, b.detours);
    assert_eq!(a.p50, b.p50);
    assert_eq!(a.p95, b.p95);
    assert_eq!(a.wormhole_frac, b.wormhole_frac);
    assert_eq!(a.histogram, b.histogram);
}

/// Calibration printout (not an assertion): the raw numbers recorded in
/// docs/fixture_calibration.md come from here.
#[test]
fn calibration_report() {
    for (name, store) in [
        ("lattice_2d_tri(48)", lattice_2d_tri(48)),
        ("lattice_3d(12)", lattice_3d(12)),
        ("small_world(48,1e-3)", small_world(48, 1e-3, SEED)),
        ("small_world(48,1e-2)", small_world(48, 1e-2, SEED)),
        ("small_world(48,5e-2)", small_world(48, 5e-2, SEED)),
        ("expander(16384,3)", expander(16384, 3, SEED)),
        ("ring(200)", ring(200)),
        ("tree(2,12)", tree(2, 12)),
    ] {
        let c = census(&store, 256);
        let (cut, d_eff, defect) = cut_and_dim(&store);
        println!(
            "{name}: n={} e={} p50={} p95={} wormhole={:.4} cut={:?} d_eff={:?} defect={:?}",
            store.node_count(),
            store.edge_count(),
            c.p50,
            c.p95,
            c.wormhole_frac,
            cut.map(|(a, b)| (format!("{a:.3}"), format!("{b:.3}"))),
            d_eff.map(|d| format!("{d:.3}")),
            defect.map(|d| format!("{d:.3}")),
        );
    }
}

/// Sensitivity calibration (EXP-L1 deliverable, run with --ignored):
/// smallest shortcut density p_min at which wormhole_frac separates from
/// the p = 0 baseline at 3σ across 10 seeds, per sample budget. Results
/// recorded in docs/fixture_calibration.md; later null results are
/// findings only relative to that recorded sensitivity.
#[test]
#[ignore]
fn sensitivity_calibration_p_min() {
    let n = 48;
    let ps = [5e-4, 1e-3, 2e-3, 5e-3, 1e-2];
    let budgets = [64usize, 256, 1024, usize::MAX];
    for &k_e in &budgets {
        let mut p_min: Option<f64> = None;
        for &p in &ps {
            let mut vals = Vec::new();
            for seed in 0..10u64 {
                let store = small_world(n, p, 1000 + seed);
                let k = k_e.min(store.edge_count());
                let c = detour_census(&store.snapshot(), 77 + seed, k, R_MAX_DETOUR);
                vals.push(c.wormhole_frac as f64);
            }
            let mean = vals.iter().sum::<f64>() / vals.len() as f64;
            let var = vals.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / vals.len() as f64;
            let sd = var.sqrt();
            // Baseline (p = 0) is exactly 0 with zero variance on the
            // clean lattice, so separation = mean > 3σ_p and mean > 0.
            let separated = mean > 0.0 && mean > 3.0 * sd;
            println!(
                "k_e={} p={:.0e}: mean={:.5} sd={:.5} separated={}",
                if k_e == usize::MAX { 0 } else { k_e }, p, mean, sd, separated
            );
            if separated && p_min.is_none() {
                p_min = Some(p);
            }
        }
        println!("k_e={:?} -> p_min={:?}", k_e, p_min);
    }
}

/// LK0R deliverable (v2 §3.2, run with --ignored): p_peak(n, R_max) with
/// per-p mean ± sd. Results recorded in docs/fixture_calibration.md and
/// consumed by the §1.3 phase gate / §1.5 quadrants.
#[test]
#[ignore]
fn p_peak_calibration_sweep() {
    let ps = [1e-3f64, 2e-3, 5e-3, 1e-2, 1.5e-2, 2e-2, 3e-2, 5e-2];
    for (width, n_seeds) in [(64usize, 10u64), (128, 10), (256, 3)] {
        for cap in [8usize, 12] {
            let mut curve: Vec<(f64, f64, f64)> = Vec::new();
            for &p in &ps {
                let mut vals = Vec::new();
                for seed in 0..n_seeds {
                    let store = small_world(width, p, 3000 + seed);
                    let c = detour_census(&store.snapshot(), 7, store.edge_count(), cap);
                    vals.push(c.wormhole_frac as f64);
                }
                let mean = vals.iter().sum::<f64>() / vals.len() as f64;
                let sd = (vals.iter().map(|v| (v - mean).powi(2)).sum::<f64>()
                    / vals.len() as f64)
                    .sqrt();
                curve.push((p, mean, sd));
            }
            let peak = curve
                .iter()
                .max_by(|a, b| a.1.total_cmp(&b.1))
                .map(|(p, _, _)| *p)
                .unwrap();
            println!("n={}² cap={cap} p_peak={peak:.4}", width);
            for (p, m, sd) in &curve {
                println!("  p={p:.4}: {m:.5} ± {sd:.5}");
            }
        }
    }
}
