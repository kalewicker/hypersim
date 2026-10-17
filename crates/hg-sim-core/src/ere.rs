//! ERE orchestration and instruments. Uses Engine::step; no alternate physics.
use hg_core::{compute_wl_class_map, HypergraphStore, NodeId};
use hg_engine::Engine;
use hg_meta::{CandidateRule, EncodingVersion, GraphRuleSource, MetaLimits, META_BASE};
pub use hg_meta::{MetaInstructionSeed, MetaOperation, VOCABULARY_PROTOCOL, REPAIR_PROTOCOL, SEALED_REPAIR_PROTOCOL, COMPLETE_PROTOCOL};
use hg_rules::{CompiledRule, EpochPolicy, MatchBudgetPolicy};
use hg_schema::{EreMetricsRow, MetricsRow, RuleReplicatorKind};
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};

fn is_false(value: &bool) -> bool { !value }

pub mod experiments;
pub mod semantic;
pub mod pooled;
pub mod admission;
pub mod checkpoint;
pub mod coherent;
/// Scripted rule birth/lifetime controls; no optimizer or scientific admission.
pub mod lifecycle_fixture;
/// Versioned native seeded paid-choice interface; no training or admission.
pub mod rule_choice;
pub mod rule_composition;
pub mod rule_relational;
pub mod rule_maintenance;
pub mod rule_prevention;
pub mod rule_prevention_transfer;
pub mod defense_allocation;
pub mod resource_timed_bridge;
#[cfg(feature = "repeated-edge-feasibility")]
pub mod signal;
#[cfg(feature = "repeated-edge-feasibility")]
pub mod valid_grammar;

pub const POOLED_PROTOCOL: &str = "E2-alternatives-pooled-v1";
pub fn gate_protocol(version: EncodingVersion, policy: MatchBudgetPolicy) -> Result<&'static str,String> {
    match policy {
        MatchBudgetPolicy::Split => Ok(version.protocol()),
        MatchBudgetPolicy::Pooled if version == EncodingVersion::AlternativeFamiliesV3 => Ok(POOLED_PROTOCOL),
        _ => Err("pooled ERE receipts require the explicit E2 encoding".into()),
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum EreArm {
    A0,
    #[default]
    A1,
    A2,
    A3,
}

/// Explicit outer-grammar seed material. Each TOML must contain exactly one
/// strict supported rule; the complete family is validated before store writes.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EreSeedFamily {
    pub core_toml: String,
    pub proposal_toml: String,
    pub active: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetaVocabularyConfig {
    pub protocol: String,
    pub instructions: Vec<hg_meta::MetaInstructionSeed>,
}

/// Validates the whole emitted genotype; executable signatures ignore dormant
/// material, while graph-distribution signatures explicitly retain it.
pub fn seed_family_signature(seeds: &[EreSeedFamily], include_dormant: bool) -> Result<String, String> {
    if seeds.is_empty() {return Err("empty family catalogue".into());}
    let limits = MetaLimits::default();
    let mut parts = Vec::new();
    let read = |text: &str| -> Result<CandidateRule, String> {
        let raw = hg_rules::parse_rules_toml(text).map_err(|e|e.to_string())?;
        if raw.len() != 1 {return Err("a family allele requires exactly one rule".into());}
        CandidateRule::from_rule(&raw[0], &limits)
    };
    let mut store = HypergraphStore::new();
    for seed in seeds {
        let core = read(&seed.core_toml)?; let proposal = read(&seed.proposal_toml)?;
        let r = hg_meta::encode_family(&mut store, &core, &proposal, seed.active, &limits)
            .map_err(|v|format!("invalid whole family: {v:?}"))?;
        let p = hg_meta::read_program(&store.snapshot(), r.root, &limits).map_err(|v|format!("{v:?}"))?;
        let hex = |b:Vec<u8>|b.iter().map(|v|format!("{v:02x}")).collect::<String>();
        parts.push(if include_dormant {format!("{}:{}:{}",hex(core.execution_bytes()),
            hex(proposal.execution_bytes()),seed.active)} else {hex(p.rcf_bytes())});
    }
    parts.sort();
    Ok(parts.join("/"))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct EreConfig {
    #[serde(skip_serializing_if = "EncodingVersion::is_legacy")]
    pub encoding_version: EncodingVersion,
    pub arm: EreArm,
    pub epoch: EpochPolicy,
    pub limits: MetaLimits,
    pub rule_region_ceiling: usize,
    pub match_ceiling: usize,
    #[serde(skip_serializing_if = "MatchBudgetPolicy::is_split")]
    pub match_budget_policy: MatchBudgetPolicy,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed_families: Option<Vec<EreSeedFamily>>,
    #[serde(skip_serializing_if = "is_false")]
    pub measure_family_maintenance: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vocabulary: Option<MetaVocabularyConfig>,
    pub meta_rules_toml: Option<String>,
    pub min_tpm_transitions: u64,
    pub meta_seed_energy: f64,
    pub measure_ei: bool,
}
impl Default for EreConfig {
    fn default() -> Self {
        Self {
            encoding_version: EncodingVersion::LegacyV1,
            arm: EreArm::A1,
            epoch: EpochPolicy::Never,
            limits: MetaLimits::default(),
            rule_region_ceiling: 500,
            match_ceiling: 50_000,
            match_budget_policy: MatchBudgetPolicy::Split,
            seed_families: None,
            measure_family_maintenance: false,
            vocabulary: None,
            meta_rules_toml: None,
            min_tpm_transitions: 1000,
            meta_seed_energy: 32.0,
            measure_ei: true,
        }
    }
}
impl EreConfig {
    pub fn protocol(&self)->Result<&'static str,String> {
        let legacy=gate_protocol(self.encoding_version,self.match_budget_policy)?;
        match &self.vocabulary {
            None=>Ok(legacy),
            Some(v) if v.protocol==hg_meta::VOCABULARY_PROTOCOL=>Ok(hg_meta::VOCABULARY_PROTOCOL),
            Some(v) if v.protocol==hg_meta::REPAIR_PROTOCOL=>Ok(hg_meta::REPAIR_PROTOCOL),
            Some(v) if v.protocol==hg_meta::SEALED_REPAIR_PROTOCOL=>Ok(hg_meta::SEALED_REPAIR_PROTOCOL),
            Some(v) if v.protocol==hg_meta::COMPLETE_PROTOCOL=>Ok(hg_meta::COMPLETE_PROTOCOL),
            Some(_)=>Err("unknown meta vocabulary protocol".into()),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct EreRuntime {
    #[serde(with = "checkpoint::config_wire")]
    config: EreConfig,
    anchors: Vec<NodeId>,
    baseline: HashSet<Vec<u8>>,
    seen: HashSet<Vec<u8>>,
    classes: Vec<u64>,
    reuse: BTreeMap<u64, usize>,
    nontrivial: HashSet<u64>,
    world_reuse: BTreeMap<u64, usize>,
    trivial_copy_seen: bool,
    before: Option<HypergraphStore>,
    #[serde(skip)]
    events: std::sync::Arc<std::sync::Mutex<Vec<hg_observe::RewriteEvent>>>,
    installed: usize,
    retired: usize,
    quarantined: usize,
    quarantine_decisions: usize,
    quarantine_by: [usize; 9],
    meta_events: usize,
    cleanup_lost: usize,
    sites: BTreeMap<u64, usize>,
    half_life: Option<f64>,
    window_start: u64,
    window_revision: Option<u64>,
    stable: bool,
    last_compile: Option<u64>,
    epoch_effective: Option<u64>,
    cap_binding: bool,
    match_binding: bool,
    previous_novel: usize,
    included_tpm: hg_causal::TpmAccumulator,
    excluded_tpm: hg_causal::TpmAccumulator,
    included_windows: hg_eval::WindowComputer,
    excluded_windows: hg_eval::WindowComputer,
    included_ticks: Vec<Vec<(hg_core::WlHash, hg_core::WlHash)>>,
    excluded_ticks: Vec<Vec<(hg_core::WlHash, hg_core::WlHash)>>,
    dynamic_changes: usize,
    accepted: usize,
    world_accepted: usize,
    attempted: usize,
    included_wl_seen: HashSet<hg_core::WlHash>,
    excluded_wl_seen: HashSet<hg_core::WlHash>,
    proposal_classes: HashSet<u64>,
    core_classes: HashSet<u64>,
    structural_proposal_classes: BTreeMap<u64,u64>,
    proposal_reuse: BTreeMap<u64, usize>,
    structural_proposal_reuse: BTreeMap<u64, usize>,
    ambiguous_role_reuse: BTreeMap<u64, usize>,
    family_modes: BTreeMap<NodeId, bool>,
    activation_count: usize,
    maintenance: Vec<hg_schema::ere::EreFamilyMaintenance>,
    editor_rule_count: usize,
    editor_events: usize,
    ordinary_meta_events: usize,
    encoded_operations: Vec<String>,
    instruction_roots: Vec<NodeId>,
    instruction_states: Vec<hg_schema::ere::EreInstructionState>,
    encoded_meta_events: usize,
    operation_traces: Vec<hg_schema::ere::EreMetaOperationTrace>,
}
struct EventSink(std::sync::Arc<std::sync::Mutex<Vec<hg_observe::RewriteEvent>>>);
impl hg_observe::Observer for EventSink {
    fn on_step_end(
        &mut self,
        _: &hg_core::GraphSnapshot<'_>,
        _: &hg_observe::StepStats,
        events: &[hg_observe::RewriteEvent],
    ) {
        self.0.lock().unwrap().extend(events.iter().cloned());
    }
}

impl EreRuntime {
    /// Read-only native window TPMs, before close_window resets them.
    /// Raw counts, not normalized estimates; no graph mutation or RNG use.
    pub fn diagnostic_tpms(&self) -> serde_json::Value {
        let dump = |t: &hg_causal::TpmAccumulator| serde_json::json!({
            "hashes": t.hashes().iter().map(|h|format!("{:016x}",h.0)).collect::<Vec<_>>(),
            "counts": t.diagnostic_counts(), "total_transitions": t.total_transitions()
        });
        serde_json::json!({"meta_included":dump(&self.included_tpm),"meta_excluded":dump(&self.excluded_tpm)})
    }
    /// A0 gets the identical inert seed region as A1. Otherwise full-graph
    /// hashes cannot possibly be identical (one initial graph has extra nodes).
    pub fn install(
        config: &EreConfig,
        store: &mut HypergraphStore,
        engine: &mut Engine,
    ) -> Result<Self, String> {
        config.protocol()?;
        let complete=config.vocabulary.as_ref().is_some_and(|v|v.protocol==hg_meta::COMPLETE_PROTOCOL);
        let compile_operations=if complete {hg_meta::compile_complete_operations}else{hg_meta::compile_operations};
        let encode_instruction=if complete {hg_meta::encode_complete_instruction}else{hg_meta::encode_instruction};
        let read_instruction=if complete {hg_meta::read_complete_instruction}else{hg_meta::read_instruction};
        let sealed_repair=complete || config.vocabulary.as_ref().is_some_and(|v|v.protocol==hg_meta::SEALED_REPAIR_PROTOCOL);
        let witnessed=sealed_repair || config.vocabulary.as_ref().is_some_and(|v|v.protocol==hg_meta::REPAIR_PROTOCOL);
        if let Some(v) = &config.vocabulary {
            if v.instructions.is_empty() || v.instructions.len()>if witnessed {8}else{4}
                || config.encoding_version!=EncodingVersion::AlternativeFamiliesV3
                || config.match_budget_policy!=MatchBudgetPolicy::Pooled || config.seed_families.is_none()
                || !config.measure_family_maintenance || config.meta_rules_toml.is_some() || config.meta_seed_energy!=0.0 {
                return Err("vocabulary requires its explicit protocol, 1..=4 instructions, E2 pooled families, maintenance, zero family fuel and no static editor".into());
            }
            for seed in &v.instructions {
                if !seed.operation.admitted(witnessed, sealed_repair) {return Err("instruction belongs to a different semantic protocol".into());}
                seed.validate().map_err(|e|format!("invalid instruction seed: {e:?}"))?;
                compile_operations(seed.operation,&config.limits).map_err(|e|format!("vocabulary outside limits: {e:?}"))?;}
            if v.instructions.iter().map(|s|s.fuel).sum::<f64>()>128.0 {return Err("vocabulary fuel budget exceeds 128".into());}
        }
        if config.measure_family_maintenance && config.encoding_version != EncodingVersion::AlternativeFamiliesV3 {
            return Err("whole-family maintenance measurement requires E2".into());
        }
        if config.rule_region_ceiling == 0 || config.match_ceiling == 0 {
            return Err("ERE ceilings must be positive".into());
        }
        if store.nodes().any(|(_, n)| n.label.0 >= META_BASE) {
            return Err("fresh ERE installation requires no existing meta band".into());
        }
        let mut candidates = Vec::new();
        for r in engine.rules() {
            candidates.push(CandidateRule::from_compiled(r, &config.limits).map_err(|reason|
                format!("rule '{}' is outside E1: {reason}; characterize/filter the envelope explicitly", r.name))?);
        }
        candidates.sort_by_key(|c| (c.rcf_bytes(), c.execution_bytes()));
        let mut programs = if let Some(seeds) = &config.seed_families {
            if config.encoding_version != EncodingVersion::AlternativeFamiliesV3 || seeds.is_empty() {
                return Err("explicit seed families require E2 and a nonempty catalogue".into());
            }
            let read = |text: &str| -> Result<CandidateRule, String> {
                let raw = hg_rules::parse_rules_toml(text).map_err(|e| e.to_string())?;
                if raw.len() != 1 { return Err("a seed allele must contain exactly one rule".into()); }
                CandidateRule::from_rule(&raw[0], &config.limits)
            };
            let ps = seeds.iter().map(|s| Ok(hg_meta::RuleProgram {
                core: read(&s.core_toml)?, proposal: Some(read(&s.proposal_toml)?), active: s.active,
            })).collect::<Result<Vec<_>, String>>()?;
            let mut supplied: Vec<_> = ps.iter().map(|p|p.core.execution_bytes()).collect();
            supplied.sort();
            let mut expected: Vec<_> = candidates.iter().map(CandidateRule::execution_bytes).collect();
            expected.sort();
            if supplied != expected { return Err("seed family cores do not match the initial rule table".into()); }
            // Validate aggregate bounds and every dormant body on a temporary store.
            let mut test = HypergraphStore::new();
            for p in &ps {
                let encode=if complete {hg_meta::encode_complete_family}else if witnessed {hg_meta::encode_witness_family}else{hg_meta::encode_family};
                encode(&mut test, &p.core, p.proposal.as_ref().unwrap(), p.active,
                    &config.limits).map_err(|v|format!("invalid seed family: {v:?}"))?;
            }
            if test.node_count() + config.vocabulary.as_ref().map_or(0,|v|v.instructions.len()) > config.rule_region_ceiling {
                return Err("explicit seed families exceed aggregate rule-region ceiling".into());
            }
            ps
        } else {
            candidates.iter().map(|c| hg_meta::RuleProgram {core:c.clone(),
                proposal:(config.encoding_version == EncodingVersion::AlternativeFamiliesV3).then(||c.clone()),
                active:false}).collect()
        };
        programs.sort_by_key(|p|(p.core.rcf_bytes(), p.core.execution_bytes(),
            p.proposal.as_ref().map(CandidateRule::execution_bytes), p.active));
        let mut anchors = Vec::new();
        for p in &programs {
            let region = if let Some(proposal) = &p.proposal {
                let encode=if complete {hg_meta::encode_complete_family}else if witnessed {hg_meta::encode_witness_family}else{hg_meta::encode_family};
                encode(store, &p.core, proposal, p.active, &config.limits)
            } else { hg_meta::encode_versioned(store, &p.core, &config.limits, config.encoding_version) }
                .map_err(|v|format!("{v:?}"))?;
            anchors.push(region.root);
        }
        if !config.meta_seed_energy.is_finite() || config.meta_seed_energy < 0.0 {
            return Err("seed energy must be finite and nonnegative".into());
        }
        for &root in &anchors {
            store
                .get_node_mut(root)
                .unwrap()
                .set_property(1, hg_core::PropertyValue::Float(config.meta_seed_energy));
        }
        let mut instruction_roots=Vec::new();
        let mut encoded_operations=Vec::new();
        let mut static_operations=Vec::new();
        if let Some(v)=&config.vocabulary {
            for seed in &v.instructions {instruction_roots.push(encode_instruction(store,seed).unwrap());}
            let ops:std::collections::BTreeSet<_>=v.instructions.iter().map(|s|s.operation).collect();
            for op in ops {for rule in compile_operations(op,&config.limits).unwrap() {
                encoded_operations.push(op.name().into());static_operations.push(rule);}}
        }
        let instruction_states=instruction_roots.iter().map(|root|{
            let s=read_instruction(&store.snapshot(),*root).unwrap();
            hg_schema::ere::EreInstructionState {instruction:format!("{root:?}"),checks:0,valid_steps:0,
                first_invalid_step:None,valid:true,operation:Some(s.operation.name().into()),fuel:Some(s.fuel)}
        }).collect();
        let mut executable: Vec<_> = programs.iter().flat_map(|p|p.candidates()).collect();
        executable.sort_by_key(|c|(c.rcf_bytes(), c.execution_bytes()));
        let baseline: HashSet<_> = executable.iter().map(CandidateRule::rcf_bytes).collect();
        let classes = executable.iter().map(CandidateRule::rcf_hash).collect();
        let epoch = if matches!(config.arm, EreArm::A0 | EreArm::A1) {
            EpochPolicy::Never
        } else {
            config.epoch
        };
        let mut source =
            GraphRuleSource::new(epoch, config.limits.clone(), config.arm == EreArm::A3);
        source.vocabulary_enabled=config.vocabulary.is_some();
        source.witness_repair_enabled=witnessed;
        source.sealed_repair_enabled=sealed_repair;
        source.complete_witness_enabled=complete;
        if matches!(config.arm, EreArm::A2 | EreArm::A3) {
            if let Some(toml) = &config.meta_rules_toml {
                let raw = hg_rules::parse_rules_toml(toml).map_err(|e| e.to_string())?;
                source.editors = hg_rules::compile_rules(&raw).map_err(|e| e.to_string())?;
                for r in &mut source.editors {
                    if r.schedule.is_some()
                        || r.pattern.n_vars > config.limits.max_vars.min(8)
                        || r.template.n_rhs_vars > config.limits.max_vars.min(8)
                        || r.pattern.edges.len() > config.limits.max_pattern_edges
                        || r.template.rhs_edges.len() > config.limits.max_pattern_edges
                        || r.pattern
                            .edges
                            .iter()
                            .any(|e| e.var_ixs.len() > config.limits.max_arity.min(8))
                        || r.template
                            .rhs_edges
                            .iter()
                            .any(|e| e.0.len() > config.limits.max_arity.min(8))
                    {
                        return Err("seeded editor exceeds bounded ERE matching limits".into());
                    }
                    r.meta_visible = true;
                    // A2 editors cannot delete/relabel anchors or side/var
                    // nodes: only pattern scaffolding may be rewritten.
                    if config.arm == EreArm::A2 {
                        for &v in &r.template.lhs_deleted {
                            if r.pattern.node_label_constraints[v].is_none_or(|l| {
                                ![hg_meta::L_PAT_EDGE, hg_meta::L_LABEL_LIT, hg_meta::L_GUARD]
                                    .contains(&l.0)
                            }) {
                                return Err("A2 editor deletes an unprotected anchor/variable or unconstrained node".into());
                            }
                        }
                        if r.template.rhs_relabels.iter().any(Option::is_some) {
                            return Err("A2 editor survivor relabels are not protected".into());
                        }
                        for (vars, label) in &r.template.rhs_edges {
                            if label.0 >= META_BASE
                                && vars.iter().any(|&v| match r.template.rhs_from_lhs[v] {
                                    Some(lhs) => r.pattern.node_label_constraints[lhs]
                                        .is_none_or(|l| l.0 < META_BASE),
                                    None => r.template.rhs_new_labels[v].0 < META_BASE,
                                })
                            {
                                return Err(
                                    "A2 editor creates a meta edge incident to unprotected data"
                                        .into(),
                                );
                            }
                        }
                    }
                }
            }
        }
        // Static baseline still compiles the original TOML table (not graph)
        // into the canonical table order. It is then frozen by StaticRuleSource.
        let editor_rule_count = source.editors.len();
        let maintenance = if config.measure_family_maintenance {
            anchors.iter().enumerate().map(|(family_index, root)| hg_schema::ere::EreFamilyMaintenance {
                family_index, anchor:format!("{root:?}"), final_valid:true,
                ..Default::default()
            }).collect()
        } else {Vec::new()};
        if config.arm == EreArm::A0 {
            let mut static_rules: Vec<CompiledRule> = executable
                .iter()
                .map(|c| c.compile(&config.limits, false).unwrap())
                .collect();
            static_rules.extend(static_operations);
            engine.replace_static_rules(static_rules.clone());
            engine.set_rule_source(Box::new(hg_rules::StaticRuleSource(static_rules)));
        } else {
            engine.set_rule_source(Box::new(source));
        }
        let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        engine.add_observer(Box::new(EventSink(events.clone())));
        engine.set_match_budget_policy(config.match_ceiling, config.match_ceiling,config.match_budget_policy);
        let mut runtime = Self {
            config: config.clone(),
            anchors,
            seen: baseline.clone(),
            baseline,
            classes,
            reuse: BTreeMap::new(),
            nontrivial: HashSet::new(),
            world_reuse: BTreeMap::new(),
            trivial_copy_seen: false,
            before: None,
            events,
            installed: 0,
            retired: 0,
            quarantined: 0,
            quarantine_decisions: 0,
            quarantine_by: [0; 9],
            meta_events: 0,
            cleanup_lost: 0,
            sites: BTreeMap::new(),
            half_life: None,
            window_start: store.step(),
            window_revision: None,
            stable: true,
            last_compile: None,
            epoch_effective: None,
            cap_binding: false,
            match_binding: false,
            previous_novel: 0,
            included_tpm: hg_causal::TpmAccumulator::new(),
            excluded_tpm: hg_causal::TpmAccumulator::new(),
            included_windows: hg_eval::WindowComputer::new(1),
            excluded_windows: hg_eval::WindowComputer::new(1),
            included_ticks: Vec::new(),
            excluded_ticks: Vec::new(),
            dynamic_changes: 0,
            accepted: 0,
            world_accepted: 0,
            attempted: 0,
            included_wl_seen: HashSet::new(),
            excluded_wl_seen: HashSet::new(),
            proposal_classes: HashSet::new(), core_classes: HashSet::new(),
            structural_proposal_classes: BTreeMap::new(), proposal_reuse: BTreeMap::new(),
            structural_proposal_reuse: BTreeMap::new(), ambiguous_role_reuse: BTreeMap::new(),
            family_modes: BTreeMap::new(), activation_count:0,
            maintenance, editor_rule_count, editor_events:0, ordinary_meta_events:0,
            encoded_operations,instruction_roots,instruction_states,encoded_meta_events:0,operation_traces:Vec::new(),
        };
        runtime.refresh_family_roles(store);
        Ok(runtime)
    }
    fn measure_maintenance(&mut self, store: &HypergraphStore) {
        if !self.config.measure_family_maintenance {return;}
        let snapshot = store.snapshot();
        for (&root, m) in self.anchors.iter().zip(&mut self.maintenance) {
            let failure = if !store.get_node(root).is_some_and(|n|n.label.0 == hg_meta::L_RULE_ROOT) {
                Some("seeded anchor missing or relabeled".into())
            } else {match hg_meta::read_program(&snapshot,root,&self.config.limits) {
                Ok(p) if p.proposal.is_some() => None,
                Ok(_) => Some("seeded anchor no longer encodes a whole E2 family".into()),
                Err(v) => Some(format!("{v:?}")),
            }};
            let valid = failure.is_none();
            m.checks += 1;
            m.valid_steps += u64::from(valid);
            if !valid {m.first_invalid_step.get_or_insert(store.step());}
            if m.final_valid && !valid {m.invalid_transitions += 1;}
            if !m.final_valid && valid {m.recovery_transitions += 1;}
            m.final_valid = valid;
            m.final_failure = failure;
        }
        let read_instruction=if self.config.vocabulary.as_ref().is_some_and(|v|v.protocol==hg_meta::COMPLETE_PROTOCOL) {hg_meta::read_complete_instruction}else{hg_meta::read_instruction};
        for (&root,state) in self.instruction_roots.iter().zip(&mut self.instruction_states) {
            let decoded=read_instruction(&snapshot,root).ok();
            state.checks+=1;state.valid_steps+=u64::from(decoded.is_some());
            state.valid=decoded.is_some();
            if !state.valid {state.first_invalid_step.get_or_insert(store.step());}
            state.operation=decoded.as_ref().map(|s|s.operation.name().into());
            state.fuel=decoded.map(|s|s.fuel);
        }
    }
    fn refresh_family_roles(&mut self, store: &HypergraphStore) {
        self.proposal_classes.clear(); self.core_classes.clear();
        self.structural_proposal_classes.clear();
        if self.config.encoding_version != EncodingVersion::AlternativeFamiliesV3 { return; }
        for (root, n) in store.nodes().filter(|(_,n)|n.label.0 == hg_meta::L_RULE_ROOT) {
            let _ = n;
            if let Ok(p) = hg_meta::read_program(&store.snapshot(), root, &self.config.limits) {
                if self.family_modes.insert(root,p.active) == Some(false) && p.active {self.activation_count += 1;}
                self.core_classes.insert(p.core.rcf_hash());
                if p.active {
                    if let Some(mut proposal) = p.proposal {
                        let class = proposal.rcf_hash();
                        self.proposal_classes.insert(class);
                        let mut core = p.core; core.priority = 0; proposal.priority = 0;
                        if core.rcf_bytes() != proposal.rcf_bytes() {
                            self.structural_proposal_classes.insert(class,proposal.rcf_hash());
                        }
                    }
                }
            }
        }
    }
    pub fn before_step(
        &mut self,
        store: &HypergraphStore,
        engine: &mut Engine,
    ) -> Result<(), String> {
        let meta_nodes = store
            .nodes()
            .filter(|(_, n)| n.label.0 >= META_BASE)
            .count();
        if meta_nodes > self.config.rule_region_ceiling {
            self.cap_binding = true;
            return Err("ERE rule-region ceiling is binding; run halted".into());
        }
        engine.compile_at_boundary(store);
        if let Some(report) = engine.take_compile_report() {
            self.installed += report.installed;
            self.retired += report.retired;
            if report.step > 0 {
                self.dynamic_changes += report.installed + report.retired;
            }
            self.quarantined += report.quarantined;
            self.quarantine_decisions += report.validated_regions + report.quarantined;
            for (a, b) in self
                .quarantine_by
                .iter_mut()
                .zip(report.quarantine_by_validator)
            {
                *a += b;
            }
            self.classes = report.classes;
            self.encoded_operations=report.encoded_meta_operations;
            self.refresh_family_roles(store);
            if let Some(last) = self.last_compile {
                self.epoch_effective = Some(report.step - last);
            }
            self.last_compile = Some(report.step);
            // RCF novelty is tracked with complete bytes to avoid hash collision
            // false novelty; initial classes are excluded from cumulative count.
            for &root in &self.anchors {
                if let Ok(p) = hg_meta::read_program(&store.snapshot(), root, &self.config.limits) {
                    self.seen.extend(p.candidates().iter().map(CandidateRule::rcf_bytes));
                }
            }
            for (root, n) in store.nodes() {
                if n.label.0 == hg_meta::L_RULE_ROOT {
                    if let Ok(p) = hg_meta::read_program(&store.snapshot(), root, &self.config.limits) {
                        self.seen.extend(p.candidates().iter().map(CandidateRule::rcf_bytes));
                    }
                }
            }
        }
        let revision = engine.rule_table_revision();
        if self.window_revision.is_some_and(|r| r != revision) {
            self.stable = false;
        }
        self.window_revision.get_or_insert(revision);
        self.before = Some(store.clone());
        Ok(())
    }
    pub fn after_step(&mut self, store: &HypergraphStore, stats: &hg_observe::StepStats) {
        self.accepted += stats.accepted_rewrites;
        self.attempted += stats.attempted_matches;
        self.match_binding |= stats.attempted_matches > self.config.match_ceiling;
        let before = self
            .before
            .take()
            .expect("before_step must precede after_step");
        // A capped non-advancing step is not a completed validity observation.
        if store.step() > before.step() {self.measure_maintenance(store);}
        let classes = compute_wl_class_map(&before, 2);
        let events: Vec<_> = self.events.lock().unwrap().drain(..).collect();
        for e in events {
            let editor_start=self.classes.len()+self.encoded_operations.len();
            let seeded_editor = e.rule_id >= editor_start
                && e.rule_id < editor_start + self.editor_rule_count;
            if seeded_editor {self.editor_events += 1;}
            if e.rule_id>=self.classes.len() && e.rule_id<editor_start {
                self.encoded_meta_events+=1;
                let instruction=e.binding.bound_nodes[0];let root=e.binding.bound_nodes[1];
                let fuel=|s:&HypergraphStore|s.get_node(instruction).and_then(|n|match n.get_property(hg_meta::K_FUEL) {
                    Some(hg_core::PropertyValue::Float(f))=>Some(f),_=>None});
                let old=hg_meta::read_program(&before.snapshot(),root,&self.config.limits).ok();
                let new=hg_meta::read_program(&store.snapshot(),root,&self.config.limits).ok();
                let signature=|p:&hg_meta::RuleProgram|format!("{:016x}",hg_meta::hash_bytes(&p.execution_bytes()));
                let proposal_signature=|p:&hg_meta::RuleProgram|p.proposal.as_ref().map(|c|format!("{:016x}",hg_meta::hash_bytes(&c.execution_bytes())));
                self.operation_traces.push(hg_schema::ere::EreMetaOperationTrace {step:store.step(),
                    operation:self.encoded_operations[e.rule_id-self.classes.len()].clone(),
                    instruction:format!("{instruction:?}"),target_family:format!("{root:?}"),
                    fuel_before:fuel(&before),fuel_after:fuel(store),family_valid_before:old.as_ref().is_some_and(|p|p.proposal.is_some()),
                    family_valid_after:new.as_ref().is_some_and(|p|p.proposal.is_some()),
                    executable_before:old.as_ref().map(signature),executable_after:new.as_ref().map(signature),
                    proposal_before:old.as_ref().and_then(proposal_signature),proposal_after:new.as_ref().and_then(proposal_signature)});
            }
            let touches_meta = e
                .binding
                .bound_nodes
                .iter()
                .any(|&n| before.get_node(n).is_some_and(|n| n.label.0 >= META_BASE));
            let creates_meta = e
                .nodes_created
                .iter()
                .any(|&n| store.get_node(n).is_some_and(|n| n.label.0 >= META_BASE));
            if !touches_meta && !creates_meta {
                self.world_accepted += 1;
            }
            if let Some(&class) = self.classes.get(e.rule_id) {
                *self.reuse.entry(class).or_default() += 1;
                if !touches_meta && !creates_meta {
                    self.nontrivial.insert(class);
                    *self.world_reuse.entry(class).or_default() += 1;
                    if self.proposal_classes.contains(&class) {
                        if self.core_classes.contains(&class) {
                            *self.ambiguous_role_reuse.entry(class).or_default() += 1;
                        } else {
                            *self.proposal_reuse.entry(class).or_default() += 1;
                            if let Some(&structural) = self.structural_proposal_classes.get(&class) {
                                *self.structural_proposal_reuse.entry(structural).or_default() += 1;
                            }
                        }
                    }
                }
                for &root in &e.nodes_created {
                    if store
                        .get_node(root)
                        .is_some_and(|n| n.label.0 == hg_meta::L_RULE_ROOT)
                        && hg_meta::read_program(&store.snapshot(), root, &self.config.limits)
                            .is_ok_and(|p| p.candidates().iter().any(|c|c.rcf_hash()==class))
                    {
                        self.trivial_copy_seen = true;
                    }
                }
            }
            if touches_meta || creates_meta {
                self.meta_events += 1;
                if !seeded_editor {self.ordinary_meta_events += 1;}
                for &n in &e.binding.bound_nodes {
                    if before.get_node(n).is_some_and(|n| n.label.0 >= META_BASE) {
                        *self.sites.entry(classes[&n].0).or_default() += 1;
                    }
                }
            }
            let explicit: HashSet<_> = e.binding.matched_edges.iter().copied().collect();
            let mut cleanup = HashSet::new();
            for &n in &e.nodes_deleted {
                for &edge in before.edges_incident_to(n) {
                    if before
                        .get_edge(edge)
                        .is_some_and(|e| e.label.0 >= META_BASE)
                        && !explicit.contains(&edge)
                        && store.get_edge(edge).is_none()
                    {
                        cleanup.insert(edge);
                    }
                }
            }
            self.cleanup_lost += cleanup.len();
        }
        let alive = self
            .anchors
            .iter()
            .filter(|&&n| store.get_node(n).is_some())
            .count();
        if self.half_life.is_none() && !self.anchors.is_empty() && alive * 2 <= self.anchors.len() {
            self.half_life = Some(store.step() as f64);
        }
        let before_excluded = meta_excluded_store(&before);
        let after_excluded = meta_excluded_store(store);
        for (a, b, tpm, ticks, registry) in [
            (
                &before,
                store,
                &mut self.included_tpm,
                &mut self.included_ticks,
                &mut self.included_wl_seen,
            ),
            (
                &before_excluded,
                &after_excluded,
                &mut self.excluded_tpm,
                &mut self.excluded_ticks,
                &mut self.excluded_wl_seen,
            ),
        ] {
            let from = compute_wl_class_map(a, 1);
            let to = compute_wl_class_map(b, 1);
            registry.extend(from.values().copied());
            registry.extend(to.values().copied());
            let mut pairs: Vec<_> = from
                .into_iter()
                .filter_map(|(id, h)| to.get(&id).map(|&next| (h, next)))
                .collect();
            pairs.sort_by_key(|(a, b)| (a.0, b.0));
            for &(a, b) in &pairs {
                tpm.record(a, b);
            }
            // No CI from a silently truncated history. Oversize windows are
            // marked unmeasured instead of allocating unbounded tick buffers.
            if ticks.len() <= 10_000 {
                ticks.push(pairs);
            }
        }
    }
    pub fn close_window(
        &mut self,
        store: &HypergraphStore,
        mut included: MetricsRow,
        mut excluded: MetricsRow,
    ) -> EreMetricsRow {
        let alive_classes = self.classes.iter().copied().collect::<HashSet<_>>().len();
        let novel = self.seen.difference(&self.baseline).count();
        let total_sites: usize = self.sites.values().sum();
        let site_entropy = (total_sites > 0).then(|| {
            self.sites
                .values()
                .map(|&n| {
                    let p = n as f64 / total_sites as f64;
                    -p * p.log2()
                })
                .sum()
        });
        let projected = meta_excluded_store(store);
        // Do not inherit stale causal estimates supplied by a caller. Native
        // estimates must originate from this revision-pinned window alone.
        for metrics in [&mut included, &mut excluded] {
            metrics.ei_micro = None;
            metrics.ei_macro = None;
            metrics.ce_best = None;
            metrics.ce_best_strategy = None;
            metrics.ce_structural = None;
            metrics.ce_energetic = None;
            metrics.ce_functional = None;
        }
        let duration = (store.step() - self.window_start).max(1) as f64;
        included.accepted_rewrites = self.accepted;
        included.attempted_matches = self.attempted;
        included.accepted_per_step = self.accepted as f64 / duration;
        included.wl_class_count = Some(self.included_wl_seen.len());
        excluded.accepted_rewrites = self.world_accepted;
        excluded.accepted_per_step = self.world_accepted as f64 / duration;
        excluded.wl_class_count = Some(self.excluded_wl_seen.len());
        self.included_windows
            .set_window_len(store.step() - self.window_start);
        self.excluded_windows
            .set_window_len(store.step() - self.window_start);
        add_features(
            &mut included,
            &self.included_windows.observe(&store.snapshot()),
        );
        add_features(
            &mut excluded,
            &self.excluded_windows.observe(&projected.snapshot()),
        );
        let mut reason = None;
        let mut widths = [None; 2];
        if self.stable && self.config.measure_ei && !self.cap_binding && !self.match_binding {
            for (i, (s, tpm, metrics)) in [
                (store, &self.included_tpm, &mut included),
                (&projected, &self.excluded_tpm, &mut excluded),
            ]
            .into_iter()
            .enumerate()
            {
                if tpm.class_count() > 512 {
                    reason = Some("TPM class ceiling exceeded; EI unmeasured".into());
                    continue;
                }
                if tpm.total_transitions() < self.config.min_tpm_transitions {
                    reason = Some("insufficient stationary transitions; EI unmeasured".into());
                    continue;
                }
                let class_map = compute_wl_class_map(s, 1);
                let mut features = ahash::AHashMap::default();
                for (id, n) in s.nodes() {
                    let incident = s.edges_incident_to(id);
                    let dominant_arity = incident
                        .iter()
                        .filter_map(|&e| s.get_edge(e))
                        .map(|e| e.arity() as u32)
                        .max()
                        .unwrap_or(0);
                    features
                        .entry(class_map[&id])
                        .or_insert(hg_causal::ClassFeatures {
                            degree: incident.len() as u32,
                            dominant_arity,
                            energy: n.energy,
                            motif_class: 0,
                        });
                }
                let settings = hg_causal::EiSettings {
                    min_transitions: self.config.min_tpm_transitions,
                    try_functional: false,
                    cadence_steps: 1,
                    ..hg_causal::EiSettings::default()
                };
                let results = hg_causal::compute_ei_results(
                    tpm,
                    &features,
                    &hg_causal::ConstructorRegistry::new(),
                    &settings,
                    store.step(),
                );
                for result in &results {
                    match result.strategy.as_str() {
                        "structural" => metrics.ce_structural = Some(result.ce_coefficient),
                        "energetic" => metrics.ce_energetic = Some(result.ce_coefficient),
                        "functional" => metrics.ce_functional = Some(result.ce_coefficient),
                        _ => {}
                    }
                }
                if let Some(best) = results
                    .iter()
                    .max_by(|a, b| a.ce_coefficient.total_cmp(&b.ce_coefficient))
                {
                    metrics.ei_micro = Some(best.ei_micro);
                    metrics.ei_macro = Some(best.ei_macro);
                    metrics.ce_best = Some(best.ce_coefficient);
                    metrics.ce_best_strategy = Some(best.strategy.as_str().into());
                }
                let ticks = if i == 0 {
                    &self.included_ticks
                } else {
                    &self.excluded_ticks
                };
                widths[i] = block_bootstrap_ei_width(ticks, self.window_start + i as u64);
            }
        }
        if !self.config.measure_ei {
            reason = Some("EI/CE disabled for reuse/PI-only training rollout".into());
        }
        if self.cap_binding || self.match_binding {
            reason = Some("ceiling bound; partial-run EI/CE withheld".into());
        }
        let mut included = hg_schema::EreSubstrateMetrics::from(&included);
        let mut excluded = hg_schema::EreSubstrateMetrics::from(&excluded);
        excluded.attempted_matches = None;
        excluded.unmeasured_reasons.insert(
            "attempted_matches".into(),
            "The engine does not expose the match-pool count for the projected substrate".into(),
        );
        for metrics in [&mut included, &mut excluded] {
            metrics.unmeasured_reasons.insert("geometry_and_biological_registries".into(),
                "Native dual-window tracking does not evaluate geometry or constructor/task/archive registries".into());
        }
        let mut row = EreMetricsRow {
            step_start: self.window_start, step_end: store.step(),
            rule_region_nodes: Some(store.nodes().filter(|(_, n)| n.label.0 >= META_BASE).count()),
            rule_region_edges: Some(store.edges().filter(|(_, e)| e.label.0 >= META_BASE
                || e.nodes.iter().any(|&n| store.get_node(n).is_some_and(|n| n.label.0 >= META_BASE))).count()),
            rules_installed: Some(self.installed), rules_retired: Some(self.retired), rules_quarantined: Some(self.quarantined),
            quarantine_by_validator: Some(self.quarantine_by), rule_rcf_alive: Some(alive_classes),
            rule_rcf_novel_cumulative: Some(novel),
            rule_rcf_novel_rate: Some((novel - self.previous_novel) as f64 / (store.step() - self.window_start).max(1) as f64),
            rule_survival_half_life: self.half_life, rule_reuse_count_by_class: Some(self.reuse.clone()),
            nontrivial_rule_reuse_count_by_class: Some(self.world_reuse.clone()),
            proposal_rule_reuse_count_by_class: (self.config.encoding_version == EncodingVersion::AlternativeFamiliesV3).then(||self.proposal_reuse.clone()),
            structural_proposal_reuse_count_by_class: (self.config.encoding_version == EncodingVersion::AlternativeFamiliesV3).then(||self.structural_proposal_reuse.clone()),
            ambiguous_role_reuse_count_by_class: (self.config.encoding_version == EncodingVersion::AlternativeFamiliesV3).then(||self.ambiguous_role_reuse.clone()),
            runtime_activation_count: (self.config.encoding_version == EncodingVersion::AlternativeFamiliesV3).then_some(self.activation_count),
            seeded_family_maintenance: self.config.measure_family_maintenance.then(||self.maintenance.clone()),
            seeded_editor_rewrite_events: self.config.measure_family_maintenance.then_some(self.editor_events),
            ordinary_meta_rewrite_events: self.config.measure_family_maintenance.then_some(self.ordinary_meta_events),
            encoded_meta_vocabulary_protocol: self.config.vocabulary.as_ref().map(|v|v.protocol.clone()),
            encoded_meta_rewrite_events: self.config.vocabulary.as_ref().map(|_|self.encoded_meta_events),
            encoded_meta_operation_traces: self.config.vocabulary.as_ref().map(|_|self.operation_traces.clone()),
            encoded_meta_instruction_states: self.config.vocabulary.as_ref().map(|_|self.instruction_states.clone()),
            novel_nontrivial_classes: Some(self.seen.difference(&self.baseline)
                .filter(|bytes| self.nontrivial.contains(&hg_meta::hash_bytes(bytes))).count()),
            survival_half_life_right_censored: self.half_life.is_none() && !self.anchors.is_empty(),
            meta_rewrite_events: Some(self.meta_events), meta_rewrite_site_entropy: site_entropy,
            meta_edges_lost_to_cleanup: Some(self.cleanup_lost),
            rule_replicator_kind: Some(if !self.nontrivial.is_empty() { RuleReplicatorKind::NonTrivial }
                else if self.trivial_copy_seen { RuleReplicatorKind::Trivial } else { RuleReplicatorKind::None }),
            epoch_len_effective: self.epoch_effective, tpm_rule_set_stable: Some(self.stable),
            rule_region_ceiling_binding: self.cap_binding, match_ceiling_binding: self.match_binding,
            quarantine_capture_alarm: self.quarantined as f64 / self.quarantine_decisions.max(1) as f64 >= 0.9,
            anchor_survival_fraction: (!self.anchors.is_empty()).then(|| self.anchors.iter().filter(|&&n| store.get_node(n).is_some()).count() as f64 / self.anchors.len() as f64),
            meta_excluded: Some(Box::new(excluded)), meta_included: Some(Box::new(included)),
            ei_unmeasured_reason: reason,
            ei_micro_ci_width_meta_included: widths[0], ei_micro_ci_width_meta_excluded: widths[1],
            rule_change_rate: Some(self.dynamic_changes as f64 / (store.step() - self.window_start).max(1) as f64),
            ei_ci_unmeasured_reason: widths.iter().any(Option::is_none).then(||
                "moving-block CI requires a stationary window of 32..=10000 steps and sufficient transitions in both variants".into()),
        };
        if !self.stable {
            for metrics in [&mut row.meta_excluded, &mut row.meta_included]
                .into_iter()
                .flatten()
            {
                metrics.ei_micro = None;
                metrics.ei_macro = None;
                metrics.ce_best = None;
                metrics.ce_structural = None;
                metrics.ce_energetic = None;
                metrics.ce_functional = None;
            }
            row.ei_unmeasured_reason =
                Some("rule table changed within the measurement window (G-R6)".into());
        }
        self.window_start = store.step();
        self.window_revision = None;
        self.stable = true;
        self.previous_novel = novel;
        self.included_tpm = hg_causal::TpmAccumulator::new();
        self.excluded_tpm = hg_causal::TpmAccumulator::new();
        self.included_ticks.clear();
        self.excluded_ticks.clear();
        self.dynamic_changes = 0;
        self.accepted = 0;
        self.world_accepted = 0;
        self.attempted = 0;
        self.installed = 0;
        self.retired = 0;
        self.quarantined = 0;
        self.quarantine_decisions = 0;
        self.quarantine_by = [0; 9];
        self.meta_events = 0;
        self.editor_events = 0;
        self.ordinary_meta_events = 0;
        self.encoded_meta_events=0;self.operation_traces.clear();
        self.cleanup_lost = 0;
        self.sites.clear();
        row
    }
    pub fn protection_intact(&self, store: &HypergraphStore) -> bool {
        self.anchors.iter().all(|&n| store.get_node(n).is_some()) && self.cleanup_lost == 0
    }
    /// Initialization-only, explicit exogenous diagnostic suffix. This does not
    /// relax `EreRuntime::install` or admit static editors to the vocabulary.
    pub fn install_diagnostic_environment(
        &mut self, rules: Vec<CompiledRule>, engine: &mut Engine,
    ) -> Result<(), String> {
        let visible = self.config.arm == EreArm::A3;
        if !matches!(self.config.arm, EreArm::A2 | EreArm::A3)
            || self.config.vocabulary.as_ref().is_none_or(|v| v.protocol != COMPLETE_PROTOCOL)
            || self.config.meta_rules_toml.is_some() || self.editor_rule_count != 0 {
            return Err("diagnostic environment is outside its explicit complete-witness profile".into());
        }
        if rules.len() > 8 || rules.iter().any(|r| r.schedule.is_some() || r.meta_visible != visible
            || r.pattern.node_label_ceiling != if visible { None } else { Some(META_BASE) }
            || r.pattern.n_vars == 0 || r.pattern.n_vars > self.config.limits.max_vars.min(8)
            || r.template.n_rhs_vars > self.config.limits.max_vars.min(8)
            || r.pattern.edges.len() > self.config.limits.max_pattern_edges
            || r.template.rhs_edges.len() > self.config.limits.max_pattern_edges
            || r.pattern.edges.iter().any(|e| e.var_ixs.len() > self.config.limits.max_arity.min(8))
            || r.template.rhs_edges.iter().any(|e| e.0.len() > self.config.limits.max_arity.min(8))) {
            return Err("diagnostic environment exceeds bounded rules or bypasses arm visibility".into());
        }
        let mut source = GraphRuleSource::new(self.config.epoch, self.config.limits.clone(), visible);
        source.vocabulary_enabled = true;
        source.witness_repair_enabled = true;
        source.sealed_repair_enabled = true;
        source.complete_witness_enabled = true;
        self.editor_rule_count = rules.len();
        source.editors = rules;
        engine.set_rule_source(Box::new(source));
        Ok(())
    }
    pub fn note_match_ceiling(&mut self, binding: bool) {
        self.match_binding |= binding;
    }
    pub fn meta_rewrite_count(&self) -> usize {
        self.meta_events
    }
    /// Observation only: no edits, source compilation or RNG consumption.
    pub fn observe_completed_step(&self, store: &HypergraphStore, engine: &Engine,
        world_before: String, rng_before_compile: u64, rng_after_compile: u64,
    ) -> hg_schema::ere::EreStepObservation {
        let hex = |bytes: Vec<u8>| bytes.iter().map(|b|format!("{b:02x}")).collect::<String>();
        let projected = meta_excluded_store(store);
        hg_schema::ere::EreStepObservation {
            step: store.step(), world_hash_before: world_before,
            world_hash_after: format!("{:016x}",projected.canonical_hash()),
            world_nodes: projected.node_count(), world_edges: projected.edge_count(),
            world_rule_table: engine.rules().iter().take(self.classes.len()).map(|r|
                CandidateRule::from_compiled(r,&self.config.limits).ok().map(|c|hex(c.execution_bytes()))).collect(),
            rule_table_revision: engine.rule_table_revision(), rng_before_compile:format!("{rng_before_compile:016x}"),
            rng_after_compile:format!("{rng_after_compile:016x}"),
            rng_after_step:format!("{:016x}",engine.rng_state_probe()), match_work: engine.match_budget_usage().work,
            match_results: engine.match_budget_usage().results, match_ceiling_binding: engine.match_ceiling_binding(),
            families: self.anchors.iter().map(|&root| {
                let decoded=hg_meta::read_program(&store.snapshot(),root,&self.config.limits).ok()
                    .filter(|p|p.proposal.is_some());
                hg_schema::ere::EreFamilyStepState {anchor:format!("{root:?}"),valid:decoded.is_some(),
                    active:decoded.as_ref().map(|p|p.active), execution_bytes:decoded.map(|p|hex(p.execution_bytes()))}
            }).collect(),
        }
    }
}

/// Projection really removes meta nodes and incident edges BEFORE hashing or
/// computing neighborhood metrics. Filtering only a final histogram is M1.
pub fn meta_excluded_store(store: &HypergraphStore) -> HypergraphStore {
    let mut projected = store.clone();
    let ids: Vec<_> = projected
        .nodes()
        .filter(|(_, n)| n.label.0 >= META_BASE)
        .map(|(id, _)| id)
        .collect();
    for n in ids {
        projected.remove_node(n);
    }
    let edges: Vec<_> = projected
        .edges()
        .filter(|(_, e)| e.label.0 >= META_BASE)
        .map(|(id, _)| id)
        .collect();
    for e in edges {
        projected.remove_edge(e);
    }
    projected
}
pub fn substrate_hash(store: &HypergraphStore) -> u64 {
    meta_excluded_store(store).canonical_hash()
}

pub fn add_features(row: &mut MetricsRow, f: &hg_eval::WindowFeatures) {
    row.n_nodes = f.n_nodes;
    row.n_edges = f.n_edges;
    row.mean_degree = f.mean_degree;
    row.activity = Some(f.activity);
    row.entropy_rate = Some(f.entropy_rate);
    row.pi_normalized = f.pi_normalized;
    row.pi_sample_size = Some(f.pi_sample_size);
    row.pi_sample_strategy = Some(f.pi_sample_strategy.into());
    row.type_histogram_entropy = Some(f.type_histogram_entropy);
    row.dominant_type_fraction = Some(f.dominant_type_fraction);
    row.type_histogram_jsd = Some(f.type_histogram_jsd);
    row.new_type_bin_rate = Some(f.new_type_bin_rate);
    row.active_persistent_structures = Some(f.active_persistent_structures);
    row.static_repeated_motifs = Some(f.static_repeated_motifs);
    row.static_persistent_motifs = Some(f.static_persistent_motifs);
}

pub fn write_records(dir: &std::path::Path, rows: &[EreMetricsRow]) -> std::io::Result<()> {
    use std::io::Write;
    let mut file = std::fs::File::create(dir.join("ere_metrics.jsonl"))?;
    for row in rows {
        serde_json::to_writer(&mut file, row).map_err(std::io::Error::other)?;
        writeln!(file)?;
    }
    let report = format!("# ERE run\n\n{} measurement windows. `null` means unmeasured, not zero.\n\nSeeded meta editors are excluded from endogenous RCF novelty. Exact RCF is capped at eight variables.\n\nLast window:\n\n```json\n{}\n```\n",
        rows.len(), serde_json::to_string_pretty(&rows.last()).map_err(std::io::Error::other)?);
    std::fs::write(dir.join("ere_report.md"), report)
}

/// Circular moving-block bootstrap preserves within-block temporal dependence.
/// 64 replicates, 16-step blocks, central 95% empirical interval. This is an
/// estimator CI conditional on the observed epoch, not a cross-seed claim CI.
pub fn block_bootstrap_ei_width(
    ticks: &[Vec<(hg_core::WlHash, hg_core::WlHash)>],
    seed: u64,
) -> Option<f64> {
    const BLOCK: usize = 16;
    const REPLICATES: usize = 64;
    if !(2 * BLOCK..=10_000).contains(&ticks.len()) || ticks.iter().all(Vec::is_empty) {
        return None;
    }
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0xE1_C100_0001 ^ seed);
    let mut values = Vec::new();
    for _ in 0..REPLICATES {
        let mut tpm = hg_causal::TpmAccumulator::new();
        let mut taken = 0;
        while taken < ticks.len() {
            let start = rng.gen_range(0..ticks.len());
            for k in 0..BLOCK.min(ticks.len() - taken) {
                for &(a, b) in &ticks[(start + k) % ticks.len()] {
                    tpm.record(a, b);
                }
            }
            taken += BLOCK.min(ticks.len() - taken);
        }
        if tpm.class_count() > 512 || tpm.total_transitions() == 0 {
            return None;
        }
        let rows: Vec<_> = tpm
            .normalized_rows()
            .into_iter()
            .map(|(_, row)| row)
            .collect();
        values.push(hg_causal::ei_from_tpm(&rows).ei);
    }
    values.sort_by(f64::total_cmp);
    Some(values[62] - values[1])
}
