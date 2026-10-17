//! Durable native action boundaries. No uncertain action is retried.
pub mod campaign;
use super::*;
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
pub const PROFILE: &str = "cumulative-durable-journal-readiness-v1";
pub fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn error(e: impl std::fmt::Display) -> String {
    e.to_string()
}
struct Writer {
    dir: PathBuf,
    sequence: usize,
    head: Option<String>,
    bytes: usize,
    limit: usize,
}
impl Writer {
    fn new(dir: &Path, limit: usize) -> Result<Self, String> {
        fs::create_dir(dir).map_err(error)?;
        File::open(dir.parent().ok_or("journal parent")?)
            .map_err(error)?
            .sync_all()
            .map_err(error)?;
        Ok(Self {
            dir: dir.to_path_buf(),
            sequence: 0,
            head: None,
            bytes: 0,
            limit,
        })
    }
    fn publish(&mut self, value: Value, torn: bool) -> Result<(), String> {
        let v = json!({"profile":PROFILE,"sequence":self.sequence,"previous_sha256":self.head,"value":value});
        let mut bytes = serde_json::to_vec(&v).map_err(error)?;
        bytes.push(b'\n');
        if self.sequence >= 2048 || self.bytes + bytes.len() > self.limit {
            return Err("journal storage/record cap".into());
        }
        let staging = self.dir.join(format!(".staging-{:08}.json", self.sequence));
        let target = self.dir.join(format!("{:08}.json", self.sequence));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&staging)
            .map_err(error)?;
        if torn {
            file.write_all(&bytes[..bytes.len() / 2]).map_err(error)?;
            file.sync_all().map_err(error)?;
            File::open(&self.dir)
                .map_err(error)?
                .sync_all()
                .map_err(error)?;
            return Err("declared torn-result interruption".into());
        }
        file.write_all(&bytes).map_err(error)?;
        file.sync_all().map_err(error)?;
        fs::hard_link(&staging, &target).map_err(error)?;
        File::open(&self.dir)
            .map_err(error)?
            .sync_all()
            .map_err(error)?;
        fs::remove_file(&staging).map_err(error)?;
        File::open(&self.dir)
            .map_err(error)?
            .sync_all()
            .map_err(error)?;
        self.head = Some(sha(&bytes));
        self.sequence += 1;
        self.bytes += bytes.len();
        Ok(())
    }
}
pub fn read(dir: &Path, expected: &str) -> Result<Vec<Value>, String> {
    let mut files = fs::read_dir(dir)
        .map_err(error)?
        .map(|e| e.map_err(error))
        .collect::<Result<Vec<_>, _>>()?;
    files.sort_by_key(|e| e.file_name());
    if files.is_empty() || files.len() > 2048 {
        return Err("source journal record domain".into());
    }
    let mut head = None;
    let mut values = Vec::new();
    let mut bytes_total = 0;
    for (i, e) in files.iter().enumerate() {
        if e.file_name().to_str() != Some(&format!("{i:08}.json"))
            || !e.file_type().map_err(error)?.is_file()
        {
            return Err("unsafe staging or source file set".into());
        }
        let bytes = fs::read(e.path()).map_err(error)?;
        bytes_total += bytes.len();
        if bytes_total > 33554432 {
            return Err("source journal byte cap".into());
        }
        let envelope: Value = serde_json::from_slice(&bytes).map_err(error)?;
        if envelope.as_object().map(|m| m.len()) != Some(4)
            || envelope["profile"] != PROFILE
            || envelope["sequence"].as_u64() != Some(i as u64)
            || envelope["previous_sha256"] != json!(head)
        {
            return Err("source journal chain".into());
        }
        head = Some(sha(&bytes));
        values.push(envelope["value"].clone());
    }
    if head.as_deref() != Some(expected) {
        return Err("source journal known head".into());
    }
    Ok(values)
}
fn world_contract() -> Value {
    serde_json::from_str(include_str!("../../../../integration_contract.json")).unwrap()
}
fn valid_assays(v: &Value) -> Result<(), String> {
    let c: Value = serde_json::from_str(include_str!("../../../../lineage_contract.json")).unwrap();
    let inputs = v["assays"].as_array().ok_or("assay list")?;
    if v.as_object().map(|m| m.len()) != Some(2)
        || v["profile"] != c["profile"]
        || inputs.len() > 128
    {
        return Err("assay request profile/cap".into());
    }
    for (i, r) in inputs.iter().enumerate() {
        let stages: Vec<i64> = serde_json::from_value(r["stages"].clone()).map_err(error)?;
        if r.as_object().map(|m| m.len()) != Some(3)
            || r["id"].as_u64() != Some(i as u64)
            || !c["assay"]["roles"].as_array().unwrap().contains(&r["role"])
            || stages.is_empty()
            || stages.len() > 8
            || stages.iter().any(|s| !(-1..=4).contains(s))
        {
            return Err("assay input denominator".into());
        }
    }
    Ok(())
}
fn validate_request(r: &Value) -> Result<(), String> {
    if r.as_object().map(|m| m.len()) != Some(12)
        || r["profile"] != PROFILE
        || r["world_contract"] != world_contract()
        || r["source_identity"].as_object().map(|m| m.len()) != Some(2)
        || r["source_identity"]["binary_sha256"]
            .as_str()
            .is_none_or(|s| s.len() != 64)
        || r["source_identity"]["source_sha256"]
            .as_str()
            .is_none_or(|s| s.len() != 64)
        || r["native_budget"].as_u64().is_none_or(|n| n > 6000)
        || r["execution_allowance"].as_u64().is_none_or(|n| n > 6000)
        || r["journal_byte_allowance"]
            .as_u64()
            .is_none_or(|n| n > 33554432)
    {
        return Err("journal request fields/identity/cap".into());
    }
    let mode = r["mode"].as_str().ok_or("journal mode")?;
    if ![
        "worlds",
        "world_continue",
        "world_replay",
        "assays",
        "assay_continue",
    ]
    .contains(&mode)
    {
        return Err("journal mode".into());
    }
    valid_assays(&r["assay_requests"])?;
    if let Some(f) = r["fault"].as_object() {
        if f.len() != 3
            || f["case"].as_u64().is_none()
            || f["tick"].as_u64().is_none()
            || !["after_result", "before_result", "torn_result"]
                .contains(&f["point"].as_str().ok_or("fault point")?)
        {
            return Err("fault fields".into());
        }
    } else if !r["fault"].is_null() {
        return Err("fault schema".into());
    }
    for f in ["source_dir", "source_head_sha256"] {
        if mode.contains("continue") || mode == "world_replay" {
            if r[f].as_str().is_none() {
                return Err("source continuation identity".into());
            }
        } else if !r[f].is_null() {
            return Err("unexpected source reference".into());
        }
    }
    if mode == "world_replay" {
        if r["replay"] != json!({"case":11,"start_tick":2}) {
            return Err("declared replay boundary".into());
        }
    } else if !r["replay"].is_null() {
        return Err("unexpected replay".into());
    }
    Ok(())
}
fn fault(r: &Value, case: usize, tick: u64, point: &str) -> bool {
    r["fault"]["case"].as_u64() == Some(case as u64)
        && r["fault"]["tick"].as_u64() == Some(tick)
        && r["fault"]["point"] == point
}
fn bound(world: &World, probes: &[(i64, i64)]) -> u64 {
    let alive = world.cells.values().filter(|(c, _)| c.alive).count() as u64;
    let grants = (world.parcels as u64)
        .min((world.reservoir / 16) as u64)
        .min(alive);
    5 * grants
        + if world.tick % 3 == 0 {
            world.cells.len() as u64 + probes.len() as u64
        } else {
            probes.len() as u64
        }
        + if world.arm == Arm::External { alive } else { 0 }
}
struct Prefix {
    cases: Vec<Value>,
    assays: Vec<Value>,
    work: u64,
    pending: bool,
}
fn prefix(values: &[Value]) -> Result<Prefix, String> {
    let mut p = Prefix {
        cases: Vec::new(),
        assays: Vec::new(),
        work: 0,
        pending: false,
    };
    for v in values.iter().skip(1) {
        match v["kind"].as_str().ok_or("source record kind")? {
            "initial" => {
                let i = v["case"].as_u64().ok_or("source case")? as usize;
                if i != p.cases.len() {
                    return Err("source initial order".into());
                }
                World::restore(v["checkpoint"].clone())?;
                p.cases.push(json!({"case":i,"initial":v["checkpoint"]["state"],"rows":[],"checkpoint":v["checkpoint"],"complete":false}));
            }
            "intent" => {
                if p.pending || v["work_before"].as_u64() != Some(p.work) {
                    return Err("source work intent".into());
                }
                p.pending = true;
            }
            "result" => {
                if !p.pending {
                    return Err("source orphan result".into());
                }
                p.pending = false;
                let n = v["native_rewrites"].as_u64().ok_or("source work")?;
                p.work = p.work.checked_add(n).ok_or("source work overflow")?;
                if v["work_total"].as_u64() != Some(p.work) {
                    return Err("source cumulative work".into());
                }
                if v["unit"] == "world" {
                    let i = v["case"].as_u64().ok_or("source case")? as usize;
                    let c = p.cases.get_mut(i).ok_or("source case missing")?;
                    World::restore(v["checkpoint"].clone())?;
                    if v["row"]["before"] != c["checkpoint"]["state"]
                        || v["row"]["after"] != v["checkpoint"]["state"]
                    {
                        return Err("source checkpoint continuity".into());
                    }
                    c["rows"].as_array_mut().unwrap().push(v["row"].clone());
                    c["checkpoint"] = v["checkpoint"].clone();
                } else if v["unit"] == "assay" {
                    if v["index"].as_u64() != Some(p.assays.len() as u64) {
                        return Err("source assay order".into());
                    }
                    p.assays.push(v["native"].clone());
                } else {
                    return Err("source unit".into());
                }
            }
            "case_end" => {
                let i = v["case"].as_u64().ok_or("source case")? as usize;
                p.cases.get_mut(i).ok_or("case end missing")?["complete"] = json!(true);
            }
            "complete" | "budget_stop" => {}
            _ => return Err("unsupported source continuation record".into()),
        }
    }
    if p.pending {
        return Err("unsafe unpaired native intent".into());
    }
    Ok(p)
}
pub fn run(request_path: &Path, dir: &Path) -> Result<Value, String> {
    let raw = fs::read(request_path).map_err(error)?;
    let r: Value = serde_json::from_slice(&raw).map_err(error)?;
    validate_request(&r)?;
    let start = native_work();
    let mode = r["mode"].as_str().unwrap();
    let mut carried = 0;
    let mut loaded = None;
    if mode.contains("continue") || mode == "world_replay" {
        let values = read(
            Path::new(r["source_dir"].as_str().unwrap()),
            r["source_head_sha256"].as_str().unwrap(),
        )?;
        let original = &values[0]["request"];
        validate_request(original)?;
        if values[0]["kind"] != "header"
            || original["mode"]
                != if mode == "assay_continue" {
                    "assays"
                } else {
                    "worlds"
                }
            || [
                "source_identity",
                "world_contract",
                "assay_requests",
                "native_budget",
            ]
            .iter()
            .any(|k| original[*k] != r[*k])
        {
            return Err("continuation source identity, budget or inputs changed".into());
        }
        let p = prefix(&values)?;
        if mode == "world_replay" {
            if values.last().unwrap()["kind"] != "complete" {
                return Err("replay requires complete declared reference".into());
            }
        } else {
            carried = p.work;
        }
        loaded = Some(p);
    }
    let budget = r["native_budget"].as_u64().unwrap();
    if carried > budget {
        return Err("source consumed budget".into());
    }
    let allowance = r["execution_allowance"].as_u64().unwrap();
    let mut writer = Writer::new(dir, r["journal_byte_allowance"].as_u64().unwrap() as usize)?;
    writer.publish(
        json!({"kind":"header","request":r,"request_sha256":sha(&raw),"carried_work":carried}),
        false,
    )?;
    if let Some(p) = &loaded {
        writer.publish(json!({"kind":"source_prefix","source_head_sha256":r["source_head_sha256"],"cases":p.cases,"assays":p.assays,"source_work":p.work,"replay":mode=="world_replay"}),false)?;
    }
    if mode.starts_with("world") {
        let contract = world_contract();
        let specs = contract["cases"].as_array().unwrap();
        for (case, spec) in specs.iter().enumerate() {
            if mode == "world_replay" && case != 11 {
                continue;
            }
            let previous = loaded.as_ref().and_then(|p| p.cases.get(case));
            if mode != "world_replay" && previous.is_some_and(|c| c["complete"] == true) {
                continue;
            }
            let mut world = if mode == "world_replay" {
                let c = previous.ok_or("replay source case")?;
                let row = &c["rows"][1];
                let checkpoint = read_replay_checkpoint(
                    Path::new(r["source_dir"].as_str().unwrap()),
                    r["source_head_sha256"].as_str().unwrap(),
                    case,
                    2,
                )?;
                if checkpoint["state"] != row["after"] {
                    return Err("replay checkpoint prefix".into());
                }
                World::restore(checkpoint)?
            } else if let Some(c) = previous {
                World::restore(c["checkpoint"].clone())?
            } else {
                World::from_case(spec, contract["development_seed"].as_u64().unwrap(), 0)?
            };
            if previous.is_none() || mode == "world_replay" {
                writer.publish(
                    json!({"kind":"initial","case":case,"checkpoint":world.checkpoint()?}),
                    false,
                )?;
            }
            while world.tick < spec["ticks"].as_u64().unwrap() {
                let tick = world.tick;
                let probes: Vec<(i64, i64)> = if spec.get("probe_schedule").is_some() {
                    serde_json::from_value(spec["probe_schedule"][tick as usize].clone())
                        .map_err(error)?
                } else {
                    Vec::new()
                };
                let reservation = bound(&world, &probes);
                let done = carried + (native_work() - start) as u64;
                if done + reservation > budget
                    || (native_work() - start) as u64 + reservation > allowance
                {
                    writer.publish(json!({"kind":"budget_stop","work_total":done,"reservation":reservation,"case":case,"tick":tick}),false)?;
                    return Ok(
                        json!({"status":"budget_stop","native_rewrites":native_work()-start}),
                    );
                }
                writer.publish(json!({"kind":"intent","unit":"world","case":case,"tick":tick,"work_before":done,"work_bound":reservation}),false)?;
                let n = native_work();
                world.advance(false, &probes)?;
                let actual = (native_work() - n) as u64;
                if actual > reservation {
                    return Err("native reservation violated".into());
                }
                if fault(&r, case, tick, "before_result") {
                    return Err("declared before-result interruption".into());
                }
                writer.publish(json!({"kind":"result","unit":"world","case":case,"tick":tick,"row":world.rows.pop().unwrap(),"checkpoint":world.checkpoint()?,"native_rewrites":actual,"work_total":carried+(native_work()-start) as u64}),fault(&r,case,tick,"torn_result"))?;
                if fault(&r, case, tick, "after_result") {
                    return Err("declared after-result interruption".into());
                }
            }
            writer.publish(
                json!({"kind":"case_end","case":case,"tick":world.tick}),
                false,
            )?;
        }
    } else {
        let skip = loaded.as_ref().map_or(0, |p| p.assays.len());
        let inputs = r["assay_requests"]["assays"].as_array().unwrap();
        for (index, original) in inputs.iter().enumerate().skip(skip) {
            let done = carried + (native_work() - start) as u64;
            if done + 20 > budget || (native_work() - start) as u64 + 20 > allowance {
                writer.publish(
                    json!({"kind":"budget_stop","work_total":done,"reservation":20,"index":index}),
                    false,
                )?;
                return Ok(json!({"status":"budget_stop","native_rewrites":native_work()-start}));
            }
            writer.publish(json!({"kind":"intent","unit":"assay","index":index,"work_before":done,"work_bound":20}),false)?;
            let mut single = original.clone();
            single["id"] = json!(0);
            let n = native_work();
            let native = crate::lineage::assays(
                &json!({"profile":r["assay_requests"]["profile"],"assays":[single]}),
            )?;
            let actual = (native_work() - n) as u64;
            if actual > 20 {
                return Err("assay reservation violated".into());
            }
            if fault(&r, 0, index as u64, "before_result") {
                return Err("declared before-result interruption".into());
            }
            writer.publish(json!({"kind":"result","unit":"assay","index":index,"native":native,"native_rewrites":actual,"work_total":carried+(native_work()-start) as u64}),fault(&r,0,index as u64,"torn_result"))?;
            if fault(&r, 0, index as u64, "after_result") {
                return Err("declared after-result interruption".into());
            }
        }
    }
    writer.publish(json!({"kind":"complete","work_total":carried+(native_work()-start) as u64,"work_here":native_work()-start}),false)?;
    Ok(
        json!({"status":"complete","native_rewrites":native_work()-start,"work_total":carried+(native_work()-start) as u64}),
    )
}
fn read_replay_checkpoint(dir: &Path, head: &str, case: usize, tick: u64) -> Result<Value, String> {
    let values = read(dir, head)?;
    values
        .into_iter()
        .find(|v| {
            v["kind"] == "result"
                && v["case"].as_u64() == Some(case as u64)
                && v["checkpoint"]["state"]["tick"].as_u64() == Some(tick)
        })
        .map(|v| v["checkpoint"].clone())
        .ok_or("missing replay checkpoint".into())
}
pub fn cli() -> Result<(), String> {
    let args = std::env::args().skip(2).collect::<Vec<_>>();
    if args.len() != 2 {
        return Err("usage: cumulative-readiness journal-run REQUEST_JSON FRESH_DIRECTORY".into());
    }
    let v = run(Path::new(&args[0]), Path::new(&args[1]))?;
    println!("{v}");
    Ok(())
}
