//! LK1 LocalityObserver tests: purity (standing rule: observers never
//! mutate), fixture parity with the LK0 offline instruments, and
//! determinism under a fixed seed.

use hg_eval::fixtures::locality::{expander, lattice_2d_tri};
use hg_eval::geometry::{
    dimension_from_profiles, sample_sources, DEFF_MIN_FIT_POINTS, DEFF_R_MAX,
    DEFF_SATURATION_FRAC,
};
use hg_observe::balls::ball_profiles;
use hg_observe::locality::{
    compute_leakage, cut_fit_from_profiles, detour_census, LeakageSettings, LocalityObserver,
};

fn settings() -> LeakageSettings {
    LeakageSettings {
        cadence_steps: 1,
        detour_sample_edges: 256,
        detour_r_max: 12,
        ball_sources: 32,
    }
}

#[test]
fn observer_never_mutates() {
    let store = lattice_2d_tri(16);
    let before = store.canonical_hash();
    let mut obs = LocalityObserver::new(settings(), 42);
    obs.observe_window(&store.snapshot(), 500, Some(2.0));
    obs.observe_window(&store.snapshot(), 1000, None);
    assert_eq!(store.canonical_hash(), before, "observer mutated the store");
    assert_eq!(obs.rows.len(), 2);
}

#[test]
fn fixture_parity_lattice() {
    // Live observer values == LK0 offline instrument values on the same
    // fixture with the same seeds (identical, not merely close: both
    // paths are the same deterministic code over the same samples).
    let store = lattice_2d_tri(32);
    let snapshot = store.snapshot();
    let mut obs = LocalityObserver::new(settings(), 42);
    let step = 500u64;
    obs.observe_window(&snapshot, step, None);
    let row = &obs.rows[0];

    let seed = obs.seed_for(step);
    let offline = detour_census(&snapshot, seed ^ 0x00DE_7011, 256, 12);
    assert_eq!(row.detour_p50, offline.p50);
    assert_eq!(row.detour_p95, offline.p95);
    assert_eq!(row.wormhole_frac, offline.wormhole_frac);
    assert_eq!(row.detour_p50, 2.0, "triangulated lattice detour");
    assert_eq!(row.wormhole_frac, 0.0);

    let sources = sample_sources(&snapshot, seed, 32);
    let profiles = ball_profiles(&snapshot, &sources, DEFF_R_MAX);
    let cut = cut_fit_from_profiles(
        &profiles,
        snapshot.node_count(),
        DEFF_R_MAX,
        DEFF_SATURATION_FRAC,
        DEFF_MIN_FIT_POINTS,
    )
    .expect("cut fit");
    assert_eq!(row.cut_exponent, Some(cut.0 as f32));
    assert_eq!(row.cut_fit_r2, Some(cut.1 as f32));
    assert!((cut.0 - 0.5).abs() < 0.1, "lattice cut exponent {}", cut.0);
}

#[test]
fn fixture_parity_expander() {
    let store = expander(16384, 3, 42);
    let snapshot = store.snapshot();
    let mut obs = LocalityObserver::new(settings(), 42);
    obs.observe_window(&snapshot, 500, None);
    let row = &obs.rows[0];
    assert!(row.wormhole_frac > 0.5, "expander wormholes: {}", row.wormhole_frac);
    assert!(row.cut_exponent.unwrap() > 0.9, "expander cut: {:?}", row.cut_exponent);
}

#[test]
fn deterministic_rows_under_fixed_seed() {
    let store = expander(2048, 4, 7);
    let snapshot = store.snapshot();
    let mut a = LocalityObserver::new(settings(), 99);
    let mut b = LocalityObserver::new(settings(), 99);
    for step in [500u64, 1000, 1500] {
        a.observe_window(&snapshot, step, Some(2.5));
        b.observe_window(&snapshot, step, Some(2.5));
    }
    assert_eq!(a.rows, b.rows);
    assert_eq!(a.histograms, b.histograms);

    let mut c = LocalityObserver::new(settings(), 100);
    c.observe_window(&snapshot, 500, Some(2.5));
    // A different run seed samples different edges/sources; on an
    // irregular graph the census will generally differ (guards against
    // an ignored seed).
    assert!(
        c.rows[0] != a.rows[0] || c.histograms[0] != a.histograms[0],
        "seed appears to be ignored by the sampler"
    );
}

#[test]
fn area_law_defect_requires_same_window_d_eff() {
    let store = lattice_2d_tri(24);
    let snapshot = store.snapshot();
    let seed = 7u64;
    let sources = sample_sources(&snapshot, seed, 32);
    let profiles = ball_profiles(&snapshot, &sources, DEFF_R_MAX);
    let d_eff = dimension_from_profiles(&profiles, snapshot.node_count());
    assert!(d_eff.is_some());

    let with = compute_leakage(
        &snapshot, 1, seed, &settings(), &profiles, DEFF_R_MAX, DEFF_SATURATION_FRAC,
        DEFF_MIN_FIT_POINTS, d_eff,
    );
    assert!(with.row.area_law_defect.is_some());
    assert!(with.row.d_bfs.is_some());

    let without = compute_leakage(
        &snapshot, 1, seed, &settings(), &profiles, DEFF_R_MAX, DEFF_SATURATION_FRAC,
        DEFF_MIN_FIT_POINTS, None,
    );
    assert_eq!(without.row.area_law_defect, None, "no stale d_eff ever");
    assert_eq!(without.row.d_bfs, None);
}
