//! v15: fresh initialization and first native R1 birth, unchanged v14 policies.
use super::*;
pub const PROTOCOL: &str = "selfmod-v15-replication-v1";
pub const CONTRACT: &str = include_str!("../../../docs/selfmod_v15_replication_preregistration.md");
pub const FRESH_SEEDS: [u64; 8] = [131, 137, 149, 157, 163, 173, 181, 191];
fn arms() -> Vec<String> {
    [Arm::Faithful, Arm::Prune]
        .iter()
        .flat_map(|a| {
            POLICIES
                .iter()
                .map(move |p| format!("{}_{}", a.name(), p.name()))
        })
        .collect()
}
fn prefix(seed: u64, horizon: usize, dir: &Path) -> io::Result<usize> {
    fs::create_dir(dir)?;
    let mut s = initialize(seed, horizon);
    save(&dir.join("initial.json"), &snapshot(&s))?;
    let mut file = BufWriter::new(
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(dir.join("prefix.jsonl"))?,
    );
    for step in 1..=256 {
        let pending = object(&mut s);
        let checkpoint = snapshot(&s);
        let (faithful_birth, selection) = finish(&mut s, pending.clone(), Some(Arm::Faithful));
        if let Some(faithful_selection) = selection {
            save(
                &dir.join("fork.json"),
                &json!({"seed":seed,"checkpoint":checkpoint,"pending":pending,
                "faithful_selection":faithful_selection,"faithful_birth":faithful_birth}),
            )?;
            for arm in [Arm::Faithful, Arm::Prune] {
                let mut state = restore(checkpoint.clone())?;
                let (row, selected) = finish(&mut state, pending.clone(), Some(arm));
                let selected = selected.unwrap();
                assert_eq!(selected.parent, faithful_selection.parent);
                assert_eq!(selected.child, faithful_selection.child);
                assert_eq!(row.rng_word_pos, faithful_birth.rng_word_pos);
                assert_eq!(row.rng_probe, faithful_birth.rng_probe);
                assert_eq!(row.world_hash, faithful_birth.world_hash);
                if arm == Arm::Faithful {
                    assert_eq!(row, faithful_birth);
                    assert_eq!(snapshot(&state), snapshot(&s));
                }
                save(
                    &dir.join(format!("{}_birth.json", arm.name())),
                    &json!({"selection":selected,"row":row,"initial":snapshot(&state)}),
                )?;
            }
            return Ok(step);
        }
        write_row(&mut file, &faithful_birth)?;
        if s.world.node_count() > s.cfg.node_explosion_cap {
            return Err(io::Error::other("hard prefix explosion stop"));
        }
    }
    save(
        &dir.join("untriggered.json"),
        &json!({"seed":seed,"prefix_steps":256,"reason":"no natural R1 birth"}),
    )?;
    Ok(256)
}
pub fn verify_prefix(seed: u64, out: &Path) -> io::Result<Value> {
    let steps = prefix(seed, 2000, out)?;
    Ok(json!({"prefix_steps":steps}))
}
pub fn run(out: &Path, preflight: bool) -> io::Result<Value> {
    let _lock = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(out.join("assay.lock"))?;
    let result =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run_locked(out, preflight)))
            .unwrap_or_else(|_| Err(io::Error::other("native/harness panic; raw retained")));
    if let Err(e) = &result {
        let prior = read(&out.join("status.json")).ok();
        status(
            out,
            json!({"state":"failed","error":e.to_string(),"last_progress":prior}),
        )?;
    }
    result
}
fn run_locked(out: &Path, preflight: bool) -> io::Result<Value> {
    let seeds: &[u64] = if preflight { &[7, 42] } else { &FRESH_SEEDS };
    let horizon = if preflight { 64 } else { 2000 };
    save(
        &out.join("manifest.json"),
        &json!({"protocol":PROTOCOL,"engineering_preflight":preflight,"seeds":seeds,"horizon":horizon,"arms":arms(),"limit":LIMIT,
        "prefix_limit":256,"continuations_requested":seeds.len()*8,"replication_threshold":6,"future_mutation":false,"prune_parameters":PRUNE_PARAMETERS,
        "configs":seeds.iter().map(|s|crate::spectrum::config(*s,horizon)).collect::<Vec<_>>()}),
    )?;
    fs::write(out.join("preregistration.md"), CONTRACT)?;
    status(out, json!({"state":"running","continuations_completed":0}))?;
    let mut results = Vec::new();
    let mut prefixes = 0;
    let mut triggered = 0;
    let mut missing = Vec::new();
    for &seed in seeds {
        let dir = out.join(format!("seed_{seed}"));
        prefixes += prefix(seed, horizon, &dir)?;
        if dir.join("untriggered.json").exists() {
            for arm in arms() {
                missing.push(json!({"seed":seed,"arm":arm,"reason":"untriggered"}));
            }
            continue;
        }
        triggered += 1;
        for genotype in [Arm::Faithful, Arm::Prune] {
            let birth = read(&dir.join(format!("{}_birth.json", genotype.name())))?;
            let state = restore(birth["initial"].clone())?;
            let population = stocks(&state).len();
            for policy in POLICIES {
                let arm = format!("{}_{}", genotype.name(), policy.name());
                let arm_dir = dir.join(&arm);
                if policy.bounded() && population > LIMIT {
                    fs::create_dir(&arm_dir)?;
                    save(
                        &arm_dir.join("inadmissible.json"),
                        &json!({"seed":seed,"arm":arm,"population":population,"limit":LIMIT,"reason":"initial population exceeds bound"}),
                    )?;
                    missing.push(
                        json!({"seed":seed,"arm":arm,"reason":"initial population exceeds bound"}),
                    );
                    continue;
                }
                let score = branch(&birth, &arm_dir, policy)?;
                assert_eq!(
                    fs::read(arm_dir.join("birth.json"))?,
                    fs::read(dir.join(format!("{}_birth.json", genotype.name())))?
                );
                if preflight {
                    let reference = out.join(format!("reference/seed_{seed}/{arm}"));
                    for file in [
                        "birth.json",
                        "rows.jsonl",
                        "score.json",
                        "reproduction.json",
                        "final.json",
                    ] {
                        assert_eq!(
                            fs::read(arm_dir.join(file))?,
                            fs::read(reference.join(file))?,
                            "historical v14 preflight parity: {seed}/{arm}/{file}"
                        );
                    }
                }
                results.push(json!({"seed":seed,"arm":arm,"score":score,"reproduction":read(&arm_dir.join("reproduction.json"))?}));
                status(
                    out,
                    json!({"state":"running","continuations_completed":results.len(),"shared_prefix_steps":prefixes,"scientific_advancing_steps":prefixes+results.len()*horizon,"active_seed":seed}),
                )?;
                println!(
                    "seed {seed} {arm}: {}/{} recorded",
                    results.len(),
                    seeds.len() * 8
                );
                io::stdout().flush()?;
            }
        }
    }
    let summary = json!({"protocol":PROTOCOL,"engineering_preflight":preflight,"triggered_seeds":triggered,"continuations_completed":results.len(),
        "continuation_steps":results.len()*horizon,"shared_prefix_steps":prefixes,"scientific_advancing_steps":prefixes+results.len()*horizon,
        "fixed_replication_denominator":seeds.len(),"missing":missing,"results":results,"historical_preflight_exact":preflight,"historical_results_pooled":false});
    save(&out.join("summary.json"), &summary)?;
    status(
        out,
        json!({"state":"complete","continuations_completed":results.len(),"scientific_advancing_steps":prefixes+results.len()*horizon}),
    )?;
    Ok(summary)
}
