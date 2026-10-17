//! Every builtin the Experiment menu offers must parse into a SimConfig
//! and complete a (shortened) headless run — the config-level guarantee
//! behind "load each builtin from the dropdown; confirm each starts".
#![cfg(feature = "live")]

use hg_gui::app::builtin_config_toml;
use hg_sim_core::{run_headless, SimConfig, BUILTIN_NAMES};

#[test]
fn every_builtin_preset_parses_and_runs() {
    for name in BUILTIN_NAMES {
        let toml = builtin_config_toml(name, 30, 42);
        let mut config = SimConfig::from_toml(&toml)
            .unwrap_or_else(|e| panic!("preset for '{name}' failed to parse: {e}"));
        // Shorten hard for test speed (subdividing rule sets grow the
        // graph fast enough that debug-mode matching dominates); the GUI
        // uses the full preset.
        config.engine.max_steps = 16;
        config.engine.metrics_window_interval = 8;
        config.engine.max_pairings_per_step = 2;
        let dir = tempfile::tempdir().unwrap();
        config.out_dir = dir.path().to_path_buf();
        config.run_id = format!("preset_{name}");
        run_headless(&config, None, None, None)
            .unwrap_or_else(|e| panic!("preset run for '{name}' failed: {e}"));
    }
}

#[test]
fn malformed_config_toml_is_an_error_not_a_panic() {
    assert!(SimConfig::from_toml("this is { not TOML").is_err());
    // Valid TOML, wrong shape: missing required experiment_name/seed.
    assert!(SimConfig::from_toml("[rules]\nsource = \"Builtin\"").is_err());
}
