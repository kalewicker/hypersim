//! P4 — residual-energy autopsy (plan v3 §6, runs because P1/EXP-L9b
//! landed PARTIAL). The audit invariant fixes the aggregate (stranded
//! live energy = q·N at quench); this tests the SPATIAL claim: is the
//! stranded energy tip-concentrated along the extruded filaments
//! (forward extrusion, third independent line) or uniform (energy
//! stalls mid-chain — a nuance to record)?
//!
//! Offline over final snapshots. Pre-committed readings (frozen in the
//! plan §6, before this ran):
//!   tip-concentrated → forward extrusion confirmed
//!   uniform along chains → extrusion picture needs a recorded nuance
//!
//! Run: HYPERSIM_P4_RUNS=<run_dir>[:<run_dir>...] cargo test -p hg-eval \
//!   --test p4_autopsy -- --ignored --nocapture

use hg_core::{HypergraphStore, NodeId, PropertyValue};
use std::collections::{HashMap, HashSet, VecDeque};

const ENERGY: u32 = 1;
const GENERATION: u32 = 6;

fn float_prop(store: &HypergraphStore, id: NodeId, key: u32) -> Option<f64> {
    match store.get_node(id)?.get_property(key) {
        Some(PropertyValue::Float(v)) => Some(v),
        _ => None,
    }
}

#[test]
#[ignore]
fn p4_residual_energy_autopsy() {
    let runs = std::env::var("HYPERSIM_P4_RUNS").expect("set HYPERSIM_P4_RUNS");
    for run_dir in runs.split(':') {
        let dir = std::path::Path::new(run_dir);
        let snaps = hg_eval::cone::load_snapshots(&dir.join("snapshots")).expect("snapshots");
        let Some((step, store)) = snaps.last() else {
            println!("{run_dir}: no snapshots — skipped");
            continue;
        };

        // Chain subgraph: degree-2, generation > 0 (the census definition).
        let mut degree: HashMap<NodeId, u32> = HashMap::new();
        let mut adj: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
        for (_, edge) in store.edges() {
            let mut ends: Vec<NodeId> = edge.nodes.to_vec();
            ends.sort_unstable();
            ends.dedup();
            for &u in &ends {
                *degree.entry(u).or_default() += 1;
                for &v in &ends {
                    if v != u {
                        adj.entry(u).or_default().push(v);
                    }
                }
            }
        }
        let eligible: HashSet<NodeId> = store
            .nodes()
            .filter(|(id, _)| {
                degree.get(id).copied() == Some(2)
                    && float_prop(store, *id, GENERATION).is_some_and(|g| g > 0.0)
            })
            .map(|(id, _)| id)
            .collect();

        // Seed = the gen-0 node with maximum residual energy (the
        // quenched reservoir). Chains are indexed from the seed-adjacent
        // end ("root", position 0) to the tip (position 1).
        let seed = store
            .nodes()
            .filter(|(id, _)| float_prop(store, *id, GENERATION) == Some(0.0))
            .max_by(|a, b| {
                float_prop(store, a.0, ENERGY)
                    .unwrap_or(0.0)
                    .total_cmp(&float_prop(store, b.0, ENERGY).unwrap_or(0.0))
            })
            .map(|(id, _)| id)
            .expect("a gen-0 node exists");

        // Energy by generation value.
        let mut by_gen: HashMap<u64, (f64, usize)> = HashMap::new();
        for &id in &eligible {
            let g = float_prop(store, id, GENERATION).unwrap() as u64;
            let e = float_prop(store, id, ENERGY).unwrap_or(0.0);
            let entry = by_gen.entry(g).or_default();
            entry.0 += e;
            entry.1 += 1;
        }

        // Energy by relative position-in-chain (deciles root→tip).
        let mut deciles = [(0.0f64, 0usize); 10];
        let mut seen: HashSet<NodeId> = HashSet::new();
        let seed_adjacent: HashSet<NodeId> =
            adj.get(&seed).map(|v| v.iter().copied().collect()).unwrap_or_default();
        for &start in &eligible {
            if seen.contains(&start) {
                continue;
            }
            // Collect the component.
            let mut comp = vec![start];
            seen.insert(start);
            let mut stack = vec![start];
            while let Some(u) = stack.pop() {
                for v in adj.get(&u).into_iter().flatten() {
                    if eligible.contains(v) && seen.insert(*v) {
                        comp.push(*v);
                        stack.push(*v);
                    }
                }
            }
            // Root = the component node adjacent to the seed (falls back
            // to the minimum id — deterministic — for detached chains).
            let root = comp
                .iter()
                .copied()
                .find(|n| seed_adjacent.contains(n))
                .unwrap_or_else(|| *comp.iter().min().unwrap());
            // BFS order from root = position along the chain.
            let mut dist: HashMap<NodeId, usize> = HashMap::new();
            dist.insert(root, 0);
            let mut queue: VecDeque<NodeId> = VecDeque::new();
            queue.push_back(root);
            let mut max_d = 0usize;
            while let Some(u) = queue.pop_front() {
                let d = dist[&u];
                max_d = max_d.max(d);
                for v in adj.get(&u).into_iter().flatten() {
                    if eligible.contains(v) && !dist.contains_key(v) {
                        dist.insert(*v, d + 1);
                        queue.push_back(*v);
                    }
                }
            }
            for (&n, &d) in &dist {
                let rel = if max_d == 0 { 0.0 } else { d as f64 / max_d as f64 };
                let bin = ((rel * 10.0) as usize).min(9);
                deciles[bin].0 += float_prop(store, n, ENERGY).unwrap_or(0.0);
                deciles[bin].1 += 1;
            }
        }

        println!("== {run_dir} (final snapshot step {step})");
        println!(
            "  seed residual energy: {:.1}",
            float_prop(store, seed, ENERGY).unwrap_or(f64::NAN)
        );
        let mut gens: Vec<u64> = by_gen.keys().copied().collect();
        gens.sort_unstable();
        for g in gens {
            let (e, n) = by_gen[&g];
            println!("  gen {g}: mean energy {:.2} over {n} nodes", e / n as f64);
        }
        println!("  position-in-chain (root→tip deciles): mean energy per node");
        for (i, (e, n)) in deciles.iter().enumerate() {
            let mean = if *n == 0 { f64::NAN } else { e / *n as f64 };
            println!("    d{i}: {mean:.2} ({n} nodes)");
        }
    }
}
