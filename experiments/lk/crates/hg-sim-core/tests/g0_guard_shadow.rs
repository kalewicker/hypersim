//! G0.1 — the guard-shadow audit (v10 §2.1, F-LK33).
//!
//! v10 §1.4 makes guard PAIRS auditable: where two guards can reject the
//! same binding, the counters must show which rejects first and whether
//! either is unreachable. `lk_fuse` carries exactly such a pair — a
//! generation guard (`w.generation >= 1`, prop0) and a degree guard
//! (`degree(w) == 2`, deg0) — and the pair has never been audited as a
//! pair.
//!
//! F-LK33 as written predicted that on clean-lattice arms the *generation*
//! guard is shadowed, because a gen-0 node on a triangulated torus has
//! degree 6 and the degree guard would reject it first. Half of that is
//! right and half is backwards, and the halves must be kept apart:
//!
//!   * ORDER (mechanism) — REFUTED. `bind_free_vars` chains the guard
//!     families with `&&`: property → compares → degree → age
//!     (matcher.rs). The generation guard is a `node_property_guard`, so
//!     it runs FIRST; the degree guard only ever sees bindings generation
//!     already admitted. The shadowing runs the opposite way from the
//!     prediction, and `guards.csv` says so at every window of every run.
//!
//!   * REACHABILITY (semantics) — CONFIRMED. Whether the generation guard
//!     is *load-bearing* is a different question, and the short-circuit
//!     counters cannot answer it: they only ever report the first
//!     rejector. The question is a counterfactual — of the bindings
//!     generation rejects, would degree have rejected them too? — and it
//!     reduces to one measurable predicate: does any gen-0 node ever reach
//!     degree 2? Answered here off the persisted snapshots, the only place
//!     the counterfactual is visible.
//!
//! Degree is computed exactly as `check_degree_compares` computes it —
//! count of DISTINCT incident hyperedges (v4 §6.2, pinned) — so this is
//! the guard's own predicate, not a re-definition of it.
//!
//! SCOPE, and why it is not a convenience. A missing generation property
//! FAILS the guard `gte 1.0`, so for the fuse rule "absent" and "gen-0"
//! are the same thing. But for THIS audit they are not: a v3-era snapshot
//! that predates the generation stamp reports its entire emergent filament
//! as skeleton, and a filament midpoint is degree 2 by construction. The
//! first version of this test read the whole archive and fired the §2.1
//! alternative on 3.5M nodes; the attribution was exactly two families
//! (`expL_v2_archive` and `exp_l10`, both with ZERO nodes carrying the
//! property, and `exp_l12`, partially stamped) and the count in the
//! stamped fraction was zero. So the predicate is evaluated only where the
//! property exists, the unstamped population is reported as the audit's
//! blind spot rather than silently dropped, and the split is asserted so
//! it cannot drift.
//!
//! INSTRUMENT-RANGE PRECONDITION (carried hazard, v10 §9.7): a scan that
//! found no snapshots would "pass" while measuring nothing — the vacuous-
//! null failure the track has caught twice. The test asserts nonzero
//! snapshots in BOTH populations, a nonempty emergent positive control,
//! and a nonempty rejected-by-generation population. Absent archives fail
//! rather than skip.

use hg_core::{EdgeId, HypergraphStore, NodeId, PropertyValue};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const GENERATION: u32 = 6;

fn runs_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../runs")
}

/// Distinct incident hyperedges — byte-for-byte the guard's own predicate
/// (`check_degree_compares`, matcher.rs), not an independent definition.
fn degree(store: &HypergraphStore, id: NodeId) -> usize {
    let mut distinct: Vec<EdgeId> = Vec::new();
    for e in store.edges_incident_to(id) {
        if !distinct.contains(e) {
            distinct.push(*e);
        }
    }
    distinct.len()
}

/// `None` = the node carries no generation property at all (v3-era
/// snapshot): unmeasurable for this audit. `Some(g)` = the stamped value.
fn generation_of(store: &HypergraphStore, id: NodeId) -> Option<f64> {
    match store.get_node(id).and_then(|d| d.get_property(GENERATION)) {
        Some(PropertyValue::Float(g)) => Some(g),
        Some(PropertyValue::Int(g)) => Some(g as f64),
        _ => None,
    }
}

#[derive(Default, Debug, Clone)]
struct Tally {
    snapshots: usize,
    /// Nodes with no generation property — the audit's blind spot.
    unstamped: u64,
    gen0_nodes: u64,
    gen1_nodes: u64,
    /// THE counterfactual, on stamped nodes only: gen-0 nodes the degree
    /// guard would have admitted had generation not rejected them first.
    gen0_at_degree2: u64,
    /// Emergent nodes the degree guard would reject (branch points/tips).
    gen1_off_degree2: u64,
    gen0_degrees: BTreeMap<usize, u64>,
    gen1_degrees: BTreeMap<usize, u64>,
    /// Degree spectrum of the UNSTAMPED nodes, so the blind spot is
    /// described rather than merely counted.
    unstamped_degrees: BTreeMap<usize, u64>,
}

impl Tally {
    fn merge(&mut self, o: &Tally) {
        self.snapshots += o.snapshots;
        self.unstamped += o.unstamped;
        self.gen0_nodes += o.gen0_nodes;
        self.gen1_nodes += o.gen1_nodes;
        self.gen0_at_degree2 += o.gen0_at_degree2;
        self.gen1_off_degree2 += o.gen1_off_degree2;
        for (dst, src) in [
            (&mut self.gen0_degrees, &o.gen0_degrees),
            (&mut self.gen1_degrees, &o.gen1_degrees),
            (&mut self.unstamped_degrees, &o.unstamped_degrees),
        ] {
            for (k, v) in src {
                *dst.entry(*k).or_default() += v;
            }
        }
    }
}

fn scan(dir: &Path) -> Tally {
    let mut t = Tally::default();
    let snaps = hg_eval::cone::load_snapshots(dir).unwrap_or_default();
    for (_step, store) in &snaps {
        t.snapshots += 1;
        for (id, _) in store.nodes() {
            let d = degree(store, id);
            match generation_of(store, id) {
                None => {
                    t.unstamped += 1;
                    *t.unstamped_degrees.entry(d).or_default() += 1;
                }
                Some(g) if g > 0.0 => {
                    t.gen1_nodes += 1;
                    *t.gen1_degrees.entry(d).or_default() += 1;
                    if d != 2 {
                        t.gen1_off_degree2 += 1;
                    }
                }
                Some(_) => {
                    t.gen0_nodes += 1;
                    *t.gen0_degrees.entry(d).or_default() += 1;
                    if d == 2 {
                        t.gen0_at_degree2 += 1;
                    }
                }
            }
        }
    }
    t
}

fn snapshot_dirs() -> Vec<PathBuf> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(rd) = std::fs::read_dir(dir) else { return };
        for e in rd.filter_map(|e| e.ok()) {
            let p = e.path();
            if !p.is_dir() {
                continue;
            }
            if p.file_name().is_some_and(|n| n == "snapshots") {
                out.push(p);
            } else {
                walk(&p, out);
            }
        }
    }
    let mut out = Vec::new();
    walk(&runs_root(), &mut out);
    out.sort();
    out
}

/// The audit. Reports the two populations §2.1 requires separately
/// (clean-lattice vs planted-handle): a planted shortcut raises its gen-0
/// endpoints' degree to 7-8, so the two have different degree spectra and
/// only the predicate — never degree 2 — is shared.
#[test]
fn g0_1_generation_guard_is_semantically_shadowed_by_degree() {
    let dirs = snapshot_dirs();
    let (mut clean, mut planted) = (Tally::default(), Tally::default());
    let (mut n_clean, mut n_planted) = (0usize, 0usize);
    let mut per_family: BTreeMap<String, Tally> = BTreeMap::new();

    for d in &dirs {
        let s = d.to_string_lossy().to_string();
        let t = scan(d);
        if t.snapshots == 0 {
            continue;
        }
        let family =
            s.split("runs/").nth(1).and_then(|r| r.split('/').next()).unwrap_or("?").to_string();
        per_family.entry(family).or_default().merge(&t);
        if s.contains("planted") {
            planted.merge(&t);
            n_planted += 1;
        } else {
            clean.merge(&t);
            n_clean += 1;
        }
    }

    println!("=== G0.1 guard-shadow audit (v10 §2.1) ===\n--- per run family ---");
    for (f, t) in &per_family {
        println!(
            "  {f:<20} snaps={:<6} stamped gen0={:<9} gen>0={:<9} gen0@deg2={:<8} UNSTAMPED={:<9} {:?}",
            t.snapshots,
            t.gen0_nodes,
            t.gen1_nodes,
            t.gen0_at_degree2,
            t.unstamped,
            t.unstamped_degrees
        );
    }
    for (name, t, n) in [("clean-lattice", &clean, n_clean), ("planted-handle", &planted, n_planted)]
    {
        println!(
            "\n{name}: {n} run dirs, {} snapshots\n  stamped gen-0 {} | degrees {:?}\n  stamped gen>0 {} | degrees {:?}\n  gen-0 AT degree 2 (would pass the degree guard): {}\n  gen>0 OFF degree 2 (degree guard would reject):  {}\n  unstamped (blind spot): {}",
            t.snapshots,
            t.gen0_nodes,
            t.gen0_degrees,
            t.gen1_nodes,
            t.gen1_degrees,
            t.gen0_at_degree2,
            t.gen1_off_degree2,
            t.unstamped,
        );
    }

    // ---- instrument-range preconditions (M2.3 discipline, carried) ----
    assert!(
        clean.snapshots > 0 && planted.snapshots > 0,
        "instrument out of range: clean={} planted={} snapshots — the audit \
         needs BOTH populations on disk (v10 §2.1)",
        clean.snapshots,
        planted.snapshots
    );
    assert!(
        clean.gen0_nodes > 0 && planted.gen0_nodes > 0,
        "instrument out of range: no STAMPED gen-0 nodes examined"
    );
    assert!(
        clean.gen1_nodes > 0,
        "instrument out of range: no gen>0 nodes in any clean snapshot — \
         nothing for the generation guard to distinguish from the skeleton"
    );

    // ---- the result ----
    // No stamped gen-0 node, in either population, ever reaches the degree
    // the fuse rule requires. Deleting the generation guard would admit
    // nothing new: it is semantically redundant, and the v4 §1.4
    // non-removal guarantee is the DEGREE guard's.
    assert_eq!(
        clean.gen0_at_degree2, 0,
        "a stamped gen-0 node reached degree 2 on a clean-lattice arm — the \
         generation guard is NOT shadowed and §2.1's alternative fires: the \
         lattice has been locally eroded, diagnose before any G3 work"
    );
    assert_eq!(
        planted.gen0_at_degree2, 0,
        "a stamped gen-0 node reached degree 2 on a planted-handle arm"
    );

    // The blind spot is bounded and attributed, not waved away: only
    // families that predate the generation stamp carry unstamped nodes.
    let leaky: Vec<&String> = per_family
        .iter()
        .filter(|(_, t)| t.unstamped > 0)
        .map(|(f, _)| f)
        .filter(|f| !matches!(f.as_str(), "expL_v2_archive" | "exp_l10" | "exp_l12"))
        .collect();
    assert!(
        leaky.is_empty(),
        "unstamped nodes appeared in a family that should carry the \
         generation property: {leaky:?} — the audit's scope statement is stale"
    );
}

/// The companion half, from `guards.csv`: the ORDER claim. The degree
/// guard has never rejected a binding anywhere in the archive, and in the
/// f_cap arms it is never even evaluated (prop1 short-circuits ahead of
/// it). So the inert instrument of the pair is the DEGREE guard — the
/// exact inverse of F-LK33's prediction — while F-LK33's conclusion, that
/// the generation guard is not what protects the lattice, survives via the
/// snapshot test above.
///
/// The second assertion is the mechanism behind that result: the
/// generation guard's rejects-per-window are EXACTLY constant in every
/// run, so the gen-0 skeleton's local edge structure never changes. No
/// rule in the family can change it — subdivide preserves endpoint degree,
/// fuse removes a midpoint and rejoins its endpoints (net zero), transport
/// is topology-inert — and the counters confirm it to the unit.
#[test]
fn g0_1_degree_guard_never_rejects_and_is_sometimes_unreachable() {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(rd) = std::fs::read_dir(dir) else { return };
        for e in rd.filter_map(|e| e.ok()) {
            let p = e.path();
            if p.is_dir() {
                walk(&p, out);
            } else if p.file_name().is_some_and(|n| n == "guards.csv") {
                out.push(p);
            }
        }
    }
    let mut files = Vec::new();
    walk(&runs_root(), &mut files);
    files.sort();

    let (mut runs, mut unreachable) = (0usize, 0usize);
    let (mut deg_evals, mut deg_rejects, mut prop0_rejects) = (0u64, 0u64, 0u64);
    let mut prop0_varies: Vec<PathBuf> = Vec::new();

    for f in &files {
        let Ok(text) = std::fs::read_to_string(f) else { continue };
        let mut per: BTreeMap<u64, BTreeMap<String, (u64, u64)>> = BTreeMap::new();
        for line in text.lines() {
            if line.starts_with('#') {
                continue;
            }
            let t: Vec<&str> = line.split(',').collect();
            if t.len() < 5 || t[0] == "window" || t[1] != "lk_fuse" {
                continue;
            }
            let (Ok(w), Ok(ev), Ok(rj)) =
                (t[0].parse::<u64>(), t[3].parse::<u64>(), t[4].parse::<u64>())
            else {
                continue;
            };
            per.entry(w).or_default().insert(t[2].to_string(), (ev, rj));
        }
        if per.is_empty() {
            continue;
        }
        runs += 1;
        let get = |w: &BTreeMap<String, (u64, u64)>, g: &str| w.get(g).copied().unwrap_or((0, 0));
        let dev: u64 = per.values().map(|w| get(w, "deg0").0).sum();
        deg_evals += dev;
        deg_rejects += per.values().map(|w| get(w, "deg0").1).sum::<u64>();
        if dev == 0 {
            unreachable += 1;
        }
        let p0: Vec<u64> = per.values().map(|w| get(w, "prop0").1).collect();
        prop0_rejects += p0.iter().sum::<u64>();
        if p0.iter().collect::<std::collections::BTreeSet<_>>().len() > 1 {
            prop0_varies.push(f.clone());
        }
    }

    println!("=== G0.1 guard-order audit: {runs} runs with lk_fuse guard rows ===");
    println!("  degree-guard evaluations      : {deg_evals}");
    println!("  degree-guard REJECTS          : {deg_rejects}");
    println!("  runs where degree unreachable : {unreachable} (f_cap arms: prop1 rejects first)");
    println!("  generation-guard rejects      : {prop0_rejects}");
    println!("  runs where gen rejects/window VARIES: {}", prop0_varies.len());

    assert!(runs > 100, "instrument out of range: only {runs} runs found");
    assert!(
        deg_evals > 0,
        "instrument out of range: the degree guard was never evaluated anywhere"
    );
    assert!(prop0_rejects > 0, "instrument out of range: generation guard never rejected");
    assert_eq!(
        deg_rejects, 0,
        "the degree guard rejected {deg_rejects} bindings — it is no longer \
         inert, and the emergent population is no longer purely 1-D"
    );
    assert!(
        prop0_varies.is_empty(),
        "generation-guard rejects/window varies in {} runs (first: {:?}) — \
         the gen-0 skeleton is no longer topologically frozen",
        prop0_varies.len(),
        prop0_varies.first()
    );
}
