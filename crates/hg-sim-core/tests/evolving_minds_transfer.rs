#![cfg(feature = "evolving-minds")]
use hg_sim_core::evolving_minds::*;

#[test]
fn gate0_registration_is_fixed_and_does_not_launch() {
    let c = TransferGate0Config::default();
    let r = c.registration().unwrap();
    assert_eq!(r["counts"]["native_advances_executed"], 14720);
    assert_eq!(r["counts"]["published_states"], 46);
    assert_eq!(r["scientific_run_launched"], false);
    let cases = c.cases().unwrap();
    assert_eq!(cases.len(), 5);
    assert!(cases
        .iter()
        .all(|x| [17, 23].contains(&x.config.seed)
            && x.config.kernels == [MutationKernel::Faithful; 2]));
    assert_eq!(
        cases
            .iter()
            .filter(|x| x.fault == StreamFault::CleanStop)
            .count(),
        1
    );
    for modified in [
        TransferGate0Config {
            worlds: vec![211, 223],
            ..c.clone()
        },
        TransferGate0Config {
            development_only: false,
            ..c.clone()
        },
        TransferGate0Config {
            horizon: 128,
            ..c.clone()
        },
        TransferGate0Config {
            minimum_net_resource_gain: 0,
            ..c.clone()
        },
        TransferGate0Config {
            required_faithful_active_transmissions: 2,
            ..c.clone()
        },
    ] {
        assert!(modified.registration().is_err());
    }
    let mut v = serde_json::to_value(c).unwrap();
    v["scientific_launch"] = true.into();
    assert!(serde_json::from_value::<TransferGate0Config>(v).is_err());
}

#[test]
fn gate0_parent_payment_and_faithful_children_are_native() {
    let rows = TransferGate0Config::default().cases().unwrap();
    for case in &rows[..2] {
        let mut s =
            FamilySimulation::new_handoff(case.config.clone(), case.handoff.clone()).unwrap();
        s.run().unwrap();
        let records = &s.admission.as_ref().unwrap().records;
        assert!(!records.is_empty());
        for r in records {
            assert!(r.settled);
            assert_eq!(r.request_charge, 1.0);
            assert_eq!(r.copying_charge, 5.0);
            assert_eq!(r.refunded, 0.0);
            if r.parent <= 2 {
                assert_eq!(r.parent_program, FundedProgram::Delayed.program().unwrap());
                let expected = if case.arm == "useful" {
                    FundedProgram::Immediate
                } else {
                    FundedProgram::Delayed
                };
                assert_eq!(r.proposed_program, expected.program().unwrap());
            } else {
                assert_eq!(r.parent_program, r.proposed_program);
            }
        }
    }
}
