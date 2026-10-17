//! Cone analysis (LK2, plan §1.3 / §5): joins the event-level causal
//! export (`causal_events.jsonl`) with per-window snapshots to compare
//! causal reach against metric distance.
//!
//! For each sampled event `e`, the causal future F(e, Δ) is collected by
//! BFS over parent→child links within Δ steps. Each descendant's site
//! nodes are mapped to GEOMETRIC distance from `e`'s site in the snapshot
//! nearest to (and not after) `e.step`; site nodes absent from that
//! snapshot are excluded and the exclusion counted. A descendant's
//! distance is the MAX over its present site nodes (worst case — the
//! conservative choice for locality claims).
//!
//! **Geometric distance (normative clarification over plan v1):** the
//! reference metric is graph distance in the WORMHOLE-CENSORED snapshot —
//! edges whose §1.1 detour exceeds `detour_r_max` are removed before
//! measuring. Raw graph distance would make the instrument vacuous:
//! causal influence propagates along edges of the same graph, so
//! `dist <= dt` identically and a front jumping a planted shortcut is
//! metric-legal (distance 1 through the very shortcut). The plan's own
//! pre-committed acceptance criterion (d) — anomalies monotone in
//! shortcut density — is only satisfiable against the geometric
//! (censored) metric. Cost: one full detour census per distinct
//! reference snapshot (cached).
//!
//! `v_eff` is the through-origin fit of per-lag-bin p95 distance vs step
//! lag, over the dynamics channel only. A descendant is anomalous when
//! its distance exceeds `v_eff · dt + slack` with dt its ACTUAL step lag
//! (the plan's `v_eff·Δ + slack` read as the per-descendant cone
//! condition; using the full window Δ would under-detect early-lag
//! violations — recorded as the implemented reading).
//!
//! Descendants reached through any cleanup-tagged causal edge are
//! reported in the separate cleanup channel (engine bookkeeping, not
//! substrate dynamics — the mandated split).

use hg_causal::{EventGraphLog, EventRecord};
use hg_core::HypergraphStore;
use hg_schema::{ClaimType, ConeRow};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use slotmap::Key;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;

/// What gets censored from the reference metric (v2 §1.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub enum CensorMode {
    /// The wormhole edge set from instrument §1.1 (the science mode).
    Wormhole,
    /// Control C2: censor an EQUAL COUNT of randomly chosen geometric
    /// (non-wormhole) edges instead. Must not manufacture anomalies —
    /// if it does, anomalies are censoring artifacts.
    RandomAblation { seed: u64 },
}

#[derive(Clone, Copy, Debug)]
pub struct ConeParams {
    pub delta: u64,
    pub sample_events: usize,
    pub seed: u64,
    pub slack: f64,
    /// Detour cap for wormhole censoring of the reference metric (same
    /// instrument and default as §1.1).
    pub detour_r_max: usize,
    pub censor: CensorMode,
}

impl Default for ConeParams {
    fn default() -> Self {
        Self {
            delta: 200,
            sample_events: 128,
            seed: 42,
            slack: 2.0,
            detour_r_max: 12,
            censor: CensorMode::Wormhole,
        }
    }
}

/// Per-reference-snapshot censoring state (computed once, cached): the
/// censor set actually applied, its size, and the snapshot's calibrated
/// phase (§1.5) that gates cone emission.
struct SnapshotCensor {
    censor: HashSet<hg_core::EdgeId>,
    phase: hg_schema::LeakagePhase,
}

fn build_censor(
    store: &HypergraphStore,
    detour_r_max: usize,
    mode: CensorMode,
) -> SnapshotCensor {
    let snapshot = store.snapshot();
    // Full wormhole census (exact, not sampled — the censor set must be
    // the real one).
    let wormholes: Vec<hg_core::EdgeId> = store
        .edges()
        .filter(|(id, _)| {
            hg_observe::locality::edge_detour(&snapshot, *id, detour_r_max).is_infinite()
        })
        .map(|(id, _)| id)
        .collect();
    let e_total = store.edge_count().max(1);
    let wormhole_frac = wormholes.len() as f32 / e_total as f32;
    // Phase from the same observables the live instrument uses: exact
    // wormhole_frac plus the cut/defect fit over the d_eff protocol.
    let sources = crate::geometry::sample_sources(&snapshot, 0xC0DE, crate::geometry::DEFF_SOURCES);
    let profiles = hg_observe::ball_profiles(&snapshot, &sources, crate::geometry::DEFF_R_MAX);
    let cut = hg_observe::locality::cut_fit_from_profiles(
        &profiles,
        snapshot.node_count(),
        crate::geometry::DEFF_R_MAX,
        crate::geometry::DEFF_SATURATION_FRAC,
        crate::geometry::DEFF_MIN_FIT_POINTS,
    );
    let d_eff = crate::geometry::dimension_from_profiles(&profiles, snapshot.node_count());
    let defect = match (cut, d_eff) {
        (Some((ce, _)), Some(d)) if d > 0.0 => Some((ce - (d - 1.0) / d) as f32),
        _ => None,
    };
    let phase = hg_observe::locality::classify_phase(
        wormhole_frac,
        defect,
        d_eff.map(|d| d as f32),
    );

    let censor: HashSet<hg_core::EdgeId> = match mode {
        CensorMode::Wormhole => wormholes.into_iter().collect(),
        CensorMode::RandomAblation { seed } => {
            // Equal count of random GEOMETRIC (non-wormhole) edges.
            let wormhole_set: HashSet<_> = wormholes.iter().copied().collect();
            let geometric: Vec<hg_core::EdgeId> = store
                .edges()
                .map(|(id, _)| id)
                .filter(|id| !wormhole_set.contains(id))
                .collect();
            let k = wormholes.len().min(geometric.len());
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            rand::seq::index::sample(&mut rng, geometric.len(), k)
                .iter()
                .map(|ix| geometric[ix])
                .collect()
        }
    };
    SnapshotCensor { censor, phase }
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct ConeAnalysis {
    pub rows: Vec<ConeRow>,
    /// Effective front speed (dynamics channel, sites/step).
    pub v_eff: f64,
    /// Dispersion probe: v_eff fitted per source-event LHS size.
    pub v_eff_by_lhs: Vec<(u32, f64)>,
    pub descendants_dynamics: usize,
    pub descendants_cleanup: usize,
    pub anomalous_dynamics: usize,
    pub anomalous_cleanup: usize,
    pub excluded_descendants: usize,
    /// Suspicious flag (LK5): more cone anomalies attributed to engine
    /// cleanup than to dynamics — the finding is about the engine's
    /// cleanup exception and must be reported as such.
    pub cleanup_channel_dominates: bool,
    /// v2 §1.3: sampled events whose reference window was out of the
    /// sparse-wormhole phase (their rows carry no cone numbers).
    pub out_of_phase_events: usize,
    /// Which censor mode produced these statistics.
    pub censor_mode: CensorMode,
    /// Max censor-set size seen across reference snapshots (C1: must be
    /// 0 on a p = 0 lattice under `Wormhole` mode).
    pub max_censor_set_size: u32,
}

struct Descendant {
    dt: u64,
    /// None = every site node absent from the reference snapshot.
    distance: Option<f64>,
    cleanup: bool,
    excluded_nodes: u32,
}

/// Full analysis over an in-memory log + snapshots (steps ascending).
pub fn analyze(
    log: &EventGraphLog,
    snapshots: &[(u64, HypergraphStore)],
    params: &ConeParams,
) -> ConeAnalysis {
    let events = &log.records;
    // children[parent] -> (child index, via_cleanup)
    let index_of: HashMap<u64, usize> =
        events.iter().enumerate().map(|(i, e)| (e.id, i)).collect();
    let mut children: HashMap<u64, Vec<(usize, bool)>> = HashMap::new();
    for (i, e) in events.iter().enumerate() {
        for &(pid, cleanup) in &e.parents {
            children.entry(pid).or_default().push((i, cleanup));
        }
    }

    // Sample source events: prefer those with a full Δ of future.
    let max_step = events.iter().map(|e| e.step).max().unwrap_or(0);
    let eligible: Vec<usize> = {
        let full_future: Vec<usize> = events
            .iter()
            .enumerate()
            .filter(|(_, e)| e.step + params.delta <= max_step && !e.site.is_empty())
            .map(|(i, _)| i)
            .collect();
        if full_future.is_empty() {
            events
                .iter()
                .enumerate()
                .filter(|(_, e)| !e.site.is_empty())
                .map(|(i, _)| i)
                .collect()
        } else {
            full_future
        }
    };
    let n_sample = params.sample_events.min(eligible.len());
    let mut rng = ChaCha8Rng::seed_from_u64(params.seed);
    let mut sampled: Vec<usize> = rand::seq::index::sample(&mut rng, eligible.len(), n_sample)
        .iter()
        .map(|ix| eligible[ix])
        .collect();
    sampled.sort_unstable();

    // Per sampled event: descendants with distances. Censoring state of
    // each reference snapshot is computed once and cached (censor set +
    // calibrated phase).
    let mut censor_cache: HashMap<usize, SnapshotCensor> = HashMap::new();
    // (source ix, descendants, censor size, phase) per sampled event.
    let mut per_event: Vec<(usize, Vec<Descendant>, u32, hg_schema::LeakagePhase)> =
        Vec::with_capacity(sampled.len());
    for &src_ix in &sampled {
        let src = &events[src_ix];
        let descendants = collect_descendants(events, &children, &index_of, src_ix, params.delta);
        let (ds, censor_size, phase) = measure_descendants(
            src,
            &descendants,
            events,
            snapshots,
            &mut censor_cache,
            params.detour_r_max,
            params.censor,
        );
        per_event.push((src_ix, ds, censor_size, phase));
    }
    let in_phase =
        |phase: &hg_schema::LeakagePhase| *phase == hg_schema::LeakagePhase::Sparse;

    // v_eff: through-origin fit of per-lag-bin p95 distance vs lag,
    // dynamics channel, finite distances only. Phase-gated: out-of-phase
    // windows contribute NOTHING (§1.3).
    let dynamics_pairs: Vec<(u64, f64)> = per_event
        .iter()
        .filter(|(_, _, _, phase)| in_phase(phase))
        .flat_map(|(_, ds, _, _)| ds.iter())
        .filter(|d| !d.cleanup)
        .filter_map(|d| d.distance.map(|dist| (d.dt, dist)))
        .filter(|(_, dist)| dist.is_finite())
        .collect();
    let v_eff = fit_front_speed(&dynamics_pairs, params.delta);

    // Dispersion probe: v_eff by source LHS size (phase-gated likewise).
    let mut by_lhs: HashMap<u32, Vec<(u64, f64)>> = HashMap::new();
    for (src_ix, ds, _, phase) in &per_event {
        if !in_phase(phase) {
            continue;
        }
        let lhs = events[*src_ix].lhs_size;
        let entry = by_lhs.entry(lhs).or_default();
        entry.extend(
            ds.iter()
                .filter(|d| !d.cleanup)
                .filter_map(|d| d.distance.map(|dist| (d.dt, dist)))
                .filter(|(_, dist)| dist.is_finite()),
        );
    }
    let mut v_eff_by_lhs: Vec<(u32, f64)> = by_lhs
        .into_iter()
        .map(|(lhs, pairs)| (lhs, fit_front_speed(&pairs, params.delta)))
        .collect();
    v_eff_by_lhs.sort_unstable_by_key(|(lhs, _)| *lhs);

    // Rows + channel totals. Infinite distance (present but unreachable)
    // is always anomalous. Out-of-phase rows carry the status and NO
    // numbers (zero-filled; a reader that consumes them anyway hits the
    // cone_emitted_out_of_phase flag).
    let mut rows = Vec::with_capacity(per_event.len());
    let (mut n_dyn, mut n_cln, mut a_dyn, mut a_cln, mut excluded) = (0, 0, 0, 0, 0);
    let mut out_of_phase_events = 0usize;
    let mut max_censor = 0u32;
    for (src_ix, ds, censor_size, phase) in &per_event {
        let src = &events[*src_ix];
        max_censor = max_censor.max(*censor_size);
        if !in_phase(phase) {
            out_of_phase_events += 1;
            rows.push(ConeRow {
                step: src.step,
                event_id: src.id,
                lhs_size: src.lhs_size,
                delta: params.delta,
                censor_set_size: *censor_size,
                cone_status: hg_schema::ConeStatus::OutOfPhase,
                claim_type: ClaimType::Estimated,
                ..Default::default()
            });
            continue;
        }
        let mut dyn_total = 0usize;
        let mut dyn_anom = 0usize;
        let mut cln_total = 0usize;
        let mut cln_anom = 0usize;
        let mut dists: Vec<f64> = Vec::new();
        let mut row_excluded = 0u32;
        for d in ds {
            row_excluded += d.excluded_nodes;
            let Some(dist) = d.distance else {
                continue; // fully excluded descendant
            };
            let anomalous =
                !dist.is_finite() || dist > v_eff * d.dt as f64 + params.slack;
            if d.cleanup {
                cln_total += 1;
                cln_anom += anomalous as usize;
            } else {
                dyn_total += 1;
                dyn_anom += anomalous as usize;
                if dist.is_finite() {
                    dists.push(dist);
                }
            }
        }
        n_dyn += dyn_total;
        n_cln += cln_total;
        a_dyn += dyn_anom;
        a_cln += cln_anom;
        excluded += row_excluded as usize;
        rows.push(ConeRow {
            step: src.step,
            event_id: src.id,
            lhs_size: src.lhs_size,
            delta: params.delta,
            cone_radius_p95: percentile(&mut dists, 0.95) as f32,
            anomalous_frac_dynamics: frac(dyn_anom, dyn_total),
            anomalous_frac_cleanup: frac(cln_anom, cln_total),
            excluded_descendants: row_excluded,
            censor_set_size: *censor_size,
            cone_status: hg_schema::ConeStatus::InPhase,
            claim_type: ClaimType::Estimated,
        });
    }

    ConeAnalysis {
        rows,
        v_eff,
        v_eff_by_lhs,
        descendants_dynamics: n_dyn,
        descendants_cleanup: n_cln,
        anomalous_dynamics: a_dyn,
        anomalous_cleanup: a_cln,
        excluded_descendants: excluded,
        cleanup_channel_dominates: a_cln > a_dyn,
        out_of_phase_events,
        censor_mode: params.censor,
        max_censor_set_size: max_censor,
    }
}

fn frac(num: usize, den: usize) -> f32 {
    if den == 0 {
        0.0
    } else {
        num as f32 / den as f32
    }
}

fn percentile(vals: &mut [f64], q: f64) -> f64 {
    if vals.is_empty() {
        return f64::NAN;
    }
    vals.sort_by(|a, b| a.total_cmp(b));
    let rank = ((q * vals.len() as f64).ceil() as usize).clamp(1, vals.len());
    vals[rank - 1]
}

/// Through-origin least squares of p95-per-lag-bin vs lag midpoint.
fn fit_front_speed(pairs: &[(u64, f64)], delta: u64) -> f64 {
    if pairs.is_empty() {
        return 0.0;
    }
    let n_bins = 10usize.min(delta.max(1) as usize);
    let bin_w = (delta.max(1) as f64 / n_bins as f64).max(1.0);
    let mut bins: Vec<Vec<f64>> = vec![Vec::new(); n_bins];
    for &(dt, dist) in pairs {
        let b = (((dt as f64 - 1.0) / bin_w) as usize).min(n_bins - 1);
        bins[b].push(dist);
    }
    let mut sxy = 0.0;
    let mut sxx = 0.0;
    for (b, mut vals) in bins.into_iter().enumerate() {
        if vals.is_empty() {
            continue;
        }
        let x = (b as f64 + 0.5) * bin_w; // bin midpoint lag
        let y = percentile(&mut vals, 0.95);
        sxy += x * y;
        sxx += x * x;
    }
    if sxx <= f64::EPSILON {
        0.0
    } else {
        sxy / sxx
    }
}

fn collect_descendants(
    events: &[EventRecord],
    children: &HashMap<u64, Vec<(usize, bool)>>,
    _index_of: &HashMap<u64, usize>,
    src_ix: usize,
    delta: u64,
) -> Vec<(usize, bool)> {
    // BFS over child links; taint = reached through any cleanup edge.
    // A node reachable both ways keeps the dynamics (false) taint.
    let src_step = events[src_ix].step;
    let mut best: HashMap<usize, bool> = HashMap::new();
    let mut queue: VecDeque<(usize, bool)> = VecDeque::new();
    queue.push_back((src_ix, false));
    while let Some((ix, taint)) = queue.pop_front() {
        if let Some(kids) = children.get(&events[ix].id) {
            for &(child_ix, via_cleanup) in kids {
                if events[child_ix].step > src_step + delta {
                    continue;
                }
                let child_taint = taint || via_cleanup;
                match best.get(&child_ix) {
                    Some(&t) if !t || t == child_taint => {} // already as good
                    _ => {
                        best.insert(child_ix, child_taint);
                        queue.push_back((child_ix, child_taint));
                    }
                }
            }
        }
    }
    let mut out: Vec<(usize, bool)> = best.into_iter().collect();
    out.sort_unstable();
    out
}

#[allow(clippy::too_many_arguments)]
fn measure_descendants(
    src: &EventRecord,
    descendants: &[(usize, bool)],
    events: &[EventRecord],
    snapshots: &[(u64, HypergraphStore)],
    censor_cache: &mut HashMap<usize, SnapshotCensor>,
    detour_r_max: usize,
    censor_mode: CensorMode,
) -> (Vec<Descendant>, u32, hg_schema::LeakagePhase) {
    // Nearest-earlier snapshot policy (§1.3): last snapshot with
    // step <= src.step; if none, the earliest available.
    let snap_ix = snapshots
        .iter()
        .rposition(|(s, _)| *s <= src.step)
        .or(if snapshots.is_empty() { None } else { Some(0) });
    let Some(snap_ix) = snap_ix else {
        return (Vec::new(), 0, hg_schema::LeakagePhase::Crossover);
    };
    let store = &snapshots[snap_ix].1;

    // Geometric metric: censor wormhole edges (detour > cap) so the
    // reference distance is the LOCAL geometry, not "whatever edge
    // happens to exist" (see module docs). Exact census + phase, cached
    // per snapshot; C2 swaps the censor for random geometric edges.
    let censor_state = censor_cache
        .entry(snap_ix)
        .or_insert_with(|| build_censor(store, detour_r_max, censor_mode));
    let censor_size = censor_state.censor.len() as u32;
    let phase = censor_state.phase;
    let wormholes = &censor_state.censor;

    // Key -> NodeId for this snapshot.
    let key_to_node: HashMap<u64, hg_core::NodeId> = store
        .nodes()
        .map(|(id, _)| (id.data().as_ffi(), id))
        .collect();

    // Multi-source BFS from the source event's present site nodes,
    // stopping once every needed target key is resolved.
    let needed: HashSet<u64> = descendants
        .iter()
        .flat_map(|(ix, _)| events[*ix].site.iter().copied())
        .collect();
    let mut dist: HashMap<hg_core::NodeId, u64> = HashMap::new();
    let mut queue: VecDeque<hg_core::NodeId> = VecDeque::new();
    for key in &src.site {
        if let Some(&id) = key_to_node.get(key) {
            dist.insert(id, 0);
            queue.push_back(id);
        }
    }
    let mut remaining: usize = needed
        .iter()
        .filter(|k| key_to_node.contains_key(*k) && !src.site.contains(k))
        .count();
    while let Some(v) = queue.pop_front() {
        if remaining == 0 {
            break;
        }
        let d = dist[&v];
        for edge_id in store.edges_incident_to(v) {
            if wormholes.contains(edge_id) {
                continue; // censored: not part of the geometric metric
            }
            if let Some(edge) = store.get_edge(*edge_id) {
                for &nb in &edge.nodes {
                    if let std::collections::hash_map::Entry::Vacant(e) = dist.entry(nb) {
                        if store.get_node(nb).is_some() {
                            e.insert(d + 1);
                            if needed.contains(&nb.data().as_ffi()) {
                                remaining = remaining.saturating_sub(1);
                            }
                            queue.push_back(nb);
                        }
                    }
                }
            }
        }
    }

    let measured = descendants
        .iter()
        .map(|&(ix, cleanup)| {
            let e = &events[ix];
            let mut excluded = 0u32;
            let mut worst: Option<f64> = None;
            for key in &e.site {
                match key_to_node.get(key) {
                    None => excluded += 1, // absent from reference snapshot
                    Some(id) => {
                        let d = dist
                            .get(id)
                            .map(|&d| d as f64)
                            .unwrap_or(f64::INFINITY); // present, unreachable
                        worst = Some(worst.map_or(d, |w: f64| w.max(d)));
                    }
                }
            }
            Descendant {
                dt: e.step.saturating_sub(src.step),
                distance: worst,
                cleanup,
                excluded_nodes: excluded,
            }
        })
        .collect();
    (measured, censor_size, phase)
}

/// Load `causal_events.jsonl` + `snapshots/` from a run directory and
/// analyze. Snapshots must have been enabled on the run.
pub fn analyze_run_dir(run_dir: &Path, params: &ConeParams) -> std::io::Result<ConeAnalysis> {
    let jsonl = std::fs::read_to_string(run_dir.join("causal_events.jsonl"))?;
    let log = EventGraphLog::from_jsonl(&jsonl).map_err(std::io::Error::other)?;
    let snapshots = load_snapshots(&run_dir.join("snapshots"))?;
    if snapshots.is_empty() {
        return Err(std::io::Error::other(
            "cone analysis requires snapshots (set [output] snapshots_enabled = true)",
        ));
    }
    Ok(analyze(&log, &snapshots, params))
}

/// Write cone.csv (CsvWriter discipline, one per run) and
/// cone_summary.json into the run directory.
pub fn write_outputs(run_dir: &Path, analysis: &ConeAnalysis) -> std::io::Result<()> {
    let mut csv: hg_output::CsvWriter<ConeRow> = hg_output::CsvWriter::create(
        &run_dir.join("cone.csv"),
        1,
        &run_dir
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default(),
        0,
        "",
    )?;
    for row in &analysis.rows {
        csv.write_row(row)?;
    }
    csv.flush()?;
    std::fs::write(
        run_dir.join("cone_summary.json"),
        serde_json::to_string_pretty(analysis).map_err(std::io::Error::other)?,
    )?;
    Ok(())
}

/// All snapshots in a run's snapshots/ dir, ascending by step.
pub fn load_snapshots(dir: &Path) -> std::io::Result<Vec<(u64, HypergraphStore)>> {
    let mut out = Vec::new();
    if !dir.exists() {
        return Ok(out);
    }
    let mut paths: Vec<_> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "bin"))
        .collect();
    paths.sort();
    for p in paths {
        let (step, payload) = hg_output::SnapshotReader::read_snapshot(&p)?;
        let store: HypergraphStore =
            bincode::deserialize(&payload).map_err(std::io::Error::other)?;
        out.push((step, store));
    }
    Ok(out)
}
