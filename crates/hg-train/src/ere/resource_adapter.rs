//! Step-13 resource-token projection and binary head. No engine or optimizer.
use hg_sim_core::ere::defense_allocation as native;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;

pub const PROFILE: &str = "E2-resource-token-adapter-engineering-v1";
pub const PROBE: [f64; 4] = [0.07, -0.11, 0.13, -0.17];
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Observation {
    wallet: f64,
    forecast: f64,
    bounty: f64,
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Full,
    ForecastMasked,
    BountyMasked,
    CuesMasked,
}
pub const MODES: [Mode; 4] = [
    Mode::Full,
    Mode::ForecastMasked,
    Mode::BountyMasked,
    Mode::CuesMasked,
];

fn key(s: &str, prefix: &str) -> Result<(u64, u64), String> {
    let (i, v) = s
        .strip_prefix(prefix)
        .and_then(|s| s.strip_suffix(')'))
        .and_then(|s| s.split_once('v'))
        .ok_or("invalid graph key")?;
    let i = i.parse::<u64>().map_err(|_| "invalid index")?;
    let v = v.parse::<u64>().map_err(|_| "invalid version")?;
    if v == 0 || s != format!("{prefix}{i}v{v})") {
        return Err("noncanonical key".into());
    }
    Ok((i, v))
}
fn reference(v: &Value) -> Result<(u64, u64), String> {
    let o = v.as_object().ok_or("invalid node reference")?;
    if o.len() != 2 {
        return Err("invalid node reference fields".into());
    }
    let i = v["idx"].as_u64().ok_or("invalid index")?;
    let ver = v["version"]
        .as_u64()
        .filter(|v| *v > 0)
        .ok_or("invalid version")?;
    Ok((i, ver))
}
fn property(data: &Value, property: u32, kind: &str) -> Result<f64, String> {
    let rows = data["properties"].as_array().ok_or("missing properties")?;
    let mut seen = BTreeSet::new();
    let mut result = None;
    for r in rows {
        let r = r
            .as_array()
            .filter(|r| r.len() == 2)
            .ok_or("invalid property row")?;
        let p = r[0].as_u64().ok_or("invalid property id")?;
        if !seen.insert(p) {
            return Err("duplicate property".into());
        }
        if p == property as u64 {
            let o = r[1]
                .as_object()
                .filter(|o| o.len() == 1)
                .ok_or("invalid typed property")?;
            let v = o.get(kind).ok_or("wrong property type")?;
            result = Some(if kind == "Int" {
                v.as_i64().ok_or("expected integer")? as f64
            } else {
                v.as_f64()
                    .filter(|v| v.is_finite())
                    .ok_or("expected finite float")?
            });
        }
    }
    result.ok_or_else(|| "missing required property".into())
}
/// Only the inventory is accepted, not its convenience/outcome wrapper.
pub fn observe(raw: &Value) -> Result<Observation, String> {
    if raw["step"].as_u64() != Some(13) {
        return Err("requires pre-allocation step13".into());
    }
    let rows = raw["nodes"].as_array().ok_or("missing nodes")?;
    let mut ids = BTreeSet::new();
    let mut tokens = BTreeSet::new();
    let mut wallet = None;
    let mut forecast = None;
    let mut bounty = None;
    for row in rows {
        let r = row
            .as_array()
            .filter(|r| r.len() == 2)
            .ok_or("invalid node row")?;
        let id = key(r[0].as_str().ok_or("invalid node key")?, "NodeId(")?;
        if !ids.insert(id) {
            return Err("duplicate node".into());
        }
        let data = &r[1]["data"];
        let label = data["label"]
            .as_u64()
            .filter(|l| *l <= u32::MAX as u64)
            .ok_or("invalid label")? as u32;
        if [native::WALLET, native::THREAT, native::PAYOFF].contains(&label) {
            if !r[1]["incident"]
                .as_array()
                .ok_or("missing token incidence")?
                .is_empty()
            {
                return Err("token is not isolated".into());
            }
            tokens.insert(id);
            match label {
                native::WALLET => {
                    if wallet.is_some() {
                        return Err("duplicate wallet".into());
                    }
                    wallet = Some(property(data, 1, "Float")?);
                }
                native::THREAT => {
                    if forecast.is_some() {
                        return Err("duplicate forecast".into());
                    }
                    if property(data, native::PENDING, "Int")? != 0. {
                        return Err("post-request observation".into());
                    }
                    forecast = Some(property(data, native::FORECAST, "Int")?);
                }
                _ => {
                    if bounty.is_some() {
                        return Err("duplicate bounty".into());
                    }
                    if property(data, native::CREDITS, "Int")? != 0. {
                        return Err("post-allocation observation".into());
                    }
                    bounty = Some(property(data, native::BOUNTY, "Int")?);
                }
            }
        }
    }
    let mut edges = BTreeSet::new();
    for r in raw["edges"].as_array().ok_or("missing edges")? {
        let r = r
            .as_array()
            .filter(|r| r.len() == 2)
            .ok_or("invalid edge row")?;
        if !edges.insert(key(r[0].as_str().ok_or("invalid edge key")?, "EdgeId(")?) {
            return Err("duplicate edge".into());
        }
        for n in r[1]["nodes"].as_array().ok_or("missing endpoints")? {
            let id = reference(n)?;
            if !ids.contains(&id) || tokens.contains(&id) {
                return Err("dangling endpoint or linked token".into());
            }
        }
    }
    let o = Observation {
        wallet: wallet.ok_or("missing wallet")?,
        forecast: forecast.ok_or("missing forecast")?,
        bounty: bounty.ok_or("missing bounty")?,
    };
    validate(&o)?;
    Ok(o)
}
fn validate(o: &Observation) -> Result<(), String> {
    if ![0., 1.].contains(&o.wallet)
        || ![0., 1.].contains(&o.forecast)
        || ![2., 12.].contains(&o.bounty)
    {
        return Err("outside v1 domain".into());
    }
    Ok(())
}
pub fn input(mode: Mode, o: &Observation) -> Result<[f64; 4], String> {
    validate(o)?;
    Ok([
        1.,
        o.wallet,
        if matches!(mode, Mode::ForecastMasked | Mode::CuesMasked) {
            0.
        } else {
            o.forecast
        },
        if matches!(mode, Mode::BountyMasked | Mode::CuesMasked) {
            0.
        } else {
            o.bounty / 12.
        },
    ])
}
pub fn initialize() -> [f64; 4] {
    [0.; 4]
}
pub fn probabilities(theta: &[f64; 4], x: &[f64; 4]) -> Result<[f64; 2], String> {
    if theta.iter().chain(x).any(|v| !v.is_finite())
        || x[0] != 1.
        || x[1..].iter().any(|v| !(0. ..=1.).contains(v))
    {
        return Err("invalid head input".into());
    }
    let z: f64 = theta.iter().zip(x).map(|(a, b)| a * b).sum();
    if !z.is_finite() {
        return Err("nonfinite logit".into());
    }
    let p = if z >= 0. {
        1. / (1. + (-z).exp())
    } else {
        let e = z.exp();
        e / (1. + e)
    };
    if p == 0. || p == 1. {
        return Err("saturated head".into());
    }
    Ok([p, 1. - p])
}
pub fn sample(theta: &[f64; 4], x: &[f64; 4], draw: f64) -> Result<usize, String> {
    if !draw.is_finite() || !(0. ..1.).contains(&draw) {
        return Err("draw must be in [0,1)".into());
    }
    Ok(usize::from(draw >= probabilities(theta, x)?[0]))
}
pub fn score_gradient(theta: &[f64; 4], x: &[f64; 4], action: usize) -> Result<[f64; 4], String> {
    if action > 1 {
        return Err("invalid action".into());
    }
    let p = probabilities(theta, x)?[0];
    Ok(x.map(|v| v * (if action == 0 { 1. } else { 0. } - p)))
}
pub fn expected_utility(
    theta: &[f64; 4],
    x: &[f64; 4],
    utility: [f64; 2],
) -> Result<(f64, [f64; 4]), String> {
    if utility
        .iter()
        .any(|v| !v.is_finite() || *v < 0. || *v > 20.)
    {
        return Err("invalid v1 utility".into());
    }
    let [p, q] = probabilities(theta, x)?;
    Ok((
        p * utility[0] + q * utility[1],
        x.map(|v| v * p * q * (utility[0] - utility[1])),
    ))
}
/// A request only. The future sim-core runner must revalidate timing/resources.
pub fn request(o: &Observation, action: usize) -> Result<native::Action, String> {
    validate(o)?;
    if o.wallet != 1. {
        return Err("unfunded allocation".into());
    }
    match action {
        0 => Ok(native::Action::Defend),
        1 => Ok(native::Action::Harvest),
        _ => Err("invalid action".into()),
    }
}
