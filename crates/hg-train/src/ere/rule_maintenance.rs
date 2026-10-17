//! Frozen-controller maintenance panel. No fitting, replacement or native feedback.
use super::{
    graph_encoder as e, graph_encoder_confirmation as confirmation,
    rule_choice_learning::DrawStream,
};
use hg_sim_core::ere::rule_maintenance as native;
use serde_json::{json, Value};
use std::{
    path::Path,
    time::{Duration, Instant},
};
pub const PROFILE: &str = "E2-rule-maintenance-panel-v1";
pub const DRAW_SEED: u64 = 993748723001;
pub fn design() -> Value {
    serde_json::from_str(include_str!(
        "../../../../configs/ere_rule_maintenance_v1.json"
    ))
    .unwrap()
}
fn save(path: &Path, v: &Value) -> Result<(), String> {
    std::fs::write(path, serde_json::to_vec(v).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}
pub fn decisions(import: &Value) -> Result<Value, String> {
    let policies = confirmation::policies(import)?;
    let theta = &policies[0].1;
    let mut stream = DrawStream(DRAW_SEED);
    let mut rows = vec![];
    for context in native::CONTEXTS {
        let raw = native::initial_observation(context)?;
        let observation = e::observe(&raw)?;
        let input = e::input(e::Mode::Graph, &observation);
        let before = stream.0;
        let draws = [stream.next_draw(), stream.next_draw()];
        rows.push(json!({"context":context,"observation":observation,"input":input,
            "probabilities":e::probabilities(theta,&input)?,"stream_before":before,"stream_after":stream.0,
            "draws":draws,"actions":e::sample(theta,&input,draws)?}));
    }
    Ok(json!(rows))
}
pub fn run(out: &Path, import: &Value) -> Result<Value, String> {
    if out.exists() {
        return Err("fresh native output required".into());
    }
    let choices = decisions(import)?;
    // All decisions and all denominator slots committed before any native application.
    let mut schedule = vec![];
    for (world, context) in native::CONTEXTS.into_iter().enumerate() {
        for arm in native::ARMS {
            let actions = if arm == native::Arm::OracleExposedRepair {
                json!([
                    native::CHOICES[0]
                        .iter()
                        .position(|x| *x == Some(context[0]))
                        .unwrap(),
                    native::CHOICES[1]
                        .iter()
                        .position(|x| *x == Some(context[1]))
                        .unwrap()
                ])
            } else {
                choices[world]["actions"].clone()
            };
            schedule.push(
                json!({"id":schedule.len(),"world":world,"context":context,"arm":arm,
                "actions":actions,"seed":native::SEED,"horizon":native::HORIZON}),
            );
        }
    }
    std::fs::create_dir(out).map_err(|e| e.to_string())?;
    save(&out.join("decisions.json"), &choices)?;
    save(&out.join("schedule.json"), &json!(schedule))?;
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
    let mut status = vec![];
    let mut halted = false;
    for slot in &schedule {
        let id = slot["id"].as_u64().unwrap();
        let mut row = slot.clone();
        if halted {
            row["status"] = json!("not_run_after_failure");
        } else {
            let result = native::run(
                serde_json::from_value(slot["arm"].clone()).unwrap(),
                serde_json::from_value(slot["context"].clone()).unwrap(),
                serde_json::from_value(slot["actions"].clone()).unwrap(),
                native::SEED,
                deadline,
            );
            match result {
                Ok(receipt) => {
                    save(&out.join(format!("cell-{id:04}.json")), &receipt)?;
                    row["status"] = json!("complete");
                }
                Err(error) => {
                    row["status"] = json!("failed");
                    row["error"] = json!(error);
                    halted = true;
                }
            }
        }
        status.push(row);
        save(&out.join("status.json"), &json!(status))?;
    }
    let complete = !halted;
    let result = json!({"profile":PROFILE,"status":if complete {"complete"} else {"failed"},"scheduled_cells":63,
        "completed_cells":status.iter().filter(|r|r["status"]=="complete").count(),
        "requested_advances":6048,"native_work_unknown":halted,"optimizer_updates":0,
        "training_admitted":false,"scientific_acceptance":false,"sustained_claim":false});
    save(&out.join("manifest.json"), &result)?;
    if !complete {
        return Err("native failure retained without retry; remaining slots not run".into());
    }
    Ok(result)
}
