//! Precommitted E2 characterization; ordinary Engine trajectories, no repair.
use super::experiments::{
    self, MutationBreakdown, MutationReport, GR1_SAMPLES, GR1_STEPS, MUTATIONS, MUTATION_SEED,
};
use super::*;
use hg_core::PropertyValue;
use hg_meta::{encode_family, family_parts, read_program, RuleProgram, K_GRAFT_MODE};
use hg_rules::StepMode;
use rand_chacha::ChaCha8Rng;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SemanticStrata {
    pub structural: MutationBreakdown,
    pub active_parent_structural: MutationBreakdown,
    pub activation: MutationBreakdown,
    pub target_roles: BTreeMap<String, MutationBreakdown>,
    pub mode_changes: usize,
    pub trajectory_changing_activation: usize,
}
impl MutationBreakdown {
    fn record(&mut self, outcome: &str) {
        self.samples += 1;
        self.valid += usize::from(outcome != "quarantined");
        match outcome {
            "neutral" => self.neutral += 1,
            "near_neutral" => self.near_neutral += 1,
            "divergent" => self.divergent += 1,
            "quarantined" => self.quarantined += 1,
            "frozen" => self.frozen += 1,
            "exploding" => self.exploding += 1,
            _ => self.unmeasured += 1,
        }
    }
    fn consistent(&self) -> bool {
        self.valid + self.quarantined == self.samples
            && self.neutral
                + self.near_neutral
                + self.divergent
                + self.frozen
                + self.exploding
                + self.unmeasured
                == self.valid
    }
    fn thresholds(&self, near_required: bool) -> bool {
        self.consistent()
            && self.samples > 0
            && self.unmeasured == 0
            && self.valid as f64 / self.samples as f64 >= 0.40
            && (!near_required || self.near_neutral as f64 / self.samples as f64 >= 0.10)
            && (self.frozen + self.exploding) as f64 / self.samples as f64 <= 0.25
    }
}
impl SemanticStrata {
    pub fn pass(&self) -> bool {
        self.structural.thresholds(true)
            && self.active_parent_structural.samples >= 2000
            && self.active_parent_structural.thresholds(true)
            && self.activation.samples >= 1000
            && self.activation.thresholds(false)
            && self.mode_changes >= 100
            && self.mode_changes <= self.activation.samples
            && self.trajectory_changing_activation >= 20
            && self.trajectory_changing_activation <= self.mode_changes
            && self.target_roles.len() == 2
            && ["Core", "Proposal"].iter().all(|key| {
                self.target_roles
                    .get(*key)
                    .is_some_and(|r| r.samples > 0 && r.consistent())
            })
    }
}

/// Recompute strata acceptance, not just trust the serialized top-level flag.
pub fn validate_receipt(value: &serde_json::Value) -> Result<(), String> {
    let s: SemanticStrata = serde_json::from_value(value["semantic"].clone())
        .map_err(|e| format!("missing/malformed E2 strata: {e}"))?;
    let n = |k: &str| value[k].as_u64().unwrap_or(0) as usize;
    let roles: Vec<_> = s.target_roles.values().collect();
    let rows = [&s.structural, &s.activation];
    let totals = rows.iter().map(|r| r.samples).sum::<usize>() == n("samples")
        && rows.iter().map(|r| r.valid).sum::<usize>() == n("valid")
        && rows.iter().map(|r| r.quarantined).sum::<usize>() == n("quarantined")
        && rows.iter().map(|r| r.neutral).sum::<usize>() == n("neutral")
        && rows.iter().map(|r| r.near_neutral).sum::<usize>() == n("near_neutral")
        && rows.iter().map(|r| r.divergent).sum::<usize>() == n("divergent")
        && rows.iter().map(|r| r.frozen + r.exploding).sum::<usize>() == n("catastrophic")
        && roles.iter().map(|r| r.samples).sum::<usize>() == s.structural.samples
        && s.active_parent_structural.samples <= s.structural.samples;
    if n("seeds") != 168 || !totals || !s.pass() {
        return Err("E2 activation/active-structural gate has not passed".into());
    }
    Ok(())
}

pub fn parent_bank() -> Vec<(usize, RuleProgram)> {
    let mut parents = Vec::new();
    for (i, core) in experiments::seed_bank().into_iter().enumerate() {
        for variant in 0..3 {
            let mut proposal = core.clone();
            proposal.priority += 1;
            match variant {
                1 => proposal.labels[0] = Some(1),
                2 => proposal.rhs.push(vec![0, 0]),
                _ => (),
            }
            for active in [false, true] {
                parents.push((
                    i,
                    RuleProgram {
                        core: core.clone(),
                        proposal: Some(proposal.clone()),
                        active,
                    },
                ));
            }
        }
    }
    parents
}
struct Trace {
    frames: Vec<(u64, usize)>,
    explosive: bool,
}
#[derive(Default, Serialize)]
struct ClassificationUsage {
    #[serde(skip)]
    previously_unmeasured_classifications: BTreeMap<usize, String>,
    traced_trajectories: usize,
    attempted_steps: usize,
    total_work: u64,
    total_results: u64,
    peak_step_work: usize,
    peak_step_results: usize,
    match_ceiling_steps: usize,
    graph_ceiling_steps: usize,
    budget_violations: usize,
}
fn trajectory(
    p: &RuleProgram,
    world: HypergraphStore,
    steps: usize,
    policy: MatchBudgetPolicy,
    budget: (usize, usize),
    usage: &mut ClassificationUsage,
) -> Result<Trace, String> {
    let initial_nodes = world.node_count();
    let initial_edges = world.edge_count();
    let mut s = world;
    let mut e = Engine::new(
        p.compile(&MetaLimits::default(), false)
            .map_err(|v| format!("{v:?}"))?,
        StepMode::MaxParallel,
        42,
        1,
    );
    e.set_match_budget_policy(budget.0, budget.1, policy);
    usage.traced_trajectories += 1;
    let mut trace = Trace {
        frames: Vec::new(),
        explosive: false,
    };
    for _ in 0..steps {
        let stats = e.step(&mut s);
        let matching = e.match_budget_usage();
        usage.attempted_steps += 1;
        usage.total_work += matching.work as u64;
        usage.total_results += matching.results as u64;
        usage.peak_step_work = usage.peak_step_work.max(matching.work);
        usage.peak_step_results = usage.peak_step_results.max(matching.results);
        usage.budget_violations += usize::from(policy == MatchBudgetPolicy::Pooled
            && (matching.work > budget.0 || matching.results > budget.1));
        usage.match_ceiling_steps += usize::from(e.match_ceiling_binding());
        usage.graph_ceiling_steps += usize::from(s.node_count() > 2000 || s.edge_count() > 4000);
        if e.match_ceiling_binding() || s.node_count() > 2000 || s.edge_count() > 4000 {
            return Err("classification ceiling bound".into());
        }
        trace
            .frames
            .push((s.canonical_hash(), stats.accepted_rewrites));
        trace.explosive |= s.node_count() > initial_nodes * 4 || s.edge_count() > initial_edges * 4;
    }
    Ok(trace)
}

pub fn mutation_gate(samples: usize, steps: usize) -> MutationReport {
    mutation_gate_with_policy(samples, steps, MatchBudgetPolicy::Split)
}
pub fn mutation_gate_with_policy(
    samples: usize,
    steps: usize,
    policy: MatchBudgetPolicy,
) -> MutationReport {
    mutation_gate_with_budget(samples, steps, policy, (200_000, 20_000),
        gate_protocol(EncodingVersion::AlternativeFamiliesV3, policy).unwrap()).0
}

/// A fixed, separately identified resource-envelope trial; never changes defaults.
pub fn mutation_gate_400k50k(samples: usize, steps: usize) -> serde_json::Value {
    let (report, usage) = mutation_gate_with_budget(samples, steps, MatchBudgetPolicy::Pooled,
        (super::pooled::TRIAL_WORK, super::pooled::TRIAL_RESULTS), super::pooled::TRIAL_PROTOCOL);
    let mut value = serde_json::to_value(report).unwrap();
    value["global_work"] = super::pooled::TRIAL_WORK.into();
    value["global_results"] = super::pooled::TRIAL_RESULTS.into();
    value["rng_seed"] = MUTATION_SEED.into();
    value["kind"] = "classification_resource_envelope_trial".into();
    value["previously_unmeasured_classifications"] =
        serde_json::to_value(&usage.previously_unmeasured_classifications).unwrap();
    value["resource_usage"] = serde_json::to_value(usage).unwrap();
    value
}

fn mutation_gate_with_budget(
    samples: usize,
    steps: usize,
    policy: MatchBudgetPolicy,
    budget: (usize, usize),
    protocol: &str,
) -> (MutationReport, ClassificationUsage) {
    let bank = parent_bank();
    let mut usage = ClassificationUsage::default();
    let parent_traces: Vec<_> = bank
        .iter()
        .map(|(_, p)| trajectory(p, experiments::mutation_fixture(&p.core), steps, policy, budget, &mut usage))
        .collect();
    let limits = MetaLimits::default();
    let mut rng = ChaCha8Rng::seed_from_u64(MUTATION_SEED);
    let mut strata = SemanticStrata::default();
    let mut r = MutationReport {
        protocol: protocol.into(),
        match_budget_policy: (policy == MatchBudgetPolicy::Pooled).then_some(policy),
        samples,
        seeds: bank.len(),
        trajectory_steps: steps,
        valid: 0,
        neutral: 0,
        near_neutral: 0,
        divergent: 0,
        quarantined: 0,
        catastrophic: 0,
        unmeasured: 0,
        at_cap_samples: 0,
        at_cap_valid: 0,
        neutral_soundness_checked: 0,
        neutral_soundness_failures: 0,
        quarantine_by_validator: [0; 9],
        pass: false,
        by_operator: BTreeMap::new(),
        by_seed_shape: BTreeMap::new(),
        semantic: None,
    };
    for sample in 0..samples {
        let i = rng.gen_range(0..bank.len());
        let (core_index, parent) = &bank[i];
        let at_cap = parent
            .core
            .lhs
            .iter()
            .chain(&parent.core.rhs)
            .any(|e| e.len() == limits.max_arity);
        r.at_cap_samples += usize::from(at_cap);
        let mut store = HypergraphStore::new();
        let region = encode_family(
            &mut store,
            &parent.core,
            parent.proposal.as_ref().unwrap(),
            parent.active,
            &limits,
        )
        .unwrap();
        let op = rng.gen_range(0..13);
        let (name, role) = if op < 11 {
            let role = if rng.gen_range(0..2) == 0 {
                "Core"
            } else {
                "Proposal"
            };
            let parts = family_parts(&store.snapshot(), region.root, &limits).unwrap();
            let target = if role == "Core" {
                &parts.core
            } else {
                &parts.proposal
            };
            experiments::mutate(&mut store, target, MUTATIONS[op], &mut rng);
            (format!("{:?}", MUTATIONS[op]), Some(role))
        } else {
            let active = op == 11;
            strata.mode_changes += usize::from(active != parent.active);
            store
                .get_node_mut(region.root)
                .unwrap()
                .set_property(K_GRAFT_MODE, PropertyValue::Int(i64::from(active)));
            (
                (if active { "Activate" } else { "Deactivate" }).into(),
                None,
            )
        };
        let outcome = match read_program(&store.snapshot(), region.root, &limits) {
            Err(v) => {
                r.quarantined += 1;
                r.quarantine_by_validator[v as usize] += 1;
                "quarantined"
            }
            Ok(child) => {
                r.valid += 1;
                r.at_cap_valid += usize::from(at_cap);
                let neutral = child.rcf_bytes() == parent.rcf_bytes();
                // Neutral soundness cannot pass on two unmeasured traces.
                let trace = if !neutral || r.neutral_soundness_checked < 50 {
                    Some(trajectory(
                        &child,
                        experiments::mutation_fixture(&parent.core),
                        steps,
                        policy,
                        budget,
                        &mut usage,
                    ))
                } else {
                    None
                };
                if parent_traces[i].is_err() {
                    r.unmeasured += 1;
                    "unmeasured"
                } else if neutral {
                    r.neutral += 1;
                    if let Some(t) = trace {
                        r.neutral_soundness_checked += 1;
                        if !matches!((&parent_traces[i], t), (Ok(a), Ok(b)) if a.frames == b.frames)
                        {
                            r.neutral_soundness_failures += 1;
                        }
                    }
                    "neutral"
                } else {
                    match (&parent_traces[i], trace.unwrap()) {
                        (Ok(a), Ok(b)) if a.frames == b.frames => {
                            r.near_neutral += 1;
                            "near_neutral"
                        }
                        (Ok(_), Ok(b)) => {
                            if role.is_none() {
                                strata.trajectory_changing_activation += 1;
                            }
                            if b.explosive
                                || b.frames
                                    .windows(50.min(steps).max(1))
                                    .any(|w| w.iter().all(|(_, fires)| *fires == 0))
                            {
                                r.catastrophic += 1;
                                if b.explosive {
                                    "exploding"
                                } else {
                                    "frozen"
                                }
                            } else {
                                r.divergent += 1;
                                "divergent"
                            }
                        }
                        _ => {
                            r.unmeasured += 1;
                            "unmeasured"
                        }
                    }
                }
            }
        };
        if super::pooled::REMAINING_CASES.contains(&sample) {
            usage.previously_unmeasured_classifications.insert(sample, outcome.into());
        }
        r.by_operator.entry(name).or_default().record(outcome);
        r.by_seed_shape
            .entry(core_index / 4)
            .or_default()
            .record(outcome);
        if let Some(role) = role {
            strata.structural.record(outcome);
            strata
                .target_roles
                .entry(role.into())
                .or_default()
                .record(outcome);
            if parent.active {
                strata.active_parent_structural.record(outcome);
            }
        } else {
            strata.activation.record(outcome);
        }
    }
    r.pass = samples >= GR1_SAMPLES
        && steps >= GR1_STEPS
        && usage.budget_violations == 0
        && r.unmeasured == 0
        && r.valid as f64 / samples as f64 >= 0.40
        && r.near_neutral as f64 / samples as f64 >= 0.10
        && r.catastrophic as f64 / samples as f64 <= 0.25
        && r.neutral_soundness_checked >= 50
        && r.neutral_soundness_failures == 0
        && r.at_cap_samples > 0
        && strata.pass();
    r.semantic = Some(strata);
    (r, usage)
}

/// Post-result diagnostic, NOT a second gate or a new mutation kernel. Replay
/// the same genotype draws but trace only the active AddPatternEdge stratum
/// containing every original unmeasured case. Limits are unchanged.
pub fn audit_classification_ceiling(samples: usize, steps: usize) -> serde_json::Value {
    let bank = parent_bank();
    let limits = MetaLimits::default();
    let mut rng = ChaCha8Rng::seed_from_u64(MUTATION_SEED);
    let mut cases = Vec::new();
    let mut traced = 0;
    for sample in 0..samples {
        // This draw order must stay exactly paired with mutation_gate.
        let i = rng.gen_range(0..bank.len());
        let (core_index, parent) = &bank[i];
        let mut s = HypergraphStore::new();
        let region = encode_family(
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
            let parts = family_parts(&s.snapshot(), region.root, &limits).unwrap();
            let target = if role == "Core" {
                &parts.core
            } else {
                &parts.proposal
            };
            experiments::mutate(&mut s, target, MUTATIONS[op], &mut rng);
            Some(role)
        } else {
            s.get_node_mut(region.root)
                .unwrap()
                .set_property(K_GRAFT_MODE, PropertyValue::Int(i64::from(op == 11)));
            None
        };
        if op != 5 || !parent.active {
            continue;
        }
        let Ok(child) = read_program(&s.snapshot(), region.root, &limits) else {
            continue;
        };
        if child.rcf_bytes() == parent.rcf_bytes() {
            continue;
        }
        traced += 1;
        let mut world = experiments::mutation_fixture(&parent.core);
        let mut e = Engine::new(
            child.compile(&limits, false).unwrap(),
            StepMode::MaxParallel,
            42,
            1,
        );
        e.set_match_budget(200_000, 20_000);
        for step in 1..=steps {
            let stats = e.step(&mut world);
            if e.match_ceiling_binding() || world.node_count() > 2000 || world.edge_count() > 4000 {
                let rules: Vec<_> = e.rules().iter().enumerate().map(|(id,r)| {
                    let (bindings,g) = hg_engine::find_bindings_counted(&world.snapshot(),r,id);
                    serde_json::json!({"rule_id":id,"enumeration_work":g.enumeration_work,
                        "enumeration_truncated":g.enumeration_truncated,"returned_bindings":bindings.len(),
                        "work_limit":r.pattern.match_work_limit,"result_limit":r.pattern.match_result_limit})
                }).collect();
                let candidates: Vec<_> = child
                    .candidates()
                    .iter()
                    .map(|c| {
                        serde_json::json!({
                    "rcf_hash":c.rcf_hash(),"priority":c.priority,"lhs":c.lhs,"rhs":c.rhs,
                    "labels":c.labels,"guards":format!("{:?}",c.guards)})
                    })
                    .collect();
                cases.push(serde_json::json!({"sample_index_zero_based":sample,"parent_state_index":i,
                    "core_seed_index":core_index,"proposal_variant_index":(i/2)%3,"target_role":role,
                    "operator":"AddPatternEdge","parent_active":parent.active,"attempted_step":step,
                    "last_completed_world_step":world.step(),"nodes":world.node_count(),"edges":world.edge_count(),
                    "match_ceiling_binding":e.match_ceiling_binding(),"accepted_on_ceiling_step":stats.accepted_rewrites,
                    "rules":rules,"executable_candidates":candidates}));
                break;
            }
        }
    }
    serde_json::json!({"protocol":EncodingVersion::AlternativeFamiliesV3.protocol(),
        "kind":"diagnostic_replay_not_gate","rng_seed":MUTATION_SEED,"genotype_draws_replayed":samples,
        "trajectory_steps":steps,"filter":"active-parent AddPatternEdge, valid RCF-changing mutants",
        "cases_traced":traced,"unmeasured_cases":cases.len(),"cases":cases})
}
