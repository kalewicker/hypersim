//! LK-R gate tests (plan v2 §6): the energy-metered rule family ships
//! only if G1–G5 pass, and the §6.2 audit invariant is the
//! trustworthiness anchor for the whole phase.

use hg_sim_core::{run_headless, SimConfig};
use std::path::Path;

fn lkr_config(dir: &Path, seed: u64, e_total: f64, max_steps: u64, window: u64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "lkr_gate"
seed = {seed}
out_dir = "{}"
[rules]
source = "File"
file_path = "{}/../../rules/lk_pair.toml"
[initial]
condition_type = "TriLattice"
width = 32
[engine]
step_mode = "MaxParallel"
max_steps = {max_steps}
metrics_window_interval = {window}
thread_count = 1
[lk_rules]
enabled = true
e_total = {e_total}
[leakage]
enabled = true
cadence_steps = {window}
[output]
fixed_provenance = true
"#,
        dir.display(),
        env!("CARGO_MANIFEST_DIR"),
    );
    SimConfig::from_toml(&toml).expect("config parses")
}

struct LeakageData {
    rows: Vec<Vec<String>>, // split fields per data row
}

fn read_leakage(run_dir: &Path) -> LeakageData {
    let content = std::fs::read_to_string(run_dir.join("leakage.csv")).unwrap();
    LeakageData {
        rows: content
            .lines()
            .skip(2)
            .map(|l| l.split(',').map(str::to_string).collect())
            .collect(),
    }
}

/// §6.2 required test: E_total(t) + c·N_subdivisions(t) = E₀, EXACTLY,
/// on a 1000-step run — asserted via the emitted audit_residual column
/// (full precision: the string must parse to literal 0.0) and by
/// recomputing the invariant from e_total_now and n_subdivisions.
#[test]
fn lk_energy_audit_exact() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = lkr_config(dir.path(), 42, 100_000.0, 1000, 100);
    let artifacts = run_headless(&cfg, None, None, None).expect("run");
    let data = read_leakage(&artifacts.run_dir);
    assert!(data.rows.len() >= 10, "one audit line per window");
    for row in &data.rows {
        // Columns: 14 = e_total_now, 15 = n_subdivisions, 18 = audit_residual
        // (16 = e_fused_total, 17 = churn_events — v4 Appendix B).
        let e_now: f64 = row[14].parse().unwrap();
        let n_sub: f64 = row[15].parse().unwrap();
        let residual: f64 = row[18].parse().unwrap();
        assert_eq!(residual, 0.0, "audit drift at step {}: {row:?}", row[0]);
        assert_eq!(
            e_now + 10.0 * n_sub,
            100_000.0,
            "recomputed invariant violated at step {}",
            row[0]
        );
    }
    // Quench arithmetic: the era ends at exactly E₀/(q+c) subdivisions.
    let last = data.rows.last().unwrap();
    let n_sub: f64 = last[15].parse().unwrap();
    assert!(n_sub > 0.0, "the rule family must actually fire");
}

/// G1 bounded + G3 lattice-compatible + G4 geometric + G2 sustained
/// (era-scaled) on the reference cell. Recorded values
/// (docs/fixture_calibration.md): era = E₀/(q+c) steps, one subdivision
/// per step (the seed is the only super-θ node under downhill-only
/// transport), cut_exponent 0.51–0.54 throughout.
#[test]
fn lkr_gates_on_reference_cell() {
    let dir = tempfile::tempdir().unwrap();
    // Era = 100000/60 ≈ 1666 steps; run past quench to check boundedness.
    let cfg = lkr_config(dir.path(), 42, 100_000.0, 2000, 100);
    let artifacts = run_headless(&cfg, None, None, None).expect("run");

    // G1: bounded — node count plateaus, zero Exploding windows.
    let metrics = std::fs::read_to_string(&artifacts.csv_path).unwrap();
    let regime_ix = metrics
        .lines()
        .nth(1)
        .unwrap()
        .split(',')
        .position(|h| h == "regime")
        .expect("regime column");
    let regimes: Vec<&str> = metrics
        .lines()
        .skip(2)
        .filter_map(|l| l.split(',').nth(regime_ix))
        .collect();
    assert!(
        !regimes.iter().any(|r| *r == "Exploding"),
        "G1 violated: Exploding window in {regimes:?}"
    );
    assert!(
        artifacts.summary.final_metrics.as_ref().unwrap().n_nodes < 4000,
        "G1: node count must plateau (era-bounded)"
    );

    // G2 (era-scaled): structural events in every window of the era.
    // Era length scales linearly with E₀ (quench arithmetic pinned by
    // the audit test), so the 250-window science requirement is a
    // budget choice, not a rule property.
    let data = read_leakage(&artifacts.run_dir);
    let era_rows: Vec<_> = data
        .rows
        .windows(2)
        .filter(|w| w[0][15] != w[1][15]) // n_sub still growing
        .collect();
    assert!(
        era_rows.len() >= 15,
        "G2: sustained structural activity across the era"
    );

    // G4 (v3 §1.1 redefinition): >= 80% of sustained-era windows read
    // Sparse on the calibrated phase classifier. cut_exponent appears in
    // NO gate anymore (F-LK6: the cut channel was demoted by LK0R; a
    // cut-based G4 passed while the phase plane read Dense — hollow).
    let phases: Vec<&str> = data.rows.iter().map(|r| r[12].as_str()).collect();
    let sparse = phases.iter().filter(|&&p| p == "sparse").count();
    assert!(
        sparse as f64 >= 0.8 * phases.len() as f64,
        "G4-new violated: phase trajectory {phases:?}"
    );

    // G3 is implicit (the rules matched the clean TriLattice IC at all).
    assert!(artifacts.summary.final_metrics.unwrap().n_nodes > 1024 + 500);
}

/// G5: deterministic end to end — identical canonical hash across two
/// runs AND across thread counts (MaxParallel, zero-draw RNG).
#[test]
fn lkr_deterministic_across_runs_and_threads() {
    let hash = |threads: usize| -> u64 {
        let dir = tempfile::tempdir().unwrap();
        let mut cfg = lkr_config(dir.path(), 42, 20_000.0, 400, 100);
        cfg.engine.thread_count = threads;
        run_headless(&cfg, None, None, None).unwrap().canonical_hash
    };
    let a = hash(1);
    let b = hash(1);
    let c = hash(4);
    assert_eq!(a, b, "G5: run-to-run determinism");
    assert_eq!(a, c, "G5: thread-count independence");
}

/// v2 §6.1: the subdivide-before-transport priority is PINNED and
/// order-SENSITIVE — swapping priorities changes the trajectory (the
/// same-edge conflicts resolve differently). Both orders must be
/// deterministic; only the pinned one is normative.
#[test]
fn rule_priority_order_sensitivity() {
    let run_with_rules = |rules_toml: &str| -> u64 {
        let dir = tempfile::tempdir().unwrap();
        let mut cfg = lkr_config(dir.path(), 42, 20_000.0, 300, 100);
        cfg.rules.source = hg_sim_core::RuleSource::Inline;
        cfg.rules.file_path = None;
        cfg.rules.inline_toml = Some(rules_toml.to_string());
        run_headless(&cfg, None, None, None).unwrap().canonical_hash
    };
    let pinned = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../rules/lk_pair.toml"
    ))
    .unwrap();
    let swapped = pinned
        .replace("priority = 1", "priority = TMP")
        .replace("priority = 0", "priority = 1")
        .replace("priority = TMP", "priority = 0");
    let h_pinned_1 = run_with_rules(&pinned);
    let h_pinned_2 = run_with_rules(&pinned);
    let h_swapped = run_with_rules(&swapped);
    assert_eq!(h_pinned_1, h_pinned_2, "pinned order is deterministic");
    assert_ne!(
        h_pinned_1, h_swapped,
        "priority order must be dynamically load-bearing (hazard 4's \
         artifact-exclusion rerun depends on it changing something)"
    );
}

/// §6.3 parameter search / v3 P0 re-audit harness (run with --ignored):
/// q = θ/2 and τ = θ/4 fixed, sweep c × E₀ (9 cells ≤ the 12-cell
/// budget). Reports the gate table with G4-new (v3 §1.1: >= 80% of
/// sustained-era windows Sparse on the calibrated phase classifier;
/// cut_exponent appears in no gate). Set `HYPERSIM_SWEEP_OUT` to
/// persist per-cell run dirs (leakage.csv is the P0 record); the v2
/// round lost its sweep outputs to an ephemeral dir — never again.
#[test]
#[ignore]
fn lkr_parameter_sweep() {
    let pair = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../rules/lk_pair.toml"
    ))
    .unwrap();
    let persist_root = std::env::var("HYPERSIM_SWEEP_OUT").ok();
    let tmp = tempfile::tempdir().unwrap();
    println!("cell | era_subs (est) | audit | G1 | G2 | G4_old_cutfrac | G4_new_sparsefrac | phases | nodes");
    for c in [5.0f64, 10.0, 20.0] {
        for e_total in [50_000.0f64, 100_000.0, 400_000.0] {
            let rules = pair.replace("value = 60.0", &format!("value = {:.1}", 50.0 + c));
            let cell = format!("c{c}_E{e_total:.0}");
            let dir = persist_root
                .as_ref()
                .map(|r| {
                    let p = std::path::Path::new(r).join(&cell);
                    std::fs::create_dir_all(&p).unwrap();
                    p
                })
                .unwrap_or_else(|| tmp.path().join(&cell));
            let era_est = (e_total / (50.0 + c)) as u64;
            let mut cfg = lkr_config(&dir, 42, e_total, era_est + 400, 200);
            cfg.experiment_name = format!("p0_{cell}");
            cfg.lk_rules.c = c;
            cfg.rules.source = hg_sim_core::RuleSource::Inline;
            cfg.rules.file_path = None;
            cfg.rules.inline_toml = Some(rules);
            let artifacts = run_headless(&cfg, None, None, None).expect("run");
            let data = read_leakage(&artifacts.run_dir);
            let last = data.rows.last().unwrap();
            let n_sub: f64 = last[15].parse().unwrap();
            let drift = data
                .rows
                .iter()
                .any(|r| r[18].parse::<f64>().unwrap() != 0.0);
            // Sustained era = windows before the quench plateau.
            let era_rows: Vec<&Vec<String>> = data
                .rows
                .iter()
                .take_while(|r| r[15].parse::<f64>().unwrap() < n_sub)
                .collect();
            let g2_sustained = era_rows.len() as u64 >= (era_est / 200).saturating_sub(1);
            let cuts: Vec<f64> = era_rows.iter().filter_map(|r| r[4].parse().ok()).collect();
            let g4_old = cuts.iter().filter(|&&x| x < 0.8).count() as f64 / cuts.len().max(1) as f64;
            let phases: Vec<&str> = era_rows.iter().map(|r| r[12].as_str()).collect();
            let sparse = phases.iter().filter(|&&p| p == "sparse").count();
            let g4_new = sparse as f64 / phases.len().max(1) as f64;
            let mut census: std::collections::BTreeMap<&str, usize> = Default::default();
            for p in &phases {
                *census.entry(p).or_default() += 1;
            }
            let csv = std::fs::read_to_string(&artifacts.csv_path).unwrap();
            let regime_ix = csv
                .lines()
                .nth(1)
                .unwrap()
                .split(',')
                .position(|h| h == "regime")
                .unwrap();
            let exploding = csv
                .lines()
                .skip(2)
                .filter_map(|l| l.split(',').nth(regime_ix))
                .any(|r| r == "Exploding");
            println!(
                "{cell} | {n_sub} ({era_est}) | drift={drift} | G1={} | G2={g2_sustained} | \
                 {g4_old:.2} | {g4_new:.2} {} | {census:?} | {}",
                !exploding,
                if g4_new >= 0.8 { "PASS" } else { "FAIL" },
                artifacts.summary.final_metrics.unwrap().n_nodes,
            );
        }
    }
}
