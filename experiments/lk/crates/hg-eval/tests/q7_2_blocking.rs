//! Q7-2 (v7 §3): tick-blocking audit — the DIRECTIONAL half of the
//! contention question that A0 did not audit.
//!
//! A0 asked "can a tick lose its node to an edge rule?" and measured
//! ρ = 1.000 (never). F-LK22 asks the OTHER direction: `lk_tick` has a
//! single-node LHS and therefore CLAIMS the node it decrements, at
//! priority 2; `lk_fuse` claims {x, w, y} at priority −1. So whenever a
//! neighbour of an expired node is still counting down, the tick wins
//! and the fuse is refused. Removal should then happen preferentially at
//! chain ENDS (where a neighbour is a gen-0 anchor with no countdown key
//! and so is never tick-eligible).
//!
//! ρ = 1.000 is evidence FOR this, not against it: tick never loses,
//! which is exactly why fuse loses when they contend.
//!
//! Signature measured here, per era window:
//!   * expired  = gen>0 ∧ countdown == 0            (removal-ready by age)
//!   * eligible = expired ∧ degree == 2             (removal-ready by shape)
//!   * BLOCKED  = eligible ∧ some neighbour is tick-eligible (countdown > 0)
//!   * FREE     = eligible ∧ no neighbour is tick-eligible
//! A large, persistent BLOCKED population is the blocking signature; FREE
//! nodes should be at chain ends (≥1 gen-0 neighbour).
//!
//! Q7_RUN_DIRS = colon-separated run dirs.

use hg_core::{HypergraphStore, NodeId, PropertyValue};
use std::collections::HashMap;

const GENERATION: u32 = 6;
const COUNTDOWN: u32 = 7;

struct Frame {
    filament: usize,
    expired: usize,
    eligible: usize,
    blocked: usize,
    free: usize,
    free_at_end: usize,
}

fn frame(store: &HypergraphStore) -> Frame {
    let mut degree: HashMap<NodeId, u32> = HashMap::new();
    let mut adjacency: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
    for (_, edge) in store.edges() {
        let mut ends: Vec<NodeId> = edge.nodes.to_vec();
        ends.sort_unstable();
        ends.dedup();
        for &u in &ends {
            *degree.entry(u).or_default() += 1;
            for &v in &ends {
                if v != u {
                    adjacency.entry(u).or_default().push(v);
                }
            }
        }
    }
    let gen_of = |id: NodeId| -> f64 {
        match store.get_node(id).and_then(|d| d.get_property(GENERATION)) {
            Some(PropertyValue::Float(g)) => g,
            _ => 0.0,
        }
    };
    // Tick-eligible = the lk_tick guard `countdown > 0` passes.
    let tick_eligible = |id: NodeId| -> bool {
        matches!(
            store.get_node(id).and_then(|d| d.get_property(COUNTDOWN)),
            Some(PropertyValue::Int(c)) if c > 0
        )
    };
    let mut f = Frame { filament: 0, expired: 0, eligible: 0, blocked: 0, free: 0, free_at_end: 0 };
    for (id, d) in store.nodes() {
        if gen_of(id) <= 0.0 {
            continue;
        }
        f.filament += 1;
        let expired = matches!(d.get_property(COUNTDOWN), Some(PropertyValue::Int(0)));
        if !expired {
            continue;
        }
        f.expired += 1;
        if degree.get(&id).copied() != Some(2) {
            continue;
        }
        f.eligible += 1;
        let nbrs: Vec<NodeId> = adjacency.get(&id).cloned().unwrap_or_default();
        if nbrs.iter().any(|&n| tick_eligible(n)) {
            f.blocked += 1;
        } else {
            f.free += 1;
            if nbrs.iter().any(|&n| gen_of(n) <= 0.0) {
                f.free_at_end += 1;
            }
        }
    }
    f
}

#[test]
#[ignore]
fn q7_2_blocking_signature() {
    let dirs = std::env::var("Q7_RUN_DIRS").expect("Q7_RUN_DIRS=dir1:dir2:…");
    println!(
        "cell | era_w | filament | expired | eligible | BLOCKED | FREE | free@end | \
         blocked/eligible | expired/filament"
    );
    for dir in dirs.split(':').filter(|s| !s.is_empty()) {
        let path = std::path::Path::new(dir);
        let snaps = hg_eval::cone::load_snapshots(path.join("snapshots").as_path()).unwrap();
        let leak = std::fs::read_to_string(path.join("leakage.csv")).unwrap_or_default();
        let lrows: Vec<(u64, f64)> = leak
            .lines()
            .skip(2)
            .filter_map(|l| {
                let f: Vec<&str> = l.split(',').collect();
                Some((f.first()?.parse().ok()?, f.get(15)?.parse().ok()?))
            })
            .collect();
        let era: std::collections::HashSet<u64> = (1..lrows.len())
            .filter(|&i| lrows[i].1 > lrows[i - 1].1)
            .map(|i| lrows[i].0)
            .collect();
        let frames: Vec<Frame> = snaps
            .iter()
            .filter(|(s, _)| era.is_empty() || era.contains(s))
            .map(|(_, st)| frame(st))
            .collect();
        if frames.is_empty() {
            continue;
        }
        let n = frames.len() as f64;
        let mean = |f: &dyn Fn(&Frame) -> f64| frames.iter().map(f).sum::<f64>() / n;
        let fil = mean(&|f| f.filament as f64);
        let exp = mean(&|f| f.expired as f64);
        let elig = mean(&|f| f.eligible as f64);
        let blk = mean(&|f| f.blocked as f64);
        let fre = mean(&|f| f.free as f64);
        let fend = mean(&|f| f.free_at_end as f64);
        let name = path
            .parent()
            .and_then(|p| p.file_name())
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| dir.into());
        println!(
            "{name} | {} | {fil:.1} | {exp:.1} | {elig:.1} | {blk:.1} | {fre:.1} | {fend:.1} | {} | {}",
            frames.len(),
            if elig > 0.0 { format!("{:.0}%", blk / elig * 100.0) } else { "n/a".into() },
            if fil > 0.0 { format!("{:.0}%", exp / fil * 100.0) } else { "n/a".into() },
        );
    }
}
