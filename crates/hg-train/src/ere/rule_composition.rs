//! Shared two-slot action interface. No optimizer or scientific admission.
use super::rule_choice as single;
use hg_sim_core::ere::{lifecycle_fixture as lifecycle, rule_composition as native};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

pub const PARAMS: usize = 12;

fn cues(context: [u32; 2]) -> Result<[u32; 2], String> {
    if !native::CONTEXTS.contains(&context) {
        return Err("unknown composition context".into());
    }
    // Same category rows across slots: no pair-specific lookup parameters.
    Ok([context[0], context[1] - 1])
}

pub fn probabilities(theta: &[f64], context: [u32; 2]) -> Result<[[f64; 4]; 2], String> {
    let cue = cues(context)?;
    Ok([
        single::probabilities(theta, cue[0])?,
        single::probabilities(theta, cue[1])?,
    ])
}

pub fn sample(theta: &[f64], context: [u32; 2], draws: [f64; 2]) -> Result<[usize; 2], String> {
    let cue = cues(context)?;
    Ok([
        single::sample(theta, cue[0], draws[0])?,
        single::sample(theta, cue[1], draws[1])?,
    ])
}

pub fn log_probability(
    theta: &[f64],
    context: [u32; 2],
    actions: [usize; 2],
) -> Result<f64, String> {
    let cue = cues(context)?;
    let joint = single::log_probability(theta, cue[0], actions[0])?
        + single::log_probability(theta, cue[1], actions[1])?;
    if !joint.is_finite() {
        return Err("joint likelihood overflow".into());
    }
    Ok(joint)
}

pub fn score_gradient(
    theta: &[f64],
    context: [u32; 2],
    actions: [usize; 2],
) -> Result<[f64; PARAMS], String> {
    let cue = cues(context)?;
    log_probability(theta, context, actions)?;
    let first = single::score_gradient(theta, cue[0], actions[0])?;
    let second = single::score_gradient(theta, cue[1], actions[1])?;
    Ok(std::array::from_fn(|i| first[i] + second[i]))
}

/// Strict fixed-profile interface check. Full typed graph/budget reconstruction
/// remains the independent auditor's responsibility; this is not attestation.
pub fn reward(receipt: &Value) -> Result<f64, String> {
    let invalid = || "invalid/incomplete composition receipt".to_string();
    let context: [u32; 2] =
        serde_json::from_value(receipt["context"].clone()).map_err(|_| invalid())?;
    let actions: [usize; 2] =
        serde_json::from_value(receipt["actions"].clone()).map_err(|_| invalid())?;
    let arm: native::Arm = serde_json::from_value(receipt["arm"].clone()).map_err(|_| invalid())?;
    if !native::CONTEXTS.contains(&context)
        || actions.iter().any(|x| *x >= 4)
        || !native::ARMS.contains(&arm)
        || receipt["profile"] != native::PROFILE
        || receipt["classification"] != "scripted_engineering_fixture"
        || !native::SEEDS.contains(&receipt["seed"].as_u64().ok_or_else(invalid)?)
        || receipt["horizon"] != 24
        || receipt["completed_steps"] != 24
        || receipt["training_admitted"] != false
        || receipt["scientific_acceptance"] != false
        || receipt["independent_physics_replay"] != false
    {
        return Err(invalid());
    }
    let initial_labels = if arm == native::Arm::PreseededLure {
        context
    } else {
        [99, 98]
    };
    let initial = vec![
        lifecycle::candidate_record(&native::candidate(true, context)),
        lifecycle::candidate_record(&native::candidate(false, initial_labels)),
    ];
    if receipt["initial_all_candidates"] != json!(initial) {
        return Err(invalid());
    }
    let exact: BTreeSet<_> = initial.iter().map(|c| c["rcf"].as_str().unwrap()).collect();
    let structural: BTreeSet<_> = initial
        .iter()
        .map(|c| c["structural_rcf"].as_str().unwrap())
        .collect();
    let rows = receipt["timeline"]
        .as_array()
        .filter(|r| r.len() == 25)
        .ok_or_else(invalid)?;
    let world = receipt["ordinary_nodes"]
        .as_array()
        .filter(|r| r.len() == 3)
        .ok_or_else(invalid)?;
    if world
        .iter()
        .enumerate()
        .any(|(i, n)| world[..i].contains(n))
    {
        return Err(invalid());
    }
    let mut fuel = match arm {
        native::Arm::LiveUnpaid => 0.,
        native::Arm::OneFuel => 1.,
        _ => 2.,
    };
    if receipt["initial_fuel"].as_f64() != Some(fuel) {
        return Err(invalid());
    }
    let order = if arm == native::Arm::ReversePaid {
        [1, 0]
    } else {
        [0, 1]
    };
    let mut labels = initial_labels;
    let mut births = BTreeMap::new();
    let mut paid_total = 0;
    let mut credited = 0u64;
    let mut installed = initial.clone();
    for (step, row) in rows.iter().enumerate() {
        let mut expected_paid = false;
        let mut editor_name = "composition_seeded_tick".to_string();
        if step > 0 {
            if arm != native::Arm::FrozenPaid && (step - 1) % 2 == 0 {
                installed = vec![
                    initial[0].clone(),
                    lifecycle::candidate_record(&native::candidate(false, labels)),
                ];
            }
            if rows[step - 1]["installed"] != json!(installed) {
                return Err(invalid());
            }
            if step == 1 || step == 5 {
                let stage = usize::from(step == 5);
                let slot = order[stage];
                expected_paid = arm != native::Arm::PreseededLure
                    && arm != native::Arm::LiveDisabled
                    && actions[slot] != 0
                    && fuel >= 1.;
                if expected_paid {
                    labels[slot] = native::CHOICES[slot][actions[slot]].unwrap();
                    fuel -= 1.;
                    paid_total += 1;
                    editor_name = format!("composition_seeded_edit_{}_slot{slot}", step - 1);
                }
            }
        }
        let genotype = vec![
            initial[0].clone(),
            lifecycle::candidate_record(&native::candidate(false, labels)),
        ];
        if row["step"] != step
            || row["state"]["clock"] != step
            || row["state"]["root_alive"] != true
            || row["state"]["literals"] != json!(labels)
            || row["state"]["fuel"].as_f64() != Some(fuel)
            || row["paid_edits"] != u64::from(expected_paid)
            || row["genotype"] != json!(genotype)
        {
            return Err(invalid());
        }
        for c in &genotype {
            let id = c["rcf"].as_str().unwrap();
            if !exact.contains(id) && !structural.contains(c["structural_rcf"].as_str().unwrap()) {
                births.entry(id.to_string()).or_insert(step);
            }
        }
        let events = row["events"].as_array().ok_or_else(invalid)?;
        if step == 0 {
            if !events.is_empty() {
                return Err(invalid());
            }
            continue;
        }
        if events.len() > 2
            || row["usage"]["truncated"] != false
            || row["rng_before"] != row["rng_after"]
            || row["stats"]["step"] != step
            || row["stats"]["accepted_rewrites"] != events.len()
            || row["compile_boundary"] != step - 1
        {
            return Err(invalid());
        }
        let editors: Vec<_> = events
            .iter()
            .filter(|e| e["kind"] == "seeded_editor")
            .collect();
        if editors.len() != 1
            || editors[0]["rcf"] != Value::Null
            || editors[0]["rule_name"] != editor_name
        {
            return Err(invalid());
        }
        for event in events.iter().filter(|e| e["kind"] != "seeded_editor") {
            let rid = event["rule_id"]
                .as_u64()
                .filter(|x| *x < 2)
                .ok_or_else(invalid)? as usize;
            if event["kind"] != "world"
                || event["bound_nodes"] != json!(world)
                || event["rcf"] != installed[rid]["rcf"]
            {
                return Err(invalid());
            }
            let id = event["rcf"].as_str().ok_or_else(invalid)?;
            if let Some(birth) = births.get(id) {
                if *birth == 0
                    || step <= *birth
                    || paid_total != 2
                    || installed[rid]["labels"] != json!([context[0], context[1], 12])
                {
                    return Err(invalid());
                }
                credited += 1;
            }
        }
    }
    if rows[24]["installed"] != json!(installed)
        || receipt["summary"] != lifecycle::summarize(&initial, rows)
    {
        return Err(invalid());
    }
    Ok(credited as f64 / 24.)
}
