//! A4 mechanism probe: measure the EFFECTIVE TICK RATE ρ directly.
//!
//! A0 question 2 flagged in advance that a single-node `lk_tick` binding
//! can lose its node to a competing EDGE-rule binding (transport /
//! subdivide / fuse all claim nodes) in the same MaxParallel step, since
//! conflict resolution is node-disjoint across ALL rules. If a filament
//! node is ticked only a fraction ρ of steps, its realized lifetime is
//! k/ρ, not k — and the standing population becomes L* = k·p/ρ (the
//! formula plan v6 §4.3 wrote for the degraded fallback, applying to the
//! primary mechanism too).
//!
//! Measurement: for every node present in two consecutive snapshots with
//! a live countdown in both, ρ = (countdown drop) / (steps elapsed),
//! aggregated. No inference from a curve fit — this reads the clock.
//!
//! L17_RUN_DIRS = colon-separated run dirs containing snapshots/.

use hg_core::{NodeId, PropertyValue};
use std::collections::HashMap;

const COUNTDOWN: u32 = 7;

#[test]
#[ignore]
fn l17_effective_tick_rate() {
    let dirs = std::env::var("L17_RUN_DIRS").expect("L17_RUN_DIRS=dir1:dir2:…");
    println!("run | paired observations | mean ρ (ticks/step) | implied lifetime k/ρ");
    for dir in dirs.split(':').filter(|s| !s.is_empty()) {
        let snaps =
            hg_eval::cone::load_snapshots(std::path::Path::new(dir).join("snapshots").as_path())
                .unwrap_or_else(|e| panic!("{dir}: {e}"));
        let live = |store: &hg_core::HypergraphStore| -> HashMap<NodeId, i64> {
            store
                .nodes()
                .filter_map(|(id, d)| match d.get_property(COUNTDOWN) {
                    Some(PropertyValue::Int(c)) if c > 0 => Some((id, c)),
                    _ => None,
                })
                .collect()
        };
        let mut total_drop = 0i64;
        let mut total_steps = 0i64;
        let mut pairs = 0usize;
        for w in snaps.windows(2) {
            let (s0, st0) = (&w[0].0, &w[0].1);
            let (s1, st1) = (&w[1].0, &w[1].1);
            let dt = (*s1 as i64) - (*s0 as i64);
            if dt <= 0 {
                continue;
            }
            let (a, b) = (live(st0), live(st1));
            for (id, c0) in &a {
                // Only nodes still alive with a live countdown in both
                // frames: a node that hit 0 (or fused) would clip the
                // drop and bias ρ downward.
                if let Some(c1) = b.get(id) {
                    total_drop += c0 - c1;
                    total_steps += dt;
                    pairs += 1;
                }
            }
        }
        let rho = if total_steps > 0 {
            total_drop as f64 / total_steps as f64
        } else {
            f64::NAN
        };
        let name = std::path::Path::new(dir)
            .parent()
            .and_then(|p| p.file_name())
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| dir.into());
        println!("{name} | {pairs} | {rho:.3} | {:.1}×k", 1.0 / rho);
    }
}
