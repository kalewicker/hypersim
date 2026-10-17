//! Development adapter. Scientific seeds and campaign launches are inadmissible.
//! Old chemistry, assay law, and journal publication remain unchanged.
use super::*;
pub const PROFILE: &str = "cumulative-campaign-adapter-readiness-v1";
const SEED: u64 = 2026101001;
fn contract() -> Value {
    serde_json::from_str(include_str!(
        "../../../../../campaign_adapter_contract.json"
    ))
    .unwrap()
}
fn fields(v: &Value, names: &[&str]) -> bool {
    v.as_object()
        .is_some_and(|m| m.len() == names.len() && names.iter().all(|n| m.contains_key(*n)))
}
fn hex(v: &Value) -> bool {
    v.as_str()
        .is_some_and(|s| s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit()))
}
fn registry(r: &Value) -> Result<Vec<Value>, String> {
    if !fields(&r["registry"], &["path", "sha256"]) || !hex(&r["registry"]["sha256"]) {
        return Err("registry reference".into());
    }
    let path = Path::new(r["registry"]["path"].as_str().ok_or("registry path")?);
    if fs::metadata(path).map_err(error)?.len() > 33554432 {
        return Err("registry byte cap".into());
    }
    let raw = fs::read(path).map_err(error)?;
    if json!(sha(&raw)) != r["registry"]["sha256"] {
        return Err("registry identity".into());
    }
    let v: Value = serde_json::from_slice(&raw).map_err(error)?;
    let items = v["assays"].as_array().ok_or("registry assays")?;
    if !fields(&v, &["profile", "assays"]) || v["profile"] != PROFILE || items.len() > 78960 {
        return Err("registry profile/cap".into());
    }
    let c = contract();
    // Validate all global entries, including entries outside this batch, before work.
    for (i, a) in items.iter().enumerate() {
        let stages: Vec<i64> = serde_json::from_value(a["stages"].clone()).map_err(error)?;
        if !fields(a, &["id", "role", "stages"])
            || a["id"].as_u64() != Some(i as u64)
            || !c["roles"].as_array().unwrap().contains(&a["role"])
            || stages.is_empty()
            || stages.len() > 8
            || stages.iter().any(|s| !(-1..=4).contains(s))
        {
            return Err("complete global assay denominator".into());
        }
    }
    Ok(items.clone())
}
fn validate(r: &Value) -> Result<Vec<Value>, String> {
    if !fields(
        r,
        &[
            "profile",
            "purpose",
            "unit",
            "seed",
            "spec",
            "registry",
            "range",
            "source_identity",
            "native_budget",
            "execution_allowance",
            "journal_byte_allowance",
            "fault",
            "source",
        ],
    ) || r["profile"] != PROFILE
        || r["purpose"] != "development"
        || r["seed"].as_u64() != Some(SEED)
        || !fields(&r["source_identity"], &["binary_sha256", "source_sha256"])
        || !hex(&r["source_identity"]["binary_sha256"])
        || !hex(&r["source_identity"]["source_sha256"])
        || r["native_budget"].as_u64().is_none_or(|n| n > 6000)
        || r["execution_allowance"].as_u64().is_none_or(|n| n > 6000)
        || r["journal_byte_allowance"]
            .as_u64()
            .is_none_or(|n| n > 33554432)
    {
        return Err("development request identity/domain/cap".into());
    }
    let binary = fs::read(std::env::current_exe().map_err(error)?).map_err(error)?;
    if json!(sha(&binary)) != r["source_identity"]["binary_sha256"] {
        return Err("executing binary identity".into());
    }
    if !r["source"].is_null()
        && (!fields(&r["source"], &["directory", "head"])
            || r["source"]["directory"].as_str().is_none()
            || !hex(&r["source"]["head"]))
    {
        return Err("source reference".into());
    }
    if !r["fault"].is_null()
        && (!fields(&r["fault"], &["index", "point"])
            || r["fault"]["index"].as_u64().is_none()
            || !["after_result", "before_result", "torn_result"]
                .contains(&r["fault"]["point"].as_str().unwrap_or("")))
    {
        return Err("declared fault".into());
    }
    match r["unit"].as_str() {
        Some("world") => {
            if !fields(
                &r["spec"],
                &[
                    "name",
                    "programs",
                    "sites",
                    "parcels_per_tick",
                    "reservoir",
                    "capital",
                    "arm",
                    "ticks",
                ],
            ) || r["spec"]["ticks"]
                .as_u64()
                .is_none_or(|n| !(1..=16).contains(&n))
                || !r["registry"].is_null()
                || !r["range"].is_null()
            {
                return Err("world input fields".into());
            }
            for p in r["spec"]["programs"].as_array().ok_or("programs")? {
                if !fields(p, &["stages", "enabled", "mask"]) {
                    return Err("program fields".into());
                }
            }
            World::from_case(&r["spec"], SEED, 0)?;
            Ok(Vec::new())
        }
        Some("stress") => {
            if !r["spec"].is_null()
                || !r["registry"].is_null()
                || !r["range"].is_null()
                || !r["source"].is_null()
                || !r["fault"].is_null()
            {
                return Err("fixed synthetic stress input".into());
            }
            Ok(Vec::new())
        }
        Some("assays") => {
            if !r["spec"].is_null() || !fields(&r["range"], &["start", "end"]) {
                return Err("assay range fields".into());
            }
            let items = registry(r)?;
            let lo = r["range"]["start"].as_u64().ok_or("range start")?;
            let hi = r["range"]["end"].as_u64().ok_or("range end")?;
            if lo >= hi || hi > items.len() as u64 || hi - lo > 128 {
                return Err("batch range/cap".into());
            }
            Ok(items)
        }
        _ => Err("development unit".into()),
    }
}
// Prepared accounting-valid state, never a reachable pedigree or discovery claim.
fn stress() -> Result<World, String> {
    let spec = contract()["stress_spec"].clone();
    let mut w = World::from_case(&spec, SEED, 0)?;
    w.tick = 15;
    w.reservoir = 128;
    for (c, _) in w.cells.values_mut() {
        c.grants = 3;
        c.initial_reservoir = 48;
        c.core.initial = 112;
        c.core.dissipated = 33;
        c.core.transferred = 24;
        c.core.tick = 15;
        c.core.graph.set_step(15);
        set_capital(&mut c.core, 55);
    }
    for id in 8..32 {
        let parent = (id - 8) % 8;
        let template = &w.cells[&parent].0;
        let p = template.program()?;
        let s = json!({"stages":p.genes.iter().map(|g|g.1).collect::<Vec<_>>(),"enabled":p.enabled,"mask":p.mask,"capital":8,"reservoir":0,"arm":w.arm});
        let mut c = RegulatedCell::from_case(&s, score(SEED, 0, 3, id), 0)?;
        c.core.id = id;
        c.core.parent = Some(parent);
        c.core.generation = 1;
        c.core.tick = 15;
        c.core.graph.set_step(15);
        c.core.dissipated = 8;
        c.alive = false;
        set_capital(&mut c.core, 0);
        w.births.push(json!({"parent":parent,"child":id,"tick":1+(id-8)/8,"site":parent,"copy_cost":11,"inherited":p,"transmitted":p}));
        w.cells.insert(id, (c, None));
    }
    w.validate()?;
    Ok(w)
}
fn mapped(original: &Value) -> Value {
    let mut local = original.clone();
    local["id"] = json!(0);
    if ["padded_remove_later", "padded_remove_both"].contains(&local["role"].as_str().unwrap()) {
        local["role"] = json!("intact");
    }
    local
}
#[derive(Clone)]
struct Resume {
    next: u64,
    work: u64,
    checkpoint: Value,
}
fn prefix(r: &Value, values: &[Value], items: &[Value]) -> Result<Resume, String> {
    let original = &values[0]["request"];
    validate(original)?;
    if values[0]["kind"] != "header"
        || !original["source"].is_null()
        || values.last().unwrap()["kind"] == "complete"
    {
        return Err("one-hop incomplete source required".into());
    }
    for k in [
        "profile",
        "purpose",
        "unit",
        "seed",
        "spec",
        "registry",
        "range",
        "source_identity",
        "native_budget",
    ] {
        if r[k] != original[k] {
            return Err(format!("continuation changed {k}"));
        }
    }
    let is_world = r["unit"] == "world";
    let mut p = Resume {
        next: if is_world {
            0
        } else {
            r["range"]["start"].as_u64().unwrap()
        },
        work: 0,
        checkpoint: Value::Null,
    };
    let mut pending = None;
    let mut stopped = false;
    for v in &values[1..] {
        if stopped {
            return Err("record after source stop".into());
        }
        match v["kind"].as_str() {
            Some("initial") if is_world && p.checkpoint.is_null() && pending.is_none() => {
                let expected = World::from_case(&r["spec"], SEED, 0)?.checkpoint()?;
                if v["checkpoint"] != expected {
                    return Err("source initialization".into());
                }
                p.checkpoint = expected;
            }
            Some("intent") if pending.is_none() => {
                let end = if is_world {
                    r["spec"]["ticks"].as_u64().unwrap()
                } else {
                    r["range"]["end"].as_u64().unwrap()
                };
                if p.next >= end {
                    return Err("source horizon exceeded".into());
                }
                let reservation = if is_world {
                    bound(&World::restore(p.checkpoint.clone())?, &[])
                } else {
                    20
                };
                if v != &json!({"kind":"intent","index":p.next,"work_before":p.work,"work_bound":reservation})
                {
                    return Err("source intent continuity".into());
                }
                if p.work + reservation > original["native_budget"].as_u64().unwrap()
                    || p.work + reservation > original["execution_allowance"].as_u64().unwrap()
                {
                    return Err("source reservation exceeds allowance".into());
                }
                pending = Some(reservation);
            }
            Some("result") if pending.is_some() => {
                let n = v["native_rewrites"].as_u64().ok_or("source counter")?;
                if n > pending.unwrap()
                    || v["index"].as_u64() != Some(p.next)
                    || v["work_total"].as_u64() != Some(p.work + n)
                {
                    return Err("source result counter".into());
                }
                if is_world {
                    let restored = World::restore(v["checkpoint"].clone())?;
                    if v["row"]["before"] != p.checkpoint["state"]
                        || v["row"]["after"] != restored.state()?
                        || restored.tick != p.next + 1
                    {
                        return Err("source checkpoint continuity".into());
                    }
                    p.checkpoint = v["checkpoint"].clone();
                } else if v["request"] != items[p.next as usize]
                    || v["native"]["results"][0]["request"] != mapped(&items[p.next as usize])
                {
                    return Err("source global assay mapping".into());
                }
                p.next += 1;
                p.work += n;
                pending = None;
            }
            Some("budget_stop") if pending.is_none() => {
                stopped = true;
            }
            _ => return Err("source prefix schema/order".into()),
        }
    }
    if pending.is_some() || (is_world && p.checkpoint.is_null()) {
        return Err("unsafe unpaired intent or absent checkpoint".into());
    }
    Ok(p)
}
pub fn run(path: &Path, dir: &Path) -> Result<Value, String> {
    let raw = fs::read(path).map_err(error)?;
    let r: Value = serde_json::from_slice(&raw).map_err(error)?;
    let items = validate(&r)?;
    let is_world = r["unit"] != "assays";
    let mut p = Resume {
        next: if is_world {
            0
        } else {
            r["range"]["start"].as_u64().unwrap()
        },
        work: 0,
        checkpoint: Value::Null,
    };
    if !r["source"].is_null() {
        let values = read(
            Path::new(r["source"]["directory"].as_str().unwrap()),
            r["source"]["head"].as_str().unwrap(),
        )?;
        p = prefix(&r, &values, &items)?;
    }
    let budget = r["native_budget"].as_u64().unwrap();
    if p.work > budget {
        return Err("carried work exceeds budget".into());
    }
    let carried = p.work;
    let start = native_work();
    let mut writer = Writer::new(dir, r["journal_byte_allowance"].as_u64().unwrap() as usize)?;
    writer.publish(
        json!({"kind":"header","request":r,"request_sha256":sha(&raw),"carried_work":carried}),
        false,
    )?;
    let mut world = if is_world {
        let w = if !p.checkpoint.is_null() {
            World::restore(p.checkpoint.clone())?
        } else if r["unit"] == "stress" {
            stress()?
        } else {
            World::from_case(&r["spec"], SEED, 0)?
        };
        p.next = w.tick;
        if r["source"].is_null() {
            writer.publish(
                json!({"kind":"initial","checkpoint":w.checkpoint()?}),
                false,
            )?;
        }
        Some(w)
    } else {
        None
    };
    let end = if r["unit"] == "stress" {
        16
    } else if is_world {
        r["spec"]["ticks"].as_u64().unwrap()
    } else {
        r["range"]["end"].as_u64().unwrap()
    };
    while p.next < end {
        let reservation = world.as_ref().map_or(20, |w| bound(w, &[]));
        if p.work + reservation > budget
            || p.work - carried + reservation > r["execution_allowance"].as_u64().unwrap()
        {
            writer.publish(json!({"kind":"budget_stop","index":p.next,"work_total":p.work,"reservation":reservation}),false)?;
            return Ok(json!({"status":"budget_stop","native_rewrites":native_work()-start}));
        }
        writer.publish(
            json!({"kind":"intent","index":p.next,"work_before":p.work,"work_bound":reservation}),
            false,
        )?;
        let before = native_work();
        let mut result = if let Some(w) = world.as_mut() {
            w.step(false)?;
            json!({"kind":"result","index":p.next,"row":w.rows.pop().unwrap(),"checkpoint":w.checkpoint()?})
        } else {
            let original = &items[p.next as usize];
            let native = crate::lineage::assays(
                &json!({"profile":crate::lineage::PROFILE,"assays":[mapped(original)]}),
            )?;
            json!({"kind":"result","index":p.next,"request":original,"native":native})
        };
        let actual = (native_work() - before) as u64;
        if actual > reservation {
            return Err("native reservation violation".into());
        }
        p.work += actual;
        result["native_rewrites"] = json!(actual);
        result["work_total"] = json!(p.work);
        let point = if r["fault"]["index"].as_u64() == Some(p.next) {
            r["fault"]["point"].as_str().unwrap_or("")
        } else {
            ""
        };
        if point == "before_result" {
            return Err("declared before-result interruption".into());
        }
        writer.publish(result, point == "torn_result")?;
        if point == "after_result" {
            return Err("declared after-result interruption".into());
        }
        p.next += 1;
    }
    writer.publish(
        json!({"kind":"complete","work_total":p.work,"work_here":p.work-carried}),
        false,
    )?;
    Ok(json!({"status":"complete","native_rewrites":native_work()-start,"work_total":p.work}))
}
