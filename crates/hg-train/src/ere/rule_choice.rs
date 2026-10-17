//! A bounded contextual action interface. No optimizer, host graph edits, or admission.
use hg_sim_core::ere::{lifecycle_fixture as lifecycle, rule_choice as native};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

pub const PARAMS: usize = 12;

fn context_index(context: u32) -> Result<usize, String> {
    native::CONTEXTS.iter().position(|x| *x == context).ok_or("unknown context".into())
}

pub fn probabilities(theta: &[f64], context: u32) -> Result<[f64; 4], String> {
    let index = context_index(context)?;
    if theta.len() != PARAMS || theta.iter().any(|x| !x.is_finite()) {
        return Err("requires twelve finite logits".into());
    }
    let row = &theta[index * 4..index * 4 + 4];
    let max = row.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    if row.iter().any(|x| !(*x - max).is_finite()) {
        return Err("logit range overflow".into());
    }
    let weights: Vec<_> = row.iter().map(|x| (*x - max).exp()).collect();
    let total: f64 = weights.iter().sum();
    Ok(std::array::from_fn(|i| weights[i] / total))
}

pub fn log_probability(theta: &[f64], context: u32, action: usize) -> Result<f64, String> {
    let p = probabilities(theta, context)?;
    if action >= 4 { return Err("unknown action".into()); }
    let index = context_index(context)?;
    let row = &theta[index * 4..index * 4 + 4];
    let max = row.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    // Do not take ln(p): underflowed probabilities can still have finite log likelihood.
    let total: f64 = row.iter().map(|x| (*x - max).exp()).sum();
    let result = row[action] - max - total.ln();
    if !result.is_finite() || p.iter().any(|x| !x.is_finite()) {
        return Err("nonfinite likelihood".into());
    }
    Ok(result)
}

pub fn sample(theta: &[f64], context: u32, draw: f64) -> Result<usize, String> {
    if !draw.is_finite() || !(0.0..1.0).contains(&draw) { return Err("draw outside [0,1)".into()); }
    let p = probabilities(theta, context)?;
    let mut cumulative = 0.;
    for (i, probability) in p.iter().enumerate() {
        cumulative += probability;
        if draw < cumulative { return Ok(i); }
    }
    // Rounding at the last bin must not select an underflowed zero-probability action.
    Ok(p.iter().rposition(|x| *x > 0.).unwrap())
}

pub fn score_gradient(theta: &[f64], context: u32, action: usize) -> Result<[f64; PARAMS], String> {
    log_probability(theta, context, action)?;
    let p = probabilities(theta, context)?;
    let offset = context_index(context)? * 4;
    let mut result = [0.; PARAMS];
    for i in 0..4 { result[offset + i] = if i == action { 1. } else { 0. }; result[offset + i] -= p[i]; }
    Ok(result)
}

/// Reconstruct reward from a complete native receipt, never its cached summary.
/// This validates the interface; the independent auditor additionally replays all
/// typed node/edge deltas and native budgets. It is not a source-attestation API.
pub fn reward(receipt: &Value) -> Result<f64, String> {
    let invalid = || "invalid or incomplete native choice receipt".to_string();
    let context = receipt["context"].as_u64().and_then(|x| u32::try_from(x).ok()).ok_or_else(invalid)?;
    context_index(context)?;
    let action = receipt["action"].as_u64().filter(|x| *x < 4).ok_or_else(invalid)? as usize;
    let arm: native::Arm = serde_json::from_value(receipt["arm"].clone()).map_err(|_| invalid())?;
    if !native::ARMS.contains(&arm) || receipt["profile"] != native::PROFILE
        || receipt["classification"] != "scripted_engineering_fixture"
        || !native::SEEDS.contains(&receipt["seed"].as_u64().ok_or_else(invalid)?)
        || receipt["completed_steps"] != native::HORIZON || receipt["horizon"] != native::HORIZON
        || receipt["training_admitted"] != false || receipt["scientific_acceptance"] != false
        || receipt["independent_physics_replay"] != false { return Err(invalid()); }
    let initial_label = if arm == native::Arm::PreseededLure { context } else { 99 };
    let initial = vec![lifecycle::candidate_record(&lifecycle::candidate(true, context)),
        lifecycle::candidate_record(&lifecycle::candidate(false, initial_label))];
    if receipt["initial_all_candidates"] != serde_json::json!(initial) { return Err(invalid()); }
    let exact: BTreeSet<_> = initial.iter().map(|c| c["rcf"].as_str().unwrap()).collect();
    let structural: BTreeSet<_> = initial.iter().map(|c| c["structural_rcf"].as_str().unwrap()).collect();
    let rows = receipt["timeline"].as_array().filter(|x| x.len() == 25).ok_or_else(invalid)?;
    let world = receipt["ordinary_nodes"].as_array().filter(|x| x.len() == 3).ok_or_else(invalid)?;
    if world.iter().enumerate().any(|(i,n)| world[..i].contains(n)) { return Err(invalid()); }
    let expected_paid = matches!(arm, native::Arm::LivePaid | native::Arm::FrozenPaid) && action != 0;
    let mut births = BTreeMap::new();
    let mut credited = 0u64;
    for (step, row) in rows.iter().enumerate() {
        if row["step"] != step || row["state"]["clock"] != step || row["state"]["root_alive"] != true
            || row["paid_edits"] != u64::from(expected_paid && step == 1) { return Err(invalid()); }
        let expected_fuel = if arm == native::Arm::LiveUnpaid { 0. } else { 1. }
            - if expected_paid && step > 0 { 1. } else { 0. };
        if row["state"]["fuel"].as_f64() != Some(expected_fuel) { return Err(invalid()); }
        let expected_label = if expected_paid && step > 0 { native::ACTIONS[action].unwrap() } else { initial_label };
        let expected_genotype = vec![initial[0].clone(),
            lifecycle::candidate_record(&lifecycle::candidate(false, expected_label))];
        if row["genotype"] != serde_json::json!(expected_genotype) { return Err(invalid()); }
        for c in &expected_genotype {
            let id = c["rcf"].as_str().unwrap();
            if !exact.contains(id) && !structural.contains(c["structural_rcf"].as_str().unwrap()) {
                births.entry(id.to_string()).or_insert(step);
            }
        }
        let events = row["events"].as_array().ok_or_else(invalid)?;
        if step == 0 { if !events.is_empty() { return Err(invalid()); } continue; }
        if events.len() > 2 { return Err(invalid()); }
        if row["usage"]["truncated"] != false || row["rng_before"] != row["rng_after"]
            || row["stats"]["step"] != step || row["stats"]["accepted_rewrites"] != events.len()
            || row["compile_boundary"] != step - 1 { return Err(invalid()); }
        let installed = rows[step - 1]["installed"].as_array().filter(|x| x.len() == 2).ok_or_else(invalid)?;
        let installed_label = if arm != native::Arm::FrozenPaid && step >= 3 { expected_label } else { initial_label };
        if *installed != vec![initial[0].clone(), lifecycle::candidate_record(&lifecycle::candidate(false, installed_label))] {
            return Err(invalid());
        }
        let editors: Vec<_> = events.iter().filter(|e| e["kind"] == "seeded_editor").collect();
        if editors.len() != 1 || editors[0]["rcf"] != Value::Null
            || editors[0]["rule_name"] != if expected_paid && step == 1 { "lifecycle_seeded_edit_0" } else { "lifecycle_seeded_tick" } {
            return Err(invalid());
        }
        for event in events.iter().filter(|e| e["kind"] != "seeded_editor") {
            let rid = event["rule_id"].as_u64().filter(|x| *x < 2).ok_or_else(invalid)? as usize;
            if event["kind"] != "world" || event["bound_nodes"] != serde_json::json!(world)
                || event["rcf"] != installed[rid]["rcf"] { return Err(invalid()); }
            let id = event["rcf"].as_str().ok_or_else(invalid)?;
            if let Some(birth) = births.get(id) {
                if !expected_paid || *birth == 0 || step <= *birth { return Err(invalid()); }
                credited += 1;
            }
        }
    }
    if rows[24]["installed"] != rows[23]["installed"] { return Err(invalid()); }
    // A forged cache is refused even though it is never used to compute credit.
    if receipt["summary"] != lifecycle::summarize(&initial, rows) { return Err(invalid()); }
    Ok(credited as f64 / native::HORIZON as f64)
}
