//! Regression for the growth_treadmill-classifies-as-Monoculture bug:
//! the "invents nothing" regimes tolerate the WL hub-degree trickle
//! (rate <= new_type_bin_rate_min = 0.01), they do NOT require exactly
//! zero new bins. The boundary must hold in both directions.

use hg_eval::classify;
use hg_eval::WindowFeatures;
use hg_schema::Regime;

/// A late growth_treadmill window as measured from the real run: strong
/// monotonic growth, high dominance, near-zero JSD, and the constant
/// ~1-bin-per-window trickle from the growing hubs' WL classes.
fn treadmill_window(new_type_bin_rate: f64) -> WindowFeatures {
    WindowFeatures {
        has_prev: true,
        n_nodes: 15_000,
        n_edges: 15_010,
        growth_ratio: 1.034,
        activity: 0.065,
        type_histogram_jsd: 0.0006,
        type_histogram_entropy: 0.08,
        dominant_type_fraction: 0.996,
        mean_degree: Some(2.0),
        new_type_bin_rate,
        ..Default::default()
    }
}

#[test]
fn hub_trickle_is_still_a_treadmill() {
    // 1 fresh WL bin per 128-step window (one growing hub): rate 0.0078.
    // Strict-zero misclassified this as Monoculture.
    assert_eq!(classify(&treadmill_window(0.0078)), Regime::GrowthTreadmill);
    // Exactly zero, the fixture case, still works.
    assert_eq!(classify(&treadmill_window(0.0)), Regime::GrowthTreadmill);
    // At the boundary itself: <= is inclusive.
    assert_eq!(classify(&treadmill_window(0.01)), Regime::GrowthTreadmill);
}

#[test]
fn real_invention_is_not_a_treadmill() {
    // Above the novelty boundary the treadmill check must NOT swallow a
    // genuinely inventive grower (the SlowGenerativeGrowth direction).
    let f = WindowFeatures {
        new_type_bin_rate: 0.05,
        dominant_type_fraction: 0.5, // diverse, so Monoculture can't win
        ..treadmill_window(0.05)
    };
    assert_eq!(classify(&f), Regime::GenerativeCandidate);
}
