//! Relation-sensitive seven-node fixture. Versioned fork; existing composition physics is preserved.
use super::lifecycle_fixture::{candidate_record, summarize};
use hg_core::{
    GraphSnapshot, HyperEdge, HypergraphStore, Label, NodeData, NodeId, PropertyOp, PropertyValue,
};
use hg_engine::Engine;
use hg_meta::*;
use hg_observe::{Observer, RewriteEvent, StepStats};
use hg_rules::{
    compile_rule, CompiledRule, EffectOpDecl, EpochPolicy, MatchBudgetPolicy, NodePropertyGuard,
    PropertyEffectDecl, Rule, StepMode,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::BTreeSet,
    sync::{Arc, Mutex},
    time::Instant,
};

pub const PROFILE: &str = "E2-rule-relational-v1";
pub const ORDINARY_CEILING: u32 = META_BASE;
pub const HORIZON: u64 = 24;
pub const SEEDS: [u64; 2] = [993748718001, 993748718002];
pub const CONTEXTS: [[u32; 2]; 9] = [
    [10, 11],
    [10, 21],
    [10, 31],
    [20, 11],
    [20, 21],
    [20, 31],
    [30, 11],
    [30, 21],
    [30, 31],
];
pub const ORDINARY_LABELS: [u32; 7] = [10, 20, 30, 11, 21, 31, 12];
pub const CHOICES: [[Option<u32>; 4]; 2] = [
    [None, Some(10), Some(20), Some(30)],
    [None, Some(11), Some(21), Some(31)],
];
pub const ARMS: [Arm; 7] = [
    Arm::LivePaid,
    Arm::FrozenPaid,
    Arm::LiveUnpaid,
    Arm::LiveDisabled,
    Arm::PreseededLure,
    Arm::OneFuel,
    Arm::ReversePaid,
];
const TOKEN: u32 = META_BASE + 0x7700;
const CLOCK: u32 = 0x7700;
const TARGET: u32 = 0x7701;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Arm {
    LivePaid,
    FrozenPaid,
    LiveUnpaid,
    LiveDisabled,
    PreseededLure,
    OneFuel,
    ReversePaid,
}

fn guard(rule: &mut Rule, var: &str, key: u32, op: PropertyOp, value: PropertyValue) {
    rule.lhs_node_property_guards.push(NodePropertyGuard {
        var: var.into(),
        property: key,
        op,
        value,
    });
}
fn effect(rule: &mut Rule, var: &str, key: u32, op: EffectOpDecl) {
    rule.rhs_property_effects.push(PropertyEffectDecl {
        var: var.into(),
        key,
        op,
    });
}

pub fn candidate(core: bool, labels: [u32; 2]) -> CandidateRule {
    let mut c = super::lifecycle_fixture::candidate(core, labels[0]);
    c.labels[1] = Some(labels[1]);
    c
}

fn editor_rules(arm: Arm, actions: [usize; 2]) -> Result<(Vec<Rule>, Vec<CompiledRule>), String> {
    let mut tick = Rule {
        name: "composition_seeded_tick".into(),
        priority: 100,
        ..Rule::default()
    };
    tick.lhs.nodes = vec!["token".into()];
    tick.rhs.nodes = tick.lhs.nodes.clone();
    tick.lhs.node_labels.insert("token".into(), Label(TOKEN));
    guard(
        &mut tick,
        "token",
        CLOCK,
        PropertyOp::Lt,
        PropertyValue::Int(HORIZON as i64),
    );
    effect(
        &mut tick,
        "token",
        CLOCK,
        EffectOpDecl::Add {
            value: PropertyValue::Int(1),
        },
    );
    let mut raw = vec![tick];
    let order = if arm == Arm::ReversePaid {
        [1, 0]
    } else {
        [0, 1]
    };
    for (stage, slot) in order.into_iter().enumerate() {
        let clock = [0, 4][stage];
        let before = [99, 98][slot];
        let after = CHOICES[slot][actions[slot]].unwrap_or(before);
        let mut rule = Rule {
            name: format!("composition_seeded_edit_{clock}_slot{slot}"),
            priority: 200,
            enabled: arm != Arm::LiveDisabled && actions[slot] != 0,
            ..Rule::default()
        };
        rule.lhs.nodes = vec!["root".into(), "literal".into(), "token".into()];
        rule.rhs.nodes = rule.lhs.nodes.clone();
        rule.lhs.edges = vec![vec!["root".into(), "literal".into()]];
        rule.rhs.edges = rule.lhs.edges.clone();
        rule.lhs.edge_labels.insert(0, Label(E_OWNS));
        rule.rhs.edge_labels.insert(0, Label(E_OWNS));
        for (name, label) in [
            ("root", L_RULE_ROOT),
            ("literal", L_LABEL_LIT),
            ("token", TOKEN),
        ] {
            rule.lhs.node_labels.insert(name.into(), Label(label));
        }
        guard(
            &mut rule,
            "root",
            K_ENCODING,
            PropertyOp::Eq,
            PropertyValue::Int(COMPLETE_VERSION),
        );
        guard(
            &mut rule,
            "literal",
            TARGET,
            PropertyOp::Eq,
            PropertyValue::Int(slot as i64 + 1),
        );
        guard(
            &mut rule,
            "literal",
            K_LITERAL,
            PropertyOp::Eq,
            PropertyValue::Int(before as i64),
        );
        guard(
            &mut rule,
            "token",
            CLOCK,
            PropertyOp::Eq,
            PropertyValue::Int(clock),
        );
        guard(
            &mut rule,
            "token",
            K_FUEL,
            PropertyOp::Gte,
            PropertyValue::Float(1.),
        );
        guard(
            &mut rule,
            "token",
            K_FUEL,
            PropertyOp::Lte,
            PropertyValue::Float(2.),
        );
        effect(
            &mut rule,
            "literal",
            K_LITERAL,
            EffectOpDecl::Set {
                value: PropertyValue::Int(after as i64),
            },
        );
        effect(
            &mut rule,
            "token",
            K_FUEL,
            EffectOpDecl::Subtract {
                value: PropertyValue::Float(1.),
            },
        );
        effect(
            &mut rule,
            "token",
            CLOCK,
            EffectOpDecl::Add {
                value: PropertyValue::Int(1),
            },
        );
        raw.push(rule);
    }
    let mut compiled = vec![];
    for rule in &raw {
        let mut c = compile_rule(rule).map_err(|e| e.to_string())?;
        c.meta_visible = true;
        for edge in &mut c.pattern.edges {
            edge.ordered = true;
        }
        compiled.push(c);
    }
    Ok((raw, compiled))
}

fn inventory(s: &HypergraphStore) -> Value {
    let mut nodes: Vec<_> = s
        .nodes()
        .map(|(id, data)| {
            let mut incident: Vec<_> = s
                .edges_incident_to(id)
                .iter()
                .map(|id| format!("{id:?}"))
                .collect();
            incident.sort();
            (format!("{id:?}"), json!({"data":data,"incident":incident}))
        })
        .collect();
    nodes.sort_by(|a, b| a.0.cmp(&b.0));
    let mut edges: Vec<_> = s
        .edges()
        .map(|(id, e)| (format!("{id:?}"), json!(e)))
        .collect();
    edges.sort_by(|a, b| a.0.cmp(&b.0));
    let labels: BTreeSet<_> = s.edges().map(|(_, e)| e.label.0).collect();
    let indexes: Vec<_> = labels
        .into_iter()
        .map(|l| {
            let mut ids: Vec<_> = s
                .edges_with_label(Label(l))
                .iter()
                .map(|id| format!("{id:?}"))
                .collect();
            ids.sort();
            (l, ids)
        })
        .collect();
    json!({"step":s.step(),"nodes":nodes,"edges":edges,"label_indexes":indexes})
}

fn world(s: &mut HypergraphStore, context: [u32; 2]) -> (Vec<NodeId>, Vec<NodeId>) {
    let all: Vec<_> = ORDINARY_LABELS
        .into_iter()
        .map(|l| s.add_node(NodeData::new(Label(l), 0)))
        .collect();
    let active = vec![
        all[ORDINARY_LABELS
            .iter()
            .position(|l| *l == context[0])
            .unwrap()],
        all[ORDINARY_LABELS
            .iter()
            .position(|l| *l == context[1])
            .unwrap()],
        all[6],
    ];
    for pair in [[active[0], active[1]], [active[1], active[2]]] {
        s.add_edge(HyperEdge::new(Label(0), pair.into_iter().collect()));
    }
    (all, active)
}

/// Pure initialization observation: no Engine invocation or native steps.
pub fn initial_observation(context: [u32; 2]) -> Result<Value, String> {
    if !CONTEXTS.contains(&context) {
        return Err("unregistered relational context".into());
    }
    let mut s = HypergraphStore::new_metadata_isolated();
    world(&mut s, context);
    Ok(inventory(&s))
}

struct Capture(Arc<Mutex<Vec<RewriteEvent>>>);
impl Observer for Capture {
    fn on_step_end(&mut self, _: &GraphSnapshot<'_>, _: &StepStats, events: &[RewriteEvent]) {
        *self.0.lock().unwrap() = events.to_vec();
    }
}

fn decoded(s: &HypergraphStore, root: NodeId) -> Result<Vec<Value>, String> {
    let p = read_complete_program(&s.snapshot(), root, &MetaLimits::default())
        .map_err(|v| format!("family {v:?}"))?;
    Ok(p.candidates().iter().map(candidate_record).collect())
}
fn table(engine: &Engine) -> Result<Vec<Value>, String> {
    if engine.rules().len() != 5 {
        return Err("unexpected world/editor table shape".into());
    }
    engine.rules()[..2]
        .iter()
        .map(|r| {
            if r.meta_visible || r.pattern.node_label_ceiling != Some(META_BASE) {
                return Err("world visibility drift".into());
            }
            CandidateRule::from_compiled(r, &MetaLimits::default()).map(|c| candidate_record(&c))
        })
        .collect()
}

/// Fixed-size native engineering cell. The caller must supply a shared deadline.
pub fn run(
    arm: Arm,
    context: [u32; 2],
    actions: [usize; 2],
    seed: u64,
    deadline: Instant,
) -> Result<Value, String> {
    if !SEEDS.contains(&seed) {
        return Err("unregistered composition seed".into());
    }
    if !ARMS.contains(&arm) || !CONTEXTS.contains(&context) || actions.iter().any(|a| *a >= 4) {
        return Err("unregistered composition arm/context/actions".into());
    }
    let limits = MetaLimits::default();
    let mut s = HypergraphStore::new_metadata_isolated();
    let (observation_nodes, nodes) = world(&mut s, context);
    let labels = if arm == Arm::PreseededLure {
        context
    } else {
        [99, 98]
    };
    let core = candidate(true, context);
    let proposal = candidate(false, labels);
    let root = encode_complete_family(&mut s, &core, &proposal, true, &limits)
        .map_err(|v| format!("encode {v:?}"))?
        .root;
    // Follow typed schema links instead of calling family_parts, whose legacy
    // version3 precondition intentionally excludes complete version10 roots.
    let link = |role: u32, from: NodeId, slot: usize| -> Result<NodeId, String> {
        s.edges()
            .find(|(_, e)| e.label == Label(role) && e.nodes[0] == from)
            .and_then(|(_, e)| e.nodes.get(slot).copied())
            .ok_or_else(|| format!("missing schema link {role}"))
    };
    let allele = link(E_GRAFT, root, 1)?;
    let left = link(E_BECOMES, allele, 1)?;
    let pattern = link(E_MEMBER, left, 1)?;
    let literals: Vec<_> = (0..2)
        .map(|slot| {
            let variable = link(E_SLOT_BASE + slot as u32, pattern, 1)?;
            link(E_HASLABEL, variable, 1)
        })
        .collect::<Result<_, String>>()?;
    for (slot, &literal) in literals.iter().enumerate() {
        if s.get_node(literal).unwrap().get_property(K_LITERAL)
            != Some(PropertyValue::Int(labels[slot] as i64))
        {
            return Err("wrong proposal literal target".into());
        }
        s.get_node_mut(literal)
            .unwrap()
            .set_property(TARGET, PropertyValue::Int(slot as i64 + 1));
    }
    let fuel = if arm == Arm::LiveUnpaid {
        0.
    } else if arm == Arm::OneFuel {
        1.
    } else {
        2.
    };
    let mut token = NodeData::new(Label(TOKEN), 0);
    token.set_property(CLOCK, PropertyValue::Int(0));
    token.set_property(K_FUEL, PropertyValue::Float(fuel));
    let token = s.add_node(token);
    let baseline = vec![candidate_record(&core), candidate_record(&proposal)];
    let before = inventory(&s);
    let world_nodes: Vec<_> = observation_nodes
        .iter()
        .map(|&id| s.get_node(id).unwrap().clone())
        .collect();
    let (raw_editors, editors) = editor_rules(arm, actions)?;
    let mut source = GraphRuleSource::new(
        if arm == Arm::FrozenPaid {
            EpochPolicy::Never
        } else {
            EpochPolicy::Fixed(2.try_into().unwrap())
        },
        limits.clone(),
        false,
    );
    source.complete_witness_enabled = true;
    source.editors = editors.clone();
    let mut engine = Engine::new(vec![], StepMode::MaxParallel, seed, 16);
    engine.set_match_budget_policy(50000, 50000, MatchBudgetPolicy::Pooled);
    engine.set_rule_source(Box::new(source));
    let capture = Arc::new(Mutex::new(vec![]));
    engine.add_observer(Box::new(Capture(capture.clone())));
    engine.compile_at_boundary(&s);
    engine.take_compile_report();
    let state = |s: &HypergraphStore| -> Value {
        let edges: Vec<_> = s
            .edges()
            .filter(|(_, e)| e.nodes.iter().all(|n| observation_nodes.contains(n)))
            .map(|(id, e)| json!({"id":id,"edge":e}))
            .collect();
        json!({"literals":literals.iter().map(|literal|s.get_node(*literal).unwrap().get_property(K_LITERAL)).collect::<Vec<_>>(),
            "fuel":s.get_node(token).unwrap().get_property(K_FUEL),
            "clock":s.get_node(token).unwrap().get_property(CLOCK),"world_edges":edges,"root_alive":s.get_node(root).is_some()})
    };
    let mut timeline = vec![
        json!({"step":0,"genotype":decoded(&s,root)?,"installed":table(&engine)?,
        "state":state(&s),"events":[],"paid_edits":0}),
    ];
    for step in 1..=HORIZON {
        if Instant::now() >= deadline {
            return Err(format!("native deadline before step {step}"));
        }
        let old = s.clone();
        let rng_before = engine.rng_state_probe();
        engine.compile_at_boundary(&s);
        let compile = engine.take_compile_report();
        let installed = table(&engine)?;
        capture.lock().unwrap().clear();
        let stats = engine.step(&mut s);
        let rng_after = engine.rng_state_probe();
        if s.step() != step || engine.match_budget_usage().truncated {
            return Err(format!("cap/nonadvance at step {step}"));
        }
        if rng_before != rng_after {
            return Err("deterministic fixture consumed RNG".into());
        }
        for (id, original) in observation_nodes.iter().zip(&world_nodes) {
            if serde_json::to_value(s.get_node(*id).unwrap()).unwrap()
                != serde_json::to_value(original).unwrap()
            {
                return Err("world node identity/data changed".into());
            }
        }
        let events = capture.lock().unwrap().clone();
        let mut records = vec![];
        let mut paid = 0;
        let changed_nodes: Vec<_> = s
            .nodes()
            .filter_map(|(id, data)| {
                let previous = old.get_node(id).unwrap();
                (serde_json::to_value(previous).unwrap() != serde_json::to_value(data).unwrap())
                    .then(|| json!({"id":id,"before":previous,"after":data}))
            })
            .collect();
        for e in events {
            let world = e.rule_id < 2;
            if world && e.binding.bound_nodes.iter().any(|n| !nodes.contains(n)) {
                return Err("meta/foreign world binding".into());
            }
            if !e.nodes_created.is_empty()
                || !e.nodes_deleted.is_empty()
                || !e.cleanup_neighbors.is_empty()
            {
                return Err("unexpected node creation/deletion/cleanup".into());
            }
            if !world && e.rule_name != "composition_seeded_tick" {
                paid += 1;
            }
            let removed: Vec<_> = e
                .edges_deleted
                .iter()
                .map(|id| json!({"id":id,"edge":old.get_edge(*id).unwrap()}))
                .collect();
            let added: Vec<_> = e
                .edges_created
                .iter()
                .map(|id| json!({"id":id,"edge":s.get_edge(*id).unwrap()}))
                .collect();
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
    let summary = summarize(&baseline, &timeline);
    Ok(
        json!({"profile":PROFILE,"classification":"scripted_engineering_fixture","arm":arm,"context":context,"actions":actions,"seed":seed,
        "horizon":HORIZON,"completed_steps":s.step(),"root":root,"target_literals":literals,"token":token,
        "ordinary_nodes":nodes,"observation_nodes":observation_nodes,"initial_fuel":fuel,"initial_all_candidates":baseline,
        "seeded_editor_rules":raw_editors,"seeded_editor_compiled":editors,
        "initial_inventory":before,"final_inventory":inventory(&s),"timeline":timeline,
        "summary":summary,"training_admitted":false,"scientific_acceptance":false,"independent_physics_replay":false}),
    )
}
