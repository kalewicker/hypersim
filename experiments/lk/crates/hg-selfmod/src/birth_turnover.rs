//! v13: a conditional ON/OFF comparison from frozen v12 birth checkpoints.
use super::*;
pub const PROTOCOL: &str = "selfmod-v13-turnover-v1";
pub const CONTRACT: &str = include_str!("../../../docs/selfmod_v13_turnover_preregistration.md");
pub const CONDITIONS: [(Arm, bool); 4] = [
    (Arm::Faithful, true),
    (Arm::Faithful, false),
    (Arm::Prune, true),
    (Arm::Prune, false),
];
pub fn name(arm: Arm, on: bool) -> String {
    format!("{}_{}", arm.name(), if on { "on" } else { "off" })
}
pub(super) fn finish_phase(s: &mut State, mut row: Row, on: bool) -> Row {
    if on {
        return finish(s, row, None).0;
    }
    // Suppress the call itself, without altering any financial/resource parameter.
    // No birth metadata, transfers or RNG draws occur. Native refill still runs.
    refill_creation_budgets(&mut s.rs, &s.cfg.economy_params());
    row.stocks = stocks(s);
    row.ledger = ledger(s);
    assert_eq!(row.ledger.audit_residual, 0);
    row.rng_word_pos = u64::try_from(s.rng.get_word_pos()).unwrap();
    row.rng_probe = probe(&s.rng);
    row.rule_cap_binding = row.stocks.len() >= s.cfg.rule_cap;
    row
}
fn continue_off(out: &Path, mut s: State, selected: Selection) -> io::Result<Value> {
    let mut file = BufWriter::new(
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(out.join("rows.jsonl"))?,
    );
    let mut totals = Totals::new(s.cfg.steps);
    let mut completed = 0;
    let mut error = None;
    for step in 1..=s.cfg.steps {
        let mut row = object(&mut s);
        row.continuation_step = Some(step);
        let row = finish_phase(&mut s, row, false);
        totals.add(&row, &selected, s.cfg.steps);
        write_row(&mut file, &row)?;
        completed += 1;
        if s.world.node_count() > s.cfg.node_explosion_cap {
            error = Some("hard node explosion stop".into());
            break;
        }
    }
    let score = verdict(&s, &selected, &totals, completed, &error);
    save(&out.join("final.json"), &snapshot(&s))?;
    save(&out.join("score.json"), &score)?;
    if error.is_some() {
        return Err(io::Error::other(
            "incomplete OFF continuation; raw retained",
        ));
    }
    Ok(score)
}
fn branch(fork: &Value, expected: &Value, out: &Path, arm: Arm, on: bool) -> io::Result<Value> {
    fs::create_dir(out)?;
    let mut s = restore(fork["checkpoint"].clone())?;
    let pending = serde_json::from_value(fork["pending"].clone())?;
    let (row, selection) = finish(&mut s, pending, Some(arm));
    let selection = selection.unwrap();
    let birth = json!({"selection":selection,"row":row,"initial":snapshot(&s)});
    assert_eq!(&birth, expected, "initial birth changed from frozen v12");
    save(&out.join("birth.json"), &birth)?;
    if on {
        continuation(out, s, selection)
    } else {
        continue_off(out, s, selection)
    }
}
fn read(path: &Path) -> io::Result<Value> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
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
    let seeds: &[u64] = if preflight { &[7, 42] } else { &SEEDS };
    let horizon = if preflight { 64 } else { 2000 };
    save(
        &out.join("manifest.json"),
        &json!({"protocol":PROTOCOL,"engineering_preflight":preflight,
        "seeds":seeds,"horizon":horizon,"arms":CONDITIONS.iter().map(|(a,on)|name(*a,*on)).collect::<Vec<_>>(),
        "continuations_requested":seeds.len()*4,"prefix_steps":0,"input_panel":"v12 precision-repaired",
        "future_mutation":false,"replication_threshold":6}),
    )?;
    save(
        &out.join("preregistration.json"),
        &json!({"contract":CONTRACT}),
    )?;
    fs::write(out.join("preregistration.md"), CONTRACT)?;
    status(out, json!({"state":"running","continuations_completed":0}))?;
    let mut results = Vec::new();
    for &seed in seeds {
        let input = out.join(format!("inputs/seed_{seed}"));
        let fork = read(&input.join("fork.json"))?;
        assert_eq!(fork["checkpoint"]["cfg"]["steps"], horizon);
        let dir = out.join(format!("seed_{seed}"));
        fs::create_dir(&dir)?;
        for (arm, on) in CONDITIONS {
            let condition = name(arm, on);
            let expected = read(&input.join(arm.name()).join("birth.json"))?;
            let arm_dir = dir.join(&condition);
            let score = branch(&fork, &expected, &arm_dir, arm, on)?;
            assert_eq!(
                fs::read(arm_dir.join("birth.json"))?,
                fs::read(input.join(arm.name()).join("birth.json"))?
            );
            if on {
                for file in ["rows.jsonl", "score.json", "final.json"] {
                    assert_eq!(
                        fs::read(arm_dir.join(file))?,
                        fs::read(input.join(arm.name()).join(file))?,
                        "ON control diverged: {seed}/{condition}/{file}"
                    );
                }
            }
            results.push(json!({"seed":seed,"arm":condition,"score":score}));
            status(
                out,
                json!({"state":"running","continuations_completed":results.len(),"scientific_advancing_steps":results.len()*horizon,"active_seed":seed}),
            )?;
            println!(
                "seed {seed} {condition}: {}/{} recorded",
                results.len(),
                seeds.len() * 4
            );
            io::stdout().flush()?;
        }
    }
    let summary = json!({"protocol":PROTOCOL,"engineering_preflight":preflight,"results":results,
        "continuations_completed":results.len(),"scientific_advancing_steps":results.len()*horizon,
        "new_off_condition_steps":seeds.len()*2*horizon,"repeated_on_control_steps":seeds.len()*2*horizon,
        "shared_prefix_steps":0,"on_controls_byte_identical":true,"duplicates_pooled":false});
    save(&out.join("summary.json"), &summary)?;
    status(
        out,
        json!({"state":"complete","continuations_completed":results.len(),"scientific_advancing_steps":results.len()*horizon}),
    )?;
    Ok(summary)
}
pub fn replay(
    fork: &Path,
    expected_birth: &Path,
    arm: Arm,
    on: bool,
    out: &Path,
) -> io::Result<Value> {
    assert!(matches!(arm, Arm::Faithful | Arm::Prune));
    branch(&read(fork)?, &read(expected_birth)?, out, arm, on)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn disabled_birth_keeps_qualified_parent_capital_and_rng_but_refills() {
        let mut s = initialize(7, 64);
        for root in rule_roots(&s.rs) {
            set_tokens(&mut s.rs, root, 120);
        }
        s.led.cumulative_influx += 180;
        let mut native = s.clone();
        let pending = object(&mut s);
        let native_pending = object(&mut native);
        assert_eq!(pending, native_pending);
        let before = stocks(&s);
        let genes = s.genes.clone();
        let seq = s.seq;
        let pos = s.rng.get_word_pos();
        let rng = probe(&s.rng);
        let off = finish_phase(&mut s, pending, false);
        let on = finish_phase(&mut native, native_pending, true);
        assert!(off.births.is_empty());
        assert!(!on.births.is_empty());
        assert_eq!(s.seq, seq);
        assert_eq!(s.genes, genes);
        assert_eq!(pos, s.rng.get_word_pos());
        assert_eq!(rng, probe(&s.rng));
        for (a, b) in before.iter().zip(off.stocks.iter()) {
            assert_eq!(a.id, b.id);
            assert_eq!(a.capital, b.capital);
            assert_eq!(b.budget, (a.budget + 1).min(200));
        }
        assert_eq!(off.ledger, on.ledger);
        assert_eq!(off.world_hash, on.world_hash);
    }
    #[test]
    fn switch_starts_after_identical_paid_birth_and_first_object() {
        let mut s = initialize(42, 64);
        for root in rule_roots(&s.rs) {
            set_tokens(&mut s.rs, root, 120);
        }
        s.led.cumulative_influx += 180;
        let pending = object(&mut s);
        let checkpoint = snapshot(&s);
        for arm in [Arm::Faithful, Arm::Prune] {
            let mut on = restore(checkpoint.clone()).unwrap();
            let mut off = restore(checkpoint.clone()).unwrap();
            let a = finish(&mut on, pending.clone(), Some(arm));
            let b = finish(&mut off, pending.clone(), Some(arm));
            assert_eq!(a, b);
            assert_eq!(snapshot(&on), snapshot(&off));
            let a = object(&mut on);
            let b = object(&mut off);
            assert_eq!(a, b);
            finish_phase(&mut on, a, true);
            finish_phase(&mut off, b, false);
            let saved = snapshot(&off);
            let mut restored = restore(saved.clone()).unwrap();
            assert_eq!(saved, snapshot(&restored));
            for _ in 0..64 {
                let a = object(&mut off);
                let b = object(&mut restored);
                assert_eq!(
                    finish_phase(&mut off, a, false),
                    finish_phase(&mut restored, b, false)
                );
            }
        }
    }
}
