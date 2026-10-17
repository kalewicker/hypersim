//! v18: fixed named R1 variants at one complete paid birth.
use super::*;
use financing::{allocator, restore_allocator};
const CONTRACT: &str = include_str!("../../../docs/selfmod_v18_variants_contract.json");
fn contract() -> Value { serde_json::from_str(CONTRACT).unwrap() }
pub fn prefix(seed: u64, out: &Path) -> io::Result<Value> {
    fs::create_dir(out)?;
    let contract = contract();
    let mut state = initialize(seed, 10000);
    save(&out.join("initial.json"), &snapshot(&state))?;
    save(&out.join("initial_allocator.json"), &allocator(&state))?;
    let mut file = BufWriter::new(OpenOptions::new().write(true).create_new(true).open(out.join("prefix.jsonl"))?);
    for step in 1..=256 {
        let pending = object(&mut state);
        let checkpoint = snapshot(&state);
        let order = allocator(&state);
        let (faithful_birth, selection) = finish(&mut state, pending.clone(), Some(Arm::Faithful));
        if let Some(faithful_selection) = selection {
            save(&out.join("fork.json"), &json!({"seed":seed,"checkpoint":checkpoint,"allocator":order,"pending":pending,"faithful_selection":faithful_selection,"faithful_birth":faithful_birth}))?;
            for variant in contract["variants"].as_array().unwrap() {
                let name = variant.as_str().unwrap();
                let genotype = if name == "faithful" { Arm::Faithful } else { Arm::Prune };
                let evidence = &contract["named_variants"][name];
                let parameters: [u64; 5] = if name == "faithful" { [0; 5] } else { serde_json::from_value(evidence["parameters"].clone()).unwrap() };
                let named = if name == "faithful" { None } else { Some((evidence["operator"].as_u64().unwrap() as usize, &parameters)) };
                let mut branch = restore(checkpoint.clone())?;
                restore_allocator(&mut branch,order.clone())?;
                let (row, selected) = finish_named(&mut branch,pending.clone(),Some(genotype),named);
                let selected = selected.unwrap();
                assert_eq!(selected.parent,faithful_selection.parent);
                assert_eq!(selected.child,faithful_selection.child);
                assert_eq!(row.rng_word_pos,faithful_birth.rng_word_pos);
                assert_eq!(row.rng_probe,faithful_birth.rng_probe);
                assert_eq!(row.world_hash,faithful_birth.world_hash);
                if genotype==Arm::Faithful {
                    assert_eq!(row,faithful_birth);
                    assert_eq!(snapshot(&branch),snapshot(&state));
                    assert_eq!(allocator(&branch),allocator(&state));
                }
                if name != "faithful" {
                    let child_gene = &branch.genes[&selected.child];
                    assert!(selected.executable && selected.changed_class && !selected.cancelled);
                    assert_eq!(child_gene.class_hash, Some(evidence["mutant_class_hash"].as_u64().unwrap()));
                    assert_eq!(serde_json::to_value(&child_gene.spec).unwrap(), evidence["decoded_spec"]);
                }
                save(&out.join(format!("{}_birth.json",name)),&json!({"row":row,"selection":selected,"initial":snapshot(&branch)}))?;
                save(&out.join(format!("{}_initial.json",name)),&snapshot(&branch))?;
                save(&out.join(format!("{}_initial_allocator.json",name)),&allocator(&branch))?;
            }
            file.flush()?;
            return Ok(json!({"seed":seed,"triggered":true,"prefix_steps":step}));
        }
        write_row(&mut file,&faithful_birth)?;
        if state.world.node_count()>state.cfg.node_explosion_cap {return Err(io::Error::other("hard prefix stop"));}
    }
    file.flush()?;
    let result=json!({"seed":seed,"triggered":false,"prefix_steps":256});
    save(&out.join("untriggered.json"),&result)?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn named_births_keep_paid_phase_siblings_rng_and_reference_material() {
        let mut native = initialize(7, 10000);
        let contract = contract();
        for _ in 0..256 {
            let pending = object(&mut native);
            let checkpoint = snapshot(&native);
            let order = allocator(&native);
            let (row, selection) = finish(&mut native, pending.clone(), Some(Arm::Faithful));
            if let Some(target) = selection {
                for name in ["prune", "matchless", "extra"] {
                    let evidence = &contract["named_variants"][name];
                    let parameters: [u64; 5] = serde_json::from_value(evidence["parameters"].clone()).unwrap();
                    let mut branch = restore(checkpoint.clone()).unwrap();
                    restore_allocator(&mut branch, order.clone()).unwrap();
                    let (actual, selected) = finish_named(&mut branch, pending.clone(), Some(Arm::Prune), Some((evidence["operator"].as_u64().unwrap() as usize, &parameters)));
                    assert_eq!(selected.unwrap().child, target.child);
                    assert_eq!(actual.roots, row.roots);
                    assert_eq!(actual.stocks, row.stocks);
                    assert_eq!(actual.ledger, row.ledger);
                    assert_eq!((actual.rng_word_pos, actual.rng_probe, actual.world_hash), (row.rng_word_pos, row.rng_probe, row.world_hash));
                    assert_eq!(actual.births.len(), row.births.len());
                    for (a, b) in actual.births.iter().zip(&row.births) {
                        if a.child != target.child { assert_eq!(a, b); }
                        else {
                            assert_eq!((a.parent, a.child, a.parent_before, a.parent_after_native, a.child_tokens, a.inherited_budget), (b.parent, b.child, b.parent_before, b.parent_after_native, b.child_tokens, b.inherited_budget));
                            assert_eq!((a.coin, a.operator, a.parameters), (b.coin, b.operator, b.parameters));
                            assert_eq!(a.gene.class_hash, Some(evidence["mutant_class_hash"].as_u64().unwrap()));
                            assert_eq!(serde_json::to_value(&a.gene.spec).unwrap(), evidence["decoded_spec"]);
                        }
                    }
                    if name == "prune" {
                        let mut old = restore(checkpoint.clone()).unwrap();
                        restore_allocator(&mut old, order.clone()).unwrap();
                        assert_eq!(actual, finish(&mut old, pending.clone(), Some(Arm::Prune)).0);
                        assert_eq!(snapshot(&branch), snapshot(&old));
                        assert_eq!(allocator(&branch), allocator(&old));
                    }
                }
                return;
            }
        }
        panic!("engineering trigger unavailable");
    }
}
