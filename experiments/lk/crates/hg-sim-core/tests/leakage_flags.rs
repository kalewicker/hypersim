//! LK5 exit criterion: the anti-gaslight suspicious flags fire correctly
//! on constructed pathological inputs (and stay silent on clean ones).

use hg_causal::ClosureReport;
use hg_schema::{ClaimType, LeakageRow};
use hg_sim_core::leakage_flags;

fn row(cut_fit_r2: Option<f32>) -> LeakageRow {
    LeakageRow {
        step: 0,
        detour_p50: 2.0,
        detour_p95: 2.0,
        wormhole_frac: 0.0,
        cut_exponent: Some(0.5),
        cut_fit_r2,
        area_law_defect: Some(0.0),
        d_bfs: Some(2.0),
        d_spec: None,
        dim_mismatch: None,
        claim_type: ClaimType::Estimated,
        ..Default::default()
    }
}

fn closure(negative: bool) -> ClosureReport {
    ClosureReport {
        cd_raw_bits: if negative { -0.1 } else { 0.1 },
        cd_clock_bits: None,
        n_windows: 300,
        sampler_spread_bits: None,
        spread_ensemble: None,
        stratum: "all".into(),
        cv_folds: 5,
        macro_fields: vec![],
        micro_fields: vec![],
        clock_fields: vec![],
        certainty: "Inferred".into(),
        pathology_negative_cd: negative,
    }
}

#[test]
fn clean_run_fires_no_flags() {
    // Good r², positive CD: only the always-checked expander self-test
    // runs, and it must PASS (so no flag). This also pins that the
    // shipped detour instrument still catches a known-leaky fixture.
    let flags = leakage_flags(&[row(Some(0.99)), row(Some(0.97))], Some(&closure(false)));
    assert!(flags.is_empty(), "clean run should fire no flags, got {flags:?}");
}

#[test]
fn low_cut_r2_flag_fires() {
    let flags = leakage_flags(&[row(Some(0.99)), row(Some(0.4))], None);
    assert!(
        flags.iter().any(|f| f.starts_with("cut_fit_r2_below_threshold")),
        "got {flags:?}"
    );
}

#[test]
fn negative_cd_flag_fires() {
    let flags = leakage_flags(&[row(Some(0.99))], Some(&closure(true)));
    assert!(
        flags.contains(&"closure_defect_negative".to_string()),
        "got {flags:?}"
    );
}

#[test]
fn expander_self_test_passes_in_this_binary() {
    // The expander_fixture_not_flagged_leaky flag must NOT fire — if it
    // does, the detour instrument in this build is broken and every
    // leakage result is untrustworthy.
    let flags = leakage_flags(&[row(Some(0.99))], None);
    assert!(
        !flags.contains(&"expander_fixture_not_flagged_leaky".to_string()),
        "the detour instrument failed its own leaky-fixture self-test"
    );
}

// ---- LK5R v2 flags (plan v2 §8) ----

use hg_schema::TrackedRow;
use hg_sim_core::leakage_flags_v2;

fn tracked(edges: u32, components: u32) -> TrackedRow {
    TrackedRow {
        tracked_edge_count: edges,
        tracked_components: components,
        ..Default::default()
    }
}

fn audit_row(n_sub: u64) -> LeakageRow {
    LeakageRow {
        n_subdivisions: Some(n_sub),
        audit_residual: Some(0.0),
        ..row(Some(0.99))
    }
}

#[test]
fn throat_divergence_flag_fires() {
    // One handle refined into 6 edges: edge count diverges from
    // component count — throat morphology must be remarked on.
    let flags = leakage_flags_v2(&[tracked(2, 1), tracked(6, 1)], &[], 200, false);
    assert!(flags.contains(&"wormhole_edges_diverge_from_components_unremarked".to_string()));
    // Parallel refinement (edges grow WITH handles) does not fire.
    let flags = leakage_flags_v2(&[tracked(6, 4)], &[], 200, false);
    assert!(flags.is_empty(), "got {flags:?}");
}

#[test]
fn quench_before_min_windows_flag_fires() {
    // The structural era ended (n_sub froze) after 3 active windows,
    // far below the 200-window closure floor.
    let rows: Vec<LeakageRow> =
        [10, 20, 30, 30, 30].iter().map(|&n| audit_row(n)).collect();
    let flags = leakage_flags_v2(&[], &rows, 200, true);
    assert!(flags.contains(&"quench_before_min_windows".to_string()), "got {flags:?}");
    // Still-active era: no quench flag.
    let rows: Vec<LeakageRow> = [10, 20, 30, 40, 50].iter().map(|&n| audit_row(n)).collect();
    let flags = leakage_flags_v2(&[], &rows, 200, true);
    assert!(!flags.contains(&"quench_before_min_windows".to_string()));
    // lk_rules off: never fires.
    let rows: Vec<LeakageRow> = [10, 20, 30, 30].iter().map(|&n| audit_row(n)).collect();
    let flags = leakage_flags_v2(&[], &rows, 200, false);
    assert!(flags.is_empty());
}

#[test]
fn audit_drift_flag_fires() {
    let mut bad = audit_row(10);
    bad.audit_residual = Some(-25.0);
    let flags = leakage_flags(&[row(Some(0.99)), bad], None);
    assert!(flags.contains(&"audit_invariant_drift".to_string()), "got {flags:?}");
}
