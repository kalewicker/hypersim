//! v16: restore a completed trajectory and advance its unchanged native policy.
use super::*;
pub(super) fn allocator(s: &State) -> Value {
    json!({"format":1,"world":s.world.allocator_order(),"rules":s.rs.allocator_order()})
}
pub(super) fn restore_allocator(s: &mut State, value: Value) -> io::Result<()> {
    assert_eq!(value["format"], 1);
    s.world.restore_allocator_order(&serde_json::from_value(value["world"].clone())?).map_err(io::Error::other)?;
    s.rs.restore_allocator_order(&serde_json::from_value(value["rules"].clone())?).map_err(io::Error::other)?;
    Ok(())
}
fn allocator_path(path: &Path) -> std::path::PathBuf {
    path.with_file_name(format!("{}_allocator.json",path.file_stem().unwrap().to_string_lossy()))
}
pub fn extend(input: &Path, policy: Policy, offset: usize, steps: usize, out: &Path) -> io::Result<Value> {
    fs::create_dir(out)?;
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let initial = read(input)?;
        let mut state = restore(initial.clone())?;
        if allocator_path(input).exists() { restore_allocator(&mut state,read(&allocator_path(input))?)?; }
        assert!(!policy.bounded() || stocks(&state).len() <= LIMIT);
        assert_eq!(snapshot(&state), initial);
        save(&out.join("initial.json"), &initial)?;
        save(&out.join("initial_allocator.json"), &allocator(&state))?;
        status(out, json!({"state":"running","offset":offset,"steps_completed":0}))?;
        let mut file = BufWriter::new(OpenOptions::new().write(true).create_new(true).open(out.join("rows.jsonl"))?);
        for i in 1..=steps {
            let mut row = object(&mut state);
            row.continuation_step = Some(offset + i);
            let receipt = finish_phase(&mut state, row, policy);
            serde_json::to_writer(&mut file, &receipt)?;
            file.write_all(b"\n")?;
            if state.world.node_count() > state.cfg.node_explosion_cap {
                file.flush()?;
                return Err(io::Error::other("hard node explosion stop; raw retained"));
            }
            if i % 1000 == 0 {
                file.flush()?;
                save(&out.join(format!("checkpoint_{}.json", offset + i)), &snapshot(&state))?;
                save(&out.join(format!("checkpoint_{}_allocator.json", offset + i)), &allocator(&state))?;
                status(out, json!({"state":"running","offset":offset,"steps_completed":i}))?;
            }
        }
        file.flush()?;
        assert_eq!(snapshot(&state)["cfg"], initial["cfg"]);
        save(&out.join("final.json"), &snapshot(&state))?;
        save(&out.join("final_allocator.json"), &allocator(&state))?;
        let summary = json!({"state":"complete","offset":offset,"steps_completed":steps});
        status(out, summary.clone())?;
        Ok(summary)
    })).unwrap_or_else(|_| Err(io::Error::other("native/harness panic; raw retained")));
    if let Err(e) = &result {
        let prior = read(&out.join("status.json")).ok();
        status(out, json!({"state":"failed","error":e.to_string(),"last_progress":prior}))?;
    }
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn allocator_sidecar_restores_exact_reuse_and_rejects_malformed_orders_atomically() {
        let mut original = slotmap::SlotMap::new();
        let a = original.insert(1);
        let b = original.insert(2);
        let c = original.insert(3);
        original.remove(c);
        original.remove(a);
        let order = original.vacant_slot_order();
        assert_eq!(order, vec![1,3]);
        let mut restored: slotmap::SlotMap<slotmap::DefaultKey,i32> = serde_json::from_value(serde_json::to_value(&original).unwrap()).unwrap();
        assert_eq!(restored.vacant_slot_order(), vec![3,1]);
        let before = restored.vacant_slot_order();
        for damaged in [vec![1],vec![1,1],vec![0,3],vec![2,3],vec![1,99]] {
            assert!(restored.restore_vacant_slot_order(&damaged).is_err());
            assert_eq!(restored.vacant_slot_order(),before);
            assert_eq!(restored[b],2);
        }
        restored.restore_vacant_slot_order(&order).unwrap();
        for value in [4,5,6] { assert_eq!(original.insert(value),restored.insert(value)); }
        assert_eq!(serde_json::to_value(&original).unwrap(),serde_json::to_value(&restored).unwrap());
    }
    #[test]
    fn restored_extension_preserves_clock_config_and_native_phase_after_old_horizon() {
        for policy in POLICIES {
            let mut continuous = initialize(7, 8);
            for i in 1..=17 {
                let mut row = object(&mut continuous);
                row.continuation_step = Some(i);
                finish_phase(&mut continuous, row, policy);
            }
            let checkpoint = snapshot(&continuous);
            let order = allocator(&continuous);
            let bytes = serde_json::to_vec(&checkpoint).unwrap();
            let mut split = restore(serde_json::from_slice(&bytes).unwrap()).unwrap();
            restore_allocator(&mut split,order).unwrap();
            for i in 18..=96 {
                let mut a = object(&mut continuous);
                let mut b = object(&mut split);
                a.continuation_step = Some(i);
                b.continuation_step = Some(i);
                assert_eq!(finish_phase(&mut continuous, a, policy), finish_phase(&mut split, b, policy));
            }
            fn mismatch(a: &Value, b: &Value, path: String) -> Option<String> {
                if a == b { return None; }
                match (a,b) {
                    (Value::Object(a),Value::Object(b)) => a.iter().find_map(|(k,v)| mismatch(v,b.get(k).unwrap_or(&Value::Null),format!("{path}/{k}"))),
                    (Value::Array(a),Value::Array(b)) if a.len()==b.len() => a.iter().zip(b).enumerate().find_map(|(i,(a,b))|mismatch(a,b,format!("{path}/{i}"))),
                    _ => Some(format!("{path}: {a} != {b}")),
                }
            }
            assert!(snapshot(&continuous)==snapshot(&split), "{}", mismatch(&snapshot(&continuous),&snapshot(&split),String::new()).unwrap());
            assert_eq!(snapshot(&split)["cfg"], checkpoint["cfg"]);
            assert_eq!(allocator(&continuous),allocator(&split));
            assert_eq!(split.world.step(), 96);
        }
    }
}
