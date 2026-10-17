//! hypersim-cli: headless runner. Every run produces exactly one CSV and
//! one Markdown report; manifest.json is the durable status record.

use clap::{Parser, Subcommand};
use hg_sim_core::{run_headless, SimConfig};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "hypersim-cli", version, about = "Headless Hypersim runner")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a simulation from a TOML config.
    Run {
        /// Path to a SimConfig TOML file.
        #[arg(short, long)]
        config: PathBuf,
        /// Override max_steps.
        #[arg(long)]
        steps: Option<u64>,
        /// Override the seed.
        #[arg(long)]
        seed: Option<u64>,
        /// Override the output directory.
        #[arg(long)]
        out_dir: Option<PathBuf>,
        /// Override the thread count.
        #[arg(long)]
        threads: Option<usize>,
    },
    /// Run the six-fixture evaluator truth table and print the results.
    Fixtures,
    /// Twin-run counterfactual harness (LK3): branch at a step, perturb
    /// one hyperedge in branch B, and measure the label-invariant damage
    /// front. Writes twin.csv into --out-dir. MaxParallel only.
    Twin {
        /// SimConfig TOML defining rules + initial condition.
        #[arg(short, long)]
        config: PathBuf,
        /// Step at which the branches split.
        #[arg(long, default_value_t = 8)]
        branch_step: u64,
        /// Windows to run after branching.
        #[arg(long, default_value_t = 16)]
        windows: u64,
        /// Steps per window.
        #[arg(long, default_value_t = 4)]
        window_len: u64,
        /// Perturbation: remove-edge | add-edge | control.
        #[arg(long, default_value = "remove-edge")]
        perturb: String,
        /// Anchor count.
        #[arg(long, default_value_t = 64)]
        anchors: usize,
        /// Anchor ball radius.
        #[arg(long, default_value_t = 3)]
        anchor_radius: usize,
        /// Sampling seed (anchors + perturbation choice).
        #[arg(long, default_value_t = 7)]
        sample_seed: u64,
        /// Where to write twin.csv (default: alongside the config).
        #[arg(long)]
        out_dir: Option<PathBuf>,
    },
    /// Campaign cross-run leakage table (LK5): rank run directories by
    /// leakage profile (steady wormhole_frac, area_law_defect, CD) and
    /// fire the campaign-level suspicious flags.
    LeakageTable {
        /// Run directories (each containing leakage.csv; closure.json
        /// optional).
        #[arg(required = true)]
        runs: Vec<PathBuf>,
    },
    /// Chain census (EXP-L9, plan v3 §3): census every snapshot in a run
    /// dir (maximal paths of degree-2, generation > 0 nodes) and write
    /// chains.csv. The run must have had [output] snapshots_enabled =
    /// true and the capped rule family (generation inheritance) active.
    Chains {
        /// Run directory (containing snapshots/).
        #[arg(long)]
        run_dir: PathBuf,
    },
    /// Cone analysis (LK2): join a run's causal_events.jsonl with its
    /// snapshots and write cone.csv + cone_summary.json into the run dir.
    /// The run must have had [leakage] cone_events_enabled = true and
    /// [output] snapshots_enabled = true.
    Cone {
        /// Run directory (containing causal_events.jsonl and snapshots/).
        #[arg(long)]
        run_dir: PathBuf,
        /// Steps of causal future per sampled event.
        #[arg(long, default_value_t = 200)]
        delta: u64,
        /// Sampled source events.
        #[arg(long, default_value_t = 128)]
        events: usize,
        /// Sampler seed.
        #[arg(long, default_value_t = 42)]
        seed: u64,
        /// Anomaly slack (sites) added to v_eff * dt.
        #[arg(long, default_value_t = 2.0)]
        slack: f64,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // RUST_LOG=info (or debug for per-window spans) enables tracing output.
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    match Cli::parse().command {
        Commands::Run {
            config,
            steps,
            seed,
            out_dir,
            threads,
        } => {
            let content = std::fs::read_to_string(&config)?;
            let mut config = SimConfig::from_toml(&content)?;
            if let Some(s) = steps {
                config.engine.max_steps = s;
            }
            if let Some(s) = seed {
                config.seed = s;
            }
            if let Some(d) = out_dir {
                config.out_dir = d;
            }
            if let Some(t) = threads {
                config.engine.thread_count = t;
            }

            // Headless: no shared state, no event listeners — outputs land
            // on disk regardless (v4 §9.1).
            let artifacts = run_headless(&config, None, None, None)?;
            println!("run dir     : {}", artifacts.run_dir.display());
            println!("csv         : {}", artifacts.csv_path.display());
            println!("report      : {}", artifacts.report_path.display());
            println!("manifest    : {}", artifacts.manifest_path.display());
            println!("final step  : {}", artifacts.final_step);
            println!("canonical   : {:016x}", artifacts.canonical_hash);
            if let Some(m) = &artifacts.summary.final_metrics {
                println!(
                    "final window: {} nodes, {} edges, regime {}",
                    m.n_nodes,
                    m.n_edges,
                    m.regime.as_deref().unwrap_or("n/a")
                );
            }
            Ok(())
        }
        Commands::Twin {
            config,
            branch_step,
            windows,
            window_len,
            perturb,
            anchors,
            anchor_radius,
            sample_seed,
            out_dir,
        } => {
            let content = std::fs::read_to_string(&config)?;
            let sim_config = hg_sim_core::SimConfig::from_toml(&content)?;
            let perturbation = match perturb.as_str() {
                "remove-edge" => hg_sim_core::TwinPerturbation::RemoveSampledEdge,
                "add-edge" => hg_sim_core::TwinPerturbation::AddRandomEdge,
                "control" => hg_sim_core::TwinPerturbation::Control,
                other => return Err(format!("unknown perturbation '{other}'").into()),
            };
            let params = hg_sim_core::TwinParams {
                anchors,
                anchor_radius,
                windows,
                window_len,
                sample_seed,
            };
            let outcome = hg_sim_core::run_twin(&sim_config, branch_step, &params, perturbation)?;
            let dir = out_dir
                .or_else(|| config.parent().map(|p| p.to_path_buf()))
                .unwrap_or_else(|| PathBuf::from("."));
            std::fs::create_dir_all(&dir)?;
            let path = dir.join("twin.csv");
            let mut w: hg_output::CsvWriter<hg_schema::TwinRow> = hg_output::CsvWriter::create(
                &path,
                1,
                &sim_config.experiment_name,
                sim_config.seed,
                "",
            )?;
            for row in &outcome.rows {
                w.write_row(row)?;
            }
            w.flush()?;
            println!("perturbation site : {} node(s)", outcome.site.len());
            println!("anchor attrition  : {}", outcome.total_attrition);
            println!("v_damage          : {:.4} sites/step [Estimated]", outcome.v_damage);
            if perturbation == hg_sim_core::TwinPerturbation::Control
                && !outcome.damage_always_zero
            {
                println!("SUSPICIOUS: control_twin_nonzero_damage — harness invalid, fix determinism before proceeding");
            }
            println!("wrote {}", path.display());
            Ok(())
        }
        Commands::LeakageTable { runs } => {
            let mut rows: Vec<(String, f32, f32, Option<f64>)> = Vec::new();
            for dir in &runs {
                let name = dir
                    .file_name()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_else(|| dir.display().to_string());
                let csv = std::fs::read_to_string(dir.join("leakage.csv"))?;
                // Steady-state = mean over the last quarter of emissions.
                let data: Vec<&str> = csv
                    .lines()
                    .filter(|l| !l.starts_with('#') && !l.starts_with("step,"))
                    .collect();
                if data.is_empty() {
                    println!("{name}: leakage.csv has no data rows — skipped");
                    continue;
                }
                let tail = &data[data.len() - data.len().div_ceil(4)..];
                let field = |line: &str, ix: usize| -> f32 {
                    line.split(',').nth(ix).and_then(|v| v.parse().ok()).unwrap_or(f32::NAN)
                };
                let mean = |ix: usize| -> f32 {
                    let vals: Vec<f32> = tail
                        .iter()
                        .map(|l| field(l, ix))
                        .filter(|v| v.is_finite())
                        .collect();
                    if vals.is_empty() {
                        f32::NAN
                    } else {
                        vals.iter().sum::<f32>() / vals.len() as f32
                    }
                };
                // Columns: 3 = wormhole_frac, 6 = area_law_defect.
                let wormhole = mean(3);
                let defect = mean(6);
                let cd = std::fs::read_to_string(dir.join("closure.json"))
                    .ok()
                    .and_then(|j| serde_json::from_str::<serde_json::Value>(&j).ok())
                    // v3 §1.2 tuple: rank by CD_clock (the interpretable
                    // number); fall back to the v1 scalar for old runs.
                    .and_then(|v| {
                        v["cd_clock_bits"]
                            .as_f64()
                            .or_else(|| v["cd_raw_bits"].as_f64())
                            .or_else(|| v["cd_bits_per_window"].as_f64())
                    });
                rows.push((name, wormhole, defect, cd));
            }
            // Rank by CD (descending, absent last), then wormhole_frac.
            rows.sort_by(|a, b| {
                b.3.unwrap_or(f64::NEG_INFINITY)
                    .total_cmp(&a.3.unwrap_or(f64::NEG_INFINITY))
                    .then(b.1.total_cmp(&a.1))
            });
            println!(
                "{:<40} {:>14} {:>16} {:>12}",
                "run", "wormhole_frac", "area_law_defect", "CD (bits)"
            );
            for (name, w, d, cd) in &rows {
                println!(
                    "{name:<40} {w:>14.4} {d:>16.4} {:>12}",
                    cd.map(|c| format!("{c:.4}")).unwrap_or_else(|| "—".into())
                );
            }
            // Campaign-level suspicious flags (anti-gaslight layer).
            if rows.len() >= 2 {
                if rows.windows(2).all(|w| w[0].1 == w[1].1) {
                    println!("SUSPICIOUS: wormhole_frac_identical_across_all_conditions");
                }
                if rows
                    .windows(2)
                    .all(|w| w[0].1 == w[1].1 && w[0].2 == w[1].2 && w[0].3 == w[1].3)
                {
                    println!("SUSPICIOUS: all_conditions_identical_leakage_profile");
                }
            }
            Ok(())
        }
        Commands::Chains { run_dir } => {
            let rows = hg_eval::chains::census_run_dir(&run_dir)?;
            if rows.is_empty() {
                println!("no snapshots found in {} — was [output] snapshots_enabled = true?", run_dir.join("snapshots").display());
            }
            for r in &rows {
                println!(
                    "step {:>10}: {} chains, len p50 {} / p95 {} / max {}",
                    r.snapshot_step,
                    r.chains_count,
                    r.chain_length_p50,
                    r.chain_length_p95,
                    r.chain_length_max
                );
            }
            println!("wrote {}", run_dir.join("chains.csv").display());
            Ok(())
        }
        Commands::Cone {
            run_dir,
            delta,
            events,
            seed,
            slack,
        } => {
            let params = hg_eval::cone::ConeParams {
                delta,
                sample_events: events,
                seed,
                slack,
                detour_r_max: 12,
                censor: hg_eval::cone::CensorMode::Wormhole,
            };
            let analysis = hg_eval::cone::analyze_run_dir(&run_dir, &params)?;
            hg_eval::cone::write_outputs(&run_dir, &analysis)?;
            println!("sampled events    : {}", analysis.rows.len());
            println!("v_eff             : {:.4} sites/step [Estimated]", analysis.v_eff);
            for (lhs, v) in &analysis.v_eff_by_lhs {
                println!("v_eff (lhs={lhs})     : {v:.4}");
            }
            println!(
                "dynamics channel  : {} descendants, {} anomalous",
                analysis.descendants_dynamics, analysis.anomalous_dynamics
            );
            println!(
                "cleanup channel   : {} descendants, {} anomalous",
                analysis.descendants_cleanup, analysis.anomalous_cleanup
            );
            println!("excluded (absent) : {}", analysis.excluded_descendants);
            println!("censor set (max)  : {}", analysis.max_censor_set_size);
            println!("out-of-phase evts : {}", analysis.out_of_phase_events);
            if analysis.out_of_phase_events > 0 {
                println!(
                    "NOTE: {} sampled events were phase-gated (cone_status = out_of_phase); \
their rows carry no cone numbers (v2 §1.3)",
                    analysis.out_of_phase_events
                );
            }
            if analysis.cleanup_channel_dominates {
                println!(
                    "SUSPICIOUS: cleanup_channel_dominates_cone_anomalies — this run's \
                     cone anomalies are engine bookkeeping, not substrate dynamics"
                );
            }
            println!("wrote {}", run_dir.join("cone.csv").display());
            Ok(())
        }
        Commands::Fixtures => {
            let outcomes = hg_eval::run_truth_table();
            println!("{:<22} {:<28} {:<8} interestingness", "fixture", "classified", "pass");
            for o in &outcomes {
                println!(
                    "{:<22} {:<28} {:<8} {:.3}",
                    o.name,
                    format!("{:?}", o.regime),
                    if o.passed { "PASS" } else { "FAIL" },
                    o.mean_interestingness
                );
            }
            if outcomes.iter().all(|o| o.passed) {
                println!("\nfixture gate: PASS");
                Ok(())
            } else {
                Err("fixture gate: FAIL".into())
            }
        }
    }
}
