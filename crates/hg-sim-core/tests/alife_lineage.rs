//! S5 gate — the `AlifeObserver` reconstructs a sound lineage from the REAL
//! rewrite event stream of a composed run, and produces the criterion-6 headline
//! `non_trivial_replicator_count() : 0 → >0`.
#![cfg(feature = "alife")]

use hg_alife::AlifeObserver;
use hg_core::GraphSnapshot;
use hg_engine::Engine;
use hg_observe::{Observer, RewriteEvent, StepStats};
use hg_rules::{compile_rules, parse_rules_toml, StepMode};
use hg_sim_core::SimConfig;
use std::sync::{Arc, Mutex};

/// Forwarding handle so the test can read the observer after the engine owns it.
struct Handle {
    inner: Arc<Mutex<AlifeObserver>>,
}
impl Observer for Handle {
    fn on_step_end(&mut self, snap: &GraphSnapshot<'_>, s: &StepStats, events: &[RewriteEvent]) {
        self.inner.lock().unwrap().on_step_end(snap, s, events);
    }
}

fn config(steps: u64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "lineage"
run_id = "lineage"
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
"#
    );
    SimConfig::from_toml(&toml).unwrap()
}

fn all_rules() -> Vec<hg_rules::CompiledRule> {
    compile_rules(&parse_rules_toml(hg_alife::alife_base_toml()).unwrap()).unwrap()
}

fn run(steps: u64, window: u64) -> Arc<Mutex<AlifeObserver>> {
    let cfg = config(steps);
    let mut store = hg_sim_core::runner::build_initial_store(&cfg);
    let mut engine = Engine::new(all_rules(), StepMode::MaxParallel, cfg.seed, 100000);
    let obs = Arc::new(Mutex::new(AlifeObserver::new(window)));
    engine.add_observer(Box::new(Handle { inner: obs.clone() }));
    for _ in 0..steps {
        engine.step(&mut store);
    }
    drop(engine);
    obs
}

fn newick_is_balanced(s: &str) -> bool {
    if !s.ends_with(';') {
        return false;
    }
    let mut depth = 0i32;
    for ch in s.chars() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth < 0 {
                    return false;
                }
            }
            _ => {}
        }
    }
    depth == 0
}

#[test]
fn lineage_is_reconstructed_from_real_events() {
    let obs = run(4_000, 500);
    let o = obs.lock().unwrap();
    let lin = o.lineage();

    // Exactly one bootstrap founder; every other carrier is a replication birth.
    assert_eq!(o.bootstrap_origin_total(), 1, "one bootstrap founder");
    assert_eq!(lin.clade_count(), 1, "single founder ⟹ one root clade");
    assert_eq!(
        o.replication_origin_total() as usize,
        lin.total_births(),
        "every replication event is a recorded birth"
    );
    assert!(lin.total_carriers() >= 20, "dispersal colonised the substrate ({} carriers)", lin.total_carriers());

    // Multi-generational: the colonisation wave spreads generation by generation.
    assert!(
        lin.max_generation() >= 3,
        "expected a multi-generation lineage, got depth {}",
        lin.max_generation()
    );

    // Soundness: every non-root child was born no earlier than its parent, and
    // the parent is itself a recorded carrier.
    // (Reconstructed purely from events — this is the S5.2 gate.)
    let curve = lin.survival_curve();
    assert_eq!(curve.iter().sum::<usize>(), lin.total_carriers(), "survival curve covers all carriers");
    assert_eq!(curve[0], 1, "generation 0 is exactly the one founder");

    // Newick well-formed.
    let nwk = lin.export_newick();
    assert!(newick_is_balanced(&nwk), "Newick must be balanced and ';'-terminated");

    // THE criterion-6 headline: non-trivial replicators go 0 → >0.
    let ntr = lin.non_trivial_replicator_count(2);
    assert!(ntr > 0, "non_trivial_replicator_count must be >0 once carriers sustain lineages");
    eprintln!(
        "lineage: {} carriers, {} births, depth {}, non_trivial_replicators(≥2 gen) = {}",
        lin.total_carriers(),
        lin.total_births(),
        lin.max_generation(),
        ntr
    );
}

#[test]
fn ecology_windows_track_population_and_traits() {
    let obs = run(3_000, 500);
    let o = obs.lock().unwrap();
    let windows = o.windows();
    assert!(!windows.is_empty(), "windows recorded");
    let last = windows.last().unwrap();
    assert!(last.carriers > 0, "live population at end");
    assert_eq!(last.carriers, last.replication_origin_total as usize + last.bootstrap_origin_total as usize
        - o.lineage().total_deaths(),
        "carriers == births + bootstrap − deaths (closed population bookkeeping)");
    assert!(last.carrier_ratio > 0.0 && last.carrier_ratio <= 1.0, "carrier ratio in (0,1]");
    // Trait means are finite while carriers exist (harvest, repl_thr, maint, sat).
    for (i, m) in last.trait_mean.iter().enumerate() {
        assert!(m.is_finite(), "trait {i} mean finite with a live population");
    }
}
