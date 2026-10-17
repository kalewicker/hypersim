//! Phase 8 exit: every section present and non-empty (or explicitly says
//! why not) on a real summary.

use hg_schema::{GateName, GateStatus, MetricsRow, Regime, RunSummary};

fn full_summary() -> RunSummary {
    let mut s = RunSummary::new();
    s.provenance.run_id = "exp_42_t0".into();
    s.provenance.experiment_name = "exp".into();
    s.provenance.seed = 42;
    s.provenance.state = "Completed".into();
    s.run_parameters = vec![
        ("rules".into(), "binary_edge (builtin)".into()),
        ("initial".into(), "white_noise n=20".into()),
    ];
    s.gate_statuses = GateName::ALL
        .iter()
        .map(|g| (*g, GateStatus::Pending))
        .collect();
    s.fixture_truth_table = vec![("FrozenGrid".into(), "Frozen".into(), true)];
    s.regime_summary = vec![(Regime::GrowthTreadmill, 7), (Regime::Unknown, 1)];
    s.final_metrics = Some(MetricsRow {
        n_nodes: 100,
        pi_normalized: Some(0.93),
        regime: Some("GrowthTreadmill".into()),
        ..Default::default()
    });
    s.evaluator_details = vec![
        ("PI Normalization".into(), "overlap-based, full sample".into()),
        ("Type Histogram Dynamics".into(), "entropy 1.2 -> 1.3".into()),
        ("Static vs Active Persistence".into(), "3 static, 0 active".into()),
        ("Novelty Archive Stats".into(), "12 signatures, 0 evictions".into()),
    ];
    s.causal_summary = vec![("tasks".into(), "4".into())];
    s.constructor_ecology = vec![("constructors".into(), "1 (fidelity 1.00)".into())];
    s.suspicious_flags = vec!["pi_normalized jumped 0.9 -> 0.2 at window 5".into()];
    s.observations = vec!["treadmill regime dominates".into()];
    s.recommended_actions = vec!["enable causal layer".into()];
    s
}

#[test]
fn all_eleven_sections_present_and_nonempty() {
    let md = hg_report::render_report(&full_summary());
    let sections = [
        "## 1. Manifest",
        "## 2. Run Parameters",
        "## 3. Campaign Summary",
        "## 4. Gate Status",
        "### Fixture Truth Table",
        "## 5. Regime Summary",
        "## 6. Evaluator Details",
        "## 7. Causal Language Summary",
        "### Constructor Ecology",
        "## 8. Final Metrics Table",
        "## 9. Suspicious Metric Flags",
        "## 10. Observations",
        "## 11. Recommended Next Actions",
    ];
    let mut last = 0;
    for sec in sections {
        let ix = md.find(sec).unwrap_or_else(|| panic!("missing section {sec}"));
        assert!(ix > last || last == 0, "section {sec} out of order");
        last = ix;
    }
    // Between each pair of consecutive sections there must be real content.
    for w in sections.windows(2) {
        let a = md.find(w[0]).unwrap() + w[0].len();
        let b = md.find(w[1]).unwrap();
        assert!(
            md[a..b].trim().len() > 3,
            "section {} is empty",
            w[0]
        );
    }
}

#[test]
fn empty_summary_sections_explain_why() {
    let md = hg_report::render_report(&RunSummary::new());
    assert!(md.contains("Not applicable — single run"));
    assert!(md.contains("evaluator disabled") || md.contains("No gates evaluated"));
    assert!(md.contains("No final metrics"));
    assert!(md.contains("None detected."));
}

#[test]
fn epistemic_tags_present() {
    let md = hg_report::render_report(&full_summary());
    assert!(md.contains("[Measured]"));
    assert!(md.contains("[Estimated]"));
}

#[test]
fn leakage_section_renders_rows_and_summary() {
    use hg_schema::leakage::{ClaimType, LeakageRow};
    let mut s = RunSummary::new();
    s.leakage_rows = vec![LeakageRow {
        step: 500,
        detour_p50: 2.0,
        detour_p95: 3.0,
        wormhole_frac: 0.01,
        cut_exponent: Some(0.52),
        cut_fit_r2: Some(0.98),
        area_law_defect: Some(0.09),
        d_bfs: Some(1.9),
        d_spec: None,
        dim_mismatch: None,
        claim_type: ClaimType::Estimated,
        ..Default::default()
    }];
    s.leakage_summary = vec![(
        "closure defect".into(),
        "[Inferred] CD = 0.0410 bits/window over 300 windows (5 folds, certainty Inferred)".into(),
    )];
    let md = hg_report::render_report(&s);
    assert!(md.contains("Locality & Leakage"));
    assert!(md.contains("wormhole_frac"));
    assert!(md.contains("closure defect"));
    assert!(md.contains("CD = 0.0410"));
}

#[test]
fn leakage_section_says_disabled_when_absent() {
    let md = hg_report::render_report(&RunSummary::new());
    assert!(md.contains("Leakage track disabled for this run."));
}
