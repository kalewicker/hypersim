//! S6 completeness — `AlifeConfig.child_node_cost` wired through the runner.
//!
//! `child_node_cost` is a SYMMETRIC SCALE (four coupled values: fundability guard,
//! target payment, child birth energy 0.8c, death recycle 0.8c). The `alife_s6_child_cost_sweep`
//! test discovered its behaviour by PATCHING those four in-test (`rules_with`). This
//! test proves the new config field drives the runner's `load_rules` to the exact
//! same rule set — a config file reproduces the sweep with no in-test mutation.
#![cfg(feature = "alife")]

use hg_core::{GraphSnapshot, PropertyValue};
use hg_engine::Engine;
use hg_observe::{Observer, RewriteEvent, StepStats};
use hg_rules::{compile_rules, parse_rules_toml, CompiledRule, EffectOpDecl, StepMode};
use hg_sim_core::SimConfig;
use std::sync::{Arc, Mutex};

const CARRIER: hg_core::Label = hg_core::Label(100);

/// The in-test patcher, copied from `alife_s6_child_cost_sweep::rules_with`, is the
/// reference the config path must match exactly.
fn rules_with(influx: f64, c: f64) -> Vec<CompiledRule> {
    let birth = 0.8 * c;
    let mut raw = parse_rules_toml(hg_alife::alife_base_toml()).unwrap();
    let mut hits = 0;
    for r in raw.iter_mut() {
        match r.name.as_str() {
            "influx" => {
                for eff in &mut r.rhs_property_effects {
                    if eff.key == 1 {
                        if let EffectOpDecl::Add { value } = &mut eff.op {
                            *value = PropertyValue::Float(influx);
                            hits += 1;
                        }
                    }
                }
            }
            "replicate" => {
                for gd in &mut r.lhs_node_property_guards {
                    if gd.var == "target" && gd.property == 1 {
                        gd.value = PropertyValue::Float(c);
                        hits += 1;
                    }
                }
                for eff in &mut r.rhs_property_effects {
                    if eff.var == "target" && eff.key == 1 {
                        if let EffectOpDecl::Subtract { value } = &mut eff.op {
                            *value = PropertyValue::Float(c);
                            hits += 1;
                        }
                    }
                    if eff.var == "child" && eff.key == 1 {
                        if let EffectOpDecl::Set { value } = &mut eff.op {
                            *value = PropertyValue::Float(birth);
                            hits += 1;
                        }
                    }
                }
            }
            "die_and_recycle" => {
                for eff in &mut r.rhs_property_effects {
                    if eff.var == "site" && eff.key == 1 {
                        if let EffectOpDecl::Add { value } = &mut eff.op {
                            *value = PropertyValue::Float(birth);
                            hits += 1;
                        }
                    }
                }
            }
            _ => {}
        }
    }
    assert_eq!(hits, 5, "expected to patch 5 coupled values");
    compile_rules(&raw).unwrap()
}

fn config_with(influx: f64, c: f64, steps: u64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "cc_cfg"
run_id = "cc_cfg"
seed = 12345
[rules]
source = "Inline"
inline_toml = "rules = []"
[initial]
condition_type = "PureCrystal"
width = 6
height = 6
arity = 3
[engine]
step_mode = "MaxParallel"
max_steps = {steps}
max_pairings_per_step = 100000
[evaluator]
enabled = false
[causal]
enabled = false
[alife]
enabled = true
seed_carrier_marker = true
initial_substrate_energy = 5.0
influx_rate = {influx}
child_node_cost = {c}
"#
    );
    SimConfig::from_toml(&toml).unwrap()
}

#[derive(Default)]
struct Traj {
    births: Vec<u32>,
    deaths: Vec<u32>,
    carriers: Vec<usize>,
}
struct Handle {
    inner: Arc<Mutex<Traj>>,
}
impl Observer for Handle {
    fn on_step_end(&mut self, snap: &GraphSnapshot<'_>, _s: &StepStats, events: &[RewriteEvent]) {
        let mut o = self.inner.lock().unwrap();
        o.births.push(events.iter().filter(|e| e.rule_name == "replicate").count() as u32);
        o.deaths.push(events.iter().filter(|e| e.rule_name == "die_and_recycle").count() as u32);
        o.carriers.push(snap.nodes().filter(|(_, d)| d.label == CARRIER).count());
    }
}

fn run(rules: Vec<CompiledRule>, config: &SimConfig, steps: u64) -> Traj {
    let mut store = hg_sim_core::runner::build_initial_store(config);
    let mut engine = Engine::new(rules, StepMode::MaxParallel, config.seed, 100000);
    let obs = Arc::new(Mutex::new(Traj::default()));
    engine.add_observer(Box::new(Handle { inner: obs.clone() }));
    for _ in 0..steps {
        engine.step(&mut store);
    }
    drop(engine);
    Arc::try_unwrap(obs).map_err(|_| ()).unwrap().into_inner().unwrap()
}

/// Config-driven child_node_cost reproduces the in-test patcher's dynamics EXACTLY.
#[test]
fn s6_child_cost_config_matches_in_test_patcher() {
    let steps = 4_000u64;
    let (influx, c) = (0.55, 2.0);
    let config = config_with(influx, c, steps);

    // Reference: rules patched in-test. Subject: rules from the runner's load_rules
    // driven by the config's child_node_cost + influx_rate fields.
    let reference = run(rules_with(influx, c), &config, steps);
    let subject = run(hg_sim_core::runner::load_rules(&config).unwrap(), &config, steps);

    assert_eq!(subject.births, reference.births, "birth series identical");
    assert_eq!(subject.deaths, reference.deaths, "death series identical");
    assert_eq!(subject.carriers, reference.carriers, "carrier trajectory identical");
}

/// The stillbirth-floor property the field exists to explore: a very low
/// child_node_cost yields fragile newborns (birth energy 0.8c near maintenance 0.5),
/// which is REACHABLE only because the cost is now config — the base toml is pinned
/// at 10. We just assert the override actually changes birth energy vs the default.
#[test]
fn s6_child_cost_override_changes_child_birth_energy() {
    // Default (None) child key-1 Set value is BIRTH_ENERGY 0.8*10 = 8.0; override to
    // c=2 makes it 1.6. Read it straight off the parsed+overridden replicate rule.
    fn child_set_value(config: &SimConfig) -> Option<f64> {
        // Parse the base biology and apply the exact override entrypoint the runner
        // uses inside load_rules, then read the child's birth-energy literal.
        let mut biology = parse_rules_toml(hg_alife::alife_base_toml()).unwrap();
        hg_sim_core::runner::apply_alife_overrides(&mut biology, &config.alife);
        for r in &biology {
            if r.name == "replicate" {
                for eff in &r.rhs_property_effects {
                    if eff.var == "child" && eff.key == 1 {
                        if let EffectOpDecl::Set { value } = &eff.op {
                            if let PropertyValue::Float(f) = value {
                                return Some(*f);
                            }
                        }
                    }
                }
            }
        }
        None
    }

    let default_cfg = config_with(0.55, 10.0, 10); // c=10 == base default
    let low_cfg = config_with(0.55, 2.0, 10);
    assert_eq!(child_set_value(&default_cfg), Some(8.0), "c=10 → birth 8.0");
    assert_eq!(child_set_value(&low_cfg), Some(1.6), "c=2 → birth 1.6 (fragile)");
}
