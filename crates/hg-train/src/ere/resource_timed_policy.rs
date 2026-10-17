//! Pure live-inventory decision; engine ownership remains in sim-core.
use super::resource_adapter as adapter;
use hg_sim_core::ere::resource_timed_bridge::Decision;
use serde_json::{json, Value};

pub fn choose(
    theta: &[f64; 4],
    mode: adapter::Mode,
    inventory: &Value,
    draw: f64,
) -> Result<(Decision, Value), String> {
    let observation = adapter::observe(inventory)?;
    let input = adapter::input(mode, &observation)?;
    let probabilities = adapter::probabilities(theta, &input)?;
    let action = adapter::sample(theta, &input, draw)?;
    let request = if input[1] == 1. {
        Some(adapter::request(&observation, action)?)
    } else {
        None
    };
    let decision = Decision { step: 13, request };
    Ok((
        decision,
        json!({"observation":observation,"input":input,"mode":mode,
        "theta":theta,"probabilities":probabilities,"draw":draw,"sampled_action":action,
        "decision":decision}),
    ))
}
