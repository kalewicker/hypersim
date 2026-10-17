//! Read-only request admission. Contains no world construction or native executor.
use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use serde_json::{json, Map, Number, Value};
use sha2::{Digest, Sha256};
use std::fmt;

pub const CONTRACT: &str = include_str!("../../../scientific_admission_contract.json");
pub const PANEL: &str = include_str!("../../../scientific_panel.json");
pub const PLANNING: &str = include_str!("../../../campaign_planning_contract.json");
pub const MAX_BYTES: usize = 33554432;

pub fn sha(raw: &[u8]) -> String {
    format!("{:x}", Sha256::digest(raw))
}

// Reject duplicate keys at every depth instead of silently interpreting the last
// occurrence. This is shared by the CLI and any future execution entry point.
struct Unique(Value);
impl<'de> Deserialize<'de> for Unique {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = Unique;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("unique-key JSON")
            }
            fn visit_bool<E: de::Error>(self, v: bool) -> Result<Unique, E> {
                Ok(Unique(json!(v)))
            }
            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Unique, E> {
                Ok(Unique(json!(v)))
            }
            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Unique, E> {
                Ok(Unique(json!(v)))
            }
            fn visit_f64<E: de::Error>(self, v: f64) -> Result<Unique, E> {
                Number::from_f64(v)
                    .map(|n| Unique(Value::Number(n)))
                    .ok_or_else(|| E::custom("nonfinite number"))
            }
            fn visit_str<E: de::Error>(self, v: &str) -> Result<Unique, E> {
                Ok(Unique(json!(v)))
            }
            fn visit_string<E: de::Error>(self, v: String) -> Result<Unique, E> {
                Ok(Unique(json!(v)))
            }
            fn visit_none<E: de::Error>(self) -> Result<Unique, E> {
                Ok(Unique(Value::Null))
            }
            fn visit_unit<E: de::Error>(self) -> Result<Unique, E> {
                Ok(Unique(Value::Null))
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Unique, A::Error> {
                let mut out = Vec::new();
                while let Some(Unique(v)) = seq.next_element()? {
                    out.push(v);
                }
                Ok(Unique(Value::Array(out)))
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Unique, A::Error> {
                let mut out = Map::new();
                while let Some(k) = map.next_key::<String>()? {
                    if out.contains_key(&k) {
                        return Err(de::Error::custom("duplicate JSON key"));
                    }
                    let Unique(v) = map.next_value()?;
                    out.insert(k, v);
                }
                Ok(Unique(Value::Object(out)))
            }
        }
        d.deserialize_any(V)
    }
}
pub fn parse(raw: &[u8]) -> Result<Value, String> {
    if raw.len() > MAX_BYTES {
        return Err("request byte cap".into());
    }
    serde_json::from_slice::<Unique>(raw)
        .map(|v| v.0)
        .map_err(|e| e.to_string())
}
fn fields(v: &Value, names: &[&str]) -> bool {
    v.as_object()
        .is_some_and(|o| o.len() == names.len() && names.iter().all(|n| o.contains_key(*n)))
}

pub fn validate(r: &Value) -> Result<Value, String> {
    let c = parse(CONTRACT.as_bytes())?;
    let panel = parse(PANEL.as_bytes())?;
    if c["panel_sha256"] != sha(PANEL.as_bytes())
        || c["planning_contract_sha256"] != sha(PLANNING.as_bytes())
    {
        return Err("compiled provenance mismatch".into());
    }
    if !fields(
        r,
        &[
            "profile",
            "mode",
            "campaign_sha256",
            "panel_sha256",
            "unit",
            "world",
            "registry",
            "registry_sha256",
            "range",
            "native_budget",
            "execution_allowance",
            "journal_byte_allowance",
            "source",
            "fault",
            "launch_authorized",
        ],
    ) || r["profile"] != c["profile"]
        || r["mode"] != "check_only"
        || r["campaign_sha256"] != sha(CONTRACT.as_bytes())
        || r["panel_sha256"] != c["panel_sha256"]
        || r["launch_authorized"] != false
        || !r["source"].is_null()
        || !r["fault"].is_null()
        || r["native_budget"].as_u64() != Some(2000000)
        || r["journal_byte_allowance"].as_u64() != Some(32505856)
    {
        return Err("closed admission profile/identity/funding".into());
    }
    let detail = match r["unit"].as_str() {
        Some("world") => {
            if !r["registry"].is_null() || !r["registry_sha256"].is_null() || !r["range"].is_null()
            {
                return Err("world-only fields".into());
            }
            let w = &r["world"];
            let index = panel["worlds"]
                .as_array()
                .ok_or("compiled panel")?
                .iter()
                .position(|expected| expected == w)
                .ok_or("world is not an exact frozen panel member")?;
            let bound = c["world_reservations"][w["arm"].as_str().ok_or("world arm")?]
                .as_u64()
                .ok_or("world reservation")?;
            if r["execution_allowance"].as_u64() != Some(bound) {
                return Err("exact world reservation".into());
            }
            json!({"unit":"world","panel_index":index,"world_id":w["world_id"],"seed":w["seed"],"work_bound":bound})
        }
        Some("assays") => {
            if !r["world"].is_null()
                || !fields(&r["range"], &["start", "end"])
                || !fields(&r["registry"], &["profile", "assays"])
            {
                return Err("assay-only fields".into());
            }
            let reg = &r["registry"];
            if reg["profile"] != c["registry_profile"]
                || r["registry_sha256"] != sha(&serde_json::to_vec(reg).map_err(|e| e.to_string())?)
            {
                return Err("complete registry identity".into());
            }
            let items = reg["assays"].as_array().ok_or("registry assays")?;
            if items.is_empty() || items.len() > 78960 {
                return Err("registry bound".into());
            }
            for (i, item) in items.iter().enumerate() {
                if !fields(item, &["id", "role", "stages"])
                    || item["id"].as_u64() != Some(i as u64)
                    || !c["roles"].as_array().unwrap().contains(&item["role"])
                {
                    return Err("complete global request identity/role".into());
                }
                let stages = item["stages"].as_array().ok_or("stage vector")?;
                if stages.is_empty()
                    || stages.len() > 8
                    || stages
                        .iter()
                        .any(|s| s.as_i64().is_none_or(|v| !(-1..=4).contains(&v)))
                {
                    return Err("complete stage domain".into());
                }
            }
            let start = r["range"]["start"].as_u64().ok_or("integer batch start")?;
            let end = r["range"]["end"].as_u64().ok_or("integer batch end")?;
            if start >= end
                || end > items.len() as u64
                || end - start > 128
                || start % 128 != 0
                || end != (start + 128).min(items.len() as u64)
            {
                return Err("fixed contiguous batch boundary".into());
            }
            let bound = 20 * (end - start);
            if r["execution_allowance"].as_u64() != Some(bound) {
                return Err("exact assay reservation".into());
            }
            json!({"unit":"assays","registry_count":items.len(),"registry_sha256":r["registry_sha256"],"range":r["range"],"work_bound":bound})
        }
        _ => return Err("scientific unit domain".into()),
    };
    Ok(
        json!({"profile":c["profile"],"request_valid":true,"execution_enabled":false,"launch_authorized":false,"native_rewrites":0,"campaign_sha256":sha(CONTRACT.as_bytes()),"panel_sha256":c["panel_sha256"],"admitted":detail}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn duplicate_keys_are_rejected() {
        assert!(parse(br#"{"x":{"a":1,"a":2}}"#).is_err());
        assert!(parse(br#"{"x":0,"x":1}"#).is_err());
    }
    #[test]
    fn trailing_data_and_nonfinite_rejected() {
        assert!(parse(b"{} {}").is_err());
        assert!(parse(b"NaN").is_err());
    }
    #[test]
    fn compiled_provenance() {
        let c = parse(CONTRACT.as_bytes()).unwrap();
        assert_eq!(c["panel_sha256"], sha(PANEL.as_bytes()));
        assert_eq!(c["planning_contract_sha256"], sha(PLANNING.as_bytes()));
        let p = parse(PANEL.as_bytes()).unwrap();
        assert_eq!(p["worlds"].as_array().unwrap().len(), 188);
    }
}
