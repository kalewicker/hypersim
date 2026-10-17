//! Ordinary-inventory relational readout and shared linear policy. No optimizer.
use super::rule_choice as softmax;
use hg_sim_core::ere::lifecycle_fixture as lifecycle;
use hg_sim_core::ere::rule_relational as native;
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

pub const PARAMS: usize = 16;

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct Observation {
    features: [[f64; 3]; 2],
    marginals: Vec<u32>,
}
impl Observation {
    pub fn features(&self) -> [[f64; 3]; 2] {
        self.features
    }
}

fn endpoint(id: &Value) -> Result<String, String> {
    if id.as_object().map(|o| o.len()) != Some(2) {
        return Err("invalid node reference".into());
    }
    Ok(format!(
        "NodeId({}v{})",
        id["idx"].as_u64().ok_or("invalid index")?,
        id["version"]
            .as_u64()
            .filter(|v| *v > 0)
            .ok_or("invalid version")?
    ))
}

/// Only initial ordinary nodes/edges are used. Metadata labels, properties,
/// allocation values, cached incidence indexes and receipt context are not cues.
pub fn observe(inventory: &Value) -> Result<Observation, String> {
    let invalid = || "invalid initial relational observation".to_string();
    if inventory["step"] != 0 {
        return Err(invalid());
    }
    let mut labels = BTreeMap::new();
    let mut ordinary = BTreeMap::new();
    for row in inventory["nodes"].as_array().ok_or_else(invalid)? {
        let id = row[0].as_str().ok_or_else(invalid)?.to_string();
        let label = row[1]["data"]["label"]
            .as_u64()
            .and_then(|l| u32::try_from(l).ok())
            .ok_or_else(invalid)?;
        if labels.insert(id.clone(), label).is_some() {
            return Err(invalid());
        }
        if label < native::ORDINARY_CEILING {
            if ordinary.insert(label, id).is_some() {
                return Err(invalid());
            }
        }
    }
    let mut marginals: Vec<_> = native::ORDINARY_LABELS.to_vec();
    marginals.sort();
    if ordinary.keys().copied().collect::<Vec<_>>() != marginals {
        return Err(invalid());
    }
    let mut adjacent: BTreeMap<String, BTreeSet<String>> = ordinary
        .values()
        .map(|id| (id.clone(), BTreeSet::new()))
        .collect();
    let mut seen = BTreeSet::new();
    let mut count = 0;
    for row in inventory["edges"].as_array().ok_or_else(invalid)? {
        if !seen.insert(row[0].as_str().ok_or_else(invalid)?.to_string()) {
            return Err(invalid());
        }
        let nodes = row[1]["nodes"].as_array().ok_or_else(invalid)?;
        let ids: Vec<_> = nodes.iter().map(endpoint).collect::<Result<_, _>>()?;
        if ids.iter().any(|n| !labels.contains_key(n)) {
            return Err(invalid());
        }
        let n = ids.iter().filter(|id| adjacent.contains_key(*id)).count();
        if n == 0 {
            continue;
        } // Irrelevant metadata never becomes an input feature.
        if n != ids.len() || ids.len() != 2 || ids[0] == ids[1] || row[1]["label"] != 0 {
            return Err(invalid());
        }
        if !adjacent.get_mut(&ids[0]).unwrap().insert(ids[1].clone()) {
            return Err(invalid());
        }
        adjacent.get_mut(&ids[1]).unwrap().insert(ids[0].clone());
        count += 1;
    }
    if count != 2 {
        return Err(invalid());
    }
    let anchor = ordinary.get(&12).unwrap();
    let mut distances = BTreeMap::from([(anchor.clone(), 0)]);
    let mut frontier = vec![anchor.clone()];
    while !frontier.is_empty() {
        let mut next = vec![];
        for n in frontier {
            for other in &adjacent[&n] {
                if !distances.contains_key(other) {
                    distances.insert(other.clone(), distances[&n] + 1);
                    next.push(other.clone());
                }
            }
        }
        frontier = next;
    }
    let features = std::array::from_fn(|slot| {
        std::array::from_fn(|k| {
            let label = native::CHOICES[slot][k + 1].unwrap();
            if distances.get(&ordinary[&label]) == Some(&(2 - slot)) {
                1.
            } else {
                0.
            }
        })
    });
    if distances.len() != 3 || features.iter().any(|f| f.iter().sum::<f64>() != 1.) {
        return Err(invalid());
    }
    Ok(Observation {
        features,
        marginals,
    })
}

fn logits(theta: &[f64], o: &Observation, slot: usize) -> Result<[f64; 12], String> {
    if theta.len() != PARAMS || theta.iter().any(|t| !t.is_finite()) {
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

pub fn probabilities(theta: &[f64], o: &Observation) -> Result<[[f64; 4]; 2], String> {
    Ok([
        softmax::probabilities(&logits(theta, o, 0)?, 10)?,
        softmax::probabilities(&logits(theta, o, 1)?, 10)?,
    ])
}
pub fn sample(theta: &[f64], o: &Observation, draws: [f64; 2]) -> Result<[usize; 2], String> {
    Ok([
        softmax::sample(&logits(theta, o, 0)?, 10, draws[0])?,
        softmax::sample(&logits(theta, o, 1)?, 10, draws[1])?,
    ])
}
pub fn log_probability(theta: &[f64], o: &Observation, actions: [usize; 2]) -> Result<f64, String> {
    let p = softmax::log_probability(&logits(theta, o, 0)?, 10, actions[0])?
        + softmax::log_probability(&logits(theta, o, 1)?, 10, actions[1])?;
    if !p.is_finite() {
        return Err("joint likelihood overflow".into());
    }
    Ok(p)
}
pub fn score_gradient(
    theta: &[f64],
    o: &Observation,
    actions: [usize; 2],
) -> Result<[f64; PARAMS], String> {
    log_probability(theta, o, actions)?;
    let p = probabilities(theta, o)?;
    let mut g = [0.; PARAMS];
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

pub fn probe_theta() -> [f64; PARAMS] {
    let mut t = [0.; PARAMS];
    for k in 0..3 {
        t[4 + (k + 1) * 3 + k] = 4.;
    }
    t
}
/// Strict fixed-profile interface check. Full typed graph/budget reconstruction
/// remains the independent auditor's responsibility; this is not attestation.
pub fn reward(receipt: &Value) -> Result<f64, String> {
    let invalid = || "invalid/incomplete relational receipt".to_string();
    let observation = observe(&receipt["initial_inventory"])?;
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
    for slot in 0..2 {
        let required = native::CHOICES[slot]
            .iter()
            .position(|c| *c == Some(context[slot]))
            .ok_or_else(invalid)?;
        if observation.features[slot][required - 1] != 1. {
            return Err(invalid());
        }
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
