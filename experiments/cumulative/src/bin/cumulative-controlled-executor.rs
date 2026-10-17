//! Versioned fixed development routes for end-to-end controller integration. Scientific dispatch remains locked.
#[path = "support/scientific_admission.rs"]
mod admission;
#[path = "support/execution_journal.rs"]
mod journal;
use hg_cumulative_readiness::{lineage, native_work, population::regulation::world::World};
use serde_json::{json, Value};
use std::{fs, io::Read, path::Path};
const CONTRACT: &str = include_str!("../../executor_controller_contract.json");
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
// Both units use this loop. A fresh journal is mandatory; no continuation path.
fn execute(route: &Value, dir: &Path) -> Result<Value, String> {
    let c: Value = admission::parse(CONTRACT.as_bytes())?;
    let mut writer =
        journal::Writer::new(dir, c["journal_byte_allowance"].as_u64().unwrap() as usize)?;
    writer.publish(json!({"kind":"header","profile":c["profile"],"purpose":"development","contract_sha256":admission::sha(CONTRACT.as_bytes()),"route":route,"binary_sha256":admission::sha(&fs::read(std::env::current_exe().map_err(error)?).map_err(error)?)}),false)?;
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
    let end = if world.is_some() {
        route["spec"]["ticks"].as_u64().unwrap()
    } else {
        route["items"].as_array().unwrap().len() as u64
    };
    let mut work = 0u64;
    for index in 0..end {
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
        if route["fault"] == "after_intent" {
            return Err("declared interruption after durable intent".into());
        }
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
    Ok(json!({"status":"complete","native_rewrites":work,"campaign_launch_authorized":false}))
}
fn run() -> Result<Value, String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("--check") if args.len()==2 => {
            let raw=read(Path::new(&args[1]))?;let r=admission::parse(&raw)?;
            let mut report=admission::validate(&r)?;
            report["request_sha256"]=json!(admission::sha(&raw));Ok(report)
        },
        Some("--execute") if args.len()==4 => {
            // Even a valid scientific request is not launch authorization. Keep
            // this unconditional lock until a reviewed campaign owner supplies
            // durable reservations and enforces all aggregate resource ceilings.
            let raw=read(Path::new(&args[1]))?;admission::validate(&admission::parse(&raw)?)?;
            let _authorization=admission::parse(&read(Path::new(&args[2]))?)?;
            Err("scientific launch disabled: campaign controller integration and explicit budget authorization required".into())
        },
        Some("--development") if args.len()==3 => {
            let c=admission::parse(CONTRACT.as_bytes())?;
            let route=c["routes"].as_array().unwrap().iter().find(|r|r["name"]==args[1]).ok_or("undeclared development route")?;
            execute(route,Path::new(&args[2]))
        },
        _ => Err("usage: --check REQUEST | --execute REQUEST AUTHORIZATION JOURNAL | --development FIXED_ROUTE JOURNAL".into())
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
