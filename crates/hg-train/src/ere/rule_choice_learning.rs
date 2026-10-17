//! Fixed-budget exposed-world learning mechanism trial, not full training admission.
use super::rule_choice as adapter;
use hg_sim_core::ere::rule_choice::{self as native, Arm};
use serde_json::{json, Value};
use std::{
    path::Path,
    time::{Duration, Instant},
};

pub const PROFILE: &str = "E2-rule-choice-learning-v2";
pub const POLICY_SEEDS: [u64; 3] = [993748715001, 993748715002, 993748715003];
pub const UPDATES: usize = 12;
pub const SAMPLES: usize = 2;
pub const CELLS: usize = 540;

/// Explicit independent policy stream, never supplied to the physics engine.
pub struct DrawStream(pub u64);
impl DrawStream {
    pub fn next_draw(&mut self) -> f64 {
        self.0 = self.0.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^= z >> 31;
        (z >> 11) as f64 * (1. / 9007199254740992.)
    }
}

#[derive(Clone, Debug)]
pub struct Sample {
    pub context: u32,
    pub action: usize,
    pub reward: f64,
}

/// Exactly two samples per context. Baseline zero; fixed rate one; no clipping.
pub fn update(theta: &[f64], samples: &[Sample]) -> Result<Vec<f64>, String> {
    adapter::probabilities(theta, native::CONTEXTS[0])?;
    if samples.len() != 6 {
        return Err("requires complete six-sample batch".into());
    }
    for context in native::CONTEXTS {
        if samples.iter().filter(|s| s.context == context).count() != SAMPLES {
            return Err("unbalanced context denominator".into());
        }
    }
    let mut direction = [0.; adapter::PARAMS];
    for s in samples {
        if !s.reward.is_finite() || !(0.0..=1.0).contains(&s.reward) {
            return Err("invalid reward".into());
        }
        let gradient = adapter::score_gradient(theta, s.context, s.action)?;
        for i in 0..adapter::PARAMS {
            direction[i] += s.reward * gradient[i] / SAMPLES as f64;
        }
    }
    let next: Vec<_> = theta.iter().zip(direction).map(|(t, g)| t + g).collect();
    adapter::probabilities(&next, native::CONTEXTS[0])?;
    Ok(next)
}

fn save(path: &Path, value: &Value) -> Result<(), String> {
    let bytes = serde_json::to_vec(value).map_err(|e| e.to_string())?;
    std::fs::write(path, bytes).map_err(|e| e.to_string())
}

struct Trial<'a> {
    out: &'a Path,
    deadline: Instant,
    index: usize,
    failures: Vec<String>,
}
impl Trial<'_> {
    fn cell(
        &mut self,
        phase: &str,
        policy_seed: u64,
        context: u32,
        arm: Arm,
        action: usize,
        policy: Option<&str>,
        iteration: Option<usize>,
        draw: Option<(f64, u64)>,
        theta: Option<&[f64]>,
    ) -> Result<Option<f64>, String> {
        let index = self.index;
        self.index += 1;
        let seed = native::SEEDS[index % native::SEEDS.len()];
        let mut row = match native::run(arm, context, action, seed, self.deadline) {
            Ok(row) => row,
            Err(error) => {
                self.failures.push(format!("cell {index}: {error}"));
                json!({"seed":seed,"context":context,"action":action,"arm":arm,"error":error,
                    "partial_native_work_unknown":true})
            }
        };
        let reward = if row.get("error").is_none() {
            match adapter::reward(&row) {
                Ok(r) => {
                    row["adapter_reward"] = json!(r);
                    Some(r)
                }
                Err(error) => {
                    self.failures.push(format!("cell {index}: {error}"));
                    row["adapter_error"] = json!(error);
                    None
                }
            }
        } else {
            None
        };
        let mut wrapper = json!({"id":index,"phase":phase,"policy_seed":policy_seed,
            "context":context,"action":action,"native_seed":seed,"policy":policy,
            "iteration":iteration,"draw":draw.map(|x|x.0),"draw_state":draw.map(|x|x.1),"native":row});
        if let Some(t) = theta {
            wrapper["policy_theta"] = json!(t);
            wrapper["probabilities"] = json!(adapter::probabilities(t, context)?);
            wrapper["log_probability"] = json!(adapter::log_probability(t, context, action)?);
            wrapper["score_gradient"] =
                json!(adapter::score_gradient(t, context, action)?.to_vec());
        }
        // Persist the native ledger before its reward can enter an update.
        save(&self.out.join(format!("cell-{index:04}.json")), &wrapper)?;
        Ok(reward)
    }

    fn evaluation(&mut self, phase: &str, seed: u64) -> Result<Option<Vec<f64>>, String> {
        let mut rewards = vec![];
        let mut valid = true;
        for context in native::CONTEXTS {
            for action in 0..4 {
                match self.cell(
                    phase,
                    seed,
                    context,
                    Arm::LivePaid,
                    action,
                    None,
                    None,
                    None,
                    None,
                )? {
                    Some(r) => rewards.push(r),
                    None => valid = false,
                }
            }
        }
        Ok(valid.then_some(rewards))
    }
}

fn endpoint(theta: &[f64], rewards: &[f64]) -> Result<Value, String> {
    if rewards.len() != 12 {
        return Err("incomplete endpoint panel".into());
    }
    let mut expected = 0.;
    let mut contexts = vec![];
    for (i, context) in native::CONTEXTS.into_iter().enumerate() {
        let p = adapter::probabilities(theta, context)?;
        let conditional: f64 = p
            .iter()
            .zip(&rewards[i * 4..i * 4 + 4])
            .map(|(p, r)| p * r)
            .sum();
        let correct = native::ACTIONS
            .iter()
            .position(|a| *a == Some(context))
            .unwrap();
        contexts.push(json!({"context":context,"probabilities":p,
            "matching_edit_probability":p[correct],"expected_reward":conditional}));
        expected += conditional / 3.;
    }
    Ok(json!({"theta":theta,"contexts":contexts,"mean_expected_reward":expected}))
}

/// Only this exact frozen trial is runnable. Output must be a fresh directory.
pub fn run(out: &Path) -> Result<Value, String> {
    std::fs::create_dir(out).map_err(|e| format!("fresh trial directory required: {e}"))?;
    let design: Value = serde_json::from_str(include_str!(
        "../../../../configs/ere_rule_choice_learning_v2.json"
    ))
    .map_err(|e| e.to_string())?;
    save(
        &out.join("manifest.json"),
        &json!({"status":"running","profile":PROFILE,"design":design,
        "bounded_learning_authorized":true,"full_training_authorized":false,"scientific_acceptance":false}),
    )?;
    std::fs::write(
        out.join("preregistration.md"),
        include_str!("../../../../docs/ere_rule_choice_learning_v2.md"),
    )
    .map_err(|e| e.to_string())?;
    if std::env::var_os("ERE_RULE_CHOICE_LEARNING_OUTPUT").is_some() {
        std::fs::copy(
            std::env::current_exe().map_err(|e| e.to_string())?,
            out.join("executed_test_binary"),
        )
        .map_err(|e| e.to_string())?;
    }
    let mut trial = Trial {
        out,
        deadline: Instant::now() + Duration::from_secs(180),
        index: 0,
        failures: vec![],
    };
    let mut transitions = vec![];
    let mut comparisons = vec![];
    let mut control_rewards = vec![];
    let mut completed_updates = 0;
    for seed in POLICY_SEEDS {
        let mut live = vec![0.; adapter::PARAMS];
        let mut reuse = live.clone();
        let baseline = trial.evaluation("baseline", seed)?;
        let mut stream = DrawStream(seed);
        for iteration in 0..UPDATES {
            let before_live = live.clone();
            let before_reuse = reuse.clone();
            let mut live_samples = vec![];
            let mut reuse_samples = vec![];
            let start = trial.index;
            let mut draws = vec![];
            for context in native::CONTEXTS {
                for _ in 0..SAMPLES {
                    let draw = stream.next_draw();
                    let state = stream.0;
                    draws.push(json!({"draw":draw,"state":state}));
                    for (policy, arm, theta, samples) in [
                        ("live", Arm::LivePaid, &live, &mut live_samples),
                        ("reuse", Arm::PreseededLure, &reuse, &mut reuse_samples),
                    ] {
                        let action = adapter::sample(theta, context, draw)?;
                        if let Some(reward) = trial.cell(
                            "training",
                            seed,
                            context,
                            arm,
                            action,
                            Some(policy),
                            Some(iteration),
                            Some((draw, state)),
                            Some(theta),
                        )? {
                            samples.push(Sample {
                                context,
                                action,
                                reward,
                            });
                        }
                    }
                }
            }
            let complete = live_samples.len() == 6 && reuse_samples.len() == 6;
            if complete {
                live = update(&live, &live_samples)?;
                reuse = update(&reuse, &reuse_samples)?;
                completed_updates += 2;
            }
            transitions.push(json!({"policy_seed":seed,"iteration":iteration,"first_cell":start,
                "last_cell":trial.index-1,"draws":draws,"before_live":before_live,"before_reuse":before_reuse,
                "after_live":live,"after_reuse":reuse,"skipped":!complete}));
            save(&out.join("transitions.json"), &json!(transitions))?;
        }
        let final_rewards = trial.evaluation("final", seed)?;
        let mut controls = vec![];
        for context in native::CONTEXTS {
            let correct = native::ACTIONS
                .iter()
                .position(|a| *a == Some(context))
                .unwrap();
            for arm in native::ARMS[1..].iter().copied() {
                let reward = trial.cell(
                    "control", seed, context, arm, correct, None, None, None, None,
                )?;
                controls.push(json!({"context":context,"arm":arm,"reward":reward}));
                control_rewards.push(reward);
            }
        }
        let comparison = match (baseline, final_rewards) {
            (Some(base), Some(finals)) => {
                let before = endpoint(&vec![0.; adapter::PARAMS], &base)?;
                let after = endpoint(&live, &finals)?;
                let after_reuse = endpoint(&reuse, &finals)?;
                let gain = after["mean_expected_reward"].as_f64().unwrap()
                    - before["mean_expected_reward"].as_f64().unwrap();
                let contexts_gain = after["contexts"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .all(|x| x["matching_edit_probability"].as_f64().unwrap() > 0.25);
                json!({"policy_seed":seed,"baseline":before,"final_live":after,"final_reuse":after_reuse,
                    "expected_reward_gain":gain,"every_context_probability_gain":contexts_gain,
                    "gain_pass":gain>=0.05,"reuse_unchanged":reuse==vec![0.;adapter::PARAMS],"controls":controls})
            }
            _ => json!({"policy_seed":seed,"invalid_endpoint":true,"controls":controls}),
        };
        comparisons.push(comparison);
        save(&out.join("comparisons.json"), &json!(comparisons))?;
    }
    let accounting_complete =
        trial.failures.is_empty() && trial.index == CELLS && completed_updates == 72;
    let learning_pass = accounting_complete
        && control_rewards.iter().all(|r| *r == Some(0.))
        && comparisons.iter().all(|r| {
            r["gain_pass"] == true
                && r["every_context_probability_gain"] == true
                && r["reuse_unchanged"] == true
        });
    let result = json!({"profile":PROFILE,"status":if accounting_complete{"native_complete"}else{"failed"},
        "design":design,"recorded_cells":trial.index,"failures":trial.failures,"completed_updates":completed_updates,
        "native_attempts":if accounting_complete{json!(12960)}else{Value::Null},
        "native_work_unknown":!accounting_complete,"learning_criterion_pass":learning_pass,
        "bounded_learning_authorized":true,"full_training_authorized":false,"training_admitted":false,
        "scientific_acceptance":false,"independent_physics_replay":false});
    save(&out.join("manifest.json"), &result)?;
    if !accounting_complete {
        return Err("trial invalid; retained outcomes, no retry".into());
    }
    Ok(result)
}
