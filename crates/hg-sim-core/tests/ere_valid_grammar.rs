#![cfg(all(feature = "hg-meta", feature = "repeated-edge-feasibility"))]
use hg_meta::*;
use hg_sim_core::ere::{
    coherent::{self, Request, Role},
    meta_excluded_store, semantic, signal,
    valid_grammar::{self, Grammar, PARAMS},
};
use serde_json::json;
use std::sync::OnceLock;
fn extended_world(mut s: hg_core::HypergraphStore) -> serde_json::Value {
    let a = s.add_node(hg_core::NodeData::new(hg_core::Label(0), 0));
    let b = s.add_node(hg_core::NodeData::new(hg_core::Label(0), 0));
    s.add_edge(hg_core::HyperEdge::new(
        hg_core::Label(0),
        vec![a, b].into(),
    ));
    coherent::inventory(&meta_excluded_store(&s))
}
fn grammar() -> &'static Grammar {
    static G: OnceLock<Grammar> = OnceLock::new();
    G.get_or_init(|| valid_grammar::compile().unwrap())
}
fn force(parent: usize, op: usize, role: usize) -> Vec<f32> {
    let mut theta = vec![-100.; PARAMS];
    theta[parent] = 100.;
    theta[168 + op] = 100.;
    theta[181 + role] = 100.;
    theta
}
#[test]
fn all_parents_and_operators_have_changed_valid_productions() {
    let g = grammar();
    assert_eq!(g.parent_count(), 168);
    let mut ops = [0; 13];
    for p in 0..168 {
        assert!(!g.operator_support(p).is_empty());
        for op in 0..13 {
            for r in 0..2 {
                ops[op] += g.production_count(p, op, r);
            }
        }
        assert_eq!(g.production_count(p, 11, 0), 0);
        assert_eq!(g.production_count(p, 12, 0), 0);
        let active = semantic::parent_bank()[p].1.active;
        assert_eq!(g.production_count(p, 11, 1), usize::from(!active));
        assert_eq!(g.production_count(p, 12, 1), usize::from(active));
    }
    assert!(ops.iter().all(|n| *n > 0), "{ops:?}");
}
#[test]
fn every_legal_production_emits_once_and_preserves_world_and_validity() {
    let g = grammar();
    let world = coherent::inventory(&signal::world());
    let bank = semantic::parent_bank();
    let continued = extended_world(signal::world());
    for p in 0..168 {
        for op in 0..13 {
            for r in 0..2 {
                for i in 0..g.production_count(p, op, r) {
                    let e = g.emit_selected(p, op, r, i, 0).unwrap();
                    assert_eq!(e.record["kernel_attempts"], 1);
                    assert_eq!(e.record["status"], "Applied");
                    assert_eq!(e.record["parent_fallback"], false);
                    assert_eq!(e.record["resampled"], false);
                    assert_eq!(world, coherent::inventory(&meta_excluded_store(&e.store)));
                    let post =
                        read_complete_program(&e.store.snapshot(), e.root, &MetaLimits::default())
                            .unwrap();
                    assert_eq!(e.program, Some(post.clone()));
                    assert_ne!(post, bank[p].1);
                    assert_eq!(extended_world(e.store), continued);
                }
            }
        }
    }
}
#[test]
fn logits_cover_all_available_parent_operator_and_role_choices() {
    let g = grammar();
    for p in 0..168 {
        for op in g.operator_support(p) {
            for r in g.role_support(p, op) {
                let e = g.emit(&force(p, op, r), 19).unwrap();
                assert_eq!(e.record["parent"], p);
                assert_eq!(e.record["operator"], op);
                assert_eq!(
                    e.record["role"],
                    json!(if r == 0 { Role::Core } else { Role::Proposal })
                );
            }
        }
    }
}
#[test]
fn deterministic_sampling_and_extreme_finite_theta() {
    let g = grammar();
    let theta = vec![0.; PARAMS];
    assert_eq!(
        g.emit(&theta, 7).unwrap().record,
        g.emit(&theta, 7).unwrap().record
    );
    for seed in 0..256 {
        assert!(g.emit(&theta, seed).unwrap().program.is_some());
    }
    for v in [f32::MAX, f32::MIN, 0.] {
        let t = vec![v; PARAMS];
        assert!(g.emit(&t, 7).is_ok());
    }
    let mut t = vec![f32::MIN; PARAMS];
    t[167] = f32::MAX;
    t[168] = f32::MAX;
    t[181] = f32::MAX;
    let e = g.emit(&t, 7).unwrap();
    assert_eq!(e.record["parent"], 167);
    assert_eq!(e.record["operator"], 0);
    assert_eq!(e.record["role"], "Proposal"); // Core AddSlot has no support at arity eight.
}
#[test]
fn absent_or_malformed_choices_fail_without_parent_fallback() {
    let g = grammar();
    assert!(g.emit(&[0.], 1).is_err());
    assert!(g.emit(&vec![f32::NAN; PARAMS], 1).is_err());
    assert!(g.emit(&vec![f32::INFINITY; PARAMS], 1).is_err());
    for (p, op, r, i) in [
        (168, 0, 0, 0),
        (0, 13, 0, 0),
        (0, 0, 2, 0),
        (163, 0, 0, 0),
        (0, 0, 0, usize::MAX),
    ] {
        assert!(g.emit_selected(p, op, r, i, 0).is_err());
    }
}
#[test]
fn known_invalid_and_noop_requests_are_not_productions() {
    let g = grammar();
    let c = g.catalogue();
    let mut s = signal::world();
    let p = semantic::parent_bank()[116].1.clone();
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
    let t = coherent::targets(&s, root, Role::Proposal, &limits).unwrap();
    let pattern = *t
        .patterns
        .iter()
        .find(|n| coherent::slots(&s, **n).len() == 2)
        .unwrap();
    let bad = json!(Request::RemoveSlot {
        slot: coherent::slots(&s, pattern)[0]
    });
    let entries = &c["parents"][116]["groups"][1]["roles"][1]["productions"];
    assert!(!entries
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e["request"] == bad));
    assert_eq!(g.production_count(163, 0, 0), 0);
    for parent in c["parents"].as_array().unwrap() {
        for group in parent["groups"].as_array().unwrap() {
            for r in group["roles"].as_array().unwrap() {
                assert!(
                    r["stats"]["Applied"].as_u64().unwrap_or(0)
                        == r["productions"].as_array().unwrap().len() as u64
                );
                for entry in r["productions"].as_array().unwrap() {
                    assert!(!entry["request"].is_null());
                }
            }
        }
    }
}
#[test]
fn fresh_parent_encoding_has_identical_request_handles() {
    let g = grammar();
    let c = g.catalogue();
    for p in 0..168 {
        let mut s = signal::world();
        let program = semantic::parent_bank()[p].1.clone();
        let limits = MetaLimits::default();
        let root = encode_complete_family(
            &mut s,
            &program.core,
            program.proposal.as_ref().unwrap(),
            program.active,
            &limits,
        )
        .unwrap()
        .root;
        assert_eq!(json!(root), c["parents"][p]["root"]);
        for r in [Role::Core, Role::Proposal] {
            let t = coherent::targets(&s, root, r, &limits).unwrap();
            assert!(!t.patterns.is_empty());
            assert!(!t.variables.is_empty());
        }
        let e = g.emit_selected(p, 10, 0, 0, 0).unwrap();
        let mut copy = s.clone();
        let a = coherent::apply(
            &mut copy,
            root,
            Role::Core,
            Request::ChangePriority,
            &limits,
        );
        assert_eq!(e.record["kernel"], a.receipt);
    }
}
