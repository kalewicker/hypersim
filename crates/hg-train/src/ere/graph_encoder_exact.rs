//! Exact development reward gradients on imported receipts; never runs physics.
use super::{
    graph_encoder as e, graph_encoder_learning as learning, rule_relational_learning as old,
};
use hg_sim_core::ere::rule_relational as native;
use serde_json::{json, Value};
use std::path::Path;

pub const PROFILE: &str = "E2-graph-encoder-exact-gradient-v1";
pub fn design() -> Value {
    serde_json::from_str(include_str!(
        "../../../../configs/ere_graph_encoder_exact_v1.json"
    ))
    .unwrap()
}
#[derive(Clone)]
pub struct World {
    pub context: [u32; 2],
    pub observation: e::Observation,
    pub rewards: Vec<f64>,
}
pub fn objective(
    mode: e::Mode,
    theta: &[f64],
    worlds: &[World],
) -> Result<(f64, Vec<f64>), String> {
    if worlds.len() != 6
        || worlds
            .iter()
            .zip(old::DEVELOPMENT)
            .any(|(w, c)| w.context != c)
        || worlds.iter().any(|w| {
            w.rewards.len() != 16
                || w.rewards
                    .iter()
                    .any(|r| !r.is_finite() || !(0.0..=1.0).contains(r))
        })
    {
        return Err("exact ordered six-world/sixteen-action denominator required".into());
    }
    let mut value = 0.;
    let mut gradient = vec![0.; e::PARAMS];
    for w in worlds {
        let input = e::input(mode, &w.observation);
        let p = e::probabilities(theta, &input)?;
        for a in 0..4 {
            for b in 0..4 {
                let weight = p[0][a] * p[1][b] * w.rewards[a * 4 + b] / 6.;
                let score = e::score_gradient(theta, &input, [a, b])?;
                value += weight;
                for (g, s) in gradient.iter_mut().zip(score) {
                    *g += weight * s;
                }
            }
        }
    }
    if !value.is_finite() || gradient.iter().any(|g| !g.is_finite()) {
        return Err("nonfinite objective".into());
    }
    Ok((value, gradient))
}
pub fn update(
    mode: e::Mode,
    theta: &[f64],
    worlds: &[World],
    rate: f64,
) -> Result<(f64, Vec<f64>, Vec<f64>), String> {
    if !rate.is_finite() || rate <= 0. || rate > 4. {
        return Err("invalid fixed rate".into());
    }
    let (value, gradient) = objective(mode, theta, worlds)?;
    let after: Vec<f64> = theta
        .iter()
        .zip(&gradient)
        .map(|(t, g)| t + rate * g)
        .collect();
    e::probabilities(&after, &e::input(mode, &worlds[0].observation))?;
    Ok((value, gradient, after))
}
fn save(path: &Path, v: &Value) -> Result<(), String> {
    std::fs::write(path, serde_json::to_vec(v).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}
pub fn run(out: &Path) -> Result<Value, String> {
    let data: Value =
        serde_json::from_slice(&std::fs::read(out.join("input.json")).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    let rows = data["worlds"].as_array().ok_or("missing worlds")?;
    if rows.len() != 9 {
        return Err("nine endpoint worlds required".into());
    }
    let mut dev = vec![];
    let mut rewards = vec![];
    for (r, c) in rows.iter().zip(native::CONTEXTS) {
        if r["context"] != json!(c) {
            return Err("wrong world order".into());
        }
        let o = e::observe(&r["raw"])?;
        // Bind the saved endpoint observations to the existing pure initializer;
        // the optimizer itself only receives saved raw graph + reward vectors.
        if json!(o) != json!(e::observe(&native::initial_observation(c)?)?) {
            return Err("endpoint observation mismatch".into());
        }
        let rs: Vec<f64> =
            serde_json::from_value(r["rewards"].clone()).map_err(|e| e.to_string())?;
        if rs.len() != 16 {
            return Err("incomplete reward row".into());
        }
        rewards.extend_from_slice(&rs);
        if old::DEVELOPMENT.contains(&c) {
            dev.push(World {
                context: c,
                observation: o,
                rewards: rs,
            });
        }
    }
    let mut graph = e::initialize();
    let mut blind = graph.clone();
    let mut transitions = vec![];
    let mut checkpoints = vec![];
    for iteration in 0..100 {
        for (mode, theta) in [(e::Mode::Graph, &mut graph), (e::Mode::Blind, &mut blind)] {
            let (value, gradient, after) = update(mode, theta, &dev, 4.)?;
            transitions.push(json!({"iteration":iteration,"mode":mode,"before":theta,"objective":value,"gradient":gradient,"after":after}));
            *theta = after;
        }
        if [12, 100].contains(&(iteration + 1)) {
            let mut c = learning::comparison(
                learning::POLICY_SEEDS[0],
                0,
                &graph,
                &blind,
                &rewards,
                &rewards,
                data["controls"].clone(),
            )?;
            c.as_object_mut().unwrap().remove("policy_seed");
            c["checkpoint"] = json!(iteration + 1);
            let pass = [
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
            .all(|k| c[k] == true);
            c["diagnostic_criterion_pass"] = json!(pass);
            checkpoints.push(c);
        }
    }
    save(&out.join("transitions.json"), &json!(transitions))?;
    save(&out.join("checkpoints.json"), &json!(checkpoints))?;
    std::fs::copy(
        std::env::current_exe().map_err(|e| e.to_string())?,
        out.join("executed_test_binary"),
    )
    .map_err(|e| e.to_string())?;
    let result = json!({"profile":PROFILE,"completed_updates":200,"native_attempts":0,"checkpoints":checkpoints.iter().map(|c|json!({"checkpoint":c["checkpoint"],"diagnostic_criterion_pass":c["diagnostic_criterion_pass"]})).collect::<Vec<_>>(),"training_admitted":false,"scientific_acceptance":false,"independent_physics_replay":false,"sustained_claim":false});
    save(&out.join("optimizer_result.json"), &result)?;
    Ok(result)
}
