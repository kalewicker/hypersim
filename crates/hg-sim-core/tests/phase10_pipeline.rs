//! Phase 10 exit criteria: a live headless run produces one CSV, one
//! report, a manifest with correct final state — with ZERO event-channel
//! listeners. Plus Integration Test E: cross-thread-count determinism
//! (docs/reproducibility_policy.md).

use hg_output::{RunManifest, RunState};
use hg_sim_core::{run_headless, SimConfig};

fn test_config(dir: &std::path::Path, threads: usize, seed: u64) -> SimConfig {
    let mut c = SimConfig::from_toml(&format!(
        r#"
experiment_name = "itest"
seed = {seed}
out_dir = "{}"

[rules]
source = "Builtin"
builtin_name = "binary_edge"

[initial]
condition_type = "WhiteNoise"
node_count = 24
edge_count = 30
max_arity = 2
label_range = 3

[engine]
max_steps = 96
metrics_window_interval = 16
max_pairings_per_step = 8
thread_count = {threads}

[output]
fixed_provenance = true
"#,
        dir.display()
    ))
    .expect("config parses");
    c.run_id = format!("run_t{threads}_s{seed}");
    c
}

#[test]
fn headless_run_produces_csv_report_manifest_with_zero_listeners() {
    let dir = tempfile::tempdir().unwrap();
    let config = test_config(dir.path(), 2, 42);
    // No shared state, no event channels: everything must still land on disk.
    let artifacts = run_headless(&config, None, None, None).unwrap();

    // Exactly one METRICS CSV (metrics.csv). The LK leakage track adds a
    // separate leakage.csv when [leakage] is enabled (default) — a
    // distinct track artifact, not a second metrics CSV.
    let csvs: Vec<_> = std::fs::read_dir(&artifacts.run_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "csv").unwrap_or(false))
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert!(csvs.contains(&"metrics.csv".to_string()), "metrics.csv present");
    assert_eq!(
        csvs.iter().filter(|n| *n == "metrics.csv").count(),
        1,
        "exactly one metrics CSV per run"
    );
    assert!(
        csvs.iter().all(|n| n == "metrics.csv" || n == "leakage.csv"),
        "only metrics.csv and (optionally) leakage.csv, got {csvs:?}"
    );

    // CSV has the schema comment, a header, and one row per window.
    let csv = std::fs::read_to_string(&artifacts.csv_path).unwrap();
    let mut lines = csv.lines();
    assert!(lines.next().unwrap().starts_with("#hypersim_csv_v=1"));
    let header = lines.next().unwrap();
    assert!(header.starts_with("window_id,step_start,step_end,n_nodes"));
    assert_eq!(lines.count(), 6, "96 steps / 16-step windows = 6 rows");

    // Report exists with all 11 sections.
    let report = std::fs::read_to_string(&artifacts.report_path).unwrap();
    for section in [
        "## 1. Manifest", "## 2. Run Parameters", "## 3. Campaign Summary",
        "## 4. Gate Status", "## 5. Regime Summary", "## 6. Evaluator Details",
        "## 7. Causal Language Summary", "## 8. Final Metrics Table",
        "## 9. Suspicious Metric Flags", "## 10. Observations",
        "## 11. Recommended Next Actions",
    ] {
        assert!(report.contains(section), "report missing {section}");
    }

    // Manifest sealed Completed with the right final step.
    let manifest = RunManifest::load(&artifacts.run_dir).unwrap();
    assert_eq!(manifest.state(), RunState::Completed);
    assert_eq!(manifest.final_step, 96);
    assert!(manifest.end_time.is_some());
}

/// Integration Test E: same config + seed at 1, 2, 8 threads => identical
/// final canonical hash AND byte-identical CSV data (fixed provenance
/// makes whole-file comparison valid for test tooling).
#[test]
fn cross_thread_count_determinism() {
    let dir = tempfile::tempdir().unwrap();

    let mut outputs = Vec::new();
    for threads in [1usize, 2, 8] {
        let config = test_config(dir.path(), threads, 7);
        let artifacts = run_headless(&config, None, None, None).unwrap();
        let csv = std::fs::read_to_string(&artifacts.csv_path).unwrap();
        // Skip the schema comment line (run_id differs per run dir here).
        let data: String = csv.lines().skip(1).collect::<Vec<_>>().join("\n");
        outputs.push((threads, artifacts.canonical_hash, data));
    }

    let (_, h1, d1) = &outputs[0];
    for (threads, h, d) in &outputs[1..] {
        assert_eq!(h, h1, "canonical hash differs at {threads} threads");
        assert_eq!(d, d1, "CSV data rows differ at {threads} threads");
    }
}

/// Same seed twice => identical outputs; different seed => different hash.
#[test]
fn same_seed_reproduces_different_seed_diverges() {
    let dir = tempfile::tempdir().unwrap();

    let mut a = test_config(dir.path(), 4, 11);
    a.run_id = "seed11_a".into();
    let mut b = test_config(dir.path(), 4, 11);
    b.run_id = "seed11_b".into();
    let mut c = test_config(dir.path(), 4, 12);
    c.run_id = "seed12".into();

    let ra = run_headless(&a, None, None, None).unwrap();
    let rb = run_headless(&b, None, None, None).unwrap();
    let rc = run_headless(&c, None, None, None).unwrap();

    assert_eq!(ra.canonical_hash, rb.canonical_hash);
    assert_ne!(ra.canonical_hash, rc.canonical_hash);
}

#[test]
fn optional_binary_outputs_are_produced_and_readable() {
    let dir = tempfile::tempdir().unwrap();
    let mut config = test_config(dir.path(), 2, 3);
    config.run_id = "binary_outputs".into();
    config.output.snapshots_enabled = true;
    config.output.append_log_enabled = true;

    let artifacts = run_headless(&config, None, None, None).unwrap();

    // 6 windows -> 6 snapshots, each readable with the right step marker.
    let snap_dir = artifacts.run_dir.join("snapshots");
    let mut snaps: Vec<_> = std::fs::read_dir(&snap_dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    snaps.sort();
    assert_eq!(snaps.len(), 6);
    let (step, payload) = hg_output::SnapshotReader::read_snapshot(&snaps[0]).unwrap();
    assert_eq!(step, 16);
    // Real round-trip now that the wire type makes snapshots readable:
    // deserialize the store and confirm it reconstructs (was: only a
    // non-empty payload check).
    let restored: hg_core::HypergraphStore =
        bincode::deserialize(&payload).expect("snapshot store deserializes");
    assert!(restored.node_count() > 0, "restored store has structure");
    assert_eq!(restored.step(), 16, "step round-trips inside the store too");

    // Append log: 6 MetricsRow records that decode.
    let mut reader = hg_output::AppendLogReader::open(&artifacts.run_dir.join("metrics.hgal")).unwrap();
    let mut rows = 0;
    while let Some(record) = reader.next_record().unwrap() {
        let row: hg_schema::MetricsRow = bincode::deserialize(&record).unwrap();
        assert_eq!(row.window_id, rows);
        rows += 1;
    }
    assert_eq!(rows, 6);
}

/// Regression (review finding): a trailing partial window must be emitted,
/// not silently dropped. 100 steps at window 16 = 6 full windows + one
/// 4-step partial; its per-step ratio divides by 4, not 16.
#[test]
fn partial_final_window_is_emitted() {
    let dir = tempfile::tempdir().unwrap();
    let mut config = test_config(dir.path(), 2, 21);
    config.run_id = "partial_window".into();
    config.engine.max_steps = 100;

    let artifacts = run_headless(&config, None, None, None).unwrap();
    let csv = std::fs::read_to_string(&artifacts.csv_path).unwrap();
    let rows: Vec<&str> = csv.lines().skip(2).collect();
    assert_eq!(rows.len(), 7, "6 full windows + 1 partial");

    let last: Vec<&str> = rows[6].split(',').collect();
    // window_id, step_start, step_end
    assert_eq!(last[0], "6");
    assert_eq!(last[1], "96");
    assert_eq!(last[2], "99");

    // accepted_per_step for the partial window divides by 4: with the cap
    // of 8 accepted per step it must be <= 8 (a /16 divisor would make the
    // math visibly wrong: 32 accepted / 16 = 2 even when all 4 steps
    // saturated). Cross-check: accepted_rewrites / 4 == accepted_per_step.
    let accepted: f64 = last[5].parse().unwrap();
    let per_step: f64 = last[7].parse().unwrap();
    assert!((per_step - accepted / 4.0).abs() < 1e-9);
}

/// Regression (review finding): stopping or dropping a live controller
/// mid-run must seal the manifest as Interrupted — never leave it Running.
#[test]
fn stop_and_drop_seal_manifest_interrupted() {
    use hg_app_api::{SimCommand, SimController};

    let dir = tempfile::tempdir().unwrap();
    let mut config = test_config(dir.path(), 2, 31);
    config.run_id = "stopped_run".into();
    config.engine.max_steps = u64::MAX / 2; // would run "forever"
    config.engine.metrics_window_interval = 1 << 20;

    let mut ctrl = hg_sim_core::LiveSimController::new(config);
    ctrl.start();
    // Let it take some steps, then stop.
    std::thread::sleep(std::time::Duration::from_millis(150));
    ctrl.send_command(SimCommand::Quit);
    ctrl.join();

    let run_dir = dir.path().join("stopped_run");
    let manifest = RunManifest::load(&run_dir).unwrap();
    assert_eq!(manifest.state(), RunState::Interrupted);
    assert!(manifest.end_time.is_some());
    assert!(manifest.final_step > 0, "ran before the stop");
    // The interrupted run still produced its report.
    assert!(run_dir.join("report.md").exists());
}

/// Pause/resume mid-run with durable manifest transitions.
#[test]
fn pause_and_resume_transition_manifest() {
    use hg_app_api::{SimCommand, SimController};

    let dir = tempfile::tempdir().unwrap();
    let mut config = test_config(dir.path(), 2, 32);
    config.run_id = "paused_run".into();
    config.engine.max_steps = u64::MAX / 2;
    config.engine.metrics_window_interval = 1 << 20;

    let mut ctrl = hg_sim_core::LiveSimController::new(config);
    // Pause requested BEFORE start: the first loop iteration parks, so the
    // test is deterministic.
    ctrl.send_command(SimCommand::Pause);
    ctrl.start();

    let run_dir = dir.path().join("paused_run");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        if let Ok(m) = RunManifest::load(&run_dir) {
            if m.state() == RunState::Paused {
                break;
            }
        }
        assert!(std::time::Instant::now() < deadline, "never reached Paused");
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    ctrl.send_command(SimCommand::Resume);
    std::thread::sleep(std::time::Duration::from_millis(100));
    ctrl.send_command(SimCommand::Quit);
    ctrl.join();

    let manifest = RunManifest::load(&run_dir).unwrap();
    assert_eq!(manifest.state(), RunState::Interrupted);
    assert!(manifest.final_step > 0, "resumed and stepped after pause");
}

#[test]
fn speed_throttle_paces_the_step_loop() {
    use hg_app_api::RunControl;
    use hg_sim_core::run_controlled;

    let dir = tempfile::tempdir().unwrap();
    let mut config = test_config(dir.path(), 1, 42);
    config.engine.max_steps = 30;
    config.run_id = "throttled".into();

    let control = RunControl::new();
    control.set_speed_steps_per_sec(200.0); // 30 steps @ 200/s ≈ 145 ms minimum
    let t0 = std::time::Instant::now();
    run_controlled(&config, None, None, None, Some(&control)).unwrap();
    let throttled = t0.elapsed();
    assert!(
        throttled >= std::time::Duration::from_millis(100),
        "throttle had no effect: {throttled:?}"
    );

    // Default (uncapped) control must not pace at all — same run is far
    // faster than the throttled floor.
    let mut config = test_config(dir.path(), 1, 42);
    config.engine.max_steps = 30;
    config.run_id = "uncapped".into();
    let control = RunControl::new();
    assert_eq!(control.speed_steps_per_sec(), f32::INFINITY);
    let t0 = std::time::Instant::now();
    run_controlled(&config, None, None, None, Some(&control)).unwrap();
    assert!(t0.elapsed() < throttled, "uncapped run slower than throttled");
}

#[test]
fn set_speed_sanitizes_invalid_values_to_uncapped() {
    use hg_app_api::RunControl;
    let control = RunControl::new();
    for bad in [0.0f32, -5.0, f32::NAN, f32::NEG_INFINITY] {
        control.set_speed_steps_per_sec(50.0);
        control.set_speed_steps_per_sec(bad);
        assert_eq!(control.speed_steps_per_sec(), f32::INFINITY, "{bad} should uncap");
    }
}

/// Regression (identical-renders bug): over the GUI node cap, the
/// published snapshot must be a contiguous BFS fragment forming a VALID
/// subgraph — every published edge's endpoints all published, and the
/// fragment's connectivity preserved. The old first-N-in-slotmap-order
/// sample de-edged splitting-rule graphs (edges pointed at out-of-cap
/// midpoints), so every big run rendered as the same edge-free blob.
#[test]
fn gui_snapshot_sample_is_a_connected_valid_subgraph() {
    use hg_app_api::SharedState;
    use hg_core::{HyperEdge, HypergraphStore, Label, NodeData};
    use smallvec::SmallVec;

    // A 6000-node path: over the 5000 cap, maximally sample-fragile.
    let mut store = HypergraphStore::new();
    let mut prev = None;
    for _ in 0..6000 {
        let id = store.add_node(NodeData::new(Label(0), 0));
        if let Some(p) = prev {
            store.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[p, id])));
        }
        prev = Some(id);
    }

    let shared = SharedState::new();
    hg_sim_core::publish_graph(&shared, &store);
    let compact = shared.compact_graph.read();

    assert_eq!(compact.nodes.len(), 5000, "cap respected");
    let members: std::collections::HashSet<_> = compact.nodes.iter().map(|n| n.id).collect();
    for edge in &compact.edges {
        assert!(
            edge.nodes.iter().all(|n| members.contains(n)),
            "published edge references an unpublished node"
        );
    }
    // A contiguous 5000-node fragment of a path has 4999 edges; a
    // membership-blind sample would keep far fewer valid ones.
    assert!(
        compact.edges.len() >= 4900,
        "sample lost connectivity: only {} edges",
        compact.edges.len()
    );
}

/// Under the cap the entire graph is published, edges included.
#[test]
fn gui_snapshot_under_cap_publishes_everything() {
    use hg_app_api::SharedState;
    use hg_core::{HyperEdge, HypergraphStore, Label, NodeData};
    use smallvec::SmallVec;

    let mut store = HypergraphStore::new();
    let a = store.add_node(NodeData::new(Label(0), 0));
    let b = store.add_node(NodeData::new(Label(1), 0));
    store.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[a, b])));

    let shared = SharedState::new();
    hg_sim_core::publish_graph(&shared, &store);
    let compact = shared.compact_graph.read();
    assert_eq!(compact.nodes.len(), 2);
    assert_eq!(compact.edges.len(), 1);
}
