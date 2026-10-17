//! Bounded native arbitration diagnostics; never substituted into an ecology.
use super::{
    config::*,
    d0::*,
    family::{GENOME_LENGTH, INTERNAL_PARTS},
    runtime::{float, int, Capture, Frame},
};
use hg_core::{HyperEdge, HypergraphStore, Label, NodeData, NodeId, PropertyValue};
use hg_engine::Engine;
use hg_observe::{Binding, StepStats};
use hg_rules::{compile_rules, parse_rules_toml, MatchBudgetPolicy, StepMode};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, Mutex},
};
pub const ARBITRATION_PROFILE: &str = "evolving-minds-d0-birth-arbitration-v1";
pub const ARBITRATION_LAWS: &str =
    include_str!("../../../../rules/evolving_minds/arbitration_v1.toml");
pub const OFFERED: u32 = 242017;
pub const LOCAL_NEXT: u32 = 242018;
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum BirthPolicy {
    Global,
    Local,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum OfferTopology {
    Disjoint,
    Competing,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArbitrationRecipe {
    pub policy: BirthPolicy,
    pub topology: OfferTopology,
    pub rotation: u8,
    pub reversed: bool,
}
impl ArbitrationRecipe {
    pub fn name(&self) -> String {
        format!(
            "{:?}_{:?}_r{}_{}",
            self.policy,
            self.topology,
            self.rotation,
            if self.reversed { "reverse" } else { "forward" }
        )
        .to_lowercase()
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BirthOffer {
    pub parent: i64,
    pub target: i64,
    pub bound: Vec<NodeId>,
    pub canonical_key: Vec<u64>,
    pub physical_conflicts: Vec<i64>,
    pub counter_only_conflicts: Vec<i64>,
    pub accepted: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArbitrationResult {
    pub profile: String,
    pub recipe: ArbitrationRecipe,
    pub frames: Vec<Frame>,
    pub offers: Vec<BirthOffer>,
    pub accepted_parents: Vec<i64>,
    pub child_ids: Vec<i64>,
    pub initial_energy: f64,
    pub warmup_dissipation: f64,
    pub copying_dissipation: f64,
    pub final_energy: f64,
    pub residual: f64,
}
fn props(label: u32, ps: Vec<(u32, PropertyValue)>) -> NodeData {
    let mut n = NodeData::new(Label(label), 0);
    for (k, v) in ps {
        n.set_property(k, v)
    }
    n
}
fn relation(s: &mut HypergraphStore, label: u32, a: NodeId, b: NodeId) {
    s.add_edge(HyperEdge::new(Label(label), [a, b].into_iter().collect()));
}
fn scaffold(recipe: &ArbitrationRecipe) -> Result<HypergraphStore, String> {
    if recipe.rotation > 7 {
        return Err("arbitration rotation".into());
    }
    use PropertyValue::{Bool as B, Float as F, Int as I};
    let mut s = HypergraphStore::new();
    let rot = recipe.rotation as usize;
    let homes = [
        rot,
        (rot + if recipe.topology == OfferTopology::Disjoint {
            4
        } else {
            2
        }) % 8,
    ];
    let targets = if recipe.topology == OfferTopology::Disjoint {
        vec![(rot + 1) % 8, (rot + 5) % 8]
    } else {
        vec![(rot + 1) % 8]
    };
    let mut sites = vec![];
    for site in 0..8 {
        let mut ps = vec![
            (ENERGY, F(32.0)),
            (241001, I(0)),
            (241009, I(site)),
            (241002, B(false)),
            (241003, B(false)),
            (241004, B(false)),
            (OCCUPIED, B(homes.contains(&(site as usize)))),
            (OFFERED, B(targets.contains(&(site as usize)))),
        ];
        if recipe.policy == BirthPolicy::Local {
            ps.push((LOCAL_NEXT, I(3 + site)))
        }
        sites.push(s.add_node(props(SITE, ps)));
    }
    for i in 0..8 {
        relation(&mut s, NEIGHBOR, sites[i], sites[(i + 1) % 8]);
    }
    if recipe.policy == BirthPolicy::Global {
        s.add_node(props(COUNTER, vec![(NEXT_ID, I(3))]));
    }
    for (i, home) in homes.into_iter().enumerate() {
        let bid = i as i64 + 1;
        let root = s.add_node(props(
            ROOT,
            vec![
                (ENERGY, F(64.0)),
                (BIRTH_ID, I(bid)),
                (PARENT_ID, I(0)),
                (AGE, I(0)),
                (CURSOR, I(0)),
                (STATUS, I(0)),
                (HAS_ACTUATOR, B(false)),
                (241007, B(false)),
                (241008, B(false)),
                (GENOME_LENGTH, I(1)),
                (INTERNAL_PARTS, I(11)),
                (MAINTENANCE, F(2.125)),
            ],
        ));
        relation(&mut s, OCCUPANCY, root, sites[home]);
        let mut slots = vec![];
        for slot in 0..8 {
            let n = s.add_node(props(
                SLOT,
                vec![(OWNER, I(bid)), (DEST, I(slot)), (USED, B(slot < 2))],
            ));
            relation(&mut s, OWN_PART, root, n);
            slots.push(n)
        }
        for slot in 0..2 {
            let n = s.add_node(props(
                SENSOR,
                vec![(OWNER, I(bid)), (DEST, I(slot)), (SIGNAL, B(false))],
            ));
            relation(&mut s, OWN_PART, root, n);
            relation(&mut s, SLOT_MAP, slots[slot as usize], n)
        }
        let g = s.add_node(props(
            GENE,
            vec![
                (OWNER, I(bid)),
                (POSITION, I(0)),
                (INSTRUCTION_ID, I(100)),
                (OPCODE, I(3)),
                (DEST, I(0)),
                (SRC, I(0)),
                (INITIAL, B(false)),
            ],
        ));
        relation(&mut s, OWN_PART, root, g);
    }
    if recipe.reversed {
        let mut other = HypergraphStore::new();
        let mut ns: Vec<_> = s.nodes().map(|(id, n)| (id, n.clone())).collect();
        ns.reverse();
        let mut map = BTreeMap::new();
        for (h, n) in ns {
            map.insert(h, other.add_node(n));
        }
        let mut es: Vec<_> = s.edges().map(|(_, e)| e.clone()).collect();
        es.reverse();
        for e in es {
            other.add_edge(HyperEdge::new(
                e.label,
                e.nodes.iter().map(|h| map[h]).collect(),
            ));
        }
        s = other;
    }
    Ok(s)
}
fn energy(s: &HypergraphStore) -> Result<f64, String> {
    s.nodes()
        .filter(|(_, n)| [ROOT, SITE].contains(&n.label.0))
        .map(|(_, n)| float(n, ENERGY))
        .sum()
}
fn strict_material(s: &HypergraphStore, recipe: &ArbitrationRecipe) -> Result<(), String> {
    if s.node_count() > 128 || s.edge_count() > 384 {
        return Err("arbitration material ceiling".into());
    }
    let mut roots = BTreeMap::new();
    let mut sites = BTreeMap::new();
    let mut counters = vec![];
    let mut owned: BTreeMap<i64, Vec<(NodeId, &NodeData)>> = BTreeMap::new();
    for (h, n) in s.nodes() {
        if n.energy != 0.0 || n.generation > s.step() {
            return Err("arbitration energy/generation".into());
        }
        let keys: Vec<u32> = match n.label.0 {
            ROOT => vec![
                ENERGY,
                BIRTH_ID,
                PARENT_ID,
                AGE,
                CURSOR,
                STATUS,
                HAS_ACTUATOR,
                241007,
                241008,
                GENOME_LENGTH,
                INTERNAL_PARTS,
                MAINTENANCE,
            ],
            SITE => {
                let mut v = vec![
                    ENERGY, 241001, 241009, 241002, 241003, 241004, OCCUPIED, OFFERED,
                ];
                if recipe.policy == BirthPolicy::Local {
                    v.push(LOCAL_NEXT)
                }
                v
            }
            COUNTER => vec![NEXT_ID],
            SLOT => vec![OWNER, DEST, USED],
            SENSOR => vec![OWNER, DEST, SIGNAL],
            GENE => vec![OWNER, POSITION, INSTRUCTION_ID, OPCODE, DEST, SRC, INITIAL],
            _ => return Err("arbitration label".into()),
        };
        if n.properties.len() != keys.len()
            || n.properties
                .iter()
                .map(|(k, _)| *k)
                .collect::<BTreeSet<_>>()
                != keys.into_iter().collect()
        {
            return Err("arbitration property schema".into());
        }
        match n.label.0 {
            ROOT => {
                if roots.insert(int(n, BIRTH_ID)?, (h, n)).is_some() || float(n, ENERGY)? < 0.0 {
                    return Err("arbitration root identity/reserve".into());
                }
            }
            SITE => {
                if sites.insert(int(n, 241009)?, (h, n)).is_some() || float(n, ENERGY)? != 32.0 {
                    return Err("arbitration site identity/reserve".into());
                }
            }
            COUNTER => counters.push(n),
            _ => owned.entry(int(n, OWNER)?).or_default().push((h, n)),
        }
    }
    if sites.len() != 8
        || owned.keys().any(|id| !roots.contains_key(id))
        || counters.len() != usize::from(recipe.policy == BirthPolicy::Global)
    {
        return Err("arbitration scaffold".into());
    }
    let mut expected = BTreeSet::new();
    let mut occupied = BTreeSet::new();
    let edge = |l: u32, a: NodeId, b: NodeId| (l, BTreeSet::from([a, b]));
    for i in 0..8 {
        expected.insert(edge(NEIGHBOR, sites[&i].0, sites[&((i + 1) % 8)].0));
    }
    for (&bid, &(h, r)) in &roots {
        let members = owned.get(&bid).ok_or("arbitration common seed missing")?;
        if members.len() != 11
            || int(r, GENOME_LENGTH)? != 1
            || int(r, INTERNAL_PARTS)? != 11
            || float(r, MAINTENANCE)? != 2.125
        {
            return Err("arbitration seed price".into());
        }
        let mut slots = BTreeMap::new();
        let mut sensors = BTreeMap::new();
        let mut genes = vec![];
        for &(p, n) in members {
            expected.insert(edge(OWN_PART, h, p));
            match n.label.0 {
                SLOT => {
                    slots.insert(int(n, DEST)?, (p, n));
                }
                SENSOR => {
                    sensors.insert(int(n, DEST)?, (p, n));
                }
                GENE => genes.push(n),
                _ => return Err("arbitration seed role".into()),
            }
        }
        if slots.keys().copied().collect::<Vec<_>>() != (0..8).collect::<Vec<_>>()
            || sensors.keys().copied().collect::<Vec<_>>() != vec![0, 1]
            || genes.len() != 1
        {
            return Err("arbitration seed slots/genes".into());
        }
        for i in 0..8 {
            if slots[&i].1.get_property(USED) != Some(PropertyValue::Bool(i < 2)) {
                return Err("arbitration used flag".into());
            }
        }
        for i in 0..2 {
            if sensors[&i].1.get_property(SIGNAL) != Some(PropertyValue::Bool(false)) {
                return Err("arbitration sensor reset".into());
            }
            expected.insert(edge(SLOT_MAP, slots[&i].0, sensors[&i].0));
        }
        for (k, v) in [
            (POSITION, 0),
            (INSTRUCTION_ID, 100),
            (OPCODE, 3),
            (DEST, 0),
            (SRC, 0),
        ] {
            if int(genes[0], k)? != v {
                return Err("arbitration genotype changed".into());
            }
        }
        if genes[0].get_property(INITIAL) != Some(PropertyValue::Bool(false)) {
            return Err("arbitration genotype Bool".into());
        }
        let occ: Vec<_> = s
            .edges()
            .filter(|(_, e)| e.label.0 == OCCUPANCY && e.nodes.contains(&h))
            .collect();
        if occ.len() != 1 {
            return Err("arbitration root occupancy".into());
        }
        let site = *occ[0]
            .1
            .nodes
            .iter()
            .find(|v| **v != h)
            .ok_or("arbitration self occupancy")?;
        let slot = int(s.get_node(site).ok_or("arbitration site")?, 241009)?;
        if !occupied.insert(slot) {
            return Err("arbitration occupied collision".into());
        }
        expected.insert(edge(OCCUPANCY, h, site));
        let child = bid > 2;
        if int(r, PARENT_ID)? != 0 && !roots.contains_key(&int(r, PARENT_ID)?) {
            return Err("arbitration parent identity".into());
        }
        for key in [HAS_ACTUATOR, 241007, 241008] {
            if r.get_property(key) != Some(PropertyValue::Bool(false)) {
                return Err("arbitration child action reset".into());
            }
        }
        if child {
            if int(r, AGE)? != 0
                || int(r, CURSOR)? != 0
                || int(r, STATUS)? != 0
                || float(r, ENERGY)? != 12.0
            {
                return Err("arbitration child runtime reset".into());
            }
        } else if !(0..=2).contains(&int(r, AGE)?)
            || ![(0, 0), (1, 2)].contains(&(int(r, CURSOR)?, int(r, STATUS)?))
        {
            return Err("arbitration parent development".into());
        }
    }
    for (&i, &(_, site)) in &sites {
        if site.get_property(OCCUPIED) != Some(PropertyValue::Bool(occupied.contains(&i))) {
            return Err("arbitration occupancy flag".into());
        }
    }
    if recipe.policy == BirthPolicy::Global && int(counters[0], NEXT_ID)? != roots.len() as i64 + 1
    {
        return Err("arbitration global frontier".into());
    }
    let actual: BTreeSet<_> = s
        .edges()
        .map(|(_, e)| (e.label.0, e.nodes.iter().copied().collect::<BTreeSet<_>>()))
        .collect();
    if actual != expected || actual.len() != s.edge_count() {
        return Err("arbitration relation inventory".into());
    }
    Ok(())
}
pub fn run_arbitration(recipe: ArbitrationRecipe) -> Result<ArbitrationResult, String> {
    let mut store = scaffold(&recipe)?;
    strict_material(&store, &recipe)?;
    let initial_energy = energy(&store)?;
    let laws = parse_rules_toml(ARBITRATION_LAWS).map_err(|e| e.to_string())?;
    let compiled = compile_rules(&laws).map_err(|e| e.to_string())?;
    let mut engine = Engine::new(vec![], StepMode::MaxParallel, 17, 1_000_000);
    engine.set_match_budget_policy(1_000_000, 100_000, MatchBudgetPolicy::Pooled);
    let capture = Arc::new(Mutex::new(vec![]));
    engine.add_observer(Box::new(Capture(capture.clone())));
    let mut frames = vec![Frame::new(&store, 0, None, vec![], StepStats::default(), 0)];
    let mut offers = vec![];
    let mut accepted = vec![];
    let mut child_ids = vec![];
    let birth = if recipe.policy == BirthPolicy::Global {
        "birth_global"
    } else {
        "birth_local"
    };
    for name in [
        "development_halt",
        "maintenance_full",
        "maintenance_full",
        birth,
    ] {
        if name == birth {
            let sites: BTreeMap<_, _> = store
                .nodes()
                .filter(|(_, n)| n.label.0 == SITE)
                .map(|(h, n)| (int(n, 241009).unwrap(), h))
                .collect();
            let counter = store
                .nodes()
                .find(|(_, n)| n.label.0 == COUNTER)
                .map(|(h, _)| h);
            let pattern = &laws.iter().find(|r| r.name == birth).unwrap().lhs;
            for (root, r) in store.nodes().filter(|(_, n)| n.label.0 == ROOT) {
                let parent = int(r, BIRTH_ID)?;
                let home = store
                    .edges()
                    .filter(|(_, e)| e.label.0 == OCCUPANCY && e.nodes.contains(&root))
                    .flat_map(|(_, e)| e.nodes.iter())
                    .find(|h| **h != root)
                    .copied()
                    .unwrap();
                let gene = store
                    .nodes()
                    .find(|(_, n)| n.label.0 == GENE && int(n, OWNER).ok() == Some(parent))
                    .unwrap()
                    .0;
                for (&slot, &target) in &sites {
                    let n = store.get_node(target).unwrap();
                    if n.get_property(OFFERED) != Some(PropertyValue::Bool(true))
                        || n.get_property(OCCUPIED) != Some(PropertyValue::Bool(false))
                        || !store.edges().any(|(_, e)| {
                            e.label.0 == NEIGHBOR
                                && e.nodes.contains(&home)
                                && e.nodes.contains(&target)
                        })
                    {
                        continue;
                    }
                    let mut m = BTreeMap::from([
                        ("r", root),
                        ("site", home),
                        ("target", target),
                        ("g0", gene),
                    ]);
                    if let Some(c) = counter {
                        m.insert("counter", c);
                    }
                    let bound: Vec<_> = pattern.nodes.iter().map(|v| m[v.as_str()]).collect();
                    let canonical_key = Binding::new(0, 0, bound.iter().copied().collect())
                        .node_ids_sorted()
                        .to_vec();
                    offers.push(BirthOffer {
                        parent,
                        target: slot,
                        bound,
                        canonical_key,
                        physical_conflicts: vec![],
                        counter_only_conflicts: vec![],
                        accepted: false,
                    });
                }
            }
            for i in 0..offers.len() {
                for j in 0..offers.len() {
                    if i == j {
                        continue;
                    }
                    let common: Vec<_> = offers[i]
                        .bound
                        .iter()
                        .filter(|h| offers[j].bound.contains(h))
                        .collect();
                    let physical = common.iter().any(|h| Some(**h) != counter);
                    if physical {
                        {
                            let p = offers[j].parent;
                            offers[i].physical_conflicts.push(p)
                        }
                    } else if !common.is_empty() {
                        {
                            let p = offers[j].parent;
                            offers[i].counter_only_conflicts.push(p)
                        }
                    }
                }
            }
        }
        engine.replace_static_rules(
            compiled
                .iter()
                .filter(|r| r.name == name)
                .cloned()
                .collect(),
        );
        capture.lock().unwrap().clear();
        let stats = engine.step(&mut store);
        if engine.match_ceiling_binding() || stats.property_effects_skipped != 0 {
            return Err("arbitration matcher/effect failure".into());
        }
        let events = std::mem::take(&mut *capture.lock().unwrap());
        if name == birth {
            for e in &events {
                let parent = int(store.get_node(e.bound[0]).unwrap(), BIRTH_ID)?;
                accepted.push(parent);
                let offer = offers
                    .iter_mut()
                    .find(|o| o.bound == e.bound)
                    .ok_or("arbitration unrecorded birth")?;
                offer.accepted = true;
                let child = e
                    .created
                    .iter()
                    .filter_map(|h| store.get_node(*h))
                    .find(|n| n.label.0 == ROOT)
                    .ok_or("arbitration child")?;
                child_ids.push(int(child, BIRTH_ID)?);
            }
        }
        strict_material(&store, &recipe)?;
        frames.push(Frame::new(
            &store,
            0,
            None,
            events,
            stats,
            engine.match_budget_usage().work,
        ));
    }
    let copying_dissipation = 4.0 * accepted.len() as f64;
    let final_energy = energy(&store)?;
    let residual = final_energy - (initial_energy - 10.5 - copying_dissipation);
    if offers.len() != 2
        || residual != 0.0
        || accepted.len()
            != if recipe.policy == BirthPolicy::Local && recipe.topology == OfferTopology::Disjoint
            {
                2
            } else {
                1
            }
    {
        return Err("arbitration coverage/conservation/calibration".into());
    }
    Ok(ArbitrationResult {
        profile: ARBITRATION_PROFILE.into(),
        recipe,
        frames,
        offers,
        accepted_parents: accepted,
        child_ids,
        initial_energy,
        warmup_dissipation: 10.5,
        copying_dissipation,
        final_energy,
        residual,
    })
}
