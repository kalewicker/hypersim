#![cfg(feature = "hg-meta")]
#[allow(dead_code)]
#[path = "support/complete.rs"]
mod complete;
#[allow(dead_code)]
#[path = "support/witness.rs"]
mod witness;
use hg_core::{GraphSnapshot, HypergraphStore, Label, NodeData, NodeId, PropertyOp, PropertyValue};
use hg_meta::*;
use hg_rules::{
    compile_rule, CompiledRule, EffectOpDecl, EpochPolicy, MatchBudgetPolicy, NodePropertyGuard,
    PropertyEffectDecl, Rule, StepMode,
};
use hg_sim_core::{
    ere::{EreArm, EreConfig, EreSeedFamily, MetaVocabularyConfig},
    runner::{run_ere_environment_diagnostic, run_ere_instrumented_diagnostic},
    RuleSource, SimConfig,
};
use serde_json::{json, Value};
use std::{
    path::Path,
    sync::{Arc, Mutex},
};

const SEEDS: [u64; 4] = [984927043601, 984927043602, 984927043603, 984927043604];
const STEPS: u64 = 64;
const CLOCK: u32 = 0xE5560001;
const MARKER: u32 = 0xE5560002;
const BOUNDARIES: [u64; 12] = [0, 8, 9, 16, 17, 24, 25, 32, 33, 40, 41, 64];

fn hex(bytes: Vec<u8>) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn save(path: &Path, value: &Value) {
    std::fs::write(path, serde_json::to_vec(value).unwrap()).unwrap();
}
fn candidate(proposal: bool) -> CandidateRule {
    CandidateRule {
        labels: vec![Some(0); 3],
        lhs: if proposal {
            vec![vec![0, 1], vec![0, 2]]
        } else {
            vec![vec![0, 1], vec![1, 2]]
        },
        rhs: if proposal {
            vec![vec![0, 1], vec![1, 2]]
        } else {
            vec![vec![0, 1], vec![0, 2]]
        },
        priority: i32::from(proposal),
        ..CandidateRule::default()
    }
}
fn config(seed: u64, protected: bool, fuel: f64) -> SimConfig {
    #[derive(serde::Serialize)]
    struct Raw {
        rules: Vec<Rule>,
    }
    let text = |p| {
        toml::to_string(&Raw {
            rules: vec![candidate(p).to_rule("ecology".into())],
        })
        .unwrap()
    };
    let mut c = SimConfig::from_toml("experiment_name = 'ERE_complete_repeated_damage_v1'\nseed = 0\n[rules]\nsource = 'Inline'\ninline_toml = ''\n").unwrap();
    c.seed = seed;
    c.experiment_name = "ERE_complete_repeated_damage_v1".into();
    c.rules.source = RuleSource::Inline;
    c.rules.inline_toml = Some(text(false));
    c.initial.width = 4;
    c.initial.height = 4;
    c.initial.arity = 2;
    c.initial.periodic = true;
    c.engine.max_steps = STEPS;
    c.engine.step_mode = StepMode::MaxParallel;
    c.engine.thread_count = 1;
    c.engine.max_pairings_per_step = 16;
    c.engine.metrics_window_interval = 1;
    c.evaluator.enabled = false;
    c.causal.enabled = false;
    c.ere = Some(EreConfig {
        arm: if protected { EreArm::A2 } else { EreArm::A3 },
        encoding_version: EncodingVersion::AlternativeFamiliesV3,
        epoch: EpochPolicy::Fixed(1.try_into().unwrap()),
        match_budget_policy: MatchBudgetPolicy::Pooled,
        measure_family_maintenance: true,
        meta_seed_energy: 0.0,
        measure_ei: false,
        seed_families: Some(vec![EreSeedFamily {
            core_toml: text(false),
            proposal_toml: text(true),
            active: false,
        }]),
        vocabulary: Some(MetaVocabularyConfig {
            protocol: COMPLETE_PROTOCOL.into(),
            instructions: vec![
                MetaInstructionSeed {
                    operation: MetaOperation::Seal,
                    fuel: 1.0,
                },
                MetaInstructionSeed {
                    operation: MetaOperation::RepairSealed,
                    fuel,
                },
            ],
        }),
        ..EreConfig::default()
    });
    c
}
fn inventory(s: &GraphSnapshot<'_>) -> Value {
    let mut nodes: Vec<_> = s
        .nodes()
        .map(|(id, data)| {
            let mut incidence: Vec<_> = s
                .edges_incident_to(id)
                .iter()
                .map(|e| format!("{e:?}"))
                .collect();
            incidence.sort();
            (format!("{id:?}"), json!({"data":data,"incident":incidence}))
        })
        .collect();
    nodes.sort_by(|a, b| a.0.cmp(&b.0));
    let mut edges: Vec<_> = s
        .edges()
        .map(|(id, e)| (format!("{id:?}"), json!(e)))
        .collect();
    edges.sort_by(|a, b| a.0.cmp(&b.0));
    let mut labels: Vec<_> = s.edges().map(|(_, e)| e.label.0).collect();
    labels.sort();
    labels.dedup();
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
fn environment(protected: bool, ownership: bool) -> (Vec<Rule>, Vec<CompiledRule>) {
    let mut raw = vec![];
    let effect = PropertyEffectDecl {
        var: "clock".into(),
        key: CLOCK,
        op: EffectOpDecl::Add {
            value: PropertyValue::Int(1),
        },
    };
    let mut tick = Rule {
        name: "environment_clock".into(),
        priority: 100,
        ..Rule::default()
    };
    tick.lhs.nodes = vec!["clock".into()];
    tick.rhs.nodes = tick.lhs.nodes.clone();
    tick.lhs.node_labels.insert("clock".into(), Label(777));
    tick.rhs_property_effects.push(effect.clone());
    raw.push(tick);
    for counter in [7, 15, 23, 31].into_iter().chain(ownership.then_some(39)) {
        let owner = counter == 39;
        let mut r = Rule {
            name: format!(
                "environment_{}_{}",
                if owner { "ownership" } else { "payload" },
                counter + 1
            ),
            priority: 200,
            ..Rule::default()
        };
        r.lhs.nodes = vec!["clock".into(), "root".into(), "side".into()];
        r.rhs.nodes = r.lhs.nodes.clone();
        r.lhs.node_labels.insert("clock".into(), Label(777));
        for (var, key, value) in [
            ("clock", CLOCK, counter),
            ("root", MARKER, 0),
            ("side", MARKER, 1),
        ] {
            r.lhs_node_property_guards.push(NodePropertyGuard {
                var: var.into(),
                property: key,
                op: PropertyOp::Eq,
                value: PropertyValue::Int(value),
            });
        }
        let edge = if owner {
            vec!["root".into(), "side".into()]
        } else {
            vec!["root".into(), "side".into(), "root".into()]
        };
        r.lhs.edges.push(edge.clone());
        r.rhs.edges.push(edge);
        if owner {
            r.lhs.edge_labels.insert(0, Label(E_OWNS));
        }
        r.rhs_property_effects.push(effect.clone());
        raw.push(r);
    }
    let compiled = raw
        .iter()
        .map(|r| {
            let mut c = compile_rule(r).unwrap();
            c.meta_visible = !protected;
            c.pattern.node_label_ceiling = protected.then_some(META_BASE);
            for edge in &mut c.pattern.edges {
                edge.ordered = true;
            }
            c
        })
        .collect();
    (raw, compiled)
}
struct Capture {
    root: NodeId,
    repair: NodeId,
    clock: NodeId,
    expected: RuleProgram,
    records: Arc<Mutex<Vec<Value>>>,
}
impl hg_observe::Observer for Capture {
    fn on_step_end(
        &mut self,
        s: &GraphSnapshot<'_>,
        stats: &hg_observe::StepStats,
        events: &[hg_observe::RewriteEvent],
    ) {
        let before = serde_json::to_vec(&inventory(s)).unwrap();
        let p = read_program(s, self.root, &MetaLimits::default()).ok();
        let fuel = read_complete_instruction(s, self.repair).unwrap().fuel;
        let ev: Vec<_> = events
            .iter()
            .map(|e| {
                json!({"rule_id":e.rule_id,"rule_name":e.rule_name,
            "bound_nodes":e.binding.bound_nodes,"matched_edges":e.binding.matched_edges,
            "nodes_created":e.nodes_created,"nodes_deleted":e.nodes_deleted,
            "edges_created":e.edges_created,"edges_deleted":e.edges_deleted,
            "cleanup_neighbors":e.cleanup_neighbors,"vanished_energy":e.vanished_energy})
            })
            .collect();
        let mut row = json!({"step":s.step(),"root":self.root,"fuel":fuel,
            "clock":s.get_node(self.clock).unwrap().get_property(CLOCK),"stats":stats,"events":ev,
            "valid":p.is_some(),"exact_original":p.as_ref()==Some(&self.expected),
            "execution_bytes":p.map(|p|hex(p.execution_bytes()))});
        if BOUNDARIES.contains(&s.step()) {
            row["inventory"] = inventory(s);
        }
        assert_eq!(
            before,
            serde_json::to_vec(&inventory(s)).unwrap(),
            "observer cannot mutate graph"
        );
        self.records.lock().unwrap().push(row);
    }
}

fn setup(
    seed: u64,
    protected: bool,
    ownership: bool,
    dir: Option<std::path::PathBuf>,
    records: Arc<Mutex<Vec<Value>>>,
    preamble: Arc<Mutex<Option<Value>>>,
) -> hg_sim_core::runner::EreEnvironmentSetup {
    Box::new(move |s: &mut HypergraphStore, e| {
        let root = s
            .nodes()
            .find(|(_, n)| n.label == Label(L_RULE_ROOT))
            .unwrap()
            .0;
        let payload = s
            .edges()
            .find(|(_, edge)| edge.label == Label(E_CORE) && edge.arity() == 2)
            .unwrap()
            .0;
        let side = s.get_edge(payload).unwrap().nodes[1];
        let instruction = |op| {
            s.nodes()
                .find(|(id, _)| {
                    read_complete_instruction(&s.snapshot(), *id).is_ok_and(|v| v.operation == op)
                })
                .unwrap()
                .0
        };
        let seal = instruction(MetaOperation::Seal);
        let repair = instruction(MetaOperation::RepairSealed);
        let expected = read_program(&s.snapshot(), root, &MetaLimits::default()).unwrap();
        let before = inventory(&s.snapshot());
        let rng = e.rng_state_probe();
        let seal_binding=compile_complete_operations(MetaOperation::Seal,&MetaLimits::default()).unwrap()
            .into_iter().find_map(|rule| {
                let (bindings,usage)=hg_engine::find_bindings_counted(&s.snapshot(),&rule,0);
                assert!(!usage.enumeration_truncated);
                bindings.into_iter().find(|b|b.matched_edges.contains(&payload)).map(|b|
                    json!({"rule_name":rule.name,"bound_nodes":b.bound_nodes,"matched_edges":b.matched_edges}))
            }).unwrap();
        let sealed = complete::seal(s, payload, seal, seed);
        s.get_node_mut(root)
            .unwrap()
            .set_property(MARKER, PropertyValue::Int(0));
        s.get_node_mut(side)
            .unwrap()
            .set_property(MARKER, PropertyValue::Int(1));
        let mut data = NodeData::new(Label(777), 0);
        data.set_property(CLOCK, PropertyValue::Int(0));
        let clock = s.add_node(data);
        assert_eq!(s.step(), 0);
        assert_eq!(rng, e.rng_state_probe());
        assert_eq!(
            read_program(&s.snapshot(), root, &MetaLimits::default()).unwrap(),
            expected
        );
        let after = inventory(&s.snapshot());
        let checkpoint = dir
            .as_ref()
            .map(|d| complete::checkpoint(d, "initial", s, root, repair));
        *preamble.lock().unwrap() = Some(
            json!({"root":root,"side":side,"seal":seal,"repair":repair,"clock":clock,
            "original_payload":payload,"sealed_payload":sealed,"seal_paid":1,"step":0,
            "seal_binding":seal_binding,
            "rng_before":format!("{rng:016x}"),"rng_after":format!("{:016x}",e.rng_state_probe()),
            "expected_program":format!("{expected:?}"),"expected_execution_bytes":hex(expected.execution_bytes()),
            "expected_core_bytes":hex(candidate(false).execution_bytes()),
            "before_inventory":before,"inventory":after,"checkpoint":checkpoint}),
        );
        e.add_observer(Box::new(Capture {
            root,
            repair,
            clock,
            expected,
            records,
        }));
        Ok(environment(protected, ownership).1)
    })
}
fn evaluate(
    seed: u64,
    protected: bool,
    fuel: f64,
    ownership: bool,
    dir: Option<std::path::PathBuf>,
) -> Value {
    let records = Arc::new(Mutex::new(vec![]));
    let preamble = Arc::new(Mutex::new(None));
    let cfg = config(seed, protected, fuel);
    let (result, rows, steps) = run_ere_environment_diagnostic(
        &cfg,
        setup(
            seed,
            protected,
            ownership,
            dir,
            records.clone(),
            preamble.clone(),
        ),
    );
    let (completed, error, hash) = match result {
        Ok(v) => (
            Some(v.final_step),
            None,
            Some(format!("{:016x}", v.canonical_hash)),
        ),
        Err(e) => {
            let done = match &e {
                hg_sim_core::runner::SimError::EreHalt {
                    completed_steps, ..
                } => Some(*completed_steps),
                _ => steps.last().map(|s| s.step),
            };
            (done, Some(e.to_string()), None)
        }
    };
    let captured = records.lock().unwrap().clone();
    let mut pre = preamble.lock().unwrap().clone();
    let table: Vec<_> = rows
        .iter()
        .flat_map(|r| r.encoded_meta_operation_traces.clone().unwrap_or_default())
        .collect();
    let summary = json!({"completed_steps":completed,"error":error,"valid_steps":captured.iter().filter(|s|s["valid"]==true).count(),
        "exact_steps":captured.iter().filter(|s|s["exact_original"]==true).count(),
        "changed_valid_steps":captured.iter().filter(|s|s["valid"]==true&&s["exact_original"]!=true).count(),
        "repair_payments":table.iter().filter(|t|t.operation=="RepairSealed").count(),
        "final_fuel":captured.last().map(|s|s["fuel"].clone()),"final_valid":captured.last().map(|s|s["valid"].clone()),
        "hazard_events":captured.iter().flat_map(|s|s["events"].as_array().unwrap()).filter(|v|v["rule_name"].as_str().unwrap().starts_with("environment_")&&v["rule_name"]!="environment_clock").count(),
        "world_events":captured.iter().flat_map(|s|s["events"].as_array().unwrap()).filter(|v|v["rule_name"].as_str().unwrap().starts_with("e1_")).count()});
    json!({"seed":seed,"protected":protected,"initial_fuel":fuel,"late_ownership_loss":ownership,
        "effective_config":cfg,"environment_raw_rules":environment(protected,ownership).0,
        "environment_ordered":true,"environment_meta_visible":!protected,"environment_node_label_ceiling":protected.then_some(META_BASE),
        "preamble":pre.take(),"completed_steps":completed,"error":error,"final_hash":hash,
        "summary":summary,"ere_windows":rows,"step_observations":steps,"native_steps":captured,
        "training_admitted":false})
}
fn expected_counts(protected: bool, fuel: f64, ownership: bool) -> (usize, usize) {
    if protected {
        (64, 0)
    } else if fuel == 0.0 {
        (7, 0)
    } else if fuel == 2.0 {
        (21, 2)
    } else if ownership {
        (35, 4)
    } else {
        (60, 4)
    }
}
fn assert_mechanism(r: &Value) {
    let protected = r["protected"].as_bool().unwrap();
    let fuel = r["initial_fuel"].as_f64().unwrap();
    let ownership = r["late_ownership_loss"].as_bool().unwrap();
    let (valid, paid) = expected_counts(protected, fuel, ownership);
    assert_eq!(r["error"], Value::Null);
    assert_eq!(r["completed_steps"], 64);
    assert_eq!(r["summary"]["valid_steps"], valid);
    assert_eq!(r["summary"]["exact_steps"], valid);
    assert_eq!(r["summary"]["repair_payments"], paid);
    assert_eq!(r["summary"]["final_fuel"], fuel - paid as f64);
    assert_eq!(r["summary"]["changed_valid_steps"], 0);
    assert_eq!(
        r["summary"]["hazard_events"],
        if protected {
            0
        } else {
            4 + usize::from(ownership)
        }
    );
    assert!(r["summary"]["world_events"].as_u64().unwrap() > 0);
    let steps = r["step_observations"].as_array().unwrap();
    assert_eq!(steps.len(), 64);
    assert!(steps
        .iter()
        .all(|s| s["rng_before_compile"] == s["rng_after_compile"]
            && !s["match_ceiling_binding"].as_bool().unwrap()));
}

#[test]
fn repeated_damage_local_controls() {
    for protected in [true, false] {
        for fuel in [0.0, 2.0, 8.0] {
            for ownership in [false, true] {
                assert_mechanism(&evaluate(SEEDS[0], protected, fuel, ownership, None));
            }
        }
    }
}
#[test]
fn empty_environment_matches_normal_loop_and_rejects_other_profiles() {
    let cfg = config(SEEDS[0], true, 8.0);
    let (a, ar, as_) = run_ere_instrumented_diagnostic(&cfg);
    let (b, br, bs) = run_ere_environment_diagnostic(&cfg, Box::new(|_, _| Ok(vec![])));
    assert_eq!(a.unwrap().canonical_hash, b.unwrap().canonical_hash);
    assert_eq!(ar, br);
    assert_eq!(as_, bs);
    let mut bad = cfg.clone();
    bad.ere
        .as_mut()
        .unwrap()
        .vocabulary
        .as_mut()
        .unwrap()
        .protocol = SEALED_REPAIR_PROTOCOL.into();
    let (r, rows, steps) =
        run_ere_environment_diagnostic(&bad, Box::new(|_, _| panic!("must reject before setup")));
    assert!(r.is_err());
    assert!(rows.is_empty() && steps.is_empty());
}
#[test]
fn setup_advance_and_visibility_bypass_are_rejected() {
    let cfg = config(SEEDS[0], true, 8.0);
    let (r, rows, steps) = run_ere_environment_diagnostic(
        &cfg,
        Box::new(|s, _| {
            s.set_step(1);
            Ok(vec![])
        }),
    );
    assert!(r.is_err());
    assert!(rows.is_empty() && steps.is_empty());
    let (r, rows, steps) =
        run_ere_environment_diagnostic(&cfg, Box::new(|_, _| Ok(environment(false, false).1)));
    assert!(r.is_err());
    assert!(rows.is_empty() && steps.is_empty());
}

#[test]
fn capped_environment_retains_partial_failure_without_payment() {
    let mut cfg = config(SEEDS[0], false, 8.0);
    cfg.ere.as_mut().unwrap().match_ceiling = 1;
    let records = Arc::new(Mutex::new(vec![]));
    let preamble = Arc::new(Mutex::new(None));
    let (r, rows, steps) = run_ere_environment_diagnostic(
        &cfg,
        setup(
            SEEDS[0],
            false,
            true,
            None,
            records.clone(),
            preamble.clone(),
        ),
    );
    assert!(matches!(
        r,
        Err(hg_sim_core::runner::SimError::EreHalt {
            completed_steps: 0,
            ..
        })
    ));
    assert!(steps.is_empty() && records.lock().unwrap().is_empty());
    assert!(preamble.lock().unwrap().is_some());
    assert!(rows.iter().any(|r| r.match_ceiling_binding));
    assert!(rows
        .iter()
        .all(|r| r.encoded_meta_rewrite_events == Some(0)));
}
#[test]
#[ignore = "explicit frozen 48-cell research run"]
fn repeated_damage_frozen_assay() {
    let out = std::path::PathBuf::from(
        std::env::var_os("ERE_REPEATED_OUTPUT").expect("set fresh ERE_REPEATED_OUTPUT"),
    );
    std::fs::create_dir(&out).expect("refusing to overwrite experiment");
    std::fs::write(
        out.join("preregistration.md"),
        include_str!("../../../docs/ere_repeated_damage_v1.md"),
    )
    .unwrap();
    std::fs::write(
        out.join("test_source.rs"),
        include_str!("ere_repeated_damage.rs"),
    )
    .unwrap();
    std::fs::write(
        out.join("complete_fixture.rs"),
        include_str!("support/complete.rs"),
    )
    .unwrap();
    std::fs::write(
        out.join("shared_fixture.rs"),
        include_str!("support/witness.rs"),
    )
    .unwrap();
    std::fs::copy(
        std::env::current_exe().unwrap(),
        out.join("executed_test_binary"),
    )
    .unwrap();
    let source = out.join("source_snapshots");
    std::fs::create_dir(&source).unwrap();
    for (name, data) in [
        ("runner.rs", include_str!("../src/runner.rs")),
        ("runtime.rs", include_str!("../src/ere.rs")),
        ("complete.rs", include_str!("../../hg-meta/src/complete.rs")),
        ("source.rs", include_str!("../../hg-meta/src/source.rs")),
        ("repair.rs", include_str!("../../hg-meta/src/repair.rs")),
        ("program.rs", include_str!("../../hg-meta/src/program.rs")),
        ("schema.rs", include_str!("../../hg-meta/src/schema.rs")),
        ("region.rs", include_str!("../../hg-meta/src/region.rs")),
        (
            "vocabulary.rs",
            include_str!("../../hg-meta/src/vocabulary.rs"),
        ),
        (
            "compiled.rs",
            include_str!("../../hg-rules/src/compiled.rs"),
        ),
        ("engine.rs", include_str!("../../hg-engine/src/engine.rs")),
        ("matcher.rs", include_str!("../../hg-engine/src/matcher.rs")),
        ("graph.rs", include_str!("../../hg-core/src/graph.rs")),
        ("wire.rs", include_str!("../../hg-core/src/wire.rs")),
        ("binlog.rs", include_str!("../../hg-output/src/binlog.rs")),
        ("Cargo.lock", include_str!("../../../Cargo.lock")),
        (
            "verify_ere_repeated_damage.py",
            include_str!("../../../scripts/verify_ere_repeated_damage.py"),
        ),
        (
            "verify_ere_complete_witness.py",
            include_str!("../../../scripts/verify_ere_complete_witness.py"),
        ),
    ] {
        std::fs::write(source.join(name), data).unwrap();
    }
    let mut plan = vec![];
    for seed in SEEDS {
        for protected in [true, false] {
            for fuel in [0.0, 2.0, 8.0] {
                for ownership in [false, true] {
                    plan.push((seed, protected, fuel, ownership));
                }
            }
        }
    }
    save(
        &out.join("manifest.json"),
        &json!({"diagnostic":"E2-complete-repeated-damage-v1","protocol":COMPLETE_PROTOCOL,
        "seeds":SEEDS,"attempts":48,"horizon":64,"step_ceiling":3072,"payload_exposure_steps":[8,16,24,32],
        "ownership_exposure_step":40,"work_limit":50000,"result_limit":50000,"max_pairings":16,
        "optimizer_evaluations":0,"held_out_evaluations":0,"training_admitted":false,
        "planned_configs":plan.iter().map(|(s,p,f,o)|json!({"config":config(*s,*p,*f),"late_ownership_loss":o})).collect::<Vec<_>>() }),
    );
    let mut results = vec![];
    for (index, (seed, protected, fuel, ownership)) in plan.into_iter().enumerate() {
        let name = format!("attempt_{index:03}");
        let dir = out.join(&name);
        std::fs::create_dir(&dir).unwrap();
        let r = evaluate(seed, protected, fuel, ownership, Some(dir.clone()));
        save(&dir.join("receipt.json"), &r);
        println!("{name}: {}", r["summary"]);
        results.push(
            json!({"attempt":name,"seed":seed,"protected":protected,"initial_fuel":fuel,
            "late_ownership_loss":ownership,"summary":r["summary"]}),
        );
        save(
            &out.join("status.json"),
            &json!({"state":"running","attempts_recorded":results.len(),"training_admitted":false}),
        );
    }
    // Every cell is persisted before any scientific assertion. Never discard a
    // nonconforming outcome or re-run a failed cell within this invocation.
    save(
        &out.join("summary.json"),
        &json!({"attempts":results,"training_admitted":false,"engineering_pass":false}),
    );
    let mut passed = true;
    for r in &results {
        let receipt: Value = serde_json::from_slice(
            &std::fs::read(
                out.join(r["attempt"].as_str().unwrap())
                    .join("receipt.json"),
            )
            .unwrap(),
        )
        .unwrap();
        if std::panic::catch_unwind(|| assert_mechanism(&receipt)).is_err() {
            passed = false;
        }
    }
    save(
        &out.join("summary.json"),
        &json!({"attempts":results,"training_admitted":false,"engineering_pass":passed}),
    );
    save(
        &out.join("status.json"),
        &json!({"state":"complete","attempts_recorded":48,"engineering_pass":passed,"training_admitted":false}),
    );
    assert!(
        passed,
        "frozen outcome differs from preregistered mechanism; receipts retained"
    );
}
