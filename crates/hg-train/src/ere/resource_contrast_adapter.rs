//! Fixed signed graph cues. No engine, parameter update, or reward-table lookup.
//! Not input-compatible with the old resource_learning funding mask.
use super::resource_adapter as old;
use hg_sim_core::ere::resource_timed_bridge::Decision;
use serde_json::{json, Value};

pub use old::{observe, Mode, Observation};
pub const PROFILE: &str = "E2-resource-cue-contrast-adapter-v1";
pub const PROBES: [[f64; 4]; 2] = [[0.; 4], old::PROBE];
pub const MODES: [Mode; 4] = old::MODES;

pub fn initialize() -> [f64; 4] {
    [0.; 4]
}
pub fn funded(o: &Observation) -> Result<bool, String> {
    Ok(old::input(Mode::Full, o)?[1] == 1.)
}
pub fn input(mode: Mode, o: &Observation) -> Result<[f64; 4], String> {
    let raw = old::input(Mode::Full, o)?;
    let f = if matches!(mode, Mode::ForecastMasked | Mode::CuesMasked) {
        0.
    } else {
        2. * raw[2] - 1.
    };
    let b = if matches!(mode, Mode::BountyMasked | Mode::CuesMasked) {
        0.
    } else {
        2. * (12. * raw[3] - 2.) / 10. - 1.
    };
    Ok([1., f, b, f * b])
}
pub fn probabilities(theta: &[f64; 4], x: &[f64; 4]) -> Result<[f64; 2], String> {
    if theta.iter().any(|v| !v.is_finite() || v.abs() > 8.)
        || x.iter().any(|v| !v.is_finite())
        || x[0] != 1.
        || ![-1., 0., 1.].contains(&x[1])
        || ![-1., 0., 1.].contains(&x[2])
        || x[3] != x[1] * x[2]
    {
        return Err("invalid bounded contrast head".into());
    }
    let z: f64 = theta.iter().zip(x).map(|(t, v)| t * v).sum();
    let p = if z >= 0. {
        1. / (1. + (-z).exp())
    } else {
        let e = z.exp();
        e / (1. + e)
    };
    if p == 0. || p == 1. {
        return Err("saturated contrast head".into());
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
fn utility_domain(u: f64) -> Result<(), String> {
    if !u.is_finite() || !(0. ..=20.).contains(&u) {
        return Err("invalid actual utility".into());
    }
    Ok(())
}
/// Score credit bound to validated funding, never to an actor coordinate.
pub fn contribution(
    theta: &[f64; 4],
    mode: Mode,
    o: &Observation,
    action: usize,
    actual_utility: f64,
) -> Result<[f64; 4], String> {
    utility_domain(actual_utility)?;
    let score = score_gradient(theta, &input(mode, o)?, action)?;
    let credit = if funded(o)? {
        (actual_utility - 9.5) / 20.
    } else {
        0.
    };
    Ok(score.map(|v| v * credit))
}
/// Arithmetic diagnostic only; the decision API accepts no utility pair.
pub fn expected_utility(
    theta: &[f64; 4],
    x: &[f64; 4],
    utility: [f64; 2],
) -> Result<(f64, [f64; 4]), String> {
    for u in utility {
        utility_domain(u)?;
    }
    let [p, q] = probabilities(theta, x)?;
    Ok((
        p * utility[0] + q * utility[1],
        x.map(|v| v * p * q * (utility[0] - utility[1])),
    ))
}
/// Pure request on actual step-13 inventory. Sim-core must revalidate at execution.
pub fn choose(
    theta: &[f64; 4],
    mode: Mode,
    inventory: &Value,
    draw: f64,
) -> Result<(Decision, Value), String> {
    let observation = observe(inventory)?;
    let x = input(mode, &observation)?;
    let probabilities = probabilities(theta, &x)?;
    let action = sample(theta, &x, draw)?;
    let is_funded = funded(&observation)?;
    let request = if is_funded {
        Some(old::request(&observation, action)?)
    } else {
        None
    };
    let decision = Decision { step: 13, request };
    Ok((
        decision,
        json!({"feature_profile":PROFILE,"observation":observation,"input":x,"mode":mode,
            "theta":theta,"funded":is_funded,"probabilities":probabilities,
            "draw":draw,"sampled_action":action,"decision":decision}),
    ))
}
