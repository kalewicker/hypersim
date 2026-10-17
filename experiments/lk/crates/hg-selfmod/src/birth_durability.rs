//! v17: fresh native first-birth pairing with complete allocator checkpoints.
use super::*;
use financing::{allocator, restore_allocator};
pub fn prefix(seed: u64, out: &Path) -> io::Result<Value> {
    fs::create_dir(out)?;
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
            for genotype in [Arm::Faithful, Arm::Prune] {
                let mut branch = restore(checkpoint.clone())?;
                restore_allocator(&mut branch,order.clone())?;
                let (row, selected) = finish(&mut branch,pending.clone(),Some(genotype));
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
                save(&out.join(format!("{}_birth.json",genotype.name())),&json!({"row":row,"selection":selected,"initial":snapshot(&branch)}))?;
                save(&out.join(format!("{}_initial.json",genotype.name())),&snapshot(&branch))?;
                save(&out.join(format!("{}_initial_allocator.json",genotype.name())),&allocator(&branch))?;
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
