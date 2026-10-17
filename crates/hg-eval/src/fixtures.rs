//! The six canonical fixtures (docs/fixture_definitions.md) and the truth
//! table. Fixtures are window *sequences over evolving stores* — node
//! identity across windows is what distinguishes static from active
//! persistence, so fixtures that "move" must genuinely delete and recreate
//! nodes in one store.

pub mod locality;

use crate::classify::{classify, interestingness};
use crate::window::WindowComputer;
use hg_core::{HyperEdge, HypergraphStore, Label, NodeData, NodeId};
use hg_schema::Regime;
use smallvec::SmallVec;

pub const FIXTURE_WINDOWS: usize = 10;

pub trait Fixture {
    fn name(&self) -> &'static str;
    /// The sequence of per-window stores (>= FIXTURE_WINDOWS).
    fn stores(&self) -> Vec<HypergraphStore>;
    /// Acceptable regimes for this fixture.
    fn expected(&self) -> &'static [Regime];
}

pub struct FixtureOutcome {
    pub name: &'static str,
    /// Majority regime over non-warmup windows (window 0 is Unknown by
    /// definition; early growth ratios are also warm-up artifacts).
    pub regime: Regime,
    pub per_window: Vec<Regime>,
    pub mean_interestingness: f64,
    pub passed: bool,
}

/// Run the evaluator over a fixture exactly as it runs on live output.
pub fn run_fixture(fixture: &dyn Fixture) -> FixtureOutcome {
    let stores = fixture.stores();
    let mut computer = WindowComputer::new(1);
    let mut regimes = Vec::new();
    let mut scores = Vec::new();
    for store in &stores {
        let snapshot = store.snapshot();
        let f = computer.observe(&snapshot);
        regimes.push(classify(&f));
        scores.push(interestingness(&f));
    }
    // Majority vote over the stable tail (skip the first 3 warm-up windows).
    let tail = &regimes[3.min(regimes.len() - 1)..];
    let mut counts: ahash::AHashMap<Regime, usize> = ahash::AHashMap::new();
    for r in tail {
        *counts.entry(*r).or_insert(0) += 1;
    }
    let majority = counts
        .into_iter()
        .max_by_key(|(_, c)| *c)
        .map(|(r, _)| r)
        .unwrap_or(Regime::Unknown);
    let mean_score = scores.iter().sum::<f64>() / scores.len().max(1) as f64;
    FixtureOutcome {
        name: fixture.name(),
        regime: majority,
        per_window: regimes,
        mean_interestingness: mean_score,
        passed: fixture.expected().contains(&majority),
    }
}

fn add_triangle(store: &mut HypergraphStore, label: u32) -> [NodeId; 3] {
    let a = store.add_node(NodeData::new(Label(label), store.step()));
    let b = store.add_node(NodeData::new(Label(label), store.step()));
    let c = store.add_node(NodeData::new(Label(label), store.step()));
    store.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[a, b, c])));
    [a, b, c]
}

/// Fixture 1: FrozenGrid — a 10×10 grid, identical every window.
pub struct FrozenGrid;
impl Fixture for FrozenGrid {
    fn name(&self) -> &'static str {
        "FrozenGrid"
    }
    fn stores(&self) -> Vec<HypergraphStore> {
        let mut base = HypergraphStore::new();
        hg_core::generate_crystal(&mut base, 10, 10, 2, false, 0);
        vec![base; FIXTURE_WINDOWS]
    }
    fn expected(&self) -> &'static [Regime] {
        &[Regime::Frozen]
    }
}

/// Fixture 2: UniformGrowthToy — one same-typed triangle added per window.
pub struct UniformGrowthToy;
impl Fixture for UniformGrowthToy {
    fn name(&self) -> &'static str {
        "UniformGrowthToy"
    }
    fn stores(&self) -> Vec<HypergraphStore> {
        let mut store = HypergraphStore::new();
        add_triangle(&mut store, 0);
        let mut stores = vec![store.clone()];
        for _ in 1..FIXTURE_WINDOWS {
            add_triangle(&mut store, 0);
            stores.push(store.clone());
        }
        stores
    }
    fn expected(&self) -> &'static [Regime] {
        &[Regime::GrowthTreadmill]
    }
}

/// Fixture 3: MovingGliderToy — a 3-node chain translating through one
/// store: trailing node deleted, leading node created, WL identity kept.
pub struct MovingGliderToy;
impl Fixture for MovingGliderToy {
    fn name(&self) -> &'static str {
        "MovingGliderToy"
    }
    fn stores(&self) -> Vec<HypergraphStore> {
        let mut store = HypergraphStore::new();
        let mut chain: Vec<NodeId> = (0..3)
            .map(|_| store.add_node(NodeData::new(Label(0), 0)))
            .collect();
        store.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[chain[0], chain[1]])));
        store.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[chain[1], chain[2]])));
        let mut stores = vec![store.clone()];
        for _ in 1..FIXTURE_WINDOWS {
            // Shift: delete trailing node (dangling cleanup removes its
            // edge), create a new leading node + edge.
            let tail = chain.remove(0);
            store.remove_node(tail);
            let head = *chain.last().unwrap();
            let new_head = store.add_node(NodeData::new(Label(0), 0));
            store.add_edge(HyperEdge::new(
                Label(0),
                SmallVec::from_slice(&[head, new_head]),
            ));
            chain.push(new_head);
            stores.push(store.clone());
        }
        stores
    }
    fn expected(&self) -> &'static [Regime] {
        &[Regime::ActivePersistentStructure, Regime::GenerativeCandidate]
    }
}

/// Fixture 4: StaticBlockToy — a K4 that never changes.
pub struct StaticBlockToy;
impl Fixture for StaticBlockToy {
    fn name(&self) -> &'static str {
        "StaticBlockToy"
    }
    fn stores(&self) -> Vec<HypergraphStore> {
        let mut store = HypergraphStore::new();
        let n: Vec<NodeId> = (0..4)
            .map(|_| store.add_node(NodeData::new(Label(0), 0)))
            .collect();
        for i in 0..4 {
            for j in (i + 1)..4 {
                store.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[n[i], n[j]])));
            }
        }
        vec![store; FIXTURE_WINDOWS]
    }
    fn expected(&self) -> &'static [Regime] {
        &[Regime::StaticOrdered]
    }
}

/// Fixture 5: WhiteNoiseToy — a fresh random graph every window.
pub struct WhiteNoiseToy;
impl Fixture for WhiteNoiseToy {
    fn name(&self) -> &'static str {
        "WhiteNoiseToy"
    }
    fn stores(&self) -> Vec<HypergraphStore> {
        (0..FIXTURE_WINDOWS as u64)
            .map(|w| {
                let mut store = HypergraphStore::new();
                hg_core::generate_white_noise(&mut store, 50, 70, 3, 10, 1000 + w);
                store
            })
            .collect()
    }
    fn expected(&self) -> &'static [Regime] {
        &[Regime::WhiteNoiseSoup, Regime::SterileChaos]
    }
}

/// Fixture 6: SlowGenerativeGrowth — one triangle per window with a fresh
/// label each time: novelty every window, NOT a treadmill.
pub struct SlowGenerativeGrowth;
impl Fixture for SlowGenerativeGrowth {
    fn name(&self) -> &'static str {
        "SlowGenerativeGrowth"
    }
    fn stores(&self) -> Vec<HypergraphStore> {
        let mut store = HypergraphStore::new();
        add_triangle(&mut store, 0);
        let mut stores = vec![store.clone()];
        for w in 1..FIXTURE_WINDOWS as u32 {
            add_triangle(&mut store, w);
            stores.push(store.clone());
        }
        stores
    }
    fn expected(&self) -> &'static [Regime] {
        &[Regime::GenerativeCandidate]
    }
}

/// Run all six fixtures; the fixture gate passes iff all pass.
pub fn run_truth_table() -> Vec<FixtureOutcome> {
    vec![
        run_fixture(&FrozenGrid),
        run_fixture(&UniformGrowthToy),
        run_fixture(&MovingGliderToy),
        run_fixture(&StaticBlockToy),
        run_fixture(&WhiteNoiseToy),
        run_fixture(&SlowGenerativeGrowth),
    ]
}

pub fn fixture_gate_status() -> hg_schema::GateStatus {
    if run_truth_table().iter().all(|o| o.passed) {
        hg_schema::GateStatus::Pass
    } else {
        hg_schema::GateStatus::Fail
    }
}
