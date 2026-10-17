//! Phase 2 exit: every type round-trips through serde.

use hg_schema::{GateName, GateStatus, MetricsRow, Regime, RunSummary};

#[test]
fn regime_roundtrips_all_variants() {
    for r in [
        Regime::Frozen,
        Regime::StaticOrdered,
        Regime::GrowthTreadmill,
        Regime::WhiteNoiseSoup,
        Regime::SterileChaos,
        Regime::Exploding,
        Regime::FragmentedIsolation,
        Regime::Monoculture,
        Regime::ActivePersistentStructure,
        Regime::GenerativeCandidate,
        Regime::Unknown,
    ] {
        let json = serde_json::to_string(&r).unwrap();
        let back: Regime = serde_json::from_str(&json).unwrap();
        assert_eq!(r, back);
    }
}

#[test]
fn regime_has_eleven_variants() {
    // A structural guard: the classifier and docs assume exactly 11.
    let names: std::collections::HashSet<_> = [
        Regime::Frozen,
        Regime::StaticOrdered,
        Regime::GrowthTreadmill,
        Regime::WhiteNoiseSoup,
        Regime::SterileChaos,
        Regime::Exploding,
        Regime::FragmentedIsolation,
        Regime::Monoculture,
        Regime::ActivePersistentStructure,
        Regime::GenerativeCandidate,
        Regime::Unknown,
    ]
    .iter()
    .map(|r| r.as_str())
    .collect();
    assert_eq!(names.len(), 11);
}

#[test]
fn gate_roundtrips() {
    for s in [GateStatus::Pass, GateStatus::Fail, GateStatus::Pending] {
        let j = serde_json::to_string(&s).unwrap();
        assert_eq!(s, serde_json::from_str::<GateStatus>(&j).unwrap());
    }
    for g in GateName::ALL {
        let j = serde_json::to_string(&g).unwrap();
        assert_eq!(g, serde_json::from_str::<GateName>(&j).unwrap());
    }
}

#[test]
fn metrics_and_summary_roundtrip() {
    let row = MetricsRow {
        n_nodes: 42,
        pi_normalized: Some(0.5),
        regime: Some(Regime::GenerativeCandidate.as_str().to_string()),
        ..Default::default()
    };
    let j = serde_json::to_string(&row).unwrap();
    assert_eq!(row, serde_json::from_str::<MetricsRow>(&j).unwrap());

    let mut s = RunSummary::new();
    s.final_metrics = Some(row);
    s.science_unlocked = false;
    let j = serde_json::to_string(&s).unwrap();
    let _back: RunSummary = serde_json::from_str(&j).unwrap();
}
