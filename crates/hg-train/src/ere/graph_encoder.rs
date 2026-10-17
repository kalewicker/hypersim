//! Trainable raw-adjacency message passing; no task-distance readout or physics.
use super::{rule_choice as softmax, rule_choice_learning::DrawStream};
use hg_sim_core::ere::rule_relational as native;
use serde::Serialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

pub const PROFILE: &str = "E2-graph-encoder-engineering-v1";
pub const LABELS: [u32; 7] = [10, 11, 12, 20, 21, 30, 31];
pub const PARAMS: usize = 110;
pub const INIT_SEED: u64 = 993748720001;
type Hidden = [[f64; 4]; 7];
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Graph,
    Blind,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Observation {
    labels: [u32; 7],
    adjacency: [[bool; 7]; 7],
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Input {
    mode: Mode,
    observation: Observation,
}

fn node_key(v: &Value) -> Result<String, String> {
    if v.as_object().map(|o| o.len()) != Some(2) {
        return Err("invalid reference".into());
    }
    Ok(format!(
        "NodeId({}v{})",
        v["idx"].as_u64().ok_or("invalid index")?,
        v["version"]
            .as_u64()
            .filter(|i| *i > 0)
            .ok_or("invalid version")?
    ))
}
fn valid_key(s: &str, kind: &str) -> bool {
    let Some(inner) = s
        .strip_prefix(kind)
        .and_then(|s| s.strip_prefix('('))
        .and_then(|s| s.strip_suffix(')'))
    else {
        return false;
    };
    let Some((i, v)) = inner.split_once('v') else {
        return false;
    };
    i.parse::<u64>().is_ok() && v.parse::<u64>().map(|v| v > 0).unwrap_or(false)
}
/// Canonical ordinary one-hots/adjacency only. No path/task interpretation.
pub fn observe(raw: &Value) -> Result<Observation, String> {
    let invalid = || "invalid initial raw-graph observation".to_string();
    if raw["step"].as_u64() != Some(0) {
        return Err(invalid());
    }
    let mut ids = BTreeMap::new();
    let mut ordinary = BTreeMap::new();
    for row in raw["nodes"].as_array().ok_or_else(invalid)? {
        if row.as_array().map(|r| r.len()) != Some(2) {
            return Err(invalid());
        }
        let id = row[0]
            .as_str()
            .filter(|s| valid_key(s, "NodeId"))
            .ok_or_else(invalid)?
            .to_owned();
        let label = row[1]["data"]["label"]
            .as_u64()
            .and_then(|l| u32::try_from(l).ok())
            .ok_or_else(invalid)?;
        if ids.insert(id.clone(), label).is_some() {
            return Err(invalid());
        }
        if label < native::ORDINARY_CEILING && ordinary.insert(label, id).is_some() {
            return Err(invalid());
        }
    }
    if ordinary.keys().copied().collect::<Vec<_>>() != LABELS {
        return Err(invalid());
    }
    let positions: BTreeMap<_, _> = LABELS
        .iter()
        .enumerate()
        .map(|(i, l)| (ordinary[l].clone(), i))
        .collect();
    let mut seen = BTreeSet::new();
    let mut adjacent = [[false; 7]; 7];
    for row in raw["edges"].as_array().ok_or_else(invalid)? {
        if row.as_array().map(|r| r.len()) != Some(2) {
            return Err(invalid());
        }
        let id = row[0]
            .as_str()
            .filter(|s| valid_key(s, "EdgeId"))
            .ok_or_else(invalid)?;
        if !seen.insert(id) {
            return Err(invalid());
        }
        let nodes = row[1]["nodes"].as_array().ok_or_else(invalid)?;
        let endpoints = nodes.iter().map(node_key).collect::<Result<Vec<_>, _>>()?;
        if endpoints.iter().any(|id| !ids.contains_key(id)) {
            return Err(invalid());
        }
        let ord: Vec<_> = endpoints
            .iter()
            .filter_map(|id| positions.get(id).copied())
            .collect();
        if ord.is_empty() {
            continue;
        }
        if ord.len() != 2
            || endpoints.len() != 2
            || ord[0] == ord[1]
            || adjacent[ord[0]][ord[1]]
            || row[1]["label"].as_u64() != Some(0)
        {
            return Err(invalid());
        }
        adjacent[ord[0]][ord[1]] = true;
        adjacent[ord[1]][ord[0]] = true;
    }
    Ok(Observation {
        labels: LABELS,
        adjacency: adjacent,
    })
}
pub fn input(mode: Mode, observation: &Observation) -> Input {
    let mut observation = observation.clone();
    if mode == Mode::Blind {
        observation.adjacency = [[false; 7]; 7];
    }
    Input { mode, observation }
}
pub fn initialize() -> Vec<f64> {
    let mut stream = DrawStream(INIT_SEED);
    (0..PARAMS)
        .map(|_| 0.15 * (2. * stream.next_draw() - 1.))
        .collect()
}
fn trace(theta: &[f64], inp: &Input) -> Result<[Hidden; 3], String> {
    if theta.len() != PARAMS || theta.iter().any(|t| !t.is_finite()) {
        return Err("requires 110 finite parameters".into());
    }
    let mut h = [[[0.; 4]; 7]; 3];
    for i in 0..7 {
        for k in 0..4 {
            let z = theta[i * 4 + k] + theta[28 + k];
            if !z.is_finite() {
                return Err("embedding overflow".into());
            }
            h[0][i][k] = z.tanh();
        }
    }
    for r in 0..2 {
        let offset = 32 + r * 36;
        for i in 0..7 {
            for k in 0..4 {
                let mut z = theta[offset + 32 + k];
                for j in 0..4 {
                    z += h[r][i][j] * theta[offset + j * 4 + k];
                    for n in 0..7 {
                        if inp.observation.adjacency[i][n] {
                            z += h[r][n][j] * theta[offset + 16 + j * 4 + k];
                        }
                    }
                }
                if !z.is_finite() {
                    return Err("message overflow".into());
                }
                h[r + 1][i][k] = z.tanh();
            }
        }
    }
    Ok(h)
}
fn candidate(slot: usize, action: usize) -> usize {
    let label = native::CHOICES[slot][action].unwrap();
    LABELS.iter().position(|l| *l == label).unwrap()
}
fn logits(theta: &[f64], inp: &Input) -> Result<[[f64; 12]; 2], String> {
    let h = trace(theta, inp)?;
    let mut z = [[0.; 12]; 2];
    for s in 0..2 {
        z[s][0] = theta[108];
        for a in 1..4 {
            z[s][a] = theta[109]
                + (0..4)
                    .map(|k| theta[104 + k] * h[2][candidate(s, a)][k])
                    .sum::<f64>();
        }
        softmax::probabilities(&z[s], 10)?;
    }
    Ok(z)
}
pub fn probabilities(theta: &[f64], inp: &Input) -> Result<[[f64; 4]; 2], String> {
    let z = logits(theta, inp)?;
    Ok([
        softmax::probabilities(&z[0], 10)?,
        softmax::probabilities(&z[1], 10)?,
    ])
}
pub fn sample(theta: &[f64], inp: &Input, draws: [f64; 2]) -> Result<[usize; 2], String> {
    let z = logits(theta, inp)?;
    Ok([
        softmax::sample(&z[0], 10, draws[0])?,
        softmax::sample(&z[1], 10, draws[1])?,
    ])
}
pub fn log_probability(theta: &[f64], inp: &Input, actions: [usize; 2]) -> Result<f64, String> {
    let z = logits(theta, inp)?;
    let lp = softmax::log_probability(&z[0], 10, actions[0])?
        + softmax::log_probability(&z[1], 10, actions[1])?;
    if !lp.is_finite() {
        return Err("joint likelihood overflow".into());
    }
    Ok(lp)
}
/// Analytic reverse-mode through both message rounds and label embeddings.
pub fn score_gradient(theta: &[f64], inp: &Input, actions: [usize; 2]) -> Result<Vec<f64>, String> {
    log_probability(theta, inp, actions)?;
    let p = probabilities(theta, inp)?;
    let h = trace(theta, inp)?;
    let mut g = vec![0.; PARAMS];
    let mut dh = [[0.; 4]; 7];
    for s in 0..2 {
        for a in 0..4 {
            let d = (if actions[s] == a { 1. } else { 0. }) - p[s][a];
            if a == 0 {
                g[108] += d;
            } else {
                let i = candidate(s, a);
                g[109] += d;
                for k in 0..4 {
                    g[104 + k] += d * h[2][i][k];
                    dh[i][k] += d * theta[104 + k];
                }
            }
        }
    }
    for r in (0..2).rev() {
        let offset = 32 + r * 36;
        let mut prev = [[0.; 4]; 7];
        for i in 0..7 {
            for k in 0..4 {
                let d = dh[i][k] * (1. - h[r + 1][i][k] * h[r + 1][i][k]);
                g[offset + 32 + k] += d;
                for j in 0..4 {
                    g[offset + j * 4 + k] += d * h[r][i][j];
                    prev[i][j] += d * theta[offset + j * 4 + k];
                    for n in 0..7 {
                        if inp.observation.adjacency[i][n] {
                            g[offset + 16 + j * 4 + k] += d * h[r][n][j];
                            prev[n][j] += d * theta[offset + 16 + j * 4 + k];
                        }
                    }
                }
            }
        }
        dh = prev;
    }
    for i in 0..7 {
        for k in 0..4 {
            let d = dh[i][k] * (1. - h[0][i][k] * h[0][i][k]);
            g[i * 4 + k] += d;
            g[28 + k] += d;
        }
    }
    if g.iter().any(|t| !t.is_finite()) {
        return Err("gradient overflow".into());
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
pub fn update(
    mode: Mode,
    theta: &[f64],
    samples: &[Sample],
    rate: f64,
) -> Result<Vec<f64>, String> {
    use super::rule_relational_learning::DEVELOPMENT;
    if !rate.is_finite()
        || !(0. < rate && rate <= 4.)
        || samples.len() != 12
        || DEVELOPMENT
            .iter()
            .any(|c| samples.iter().filter(|s| s.context == *c).count() != 2)
    {
        return Err("invalid balanced development update/rate".into());
    }
    let mut g = vec![0.; PARAMS];
    for s in samples {
        if s.input.mode != mode || !s.reward.is_finite() || !(0.0..=1.0).contains(&s.reward) {
            return Err("invalid mode/reward".into());
        }
        let gradient = score_gradient(theta, &s.input, s.actions)?;
        for i in 0..PARAMS {
            g[i] += rate * s.reward * gradient[i] / 12.;
        }
    }
    let next: Vec<_> = theta.iter().zip(g).map(|(t, g)| t + g).collect();
    probabilities(&next, &samples[0].input)?;
    Ok(next)
}
