#![cfg(feature = "hg-meta")]
use hg_sim_core::ere::{rule_maintenance as m, rule_relational as old};
use std::time::{Duration, Instant};
fn deadline() -> Instant {
    Instant::now() + Duration::from_secs(60)
}
#[test]
fn no_hazard_24_step_parity() {
    for context in m::CONTEXTS {
        let actions = [
            m::CHOICES[0]
                .iter()
                .position(|x| *x == Some(context[0]))
                .unwrap(),
            m::CHOICES[1]
                .iter()
                .position(|x| *x == Some(context[1]))
                .unwrap(),
        ];
        let a = old::run(old::Arm::LivePaid, context, actions, m::SEED, deadline()).unwrap();
        let b = m::parity(context, actions, deadline()).unwrap();
        for key in [
            "initial_inventory",
            "final_inventory",
            "seeded_editor_rules",
            "seeded_editor_compiled",
            "initial_all_candidates",
        ] {
            assert_eq!(a[key], b[key], "{key}");
        }
        for (x, y) in a["timeline"]
            .as_array()
            .unwrap()
            .iter()
            .zip(b["timeline"].as_array().unwrap())
        {
            for key in [
                "step",
                "genotype",
                "installed",
                "events",
                "paid_edits",
                "changed_nodes",
                "stats",
                "rng_before",
                "rng_after",
                "usage",
                "compile_report",
            ] {
                assert_eq!(x[key], y[key], "{key}");
            }
        }
    }
}
#[test]
fn paid_post_birth_repair_and_controls() {
    for (arm, hazards, repairs) in [
        (m::Arm::LearnedProtected, 0, 0),
        (m::Arm::LearnedExposedRepair, 4, 4),
        (m::Arm::LearnedExhaustion, 4, 2),
        (m::Arm::LearnedUnpaid, 4, 0),
        (m::Arm::LearnedFrozen, 4, 4),
        (m::Arm::LearnedNoHazard, 0, 0),
        (m::Arm::OracleExposedRepair, 4, 4),
    ] {
        let r = m::run(arm, [10, 11], [1, 1], m::SEED, deadline()).unwrap();
        if let Some(path)=std::env::var_os("ERE_RULE_MAINTENANCE_PREFLIGHT") {
            let path=std::path::PathBuf::from(path).join(format!("{}.json",serde_json::to_value(arm).unwrap().as_str().unwrap()));
            std::fs::write(path,serde_json::to_vec(&r).unwrap()).unwrap();
        }
        assert_eq!(r["completed_steps"], 96);
        assert_eq!(r["summary"]["birth_payments"], 2);
        assert_eq!(r["summary"]["hazard_events"], hazards);
        assert_eq!(r["summary"]["repair_payments"], repairs);
        assert_eq!(r["initialization"]["paid_applications"], 1);
        let rows = r["timeline"].as_array().unwrap();
        for step in [17, 33, 49, 65] {
            if (arm == m::Arm::LearnedExhaustion && step >= 49) || arm == m::Arm::LearnedUnpaid {
                assert_eq!(rows[step]["genotype"], serde_json::json!([]));
            } else {
                assert_eq!(
                    rows[step]["genotype"], rows[5]["genotype"],
                    "post-birth body changed"
                );
            }
        }
        assert_eq!(r["scientific_acceptance"], false);
    }
}
