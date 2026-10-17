//! Versioned native paid-choice fixture; seeded actions, not learned editor semantics.
use super::lifecycle_fixture::{candidate, candidate_record, summarize};
use hg_core::{GraphSnapshot, HyperEdge, HypergraphStore, Label, NodeData, NodeId, PropertyOp, PropertyValue};
use hg_engine::Engine;
use hg_meta::*;
use hg_observe::{Observer, RewriteEvent, StepStats};
use hg_rules::{compile_rule, CompiledRule, EffectOpDecl, EpochPolicy, MatchBudgetPolicy, NodePropertyGuard, PropertyEffectDecl, Rule, StepMode};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{collections::BTreeSet, sync::{Arc, Mutex}, time::Instant};

pub const PROFILE: &str = "E2-rule-choice-adapter-v1";
pub const HORIZON: u64 = 24;
pub const SEEDS: [u64; 2] = [993748714001, 993748714002];
pub const CONTEXTS: [u32; 3] = [10,20,30];
pub const ACTIONS: [Option<u32>; 4] = [None,Some(10),Some(20),Some(30)];
pub const ARMS: [Arm; 5] = [Arm::LivePaid, Arm::FrozenPaid,
    Arm::LiveUnpaid, Arm::LiveDisabled, Arm::PreseededLure];
const TOKEN: u32 = META_BASE + 0x7600;
const CLOCK: u32 = 0x7600;
const TARGET: u32 = 0x7601;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Arm { LivePaid, FrozenPaid, LiveUnpaid, LiveDisabled, PreseededLure }

fn guard(rule: &mut Rule, var: &str, key: u32, op: PropertyOp, value: PropertyValue) {
    rule.lhs_node_property_guards.push(NodePropertyGuard { var:var.into(), property:key, op, value });
}
fn effect(rule: &mut Rule, var: &str, key: u32, op: EffectOpDecl) {
    rule.rhs_property_effects.push(PropertyEffectDecl { var:var.into(), key, op });
}

fn editor_rules(arm: Arm, action: usize) -> Result<(Vec<Rule>, Vec<CompiledRule>), String> {
    let mut tick = Rule { name:"lifecycle_seeded_tick".into(), priority:100, ..Rule::default() };
    tick.lhs.nodes = vec!["token".into()]; tick.rhs.nodes = tick.lhs.nodes.clone();
    tick.lhs.node_labels.insert("token".into(), Label(TOKEN));
    guard(&mut tick,"token",CLOCK,PropertyOp::Lt,PropertyValue::Int(HORIZON as i64));
    effect(&mut tick,"token",CLOCK,EffectOpDecl::Add { value:PropertyValue::Int(1) });
    let mut raw = vec![tick];
    for (clock, before, after) in [(0,99,ACTIONS[action].unwrap_or(99) as i64),(8,10,99),(16,99,10)] {
        let enabled = arm != Arm::LiveDisabled && action != 0 && clock == 0;
        let mut rule = Rule { name:format!("lifecycle_seeded_edit_{clock}"), priority:200,
            enabled, ..Rule::default() };
        rule.lhs.nodes = vec!["root".into(),"literal".into(),"token".into()];
        rule.rhs.nodes = rule.lhs.nodes.clone();
        rule.lhs.edges = vec![vec!["root".into(),"literal".into()]];
        rule.rhs.edges = rule.lhs.edges.clone();
        rule.lhs.edge_labels.insert(0,Label(E_OWNS)); rule.rhs.edge_labels.insert(0,Label(E_OWNS));
        for (name,label) in [("root",L_RULE_ROOT),("literal",L_LABEL_LIT),("token",TOKEN)] {
            rule.lhs.node_labels.insert(name.into(),Label(label));
        }
        guard(&mut rule,"root",K_ENCODING,PropertyOp::Eq,PropertyValue::Int(COMPLETE_VERSION));
        guard(&mut rule,"literal",TARGET,PropertyOp::Eq,PropertyValue::Int(1));
        guard(&mut rule,"literal",K_LITERAL,PropertyOp::Eq,PropertyValue::Int(before));
        guard(&mut rule,"token",CLOCK,PropertyOp::Eq,PropertyValue::Int(clock));
        guard(&mut rule,"token",K_FUEL,PropertyOp::Gte,PropertyValue::Float(1.));
        guard(&mut rule,"token",K_FUEL,PropertyOp::Lte,PropertyValue::Float(3.));
        effect(&mut rule,"literal",K_LITERAL,EffectOpDecl::Set { value:PropertyValue::Int(after) });
        effect(&mut rule,"token",K_FUEL,EffectOpDecl::Subtract { value:PropertyValue::Float(1.) });
        effect(&mut rule,"token",CLOCK,EffectOpDecl::Add { value:PropertyValue::Int(1) });
        raw.push(rule);
    }
    let mut compiled = vec![];
    for rule in &raw {
        let mut c = compile_rule(rule).map_err(|e| e.to_string())?;
        c.meta_visible = true;
        for edge in &mut c.pattern.edges { edge.ordered = true; }
        compiled.push(c);
    }
    Ok((raw,compiled))
}

fn inventory(s: &HypergraphStore) -> Value {
    let mut nodes: Vec<_> = s.nodes().map(|(id,data)| {
        let mut incident: Vec<_> = s.edges_incident_to(id).iter().map(|id|format!("{id:?}")).collect();
        incident.sort(); (format!("{id:?}"),json!({"data":data,"incident":incident}))
    }).collect(); nodes.sort_by(|a,b|a.0.cmp(&b.0));
    let mut edges: Vec<_> = s.edges().map(|(id,e)|(format!("{id:?}"),json!(e))).collect();
    edges.sort_by(|a,b|a.0.cmp(&b.0));
    let labels: BTreeSet<_> = s.edges().map(|(_,e)|e.label.0).collect();
    let indexes: Vec<_> = labels.into_iter().map(|l| {
        let mut ids: Vec<_> = s.edges_with_label(Label(l)).iter().map(|id|format!("{id:?}")).collect();
        ids.sort(); (l,ids)
    }).collect();
    json!({"step":s.step(),"nodes":nodes,"edges":edges,"label_indexes":indexes})
}

struct Capture(Arc<Mutex<Vec<RewriteEvent>>>);
impl Observer for Capture {
    fn on_step_end(&mut self, _: &GraphSnapshot<'_>, _: &StepStats, events: &[RewriteEvent]) {
        *self.0.lock().unwrap() = events.to_vec();
    }
}

fn decoded(s: &HypergraphStore, root: NodeId) -> Result<Vec<Value>, String> {
    let p = read_complete_program(&s.snapshot(),root,&MetaLimits::default()).map_err(|v|format!("family {v:?}"))?;
    Ok(p.candidates().iter().map(candidate_record).collect())
}
fn table(engine: &Engine) -> Result<Vec<Value>, String> {
    if engine.rules().len()!=6 {return Err("unexpected world/editor table shape".into());}
    engine.rules()[..2].iter().map(|r| {
        if r.meta_visible || r.pattern.node_label_ceiling != Some(META_BASE) {return Err("world visibility drift".into());}
        CandidateRule::from_compiled(r,&MetaLimits::default()).map(|c|candidate_record(&c))
    }).collect()
}

/// Fixed-size native engineering cell. The caller must supply a shared deadline.
pub fn run(arm: Arm, context: u32, action: usize, seed: u64, deadline: Instant) -> Result<Value, String> {
    if !SEEDS.contains(&seed) {return Err("unregistered paid-choice seed".into());}
    if !ARMS.contains(&arm) || !CONTEXTS.contains(&context) || action >= ACTIONS.len() {
        return Err("unregistered choice arm/context/action".into());
    }
    let limits = MetaLimits::default(); let mut s = HypergraphStore::new_metadata_isolated();
    let nodes: Vec<_> = [context,11,12].into_iter().map(|l|s.add_node(NodeData::new(Label(l),0))).collect();
    for pair in [[nodes[0],nodes[1]],[nodes[1],nodes[2]]] { s.add_edge(HyperEdge::new(Label(0),pair.into_iter().collect())); }
    let label = if arm==Arm::PreseededLure {context} else {99};
    let core = candidate(true,context); let proposal = candidate(false,label);
    let root = encode_complete_family(&mut s,&core,&proposal,true,&limits).map_err(|v|format!("encode {v:?}"))?.root;
    // Follow typed schema links instead of calling family_parts, whose legacy
    // version3 precondition intentionally excludes complete version10 roots.
    let link = |role: u32, from: NodeId, slot: usize| -> Result<NodeId,String> {
        s.edges().find(|(_,e)|e.label==Label(role) && e.nodes[0]==from)
            .and_then(|(_,e)|e.nodes.get(slot).copied()).ok_or_else(||format!("missing schema link {role}"))
    };
    let allele = link(E_GRAFT,root,1)?;
    let left = link(E_BECOMES,allele,1)?;
    let pattern = link(E_MEMBER,left,1)?;
    let variable = link(E_SLOT_BASE,pattern,1)?;
    let literal = link(E_HASLABEL,variable,1)?;
    if s.get_node(literal).unwrap().get_property(K_LITERAL)!=Some(PropertyValue::Int(label as i64)) {
        return Err("proposal target does not select variable0".into());
    }
    s.get_node_mut(literal).unwrap().set_property(TARGET,PropertyValue::Int(1));
    let fuel = if arm==Arm::LiveUnpaid {0.} else {1.};
    let mut token = NodeData::new(Label(TOKEN),0);
    token.set_property(CLOCK,PropertyValue::Int(0)); token.set_property(K_FUEL,PropertyValue::Float(fuel));
    let token = s.add_node(token);
    let baseline = vec![candidate_record(&core),candidate_record(&proposal)];
    let before = inventory(&s); let world_nodes: Vec<_> = nodes.iter().map(|&id|s.get_node(id).unwrap().clone()).collect();
    let (raw_editors,editors) = editor_rules(arm,action)?;
    let mut source = GraphRuleSource::new(if arm==Arm::FrozenPaid {EpochPolicy::Never}
        else {EpochPolicy::Fixed(2.try_into().unwrap())},limits.clone(),false);
    source.complete_witness_enabled = true; source.editors = editors.clone();
    let mut engine = Engine::new(vec![],StepMode::MaxParallel,seed,16);
    engine.set_match_budget_policy(50000,50000,MatchBudgetPolicy::Pooled);
    engine.set_rule_source(Box::new(source));
    let capture = Arc::new(Mutex::new(vec![])); engine.add_observer(Box::new(Capture(capture.clone())));
    engine.compile_at_boundary(&s); engine.take_compile_report();
    let state = |s: &HypergraphStore| -> Value {
        let edges: Vec<_> = s.edges().filter(|(_,e)|e.nodes.iter().all(|n|nodes.contains(n)))
            .map(|(id,e)|json!({"id":id,"edge":e})).collect();
        json!({"literal":s.get_node(literal).unwrap().get_property(K_LITERAL),
            "fuel":s.get_node(token).unwrap().get_property(K_FUEL),
            "clock":s.get_node(token).unwrap().get_property(CLOCK),"world_edges":edges,"root_alive":s.get_node(root).is_some()})
    };
    let mut timeline = vec![json!({"step":0,"genotype":decoded(&s,root)?,"installed":table(&engine)?,
        "state":state(&s),"events":[],"paid_edits":0})];
    for step in 1..=HORIZON {
        if Instant::now()>=deadline {return Err(format!("native deadline before step {step}"));}
        let old = s.clone(); let rng_before = engine.rng_state_probe();
        engine.compile_at_boundary(&s);
        let compile = engine.take_compile_report(); let installed = table(&engine)?;
        capture.lock().unwrap().clear(); let stats = engine.step(&mut s);
        let rng_after = engine.rng_state_probe();
        if s.step()!=step || engine.match_budget_usage().truncated {return Err(format!("cap/nonadvance at step {step}"));}
        if rng_before!=rng_after {return Err("deterministic fixture consumed RNG".into());}
        for (id,original) in nodes.iter().zip(&world_nodes) {
            if serde_json::to_value(s.get_node(*id).unwrap()).unwrap()!=serde_json::to_value(original).unwrap() {
                return Err("world node identity/data changed".into());
            }
        }
        let events = capture.lock().unwrap().clone(); let mut records = vec![]; let mut paid = 0;
        let changed_nodes: Vec<_> = s.nodes().filter_map(|(id,data)| {
            let previous = old.get_node(id).unwrap();
            (serde_json::to_value(previous).unwrap()!=serde_json::to_value(data).unwrap())
                .then(||json!({"id":id,"before":previous,"after":data}))
        }).collect();
        for e in events {
            let world = e.rule_id<2;
            if world && e.binding.bound_nodes.iter().any(|n|!nodes.contains(n)) {return Err("meta/foreign world binding".into());}
            if !e.nodes_created.is_empty() || !e.nodes_deleted.is_empty() || !e.cleanup_neighbors.is_empty() {
                return Err("unexpected node creation/deletion/cleanup".into());
            }
            if !world && e.rule_name!="lifecycle_seeded_tick" {paid+=1;}
            let removed: Vec<_> = e.edges_deleted.iter().map(|id|json!({"id":id,"edge":old.get_edge(*id).unwrap()})).collect();
            let added: Vec<_> = e.edges_created.iter().map(|id|json!({"id":id,"edge":s.get_edge(*id).unwrap()})).collect();
            records.push(json!({"rule_id":e.rule_id,"rule_name":e.rule_name,"kind":if world {"world"} else {"seeded_editor"},
                "rcf":if world {installed[e.rule_id]["rcf"].clone()} else {Value::Null},
                "bound_nodes":e.binding.bound_nodes,"matched_edges":e.binding.matched_edges,
                "edges_deleted":removed,"edges_created":added,
                "nodes_created":e.nodes_created,"nodes_deleted":e.nodes_deleted,"cleanup_neighbors":e.cleanup_neighbors}));
        }
        // Preserve actual pre-step table at boundary step-1. Earlier after-step
        // rows initially held the table last used; replace only its measurement.
        timeline.last_mut().unwrap()["installed"] = json!(installed);
        timeline.push(json!({"step":step,"genotype":decoded(&s,root)?,"installed":table(&engine)?,
            "state":state(&s),"events":records,"paid_edits":paid,"stats":stats,
            "changed_nodes":changed_nodes,
            "compile_boundary":step-1,"compile_report":compile,"rng_before":rng_before,"rng_after":rng_after,
            "usage":engine.match_budget_usage()}));
    }
    let summary = summarize(&baseline,&timeline);
    Ok(json!({"profile":PROFILE,"classification":"scripted_engineering_fixture","arm":arm,"context":context,"action":action,"seed":seed,
        "horizon":HORIZON,"completed_steps":s.step(),"root":root,"target_literal":literal,"token":token,
        "ordinary_nodes":nodes,"initial_fuel":fuel,"initial_all_candidates":baseline,
        "seeded_editor_rules":raw_editors,"seeded_editor_compiled":editors,
        "initial_inventory":before,"final_inventory":inventory(&s),"timeline":timeline,
        "summary":summary,"training_admitted":false,"scientific_acceptance":false,"independent_physics_replay":false}))
}
