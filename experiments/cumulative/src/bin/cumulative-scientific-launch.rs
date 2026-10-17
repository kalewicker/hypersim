//! Scientific unit executor; preview cannot execute. Controller owns aggregate funding.
#[path = "support/scientific_admission.rs"]
mod admission;
#[path = "support/scientific_binding.rs"]
mod binding;
#[path = "support/execution_journal.rs"]
mod journal;
use hg_cumulative_readiness::{lineage, native_work, population::regulation::world::World};
use serde_json::{json, Value};
use std::io::Write;
use std::{fs, io::Read, path::Path};
const CONTRACT: &str = binding::CONTRACT;
fn error(e: impl std::fmt::Display) -> String {
    e.to_string()
}
fn read(path: &Path) -> Result<Vec<u8>, String> {
    let m = fs::symlink_metadata(path).map_err(error)?;
    if !m.is_file() || m.len() > admission::MAX_BYTES as u64 {
        return Err("regular bounded input required".into());
    }
    let mut raw = Vec::new();
    fs::File::open(path)
        .map_err(error)?
        .take(admission::MAX_BYTES as u64 + 1)
        .read_to_end(&mut raw)
        .map_err(error)?;
    if raw.len() > admission::MAX_BYTES {
        return Err("input byte cap".into());
    }
    Ok(raw)
}
fn bound(state: &Value) -> u64 {
    let cells = state["cells"].as_array().unwrap();
    let alive = cells.iter().filter(|c| c["state"]["alive"] == true).count() as u64;
    let grants = state["parcels_per_tick"]
        .as_u64()
        .unwrap()
        .min(state["reservoir"].as_u64().unwrap() / 16)
        .min(alive);
    5 * grants
        + if state["tick"].as_u64().unwrap() % 3 == 0 {
            cells.len() as u64
        } else {
            0
        }
        + if state["arm"] == "external" { alive } else { 0 }
}
fn mapped(original: &Value) -> Value {
    let mut local = original.clone();
    local["id"] = json!(0);
    if ["padded_remove_later", "padded_remove_both"].contains(&local["role"].as_str().unwrap()) {
        local["role"] = json!("intact");
    }
    local
}
// The native process verifies the same durable owner and last intent used by
// the Python dispatcher. Aggregate ledger replay remains the controller's job.
fn durable_binding(e: &Value, dir: &Path) -> Result<(), String> {
    let raw_dir = dir.parent().ok_or("journal parent")?;
    let spool = raw_dir.parent().ok_or("spool parent")?;
    let root = spool
        .parent()
        .ok_or("campaign parent")?
        .canonicalize()
        .map_err(error)?;
    if raw_dir.file_name().and_then(|s| s.to_str()) != Some("raw")
        || spool.file_name().and_then(|s| s.to_str()) != Some("native_spool")
        || e["authorization"]["campaign_id"]
            != admission::sha(root.to_str().ok_or("campaign path encoding")?.as_bytes())
    {
        return Err("authorized campaign directory".into());
    }
    let owner_raw = read(&root.join("controller/contract.json"))?;
    let owner = admission::parse(&owner_raw)?;
    if e["controller"]["owner_sha256"] != admission::sha(&owner_raw)
        || owner["authorization"] != e["authorization"]
        || owner["profile"] != "cumulative-scientific-controller-v1"
        || owner["purpose"] != "launch"
    {
        return Err("durable approved owner".into());
    }
    let mut events = fs::read_dir(root.join("controller/events"))
        .map_err(error)?
        .map(|x| x.map(|p| p.path()).map_err(error))
        .collect::<Result<Vec<_>, _>>()?;
    events.sort();
    if events.is_empty() || events.len() > 2000 {
        return Err("durable intent event count".into());
    }
    for (i, path) in events.iter().enumerate() {
        if path.file_name().and_then(|s| s.to_str()) != Some(&format!("{i:08}.json")) {
            return Err("durable intent event sequence".into());
        }
    }
    let raw = read(events.last().unwrap())?;
    let entry = admission::parse(&raw)?;
    let intent = &entry["event"];
    let unit = &intent["value"]["unit"];
    let r = &e["request"];
    let logical = if r["unit"] == "world" {
        e["controller"]["request_sha256"]
            .as_str()
            .unwrap()
            .to_string()
    } else {
        let mut registry = serde_json::to_vec(&r["registry"]).map_err(error)?;
        registry.push(b'\n');
        let mut logical=serde_json::to_vec(&json!({"registry_sha256":admission::sha(&registry),"start":r["range"]["start"],"end":r["range"]["end"]})).map_err(error)?;
        logical.push(b'\n');
        admission::sha(&logical)
    };
    if admission::sha(&raw) != e["controller"]["intent_sha256"]
        || intent["kind"] != "intent"
        || unit["request_sha256"] != logical
        || unit["work_bound"] != r["execution_allowance"]
        || unit["unit"] != r["unit"]
    {
        return Err("durable exact funded intent".into());
    }
    // Atomically consume the intent before constructing any world. Even a
    // manually repeated native invocation cannot reuse it with a new journal.
    let claims = root.join("native_spool/claims");
    if !claims.exists() {
        fs::create_dir(&claims).map_err(error)?;
        fs::File::open(claims.parent().unwrap())
            .map_err(error)?
            .sync_all()
            .map_err(error)?;
    }
    let claim = claims.join(e["controller"]["intent_sha256"].as_str().unwrap());
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(claim)
        .map_err(error)?;
    writeln!(
        file,
        "{}",
        admission::sha(&serde_json::to_vec(e).map_err(error)?)
    )
    .map_err(error)?;
    file.sync_all().map_err(error)?;
    fs::File::open(&claims)
        .map_err(error)?
        .sync_all()
        .map_err(error)?;
    Ok(())
}
// Both units use this loop. A fresh journal is mandatory; no continuation path.
fn execute(envelope: &Value, route: &Value, dir: &Path) -> Result<Value, String> {
    let c: Value = admission::parse(CONTRACT.as_bytes())?;
    let mut writer =
        journal::Writer::new(dir, c["journal_byte_allowance"].as_u64().unwrap() as usize)?;
    writer.publish(json!({"kind":"header","profile":"cumulative-scientific-execution-binding-v1","binding":envelope}),false)?;
    let mut world = if route["unit"] == "world" {
        let w = World::from_case(&route["spec"], route["seed"].as_u64().unwrap(), 0)?;
        writer.publish(
            json!({"kind":"initial","checkpoint":w.checkpoint()?}),
            false,
        )?;
        Some(w)
    } else {
        None
    };
    let end = route["end"].as_u64().unwrap();
    let mut work = 0u64;
    for index in route["start"].as_u64().unwrap()..end {
        let reservation = match &world {
            Some(w) => bound(&w.state()?),
            None => 20,
        };
        if work + reservation > route["execution_allowance"].as_u64().unwrap() {
            return Err("unit reservation exhausted".into());
        }
        writer.publish(
            json!({"kind":"intent","index":index,"work_before":work,"work_bound":reservation}),
            false,
        )?;
        let before = native_work();
        let mut result = if let Some(w) = world.as_mut() {
            w.step(false)?;
            json!({"kind":"result","index":index,"row":w.rows.pop().ok_or("missing native row")?,"checkpoint":w.checkpoint()?})
        } else {
            let original = &route["items"][index as usize];
            let native =
                lineage::assays(&json!({"profile":lineage::PROFILE,"assays":[mapped(original)]}))?;
            json!({"kind":"result","index":index,"request":original,"native":native})
        };
        let actual = (native_work() - before) as u64;
        if actual > reservation {
            return Err("native reservation violation".into());
        }
        work += actual;
        result["native_rewrites"] = json!(actual);
        result["work_total"] = json!(work);
        writer.publish(result, false)?;
    }
    writer.publish(
        json!({"kind":"complete","work_total":work,"work_here":work}),
        false,
    )?;
    Ok(json!({"status":"complete","native_rewrites":work,"campaign_launch_authorized":true}))
}
fn run() -> Result<Value, String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if !((args.len() == 2 && args[0] == "--inspect") || (args.len() == 3 && args[0] == "--execute"))
    {
        return Err("usage: --inspect BINDING | --execute BINDING FRESH_JOURNAL".into());
    }
    let envelope = admission::parse(&read(Path::new(&args[1]))?)?;
    let binary = admission::sha(&fs::read(std::env::current_exe().map_err(error)?).map_err(error)?);
    let executing = args[0] == "--execute";
    let route = binding::validate(&envelope, &binary, executing)?;
    if executing {
        durable_binding(&envelope, Path::new(&args[2]))?;
        execute(&envelope, &route, Path::new(&args[2]))
    } else {
        Ok(
            json!({"binding_valid":true,"execution_performed":false,"native_rewrites":native_work(),"route":route}),
        )
    }
}
fn main() {
    match run() {
        Ok(v) => println!("{v}"),
        Err(e) => {
            eprintln!(
                "{}",
                json!({"error":e,"native_rewrites":native_work(),"campaign_launch_authorized":false})
            );
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod execution_boundary_tests {
    use super::*;
    #[test]
    fn durable_intent_is_single_use_without_native_work() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::path::PathBuf::from(format!(
            "/private/tmp/cumulative-binding-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir(&root).unwrap();
        fs::create_dir_all(root.join("controller/events")).unwrap();
        fs::create_dir_all(root.join("native_spool/raw")).unwrap();
        // Minimal ledger proof fixture is deliberately not a valid authorization
        // envelope; the separate validator would reject execution of this data.
        let a = json!({"campaign_id":admission::sha(root.to_str().unwrap().as_bytes())});
        let owner = json!({"profile":"cumulative-scientific-controller-v1","purpose":"launch","authorization":a});
        let owner_raw = serde_json::to_vec(&owner).unwrap();
        fs::write(root.join("controller/contract.json"), &owner_raw).unwrap();
        let r = json!({"unit":"world","execution_allowance":250});
        let request_hash = admission::sha(&serde_json::to_vec(&r).unwrap());
        let event = json!({"event":{"kind":"intent","value":{"unit":{"unit":"world","work_bound":250,"request_sha256":request_hash}}}});
        let raw = serde_json::to_vec(&event).unwrap();
        fs::write(root.join("controller/events/00000000.json"), &raw).unwrap();
        let mut e = json!({"authorization":a,"request":r,"controller":{"owner_sha256":admission::sha(&owner_raw),"intent_sha256":admission::sha(&raw),"request_sha256":request_hash}});
        let before = native_work();
        let dir = root.join("native_spool/raw/u000000");
        let correct = e["controller"]["intent_sha256"].clone();
        e["controller"]["intent_sha256"] = json!("0".repeat(64));
        assert!(durable_binding(&e, &dir).is_err());
        assert!(!root.join("native_spool/claims").exists());
        e["controller"]["intent_sha256"] = correct;
        assert!(durable_binding(&e, &dir).is_ok());
        assert!(durable_binding(&e, &root.join("native_spool/raw/u000001")).is_err());
        assert!(!dir.exists());
        assert_eq!(before, native_work());
        fs::remove_dir_all(root).unwrap();
    }
}
