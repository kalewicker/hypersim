//! EXP-L10 conduit probe (follow-up, 2026-07-15): the arm table's
//! mouth_energy_max is dominated by the seed's own budget in at-mouth
//! arms (the seed IS a mouth), so it cannot answer V3's actual
//! question — does transport cross a handle? This rerun of the
//! uncapped×at-mouth×s42 arm with snapshots enabled reads the final
//! energy at every planted-shortcut endpoint directly. Any non-seed
//! mouth with energy > 0 means the handle conducted; the far mouth of
//! the SEED'S OWN shortcut is the decisive site.

use hg_core::PropertyValue;
use hg_sim_core::{run_headless, SimConfig};

#[test]
#[ignore]
fn exp_l10_conduit_probe() {
    let rules = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../rules/lk_pair.toml"
    ))
    .unwrap();
    let out = std::env::var("HYPERSIM_SWEEP_OUT")
        .unwrap_or_else(|_| std::env::temp_dir().display().to_string());
    let toml = format!(
        r#"
experiment_name = "l10_conduit_probe"
seed = 42
out_dir = "{out}"
[rules]
source = "Inline"
[initial]
condition_type = "TriLattice"
width = 48
defect_density = 0.005
[engine]
step_mode = "MaxParallel"
max_steps = 26000
metrics_window_interval = 100
thread_count = 3
[lk_rules]
enabled = true
e_total = 1500000.0
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

    // The IC build is deterministic: reconstruct the planted shortcut
    // set (and the seed mouth = min endpoint of the min planted edge,
    // mirroring the placement rule).
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
    let mut crossed = 0usize;
    for (i, &(a, b)) in mouth_pairs.iter().enumerate() {
        let (ea, eb) = (energy(a), energy(b));
        let tag = if a == seed_mouth || b == seed_mouth {
            "  <-- seed's own handle"
        } else {
            ""
        };
        if i < 12 || tag.len() > 1 {
            println!("shortcut {i}: mouth {a:?} E={ea:.1} | mouth {b:?} E={eb:.1}{tag}");
        }
        // Energy at a non-seed mouth means SOME energy arrived there
        // (by transport through the lattice or through the handle).
        if (a != seed_mouth && ea > 0.0) || (b != seed_mouth && eb > 0.0) {
            crossed += 1;
        }
    }
    println!(
        "{crossed}/{} shortcuts have a non-seed mouth with energy > 0",
        mouth_pairs.len()
    );
}
