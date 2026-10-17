//! One bounded raw-graph/masked native learning trial; negative is complete.
use super::{
    graph_encoder as encoder, rule_choice_learning::DrawStream,
    rule_relational as reward_interface, rule_relational_learning as old,
};
use encoder::{Mode, Sample};
use hg_sim_core::ere::rule_relational::{self as native, Arm};
use serde_json::{json, Value};
use std::{
    path::Path,
    time::{Duration, Instant},
};

pub const PROFILE: &str = "E2-graph-encoder-learning-v1";
pub const POLICY_SEEDS: [u64; 3] = [993748721001, 993748721002, 993748721003];
pub const CELLS: usize = 1782;
pub fn design() -> Value {
    serde_json::from_str(include_str!(
        "../../../../configs/ere_graph_encoder_learning_v1.json"
    ))
    .unwrap()
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
        return Err("invalid full endpoint".into());
    }
    let mut contexts = vec![];
    let mut balanced = 0.;
    let mut reserved = 0.;
    for (i, c) in native::CONTEXTS.into_iter().enumerate() {
        let observation = encoder::observe(&native::initial_observation(c)?)?;
        let input = encoder::input(mode, &observation);
        let p = encoder::probabilities(theta, &input)?;
        let expected = (0..4)
            .flat_map(|a| (0..4).map(move |b| p[0][a] * p[1][b] * rewards[i * 16 + a * 4 + b]))
            .sum::<f64>();
        let correct = std::array::from_fn::<_, 2, _>(|s| {
            native::CHOICES[s]
                .iter()
                .position(|v| *v == Some(c[s]))
                .unwrap()
        });
        let is_reserved = old::RESERVED.contains(&c);
        balanced += expected / 9.;
        if is_reserved {
            reserved += expected / 3.;
        }
        contexts.push(json!({"context":c,"observation":observation,"input":input,"probabilities":p,"correct_joint_probability":p[0][correct[0]]*p[1][correct[1]],"expected_reward":expected,"reserved":is_reserved}));
    }
    Ok(
        json!({"mode":mode,"theta":theta,"contexts":contexts,"balanced_expected_reward":balanced,"reserved_expected_reward":reserved}),
    )
}
pub fn comparison(
    seed: u64,
    index: usize,
    graph: &[f64],
    blind: &[f64],
    base: &[f64],
    finals: &[f64],
    controls: Value,
) -> Result<Value, String> {
    let initial = encoder::initialize();
    let before = endpoint(Mode::Graph, &initial, base)?;
    let before_blind = endpoint(Mode::Blind, &initial, base)?;
    let after = endpoint(Mode::Graph, graph, finals)?;
    let blind_end = endpoint(Mode::Blind, blind, finals)?;
    let frozen = endpoint(Mode::Graph, &initial, finals)?;
    let frozen_blind = endpoint(Mode::Blind, &initial, finals)?;
    let uniform_base = endpoint(Mode::Graph, &[0.; 110], base)?;
    let uniform_final = endpoint(Mode::Graph, &[0.; 110], finals)?;
    let control_theta: Vec<f64> =
        serde_json::from_value(design()["positive_control_thetas"][index].clone())
            .map_err(|e| e.to_string())?;
    let positive_base = old::endpoint(old::Mode::Graph, &control_theta, base)?;
    let positive_final = old::endpoint(old::Mode::Graph, &control_theta, finals)?;
    let g = after["balanced_expected_reward"].as_f64().unwrap();
    let b = blind_end["balanced_expected_reward"].as_f64().unwrap();
    let gain = after["reserved_expected_reward"].as_f64().unwrap()
        - before["reserved_expected_reward"].as_f64().unwrap();
    let every = after["contexts"]
        .as_array()
        .unwrap()
        .iter()
        .zip(before["contexts"].as_array().unwrap())
        .filter(|(c, _)| c["reserved"] == true)
        .all(|(c, o)| {
            c["correct_joint_probability"].as_f64().unwrap()
                > o["correct_joint_probability"].as_f64().unwrap()
        });
    let controls_pass = controls
        .as_array()
        .ok_or("missing controls")?
        .iter()
        .all(|c| {
            c["reward"].as_f64()
                == Some(if c["arm"] == "reverse_paid" {
                    9. / 24.
                } else {
                    0.
                })
        });
    Ok(
        json!({"policy_seed":seed,"baseline":before,"baseline_blind":before_blind,"final_graph":after,"final_blind":blind_end,"final_frozen":frozen,"final_frozen_blind":frozen_blind,"uniform_base":uniform_base,"uniform_final":uniform_final,"positive_control_base":positive_base,"positive_control_final":positive_final,
        "ceiling_margin":g-1./24.,"graph_minus_blind":g-b,"reserved_gain":gain,"ceiling_pass":g>=1./24.+0.01,"contrast_pass":g-b>=0.01,"blind_bound_pass":b<=1./24.+2e-12,"reserved_gain_pass":gain>=0.01,"every_reserved_joint_probability_gain":every,
        "frozen_unchanged":before==frozen && before_blind==frozen_blind,"uniform_unchanged":uniform_base==uniform_final,"native_matrix_unchanged":base==finals,"positive_control_unchanged":positive_base==positive_final,"positive_control_pass":positive_final["balanced_expected_reward"].as_f64().unwrap()>=1./24.+0.01,"controls_pass":controls_pass,"controls":controls}),
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
        let mut accepted = None;
        if row.get("error").is_none() {
            match (
                reward_interface::observe(&row["initial_inventory"]),
                encoder::observe(&row["initial_inventory"]),
            ) {
                (Ok(hand), Ok(raw)) => {
                    row["encoded_observation"] = json!(hand);
                    row["raw_graph_observation"] = json!(raw);
                }
                _ => {
                    self.failures.push(format!("cell{id}: invalid observation"));
                    row["observation_error"] = json!(true);
                }
            }
            match reward_interface::reward(&row) {
                Ok(r) => {
                    row["adapter_reward"] = json!(r);
                    accepted = Some(r);
                }
                Err(e) => {
                    self.failures.push(format!("cell{id}: {e}"));
                    row["adapter_error"] = json!(e);
                }
            }
        }
        if let Some(d) = &decision {
            if row["raw_graph_observation"] != d["observation"] {
                self.failures
                    .push(format!("cell{id}: decision observation mismatch"));
                row["decision_observation_error"] = json!(true);
            }
        }
        if row.get("observation_error").is_some() || row.get("decision_observation_error").is_some()
        {
            accepted = None;
        }
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
    let design = design();
    save(
        &out.join("manifest.json"),
        &json!({"profile":PROFILE,"status":"running","design":design,"scientific_acceptance":false}),
    )?;
    std::fs::write(
        out.join("preregistration.md"),
        include_str!("../../../../docs/ere_graph_encoder_learning_v1.md"),
    )
    .map_err(|e| e.to_string())?;
    if std::env::var_os("ERE_GRAPH_ENCODER_LEARNING_OUTPUT").is_some() {
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
    for (si, seed) in POLICY_SEEDS.into_iter().enumerate() {
        let mut graph = encoder::initialize();
        let mut blind = graph.clone();
        let baseline = trial.evaluation("baseline", seed)?;
        let mut stream = DrawStream(seed);
        for iteration in 0..12 {
            let before_graph = graph.clone();
            let before_blind = blind.clone();
            let first = trial.index;
            let mut gs = vec![];
            let mut bs = vec![];
            let mut drawn = vec![];
            for c in old::DEVELOPMENT {
                for _ in 0..2 {
                    let observation = encoder::observe(&native::initial_observation(c)?)?;
                    let d0 = stream.next_draw();
                    let s0 = stream.0;
                    let d1 = stream.next_draw();
                    let s1 = stream.0;
                    let draws = [d0, d1];
                    drawn.push(json!({"draws":draws,"states":[s0,s1]}));
                    for (mode, theta, samples) in [
                        (Mode::Graph, &graph, &mut gs),
                        (Mode::Blind, &blind, &mut bs),
                    ] {
                        let input = encoder::input(mode, &observation);
                        let actions = encoder::sample(theta, &input, draws)?;
                        let decision = json!({"mode":mode,"iteration":iteration,"observation":observation,"input":input,"draws":draws,"draw_states":[s0,s1],"theta":theta,"probabilities":encoder::probabilities(theta,&input)?,"log_probability":encoder::log_probability(theta,&input,actions)?,"score_gradient":encoder::score_gradient(theta,&input,actions)?});
                        if let Some(r) = trial.cell(
                            "training",
                            seed,
                            c,
                            Arm::LivePaid,
                            actions,
                            Some(decision),
                        )? {
                            samples.push(Sample {
                                context: c,
                                input,
                                actions,
                                reward: r,
                            });
                        }
                    }
                }
            }
            let complete = gs.len() == 12 && bs.len() == 12;
            if complete {
                graph = encoder::update(Mode::Graph, &graph, &gs, 4.)?;
                blind = encoder::update(Mode::Blind, &blind, &bs, 4.)?;
                updates += 2;
            }
            transitions.push(json!({"policy_seed":seed,"iteration":iteration,"first_cell":first,"last_cell":trial.index-1,"draws":drawn,"before_graph":before_graph,"before_blind":before_blind,"after_graph":graph,"after_blind":blind,"skipped":!complete}));
            save(&out.join("transitions.json"), &json!(transitions))?;
        }
        let finals = trial.evaluation("final", seed)?;
        let mut controls = vec![];
        for c in old::RESERVED {
            let actions = std::array::from_fn(|s| {
                native::CHOICES[s]
                    .iter()
                    .position(|a| *a == Some(c[s]))
                    .unwrap()
            });
            for arm in native::ARMS[1..].iter().copied() {
                let reward = trial.cell("control", seed, c, arm, actions, None)?;
                controls.push(json!({"context":c,"arm":arm,"reward":reward}));
            }
        }
        let row = match (baseline, finals) {
            (Some(base), Some(finals)) => {
                comparison(seed, si, &graph, &blind, &base, &finals, json!(controls))?
            }
            _ => json!({"policy_seed":seed,"invalid_endpoint":true,"controls":controls}),
        };
        comparisons.push(row);
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
                "uniform_unchanged",
                "native_matrix_unchanged",
                "positive_control_unchanged",
                "positive_control_pass",
                "controls_pass",
            ]
            .iter()
            .all(|k| c[*k] == true)
        });
    let m = json!({"profile":PROFILE,"status":if complete{"native_complete"}else{"failed"},"design":design,"recorded_cells":trial.index,"completed_updates":updates,"native_attempts":if complete{json!(42768)}else{Value::Null},"native_work_unknown":!complete,"failures":trial.failures,"learning_criterion_pass":passed,"bounded_learning_authorized":true,"full_training_authorized":false,"training_admitted":false,"scientific_acceptance":false,"independent_physics_replay":false});
    save(&out.join("manifest.json"), &m)?;
    if !complete {
        return Err("invalid trial retained; no retry".into());
    }
    Ok(m)
}
