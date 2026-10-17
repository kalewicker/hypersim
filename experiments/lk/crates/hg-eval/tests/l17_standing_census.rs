//! A4 primary readout (offline): the STANDING FILAMENT POPULATION per
//! window, read directly off the persisted snapshots as the count of
//! emergent nodes (generation > 0).
//!
//! Why not `wormhole_count_est`: that census has a DETECTION THRESHOLD.
//! An interior link registers as a wormhole only once its detour 2L+2
//! exceeds R_max = 12 (the Q0/F-LK14 arithmetic), i.e. only on chains of
//! length ≥ 6. A standing population of k = 10 links spread over the
//! pump's 6 chains is ~1.7 links/chain — structurally present, entirely
//! invisible to the wormhole instrument. Reading L* off that census
//! would report "suppressed" for any coexistence state below the
//! threshold. The generation > 0 count has no threshold and is exactly
//! the quantity L* = k·p predicts (one node born per subdivision, each
//! living k steps).
//!
//! L17_RUN_DIRS = colon-separated run dirs (each containing snapshots/).
//! Prints, per run, the standing census over the production era.

use hg_core::PropertyValue;

const GENERATION: u32 = 6;
const COUNTDOWN: u32 = 7;

#[test]
#[ignore]
fn l17_standing_census() {
    let dirs = std::env::var("L17_RUN_DIRS").expect("L17_RUN_DIRS=dir1:dir2:…");
    println!("run | windows | L*(gen>0 mean±sd over era) | L_peak | live_cd mean | era_w");
    for dir in dirs.split(':').filter(|s| !s.is_empty()) {
        let snaps = hg_eval::cone::load_snapshots(std::path::Path::new(dir).join("snapshots").as_path())
            .unwrap_or_else(|e| panic!("{dir}: {e}"));
        if snaps.is_empty() {
            println!("{dir} | (no snapshots)");
            continue;
        }
        // Per-window standing population and live-countdown mean.
        let series: Vec<(u64, usize, f64)> = snaps
            .iter()
            .map(|(step, store)| {
                let mut n = 0usize;
                let mut cds: Vec<i64> = Vec::new();
                for (_, d) in store.nodes() {
                    if matches!(d.get_property(GENERATION), Some(PropertyValue::Float(g)) if g > 0.0)
                    {
                        n += 1;
                        if let Some(PropertyValue::Int(c)) = d.get_property(COUNTDOWN) {
                            if c > 0 {
                                cds.push(c);
                            }
                        }
                    }
                }
                let cd_mean = if cds.is_empty() {
                    0.0
                } else {
                    cds.iter().sum::<i64>() as f64 / cds.len() as f64
                };
                (*step, n, cd_mean)
            })
            .collect();
        // PRODUCTION ERA, read from leakage.csv so this census and the
        // driver's p refer to the SAME window (an earlier version used
        // its own heuristic era here, which included post-quench drain
        // windows and inflated L* by ~2× relative to a p measured only
        // over the production segment — the two numbers were not
        // comparable, and L*/p looked like 2k when the tick rate is
        // exactly 1.0). Era = steps where n_subdivisions is rising.
        let leak = std::fs::read_to_string(std::path::Path::new(dir).join("leakage.csv"))
            .unwrap_or_default();
        let lrows: Vec<(u64, f64)> = leak
            .lines()
            .skip(2)
            .filter_map(|l| {
                let f: Vec<&str> = l.split(',').collect();
                Some((f.first()?.parse().ok()?, f.get(15)?.parse().ok()?))
            })
            .collect();
        let era_steps: std::collections::HashSet<u64> = (1..lrows.len())
            .filter(|&i| lrows[i].1 > lrows[i - 1].1)
            .map(|i| lrows[i].0)
            .collect();
        let era: Vec<(u64, usize, f64)> = if era_steps.is_empty() {
            series.clone()
        } else {
            series
                .iter()
                .copied()
                .filter(|(s, _, _)| era_steps.contains(s))
                .collect()
        };
        let vals: Vec<f64> = era.iter().map(|&(_, n, _)| n as f64).collect();
        let m = vals.iter().sum::<f64>() / vals.len().max(1) as f64;
        let sd =
            (vals.iter().map(|x| (x - m).powi(2)).sum::<f64>() / vals.len().max(1) as f64).sqrt();
        let peak = vals.iter().copied().fold(0.0f64, f64::max);
        let cdm = era.iter().map(|&(_, _, c)| c).sum::<f64>() / era.len().max(1) as f64;
        let name = std::path::Path::new(dir)
            .parent()
            .and_then(|p| p.file_name())
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| dir.into());
        println!(
            "{name} | {} | {m:.1}±{sd:.1} | {peak:.0} | {cdm:.1} | {}",
            series.len(),
            era.len()
        );
    }
}
