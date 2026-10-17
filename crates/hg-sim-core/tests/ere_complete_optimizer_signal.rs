#![cfg(all(feature = "hg-meta", feature = "repeated-edge-feasibility"))]
#[allow(dead_code)]
#[path = "support/coherent.rs"]
mod legacy;
#[allow(dead_code)]
#[path = "support/witness.rs"]
mod witness;
use hg_meta::*;
use hg_sim_core::ere::{coherent as new, semantic, signal};

#[test]
fn shared_kernel_matches_gate_helper_for_every_parent_role_and_operator() {
    for (_, p) in semantic::parent_bank() {
        let mut s = signal::world();
        let limits = MetaLimits::default();
        let root = encode_complete_family(
            &mut s,
            &p.core,
            p.proposal.as_ref().unwrap(),
            p.active,
            &limits,
        )
        .unwrap()
        .root;
        for role in [new::Role::Core, new::Role::Proposal] {
            let t = new::targets(&s, root, role, &limits).unwrap();
            let pattern = t.patterns[0];
            let variable = t.variables[0];
            let second = *t.variables.last().unwrap();
            let slot = new::slots(&s, pattern)[0];
            let requests = vec![
                new::Request::AddSlot { pattern, variable },
                new::Request::RemoveSlot { slot },
                new::Request::RetargetSlot {
                    slot,
                    variable: second,
                },
                new::Request::AddVar { pattern },
                new::Request::RemoveVar { variable },
                new::Request::AddPatternEdge {
                    side: t.right,
                    a: variable,
                    b: second,
                },
                new::Request::RemovePatternEdge { pattern },
                new::Request::AddLabel { variable, value: 1 },
                new::Request::RemoveLabel { variable },
                new::Request::ChangeLabel { variable, value: 1 },
                new::Request::ChangePriority,
                new::Request::Activate,
                new::Request::Deactivate,
            ];
            for req in requests {
                let old = match req.clone() {
                    new::Request::AddSlot { pattern, variable } => {
                        legacy::Request::AddSlot { pattern, variable }
                    }
                    new::Request::RemoveSlot { slot } => legacy::Request::RemoveSlot { slot },
                    new::Request::RetargetSlot { slot, variable } => {
                        legacy::Request::RetargetSlot { slot, variable }
                    }
                    new::Request::AddVar { pattern } => legacy::Request::AddVar { pattern },
                    new::Request::RemoveVar { variable } => legacy::Request::RemoveVar { variable },
                    new::Request::AddPatternEdge { side, a, b } => {
                        legacy::Request::AddPatternEdge { side, a, b }
                    }
                    new::Request::RemovePatternEdge { pattern } => {
                        legacy::Request::RemovePatternEdge { pattern }
                    }
                    new::Request::AddLabel { variable, value } => {
                        legacy::Request::AddLabel { variable, value }
                    }
                    new::Request::RemoveLabel { variable } => {
                        legacy::Request::RemoveLabel { variable }
                    }
                    new::Request::ChangeLabel { variable, value } => {
                        legacy::Request::ChangeLabel { variable, value }
                    }
                    new::Request::ChangePriority => legacy::Request::ChangePriority,
                    new::Request::Activate => legacy::Request::Activate,
                    new::Request::Deactivate => legacy::Request::Deactivate,
                };
                let mut a = s.clone();
                let mut b = s.clone();
                let a = new::apply(&mut a, root, role, req, &limits);
                let b = legacy::apply(
                    &mut b,
                    root,
                    if role == new::Role::Core {
                        legacy::Role::Core
                    } else {
                        legacy::Role::Proposal
                    },
                    old,
                    &limits,
                );
                assert_eq!(a.receipt, b.receipt);
                assert_eq!(a.before_program, b.before_program);
                assert_eq!(a.after_program, b.after_program);
                assert_eq!(
                    bincode::serialize(&a.staged).unwrap(),
                    bincode::serialize(&b.staged).unwrap()
                );
            }
        }
    }
}
