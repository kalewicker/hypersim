//! Throwaway probe: interior (gen>0, degree-2) node energy distribution
//! in a persisted snapshot, to explain why the f_cap fuse guard rejects.
//! ENERGY_PROBE_DIR = a run dir containing snapshots/.

use hg_core::PropertyValue;

#[test]
#[ignore]
fn probe_interior_energies() {
    let dir = std::env::var("ENERGY_PROBE_DIR").expect("ENERGY_PROBE_DIR");
    let snaps =
        hg_eval::cone::load_snapshots(std::path::Path::new(&dir).join("snapshots").as_path())
            .unwrap();
    let (step, store) = snaps.last().expect("snapshot");
    // Degree map.
    let mut degree: std::collections::HashMap<hg_core::NodeId, u32> = Default::default();
    for (_, e) in store.edges() {
        let mut ends: Vec<_> = e.nodes.to_vec();
        ends.sort_unstable();
        ends.dedup();
        for n in ends {
            *degree.entry(n).or_default() += 1;
        }
    }
    let mut energies: Vec<f64> = Vec::new();
    for (id, d) in store.nodes() {
        let gen = matches!(d.get_property(6), Some(PropertyValue::Float(g)) if g > 0.0);
        let deg2 = degree.get(&id).copied() == Some(2);
        if gen && deg2 {
            if let Some(PropertyValue::Float(e)) = d.get_property(1) {
                energies.push(e);
            }
        }
    }
    energies.sort_by(|a, b| a.total_cmp(b));
    let n = energies.len();
    let pct = |p: f64| energies[((n as f64 - 1.0) * p) as usize];
    println!(
        "step {step} | interior gen>0 deg2 nodes = {n} | \
         energy min {:.2} p10 {:.2} p50 {:.2} p90 {:.2} max {:.2}",
        energies.first().copied().unwrap_or(f64::NAN),
        pct(0.10),
        pct(0.50),
        pct(0.90),
        energies.last().copied().unwrap_or(f64::NAN),
    );
    // How many below each candidate f_cap.
    for cap in [10.0, 25.0, 40.0, 49.0, 74.0] {
        let below = energies.iter().filter(|&&e| e < cap).count();
        println!("  below f_cap {cap:.0}: {below}");
    }
}
