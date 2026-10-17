//! Q1 V3 rider (v4 §3): with EXP-L12 landing on the distributed-growth
//! row, V3 (conduit amplification) gets its first fair test — an
//! at-mouth encounter arm under `transport_first`, where a super-θ node
//! CAN transport. The priority-swap exclusion does not apply: the swap
//! IS the configuration.
//!
//! Readouts: far-mouth energy of the seed's own handle (did the handle
//! conduct?), t_far_nucleus / t_ambient_front_at_far from tracked.csv.

use hg_core::PropertyValue;
use hg_sim_core::{run_headless, SimConfig};

#[test]
#[ignore]
fn exp_l12_v3_rider_at_mouth_under_swap() {
    let rules = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../rules/lk_pair.toml"
    ))
    .unwrap();
    let out = std::env::var("HYPERSIM_SWEEP_OUT")
        .unwrap_or_else(|_| std::env::temp_dir().display().to_string());
    let toml = format!(
        r#"
experiment_name = "l12_v3_rider"
seed = 42
out_dir = "{out}"
[rules]
source = "Inline"
[initial]
condition_type = "TriLattice"
width = 32
defect_density = 0.005
[engine]
step_mode = "MaxParallel"
max_steps = 20000
metrics_window_interval = 500
thread_count = 1
[lk_rules]
enabled = true
e_total = 400000.0
priority = "transport_first"
seed_placement = "at_tracked_mouth"
[leakage]
enabled = true
cadence_steps = 100
[leakage.tracked]
enabled = true
recompute_cadence = 100
[output]
snapshots_enabled = true
fixed_provenance = true
"#
    );
    let mut cfg = SimConfig::from_toml(&toml).expect("config parses");
    cfg.rules.inline_toml = Some(rules);

    let (ic_store, planted) = hg_sim_core::runner::build_initial_store_with_tracked(&cfg);
    let mut planted_sorted = planted.clone();
    planted_sorted.sort_unstable();
    let mouth_pairs: Vec<(hg_core::NodeId, hg_core::NodeId)> = planted_sorted
        .iter()
        .filter_map(|e| {
            let edge = ic_store.get_edge(*e)?;
            let mut ns: Vec<hg_core::NodeId> = edge.nodes.to_vec();
            ns.sort_unstable();
            Some((ns[0], ns[1]))
        })
        .collect();
    let seed_mouth = mouth_pairs.first().map(|p| p.0).expect("planted shortcuts exist");

    let artifacts = run_headless(&cfg, None, None, None).expect("run");
    let snaps = hg_eval::cone::load_snapshots(&artifacts.run_dir.join("snapshots")).unwrap();
    let (step, store) = snaps.last().expect("snapshots written");
    let energy = |id: hg_core::NodeId| -> f64 {
        match store.get_node(id).and_then(|d| d.get_property(1)) {
            Some(PropertyValue::Float(v)) => v,
            _ => f64::NAN,
        }
    };
    println!("final snapshot step {step}; seed mouth = {seed_mouth:?}");
    let mut conducted = 0usize;
    for (i, &(a, b)) in mouth_pairs.iter().enumerate() {
        let (ea, eb) = (energy(a), energy(b));
        if a == seed_mouth || b == seed_mouth {
            println!("seed's own handle (shortcut {i}): near {ea:.1} | far {eb:.1}");
        }
        if (a != seed_mouth && ea > 0.0) || (b != seed_mouth && eb > 0.0) {
            conducted += 1;
        }
    }
    println!("{conducted}/{} shortcuts have an energized non-seed mouth", mouth_pairs.len());
    let tracked = std::fs::read_to_string(artifacts.run_dir.join("tracked.csv")).unwrap();
    let last: Vec<&str> = tracked.lines().last().unwrap().split(',').collect();
    println!(
        "mouth_energy_max = {} | t_far_nucleus = {:?} | t_ambient_front_at_far = {:?}",
        last[9],
        last[10].parse::<u64>().ok(),
        last[11].parse::<u64>().ok(),
    );
}
