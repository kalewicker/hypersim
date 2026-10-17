#![cfg(feature = "hg-meta")]
//! Bounded, separately frozen semantic gates. Never used for training admission.
#[allow(dead_code)]
#[path = "support/complete.rs"]
mod complete;
#[allow(dead_code)]
#[path = "support/witness.rs"]
mod witness;
use hg_core::{HyperEdge, HypergraphStore, Label, NodeData, NodeId, PropertyOp, PropertyValue};
use hg_engine::Engine;
use hg_meta::*;
use hg_rules::{EpochPolicy, MatchBudgetPolicy, RuleSetDelta, RuleSource, StepMode};
use hg_schema::MetricsRow;
use hg_sim_core::ere::{
    experiments, meta_excluded_store, EreArm, EreConfig, EreRuntime, EreSeedFamily,
    MetaVocabularyConfig,
};
use serde_json::{json, Value};
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

const PROFILE: &str = "E2-complete-deterministic-gates-v1";
const VALIDATORS: [&str; 9] = [
    "Region",
    "Sides",
    "Members",
    "Slots",
    "Variables",
    "Labels",
    "Guards",
    "Bounds",
    "Compilability",
];
fn hex(bytes: Vec<u8>) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn save(path: &Path, value: &Value) {
    std::fs::write(path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
}
fn candidate(c: &CandidateRule) -> Value {
    let typed = |v| match v {
        PropertyValue::Int(n) => json!({"Int": n}),
        PropertyValue::Float(n) => json!({"Float": n}),
        PropertyValue::Bool(n) => json!({"Bool": n}),
    };
    json!({"priority": c.priority, "labels": c.labels, "lhs": c.lhs, "rhs": c.rhs,
        "guards": c.guards.iter().map(|g| json!({"var":g.var,"key":g.key,"op":format!("{:?}",g.op),"value":typed(g.value)})).collect::<Vec<_>>(),
        "rcf":hex(c.rcf_bytes()),"execution":hex(c.execution_bytes())})
}
fn program(p: &RuleProgram) -> Value {
    json!({"core":candidate(&p.core),"proposal":p.proposal.as_ref().map(candidate),"active":p.active,
        "rcf":hex(p.rcf_bytes()),"execution":hex(p.execution_bytes())})
}
fn source(limits: MetaLimits) -> GraphRuleSource {
    let mut src = GraphRuleSource::new(EpochPolicy::Fixed(1.try_into().unwrap()), limits, false);
    src.complete_witness_enabled = true;
    src
}
fn compile(src: &mut GraphRuleSource, s: &HypergraphStore) -> Value {
    let before = bincode::serialize(s).unwrap();
    let value = match src.rules(&s.snapshot(), s.step()) {
        RuleSetDelta::Replace { rules, report } => json!({"report":report,
            "table":rules.iter().map(|r| candidate(&CandidateRule::from_compiled(r,&MetaLimits::default()).unwrap())).collect::<Vec<_>>() }),
        RuleSetDelta::Unchanged => panic!("fixed1 source must emit a report"),
    };
    assert_eq!(before, bincode::serialize(s).unwrap());
    value
}
fn small(i: u32) -> CandidateRule {
    CandidateRule {
        priority: 0,
        labels: vec![Some(10 + i), Some(100 + i), Some(200 + i)],
        lhs: vec![vec![0, 1], vec![1, 2]],
        rhs: vec![vec![0, 2], vec![0, 1]],
        guards: vec![Guard {
            var: 0,
            key: 42,
            op: PropertyOp::Gte,
            value: PropertyValue::Int(0),
        }],
    }
}
fn proposal(c: &CandidateRule) -> CandidateRule {
    let mut p = c.clone();
    std::mem::swap(&mut p.lhs, &mut p.rhs);
    p.priority += 1;
    p
}
fn renamed(c: &CandidateRule) -> CandidateRule {
    let n = c.labels.len();
    let mut p = c.clone();
    p.labels.reverse();
    for edges in [&mut p.lhs, &mut p.rhs] {
        for e in edges.iter_mut() {
            for v in e {
                *v = n - 1 - *v;
            }
        }
        edges.reverse();
    }
    for g in &mut p.guards {
        g.var = n - 1 - g.var;
    }
    p
}
fn encoded(a: &CandidateRule, b: &CandidateRule, prefix: usize) -> (HypergraphStore, NodeId) {
    let mut s = HypergraphStore::new();
    let mut garbage = vec![];
    for i in 0..prefix {
        garbage.push(s.add_node(NodeData::new(Label(i as u32), 0)));
    }
    for id in garbage {
        s.remove_node(id);
    }
    // Keep an unrelated allocation prefix too; neither may enter the family.
    for _ in 0..prefix / 2 {
        s.add_node(NodeData::new(Label(900), 0));
    }
    let root = encode_complete_family(&mut s, a, b, true, &MetaLimits::default())
        .unwrap()
        .root;
    (s, root)
}
fn r0() -> Value {
    let mut rows = vec![];
    for i in 0..20 {
        let c = small(i);
        let p = proposal(&c);
        let mut distinct = c.clone();
        distinct.priority += 1;
        let mut variants = vec![];
        for (name, a, b, prefix) in [
            ("base", c.clone(), p.clone(), 0),
            ("renamed", renamed(&c), renamed(&p), i as usize + 7),
            ("priority", distinct.clone(), p.clone(), i as usize + 13),
        ] {
            let (s, root) = encoded(&a, &b, prefix);
            let decoded = witness::decoded(&s, root).unwrap();
            variants.push(json!({"name":name,"prefix":prefix,"root":root,"input":program(&RuleProgram{core:a,proposal:Some(b),active:true}),"decoded":program(&decoded),"inventory":witness::inventory(&s)}));
        }
        let mut orders = vec![];
        for reverse in [false, true] {
            let mut s = HypergraphStore::new();
            let second = small(i + 30);
            for a in if reverse {
                vec![&second, &c]
            } else {
                vec![&c, &second]
            } {
                encode_complete_family(&mut s, a, &proposal(a), true, &MetaLimits::default())
                    .unwrap();
            }
            let mut engine = Engine::new(vec![], StepMode::MaxParallel, 42, 16);
            engine.set_rule_source(Box::new(source(MetaLimits::default())));
            let before = bincode::serialize(&s).unwrap();
            let rng = engine.rng_state_probe();
            engine.compile_at_boundary(&s);
            let report = engine.take_compile_report().unwrap();
            orders.push(json!({"reverse":reverse,"report":report,"table":engine.rules().iter().map(|r|candidate(&CandidateRule::from_compiled(r,&MetaLimits::default()).unwrap())).collect::<Vec<_>>(),
                "rng_before":format!("{rng:016x}"),"rng_after":format!("{:016x}",engine.rng_state_probe()),"read_only":before==bincode::serialize(&s).unwrap(),"inventory":witness::inventory(&s)}));
        }
        let pass = variants[0]["decoded"]["rcf"] == variants[1]["decoded"]["rcf"]
            && variants[0]["decoded"]["rcf"] != variants[2]["decoded"]["rcf"]
            && variants.iter().all(|v| v["input"] == v["decoded"])
            && orders[0]["table"] == orders[1]["table"]
            && orders[0]["report"]["classes"] == orders[1]["report"]["classes"]
            && orders
                .iter()
                .all(|v| v["read_only"] == true && v["rng_before"] == v["rng_after"]);
        rows.push(json!({"index":i,"variants":variants,"orders":orders,"pass":pass}));
    }
    let (mut s, root) = encoded(&small(0), &proposal(&small(0)), 11);
    let op = complete::instruction(&mut s, MetaOperation::Seal, 1.0);
    let edge = s.edges().find(|(_, e)| e.label == Label(E_CORE)).unwrap().0;
    let before = witness::inventory(&s);
    let decoded_before = program(&witness::decoded(&s, root).unwrap());
    complete::seal(&mut s, edge, op, 42);
    let after = witness::inventory(&s);
    let decoded_after = program(&witness::decoded(&s, root).unwrap());
    let seal_pass =
        before != after && decoded_before == decoded_after && complete::fuel(&s, op) == 0.0;
    json!({"gate":"G-R0","pass":rows.iter().all(|r|r["pass"]==true)&&seal_pass,"rows":rows,
        "paid_seal":{"root":root,"instruction":op,"fuel_before":1.0,"fuel_after":complete::fuel(&s,op),"before":before,"after":after,"decoded_before":decoded_before,"decoded_after":decoded_after,"pass":seal_pass}})
}
fn all_role_fixture() -> (HypergraphStore, NodeId) {
    let mut s = HypergraphStore::new();
    let c = witness::candidate();
    let mut p = c.clone();
    p.priority = 1;
    let root = encode_complete_family(&mut s, &c, &p, true, &MetaLimits::default())
        .unwrap()
        .root;
    (s, root)
}
fn r4() -> Value {
    let (base, root) = all_role_fixture();
    let names = [
        "region_cap",
        "reverse_sides",
        "missing_ownership",
        "arity_cap",
        "bad_order",
        "bad_literal",
        "bad_guard",
        "pattern_cap",
        "bad_priority",
        "duplicate_ownership",
        "reverse_ownership",
        "wrong_schema",
        "missing_schema",
        "wrong_obligation",
        "missing_obligation",
        "duplicate_obligation",
        "owned_orphan",
        "unexpected_topology",
    ];
    let mut cases = vec![];
    for ix in 0..names.len() + 13 {
        let name = if ix < names.len() {
            names[ix].to_string()
        } else {
            format!("double_erasure_role{:02}", ix - names.len())
        };
        let mut s = base.clone();
        let mut limits = MetaLimits::default();
        let mut src = source(limits.clone());
        let primed = compile(&mut src, &s);
        let edge = s.edges().find(|(_, e)| e.label == Label(E_OWNS)).unwrap().0;
        let variable = s.nodes().find(|(_, n)| n.label == Label(L_VAR)).unwrap().0;
        let literal = s
            .nodes()
            .find(|(_, n)| n.label == Label(L_LABEL_LIT))
            .unwrap()
            .0;
        let guard = s
            .nodes()
            .find(|(_, n)| n.label == Label(L_GUARD))
            .unwrap()
            .0;
        let allele = s
            .nodes()
            .find(|(_, n)| n.label == Label(L_ALLELE_ROOT))
            .unwrap()
            .0;
        let expected = if ix < 9 {
            ix
        } else if ix == 17 {
            7
        } else {
            2
        };
        match ix {
            0 => limits.max_region_nodes = 2,
            1 => {
                let id = s
                    .edges()
                    .find(|(_, e)| e.label == Label(E_BECOMES))
                    .unwrap()
                    .0;
                s.get_edge_mut(id).unwrap().nodes.swap(1, 2);
            }
            2 => {
                s.remove_edge(edge);
            }
            3 => limits.max_arity = 1,
            4 => s
                .get_node_mut(variable)
                .unwrap()
                .set_property(K_ORDER, PropertyValue::Int(99)),
            5 => s
                .get_node_mut(literal)
                .unwrap()
                .set_property(K_LITERAL, PropertyValue::Bool(true)),
            6 => s
                .get_node_mut(guard)
                .unwrap()
                .set_property(K_GOP, PropertyValue::Int(99)),
            7 => limits.max_pattern_edges = 0,
            8 => s
                .get_node_mut(allele)
                .unwrap()
                .set_property(K_PRIORITY, PropertyValue::Bool(true)),
            9 => {
                s.add_edge(s.get_edge(edge).unwrap().clone());
            }
            10 => {
                s.get_edge_mut(edge).unwrap().nodes.swap(0, 1);
            }
            11 => s
                .get_node_mut(variable)
                .unwrap()
                .set_property(K_SCHEMA_LABEL, PropertyValue::Int(99)),
            12 => s
                .get_node_mut(variable)
                .unwrap()
                .properties
                .retain(|(k, _)| *k != K_SCHEMA_LABEL),
            13 => s
                .get_node_mut(variable)
                .unwrap()
                .set_property(K_REQUIRED_BASE, PropertyValue::Int(99)),
            14 => s
                .get_node_mut(variable)
                .unwrap()
                .properties
                .retain(|(k, _)| *k != K_REQUIRED_BASE),
            15 => s
                .get_node_mut(variable)
                .unwrap()
                .properties
                .push((K_REQUIRED_BASE, PropertyValue::Int(0))),
            16 => {
                let mut n = NodeData::new(Label(L_VAR), 0);
                n.set_property(K_SCHEMA_LABEL, PropertyValue::Int(L_VAR as i64));
                for j in 0..26 {
                    n.set_property(K_REQUIRED_BASE + j, PropertyValue::Int(0));
                }
                let orphan = s.add_node(n);
                s.add_edge(HyperEdge::new(
                    Label(E_OWNS),
                    [root, orphan].into_iter().collect(),
                ));
                let old = match s
                    .get_node(root)
                    .unwrap()
                    .get_property(K_OWNED_COUNT)
                    .unwrap()
                {
                    PropertyValue::Int(n) => n,
                    _ => unreachable!(),
                };
                s.get_node_mut(root)
                    .unwrap()
                    .set_property(K_OWNED_COUNT, PropertyValue::Int(old + 1));
            }
            17 => {
                s.add_edge(HyperEdge::new(
                    Label(META_BASE + 0x777),
                    [variable, literal].into_iter().collect(),
                ));
            }
            _ => {
                let role = ix - names.len();
                let (id, e) = s
                    .edges()
                    .find(|(_, e)| e.label == Label(WITNESS_ROLES[role]) && e.arity() == 2)
                    .map(|(id, e)| (id, e.clone()))
                    .unwrap();
                let wid = s
                    .edges()
                    .find(|(_, w)| {
                        w.label == Label(WITNESS_BASE + role as u32)
                            && w.nodes.as_slice() == [root, e.nodes[0], e.nodes[1]]
                    })
                    .unwrap()
                    .0;
                complete::erased(&mut s, id);
                complete::erased(&mut s, wid);
            }
        }
        // Local cap changes apply to the already primed source too.
        src.limits = limits.clone();
        let input = witness::inventory(&s);
        let bytes = bincode::serialize(&s).unwrap();
        let error = read_program(&s.snapshot(), root, &limits)
            .err()
            .map(|v| format!("{v:?}"));
        let after = compile(&mut src, &s);
        let repeat = compile(&mut src, &s);
        let mut counters = [0; 9];
        counters[expected] = 1;
        let pass = primed["table"].as_array().unwrap().len() == 2
            && error.as_deref() == Some(VALIDATORS[expected])
            && after["table"] == json!([])
            && repeat["table"] == json!([])
            && after["report"]["retired"] == 1
            && after["report"]["quarantined"] == 1
            && after["report"]["quarantine_by_validator"] == json!(counters)
            && repeat["report"]["quarantine_by_validator"] == json!(counters)
            && repeat["report"]["retired"] == 0
            && bytes == bincode::serialize(&s).unwrap();
        cases.push(json!({"case":name,"root":root,"limits":limits,"expected_validator":VALIDATORS[expected],"actual_validator":error,
            "synthetic_input":true,"inventory":input,"after_inventory":witness::inventory(&s),"primed":primed,"after":after,"repeat":repeat,"pass":pass}));
    }
    let clean = compile(&mut source(MetaLimits::default()), &base);
    let clean_pass = clean["table"].as_array().unwrap().len() == 2
        && clean["report"]["quarantine_by_validator"] == json!(vec![0; 9]);
    json!({"gate":"G-R4","root":root,"base":witness::inventory(&base),"clean":clean,"clean_pass":clean_pass,"pass":clean_pass&&cases.iter().all(|c|c["pass"]==true),"cases":cases})
}

struct Capture(Arc<Mutex<Vec<Value>>>);
impl hg_observe::Observer for Capture {
    fn on_step_end(
        &mut self,
        _: &hg_core::GraphSnapshot<'_>,
        stats: &hg_observe::StepStats,
        events: &[hg_observe::RewriteEvent],
    ) {
        self.0.lock().unwrap().push(json!({"stats":stats,"events":events.iter().map(|e|json!({"rule_id":e.rule_id,"rule_name":e.rule_name,"bound_nodes":e.binding.bound_nodes,"matched_edges":e.binding.matched_edges,"nodes_created":e.nodes_created,"nodes_deleted":e.nodes_deleted,"edges_created":e.edges_created,"edges_deleted":e.edges_deleted,"cleanup_neighbors":e.cleanup_neighbors,"vanished_energy":e.vanished_energy})).collect::<Vec<_>>() }));
    }
}
fn seed(c: &CandidateRule, p: &CandidateRule) -> EreSeedFamily {
    #[derive(serde::Serialize)]
    struct Rules {
        rules: Vec<hg_rules::Rule>,
    }
    let toml = |c: &CandidateRule| {
        toml::to_string(&Rules {
            rules: vec![c.to_rule("gate".into())],
        })
        .unwrap()
    };
    EreSeedFamily {
        core_toml: toml(c),
        proposal_toml: toml(p),
        active: false,
    }
}
fn stationarity(condition: &str, probe: bool) -> Value {
    let core = CandidateRule {
        labels: vec![Some(10), Some(11), Some(12)],
        lhs: vec![vec![0, 1], vec![1, 2]],
        rhs: vec![vec![0, 1], vec![1, 2]],
        ..Default::default()
    };
    let mut p = core.clone();
    p.rhs = vec![vec![0, 1], vec![0, 2]];
    p.priority = 1;
    let operation = if condition == "metadata_seal" {
        MetaOperation::Seal
    } else {
        MetaOperation::WitnessActivate
    };
    let initial_fuel = if condition == "stable" { 0.0 } else { 1.0 };
    let cfg = EreConfig {
        encoding_version: EncodingVersion::AlternativeFamiliesV3,
        arm: EreArm::A3,
        epoch: EpochPolicy::Fixed(1.try_into().unwrap()),
        match_budget_policy: MatchBudgetPolicy::Pooled,
        seed_families: Some(vec![seed(&core, &p)]),
        measure_family_maintenance: true,
        vocabulary: Some(MetaVocabularyConfig {
            protocol: COMPLETE_PROTOCOL.into(),
            instructions: vec![MetaInstructionSeed {
                operation,
                fuel: initial_fuel,
            }],
        }),
        meta_seed_energy: 0.0,
        measure_ei: true,
        min_tpm_transitions: 1,
        ..Default::default()
    };
    let mut s = experiments::fixture(&core);
    let mut e = witness::engine(
        vec![core.compile(&MetaLimits::default(), false).unwrap()],
        42,
        50000,
    );
    let mut rt = EreRuntime::install(&cfg, &mut s, &mut e).unwrap();
    let op = s
        .nodes()
        .find(|(id, _)| read_complete_instruction(&s.snapshot(), *id).is_ok())
        .unwrap()
        .0;
    let native = Arc::new(Mutex::new(vec![]));
    e.add_observer(Box::new(Capture(native.clone())));
    let initial = witness::inventory(&s);
    let mut steps = vec![];
    let mut windows = vec![];
    let horizon = if condition == "activation" { 96 } else { 64 };
    for tick in 1..=horizon {
        let world = format!("{:016x}", meta_excluded_store(&s).canonical_hash());
        let rng = e.rng_state_probe();
        rt.before_step(&s, &mut e).unwrap();
        let compiled_rng = e.rng_state_probe();
        let stats = e.step(&mut s);
        rt.note_match_ceiling(e.match_ceiling_binding());
        rt.after_step(&s, &stats);
        let observation = rt.observe_completed_step(&s, &e, world, rng, compiled_rng);
        // Export is read-only; the parity test compares probe/no-probe trajectories.
        if probe {
            let before = bincode::serialize(&s).unwrap();
            let _ = rt.diagnostic_tpms();
            assert_eq!(before, bincode::serialize(&s).unwrap());
            assert_eq!(compiled_rng, e.rng_state_probe());
        }
        steps.push(json!({"step":s.step(),"inventory":witness::inventory(&s),"native":native.lock().unwrap().last().unwrap(),"observation":observation,"fuel":complete::fuel(&s,op),"usage":e.match_budget_usage()}));
        if tick == 64 || tick == 96 {
            let tpm = rt.diagnostic_tpms();
            let row = rt.close_window(&s, MetricsRow::default(), MetricsRow::default());
            windows.push(json!({"end":tick,"tpm":tpm,"row":row}));
        }
    }
    let stable = condition != "activation";
    let finite = |w: &Value| {
        ["meta_included", "meta_excluded"].iter().all(|k| {
            ["ei_micro", "ei_macro"]
                .iter()
                .all(|m| w["row"][k][m].as_f64().is_some_and(f64::is_finite))
        })
    };
    let mut pass = windows[0]["row"]["tpm_rule_set_stable"] == stable
        && steps.iter().all(|x| {
            x["observation"]["rng_before_compile"] == x["observation"]["rng_after_compile"]
                && x["observation"]["match_ceiling_binding"] == false
        });
    if stable {
        pass &= finite(&windows[0]);
    } else {
        pass &= ["meta_included", "meta_excluded"].iter().all(|k| {
            [
                "ei_micro",
                "ei_macro",
                "ce_best",
                "ce_best_strategy",
                "ce_structural",
                "ce_energetic",
                "ce_functional",
            ]
            .iter()
            .all(|m| windows[0]["row"][k][m].is_null())
        }) && finite(&windows[1])
            && windows[1]["row"]["tpm_rule_set_stable"] == true;
    }
    pass &= steps[0]["fuel"] == json!(0.0);
    json!({"condition":condition,"config":cfg,"initial":initial,"instruction":op,"initial_fuel":initial_fuel,"steps":steps,"windows":windows,"pass":pass})
}
fn r6() -> Value {
    let controls = vec![
        stationarity("stable", true),
        stationarity("activation", true),
        stationarity("metadata_seal", true),
    ];
    json!({"gate":"G-R6","pass":controls.iter().all(|c|c["pass"]==true),"controls":controls})
}
fn leg(e: &mut Engine, s: &mut HypergraphStore) -> Value {
    let rng = e.rng_state_probe();
    let stats = e.step(s);
    json!({"step":s.step(),"full_hash":format!("{:016x}",s.canonical_hash()),"world_hash":format!("{:016x}",meta_excluded_store(s).canonical_hash()),
        "nodes":s.node_count(),"edges":s.edge_count(),"stats":stats,"rng_before":format!("{rng:016x}"),"rng_after":format!("{:016x}",e.rng_state_probe()),
        "usage":e.match_budget_usage(),"ceiling":e.match_ceiling_binding()})
}
fn r2(rules_dir: &Path, horizon: usize) -> Value {
    let mut files: Vec<_> = std::fs::read_dir(rules_dir)
        .unwrap()
        .map(|x| x.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "toml"))
        .collect();
    files.sort();
    let mut rows = vec![];
    let mut library = vec![];
    let limits = MetaLimits::default();
    for path in files {
        let text = std::fs::read_to_string(&path).unwrap();
        let file = path.file_name().unwrap().to_str().unwrap();
        library.push(json!({"file":file,"toml":text}));
        let parsed = match hg_rules::parse_rules_toml(&text) {
            Ok(rs) => rs,
            Err(error) => {
                rows.push(json!({"file":file,"name":null,"status":"parse-error","reason":error.to_string(),"pass":false}));
                continue;
            }
        };
        for (index, raw) in parsed.into_iter().enumerate() {
            let c = match CandidateRule::from_rule(&raw, &limits) {
                Ok(c) => c,
                Err(reason) => {
                    rows.push(json!({"file":file,"index":index,"name":raw.name,"raw":raw,"status":"outside-envelope","reason":reason,"pass":true}));
                    continue;
                }
            };
            let mut s = experiments::fixture(&c);
            let root = match encode_complete_family(&mut s, &c, &c, false, &limits) {
                Ok(r) => r.root,
                Err(v) => {
                    rows.push(json!({"file":file,"index":index,"name":raw.name,"raw":raw,"candidate":candidate(&c),"status":"outside-complete-encoding","reason":format!("{v:?}"),"pass":true}));
                    continue;
                }
            };
            let p = witness::decoded(&s, root).unwrap();
            let initial = witness::inventory(&s);
            let mut other = s.clone();
            let mut compiled = hg_rules::compile_rule(&raw).unwrap();
            compiled.pattern.node_label_ceiling = Some(META_BASE);
            let mut direct = Engine::new(vec![compiled], StepMode::MaxParallel, 42, 1);
            let mut decoded = Engine::new(
                p.compile(&limits, false).unwrap(),
                StepMode::MaxParallel,
                42,
                1,
            );
            for e in [&mut direct, &mut decoded] {
                e.set_match_budget_policy(2_000_000, 100_000, MatchBudgetPolicy::Pooled);
            }
            let mut trace = vec![];
            let mut status = "identical";
            let mut engaged = 0;
            for _ in 0..horizon {
                let a = leg(&mut direct, &mut s);
                let b = leg(&mut decoded, &mut other);
                engaged += a["stats"]["accepted_rewrites"].as_u64().unwrap();
                let equal = a == b;
                let cap = a["ceiling"] == true
                    || b["ceiling"] == true
                    || s.node_count() > 2000
                    || s.edge_count() > 4000
                    || other.node_count() > 2000
                    || other.edge_count() > 4000;
                trace.push(json!({"direct":a,"decoded":b}));
                if !equal {
                    status = "divergent";
                    break;
                }
                if cap {
                    status = "unmeasured:ceiling";
                    break;
                }
            }
            if engaged == 0 && status == "identical" {
                status = "unmeasured:not-engaged";
            }
            rows.push(json!({"file":file,"index":index,"name":raw.name,"raw":raw,"candidate":candidate(&c),"program":program(&p),"root":root,
                "initial":initial,"direct_final":witness::inventory(&s),"decoded_final":witness::inventory(&other),"trace":trace,"engaged":engaged,"status":status,"pass":status=="identical"&&horizon==1000}));
        }
    }
    let supported = rows.iter().filter(|r| r["status"] == "identical").count();
    json!({"gate":"G-R2","horizon":horizon,"library":library,"rows":rows,"supported":supported,
        "pass":horizon==1000&&supported>0&&rows.iter().all(|r|r["pass"]==true),"scope":"within reported strict complete encoding envelope only"})
}
fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn freeze_inputs(checker: &Path, out: &Path) {
    // The checker imports its companion. Never let that import write pyc files
    // into the immutable source tree between enumeration and hashing.
    let status = std::process::Command::new("python3")
        .arg("-B")
        .arg(checker)
        .arg(out)
        .arg("--freeze")
        .status()
        .unwrap();
    assert!(status.success());
}

#[test]
fn frozen_packaging_does_not_create_unlisted_bytecode() {
    let temp = tempfile::tempdir().unwrap();
    let out = temp.path();
    let paths = [
        "docs/ere_complete_gate_harness_v1.md",
        "scripts/verify_ere_complete_gates.py",
        "scripts/verify_ere_complete_witness.py",
    ];
    for path in paths {
        let target = out.join("source_snapshots").join(path);
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::copy(repo().join(path), target).unwrap();
    }
    std::fs::copy(
        std::env::current_exe().unwrap(),
        out.join("executed_test_binary"),
    )
    .unwrap();
    save(&out.join("manifest.json"), &json!({"sources":paths}));
    freeze_inputs(
        &out.join("source_snapshots/scripts/verify_ere_complete_gates.py"),
        out,
    );
    assert!(!out.join("source_snapshots/scripts/__pycache__").exists());
    let checks: Value =
        serde_json::from_slice(&std::fs::read(out.join("frozen_inputs_sha256.json")).unwrap())
            .unwrap();
    let mut archived: Vec<_> = checks
        .as_object()
        .unwrap()
        .keys()
        .filter_map(|k| k.strip_prefix("source_snapshots/"))
        .collect();
    archived.sort();
    let mut expected = paths.to_vec();
    expected.sort();
    assert_eq!(archived, expected);
}

#[test]
fn canonical_and_validator_controls() {
    assert_eq!(r0()["pass"], true);
    let r = r4();
    for c in r["cases"].as_array().unwrap() {
        assert_eq!(c["pass"], true, "{}", c);
    }
    assert_eq!(r["pass"], true);
}
#[test]
fn measured_stationarity_and_read_only_probe_parity() {
    let r = r6();
    for c in r["controls"].as_array().unwrap() {
        assert_eq!(c["pass"], true, "condition {}", c["condition"]);
    }
    let a = stationarity("stable", true);
    let b = stationarity("stable", false);
    for key in ["initial", "steps", "windows"] {
        assert!(a[key] == b[key], "probe parity differs at {key}");
    }
}
#[test]
fn short_library_preflight_cannot_pass_full_gate() {
    let r = r2(&repo().join("rules"), 4);
    assert_eq!(r["pass"], false);
    assert!(r["supported"].as_u64().unwrap() > 0);
}

#[test]
#[ignore = "development-only auditor integration; not a gate receipt"]
fn auditor_development_preflight() {
    let out = PathBuf::from(std::env::var_os("ERE_COMPLETE_GATES_PREFLIGHT").unwrap());
    std::fs::create_dir(&out).expect("fresh preflight directory required");
    for (name, value) in [
        ("G-R0", r0()),
        ("G-R4", r4()),
        ("G-R6", r6()),
        ("G-R2", r2(&repo().join("rules"), 4)),
    ] {
        save(&out.join(format!("{name}.json")), &value);
    }
    save(
        &out.join("development_only.json"),
        &json!({"gate_pass":false,"r2_horizon":4}),
    );
}

#[test]
#[ignore = "explicit frozen complete semantic assay; requires fresh output directory"]
fn complete_gates_frozen_assay() {
    let out = PathBuf::from(
        std::env::var_os("ERE_COMPLETE_GATES_OUTPUT").expect("ERE_COMPLETE_GATES_OUTPUT required"),
    );
    std::fs::create_dir(&out).expect("refusing to overwrite gate receipts");
    let root = repo();
    let snapshots = out.join("source_snapshots");
    std::fs::create_dir(&snapshots).unwrap();
    let mut archived = vec![];
    let mut paths = vec![
        "docs/ere_complete_gate_harness_v1.md".to_string(),
        "scripts/verify_ere_complete_gates.py".into(),
        "scripts/verify_ere_complete_witness.py".into(),
        "crates/hg-sim-core/tests/ere_complete_gates.rs".into(),
        "crates/hg-sim-core/tests/support/witness.rs".into(),
        "crates/hg-sim-core/tests/support/complete.rs".into(),
        "Cargo.lock".into(),
        "Cargo.toml".into(),
    ];
    for dir in [
        "crates/hg-meta/src",
        "crates/hg-engine/src",
        "crates/hg-rules/src",
        "crates/hg-core/src",
        "crates/hg-causal/src",
        "crates/hg-eval/src",
        "crates/hg-schema/src",
    ] {
        paths.extend(
            std::fs::read_dir(root.join(dir))
                .unwrap()
                .map(|x| x.unwrap().path())
                .filter(|p| p.extension().is_some_and(|e| e == "rs"))
                .map(|p| p.strip_prefix(&root).unwrap().to_str().unwrap().to_string()),
        );
    }
    paths.push("crates/hg-sim-core/src/ere.rs".into());
    for dir in ["crates/hg-sim-core/src/ere", "rules"] {
        paths.extend(
            std::fs::read_dir(root.join(dir))
                .unwrap()
                .map(|x| x.unwrap().path())
                .filter(|p| p.is_file())
                .map(|p| p.strip_prefix(&root).unwrap().to_str().unwrap().to_string()),
        );
    }
    paths.sort();
    paths.dedup();
    for path in paths {
        let target = snapshots.join(&path);
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::copy(root.join(&path), &target).unwrap();
        archived.push(path);
    }
    std::fs::copy(
        std::env::current_exe().unwrap(),
        out.join("executed_test_binary"),
    )
    .unwrap();
    save(
        &out.join("manifest.json"),
        &json!({"profile":PROFILE,"protocol":COMPLETE_PROTOCOL,"family_version":10,"instruction_version":11,"frozen_before_measurement":true,
        "sources":archived,"gates":["G-R0","G-R4","G-R6","G-R2"],"pending":["G-R1","G-R3","G-R5","optimizer stateability","training admission"],
        "r0_families":20,"r4_cases":31,"r6_steps":224,"r2_steps_per_supported_rule":1000,"training_admitted":false}),
    );
    freeze_inputs(
        &snapshots.join("scripts/verify_ere_complete_gates.py"),
        &out,
    );
    let r0 = r0();
    save(&out.join("G-R0.json"), &r0);
    println!("G-R0 pass={}", r0["pass"]);
    let r4 = r4();
    save(&out.join("G-R4.json"), &r4);
    println!("G-R4 pass={}", r4["pass"]);
    let r6 = r6();
    save(&out.join("G-R6.json"), &r6);
    println!("G-R6 pass={}", r6["pass"]);
    let r2 = r2(&snapshots.join("rules"), 1000);
    save(&out.join("G-R2.json"), &r2);
    println!("G-R2 pass={} supported={}", r2["pass"], r2["supported"]);
    let pass = [&r0, &r4, &r6, &r2].iter().all(|r| r["pass"] == true);
    save(
        &out.join("summary.json"),
        &json!({"profile":PROFILE,"pass":pass,"gates":[{"gate":"G-R0","pass":r0["pass"]},{"gate":"G-R4","pass":r4["pass"]},{"gate":"G-R6","pass":r6["pass"]},{"gate":"G-R2","pass":r2["pass"],"supported":r2["supported"],"total":r2["rows"].as_array().unwrap().len()}],"training_admitted":false}),
    );
    assert!(pass, "raw failures retained in {}", out.display());
    let audited = std::process::Command::new("python3")
        .arg("-B")
        .arg(snapshots.join("scripts/verify_ere_complete_gates.py"))
        .arg(&out)
        .arg("--tamper-tests")
        .arg("--write-report")
        .status()
        .unwrap();
    assert!(
        audited.success(),
        "native pass is not sufficient: frozen auditor rejected {}",
        out.display()
    );
}
