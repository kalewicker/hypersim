//! v19: a single further edit at a native funded birth in an established family.
use super::*;
use financing::{allocator, restore_allocator};
const CONTRACT: &str = include_str!("../../../docs/selfmod_v19_second_edit_contract.json");
fn contract() -> Value { serde_json::from_str(CONTRACT).unwrap() }
pub(super) fn resolve_parameters(rs: &HypergraphStore, child: NodeId, op: usize, original: &[u64; 5], var: i64) -> ([u64; 5], Value) {
    use crate::schema::{HAS_PNODE, K_VAR_ID};
    use crate::store_ext::children;
    assert_eq!(op, 3, "semantic adapter only supports drop_pnode_constraint");
    let nodes = children(rs, child, HAS_PNODE);
    let order: Vec<_> = nodes.iter().map(|&id| json!({"id":key_u64(id),"variable":get_int(rs,id,K_VAR_ID).expect("constraint variable")})).collect();
    let indices: Vec<_> = nodes.iter().enumerate().filter(|(_, id)| get_int(rs, **id, K_VAR_ID) == Some(var)).map(|(index, _)| index).collect();
    assert_eq!(indices.len(), 1, "exactly one semantic target required");
    let index = indices[0];
    let mut effective = *original;
    effective[0] = index as u64;
    (effective, json!({"operator":op,"target_variable":var,"child":key_u64(child),"native_pnode_order":order,"target_index":index,"original_parameters":original,"effective_parameters":effective,"scheduler_draws":0}))
}

/// Engineering-only repair verification of an already saved post-object fork.
/// Does not perform object steps or modify its input archive.
pub fn semantic_regression(input: &Path, out: &Path) -> io::Result<Value> {
    fs::create_dir(out)?;
    let fork = read(input)?;
    let checkpoint = fork["checkpoint"].clone();
    let order = fork["allocator"].clone();
    let pending: Row = serde_json::from_value(fork["pending"].clone()).unwrap();
    let parent = fork["parent"].as_u64().unwrap();
    let faithful: Receipt = serde_json::from_value(fork["faithful_birth"].clone()).unwrap();
    let native_selected: Selection = serde_json::from_value(fork["faithful_selection"].clone()).unwrap();
    let c = contract();
    let parameters: [u64; 5] = serde_json::from_value(c["second_edit"]["parameters"].clone()).unwrap();
    let mut targeting = None;
    for arm in ["faithful", "edited"] {
        let mut branch = restore(checkpoint.clone())?;
        restore_allocator(&mut branch, order.clone())?;
        assert_eq!(snapshot(&branch), checkpoint);
        let named = if arm == "edited" { Some((3, &parameters)) } else { None };
        let (receipt, selected, resolved) = finish_phase_targeted(&mut branch, pending.clone(), Policy::Replace8, Some(parent), named, if arm == "edited" { Some(0) } else { None });
        let selected = selected.unwrap();
        assert_eq!((selected.parent, selected.child), (native_selected.parent, native_selected.child));
        assert_eq!(receipt.row.roots, faithful.row.roots);
        assert_eq!(receipt.row.stocks, faithful.row.stocks);
        assert_eq!(receipt.row.ledger, faithful.row.ledger);
        assert_eq!(receipt.bounded, faithful.bounded);
        assert_eq!((receipt.row.world_hash, receipt.row.rng_word_pos, receipt.row.rng_probe), (faithful.row.world_hash, faithful.row.rng_word_pos, faithful.row.rng_probe));
        for (actual, original) in receipt.row.births.iter().zip(&faithful.row.births) {
            if actual.child != selected.child { assert_eq!(actual, original); }
            else {
                let mut actual = actual.clone();
                actual.gene = original.gene.clone();
                assert_eq!(actual, *original);
            }
        }
        if arm == "faithful" { assert_eq!(receipt, faithful); assert!(resolved.is_none()); }
        else {
            let gene = &branch.genes[&selected.child];
            assert!(selected.executable && selected.changed_class && !selected.cancelled);
            assert_eq!(serde_json::to_value(&gene.spec).unwrap(), c["composite_spec"]);
            assert_eq!(gene.class_hash, describe()["composite_class_hash"].as_u64());
            targeting = resolved;
        }
        save(&out.join(format!("{arm}_birth.json")), &json!({"row":receipt,"selection":selected,"initial":snapshot(&branch)}))?;
        save(&out.join(format!("{arm}_initial.json")), &snapshot(&branch))?;
        save(&out.join(format!("{arm}_initial_allocator.json")), &allocator(&branch))?;
    }
    let report = json!({"passed":true,"engineering_only":true,"excluded_from_science":true,"object_steps":0,"targeting":targeting,"native_paid_fields_and_siblings_equal":true});
    save(&out.join("verification.json"), &report)?;
    Ok(report)
}
pub fn describe() -> Value {
    let c = contract();
    let spec: RuleSpec = serde_json::from_value(c["composite_spec"].clone()).unwrap();
    assert!(spec_to_compiled(&spec, "second-edit").is_some());
    json!({"composite_spec":spec,"composite_class_hash":spec.class_hash(),"executable":true,"object_steps":0})
}
fn first_target(s: &State) -> Option<u64> {
    stocks(s).into_iter().find(|x| s.genes[&x.id].lineage && x.capital >= s.cfg.repro_threshold).map(|x| x.id)
}
pub fn second_prefix(input: &Path, out: &Path) -> io::Result<Value> {
    second_prefix_mode(input, out, false)
}
pub fn second_prefix_semantic(input: &Path, out: &Path) -> io::Result<Value> {
    second_prefix_mode(input, out, true)
}
pub fn describe_semantic() -> Value {
    let c: Value = serde_json::from_str(include_str!("../../../docs/selfmod_v19b_second_edit_contract.json")).unwrap();
    let description = describe();
    assert_eq!(c["composite_spec"], description["composite_spec"]);
    assert_eq!(c["composite_class_hash"], description["composite_class_hash"]);
    assert_eq!(c["second_edit"]["operator"], 3);
    assert_eq!(c["target_adapter"]["target_variable"], 0);
    assert_eq!(c["second_edit"]["parameters"], contract()["second_edit"]["parameters"]);
    description
}
fn second_prefix_mode(input: &Path, out: &Path, semantic: bool) -> io::Result<Value> {
    if semantic { describe_semantic(); }
    fs::create_dir(out)?;
    let initial = read(input)?;
    let mut state = restore(initial.clone())?;
    restore_allocator(&mut state, read(&input.with_file_name(format!("{}_allocator.json", input.file_stem().unwrap().to_string_lossy())))?)?;
    assert_eq!(snapshot(&state), initial);
    assert!(stocks(&state).len() <= LIMIT);
    save(&out.join("initial.json"), &initial)?;
    save(&out.join("initial_allocator.json"), &allocator(&state))?;
    let c = contract();
    let parameters: [u64; 5] = serde_json::from_value(c["second_edit"]["parameters"].clone()).unwrap();
    let op = c["second_edit"]["operator"].as_u64().unwrap() as usize;
    let mut file = BufWriter::new(OpenOptions::new().write(true).create_new(true).open(out.join("rows.jsonl"))?);
    for i in 1..=256 {
        let mut pending = object(&mut state);
        pending.continuation_step = Some(10000 + i);
        if let Some(parent) = first_target(&state) {
            let checkpoint = snapshot(&state);
            let order = allocator(&state);
            let (faithful, native_selected) = finish_phase_selected(&mut state, pending.clone(), Policy::Replace8, Some(parent), None);
            let native_selected = native_selected.unwrap();
            save(&out.join("fork.json"), &json!({"parent":parent,"search_steps":i,"checkpoint":checkpoint,"allocator":order,"pending":pending,"faithful_selection":native_selected,"faithful_birth":faithful}))?;
            for arm in ["faithful", "edited"] {
                let mut branch = restore(checkpoint.clone())?;
                restore_allocator(&mut branch, order.clone())?;
                let named = if arm == "edited" { Some((op, &parameters)) } else { None };
                let (receipt, selected, targeting) = finish_phase_targeted(&mut branch, pending.clone(), Policy::Replace8, Some(parent), named, if semantic && arm == "edited" { Some(0) } else { None });
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
                if let Some(evidence) = targeting { save(&out.join("edited_targeting.json"), &evidence)?; }
            }
            file.flush()?;
            return Ok(json!({"triggered":true,"search_steps":i}));
        }
        let receipt = finish_phase(&mut state, pending, Policy::Replace8);
        serde_json::to_writer(&mut file, &receipt)?;file.write_all(b"\n")?;
        if state.world.node_count() > state.cfg.node_explosion_cap { return Err(io::Error::other("hard second-prefix stop")); }
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
    fn semantic_constraint_target_handles_both_native_node_orders() {
        use crate::schema::{HAS_PNODE, K_VAR_ID, K_LABEL_CONSTRAINT};
        use crate::store_ext::children;
        for order in [[0, 1], [1, 0]] {
            let mut state = initialize(7, 10000);
            let root = rule_roots(&state.rs)[0];
            apply_recorded_operator(&mut state.rs, root, 11, &PRUNE_PARAMETERS, &state.cfg.caps()).unwrap();
            let nodes = children(&state.rs, root, HAS_PNODE);
            assert_eq!(nodes.len(), 2);
            for (&node, var) in nodes.iter().zip(order) {
                set_int(&mut state.rs, node, K_VAR_ID, var);
                set_int(&mut state.rs, node, K_LABEL_CONSTRAINT, var);
            }
            let c = contract();
            assert_eq!(serde_json::to_value(decode_spec(&state.rs, root, &state.cfg.caps()).unwrap()).unwrap(), c["initial_prune"]["decoded_spec"]);
            let original: [u64;5] = serde_json::from_value(c["second_edit"]["parameters"].clone()).unwrap();
            let (effective, evidence) = resolve_parameters(&state.rs, root, 3, &original, 0);
            assert_eq!(effective[0], if order[0] == 0 { 0 } else { 1 });
            assert_eq!(effective[1..], original[1..]);
            assert_eq!(evidence["scheduler_draws"], 0);
            apply_recorded_operator(&mut state.rs, root, 3, &effective, &state.cfg.caps()).unwrap();
            assert_eq!(serde_json::to_value(decode_spec(&state.rs, root, &state.cfg.caps()).unwrap()).unwrap(), c["composite_spec"]);
        }
    }
    #[test]
    #[should_panic(expected = "exactly one semantic target required")]
    fn semantic_constraint_target_rejects_missing_variable() {
        let state = initialize(7, 10000);
        resolve_parameters(&state.rs, rule_roots(&state.rs)[0], 3, &[0;5], 99);
    }
    #[test]
    #[should_panic(expected = "exactly one semantic target required")]
    fn semantic_constraint_target_rejects_duplicate_variable() {
        use crate::schema::{HAS_PNODE, K_VAR_ID};
        let mut state = initialize(7, 10000);
        let root = rule_roots(&state.rs)[0];
        for node in crate::store_ext::children(&state.rs, root, HAS_PNODE) {
            set_int(&mut state.rs, node, K_VAR_ID, 0);
        }
        resolve_parameters(&state.rs, root, 3, &[0;5], 0);
    }
    #[test]
    fn second_edit_keeps_native_paid_donor_retirement_siblings_and_rng() {
        let mut initial = initialize(7, 10000);
        let root = rule_roots(&initial.rs)[0];
        apply_recorded_operator(&mut initial.rs, root, 11, &PRUNE_PARAMETERS, &initial.cfg.caps()).unwrap();
        register(&mut initial, root, None, true);
        while stocks(&initial).len() < 8 {
            let child = crate::store_ext::duplicate_rule(&mut initial.rs, root);
            register(&mut initial, child, Some(key_u64(root)), true);
        }
        for root in rule_roots(&initial.rs) { set_tokens(&mut initial.rs, root, 120); }
        initial.led.cumulative_influx = initial.led.free + total_bound(&initial.rs);
        let pending = object(&mut initial);
        let parent = first_target(&initial).unwrap();
        let mut ordinary = initial.clone();
        let native = finish_phase(&mut ordinary, pending.clone(), Policy::Replace8);
        let mut faithful = initial.clone();
        let (same, selection) = finish_phase_selected(&mut faithful, pending.clone(), Policy::Replace8, Some(parent), None);
        assert_eq!(same, native);assert_eq!(snapshot(&faithful), snapshot(&ordinary));assert_eq!(allocator(&faithful), allocator(&ordinary));
        let c = contract();let parameters: [u64; 5] = serde_json::from_value(c["second_edit"]["parameters"].clone()).unwrap();
        let (edit, second) = finish_phase_selected(&mut initial, pending, Policy::Replace8, Some(parent), Some((3, &parameters)));
        let selected = second.unwrap();assert_eq!(selected.child, selection.unwrap().child);
        assert!(selected.changed_class && selected.executable);
        assert_eq!(edit.bounded, native.bounded);assert_eq!(edit.row.roots, native.row.roots);assert_eq!(edit.row.stocks, native.row.stocks);assert_eq!(edit.row.ledger, native.row.ledger);
        assert_eq!((edit.row.rng_word_pos, edit.row.rng_probe), (native.row.rng_word_pos, native.row.rng_probe));
        for (actual, original) in edit.row.births.iter().zip(&native.row.births) {
            if actual.child != selected.child { assert_eq!(actual, original); }
            else {
                assert_eq!(serde_json::to_value(&actual.gene.spec).unwrap(), c["composite_spec"]);
                assert_eq!((actual.parent_before, actual.parent_after_native, actual.child_tokens, actual.inherited_budget, actual.operator, actual.parameters), (original.parent_before, original.parent_after_native, original.child_tokens, original.inherited_budget, original.operator, original.parameters));
            }
        }
        assert!(!stocks(&initial).iter().any(|x| x.id == parent));
        assert_eq!(stocks(&initial).len(), 8);
        assert_eq!(initial.genes[&selected.child].class_hash, describe()["composite_class_hash"].as_u64());
    }
}
