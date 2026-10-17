//! `hg-train` — run one campaign from a campaign TOML.
//!
//! Usage: `hg-train <campaign.toml> [--base-dir DIR]`
//!
//! The campaign file carries the full `SimConfig` plus `[cognition]` and
//! `[training]`, so one file fully determines a campaign. Output lands in
//! `<base-dir>/runs/cognition/<campaign_id>/`; there is no output-path key,
//! by design (`docs/campaign_output.md` §5.1).

use hg_train::campaign::{CampaignSpec, TrainError};
use hg_train::config::TrainingConfig;
use serde::Deserialize;

#[derive(Deserialize)]
struct CampaignFile {
    #[serde(flatten)]
    sim: hg_sim_core::SimConfig,
    #[serde(default)]
    training: TrainingConfig,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let mut args = std::env::args().skip(1);
    let path = args
        .next()
        .ok_or("usage: hg-train <campaign.toml> [--base-dir DIR]")?;
    let mut base_dir = std::path::PathBuf::from(".");
    while let Some(a) = args.next() {
        if a == "--base-dir" {
            base_dir = args.next().ok_or("--base-dir needs a value")?.into();
        } else {
            return Err(format!("unknown argument {a}").into());
        }
    }

    let text = std::fs::read_to_string(&path)?;
    let file: CampaignFile = toml::from_str(&text)?;
    let cognition = file
        .sim
        .cognition
        .clone()
        .ok_or("campaign config has no [cognition] section — nothing to train")?;

    let campaign_id = format!(
        "{}_{}",
        file.sim.experiment_name, file.training.campaign_seed
    );
    let spec = CampaignSpec {
        campaign_id,
        base_dir,
        sim_config: &file.sim,
        cognition,
        training: file.training,
    };

    let out = hg_train::run_campaign(&spec).map_err(|e: TrainError| e.to_string())?;
    println!("campaign {} complete", out.campaign_id);
    println!("  csv    {}", out.csv_path.display());
    println!("  report {}", out.report_path.display());
    println!("  rollouts {}", out.total_rollouts);
    if let Some(g) = out.generations.last() {
        println!(
            "  champion hold-out fitness {:.4} ± {:.4}",
            g.champion_holdout_mean, g.champion_holdout_sd
        );
    }
    Ok(())
}
