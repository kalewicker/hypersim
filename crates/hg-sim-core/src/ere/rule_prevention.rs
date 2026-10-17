//! Finite graph-request paid interception primitive; prior kernels and codecs are unchanged.
use super::lifecycle_fixture::candidate_record;
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
use rand::SeedableRng;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::BTreeSet,
    sync::{Arc, Mutex},
    time::Instant,
};

pub const PROFILE: &str = "E2-rule-prevention-primitive-v1";
pub const ORDINARY_CEILING: u32 = META_BASE;
pub const HORIZON: u64 = 96;
pub const SEED: u64 = 993748718001;
pub const HAZARD_STEPS: [u64; 4] = [16, 32, 48, 64];
const MARKER: u32 = 0x7702;
pub const DEFENSE: u32 = META_BASE + 0x7703;
pub const THREAT: u32 = META_BASE + 0x7704;
pub const PENDING: u32 = 0x7704;
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
pub const ARMS: [Arm; 5] = [
    Arm::Paid,
    Arm::Exhaustion,
    Arm::Unpaid,
    Arm::Disabled,
    Arm::NoHazard,
];
const TOKEN: u32 = META_BASE + 0x7700;
const CLOCK: u32 = 0x7700;
const TARGET: u32 = 0x7701;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Arm {
    Paid,
    Exhaustion,
    Unpaid,
    Disabled,
    NoHazard,
}
impl Arm {
    pub fn defense_fuel(self) -> f64 {
        match self {
            Self::Exhaustion => 2.,
            Self::Unpaid => 0.,
            _ => 4.,
        }
    }
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

fn editor_rules(
    arm: Arm,
    actions: [usize; 2],
    horizon: u64,
    maintenance: bool,
) -> Result<(Vec<Rule>, Vec<CompiledRule>), String> {
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
        PropertyValue::Int(horizon as i64),
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
    let order = [0, 1];
    for (stage, slot) in order.into_iter().enumerate() {
        let clock = [0, 4][stage];
        let before = [99, 98][slot];
        let after = CHOICES[slot][actions[slot]].unwrap_or(before);
        let mut rule = Rule {
            name: format!("composition_seeded_edit_{clock}_slot{slot}"),
            priority: 200,
            enabled: actions[slot] != 0,
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
    if maintenance {
        for completed in HAZARD_STEPS {
            let mut r = Rule {
                name: format!("prevention_request_{}", completed - 1),
                priority: 300,
                enabled: arm != Arm::NoHazard,
                ..Rule::default()
            };
            r.lhs.nodes = vec!["token".into(), "threat".into()];
            r.rhs.nodes = r.lhs.nodes.clone();
            for (v, l) in [("token", TOKEN), ("threat", THREAT)] {
                r.lhs.node_labels.insert(v.into(), Label(l));
            }
            guard(
                &mut r,
                "token",
                CLOCK,
                PropertyOp::Eq,
                PropertyValue::Int(completed as i64 - 2),
            );
            guard(
                &mut r,
                "threat",
                PENDING,
                PropertyOp::Eq,
                PropertyValue::Int(0),
            );
            effect(
                &mut r,
                "token",
                CLOCK,
                EffectOpDecl::Add {
                    value: PropertyValue::Int(1),
                },
            );
            effect(
                &mut r,
                "threat",
                PENDING,
                EffectOpDecl::Set {
                    value: PropertyValue::Int(1),
                },
            );
            raw.push(r);
        }
        for absorb in [false, true] {
            let mut r = Rule {
                name: if absorb {
                    "prevention_paid_absorb"
                } else {
                    "prevention_damage"
                }
                .into(),
                priority: if absorb { 400 } else { 300 },
                enabled: !absorb || arm != Arm::Disabled,
                ..Rule::default()
            };
            r.lhs.nodes = ["a", "b", "root", "token", "threat"]
                .into_iter()
                .map(str::to_owned)
                .collect();
            if absorb {
                r.lhs.nodes.push("defense".into());
            }
            r.rhs.nodes = r.lhs.nodes.clone();
            r.lhs.edges = vec![vec!["a".into(), "b".into(), "root".into()]];
            r.rhs.edges = r.lhs.edges.clone();
            if absorb {
                r.lhs.edge_labels.insert(0, Label(E_SLOT_BASE));
                r.rhs.edge_labels.insert(0, Label(E_SLOT_BASE));
            }
            for (v, l) in [
                ("a", L_PAT_EDGE),
                ("b", L_VAR),
                ("root", L_RULE_ROOT),
                ("token", TOKEN),
                ("threat", THREAT),
            ] {
                r.lhs.node_labels.insert(v.into(), Label(l));
            }
            for (v, k, x) in [("a", MARKER, 1), ("b", MARKER, 2), ("threat", PENDING, 1)] {
                guard(&mut r, v, k, PropertyOp::Eq, PropertyValue::Int(x));
            }
            effect(
                &mut r,
                "token",
                CLOCK,
                EffectOpDecl::Add {
                    value: PropertyValue::Int(1),
                },
            );
            effect(
                &mut r,
                "threat",
                PENDING,
                EffectOpDecl::Set {
                    value: PropertyValue::Int(0),
                },
            );
            if absorb {
                r.lhs.node_labels.insert("defense".into(), Label(DEFENSE));
                guard(
                    &mut r,
                    "defense",
                    K_FUEL,
                    PropertyOp::Gte,
                    PropertyValue::Float(1.),
                );
                guard(
                    &mut r,
                    "defense",
                    K_FUEL,
                    PropertyOp::Lte,
                    PropertyValue::Float(4.),
                );
                effect(
                    &mut r,
                    "defense",
                    K_FUEL,
                    EffectOpDecl::Subtract {
                        value: PropertyValue::Float(1.),
                    },
                );
            }
            raw.push(r);
        }
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
    let p = read_complete_program(&s.snapshot(), root, &MetaLimits::default()).ok();
    Ok(
        p.map(|p| p.candidates().iter().map(candidate_record).collect())
            .unwrap_or_default(),
    )
}
fn table(engine: &Engine) -> Result<Vec<Value>, String> {
    engine
        .rules()
        .iter()
        .filter(|r| !r.meta_visible && !r.name.starts_with("prevention_"))
        .map(|r| {
            if r.pattern.node_label_ceiling != Some(META_BASE) {
                return Err("world visibility drift".into());
            }
            CandidateRule::from_compiled(r, &MetaLimits::default()).map(|c| candidate_record(&c))
        })
        .collect()
}
pub fn run(arm: Arm, deadline: Instant) -> Result<Value, String> {
    if Instant::now() >= deadline {
        return Err("expired primitive deadline".into());
    }
    run_inner(arm, [10, 11], [1, 1], SEED, deadline, HORIZON, true)
}
/// Catalogue-only preflight; no graph mutation or native step.
pub fn catalogue(arm: Arm) -> Result<(Vec<Rule>, Vec<CompiledRule>), String> {
    editor_rules(arm, [1, 1], HORIZON, true)
}
/// Fixed-size native engineering cell. The caller must supply a shared deadline.
fn run_inner(
    arm: Arm,
    context: [u32; 2],
    actions: [usize; 2],
    seed: u64,
    deadline: Instant,
    horizon: u64,
    maintenance: bool,
) -> Result<Value, String> {
    if seed != SEED {
        return Err("unregistered composition seed".into());
    }
    if !ARMS.contains(&arm) || context != [10, 11] || actions != [1, 1] {
        return Err("unregistered composition arm/context/actions".into());
    }
    let limits = MetaLimits::default();
    let mut s = HypergraphStore::new_metadata_isolated();
    let (observation_nodes, nodes) = world(&mut s, context);
    let labels = [99, 98];
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
    let target_a = pattern;
    let target_b = link(E_SLOT_BASE, pattern, 1)?;
    let payload = s
        .edges()
        .find(|(_, e)| e.label == Label(E_SLOT_BASE) && e.nodes[..] == [target_a, target_b])
        .unwrap()
        .0;
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
    let fuel = 2.;
    let mut token = NodeData::new(Label(TOKEN), 0);
    token.set_property(CLOCK, PropertyValue::Int(0));
    token.set_property(K_FUEL, PropertyValue::Float(fuel));
    let token = s.add_node(token);
    let mut defense_data = NodeData::new(Label(DEFENSE), 0);
    defense_data.set_property(K_FUEL, PropertyValue::Float(arm.defense_fuel()));
    let defense = s.add_node(defense_data);
    let mut threat_data = NodeData::new(Label(THREAT), 0);
    threat_data.set_property(PENDING, PropertyValue::Int(0));
    let threat = s.add_node(threat_data);
    let baseline = vec![candidate_record(&core), candidate_record(&proposal)];
    let mut initialization = Value::Null;
    let mut repair_instruction = None;
    if maintenance {
        s.get_node_mut(target_a)
            .unwrap()
            .set_property(MARKER, PropertyValue::Int(1));
        s.get_node_mut(target_b)
            .unwrap()
            .set_property(MARKER, PropertyValue::Int(2));
        let seal = encode_complete_instruction(
            &mut s,
            &MetaInstructionSeed {
                operation: MetaOperation::Seal,
                fuel: 1.,
            },
        )
        .map_err(|v| format!("seal instruction {v:?}"))?;
        let repair = encode_complete_instruction(
            &mut s,
            &MetaInstructionSeed {
                operation: MetaOperation::RepairSealed,
                fuel: 0.,
            },
        )
        .map_err(|v| format!("repair instruction {v:?}"))?;
        repair_instruction = Some(repair);
        let before = inventory(&s);
        let expected = decoded(&s, root)?;
        let mut chosen = None;
        for mut rule in compile_complete_operations(MetaOperation::Seal, &limits)
            .map_err(|v| format!("{v:?}"))?
        {
            rule.pattern.match_work_limit = Some(50000);
            rule.pattern.match_result_limit = Some(50000);
            let (bindings, usage) = hg_engine::find_bindings_counted(&s.snapshot(), &rule, 0);
            if usage.enumeration_truncated {
                return Err("seal enumeration cap".into());
            }
            if let Some(binding) = bindings
                .into_iter()
                .find(|b| b.matched_edges.contains(&payload))
            {
                chosen = Some((rule, binding));
                break;
            }
        }
        let (rule, binding) = chosen.ok_or("no designated proposal E_SLOT0 Seal binding")?;
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
        let probe = rng.clone();
        hg_engine::apply_binding(&mut s, &rule, &binding, &mut rng);
        if rng != probe
            || s.step() != 0
            || decoded(&s, root)? != expected
            || read_complete_instruction(&s.snapshot(), seal).unwrap().fuel != 0.
        {
            return Err("seal initialization changed program/RNG/step/payment".into());
        }
        initialization = json!({"step":0,"paid_applications":1,"role":E_SLOT_BASE,
            "endpoints":[target_a,target_b,root],"seal_instruction":seal,"repair_instruction":repair,
            "original_payload":payload,"compiled_rule":rule,"binding":{"bound_nodes":binding.bound_nodes,"matched_edges":binding.matched_edges},
            "before_inventory":before,"after_inventory":inventory(&s),"rng_unchanged":true});
    }
    let before = inventory(&s);
    let world_nodes: Vec<_> = observation_nodes
        .iter()
        .map(|&id| s.get_node(id).unwrap().clone())
        .collect();
    let (raw_editors, editors) = editor_rules(arm, actions, horizon, maintenance)?;
    let mut source = GraphRuleSource::new(
        EpochPolicy::Fixed(2.try_into().unwrap()),
        limits.clone(),
        false,
    );
    source.complete_witness_enabled = true;
    source.vocabulary_enabled = maintenance;
    source.witness_repair_enabled = maintenance;
    source.sealed_repair_enabled = maintenance;
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
            "clock":s.get_node(token).unwrap().get_property(CLOCK),"world_edges":edges,"root_alive":s.get_node(root).is_some(),
            "repair_fuel":repair_instruction.map(|id|s.get_node(id).unwrap().get_property(K_FUEL)),
            "defense_fuel":s.get_node(defense).unwrap().get_property(K_FUEL),
            "pending_threat":s.get_node(threat).unwrap().get_property(PENDING),
            "target_payload_label":if maintenance {s.edges().find(|(_,e)| e.nodes[..]==[target_a,target_b,root]).map(|(_,e)|e.label.0)} else {None}})
    };
    let mut timeline = vec![
        json!({"step":0,"genotype":decoded(&s,root)?,"installed":table(&engine)?,
        "state":state(&s),"events":[],"paid_edits":0}),
    ];
    let mut table_catalogue: Vec<Value> = vec![];
    for step in 1..=horizon {
        if Instant::now() >= deadline {
            return Err(format!("native deadline before step {step}"));
        }
        let old = s.clone();
        let rng_before = engine.rng_state_probe();
        engine.compile_at_boundary(&s);
        let compile = engine.take_compile_report();
        let installed = table(&engine)?;
        let pre_step_rules = engine.rules().to_vec();
        let table_value = serde_json::to_value(&pre_step_rules).unwrap();
        let table_index = if let Some(i) = table_catalogue.iter().position(|v| *v == table_value) {
            i
        } else {
            table_catalogue.push(table_value);
            table_catalogue.len() - 1
        };
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
            let rule = &pre_step_rules[e.rule_id];
            let world = !rule.meta_visible && !rule.name.starts_with("prevention_");
            if world
                && (e.binding.bound_nodes.iter().any(|n| !nodes.contains(n))
                    || e.binding
                        .bound_nodes
                        .iter()
                        .copied()
                        .collect::<BTreeSet<_>>()
                        .len()
                        != 3)
            {
                return Err("meta/foreign world binding".into());
            }
            if !e.nodes_created.is_empty()
                || !e.nodes_deleted.is_empty()
                || !e.cleanup_neighbors.is_empty()
            {
                return Err("unexpected node creation/deletion/cleanup".into());
            }
            if !world && e.rule_name.starts_with("composition_seeded_edit") {
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
            records.push(json!({"rule_id":e.rule_id,"rule_name":e.rule_name,"kind":if world {"world"} else if e.rule_name.starts_with("complete_encoded_meta_") {"encoded_repair"} else if e.rule_name=="prevention_damage" {"hazard"} else if e.rule_name=="prevention_paid_absorb" {"absorb"} else if e.rule_name.starts_with("prevention_request_") {"request"} else {"seeded_editor"},
                "rcf":if world {candidate_record(&CandidateRule::from_compiled(rule,&limits)?)["rcf"].clone()} else {Value::Null},
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
            "pre_step_table":table_index,"compile_boundary":step-1,"compile_report":compile,"rng_before":rng_before,"rng_after":rng_after,
            "usage":engine.match_budget_usage()}));
    }
    let summary = json!({"birth_payments":timeline.iter().map(|r|r["paid_edits"].as_u64().unwrap()).sum::<u64>(),
        "absorbed_threats":timeline.iter().flat_map(|r|r["events"].as_array().unwrap()).filter(|e|e["kind"]=="absorb").count(),
        "issued_threats":timeline.iter().flat_map(|r|r["events"].as_array().unwrap()).filter(|e|e["kind"]=="request").count(),
        "hazard_events":timeline.iter().flat_map(|r|r["events"].as_array().unwrap()).filter(|e|e["kind"]=="hazard").count(),
        "repair_payments":timeline.iter().flat_map(|r|r["events"].as_array().unwrap()).filter(|e|e["kind"]=="encoded_repair").count()});
    Ok(
        json!({"profile":PROFILE,"classification":"scripted_engineering_fixture","arm":arm,"context":context,"actions":actions,"seed":seed,
        "horizon":horizon,"maintenance":maintenance,"initialization":initialization,"hazard_schedule":if maintenance && arm!=Arm::NoHazard {HAZARD_STEPS.to_vec()} else {vec![]},"completed_steps":s.step(),"defense":defense,"threat":threat,"initial_defense_fuel":arm.defense_fuel(),"root":root,"target_literals":literals,"token":token,
        "ordinary_nodes":nodes,"observation_nodes":observation_nodes,"initial_fuel":fuel,"initial_all_candidates":baseline,
        "seeded_editor_rules":raw_editors,"seeded_editor_compiled":editors,
        "initial_inventory":before,"final_inventory":inventory(&s),"timeline":timeline,
        "table_catalogue":table_catalogue,"summary":summary,"sustained_claim":false,"training_admitted":false,"scientific_acceptance":false,"independent_physics_replay":false}),
    )
}
