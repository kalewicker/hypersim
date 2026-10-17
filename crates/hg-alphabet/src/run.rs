//! The measurement harness: drive a bare ruleset and accumulate its realised
//! event alphabet.
//!
//! Zero biology (A-track binding rule 4). The loop here is the ordinary engine
//! step loop; the only addition is that the WL class map and degree map of
//! `G_t` are captured *before* the step, and the same maps of `G_{t+1}` after
//! it — with the post-step maps carried forward as the next step's pre-step
//! maps, so each step boundary is computed once, not twice.

use crate::alphabet::{RunAlphabets, StopReason};
use crate::signature::{class_maps, degrees, signatures, ClassMap, DegreeMap, N_RADII};
use hg_core::{HypergraphStore, Label, NodeData, NodeId, PropertyValue};
use hg_engine::Engine;
use hg_observe::{Observer, RewriteEvent, StepStats};
use hg_rules::{compile_rules, parse_rules_toml, StepMode};
use std::sync::{Arc, Mutex};

/// Consecutive event-free steps after which a run is declared quiescent.
pub const QUIESCENCE_STEPS: u64 = 8;

/// Substrate property keys used by the bounded-churn ruleset. These are the
/// same numeric keys the H1 characterisation used, kept identical so the
/// substrate is byte-comparable with the characterised one — but nothing here
/// reads or writes them except the churn rules themselves, which are plain
/// rewrite rules. WL identity excludes the properties map (the Q2 invariant),
/// so the churn substrate's WL structure moves only through topology.
pub const KEY_FILL: u32 = 1;
pub const KEY_AGE: u32 = 21;
pub const KEY_OCCUPIED: u32 = 22;

#[derive(Clone, Debug)]
pub enum Init {
    PureCrystal {
        w: usize,
        h: usize,
        arity: usize,
        /// Mark the first node label 1 — required by rules whose LHS needs a
        /// seed (growth_treadmill).
        seed_label1: bool,
    },
    WhiteNoise {
        nodes: usize,
        edges: usize,
        max_arity: usize,
        label_range: u32,
        seed: u64,
        seed_label1: bool,
    },
    GliderRing {
        n: usize,
    },
    /// A single K4 (4 nodes, all 6 pairwise 2-edges) — the smallest structure
    /// `tetrahedral` can match. The crystal generators produce triangles, not
    /// tetrahedra, so without this the rule would never fire.
    K4,
    /// Crystal seeded for the bounded-churn ruleset.
    ChurnCrystal {
        w: usize,
        h: usize,
        arity: usize,
        fill: f64,
    },
    /// **B-track composite substrate**: a `ChurnCrystal` (2-edge triangle
    /// lattice, property-seeded) PLUS a disjoint `GliderRing` (3-edges).
    ///
    /// It exists because no single shipped initial condition hosts five
    /// size-bounded rules. `generate_crystal` emits only 2-node edges at every
    /// arity, so `glider` — the zoo's only perpetual, node-count-neutral,
    /// single-rule dynamic — cannot fire on a crystal at all; and the crystal
    /// rules (`churn`, `transfer`, …) need 2-edges the ring does not have.
    /// Joining them gives one fixed substrate on which a nested 1/2/3/5 rule
    /// series is expressible with node count flat throughout.
    ///
    /// **The components are disjoint, and that is a stated limitation**: ring
    /// rules and crystal rules cannot interact, so any super-linear composition
    /// effect must come from the four crystal-resident rules. `influx` is the
    /// one rule spanning both components.
    CompositeChurnRing {
        w: usize,
        h: usize,
        arity: usize,
        fill: f64,
        ring: usize,
    },
    /// **C-track**: a crystal whose node labels are drawn uniformly from
    /// `0..states`. Required by `cyclic3` for mechanism, not for tuning — on a
    /// uniform crystal no node has a neighbour one state ahead, so nothing
    /// fires. The labelling is the initial condition of a cyclic cellular
    /// automaton, exactly as in the literature the rule comes from.
    LabelledCrystal {
        w: usize,
        h: usize,
        arity: usize,
        states: u32,
        seed: u64,
        /// Periodic (torus) boundary. An OPEN boundary is a sink: waves
        /// propagate off the edge and the lattice freezes, which is what killed
        /// `cyclic3` at the C2 gate on its pre-registered 6×6 open substrate.
        periodic: bool,
    },
}

impl Init {
    pub fn build(&self) -> HypergraphStore {
        let mut store = HypergraphStore::new();
        match *self {
            Init::PureCrystal {
                w,
                h,
                arity,
                seed_label1,
            } => {
                hg_core::generate_crystal(&mut store, w, h, arity, false, 1);
                if seed_label1 {
                    mark_first(&mut store);
                }
            }
            Init::WhiteNoise {
                nodes,
                edges,
                max_arity,
                label_range,
                seed,
                seed_label1,
            } => {
                hg_core::generate_white_noise(&mut store, nodes, edges, max_arity, label_range, seed);
                if seed_label1 {
                    mark_first(&mut store);
                }
            }
            Init::GliderRing { n } => hg_core::generate_glider_ring(&mut store, n),
            Init::K4 => {
                let ids: Vec<NodeId> = (0..4)
                    .map(|_| store.add_node(NodeData::new(Label(0), 0)))
                    .collect();
                for i in 0..4 {
                    for j in (i + 1)..4 {
                        store.add_edge(hg_core::HyperEdge::new(
                            Label(0),
                            smallvec::SmallVec::from_slice(&[ids[i], ids[j]]),
                        ));
                    }
                }
            }
            Init::ChurnCrystal { w, h, arity, fill } => {
                hg_core::generate_crystal(&mut store, w, h, arity, false, 1);
                seed_churn_properties(&mut store, fill);
            }
            Init::CompositeChurnRing {
                w,
                h,
                arity,
                fill,
                ring,
            } => {
                hg_core::generate_crystal(&mut store, w, h, arity, false, 1);
                hg_core::generate_glider_ring(&mut store, ring);
                // Property-seed EVERY node, ring included: `influx` is the one
                // rule that spans both components and it guards on key 1.
                seed_churn_properties(&mut store, fill);
                // …except the ring's mark, which `generate_glider_ring` set to
                // label 1 and which must survive: re-marking is what keeps the
                // glider dynamic alive.
            }
            Init::LabelledCrystal {
                w,
                h,
                arity,
                states,
                seed,
                periodic,
            } => {
                use rand::{Rng, SeedableRng};
                hg_core::generate_crystal(&mut store, w, h, arity, periodic, 1);
                let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
                let ids: Vec<NodeId> = store.nodes().map(|(id, _)| id).collect();
                for id in ids {
                    let l = rng.gen_range(0..states.max(1));
                    store.get_node_mut(id).expect("just enumerated").label = Label(l);
                }
            }
        }
        store
    }
}

fn seed_churn_properties(store: &mut HypergraphStore, fill: f64) {
    let ids: Vec<NodeId> = store.nodes().map(|(id, _)| id).collect();
    for id in ids {
        let d = store.get_node_mut(id).expect("just enumerated");
        d.set_property(KEY_FILL, PropertyValue::Float(fill));
        d.set_property(KEY_AGE, PropertyValue::Float(0.0));
        d.set_property(KEY_OCCUPIED, PropertyValue::Bool(false));
    }
}

fn mark_first(store: &mut HypergraphStore) {
    let first = store.nodes().next().map(|(id, _)| id);
    if let Some(id) = first {
        store.get_node_mut(id).expect("just enumerated").label = Label(1);
    }
}

/// One measurement run, fully specified. Budgets are committed in the
/// pre-registration §7; the binding one is recorded as the stop reason.
#[derive(Clone, Debug)]
pub struct RunSpec {
    pub name: String,
    pub rules_toml: String,
    pub init: Init,
    pub step_mode: StepMode,
    pub max_steps: u64,
    /// Stop once the graph reaches this many nodes. `usize::MAX` = no cap.
    pub node_cap: usize,
    pub seed: u64,
    pub max_pairings_per_step: usize,
    /// Steps discarded from the post-burn-in accounting. `0` = no burn-in
    /// variant is produced.
    pub burn_in_steps: u64,
    /// Stop after exactly this many accepted rewrites. `u64::MAX` = no cap
    /// (the A-track default, so A-track specs are unaffected).
    ///
    /// Events are truncated EXACTLY at the cap rather than at the end of the
    /// step that crosses it: rows are compared at equal `n_events`, and
    /// `|A| ≤ n_events`, so a row that overshot by a partial step would carry a
    /// larger alphabet for no structural reason.
    pub event_cap: u64,
}

impl RunSpec {
    pub fn new(name: &str, rules_toml: &str, init: Init, step_mode: StepMode) -> Self {
        Self {
            name: name.to_string(),
            rules_toml: rules_toml.to_string(),
            init,
            step_mode,
            max_steps: 400,
            node_cap: 20_000,
            seed: 1,
            max_pairings_per_step: 1_000_000,
            burn_in_steps: 0,
            event_cap: u64::MAX,
        }
    }

    pub fn event_cap(mut self, n: u64) -> Self {
        self.event_cap = n;
        self
    }

    pub fn steps(mut self, n: u64) -> Self {
        self.max_steps = n;
        self
    }
    pub fn node_cap(mut self, n: usize) -> Self {
        self.node_cap = n;
        self
    }
    pub fn seed(mut self, s: u64) -> Self {
        self.seed = s;
        self
    }
    pub fn burn_in(mut self, n: u64) -> Self {
        self.burn_in_steps = n;
        self
    }
}

/// A run's alphabets: whole-run always, plus the post-burn-in restriction when
/// the spec asks for one (the M1 correction — churn must be scored after
/// burn-in, because its startup transients dominate the class count).
pub struct RunOutcome {
    pub full: RunAlphabets,
    pub post_burn_in: Option<RunAlphabets>,
}

/// Captures the step's rewrite events out of the observer chain. The engine
/// hands events to observers rather than returning them, so this is the seam.
struct EventSink {
    events: Arc<Mutex<Vec<RewriteEvent>>>,
}

impl Observer for EventSink {
    fn on_step_end(
        &mut self,
        _snapshot: &hg_core::GraphSnapshot<'_>,
        _stats: &StepStats,
        events: &[RewriteEvent],
    ) {
        self.events
            .lock()
            .expect("event sink poisoned")
            .extend_from_slice(events);
    }
}

fn distinct_classes(map: &ClassMap) -> usize {
    let mut v: Vec<u64> = map.values().map(|h| h.0).collect();
    v.sort_unstable();
    v.dedup();
    v.len()
}

/// Run one spec and accumulate its realised event alphabet.
pub fn measure(spec: &RunSpec) -> RunOutcome {
    let mut store = spec.init.build();
    let rules = compile_rules(&parse_rules_toml(&spec.rules_toml).expect("rules parse"))
        .expect("rules compile");
    let mut engine = Engine::new(
        rules,
        spec.step_mode,
        spec.seed,
        spec.max_pairings_per_step,
    );
    let sink = Arc::new(Mutex::new(Vec::new()));
    engine.add_observer(Box::new(EventSink {
        events: Arc::clone(&sink),
    }));

    let mut full = RunAlphabets::new(N_RADII);
    let mut burn = (spec.burn_in_steps > 0).then(|| RunAlphabets::new(N_RADII));

    let mut pre: Vec<ClassMap> = class_maps(&store);
    let mut pre_deg: DegreeMap = degrees(&store);

    let mut event_index: u64 = 0;
    let mut burn_event_index: u64 = 0;
    let mut quiet: u64 = 0;
    let mut steps_run: u64 = 0;
    let mut stop = StopReason::StepCap;

    for step in 0..spec.max_steps {
        full.node_class_count.push((step, distinct_classes(&pre[2])));

        sink.lock().expect("event sink poisoned").clear();
        engine.step(&mut store);
        let events: Vec<RewriteEvent> = std::mem::take(&mut *sink.lock().expect("event sink poisoned"));

        let post: Vec<ClassMap> = class_maps(&store);
        let post_deg: DegreeMap = degrees(&store);

        for e in &events {
            if event_index >= spec.event_cap {
                break;
            }
            let sig = signatures(e, &pre, &post, &pre_deg, &post_deg);
            event_index += 1;
            record(&mut full, &sig, e.rule_id, event_index, step);
            if let Some(b) = burn.as_mut() {
                if step >= spec.burn_in_steps {
                    burn_event_index += 1;
                    // Re-based indices: the post-burn-in novelty curve must be
                    // measured against ITS OWN event and step origin, or the
                    // discarded prefix silently shifts the fit window.
                    record(b, &sig, e.rule_id, burn_event_index, step - spec.burn_in_steps);
                }
            }
        }

        pre = post;
        pre_deg = post_deg;
        steps_run = step + 1;

        if events.is_empty() {
            quiet += 1;
            if quiet >= QUIESCENCE_STEPS {
                stop = StopReason::Quiescent;
                break;
            }
        } else {
            quiet = 0;
        }
        if store.node_count() >= spec.node_cap {
            stop = StopReason::NodeCap;
            break;
        }
        if event_index >= spec.event_cap {
            stop = StopReason::EventCap;
            break;
        }
    }

    full.steps_run = steps_run;
    full.stop_reason = stop;
    full.final_nodes = store.node_count();
    full.final_edges = store.edge_count();
    if let Some(b) = burn.as_mut() {
        b.steps_run = steps_run.saturating_sub(spec.burn_in_steps);
        b.stop_reason = stop;
        b.final_nodes = store.node_count();
        b.final_edges = store.edge_count();
    }

    RunOutcome {
        full,
        post_burn_in: burn,
    }
}

fn record(
    acc: &mut RunAlphabets,
    sig: &crate::signature::EventSignatures,
    rule_id: usize,
    idx: u64,
    step: u64,
) {
    acc.rule_only.record(sig.rule_only, idx, step);
    acc.degree.record(sig.degree, idx, step);
    for r in 0..N_RADII {
        acc.context[r].record(sig.context[r], idx, step);
        acc.delta[r].record(sig.delta[r], idx, step);
    }
    // C0: the same S1 symbols, also accumulated per rule so the share factor in
    // `A_eff` is separable. Grown lazily — the harness does not know the rule
    // count, and an unfired rule must contribute no row rather than an empty one.
    if acc.per_rule_context.len() <= rule_id {
        acc.per_rule_context
            .resize_with(rule_id + 1, || vec![crate::SymbolStats::new(); N_RADII]);
    }
    for r in 0..N_RADII {
        acc.per_rule_context[rule_id][r].record(sig.context[r], idx, step);
    }
}
