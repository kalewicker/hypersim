//! A1 (v6 §3): transfer-on-fuse ledger variant. On fuse, w's energy
//! relocates to the lowest-generation surviving endpoint instead of
//! vanishing. Two claims:
//!   (1) audit closes on E_total + c·N_sub = E₀ EXACTLY, with the
//!       vanished ledger term staying 0 (energy conserved in-graph);
//!   (2) era length ≈ E₀/c (only the c sink bleeds per subdivide/fuse
//!       cycle), ~6× longer than vanish mode's E₀/(q+c) at reference.
//!
//! Reference cell params: transport_first, q=50, τ=15, c=10.

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

struct Out {
    audit_ok: bool,
    e_now: f64,
    n_sub: f64,
    e_fused: f64,
    quench: u64,
    churn: u32,
}

fn run(dir: &Path, ledger: &str, e_total: f64, max_steps: u64) -> Out {
    let toml = format!(
        r#"
experiment_name = "a1_transfer"
seed = 42
out_dir = "{}"
[rules]
source = "Inline"
[initial]
condition_type = "TriLattice"
width = 32
[engine]
step_mode = "MaxParallel"
max_steps = {max_steps}
metrics_window_interval = 100
max_pairings_per_step = 100000
thread_count = 1
[lk_rules]
enabled = true
e_total = {e_total}
g_max = 1.0e18
priority = "transport_first"
q = 50.0
tau = 15.0
[lk_rules.fuse]
enabled = true
ledger_mode = "{ledger}"
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
    let artifacts = run_headless(&cfg, None, None, None).expect("run");
    let leakage = std::fs::read_to_string(artifacts.run_dir.join("leakage.csv")).unwrap();
    let rows: Vec<Vec<&str>> = leakage.lines().skip(2).map(|l| l.split(',').collect()).collect();
    let audit_ok = rows.iter().all(|r| r[18].parse::<f64>().unwrap() == 0.0);
    let last = rows.last().unwrap();
    let e_now: f64 = last[14].parse().unwrap();
    let n_sub: f64 = last[15].parse().unwrap();
    let e_fused: f64 = last[16].parse().unwrap();
    // Quench step = first window where n_subdivisions stops changing.
    // Quench = first window AFTER production begins where n_sub holds
    // steady (skip any zero-subdivision startup plateau).
    let quench = rows
        .windows(2)
        .find(|w| w[0][15] == w[1][15] && w[0][15].parse::<f64>().unwrap_or(0.0) > 0.0)
        .map(|w| w[1][0].parse::<u64>().unwrap())
        .unwrap_or(max_steps);
    let churn: u32 = last[17].parse().unwrap_or(0);
    Out { audit_ok, e_now, n_sub, e_fused, quench, churn }
}

/// Transfer mode: audit exact on E_total + c·N_sub = E₀, ledger term 0.
#[test]
fn a1_transfer_conserves_energy_in_graph() {
    let dir = tempfile::tempdir().unwrap();
    let e0 = 1.0e5;
    // era = E₀/c ≈ 10000; run past it.
    let o = run(dir.path(), "transfer", e0, 14000);
    assert!(o.audit_ok, "transfer-mode audit residual must be exactly 0");
    assert_eq!(o.e_fused, 0.0, "transfer mode must NOT vanish any energy (ledger term 0)");
    // Direct in-graph invariant: E_total + c·N_sub = E₀.
    assert_eq!(o.e_now + 10.0 * o.n_sub, e0, "E_total + c·N_sub = E₀ exact in transfer mode");
    // Churn (hazard 3): pre-registered transfer-mode threshold recorded
    // in the A1 notes (leakage_experiment_findings.md) before A4 runs.
    println!("transfer churn = {} (gate=none, no countdown)", o.churn);
}

/// Era length: transfer E₀/c vs vanish E₀/(q+c) — the ~6× stretch at
/// reference params (q=50, c=10 ⇒ 60/10). Both audit-exact.
#[test]
fn a1_transfer_era_is_longer() {
    let d1 = tempfile::tempdir().unwrap();
    let d2 = tempfile::tempdir().unwrap();
    let e0 = 1.0e5;
    let v = run(d1.path(), "vanish", e0, 14000);
    let t = run(d2.path(), "transfer", e0, 14000);
    assert!(v.audit_ok && t.audit_ok, "both audits exact");
    assert!(v.e_fused > 0.0, "vanish mode must ledger energy");
    // Vanish era ≈ E₀/(q+c) = 1666; transfer era ≈ E₀/c = 10000. The
    // transfer era is substantially longer (attacks the window floor
    // from the production side). Compare the subdivision counts, which
    // are the era's currency.
    println!(
        "vanish: n_sub {} quench@{} churn {} | transfer: n_sub {} quench@{} churn {}",
        v.n_sub, v.quench, v.churn, t.n_sub, t.quench, t.churn
    );
    assert!(
        t.n_sub > 3.0 * v.n_sub,
        "transfer era must sustain many more subdivisions (n_sub {} vs {})",
        t.n_sub, v.n_sub
    );
}
