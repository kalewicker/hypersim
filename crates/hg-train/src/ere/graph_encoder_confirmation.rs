//! Fixed-policy native transfer; no optimizer and no changes to native physics.
use super::{
    graph_encoder as e, graph_encoder_learning as learning, rule_choice_learning::DrawStream,
    rule_relational as rewards, rule_relational_learning as old,
};
use hg_sim_core::ere::rule_relational::{self as native, Arm};
use serde_json::{json, Value};
use std::{
    path::Path,
    time::{Duration, Instant},
};

pub const PROFILE: &str = "E2-graph-encoder-native-confirmation-v1";
pub const POLICIES: [&str; 4] = [
    "learned_graph",
    "trained_blind",
    "initial_graph",
    "learned_graph_masked",
];
pub const DRAW_SEED: u64 = 993748722001;
pub fn design() -> Value {
    serde_json::from_str(include_str!(
        "../../../../configs/ere_graph_encoder_confirmation_v1.json"
    ))
    .unwrap()
}
pub fn policies(import: &Value) -> Result<Vec<(e::Mode, Vec<f64>)>, String> {
    let rows = import.as_array().ok_or("invalid checkpoint import")?;
    if rows.len() != 2 || rows[0]["checkpoint"] != 12 || rows[1]["checkpoint"] != 100 {
        return Err("wrong frozen checkpoints".into());
    }
    let graph: Vec<f64> = serde_json::from_value(rows[1]["final_graph"]["theta"].clone())
        .map_err(|e| e.to_string())?;
    let blind: Vec<f64> = serde_json::from_value(rows[1]["final_blind"]["theta"].clone())
        .map_err(|e| e.to_string())?;
    let o = e::observe(&native::initial_observation(native::CONTEXTS[0])?)?;
    for (mode, t) in [(e::Mode::Graph, &graph), (e::Mode::Blind, &blind)] {
        e::probabilities(t, &e::input(mode, &o))?;
    }
    Ok(vec![
        (e::Mode::Graph, graph.clone()),
        (e::Mode::Blind, blind),
        (e::Mode::Graph, e::initialize()),
        (e::Mode::Blind, graph),
    ])
}
pub fn comparison(
    seed: u64,
    policies: &[(e::Mode, Vec<f64>)],
    matrix: &[f64],
    controls: Value,
    base: &[f64],
    import: &Value,
) -> Result<Value, String> {
    if policies.len() != 4 || controls.as_array().map(|v| v.len()) != Some(18) {
        return Err("missing fixed policies/controls".into());
    }
    let ends = policies
        .iter()
        .map(|(mode, t)| learning::endpoint(*mode, t, matrix))
        .collect::<Result<Vec<_>, _>>()?;
    let g = ends[0]["balanced_expected_reward"].as_f64().unwrap();
    let reward = |i: usize| ends[i]["balanced_expected_reward"].as_f64().unwrap();
    let gain = ends[0]["reserved_expected_reward"].as_f64().unwrap()
        - ends[2]["reserved_expected_reward"].as_f64().unwrap();
    let every = ends[0]["contexts"]
        .as_array()
        .unwrap()
        .iter()
        .zip(ends[2]["contexts"].as_array().unwrap())
        .filter(|(c, _)| c["reserved"] == true)
        .all(|(c, b)| {
            c["correct_joint_probability"].as_f64().unwrap()
                > b["correct_joint_probability"].as_f64().unwrap()
        });
    let flags = json!({"graph_reward_pass":g>=1./24.+0.01,"trained_blind_bound_pass":reward(1)<=1./24.+2e-12,
        "trained_blind_gap_pass":g-reward(1)>=0.01,"initial_gap_pass":g-reward(2)>=0.01,"same_theta_mask_gap_pass":g-reward(3)>=0.01,
        "reserved_gain_pass":gain>=0.01,"every_reserved_probability_gain":every,
        "controls_pass":controls.as_array().unwrap().iter().all(|c|c["reward"].as_f64()==Some(if c["arm"]=="reverse_paid"{9./24.}else{0.})),
        "native_matrix_unchanged":matrix==base,"archived_graph_endpoint_match":ends[0]==import[1]["final_graph"],"archived_blind_endpoint_match":ends[1]==import[1]["final_blind"]});
    let pass = flags.as_object().unwrap().values().all(|v| v == true);
    Ok(
        json!({"native_seed":seed,"policies":POLICIES.iter().zip(&ends).map(|(name,e)|json!({"policy":name,"endpoint":e})).collect::<Vec<_>>(),
        "graph_minus_trained_blind":g-reward(1),"graph_minus_initial":g-reward(2),"graph_minus_same_theta_mask":g-reward(3),"reserved_gain":gain,"flags":flags,"confirmation_pass":pass,"controls":controls}),
    )
}
fn save(path: &Path, v: &Value) -> Result<(), String> {
    std::fs::write(path, serde_json::to_vec(v).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
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
        seed: u64,
        c: [u32; 2],
        arm: Arm,
        actions: [usize; 2],
        decision: Option<Value>,
    ) -> Result<Option<f64>, String> {
        let id = self.index;
        self.index += 1;
        let mut wrapper = json!({"id":id,"phase":phase,"native_seed":seed,"context":c,"arm":arm,"actions":actions,"decision":decision});
        if decision.is_some() {
            save(&self.out.join(format!("decision-{id:04}.json")), &wrapper)?;
        }
        let mut row = match native::run(arm, c, actions, seed, self.deadline) {
            Ok(r) => r,
            Err(error) => {
                self.failures.push(format!("cell{id}: {error}"));
                json!({"error":error,"partial_native_work_unknown":true})
            }
        };
        let result = if row.get("error").is_none() {
            match (
                rewards::observe(&row["initial_inventory"]),
                e::observe(&row["initial_inventory"]),
                rewards::reward(&row),
            ) {
                (Ok(hand), Ok(raw), Ok(reward)) => {
                    row["encoded_observation"] = json!(hand);
                    row["raw_graph_observation"] = json!(raw);
                    row["adapter_reward"] = json!(reward);
                    if let Some(d) = &decision {
                        if d["observation"] != row["raw_graph_observation"] {
                            self.failures
                                .push(format!("cell{id}: decision observation mismatch"));
                            row["decision_observation_error"] = json!(true);
                        }
                    }
                    (!row["decision_observation_error"].as_bool().unwrap_or(false))
                        .then_some(reward)
                }
                _ => {
                    self.failures
                        .push(format!("cell{id}: observation/reward error"));
                    row["adapter_error"] = json!(true);
                    None
                }
            }
        } else {
            None
        };
        wrapper["native"] = row;
        save(&self.out.join(format!("cell-{id:04}.json")), &wrapper)?;
        Ok(result)
    }
}
pub fn run(out: &Path, import: &Value, base: &[f64]) -> Result<Value, String> {
    std::fs::create_dir(out).map_err(|e| format!("fresh native directory required: {e}"))?;
    let policies = policies(import)?;
    save(
        &out.join("manifest.json"),
        &json!({"profile":PROFILE,"status":"running","design":design(),"scientific_acceptance":false,"training_admitted":false,"sustained_claim":false}),
    )?;
    std::fs::copy(
        std::env::current_exe().map_err(|e| e.to_string())?,
        out.join("executed_test_binary"),
    )
    .map_err(|e| e.to_string())?;
    let mut trial = Trial {
        out,
        deadline: Instant::now() + Duration::from_secs(180),
        index: 0,
        failures: vec![],
    };
    let mut matrices = vec![];
    let mut controls = vec![];
    let mut sampled = vec![];
    for seed in native::SEEDS {
        let mut matrix = vec![];
        for c in native::CONTEXTS {
            for a in 0..4 {
                for b in 0..4 {
                    let Some(r) = trial.cell("matrix", seed, c, Arm::LivePaid, [a, b], None)?
                    else {
                        return Err("native matrix failure retained without retry".into());
                    };
                    matrix.push(r);
                }
            }
        }
        matrices.push(matrix);
    }
    for seed in native::SEEDS {
        let mut rows = vec![];
        for c in old::RESERVED {
            let correct = std::array::from_fn(|s| {
                native::CHOICES[s]
                    .iter()
                    .position(|x| *x == Some(c[s]))
                    .unwrap()
            });
            for arm in native::ARMS[1..].iter().copied() {
                let Some(r) = trial.cell("control", seed, c, arm, correct, None)? else {
                    return Err("native control failure retained without retry".into());
                };
                rows.push(json!({"context":c,"arm":arm,"reward":r}));
            }
        }
        controls.push(json!(rows));
    }
    let mut stream = DrawStream(DRAW_SEED);
    for seed in native::SEEDS {
        for c in native::CONTEXTS {
            let o = e::observe(&native::initial_observation(c)?)?;
            let mut draws = [0.; 2];
            let mut states = [0; 2];
            for s in 0..2 {
                draws[s] = stream.next_draw();
                states[s] = stream.0;
            }
            for (i, (mode, t)) in policies.iter().enumerate() {
                let input = e::input(*mode, &o);
                let p = e::probabilities(t, &input)?;
                let actions = e::sample(t, &input, draws)?;
                let decision = json!({"policy":POLICIES[i],"mode":mode,"observation":o,"input":input,"theta":t,"draws":draws,"draw_states":states,"probabilities":p});
                let id = trial.index;
                let Some(r) =
                    trial.cell("decision", seed, c, Arm::LivePaid, actions, Some(decision))?
                else {
                    return Err("native decision failure retained without retry".into());
                };
                sampled.push(json!({"id":id,"native_seed":seed,"context":c,"policy":POLICIES[i],"actions":actions,"reward":r}));
            }
        }
    }
    let comparisons = native::SEEDS
        .iter()
        .enumerate()
        .map(|(i, seed)| {
            comparison(
                *seed,
                &policies,
                &matrices[i],
                controls[i].clone(),
                base,
                import,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let pass = comparisons.iter().all(|c| c["confirmation_pass"] == true);
    save(&out.join("comparisons.json"), &json!(comparisons))?;
    save(&out.join("sampled.json"), &json!(sampled))?;
    let result = json!({"profile":PROFILE,"status":"native_complete","design":design(),"recorded_cells":trial.index,"native_attempts":9504,"advancing_steps":9504,"optimizer_updates":0,
        "failures":trial.failures,"native_work_unknown":false,"confirmation_pass":pass,"training_admitted":false,"scientific_acceptance":false,"sustained_claim":false,"independent_physics_replay":false});
    save(&out.join("manifest.json"), &result)?;
    Ok(result)
}
