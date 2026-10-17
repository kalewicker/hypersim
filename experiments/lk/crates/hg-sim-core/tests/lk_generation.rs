//! Generation-capped subdivision (plan v3 §1.4 / §3): the four required
//! EXP-L9 grammar tests. Generation is node-property key 6 (Float;
//! docs/property_registry.md); the normative rule file is
//! rules/lk_pair_capped.toml (guard default g_max = 4.0).

use hg_core::{HyperEdge, HypergraphStore, Label, NodeData, NodeId, PropertyValue};
use hg_engine::Engine;
use hg_rules::{compile_rules, parse_rules_toml, StepMode};
use hg_sim_core::{run_headless, SimConfig};
use smallvec::smallvec;

const ENERGY: u32 = 1;
const GENERATION: u32 = 6;

fn capped_rules_toml() -> String {
    std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../rules/lk_pair_capped.toml"
    ))
    .unwrap()
}

fn float_prop(store: &HypergraphStore, id: NodeId, key: u32) -> Option<f64> {
    match store.get_node(id)?.get_property(key) {
        Some(PropertyValue::Float(v)) => Some(v),
        _ => None,
    }
}

/// v3 §1.4, pinned semantics AND pinned order-sensitivity:
/// `scale_from ×1.0` creates the missing key on the fresh node, `add`
/// then increments — child generation = parent + 1. With the two
/// effects swapped, `add` skips on the missing key and `scale_from`
/// then copies the parent unchanged — child generation = parent. The
/// declaration order is load-bearing, same discipline as the energy
/// effects.
#[test]
fn generation_inheritance_pinned() {
    let apply_one = |rules_toml: &str, parent_gen: f64| -> f64 {
        let mut store = HypergraphStore::new();
        let mut x = NodeData::new(Label(0), 0);
        x.set_property(ENERGY, PropertyValue::Float(200.0));
        x.set_property(GENERATION, PropertyValue::Float(parent_gen));
        let x = store.add_node(x);
        let mut y = NodeData::new(Label(0), 0);
        y.set_property(ENERGY, PropertyValue::Float(0.0));
        y.set_property(GENERATION, PropertyValue::Float(0.0));
        let y = store.add_node(y);
        store.add_edge(HyperEdge::new(Label(0), smallvec![x, y]));
        let rules = compile_rules(&parse_rules_toml(rules_toml).unwrap()).unwrap();
        let mut engine = Engine::new(rules, StepMode::MaxParallel, 42, 100000);
        let before: Vec<NodeId> = store.nodes().map(|(id, _)| id).collect();
        engine.step(&mut store);
        let fresh: Vec<NodeId> = store
            .nodes()
            .map(|(id, _)| id)
            .filter(|id| !before.contains(id))
            .collect();
        assert_eq!(fresh.len(), 1, "exactly one subdivision fired");
        float_prop(&store, fresh[0], GENERATION).expect("fresh node carries generation")
    };

    let pinned = capped_rules_toml();
    // Parent gen 3 < guard 4: fires; child must read 4 (scale_from
    // creates the key with 3.0, add increments to 4.0).
    assert_eq!(apply_one(&pinned, 3.0), 4.0, "pinned order: parent + 1");

    // Swapped order: add-on-missing-key skips, scale_from then writes
    // the parent value — child reads 3, NOT 4. Order sensitivity pinned.
    let scale_block = "\
[[rules.rhs.property_effects]]
var        = \"w\"
key        = 6
op         = \"scale_from\"
source_var = \"x\"
source_key = 6
factor     = 1.0
";
    let add_block = "\
[[rules.rhs.property_effects]]
var   = \"w\"
key   = 6
op    = \"add\"
value = 1.0
";
    let joined = format!("{scale_block}\n{add_block}");
    assert!(pinned.contains(&joined), "normative file declares scale_from before add");
    let swapped = pinned.replace(&joined, &format!("{add_block}\n{scale_block}"));
    assert_eq!(
        apply_one(&swapped, 3.0),
        3.0,
        "swapped order silently loses the increment — the reason the order is pinned"
    );
}

/// v3 hazard 1 (load-bearing): every ORIGINAL lattice node carries
/// generation = 0.0 from the IC initializer when [lk_rules] g_max is
/// set. A missing key would make the capped guard silently not-match
/// and fake a collapse result.
#[test]
fn ic_initializer_sets_generation_zero() {
    let dir = tempfile::tempdir().unwrap();
    let toml = format!(
        r#"
experiment_name = "gen_ic"
seed = 42
out_dir = "{}"
[rules]
source = "File"
file_path = "{}/../../rules/lk_pair_capped.toml"
[initial]
condition_type = "TriLattice"
width = 16
[engine]
step_mode = "MaxParallel"
max_steps = 100
metrics_window_interval = 50
thread_count = 1
[lk_rules]
enabled = true
e_total = 6000.0
g_max = 4.0
[output]
snapshots_enabled = true
fixed_provenance = true
"#,
        dir.path().display(),
        env!("CARGO_MANIFEST_DIR"),
    );
    let cfg = SimConfig::from_toml(&toml).unwrap();
    let artifacts = run_headless(&cfg, None, None, None).expect("run");
    let snaps = hg_eval::cone::load_snapshots(&artifacts.run_dir.join("snapshots")).unwrap();
    let (_, store) = snaps.last().expect("at least one snapshot");
    let mut gen_zero = 0usize;
    let mut gen_pos = 0usize;
    for (id, data) in store.nodes() {
        match data.get_property(GENERATION) {
            Some(PropertyValue::Float(0.0)) => gen_zero += 1,
            Some(PropertyValue::Float(g)) if g > 0.0 => gen_pos += 1,
            other => panic!("node {id:?} has no valid generation: {other:?}"),
        }
    }
    // TriLattice(16) = 256 original nodes, all generation 0; every
    // fresh node carries generation > 0 via the inheritance effects.
    assert_eq!(gen_zero, 256, "every IC node at generation 0, none missing");
    assert!(gen_pos > 0, "subdivision produced generation > 0 children");
    assert_eq!(store.node_count(), 256 + gen_pos);
}

/// v3 §3 required test, scripted linear chain fixture: a reservoir R
/// that is barred from subdividing (its generation is scripted AT the
/// cap) feeds the chain head to θ, so subdivision cascades through the
/// generations — and must stop at EXACTLY g_max: a generation-g_max
/// node sits above θ and stays unsubdivided (the guard, not energy,
/// does the stopping), and the same fixture with the cap moved up
/// cascades past it.
#[test]
fn chain_stops_at_exactly_g_max() {
    let run_fixture = |g_max: f64, steps: u64| -> (f64, bool) {
        let rules_toml = capped_rules_toml().replace(
            "property = 6\nop       = \"lt\"\nvalue    = 4.0",
            &format!("property = 6\nop       = \"lt\"\nvalue    = {g_max:.1}"),
        );
        let mut store = HypergraphStore::new();
        let mut r = NodeData::new(Label(0), 0);
        r.set_property(ENERGY, PropertyValue::Float(100_000.0));
        // Scripted: the reservoir sits at the cap so it can never
        // subdivide — it only transports, re-concentrating the chain.
        r.set_property(GENERATION, PropertyValue::Float(g_max));
        let r = store.add_node(r);
        let mut c0 = NodeData::new(Label(0), 0);
        c0.set_property(ENERGY, PropertyValue::Float(0.0));
        c0.set_property(GENERATION, PropertyValue::Float(0.0));
        let c0 = store.add_node(c0);
        store.add_edge(HyperEdge::new(Label(0), smallvec![r, c0]));
        let rules = compile_rules(&parse_rules_toml(&rules_toml).unwrap()).unwrap();
        let mut engine = Engine::new(rules, StepMode::MaxParallel, 42, 100000);
        for _ in 0..steps {
            engine.step(&mut store);
        }
        let max_gen = store
            .nodes()
            .filter_map(|(id, _)| float_prop(&store, id, GENERATION))
            .fold(0.0f64, f64::max);
        // The stopping proof: some node AT the cap holds >= θ (it would
        // subdivide if the guard let it).
        let capped_super_theta = store.nodes().any(|(id, _)| {
            float_prop(&store, id, GENERATION) == Some(g_max)
                && float_prop(&store, id, ENERGY).is_some_and(|e| e >= 100.0)
        });
        (max_gen, capped_super_theta)
    };

    let (max_gen, capped_super_theta) = run_fixture(2.0, 400);
    assert_eq!(max_gen, 2.0, "cascade stops at exactly g_max = 2");
    assert!(
        capped_super_theta,
        "a generation-2 node holds >= θ and is stopped by the guard, not by energy"
    );
    // Control: the identical fixture with the cap at 6 cascades past 2.
    let (max_gen_hi, _) = run_fixture(6.0, 400);
    assert!(
        max_gen_hi > 2.0,
        "with the cap lifted the same fixture cascades past generation 2 (got {max_gen_hi})"
    );
}

/// v3 §3: the §6.2 audit invariant (E_total + c·N_sub = E₀, exact) is
/// untouched by the generation guard and inheritance effects.
#[test]
fn audit_invariant_holds_with_generation_guard() {
    let dir = tempfile::tempdir().unwrap();
    let toml = format!(
        r#"
experiment_name = "gen_audit"
seed = 42
out_dir = "{}"
[rules]
source = "File"
file_path = "{}/../../rules/lk_pair_capped.toml"
[initial]
condition_type = "TriLattice"
width = 32
[engine]
step_mode = "MaxParallel"
max_steps = 1000
metrics_window_interval = 100
thread_count = 1
[lk_rules]
enabled = true
e_total = 100000.0
g_max = 4.0
[leakage]
enabled = true
cadence_steps = 100
[output]
fixed_provenance = true
"#,
        dir.path().display(),
        env!("CARGO_MANIFEST_DIR"),
    );
    let cfg = SimConfig::from_toml(&toml).unwrap();
    let artifacts = run_headless(&cfg, None, None, None).expect("run");
    let content = std::fs::read_to_string(artifacts.run_dir.join("leakage.csv")).unwrap();
    let rows: Vec<Vec<&str>> = content
        .lines()
        .skip(2)
        .map(|l| l.split(',').collect())
        .collect();
    assert!(rows.len() >= 10, "one audit line per window");
    for row in &rows {
        let e_now: f64 = row[14].parse().unwrap();
        let n_sub: f64 = row[15].parse().unwrap();
        let residual: f64 = row[18].parse().unwrap();
        assert_eq!(residual, 0.0, "audit drift at step {}", row[0]);
        assert_eq!(e_now + 10.0 * n_sub, 100000.0, "recomputed invariant at step {}", row[0]);
    }
    let n_sub: f64 = rows.last().unwrap()[15].parse().unwrap();
    assert!(n_sub > 0.0, "the capped family must actually fire");
}
