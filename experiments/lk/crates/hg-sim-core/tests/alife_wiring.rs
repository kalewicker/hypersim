//! S3 wiring (scope §4): the `alife` feature appends the biology to the rule
//! set and injects the carrier classifier. Only compiled under `--features
//! alife`. Also covers the S3.1 bootstrap_seed gate, since the wiring's load
//! target must be a valid, correctly-firing rule file.
#![cfg(feature = "alife")]

use hg_core::{HypergraphStore, Label, NodeData, PropertyValue};
use hg_engine::Engine;
use hg_rules::{compile_rules, parse_rules_toml, StepMode};
use hg_sim_core::{run_headless, runner::load_rules, SimConfig};

fn biology_rule_count() -> usize {
    parse_rules_toml(hg_alife::alife_base_toml())
        .expect("alife_base.toml parses")
        .len()
}

#[test]
fn alife_base_toml_parses_and_compiles() {
    let rules = parse_rules_toml(hg_alife::alife_base_toml()).expect("parses");
    assert!(!rules.is_empty(), "alife_base.toml has at least one rule");
    assert!(
        rules.iter().any(|r| r.name == "bootstrap_seed"),
        "bootstrap_seed present"
    );
    // Compiles without error (RHS labels, property effects all resolve).
    compile_rules(&rules).expect("alife_base.toml compiles");
}

#[test]
fn load_rules_appends_biology_only_when_enabled() {
    let base = r#"
experiment_name = "w"
run_id = "w"
seed = 1
[rules]
source = "Builtin"
builtin_name = "triangle_subdivide"
[causal]
enabled = false
[evaluator]
enabled = false
"#;
    // Disabled: base rules only.
    let cfg_off = SimConfig::from_toml(base).unwrap();
    let n_base = load_rules(&cfg_off).unwrap().len();

    // Enabled: base + biology.
    let cfg_on = SimConfig::from_toml(&format!("{base}\n[alife]\nenabled = true\n")).unwrap();
    let n_on = load_rules(&cfg_on).unwrap().len();

    assert_eq!(
        n_on,
        n_base + biology_rule_count(),
        "enabling alife must append exactly the biology rules"
    );
    assert!(n_base > 0, "base builtin should contribute rules");
}

/// The new initial condition (scope §6.3): `seed_carrier_marker` marks exactly
/// one substrate node, and driving the configured (base + biology) rules from
/// that store yields exactly one carrier. Exercises build_initial_store +
/// load_rules together, config-driven.
#[test]
fn seed_carrier_marker_bootstraps_one_carrier_from_config() {
    let toml = r#"
experiment_name = "seed"
run_id = "seed"
seed = 7
[rules]
source = "Inline"
inline_toml = "rules = []"
[initial]
condition_type = "PureCrystal"
width = 5
height = 5
arity = 3
[engine]
step_mode = "MaxParallel"
[alife]
enabled = true
seed_carrier_marker = true
"#;
    let config = SimConfig::from_toml(toml).unwrap();
    let mut store = hg_sim_core::runner::build_initial_store(&config);
    let marked = store
        .nodes()
        .filter(|(_, d)| d.get_property(20) == Some(PropertyValue::Bool(true)))
        .count();
    assert_eq!(marked, 1, "exactly one carrier_marker seeded on the substrate");

    let rules = load_rules(&config).unwrap();
    let mut engine = Engine::new(
        rules,
        config.engine.step_mode,
        config.seed,
        config.engine.max_pairings_per_step,
    );
    for _ in 0..4 {
        engine.step(&mut store);
    }
    let carriers = store
        .nodes()
        .filter(|(_, d)| d.label == hg_alife::CARRIER_LABEL)
        .count();
    assert_eq!(carriers, 1, "bootstrap produced one carrier from the seeded marker");
}

/// End-to-end: a fully-wired alife run (causal + classifier injected) completes,
/// and the seeded carrier actually enters the graph — the final canonical hash
/// differs from the same run with no carrier seeded.
#[test]
fn alife_run_completes_and_carrier_changes_canonical_hash() {
    let base = r#"
experiment_name = "e2e"
run_id = "e2e"
seed = 11
[rules]
source = "Inline"
inline_toml = "rules = []"
[initial]
condition_type = "PureCrystal"
width = 5
height = 5
arity = 3
[engine]
step_mode = "MaxParallel"
max_steps = 6
metrics_window_interval = 3
[evaluator]
enabled = false
[causal]
enabled = true
"#;
    let run = |seed_marker: bool, tag: &str| {
        let mut c = SimConfig::from_toml(&format!(
            "{base}\n[alife]\nenabled = true\nseed_carrier_marker = {seed_marker}\n"
        ))
        .unwrap();
        c.out_dir = std::env::temp_dir().join(format!("hypersim_alife_e2e_{tag}"));
        let _ = std::fs::remove_dir_all(&c.out_dir);
        run_headless(&c, None, None, None).unwrap().canonical_hash
    };
    assert_ne!(
        run(true, "with"),
        run(false, "without"),
        "seeding a carrier must change the final canonical hash (the carrier entered the graph)"
    );
}

/// S3.1 gate: a marked substrate node sprouts exactly one carrier, attached,
/// marker cleared; further steps create no more carriers.
#[test]
fn bootstrap_seed_creates_exactly_one_carrier() {
    let rules = compile_rules(&parse_rules_toml(hg_alife::alife_base_toml()).unwrap()).unwrap();
    let mut store = HypergraphStore::new();
    // One marked site plus a few unmarked substrate nodes it must not affect.
    let site = store.add_node(NodeData::new(Label(0), 0));
    store
        .get_node_mut(site)
        .unwrap()
        .set_property(20, PropertyValue::Bool(true));
    for _ in 0..3 {
        store.add_node(NodeData::new(Label(0), 0));
    }

    let mut engine = Engine::new(rules, StepMode::MaxParallel, 1, 1000);
    engine.step(&mut store);

    let carriers: Vec<_> = store
        .nodes()
        .filter(|(_, d)| d.label == hg_alife::CARRIER_LABEL)
        .map(|(id, _)| id)
        .collect();
    assert_eq!(carriers.len(), 1, "exactly one carrier after bootstrap");
    let carrier = carriers[0];

    // Marker cleared on the site.
    assert_eq!(
        store.get_node(site).unwrap().get_property(20),
        Some(PropertyValue::Bool(false)),
        "the carrier_marker must be cleared so bootstrap fires once"
    );
    // Carrier seeded with energy 10 and a genome.
    assert_eq!(
        store.get_node(carrier).unwrap().get_property(1),
        Some(PropertyValue::Float(10.0))
    );
    assert_eq!(
        store.get_node(carrier).unwrap().get_property(11),
        Some(PropertyValue::Float(15.0)),
        "replication_threshold set"
    );
    // Occupancy edge: the carrier is attached to the site.
    let attached = store
        .edges()
        .any(|(_, e)| e.nodes.contains(&site) && e.nodes.contains(&carrier));
    assert!(attached, "carrier must be attached to the seeded site by an edge");

    // Many more steps create no further carriers (one-shot).
    for _ in 0..10 {
        engine.step(&mut store);
    }
    let n_carriers = store
        .nodes()
        .filter(|(_, d)| d.label == hg_alife::CARRIER_LABEL)
        .count();
    assert_eq!(n_carriers, 1, "bootstrap_seed must not refire");
}
