//! Regime classification and interestingness (docs/metric_formulas.md).
//! First match wins, evaluated top to bottom; window 0 is always Unknown.

use crate::thresholds::*;
use crate::window::WindowFeatures;
use hg_schema::Regime;

pub fn classify(f: &WindowFeatures) -> Regime {
    if !f.has_prev {
        return Regime::Unknown;
    }

    // 1. Exploding
    if f.growth_ratio >= GROWTH_EXPLODE_MIN || f.n_nodes > NODE_CAP {
        return Regime::Exploding;
    }
    // 2. Frozen — unchanging lattice with boundary-induced class
    // diversity. A mover disqualifies it: a pure-relabel glider has zero
    // node churn AND a symmetric ring keeps the window histogram
    // identical (jsd = 0), yet its mark class persists with CHANGING
    // members — active, not frozen.
    if f.activity == 0.0
        && f.type_histogram_jsd == 0.0
        && f.type_histogram_entropy > 0.0
        && f.active_persistent_structures == 0
    {
        return Regime::Frozen;
    }
    // 3. StaticOrdered — a single unchanging equivalence class.
    if f.activity == 0.0
        && f.type_histogram_jsd == 0.0
        && f.active_persistent_structures == 0
    {
        return Regime::StaticOrdered;
    }
    // 4. WhiteNoiseSoup — distribution churns and nothing persists.
    if f.type_histogram_jsd >= JSD_NOISE_MIN
        && f.pi_normalized.unwrap_or(1.0) < PI_NOISE_MAX
    {
        return Regime::WhiteNoiseSoup;
    }
    // 5. SterileChaos — churns hard but invents nothing ("nothing" up
    // to the novelty boundary; see NEW_TYPE_BIN_RATE_MIN).
    if f.activity >= ACTIVITY_CHAOS_MIN
        && f.type_histogram_jsd >= JSD_NOISE_MIN
        && f.new_type_bin_rate <= NEW_TYPE_BIN_RATE_MIN
    {
        return Regime::SterileChaos;
    }
    // 6. FragmentedIsolation
    if f.mean_degree.unwrap_or(f64::MAX) < MEAN_DEGREE_FRAG_MAX {
        return Regime::FragmentedIsolation;
    }
    // 7. GrowthTreadmill — grows, invents nothing. Checked before
    // Monoculture: a growing single-type system is the treadmill
    // archetype (UniformGrowthToy), not a monoculture that happens to grow.
    if f.growth_ratio >= GROWTH_MIN && f.new_type_bin_rate <= NEW_TYPE_BIN_RATE_MIN {
        return Regime::GrowthTreadmill;
    }
    // 8. Monoculture
    if f.dominant_type_fraction >= DOMINANT_MONOCULTURE_MIN && f.activity > 0.0 {
        return Regime::Monoculture;
    }
    // 9. ActivePersistentStructure — immediately before GenerativeCandidate
    // (v4 §7): it moves, but invents nothing new.
    if f.active_persistent_structures >= 1 && f.new_type_bin_rate <= NEW_TYPE_BIN_RATE_MIN {
        return Regime::ActivePersistentStructure;
    }
    // 10. GenerativeCandidate — novelty carried by a mover or by growth.
    if f.new_type_bin_rate > NEW_TYPE_BIN_RATE_MIN
        && (f.active_persistent_structures >= 1 || f.growth_ratio >= GROWTH_MIN)
    {
        return Regime::GenerativeCandidate;
    }
    // 11.
    Regime::Unknown
}

/// Interestingness: weighted sum with penalties, clamped to [0,1]
/// (docs/metric_formulas.md).
pub fn interestingness(f: &WindowFeatures) -> f64 {
    let diversity = if f.n_type_bins >= 2 {
        f.type_histogram_entropy / (f.n_type_bins.max(2) as f64).log2()
    } else {
        0.0
    };
    let novelty = ((f.new_type_bins as f64) / f.n_type_bins.max(1) as f64).min(1.0);
    let persistence = f.pi_normalized.unwrap_or(0.0);

    let mut raw = 0.30 * persistence + 0.40 * novelty + 0.30 * diversity;
    if f.growth_ratio >= GROWTH_EXPLODE_MIN {
        raw -= 0.50;
    }
    if f.dominant_type_fraction >= DOMINANT_MONOCULTURE_MIN {
        raw -= 0.30;
    }
    if f.pi_normalized.unwrap_or(0.0) < PI_NOISE_MAX && f.has_prev {
        raw -= 0.40; // noise penalty: nothing persists
    }
    raw.clamp(0.0, 1.0)
}
