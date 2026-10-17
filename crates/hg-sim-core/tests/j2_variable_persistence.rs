//! J2.2 — cheap well-definedness probe for the non-partition fallback (V5 / J-track).
//!
//! Criterion committed in `docs/validation_v5_j2_variable_persistence_criterion.md`
//! (J2.1, commit 57ff48f). A ΦID/causal-decoupling measure needs a variable set
//! that PERSISTS across the window. Horn 2 (churn-invariant aggregates sit at the
//! F0-fatal resolution) is already measured by F0/V1 (p=0.495 at the (9,1)/label
//! resolution). This probe confirms horn 1 empirically: under substrate motion,
//! does any FIXED micro-variable set (node identities) persist, or do only global
//! aggregates survive?
//!
//! Metric: the persistent micro-variable set = nodes present in EVERY step of the
//! window (a candidate fixed variable set for a dependency measure). On the static
//! substrate this is the fixed lattice (why CE/ΦID could be posed at all); under
//! motion it should collapse. Same builtins as H0.3, same compose-from-raw path.
#![cfg(feature = "alife")]

use hg_alife::CARRIER_LABEL;
use hg_core::{Label, NodeId};
use hg_engine::Engine;
use hg_rules::{builtin_rules, compile_rules, parse_rules_toml, StepMode};
use hg_sim_core::runner::apply_alife_overrides;
use hg_sim_core::SimConfig;
use std::collections::HashSet;

const WL_DEPTH_NOTE: u8 = 2;
const NODE_CAP: usize = 25_000;

fn config(seed: u64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "j2"
run_id = "j2"
seed = {seed}
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
max_steps = 6000
metrics_window_interval = 128
max_pairings_per_step = 100000
[evaluator]
enabled = false
[causal]
enabled = true
[alife]
enabled = true
seed_carrier_marker = true
initial_substrate_energy = 5.0
influx_rate = 0.34
bootstrap_replication_threshold = 3.0
"#
    );
    SimConfig::from_toml(&toml).unwrap()
}

fn build_rules(cfg: &SimConfig, motion: Option<&str>) -> Vec<hg_rules::CompiledRule> {
    let mut raw = Vec::new();
    if let Some(name) = motion {
        raw.extend(builtin_rules(name).unwrap());
    }
    let mut biology = parse_rules_toml(hg_alife::alife_base_toml()).unwrap();
    apply_alife_overrides(&mut biology, &cfg.alife);
    raw.extend(biology);
    compile_rules(&raw).unwrap()
}

struct Persist {
    steps: u64,
    n_start: usize,
    n_end: usize,
    persistent_all: usize,   // nodes present in EVERY step (fixed micro-variable set)
    persistent_substrate: usize,
    persistent_carrier: usize,
}

fn run(seed: u64, motion: Option<&str>, window: u64) -> Persist {
    let cfg = config(seed);
    let rules = build_rules(&cfg, motion);
    let mut store = hg_sim_core::runner::build_initial_store(&cfg);
    if motion == Some("growth_treadmill") {
        let first = store.nodes().next().map(|(id, _)| id);
        if let Some(id) = first {
            store.get_node_mut(id).unwrap().label = Label(1);
        }
    }
    let n_start = store.nodes().count();
    let mut engine = Engine::new(rules, StepMode::MaxParallel, cfg.seed, 100_000);

    // survivors = nodes present at step 1, intersected with every later step's set.
    let mut survivors: Option<HashSet<NodeId>> = None;
    let mut n_end = n_start;
    let mut steps = 0u64;
    for step in 1..=window {
        engine.step(&mut store);
        let present: HashSet<NodeId> = store.nodes().map(|(id, _)| id).collect();
        n_end = present.len();
        survivors = Some(match survivors {
            None => present,
            Some(s) => s.intersection(&present).copied().collect(),
        });
        steps = step;
        if n_end > NODE_CAP {
            break;
        }
    }
    let surv = survivors.unwrap_or_default();
    let (mut ps, mut pc) = (0usize, 0usize);
    for &id in &surv {
        match store.get_node(id) {
            Some(d) if d.label == CARRIER_LABEL => pc += 1,
            Some(_) => ps += 1,
            None => {}
        }
    }
    Persist { steps, n_start, n_end, persistent_all: surv.len(), persistent_substrate: ps, persistent_carrier: pc }
}

#[test]
fn j2_variable_persistence_probe() {
    let seed = 12345;
    let window = 120;
    eprintln!("\n================ J2.2 — MICRO-VARIABLE PERSISTENCE UNDER MOTION (seed {seed}, W={window}) ================");
    eprintln!("  (persistent = node present in EVERY step of the window = a candidate FIXED variable for a dependency measure)");
    eprintln!("  run                    | steps | n_start->end | persistent (all / substrate / carrier)");
    eprintln!("  -----------------------+-------+--------------+----------------------------------------");
    for (label, motion) in [
        ("static (baseline)", None),
        ("growth_treadmill", Some("growth_treadmill")),
        ("triangle_subdivide", Some("triangle_subdivide")),
    ] {
        let p = run(seed, motion, window);
        eprintln!(
            "  {:<22} | {:>5} | {:>5}->{:>6} | {:>3} / {:>3} / {:>3}",
            label, p.steps, p.n_start, p.n_end, p.persistent_all, p.persistent_substrate, p.persistent_carrier
        );
    }
    eprintln!("=======================================================================================================\n");
    eprintln!("HONEST reading (corrects a naive prediction): the available builtins GROW (add nodes) but do NOT");
    eprintln!("DELETE — so original node identities PERSIST (37-48 survive the whole window). They therefore do");
    eprintln!("NOT test the deleting-churn case; no builtin churns-without-growing (same gap the route-A probe");
    eprintln!("found). So J2.2 cannot empirically show node-identity churn. What it DOES show, and what matters:");
    eprintln!("the CARRIER population turns over as individuals (<=1 persists) on static AND motion alike — carrier");
    eprintln!("micro-variables are never a persistent fixed set. F8 handled this NOT with persistent carrier");
    eprintln!("variables but with a CLASS resolution (4 site-geometric WL classes) — which J0 showed is borrowed");
    eprintln!("and H0 showed ephemeralises under motion. So the decisive J2 question is RESOLUTION: is there a");
    eprintln!("churn-invariant carrier resolution above the F0 bar? Answered by the rule-4 energy-band check in");
    eprintln!("`j2_churn_invariant_resolution.rs`. (WL depth ref {WL_DEPTH_NOTE}.)");
}
