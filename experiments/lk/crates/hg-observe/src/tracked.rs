//! `TrackedEdgeObserver` (LK1R, plan v2 §4): exact lineage tracking of
//! the planted shortcut population.
//!
//! Planted shortcuts number ~p·E (dozens); a sampled census will not see
//! them, and fractions dilute under growth (hazard 1) — absolute counts
//! for EXP-L2 can only come from tracking the planted set exactly.
//!
//! **Lineage closure:** the observer starts with the planted shortcut
//! EdgeIds. On every `RewriteEvent` whose LHS consumes a tracked edge,
//! ALL the event's created edges join the tracked set (for subdivision,
//! LHS 1 edge → RHS 2 edges, exactly the replacement path; for wider
//! rules this over-approximates — documented, conservative for leakage).
//! Tracked edges deleted by dangling cleanup with no replacement leave
//! the set. Lineage membership is `measured`; the detour-derived columns
//! are `estimated`.
//!
//! **Morphology outputs (per window):** tracked edge count vs component
//! count (edge count measures refinement of a handle; component count
//! measures handles — divergence is the throat signal, hazard 2),
//! exact detours, `throat_length_max` (longest path within one tracked
//! component), and `far_nucleus_flag` (V3 signature: fresh growth at a
//! tracked far endpoint before the ambient front reaches it).

use crate::observer::Observer;
use crate::RewriteEvent;
use hg_core::{EdgeId, GraphSnapshot, NodeId};
use hg_schema::{ClaimType, TrackedRow};
use std::collections::{HashMap, HashSet, VecDeque};

pub struct TrackedEdgeObserver {
    tracked: HashSet<EdgeId>,
    /// Detour cap (same instrument as §1.1).
    r_max: usize,
    /// Seed site(s) of the energy/growth dynamic, for the far-nucleus
    /// probe (empty disables the probe).
    seed_sites: Vec<NodeId>,
    /// Exact-detour recompute cadence in steps ([leakage.tracked]).
    cadence_steps: u64,
    next_emit_at: u64,
    last_step: u64,
    /// v3 §4 encounter columns: running max of mouth energy, first-fire
    /// step of the far-nucleus flag, first step the ambient front
    /// reached the initial far-mouth distance, and that frozen distance
    /// (measured once, at the first window with tracked edges — the
    /// IC's geometry is the quantity of interest).
    mouth_energy_max: f32,
    t_far_nucleus: Option<u64>,
    t_ambient_front_at_far: Option<u64>,
    d_far_initial: Option<f64>,
    pub rows: Vec<TrackedRow>,
}

impl TrackedEdgeObserver {
    pub fn new(planted: Vec<EdgeId>, r_max: usize, seed_sites: Vec<NodeId>) -> Self {
        Self {
            tracked: planted.into_iter().collect(),
            r_max,
            seed_sites,
            cadence_steps: 1,
            next_emit_at: 0,
            last_step: 0,
            mouth_energy_max: f32::NEG_INFINITY,
            t_far_nucleus: None,
            t_ambient_front_at_far: None,
            d_far_initial: None,
            rows: Vec::new(),
        }
    }

    pub fn with_cadence(mut self, cadence_steps: u64) -> Self {
        self.cadence_steps = cadence_steps.max(1);
        self.next_emit_at = self.cadence_steps;
        self
    }

    pub fn tracked_count(&self) -> usize {
        self.tracked.len()
    }

    /// Lineage closure over one event (public so scripted-event tests
    /// drive exactly the shipping logic).
    pub fn ingest_event(&mut self, event: &RewriteEvent) {
        let consumed_tracked = event
            .edges_deleted
            .iter()
            .any(|e| self.tracked.contains(e));
        for e in &event.edges_deleted {
            self.tracked.remove(e);
        }
        if consumed_tracked {
            for e in &event.edges_created {
                self.tracked.insert(*e);
            }
        }
    }

    /// Compute one tracked.csv row from the current snapshot. Exact
    /// detours: one truncated BFS per tracked edge (dozens — cheap).
    pub fn measure(&mut self, snapshot: &GraphSnapshot<'_>, window: u64, step: u64) -> TrackedRow {
        // Drop edges that no longer exist (deleted without our seeing a
        // consuming event, e.g. cleanup of an untracked node).
        self.tracked.retain(|e| snapshot.get_edge(*e).is_some());
        let edges: Vec<EdgeId> = {
            let mut v: Vec<EdgeId> = self.tracked.iter().copied().collect();
            v.sort_unstable(); // deterministic iteration for percentiles
            v
        };

        let mut detours: Vec<f32> = edges
            .iter()
            .map(|&e| crate::locality::edge_detour(snapshot, e, self.r_max))
            .collect();
        detours.sort_by(|a, b| a.total_cmp(b));
        let wormholes = detours.iter().filter(|d| d.is_infinite()).count() as u32;
        let pct = |q: f64| -> f32 {
            if detours.is_empty() {
                return f32::NAN;
            }
            let rank = ((q * detours.len() as f64).ceil() as usize).clamp(1, detours.len());
            detours[rank - 1]
        };

        let components = tracked_components(snapshot, &edges);
        let throat_length_max = throat_length_max(snapshot, &edges);

        // v3 §4 encounter instrumentation. Censored distances from the
        // seed are shared by the far-mouth timing and the far-nucleus
        // probe.
        let tracked_set: HashSet<EdgeId> = edges.iter().copied().collect();
        let dist = self.censored_dist(snapshot, &tracked_set);
        let endpoint_set: HashSet<NodeId> = edges
            .iter()
            .filter_map(|e| snapshot.get_edge(*e))
            .flat_map(|e| e.nodes.iter().copied())
            .collect();
        // Mouth energy (key 1 = the LK-R energy field): running max.
        for &m in &endpoint_set {
            if let Some(hg_core::PropertyValue::Float(e)) =
                snapshot.get_node(m).and_then(|d| d.get_property(1))
            {
                self.mouth_energy_max = self.mouth_energy_max.max(e as f32);
            }
        }
        // Freeze the initial far-mouth distance on first sight.
        if self.d_far_initial.is_none() && !endpoint_set.is_empty() {
            let d_far = endpoint_set
                .iter()
                .filter_map(|m| dist.get(m).map(|&d| d as f64))
                .fold(f64::NAN, f64::max);
            if d_far.is_finite() {
                self.d_far_initial = Some(d_far);
            }
        }

        let (far_nucleus_flag, ambient_front) =
            self.far_nucleus(snapshot, &edges, step, &dist, &endpoint_set);
        if far_nucleus_flag && self.t_far_nucleus.is_none() {
            self.t_far_nucleus = Some(step);
        }
        if self.t_ambient_front_at_far.is_none() {
            if let Some(d_far) = self.d_far_initial {
                // Same slack as the far-nucleus comparison (±2 sites).
                if ambient_front + 2.0 >= d_far {
                    self.t_ambient_front_at_far = Some(step);
                }
            }
        }

        let row = TrackedRow {
            window,
            step,
            tracked_edge_count: edges.len() as u32,
            tracked_components: components,
            tracked_wormhole_count: wormholes,
            tracked_detour_p50: pct(0.50),
            tracked_detour_p95: pct(0.95),
            throat_length_max,
            far_nucleus_flag,
            mouth_energy_max: self
                .mouth_energy_max
                .is_finite()
                .then_some(self.mouth_energy_max),
            t_far_nucleus: self.t_far_nucleus,
            t_ambient_front_at_far: self.t_ambient_front_at_far,
            claim_type: ClaimType::Estimated,
        };
        self.rows.push(row.clone());
        row
    }

    /// Distance-from-seed avoiding tracked edges (through-handle distance
    /// would put the far endpoint spuriously near).
    fn censored_dist(
        &self,
        snapshot: &GraphSnapshot<'_>,
        tracked: &HashSet<EdgeId>,
    ) -> HashMap<NodeId, u64> {
        let mut dist: HashMap<NodeId, u64> = HashMap::new();
        let mut queue: VecDeque<NodeId> = VecDeque::new();
        for &s in &self.seed_sites {
            if snapshot.get_node(s).is_some() {
                dist.insert(s, 0);
                queue.push_back(s);
            }
        }
        while let Some(v) = queue.pop_front() {
            let d = dist[&v];
            for edge_id in snapshot.edges_incident_to(v) {
                if tracked.contains(edge_id) {
                    continue;
                }
                if let Some(edge) = snapshot.get_edge(*edge_id) {
                    for &nb in &edge.nodes {
                        if !dist.contains_key(&nb) && snapshot.get_node(nb).is_some() {
                            dist.insert(nb, d + 1);
                            queue.push_back(nb);
                        }
                    }
                }
            }
        }
        dist
    }

    /// V3 signature probe: does a tracked component endpoint, far from
    /// the seed in the TRACKED-CENSORED metric, have fresh growth
    /// (nodes created within the last window) in its radius-2
    /// neighborhood, before the ambient front (p95 of distance-from-seed
    /// over all OTHER fresh nodes + slack 2) reaches it? Distances from
    /// the seed avoid tracked edges — through-handle distance would put
    /// the far endpoint spuriously near.
    /// Returns (flag, ambient_front). ambient_front is 0.0 when no
    /// ambient fresh growth exists yet.
    fn far_nucleus(
        &self,
        snapshot: &GraphSnapshot<'_>,
        edges: &[EdgeId],
        step: u64,
        dist: &HashMap<NodeId, u64>,
        endpoint_set: &HashSet<NodeId>,
    ) -> (bool, f64) {
        if self.seed_sites.is_empty() || edges.is_empty() {
            return (false, 0.0);
        }

        // Fresh nodes this window (generation = creation step).
        let window_len = 1.max(step / 8); // conservative: recent eighth
        let fresh_after = step.saturating_sub(window_len);

        // Ambient front: p95 of censored distance-from-seed over fresh
        // nodes NOT near tracked endpoints.
        let near_endpoint = |n: NodeId| -> bool {
            if endpoint_set.contains(&n) {
                return true;
            }
            snapshot.edges_incident_to(n).iter().any(|e| {
                snapshot
                    .get_edge(*e)
                    .is_some_and(|edge| edge.nodes.iter().any(|m| endpoint_set.contains(m)))
            })
        };
        let mut ambient: Vec<f64> = Vec::new();
        let mut nucleus_candidates: Vec<f64> = Vec::new();
        for (id, data) in snapshot.nodes() {
            if data.birth_step <= fresh_after {
                continue;
            }
            let d = dist.get(&id).map(|&d| d as f64).unwrap_or(f64::INFINITY);
            if near_endpoint(id) {
                nucleus_candidates.push(d);
            } else if d.is_finite() {
                ambient.push(d);
            }
        }
        let ambient_front = if ambient.is_empty() {
            0.0
        } else {
            ambient.sort_by(|a, b| a.total_cmp(b));
            let rank = ((0.95 * ambient.len() as f64).ceil() as usize).clamp(1, ambient.len());
            ambient[rank - 1]
        };
        if nucleus_candidates.is_empty() {
            return (false, ambient_front);
        }
        let flag = nucleus_candidates
            .iter()
            .any(|&d| d > ambient_front + 2.0);
        (flag, ambient_front)
    }
}

/// Connected components of the tracked-edge subgraph (endpoint-sharing).
fn tracked_components(snapshot: &GraphSnapshot<'_>, edges: &[EdgeId]) -> u32 {
    crate::locality::wormhole_components(snapshot, edges)
}

/// Longest shortest-path (in tracked-edge hops) between any two nodes of
/// one tracked component — the throat length. Components are small
/// (handles), so per-node BFS within the tracked subgraph is cheap.
fn throat_length_max(snapshot: &GraphSnapshot<'_>, edges: &[EdgeId]) -> u32 {
    // Adjacency restricted to tracked edges.
    let mut adj: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
    for e in edges {
        if let Some(edge) = snapshot.get_edge(*e) {
            for &a in &edge.nodes {
                for &b in &edge.nodes {
                    if a != b {
                        adj.entry(a).or_default().push(b);
                    }
                }
            }
        }
    }
    let mut best = 0u32;
    for &start in adj.keys() {
        let mut dist: HashMap<NodeId, u32> = HashMap::new();
        dist.insert(start, 0);
        let mut queue: VecDeque<NodeId> = VecDeque::new();
        queue.push_back(start);
        while let Some(v) = queue.pop_front() {
            let d = dist[&v];
            best = best.max(d);
            if let Some(nbs) = adj.get(&v) {
                for &nb in nbs {
                    if let std::collections::hash_map::Entry::Vacant(e) = dist.entry(nb) {
                        e.insert(d + 1);
                        queue.push_back(nb);
                    }
                }
            }
        }
    }
    best
}

impl Observer for TrackedEdgeObserver {
    fn on_step_end(
        &mut self,
        _snapshot: &GraphSnapshot<'_>,
        stats: &crate::StepStats,
        events: &[RewriteEvent],
    ) {
        self.last_step = stats.step;
        for event in events {
            self.ingest_event(event);
        }
    }

    fn on_window_complete(&mut self, snapshot: &GraphSnapshot<'_>, window: u64) {
        if self.last_step >= self.next_emit_at {
            let c = self.cadence_steps;
            self.next_emit_at = (self.last_step / c + 1) * c;
            let step = self.last_step;
            self.measure(snapshot, window, step);
        }
    }
}

/// Chain adapter: the engine owns its observers, but the runner needs to
/// drain tracked.csv rows at run end — share the observer through an
/// `Arc<Mutex<_>>` (same pattern as the causal event log).
pub struct SharedTracked(pub std::sync::Arc<std::sync::Mutex<TrackedEdgeObserver>>);

impl Observer for SharedTracked {
    fn on_step_end(
        &mut self,
        snapshot: &GraphSnapshot<'_>,
        stats: &crate::StepStats,
        events: &[RewriteEvent],
    ) {
        self.0.lock().expect("tracked lock").on_step_end(snapshot, stats, events);
    }

    fn on_window_complete(&mut self, snapshot: &GraphSnapshot<'_>, window: u64) {
        self.0.lock().expect("tracked lock").on_window_complete(snapshot, window);
    }
}

/// Counts firings of one named rule (LK-R audit input: N_subdivisions).
/// Shared through an atomic so the runner reads it while the engine's
/// chain owns the observer.
pub struct RuleFireCounter {
    pub rule_name: String,
    pub count: std::sync::Arc<std::sync::atomic::AtomicU64>,
}

impl Observer for RuleFireCounter {
    fn on_step_end(
        &mut self,
        _snapshot: &GraphSnapshot<'_>,
        _stats: &crate::StepStats,
        events: &[RewriteEvent],
    ) {
        let n = events.iter().filter(|e| e.rule_name == self.rule_name).count() as u64;
        if n > 0 {
            self.count
                .fetch_add(n, std::sync::atomic::Ordering::Relaxed);
        }
    }
}

/// v4 §6.3 fused-energy ledger accumulator: sums `vanished_energy`
/// over every rewrite event (in the LK family only fuse deletes
/// nodes). Shared through a mutex so the runner reads the running
/// total for the per-window audit line. f64 bits in an AtomicU64 would
/// race on read-modify-write; the mutex is uncontended (engine chain).
pub struct EnergyVanishedCounter {
    pub sum: std::sync::Arc<std::sync::Mutex<f64>>,
}

impl Observer for EnergyVanishedCounter {
    fn on_step_end(
        &mut self,
        _snapshot: &GraphSnapshot<'_>,
        _stats: &crate::StepStats,
        events: &[RewriteEvent],
    ) {
        let total: f64 = events.iter().map(|e| e.vanished_energy).sum();
        if total != 0.0 {
            *self.sum.lock().expect("vanished lock") += total;
        }
    }
}

/// v4 §7 churn counter: fuse-then-resubdivide at the same site within
/// `window_steps`. "Site" = the unordered endpoint pair (x, y) the fuse
/// rejoined; a later subdivision binding that same pair within the
/// window is one churn event. Measured (suspicious-flag input), never
/// prevented.
pub struct ChurnCounter {
    pub fuse_rule: String,
    pub subdivide_rule: String,
    pub window_steps: u64,
    pub count: std::sync::Arc<std::sync::atomic::AtomicU64>,
    recent_fuses: std::collections::VecDeque<(u64, NodeId, NodeId)>,
}

impl ChurnCounter {
    pub fn new(
        window_steps: u64,
        count: std::sync::Arc<std::sync::atomic::AtomicU64>,
    ) -> Self {
        Self {
            fuse_rule: "lk_fuse".into(),
            subdivide_rule: "lk_metered_subdivide".into(),
            window_steps,
            count,
            recent_fuses: Default::default(),
        }
    }
}

impl Observer for ChurnCounter {
    fn on_step_end(
        &mut self,
        _snapshot: &GraphSnapshot<'_>,
        stats: &crate::StepStats,
        events: &[RewriteEvent],
    ) {
        let step = stats.step;
        while let Some(&(s, _, _)) = self.recent_fuses.front() {
            if s + self.window_steps < step {
                self.recent_fuses.pop_front();
            } else {
                break;
            }
        }
        for event in events {
            if event.rule_name == self.fuse_rule {
                // lk_fuse LHS declares ["x", "w", "y"]: indices 0 and 2
                // are the rejoined endpoints.
                if let (Some(&x), Some(&y)) = (
                    event.binding.bound_nodes.first(),
                    event.binding.bound_nodes.get(2),
                ) {
                    self.recent_fuses.push_back((step, x.min(y), x.max(y)));
                }
            } else if event.rule_name == self.subdivide_rule {
                // lk_metered_subdivide LHS declares ["x", "y"].
                if let (Some(&x), Some(&y)) = (
                    event.binding.bound_nodes.first(),
                    event.binding.bound_nodes.get(1),
                ) {
                    let key = (x.min(y), x.max(y));
                    if self
                        .recent_fuses
                        .iter()
                        .any(|&(_, a, b)| (a, b) == key)
                    {
                        self.count
                            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    }
                }
            }
        }
    }
}
