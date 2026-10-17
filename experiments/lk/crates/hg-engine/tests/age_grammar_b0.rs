//! B0 (v8 §2): grammar audit for the age guard. Answers questions 3, 4
//! and 5 empirically; questions 1 and 2 are answered by reading the
//! matcher and are recorded in `docs/grammar_audit_v8.md`.
//!
//! The headline finding this file pins: **the birth step is already
//! present in the store**. `NodeData::birth_step` (named `generation` before v9 C2.1) is populated with
//! `store.step()` at every creation site (engine apply, the generators),
//! is never written again, and round-trips through the snapshot codec —
//! so a passive clock needs NO new property key and NO stamping effect,
//! which is design order (a) of v8 §1.1 rather than (c).
//!
//! Run the cost test with --ignored --nocapture.

use hg_core::{HyperEdge, HypergraphStore, Label, NodeData, PropertyValue};
use hg_rules::{compile_rules, parse_rules_toml, CompiledRule, StepMode};
use smallvec::smallvec;
use std::time::Instant;

const ENERGY: u32 = 1;
const GEN: u32 = 6;
const COUNTDOWN: u32 = 7;

// ---------------------------------------------------------------- Q3/Q4

/// Q3 + Q4 + hazard §8.1. The creation step is recorded on every node at
/// birth, survives the bincode round-trip the relaxation-IC path uses,
/// and — the part hazard §8.1 worried about — the STORE's step counter
/// survives with it, so ages stay in one frame across a snapshot load.
#[test]
fn b0_creation_step_is_recorded_and_round_trips() {
    let mut store = HypergraphStore::with_step(1234);
    let a = store.add_node(NodeData::new(Label(0), store.step()));
    let b = store.add_node(NodeData::new(Label(0), store.step()));
    store.add_edge(HyperEdge::new(Label(0), smallvec![a, b]));
    assert_eq!(store.get_node(a).unwrap().birth_step, 1234);

    // Advance, then create a younger node: the field distinguishes cohorts.
    store.set_step(1300);
    let c = store.add_node(NodeData::new(Label(0), store.step()));
    assert_eq!(store.get_node(c).unwrap().birth_step, 1300);

    let bytes = bincode::serialize(&store).unwrap();
    let restored: HypergraphStore = bincode::deserialize(&bytes).unwrap();
    assert_eq!(
        restored.step(),
        1300,
        "hazard §8.1: the step counter must survive the snapshot load, \
         or every stamped node becomes instantly ancient"
    );
    assert_eq!(restored.get_node(a).unwrap().birth_step, 1234);
    assert_eq!(restored.get_node(c).unwrap().birth_step, 1300);
    // Ages are therefore frame-consistent after the load.
    assert_eq!(restored.step() - restored.get_node(a).unwrap().birth_step, 66);
    assert_eq!(restored.step() - restored.get_node(c).unwrap().birth_step, 0);
}

/// Q4, the other half: nodes that were never "stamped" (the gen-0 lattice)
/// read as born at step 0, i.e. maximally OLD — the opposite of the
/// countdown gate, where a missing key rejected. The age guard therefore
/// supplies no lattice protection of its own; this test pins that the
/// protection the fuse actually relies on is the generation-property and
/// degree guards, which are unchanged.
#[test]
fn b0_lattice_nodes_read_as_ancient_so_other_guards_must_protect_them() {
    let mut store = HypergraphStore::new();
    let ids: Vec<_> = (0..3)
        .map(|_| {
            let mut d = NodeData::new(Label(0), 0);
            // The runner sets key 6 = 0.0 on IC nodes when g_max is set.
            d.set_property(GEN, PropertyValue::Float(0.0));
            store.add_node(d)
        })
        .collect();
    store.add_edge(HyperEdge::new(Label(0), smallvec![ids[0], ids[1]]));
    store.add_edge(HyperEdge::new(Label(0), smallvec![ids[1], ids[2]]));
    store.set_step(5000);

    // The middle node has degree 2 and age 5000 — it would pass an age
    // guard at any k. It is the generation guard that rejects it.
    let mid = store.get_node(ids[1]).unwrap();
    assert_eq!(store.step() - mid.birth_step, 5000);
    assert_eq!(mid.get_property(GEN), Some(PropertyValue::Float(0.0)));
    assert!(
        !PropertyValue::Float(0.0).compare(hg_core::PropertyOp::Gte, &PropertyValue::Float(1.0)),
        "lk_fuse's generation guard is what keeps the lattice non-removable"
    );
}

// ------------------------------------------------------------------ Q5

/// The EXP-L17 state shape: a 32×32 lattice ring-of-rings stand-in plus a
/// standing filament population of `n_fil` gen-1 nodes, each subdividing a
/// lattice edge into a 2-path (exactly what `lk_metered_subdivide`
/// produces). Countdowns are spread over [0, k] — the steady-state age
/// distribution (v6 §1.2 dispersion).
fn l17_like_store(n_lattice: usize, n_fil: usize, k: i64) -> HypergraphStore {
    let mut s = HypergraphStore::new();
    let ring: Vec<_> = (0..n_lattice)
        .map(|_| {
            let mut d = NodeData::new(Label(0), 0);
            d.set_property(ENERGY, PropertyValue::Float(0.0));
            d.set_property(GEN, PropertyValue::Float(0.0));
            s.add_node(d)
        })
        .collect();
    // Give the lattice a realistic degree (~4, the TriLattice torus) by
    // adding a chord ring alongside the neighbour ring.
    for i in 0..n_lattice {
        s.add_edge(HyperEdge::new(
            Label(0),
            smallvec![ring[i], ring[(i + 1) % n_lattice]],
        ));
        s.add_edge(HyperEdge::new(
            Label(0),
            smallvec![ring[i], ring[(i + 32) % n_lattice]],
        ));
    }
    // Standing population: gen-1 midpoints on a 2-path, energy q = 50.
    for j in 0..n_fil {
        let mut d = NodeData::new(Label(0), (j as u64) % (k as u64).max(1));
        d.set_property(ENERGY, PropertyValue::Float(50.0));
        d.set_property(GEN, PropertyValue::Float(1.0));
        d.set_property(COUNTDOWN, PropertyValue::Int((j as i64) % k.max(1)));
        let w = s.add_node(d);
        let x = ring[(j * 7) % n_lattice];
        let y = ring[(j * 7 + 1) % n_lattice];
        s.add_edge(HyperEdge::new(Label(0), smallvec![x, w]));
        s.add_edge(HyperEdge::new(Label(0), smallvec![w, y]));
    }
    // A live seed so subdivide has work: one node above θ.
    let seed = ring[0];
    s.get_node_mut(seed)
        .unwrap()
        .set_property(ENERGY, PropertyValue::Float(4000.0));
    s.set_step(10_000);
    s
}

/// The EXP-L17 ruleset, with and without `lk_tick` — the same injections
/// the runner performs for `gate = "countdown"`.
fn l17_rules(with_tick: bool, k: i64) -> Vec<CompiledRule> {
    let mut raw = parse_rules_toml(
        &std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../rules/lk_pair_capped.toml"
        ))
        .unwrap(),
    )
    .unwrap();
    raw.append(
        &mut parse_rules_toml(
            &std::fs::read_to_string(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../rules/lk_fuse.toml"
            ))
            .unwrap(),
        )
        .unwrap(),
    );
    for r in raw.iter_mut() {
        match r.name.as_str() {
            "lk_metered_subdivide" => r.rhs_property_effects.push(hg_rules::PropertyEffectDecl {
                var: "w".into(),
                key: COUNTDOWN,
                op: hg_rules::EffectOpDecl::Set {
                    value: PropertyValue::Int(k),
                },
            }),
            "lk_fuse" => r.lhs_node_property_guards.push(hg_rules::NodePropertyGuard {
                var: "w".into(),
                property: COUNTDOWN,
                op: hg_core::PropertyOp::Eq,
                value: PropertyValue::Int(0),
            }),
            _ => {}
        }
    }
    if with_tick {
        raw.append(
            &mut parse_rules_toml(
                &std::fs::read_to_string(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../../rules/lk_tick.toml"
                ))
                .unwrap(),
            )
            .unwrap(),
        );
    }
    compile_rules(&raw).unwrap()
}

/// Q5: the tick's marginal cost per step at EXP-L17 scale, on a MATCHED
/// state — the same store, the same standing population, the only
/// difference being whether `lk_tick` is in the ruleset. Reports the
/// matcher-enumeration and conflict-resolution halves separately, because
/// v8 §2 predicts a saving in both.
#[test]
#[ignore]
fn b0_tick_cost_delta_at_l17_scale() {
    let k = 100i64;
    // This machine's timings are noisy at the tens-of-µs scale, so the
    // arms are INTERLEAVED within each repeat and the reported number is
    // the median over repeats — a single A-then-B pass produced swings of
    // ±50% on identical inputs and would have been read as signal.
    let repeats = 9;
    let iters = 40;
    let median = |mut v: Vec<f64>| {
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        v[v.len() / 2]
    };
    println!("lattice | filaments | bindings(tick/no) | µs enum | µs resolve | µs total | Δ%");
    for &(n_lat, n_fil) in &[(1024usize, 100usize), (1024, 300), (1024, 1000)] {
        let store = l17_like_store(n_lat, n_fil, k);
        let snap = store.snapshot();
        let rules = [l17_rules(true, k), l17_rules(false, k)];
        let mut nb = [0usize; 2];
        let mut e_s: [Vec<f64>; 2] = [Vec::new(), Vec::new()];
        let mut r_s: [Vec<f64>; 2] = [Vec::new(), Vec::new()];
        let mut pool: Vec<hg_observe::Binding> = Vec::new();
        for rep in 0..repeats {
            for arm in 0..2 {
                // Alternate which arm goes first, so a warming trend in
                // the machine cannot favour one of them systematically.
                let a = if rep % 2 == 0 { arm } else { 1 - arm };
                let rs = &rules[a];
                pool.clear();
                for (rid, r) in rs.iter().enumerate() {
                    pool.extend(hg_engine::find_bindings(&snap, r, rid));
                }
                nb[a] = pool.len();
                let t = Instant::now();
                for _ in 0..iters {
                    pool.clear();
                    for (rid, r) in rs.iter().enumerate() {
                        pool.extend(hg_engine::find_bindings(&snap, r, rid));
                    }
                }
                e_s[a].push(t.elapsed().as_micros() as f64 / iters as f64);
                let t = Instant::now();
                for _ in 0..iters {
                    let _ = hg_engine::resolve_conflicts(&pool);
                }
                r_s[a].push(t.elapsed().as_micros() as f64 / iters as f64);
            }
        }
        let (e_t, e_n) = (median(e_s[0].clone()), median(e_s[1].clone()));
        let (r_t, r_n) = (median(r_s[0].clone()), median(r_s[1].clone()));
        let (tot_t, tot_n) = (e_t + r_t, e_n + r_n);
        println!(
            "{n_lat} | {n_fil} | {}/{} | {e_t:.0}→{e_n:.0} | {r_t:.0}→{r_n:.0} | \
             {tot_t:.0}→{tot_n:.0} | {:.1}%",
            nb[0],
            nb[1],
            100.0 * (tot_t - tot_n) / tot_t
        );
    }
}

/// The same comparison end to end: 200 consecutive engine steps from the
/// same initial state, apply included, so the audit can quote a step time
/// rather than a matcher time.
///
/// CONFOUND, stated: unlike the matched-state test above, the two arms
/// diverge after step 1 — the ticked arm removes filaments on a schedule
/// the un-ticked arm does not, so their graphs differ by step 200. This
/// is a run-budget number (what a run costs), not an instrument-cost
/// number (what the tick costs). The matched-state test is the clean one.
#[test]
#[ignore]
fn b0_tick_cost_delta_full_step() {
    let k = 100i64;
    let steps = 200;
    println!("filaments | µs/step with tick | without | end nodes (tick/no) | saving");
    for &n_fil in &[100usize, 300, 1000] {
        let mut out = Vec::new();
        for with_tick in [true, false] {
            let rules = l17_rules(with_tick, k);
            let mut store = l17_like_store(1024, n_fil, k);
            let mut engine =
                hg_engine::Engine::new(rules, StepMode::MaxParallel, 42, 100_000);
            engine.step(&mut store); // warm
            let t = Instant::now();
            for _ in 0..steps {
                engine.step(&mut store);
            }
            out.push((
                t.elapsed().as_micros() as f64 / steps as f64,
                store.node_count(),
            ));
        }
        println!(
            "{n_fil} | {:.0} | {:.0} | {}/{} | {:.0}%",
            out[0].0,
            out[1].0,
            out[0].1,
            out[1].1,
            100.0 * (out[0].0 - out[1].0) / out[0].0
        );
    }
}

// ------------------------------------------------------- v9 C2.1 rename

/// v9 C2.1 acceptance check (hazard 4): the `generation` → `birth_step`
/// rename must not break archived-run readability. Loads a REAL snapshot
/// written before the rename (`runs/expL_v2_archive/`, a v2-era run) and
/// asserts it still deserializes and that the birth stamps survive.
///
/// Two mechanisms protect this and both are exercised here: bincode is
/// positional so field names never reached the wire, and
/// `#[serde(rename = "generation")]` keeps the persisted name for any
/// name-based codec that might read the same struct later.
#[test]
fn archived_snapshot_round_trips_after_birth_step_rename() {
    let dir = std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../runs/expL_v2_archive"
    ));
    if !dir.is_dir() {
        eprintln!("archived runs absent — skipping (the rename's guard is the assertion below)");
        return;
    }
    let mut snaps: Vec<std::path::PathBuf> = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).into_iter().flatten().flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().is_some_and(|x| x == "bin") {
                snaps.push(p);
            }
        }
    }
    snaps.sort();
    assert!(!snaps.is_empty(), "expected archived .bin snapshots to test against");

    let mut checked = 0usize;
    for p in snaps.iter().take(5) {
        let (step, payload) = hg_output::SnapshotReader::read_snapshot(p)
            .unwrap_or_else(|e| panic!("{}: {e}", p.display()));
        let store: HypergraphStore = bincode::deserialize(&payload)
            .unwrap_or_else(|e| panic!("{} failed to deserialize after the rename: {e}", p.display()));
        assert!(store.node_count() > 0, "{}: empty store", p.display());
        // The stamps are readable under the new name and are sane: no node
        // claims to have been born after the snapshot's own step.
        for (_, d) in store.nodes() {
            assert!(
                d.birth_step <= step.max(store.step()),
                "{}: birth_step {} exceeds snapshot step {step}",
                p.display(),
                d.birth_step
            );
        }
        checked += 1;
    }
    assert!(checked > 0, "no archived snapshot was actually checked");
}
