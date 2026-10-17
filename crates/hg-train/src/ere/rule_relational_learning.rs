//! Capacity-matched graph-aware/blind bounded learning. No broad admission.
use super::{rule_choice as softmax, rule_choice_learning::DrawStream, rule_relational as graph};
use hg_sim_core::ere::rule_relational::{self as native, Arm};
use serde::Serialize;
use serde_json::{json, Value};
use std::{
    path::Path,
    time::{Duration, Instant},
};

pub const PROFILE: &str = "E2-rule-relational-learning-v1";
pub const POLICY_SEEDS: [u64; 3] = [993748719001, 993748719002, 993748719003];
pub const DEVELOPMENT: [[u32; 2]; 6] = [[10, 21], [10, 31], [20, 11], [20, 31], [30, 11], [30, 21]];
pub const RESERVED: [[u32; 2]; 3] = [[10, 11], [20, 21], [30, 31]];
pub const CELLS: usize = 1782;

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Graph,
    Blind,
}
#[derive(Clone, Debug, Serialize)]
pub struct Input {
    mode: Mode,
    features: [[f64; 3]; 2],
}
pub fn input(mode: Mode, observation: &graph::Observation) -> Input {
    Input {
        mode,
        features: if mode == Mode::Graph {
            observation.features()
        } else {
            [[1. / 3.; 3]; 2]
        },
    }
}
fn logits(theta: &[f64], o: &Input, slot: usize) -> Result<[f64; 12], String> {
    if theta.len() != 16 || theta.iter().any(|t| !t.is_finite()) {
        return Err("requires sixteen finite parameters".into());
    }
    let mut t = [0.; 12];
    for a in 0..4 {
        t[a] = theta[a]
            + (0..3)
                .map(|k| theta[4 + a * 3 + k] * o.features[slot][k])
                .sum::<f64>();
    }
    softmax::probabilities(&t, 10)?;
    Ok(t)
}
pub fn probabilities(theta: &[f64], o: &Input) -> Result<[[f64; 4]; 2], String> {
    Ok([
        softmax::probabilities(&logits(theta, o, 0)?, 10)?,
        softmax::probabilities(&logits(theta, o, 1)?, 10)?,
    ])
}
pub fn sample(theta: &[f64], o: &Input, draws: [f64; 2]) -> Result<[usize; 2], String> {
    Ok([
        softmax::sample(&logits(theta, o, 0)?, 10, draws[0])?,
        softmax::sample(&logits(theta, o, 1)?, 10, draws[1])?,
    ])
}
pub fn log_probability(theta: &[f64], o: &Input, actions: [usize; 2]) -> Result<f64, String> {
    let joint = softmax::log_probability(&logits(theta, o, 0)?, 10, actions[0])?
        + softmax::log_probability(&logits(theta, o, 1)?, 10, actions[1])?;
    if !joint.is_finite() {
        return Err("joint likelihood overflow".into());
    }
    Ok(joint)
}
pub fn score_gradient(theta: &[f64], o: &Input, actions: [usize; 2]) -> Result<[f64; 16], String> {
    log_probability(theta, o, actions)?;
    let p = probabilities(theta, o)?;
    let mut g = [0.; 16];
    for slot in 0..2 {
        for a in 0..4 {
            let d = (if a == actions[slot] { 1. } else { 0. }) - p[slot][a];
            g[a] += d;
            for k in 0..3 {
                g[4 + a * 3 + k] += d * o.features[slot][k];
            }
        }
    }
    Ok(g)
}
#[derive(Clone, Debug)]
pub struct Sample {
    pub context: [u32; 2],
    pub input: Input,
    pub actions: [usize; 2],
    pub reward: f64,
}
pub fn update(mode: Mode, theta: &[f64], samples: &[Sample]) -> Result<Vec<f64>, String> {
    if samples.len() != 12
        || DEVELOPMENT
            .iter()
            .any(|c| samples.iter().filter(|s| s.context == *c).count() != 2)
    {
        return Err("requires two samples per development world; no reserved updates".into());
    }
    let mut direction = [0.; 16];
    for s in samples {
        if s.input.mode != mode || !s.reward.is_finite() || !(0.0..=1.0).contains(&s.reward) {
            return Err("invalid mode/reward".into());
        }
        let g = score_gradient(theta, &s.input, s.actions)?;
        for i in 0..16 {
            direction[i] += 4. * s.reward * g[i] / 12.;
        }
    }
    let next: Vec<_> = theta.iter().zip(direction).map(|(t, g)| t + g).collect();
    probabilities(&next, &samples[0].input)?;
    Ok(next)
}
fn save(path: &Path, v: &Value) -> Result<(), String> {
    std::fs::write(path, serde_json::to_vec(v).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}
pub fn endpoint(mode: Mode, theta: &[f64], rewards: &[f64]) -> Result<Value, String> {
    if rewards.len() != 144
        || rewards
            .iter()
            .any(|r| !r.is_finite() || !(0.0..=1.0).contains(r))
    {
        return Err("incomplete/invalid endpoint".into());
    }
    let mut rows = vec![];
    let mut mean = 0.;
    let mut reserved = 0.;
    for (i, c) in native::CONTEXTS.into_iter().enumerate() {
        let o = graph::observe(&native::initial_observation(c)?)?;
        let inp = input(mode, &o);
        let p = probabilities(theta, &inp)?;
        let mut expected = 0.;
        for a in 0..4 {
            for b in 0..4 {
                expected += p[0][a] * p[1][b] * rewards[i * 16 + a * 4 + b];
            }
        }
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
        rows.push(json!({"context":c,"observation":o,"input":inp,"probabilities":p,"correct_joint_probability":p[0][correct[0]]*p[1][correct[1]],"expected_reward":expected,"reserved":RESERVED.contains(&c)}));
        mean += expected / 9.;
        if RESERVED.contains(&c) {
            reserved += expected / 3.;
        }
    }
    Ok(
        json!({"mode":mode,"theta":theta,"contexts":rows,"balanced_expected_reward":mean,"reserved_expected_reward":reserved}),
    )
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
        decision: Option<Value>,
    ) -> Result<Option<f64>, String> {
        let id = self.index;
        self.index += 1;
        let seed = native::SEEDS[(id / 2) % 2];
        let mut wrapper = json!({"id":id,"phase":phase,"policy_seed":policy_seed,"context":context,"actions":actions,"native_seed":seed,"decision":decision});
        // Freeze emission before Engine; no outcome may select or alter this action.
        if decision.is_some() {
            save(&self.out.join(format!("decision-{id:04}.json")), &wrapper)?;
        }
        let mut row = match native::run(arm, context, actions, seed, self.deadline) {
            Ok(r) => r,
            Err(e) => {
                self.failures.push(format!("cell{id}: {e}"));
                json!({"error":e,"partial_native_work_unknown":true})
            }
        };
        let reward = if row.get("error").is_none() {
            match graph::observe(&row["initial_inventory"]) {
                Ok(o) => row["encoded_observation"] = json!(o),
                Err(e) => {
                    self.failures.push(e.clone());
                    row["observation_error"] = json!(e);
                }
            }
            match graph::reward(&row) {
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
        if let Some(d) = &decision {
            if row["encoded_observation"] != d["observation"] {
                self.failures.push(format!(
                    "cell{id}: pre-emission observation differs from native inventory"
                ));
                row["decision_observation_error"] = json!(true);
            }
        }
        let accepted = reward.filter(|_| {
            row.get("observation_error").is_none()
                && row.get("decision_observation_error").is_none()
        });
        wrapper["native"] = row;
        save(&self.out.join(format!("cell-{id:04}.json")), &wrapper)?;
        Ok(accepted)
    }
    fn evaluation(&mut self, phase: &str, seed: u64) -> Result<Option<Vec<f64>>, String> {
        let mut rewards = vec![];
        let mut valid = true;
        for c in native::CONTEXTS {
            for a in 0..4 {
                for b in 0..4 {
                    match self.cell(phase, seed, c, Arm::LivePaid, [a, b], None)? {
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
    std::fs::create_dir(out).map_err(|e| format!("fresh output required: {e}"))?;
    let design: Value = serde_json::from_str(include_str!(
        "../../../../configs/ere_rule_relational_learning_v1.json"
    ))
    .map_err(|e| e.to_string())?;
    save(
        &out.join("manifest.json"),
        &json!({"status":"running","profile":PROFILE,"design":design,"scientific_acceptance":false}),
    )?;
    std::fs::write(
        out.join("preregistration.md"),
        include_str!("../../../../docs/ere_rule_relational_learning_v1.md"),
    )
    .map_err(|e| e.to_string())?;
    if std::env::var_os("ERE_RULE_RELATIONAL_LEARNING_OUTPUT").is_some() {
        std::fs::copy(
            std::env::current_exe().map_err(|e| e.to_string())?,
            out.join("executed_test_binary"),
        )
        .map_err(|e| e.to_string())?;
    }
    let mut trial = Trial {
        out,
        deadline: Instant::now() + Duration::from_secs(420),
        index: 0,
        failures: vec![],
    };
    let mut transitions = vec![];
    let mut comparisons = vec![];
    let mut updates = 0;
    for seed in POLICY_SEEDS {
        let mut graph_theta = vec![0.; 16];
        let mut blind_theta = graph_theta.clone();
        let baseline = trial.evaluation("baseline", seed)?;
        let mut stream = DrawStream(seed);
        for iteration in 0..12 {
            let before_graph = graph_theta.clone();
            let before_blind = blind_theta.clone();
            let start = trial.index;
            let mut graph_samples = vec![];
            let mut blind_samples = vec![];
            let mut drawn = vec![];
            for c in DEVELOPMENT {
                for _ in 0..2 {
                    let o = graph::observe(&native::initial_observation(c)?)?;
                    let d0 = stream.next_draw();
                    let s0 = stream.0;
                    let d1 = stream.next_draw();
                    let s1 = stream.0;
                    let draws = [d0, d1];
                    drawn.push(json!({"draws":draws,"states":[s0,s1]}));
                    for (mode, t, samples) in [
                        (Mode::Graph, &graph_theta, &mut graph_samples),
                        (Mode::Blind, &blind_theta, &mut blind_samples),
                    ] {
                        let inp = input(mode, &o);
                        let actions = sample(t, &inp, draws)?;
                        let d = json!({"mode":mode,"iteration":iteration,"observation":o,"input":inp,"draws":draws,"draw_states":[s0,s1],"theta":t,"probabilities":probabilities(t,&inp)?,"log_probability":log_probability(t,&inp,actions)?,"score_gradient":score_gradient(t,&inp,actions)?.to_vec()});
                        if let Some(r) =
                            trial.cell("training", seed, c, Arm::LivePaid, actions, Some(d))?
                        {
                            samples.push(Sample {
                                context: c,
                                input: inp,
                                actions,
                                reward: r,
                            });
                        }
                    }
                }
            }
            let complete = graph_samples.len() == 12 && blind_samples.len() == 12;
            if complete {
                graph_theta = update(Mode::Graph, &graph_theta, &graph_samples)?;
                blind_theta = update(Mode::Blind, &blind_theta, &blind_samples)?;
                updates += 2;
            }
            transitions.push(json!({"policy_seed":seed,"iteration":iteration,"first_cell":start,"last_cell":trial.index-1,"draws":drawn,
                "before_graph":before_graph,"before_blind":before_blind,"after_graph":graph_theta,"after_blind":blind_theta,"skipped":!complete}));
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
                let reward = trial.cell("control", seed, c, arm, correct, None)?;
                controls.push(json!({"context":c,"arm":arm,"reward":reward}));
            }
        }
        let comparison = match (baseline, finals) {
            (Some(base), Some(finals)) => {
                let before = endpoint(Mode::Graph, &[0.; 16], &base)?;
                let after = endpoint(Mode::Graph, &graph_theta, &finals)?;
                let blind = endpoint(Mode::Blind, &blind_theta, &finals)?;
                let frozen = endpoint(Mode::Graph, &[0.; 16], &finals)?;
                let balanced = after["balanced_expected_reward"].as_f64().unwrap();
                let blind_balanced = blind["balanced_expected_reward"].as_f64().unwrap();
                let reserved_gain = after["reserved_expected_reward"].as_f64().unwrap()
                    - before["reserved_expected_reward"].as_f64().unwrap();
                let controls_pass = controls.iter().all(|c| {
                    c["reward"].as_f64()
                        == Some(if c["arm"] == "reverse_paid" {
                            9. / 24.
                        } else {
                            0.
                        })
                });
                json!({"policy_seed":seed,"baseline":before,"final_graph":after,"final_blind":blind,"final_frozen":frozen,
                    "ceiling_margin":balanced-1./24.,"graph_minus_blind":balanced-blind_balanced,"reserved_gain":reserved_gain,
                    "ceiling_pass":balanced>=1./24.+0.01,"contrast_pass":balanced-blind_balanced>=0.01,"blind_bound_pass":blind_balanced<=1./24.+2e-12,
                    "reserved_gain_pass":reserved_gain>=0.01,"every_reserved_joint_probability_gain":after["contexts"].as_array().unwrap().iter().filter(|c|c["reserved"]==true).all(|c|c["correct_joint_probability"].as_f64().unwrap()>1./16.),
                    "frozen_unchanged":before==frozen,"native_matrix_unchanged":base==finals,"controls_pass":controls_pass,"controls":controls})
            }
            _ => json!({"policy_seed":seed,"invalid_endpoint":true,"controls":controls}),
        };
        comparisons.push(comparison);
        save(&out.join("comparisons.json"), &json!(comparisons))?;
    }
    let complete = trial.failures.is_empty() && trial.index == CELLS && updates == 72;
    let passed = complete
        && comparisons.iter().all(|c| {
            [
                "ceiling_pass",
                "contrast_pass",
                "blind_bound_pass",
                "reserved_gain_pass",
                "every_reserved_joint_probability_gain",
                "frozen_unchanged",
                "native_matrix_unchanged",
                "controls_pass",
            ]
            .iter()
            .all(|k| c[*k] == true)
        });
    let result = json!({"status":if complete{"native_complete"}else{"failed"},"profile":PROFILE,"design":design,"recorded_cells":trial.index,"completed_updates":updates,"failures":trial.failures,
        "native_attempts":if complete{json!(42768)}else{Value::Null},"native_work_unknown":!complete,"learning_criterion_pass":passed,
        "bounded_learning_authorized":true,"full_training_authorized":false,"training_admitted":false,"scientific_acceptance":false,"independent_physics_replay":false});
    save(&out.join("manifest.json"), &result)?;
    if !complete {
        return Err("invalid trial retained; no retry".into());
    }
    Ok(result)
}
