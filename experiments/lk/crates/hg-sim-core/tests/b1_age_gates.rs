//! B1 gates (v8 §3.2) for the PASSIVE clock — the age-gated fuse:
//!   BG1 guard engagement — the age guard's counters show evals > 0 AND
//!        rejects > 0 (a zero-reject sweep is uninterpretable, v4 §1.1).
//!   BG2 lifetime exactness — on the SAME scripted throat AG2 used, w
//!        fuses on exactly the step `step − birth_step` first reaches k,
//!        so the two mechanisms are compared on identical ground.
//!   BG3 dispersion — mid-era interior age distribution has p10 ≠ p90.
//!   BG4 audit exact in both ledger modes.
//!   BG5 determinism — byte-identical reruns; zero draws preserved.
//!   BG6 zero back-action — no rule ever loses a binding to the clock,
//!        because the clock has no bindings. Asserted, not assumed.
//!
//! Plus BG-lattice, which is not in the plan and should be: the countdown
//! gate protected the original lattice for free (gen-0 nodes carry no
//! countdown key, and a missing property rejects). The age guard cannot —
//! every node has a birth step and IC nodes read as maximally old — so
//! the protection now rests entirely on lk_fuse's own generation and
//! degree guards. That is worth a test rather than a comment.

use hg_core::{HyperEdge, HypergraphStore, Label, NodeData, NodeId, PropertyValue};
use hg_rules::StepMode;
use hg_sim_core::{run_headless, SimConfig};
use smallvec::smallvec;
use std::path::Path;

const ENERGY: u32 = 1;
const GENERATION: u32 = 6;

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

/// Deliberately the same shape as `a2_countdown_gates::countdown_config`
/// with one field changed, so the two gates' fixtures differ only in the
/// clock (v8 §4: one changed variable).
fn age_config(dir: &Path, k: i64, ledger: &str, max_steps: u64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "b1_age"
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
e_total = 100000.0
g_max = 1.0e18
priority = "transport_first"
q = 50.0
tau = 15.0
[lk_rules.fuse]
enabled = true
gate = "age"
lifetime_k = {k}
ledger_mode = "{ledger}"
[leakage]
enabled = true
cadence_steps = 100
[output]
snapshots_enabled = true
fixed_provenance = true
"#,
        dir.display(),
    );
    let mut cfg = SimConfig::from_toml(&toml).unwrap();
    cfg.rules.inline_toml = Some(rules_toml(50.0, 15.0));
    cfg
}

/// Read the (evals, rejects) totals for a guard family prefix on a rule.
fn guard_totals(run_dir: &Path, rule: &str, guard_id: &str) -> (u64, u64) {
    let guards = std::fs::read_to_string(run_dir.join("guards.csv")).unwrap();
    let (mut e, mut r) = (0u64, 0u64);
    for l in guards.lines().skip(2) {
        let f: Vec<&str> = l.split(',').collect();
        if f.len() > 4 && f[1] == rule && f[2] == guard_id {
            e += f[3].parse::<u64>().unwrap_or(0);
            r += f[4].parse::<u64>().unwrap_or(0);
        }
    }
    (e, r)
}

/// BG1 (engagement), BG3 (dispersion), BG4-vanish (audit), BG6 (zero
/// back-action) on one run.
#[test]
fn bg1_engaged_bg3_dispersion_bg4_audit_bg6_no_back_action() {
    let dir = tempfile::tempdir().unwrap();
    let k = 100i64;
    let cfg = age_config(dir.path(), k, "vanish", 3000);
    let artifacts = run_headless(&cfg, None, None, None).expect("run");

    // BG4: audit residual exactly 0 every window.
    let leakage = std::fs::read_to_string(artifacts.run_dir.join("leakage.csv")).unwrap();
    let lrows: Vec<Vec<&str>> = leakage.lines().skip(2).map(|l| l.split(',').collect()).collect();
    assert!(
        lrows.iter().all(|r| r[18].parse::<f64>().unwrap() == 0.0),
        "BG4: vanish-mode audit must be exact"
    );

    // BG1: the age guard is age0 on lk_fuse (it is appended after the
    // declared property guard, and lives in its own counter family).
    let (evals, rejects) = guard_totals(&artifacts.run_dir, "lk_fuse", "age0");
    assert!(evals > 0, "BG1: the age guard must be evaluated");
    assert!(
        rejects > 0,
        "BG1: the age guard must reject young nodes — a zero-reject sweep \
         means the gate never bound and the run is uninterpretable"
    );

    // BG6: no rule ever loses a binding to the clock, because the clock
    // has no bindings. Two independent readings: the derived
    // instrument_losses column, and the absence of any contention row
    // naming an instrument rule.
    let guards = std::fs::read_to_string(artifacts.run_dir.join("guards.csv")).unwrap();
    let mut saw_losses_row = false;
    for l in guards.lines().skip(2) {
        let f: Vec<&str> = l.split(',').collect();
        if f.len() < 5 {
            continue;
        }
        if f[2] == "contention:instrument_losses" {
            saw_losses_row = true;
            assert_eq!(
                f[3].parse::<u64>().unwrap(),
                0,
                "BG6: instrument_losses must be 0 under the passive clock"
            );
        }
        assert!(
            f[2] != "contention:lk_tick",
            "BG6: lk_tick must not appear as a contention winner — the \
             passive clock does not load the ruleset at all"
        );
    }
    assert!(
        saw_losses_row,
        "BG6: the instrument_losses readout must be WRITTEN, including as \
         an explicit zero — a missing row and a zero row are different \
         observations"
    );

    // BG3: interior AGE distribution is dispersed. Read a mid-era
    // snapshot and take ages of the gen ≥ 1 (filament) nodes, which are
    // the population the gate acts on.
    let snaps = hg_eval::cone::load_snapshots(&artifacts.run_dir.join("snapshots")).unwrap();
    let (snap_step, mid) = {
        let s = &snaps[snaps.len() / 3];
        (s.0, &s.1)
    };
    let mut ages: Vec<u64> = mid
        .nodes()
        .filter(|(_, d)| matches!(d.get_property(GENERATION), Some(PropertyValue::Float(g)) if g >= 1.0))
        .map(|(_, d)| snap_step.saturating_sub(d.birth_step))
        .collect();
    ages.sort_unstable();
    assert!(
        ages.len() >= 10,
        "BG3: need a populated interior to check dispersion (got {})",
        ages.len()
    );
    let p10 = ages[ages.len() / 10];
    let p90 = ages[ages.len() * 9 / 10];
    assert_ne!(
        p10, p90,
        "BG3: ages must be dispersed (p10 {p10} == p90 {p90} is degenerate)"
    );
    // Dispersion must also be INSIDE the gate's window: an all-expired
    // population would be "dispersed" and useless.
    assert!(
        p10 < k as u64,
        "BG3: the young tail must be below k, or the gate is not throttling"
    );
}

/// BG4 (transfer): audit exact on E_total + c·N_sub = E₀, ledger 0.
#[test]
fn bg4_transfer_audit_exact() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = age_config(dir.path(), 100, "transfer", 3000);
    let artifacts = run_headless(&cfg, None, None, None).expect("run");
    let leakage = std::fs::read_to_string(artifacts.run_dir.join("leakage.csv")).unwrap();
    let rows: Vec<Vec<&str>> = leakage.lines().skip(2).map(|l| l.split(',').collect()).collect();
    assert!(
        rows.iter().all(|r| r[18].parse::<f64>().unwrap() == 0.0),
        "BG4: transfer audit exact"
    );
    let last = rows.last().unwrap();
    assert_eq!(
        last[16].parse::<f64>().unwrap(),
        0.0,
        "BG4: transfer mode ledgers nothing"
    );
}

/// BG5: determinism — two independent runs are byte-identical (canonical
/// hash) ⇒ the age path draws no RNG.
#[test]
fn bg5_determinism() {
    let d1 = tempfile::tempdir().unwrap();
    let d2 = tempfile::tempdir().unwrap();
    let a = run_headless(&age_config(d1.path(), 100, "transfer", 2000), None, None, None).unwrap();
    let b = run_headless(&age_config(d2.path(), 100, "transfer", 2000), None, None, None).unwrap();
    assert_eq!(
        a.canonical_hash, b.canonical_hash,
        "BG5: age-gated runs must be byte-identical"
    );
}

/// The scripted throat AG2 used, verbatim except for the clock: S–w–N
/// with w at generation 1 and sub-θ energy, endpoints anchored so only w
/// is fusable. Returns the step on which w disappeared.
fn scripted_throat_fuse_step(cfg: &SimConfig, k: i64) -> Option<u64> {
    let mut store = HypergraphStore::new();
    let mk = |s: &mut HypergraphStore, gen: f64| -> NodeId {
        let mut d = NodeData::new(Label(0), s.step());
        d.set_property(ENERGY, PropertyValue::Float(30.0));
        d.set_property(GENERATION, PropertyValue::Float(gen));
        s.add_node(d)
    };
    let s = mk(&mut store, 0.0);
    let n = mk(&mut store, 0.0);
    let w = mk(&mut store, 1.0);
    store.add_edge(HyperEdge::new(Label(0), smallvec![s, w]));
    store.add_edge(HyperEdge::new(Label(0), smallvec![w, n]));
    let a = mk(&mut store, 0.0);
    let b = mk(&mut store, 0.0);
    store.add_edge(HyperEdge::new(Label(0), smallvec![s, a]));
    store.add_edge(HyperEdge::new(Label(0), smallvec![n, b]));

    let rules = hg_sim_core::load_rules(cfg).unwrap();
    assert!(
        rules.iter().all(|r| r.name != "lk_tick"),
        "the passive clock must not load a tick rule"
    );
    let mut engine = hg_engine::Engine::new(rules, StepMode::MaxParallel, 42, 100_000);
    for step in 1..=(k as u64 + 3) {
        engine.step(&mut store);
        if store.get_node(w).is_none() {
            // Endpoints rejoined.
            let joined = store.edges().any(|(_, e)| {
                let mut v: Vec<NodeId> = e.nodes.to_vec();
                v.sort_unstable();
                let mut want = vec![s, n];
                want.sort_unstable();
                v == want
            });
            assert!(joined, "BG2: S–N rejoined on fuse");
            return Some(step);
        }
    }
    None
}

/// BG2: lifetime exactness, on AG2's fixture. w is born at step 0, so
/// `step − birth_step` first reaches k during the (k+1)-th engine step —
/// which is exactly where AG2 found the countdown mechanism fusing (k
/// ticks, then the fuse fires the following step). The two clocks agree
/// step-for-step, which is what makes EXP-L18 a controlled comparison
/// rather than two different experiments.
#[test]
fn bg2_lifetime_exactness_matches_the_countdown_fixture() {
    for k in [3i64, 5, 10] {
        let cfg = age_config(Path::new("/tmp/bg2_unused"), k, "vanish", 1);
        assert_eq!(
            scripted_throat_fuse_step(&cfg, k),
            Some(k as u64 + 1),
            "BG2: w must fuse on exactly the step its age first reaches k \
             (k = {k}) — the same step AG2 pinned for the countdown"
        );
    }
}

/// BG-lattice (not in the plan; see the module header). Under the age
/// guard a gen-0 lattice node is maximally OLD, so it passes the age
/// guard at every k. The lattice must stay non-removable anyway, via
/// lk_fuse's own generation guard.
#[test]
fn bg_lattice_stays_non_removable_though_it_passes_the_age_guard() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = age_config(dir.path(), 100, "vanish", 1500);
    let artifacts = run_headless(&cfg, None, None, None).expect("run");

    // Every surviving node that ever had generation 0 still exists: node
    // COUNT never drops below the initial lattice size.
    let snaps = hg_eval::cone::load_snapshots(&artifacts.run_dir.join("snapshots")).unwrap();
    let lattice_n = 32 * 32;
    for (step, s) in &snaps {
        let gen0 = s
            .nodes()
            .filter(|(_, d)| !matches!(d.get_property(GENERATION), Some(PropertyValue::Float(g)) if g >= 1.0))
            .count();
        assert_eq!(
            gen0, lattice_n,
            "BG-lattice: the original lattice must be non-removable (step {step})"
        );
    }

    // And the reason is the generation guard, not the age guard: it must
    // be doing the rejecting. prop0 on lk_fuse is `generation >= 1.0`.
    let (evals, rejects) = guard_totals(&artifacts.run_dir, "lk_fuse", "prop0");
    assert!(
        evals > 0 && rejects > 0,
        "BG-lattice: the generation guard must be the one rejecting lattice \
         candidates (evals {evals}, rejects {rejects})"
    );
}

// ------------------------------------------------- v9 C2.2: the flag

/// v9 §1.3 — `age_gate_without_generation_guard`, with a fixture that
/// triggers it. The countdown gate's lattice protection was automatic (a
/// missing key rejects); the age gate's is a convention, so the
/// convention is checked.
///
/// Asserted in BOTH directions: the production ruleset must NOT trigger
/// it (lk_fuse carries `generation >= 1.0`), and a rule stripped of that
/// guard MUST trigger it. A flag that cannot fire is not a flag.
#[test]
fn c2_age_gate_without_generation_guard_flag() {
    let cfg = age_config(Path::new("/tmp/c2_unused"), 100, "vanish", 1);
    let rules = hg_sim_core::load_rules(&cfg).unwrap();
    assert!(
        rules
            .iter()
            .any(|r| !r.pattern.age_compares.is_empty()),
        "fixture precondition: the age gate must actually be loaded"
    );
    assert!(
        hg_sim_core::age_gate_flags(&rules).is_empty(),
        "the production age-gated ruleset carries its generation guard and \
         must not trigger the flag"
    );

    // Now strip the generation guard from lk_fuse and re-check.
    let stripped: Vec<hg_rules::CompiledRule> = rules
        .into_iter()
        .map(|mut r| {
            if r.name == "lk_fuse" {
                r.pattern.property_guards.retain(|g| g.key != 6);
            }
            r
        })
        .collect();
    let flags = hg_sim_core::age_gate_flags(&stripped);
    assert_eq!(
        flags.len(),
        1,
        "an age-gated rule without a generation guard must be flagged, got {flags:?}"
    );
    assert!(
        flags[0].contains("age_gate_without_generation_guard") && flags[0].contains("lk_fuse"),
        "the flag must name the offending rule: {}",
        flags[0]
    );
}
