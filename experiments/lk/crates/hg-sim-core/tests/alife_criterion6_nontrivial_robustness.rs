//! Criterion 6 validation — is the `non_trivial_replicator_count` 0 → >0 crossing
//! REAL, or a label/WL-class artifact?
//!
//! Criterion 6 is the headline and the literal partition Experiment 6 coarse-grains
//! over, so a spurious crossing gives a clean CE number across nothing. Two worries,
//! both addressed here:
//!
//! (A) NEGATIVE CONTROL / firewall: can the instrument WITHHOLD `non_trivial` from a
//!     hand-built trivial copier? The measure must NOT assign it to everything that
//!     merely carries the carrier label. We build lineages where every node shares
//!     the SAME carrier label/genome but the lineage is SHALLOW (one-off births, and
//!     a prolific-but-shallow star) and confirm the count stays 0. Then a genuinely
//!     SUSTAINED chain flips it >0. The count depends on lineage DEPTH, not label.
//!
//! (B) STRUCTURAL CROSSING: in a real run, does the 0 → >0 crossing coincide with an
//!     actual depth-K sustained-self-copying event, or does it fire for an
//!     uninteresting reason? We track the per-window count over the turnover run and
//!     assert the crossing window is exactly where a lineage first reaches depth
//!     `NON_TRIVIAL_MIN_GENERATIONS`, and that no such depth existed before it.
//!
//! By construction `non_trivial_replicator_count` reads only parent/child topology
//! (no `motif_class`, no WL class, no label beyond identifying carriers at record
//! time) — so it CANNOT be a WL-class labelling artifact. (A) demonstrates that
//! operationally; (B) ties the crossing to a structural event.
#![cfg(feature = "alife")]

use hg_alife::genome::Genome;
use hg_alife::{AlifeObserver, LineageTree};
use hg_alife::observer::NON_TRIVIAL_MIN_GENERATIONS as MINGEN;
use hg_core::{GraphSnapshot, HypergraphStore, Label, NodeData, NodeId};
use hg_engine::Engine;
use hg_observe::{Observer, RewriteEvent, StepStats};
use hg_rules::StepMode;
use hg_sim_core::SimConfig;
use std::sync::{Arc, Mutex};

const CARRIER: Label = Label(100);

fn g() -> Genome {
    // Every carrier IDENTICAL: same label, same genome. If the count were a
    // label/class artifact it would fire here regardless of lineage shape.
    Genome { harvest_rate: Some(1.0), replication_threshold: Some(3.0), maintenance_cost: Some(1.0), satiation: Some(20.0) }
}
fn ids(n: usize) -> Vec<NodeId> {
    let mut s = HypergraphStore::new();
    (0..n).map(|_| s.add_node(NodeData::new(CARRIER, 0))).collect()
}

// ---- (A) negative control: trivial copiers are withheld ---------------------
#[test]
fn criterion6_withholds_nontrivial_from_trivial_copiers() {
    // ALL ids from ONE pool so keys are distinct (mimicking a single engine store;
    // minting from separate throwaway stores would alias slot keys and fabricate a
    // cycle — the lineage is a tree by construction in a real run).
    let v = ids(60);

    // Case 1: a chain of one-off births — founder→a (a never reproduces).
    let mut t = LineageTree::new();
    t.record_birth(v[0], None, g(), 1);
    t.record_birth(v[1], Some(v[0]), g(), 5); // founder is a parent, but depth 1
    assert_eq!(
        t.non_trivial_replicator_count(MINGEN),
        0,
        "a one-off birth (depth {MINGEN} not reached) is a TRIVIAL copier — must be withheld"
    );

    // Case 2: a PROLIFIC but SHALLOW copier — founder with 50 children, none of whom
    // reproduce. Fecundity alone is not non-triviality; deepest descendant depth = 1.
    let mut t2 = LineageTree::new();
    t2.record_birth(v[2], None, g(), 1);
    for (k, &child) in v[3..53].iter().enumerate() {
        t2.record_birth(child, Some(v[2]), g(), 5 + k as u64);
    }
    assert_eq!(t2.total_births(), 50, "50 children recorded");
    assert_eq!(
        t2.non_trivial_replicator_count(MINGEN),
        0,
        "a prolific-but-shallow copier is still TRIVIAL — must be withheld despite many births"
    );

    // Case 3: extend ONE branch into a sustained chain ⟹ the count flips >0, driven
    // purely by DEPTH (label/genome unchanged from the withheld cases). v[3] is one
    // of the founder's children; give it a child (v[53]) and grandchild (v[54]).
    let mut t3 = t2;
    t3.record_birth(v[53], Some(v[3]), g(), 100); // v[3] now has a child …
    t3.record_birth(v[54], Some(v[53]), g(), 105); // … and a grandchild ⟹ depth ≥ 2
    assert!(
        t3.non_trivial_replicator_count(MINGEN) > 0,
        "a genuinely SUSTAINED lineage (depth ≥ {MINGEN}) must count — the crossing is depth-driven"
    );
}

// ---- (B) structural crossing in a real run ----------------------------------
struct Handle {
    alife: Arc<Mutex<AlifeObserver>>,
}
impl Observer for Handle {
    fn on_step_end(&mut self, snap: &GraphSnapshot<'_>, s: &StepStats, events: &[RewriteEvent]) {
        self.alife.lock().unwrap().on_step_end(snap, s, events);
    }
}

fn exp5_config(steps: u64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "crit6"
run_id = "crit6"
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
influx_rate = 0.34
bootstrap_replication_threshold = 3.0
"#
    );
    SimConfig::from_toml(&toml).unwrap()
}

#[test]
fn criterion6_crossing_coincides_with_depth_k_lineage_event() {
    let steps = 6_000u64;
    let config = exp5_config(steps);
    let rules = hg_sim_core::runner::load_rules(&config).unwrap();
    let mut store = hg_sim_core::runner::build_initial_store(&config);
    let mut engine = Engine::new(rules, StepMode::MaxParallel, config.seed, 100000);
    let alife = Arc::new(Mutex::new(AlifeObserver::new(200)));
    engine.add_observer(Box::new(Handle { alife: alife.clone() }));
    for _ in 0..steps {
        engine.step(&mut store);
    }
    drop(engine);

    let a = alife.lock().unwrap();
    let win = a.windows();

    // Find the first window where non_trivial_replicators crosses 0 → >0.
    let crossing = win.iter().position(|w| w.non_trivial_replicators > 0);
    assert!(crossing.is_some(), "the count must cross 0 → >0 in the turnover regime");
    let ci = crossing.unwrap();

    // Before the crossing: no window had a lineage as deep as MINGEN.
    for (i, w) in win.iter().enumerate().take(ci) {
        assert!(
            w.max_generation < MINGEN,
            "window {i} has non_trivial=0 but max_generation {} ≥ {MINGEN} — inconsistent",
            w.max_generation
        );
        assert_eq!(w.non_trivial_replicators, 0, "pre-crossing windows must be 0");
    }
    // At the crossing: a lineage HAS reached depth ≥ MINGEN — a genuine sustained
    // self-copying event, not a labelling coincidence.
    assert!(
        win[ci].max_generation >= MINGEN,
        "at the crossing, some lineage must be depth ≥ {MINGEN} (got max_gen {})",
        win[ci].max_generation
    );

    eprintln!(
        "\nCRITERION 6: 0→>0 crossing at window {ci} (step ~{}), max_generation {} ≥ {MINGEN}, count {}",
        win[ci].step, win[ci].max_generation, win[ci].non_trivial_replicators
    );
    eprintln!("  ⟹ the crossing is a STRUCTURAL depth-{MINGEN} sustained-self-copying event, not a WL/label artifact.");
    eprintln!("  ⟹ negative control passes: trivial (shallow) copiers are withheld even with identical labels.\n");

    // The final count matches a direct recomputation from the lineage topology alone.
    let lin = a.lineage();
    assert_eq!(
        win.last().unwrap().non_trivial_replicators,
        lin.non_trivial_replicator_count(MINGEN),
        "per-window count is consistent with the lineage-topology recomputation"
    );
}
