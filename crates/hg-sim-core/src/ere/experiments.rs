//! Gate and experiment harnesses, using the production Engine path.
use super::*;
use hg_core::{HyperEdge, Label, NodeData, PropertyValue};
use hg_rules::{compile_rule, StepMode};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::path::Path;

pub const GR1_SAMPLES: usize = 10_000;
pub const GR1_STEPS: usize = 100;
pub const GR2_STEPS: usize = 1000;
pub const GR3_STEPS: usize = 100_000;
pub const EPOCH_SWEEP: [Option<u64>; 6] =
    [None, Some(10_000), Some(1000), Some(100), Some(10), Some(1)];
pub const MUTATION_SEED: u64 = 0xE1_5231_0001;

/// >=20 RCF-distinct structural/guarded seeds, not merely priority copies.
pub fn seed_bank() -> Vec<CandidateRule> {
    let shapes = [
        (vec![vec![0, 1]], vec![vec![0, 1]]),
        (vec![vec![0, 1]], vec![vec![0, 2], vec![1, 2]]),
        (vec![vec![0, 1, 2]], vec![vec![0, 1], vec![1, 2]]),
        (vec![vec![0, 1], vec![1, 2]], vec![vec![0, 2]]),
        (vec![vec![0, 1], vec![1, 2]], vec![vec![0, 1, 2]]),
        (
            vec![vec![0, 1, 2]],
            vec![vec![0, 1, 3], vec![1, 2, 3], vec![0, 2, 3]],
        ),
        // Ceiling-adjacent SB-R1 parent without factorial eight-var matching.
        (
            vec![vec![0, 1, 0, 1, 0, 1, 0, 1]],
            vec![vec![0, 1, 0, 1, 0, 1, 0, 1]],
        ),
    ];
    let mut seeds = Vec::new();
    for (lhs, rhs) in shapes {
        let n = lhs.iter().chain(&rhs).flatten().copied().max().unwrap() + 1;
        for variant in 0..4 {
            let mut c = CandidateRule {
                lhs: lhs.clone(),
                rhs: rhs.clone(),
                labels: vec![None; n],
                ..CandidateRule::default()
            };
            if variant & 1 != 0 {
                c.labels[0] = Some(0);
            }
            if variant & 2 != 0 {
                c.guards.push(hg_meta::Guard {
                    var: 0,
                    key: 1,
                    op: hg_core::PropertyOp::Gte,
                    value: PropertyValue::Int(0),
                });
            }
            seeds.push(c);
        }
    }
    seeds
}

pub fn fixture(c: &CandidateRule) -> HypergraphStore {
    let mut s = HypergraphStore::new();
    let ids: Vec<_> = c
        .labels
        .iter()
        .map(|l| {
            let mut n = NodeData::new(Label(l.unwrap_or(0)), 0);
            n.set_property(1, PropertyValue::Int(100));
            s.add_node(n)
        })
        .collect();
    for e in &c.lhs {
        s.add_edge(HyperEdge::new(
            Label(0),
            e.iter().map(|v| ids[*v]).collect(),
        ));
    }
    s
}

/// Identity checks use the engine directly, not a second interpreter.
fn trajectory(c: &CandidateRule, steps: usize) -> Result<Vec<(u64, usize)>, String> {
    trajectory_on(c, mutation_fixture(c), steps)
}
pub(super) fn mutation_fixture(c: &CandidateRule) -> HypergraphStore {
    let mut s = fixture(c);
    // Fixed neutral background gives meaningful 4x explosion thresholds,
    // rather than treating a single legitimate birth as "explosion".
    let ids: Vec<_> = (0..64)
        .map(|_| {
            let mut n = NodeData::new(Label(0), 0);
            n.set_property(1, PropertyValue::Int(100));
            s.add_node(n)
        })
        .collect();
    for i in 0..96 {
        s.add_edge(HyperEdge::new(
            Label(0),
            [ids[i % 64], ids[(i + 1) % 64]].into_iter().collect(),
        ));
    }
    s
}
fn trajectory_on(
    c: &CandidateRule,
    mut s: HypergraphStore,
    steps: usize,
) -> Result<Vec<(u64, usize)>, String> {
    let mut engine = Engine::new(
        vec![c
            .compile(&MetaLimits::default(), false)
            .map_err(|v| format!("{v:?}"))?],
        StepMode::MaxParallel,
        42,
        1,
    );
    engine.set_match_budget(200_000, 20_000);
    let mut frames = Vec::new();
    for _ in 0..steps {
        let stats = engine.step(&mut s);
        if engine.match_ceiling_binding() || s.node_count() > 2000 || s.edge_count() > 4000 {
            return Err("classification ceiling bound".into());
        }
        frames.push((s.canonical_hash(), stats.accepted_rewrites));
    }
    Ok(frames)
}

#[derive(Clone, Copy, Debug, Serialize)]
pub enum MutationOp {
    AddSlot,
    RemoveSlot,
    RetargetSlot,
    AddVar,
    RemoveVar,
    AddPatternEdge,
    RemovePatternEdge,
    AddLabel,
    RemoveLabel,
    ChangeLabel,
    ChangePriority,
}
pub const MUTATIONS: [MutationOp; 11] = [
    MutationOp::AddSlot,
    MutationOp::RemoveSlot,
    MutationOp::RetargetSlot,
    MutationOp::AddVar,
    MutationOp::RemoveVar,
    MutationOp::AddPatternEdge,
    MutationOp::RemovePatternEdge,
    MutationOp::AddLabel,
    MutationOp::RemoveLabel,
    MutationOp::ChangeLabel,
    MutationOp::ChangePriority,
];

/// One graph edit. Inapplicable draws are honest Neutral samples. There is
/// no repair/resampling loop to inflate Valid or NearNeutral statistics.
pub(super) fn mutate(
    s: &mut HypergraphStore,
    region: &hg_meta::EncodedRegion,
    op: MutationOp,
    rng: &mut ChaCha8Rng,
) {
    use hg_meta::*;
    let select =
        |xs: &[NodeId], rng: &mut ChaCha8Rng| xs.get(rng.gen_range(0..xs.len().max(1))).copied();
    let vars: Vec<_> = region
        .nodes
        .iter()
        .copied()
        .filter(|&n| s.get_node(n).is_some_and(|n| n.label.0 == L_VAR))
        .collect();
    let pats: Vec<_> = region
        .nodes
        .iter()
        .copied()
        .filter(|&n| s.get_node(n).is_some_and(|n| n.label.0 == L_PAT_EDGE))
        .collect();
    let Some(var) = select(&vars, rng) else {
        return;
    };
    let Some(pat) = select(&pats, rng) else {
        return;
    };
    let slots: Vec<_> = s
        .edges_incident_to(pat)
        .iter()
        .copied()
        .filter(|&e| {
            s.get_edge(e)
                .is_some_and(|e| (E_SLOT_BASE..E_HASLABEL).contains(&e.label.0))
        })
        .collect();
    let slot = slots.get(rng.gen_range(0..slots.len().max(1))).copied();
    let make_edge = |s: &mut HypergraphStore, label, ns: &[NodeId]| {
        s.add_edge(HyperEdge::new(Label(label), ns.iter().copied().collect()));
    };
    match op {
        MutationOp::AddSlot => {
            let used: HashSet<_> = slots
                .iter()
                .filter_map(|&e| s.get_edge(e))
                .map(|e| e.label.0)
                .collect();
            if let Some(k) = (0..9).find(|k| !used.contains(&(E_SLOT_BASE + k))) {
                make_edge(s, E_SLOT_BASE + k, &[pat, var]);
            }
        }
        MutationOp::RemoveSlot => {
            if let Some(e) = slot {
                s.remove_edge(e);
            }
        }
        MutationOp::RetargetSlot => {
            if let Some(e) = slot {
                let label = s.get_edge(e).unwrap().label.0;
                s.remove_edge(e);
                make_edge(s, label, &[pat, var]);
            }
        }
        MutationOp::AddVar => {
            if vars.len() < 8 {
                let used: HashSet<_> = vars
                    .iter()
                    .filter_map(|&v| s.get_node(v)?.get_property(K_ORDER))
                    .filter_map(|p| {
                        if let PropertyValue::Int(i) = p {
                            Some(i)
                        } else {
                            None
                        }
                    })
                    .collect();
                let mut n = NodeData::new(Label(L_VAR), 0);
                n.set_property(
                    K_ORDER,
                    PropertyValue::Int((0..8).find(|i| !used.contains(i)).unwrap()),
                );
                let n = s.add_node(n);
                let used: HashSet<_> = slots
                    .iter()
                    .filter_map(|&e| s.get_edge(e))
                    .map(|e| e.label.0)
                    .collect();
                if let Some(k) = (0..9).find(|k| !used.contains(&(E_SLOT_BASE + k))) {
                    make_edge(s, E_SLOT_BASE + k, &[pat, n]);
                }
            }
        }
        MutationOp::RemoveVar => s.remove_node(var),
        MutationOp::AddPatternEdge => {
            let sides: Vec<_> = region
                .nodes
                .iter()
                .copied()
                .filter(|&n| {
                    s.get_node(n)
                        .is_some_and(|n| n.label.0 == L_SIDE_L || n.label.0 == L_SIDE_R)
                })
                .collect();
            if let Some(side) = select(&sides, rng) {
                let p = s.add_node(NodeData::new(Label(L_PAT_EDGE), 0));
                make_edge(s, E_MEMBER, &[side, p]);
                make_edge(s, E_SLOT_BASE, &[p, var]);
                if let Some(v2) = select(&vars, rng) {
                    make_edge(s, E_SLOT_BASE + 1, &[p, v2]);
                }
            }
        }
        MutationOp::RemovePatternEdge => s.remove_node(pat),
        MutationOp::AddLabel => {
            if !s
                .edges_incident_to(var)
                .iter()
                .any(|&e| s.get_edge(e).is_some_and(|e| e.label.0 == E_HASLABEL))
            {
                let mut n = NodeData::new(Label(L_LABEL_LIT), 0);
                n.set_property(K_LITERAL, PropertyValue::Int(rng.gen_range(0..3)));
                let lit = s.add_node(n);
                make_edge(s, E_HASLABEL, &[var, lit]);
            }
        }
        MutationOp::RemoveLabel | MutationOp::ChangeLabel => {
            let labels: Vec<_> = s
                .edges_incident_to(var)
                .iter()
                .filter_map(|&e| s.get_edge(e))
                .filter(|e| e.label.0 == E_HASLABEL)
                .map(|e| e.nodes[1])
                .collect();
            if let Some(lit) = select(&labels, rng) {
                if matches!(op, MutationOp::RemoveLabel) {
                    s.remove_node(lit);
                } else {
                    s.get_node_mut(lit)
                        .unwrap()
                        .set_property(K_LITERAL, PropertyValue::Int(rng.gen_range(0..3)));
                }
            }
        }
        MutationOp::ChangePriority => {
            let n = s.get_node_mut(region.root).unwrap();
            if let Some(PropertyValue::Int(p)) = n.get_property(K_PRIORITY) {
                n.set_property(K_PRIORITY, PropertyValue::Int(p + 1));
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MutationBreakdown {
    pub samples: usize,
    pub valid: usize,
    pub neutral: usize,
    pub near_neutral: usize,
    pub divergent: usize,
    pub quarantined: usize,
    pub frozen: usize,
    pub exploding: usize,
    pub unmeasured: usize,
}
impl Default for MutationBreakdown {
    fn default() -> Self {
        Self {
            samples: 0,
            valid: 0,
            neutral: 0,
            near_neutral: 0,
            divergent: 0,
            quarantined: 0,
            frozen: 0,
            exploding: 0,
            unmeasured: 0,
        }
    }
}
#[derive(Debug, Serialize)]
pub struct MutationReport {
    pub protocol: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub match_budget_policy: Option<MatchBudgetPolicy>,
    pub samples: usize,
    pub seeds: usize,
    pub trajectory_steps: usize,
    pub valid: usize,
    pub neutral: usize,
    pub near_neutral: usize,
    pub divergent: usize,
    pub quarantined: usize,
    pub catastrophic: usize,
    pub unmeasured: usize,
    pub at_cap_samples: usize,
    pub at_cap_valid: usize,
    pub neutral_soundness_checked: usize,
    pub neutral_soundness_failures: usize,
    pub quarantine_by_validator: [usize; 9],
    pub pass: bool,
    pub by_operator: BTreeMap<String, MutationBreakdown>,
    pub by_seed_shape: BTreeMap<usize, MutationBreakdown>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub semantic: Option<super::semantic::SemanticStrata>,
}
impl MutationReport {
    fn record_outcome(&mut self, op: MutationOp, seed: usize, outcome: &str) {
        for row in [
            self.by_operator.entry(format!("{op:?}")).or_default(),
            self.by_seed_shape.entry(seed / 4).or_default(),
        ] {
            row.samples += 1;
            if outcome != "quarantined" {
                row.valid += 1;
            }
            match outcome {
                "neutral" => row.neutral += 1,
                "near_neutral" => row.near_neutral += 1,
                "divergent" => row.divergent += 1,
                "quarantined" => row.quarantined += 1,
                "frozen" => row.frozen += 1,
                "exploding" => row.exploding += 1,
                _ => row.unmeasured += 1,
            }
        }
    }
}
pub fn mutation_gate(samples: usize, steps: usize) -> MutationReport {
    mutation_gate_versioned(samples, steps, EncodingVersion::LegacyV1)
}
pub fn mutation_gate_versioned(
    samples: usize,
    steps: usize,
    version: EncodingVersion,
) -> MutationReport {
    if version == EncodingVersion::AlternativeFamiliesV3 {
        return super::semantic::mutation_gate(samples, steps);
    }
    let bank = seed_bank();
    let limits = MetaLimits::default();
    let mut rng = ChaCha8Rng::seed_from_u64(MUTATION_SEED);
    let parent_frames: Vec<_> = bank.iter().map(|c| trajectory(c, steps)).collect();
    let mut r = MutationReport {
        protocol: version.protocol().into(),
        match_budget_policy: None,
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
    for _ in 0..samples {
        let i = rng.gen_range(0..bank.len());
        let parent = &bank[i];
        let at_cap = parent
            .lhs
            .iter()
            .chain(&parent.rhs)
            .any(|e| e.len() == limits.max_arity);
        r.at_cap_samples += usize::from(at_cap);
        let mut s = HypergraphStore::new();
        let region = hg_meta::encode_versioned(&mut s, parent, &limits, version).unwrap();
        let op = MUTATIONS[rng.gen_range(0..MUTATIONS.len())];
        mutate(&mut s, &region, op, &mut rng);
        let c = match hg_meta::read(&s.snapshot(), region.root, &limits) {
            Ok(c) => c,
            Err(v) => {
                r.quarantined += 1;
                r.quarantine_by_validator[v as usize] += 1;
                r.record_outcome(op, i, "quarantined");
                continue;
            }
        };
        r.valid += 1;
        r.at_cap_valid += usize::from(at_cap);
        let neutral = c.rcf_bytes() == parent.rcf_bytes();
        if neutral {
            r.neutral += 1;
            r.record_outcome(op, i, "neutral");
            if r.neutral_soundness_checked < 50 {
                r.neutral_soundness_checked += 1;
                if trajectory_on(&c, mutation_fixture(parent), steps)
                    .as_ref()
                    .ok()
                    != parent_frames[i].as_ref().ok()
                {
                    r.neutral_soundness_failures += 1;
                }
            }
            continue;
        }
        let world = mutation_fixture(parent);
        let initial_nodes = world.node_count();
        let initial_edges = world.edge_count();
        let mut mutant = world;
        let mut engine = Engine::new(
            vec![c.compile(&limits, false).unwrap()],
            StepMode::MaxParallel,
            42,
            1,
        );
        engine.set_match_budget(200_000, 20_000);
        let mut frames = Vec::new();
        let mut explosive = false;
        let mut unmeasured = false;
        for _ in 0..steps {
            let stats = engine.step(&mut mutant);
            if engine.match_ceiling_binding() {
                unmeasured = true;
                break;
            }
            frames.push((mutant.canonical_hash(), stats.accepted_rewrites));
            explosive |=
                mutant.node_count() > initial_nodes * 4 || mutant.edge_count() > initial_edges * 4;
            if mutant.node_count() > 2000 || mutant.edge_count() > 4000 {
                unmeasured = true;
                break;
            }
        }
        let child: Result<Vec<(u64, usize)>, String> = if unmeasured {
            Err("classification ceiling bound".into())
        } else {
            Ok(frames)
        };
        match (&parent_frames[i], child) {
            (Ok(parent), Ok(child)) if *parent == child => {
                r.near_neutral += 1;
                r.record_outcome(op, i, "near_neutral");
            }
            (_, Ok(child))
                if explosive
                    || child
                        .windows(50.min(steps).max(1))
                        .any(|w| w.iter().all(|(_, fires)| *fires == 0)) =>
            {
                r.catastrophic += 1;
                r.record_outcome(op, i, if explosive { "exploding" } else { "frozen" });
            }
            (Ok(_), Ok(_)) => {
                r.divergent += 1;
                r.record_outcome(op, i, "divergent");
            }
            _ => {
                r.unmeasured += 1;
                r.record_outcome(op, i, "unmeasured");
            }
        }
    }
    r.pass = samples >= GR1_SAMPLES
        && steps >= GR1_STEPS
        && r.seeds >= 20
        && r.unmeasured == 0
        && r.valid as f64 / samples as f64 >= 0.40
        && r.near_neutral as f64 / samples as f64 >= 0.10
        && r.catastrophic as f64 / samples as f64 <= 0.25
        && r.neutral_soundness_failures == 0
        && r.at_cap_samples > 0;
    r
}

#[derive(Debug, Serialize)]
pub struct EnvelopeRule {
    pub file: String,
    pub name: String,
    pub status: String,
    pub steps: usize,
}
pub fn expressivity_gate(dir: &Path, steps: usize) -> Result<Vec<EnvelopeRule>, String> {
    expressivity_gate_versioned(dir, steps, EncodingVersion::LegacyV1)
}
pub fn expressivity_gate_versioned(
    dir: &Path,
    steps: usize,
    version: EncodingVersion,
) -> Result<Vec<EnvelopeRule>, String> {
    expressivity_gate_with_policy(dir,steps,version,MatchBudgetPolicy::Split)
}
pub fn expressivity_gate_with_policy(dir: &Path, steps: usize, version: EncodingVersion, policy: MatchBudgetPolicy) -> Result<Vec<EnvelopeRule>, String> {
    gate_protocol(version,policy)?;
    let mut paths: Vec<_> = std::fs::read_dir(dir)
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "toml"))
        .collect();
    paths.sort();
    let limits = MetaLimits::default();
    let mut rows = Vec::new();
    for path in paths {
        let rules = hg_rules::load_rules_from_file(&path).map_err(|e| e.to_string())?;
        for raw in rules {
            let candidate = CandidateRule::from_rule(&raw, &limits);
            let mut row = EnvelopeRule {
                file: path.display().to_string(),
                name: raw.name.clone(),
                status: String::new(),
                steps: 0,
            };
            match candidate {
                Err(reason) => row.status = format!("outside-envelope:{reason}"),
                Ok(c) => {
                    let mut world = fixture(&c);
                    let mut encoded = world.clone();
                    let region = hg_meta::encode_versioned(&mut encoded, &c, &limits, version)
                        .map_err(|v| format!("{v:?}"))?;
                    let roundtrip = hg_meta::read_program(&encoded.snapshot(), region.root, &limits)
                        .map_err(|v| format!("{v:?}"))?;
                    let mut original = compile_rule(&raw).map_err(|e| e.to_string())?;
                    original.pattern.node_label_ceiling = Some(META_BASE);
                    let mut direct = Engine::new(vec![original], StepMode::MaxParallel, 42, 1);
                    let mut from_graph = Engine::new(
                        roundtrip.compile(&limits, false).unwrap(),
                        StepMode::MaxParallel,
                        42,
                        1,
                    );
                    // Identical initial graphs, including inert meta data.
                    world = encoded.clone();
                    let mut engaged = 0;
                    direct.set_match_budget_policy(2_000_000, 100_000,policy);
                    from_graph.set_match_budget_policy(2_000_000, 100_000,policy);
                    for _ in 0..steps {
                        let a = direct.step(&mut world);
                        let b = from_graph.step(&mut encoded);
                        if direct.match_ceiling_binding() || from_graph.match_ceiling_binding() {
                            row.status = "unmeasured:match-ceiling".into();
                            break;
                        }
                        if world.canonical_hash() != encoded.canonical_hash()
                            || a.accepted_rewrites != b.accepted_rewrites
                        {
                            row.status = "divergent".into();
                            break;
                        }
                        engaged += a.accepted_rewrites;
                        row.steps += 1;
                        if world.node_count() > 2000 || world.edge_count() > 4000 {
                            row.status = "unmeasured:bloat-ceiling".into();
                            break;
                        }
                    }
                    if row.status.is_empty() {
                        row.status = if engaged > 0 {
                            "identical"
                        } else {
                            "unmeasured:not-engaged"
                        }
                        .into();
                    }
                }
            }
            rows.push(row);
        }
    }
    Ok(rows)
}

/// G-R3 compares every full-graph hash and canonical per-step CSV bytes.
pub fn identity_gate(steps: usize) -> Result<bool, String> {
    identity_gate_versioned(steps, EncodingVersion::LegacyV1)
}
pub fn identity_gate_versioned(steps: usize, version: EncodingVersion) -> Result<bool, String> {
    identity_gate_with_policy(steps,version,MatchBudgetPolicy::Split)
}
pub fn identity_gate_with_policy(steps: usize, version: EncodingVersion, policy: MatchBudgetPolicy) -> Result<bool, String> {
    Ok(identity_gate_report(steps, version, policy)?["pass"] == true)
}

/// Detailed verification evidence; does not alter engine execution or defaults.
pub fn identity_gate_report(steps: usize, version: EncodingVersion, policy: MatchBudgetPolicy) -> Result<serde_json::Value, String> {
    let protocol = gate_protocol(version, policy)?;
    let c = &seed_bank()[0];
    let mut a = fixture(c);
    let mut b = a.clone();
    let original = compile_rule(&c.to_rule("fixture".into())).unwrap();
    let mut ea = Engine::new(vec![original.clone()], StepMode::SequentialRandom, 42, 1);
    let mut eb = Engine::new(vec![original], StepMode::SequentialRandom, 42, 1);
    let mut ra = EreRuntime::install(
        &EreConfig {
            arm: EreArm::A0,
            encoding_version: version,
            match_budget_policy: policy,
            ..EreConfig::default()
        },
        &mut a,
        &mut ea,
    )?;
    let mut rb = EreRuntime::install(
        &EreConfig {
            arm: EreArm::A1,
            encoding_version: version,
            match_budget_policy: policy,
            ..EreConfig::default()
        },
        &mut b,
        &mut eb,
    )?;
    let mut csv_a = String::from("step,hash,accepted,attempted\n");
    let mut csv_b = csv_a.clone();
    let initial_rng = ea.rng_state_probe();
    let mut completed = 0;
    let mut accepted = 0u64;
    let mut attempted = 0u64;
    let mut hashes_equal = true;
    let mut rng_equal = true;
    let mut progressed = true;
    let mut ceiling = false;
    let mut peak_work = 0;
    let mut peak_results = 0;
    for _ in 0..steps {
        let before_a = a.step();
        let before_b = b.step();
        ra.before_step(&a, &mut ea)?;
        rb.before_step(&b, &mut eb)?;
        let sa = ea.step(&mut a);
        let sb = eb.step(&mut b);
        ra.after_step(&a, &sa);
        rb.after_step(&b, &sb);
        hashes_equal &= a.canonical_hash() == b.canonical_hash();
        rng_equal &= ea.rng_state_probe() == eb.rng_state_probe();
        progressed &= a.step() == before_a + 1 && b.step() == before_b + 1;
        ceiling |= ea.match_ceiling_binding() || eb.match_ceiling_binding();
        for engine in [&ea, &eb] {
            peak_work = peak_work.max(engine.match_budget_usage().work);
            peak_results = peak_results.max(engine.match_budget_usage().results);
        }
        accepted += sa.accepted_rewrites as u64;
        attempted += sa.attempted_matches as u64;
        csv_a.push_str(&format!(
            "{},{},{},{}\n",
            a.step(),
            a.canonical_hash(),
            sa.accepted_rewrites,
            sa.attempted_matches
        ));
        csv_b.push_str(&format!(
            "{},{},{},{}\n",
            b.step(),
            b.canonical_hash(),
            sb.accepted_rewrites,
            sb.attempted_matches
        ));
        if !hashes_equal || !rng_equal || !progressed || ceiling { break; }
        completed += 1;
    }
    let csv_equal = csv_a == csv_b;
    let pass = steps > 0 && completed == steps && hashes_equal && csv_equal && rng_equal
        && progressed && !ceiling && accepted > 0 && attempted > 0;
    let mut report = serde_json::json!({"protocol":protocol,"steps":steps,"pass":pass,
        "completed_steps":completed,"a0_world_step":a.step(),"a1_world_step":b.step(),
        "graph_hashes_equal":hashes_equal,"canonical_csv_equal":csv_equal,
        "canonical_csv_bytes":csv_a.len(),"rng_states_equal":rng_equal,
        "rng_consumed":initial_rng != ea.rng_state_probe(),"every_step_advanced":progressed,
        "match_ceiling_bound":ceiling,"accepted_rewrites":accepted,"attempted_matches":attempted,
        "global_work":50_000,"global_results":50_000,"peak_step_work":peak_work,
        "peak_step_results":peak_results});
    if policy == MatchBudgetPolicy::Pooled {report["match_budget_policy"] = serde_json::to_value(policy).unwrap();}
    Ok(report)
}

pub fn cli(args: &[String]) -> Result<(), String> {
    let command = args.first().map(String::as_str).unwrap_or("help");
    let out = args.get(1).map(String::as_str).unwrap_or("output/ere");
    let dir = Path::new(out);
    // Fixed classification-only trial. Its distinct protocol cannot unlock the
    // original pooled campaign, and no production defaults are changed.
    if command == "gr1-e2p-400k50k" || command == "replay-e2p-400k50k" {
        if args.len() != if command == "gr1-e2p-400k50k" { 2 } else { 3 } {
            return Err("usage: ere gr1-e2p-400k50k OUTPUT_DIR; ere replay-e2p-400k50k OUTPUT_DIR ORIGINAL_CLASSIFICATION_AUDIT.json".into());
        }
        let filename = if command.starts_with("gr1") { "exp_r01.json" } else { "pooled_replay.json" };
        if dir.join(filename).exists() {
            return Err("trial requires a fresh receipt path".into());
        }
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        let started = std::time::Instant::now();
        let mut receipt = if command.starts_with("gr1") {
            super::semantic::mutation_gate_400k50k(GR1_SAMPLES, GR1_STEPS)
        } else {
            let original = serde_json::from_slice(&std::fs::read(&args[2]).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
            super::pooled::replay_400k50k(&original)?
        };
        receipt["elapsed_seconds"] = started.elapsed().as_secs_f64().into();
        save_json(dir, filename, &receipt)?;
        println!("{}", serde_json::to_string_pretty(&receipt).unwrap());
        let key = if command.starts_with("gr1") { "pass" } else { "correctness_pass" };
        return if receipt[key] == true { Ok(()) } else { Err("400k/50k trial did not pass; receipt preserved".into()) };
    }
    let policy = if command.ends_with("-e2p") {MatchBudgetPolicy::Pooled} else {MatchBudgetPolicy::Split};
    let version = if command.ends_with("-e2") || command.ends_with("-e2p") {
        EncodingVersion::AlternativeFamiliesV3
    } else if command.ends_with("-e11") {
        EncodingVersion::SharedPatternsV2
    } else {
        EncodingVersion::LegacyV1
    };
    let protocol = gate_protocol(version,policy)?;
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    match command {
        "replay-e2p" => {
            let path = args.get(2).ok_or("replay-e2p requires OUTPUT_DIR ORIGINAL_CLASSIFICATION_AUDIT.json")?;
            let original = serde_json::from_slice(&std::fs::read(path).map_err(|e| e.to_string())?).map_err(|e|e.to_string())?;
            let replay = super::pooled::replay(&original)?;
            save_json(dir,"pooled_replay.json",&replay)?;
            println!("{}",serde_json::to_string_pretty(&replay).unwrap());
            if replay["correctness_pass"] != true {return Err("pooled/oracle correctness check failed".into());}
        }
        "audit-e2" => {
            let audit = super::semantic::audit_classification_ceiling(GR1_SAMPLES, GR1_STEPS);
            save_json(dir, "classification_audit.json", &audit)?;
            println!("{}",serde_json::to_string_pretty(&audit).unwrap());
        }
        "fixtures" | "fixtures-e11" | "fixtures-e2" | "fixtures-e2p" => {
            let started = std::time::Instant::now();
            let mut gates = fixture_gates_with_policy(10_000, version,policy)?;
            gates["elapsed_seconds"] = started.elapsed().as_secs_f64().into();
            save_json(dir, "fixture_gates.json", &gates)?;
            println!("{}", serde_json::to_string_pretty(&gates).unwrap());
            if !gates["pass"].as_bool().unwrap_or(false) { return Err("native fixture gate failed".into()); }
        }
        "gr1" | "gr1-e11" | "gr1-e2" | "gr1-e2p" => {
            let report = if policy == MatchBudgetPolicy::Pooled {
                super::semantic::mutation_gate_with_policy(GR1_SAMPLES,GR1_STEPS,policy)
            } else {mutation_gate_versioned(GR1_SAMPLES, GR1_STEPS, version)};
            save_json(dir, "exp_r01.json", &report)?;
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
            if !report.pass { return Err("G-R1 failed; downstream campaign is not authorized by the protocol".into()); }
        }
        "gr2" | "gr2-e11" | "gr2-e2" | "gr2-e2p" => {
            let started = std::time::Instant::now();
            let rows = expressivity_gate_with_policy(Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../rules")), GR2_STEPS, version,policy)?;
            let mut header = serde_json::json!({"encoding":version, "protocol":protocol,
                "trajectory_steps":GR2_STEPS,"global_work":2_000_000,"global_results":100_000,
                "library_rules":rows.len(),"elapsed_seconds":started.elapsed().as_secs_f64()});
            if policy == MatchBudgetPolicy::Pooled {header["match_budget_policy"] = serde_json::to_value(policy).unwrap();}
            save_json(dir, "exp_r02_encoding.json", &header)?;
            save_json(dir, "exp_r02.json", &rows)?;
            println!("{}", serde_json::to_string_pretty(&rows).unwrap());
            if rows.iter().any(|r| r.status == "divergent" || r.status.starts_with("unmeasured")) { return Err("G-R2 not passed".into()); }
        }
        "gr3" | "gr3-e11" | "gr3-e2" | "gr3-e2p" => {
            let started = std::time::Instant::now();
            let mut header = identity_gate_report(GR3_STEPS, version,policy)?;
            header["elapsed_seconds"] = started.elapsed().as_secs_f64().into();
            let pass = header["pass"] == true;
            save_json(dir, "exp_r03.json", &header)?;
            println!("{}", serde_json::to_string_pretty(&header).unwrap());
            if !pass { return Err("G-R3 failed".into()); }
        }
        "arms" | "sweep" => {
            let config_path = args.get(2).ok_or("arms/sweep requires OUTPUT_DIR BASE_CONFIG.toml GATE_RECEIPTS_DIR")?;
            let receipts = Path::new(args.get(3).ok_or("missing GATE_RECEIPTS_DIR")?);
            let config = crate::SimConfig::from_toml(&std::fs::read_to_string(config_path).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
            let ere = config.ere.clone().unwrap_or_default();
            require_research_gates_with_policy(receipts, ere.encoding_version,ere.match_budget_policy)?;
            run_arm_experiments(&config, dir, command == "sweep")?;
        }
        _ => return Err("usage: ere gr1|gr2|gr3|fixtures OUTPUT_DIR (append -e11, -e2 or -e2p); ere audit-e2 DIAGNOSTIC_DIR; ere replay-e2p OUTPUT_DIR ORIGINAL_CLASSIFICATION_AUDIT.json; ere arms|sweep OUTPUT_DIR BASE_CONFIG.toml GATE_RECEIPTS_DIR".into()),
    }
    Ok(())
}

pub fn require_research_gates(dir: &Path) -> Result<(), String> {
    require_research_gates_versioned(dir, EncodingVersion::LegacyV1)
}
pub fn require_research_gates_versioned(
    dir: &Path,
    version: EncodingVersion,
) -> Result<(), String> {
    require_research_gates_with_policy(dir,version,MatchBudgetPolicy::Split)
}
pub fn require_research_gates_with_policy(dir: &Path, version: EncodingVersion, policy: MatchBudgetPolicy) -> Result<(),String> {
    let protocol = gate_protocol(version,policy)?;
    let load = |name| -> Result<serde_json::Value, String> {
        serde_json::from_slice(
            &std::fs::read(dir.join(name)).map_err(|e| format!("missing {name}: {e}"))?,
        )
        .map_err(|e| e.to_string())
    };
    let gr1 = load("exp_r01.json")?;
    let count = |key: &str| gr1[key].as_u64().unwrap_or(0);
    let samples = count("samples");
    let valid = count("valid");
    let classified = count("neutral")
        + count("near_neutral")
        + count("divergent")
        + count("catastrophic")
        + count("unmeasured");
    if gr1["pass"] != true
        || samples < GR1_SAMPLES as u64
        || gr1["trajectory_steps"].as_u64().unwrap_or(0) < GR1_STEPS as u64
        || gr1["protocol"] != protocol
        || (policy == MatchBudgetPolicy::Pooled && gr1["match_budget_policy"] != "Pooled")
        || (policy == MatchBudgetPolicy::Split && gr1.get("match_budget_policy").is_some_and(|p|p != "Split"))
        || count("seeds") < 20
        || valid + count("quarantined") != samples
        || classified != valid
        || count("unmeasured") != 0
        || count("neutral_soundness_checked") < 50
        || gr1["neutral_soundness_failures"].as_u64() != Some(0)
        || count("at_cap_samples") == 0
        || valid as f64 / (samples as f64) < 0.40
        || count("near_neutral") as f64 / (samples as f64) < 0.10
        || count("catastrophic") as f64 / (samples as f64) > 0.25
    {
        return Err(format!(
            "full {} G-R1 receipt has not passed",
            protocol
        ));
    }
    if version == EncodingVersion::AlternativeFamiliesV3 {
        super::semantic::validate_receipt(&gr1)?;
    }
    let gr2 = load("exp_r02.json")?;
    if version != EncodingVersion::LegacyV1 || dir.join("exp_r02_encoding.json").exists() {
        let header = load("exp_r02_encoding.json")?;
        if header["protocol"] != protocol || (policy == MatchBudgetPolicy::Pooled && header["match_budget_policy"] != "Pooled") {
            return Err("G-R2 receipt belongs to a different encoding".into());
        }
    }
    if gr2.as_array().is_none_or(|rows| {
        rows.is_empty()
            || !rows.iter().any(|r| r["status"] == "identical")
            || rows.iter().any(|r| {
                r["status"]
                    .as_str()
                    .is_none_or(|s| s != "identical" && !s.starts_with("outside-envelope:"))
                    || r["status"] == "identical"
                        && r["steps"].as_u64().unwrap_or(0) < GR2_STEPS as u64
            })
    }) {
        return Err("full native E1 G-R2 receipt has not passed".into());
    }
    let gr3 = load("exp_r03.json")?;
    if (version != EncodingVersion::LegacyV1 || gr3.get("protocol").is_some())
        && (gr3["protocol"] != protocol || (policy == MatchBudgetPolicy::Pooled && gr3["match_budget_policy"] != "Pooled"))
    {
        return Err("G-R3 receipt belongs to a different encoding".into());
    }
    if gr3["pass"] != true || gr3["steps"].as_u64().unwrap_or(0) < GR3_STEPS as u64 {
        return Err("full G-R3 receipt has not passed".into());
    }
    let fixtures = load("fixture_gates.json")?;
    if (version != EncodingVersion::LegacyV1 || fixtures.get("protocol").is_some())
        && (fixtures["protocol"] != protocol || (policy == MatchBudgetPolicy::Pooled && fixtures["match_budget_policy"] != "Pooled"))
    {
        return Err("fixture receipt belongs to a different encoding".into());
    }
    if fixtures["pass"] != true
        || fixtures["protection_steps"].as_u64().unwrap_or(0) < 10_000
        || ["gr0", "gr4", "gr5", "gr6"]
            .iter()
            .any(|key| fixtures[key] != true)
        || fixtures["meta_editor_events"].as_u64().unwrap_or(0) == 0
    {
        return Err("G-R0/G-R4/G-R5/G-R6 fixtures have not passed".into());
    }
    Ok(())
}

/// Explicit seeded editor. E1 cannot emit edge labels or pay for property
/// edits, so this TOML motif is declared scaffolding, not endogenous novelty.
pub fn seeded_editor() -> String {
    format!(
        r#"
[[rules]]
name="ERE_seeded_priority_editor"
priority=100
[rules.lhs]
nodes=["root","left","right"]
edges=[["root","left","right"]]
[rules.lhs.node_labels]
root={root}
left={left}
right={right}
[rules.lhs.edge_labels]
0={becomes}
[[rules.lhs.node_property_guards]]
var="root"
property=1
op="gte"
value=1.0
[rules.rhs]
nodes=["root","left","right"]
edges=[["root","left","right"]]
[rules.rhs.edge_labels]
0={becomes}
[[rules.rhs.property_effects]]
var="root"
key=1
op="subtract"
value=1.0
[[rules.rhs.property_effects]]
var="root"
key=2
op="add"
value=1
"#,
        root = hg_meta::L_RULE_ROOT,
        left = hg_meta::L_SIDE_L,
        right = hg_meta::L_SIDE_R,
        becomes = hg_meta::E_BECOMES
    )
}

/// E2 scaffold pays family energy to activate and change proposal priority.
/// Its own events/classes are excluded from endogenous novelty and reuse.
pub fn seeded_editor_versioned(version: EncodingVersion) -> String {
    if version != EncodingVersion::AlternativeFamiliesV3 {
        return seeded_editor();
    }
    format!(r#"
[[rules]]
name="ERE_seeded_proposal_editor"
priority=100
[rules.lhs]
nodes=["root","proposal"]
edges=[["root","proposal"]]
[rules.lhs.node_labels]
root={root}
proposal={proposal}
[rules.lhs.edge_labels]
0={graft}
[[rules.lhs.node_property_guards]]
var="root"
property=1
op="gte"
value=1.0
[rules.rhs]
nodes=["root","proposal"]
edges=[["root","proposal"]]
[rules.rhs.edge_labels]
0={graft}
[[rules.rhs.property_effects]]
var="root"
key=1
op="subtract"
value=1.0
[[rules.rhs.property_effects]]
var="root"
key={mode}
op="set"
value=1
[[rules.rhs.property_effects]]
var="proposal"
key=2
op="add"
value=1
"#, root=hg_meta::L_RULE_ROOT, proposal=hg_meta::L_ALLELE_ROOT,
    graft=hg_meta::E_GRAFT, mode=hg_meta::K_GRAFT_MODE)
}

pub fn fixture_gates(steps: usize) -> Result<serde_json::Value, String> {
    fixture_gates_versioned(steps, EncodingVersion::LegacyV1)
}
pub fn fixture_gates_versioned(
    steps: usize,
    version: EncodingVersion,
) -> Result<serde_json::Value, String> {
    fixture_gates_with_policy(steps,version,MatchBudgetPolicy::Split)
}
pub fn fixture_gates_with_policy(steps: usize, version: EncodingVersion, policy: MatchBudgetPolicy) -> Result<serde_json::Value,String> {
    let protocol = gate_protocol(version,policy)?;
    use hg_meta::*;
    use hg_rules::{RuleSetDelta, RuleSource};
    let bank = seed_bank();
    let limits = MetaLimits::default();
    let mut rcf_pass = true;
    for c in bank.iter().take(20) {
        let mut renamed = c.clone();
        let n = c.labels.len();
        renamed.labels.reverse();
        for e in renamed.lhs.iter_mut().chain(&mut renamed.rhs) {
            for v in e {
                *v = n - 1 - *v;
            }
        }
        for g in &mut renamed.guards {
            g.var = n - 1 - g.var;
        }
        renamed.lhs.reverse();
        renamed.rhs.reverse();
        rcf_pass &= c.rcf_bytes() == renamed.rcf_bytes();
        renamed.priority += 1;
        rcf_pass &= c.rcf_bytes() != renamed.rcf_bytes();
    }
    let mut counts = [0; 9];
    for v in 0..9 {
        let mut s = HypergraphStore::new();
        let region = encode_versioned(&mut s, &bank[0], &limits, version).unwrap();
        let first = |label| {
            region
                .nodes
                .iter()
                .copied()
                .find(|&n| s.get_node(n).unwrap().label.0 == label)
                .unwrap()
        };
        let side = first(L_SIDE_L);
        let pat = first(L_PAT_EDGE);
        let var = first(L_VAR);
        let mut cap = limits.clone();
        match v {
            0 => cap.max_region_nodes = 2,
            1 => s.remove_edge(region.edges[0]),
            2 => {
                s.add_edge(HyperEdge::new(
                    Label(E_MEMBER),
                    [side, pat].into_iter().collect(),
                ));
            }
            3 => {
                s.add_edge(HyperEdge::new(
                    Label(E_SLOT_BASE),
                    [pat, var].into_iter().collect(),
                ));
            }
            4 => s
                .get_node_mut(var)
                .unwrap()
                .set_property(K_ORDER, PropertyValue::Int(99)),
            5 | 6 => {
                let label = if v == 5 { L_LABEL_LIT } else { L_GUARD };
                let relation = if v == 5 { E_HASLABEL } else { E_GUARDS };
                let n = s.add_node(NodeData::new(Label(label), 0));
                s.add_edge(HyperEdge::new(
                    Label(relation),
                    [var, n].into_iter().collect(),
                ));
            }
            7 => cap.max_pattern_edges = 0,
            8 => s
                .get_node_mut(region.root)
                .unwrap()
                .set_property(if version == EncodingVersion::AlternativeFamiliesV3 {K_GRAFT_MODE} else {K_PRIORITY}, PropertyValue::Bool(true)),
            _ => unreachable!(),
        }
        let mut source = GraphRuleSource::new(EpochPolicy::Never, cap, false);
        if let RuleSetDelta::Replace { report, .. } = source.rules(&s.snapshot(), 0) {
            for (a, b) in counts.iter_mut().zip(report.quarantine_by_validator) {
                *a += b;
            }
        }
    }
    let mut world = fixture(&bank[0]);
    let mut engine = Engine::new(
        vec![compile_rule(&bank[0].to_rule("fixture".into())).unwrap()],
        StepMode::MaxParallel,
        42,
        1,
    );
    let config = EreConfig {
        encoding_version: version,
        match_budget_policy: policy,
        arm: EreArm::A2,
        epoch: EpochPolicy::Fixed(10.try_into().unwrap()),
        meta_rules_toml: Some(seeded_editor_versioned(version)),
        ..EreConfig::default()
    };
    let mut runtime = EreRuntime::install(&config, &mut world, &mut engine)?;
    let mut protected = true;
    let mut protection_completed_steps = 0;
    let mut protection_accepted_rewrites = 0u64;
    let mut protection_peak_work = 0;
    let mut protection_peak_results = 0;
    for _ in 0..steps {
        let before = world.step();
        runtime.before_step(&world, &mut engine)?;
        let stats = engine.step(&mut world);
        runtime.after_step(&world, &stats);
        protected &= runtime.protection_intact(&world) && !engine.match_ceiling_binding()
            && world.step() == before + 1;
        protection_completed_steps += usize::from(world.step() == before + 1);
        protection_accepted_rewrites += stats.accepted_rewrites as u64;
        protection_peak_work = protection_peak_work.max(engine.match_budget_usage().work);
        protection_peak_results = protection_peak_results.max(engine.match_budget_usage().results);
    }
    // Positive G-R6 control: a changed priority inside a window must suppress
    // both TPM variants; stability cannot pass merely because nothing changed.
    let mut changed = fixture(&bank[0]);
    let mut e = Engine::new(
        vec![bank[0].compile(&limits, true).unwrap()],
        StepMode::MaxParallel,
        42,
        1,
    );
    let mut r = EreRuntime::install(
        &EreConfig {
            encoding_version: version,
            match_budget_policy: policy,
            arm: EreArm::A3,
            epoch: EpochPolicy::Fixed(1.try_into().unwrap()),
            ..EreConfig::default()
        },
        &mut changed,
        &mut e,
    )?;
    r.before_step(&changed, &mut e)?;
    let stats = e.step(&mut changed);
    r.after_step(&changed, &stats);
    let root = changed
        .nodes()
        .find(|(_, n)| n.label.0 == L_RULE_ROOT)
        .unwrap()
        .0;
    if version == EncodingVersion::AlternativeFamiliesV3 {
        let parts = family_parts(&changed.snapshot(), root, &limits).map_err(|v| format!("{v:?}"))?;
        changed.get_node_mut(parts.proposal.root).unwrap().set_property(K_PRIORITY, PropertyValue::Int(17));
        changed.get_node_mut(root).unwrap().set_property(K_GRAFT_MODE, PropertyValue::Int(1));
    } else {
        changed.get_node_mut(root).unwrap().set_property(K_PRIORITY, PropertyValue::Int(17));
    }
    r.before_step(&changed, &mut e)?;
    let stats = e.step(&mut changed);
    r.after_step(&changed, &stats);
    let row = r.close_window(&changed, MetricsRow::default(), MetricsRow::default());
    let stationarity = row.tpm_rule_set_stable == Some(false)
        && row.meta_excluded.as_ref().unwrap().ei_micro.is_none()
        && row.meta_included.as_ref().unwrap().ei_micro.is_none();
    let engaged = runtime.meta_rewrite_count() > 0;
    let mut result = serde_json::json!({"protocol":protocol,"rcf_pairs":40,"gr0":rcf_pass,"quarantine_by_validator":counts,
        "gr4":counts == [1;9],"gr5":protected,"protection_steps":steps,"gr6":stationarity,
        "protection_completed_steps":protection_completed_steps,"protection_world_step":world.step(),
        "protection_accepted_rewrites":protection_accepted_rewrites,
        "global_work":50_000,"global_results":50_000,
        "peak_step_work":protection_peak_work,"peak_step_results":protection_peak_results,
        "changed_rule_set_stable":row.tpm_rule_set_stable,
        "changed_meta_included_ei":row.meta_included.as_ref().unwrap().ei_micro,
        "changed_meta_excluded_ei":row.meta_excluded.as_ref().unwrap().ei_micro,
        "meta_editor_events":runtime.meta_rewrite_count(),
        "pass":rcf_pass && counts == [1;9] && protected && stationarity && engaged});
    if policy == MatchBudgetPolicy::Pooled {result["match_budget_policy"] = serde_json::to_value(policy).unwrap();}
    Ok(result)
}

pub fn canonical_seed_signature(toml: &str) -> Result<String, String> {
    let raw = hg_rules::parse_rules_toml(toml).map_err(|e| e.to_string())?;
    let mut classes: Vec<_> = raw
        .iter()
        .map(|r| CandidateRule::from_rule(r, &MetaLimits::default()).map(|c| c.rcf_bytes()))
        .collect::<Result<_, _>>()?;
    classes.sort();
    Ok(classes
        .into_iter()
        .map(|b| b.iter().map(|x| format!("{x:02x}")).collect::<String>())
        .collect::<Vec<_>>()
        .join(":"))
}

pub fn run_arm_experiments(base: &crate::SimConfig, dir: &Path, sweep: bool) -> Result<(), String> {
    if base.ere.as_ref().is_some_and(|e|e.vocabulary.is_some()) {
        return Err("legacy arm/epoch sweep cannot reuse vocabulary semantics".into());
    }
    let epochs = if sweep {
        EPOCH_SWEEP.to_vec()
    } else {
        vec![Some(1000)]
    };
    save_json(
        dir,
        "manifest.json",
        &serde_json::json!({"base_config":base,"epochs":epochs,
        "arms":["A0","A1","A2","A3"], "ci_reporting_width_bits":0.25,
        "seeded_editors":"energy-paid version-specific priority/activation editor; static scaffold, excluded from novelty"}),
    )?;
    let mut summaries = Vec::new();
    for epoch in epochs {
        for arm in [EreArm::A0, EreArm::A1, EreArm::A2, EreArm::A3] {
            if sweep && matches!(arm, EreArm::A0 | EreArm::A1) && epoch != None {
                continue;
            }
            let mut config = base.clone();
            let mut ere = config.ere.clone().unwrap_or_default();
            ere.arm = arm;
            ere.epoch = epoch
                .map(|n| EpochPolicy::Fixed(n.try_into().unwrap()))
                .unwrap_or(EpochPolicy::Never);
            if ere.meta_rules_toml.is_none() {
                ere.meta_rules_toml = Some(seeded_editor_versioned(ere.encoding_version));
            }
            config.ere = Some(ere);
            config.causal.enabled = false;
            config.out_dir = dir.into();
            config.run_id = format!(
                "{arm:?}_E{}",
                epoch
                    .map(|x| x.to_string())
                    .unwrap_or_else(|| "infinity".into())
            );
            match crate::run_headless(&config, None, None, None) {
                Ok(run) => {
                    let rows: Vec<_> = run.history.iter().filter_map(|r| r.ere.as_deref()).collect();
                    let stable = rows.iter().filter(|r| r.tpm_rule_set_stable == Some(true)).count() as f64 / rows.len().max(1) as f64;
                    let widths: Vec<_> = rows.iter().filter_map(|r| r.ei_micro_ci_width_meta_excluded).collect();
                    summaries.push(serde_json::json!({"arm":format!("{arm:?}"),"epoch":epoch,"status":"completed",
                        "final_step":run.final_step,"stationary_fraction":stable,"ei_ci_width_excluded": widths,
                        "last_window":rows.last()}));
                }
                Err(error) => summaries.push(serde_json::json!({"arm":format!("{arm:?}"),"epoch":epoch,"status":"halted","reason":error.to_string()})),
            }
        }
    }
    save_json(
        dir,
        if sweep {
            "exp_r08.json"
        } else {
            "exp_r04_r05_r06_r07_r10.json"
        },
        &summaries,
    )?;
    let report = format!("# ERE {}\n\nAll controls use the same initial substrate and inert encoded seed regions. Headline comparisons are A3 minus A1 and A3 minus A2.\n\nOnly stationary epochs support EI/CE; moving-block CIs use 16-step blocks and 64 replicates. A missing value is unmeasured, never zero. Seeded priority edits are not evidence of evolved computation.\n\n```json\n{}\n```\n", if sweep { "epoch sweep" } else { "four-arm experiments" }, serde_json::to_string_pretty(&summaries).unwrap());
    std::fs::write(dir.join("report.md"), report).map_err(|e| e.to_string())?;
    Ok(())
}
fn save_json<T: Serialize>(dir: &Path, name: &str, data: &T) -> Result<(), String> {
    let path = dir.join(name);
    if path.exists() {
        return Err(format!(
            "refusing to overwrite experiment artifact {}",
            path.display()
        ));
    }
    std::fs::write(
        path,
        serde_json::to_vec_pretty(data).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}
