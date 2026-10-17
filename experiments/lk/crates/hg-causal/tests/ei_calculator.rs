//! EI/CE calculator tests (hypersim_build_ei_calculator.md): the five
//! plan-required tests plus the determinism/degeneracy identity and the
//! single-class guard. Test 3's fixture values are hand-verified in the
//! build doc, so these assert ground truth, not the implementation.

use ahash::AHashMap;
use hg_causal::{
    coarse_grain, compute_ei_results, ei_from_tpm, ClassFeatures, ConstructorRegistry,
    EiSettings, TpmAccumulator,
};
use hg_core::WlHash;

const TOL: f64 = 1e-9;

/// Uniform TPM (every row = 1/M): H_int = log2(M) = H_noise → EI = 0.
#[test]
fn ei_white_noise_is_zero() {
    let rows: Vec<Vec<f64>> = (0..4).map(|_| vec![0.25; 4]).collect();
    let c = ei_from_tpm(&rows);
    assert!(c.ei.abs() < TOL, "white noise EI must be 0, got {}", c.ei);
    assert!((c.h_int - 2.0).abs() < TOL);
    assert!((c.h_noise - 2.0).abs() < TOL);
}

/// Identity TPM: H_noise = 0, H_int = log2(M) → EI = log2(M).
#[test]
fn ei_structured_tpm_is_positive() {
    let rows: Vec<Vec<f64>> = (0..4)
        .map(|i| {
            let mut r = vec![0.0; 4];
            r[i] = 1.0;
            r
        })
        .collect();
    let c = ei_from_tpm(&rows);
    assert!((c.ei - 2.0).abs() < TOL, "identity EI must be log2(4), got {}", c.ei);
    assert!(c.h_noise.abs() < TOL);
    assert!((c.determinism - 1.0).abs() < TOL);
    assert!(c.degeneracy.abs() < TOL);
}

/// Hand-verified 4-state degenerate TPM: states 1,2,3 each → 4 with
/// prob 1; state 4 → uniform over {1,2,3}. Macro A = {1,2,3}, B = {4}
/// gives [[0,1],[1,0]] → EI_macro = 1 bit and CE ≈ 1.2325 > 1.
#[test]
fn ce_coefficient_emergent() {
    let mut tpm = TpmAccumulator::new();
    for _ in 0..3 {
        tpm.record(WlHash(1), WlHash(4));
        tpm.record(WlHash(2), WlHash(4));
        tpm.record(WlHash(3), WlHash(4));
        tpm.record(WlHash(4), WlHash(1));
        tpm.record(WlHash(4), WlHash(2));
        tpm.record(WlHash(4), WlHash(3));
    }
    let rows = tpm.normalized_rows();
    let bare: Vec<Vec<f64>> = rows.iter().map(|(_, r)| r.clone()).collect();
    let micro = ei_from_tpm(&bare);
    assert!((micro.h_noise - 0.3962).abs() < 1e-3);
    assert!((micro.h_int - 1.2075).abs() < 1e-3);
    assert!((micro.ei - 0.8113).abs() < 1e-3);

    // Partition by dense class index: WlHash(4) alone in group 1.
    let group_of: Vec<usize> = tpm
        .hashes()
        .iter()
        .map(|h| if h.0 == 4 { 1 } else { 0 })
        .collect();
    let macro_rows = coarse_grain(&rows, &group_of, 2);
    let mac = ei_from_tpm(&macro_rows);
    assert!((mac.ei - 1.0).abs() < 1e-3, "macro EI must be 1 bit, got {}", mac.ei);

    let ce = mac.ei / micro.ei;
    assert!((ce - 1.2325).abs() < 1e-3, "CE must be ~1.2325, got {ce}");
    assert!(ce > 1.0, "this fixture IS causally emergent by construction");
}

/// 999 transitions → no result; 1000 → results present.
#[test]
fn ei_requires_minimum_samples() {
    let settings = EiSettings::default(); // min_transitions = 1000
    let features: AHashMap<WlHash, ClassFeatures> = AHashMap::new();
    let registry = ConstructorRegistry::new();
    let mut tpm = TpmAccumulator::new();
    for i in 0..999u64 {
        tpm.record(WlHash(i % 3), WlHash((i + 1) % 3));
    }
    assert!(
        compute_ei_results(&tpm, &features, &registry, &settings, 0).is_empty(),
        "999 transitions is below the gate"
    );
    tpm.record(WlHash(0), WlHash(1));
    let results = compute_ei_results(&tpm, &features, &registry, &settings, 0);
    assert!(!results.is_empty(), "1000 transitions crosses the gate");
    assert_eq!(results[0].transition_count, 1000);
}

/// Every macro row sums to 1 under an UNEVEN partition (group sizes 1,
/// 2, and 3 — equal-size groups can mask a naive row-summing bug).
#[test]
fn coarse_graining_row_stochastic() {
    // 6 live classes with assorted (row-stochastic) micro rows.
    let rows: Vec<(u32, Vec<f64>)> = vec![
        (0, vec![0.5, 0.2, 0.1, 0.1, 0.05, 0.05]),
        (1, vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0]),
        (2, vec![0.1, 0.1, 0.1, 0.1, 0.3, 0.3]),
        (3, vec![0.25, 0.25, 0.25, 0.25, 0.0, 0.0]),
        (4, vec![0.0, 0.0, 0.0, 0.0, 0.5, 0.5]),
        (5, vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
    ];
    let group_of = vec![0, 0, 0, 1, 1, 2]; // sizes 3, 2, 1 — uneven
    let macro_rows = coarse_grain(&rows, &group_of, 3);
    assert_eq!(macro_rows.len(), 3);
    for (i, row) in macro_rows.iter().enumerate() {
        let sum: f64 = row.iter().sum();
        assert!(
            (sum - 1.0).abs() < TOL,
            "macro row {i} sums to {sum}, not 1 — row-stochasticity broken"
        );
    }
}

/// EI = (determinism − degeneracy) · log2(M) for arbitrary TPMs — a free
/// cross-check that all three quantities are computed consistently.
#[test]
fn ei_identity_det_deg() {
    let cases: Vec<Vec<Vec<f64>>> = vec![
        vec![vec![0.7, 0.3], vec![0.4, 0.6]],
        vec![
            vec![0.1, 0.6, 0.3],
            vec![0.5, 0.25, 0.25],
            vec![0.0, 0.0, 1.0],
        ],
        vec![
            vec![0.25, 0.25, 0.25, 0.25],
            vec![1.0, 0.0, 0.0, 0.0],
            vec![0.0, 0.5, 0.5, 0.0],
            vec![0.9, 0.1, 0.0, 0.0],
        ],
    ];
    for rows in &cases {
        let c = ei_from_tpm(rows);
        let m = rows.len() as f64;
        let reconstructed = (c.determinism - c.degeneracy) * m.log2();
        assert!(
            (c.ei - reconstructed).abs() < 1e-9,
            "identity violated: EI {} vs (det-deg)*log2M {}",
            c.ei,
            reconstructed
        );
    }
}

/// M = 1: nothing to discriminate — EI = 0, det = deg = 0, no NaN.
#[test]
fn single_class_system_is_zero() {
    let rows = vec![vec![1.0]];
    let c = ei_from_tpm(&rows);
    assert_eq!(c.ei, 0.0);
    assert_eq!(c.determinism, 0.0);
    assert_eq!(c.degeneracy, 0.0);
    assert!(!c.ei.is_nan() && !c.determinism.is_nan() && !c.degeneracy.is_nan());
    // And through the full pipeline: one class self-transitioning.
    let mut tpm = TpmAccumulator::new();
    for _ in 0..1000 {
        tpm.record(WlHash(7), WlHash(7));
    }
    let results = compute_ei_results(
        &tpm,
        &AHashMap::new(),
        &ConstructorRegistry::new(),
        &EiSettings::default(),
        0,
    );
    for r in &results {
        assert_eq!(r.ei_micro, 0.0);
        assert_eq!(r.ce_coefficient, 0.0, "guarded CE: 0/0 is 0, not NaN");
        assert!(!r.ei_macro.is_nan());
    }
}

/// Experiment-6 carrier/substrate partition (S7.2). Pre-replication there are no
/// carrier classes, so the partition must collapse to K = 1 (degenerate baseline);
/// once carrier classes appear it must become a genuine 2-group partition.
#[test]
fn carrier_substrate_partition_is_degenerate_without_carriers() {
    let sub = |mc: u32| ClassFeatures { degree: 3, dominant_arity: 2, energy: 0.0, motif_class: mc };

    // No carriers: every class motif_class == 0 ⟹ K = 1 (all one group).
    let only_substrate = vec![sub(0), sub(0), sub(0)];
    let (groups, k) = hg_causal::carrier_substrate_partition(&only_substrate);
    assert_eq!(k, 1, "no carrier classes ⟹ degenerate K=1 (CE=0 baseline)");
    assert!(groups.iter().all(|&g| g == 0));

    // Carriers present: substrate (0) vs carrier (1) ⟹ K = 2, compact indices.
    let mixed = vec![sub(0), sub(1), sub(0), sub(1)];
    let (groups, k) = hg_causal::carrier_substrate_partition(&mixed);
    assert_eq!(k, 2, "substrate + carrier classes ⟹ genuine 2-group partition");
    // Same raw motif_class ⟹ same group; different ⟹ different.
    assert_eq!(groups[0], groups[2], "both substrate ⟹ same group");
    assert_eq!(groups[1], groups[3], "both carrier ⟹ same group");
    assert_ne!(groups[0], groups[1], "substrate and carrier are distinct groups");

    // A carrier-only population (post-onset, substrate filtered out of the TPM) is
    // also degenerate — the partition means nothing without BOTH sides present.
    let only_carrier = vec![sub(1), sub(1)];
    let (_g, k) = hg_causal::carrier_substrate_partition(&only_carrier);
    assert_eq!(k, 1, "carrier-only classes ⟹ degenerate K=1");
}
