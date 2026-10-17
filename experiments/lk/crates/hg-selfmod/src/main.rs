//! CLI: gate0 | gate1 | gate2 | gate42 | gate43 | gate44 | gates | phaseA |
//! phaseB, with optional config file and `--key=value` overrides. Every
//! experiment run snapshots its config.

use hg_selfmod::economy::{EconomyMode, FissionMode, IncomeMode};
use hg_selfmod::gates;
use hg_selfmod::resolve::TieBreak;
use hg_selfmod::runner::{run_phase, write_run, RunConfig, TruncationMode};
use std::path::PathBuf;

const SEEDS: [u64; 8] = [1, 2, 3, 7, 42, 99, 123, 777];

fn usage() -> ! {
    eprintln!(
        "usage: hg-selfmod <gate0|gate1|gate2|gate42|gate43|gate44|gates|phaseA|phaseB> \
         [--config=FILE] [--out=DIR] [--key=value ...]\n\
         keys: seed steps log_every economy(stock|flux) total e0 birth_cost rent fire_reward\n\
               repro_threshold influx free_cap mu p_dup p_del abio p_meta rule_cap grace_steps\n\
               world_ceiling edge_ceiling creation_budget_init creation_budget_max\n\
               creation_refill match_cap_per_rule match_enum_budget wl_radius seed_pop"
    );
    std::process::exit(2)
}

fn apply_override(cfg: &mut RunConfig, key: &str, val: &str) -> bool {
    macro_rules! set {
        ($field:ident) => {
            match val.parse() {
                Ok(v) => {
                    cfg.$field = v;
                    true
                }
                Err(_) => false,
            }
        };
    }
    match key {
        "seed" => set!(seed),
        "steps" => set!(steps),
        "log_every" => set!(log_every),
        "economy" => match val.parse::<EconomyMode>() {
            Ok(m) => {
                cfg.economy = m;
                true
            }
            Err(_) => false,
        },
        "total" => set!(total),
        "e0" => set!(e0),
        "birth_cost" => set!(birth_cost),
        "rent" => set!(rent),
        "fire_reward" => set!(fire_reward),
        "repro_threshold" => set!(repro_threshold),
        "influx" => set!(influx),
        "free_cap" => set!(free_cap),
        "mu" => set!(mu),
        "p_dup" => set!(p_dup),
        "p_del" => set!(p_del),
        "abio" => set!(abio),
        "p_meta" => set!(p_meta),
        "rule_cap" => set!(rule_cap),
        "grace_steps" => set!(grace_steps),
        "world_ceiling" => set!(world_ceiling),
        "edge_ceiling" => set!(edge_ceiling),
        "creation_budget_init" => set!(creation_budget_init),
        "creation_budget_max" => set!(creation_budget_max),
        "creation_refill" => set!(creation_refill),
        "world_nodes" => set!(world_nodes),
        "world_edges" => set!(world_edges),
        "match_cap_per_rule" => set!(match_cap_per_rule),
        "truncation_mode" => match val.parse::<TruncationMode>() {
            Ok(m) => {
                cfg.truncation_mode = m;
                true
            }
            Err(_) => false,
        },
        "match_enum_budget" => set!(match_enum_budget),
        "wl_radius" => set!(wl_radius),
        "seed_pop" => {
            cfg.seed_pop = val.to_string();
            true
        }
        "income_mode" => match val.parse::<IncomeMode>() {
            Ok(m) => {
                cfg.income_mode = m;
                true
            }
            Err(_) => false,
        },
        "income_pool" => set!(income_pool),
        "match_cost_coeff" => set!(match_cost_coeff),
        "fission_mode" => match val.parse::<FissionMode>() {
            Ok(m) => {
                cfg.fission_mode = m;
                true
            }
            Err(_) => false,
        },
        "split_floor" => set!(split_floor),
        "partial_cost" => set!(partial_cost),
        "refractory_steps" => set!(refractory_steps),
        "copy_number" => set!(copy_number),
        "forced_dup_step" => set!(forced_dup_step),
        "forced_dup_members" => {
            cfg.forced_dup_members = val.to_string();
            true
        }
        "invasion_step" => set!(invasion_step),
        "invader_kind" => {
            cfg.invader_kind = val.to_string();
            true
        }
        "invader_theta" => set!(invader_theta),
        "invader_priority" => set!(invader_priority),
        "substitute_member" => set!(substitute_member),
        "tie_break" => match val.parse::<TieBreak>() {
            Ok(m) => {
                cfg.tie_break = m;
                true
            }
            Err(_) => false,
        },
        "jitter_eps" => set!(jitter_eps),
        "merit_decay" => set!(merit_decay),
        "merit_base" => set!(merit_base),
        _ => false,
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let Some(mode) = args.get(1) else { usage() };

    let mut cfg = RunConfig::default();
    let mut out_dir: Option<PathBuf> = None;
    for arg in &args[2..] {
        if let Some(rest) = arg.strip_prefix("--config=") {
            let text = std::fs::read_to_string(rest).unwrap_or_else(|e| {
                eprintln!("cannot read config {rest}: {e}");
                std::process::exit(2);
            });
            cfg = toml::from_str(&text).unwrap_or_else(|e| {
                eprintln!("bad config {rest}: {e}");
                std::process::exit(2);
            });
        } else if let Some(rest) = arg.strip_prefix("--out=") {
            out_dir = Some(PathBuf::from(rest));
        } else if let Some(rest) = arg.strip_prefix("--") {
            let Some((k, v)) = rest.split_once('=') else {
                usage()
            };
            if !apply_override(&mut cfg, k, v) {
                eprintln!("unknown or unparsable override --{rest}");
                usage();
            }
        } else {
            usage();
        }
    }

    match mode.as_str() {
        "gate0" => {
            if !gates::gate0() {
                std::process::exit(1);
            }
        }
        "gate1" => {
            if !gates::gate1(&[7, 42, 999], 100) {
                std::process::exit(1);
            }
        }
        "gate2" => {
            let steps = if cfg.steps == RunConfig::default().steps {
                10000
            } else {
                cfg.steps
            };
            if !gates::gate2(cfg.seed, steps) {
                std::process::exit(1);
            }
        }
        "gate42" => {
            if !gates::gate4_2() {
                std::process::exit(1);
            }
        }
        "gate43" => {
            let steps = if cfg.steps == RunConfig::default().steps {
                1000
            } else {
                cfg.steps
            };
            if gates::gate4_3(&SEEDS, steps).is_none() {
                std::process::exit(1);
            }
        }
        "gate51" => {
            if !gates::gate5_1(
                &SEEDS,
                if cfg.steps == RunConfig::default().steps {
                    2000
                } else {
                    cfg.steps
                },
            ) {
                std::process::exit(1);
            }
        }
        "gate52" => {
            let st = if cfg.steps == RunConfig::default().steps {
                2000
            } else {
                cfg.steps
            };
            if !gates::gate5_2(&SEEDS, st, cfg.income_pool, cfg.match_cost_coeff) {
                std::process::exit(1);
            }
        }
        "gate53diag" => {
            gates::gate5_3_diagnose(
                cfg.seed,
                if cfg.steps == RunConfig::default().steps {
                    1400
                } else {
                    cfg.steps
                },
                if cfg.invasion_step == 0 {
                    500
                } else {
                    cfg.invasion_step
                },
            );
        }
        "p0c" => {
            let st = if cfg.steps == RunConfig::default().steps {
                2000
            } else {
                cfg.steps
            };
            let inv = if cfg.invasion_step == 0 {
                500
            } else {
                cfg.invasion_step
            };
            let mut ok = false;
            let _ = &cfg;
            for (kind, theta) in [("guard", 0.1), ("guard", 0.25), ("v5", 0.0)] {
                ok |= gates::p0c_invasion(
                    &SEEDS,
                    st,
                    inv,
                    kind,
                    theta,
                    cfg.match_cost_coeff,
                    cfg.tie_break,
                );
                println!();
            }
            if !ok {
                std::process::exit(1);
            }
        }
        // Single-arm P0c, driven entirely from the config: robustness checks
        // over horizon and seed count.
        "p0c1" => {
            let st = if cfg.steps == RunConfig::default().steps {
                2000
            } else {
                cfg.steps
            };
            let inv = if cfg.invasion_step == 0 {
                500
            } else {
                cfg.invasion_step
            };
            let n = if cfg.copy_number > 1 {
                cfg.copy_number.min(24)
            } else {
                8
            };
            gates::p0c_invasion(
                &gates::P0A_SEEDS[..n],
                st,
                inv,
                &cfg.invader_kind,
                cfg.invader_theta,
                cfg.match_cost_coeff,
                cfg.tie_break,
            );
        }
        "orderprobe" => {
            gates::v6_rule_order_probe(&[
                0.01, 0.02, 0.05, 0.08, 0.1, 0.15, 0.2, 0.25, 0.5, 0.75, 0.9,
            ]);
        }
        // v8: establishment-probability gate. `copy_number` doubles as the
        // trial count here (default 24 when left at 1).
        "est8" => {
            let trials = if cfg.copy_number > 1 {
                cfg.copy_number
            } else {
                24
            };
            gates::v8_establishment(
                trials,
                if cfg.steps == RunConfig::default().steps {
                    2000
                } else {
                    cfg.steps
                },
                if cfg.invasion_step == 0 {
                    500
                } else {
                    cfg.invasion_step
                },
                cfg.invader_theta,
                cfg.match_cost_coeff,
                cfg.tie_break,
                cfg.jitter_eps,
                cfg.merit_decay,
                cfg.merit_base,
            );
        }
        "gate62" => {
            let st = if cfg.steps == RunConfig::default().steps {
                2000
            } else {
                cfg.steps
            };
            let inv = if cfg.invasion_step == 0 {
                500
            } else {
                cfg.invasion_step
            };
            let thetas = [0.01, 0.02, 0.05, 0.08, 0.1, 0.15, 0.2, 0.25, 0.5, 0.75, 0.9];
            if gates::gate6_2(
                &gates::P0A_SEEDS[..16],
                st,
                &thetas,
                inv,
                cfg.tie_break,
                cfg.match_cost_coeff,
            )
            .is_none()
            {
                std::process::exit(1);
            }
        }
        "gate61" => {
            let st = if cfg.steps == RunConfig::default().steps {
                400
            } else {
                cfg.steps
            };
            if !gates::gate6_1(&SEEDS[..3], st) {
                std::process::exit(1);
            }
        }
        "p0a" => {
            let st = if cfg.steps == RunConfig::default().steps {
                20000
            } else {
                cfg.steps
            };
            if gates::p0a_basin(&gates::P0A_SEEDS, st).len() < 8 {
                std::process::exit(1);
            }
        }
        "p1" => {
            let st = if cfg.steps == RunConfig::default().steps {
                20000
            } else {
                cfg.steps
            };
            gates::p1_object_only(&SEEDS, st.min(10000));
            gates::p1_contention(&SEEDS, st);
        }
        "p0b" => {
            let st = if cfg.steps == RunConfig::default().steps {
                2000
            } else {
                cfg.steps
            };
            let pop = if cfg.seed_pop == RunConfig::default().seed_pop {
                "churn3".to_string()
            } else {
                cfg.seed_pop.clone()
            };
            gates::p0b_fate_report(
                &SEEDS,
                st,
                &pop,
                cfg.match_cap_per_rule,
                cfg.truncation_mode,
            );
        }
        "gate44" => {
            let steps = if cfg.steps == RunConfig::default().steps {
                10000
            } else {
                cfg.steps
            };
            if !gates::gate4_4(steps) {
                std::process::exit(1);
            }
        }
        // G4.0: the v3 gates must still pass after the v4 changes.
        "gates" => {
            let mut ok = gates::gate0();
            ok &= gates::gate1(&[7, 42, 999], 100);
            ok &= gates::gate2(cfg.seed, 10000);
            ok &= gates::gate4_2();
            if !ok {
                std::process::exit(1);
            }
            println!("G4.0 PASS: v3 gates G0/G1/G2 still hold under the v4 changes");
        }
        "phaseA" | "phaseB" => {
            let phase = if mode == "phaseA" { 'A' } else { 'B' };
            if phase == 'B' {
                // Genesis stays OFF in Phase B: reproduction must be the only
                // birth channel, or it isn't selection.
                cfg.abio = 0.0;
                cfg.p_dup = 0.0;
                cfg.p_del = 0.0;
            }
            let res = run_phase(phase, &cfg);
            let dir = out_dir.unwrap_or_else(|| {
                PathBuf::from(format!("crates/hg-selfmod/runs/{}_seed{}", mode, cfg.seed))
            });
            write_run(&dir, &cfg, phase, &res).expect("write run outputs");
            println!(
                "PHASE {} seed={} economy={:?} mu={} pop={} | term={} survived={} rules={} \
                 M_rules={} M_data={} slopes=({:.2e},{:.2e}) t_ff={} t_freeze={} audit={}",
                phase,
                cfg.seed,
                cfg.economy,
                cfg.mu,
                cfg.seed_pop,
                res.term,
                res.survived,
                res.final_rules,
                res.m_rules,
                res.m_data,
                res.slope_m_rules,
                res.slope_m_data,
                res.time_to_first_fission
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "none".into()),
                res.time_to_freeze
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "none".into()),
                res.audit_residual,
            );
        }
        _ => usage(),
    }
}
