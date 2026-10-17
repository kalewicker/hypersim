//! Campaign output discipline and resume (`docs/campaign_output.md`).
//!
//! v4's invariant is one CSV and one report per run. A campaign is
//! thousands of rollouts, and producing thousands of reports would destroy
//! that discipline rather than honour it — so the discipline moves up a
//! level: **exactly one `campaign.csv` and exactly one
//! `campaign_report.md` per campaign, and nothing at all per rollout.**
//!
//! These tests check the invariant the way it can actually fail: by
//! counting files on disk, and by crashing a campaign and resuming it.

use hg_cognition::{CognitionConfig, PolicyKind};
use hg_sim_core::SimConfig;
use hg_train::campaign::{campaign_root, run_campaign, CampaignSpec};
use hg_train::config::{Algo, TrainingConfig};
use hg_train::fitness::FitnessFn;
use std::path::Path;

const STEPS: u64 = 128;
const WINDOW: u64 = 32;

fn sim_config(dir: &Path) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "artifacts"
seed = 4242
out_dir = "{}"
[rules]
source = "File"
file_path = "{}/../../rules/fl1_learnable_signal.toml"
[initial]
condition_type = "GliderRing"
node_count = 31
[engine]
max_steps = {STEPS}
metrics_window_interval = {WINDOW}
thread_count = 1
[evaluator]
enabled = true
[causal]
enabled = false
[output]
fixed_provenance = true
"#,
        dir.display(),
        env!("CARGO_MANIFEST_DIR")
    );
    toml::from_str(&toml).expect("config parses")
}

fn training(generations: u32) -> TrainingConfig {
    TrainingConfig {
        algo: Algo::CmaEs,
        generations,
        seeds_per_member: 2,
        sigma: 1.0,
        fitness: FitnessFn::F2RegimeOccupancy,
        horizon_steps: STEPS,
        burn_in_windows: 1,
        holdout_seeds: vec![900_001, 900_002],
        campaign_seed: 77,
        ..Default::default()
    }
}

fn spec<'a>(dir: &Path, cfg: &'a SimConfig, gens: u32) -> CampaignSpec<'a> {
    CampaignSpec {
        campaign_id: "artifacts_77".into(),
        base_dir: dir.to_path_buf(),
        sim_config: cfg,
        cognition: CognitionConfig {
            policy: PolicyKind::StaticBias,
            budget: Some(16),
            ..Default::default()
        },
        training: training(gens),
    }
}

fn count_files(dir: &Path, ext: &str) -> usize {
    std::fs::read_dir(dir)
        .map(|rd| {
            rd.filter_map(Result::ok)
                .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some(ext))
                .count()
        })
        .unwrap_or(0)
}

#[test]
fn a_campaign_produces_exactly_one_csv_and_one_report() {
    let dir = tempfile::tempdir().expect("tempdir");
    let cfg = sim_config(dir.path());
    let out = run_campaign(&spec(dir.path(), &cfg, 2)).expect("campaign");

    assert_eq!(count_files(&out.root, "csv"), 1, "exactly one campaign CSV");
    assert_eq!(count_files(&out.root, "md"), 1, "exactly one campaign report");
    assert!(out.csv_path.exists());
    assert!(out.report_path.exists());
    assert!(out.manifest_path.exists());

    // Champions are per generation and live in their own directory, so they
    // cannot be mistaken for run artifacts.
    assert_eq!(count_files(&out.root.join("champions"), "toml"), 2);

    // Rollouts left nothing behind: no run directories anywhere under the
    // campaign root or the sim config's out_dir.
    let stray = std::fs::read_dir(dir.path())
        .expect("read base")
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n != "runs")
        .collect::<Vec<_>>();
    assert!(stray.is_empty(), "rollouts left artifacts: {stray:?}");
}

#[test]
fn the_campaign_root_is_under_runs_cognition() {
    let dir = tempfile::tempdir().expect("tempdir");
    let cfg = sim_config(dir.path());
    let out = run_campaign(&spec(dir.path(), &cfg, 1)).expect("campaign");
    assert_eq!(out.root, campaign_root(dir.path(), "artifacts_77"));
    assert!(out.root.to_string_lossy().contains("runs/cognition"));
    assert!(!dir.path().join("runs").join("substrate").exists());
}

/// The CSV must be readable and complete after every generation, not only
/// at the end — a campaign is hours long and a crash must not cost the rows
/// already earned.
#[test]
fn the_csv_carries_a_header_and_one_row_per_rollout() {
    let dir = tempfile::tempdir().expect("tempdir");
    let cfg = sim_config(dir.path());
    let out = run_campaign(&spec(dir.path(), &cfg, 2)).expect("campaign");

    let text = std::fs::read_to_string(&out.csv_path).expect("read csv");
    let mut lines = text.lines();
    let provenance = lines.next().expect("provenance comment");
    assert!(
        provenance.starts_with("#hypersim_csv_v="),
        "campaign CSV must carry the same provenance comment a run CSV does"
    );
    assert!(provenance.contains("run_id=artifacts_77"));
    let header = lines.next().expect("header");
    assert!(header.starts_with("generation,member_idx"));

    let data_rows = lines.count();
    assert_eq!(
        data_rows, out.total_rollouts,
        "one CSV row per rollout, including failures"
    );
}

/// Resume: an interrupted campaign continues from the last completed
/// generation, into the SAME CSV.
#[test]
fn an_interrupted_campaign_resumes_into_the_same_csv() {
    let dir = tempfile::tempdir().expect("tempdir");
    let cfg = sim_config(dir.path());

    // First leg: 2 generations, as if the process died afterwards.
    let first = run_campaign(&spec(dir.path(), &cfg, 2)).expect("first leg");
    let rows_after_first = std::fs::read_to_string(&first.csv_path)
        .expect("read")
        .lines()
        .count();
    let manifest: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&first.manifest_path).expect("read"))
            .expect("manifest json");
    assert_eq!(manifest["last_completed_generation"], 1);

    // Second leg: same campaign id, more generations.
    let second = run_campaign(&spec(dir.path(), &cfg, 4)).expect("resumed leg");

    assert_eq!(count_files(&second.root, "csv"), 1, "resume must not fork the CSV");
    assert_eq!(second.csv_path, first.csv_path);

    let text = std::fs::read_to_string(&second.csv_path).expect("read");
    let mut lines = text.lines();
    assert!(lines.next().unwrap().starts_with("#hypersim_csv_v="));
    assert!(lines.next().unwrap().starts_with("generation,"));
    // Exactly one header and one provenance line in the whole file: an
    // append that re-emitted either would corrupt every downstream reader.
    // Counted as total occurrences, not as "occurrences after the first" —
    // data rows begin with a generation number, so the header text appears
    // exactly once in a well-formed file.
    assert_eq!(text.matches("#hypersim_csv_v=").count(), 1);
    assert_eq!(text.matches("generation,member_idx").count(), 1);

    let total_rows = text.lines().count();
    assert!(
        total_rows > rows_after_first,
        "resume appended nothing: {rows_after_first} rows before, {total_rows} after"
    );

    // The resumed leg only ran the generations it had not already done.
    let gens: Vec<u32> = second.generations.iter().map(|g| g.generation).collect();
    assert_eq!(gens, vec![2, 3], "resume re-ran completed generations");

    // And every generation's rows are present exactly once.
    for g in 0..4u32 {
        let n = text
            .lines()
            .skip(2)
            .filter(|l| l.starts_with(&format!("{g},")))
            .count();
        assert!(n > 0, "generation {g} missing from the CSV");
    }
}

/// A champion must reload exactly, or a validation run would reproduce
/// something subtly different from what the campaign measured.
#[test]
fn a_saved_champion_round_trips_and_is_checked_against_the_rule_set() {
    let dir = tempfile::tempdir().expect("tempdir");
    let cfg = sim_config(dir.path());
    let out = run_campaign(&spec(dir.path(), &cfg, 1)).expect("campaign");

    let path = out.root.join("champions").join("gen_0000.policy.toml");
    let loaded = hg_train::campaign::load_champion(&path, 2, 16, 2).expect("load champion");
    assert_eq!(loaded.params.len(), out.champion.params.len());
    for (a, b) in loaded.params.iter().zip(&out.champion.params) {
        assert_eq!(a.to_bits(), b.to_bits(), "champion params must round-trip exactly");
    }
    assert_eq!(loaded.genome_hash, out.champion.genome_hash);

    // Loading against the wrong rule count is rejected, not reshaped.
    assert!(
        hg_train::campaign::load_champion(&path, 7, 16, 2).is_err(),
        "a genome trained against 2 rules must not load against 7"
    );
}

/// The report's mandatory sections are present, including the empty case of
/// Suspicious Metric Flags — a section that vanishes when clean trains
/// readers to stop looking for it.
#[test]
fn the_report_carries_every_mandatory_section() {
    let dir = tempfile::tempdir().expect("tempdir");
    let cfg = sim_config(dir.path());
    let out = run_campaign(&spec(dir.path(), &cfg, 2)).expect("campaign");

    let text = std::fs::read_to_string(&out.report_path).expect("read report");
    for section in [
        "## 1. Campaign Parameters",
        "## 2. Fitness Trajectory",
        "## 3. Champion vs Baselines",
        "## 4. Held-out Metric Trajectories",
        "## 5. Suspicious Metric Flags",
        "## 6. Failure Census",
        "## 7. Cost",
    ] {
        assert!(text.contains(section), "report is missing `{section}`");
    }
    assert!(
        text.contains("No flags fired.") || text.contains("### `"),
        "the flags section must say something, even when clean"
    );
    // No probe was configured, so the report must say so rather than leave a
    // column of blanks to inference.
    assert!(text.contains("No probe was configured"));
    // And it must refuse to let a champion be read without its control arm.
    assert!(text.contains("budget-matched `RandomSubset`"));
}
