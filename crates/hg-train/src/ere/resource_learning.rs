//! Frozen on-policy runner. Engine ownership stays in sim-core; no cached rewards.
use super::{resource_adapter as a, resource_timed_policy as p};
use hg_sim_core::ere::{defense_allocation as old, resource_timed_bridge as n};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::Path,
    time::{Duration, Instant},
};

pub const PROFILE: &str = "E2-resource-on-policy-runner-v1";
pub const SEEDS: [u64; 4] = [993748721101, 993748721102, 993748721103, 993748721104];
pub const MODES: [a::Mode; 2] = [a::Mode::Full, a::Mode::CuesMasked];
pub const MAX_ADVANCES: u64 = 238080;
pub const MAX_SEALS: u64 = 7440;
pub const MAX_UPDATES: u64 = 800;
pub const OUTPUT_LIMIT: u64 = 1073741824;
pub const CELL_RESERVE: u64 = 2097152;
pub const DEADLINE_SECONDS: u64 = 7200;

pub struct DrawStream(u64);
impl DrawStream {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }
    pub fn next(&mut self) -> f64 {
        self.0 = self.0.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        ((z ^ (z >> 31)) >> 11) as f64 / (1u64 << 53) as f64
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Slot {
    pub id: usize,
    pub phase: String,
    pub model: Option<usize>,
    pub seed: Option<u64>,
    pub mode: a::Mode,
    pub update: Option<usize>,
    pub state: usize,
    pub draw: f64,
    pub baseline: Option<usize>,
}
pub fn schedule() -> Vec<Slot> {
    let mut slots = vec![];
    for (replicate, seed) in SEEDS.into_iter().enumerate() {
        for (arm, mode) in MODES.into_iter().enumerate() {
            let mut stream = DrawStream::new(seed);
            for update in 1..=100 {
                for state in 0..8 {
                    slots.push(Slot {
                        id: slots.len(),
                        phase: "training".into(),
                        model: Some(replicate * 2 + arm),
                        seed: Some(seed),
                        mode,
                        update: Some(update),
                        state,
                        draw: stream.next(),
                        baseline: None,
                    });
                }
            }
        }
    }
    for (replicate, seed) in SEEDS.into_iter().enumerate() {
        for (arm, mode) in MODES.into_iter().enumerate() {
            for state in 0..8 {
                for i in 0..16 {
                    slots.push(Slot {
                        id: slots.len(),
                        phase: "confirmation".into(),
                        model: Some(replicate * 2 + arm),
                        seed: Some(seed),
                        mode,
                        update: None,
                        state,
                        draw: (i as f64 + 0.5) / 16.,
                        baseline: None,
                    });
                }
            }
        }
    }
    for state in 0..8 {
        for action in 0..2 {
            slots.push(Slot {
                id: slots.len(),
                phase: "baseline".into(),
                model: None,
                seed: None,
                mode: a::Mode::CuesMasked,
                update: None,
                state,
                draw: if action == 0 { 0.25 } else { 0.75 },
                baseline: Some(action),
            });
        }
    }
    slots
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Sample {
    pub state: usize,
    pub input: [f64; 4],
    pub action: usize,
    pub utility: f64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Update {
    pub before: [f64; 4],
    pub after: [f64; 4],
    pub contributions: Vec<[f64; 4]>,
    pub mean_gradient: [f64; 4],
    pub norm: f64,
    pub norm_scale: f64,
    pub clipped_gradient: [f64; 4],
}
pub fn clip(g: [f64; 4]) -> Result<(f64, f64, [f64; 4]), String> {
    if g.iter().any(|v| !v.is_finite()) {
        return Err("nonfinite gradient".into());
    }
    let norm = g.iter().fold(0f64, |n, v| n.hypot(*v));
    if !norm.is_finite() {
        return Err("nonfinite gradient norm".into());
    }
    let scale = if norm > 1. { 1. / norm } else { 1. };
    Ok((norm, scale, g.map(|v| v * scale)))
}
pub fn project(t: [f64; 4]) -> Result<[f64; 4], String> {
    if t.iter().any(|v| !v.is_finite()) {
        return Err("nonfinite parameters".into());
    }
    Ok(t.map(|v| v.clamp(-8., 8.)))
}
pub fn batch_update(theta: [f64; 4], samples: &[Sample]) -> Result<Update, String> {
    if project(theta)? != theta || samples.len() != 8 {
        return Err("requires bounded theta and full eight-cell batch".into());
    }
    let mut contributions = vec![];
    let mut mean = [0.; 4];
    for (state, s) in samples.iter().enumerate() {
        if s.state != state
            || !s.utility.is_finite()
            || !(0. ..=20.).contains(&s.utility)
            || ![0., 1.].contains(&s.input[1])
        {
            return Err("invalid batch sample/domain/order".into());
        }
        let score = a::score_gradient(&theta, &s.input, s.action)?;
        let credit = if s.input[1] == 1. {
            (s.utility - 9.5) / 20.
        } else {
            0.
        };
        let g = score.map(|v| v * credit);
        for i in 0..4 {
            mean[i] += g[i] / 8.;
        }
        contributions.push(g);
    }
    let (norm, norm_scale, clipped_gradient) = clip(mean)?;
    let after = project(std::array::from_fn(|i| theta[i] + clipped_gradient[i]))?;
    Ok(Update {
        before: theta,
        after,
        contributions,
        mean_gradient: mean,
        norm,
        norm_scale,
        clipped_gradient,
    })
}
/// Actual native event/property credit only. No response-table or scenario lookup.
pub fn utility(r: &Value) -> Result<f64, String> {
    if r["profile"] != n::PROFILE
        || r["completed_steps"] != 32
        || r["native_advances"] != 32
        || r["seal_applications"] != 1
        || r["decision_calls"] != 1
    {
        return Err("incomplete/non-native credit receipt".into());
    }
    let identity: String = n::candidate(false, [10, 11])
        .rcf_bytes()
        .iter()
        .map(|v| format!("{v:02x}"))
        .collect();
    let rows = r["timeline"]
        .as_array()
        .filter(|v| v.len() == 33)
        .ok_or("invalid native timeline")?;
    let mut fires = 0;
    for (step, row) in rows.iter().enumerate() {
        if row["step"].as_u64() != Some(step as u64) {
            return Err("invalid native step order".into());
        }
        for event in row["events"].as_array().ok_or("missing native events")? {
            if step >= 17 && event["kind"] == "world" && event["rcf"].as_str() == Some(&identity) {
                fires += 1;
            }
        }
    }
    let nodes = r["final_inventory"]["nodes"]
        .as_array()
        .ok_or("missing native final inventory")?;
    let payoff: Vec<_> = nodes
        .iter()
        .filter(|r| r[1]["data"]["label"] == old::PAYOFF)
        .collect();
    if payoff.len() != 1 {
        return Err("missing/duplicate native payoff".into());
    }
    let props = payoff[0][1]["data"]["properties"]
        .as_array()
        .ok_or("missing credit properties")?;
    let credit: Vec<_> = props.iter().filter(|r| r[0] == old::CREDITS).collect();
    if credit.len() != 1 {
        return Err("missing/duplicate native credits".into());
    }
    let credits = credit[0][1]["Int"]
        .as_i64()
        .filter(|v| [0, 2, 12].contains(v))
        .ok_or("invalid native credit type/domain")?;
    if r["summary"]["harvested_credits"].as_i64() != Some(credits) {
        return Err("native credit summary mismatch".into());
    }
    let u = fires as f64 + credits as f64;
    if !(0. ..=20.).contains(&u) {
        return Err("native utility outside frozen domain".into());
    }
    Ok(u)
}
pub fn gzip_bytes(value: &Value) -> Result<Vec<u8>, String> {
    let mut writer = flate2::GzBuilder::new()
        .mtime(0)
        .write(Vec::new(), flate2::Compression::fast());
    serde_json::to_writer(&mut writer, value).map_err(|e| e.to_string())?;
    writer.finish().map_err(|e| e.to_string())
}
fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut f = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    f.write_all(bytes)
        .and_then(|_| f.sync_all())
        .map_err(|e| e.to_string())
}
fn manifest(path: &Path, v: &Value) -> Result<(), String> {
    let tmp = path.with_extension("tmp");
    let mut f = File::create(&tmp).map_err(|e| e.to_string())?;
    serde_json::to_writer(&mut f, v).map_err(|e| e.to_string())?;
    f.sync_all().map_err(|e| e.to_string())?;
    fs::rename(tmp, path).map_err(|e| e.to_string())
}
fn log(ledger: &mut File, event: Value) -> Result<(), String> {
    serde_json::to_writer(&mut *ledger, &event).map_err(|e| e.to_string())?;
    ledger
        .write_all(b"\n")
        .and_then(|_| ledger.sync_all())
        .map_err(|e| e.to_string())
}
/// Explicit acknowledgement required. Only the CLI's campaign branch calls this.
pub fn campaign(root: &Path, ack: u64, bootstrap_bytes: u64) -> Result<Value, String> {
    if ack != MAX_ADVANCES {
        return Err("explicit frozen native-budget acknowledgement required".into());
    }
    if bootstrap_bytes >= OUTPUT_LIMIT - CELL_RESERVE {
        return Err("bootstrap exhausts output budget".into());
    }
    if directory_bytes(root)? != bootstrap_bytes {
        return Err("bootstrap accounting mismatch".into());
    }
    let out = root.join("campaign");
    fs::create_dir(&out).map_err(|e| e.to_string())?;
    fs::create_dir(out.join("cells")).map_err(|e| e.to_string())?;
    fs::create_dir(out.join("updates")).map_err(|e| e.to_string())?;
    let slots = schedule();
    let supplied: Vec<Slot> =
        serde_json::from_slice(&fs::read(root.join("schedule.json")).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    if slots != supplied {
        return Err("frozen schedule mismatch".into());
    }
    let mut ledger = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(out.join("ledger.jsonl"))
        .map_err(|e| e.to_string())?;
    let deadline = Instant::now() + Duration::from_secs(DEADLINE_SECONDS);
    let mut theta = [[0.; 4]; 8];
    let mut samples = vec![];
    let mut m = json!({"profile":PROFILE,"status":"running","completed_cells":0,"native_advances":0,"seal_applications":0,"optimizer_updates":0,"native_work_unknown":false,
        "active_slot":null,"bootstrap_bytes":bootstrap_bytes,"compressed_receipt_bytes":0,"total_output_bytes_lower_bound":bootstrap_bytes,
        "scientific_acceptance":false,"training_admitted":false,"sustained_claim":false,"independent_physics_replay":false});
    let result = (|| -> Result<(), String> {
        for slot in slots {
            if Instant::now() >= deadline {
                return Err("campaign deadline before cell".into());
            }
            if m["native_advances"].as_u64().unwrap() + 32 > MAX_ADVANCES
                || m["seal_applications"].as_u64().unwrap() + 1 > MAX_SEALS
            {
                return Err("native budget before cell".into());
            }
            // Account actual whole-root files; reserve enough for one bounded gzip cell and ledger/manifest/update overhead.
            let used = directory_bytes(root)?;
            if used + CELL_RESERVE > OUTPUT_LIMIT {
                return Err("output budget before cell".into());
            }
            let parameters = slot.model.map(|i| theta[i]).unwrap_or([0.; 4]);
            m["active_slot"] = json!(slot.id);
            m["native_work_unknown"] = json!(true);
            manifest(&out.join("manifest.json"), &m)?;
            log(
                &mut ledger,
                json!({"kind":"start","id":slot.id,"theta":parameters}),
            )?;
            let mut policy = Value::Null;
            let native = n::run(
                old::SCENARIOS[slot.state / 2],
                old::RESOURCES[slot.state % 2],
                deadline,
                |inventory| {
                    let (d, v) = p::choose(&parameters, slot.mode, inventory, slot.draw)?;
                    policy = v;
                    Ok(d)
                },
            );
            m["native_work_unknown"] = json!(false);
            let receipt = match native {
                Ok(r) => {
                    m["native_advances"] = json!(m["native_advances"].as_u64().unwrap() + 32);
                    m["seal_applications"] = json!(m["seal_applications"].as_u64().unwrap() + 1);
                    r
                }
                Err(fail) => {
                    m["native_advances"] =
                        json!(m["native_advances"].as_u64().unwrap() + fail.native_advances);
                    m["seal_applications"] =
                        json!(m["seal_applications"].as_u64().unwrap() + fail.seal_applications);
                    log(
                        &mut ledger,
                        json!({"kind":"failure","id":slot.id,"failure":fail}),
                    )?;
                    return Err(fail.message);
                }
            };
            let credited = utility(&receipt);
            let cell = json!({"slot":slot,"policy":policy,"receipt":receipt,
                "actual_utility":credited.as_ref().ok(),"credit_error":credited.as_ref().err()});
            let bytes = gzip_bytes(&cell)?;
            if bytes.len() as u64 > CELL_RESERVE / 2 {
                return Err("cell exceeds reserved compressed size".into());
            }
            write_new(
                &out.join(format!("cells/cell-{:04}.json.gz", slot.id)),
                &bytes,
            )?;
            m["compressed_receipt_bytes"] =
                json!(m["compressed_receipt_bytes"].as_u64().unwrap() + bytes.len() as u64);
            m["completed_cells"] = json!(m["completed_cells"].as_u64().unwrap() + 1);
            let reward = match credited {
                Ok(reward) => reward,
                Err(error) => {
                    log(
                        &mut ledger,
                        json!({"kind":"credit_failure","id":slot.id,"error":error}),
                    )?;
                    return Err(error);
                }
            };
            log(
                &mut ledger,
                json!({"kind":"complete","id":slot.id,"actual_utility":reward,"compressed_bytes":bytes.len()}),
            )?;
            if slot.phase == "training" {
                samples.push(Sample {
                    state: slot.state,
                    input: serde_json::from_value(policy["input"].clone())
                        .map_err(|e| e.to_string())?,
                    action: policy["sampled_action"].as_u64().ok_or("missing action")? as usize,
                    utility: reward,
                });
                if slot.state == 7 {
                    let model = slot.model.unwrap();
                    let update = batch_update(parameters, &samples)?;
                    let index = m["optimizer_updates"].as_u64().unwrap();
                    if index >= MAX_UPDATES {
                        return Err("optimizer budget".into());
                    }
                    let data = json!({"index":index,"model":model,"update":slot.update,"samples":samples,"arithmetic":update});
                    write_new(
                        &out.join(format!("updates/update-{index:04}.json")),
                        &serde_json::to_vec(&data).unwrap(),
                    )?;
                    theta[model] = update.after;
                    m["optimizer_updates"] = json!(index + 1);
                    samples.clear();
                    log(
                        &mut ledger,
                        json!({"kind":"update","index":index,"model":model,"after":theta[model]}),
                    )?;
                }
            }
            m["active_slot"] = Value::Null;
            m["total_output_bytes_lower_bound"] = json!(directory_bytes(root)?);
            manifest(&out.join("manifest.json"), &m)?;
        }
        if !samples.is_empty()
            || m["native_advances"] != MAX_ADVANCES
            || m["seal_applications"] != MAX_SEALS
            || m["optimizer_updates"] != MAX_UPDATES
        {
            return Err("incomplete fixed denominator".into());
        }
        write_new(
            &out.join("endpoints.json"),
            &serde_json::to_vec(&theta).unwrap(),
        )?;
        Ok(())
    })();
    m["status"] = json!(if result.is_ok() { "complete" } else { "failed" });
    m["error"] = result
        .as_ref()
        .err()
        .map(|s| json!(s))
        .unwrap_or(Value::Null);
    manifest(&out.join("manifest.json"), &m)?;
    result.map(|_| m)
}
pub fn directory_bytes(root: &Path) -> Result<u64, String> {
    let mut total = 0;
    for entry in fs::read_dir(root).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let kind = entry.file_type().map_err(|e| e.to_string())?;
        if kind.is_symlink() {
            return Err("symlink in campaign output".into());
        }
        total += if kind.is_dir() {
            directory_bytes(&entry.path())?
        } else {
            entry.metadata().map_err(|e| e.to_string())?.len()
        };
    }
    Ok(total)
}
