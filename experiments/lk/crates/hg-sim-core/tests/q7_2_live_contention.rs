//! Q7-2 check 3 (v7 §3): the DIRECT contention instrument, live.
//!
//! Runs short countdown-gated cells and reports, per rule pair, how many
//! bindings were refused for node contention and which rule took the
//! node. This closes the §1.2 audit direction permanently: F-LK22 says
//! `lk_tick` (single-node LHS, priority 2) should beat both `lk_fuse`
//! (claims {x,w,y}, priority −1) AND `lk_metered_subdivide` (claims
//! {x,y}) whenever they contend for a live-countdown node — which would
//! throttle removal and production together.
//!
//! Run with --ignored --nocapture.

use hg_sim_core::{run_headless, SimConfig};
use std::path::Path;

fn rules_toml(q: f64, tau: f64) -> String {
    std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../rules/lk_pair_capped.toml"
    ))
    .unwrap()
    .replace(
        "property = 6\nop       = \"lt\"\nvalue    = 4.0",
        "property = 6\nop       = \"lt\"\nvalue    = 1.0e18",
    )
    .replace("value = 50.0\n", &format!("value = {q:.1}\n"))
    .replace("value = 60.0", &format!("value = {:.1}", q + 10.0))
    .replace("offset = 50.0", &format!("offset = {:.1}", 2.0 * tau))
    .replace("value = 25.0", &format!("value = {tau:.1}"))
}

fn run(dir: &Path, regime: &str, k: i64, steps: u64) -> Vec<(String, String, u64)> {
    let toml = format!(
        r#"
experiment_name = "q7_contention"
seed = 42
out_dir = "{}"
[rules]
source = "Inline"
[initial]
condition_type = "TriLattice"
width = 32
[engine]
step_mode = "MaxParallel"
max_steps = {steps}
metrics_window_interval = 100
max_pairings_per_step = 100000
thread_count = 1
[lk_rules]
enabled = true
e_total = 100000.0
g_max = 1.0e18
priority = "{regime}"
q = 50.0
tau = 15.0
[lk_rules.fuse]
enabled = true
gate = "countdown"
countdown_k = {k}
ledger_mode = "vanish"
[leakage]
enabled = true
cadence_steps = 100
[output]
fixed_provenance = true
"#,
        dir.display(),
    );
    let mut cfg = SimConfig::from_toml(&toml).unwrap();
    cfg.rules.inline_toml = Some(rules_toml(50.0, 15.0));
    let a = run_headless(&cfg, None, None, None).expect("run");
    let guards = std::fs::read_to_string(a.run_dir.join("guards.csv")).unwrap_or_default();
    let mut agg: std::collections::HashMap<(String, String), u64> = Default::default();
    for l in guards.lines().skip(2) {
        let f: Vec<&str> = l.split(',').collect();
        if f.len() > 4 {
            if let Some(winner) = f[2].strip_prefix("contention:") {
                *agg.entry((f[1].to_string(), winner.to_string())).or_insert(0) +=
                    f[4].parse::<u64>().unwrap_or(0);
            }
        }
    }
    let mut out: Vec<(String, String, u64)> =
        agg.into_iter().map(|((l, w), c)| (l, w, c)).collect();
    out.sort_by(|a, b| b.2.cmp(&a.2));
    out
}

#[test]
#[ignore]
fn q7_2_live_contention() {
    let tmp = tempfile::tempdir().unwrap();
    for (regime, k) in [
        ("subdivide_first", 30i64),
        ("subdivide_first", 100),
        ("transport_first", 100),
    ] {
        let label = if regime == "subdivide_first" { "pump" } else { "swap" };
        let losses = run(&tmp.path().join(format!("{label}_k{k}")), regime, k, 3000);
        println!("--- {label} k={k} (loser LOST TO winner : refusals) ---");
        if losses.is_empty() {
            println!("  (no contention recorded)");
        }
        for (loser, winner, count) in &losses {
            println!("  {loser} LOST TO {winner} : {count}");
        }
    }
}
