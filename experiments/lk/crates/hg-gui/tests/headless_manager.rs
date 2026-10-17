//! HeadlessRunManager: background runs complete (sealed Completed) and
//! can be stopped individually (sealed Interrupted), independent of any
//! live controller.
#![cfg(feature = "live")]

use hg_gui::app::builtin_config_toml;
use hg_gui::headless::HeadlessRunManager;
use hg_sim_core::SimConfig;
use std::time::{Duration, Instant};

fn small_config(dir: &std::path::Path, run_id: &str, max_steps: u64) -> SimConfig {
    let mut config = SimConfig::from_toml(&builtin_config_toml("binary_edge", 24, 42)).unwrap();
    config.engine.max_steps = max_steps;
    config.engine.metrics_window_interval = 8;
    config.engine.max_pairings_per_step = 4;
    config.engine.thread_count = 1;
    config.out_dir = dir.to_path_buf();
    config.run_id = run_id.into();
    config
}

fn wait_finished(mgr: &mut HeadlessRunManager, timeout: Duration) {
    let t0 = Instant::now();
    while mgr.any_active() {
        assert!(t0.elapsed() < timeout, "headless run did not finish in {timeout:?}");
        mgr.poll();
        std::thread::sleep(Duration::from_millis(20));
    }
    mgr.poll();
}

fn manifest_state(dir: &std::path::Path, run_id: &str) -> String {
    let raw = std::fs::read_to_string(dir.join(run_id).join("manifest.json")).unwrap();
    let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
    v["state"].as_str().unwrap_or_default().to_string()
}

#[test]
fn background_run_completes_and_reports_outcome() {
    let dir = tempfile::tempdir().unwrap();
    let mut mgr = HeadlessRunManager::default();
    mgr.launch("demo".into(), small_config(dir.path(), "bg_complete", 32));
    assert!(mgr.any_active() || mgr.runs[0].finished());
    wait_finished(&mut mgr, Duration::from_secs(60));

    let run = &mgr.runs[0];
    let line = run.outcome.as_ref().unwrap().as_ref().expect("run should succeed");
    assert!(line.contains("Completed"), "outcome: {line}");
    assert_eq!(manifest_state(dir.path(), "bg_complete"), "Completed");

    // Finished rows are dismissable; live ones would not be.
    mgr.dismiss(0);
    assert!(mgr.is_empty());
}

#[test]
fn background_run_can_be_stopped_and_seals_interrupted() {
    let dir = tempfile::tempdir().unwrap();
    let mut mgr = HeadlessRunManager::default();
    let mut config = small_config(dir.path(), "bg_stopped", 1_000_000);
    // Throttle hard so the run is guaranteed still going when we stop it.
    config.engine.metrics_window_interval = 64;
    mgr.launch("longrun".into(), config);
    mgr.runs[0].control.set_speed_steps_per_sec(50.0);

    std::thread::sleep(Duration::from_millis(200));
    mgr.stop(0);
    wait_finished(&mut mgr, Duration::from_secs(60));

    let line = mgr.runs[0].outcome.as_ref().unwrap().as_ref().expect("stop is not an error");
    assert!(line.contains("Interrupted"), "outcome: {line}");
    assert_eq!(manifest_state(dir.path(), "bg_stopped"), "Interrupted");
}

#[test]
fn drop_stops_and_seals_all_background_runs() {
    let dir = tempfile::tempdir().unwrap();
    {
        let mut mgr = HeadlessRunManager::default();
        let config = small_config(dir.path(), "bg_dropped", 1_000_000);
        mgr.launch("abandoned".into(), config);
        mgr.runs[0].control.set_speed_steps_per_sec(50.0);
        std::thread::sleep(Duration::from_millis(100));
        // mgr dropped here with the run still going.
    }
    assert_eq!(manifest_state(dir.path(), "bg_dropped"), "Interrupted");
}
