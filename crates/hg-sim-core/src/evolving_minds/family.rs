//! Paid faithful D0 copying, bounded ecology and complete native event journal.
use super::admission::*;
use super::descendant::*;
use super::funded_admission::*;
use super::handoff::*;
use super::lifecycle::*;
use super::mutation::*;
use super::retained::*;
use super::stream::*;
use super::transfer_environment::*;
use super::{
    choice::{ChoiceSchedule, CueMode, ATTEMPT, CHOICE, CLOCK, COIN, CUE, RESOURCE},
    config::*,
    d0::*,
    runtime::{checksum, float, int, Capture},
    Event, Frame,
};
use bincode::Options;
use hg_core::{HyperEdge, HypergraphStore, Label, NodeData, NodeId, PropertyValue};
use hg_engine::{engine::EngineCheckpoint, Engine};
use hg_observe::StepStats;
use hg_rules::{compile_rules, parse_rules_toml, CompiledRule, MatchBudgetPolicy, StepMode};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, Mutex},
};
pub const FAMILY_PROFILE: &str = "evolving-minds-d0-family-v1";
pub const FAMILY_LAWS: &str = include_str!("../../../../rules/evolving_minds/family_v1.toml");
pub const GENOME_LENGTH: u32 = 242012;
pub const INTERNAL_PARTS: u32 = 242013;
pub const FAMILY_TICKS: u64 = 64;
pub const FAMILY_STEPS: u64 = 43;
pub fn copy_cost(length: usize) -> f64 {
    1.0 + 0.25 * (11 + length) as f64
}
pub fn seed_upkeep(length: usize) -> f64 {
    2.0 + 0.125 * length as f64
}
pub fn family_program(name: &str) -> Result<D0Program, String> {
    if name == "inactive" {
        Ok(D0Program {
            instructions: vec![D0Instruction {
                id: 100,
                op: D0Op::Halt,
                slot: 0,
                source: 0,
                initial: false,
            }],
        })
    } else {
        d0_program(name)
    }
}
pub fn family_law_text(s: &ChoiceSchedule) -> String {
    let d0 = d0_law_text(s);
    let start = d0.find("\n[[rules]]\nname = \"world_").unwrap();
    format!("{FAMILY_LAWS}{}", &d0[start..])
}
fn bool_value(n: &NodeData, k: u32) -> Result<bool, String> {
    match n.get_property(k) {
        Some(PropertyValue::Bool(v)) => Ok(v),
        _ => Err(format!("family Bool {k}")),
    }
}
fn total(store: &HypergraphStore) -> Result<f64, String> {
    store
        .nodes()
        .filter(|(_, n)| [ROOT, SITE, DEAD, ESCROW].contains(&n.label.0))
        .map(|(_, n)| float(n, ENERGY))
        .sum()
}
fn key(label: u32, a: NodeId, b: NodeId) -> (u32, BTreeSet<NodeId>) {
    (label, BTreeSet::from([a, b]))
}
fn schema(n: &NodeData, mutation: bool, lifecycle: bool, admission: bool) -> Result<(), String> {
    let keys: Vec<_> = match n.label.0 {
        ROOT | DEAD => vec![
            ENERGY,
            BIRTH_ID,
            AGE,
            CURSOR,
            STATUS,
            HAS_ACTUATOR,
            ATTEMPT,
            CHOICE,
            MAINTENANCE,
            PARENT_ID,
            GENOME_LENGTH,
            INTERNAL_PARTS,
        ],
        SITE => {
            let mut k = vec![ENERGY, CLOCK, 241009, RESOURCE, CUE, COIN, OCCUPIED];
            if admission {
                k.push(LOCAL_ID)
            }
            k
        }
        COUNTER => {
            if mutation {
                vec![NEXT_ID, REQUEST]
            } else {
                vec![NEXT_ID]
            }
        }
        ESCROW if mutation || admission => vec![ENERGY, OWNER, GENOME_LENGTH, REQUEST, TX_STATUS],
        SENSOR => vec![OWNER, DEST, SIGNAL],
        STATE | ACTUATOR => vec![OWNER, DEST, SIGNAL, BUFFER, HAS_INPUT],
        SLOT => vec![OWNER, DEST, USED],
        GENE => vec![OWNER, POSITION, INSTRUCTION_ID, OPCODE, DEST, SRC, INITIAL],
        CONNECTION => {
            if lifecycle {
                vec![OWNER, COMMUNICATION_ENABLED]
            } else {
                vec![OWNER]
            }
        }
        _ => return Err("family unsupported label".into()),
    };
    if n.energy != 0.0
        || n.properties.len() != keys.len()
        || n.properties
            .iter()
            .map(|(k, _)| *k)
            .collect::<BTreeSet<_>>()
            != keys.into_iter().collect()
    {
        return Err("family hidden energy/property schema".into());
    }
    Ok(())
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FamilyBirth {
    pub id: i64,
    pub parent: Option<i64>,
    pub born_tick: u64,
    pub death_tick: Option<u64>,
    pub site: i64,
    pub program: D0Program,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FamilyStep {
    pub tick: u64,
    pub pass: u8,
    pub phase: String,
    pub step: u64,
    pub events: Vec<Event>,
    pub stats: StepStats,
    pub match_work: usize,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct FamilyRow {
    pub tick: u64,
    pub engine_step: u64,
    pub live_organisms: usize,
    pub births: u64,
    pub deaths: u64,
    pub max_ancestry_depth: usize,
    pub development_paid: u64,
    pub development_failures: u64,
    pub attempts: u64,
    pub successes: u64,
    pub failures: u64,
    pub harvested: f64,
    pub influx: f64,
    pub dissipated: f64,
    pub total_energy: f64,
    pub residual: f64,
    pub match_work: usize,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct Ledger {
    initial: f64,
    influx: f64,
    dissipated: f64,
    harvested: f64,
    births: u64,
    deaths: u64,
    development_paid: u64,
    development_failures: u64,
    attempts: u64,
    successes: u64,
    failures: u64,
    journal_bytes: usize,
}
#[derive(Serialize, Deserialize)]
struct Saved {
    profile: String,
    program: D0Program,
    schedule: ChoiceSchedule,
    store: HypergraphStore,
    allocator: hg_core::graph::GraphAllocationState,
    engine: EngineCheckpoint,
    tick: u64,
    lineage: BTreeMap<i64, FamilyBirth>,
    rows: Vec<FamilyRow>,
    journal: Vec<FamilyStep>,
    ledger: Ledger,
    law_identity: u64,
}
pub struct FamilySimulation {
    pub program: D0Program,
    pub schedule: ChoiceSchedule,
    pub initial: Frame,
    pub lineage: BTreeMap<i64, FamilyBirth>,
    pub rows: Vec<FamilyRow>,
    pub journal: Vec<FamilyStep>,
    store: HypergraphStore,
    engine: Engine,
    capture: Arc<Mutex<Vec<Event>>>,
    rules: Vec<CompiledRule>,
    tick: u64,
    ledger: Ledger,
    quiescent: bool,
    pub mutation: Option<MutationState>,
    pub lifecycle: Option<LifecycleState>,
    pub admission: Option<AdmissionState>,
    pub streaming: Option<LiveAncestry>,
    pub neighborhood: Option<super::neighborhood::NeighborhoodRecipe>,
    funding_endowment: Option<u8>,
    funded_admission: Option<FundedAdmissionConfig>,
    transfer_environment: Option<TransferEnvironment>,
    mutation_stress: bool,
    retained: Option<RetainedState>,
    handoff: Option<HandoffConfig>,
    descendant: Option<DescendantState>,
    neighborhood_reserve_rules: Vec<CompiledRule>,
    pub(crate) neighborhood_lifetimes: BTreeMap<i64, super::neighborhood::ChildLifetime>,
}
impl FamilySimulation {
    pub fn new(program: D0Program, seed: u64, mode: CueMode) -> Result<Self, String> {
        program.validate()?;
        let base = D0Simulation::new(program.clone(), seed, mode, 64.0)?;
        let mut store = base.store().clone();
        use PropertyValue::{Bool as B, Float as F, Int as I};
        let ids: Vec<_> = store.nodes().map(|(id, _)| id).collect();
        let mut sites = vec![];
        for id in ids {
            let n = store.get_node_mut(id).unwrap();
            if n.label.0 == ROOT {
                n.set_property(MAINTENANCE, F(seed_upkeep(program.instructions.len())));
                n.set_property(PARENT_ID, I(0));
                n.set_property(GENOME_LENGTH, I(program.instructions.len() as i64));
                n.set_property(INTERNAL_PARTS, I((10 + program.instructions.len()) as i64));
            } else if n.label.0 == SITE {
                n.set_property(ENERGY, F(32.0));
                n.set_property(OCCUPIED, B(true));
                sites.push(id);
            }
        }
        for slot in 1..8 {
            let mut n = NodeData::new(Label(SITE), 0);
            for (k, v) in [
                (ENERGY, F(32.0)),
                (CLOCK, I(0)),
                (241009, I(slot)),
                (RESOURCE, B(false)),
                (CUE, B(false)),
                (COIN, B(false)),
                (OCCUPIED, B(false)),
            ] {
                n.set_property(k, v)
            }
            sites.push(store.add_node(n));
        }
        for i in 0..8 {
            store.add_edge(HyperEdge::new(
                Label(NEIGHBOR),
                [sites[i], sites[(i + 1) % 8]].into_iter().collect(),
            ));
        }
        let mut n = NodeData::new(Label(COUNTER), 0);
        n.set_property(NEXT_ID, I(2));
        store.add_node(n);
        let schedule = ChoiceSchedule::new(seed, mode);
        let rules = compile_rules(
            &parse_rules_toml(&family_law_text(&schedule)).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        let mut engine = Engine::new(vec![], StepMode::MaxParallel, seed, 1_000_000);
        engine.set_match_budget_policy(1_000_000, 100_000, MatchBudgetPolicy::Pooled);
        let capture = Arc::new(Mutex::new(vec![]));
        engine.add_observer(Box::new(Capture(capture.clone())));
        let initial = Frame::new(&store, 0, None, vec![], StepStats::default(), 0);
        let ledger = Ledger {
            initial: total(&store)?,
            ..Default::default()
        };
        let lineage = BTreeMap::from([(
            1,
            FamilyBirth {
                id: 1,
                parent: None,
                born_tick: 0,
                death_tick: None,
                site: 0,
                program: program.clone(),
            },
        )]);
        let sim = Self {
            program,
            schedule,
            initial,
            lineage,
            rows: vec![],
            journal: vec![],
            store,
            engine,
            capture,
            rules,
            tick: 0,
            ledger,
            quiescent: true,
            mutation: None,
            lifecycle: None,
            admission: None,
            streaming: None,
            neighborhood: None,
            funding_endowment: None,
            funded_admission: None,
            transfer_environment: None,
            mutation_stress: false,
            retained: None,
            handoff: None,
            descendant: None,
            neighborhood_reserve_rules: vec![],
            neighborhood_lifetimes: BTreeMap::new(),
        };
        sim.validate()?;
        Ok(sim)
    }
    pub fn new_admission(recipe: AdmissionRecipe) -> Result<Self, String> {
        Self::new_admission_impl(recipe, None)
    }
    pub fn new_funded_admission(config: FundedAdmissionConfig) -> Result<Self, String> {
        config.validate()?;
        if config
            .kernels
            .contains(&MutationKernel::GrammarSourceKernel)
        {
            return Err("grammar mutation requires stress profile".into());
        }
        Self::new_admission_impl(config.coordinate_recipe(), Some(config))
    }
    pub fn new_transfer(
        config: FundedAdmissionConfig,
        environment: TransferEnvironment,
    ) -> Result<Self, String> {
        environment.validate(&config)?;
        let mut s = Self::new_funded_admission(config.clone())?;
        s.schedule = environment.schedule(config.seed);
        s.rules = compile_rules(
            &parse_rules_toml(&environment.law_text(&config)?).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        s.transfer_environment = Some(environment);
        s.validate()?;
        Ok(s)
    }
    pub fn new_mutation_stress(
        config: FundedAdmissionConfig,
        environment: TransferEnvironment,
    ) -> Result<Self, String> {
        environment.validate_stress(&config)?;
        let mut s = Self::new_admission_impl(config.coordinate_recipe(), Some(config.clone()))?;
        s.schedule = environment.schedule(config.seed);
        s.rules = compile_rules(
            &parse_rules_toml(&environment.law_text(&config)?).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        s.transfer_environment = Some(environment);
        s.mutation_stress = true;
        s.validate()?;
        Ok(s)
    }
    pub fn mutation_stress_policy(&self) -> Option<&'static str> {
        self.mutation_stress.then_some(MUTATION_STRESS_POLICY)
    }
    pub fn transfer_environment(&self) -> Option<&TransferEnvironment> {
        self.transfer_environment.as_ref()
    }
    pub fn new_retained(config: FundedAdmissionConfig) -> Result<Self, String> {
        let mut s = Self::new_funded_admission(config)?;
        s.retained = Some(RetainedState::default());
        s.validate()?;
        Ok(s)
    }
    pub fn new_handoff(
        config: FundedAdmissionConfig,
        handoff: HandoffConfig,
    ) -> Result<Self, String> {
        handoff.validate(&config)?;
        let mut s = Self::new_funded_admission(config)?;
        s.handoff = Some(handoff);
        s.validate()?;
        Ok(s)
    }
    pub fn new_descendant(
        config: FundedAdmissionConfig,
        policy: DescendantConfig,
    ) -> Result<Self, String> {
        policy.validate(&config)?;
        let mut s = Self::new_funded_admission(config)?;
        s.descendant = Some(DescendantState {
            config: policy,
            targets: BTreeMap::new(),
        });
        s.validate()?;
        Ok(s)
    }
    pub fn descendant_state(&self) -> Option<&DescendantState> {
        self.descendant.as_ref()
    }
    pub fn descendant_config(&self) -> Option<&DescendantConfig> {
        self.descendant.as_ref().map(|x| &x.config)
    }
    pub fn handoff_config(&self) -> Option<&HandoffConfig> {
        self.handoff.as_ref()
    }
    pub fn retained_state(&self) -> Option<&RetainedState> {
        self.retained.as_ref()
    }
    pub fn funded_config(&self) -> Option<&FundedAdmissionConfig> {
        self.funded_admission.as_ref()
    }
    fn admission_endowment(&self) -> u8 {
        self.funded_admission.as_ref().map_or(12, |c| c.endowment)
    }
    fn new_admission_impl(
        recipe: AdmissionRecipe,
        config: Option<FundedAdmissionConfig>,
    ) -> Result<Self, String> {
        if config.is_none() {
            recipe.validate()?;
        }
        let programs = if let Some(c) = &config {
            c.programs()?
        } else {
            recipe.programs()?
        };
        let homes = config
            .as_ref()
            .map_or_else(|| recipe.homes(), |c| c.homes());
        let seed = config.as_ref().map_or(17, |c| c.seed);
        let mut s = Self::new(programs[0].clone(), seed, CueMode::Informative)?;
        let counter = s
            .store
            .nodes()
            .find(|(_, n)| n.label.0 == COUNTER)
            .unwrap()
            .0;
        s.store.remove_node(counter);
        let sites: BTreeMap<_, _> = s
            .store
            .nodes()
            .filter(|(_, n)| n.label.0 == SITE)
            .map(|(h, n)| (int(n, 241009).unwrap(), h))
            .collect();
        for (&slot, &h) in &sites {
            let n = s.store.get_node_mut(h).unwrap();
            n.set_property(OCCUPIED, PropertyValue::Bool(homes.contains(&slot)));
            n.set_property(LOCAL_ID, PropertyValue::Int(3 + recipe.logical(slot)));
        }
        let edge = s
            .store
            .edges()
            .find(|(_, e)| e.label.0 == OCCUPANCY)
            .unwrap()
            .0;
        s.store.remove_edge(edge);
        let root = s.store.nodes().find(|(_, n)| n.label.0 == ROOT).unwrap().0;
        s.store.add_edge(HyperEdge::new(
            Label(OCCUPANCY),
            [root, sites[&homes[0]]].into_iter().collect(),
        ));
        s.lineage.get_mut(&1).unwrap().site = homes[0];
        let second = D0Simulation::new(programs[1].clone(), seed, CueMode::Informative, 64.0)?;
        let mut translated = BTreeMap::new();
        for (h, n) in second.store().nodes().filter(|(_, n)| n.label.0 != SITE) {
            let mut n = n.clone();
            if n.label.0 == ROOT {
                n.set_property(BIRTH_ID, PropertyValue::Int(2));
                n.set_property(PARENT_ID, PropertyValue::Int(0));
                n.set_property(
                    GENOME_LENGTH,
                    PropertyValue::Int(programs[1].instructions.len() as i64),
                );
                n.set_property(
                    INTERNAL_PARTS,
                    PropertyValue::Int((10 + programs[1].instructions.len()) as i64),
                );
                n.set_property(
                    MAINTENANCE,
                    PropertyValue::Float(seed_upkeep(programs[1].instructions.len())),
                );
            } else {
                n.set_property(OWNER, PropertyValue::Int(2));
            }
            translated.insert(h, s.store.add_node(n));
        }
        for (_, e) in second.store().edges() {
            let ns = e
                .nodes
                .iter()
                .map(|h| translated.get(h).copied().unwrap_or(sites[&homes[1]]))
                .collect();
            s.store.add_edge(HyperEdge::new(e.label, ns));
        }
        s.lineage.insert(
            2,
            FamilyBirth {
                id: 2,
                parent: None,
                born_tick: 0,
                death_tick: None,
                site: homes[1],
                program: programs[1].clone(),
            },
        );
        if recipe.reversed {
            let mut replacement = HypergraphStore::new();
            let mut ns: Vec<_> = s.store.nodes().map(|(h, n)| (h, n.clone())).collect();
            ns.reverse();
            let mut map = BTreeMap::new();
            for (h, n) in ns {
                map.insert(h, replacement.add_node(n));
            }
            let mut es: Vec<_> = s.store.edges().map(|(_, e)| e.clone()).collect();
            es.reverse();
            for e in es {
                replacement.add_edge(HyperEdge::new(
                    e.label,
                    e.nodes.iter().map(|h| map[h]).collect(),
                ));
            }
            s.store = replacement;
        }
        s.funded_admission = config;
        s.admission = Some(AdmissionState::new(recipe));
        s.rules = compile_rules(&parse_rules_toml(&s.law_text()).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        s.ledger.initial = total(&s.store)?;
        s.initial = Frame::new(&s.store, 0, None, vec![], StepStats::default(), 0);
        s.validate()?;
        Ok(s)
    }
    pub(crate) fn configure_lifecycle(
        program: D0Program,
        seed: u64,
        mode: CueMode,
        state: LifecycleState,
    ) -> Result<Self, String> {
        if program != family_program("cue")? || seed != 17 || mode != CueMode::Informative {
            return Err("lifecycle fixed recipe".into());
        }
        let mut s = Self::new(program, seed, mode)?;
        s.lifecycle = Some(state);
        s.rules = compile_rules(
            &parse_rules_toml(&lifecycle_law_text(&s.schedule)).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        s.validate()?;
        Ok(s)
    }
    pub fn new_mutation(
        program: D0Program,
        seed: u64,
        mode: CueMode,
        kernel: MutationKernel,
    ) -> Result<Self, String> {
        if kernel == MutationKernel::GrammarSourceKernel {
            return Err("grammar mutation requires stress profile".into());
        }
        let mut s = Self::new(program, seed, mode)?;
        let counter = s
            .store
            .nodes()
            .find(|(_, n)| n.label.0 == COUNTER)
            .unwrap()
            .0;
        s.store
            .get_node_mut(counter)
            .unwrap()
            .set_property(REQUEST, PropertyValue::Int(1));
        s.mutation = Some(MutationState::new(kernel, seed));
        s.rules = compile_rules(
            &parse_rules_toml(&mutation_law_text(&s.schedule)).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        s.initial = Frame::new(&s.store, 0, None, vec![], StepStats::default(), 0);
        s.validate()?;
        Ok(s)
    }
    /// Initial allocation permutation control; does not change physical sites or semantic IDs.
    pub fn new_mutation_reversed(
        program: D0Program,
        seed: u64,
        mode: CueMode,
        kernel: MutationKernel,
    ) -> Result<Self, String> {
        let mut s = Self::new_mutation(program, seed, mode, kernel)?;
        let mut replacement = HypergraphStore::new();
        let mut nodes: Vec<_> = s.store.nodes().map(|(id, n)| (id, n.clone())).collect();
        nodes.reverse();
        let mut translated = BTreeMap::new();
        for (old, n) in nodes {
            translated.insert(old, replacement.add_node(n));
        }
        let mut edges: Vec<_> = s.store.edges().map(|(_, e)| e.clone()).collect();
        edges.reverse();
        for e in edges {
            replacement.add_edge(HyperEdge::new(
                e.label,
                e.nodes.iter().map(|id| translated[id]).collect(),
            ));
        }
        s.mutation.as_mut().unwrap().allocation_reversed = true;
        s.store = replacement;
        s.initial = Frame::new(&s.store, 0, None, vec![], StepStats::default(), 0);
        s.validate()?;
        Ok(s)
    }
    /// Separate bounded funding profile. It intentionally cannot export legacy checkpoints.
    pub fn new_funding(
        recipe: super::neighborhood::NeighborhoodRecipe,
        endowment: u8,
    ) -> Result<Self, String> {
        super::funding::validate_endowment(endowment)?;
        let mut s = Self::new_neighborhood(recipe)?;
        s.funding_endowment = Some(endowment);
        s.rules = compile_rules(
            &parse_rules_toml(&super::funding::funding_law_text(&s.schedule, endowment)?)
                .map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        s.neighborhood_reserve_rules = compile_rules(
            &parse_rules_toml(&super::funding::funding_descendant_text(endowment)?)
                .map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        s.validate()?;
        Ok(s)
    }
    pub fn new_neighborhood(
        recipe: super::neighborhood::NeighborhoodRecipe,
    ) -> Result<Self, String> {
        recipe.validate()?;
        let mut s = Self::new_mutation(
            family_program(&recipe.program)?,
            17,
            CueMode::Informative,
            MutationKernel::Faithful,
        )?;
        s.neighborhood_reserve_rules = compile_rules(
            &parse_rules_toml(&super::neighborhood::descendant_reserve_text()?)
                .map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        s.neighborhood = Some(recipe);
        s.validate()?;
        Ok(s)
    }
    pub(crate) fn law_text(&self) -> String {
        if let Some(e) = &self.transfer_environment {
            e.law_text(
                self.funded_admission
                    .as_ref()
                    .expect("transfer funded configuration"),
            )
            .expect("validated temporal laws")
        } else if let Some(c) = &self.funded_admission {
            c.law_text().expect("validated funded native laws")
        } else if self.admission.is_some() {
            admission_law_text(&self.schedule)
        } else if self.lifecycle.is_some() {
            lifecycle_law_text(&self.schedule)
        } else if self.mutation.is_some() {
            mutation_law_text(&self.schedule)
        } else {
            family_law_text(&self.schedule)
        }
    }
    pub fn store(&self) -> &HypergraphStore {
        &self.store
    }
    pub fn final_frame(&self) -> Frame {
        Frame::new(
            &self.store,
            self.tick,
            None,
            vec![],
            StepStats::default(),
            0,
        )
    }
    pub fn graph_program(&self, owner: i64) -> Result<D0Program, String> {
        let mut genes: Vec<_> = self
            .store
            .nodes()
            .filter(|(_, n)| n.label.0 == GENE && int(n, OWNER).ok() == Some(owner))
            .collect();
        genes.sort_by_key(|(_, n)| int(n, POSITION).unwrap_or(-1));
        let mut instructions = vec![];
        for (pos, (_, g)) in genes.iter().enumerate() {
            if int(g, POSITION)? != pos as i64 {
                return Err("family genotype order".into());
            }
            let op = match int(g, OPCODE)? {
                0 => D0Op::State,
                1 => D0Op::Actuator,
                2 => D0Op::Wire,
                3 => D0Op::Halt,
                _ => return Err("family genotype opcode".into()),
            };
            instructions.push(D0Instruction {
                id: u32::try_from(int(g, INSTRUCTION_ID)?).map_err(|_| "family instruction ID")?,
                op,
                slot: u8::try_from(int(g, DEST)?).map_err(|_| "family destination")?,
                source: u8::try_from(int(g, SRC)?).map_err(|_| "family source")?,
                initial: bool_value(g, INITIAL)?,
            });
        }
        let p = D0Program { instructions };
        p.validate()?;
        Ok(p)
    }
    pub fn validate(&self) -> Result<(), String> {
        if [
            self.mutation.is_some(),
            self.lifecycle.is_some(),
            self.admission.is_some(),
        ]
        .into_iter()
        .filter(|v| *v)
        .count()
            > 1
        {
            return Err("incompatible experimental profiles".into());
        }
        if let Some(c) = &self.funded_admission {
            c.validate()?;
            if self
                .admission
                .as_ref()
                .is_none_or(|a| a.recipe != c.coordinate_recipe())
                || self.mutation.is_some()
                || self.lifecycle.is_some()
                || self.neighborhood.is_some()
                || self.funding_endowment.is_some()
                || self.program != c.programs()?[0]
                || self.schedule
                    != self.transfer_environment.as_ref().map_or_else(
                        || ChoiceSchedule::new(c.seed, CueMode::Informative),
                        |e| e.schedule(c.seed),
                    )
            {
                return Err("funded admission profile identity".into());
            }
        }
        if self.mutation_stress && self.transfer_environment.is_none() {
            return Err("stress environment missing".into());
        }
        if let Some(e) = &self.transfer_environment {
            let cfg = self
                .funded_admission
                .as_ref()
                .ok_or("transfer requires funded config")?;
            if self.mutation_stress {
                e.validate_stress(cfg)?;
            } else {
                e.validate(cfg)?;
            }
            if self.handoff.is_some() || self.descendant.is_some() || self.retained.is_some() {
                return Err("temporal transfer proposal policy conflict".into());
            }
        }
        if let Some(d) = &self.descendant {
            d.validate(
                self.funded_admission
                    .as_ref()
                    .ok_or("descendant funded config")?,
                self.tick + u64::from(!self.quiescent),
            )?;
            if self.retained.is_some() || self.handoff.is_some() {
                return Err("incompatible descendant policies".into());
            }
        }
        if let Some(h) = &self.handoff {
            h.validate(
                self.funded_admission
                    .as_ref()
                    .ok_or("handoff funded config")?,
            )?;
            if self.retained.is_some() {
                return Err("incompatible proposal policies".into());
            }
        }
        if let Some(r) = &self.retained {
            let cfg = self
                .funded_admission
                .as_ref()
                .ok_or("retained missing funded configuration")?;
            r.validate(cfg, self.tick + u64::from(!self.quiescent))?;
            if let Some(a) = &self.admission {
                for record in a.records.iter().filter(|x| [1, 2].contains(&x.parent)) {
                    if !r.samples.contains_key(&record.parent) {
                        return Err("retained request without first sample".into());
                    }
                }
            }
        }
        if let Some(endowment) = self.funding_endowment {
            super::funding::validate_endowment(endowment)?;
            if self.neighborhood.is_none() {
                return Err("funding missing fixed diagnostic profile".into());
            }
        }
        if let Some(n) = &self.neighborhood {
            n.validate()?;
            if self.mutation.as_ref().is_none_or(|m| {
                m.kernel != MutationKernel::Faithful || m.seed != 17 || m.allocation_reversed
            }) || self.streaming.is_some()
                || self.lifecycle.is_some()
                || self.admission.is_some()
                || self.program != family_program(&n.program)?
                || self.schedule != ChoiceSchedule::new(17, CueMode::Informative)
            {
                return Err("neighborhood fixed profile".into());
            }
        }
        if let Some(live) = &self.streaming {
            if self.admission.is_none()
                || self.mutation.is_some()
                || self.lifecycle.is_some()
                || live.members.keys().ne(self.lineage.keys())
                || self.ledger.births + 2
                    != self.ledger.deaths
                        + self
                            .lineage
                            .values()
                            .filter(|b| b.death_tick.is_none())
                            .count() as u64
            {
                return Err("stream live lineage coverage".into());
            }
            for (&id, b) in &self.lineage {
                let a = &live.members[&id];
                if ![1, 2].contains(&a.founder)
                    || a.depth > live.maximum_depth
                    || a.depth > self.ledger.births as usize
                    || b.parent.is_none() != (a.depth == 0)
                    || b.parent.is_none() && a.founder != id
                {
                    return Err("stream live ancestor metadata".into());
                }
                if let Some(parent) = b.parent.and_then(|p| live.members.get(&p)) {
                    if parent.depth + 1 != a.depth || parent.founder != a.founder {
                        return Err("stream live parent metadata".into());
                    }
                }
            }
        }
        if self.store.node_count() > 256 || self.store.edge_count() > 768 {
            return Err("family expansion ceiling".into());
        }
        let mut roots = BTreeMap::new();
        let mut sites = BTreeMap::new();
        let mut owned: BTreeMap<i64, Vec<(NodeId, &NodeData)>> = BTreeMap::new();
        let mut counters = vec![];
        for (id, n) in self.store.nodes() {
            schema(
                n,
                self.mutation.is_some(),
                self.lifecycle.is_some(),
                self.admission.is_some(),
            )?;
            if self.lifecycle.is_some() && n.label.0 == CONNECTION {
                bool_value(n, COMMUNICATION_ENABLED)?;
            }
            if n.generation > self.store.step() {
                return Err("family creation clock".into());
            }
            match n.label.0 {
                ROOT | DEAD => {
                    if float(n, ENERGY)? < 0.0 || roots.insert(int(n, BIRTH_ID)?, (id, n)).is_some()
                    {
                        return Err("family root reserve/identity".into());
                    }
                }
                SITE => {
                    if float(n, ENERGY)? < 0.0 || sites.insert(int(n, 241009)?, (id, n)).is_some() {
                        return Err("family site reserve/slot".into());
                    }
                    for k in [OCCUPIED, RESOURCE, CUE, COIN] {
                        bool_value(n, k)?;
                    }
                    if !(0..=64).contains(&int(n, CLOCK)?) {
                        return Err("family clock".into());
                    }
                }
                COUNTER => counters.push(n),
                ESCROW if self.mutation.is_some() || self.admission.is_some() => {}
                _ => owned.entry(int(n, OWNER)?).or_default().push((id, n)),
            }
        }
        if sites.keys().copied().collect::<Vec<_>>() != (0..8).collect::<Vec<_>>()
            || if self.admission.is_some() {
                !counters.is_empty()
            } else {
                counters.len() != 1 || int(counters[0], NEXT_ID)? != self.lineage.len() as i64 + 1
            }
            || owned.keys().any(|bid| !roots.contains_key(bid))
        {
            return Err("family scaffold/counter/orphan".into());
        }
        let actual: BTreeSet<_> = self
            .store
            .edges()
            .map(|(_, e)| (e.label.0, e.nodes.iter().copied().collect::<BTreeSet<_>>()))
            .collect();
        if actual.len() != self.store.edge_count()
            || self
                .store
                .edges()
                .any(|(_, e)| e.nodes.len() != 2 || e.nodes[0] == e.nodes[1])
        {
            return Err("family duplicate/self/arity relation".into());
        }
        if let Some(m) = &self.mutation {
            m.validate_endowment(&self.store, self.funding_endowment.unwrap_or(12) as f64)?;
            if int(counters[0], REQUEST)? != m.records.len() as i64 + 1 {
                return Err("mutation request counter".into());
            }
        }
        if let Some(a) = &self.admission {
            a.validate_funded(&self.store, self.admission_endowment())?;
            let elapsed = self.store.step() / ADMISSION_STEPS
                + u64::from(self.store.step() % ADMISSION_STEPS >= 3);
            for (&slot, &(_, n)) in &sites {
                if int(n, LOCAL_ID)? != 3 + a.recipe.logical(slot) + 8 * elapsed as i64 {
                    return Err("admission site frontier".into());
                }
            }
        }
        let mut expected = BTreeSet::new();
        if self.mutation.is_some() || self.admission.is_some() {
            for (_, e) in self
                .store
                .edges()
                .filter(|(_, e)| [FUNDS, PLACE].contains(&e.label.0))
            {
                expected.insert((e.label.0, e.nodes.iter().copied().collect()));
            }
        }
        for i in 0..8 {
            expected.insert(key(NEIGHBOR, sites[&i].0, sites[&((i + 1) % 8)].0));
        }
        let mut occupied = BTreeSet::new();
        for (&bid, &(root, r)) in &roots {
            let birth = self
                .lineage
                .get(&bid)
                .ok_or("family root missing ancestry")?;
            if int(r, PARENT_ID)? != birth.parent.unwrap_or(0)
                || int(r, GENOME_LENGTH)? != birth.program.instructions.len() as i64
                || int(r, INTERNAL_PARTS)? != owned.get(&bid).map_or(0, |p| p.len()) as i64
            {
                return Err("family ancestry/part count".into());
            }
            let members = owned.get(&bid).map(Vec::as_slice).unwrap_or(&[]);
            for &(id, _) in members {
                expected.insert(key(OWN_PART, root, id));
            }
            if r.label.0 == DEAD {
                if birth.death_tick.is_none() || float(r, ENERGY)? != 0.0 {
                    return Err("family unsettled tombstone".into());
                }
                // Partial disassembly permits missing relations, only within this dead owner.
                for (_, e) in self.store.edges() {
                    let a = e.nodes[0];
                    let b = e.nodes[1];
                    let na = self.store.get_node(a).unwrap();
                    let nb = self.store.get_node(b).unwrap();
                    if int(na, OWNER).ok() != Some(bid) || int(nb, OWNER).ok() != Some(bid) {
                        continue;
                    }
                    let legal = match e.label.0 {
                        NEXT_INSTRUCTION => {
                            na.label.0 == GENE
                                && nb.label.0 == GENE
                                && (int(na, POSITION)? - int(nb, POSITION)?).abs() == 1
                        }
                        SLOT_MAP => {
                            let (slot, part) = if na.label.0 == SLOT {
                                (na, nb)
                            } else {
                                (nb, na)
                            };
                            slot.label.0 == SLOT
                                && [SENSOR, STATE, ACTUATOR].contains(&part.label.0)
                                && int(slot, DEST)? == int(part, DEST)?
                        }
                        SOURCE | TARGET => {
                            let (wire, part) = if na.label.0 == CONNECTION {
                                (na, nb)
                            } else {
                                (nb, na)
                            };
                            wire.label.0 == CONNECTION
                                && if e.label.0 == SOURCE {
                                    [SENSOR, STATE, ACTUATOR].contains(&part.label.0)
                                } else {
                                    [STATE, ACTUATOR].contains(&part.label.0)
                                }
                        }
                        _ => false,
                    };
                    if !legal {
                        return Err("family dead relation grammar".into());
                    }
                    expected.insert((e.label.0, e.nodes.iter().copied().collect()));
                }
                continue;
            }
            if birth.death_tick.is_some()
                || int(r, AGE)? < 0
                || int(r, AGE)? > 16
                || !occupied.insert(birth.site)
            {
                return Err("family live age/site/ancestry".into());
            }
            expected.insert(key(OCCUPANCY, root, sites[&birth.site].0));
            let program = self.graph_program(bid)?;
            if program.canonical_bytes()? != birth.program.canonical_bytes()? {
                return Err("family unfaithful genotype".into());
            }
            let cursor = usize::try_from(int(r, CURSOR)?).map_err(|_| "family cursor")?;
            let status = int(r, STATUS)?;
            if cursor > program.instructions.len()
                || !(0..=3).contains(&status)
                || (status == 2) != (cursor == program.instructions.len())
                || (status == 3
                    && (cursor == program.instructions.len()
                        || shape(&program, cursor + 1).is_ok()))
            {
                return Err("family execution status".into());
            }
            let (wanted, wanted_wires) = shape(&program, cursor)?;
            let mut parts = BTreeMap::new();
            let mut slots = BTreeMap::new();
            let mut genes = BTreeMap::new();
            let mut wires = vec![];
            for &(id, n) in members {
                match n.label.0 {
                    SENSOR | STATE | ACTUATOR => {
                        if parts.insert(int(n, DEST)?, (id, n)).is_some() {
                            return Err("family duplicate material slot".into());
                        }
                        bool_value(n, SIGNAL)?;
                        if n.label.0 != SENSOR {
                            bool_value(n, BUFFER)?;
                            bool_value(n, HAS_INPUT)?;
                        }
                    }
                    SLOT => {
                        if slots.insert(int(n, DEST)?, (id, n)).is_some() {
                            return Err("family duplicate slot descriptor".into());
                        }
                        bool_value(n, USED)?;
                    }
                    GENE => {
                        genes.insert(int(n, POSITION)?, id);
                    }
                    CONNECTION => wires.push(id),
                    _ => return Err("family owned material role".into()),
                }
            }
            if parts.len() != wanted.len()
                || slots.keys().copied().collect::<Vec<_>>() != (0..8).collect::<Vec<_>>()
                || genes.len() != program.instructions.len()
                || wires.len() != wanted_wires.len()
            {
                return Err("family material inventory".into());
            }
            for (&slot, &(label, _)) in &wanted {
                let &(id, n) = parts.get(&(slot as i64)).ok_or("family missing slot")?;
                if n.label.0 != label
                    || (label != SENSOR
                        && bool_value(n, HAS_INPUT)?
                            != wanted_wires.iter().any(|&(_, d)| d == slot))
                {
                    return Err("family port role/input".into());
                }
                expected.insert(key(SLOT_MAP, slots[&(slot as i64)].0, id));
            }
            for (&slot, &(_, n)) in &slots {
                if bool_value(n, USED)? != parts.contains_key(&slot) {
                    return Err("family slot occupancy".into());
                }
            }
            for i in 1..genes.len() {
                expected.insert(key(
                    NEXT_INSTRUCTION,
                    genes[&((i - 1) as i64)],
                    genes[&(i as i64)],
                ));
            }
            let mut remaining = wanted_wires.clone();
            for w in wires {
                let srcs: Vec<_> = parts
                    .iter()
                    .filter(|(_, (p, _))| actual.contains(&key(SOURCE, *p, w)))
                    .map(|(&s, _)| s as u8)
                    .collect();
                let dsts: Vec<_> = parts
                    .iter()
                    .filter(|(_, (p, _))| actual.contains(&key(TARGET, w, *p)))
                    .map(|(&s, _)| s as u8)
                    .collect();
                if srcs.len() != 1 || dsts.len() != 1 {
                    return Err("family wire ports".into());
                }
                let pair = (srcs[0], dsts[0]);
                let i = remaining
                    .iter()
                    .position(|p| *p == pair)
                    .ok_or("family unprogrammed wire")?;
                remaining.remove(i);
                expected.insert(key(SOURCE, parts[&(pair.0 as i64)].0, w));
                expected.insert(key(TARGET, w, parts[&(pair.1 as i64)].0));
            }
            let added = members.len() - 10 - program.instructions.len();
            if float(r, MAINTENANCE)?
                != seed_upkeep(program.instructions.len()) + 0.25 * added as f64
                || bool_value(r, HAS_ACTUATOR)?
                    != parts.values().any(|(_, n)| n.label.0 == ACTUATOR)
                || (bool_value(r, ATTEMPT)? && status != 2)
            {
                return Err("family price/action grammar".into());
            }
            bool_value(r, CHOICE)?;
        }
        if actual != expected {
            return Err("family full relation grammar".into());
        }
        for (&slot, &(_, n)) in &sites {
            if bool_value(n, OCCUPIED)? != occupied.contains(&slot) {
                return Err("family site occupancy".into());
            }
        }
        for (&bid, b) in &self.lineage {
            if b.id != bid
                || (b.death_tick.is_none() && !roots.contains_key(&bid))
                || b.parent.is_some_and(|p| {
                    p <= 0
                        || p >= bid
                        || (self.streaming.is_none() && !self.lineage.contains_key(&p))
                })
            {
                return Err("family ancestry coverage".into());
            }
        }
        Ok(())
    }
    pub fn advance_tick(&mut self) -> Result<(), String> {
        if !self.quiescent || self.tick >= 64 {
            return Err("family tick boundary".into());
        }
        self.validate()?;
        if self.streaming.is_some()
            && (!self.journal.is_empty()
                || !self.rows.is_empty()
                || self
                    .admission
                    .as_ref()
                    .is_some_and(|a| !a.rounds.is_empty() || !a.records.is_empty()))
        {
            return Err("stream tick must drain before advancing".into());
        }
        self.quiescent = false;
        let mut work = 0;
        let mut phases: Vec<_> = vec![
            "environment_",
            "world_",
            "sense_0",
            "sense_1",
            "prepare_2_",
            "prepare_3_",
            "prepare_4_",
            "prepare_5_",
            "prepare_6_",
            "prepare_7_",
            "commit_",
            "development_",
            "attempt_",
            "settle_",
            "maintenance_",
            "birth_",
            "death_",
        ];
        if self.mutation.is_some() || self.admission.is_some() {
            phases.splice(15..16, ["reserve_", "transaction_", "escrow_"]);
        }
        if self.lifecycle.is_some() {
            phases.insert(4, "communication_");
        }
        if self.admission.is_some() {
            phases.insert(2, "identity_");
        }
        phases.extend(std::iter::repeat("cleanup_").take(26));
        for (pass, prefix) in phases.into_iter().enumerate() {
            let before = self.store.clone();
            if prefix == "reserve_" && self.admission.is_some() {
                let endowment = self.admission_endowment();
                let rules = self.admission.as_mut().unwrap().reserve_rules_funded(
                    &self.store,
                    &self.lineage,
                    self.schedule.seed,
                    self.tick + 1,
                    endowment,
                )?;
                self.engine.replace_static_rules(rules);
            } else if prefix == "reserve_"
                && self.neighborhood.is_some()
                && !self.mutation.as_ref().unwrap().records.is_empty()
            {
                self.engine
                    .replace_static_rules(self.neighborhood_reserve_rules.clone());
            } else if prefix == "transaction_" && self.admission.is_some() {
                let endowment = self.admission_endowment();
                self.engine.replace_static_rules(
                    self.admission
                        .as_mut()
                        .unwrap()
                        .settlement_rules_funded(endowment)?,
                );
            } else if prefix == "communication_" {
                let name = self.lifecycle.as_ref().unwrap().arm.law(self.tick + 1);
                self.engine.replace_static_rules(
                    self.rules
                        .iter()
                        .filter(|r| r.name == name)
                        .cloned()
                        .collect(),
                );
            } else if prefix == "transaction_" {
                let laws = if self.mutation.as_ref().unwrap().pending.is_some() {
                    self.mutation
                        .as_ref()
                        .unwrap()
                        .record()?
                        .settlement_rule
                        .clone()
                } else {
                    String::new()
                };
                let rules = if laws.is_empty() {
                    vec![]
                } else {
                    compile_rules(&parse_rules_toml(&laws).map_err(|e| e.to_string())?)
                        .map_err(|e| e.to_string())?
                };
                self.engine.replace_static_rules(rules);
            } else {
                self.engine.replace_static_rules(
                    self.rules
                        .iter()
                        .filter(|r| r.name.starts_with(prefix))
                        .cloned()
                        .collect(),
                );
            }
            self.capture.lock().unwrap().clear();
            let stats = self.engine.step(&mut self.store);
            if self.engine.match_ceiling_binding() || stats.property_effects_skipped != 0 {
                return Err("family match ceiling/skipped effect".into());
            }
            let events = std::mem::take(&mut *self.capture.lock().unwrap());
            if prefix == "maintenance_"
                && events.len() != before.nodes().filter(|(_, n)| n.label.0 == ROOT).count()
            {
                return Err("family maintenance coverage".into());
            }
            if self.admission.is_none()
                && ["birth_", "reserve_"].contains(&prefix)
                && events.len() > 1
            {
                return Err("family serialized birth bound".into());
            }
            if prefix == "reserve_" {
                if let Some(a) = &self.admission {
                    let round = a.rounds.last().unwrap();
                    let mut actual = vec![];
                    for e in &events {
                        let p = e
                            .bound
                            .iter()
                            .filter_map(|h| before.get_node(*h))
                            .find(|n| n.label.0 == ROOT)
                            .ok_or("admission observed parent")?;
                        let target = e
                            .created
                            .iter()
                            .filter_map(|h| self.store.get_node(*h))
                            .find(|n| n.label.0 == ESCROW)
                            .ok_or("admission observed escrow")?;
                        actual.push((int(p, BIRTH_ID)?, int(target, REQUEST)?));
                    }
                    actual.sort();
                    let mut expected: Vec<_> = round
                        .selected
                        .iter()
                        .map(|&i| {
                            let o = &round.offers[i];
                            (o.parent, 3 + o.target + 8 * (self.tick + 1) as i64)
                        })
                        .collect();
                    expected.sort();
                    if actual != expected {
                        return Err("admission selected/native coverage".into());
                    }
                }
            }
            let mut processing: Vec<_> = events.iter().collect();
            if self.descendant.is_some() && prefix == "reserve_" {
                processing.sort_by_key(|e| {
                    e.bound
                        .iter()
                        .filter_map(|h| before.get_node(*h))
                        .find(|n| n.label.0 == ROOT)
                        .and_then(|n| int(n, BIRTH_ID).ok())
                        .unwrap_or(i64::MAX)
                });
            }
            for e in processing {
                if e.vanished_energy != 0.0 {
                    return Err("family unsettled deletion energy".into());
                }
                if self.neighborhood.is_some() {
                    if let Some(root) = e
                        .bound
                        .iter()
                        .filter_map(|h| before.get_node(*h))
                        .find(|n| n.label.0 == ROOT)
                    {
                        let id = int(root, BIRTH_ID)?;
                        if let Some(c) = self.neighborhood_lifetimes.get_mut(&id) {
                            match e.rule.as_str() {
                                n if n.starts_with("development_") && n != "development_wait" => {
                                    c.development_paid += 1;
                                    if n == "development_fail" {
                                        c.development_failures += 1;
                                    }
                                    if n == "development_halt" {
                                        c.paid_construction_completed = true;
                                    }
                                }
                                "attempt_commit" => c.attempts += 1,
                                "settle_success" => {
                                    c.successes += 1;
                                    c.harvested += 8.0;
                                }
                                "settle_failure" => c.failures += 1,
                                n if n.starts_with("maintenance_") => {
                                    c.maintenance_paid +=
                                        float(root, ENERGY)?.min(float(root, MAINTENANCE)?);
                                    c.last_age = int(root, AGE)? + 1;
                                }
                                "death_age" | "death_empty" => {
                                    c.death_tick = Some(self.tick + 1);
                                    c.death_cause = Some(e.rule.clone());
                                    c.reserve_at_death = Some(float(root, ENERGY)?);
                                }
                                _ => {}
                            }
                        }
                    }
                }
                match e.rule.as_str() {
                    "environment_influx" => self.ledger.influx += 8.0,
                    "development_wait" => {}
                    s if s.starts_with("development_") => {
                        self.ledger.development_paid += 1;
                        self.ledger.dissipated += 1.0;
                        if s == "development_fail" {
                            self.ledger.development_failures += 1
                        }
                    }
                    "attempt_commit" => {
                        self.ledger.attempts += 1;
                        self.ledger.dissipated += 1.0
                    }
                    "settle_success" => {
                        self.ledger.successes += 1;
                        self.ledger.harvested += 8.0
                    }
                    "settle_failure" => self.ledger.failures += 1,
                    s if s.starts_with("maintenance_") => {
                        let r = e
                            .bound
                            .iter()
                            .filter_map(|id| before.get_node(*id))
                            .find(|n| n.label.0 == ROOT)
                            .ok_or("family maintained root")?;
                        self.ledger.dissipated += float(r, ENERGY)?.min(float(r, MAINTENANCE)?);
                    }
                    s if s.starts_with("reserve_") => {
                        let parent = e
                            .bound
                            .iter()
                            .filter_map(|id| before.get_node(*id))
                            .find(|n| n.label.0 == ROOT)
                            .ok_or("mutation admitting parent")?;
                        let p = self.graph_program(int(parent, BIRTH_ID)?)?;
                        if let Some(a) = &mut self.admission {
                            let bid = int(parent, BIRTH_ID)?;
                            if let Some(c) = &self.funded_admission {
                                let mut founder = bid;
                                if let Some(live) = &self.streaming {
                                    founder = live
                                        .members
                                        .get(&bid)
                                        .ok_or("funded live founder")?
                                        .founder;
                                } else {
                                    while let Some(parent) = self
                                        .lineage
                                        .get(&founder)
                                        .ok_or("funded founder ancestry")?
                                        .parent
                                    {
                                        founder = parent;
                                    }
                                }
                                let kernel = *c
                                    .kernels
                                    .get((founder - 1) as usize)
                                    .ok_or("funded founder kernel")?;
                                let kernel =
                                    if self.retained.as_ref().is_some_and(|r| {
                                        bid != founder || r.samples.contains_key(&bid)
                                    }) || (self.handoff.is_some() && bid == founder)
                                    {
                                        MutationKernel::Faithful
                                    } else {
                                        kernel
                                    };
                                a.admit_origin_kernel(
                                    &self.store,
                                    bid,
                                    p,
                                    self.schedule.seed,
                                    self.tick + 1,
                                    founder,
                                    kernel,
                                )?;
                                if let Some(r) = &mut self.retained {
                                    r.apply(a, bid)?;
                                }
                                if let Some(h) = &self.handoff {
                                    h.apply(a, bid)?;
                                }
                                if let Some(d) = &mut self.descendant {
                                    d.apply(a, bid, founder)?;
                                }
                            } else if let Some(live) = &self.streaming {
                                a.admit_origin(
                                    &self.store,
                                    bid,
                                    p,
                                    self.schedule.seed,
                                    self.tick + 1,
                                    live.members
                                        .get(&bid)
                                        .ok_or("stream founder metadata")?
                                        .founder,
                                )?;
                            } else {
                                a.admit(
                                    &self.store,
                                    bid,
                                    p,
                                    self.schedule.seed,
                                    self.tick + 1,
                                    &self.lineage,
                                )?;
                            }
                        } else {
                            let m = self.mutation.as_mut().unwrap();
                            if let Some(recipe) = &self.neighborhood {
                                if m.records.is_empty() {
                                    m.admit_enumerated(
                                        &self.store,
                                        p,
                                        self.tick + 1,
                                        recipe.edit(),
                                    )?;
                                } else {
                                    m.admit(&self.store, p, self.tick + 1)?;
                                }
                            } else {
                                m.admit(&self.store, p, self.tick + 1)?;
                            }
                        }
                        if let Some(endowment) = self.funding_endowment {
                            let r = self
                                .mutation
                                .as_mut()
                                .unwrap()
                                .records
                                .last_mut()
                                .ok_or("funding missing proposal")?;
                            r.settlement_rule =
                                super::funding::funded_text(&r.settlement_rule, endowment)?;
                        }
                        self.ledger.dissipated += 1.0;
                    }
                    s if s.starts_with("transaction_selected_")
                        && !e
                            .created
                            .iter()
                            .filter_map(|h| self.store.get_node(*h))
                            .any(|n| n.label.0 == ROOT) =>
                    {
                        let escrow = e
                            .bound
                            .iter()
                            .filter_map(|h| before.get_node(*h))
                            .find(|n| n.label.0 == ESCROW)
                            .ok_or("admission rejected escrow")?;
                        let endowment = self.admission_endowment();
                        self.admission.as_mut().unwrap().settle_funded(
                            int(escrow, REQUEST)?,
                            None,
                            endowment,
                        )?;
                    }
                    "transaction_reject" => {
                        self.mutation
                            .as_mut()
                            .unwrap()
                            .settle_endowment(None, self.funding_endowment.unwrap_or(12) as f64)?;
                    }
                    "escrow_dispose" => {
                        if let Some(m) = &mut self.mutation {
                            m.dispose()?;
                        } else if self.admission.is_none() {
                            return Err("unprofiled escrow disposal".into());
                        }
                    }
                    s if s.starts_with("birth_")
                        || s.starts_with("transaction_copy_")
                        || s.starts_with("transaction_selected_") =>
                    {
                        let child = e
                            .created
                            .iter()
                            .filter_map(|id| self.store.get_node(*id))
                            .find(|n| n.label.0 == ROOT)
                            .ok_or("family missing child")?;
                        let id = int(child, BIRTH_ID)?;
                        let parent = int(child, PARENT_ID)?;
                        if self.neighborhood.is_some() {
                            self.neighborhood_lifetimes.insert(
                                id,
                                super::neighborhood::ChildLifetime {
                                    id,
                                    born_tick: self.tick + 1,
                                    ..Default::default()
                                },
                            );
                        }
                        let site = e
                            .bound
                            .iter()
                            .filter_map(|id| self.store.get_node(*id))
                            .find(|n| {
                                n.label.0 == SITE
                                    && bool_value(n, OCCUPIED).ok() == Some(true)
                                    && before.nodes().any(|(_, b)| {
                                        b.label.0 == SITE
                                            && int(b, 241009).ok() == int(n, 241009).ok()
                                            && bool_value(b, OCCUPIED).ok() == Some(false)
                                    })
                            })
                            .ok_or("family birth target")?;
                        let program = self.graph_program(id)?;
                        if !self.lineage.contains_key(&parent)
                            || self.lineage[&parent].death_tick.is_some()
                            || if let Some(m) = &self.mutation {
                                m.record()?.parent != parent
                                    || m.record()?.proposed_program != program
                            } else if let Some(a) = &self.admission {
                                a.records
                                    .iter()
                                    .find(|r| r.request_id == id)
                                    .ok_or("admission child proposal")?
                                    .proposed_program
                                    != program
                            } else {
                                self.lineage[&parent].program.canonical_bytes()?
                                    != program.canonical_bytes()?
                            }
                        {
                            return Err("family parent/copy identity".into());
                        }
                        self.ledger.dissipated += copy_cost(program.instructions.len());
                        self.ledger.births += 1;
                        if let Some(m) = &mut self.mutation {
                            m.settle_endowment(
                                Some(id),
                                self.funding_endowment.unwrap_or(12) as f64,
                            )?;
                        }
                        let endowment = self.admission_endowment();
                        if let Some(a) = &mut self.admission {
                            a.settle_funded(id, Some(id), endowment)?;
                        }
                        if let Some(live) = &mut self.streaming {
                            let a = live
                                .members
                                .get(&parent)
                                .ok_or("stream child ancestry")?
                                .clone();
                            let depth = a.depth + 1;
                            live.maximum_depth = live.maximum_depth.max(depth);
                            if live
                                .members
                                .insert(
                                    id,
                                    LiveAncestor {
                                        founder: a.founder,
                                        depth,
                                    },
                                )
                                .is_some()
                            {
                                return Err("stream reused ancestry identity".into());
                            }
                        }
                        if self
                            .lineage
                            .insert(
                                id,
                                FamilyBirth {
                                    id,
                                    parent: Some(parent),
                                    born_tick: self.tick + 1,
                                    death_tick: None,
                                    site: int(site, 241009)?,
                                    program,
                                },
                            )
                            .is_some()
                        {
                            return Err("family reused birth identity".into());
                        }
                    }
                    s if s.starts_with("death_") => {
                        let r = e
                            .bound
                            .iter()
                            .filter_map(|id| before.get_node(*id))
                            .find(|n| n.label.0 == ROOT)
                            .ok_or("family dying root")?;
                        let b = self
                            .lineage
                            .get_mut(&int(r, BIRTH_ID)?)
                            .ok_or("family unknown death")?;
                        if b.death_tick.replace(self.tick + 1).is_some() {
                            return Err("family repeated death".into());
                        }
                        self.ledger.deaths += 1;
                    }
                    _ => {}
                }
            }
            self.validate()?;
            if total(&self.store)?
                != self.ledger.initial + self.ledger.influx - self.ledger.dissipated
            {
                return Err("family conservation".into());
            }
            let usage = self.engine.match_budget_usage().work;
            work += usage;
            let entry = FamilyStep {
                tick: self.tick + 1,
                pass: pass as u8,
                phase: prefix.into(),
                step: self.store.step(),
                events,
                stats,
                match_work: usage,
            };
            self.ledger.journal_bytes +=
                serde_json::to_vec(&entry).map_err(|e| e.to_string())?.len() + 1;
            if self.ledger.journal_bytes > 16 * 1024 * 1024 {
                return Err("family journal ceiling".into());
            }
            self.journal.push(entry);
        }
        if self.store.nodes().any(|(_, n)| {
            n.label.0 == DEAD
                || n.label.0 == ESCROW
                || (n.label.0 == ROOT && bool_value(n, ATTEMPT).ok() != Some(false))
        }) || self.ledger.attempts != self.ledger.successes + self.ledger.failures
        {
            return Err("family unsettled tick".into());
        }
        self.tick += 1;
        let maximum_depth = if let Some(live) = &self.streaming {
            live.maximum_depth
        } else {
            let mut depth = BTreeMap::new();
            for (&bid, b) in &self.lineage {
                depth.insert(bid, b.parent.map_or(0, |p| depth[&p] + 1));
            }
            depth.values().copied().max().unwrap_or(0)
        };
        self.rows.push(FamilyRow {
            tick: self.tick,
            engine_step: self.store.step(),
            live_organisms: self
                .store
                .nodes()
                .filter(|(_, n)| n.label.0 == ROOT)
                .count(),
            births: self.ledger.births,
            deaths: self.ledger.deaths,
            max_ancestry_depth: maximum_depth,
            development_paid: self.ledger.development_paid,
            development_failures: self.ledger.development_failures,
            attempts: self.ledger.attempts,
            successes: self.ledger.successes,
            failures: self.ledger.failures,
            harvested: self.ledger.harvested,
            influx: self.ledger.influx,
            dissipated: self.ledger.dissipated,
            total_energy: total(&self.store)?,
            residual: 0.0,
            match_work: work,
        });
        self.quiescent = true;
        Ok(())
    }
    pub fn run(&mut self) -> Result<(), String> {
        while self.tick < 64 {
            self.advance_tick()?
        }
        Ok(())
    }
    pub fn checkpoint(&self) -> Result<Vec<u8>, String> {
        if self.transfer_environment.is_some() {
            return Err("temporal profile requires its live codec".into());
        }
        if !self.quiescent
            || self.streaming.is_some()
            || self.neighborhood.is_some()
            || self.funded_admission.is_some()
        {
            return Err("family checkpoint boundary/profile".into());
        }
        self.validate()?;
        let state = Saved {
            profile: if self.admission.is_some() {
                ADMISSION_PROFILE
            } else if self.lifecycle.is_some() {
                LIFECYCLE_PROFILE
            } else if self.mutation.is_some() {
                MUTATION_PROFILE
            } else {
                FAMILY_PROFILE
            }
            .into(),
            program: self.program.clone(),
            schedule: self.schedule.clone(),
            store: self.store.clone(),
            allocator: self.store.allocation_state(),
            engine: self.engine.diagnostic_checkpoint()?,
            tick: self.tick,
            lineage: self.lineage.clone(),
            rows: self.rows.clone(),
            journal: self.journal.clone(),
            ledger: self.ledger.clone(),
            law_identity: checksum(self.law_text().as_bytes()),
        };
        let payload = if let Some(a) = &self.admission {
            bincode::DefaultOptions::new()
                .with_fixint_encoding()
                .serialize(&(state, a))
        } else if let Some(c) = &self.lifecycle {
            bincode::DefaultOptions::new()
                .with_fixint_encoding()
                .serialize(&(state, c))
        } else if let Some(m) = &self.mutation {
            bincode::DefaultOptions::new()
                .with_fixint_encoding()
                .serialize(&(state, m))
        } else {
            bincode::DefaultOptions::new()
                .with_fixint_encoding()
                .serialize(&state)
        }
        .map_err(|e| e.to_string())?;
        if payload.len() > 16 * 1024 * 1024 {
            return Err("family checkpoint bound".into());
        }
        let mut bytes = if self.admission.is_some() {
            b"HGEMA001"
        } else if self.lifecycle.is_some() {
            b"HGEML001"
        } else if self.mutation.is_some() {
            b"HGEMM002"
        } else {
            b"HGEMF001"
        }
        .to_vec();
        bytes.extend((payload.len() as u64).to_le_bytes());
        bytes.extend(checksum(&payload).to_le_bytes());
        bytes.extend(payload);
        Ok(bytes)
    }
    pub fn resume(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 24
            || bytes.len() > 16 * 1024 * 1024 + 24
            || (&bytes[..8] != b"HGEMF001"
                && &bytes[..8] != b"HGEMM002"
                && &bytes[..8] != b"HGEMM001"
                && &bytes[..8] != b"HGEML001"
                && &bytes[..8] != b"HGEMA001")
            || u64::from_le_bytes(bytes[8..16].try_into().unwrap()) as usize != bytes.len() - 24
            || checksum(&bytes[24..]) != u64::from_le_bytes(bytes[16..24].try_into().unwrap())
        {
            return Err("family checkpoint envelope".into());
        }
        let options = bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .with_limit(16 * 1024 * 1024)
            .reject_trailing_bytes();
        let (mut c, mutation, lifecycle, admission): (
            Saved,
            Option<MutationState>,
            Option<LifecycleState>,
            Option<AdmissionState>,
        ) = if &bytes[..8] == b"HGEMA001" {
            let (c, a) = options
                .deserialize::<(Saved, AdmissionState)>(&bytes[24..])
                .map_err(|e| e.to_string())?;
            (c, None, None, Some(a))
        } else if &bytes[..8] == b"HGEML001" {
            let (c, l) = options
                .deserialize::<(Saved, LifecycleState)>(&bytes[24..])
                .map_err(|e| e.to_string())?;
            (c, None, Some(l), None)
        } else if &bytes[..8] == b"HGEMM002" {
            let (c, m) = options
                .deserialize::<(Saved, MutationState)>(&bytes[24..])
                .map_err(|e| e.to_string())?;
            (c, Some(m), None, None)
        } else if &bytes[..8] == b"HGEMM001" {
            let (c, m) = options
                .deserialize::<(Saved, LegacyMutationState)>(&bytes[24..])
                .map_err(|e| e.to_string())?;
            (c, Some(m.into()), None, None)
        } else {
            (
                options
                    .deserialize(&bytes[24..])
                    .map_err(|e| e.to_string())?,
                None,
                None,
                None,
            )
        };
        let steps = if admission.is_some() {
            ADMISSION_STEPS
        } else if lifecycle.is_some() {
            LIFECYCLE_STEPS
        } else if mutation.is_some() {
            MUTATION_STEPS
        } else {
            FAMILY_STEPS
        };
        let profile = if admission.is_some() {
            ADMISSION_PROFILE
        } else if lifecycle.is_some() {
            LIFECYCLE_PROFILE
        } else if mutation.is_some() {
            MUTATION_PROFILE
        } else {
            FAMILY_PROFILE
        };
        let laws = if admission.is_some() {
            admission_law_text(&c.schedule)
        } else if lifecycle.is_some() {
            lifecycle_law_text(&c.schedule)
        } else if mutation.is_some() {
            mutation_law_text(&c.schedule)
        } else {
            family_law_text(&c.schedule)
        };
        if c.profile != profile
            || c.schedule != ChoiceSchedule::new(c.schedule.seed, c.schedule.mode)
            || c.law_identity != checksum(laws.as_bytes())
            || c.tick > 64
            || c.rows.len() != c.tick as usize
            || c.journal.len() != c.tick as usize * steps as usize
            || c.store.step() != c.tick * steps
            || c.engine.source.is_some()
        {
            return Err("family checkpoint identity/coverage".into());
        }
        c.store
            .restore_allocation_state(c.allocator)
            .map_err(str::to_string)?;
        let mut sim = if let Some(a) = &admission {
            {
                if c.program != a.recipe.programs()?[0]
                    || c.schedule.seed != 17
                    || c.schedule.mode != CueMode::Informative
                {
                    return Err("admission checkpoint fixed identity".into());
                }
                Self::new_admission(a.recipe.clone())?
            }
        } else if let Some(l) = &lifecycle {
            Self::configure_lifecycle(c.program, c.schedule.seed, c.schedule.mode, l.clone())?
        } else if let Some(m) = &mutation {
            if m.seed != c.schedule.seed || m.pending.is_some() {
                return Err("mutation checkpoint boundary/stream".into());
            }
            if m.allocation_reversed {
                Self::new_mutation_reversed(c.program, c.schedule.seed, c.schedule.mode, m.kernel)?
            } else {
                Self::new_mutation(c.program, c.schedule.seed, c.schedule.mode, m.kernel)?
            }
        } else {
            Self::new(c.program, c.schedule.seed, c.schedule.mode)?
        };
        while sim.tick < c.tick {
            sim.advance_tick()?
        }
        for (name, a, b) in [
            (
                "material",
                serde_json::to_value(&sim.store),
                serde_json::to_value(&c.store),
            ),
            (
                "allocator",
                serde_json::to_value(sim.store.allocation_state()),
                serde_json::to_value(c.store.allocation_state()),
            ),
            (
                "engine",
                serde_json::to_value(sim.engine.diagnostic_checkpoint()?),
                serde_json::to_value(&c.engine),
            ),
            (
                "lineage",
                serde_json::to_value(&sim.lineage),
                serde_json::to_value(&c.lineage),
            ),
            (
                "rows",
                serde_json::to_value(&sim.rows),
                serde_json::to_value(&c.rows),
            ),
            (
                "journal",
                serde_json::to_value(&sim.journal),
                serde_json::to_value(&c.journal),
            ),
            (
                "ledger",
                serde_json::to_value(&sim.ledger),
                serde_json::to_value(&c.ledger),
            ),
        ] {
            if a.map_err(|e| e.to_string())? != b.map_err(|e| e.to_string())? {
                return Err(format!("family checkpoint {name} history mismatch"));
            }
        }
        if serde_json::to_value(&sim.mutation).map_err(|e| e.to_string())?
            != serde_json::to_value(&mutation).map_err(|e| e.to_string())?
        {
            return Err("mutation checkpoint sampling history mismatch".into());
        }
        if sim.lifecycle != lifecycle {
            return Err("lifecycle checkpoint arm mismatch".into());
        }
        if sim.admission != admission {
            return Err("admission checkpoint arbitration/proposal history mismatch".into());
        }
        sim.store = c.store;
        sim.engine = Engine::from_diagnostic_checkpoint(c.engine, None)?;
        sim.capture = Arc::new(Mutex::new(vec![]));
        sim.engine
            .add_observer(Box::new(Capture(sim.capture.clone())));
        sim.validate()?;
        Ok(sim)
    }
}

#[cfg(test)]
mod admission_checkpoint_tests {
    use super::*;
    #[test]
    fn forged_admission_histories_and_partial_boundary_are_rejected() {
        let mut s = FamilySimulation::new_admission(AdmissionRecipe {
            case: "faithful_disjoint".into(),
            rotation: 0,
            reversed: false,
        })
        .unwrap();
        s.quiescent = false;
        assert!(s.checkpoint().is_err());
        s.quiescent = true;
        for _ in 0..3 {
            s.advance_tick().unwrap();
        }
        let bytes = s.checkpoint().unwrap();
        let options = bincode::DefaultOptions::new().with_fixint_encoding();
        for mutation in 0..3 {
            let (mut saved, mut a): (Saved, AdmissionState) =
                options.deserialize(&bytes[24..]).unwrap();
            match mutation {
                0 => saved.program.instructions[0].id += 1,
                1 => a.rounds[2].selected.clear(),
                _ => a.records[0].request_charge = 0.0,
            }
            let payload = options.serialize(&(saved, a)).unwrap();
            let mut forged = b"HGEMA001".to_vec();
            forged.extend((payload.len() as u64).to_le_bytes());
            forged.extend(checksum(&payload).to_le_bytes());
            forged.extend(payload);
            assert!(
                FamilySimulation::resume(&forged).is_err(),
                "forgery {mutation}"
            );
        }
        let mut trailing = bytes;
        trailing.push(0);
        assert!(FamilySimulation::resume(&trailing).is_err());
    }
}

#[derive(Serialize, Deserialize)]
struct LiveSaved {
    profile: String,
    program: D0Program,
    schedule: ChoiceSchedule,
    store: HypergraphStore,
    allocator: hg_core::graph::GraphAllocationState,
    engine: EngineCheckpoint,
    tick: u64,
    lineage: BTreeMap<i64, FamilyBirth>,
    ledger: Ledger,
    law_identity: u64,
    admission: AdmissionState,
    live: LiveAncestry,
    cursor: EvidenceCursor,
}
#[derive(Serialize, Deserialize)]
struct FundedLiveSaved {
    config: FundedAdmissionConfig,
    state: LiveSaved,
}
#[derive(Serialize, Deserialize)]
struct RetainedLiveSaved {
    funded: FundedLiveSaved,
    retained: RetainedState,
}

#[derive(Serialize, Deserialize)]
struct HandoffLiveSaved {
    funded: FundedLiveSaved,
    handoff: HandoffConfig,
}

#[derive(Serialize, Deserialize)]
struct DescendantLiveSaved {
    funded: FundedLiveSaved,
    descendant: DescendantState,
}

#[derive(Serialize, Deserialize)]
struct TransferLiveSaved {
    funded: FundedLiveSaved,
    environment: TransferEnvironment,
}

#[cfg(test)]
mod temporal_checkpoint_tests {
    use super::*;
    #[test]
    fn temporal_rehashed_environment_forgery_is_refused() {
        let case = super::super::transfer_gate1_cases().unwrap()[1].clone();
        let mut s = FamilySimulation::new_transfer_stream(case.config, case.environment).unwrap();
        for _ in 0..8 {
            s.advance_tick().unwrap();
            s.drain_stream_tick().unwrap();
        }
        let cursor = EvidenceCursor {
            segment_sha256: "a".repeat(64),
            tick: 8,
            step: 368,
            local_ticks: 8,
            offset: 0,
            log_sha256: "b".repeat(64),
            previous_commit_sha256: None,
        };
        let cp = s.live_checkpoint(cursor.clone()).unwrap();
        let mut att = s.live_attestation(cursor.clone()).unwrap();
        let mut saved: TransferLiveSaved = bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .deserialize(&cp[24..])
            .unwrap();
        saved.environment.pattern = TemporalPattern::Blocks4;
        att["native"]["transfer_environment"] = serde_json::to_value(&saved.environment).unwrap();
        let payload = bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .serialize(&saved)
            .unwrap();
        let mut forged = b"HGEMTA01".to_vec();
        forged.extend((payload.len() as u64).to_le_bytes());
        forged.extend(checksum(&payload).to_le_bytes());
        forged.extend(payload);
        assert!(FamilySimulation::resume_live(&forged, &cursor, &att).is_err());
    }
}

#[cfg(test)]
mod retained_checkpoint_tests {
    use super::*;
    #[test]
    fn retained_checkpoint_rejects_rehashed_provenance_forgery() {
        let mut s =
            FamilySimulation::new_retained_stream(retained_cases()[3].config.clone()).unwrap();
        for _ in 0..8 {
            s.advance_tick().unwrap();
            s.drain_stream_tick().unwrap();
        }
        let cursor = EvidenceCursor {
            segment_sha256: "a".repeat(64),
            tick: 8,
            step: 368,
            local_ticks: 8,
            offset: 0,
            log_sha256: "b".repeat(64),
            previous_commit_sha256: None,
        };
        let cp = s.live_checkpoint(cursor.clone()).unwrap();
        for edit in 0..6 {
            let mut x: RetainedLiveSaved = bincode::DefaultOptions::new()
                .with_fixint_encoding()
                .deserialize(&cp[24..])
                .unwrap();
            let first = x.retained.samples.get_mut(&1).unwrap();
            match edit {
                0 => first.proposed_bytes[0] ^= 1,
                1 => first.tick = 30,
                2 => first.raw_draws.push(99),
                3 => first.settled = true,
                4 => first.operator = "source".into(),
                _ => {
                    let extra = first.clone();
                    x.retained.samples.insert(3, extra);
                }
            }
            let mut att = s.live_attestation(cursor.clone()).unwrap();
            att["native"]["retained"] = serde_json::to_value(&x.retained).unwrap();
            let payload = bincode::DefaultOptions::new()
                .with_fixint_encoding()
                .serialize(&x)
                .unwrap();
            let mut forged = b"HGEMRP01".to_vec();
            forged.extend((payload.len() as u64).to_le_bytes());
            forged.extend(checksum(&payload).to_le_bytes());
            forged.extend(payload);
            assert!(
                FamilySimulation::resume_live(&forged, &cursor, &att).is_err(),
                "forgery {edit}"
            );
        }
    }
}

#[cfg(test)]
mod handoff_checkpoint_tests {
    use super::*;
    #[test]
    fn handoff_rehashed_invalid_policy_and_kernel_are_refused() {
        let mut cfg = super::super::handoff_cases()[0].config.clone();
        cfg.seed = 17;
        let h = HandoffConfig {
            policy: HANDOFF_POLICY.into(),
            supplied: SuppliedEdit::Useful,
            descendant_kernel: MutationKernel::Faithful,
        };
        let mut s = FamilySimulation::new_handoff_stream(cfg, h).unwrap();
        for _ in 0..8 {
            s.advance_tick().unwrap();
            s.drain_stream_tick().unwrap();
        }
        let cursor = EvidenceCursor {
            segment_sha256: "a".repeat(64),
            tick: 8,
            step: 368,
            local_ticks: 8,
            offset: 0,
            log_sha256: "b".repeat(64),
            previous_commit_sha256: None,
        };
        let cp = s.live_checkpoint(cursor.clone()).unwrap();
        for kernel in [false, true] {
            let mut x: HandoffLiveSaved = bincode::DefaultOptions::new()
                .with_fixint_encoding()
                .deserialize(&cp[24..])
                .unwrap();
            if kernel {
                x.handoff.descendant_kernel = MutationKernel::SourceKernel;
            } else {
                x.handoff.policy = "foreign".into();
            }
            let mut att = s.live_attestation(cursor.clone()).unwrap();
            att["native"]["handoff_config"] = serde_json::to_value(&x.handoff).unwrap();
            let payload = bincode::DefaultOptions::new()
                .with_fixint_encoding()
                .serialize(&x)
                .unwrap();
            let mut forged = b"HGEMHO01".to_vec();
            forged.extend((payload.len() as u64).to_le_bytes());
            forged.extend(checksum(&payload).to_le_bytes());
            forged.extend(payload);
            assert!(FamilySimulation::resume_live(&forged, &cursor, &att).is_err());
        }
    }
}

impl FamilySimulation {
    pub fn new_stream(recipe: AdmissionRecipe) -> Result<Self, String> {
        Self::enable_live(Self::new_admission(recipe)?)
    }
    pub fn new_funded_stream(config: FundedAdmissionConfig) -> Result<Self, String> {
        Self::enable_live(Self::new_funded_admission(config)?)
    }
    pub fn new_transfer_stream(
        config: FundedAdmissionConfig,
        environment: TransferEnvironment,
    ) -> Result<Self, String> {
        Self::enable_live(Self::new_transfer(config, environment)?)
    }
    pub fn new_mutation_stress_stream(
        config: FundedAdmissionConfig,
        environment: TransferEnvironment,
    ) -> Result<Self, String> {
        Self::enable_live(Self::new_mutation_stress(config, environment)?)
    }
    pub fn new_retained_stream(config: FundedAdmissionConfig) -> Result<Self, String> {
        Self::enable_live(Self::new_retained(config)?)
    }
    pub fn new_handoff_stream(
        config: FundedAdmissionConfig,
        handoff: HandoffConfig,
    ) -> Result<Self, String> {
        Self::enable_live(Self::new_handoff(config, handoff)?)
    }
    pub fn new_descendant_stream(
        config: FundedAdmissionConfig,
        policy: DescendantConfig,
    ) -> Result<Self, String> {
        Self::enable_live(Self::new_descendant(config, policy)?)
    }
    pub(crate) fn stream_profile(&self) -> &'static str {
        if self.mutation_stress {
            MUTATION_STRESS_STREAM_PROFILE
        } else if self.transfer_environment.is_some() {
            TRANSFER_ENV_STREAM_PROFILE
        } else if self.descendant.is_some() {
            DESCENDANT_STREAM_PROFILE
        } else if self.handoff.is_some() {
            HANDOFF_STREAM_PROFILE
        } else if self.retained.is_some() {
            RETAINED_STREAM_PROFILE
        } else if self.funded_admission.is_some() {
            FUNDED_STREAM_PROFILE
        } else {
            STREAM_PROFILE
        }
    }
    fn enable_live(mut s: Self) -> Result<Self, String> {
        s.streaming = Some(LiveAncestry {
            members: BTreeMap::from([
                (
                    1,
                    LiveAncestor {
                        founder: 1,
                        depth: 0,
                    },
                ),
                (
                    2,
                    LiveAncestor {
                        founder: 2,
                        depth: 0,
                    },
                ),
            ]),
            maximum_depth: 0,
        });
        s.validate()?;
        Ok(s)
    }
    pub fn completed_tick(&self) -> u64 {
        self.tick
    }
    pub fn drain_stream_tick(&mut self) -> Result<TickBundle, String> {
        if !self.quiescent
            || self.streaming.is_none()
            || self.journal.len() != 46
            || self.rows.len() != 1
            || self.admission.as_ref().is_none_or(|a| {
                a.rounds.len() != 1 || a.records.iter().any(|r| !r.settled || r.tick != self.tick)
            })
        {
            return Err("stream drain boundary/coverage".into());
        }
        self.validate()?;
        let lineage_delta = self
            .lineage
            .values()
            .filter(|b| b.born_tick == self.tick || b.death_tick == Some(self.tick))
            .cloned()
            .collect();
        let profile = self.stream_profile().to_string();
        let a = self.admission.as_mut().unwrap();
        let bundle = TickBundle {
            profile,
            tick: self.tick,
            entries: std::mem::take(&mut self.journal),
            row: self.rows.remove(0),
            round: a.rounds.remove(0),
            records: std::mem::take(&mut a.records),
            lineage_delta,
        };
        self.lineage.retain(|_, b| b.death_tick.is_none());
        self.streaming
            .as_mut()
            .unwrap()
            .members
            .retain(|id, _| self.lineage.contains_key(id));
        self.validate()?;
        Ok(bundle)
    }
    fn live_saved(&self, cursor: EvidenceCursor) -> Result<LiveSaved, String> {
        cursor.validate()?;
        self.validate()?;
        if !self.quiescent
            || self.streaming.is_none()
            || !self.journal.is_empty()
            || !self.rows.is_empty()
            || self
                .admission
                .as_ref()
                .is_none_or(|a| !a.rounds.is_empty() || !a.records.is_empty())
            || cursor.tick != self.tick
            || cursor.step != self.store.step()
        {
            return Err("live checkpoint drained boundary/cursor".into());
        }
        Ok(LiveSaved {
            profile: self.stream_profile().into(),
            program: self.program.clone(),
            schedule: self.schedule.clone(),
            store: self.store.clone(),
            allocator: self.store.allocation_state(),
            engine: self.engine.diagnostic_checkpoint()?,
            tick: self.tick,
            lineage: self.lineage.clone(),
            ledger: self.ledger.clone(),
            law_identity: checksum(self.law_text().as_bytes()),
            admission: self.admission.as_ref().unwrap().clone(),
            live: self.streaming.as_ref().unwrap().clone(),
            cursor,
        })
    }
    pub fn live_attestation(&self, cursor: EvidenceCursor) -> Result<serde_json::Value, String> {
        let saved = self.live_saved(cursor)?;
        let mut native = serde_json::to_value(saved).map_err(|e| e.to_string())?;
        if let Some(c) = &self.funded_admission {
            native["funded_config"] = serde_json::to_value(c).map_err(|e| e.to_string())?;
        }
        if let Some(r) = &self.retained {
            native["retained"] = serde_json::to_value(r).map_err(|e| e.to_string())?;
        }
        if let Some(h) = &self.handoff {
            native["handoff_config"] = serde_json::to_value(h).map_err(|e| e.to_string())?;
        }
        if let Some(d) = &self.descendant {
            native["descendant"] = serde_json::to_value(d).map_err(|e| e.to_string())?;
        }
        if let Some(e) = &self.transfer_environment {
            native["transfer_environment"] = serde_json::to_value(e).map_err(|e| e.to_string())?;
        }
        if self.mutation_stress {
            native["mutation_stress_policy"] = MUTATION_STRESS_POLICY.into();
        }
        Ok(serde_json::json!({"native":native,"frame":self.final_frame()}))
    }
    pub fn live_checkpoint(&self, cursor: EvidenceCursor) -> Result<Vec<u8>, String> {
        let saved = self.live_saved(cursor)?;
        let payload = if let Some(environment) = &self.transfer_environment {
            bincode::DefaultOptions::new()
                .with_fixint_encoding()
                .serialize(&TransferLiveSaved {
                    funded: FundedLiveSaved {
                        config: self.funded_admission.clone().ok_or("transfer config")?,
                        state: saved,
                    },
                    environment: environment.clone(),
                })
        } else if let Some(descendant) = &self.descendant {
            bincode::DefaultOptions::new()
                .with_fixint_encoding()
                .serialize(&DescendantLiveSaved {
                    funded: FundedLiveSaved {
                        config: self.funded_admission.clone().ok_or("descendant config")?,
                        state: saved,
                    },
                    descendant: descendant.clone(),
                })
        } else if let Some(handoff) = &self.handoff {
            bincode::DefaultOptions::new()
                .with_fixint_encoding()
                .serialize(&HandoffLiveSaved {
                    funded: FundedLiveSaved {
                        config: self.funded_admission.clone().ok_or("handoff config")?,
                        state: saved,
                    },
                    handoff: handoff.clone(),
                })
        } else if let Some(retained) = &self.retained {
            bincode::DefaultOptions::new()
                .with_fixint_encoding()
                .serialize(&RetainedLiveSaved {
                    funded: FundedLiveSaved {
                        config: self
                            .funded_admission
                            .clone()
                            .ok_or("retained configuration")?,
                        state: saved,
                    },
                    retained: retained.clone(),
                })
        } else if let Some(config) = &self.funded_admission {
            bincode::DefaultOptions::new()
                .with_fixint_encoding()
                .serialize(&FundedLiveSaved {
                    config: config.clone(),
                    state: saved,
                })
        } else {
            bincode::DefaultOptions::new()
                .with_fixint_encoding()
                .serialize(&saved)
        }
        .map_err(|e| e.to_string())?;
        if payload.len() > LIVE_LIMIT {
            return Err("live checkpoint ceiling".into());
        }
        let mut bytes = if self.mutation_stress {
            b"HGEMMS01".to_vec()
        } else if self.transfer_environment.is_some() {
            b"HGEMTA01".to_vec()
        } else if self.descendant.is_some() {
            b"HGEMDN01".to_vec()
        } else if self.handoff.is_some() {
            b"HGEMHO01".to_vec()
        } else if self.retained.is_some() {
            b"HGEMRP01".to_vec()
        } else if self.funded_admission.is_some() {
            b"HGEMFS01".to_vec()
        } else {
            b"HGEMS001".to_vec()
        };
        bytes.extend((payload.len() as u64).to_le_bytes());
        bytes.extend(checksum(&payload).to_le_bytes());
        bytes.extend(payload);
        Ok(bytes)
    }
    /// Requires an externally validated immutable commit and its bound attestation.
    /// No historical engine execution; callers verify the published prefix hashes.
    pub fn resume_live(
        bytes: &[u8],
        cursor: &EvidenceCursor,
        attestation: &serde_json::Value,
    ) -> Result<Self, String> {
        cursor.validate()?;
        if bytes.len() < 24
            || bytes.len() > LIVE_LIMIT + 24
            || (&bytes[..8] != b"HGEMS001"
                && &bytes[..8] != b"HGEMFS01"
                && &bytes[..8] != b"HGEMRP01"
                && &bytes[..8] != b"HGEMHO01"
                && &bytes[..8] != b"HGEMDN01"
                && &bytes[..8] != b"HGEMTA01"
                && &bytes[..8] != b"HGEMMS01")
            || u64::from_le_bytes(bytes[8..16].try_into().unwrap()) as usize != bytes.len() - 24
            || checksum(&bytes[24..]) != u64::from_le_bytes(bytes[16..24].try_into().unwrap())
        {
            return Err("live checkpoint envelope".into());
        }
        let mut descendant = None;
        let mut transfer_environment = None;
        let mutation_stress = &bytes[..8] == b"HGEMMS01";
        let (mut c, config, retained, handoff): (
            LiveSaved,
            Option<FundedAdmissionConfig>,
            Option<RetainedState>,
            Option<HandoffConfig>,
        ) = if &bytes[..8] == b"HGEMTA01" || mutation_stress {
            let x: TransferLiveSaved = bincode::DefaultOptions::new()
                .with_fixint_encoding()
                .with_limit(LIVE_LIMIT as u64)
                .reject_trailing_bytes()
                .deserialize(&bytes[24..])
                .map_err(|e| e.to_string())?;
            if mutation_stress {
                x.environment.validate_stress(&x.funded.config)?;
            } else {
                x.environment.validate(&x.funded.config)?;
            }
            transfer_environment = Some(x.environment);
            (x.funded.state, Some(x.funded.config), None, None)
        } else if &bytes[..8] == b"HGEMDN01" {
            let x: DescendantLiveSaved = bincode::DefaultOptions::new()
                .with_fixint_encoding()
                .with_limit(LIVE_LIMIT as u64)
                .reject_trailing_bytes()
                .deserialize(&bytes[24..])
                .map_err(|e| e.to_string())?;
            x.descendant
                .validate(&x.funded.config, x.funded.state.tick)?;
            descendant = Some(x.descendant);
            (x.funded.state, Some(x.funded.config), None, None)
        } else if &bytes[..8] == b"HGEMHO01" {
            let x: HandoffLiveSaved = bincode::DefaultOptions::new()
                .with_fixint_encoding()
                .with_limit(LIVE_LIMIT as u64)
                .reject_trailing_bytes()
                .deserialize(&bytes[24..])
                .map_err(|e| e.to_string())?;
            x.handoff.validate(&x.funded.config)?;
            (x.funded.state, Some(x.funded.config), None, Some(x.handoff))
        } else if &bytes[..8] == b"HGEMRP01" {
            let x: RetainedLiveSaved = bincode::DefaultOptions::new()
                .with_fixint_encoding()
                .with_limit(LIVE_LIMIT as u64)
                .reject_trailing_bytes()
                .deserialize(&bytes[24..])
                .map_err(|e| e.to_string())?;
            x.funded.config.validate()?;
            x.retained.validate(&x.funded.config, x.funded.state.tick)?;
            (
                x.funded.state,
                Some(x.funded.config),
                Some(x.retained),
                None,
            )
        } else if &bytes[..8] == b"HGEMFS01" {
            let x: FundedLiveSaved = bincode::DefaultOptions::new()
                .with_fixint_encoding()
                .with_limit(LIVE_LIMIT as u64)
                .reject_trailing_bytes()
                .deserialize(&bytes[24..])
                .map_err(|e| e.to_string())?;
            x.config.validate()?;
            (x.state, Some(x.config), None, None)
        } else {
            (
                bincode::DefaultOptions::new()
                    .with_fixint_encoding()
                    .with_limit(LIVE_LIMIT as u64)
                    .reject_trailing_bytes()
                    .deserialize(&bytes[24..])
                    .map_err(|e| e.to_string())?,
                None,
                None,
                None,
            )
        };
        let profile = if mutation_stress {
            MUTATION_STRESS_STREAM_PROFILE
        } else if transfer_environment.is_some() {
            TRANSFER_ENV_STREAM_PROFILE
        } else if descendant.is_some() {
            DESCENDANT_STREAM_PROFILE
        } else if handoff.is_some() {
            HANDOFF_STREAM_PROFILE
        } else if retained.is_some() {
            RETAINED_STREAM_PROFILE
        } else if config.is_some() {
            FUNDED_STREAM_PROFILE
        } else {
            STREAM_PROFILE
        };
        let seed = config.as_ref().map_or(17, |x| x.seed);
        let program = if let Some(x) = &config {
            x.programs()?[0].clone()
        } else {
            c.admission.recipe.programs()?[0].clone()
        };
        let laws = if let Some(e) = &transfer_environment {
            e.law_text(config.as_ref().ok_or("transfer checkpoint configuration")?)?
        } else if let Some(x) = &config {
            x.law_text()?
        } else {
            admission_law_text(&c.schedule)
        };
        if config
            .as_ref()
            .is_some_and(|x| x.coordinate_recipe() != c.admission.recipe)
        {
            return Err("funded checkpoint coordinates".into());
        }
        if c.profile != profile
            || c.cursor != *cursor
            || c.tick != cursor.tick
            || c.store.step() != cursor.step
            || c.schedule
                != transfer_environment.as_ref().map_or_else(
                    || ChoiceSchedule::new(seed, CueMode::Informative),
                    |e| e.schedule(seed),
                )
            || c.program != program
            || c.law_identity != checksum(laws.as_bytes())
            || !c.admission.records.is_empty()
            || !c.admission.rounds.is_empty()
            || c.lineage
                .values()
                .any(|b| b.death_tick.is_some() || b.born_tick > c.tick)
            || c.engine.source.is_some()
        {
            return Err("live checkpoint identity/state boundary".into());
        }
        c.store
            .restore_allocation_state(c.allocator.clone())
            .map_err(str::to_string)?;
        let mut s = if let Some(x) = config {
            if let Some(e) = &transfer_environment {
                if mutation_stress {
                    Self::new_mutation_stress_stream(x, e.clone())?
                } else {
                    Self::new_transfer_stream(x, e.clone())?
                }
            } else if let Some(d) = &descendant {
                Self::new_descendant_stream(x, d.config.clone())?
            } else if let Some(h) = &handoff {
                Self::new_handoff_stream(x, h.clone())?
            } else if retained.is_some() {
                Self::new_retained_stream(x)?
            } else {
                Self::new_funded_stream(x)?
            }
        } else {
            Self::new_stream(c.admission.recipe.clone())?
        };
        s.program = c.program;
        s.schedule = c.schedule;
        s.store = c.store;
        s.engine = Engine::from_diagnostic_checkpoint(c.engine, None)?;
        s.capture = Arc::new(Mutex::new(vec![]));
        s.engine.add_observer(Box::new(Capture(s.capture.clone())));
        s.tick = c.tick;
        s.retained = retained;
        s.handoff = handoff;
        s.descendant = descendant;
        s.lineage = c.lineage;
        s.ledger = c.ledger;
        s.admission = Some(c.admission);
        s.streaming = Some(c.live);
        s.validate()?;
        if s.ledger.initial != 384.0
            || total(&s.store)? != s.ledger.initial + s.ledger.influx - s.ledger.dissipated
            || s.ledger.attempts != s.ledger.successes + s.ledger.failures
            || s.live_attestation(cursor.clone())? != *attestation
        {
            return Err("live checkpoint attestation/ledger mismatch".into());
        }
        Ok(s)
    }
}

#[cfg(test)]
mod descendant_checkpoint_tests {
    use super::*;
    #[test]
    fn rehashed_descendant_receipt_forgery_is_refused() {
        let mut cfg = super::super::descendant_cases()[0].config.clone();
        cfg.seed = 17;
        let policy = DescendantConfig {
            policy: DESCENDANT_POLICY.into(),
            target_id: Some(103),
            new_source: Some(1),
        };
        let mut s = FamilySimulation::new_descendant_stream(cfg, policy).unwrap();
        for _ in 0..24 {
            s.advance_tick().unwrap();
            s.drain_stream_tick().unwrap();
        }
        let cursor = EvidenceCursor {
            segment_sha256: "a".repeat(64),
            tick: 24,
            step: 1104,
            local_ticks: 24,
            offset: 0,
            log_sha256: "b".repeat(64),
            previous_commit_sha256: None,
        };
        let cp = s.live_checkpoint(cursor.clone()).unwrap();
        let att = s.live_attestation(cursor.clone()).unwrap();
        for defect in 0..4 {
            let mut x: DescendantLiveSaved = bincode::DefaultOptions::new()
                .with_fixint_encoding()
                .deserialize(&cp[24..])
                .unwrap();
            let r = x.descendant.targets.values_mut().next().unwrap();
            match defect {
                0 => r.tick = 25,
                1 => r.raw_draws.push(7),
                2 => r.proposed_bytes[0] = 255,
                _ => x.descendant.config.policy = "unknown".into(),
            }
            let mut bad = att.clone();
            bad["native"]["descendant"] = serde_json::to_value(&x.descendant).unwrap();
            let payload = bincode::DefaultOptions::new()
                .with_fixint_encoding()
                .serialize(&x)
                .unwrap();
            let mut bytes = b"HGEMDN01".to_vec();
            bytes.extend((payload.len() as u64).to_le_bytes());
            bytes.extend(checksum(&payload).to_le_bytes());
            bytes.extend(payload);
            assert!(FamilySimulation::resume_live(&bytes, &cursor, &bad).is_err());
        }
    }
}
