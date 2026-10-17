//! Fixed shared-policy composition trial. No broad training or science admission.
use super::{rule_choice_learning::DrawStream, rule_composition as adapter};
use hg_sim_core::ere::rule_composition::{self as native, Arm};
use serde_json::{json, Value};
use std::{
    path::Path,
    time::{Duration, Instant},
};

pub const PROFILE: &str = "E2-rule-composition-learning-v1";
pub const POLICY_SEEDS: [u64; 3] = [993748717001, 993748717002, 993748717003];
pub const DEVELOPMENT: [[u32; 2]; 6] = [[10, 21], [10, 31], [20, 11], [20, 31], [30, 11], [30, 21]];
pub const RESERVED: [[u32; 2]; 3] = [[10, 11], [20, 21], [30, 31]];
pub const CELLS: usize = 1206;
pub const UPDATES: usize = 12;

#[derive(Clone, Debug)]
pub struct Sample {
    pub context: [u32; 2],
    pub actions: [usize; 2],
    pub reward: f64,
}

pub fn update(theta: &[f64], samples: &[Sample]) -> Result<Vec<f64>, String> {
    adapter::probabilities(theta, DEVELOPMENT[0])?;
    if samples.len() != 12
        || DEVELOPMENT
            .iter()
            .any(|c| samples.iter().filter(|s| s.context == *c).count() != 2)
    {
        return Err("requires two samples per six development pairs; no reserved updates".into());
    }
    let mut direction = [0.; 12];
    for s in samples {
        if !s.reward.is_finite() || !(0.0..=1.0).contains(&s.reward) {
            return Err("invalid reward".into());
        }
        let g = adapter::score_gradient(theta, s.context, s.actions)?;
        for i in 0..12 {
            direction[i] += 4. * s.reward * g[i] / 12.;
        }
    }
    let next: Vec<_> = theta.iter().zip(direction).map(|(t, g)| t + g).collect();
    adapter::probabilities(&next, DEVELOPMENT[0])?;
    Ok(next)
}

fn save(path: &Path, v: &Value) -> Result<(), String> {
    std::fs::write(path, serde_json::to_vec(v).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}

pub fn endpoint(theta: &[f64], rewards: &[f64]) -> Result<Value, String> {
    if rewards.len() != 48
        || rewards
            .iter()
            .any(|r| !r.is_finite() || !(0.0..=1.0).contains(r))
    {
        return Err("incomplete/invalid reserved endpoint".into());
    }
    let mut rows = vec![];
    let mut mean = 0.;
    for (i, c) in RESERVED.into_iter().enumerate() {
        let p = adapter::probabilities(theta, c)?;
        let mut expected = 0.;
        for a in 0..4 {
            for b in 0..4 {
                expected += p[0][a] * p[1][b] * rewards[i * 16 + a * 4 + b];
            }
        }
        let correct = [
            native::CHOICES[0]
                .iter()
                .position(|x| *x == Some(c[0]))
                .unwrap(),
            native::CHOICES[1]
                .iter()
                .position(|x| *x == Some(c[1]))
                .unwrap(),
        ];
        rows.push(json!({"context":c,"probabilities":p,"correct_joint_probability":p[0][correct[0]]*p[1][correct[1]],"expected_reward":expected}));
        mean += expected / 3.;
    }
    Ok(json!({"theta":theta,"contexts":rows,"mean_expected_reward":mean}))
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
        context: [u32; 2],
        arm: Arm,
        actions: [usize; 2],
        policy: Option<&str>,
        iteration: Option<usize>,
        draws: Option<[(f64, u64); 2]>,
        theta: Option<&[f64]>,
    ) -> Result<Option<f64>, String> {
        let id = self.index;
        self.index += 1;
        let seed = native::SEEDS[(id / 2) % 2];
        let mut row = match native::run(arm, context, actions, seed, self.deadline) {
            Ok(r) => r,
            Err(e) => {
                self.failures.push(format!("cell{id}: {e}"));
                json!({"error":e,"partial_native_work_unknown":true})
            }
        };
        let reward = if row.get("error").is_none() {
            match adapter::reward(&row) {
                Ok(r) => {
                    row["adapter_reward"] = json!(r);
                    Some(r)
                }
                Err(e) => {
                    self.failures.push(format!("cell{id}: {e}"));
                    row["adapter_error"] = json!(e);
                    None
                }
            }
        } else {
            None
        };
        let mut wrapper = json!({"id":id,"phase":phase,"policy_seed":policy_seed,"context":context,
            "actions":actions,"native_seed":seed,"policy":policy,"iteration":iteration,
            "draws":draws.map(|d|d.map(|x|x.0)),"draw_states":draws.map(|d|d.map(|x|x.1)),"native":row});
        if let Some(t) = theta {
            wrapper["policy_theta"] = json!(t);
            wrapper["probabilities"] = json!(adapter::probabilities(t, context)?);
            wrapper["log_probability"] = json!(adapter::log_probability(t, context, actions)?);
            wrapper["score_gradient"] =
                json!(adapter::score_gradient(t, context, actions)?.to_vec());
        }
        save(&self.out.join(format!("cell-{id:04}.json")), &wrapper)?;
        Ok(reward)
    }
    fn evaluation(&mut self, phase: &str, seed: u64) -> Result<Option<Vec<f64>>, String> {
        let mut rewards = vec![];
        let mut valid = true;
        for c in RESERVED {
            for a in 0..4 {
                for b in 0..4 {
                    match self.cell(
                        phase,
                        seed,
                        c,
                        Arm::LivePaid,
                        [a, b],
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
        }
        Ok(valid.then_some(rewards))
    }
}

pub fn run(out: &Path) -> Result<Value, String> {
    std::fs::create_dir(out).map_err(|e| format!("fresh directory required: {e}"))?;
    let design: Value = serde_json::from_str(include_str!(
        "../../../../configs/ere_rule_composition_learning_v1.json"
    ))
    .map_err(|e| e.to_string())?;
    save(
        &out.join("manifest.json"),
        &json!({"status":"running","profile":PROFILE,"design":design,"scientific_acceptance":false}),
    )?;
    std::fs::write(
        out.join("preregistration.md"),
        include_str!("../../../../docs/ere_rule_composition_learning_v1.md"),
    )
    .map_err(|e| e.to_string())?;
    if std::env::var_os("ERE_RULE_COMPOSITION_LEARNING_OUTPUT").is_some() {
        std::fs::copy(
            std::env::current_exe().map_err(|e| e.to_string())?,
            out.join("executed_test_binary"),
        )
        .map_err(|e| e.to_string())?;
    }
    let mut trial = Trial {
        out,
        deadline: Instant::now() + Duration::from_secs(300),
        index: 0,
        failures: vec![],
    };
    let mut transitions = vec![];
    let mut comparisons = vec![];
    let mut completed_updates = 0;
    for seed in POLICY_SEEDS {
        let mut live = vec![0.; 12];
        let mut reuse = live.clone();
        let baseline = trial.evaluation("baseline", seed)?;
        let mut stream = DrawStream(seed);
        for iteration in 0..UPDATES {
            let before_live = live.clone();
            let before_reuse = reuse.clone();
            let start = trial.index;
            let mut live_samples = vec![];
            let mut reuse_samples = vec![];
            let mut drawn = vec![];
            for c in DEVELOPMENT {
                for _ in 0..2 {
                    let d0 = stream.next_draw();
                    let s0 = stream.0;
                    let d1 = stream.next_draw();
                    let s1 = stream.0;
                    let draws = [(d0, s0), (d1, s1)];
                    drawn.push(json!({"draws":[d0,d1],"states":[s0,s1]}));
                    for (name, arm, t, samples) in [
                        ("live", Arm::LivePaid, &live, &mut live_samples),
                        ("reuse", Arm::PreseededLure, &reuse, &mut reuse_samples),
                    ] {
                        let actions = adapter::sample(t, c, [d0, d1])?;
                        if let Some(r) = trial.cell(
                            "training",
                            seed,
                            c,
                            arm,
                            actions,
                            Some(name),
                            Some(iteration),
                            Some(draws),
                            Some(t),
                        )? {
                            samples.push(Sample {
                                context: c,
                                actions,
                                reward: r,
                            });
                        }
                    }
                }
            }
            let complete = live_samples.len() == 12 && reuse_samples.len() == 12;
            if complete {
                live = update(&live, &live_samples)?;
                reuse = update(&reuse, &reuse_samples)?;
                completed_updates += 2;
            }
            transitions.push(json!({"policy_seed":seed,"iteration":iteration,"first_cell":start,"last_cell":trial.index-1,
                "draws":drawn,"before_live":before_live,"before_reuse":before_reuse,"after_live":live,"after_reuse":reuse,"skipped":!complete}));
            save(&out.join("transitions.json"), &json!(transitions))?;
        }
        let finals = trial.evaluation("final", seed)?;
        let mut controls = vec![];
        for c in RESERVED {
            let correct = [
                native::CHOICES[0]
                    .iter()
                    .position(|a| *a == Some(c[0]))
                    .unwrap(),
                native::CHOICES[1]
                    .iter()
                    .position(|a| *a == Some(c[1]))
                    .unwrap(),
            ];
            for arm in native::ARMS[1..].iter().copied() {
                let r = trial.cell("control", seed, c, arm, correct, None, None, None, None)?;
                controls.push(json!({"context":c,"arm":arm,"reward":r}));
            }
        }
        let comparison = match (baseline, finals) {
            (Some(base), Some(finals)) => {
                let before = endpoint(&vec![0.; 12], &base)?;
                let after = endpoint(&live, &finals)?;
                let frozen = endpoint(&vec![0.; 12], &finals)?;
                let after_reuse = endpoint(&reuse, &finals)?;
                let gain = after["mean_expected_reward"].as_f64().unwrap()
                    - before["mean_expected_reward"].as_f64().unwrap();
                let controls_pass = controls.iter().all(|c| {
                    c["reward"].as_f64()
                        == Some(if c["arm"] == "reverse_paid" {
                            9. / 24.
                        } else {
                            0.
                        })
                });
                json!({"policy_seed":seed,"baseline":before,"final_live":after,"final_reuse":after_reuse,"final_frozen":frozen,
                    "expected_reward_gain":gain,"gain_pass":gain>=0.01,"every_reserved_joint_probability_gain":after["contexts"].as_array().unwrap().iter().all(|c|c["correct_joint_probability"].as_f64().unwrap()>1./16.),
                    "reuse_unchanged":reuse==vec![0.;12],"frozen_unchanged":before==frozen,"controls_pass":controls_pass,"controls":controls})
            }
            _ => json!({"policy_seed":seed,"invalid_endpoint":true,"controls":controls}),
        };
        comparisons.push(comparison);
        save(&out.join("comparisons.json"), &json!(comparisons))?;
    }
    let complete = trial.failures.is_empty() && trial.index == CELLS && completed_updates == 72;
    let passed = complete
        && comparisons.iter().all(|c| {
            c["gain_pass"] == true
                && c["every_reserved_joint_probability_gain"] == true
                && c["reuse_unchanged"] == true
                && c["frozen_unchanged"] == true
                && c["controls_pass"] == true
        });
    let result = json!({"status":if complete{"native_complete"}else{"failed"},"profile":PROFILE,"design":design,
        "recorded_cells":trial.index,"completed_updates":completed_updates,"failures":trial.failures,
        "native_attempts":if complete{json!(28944)}else{Value::Null},"native_work_unknown":!complete,
        "learning_criterion_pass":passed,"bounded_learning_authorized":true,"full_training_authorized":false,
        "training_admitted":false,"scientific_acceptance":false,"independent_physics_replay":false});
    save(&out.join("manifest.json"), &result)?;
    if !complete {
        return Err("invalid trial preserved; no retry".into());
    }
    Ok(result)
}
