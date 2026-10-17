//! Authored diagnostic descent and standardized assays. Never evolution evidence.
use super::*;
pub const PROFILE: &str = "cumulative-lineage-intervention-readiness-v1";

pub fn witness() -> Result<Value, String> {
    let start = native_work();
    let contract: Value = serde_json::from_str(include_str!("../lineage_contract.json"))
        .map_err(|e| e.to_string())?;
    let seed = contract["development_seed"].as_u64().unwrap();
    let mut controls = Vec::new();
    for spec in contract["controls"].as_array().unwrap() {
        let stages: Vec<i64> =
            serde_json::from_value(spec["founder"].clone()).map_err(|e| e.to_string())?;
        let mut c = Cell::new(&stages, Arm::Endogenous, seed)?;
        c.graph.get_node_mut(c.editor).unwrap().set_property(
            EDIT_MODE,
            PropertyValue::Int(spec["editor_mode"].as_i64().unwrap()),
        );
        let mut generations = Vec::new();
        for generation in 0..4 {
            let initial = c.state()?;
            // Resource-before-edit, matching the shared world. This is one of
            // the four registered cycles, never an extra warmup or endowment.
            c.cycle(false)?;
            for serial in spec["edits_by_generation"][generation].as_array().unwrap() {
                c.edit(false, Some(serial.as_i64().unwrap()))?;
            }
            for _ in 0..3 {
                c.cycle(false)?;
            }
            let child = if generation < 3 {
                Some(c.reproduce(generation as u64 + 1)?)
            } else {
                None
            };
            generations.push(json!({"initial":initial,"final":c.state()?,"receipts":c.receipts}));
            if let Some(next) = child {
                c = next;
            }
        }
        controls.push(json!({"name":spec["name"],"generations":generations}));
    }
    Ok(
        json!({"profile":PROFILE,"contract":contract,"controls":controls,"native_rewrites":native_work()-start,
        "discovery_evidence":false,"shared_economy_claim":false,"readiness_gate_complete":false,
        "scientific_acceptance":false,"evolution_demonstrated":false,"campaign_launch_authorized":false}),
    )
}

pub fn assays(requests: &Value) -> Result<Value, String> {
    let start = native_work();
    let contract: Value = serde_json::from_str(include_str!("../lineage_contract.json"))
        .map_err(|e| e.to_string())?;
    if requests.as_object().map(|m| m.len()) != Some(2) || requests["profile"] != PROFILE {
        return Err("assay request fields/profile".into());
    }
    let inputs = requests["assays"].as_array().ok_or("assay list")?;
    if inputs.len() > 128 {
        return Err("complete assay request cap".into());
    }
    // Validate the entire denominator before native work.
    for (id, r) in inputs.iter().enumerate() {
        let stages: Vec<i64> =
            serde_json::from_value(r["stages"].clone()).map_err(|e| e.to_string())?;
        if r.as_object().map(|m| m.len()) != Some(3)
            || r["id"].as_u64() != Some(id as u64)
            || !contract["assay"]["roles"]
                .as_array()
                .unwrap()
                .contains(&r["role"])
            || stages.is_empty()
            || stages.len() > 8
            || stages.iter().any(|s| !(-1..=4).contains(s))
        {
            return Err("assay input domain".into());
        }
    }
    let seed = contract["development_seed"].as_u64().unwrap();
    let mut results = Vec::new();
    for r in inputs {
        let stages: Vec<i64> = serde_json::from_value(r["stages"].clone()).unwrap();
        let mut c = Cell::new(&stages, Arm::Frozen, seed)?;
        let initial = c.state()?;
        for _ in 0..4 {
            c.cycle(false)?;
        }
        let status = if c.capital()? >= 3 + stages.len() as i64 + 8 {
            c.reproduce(1)?;
            "complete"
        } else {
            "unfunded_copy"
        };
        let score = c.capital()? + c.transferred - 40;
        results.push(json!({"request":r,"initial":initial,"final":c.state()?,"receipts":c.receipts,"score":score,"status":status}));
    }
    Ok(
        json!({"profile":PROFILE,"results":results,"native_rewrites":native_work()-start,
        "readiness_gate_complete":false,"scientific_acceptance":false,"evolution_demonstrated":false,
        "campaign_launch_authorized":false}),
    )
}
