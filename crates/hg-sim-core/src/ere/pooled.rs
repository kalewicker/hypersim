//! Paired replay of the original E2 cap cases, not an acceptance gate.
use super::experiments::{self, GR1_SAMPLES, GR1_STEPS, MUTATIONS, MUTATION_SEED};
use super::*;
use hg_core::PropertyValue;
use hg_meta::{encode_family, family_parts, read_program, RuleProgram, K_GRAFT_MODE};
use hg_rules::StepMode;
use rand_chacha::ChaCha8Rng;

pub const TRIAL_PROTOCOL: &str = "E2-alternatives-pooled-400k50k-v1";
pub const TRIAL_WORK: usize = 400_000;
pub const TRIAL_RESULTS: usize = 50_000;
pub const REMAINING_CASES: [usize; 8] = [1136, 2518, 3183, 4554, 6859, 7127, 8380, 8679];

struct Trace {
    frames: Vec<(u64, usize, usize, u64)>,
    status: &'static str,
    attempted_step: usize,
    peak_work: usize,
    peak_results: usize,
    final_hash: u64,
    ceiling: serde_json::Value,
}
fn trace(
    p: &RuleProgram,
    core: &CandidateRule,
    steps: usize,
    policy: MatchBudgetPolicy,
    work: usize,
    results: usize,
) -> Trace {
    let mut s = experiments::mutation_fixture(core);
    let mut e = Engine::new(
        p.compile(&MetaLimits::default(), false).unwrap(),
        StepMode::MaxParallel,
        42,
        1,
    );
    e.set_match_budget_policy(work, results, policy);
    let mut t = Trace {
        frames: Vec::new(),
        status: "completed",
        attempted_step: 0,
        peak_work: 0,
        peak_results: 0,
        final_hash: s.canonical_hash(),
        ceiling: serde_json::Value::Null,
    };
    for step in 1..=steps {
        let rng_before = e.rng_state_probe();
        let stats = e.step(&mut s);
        let usage = e.match_budget_usage();
        t.attempted_step = step;
        t.peak_work = t.peak_work.max(usage.work);
        t.peak_results = t.peak_results.max(usage.results);
        if e.match_ceiling_binding() {
            let rules: Vec<_> = usage
                .rules
                .iter()
                .map(|r| {
                    serde_json::json!({
                "rule_id":r.rule_id,"work":r.work,"results":r.results,"truncated":r.truncated,
                "work_allowance":r.work_allowance,"result_allowance":r.result_allowance})
                })
                .collect();
            t.status = "match_ceiling";
            t.ceiling = serde_json::json!({"work":usage.work,"results":usage.results,"rules":rules,
                "accepted":stats.accepted_rewrites,"rng_unchanged":rng_before==e.rng_state_probe(),
                "world_step":s.step(),"nodes":s.node_count(),"edges":s.edge_count()});
            break;
        }
        if s.node_count() > 2000 || s.edge_count() > 4000 {
            t.status = "graph_ceiling";
            break;
        }
        t.frames.push((
            s.canonical_hash(),
            stats.accepted_rewrites,
            stats.attempted_matches,
            e.rng_state_probe(),
        ));
    }
    t.final_hash = s.canonical_hash();
    t
}
fn summary(t: &Trace) -> serde_json::Value {
    serde_json::json!({"status":t.status,"completed_steps":t.frames.len(),
        "last_attempted_step":t.attempted_step,"peak_work":t.peak_work,
        "peak_results":t.peak_results,"final_hash":t.final_hash,"ceiling":t.ceiling})
}
fn candidates(p: &RuleProgram) -> Vec<serde_json::Value> {
    p.candidates()
        .iter()
        .map(|c| {
            serde_json::json!({"rcf_hash":c.rcf_hash(),"priority":c.priority,
        "lhs":c.lhs,"rhs":c.rhs,"labels":c.labels,"guards":format!("{:?}",c.guards)})
        })
        .collect()
}

pub fn replay(original: &serde_json::Value) -> Result<serde_json::Value, String> {
    replay_with_budget(original, 200_000, 20_000, POOLED_PROTOCOL, None)
}

/// Prospective replay of the eight cases unresolved by pooled-v1. Not a gate.
pub fn replay_400k50k(original: &serde_json::Value) -> Result<serde_json::Value, String> {
    replay_with_budget(original, TRIAL_WORK, TRIAL_RESULTS, TRIAL_PROTOCOL, Some(&REMAINING_CASES))
}

fn replay_with_budget(
    original: &serde_json::Value,
    work: usize,
    results: usize,
    protocol: &str,
    subset: Option<&[usize]>,
) -> Result<serde_json::Value, String> {
    if original["protocol"] != EncodingVersion::AlternativeFamiliesV3.protocol()
        || original["kind"] != "diagnostic_replay_not_gate"
        || original["genotype_draws_replayed"] != GR1_SAMPLES
        || original["trajectory_steps"] != GR1_STEPS
        || original["rng_seed"] != MUTATION_SEED
    {
        return Err("replay requires the original full E2 diagnostic".into());
    }
    let mut targets = BTreeMap::new();
    for c in original["cases"].as_array().ok_or("missing cap cases")? {
        let i = c["sample_index_zero_based"]
            .as_u64()
            .ok_or("missing sample index")? as usize;
        if i >= GR1_SAMPLES || targets.insert(i, c).is_some() {
            return Err("invalid/duplicate sample index".into());
        }
    }
    if targets.len() != 14 {
        return Err("expected the 14 original ceiling cases".into());
    }
    if let Some(indices) = subset {
        if indices.iter().any(|i| !targets.contains_key(i)) {
            return Err("missing preregistered replay target".into());
        }
        targets.retain(|i, _| indices.contains(i));
    }
    let bank = super::semantic::parent_bank();
    let limits = MetaLimits::default();
    let mut rng = ChaCha8Rng::seed_from_u64(MUTATION_SEED);
    let mut rows = Vec::new();
    let mut correctness = true;
    let mut rescued = 0;
    for sample in 0..GR1_SAMPLES {
        // Same draw order/types as the frozen E2 assay and original audit.
        let i = rng.gen_range(0..bank.len());
        let (core_index, parent) = &bank[i];
        let mut s = HypergraphStore::new();
        let r = encode_family(
            &mut s,
            &parent.core,
            parent.proposal.as_ref().unwrap(),
            parent.active,
            &limits,
        )
        .unwrap();
        let op = rng.gen_range(0..13);
        let role = if op < 11 {
            let role = if rng.gen_range(0..2) == 0 {
                "Core"
            } else {
                "Proposal"
            };
            let parts = family_parts(&s.snapshot(), r.root, &limits).unwrap();
            experiments::mutate(
                &mut s,
                if role == "Core" {
                    &parts.core
                } else {
                    &parts.proposal
                },
                MUTATIONS[op],
                &mut rng,
            );
            Some(role)
        } else {
            s.get_node_mut(r.root)
                .unwrap()
                .set_property(K_GRAFT_MODE, PropertyValue::Int(i64::from(op == 11)));
            None
        };
        let Some(old) = targets.get(&sample) else {
            continue;
        };
        let child = read_program(&s.snapshot(), r.root, &limits)
            .map_err(|v| format!("replay decode changed: {v:?}"))?;
        if op != 5
            || !parent.active
            || old["parent_state_index"] != i
            || old["core_seed_index"] != *core_index
            || old["target_role"] != serde_json::to_value(role).unwrap()
            || old["executable_candidates"] != serde_json::to_value(candidates(&child)).unwrap()
        {
            return Err(format!(
                "original genotype draw mismatch at sample {sample}"
            ));
        }
        let split = trace(
            &child,
            &parent.core,
            GR1_STEPS,
            MatchBudgetPolicy::Split,
            200_000,
            20_000,
        );
        let pooled = trace(
            &child,
            &parent.core,
            GR1_STEPS,
            MatchBudgetPolicy::Pooled,
            work,
            results,
        );
        // Only an oracle. Its resource limits cannot become a gate receipt.
        let oracle = trace(
            &child,
            &parent.core,
            GR1_STEPS,
            MatchBudgetPolicy::Split,
            20_000_000,
            2_000_000,
        );
        let reproduced = split.status == "match_ceiling"
            && old["attempted_step"] == split.attempted_step
            && split.ceiling["accepted"] == 0
            && split.ceiling["rng_unchanged"] == true;
        let prefix_matches = oracle.frames.len() >= pooled.frames.len()
            && oracle.frames[..pooled.frames.len()] == pooled.frames;
        let split_prefix_matches = oracle.frames.len() >= split.frames.len()
            && oracle.frames[..split.frames.len()] == split.frames;
        let budgets = pooled.peak_work <= work && pooled.peak_results <= results;
        let safe_halt = pooled.status != "match_ceiling"
            || (pooled.ceiling["accepted"] == 0 && pooled.ceiling["rng_unchanged"] == true);
        let pass = reproduced
            && prefix_matches
            && split_prefix_matches
            && budgets
            && safe_halt
            && oracle.status == "completed";
        correctness &= pass;
        rescued += usize::from(pooled.status == "completed");
        rows.push(serde_json::json!({"sample_index_zero_based":sample,"target_role":role,
            "split_reproduced":reproduced,"pooled_completed_prefix_matches_oracle":prefix_matches,
            "split_completed_prefix_matches_oracle":split_prefix_matches,"budgets_respected":budgets,
            "correctness_pass":pass,"split":summary(&split),"pooled":summary(&pooled),"oracle":summary(&oracle)}));
    }
    correctness &= rows.len() == targets.len();
    Ok(
        serde_json::json!({"protocol":protocol,"kind":"targeted_replay_not_gate",
        "match_budget_policy":"Pooled","global_work":work,"global_results":results,
        "oracle_work":20_000_000,"oracle_results":2_000_000,"samples_replayed":GR1_SAMPLES,
        "trajectory_steps":GR1_STEPS,"targeted_cases":rows.len(),"rescued_cases":rescued,
        "remaining_bounded_cases":rows.len()-rescued,"correctness_pass":correctness,"cases":rows}),
    )
}
