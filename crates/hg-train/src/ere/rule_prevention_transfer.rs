//! Fixed frozen-policy transfer into the paid-prevention environment; no fitting.
use super::{
    graph_encoder as e, graph_encoder_confirmation as confirmation,
    rule_choice_learning::DrawStream,
};
use hg_sim_core::ere::rule_prevention_transfer as native;
use serde_json::{json, Value};
use std::{
    path::Path,
    time::{Duration, Instant},
};
pub const PROFILE: &str = "E2-rule-prevention-transfer-panel-v1";
pub const DRAW_SEED: u64 = 993748724001;
pub const CONDITIONS: [&str; 8] = [
    "learned_paid",
    "learned_exhaustion",
    "learned_unpaid",
    "learned_disabled",
    "learned_no_hazard",
    "masked_paid",
    "blind_paid",
    "oracle_paid",
];
pub fn design() -> Value {
    serde_json::from_str(include_str!(
        "../../../../configs/ere_rule_prevention_transfer_v1.json"
    ))
    .unwrap()
}
fn save(path: &Path, v: &Value) -> Result<(), String> {
    std::fs::write(path, serde_json::to_vec(v).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}
/// Pure graph observation/inference: all paired draws and choices precede physics.
pub fn decisions(import: &Value) -> Result<Value, String> {
    let policies = confirmation::policies(import)?;
    let mut stream = DrawStream(DRAW_SEED);
    let mut rows = vec![];
    for context in native::CONTEXTS {
        let raw = native::initial_observation(context)?;
        let observation = e::observe(&raw)?;
        let before = stream.0;
        let draws = [stream.next_draw(), stream.next_draw()];
        let mut choices = vec![];
        for index in [0, 3, 1] {
            let (mode, theta) = &policies[index];
            let input = e::input(*mode, &observation);
            choices.push(json!({"policy_index":index,"input":input,
                "probabilities":e::probabilities(theta,&input)?,"actions":e::sample(theta,&input,draws)?}));
        }
        rows.push(json!({"context":context,"observation":observation,
            "stream_before":before,"stream_after":stream.0,"draws":draws,"policies":choices}));
    }
    Ok(json!(rows))
}
pub fn schedule(choices: &Value) -> Result<Value, String> {
    if choices.as_array().map(Vec::len) != Some(9) {
        return Err("nine worlds required".into());
    }
    let mut rows = vec![];
    let arms = [
        native::Arm::Paid,
        native::Arm::Exhaustion,
        native::Arm::Unpaid,
        native::Arm::Disabled,
        native::Arm::NoHazard,
        native::Arm::Paid,
        native::Arm::Paid,
        native::Arm::Paid,
    ];
    for (world, context) in native::CONTEXTS.into_iter().enumerate() {
        if choices[world]["context"] != json!(context) {
            return Err("world order mismatch".into());
        }
        for (i, condition) in CONDITIONS.into_iter().enumerate() {
            let actions = match i {
                0..=4 => choices[world]["policies"][0]["actions"].clone(),
                5 => choices[world]["policies"][1]["actions"].clone(),
                6 => choices[world]["policies"][2]["actions"].clone(),
                _ => json!([
                    native::CHOICES[0]
                        .iter()
                        .position(|x| *x == Some(context[0]))
                        .unwrap(),
                    native::CHOICES[1]
                        .iter()
                        .position(|x| *x == Some(context[1]))
                        .unwrap()
                ]),
            };
            let parsed: [usize; 2] =
                serde_json::from_value(actions.clone()).map_err(|e| e.to_string())?;
            if parsed.iter().any(|a| *a >= 4) {
                return Err("invalid actions".into());
            }
            rows.push(json!({"id":rows.len(),"world":world,"context":context,
                "condition":condition,"arm":arms[i],"actions":actions,"seed":native::SEED,"horizon":96}));
        }
    }
    Ok(json!(rows))
}
pub fn run(out: &Path, import: &Value) -> Result<Value, String> {
    if out.exists() {
        return Err("fresh native output required".into());
    }
    let choices = decisions(import)?;
    let preflight: Value = serde_json::from_slice(
        &std::fs::read(
            out.parent()
                .ok_or("missing archive parent")?
                .join("decision_preflight.json"),
        )
        .map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    if choices != preflight {
        return Err("prelaunch decisions changed".into());
    }
    let slots = schedule(&choices)?;
    let slots = slots.as_array().unwrap();
    std::fs::create_dir(out).map_err(|e| e.to_string())?;
    save(&out.join("decisions.json"), &choices)?;
    save(&out.join("schedule.json"), &json!(slots))?;
    let mut statuses = slots
        .iter()
        .map(|slot| {
            let mut r = slot.clone();
            r["status"] = json!("pending");
            r
        })
        .collect::<Vec<_>>();
    save(&out.join("status.json"), &json!(statuses))?;
    save(
        &out.join("manifest.json"),
        &json!({"status":"running","profile":PROFILE,"design":design(),
        "optimizer_updates":0,"training_admitted":false,"scientific_acceptance":false,"sustained_claim":false}),
    )?;
    std::fs::copy(
        std::env::current_exe().map_err(|e| e.to_string())?,
        out.join("executed_test_binary"),
    )
    .map_err(|e| e.to_string())?;
    let deadline = Instant::now() + Duration::from_secs(180);
    let mut halted = false;
    for slot in slots {
        let id = slot["id"].as_u64().unwrap() as usize;
        if halted {
            statuses[id]["status"] = json!("not_run_after_failure");
        } else {
            let result = native::run(
                serde_json::from_value(slot["arm"].clone()).unwrap(),
                serde_json::from_value(slot["context"].clone()).unwrap(),
                serde_json::from_value(slot["actions"].clone()).unwrap(),
                deadline,
            );
            match result {
                Ok(receipt) => {
                    save(&out.join(format!("cell-{id:04}.json")), &receipt)?;
                    statuses[id]["status"] = json!("complete");
                }
                Err(error) => {
                    statuses[id]["status"] = json!("failed");
                    statuses[id]["error"] = json!(error);
                    halted = true;
                }
            }
        }
        save(&out.join("status.json"), &json!(statuses))?;
    }
    let result = json!({"profile":PROFILE,"status":if halted {"failed"} else {"complete"},
        "scheduled_cells":72,"completed_cells":statuses.iter().filter(|r|r["status"]=="complete").count(),
        "requested_advances":6912,"native_work_unknown":halted,"optimizer_updates":0,
        "training_admitted":false,"scientific_acceptance":false,"sustained_claim":false});
    save(&out.join("manifest.json"), &result)?;
    if halted {
        return Err("native failure retained; no retry".into());
    }
    Ok(result)
}
