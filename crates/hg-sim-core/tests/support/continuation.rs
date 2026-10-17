//! Uses the exact label-separated fixture; all continuation runs use Engine
//! and EreRuntime's ordinary compile/step/after_step path, never host repair.
use super::*;
use hg_engine::Engine;
use hg_schema::MetricsRow;
use hg_sim_core::ere::{checkpoint, meta_excluded_store, EreRuntime};

const CUTS: [u64; 11] = [0, 7, 8, 9, 10, 24, 25, 33, 40, 41, 64];

struct Live {
    store: HypergraphStore,
    engine: Engine,
    runtime: EreRuntime,
    records: Arc<Mutex<Vec<Value>>>,
    interval: u64,
}
fn initial(protected: bool, fuel: f64, ownership: bool, random: bool) -> (Live, SimConfig) {
    let mut cfg = config(SEEDS[0], protected, fuel);
    if random {
        cfg.engine.step_mode = StepMode::SequentialRandom;
        cfg.engine.metrics_window_interval = 16;
    }
    let mut store = hg_sim_core::runner::build_initial_store(&cfg);
    let rules = hg_sim_core::runner::load_rules(&cfg).unwrap();
    let mut engine = Engine::new(
        rules,
        cfg.engine.step_mode,
        cfg.seed,
        cfg.engine.max_pairings_per_step,
    );
    let mut runtime =
        EreRuntime::install(cfg.ere.as_ref().unwrap(), &mut store, &mut engine).unwrap();
    let records = Arc::new(Mutex::new(vec![]));
    let preamble = Arc::new(Mutex::new(None));
    let suffix = setup(
        cfg.seed,
        protected,
        ownership,
        None,
        records.clone(),
        preamble,
    )(&mut store, &mut engine)
    .unwrap();
    runtime
        .install_diagnostic_environment(suffix, &mut engine)
        .unwrap();
    (
        Live {
            store,
            engine,
            runtime,
            records,
            interval: cfg.engine.metrics_window_interval,
        },
        cfg,
    )
}
fn restore(bytes: &[u8], interval: u64, expected: &RuleProgram) -> Live {
    let (store, mut engine, runtime) = checkpoint::decode(bytes).unwrap();
    let root = store
        .nodes()
        .find(|(_, n)| n.label == Label(L_RULE_ROOT))
        .unwrap()
        .0;
    let repair = store
        .nodes()
        .find(|(id, _)| {
            read_complete_instruction(&store.snapshot(), *id)
                .is_ok_and(|i| i.operation == MetaOperation::RepairSealed)
        })
        .unwrap()
        .0;
    let clock = store
        .nodes()
        .find(|(_, n)| n.label == Label(777))
        .unwrap()
        .0;
    let mut nodes: Vec<_> = store
        .nodes()
        .filter(|(_, n)| [10, 11, 12].contains(&n.label.0))
        .map(|(id, _)| id)
        .collect();
    nodes.sort_unstable();
    let motifs = nodes.chunks_exact(3).map(|n| [n[0], n[1], n[2]]).collect();
    let records = Arc::new(Mutex::new(vec![]));
    engine.add_observer(Box::new(Capture {
        root,
        repair,
        clock,
        expected: expected.clone(),
        records: records.clone(),
        motifs,
        previous_world: logical_edges(&world_edges(&store.snapshot())),
    }));
    Live {
        store,
        engine,
        runtime,
        records,
        interval,
    }
}
fn tick(v: &mut Live) -> Value {
    let before = format!("{:016x}", meta_excluded_store(&v.store).canonical_hash());
    let rng = v.engine.rng_state_probe();
    v.runtime.before_step(&v.store, &mut v.engine).unwrap();
    let compiled_rng = v.engine.rng_state_probe();
    let stats = v.engine.step(&mut v.store);
    v.runtime
        .note_match_ceiling(v.engine.match_ceiling_binding());
    v.runtime.after_step(&v.store, &stats);
    let observation =
        v.runtime
            .observe_completed_step(&v.store, &v.engine, before, rng, compiled_rng);
    let window = v.store.step().is_multiple_of(v.interval).then(|| {
        v.runtime
            .close_window(&v.store, MetricsRow::default(), MetricsRow::default())
    });
    json!({"step":v.store.step(),"inventory":inventory(&v.store.snapshot()),
        "native":v.records.lock().unwrap().last().unwrap(),"observation":observation,
        "window":window,"installed_rules":v.engine.rules()})
}
fn expected_program(s: &HypergraphStore) -> RuleProgram {
    let root = s
        .nodes()
        .find(|(_, n)| n.label == Label(L_RULE_ROOT))
        .unwrap()
        .0;
    read_program(&s.snapshot(), root, &MetaLimits::default()).unwrap()
}
fn cell(protected: bool, fuel: f64, ownership: bool, random: bool, dir: Option<&Path>) -> Value {
    let (mut live, cfg) = initial(protected, fuel, ownership, random);
    let expected = expected_program(&live.store);
    let cuts: Vec<_> = if random { vec![7, 8, 9] } else { CUTS.to_vec() };
    let start_rng = live.engine.rng_state_probe();
    let initial_inventory = inventory(&live.store.snapshot());
    let mut snapshots = vec![];
    let mut baseline = vec![];
    for step in 0..=64 {
        if cuts.contains(&step) {
            let bytes = checkpoint::encode(&live.store, &live.engine, &live.runtime).unwrap();
            if let Some(d) = dir {
                std::fs::write(d.join(format!("checkpoint_{step:02}.bin")), &bytes).unwrap();
            }
            snapshots.push((step, bytes));
        }
        if step < 64 {
            baseline.push(tick(&mut live));
        }
    }
    let rng_consumed = start_rng != live.engine.rng_state_probe();
    let terminal_window =
        live.runtime
            .close_window(&live.store, MetricsRow::default(), MetricsRow::default());
    let terminal_guards = live.engine.drain_guard_stats();
    let mut forks = vec![];
    for (cut, bytes) in snapshots {
        // Read actual persisted files in the frozen assay, not the retained
        // in-memory serialization buffer.
        let bytes = dir.map_or(bytes, |d| {
            std::fs::read(d.join(format!("checkpoint_{cut:02}.bin"))).unwrap()
        });
        let mut back = restore(&bytes, live.interval, &expected);
        let boundary_equal = inventory(&back.store.snapshot())
            == if cut == 0 {
                initial_inventory.clone()
            } else {
                baseline[(cut - 1) as usize]["inventory"].clone()
            };
        let mut suffix = vec![];
        while back.store.step() < 64 {
            suffix.push(tick(&mut back));
        }
        let fork_terminal_window =
            back.runtime
                .close_window(&back.store, MetricsRow::default(), MetricsRow::default());
        let fork_terminal_guards = back.engine.drain_guard_stats();
        let terminal_equal = fork_terminal_window == terminal_window
            && fork_terminal_guards == terminal_guards
            && serde_json::to_value(back.engine.rules()).unwrap()
                == serde_json::to_value(live.engine.rules()).unwrap()
            && back.engine.rule_table_revision() == live.engine.rule_table_revision();
        let pass = boundary_equal
            && terminal_equal
            && suffix == baseline[cut as usize..]
            && back.engine.rng_state_probe() == live.engine.rng_state_probe()
            && inventory(&back.store.snapshot()) == inventory(&live.store.snapshot());
        forks.push(json!({"cut":cut,"checkpoint_bytes":bytes.len(),"boundary_equal":boundary_equal,"terminal_equal":terminal_equal,
            "terminal_window":fork_terminal_window,"terminal_guards":fork_terminal_guards,
            "suffix_steps":suffix.len(),"suffix":suffix,"pass":pass}));
    }
    let mut expected_outcomes = true;
    if !random {
        let (valid, repairs) = expected_counts(protected, fuel, ownership);
        expected_outcomes = baseline
            .iter()
            .filter(|s| s["native"]["exact_original"] == true)
            .count()
            == valid
            && baseline
                .iter()
                .flat_map(|s| {
                    s["window"]["encoded_meta_operation_traces"]
                        .as_array()
                        .cloned()
                        .unwrap_or_default()
                })
                .filter(|t| t["operation"] == "RepairSealed")
                .count()
                == repairs;
    }
    let pass =
        forks.iter().all(|f| f["pass"] == true) && expected_outcomes && (!random || rng_consumed);
    json!({"protected":protected,"fuel":fuel,"ownership":ownership,"random":random,"effective_config":cfg,
        "baseline":baseline,"initial_inventory":initial_inventory,"terminal_window":terminal_window,"terminal_guards":terminal_guards,
        "forks":forks,"rng_consumed":rng_consumed,"expected_outcomes":expected_outcomes,
        "pass":pass,"training_admitted":false})
}

#[test]
fn continuation_local_paid_and_midwindow_rng() {
    let out = std::env::var_os("ERE_CHECKPOINT_PREFLIGHT").map(std::path::PathBuf::from);
    if let Some(d) = &out {
        assert!(!d.exists());
        std::fs::create_dir_all(d).unwrap();
    }
    for (index, protected, ownership, random) in [(11, false, true, false), (12, true, false, true)]
    {
        let dir = out.as_ref().map(|d| d.join(format!("cell_{index:02}")));
        if let Some(d) = &dir {
            std::fs::create_dir(d).unwrap();
        }
        let receipt = cell(protected, 8.0, ownership, random, dir.as_deref());
        if let Some(d) = &dir {
            save(&d.join("receipt.json"), &receipt);
        }
        assert_eq!(receipt["pass"], true);
    }
}

#[test]
fn continuation_rejects_corruption_and_nonquiescent_boundary() {
    let (mut v, _) = initial(false, 8.0, false, false);
    let good = checkpoint::encode(&v.store, &v.engine, &v.runtime).unwrap();
    for bad in [
        vec![],
        good[..good.len() - 1].to_vec(),
        {
            let mut b = good.clone();
            b.push(0);
            b
        },
        {
            let mut b = good.clone();
            b[0] ^= 1;
            b
        },
        {
            let mut b = good.clone();
            *b.last_mut().unwrap() ^= 1;
            b
        },
    ] {
        assert!(checkpoint::decode(&bad).is_err());
    }
    v.runtime.before_step(&v.store, &mut v.engine).unwrap();
    assert!(checkpoint::encode(&v.store, &v.engine, &v.runtime).is_err());
    let (mut v, _) = initial(false, 8.0, false, false);
    v.engine
        .set_policy(Box::new(hg_observe::policy::SelectAllPolicy));
    assert!(checkpoint::encode(&v.store, &v.engine, &v.runtime).is_err());
    let (mut v, _) = initial(false, 8.0, false, false);
    v.engine
        .set_rule_source(Box::new(hg_rules::StaticRuleSource(vec![])));
    assert!(checkpoint::encode(&v.store, &v.engine, &v.runtime).is_err());
    let mut cfg = config(SEEDS[0], true, 0.0);
    cfg.ere.as_mut().unwrap().vocabulary = None;
    let mut s = hg_sim_core::runner::build_initial_store(&cfg);
    let mut e = Engine::new(
        hg_sim_core::runner::load_rules(&cfg).unwrap(),
        StepMode::MaxParallel,
        cfg.seed,
        16,
    );
    let r = EreRuntime::install(cfg.ere.as_ref().unwrap(), &mut s, &mut e).unwrap();
    assert!(checkpoint::encode(&s, &e, &r).is_err());
}

#[test]
fn continuation_compiled_property_variants_roundtrip_without_changing_toml() {
    for value in [
        PropertyValue::Int(5),
        PropertyValue::Float(5.0),
        PropertyValue::Bool(true),
    ] {
        let mut rule = candidate(false).to_rule("typed_checkpoint".into());
        rule.lhs_node_property_guards.push(NodePropertyGuard {
            var: "v0".into(),
            property: 77,
            op: PropertyOp::Eq,
            value,
        });
        rule.rhs_property_effects.push(PropertyEffectDecl {
            var: "v0".into(),
            key: 77,
            op: EffectOpDecl::Set { value },
        });
        let mut compiled = compile_rule(&rule).unwrap();
        compiled
            .pattern
            .property_compares
            .push(hg_rules::CompiledPropertyCompare {
                var_a_ix: 0,
                key_a: 77,
                op: PropertyOp::Eq,
                var_b_ix: 1,
                key_b: 77,
                offset: Some(value),
            });
        let bytes = bincode::serialize(&compiled).unwrap();
        let back: CompiledRule = bincode::deserialize(&bytes).unwrap();
        assert_eq!(back.pattern.property_guards[0].value, value);
        assert_eq!(
            serde_json::to_value(back).unwrap(),
            serde_json::to_value(compiled).unwrap()
        );
        // Raw declarations still use bare values, not compiled wire tags.
        let raw = serde_json::to_value(&rule).unwrap();
        assert_eq!(
            raw["lhs_node_property_guards"][0]["value"],
            serde_json::to_value(value).unwrap()
        );
    }
}

#[test]
#[ignore = "explicit frozen checkpoint matrix; requires a fresh output directory"]
fn continuation_frozen_assay() {
    let out = std::path::PathBuf::from(
        std::env::var_os("ERE_CHECKPOINT_OUTPUT").expect("ERE_CHECKPOINT_OUTPUT"),
    );
    assert!(!out.exists(), "refuse checkpoint evidence overwrite");
    std::fs::create_dir_all(&out).unwrap();
    std::fs::write(
        out.join("preregistration.md"),
        include_str!("../../../../docs/ere_checkpoint_continuation_v1.md"),
    )
    .unwrap();
    std::fs::write(
        out.join("continuation_source.rs"),
        include_str!("continuation.rs"),
    )
    .unwrap();
    std::fs::write(
        out.join("label_fixture.rs"),
        include_str!("../ere_label_separated.rs"),
    )
    .unwrap();
    std::fs::write(
        out.join("checkpoint_source.rs"),
        include_str!("../../src/ere/checkpoint.rs"),
    )
    .unwrap();
    std::fs::copy(
        std::env::current_exe().unwrap(),
        out.join("executed_test_binary"),
    )
    .unwrap();
    let source = out.join("source_snapshots");
    std::fs::create_dir(&source).unwrap();
    for (name, content) in [
        (
            "engine.rs",
            include_str!("../../../hg-engine/src/engine.rs"),
        ),
        (
            "matcher.rs",
            include_str!("../../../hg-engine/src/matcher.rs"),
        ),
        (
            "engine_Cargo.toml",
            include_str!("../../../hg-engine/Cargo.toml"),
        ),
        (
            "graph_source.rs",
            include_str!("../../../hg-meta/src/source.rs"),
        ),
        (
            "meta_Cargo.toml",
            include_str!("../../../hg-meta/Cargo.toml"),
        ),
        ("sim_Cargo.toml", include_str!("../../Cargo.toml")),
        (
            "rule_source.rs",
            include_str!("../../../hg-rules/src/source.rs"),
        ),
        (
            "compiled.rs",
            include_str!("../../../hg-rules/src/compiled.rs"),
        ),
        (
            "effects.rs",
            include_str!("../../../hg-rules/src/effects.rs"),
        ),
        ("runtime.rs", include_str!("../../src/ere.rs")),
        ("runner.rs", include_str!("../../src/runner.rs")),
        ("window.rs", include_str!("../../../hg-eval/src/window.rs")),
        ("ei.rs", include_str!("../../../hg-causal/src/ei.rs")),
        ("graph.rs", include_str!("../../../hg-core/src/graph.rs")),
        ("wire.rs", include_str!("../../../hg-core/src/wire.rs")),
        (
            "complete.rs",
            include_str!("../../../hg-meta/src/complete.rs"),
        ),
        ("schema.rs", include_str!("../../../hg-meta/src/schema.rs")),
        ("complete_fixture.rs", include_str!("complete.rs")),
        ("shared_fixture.rs", include_str!("witness.rs")),
        ("Cargo.lock", include_str!("../../../../Cargo.lock")),
        ("Cargo.toml", include_str!("../../../../Cargo.toml")),
        (
            "verify_ere_checkpoint.py",
            include_str!("../../../../scripts/verify_ere_checkpoint.py"),
        ),
        (
            "verify_ere_complete_witness.py",
            include_str!("../../../../scripts/verify_ere_complete_witness.py"),
        ),
    ] {
        std::fs::write(source.join(name), content).unwrap();
    }
    // Preserve successful and failed local receipts separately from the
    // frozen matrix. This optional archive is never used as a replacement cell.
    if let Some(paths) = std::env::var_os("ERE_CHECKPOINT_PREFLIGHT_ARCHIVES") {
        let archive = out.join("development_preflight");
        std::fs::create_dir(&archive).unwrap();
        for (index, path) in std::env::split_paths(&paths).enumerate() {
            let target = archive.join(format!("preflight_{index}"));
            std::fs::create_dir(&target).unwrap();
            for cell in std::fs::read_dir(path).unwrap() {
                let cell = cell.unwrap();
                let dest = target.join(cell.file_name());
                std::fs::create_dir(&dest).unwrap();
                for file in std::fs::read_dir(cell.path()).unwrap() {
                    let file = file.unwrap();
                    assert!(file.file_type().unwrap().is_file());
                    std::fs::copy(file.path(), dest.join(file.file_name())).unwrap();
                }
            }
        }
    }
    save(
        &out.join("status.json"),
        &json!({"status":"running","training_admitted":false}),
    );
    let mut receipts = vec![];
    for protected in [true, false] {
        for fuel in [0.0, 2.0, 8.0] {
            for ownership in [false, true] {
                let dir = out.join(format!("cell_{:02}", receipts.len()));
                std::fs::create_dir(&dir).unwrap();
                let receipt = cell(protected, fuel, ownership, false, Some(&dir));
                save(&dir.join("receipt.json"), &receipt);
                receipts.push(receipt);
            }
        }
    }
    let dir = out.join("cell_12");
    std::fs::create_dir(&dir).unwrap();
    let receipt = cell(true, 8.0, false, true, Some(&dir));
    save(&dir.join("receipt.json"), &receipt);
    receipts.push(receipt);
    let mut cross_process = vec![];
    for index in [11, 12] {
        let dir = out.join(format!("cell_{index:02}"));
        let child = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "continuation::continuation_child_resume",
                "--ignored",
                "--exact",
                "--nocapture",
            ])
            .env("ERE_CHECKPOINT_CHILD", &dir)
            .output()
            .unwrap();
        std::fs::write(dir.join("child_stdout.txt"), &child.stdout).unwrap();
        std::fs::write(dir.join("child_stderr.txt"), &child.stderr).unwrap();
        cross_process.push(json!({"cell":index,"cut":8,"suffix_steps":56,"parent_pid":std::process::id(),"exit_code":child.status.code(),"pass":child.status.success()}));
    }
    let pass = receipts.iter().all(|r| r["pass"] == true)
        && cross_process.iter().all(|r| r["pass"] == true);
    save(&out.join("cross_process.json"), &json!(cross_process));
    let suffix_steps: usize = receipts
        .iter()
        .flat_map(|r| r["forks"].as_array().unwrap())
        .map(|f| f["suffix_steps"].as_u64().unwrap() as usize)
        .sum();
    save(
        &out.join("summary.json"),
        &json!({"profile":"E2-complete-checkpoint-continuation-v1","cells":13,"forks":135,
        "baseline_steps":832,"suffix_steps":suffix_steps,"cross_process_resumes":2,"cross_process_steps":112,"total_steps":832+suffix_steps+112,"pass":pass,"training_admitted":false}),
    );
    save(
        &out.join("status.json"),
        &json!({"status":"complete","pass":pass,"training_admitted":false}),
    );
    assert_eq!(suffix_steps, 5484);
    assert!(pass, "inspect persisted raw receipts");
}

#[test]
#[ignore = "child process entry point; requires a recorded parent cell"]
fn continuation_child_resume() {
    let dir = std::path::PathBuf::from(
        std::env::var_os("ERE_CHECKPOINT_CHILD").expect("ERE_CHECKPOINT_CHILD"),
    );
    let target = dir.join("child_resume.json");
    assert!(!target.exists(), "refuse child receipt overwrite");
    let receipt: Value =
        serde_json::from_slice(&std::fs::read(dir.join("receipt.json")).unwrap()).unwrap();
    let bytes = std::fs::read(dir.join("checkpoint_08.bin")).unwrap();
    let interval = receipt["effective_config"]["engine"]["metrics_window_interval"]
        .as_u64()
        .unwrap();
    let expected = RuleProgram {
        core: candidate(false),
        proposal: Some(candidate(true)),
        active: true,
    };
    let mut back = restore(&bytes, interval, &expected);
    let mut suffix = vec![];
    while back.store.step() < 64 {
        suffix.push(tick(&mut back));
    }
    let terminal_window =
        back.runtime
            .close_window(&back.store, MetricsRow::default(), MetricsRow::default());
    let terminal_guards = back.engine.drain_guard_stats();
    let pass = suffix == receipt["baseline"].as_array().unwrap()[8..]
        && serde_json::to_value(&terminal_window).unwrap() == receipt["terminal_window"]
        && serde_json::to_value(&terminal_guards).unwrap() == receipt["terminal_guards"];
    save(
        &target,
        &json!({"cut":8,"process_id":std::process::id(),"suffix_steps":suffix.len(),"suffix":suffix,
        "terminal_window":terminal_window,"terminal_guards":terminal_guards,"pass":pass,"training_admitted":false}),
    );
    assert!(pass, "inspect child_resume.json");
}
