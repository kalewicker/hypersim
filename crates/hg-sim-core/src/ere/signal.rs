//! Bounded complete-family optimizer diagnostic. Native physics, frozen metadata.
use super::{
    coherent::{self, Request, Role, Status},
    meta_excluded_store, semantic,
};
use hg_core::{HyperEdge, HypergraphStore, Label, NodeData, NodeId, PropertyValue};
use hg_engine::Engine;
use hg_meta::*;
use hg_rules::{EpochPolicy, MatchBudgetPolicy, StepMode};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde_json::{json, Value};
use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

pub const PARAMS: usize = 183;
pub const HORIZON: usize = 100;
pub struct Emission {
    pub store: HypergraphStore,
    pub root: NodeId,
    pub program: Option<RuleProgram>,
    pub record: Value,
}
fn hex(bytes: Vec<u8>) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
pub fn world() -> HypergraphStore {
    let mut s = HypergraphStore::new_metadata_isolated();
    let ids: Vec<_> = (0..64)
        .map(|_| {
            let mut n = NodeData::new(Label(0), 0);
            n.set_property(1, PropertyValue::Int(100));
            s.add_node(n)
        })
        .collect();
    for i in 0..96 {
        s.add_edge(HyperEdge::new(
            Label(0),
            vec![ids[i % 64], ids[(i + 1) % 64]].into(),
        ));
    }
    s.add_edge(HyperEdge::new(
        Label(0),
        vec![ids[0], ids[1], ids[2]].into(),
    ));
    s.add_edge(HyperEdge::new(
        Label(0),
        vec![
            ids[0], ids[1], ids[0], ids[1], ids[0], ids[1], ids[0], ids[1],
        ]
        .into(),
    ));
    s
}
fn pick(logits: &[f32], rng: &mut ChaCha8Rng) -> usize {
    let max = logits.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let weights: Vec<_> = logits.iter().map(|x| ((*x - max) as f64).exp()).collect();
    let mut draw = rng.gen::<f64>() * weights.iter().sum::<f64>();
    for (i, w) in weights.iter().enumerate() {
        draw -= w;
        if draw <= 0.0 {
            return i;
        }
    }
    logits.len() - 1
}
pub fn emit(theta: &[f32], seed: u64) -> Result<Emission, String> {
    if theta.len() != PARAMS || theta.iter().any(|x| !x.is_finite()) {
        return Err("expected 183 finite logits".into());
    }
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let parent = pick(&theta[..168], &mut rng);
    let op = pick(&theta[168..181], &mut rng);
    let role = pick(&theta[181..], &mut rng);
    selected(parent, op, role, &mut rng, seed)
}
pub fn forced(active: bool, seed: u64) -> Result<Emission, String> {
    selected(
        4,
        if active { 11 } else { 12 },
        1,
        &mut ChaCha8Rng::seed_from_u64(seed),
        seed,
    )
}
fn selected(
    parent: usize,
    op: usize,
    role: usize,
    rng: &mut ChaCha8Rng,
    seed: u64,
) -> Result<Emission, String> {
    let p = semantic::parent_bank().remove(parent).1;
    let mut s = world();
    let initial_world_hash = format!("{:016x}", meta_excluded_store(&s).canonical_hash());
    let limits = MetaLimits::default();
    let root = encode_complete_family(
        &mut s,
        &p.core,
        p.proposal.as_ref().unwrap(),
        p.active,
        &limits,
    )
    .map_err(|v| format!("encode {v:?}"))?
    .root;
    let role = if role == 0 && op < 11 {
        Role::Core
    } else {
        Role::Proposal
    };
    let t = coherent::targets(&s, root, role, &limits)?;
    let pattern = t.patterns[rng.gen_range(0..t.patterns.len())];
    let variable = t.variables[rng.gen_range(0..t.variables.len())];
    let second = t.variables[rng.gen_range(0..t.variables.len())];
    let side = if rng.gen_bool(0.5) { t.left } else { t.right };
    let slots = coherent::slots(&s, pattern);
    let slot = slots[rng.gen_range(0..slots.len())];
    let value = rng.gen_range(0..2);
    let req = match op {
        0 => Request::AddSlot { pattern, variable },
        1 => Request::RemoveSlot { slot },
        2 => Request::RetargetSlot { slot, variable },
        3 => Request::AddVar { pattern },
        4 => Request::RemoveVar { variable },
        5 => Request::AddPatternEdge {
            side,
            a: variable,
            b: second,
        },
        6 => Request::RemovePatternEdge { pattern },
        7 => Request::AddLabel { variable, value },
        8 => Request::RemoveLabel { variable },
        9 => Request::ChangeLabel { variable, value },
        10 => Request::ChangePriority,
        11 => Request::Activate,
        12 => Request::Deactivate,
        _ => unreachable!(),
    };
    let a = coherent::apply(&mut s, root, role, req, &limits);
    let admitted = matches!(a.status, Status::Applied | Status::Inapplicable);
    let program = if admitted { a.after_program } else { None };
    let signature = program.as_ref().map(|p| hex(p.execution_bytes()));
    assert_eq!(
        initial_world_hash,
        format!("{:016x}", meta_excluded_store(&a.staged).canonical_hash())
    );
    let classes=program.as_ref().map(|p|json!({"active":p.active,"core":hex(structural(&p.core)),"proposal":p.proposal.as_ref().map(|q|hex(structural(q))),"proposal_execution":p.proposal.as_ref().map(|q|hex(q.execution_bytes())),"rcf":hex(p.rcf_bytes())}));
    let record = json!({"emission_seed":seed,"parent":parent,"operator":op,"role":role,"status":format!("{:?}",a.status),"admitted":admitted,"signature":signature,"classes":classes,"initial_world_hash":initial_world_hash,"root":root,"kernel":a.receipt});
    Ok(Emission {
        store: a.staged,
        root,
        program,
        record,
    })
}
struct Capture(Arc<Mutex<Vec<usize>>>);
impl hg_observe::Observer for Capture {
    fn on_step_end(
        &mut self,
        _: &hg_core::GraphSnapshot<'_>,
        _: &hg_observe::StepStats,
        events: &[hg_observe::RewriteEvent],
    ) {
        *self.0.lock().unwrap() = events.iter().map(|e| e.rule_id).collect();
    }
}
fn structural(c: &CandidateRule) -> Vec<u8> {
    let mut c = c.clone();
    c.priority = 0;
    c.rcf_bytes()
}
pub fn evaluate(emission: &Emission, seed: u64, deadline: Instant) -> Result<Value, String> {
    evaluate_with_budget(emission, seed, deadline, 400_000, 50_000)
}
pub fn evaluate_with_budget(
    emission: &Emission,
    seed: u64,
    deadline: Instant,
    work: usize,
    results: usize,
) -> Result<Value, String> {
    let mut result = evaluate_with_horizon(emission, seed, deadline, work, results, HORIZON)?;
    result.as_object_mut().unwrap().remove("horizon");
    if let Some(steps) = result["steps"].as_array_mut() {
        for step in steps {
            step.as_object_mut().unwrap().remove("nodes");
            step.as_object_mut().unwrap().remove("edges");
        }
    }
    Ok(result)
}
/// Versioned scaling API; legacy wrappers retain exact 100-step receipt shape.
pub fn evaluate_with_horizon(
    emission: &Emission,
    seed: u64,
    deadline: Instant,
    work: usize,
    results: usize,
    horizon: usize,
) -> Result<Value, String> {
    if !(2..=10_000).contains(&horizon) {
        return Err("scaling horizon must be in 2..=10000".into());
    }
    let Some(p) = &emission.program else {
        return Ok(
            json!({"horizon":horizon,"physics_seed":seed,"score":null,"reason":"rejected_emission","steps":[],"attempted_steps":0,"advancing_steps":0}),
        );
    };
    let mut s = emission.store.clone();
    let mut src = GraphRuleSource::new(EpochPolicy::Never, MetaLimits::default(), false);
    src.complete_witness_enabled = true;
    let mut engine = Engine::new(vec![], StepMode::MaxParallel, seed, 1);
    engine.set_match_budget_policy(work, results, MatchBudgetPolicy::Pooled);
    engine.set_rule_source(Box::new(src));
    engine.compile_at_boundary(&s);
    let candidates = p.candidates();
    let proposal = p
        .proposal
        .as_ref()
        .filter(|q| p.active && structural(q) != structural(&p.core));
    let credited: Vec<_> = candidates
        .iter()
        .enumerate()
        .filter(|(_, c)| proposal.is_some_and(|q| q.execution_bytes() == c.execution_bytes()))
        .map(|(i, _)| i)
        .collect();
    let capture = Arc::new(Mutex::new(vec![]));
    engine.add_observer(Box::new(Capture(capture.clone())));
    let mut steps = vec![];
    let mut fires = 0usize;
    let mut reason = None;
    for step in 1..=horizon {
        if Instant::now() >= deadline {
            return Err("campaign wall ceiling".into());
        }
        let before = s.step();
        capture.lock().unwrap().clear();
        let stats = engine.step(&mut s);
        let ids = capture.lock().unwrap().clone();
        assert_eq!(stats.accepted_rewrites, ids.len());
        fires += ids.iter().filter(|id| credited.contains(id)).count();
        let valid =
            read_complete_program(&s.snapshot(), emission.root, &MetaLimits::default()).is_ok();
        let cap = engine.match_ceiling_binding() || s.node_count() > 2000 || s.edge_count() > 4000;
        let advancing = s.step() - before;
        steps.push(json!({"nodes":s.node_count(),"edges":s.edge_count(),"step":step,"rule_ids":ids,"usage":engine.match_budget_usage(),"cap":cap,"valid_family":valid,"advanced":advancing,"world_hash":format!("{:016x}",meta_excluded_store(&s).canonical_hash())}));
        if cap || !valid || advancing != 1 {
            reason = Some(if cap {
                "resource_cap"
            } else if !valid {
                "invalid_family"
            } else {
                "nonadvancing"
            });
            break;
        }
    }
    let score = if reason.is_none() && steps.len() == horizon {
        Some((fires.saturating_sub(1) as f64).ln_1p() / (horizon as f64).ln())
    } else {
        None
    };
    Ok(
        json!({"horizon":horizon,"physics_seed":seed,"score":score,"reason":reason,"credited_rule_ids":credited,"candidate_execution":candidates.iter().map(|c|hex(c.execution_bytes())).collect::<Vec<_>>(),"proposal_fires":fires,"attempted_steps":steps.len(),"advancing_steps":s.step(),"steps":steps}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn emissions_are_deterministic_and_world_is_fixed() {
        let a = emit(&vec![0.0; PARAMS], 1).unwrap();
        let b = emit(&vec![0.0; PARAMS], 1).unwrap();
        assert_eq!(a.record, b.record);
        for seed in 0..64 {
            assert_eq!(
                emit(&vec![0.0; PARAMS], seed).unwrap().record["initial_world_hash"],
                a.record["initial_world_hash"]
            );
        }
        assert!(emit(&[0.0], 1).is_err());
        assert!(emit(&vec![f32::NAN; PARAMS], 1).is_err());
    }
    #[test]
    fn forced_activation_has_real_reuse_dormancy_has_none() {
        let deadline = Instant::now() + std::time::Duration::from_secs(30);
        let active = evaluate(&forced(true, 7).unwrap(), 101, deadline).unwrap();
        let dormant = evaluate(&forced(false, 7).unwrap(), 101, deadline).unwrap();
        assert_eq!(active["score"], json!(1.0));
        assert_eq!(dormant["score"], json!(0.0));
        assert_eq!(active["proposal_fires"], 100);
        assert_eq!(dormant["proposal_fires"], 0);
    }
    #[test]
    fn priority_only_proposal_never_gets_credit_and_rejected_is_null() {
        let deadline = Instant::now() + std::time::Duration::from_secs(30);
        let identity = selected(1, 10, 1, &mut ChaCha8Rng::seed_from_u64(2), 2).unwrap();
        let result = evaluate(&identity, 101, deadline).unwrap();
        assert_eq!(result["score"], json!(0.0));
        assert_eq!(result["credited_rule_ids"], json!([]));
        let rejected = selected(0, 6, 0, &mut ChaCha8Rng::seed_from_u64(2), 2).unwrap();
        assert!(!rejected.record["admitted"].as_bool().unwrap());
        let result = evaluate(&rejected, 101, deadline).unwrap();
        assert!(result["score"].is_null());
        assert_eq!(result["attempted_steps"], 0);
    }
    #[test]
    fn binding_cap_is_null_not_zero_and_wall_expiry_stops() {
        let e = forced(true, 7).unwrap();
        let result = evaluate_with_budget(
            &e,
            101,
            Instant::now() + std::time::Duration::from_secs(30),
            1,
            1,
        )
        .unwrap();
        assert!(result["score"].is_null());
        assert_eq!(result["reason"], "resource_cap");
        assert_eq!(result["attempted_steps"], 1);
        assert!(evaluate(&e, 101, Instant::now()).is_err());
    }
}
