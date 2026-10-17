fn main() {
    let args: Vec<_> = std::env::args().collect();
    #[cfg(feature = "ere-complete-signal")]
    if args
        .get(1)
        .is_some_and(|a| a == "direction-development-complete" || a == "direction-assay-complete")
    {
        let result = if args.len() != 4 {
            Err("usage: ere-train direction-development-complete|direction-assay-complete SEALED_TRAINING_DIR FRESH_OUTPUT_DIR".into())
        } else {
            hg_train::ere::direction::run(
                std::path::Path::new(&args[2]),
                std::path::Path::new(&args[3]),
                args[1] == "direction-development-complete",
            )
        };
        if let Err(error) = result {
            eprintln!("{error}");
            std::process::exit(1);
        }
        return;
    }
    #[cfg(feature = "ere-complete-signal")]
    if args
        .get(1)
        .is_some_and(|a| a == "minibatch-development-complete" || a == "train-minibatch-complete")
    {
        let result = if args.len() != 4 {
            Err("usage: ere-train minibatch-development-complete|train-minibatch-complete SEALED_TRAINING_DIR FRESH_OUTPUT_DIR".into())
        } else {
            hg_train::ere::minibatch::run(
                std::path::Path::new(&args[2]),
                std::path::Path::new(&args[3]),
                args[1] == "minibatch-development-complete",
            )
        };
        if let Err(error) = result {
            eprintln!("{error}");
            std::process::exit(1);
        }
        return;
    }
    #[cfg(feature = "ere-complete-signal")]
    if args.get(1).is_some_and(|a| a == "train-feasible-complete") {
        let result = if args.len() != 4 {
            Err(
                "usage: ere-train train-feasible-complete SEALED_TRAINING_DIR FRESH_OUTPUT_DIR"
                    .into(),
            )
        } else {
            hg_train::ere::feasibility_training::run(
                std::path::Path::new(&args[2]),
                std::path::Path::new(&args[3]),
            )
        };
        if let Err(error) = result {
            eprintln!("{error}");
            std::process::exit(1);
        }
        return;
    }
    #[cfg(feature = "ere-complete-signal")]
    if args.get(1).is_some_and(|a| a == "confirm-short-complete") {
        let result: Result<(), String> = (|| {
            if args.len() != 4 {
                return Err(
                    "usage: ere-train confirm-short-complete SEALED_TRAINING_DIR FRESH_OUTPUT_DIR"
                        .into(),
                );
            }
            hg_train::ere::confirmation::run(
                std::path::Path::new(&args[2]),
                std::path::Path::new(&args[3]),
            )
        })();
        if let Err(error) = result {
            eprintln!("{error}");
            std::process::exit(1);
        }
        return;
    }
    #[cfg(feature = "ere-complete-signal")]
    if args
        .get(1)
        .is_some_and(|a| a == "train-short-continuation-complete")
    {
        let result: Result<(), String> = (|| {
            if args.len() != 5 {
                return Err("usage: ere-train train-short-continuation-complete RECEIPTS.json SEALED_PREFIX FRESH_OUTPUT_DIR".into());
            }
            let bundle = std::path::Path::new(&args[2]);
            let mut paths: hg_train::ere::complete_admission::ReceiptPaths =
                serde_json::from_slice(&std::fs::read(bundle).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            paths.resolve_relative(bundle.parent().unwrap_or(std::path::Path::new(".")));
            hg_train::ere::short_training::run_continuation(
                &paths,
                std::path::Path::new(&args[3]),
                std::path::Path::new(&args[4]),
            )
        })();
        if let Err(error) = result {
            eprintln!("{error}");
            std::process::exit(1);
        }
        return;
    }
    #[cfg(feature = "ere-complete-signal")]
    if args.get(1).is_some_and(|a| a == "train-short-complete") {
        let result: Result<(), String> = (|| {
            if args.len() != 4 {
                return Err(
                    "usage: ere-train train-short-complete RECEIPTS.json FRESH_OUTPUT_DIR".into(),
                );
            }
            let bundle = std::path::Path::new(&args[2]);
            let mut paths: hg_train::ere::complete_admission::ReceiptPaths =
                serde_json::from_slice(&std::fs::read(bundle).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            paths.resolve_relative(bundle.parent().unwrap_or(std::path::Path::new(".")));
            hg_train::ere::short_training::run(&paths, std::path::Path::new(&args[3]))
        })();
        if let Err(error) = result {
            eprintln!("{error}");
            std::process::exit(1);
        }
        return;
    }
    #[cfg(feature = "ere-complete-signal")]
    if args.get(1).is_some_and(|a| a == "scaling-complete") {
        let result: Result<(), String> = (|| {
            if args.len() != 5 {
                return Err(
                    "usage: ere-train scaling-complete RECEIPTS.json DESIGN.json FRESH_OUTPUT_DIR"
                        .into(),
                );
            }
            let bundle = std::path::Path::new(&args[2]);
            let mut paths: hg_train::ere::complete_admission::ReceiptPaths =
                serde_json::from_slice(&std::fs::read(bundle).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            paths.resolve_relative(bundle.parent().unwrap_or(std::path::Path::new(".")));
            let design: hg_train::ere::scaling::Design =
                serde_json::from_slice(&std::fs::read(&args[3]).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            hg_train::ere::scaling::run(&paths, &design, std::path::Path::new(&args[4]))
        })();
        if let Err(error) = result {
            eprintln!("{error}");
            std::process::exit(1);
        }
        return;
    }
    #[cfg(feature = "ere-complete-signal")]
    if args
        .get(1)
        .is_some_and(|a| a == "complete-preflight" || a == "complete-bounded")
    {
        let result = (|| {
            if args.len() != 5 {
                return Err("usage: ere-train complete-preflight|complete-bounded RECEIPTS.json REQUEST.json FRESH_OUTPUT_DIR".into());
            }
            let bundle_path = std::path::Path::new(&args[2]);
            let mut paths: hg_train::ere::complete_admission::ReceiptPaths =
                serde_json::from_slice(&std::fs::read(bundle_path).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            paths.resolve_relative(bundle_path.parent().unwrap_or(std::path::Path::new(".")));
            let request: hg_train::ere::complete_admission::Request =
                serde_json::from_slice(&std::fs::read(&args[3]).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            let out = std::path::Path::new(&args[4]);
            if args[1] == "complete-preflight" {
                hg_train::ere::complete_runner::preflight(&paths, &request, out)
            } else {
                hg_train::ere::complete_runner::run_bounded(&paths, &request, out)
            }
        })();
        if let Err(error) = result {
            eprintln!("{error}");
            std::process::exit(1);
        }
        return;
    }
    #[cfg(feature = "ere-complete-signal")]
    if args
        .get(1)
        .is_some_and(|a| a == "signal-runtime50k-complete")
    {
        let result = if args.len() != 6 {
            Err("usage: ere-train signal-runtime50k-complete OUTPUT_DIR CLOSURE_GATE_DIR GRAMMAR_DIR PRIOR_SIGNAL_DIR".into())
        } else {
            hg_train::ere::runtime_signal::run(
                std::path::Path::new(&args[2]),
                std::path::Path::new(&args[3]),
                std::path::Path::new(&args[4]),
                std::path::Path::new(&args[5]),
            )
        };
        if let Err(e) = result {
            eprintln!("{e}");
            std::process::exit(1);
        }
        return;
    }
    #[cfg(feature = "ere-complete-signal")]
    if args.get(1).is_some_and(|a| a == "signal-valid-complete") {
        let result = if args.len() != 5 {
            Err(
                "usage: ere-train signal-valid-complete OUTPUT_DIR CLOSURE_GATE_DIR GRAMMAR_DIR"
                    .into(),
            )
        } else {
            hg_train::ere::valid_signal::run(
                std::path::Path::new(&args[2]),
                std::path::Path::new(&args[3]),
                std::path::Path::new(&args[4]),
            )
        };
        if let Err(e) = result {
            eprintln!("{e}");
            std::process::exit(1);
        }
        return;
    }
    #[cfg(feature = "ere-complete-signal")]
    if args.get(1).is_some_and(|a| a == "grammar-complete") {
        let result = if args.len() != 3 {
            Err("usage: ere-train grammar-complete OUTPUT_DIR".into())
        } else {
            hg_train::ere::valid_grammar::export(std::path::Path::new(&args[2]))
        };
        if let Err(e) = result {
            eprintln!("{e}");
            std::process::exit(1);
        }
        return;
    }
    #[cfg(feature = "ere-complete-signal")]
    if args.get(1).is_some_and(|a| a == "signal-complete") {
        let result = if args.len() != 4 {
            Err("usage: ere-train signal-complete OUTPUT_DIR CLOSURE_GATE_DIR".into())
        } else {
            hg_train::ere::signal::run(
                std::path::Path::new(&args[2]),
                std::path::Path::new(&args[3]),
            )
        };
        if let Err(e) = result {
            eprintln!("{e}");
            std::process::exit(1);
        }
        return;
    }
    if args
        .get(1)
        .is_some_and(|a| a == "vocabulary-assay" || a == "persistence-assay" || a == "repair-assay")
    {
        let result = (|| {
            if args.len() != 4 {
                return Err("usage: ere-train vocabulary-assay|persistence-assay|repair-assay BASE_CONFIG.toml OUTPUT_DIR".into());
            }
            let cfg = hg_sim_core::SimConfig::from_toml(
                &std::fs::read_to_string(&args[2]).map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;
            let run = if args[1] == "repair-assay" {
                hg_train::ere::repair::run_assay
            } else if args[1] == "persistence-assay" {
                hg_train::ere::persistence::run_assay
            } else {
                hg_train::ere::vocabulary::run_assay
            };
            run(&cfg, std::path::Path::new(&args[3]))
        })();
        if let Err(error) = result {
            eprintln!("{error}");
            std::process::exit(1);
        }
        return;
    }
    if args
        .get(1)
        .is_some_and(|a| a == "pilot-e2" || a == "pilot-maintenance")
    {
        let result = (|| {
            if args.len() != 6 {
                return Err("usage: ere-train pilot-e2 BASE_CONFIG.toml OUTPUT_DIR CLASSIFICATION_DIR DOWNSTREAM_GATE_DIR".into());
            }
            let config = hg_sim_core::SimConfig::from_toml(
                &std::fs::read_to_string(&args[2]).map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;
            let run = if args[1] == "pilot-maintenance" {
                hg_train::ere::maintenance::run_pilot
            } else {
                hg_train::ere::e2::run_pilot
            };
            run(
                &config,
                std::path::Path::new(&args[3]),
                std::path::Path::new(&args[4]),
                std::path::Path::new(&args[5]),
            )
        })();
        if let Err(error) = result {
            eprintln!("{error}");
            std::process::exit(1);
        }
        return;
    }
    if args.len() != 4 {
        eprintln!("usage: ere-train BASE_CONFIG.toml OUTPUT_DIR GATE_RECEIPTS_DIR");
        std::process::exit(2);
    }
    let result = (|| {
        let text = std::fs::read_to_string(&args[1]).map_err(|e| e.to_string())?;
        let config = hg_sim_core::SimConfig::from_toml(&text).map_err(|e| e.to_string())?;
        hg_train::ere::run_campaign(
            &config,
            std::path::Path::new(&args[2]),
            std::path::Path::new(&args[3]),
            &hg_train::ere::Budget::default(),
        )
    })();
    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
