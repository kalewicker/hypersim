//! B1 LIVENESS PROBE — fire counts only, no alphabet cardinality.
//!
//! Mirrors `docs/policy_config.md` §2.5, whose measured liveness matrix gated
//! Stage 0a: a rule set only matches on an initial condition containing its
//! LHS, and composing rules that never fire would produce a rule-count series
//! whose rows are void by B1.2's own check (`A_rule_only` must equal the rule
//! count). Run BEFORE the B1 composition is pre-registered, so the scheme is
//! fixed by measured liveness rather than by guesswork — and deliberately
//! reporting nothing about alphabets, so no cardinality exists before the
//! pre-registration commit.

use hg_alphabet::run::Init;
use hg_engine::Engine;
use hg_observe::{Observer, RewriteEvent, StepStats};
use hg_rules::{compile_rules, parse_rules_toml, StepMode};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

struct Counter {
    by_rule: Arc<Mutex<BTreeMap<String, u64>>>,
    early: Arc<Mutex<BTreeMap<String, u64>>>,
    step: Arc<Mutex<u64>>,
}

impl Observer for Counter {
    fn on_step_end(
        &mut self,
        _s: &hg_core::GraphSnapshot<'_>,
        _st: &StepStats,
        events: &[RewriteEvent],
    ) {
        let step = *self.step.lock().unwrap();
        let mut m = self.by_rule.lock().unwrap();
        let mut e = self.early.lock().unwrap();
        for ev in events {
            *m.entry(ev.rule_name.clone()).or_insert(0) += 1;
            if step < 50 {
                *e.entry(ev.rule_name.clone()).or_insert(0) += 1;
            }
        }
        *self.step.lock().unwrap() = step + 1;
    }
}

fn probe(label: &str, toml: &str, init: &Init, steps: u64) {
    let mut store = init.build();
    let n0 = store.node_count();
    let rules = compile_rules(&parse_rules_toml(toml).expect("parse")).expect("compile");
    let names: Vec<String> = rules.iter().map(|r| r.name.clone()).collect();
    let mut engine = Engine::new(rules, StepMode::MaxParallel, 1, 1_000_000);
    let by_rule = Arc::new(Mutex::new(BTreeMap::new()));
    let early = Arc::new(Mutex::new(BTreeMap::new()));
    engine.add_observer(Box::new(Counter {
        by_rule: Arc::clone(&by_rule),
        early: Arc::clone(&early),
        step: Arc::new(Mutex::new(0)),
    }));
    let mut quiet = 0;
    let mut ran = 0;
    for _ in 0..steps {
        let st = engine.step(&mut store);
        ran += 1;
        if st.accepted_rewrites == 0 {
            quiet += 1;
            if quiet >= 8 {
                break;
            }
        } else {
            quiet = 0;
        }
        if store.node_count() > 200_000 {
            break;
        }
    }
    let m = by_rule.lock().unwrap();
    let e = early.lock().unwrap();
    let total: u64 = m.values().sum();
    eprintln!(
        "\n  === {label} ===  steps_run={ran}/{steps}  n {n0} -> {}  edges {}  events={total}",
        store.node_count(),
        store.edge_count()
    );
    for n in &names {
        let c = m.get(n).copied().unwrap_or(0);
        eprintln!(
            "      {n:<22} fired={c:<9} (first 50 steps: {:<7}) {}",
            e.get(n).copied().unwrap_or(0),
            if c == 0 { "<< DEAD" } else { "" }
        );
    }
}

const INFLUX: &str = r#"
[[rules]]
name = "influx"
priority = 0
[rules.lhs]
nodes = ["s"]
edges = []
[rules.lhs.node_labels]
s = 0
[[rules.lhs.node_property_guards]]
var = "s"
property = 1
op = "lt"
value = 20.0
[rules.rhs]
nodes = ["s"]
edges = []
[[rules.rhs.property_effects]]
var = "s"
key = 1
op = "add"
value = 1.0
"#;

const CHURN: &str = r#"
[[rules]]
name = "churn"
priority = 0
[rules.lhs]
nodes = ["a", "old", "b"]
edges = [["a", "old"], ["old", "b"]]
[rules.lhs.node_labels]
a = 0
old = 0
b = 0
[[rules.lhs.node_property_guards]]
var = "old"
property = 1
op = "gte"
value = 20.0
[[rules.lhs.node_property_guards]]
var = "old"
property = 22
op = "eq"
value = false
[[rules.lhs.var_guards]]
type = "vars_distinct"
vars = ["a", "old", "b"]
[rules.rhs]
nodes = ["a", "fresh", "b"]
edges = [["a", "fresh"], ["fresh", "b"]]
[rules.rhs.node_labels]
fresh = 0
[[rules.rhs.property_effects]]
var = "fresh"
key = 1
op = "set"
value = 0.0
[[rules.rhs.property_effects]]
var = "fresh"
key = 22
op = "set"
value = false
"#;

/// `conservation_soak`'s transfer rule, verbatim apart from the flattened
/// priority: conservative 10-unit move across a 2-edge, node count neutral.
const TRANSFER: &str = r#"
[[rules]]
name = "transfer"
priority = 0
[rules.lhs]
nodes = ["src", "dst"]
edges = [["src", "dst"]]
[[rules.lhs.node_property_guards]]
var = "src"
property = 1
op = "gt"
value = 10.0
[rules.rhs]
nodes = ["src", "dst"]
edges = [["src", "dst"]]
[[rules.rhs.property_effects]]
var = "src"
key = 1
op = "subtract"
value = 10.0
[[rules.rhs.property_effects]]
var = "dst"
key = 1
op = "add"
value = 10.0
"#;

const GLIDER: &str = include_str!("../../../rules/glider.toml");

/// `lk_transport`: conservative diffusion, but gated on a 50-unit gradient.
const LK_TRANSPORT: &str = r#"
[[rules]]
name = "lk_transport"
priority = 0
[rules.lhs]
nodes = ["x", "y"]
edges = [["x", "y"]]
[[rules.lhs.var_guards]]
type = "vars_distinct"
vars = ["x", "y"]
[[rules.lhs.node_property_compares]]
var_a = "x"
key_a = 1
op = "gte"
var_b = "y"
key_b = 1
offset = 50.0
[rules.rhs]
nodes = ["x", "y"]
edges = [["x", "y"]]
[[rules.rhs.property_effects]]
var = "x"
key = 1
op = "subtract"
value = 25.0
[[rules.rhs.property_effects]]
var = "y"
key = 1
op = "add"
value = 25.0
"#;

/// `excitation_front`'s excite rule — node-count neutral, but monotone.
const EXCITE: &str = r#"
[[rules]]
name = "excite"
priority = 0
[rules.lhs]
nodes = ["a", "b"]
edges = [["a", "b"]]
[rules.lhs.node_labels]
a = 1
b = 0
[[rules.lhs.var_guards]]
type = "vars_distinct"
vars = ["a", "b"]
[rules.rhs]
nodes = ["a", "b"]
edges = [["a", "b"]]
[rules.rhs.node_labels]
b = 1
"#;

fn composite() -> Init {
    Init::CompositeChurnRing {
        w: 6,
        h: 6,
        arity: 3,
        fill: 5.0,
        ring: 24,
    }
}

#[test]
#[ignore = "liveness probe"]
fn b1_liveness_probe() {
    eprintln!("\n########## B1 LIVENESS PROBE (fire counts only; NO alphabet) ##########");
    eprintln!("  Substrate: CompositeChurnRing(6x6 arity3 fill 5.0) + GliderRing(24), MaxParallel.");

    // Does a single rule sustain itself? The nesting base must be perpetual.
    probe("1: glider alone", GLIDER, &composite(), 2000);
    probe("1: influx alone", INFLUX, &composite(), 2000);
    probe(
        "2: glider + influx",
        &format!("{GLIDER}\n{INFLUX}"),
        &composite(),
        2000,
    );
    probe(
        "3: glider + influx + churn",
        &format!("{GLIDER}\n{INFLUX}\n{CHURN}"),
        &composite(),
        2000,
    );
    probe(
        "4: + transfer",
        &format!("{GLIDER}\n{INFLUX}\n{CHURN}\n{TRANSFER}"),
        &composite(),
        2000,
    );
    // Fifth-rule candidates, each added to the 4-rule set.
    probe(
        "5a: + lk_transport (gradient-gated)",
        &format!("{GLIDER}\n{INFLUX}\n{CHURN}\n{TRANSFER}\n{LK_TRANSPORT}"),
        &composite(),
        2000,
    );
    probe(
        "5b: + excite (monotone relabel)",
        &format!("{GLIDER}\n{INFLUX}\n{CHURN}\n{TRANSFER}\n{EXCITE}"),
        &composite(),
        2000,
    );
    eprintln!("\n######################################################################\n");
}
