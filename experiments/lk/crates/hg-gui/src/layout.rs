//! Incremental force-directed layout for the graph canvas.
//!
//! Positions live GUI-side, keyed by the real `NodeId` (same identifier
//! the causal inspector uses), and persist across frames so the layout
//! relaxes continuously while the sim runs. Presentation-only: nothing
//! here feeds back into the engine, so engine determinism is unaffected.

use eframe::egui::{Rect, Vec2};
use hg_app_api::{CompactGraphSnapshot, NodeId};
use slotmap::Key;
use std::collections::{HashMap, HashSet};

/// Preferred edge length in world units.
const EDGE_LEN: f32 = 40.0;
/// Spring constant for attraction along edges.
const K_ATTRACT: f32 = 0.06;
/// Repulsion strength (~ force at distance EDGE_LEN is K_REPEL/EDGE_LEN²).
const K_REPEL: f32 = EDGE_LEN * EDGE_LEN * 0.8;
/// Repulsion cutoff: beyond this distance nodes ignore each other.
const REPEL_CUTOFF: f32 = EDGE_LEN * 2.5;
/// Max displacement per node per iteration (acts as temperature cap and
/// divergence guard — the doc's "per-step clamping").
const MAX_DISP: f32 = 8.0;
/// Cap on repulsion partners considered per node per iteration, so one
/// overcrowded grid cell can't make a frame quadratic. Too small and
/// dense clumps stop repelling internally (blob-collapse); 48 keeps a
/// 5k-node frame around ~250k pair ops.
const MAX_REPEL_PARTNERS: usize = 48;

#[derive(Default)]
pub struct ForceLayout {
    positions: HashMap<NodeId, Vec2>,
    /// Count of nodes ever seeded — drives the sunflower-spiral placement
    /// so successive seeds spread out instead of stacking at the origin.
    seeded: u64,
}

/// Deterministic per-node jitter angle from the NodeId (splitmix64 mix).
fn id_mix(id: NodeId) -> u64 {
    let mut x = id.data().as_ffi().wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

impl ForceLayout {
    pub fn clear(&mut self) {
        self.positions.clear();
        self.seeded = 0;
    }

    pub fn position(&self, id: NodeId) -> Option<Vec2> {
        self.positions.get(&id).copied()
    }

    /// World-space AABB of all current positions.
    pub fn bounds(&self) -> Option<Rect> {
        let mut it = self.positions.values();
        let first = *it.next()?;
        let mut min = first;
        let mut max = first;
        for &p in it {
            min = min.min(p);
            max = max.max(p);
        }
        Some(Rect::from_min_max(min.to_pos2(), max.to_pos2()))
    }

    /// Drop positions for dead nodes; seed positions for new ones. New
    /// nodes with an already-placed neighbor start at that neighborhood's
    /// centroid plus a small deterministic jitter (keeps subdivision
    /// local); orphans go on a sunflower spiral (a nonzero spread —
    /// co-located seeds produce zero net force and never separate).
    pub fn sync(&mut self, graph: &CompactGraphSnapshot) {
        let live: HashSet<NodeId> = graph.nodes.iter().map(|n| n.id).collect();
        self.positions.retain(|id, _| live.contains(id));

        // One pass over edges: centroid of already-positioned endpoints,
        // offered to any unpositioned endpoint of the same edge.
        let mut suggested: HashMap<NodeId, Vec2> = HashMap::new();
        for edge in &graph.edges {
            let placed: Vec<Vec2> = edge
                .nodes
                .iter()
                .filter_map(|id| self.positions.get(id).copied())
                .collect();
            if placed.is_empty() {
                continue;
            }
            let centroid =
                placed.iter().fold(Vec2::ZERO, |acc, &p| acc + p) / placed.len() as f32;
            for id in &edge.nodes {
                if !self.positions.contains_key(id) {
                    suggested.entry(*id).or_insert(centroid);
                }
            }
        }

        for node in &graph.nodes {
            if self.positions.contains_key(&node.id) {
                continue;
            }
            let pos = match suggested.get(&node.id) {
                Some(&centroid) => {
                    let angle = (id_mix(node.id) % 6283) as f32 / 1000.0;
                    centroid + Vec2::angled(angle) * (EDGE_LEN * 0.5)
                }
                None => {
                    // Sunflower spiral: r ∝ √k, golden-angle steps.
                    let k = self.seeded as f32;
                    let r = EDGE_LEN * 0.5 * k.sqrt();
                    Vec2::angled(k * 2.399_963) * r
                }
            };
            self.positions.insert(node.id, pos);
            self.seeded += 1;
        }
    }

    /// Run a few relaxation iterations. Returns true if the layout is
    /// still visibly moving (caller should request a repaint).
    pub fn step(&mut self, graph: &CompactGraphSnapshot) -> bool {
        let n = graph.nodes.len();
        if n < 2 {
            return false;
        }
        // Budget: more iterations while small, one per frame when large.
        // Trees need sustained iteration to unfold into filaments — the
        // layout runs every frame, so the budget accumulates over time.
        let iters = (4000 / n).clamp(1, 4);

        // Snapshot order is deterministic; all force loops run over these
        // indices, never over HashMap iteration order.
        let index_of: HashMap<NodeId, usize> = graph
            .nodes
            .iter()
            .enumerate()
            .map(|(i, node)| (node.id, i))
            .collect();
        let mut pos: Vec<Vec2> = graph
            .nodes
            .iter()
            .map(|node| self.positions.get(&node.id).copied().unwrap_or(Vec2::ZERO))
            .collect();

        let mut max_disp = 0.0f32;
        for _ in 0..iters {
            let mut disp = vec![Vec2::ZERO; n];

            // Repulsion via a uniform grid: each node only sees the 3×3
            // cell neighborhood, so an iteration stays ~O(n).
            let cell = REPEL_CUTOFF;
            let mut grid: HashMap<(i32, i32), Vec<usize>> = HashMap::new();
            for (i, p) in pos.iter().enumerate() {
                grid.entry(((p.x / cell) as i32, (p.y / cell) as i32))
                    .or_default()
                    .push(i);
            }
            for (i, p) in pos.iter().enumerate() {
                let (cx, cy) = ((p.x / cell) as i32, (p.y / cell) as i32);
                let mut partners = 0usize;
                'scan: for dx in -1..=1 {
                    for dy in -1..=1 {
                        let Some(bucket) = grid.get(&(cx + dx, cy + dy)) else {
                            continue;
                        };
                        for &j in bucket {
                            if j == i {
                                continue;
                            }
                            let delta = *p - pos[j];
                            let d2 = delta.length_sq().max(1.0);
                            if d2 > REPEL_CUTOFF * REPEL_CUTOFF {
                                continue;
                            }
                            disp[i] += delta * (K_REPEL / (d2 * d2.sqrt()));
                            partners += 1;
                            if partners >= MAX_REPEL_PARTNERS {
                                break 'scan;
                            }
                        }
                    }
                }
            }

            // Attraction along edges: same consecutive-pairs (+ closing
            // pair for arity > 2) decomposition the canvas draws.
            let spring = |a: usize, b: usize, disp: &mut Vec<Vec2>| {
                let delta = pos[a] - pos[b];
                let dist = delta.length().max(0.1);
                let f = delta / dist * ((dist - EDGE_LEN) * K_ATTRACT * 0.5);
                disp[a] -= f;
                disp[b] += f;
            };
            for edge in &graph.edges {
                let idx: Vec<usize> = edge
                    .nodes
                    .iter()
                    .filter_map(|id| index_of.get(id).copied())
                    .collect();
                if idx.len() >= 2 {
                    for w in idx.windows(2) {
                        spring(w[0], w[1], &mut disp);
                    }
                    if idx.len() > 2 {
                        spring(idx[idx.len() - 1], idx[0], &mut disp);
                    }
                }
            }

            // Integrate with clamping (divergence/NaN guard).
            for (p, d) in pos.iter_mut().zip(&disp) {
                let mut d = *d;
                if !d.is_finite() {
                    d = Vec2::ZERO;
                }
                let len = d.length();
                if len > MAX_DISP {
                    d *= MAX_DISP / len;
                }
                *p += d;
                max_disp = max_disp.max(d.length());
            }
        }

        for (node, p) in graph.nodes.iter().zip(&pos) {
            if p.is_finite() {
                self.positions.insert(node.id, *p);
            } else {
                // Diverged despite clamping: reseed on the spiral.
                let k = self.seeded as f32;
                self.positions
                    .insert(node.id, Vec2::angled(k * 2.399_963) * (EDGE_LEN * 0.5 * k.sqrt()));
                self.seeded += 1;
            }
        }

        max_disp > 0.05
    }
}
