//! v21: reversible third edit within the second founder's descendants.
use super::*;
use financing::{allocator, restore_allocator};
fn contract() -> Value { serde_json::from_str(include_str!("../../../docs/selfmod_v21_reversible_edit_contract.json")).unwrap() }
pub fn describe() -> Value {
    let c = contract();let spec: RuleSpec = serde_json::from_value(c["composite_spec"].clone()).unwrap();
    assert!(spec_to_compiled(&spec, "third-edit").is_some());
    json!({"composite_spec":spec,"composite_class_hash":spec.class_hash(),"executable":true,"object_steps":0})
}
fn target_evidence(before: &State, after: &State, child: u64, parameters: &[u64;5]) -> Value {
    use crate::schema::{HAS_PNODE, K_VAR_ID, K_LABEL_CONSTRAINT};
    use crate::store_ext::children;
    let root = rule_roots(&before.rs).into_iter().find(|&id| key_u64(id) == child).unwrap();
    let order: Vec<_> = children(&before.rs, root, HAS_PNODE).into_iter().map(|id| json!({"id":key_u64(id),"variable":get_int(&before.rs,id,K_VAR_ID).unwrap()})).collect();
    assert_eq!(order.len(),1);assert_eq!(order[0]["variable"],1);
    assert_eq!(parameters,&[0;5]);
    let root = rule_roots(&after.rs).into_iter().find(|&id| key_u64(id) == child).unwrap();
    let nodes=children(&after.rs,root,HAS_PNODE);assert_eq!(nodes.len(),2);
    let added=nodes.into_iter().find(|&id| get_int(&after.rs,id,K_VAR_ID)==Some(0)).unwrap();
    assert_eq!(get_int(&after.rs,added,K_LABEL_CONSTRAINT),Some(0));
    json!({"operator":2,"target_variable":0,"target_label":0,"child":child,"native_pnode_order":order,"added_constraint":key_u64(added),"original_parameters":parameters,"effective_parameters":parameters,"scheduler_draws":0})
}
fn descendant(s: &State, id: u64, founder: u64) -> bool {
    assert!(s.genes.contains_key(&founder));
    let mut current = Some(id);
    for _ in 0..=s.genes.len() {
        let Some(id) = current else { return false; };
        if id == founder { return true; }
        current = s.genes[&id].parent;
    }
    panic!("genealogy cycle");
}
fn first_target(s: &State, founder: u64) -> Option<u64> {
    stocks(s).into_iter().find(|x| descendant(s, x.id, founder) && x.capital >= s.cfg.repro_threshold).map(|x| x.id)
}
pub fn prefix(input: &Path, founder: u64, out: &Path) -> io::Result<Value> {
    fs::create_dir(out)?;
    let initial = read(input)?;
    let mut state = restore(initial.clone())?;
    restore_allocator(&mut state, read(&input.with_file_name(format!("{}_allocator.json", input.file_stem().unwrap().to_string_lossy())))?)?;
    assert_eq!(snapshot(&state), initial);
    assert!(stocks(&state).len() <= LIMIT);
    save(&out.join("initial.json"), &initial)?;
    save(&out.join("initial_allocator.json"), &allocator(&state))?;
    let c = contract();
    let gene = &state.genes[&founder];
    assert!(gene.lineage && gene.member == 0 && gene.executable);
    assert_eq!(serde_json::to_value(&gene.spec).unwrap(), c["second_composite_spec"]);
    assert_eq!(gene.class_hash, c["second_composite_class_hash"].as_u64());
    let parameters: [u64; 5] = serde_json::from_value(c["third_edit"]["parameters"].clone()).unwrap();
    let op = c["third_edit"]["operator"].as_u64().unwrap() as usize;
    let mut file = BufWriter::new(OpenOptions::new().write(true).create_new(true).open(out.join("rows.jsonl"))?);
    for i in 1..=256 {
        let mut pending = object(&mut state);
        pending.continuation_step = Some(20000 + i);
        if let Some(parent) = first_target(&state, founder) {
            assert_eq!(serde_json::to_value(&state.genes[&parent].spec).unwrap(), c["second_composite_spec"]);
            let checkpoint = snapshot(&state);
            let order = allocator(&state);
            let (faithful, native_selected) = finish_phase_selected(&mut state, pending.clone(), Policy::Replace8, Some(parent), None);
            let native_selected = native_selected.unwrap();
            save(&out.join("fork.json"), &json!({"founder":founder,"parent":parent,"search_steps":i,"checkpoint":checkpoint,"allocator":order,"pending":pending,"faithful_selection":native_selected,"faithful_birth":faithful}))?;
            for arm in ["faithful", "edited"] {
                let mut branch = restore(checkpoint.clone())?;
                restore_allocator(&mut branch, order.clone())?;
                let named = if arm == "edited" { Some((op, &parameters)) } else { None };
                let (receipt, selected) = finish_phase_selected(&mut branch, pending.clone(), Policy::Replace8, Some(parent), named);
                let selected = selected.unwrap();
                assert_eq!((selected.parent, selected.child), (native_selected.parent, native_selected.child));
                assert_eq!((receipt.row.world_hash, receipt.row.rng_word_pos, receipt.row.rng_probe), (faithful.row.world_hash, faithful.row.rng_word_pos, faithful.row.rng_probe));
                assert_eq!(receipt.row.stocks, faithful.row.stocks);
                assert_eq!(receipt.row.ledger, faithful.row.ledger);
                assert_eq!(receipt.bounded, faithful.bounded);
                if arm == "faithful" {
                    assert_eq!(receipt, faithful);
                    assert_eq!(snapshot(&branch), snapshot(&state));
                    assert_eq!(allocator(&branch), allocator(&state));
                } else {
                    let gene = &branch.genes[&selected.child];
                    assert!(selected.executable && selected.changed_class && !selected.cancelled);
                    assert_eq!(serde_json::to_value(&gene.spec).unwrap(), c["composite_spec"]);
                    assert_eq!(gene.class_hash, describe()["composite_class_hash"].as_u64());
                }
                save(&out.join(format!("{arm}_birth.json")), &json!({"row":receipt,"selection":selected,"initial":snapshot(&branch)}))?;
                save(&out.join(format!("{arm}_initial.json")), &snapshot(&branch))?;
                save(&out.join(format!("{arm}_initial_allocator.json")), &allocator(&branch))?;
                if arm == "edited" { save(&out.join("edited_targeting.json"), &target_evidence(&state, &branch, selected.child, &parameters))?; }
            }
            file.flush()?;
            return Ok(json!({"triggered":true,"search_steps":i}));
        }
        let receipt = finish_phase(&mut state, pending, Policy::Replace8);
        serde_json::to_writer(&mut file, &receipt)?;file.write_all(b"\n")?;
        if state.world.node_count() > state.cfg.node_explosion_cap { return Err(io::Error::other("hard third-prefix stop")); }
    }
    file.flush()?;
    let result = json!({"triggered":false,"search_steps":256});
    save(&out.join("untriggered.json"), &result)?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reversion_excludes_cousins_and_keeps_native_paid_phase() {
        let c = contract();let parameters: [u64;5] = serde_json::from_value(c["third_edit"]["parameters"].clone()).unwrap();
        let mut state = initialize(7,10000);let ancestor = rule_roots(&state.rs)[0];
        apply_recorded_operator(&mut state.rs,ancestor,11,&PRUNE_PARAMETERS,&state.cfg.caps()).unwrap();
        register(&mut state,ancestor,None,true);
        let founder = crate::store_ext::duplicate_rule(&mut state.rs,ancestor);
        let second: [u64;5] = serde_json::from_value(c["second_edit"]["parameters"].clone()).unwrap();
        let (resolved,_) = second_edit::resolve_parameters(&state.rs,founder,3,&second,0);
        apply_recorded_operator(&mut state.rs,founder,3,&resolved,&state.cfg.caps()).unwrap();
        register(&mut state,founder,Some(key_u64(ancestor)),true);
        while stocks(&state).len()<8 {
            let cousin=crate::store_ext::duplicate_rule(&mut state.rs,ancestor);
            register(&mut state,cousin,Some(key_u64(ancestor)),true);
        }
        for root in rule_roots(&state.rs) { set_tokens(&mut state.rs,root,120); }
        state.led.cumulative_influx=state.led.free+total_bound(&state.rs);
        let pending=object(&mut state);let id=key_u64(founder);
        assert!(!descendant(&state,key_u64(ancestor),id));assert_eq!(first_target(&state,id),Some(id));
        let mut native=state.clone();let (faithful,selected)=finish_phase_selected(&mut native,pending.clone(),Policy::Replace8,Some(id),None);
        let (edited,child)=finish_phase_selected(&mut state,pending,Policy::Replace8,Some(id),Some((2,&parameters)));let child=child.unwrap();
        assert_eq!(child.child,selected.unwrap().child);assert!(child.executable&&child.changed_class);
        assert_eq!(edited.row.roots,faithful.row.roots);assert_eq!(edited.row.stocks,faithful.row.stocks);assert_eq!(edited.row.ledger,faithful.row.ledger);assert_eq!(edited.bounded,faithful.bounded);
        assert_eq!((edited.row.world_hash,edited.row.rng_word_pos,edited.row.rng_probe),(faithful.row.world_hash,faithful.row.rng_word_pos,faithful.row.rng_probe));
        for (a,b) in edited.row.births.iter().zip(&faithful.row.births) {
            let mut normalized=a.clone();if a.child==child.child { normalized.gene=b.gene.clone(); }
            assert_eq!(&normalized,b);
        }
        assert_eq!(serde_json::to_value(&state.genes[&child.child].spec).unwrap(),c["composite_spec"]);
        assert_eq!(state.genes[&child.child].class_hash,describe()["composite_class_hash"].as_u64());
        assert_eq!(target_evidence(&native,&state,child.child,&parameters)["target_variable"],0);
        assert!(descendant(&state,child.child,id));assert!(!stocks(&state).iter().any(|x|x.id==id));
    }
}
