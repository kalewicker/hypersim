//! CausalObserver: builds attributes, tasks, constructors, claims, and EI
//! from per-window WL-class transitions.

use crate::ei::{compute_ei_results, ClassFeatures, EiResult, EiSettings, TpmAccumulator};
use crate::motif::{MotifClassifier, NullMotifClassifier};
use crate::{
    Attribute, CertaintyClass, ConstructorRegistry, KnowledgeGraph, Predicate, TaskLattice,
};
use ahash::{AHashMap, AHashSet};
use hg_core::{compute_wl_class_map_snapshot, GraphSnapshot, NodeId, WlHash};
use hg_observe::{Observer, RewriteEvent, StepStats};

const WL_RADIUS: u8 = 1;

pub struct CausalObserver {
    pub lattice: TaskLattice,
    pub constructors: ConstructorRegistry,
    pub knowledge: KnowledgeGraph,
    classifier: Box<dyn MotifClassifier>,
    prev: Option<PrevWindow>,
    /// Cumulative node-level class-transition counts (the micro TPM).
    pub tpm: TpmAccumulator,
    /// Exemplar features per class, refreshed from a witness node each
    /// window (structural/energetic coarse-graining input). Append-only:
    /// vanished classes keep their last-known features because the TPM
    /// still carries their historical rows.
    class_features: AHashMap<WlHash, ClassFeatures>,
    pub ei_settings: EiSettings,
    windows_seen: u64,
    next_emit_at: u64,
    /// Latest per-strategy EI/CE results (empty until the emission
    /// interval AND the minimum-transition gate are both crossed).
    pub latest_ei: Vec<EiResult>,
    /// EI_micro trajectory, one entry per emission (stability check).
    pub ei_micro_history: Vec<f64>,
    /// M(t): (window step, registry size) per emission — saturation curve.
    pub registry_size_history: Vec<(u64, u32)>,
}

struct PrevWindow {
    class_of: AHashMap<NodeId, WlHash>,
    hist: AHashMap<WlHash, usize>,
    adjacency: AHashMap<WlHash, AHashSet<WlHash>>,
    /// Per-node 2-hop class reach — kept so constructor survival can be
    /// judged per focal INSTANCE (NodeId), not per class.
    node_reach: AHashMap<NodeId, AHashSet<WlHash>>,
    /// Prev nodes by class (focal-candidate lookup).
    members: AHashMap<WlHash, Vec<NodeId>>,
}

impl Default for CausalObserver {
    fn default() -> Self {
        Self::new()
    }
}

impl CausalObserver {
    pub fn new() -> Self {
        Self {
            lattice: TaskLattice::new(),
            constructors: ConstructorRegistry::new(),
            knowledge: KnowledgeGraph::new(),
            classifier: Box::new(NullMotifClassifier),
            prev: None,
            tpm: TpmAccumulator::new(),
            class_features: AHashMap::new(),
            ei_settings: EiSettings::default(),
            windows_seen: 0,
            next_emit_at: 0,
            latest_ei: Vec::new(),
            ei_micro_history: Vec::new(),
            registry_size_history: Vec::new(),
        }
    }

    pub fn with_classifier(mut self, classifier: Box<dyn MotifClassifier>) -> Self {
        self.classifier = classifier;
        self
    }

    pub fn with_ei_settings(mut self, settings: EiSettings) -> Self {
        self.next_emit_at = settings.interval_steps;
        self.ei_settings = settings;
        self
    }

    fn attribute(&mut self, snapshot: &GraphSnapshot<'_>, h: WlHash, count: usize, any_node: Option<NodeId>) -> Attribute {
        let motif_class = any_node
            .map(|n| self.classifier.classify(snapshot, n))
            .unwrap_or(0);
        Attribute {
            wl_hash: h,
            radius: WL_RADIUS,
            motif_class,
            node_count: count as u32,
            certainty: CertaintyClass::Empirical, // directly measured
        }
    }

    /// Verifier-facing dump of the raw TPM + active partitions
    /// (verify_ei.py). Reflects the accumulator's CURRENT state.
    pub fn ei_dump(&self) -> crate::ei::EiDump {
        // Canonical order: classes sorted by hash. Dense-index assignment
        // follows in-run recording order, which varies across processes
        // (hash-map iteration); EI is permutation-invariant but the dump
        // must be byte-stable per docs/reproducibility_policy.md.
        // Uses the shared `TpmAccumulator::canonical_order` so production and
        // every analysis harness go through ONE implementation of this rule.
        let hashes = self.tpm.hashes();
        let order = self.tpm.canonical_order();
        let counts = self.tpm.counts_dense();
        let partitions = crate::ei::partitions_for(
            &self.tpm,
            &self.class_features,
            &self.constructors,
            &self.ei_settings,
        );
        crate::ei::EiDump {
            classes: order.iter().map(|&i| format!("{:016x}", hashes[i].0)).collect(),
            counts: order
                .iter()
                .map(|&i| order.iter().map(|&j| counts[i][j]).collect())
                .collect(),
            cadence_steps: self.ei_settings.cadence_steps,
            partitions: partitions
                .into_iter()
                .map(|(name, groups)| (name, order.iter().map(|&i| groups[i]).collect()))
                .collect(),
        }
    }

    /// Process one window snapshot, computing the WL class map internally.
    pub fn process_window(&mut self, snapshot: &GraphSnapshot<'_>, step: u64) {
        let classes = compute_wl_class_map_snapshot(snapshot, WL_RADIUS);
        self.process_window_with_classes(snapshot, step, &classes)
    }

    /// Same, with the once-per-window WL class map supplied by the caller
    /// (shared with the evaluator so both layers agree on "type").
    pub fn process_window_with_classes(
        &mut self,
        snapshot: &GraphSnapshot<'_>,
        step: u64,
        classes: &AHashMap<NodeId, WlHash>,
    ) {
        // Current classes.
        let mut class_of: AHashMap<NodeId, WlHash> = AHashMap::new();
        let mut hist: AHashMap<WlHash, usize> = AHashMap::new();
        let mut witness_node: AHashMap<WlHash, NodeId> = AHashMap::new();
        for (id, _) in snapshot.nodes() {
            let h = classes.get(&id).copied().unwrap_or(WlHash(0));
            class_of.insert(id, h);
            *hist.entry(h).or_insert(0) += 1;
            witness_node.entry(h).or_insert(id);
        }

        // Class reach: classes within 2 hops of each class. Two hops, not
        // one, deliberately: at radius-1 WL a class *directly* adjacent to
        // a transforming node changes class itself (the neighbor's label
        // is part of its hash), so a persisting catalyst is necessarily
        // buffered one node away from what it transforms.
        let mut node_adj: AHashMap<NodeId, AHashSet<NodeId>> = AHashMap::new();
        for (_, edge) in snapshot.edges() {
            for &a in &edge.nodes {
                for &b in &edge.nodes {
                    if a != b {
                        node_adj.entry(a).or_default().insert(b);
                    }
                }
            }
        }
        let mut adjacency: AHashMap<WlHash, AHashSet<WlHash>> = AHashMap::new();
        let mut node_reach: AHashMap<NodeId, AHashSet<WlHash>> = AHashMap::new();
        let mut members: AHashMap<WlHash, Vec<NodeId>> = AHashMap::new();
        for (&node, &cls) in &class_of {
            members.entry(cls).or_default().push(node);
            let mut reach: AHashSet<WlHash> = AHashSet::new();
            if let Some(n1s) = node_adj.get(&node) {
                for n1 in n1s {
                    if let Some(&c1) = class_of.get(n1) {
                        reach.insert(c1);
                    }
                    if let Some(n2s) = node_adj.get(n1) {
                        for n2 in n2s {
                            if *n2 != node {
                                if let Some(&c2) = class_of.get(n2) {
                                    reach.insert(c2);
                                }
                            }
                        }
                    }
                }
            }
            adjacency.entry(cls).or_default().extend(reach.iter().copied());
            node_reach.insert(node, reach);
        }

        // Exemplar features per class (structural/energetic coarse-
        // graining input): degree, dominant incident-edge arity, and
        // energy of one witness node per class.
        {
            let witnesses: AHashSet<NodeId> = witness_node.values().copied().collect();
            let mut arity_counts: AHashMap<NodeId, AHashMap<usize, u32>> = AHashMap::new();
            for (_, edge) in snapshot.edges() {
                for &n in &edge.nodes {
                    if witnesses.contains(&n) {
                        *arity_counts
                            .entry(n)
                            .or_default()
                            .entry(edge.nodes.len())
                            .or_insert(0) += 1;
                    }
                }
            }
            for (&cls, &w) in &witness_node {
                let degree = node_adj.get(&w).map(|s| s.len()).unwrap_or(0) as u32;
                let dominant_arity = arity_counts
                    .get(&w)
                    .and_then(|m| m.iter().max_by_key(|&(_, &c)| c).map(|(&a, _)| a as u32))
                    .unwrap_or(0);
                let energy = snapshot.get_node(w).map(|d| d.energy).unwrap_or(0.0);
                let motif_class = self.classifier.classify(snapshot, w);
                self.class_features
                    .insert(cls, ClassFeatures { degree, dominant_arity, energy, motif_class });
            }
        }

        if let Some(prev) = self.prev.take() {
            // 1. Node-level class transitions (surviving nodes).
            let mut window_transitions: AHashMap<(WlHash, WlHash), usize> = AHashMap::new();
            // Only nodes existing at BOTH observation points (same
            // NodeId) record transitions — created and deleted nodes are
            // not transitions. Self-transitions ARE recorded (diagonal
            // mass; see ei.rs).
            for (node, &h_now) in &class_of {
                if let Some(&h_prev) = prev.class_of.get(node) {
                    *window_transitions.entry((h_prev, h_now)).or_insert(0) += 1;
                    self.tpm.record(h_prev, h_now);
                }
            }

            // 2. Tasks: class transformations (h_prev != h_now).
            for (&(h_prev, h_now), _count) in &window_transitions {
                if h_prev == h_now {
                    continue;
                }
                let input = self.attribute(snapshot, h_prev, *prev.hist.get(&h_prev).unwrap_or(&0), None);
                let output = self.attribute(
                    snapshot,
                    h_now,
                    *hist.get(&h_now).unwrap_or(&0),
                    witness_node.get(&h_now).copied(),
                );
                let ix = self.lattice.witness(&input, &output, step);
                let task_id = self.lattice.tasks().nth(ix.index()).map(|t| t.id).unwrap_or(0);
                self.knowledge.add_claim(
                    format!("{:016x}", h_prev.0),
                    Predicate::TransformsInto,
                    format!("{:016x}", h_now.0),
                    CertaintyClass::Empirical,
                    step,
                );

                // 3. Constructors: a class that was positioned (within
                // 2-hop reach) to catalyze the transforming input class.
                // Survival is judged per focal INSTANCE by NodeId: the
                // cycle is CONFIRMED only if some focal node — a prev
                // node of this class whose own reach included the input
                // class — is still alive now, in the same class, with the
                // SAME NodeId. A structurally identical pattern elsewhere
                // (same WL hash, different nodes) is DEGRADED, not
                // survival. (Previously non-persisting cycles were
                // skipped instead of degraded, so fidelity was a uniform
                // fake 1.00.)
                for (&cls, neighbors) in &prev.adjacency {
                    if !neighbors.contains(&h_prev) {
                        continue;
                    }
                    let confirmed = prev.members.get(&cls).is_some_and(|nodes| {
                        nodes.iter().any(|n| {
                            prev.node_reach
                                .get(n)
                                .is_some_and(|reach| reach.contains(&h_prev))
                                && class_of.get(n) == Some(&cls)
                        })
                    });
                    let pattern = self.attribute(
                        snapshot,
                        cls,
                        *hist.get(&cls).unwrap_or(&0),
                        witness_node.get(&cls).copied(),
                    );
                    let grew = hist.get(&cls).copied().unwrap_or(0)
                        > prev.hist.get(&cls).copied().unwrap_or(0);
                    self.constructors
                        .record_cycle(&pattern, task_id, confirmed, grew && confirmed);
                    // Claims only for confirmed cycles: a degraded cycle
                    // means the catalyst did NOT survive it.
                    if confirmed {
                        self.knowledge.add_claim(
                            format!("{:016x}", cls.0),
                            Predicate::Catalyzes,
                            format!("task_{task_id}"),
                            CertaintyClass::Model,
                            step,
                        );
                        if grew {
                            self.knowledge.add_claim(
                                format!("{:016x}", cls.0),
                                Predicate::Replicates,
                                format!("{}", hist.get(&cls).copied().unwrap_or(0)),
                                CertaintyClass::Model,
                                step,
                            );
                        }
                    }
                }
            }

            // 4. Persistence / emergence / vanishing claims.
            for h in hist.keys() {
                if prev.hist.contains_key(h) {
                    if window_transitions.contains_key(&(*h, *h)) {
                        self.knowledge.add_claim(
                            format!("{:016x}", h.0),
                            Predicate::PersistsAcross,
                            format!("window_{step}"),
                            CertaintyClass::Empirical,
                            step,
                        );
                    }
                } else {
                    self.knowledge.add_claim(
                        format!("{:016x}", h.0),
                        Predicate::EmergesAt,
                        format!("window_{step}"),
                        CertaintyClass::Empirical,
                        step,
                    );
                }
            }
            for h in prev.hist.keys() {
                if !hist.contains_key(h) {
                    self.knowledge.add_claim(
                        format!("{:016x}", h.0),
                        Predicate::VanishesAt,
                        format!("window_{step}"),
                        CertaintyClass::Empirical,
                        step,
                    );
                }
            }

            // 5. EI/CE emission: one EiResult per enabled strategy per
            // ei_interval_steps, gated on the minimum-transition count.
            // The observer sees windows; steps elapsed = windows seen x
            // cadence (constant within a run — never mix cadences in one
            // TPM).
            let steps_observed = self.windows_seen * self.ei_settings.cadence_steps;
            if steps_observed >= self.next_emit_at {
                let interval = self.ei_settings.interval_steps.max(1);
                self.next_emit_at = (steps_observed / interval + 1) * interval;
                let results = compute_ei_results(
                    &self.tpm,
                    &self.class_features,
                    &self.constructors,
                    &self.ei_settings,
                    step,
                );
                if !results.is_empty() {
                    self.ei_micro_history.push(results[0].ei_micro);
                    self.registry_size_history
                        .push((step, results[0].class_registry_size));
                    // CE claims: Empirical (>= min transitions by the
                    // gate above), never Law — measured, not proven.
                    for r in &results {
                        self.knowledge.add_claim(
                            format!("ce_{}", r.strategy.as_str()),
                            Predicate::ExhibitsCe,
                            format!("{:.4}", r.ce_coefficient),
                            CertaintyClass::Empirical,
                            step,
                        );
                    }
                    self.latest_ei = results;
                }
            }
        }
        self.windows_seen += 1;

        self.prev = Some(PrevWindow {
            class_of,
            hist,
            adjacency,
            node_reach,
            members,
        });
    }
}

impl Observer for CausalObserver {
    fn on_step_end(
        &mut self,
        _snapshot: &GraphSnapshot<'_>,
        _stats: &StepStats,
        _events: &[RewriteEvent],
    ) {
    }

    fn on_window_complete(&mut self, snapshot: &GraphSnapshot<'_>, window: u64) {
        self.process_window(snapshot, window);
    }

    fn on_run_end(&mut self, _snapshot: &GraphSnapshot<'_>) {}
}
