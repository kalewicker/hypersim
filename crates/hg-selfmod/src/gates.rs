//! Gates G0–G2 (plan §8). Each blocks the next; G1 is the binding
//! acceptance test for the port (v2 F1 on the real engine).

use crate::decode::{decode_all, decode_spec, decode_spec_counted, DecodedRule, N_VALIDATORS};
use crate::decode::{V1, V2, V3, V4, V5, V6, V7, V8, V9};
use crate::economy::EconomyMode;
use crate::encode::install_rule;
use crate::runner::{object_step, run_phase, seed_rule_specs, seed_world, RunConfig};
use crate::schema::*;
use crate::store_ext::*;
use hg_core::{HyperEdge, HypergraphStore, Label, NodeData, NodeId, PropertyValue};
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use smallvec::SmallVec;
use std::collections::HashMap;

/// Budget map granting every rule an effectively unlimited creation
/// allowance — gates test matching/apply semantics, not F3 throttling.
fn unlimited_budgets(rules: &[DecodedRule]) -> HashMap<NodeId, i64> {
    rules.iter().map(|d| (d.root, i64::MAX / 4)).collect()
}

// ---------- G0: round-trip identity ----------

pub(crate) fn g0_fixtures() -> Vec<RuleSpec> {
    let mut v = seed_rule_specs(); // 4: guards, relabel, delete, priorities
                                   // 5. Multi-var (4), disconnected LHS (two separate pedges + isolated var).
    v.push(RuleSpec {
        priority: 2,
        n_vars: 4,
        var_labels: vec![Some(0), None, Some(2), None],
        pedges: vec![vec![0, 1], vec![2, 3]],
        guards: vec![],
        deletes: vec![],
        relabels: vec![(1, 3)],
        creates: vec![],
        redges: vec![vec![0, 1], vec![2, 3]],

        distinct: vec![],

        effects: vec![],
    });
    // 6. Arity-3 pattern edge and arity-3 RHS edge.
    v.push(RuleSpec {
        priority: 0,
        n_vars: 3,
        var_labels: vec![Some(1), Some(1), None],
        pedges: vec![vec![0, 1, 2]],
        guards: vec![],
        deletes: vec![],
        relabels: vec![],
        creates: vec![0],
        redges: vec![vec![0, 1, 3]],

        distinct: vec![],

        effects: vec![],
    });
    // 7. Multi-delete.
    v.push(RuleSpec {
        priority: -1,
        n_vars: 3,
        var_labels: vec![Some(3), Some(3), Some(3)],
        pedges: vec![vec![0, 1], vec![1, 2]],
        guards: vec![],
        deletes: vec![1, 2],
        relabels: vec![],
        creates: vec![],
        redges: vec![],

        distinct: vec![],

        effects: vec![],
    });
    // 8. Guard-heavy (two guards, Int + Float).
    v.push(RuleSpec {
        priority: 0,
        n_vars: 2,
        var_labels: vec![None, None],
        pedges: vec![vec![0, 1]],
        guards: vec![
            GuardSpec {
                var: 0,
                key: K_ENERGY,
                op: 1,
                val: PropertyValue::Float(0.5),
            },
            GuardSpec {
                var: 1,
                key: 7,
                op: 2,
                val: PropertyValue::Int(3),
            },
        ],
        deletes: vec![],
        relabels: vec![],
        creates: vec![],
        redges: vec![vec![0, 1]],

        distinct: vec![],

        effects: vec![],
    });
    // 9. Max creates, edges over survivors ∪ creates.
    v.push(RuleSpec {
        priority: 0,
        n_vars: 1,
        var_labels: vec![Some(2)],
        pedges: vec![],
        guards: vec![],
        deletes: vec![],
        relabels: vec![],
        creates: vec![0, 1, 2],
        redges: vec![vec![0, 1], vec![1, 2], vec![2, 3], vec![0, 1, 3]],

        distinct: vec![],

        effects: vec![],
    });
    // 10. Isolated-var-only rule with delete + create (rebuild pattern).
    v.push(RuleSpec {
        priority: 0,
        n_vars: 2,
        var_labels: vec![Some(0), Some(3)],
        pedges: vec![],
        guards: vec![],
        deletes: vec![0],
        relabels: vec![(1, 0)],
        creates: vec![3],
        redges: vec![vec![1, 2]],

        distinct: vec![],

        effects: vec![],
    });
    v
}

/// Build the 5 deliberately malformed subgraphs; each must decode to None.
fn g0_malformed(rs: &mut HypergraphStore, seq: &mut i64) -> Vec<(String, hg_core::NodeId)> {
    let mut out = Vec::new();

    // M1: guard var out of range.
    let m1 = install_rule(
        rs,
        &RuleSpec {
            priority: 0,
            n_vars: 2,
            var_labels: vec![None, None],
            pedges: vec![vec![0, 1]],
            guards: vec![GuardSpec {
                var: 0,
                key: 1,
                op: 0,
                val: PropertyValue::Int(0),
            }],
            deletes: vec![],
            relabels: vec![],
            creates: vec![],
            redges: vec![],

            distinct: vec![],

            effects: vec![],
        },
        seq,
    );
    for gn in children(rs, m1, HAS_GUARD) {
        set_int(rs, gn, K_GVAR, 7);
    }
    out.push(("guard var out of range".to_string(), m1));

    // M2: REdge ref to a deleted var.
    let m2 = install_rule(
        rs,
        &RuleSpec {
            priority: 0,
            n_vars: 2,
            var_labels: vec![None, None],
            pedges: vec![vec![0, 1]],
            guards: vec![],
            deletes: vec![1],
            relabels: vec![],
            creates: vec![],
            redges: vec![],

            distinct: vec![],

            effects: vec![],
        },
        seq,
    );
    {
        let re = add_rule_node(
            rs,
            REDGE,
            vec![
                (K_ARITY, PropertyValue::Int(2)),
                (K_EP0, PropertyValue::Int(0)),
                (K_EP0 + 1, PropertyValue::Int(1)), // var 1 is deleted
            ],
        );
        add_role_edge(rs, m2, re, HAS_REDGE);
    }
    out.push(("REdge ref to deleted var".to_string(), m2));

    // M3: PEdge arity beyond cap.
    let m3 = install_rule(
        rs,
        &RuleSpec {
            priority: 0,
            n_vars: 4,
            var_labels: vec![None; 4],
            pedges: vec![],
            guards: vec![],
            deletes: vec![],
            relabels: vec![],
            creates: vec![],
            redges: vec![],

            distinct: vec![],

            effects: vec![],
        },
        seq,
    );
    {
        let mut props = vec![(K_ARITY, PropertyValue::Int(5))];
        for i in 0..5u32 {
            props.push((K_EP0 + i, PropertyValue::Int(0)));
        }
        let pe = add_rule_node(rs, PEDGE, props);
        add_role_edge(rs, m3, pe, HAS_PEDGE);
    }
    out.push(("PEdge arity 5 beyond cap".to_string(), m3));

    // M4: n_vars = 0.
    let m4 = install_rule(
        rs,
        &RuleSpec {
            priority: 0,
            n_vars: 1,
            var_labels: vec![None],
            pedges: vec![],
            guards: vec![],
            deletes: vec![],
            relabels: vec![],
            creates: vec![],
            redges: vec![],

            distinct: vec![],

            effects: vec![],
        },
        seq,
    );
    set_int(rs, m4, K_NVARS, 0);
    out.push(("n_vars = 0".to_string(), m4));

    // M5: create indices with a gap (0, 2).
    let m5 = install_rule(
        rs,
        &RuleSpec {
            priority: 0,
            n_vars: 1,
            var_labels: vec![None],
            pedges: vec![],
            guards: vec![],
            deletes: vec![],
            relabels: vec![],
            creates: vec![0, 1],
            redges: vec![],

            distinct: vec![],

            effects: vec![],
        },
        seq,
    );
    for cn in children(rs, m5, HAS_CREATE) {
        if get_int(rs, cn, K_CREATE_INDEX) == Some(1) {
            set_int(rs, cn, K_CREATE_INDEX, 2);
        }
    }
    out.push(("create index gap".to_string(), m5));

    out
}

pub fn gate0() -> bool {
    let caps = Caps::default();
    let mut ok = true;

    let fixtures = g0_fixtures();
    let mut rs = HypergraphStore::new();
    let mut seq = 0i64;
    for (i, spec) in fixtures.iter().enumerate() {
        let root = install_rule(&mut rs, spec, &mut seq);
        match decode_spec(&rs, root, &caps) {
            Some(decoded) => {
                if &decoded != spec {
                    println!("G0 FAIL: fixture {i} round-trip mismatch\n  in:  {spec:?}\n  out: {decoded:?}");
                    ok = false;
                } else if decoded.class_hash() != spec.class_hash() {
                    println!("G0 FAIL: fixture {i} class_hash mismatch");
                    ok = false;
                }
            }
            None => {
                println!("G0 FAIL: fixture {i} decoded to None");
                ok = false;
            }
        }
    }

    let mut rs2 = HypergraphStore::new();
    let mut seq2 = 0i64;
    for (what, root) in g0_malformed(&mut rs2, &mut seq2) {
        if let Some(spec) = decode_spec(&rs2, root, &caps) {
            println!("G0 FAIL: malformed subgraph ({what}) decoded to Some({spec:?})");
            ok = false;
        }
    }

    // G0b (E1d): distinct-pair round-trip + inert normalizations. Kept out of
    // g0_fixtures() on purpose: that list feeds the G-R1 seed set, which must
    // stay byte-identical across the E1d landing (mutators never emit
    // DISTINCT nodes, so G-R1 behavior is provably unaffected).
    {
        let dspec = RuleSpec {
            priority: 1,
            n_vars: 3,
            var_labels: vec![Some(0), None, Some(1)],
            pedges: vec![vec![0, 1, 2]],
            guards: vec![],
            deletes: vec![],
            relabels: vec![(2, 4)],
            creates: vec![7],
            redges: vec![vec![0, 1, 3]],
            distinct: vec![(0, 2), (1, 2)],

            effects: vec![],
        };
        let mut rs = HypergraphStore::new();
        let mut seq = 0i64;
        let root = install_rule(&mut rs, &dspec, &mut seq);
        match decode_spec(&rs, root, &caps) {
            Some(d) if d == dspec && d.class_hash() == dspec.class_hash() => {}
            other => {
                println!("G0 FAIL: distinct round-trip mismatch: {other:?}");
                ok = false;
            }
        }
        // Self-pair DISTINCT drops as inert (mirrors compile_rule's skip).
        let mut rsb = HypergraphStore::new();
        let mut qb = 0i64;
        let rb = install_rule(&mut rsb, &dspec, &mut qb);
        let self_dn = add_rule_node(
            &mut rsb,
            DISTINCT,
            vec![
                (K_VAR_ID, PropertyValue::Int(1)),
                (K_VAR_ID2, PropertyValue::Int(1)),
            ],
        );
        add_role_edge(&mut rsb, rb, self_dn, HAS_DISTINCT);
        match decode_spec(&rsb, rb, &caps) {
            Some(d) if d == dspec => {}
            other => {
                println!("G0 FAIL: self-pair DISTINCT not dropped: {other:?}");
                ok = false;
            }
        }
        // Relabel-of-deleted drops as inert (recon note (a) pin).
        let mut rsc = HypergraphStore::new();
        let mut qc = 0i64;
        let mut wspec = dspec.clone();
        wspec.deletes = vec![2];
        let rc = install_rule(&mut rsc, &wspec, &mut qc);
        match decode_spec(&rsc, rc, &caps) {
            Some(d) if d.relabels.is_empty() && d.deletes == vec![2] => {}
            other => {
                println!("G0 FAIL: relabel-of-deleted not dropped: {other:?}");
                ok = false;
            }
        }
        // Full guard-op set round-trips (E1-G completion: lte/gte/ne).
        let mut rsd = HypergraphStore::new();
        let mut qd = 0i64;
        let gspec = RuleSpec {
            priority: 0,
            n_vars: 2,
            var_labels: vec![None, None],
            pedges: vec![vec![0, 1]],
            guards: vec![
                GuardSpec {
                    var: 0,
                    key: 1,
                    op: 3,
                    val: PropertyValue::Int(5),
                },
                GuardSpec {
                    var: 0,
                    key: 2,
                    op: 4,
                    val: PropertyValue::Float(0.5),
                },
                GuardSpec {
                    var: 1,
                    key: 3,
                    op: 5,
                    val: PropertyValue::Bool(true),
                },
            ],
            deletes: vec![],
            relabels: vec![],
            creates: vec![],
            redges: vec![vec![0, 1]],
            distinct: vec![],
            effects: vec![],
        };
        let rd = install_rule(&mut rsd, &gspec, &mut qd);
        match decode_spec(&rsd, rd, &caps) {
            Some(d) if d == gspec => {}
            other => {
                println!("G0 FAIL: guard-op round-trip mismatch: {other:?}");
                ok = false;
            }
        }
    }

    if ok {
        println!(
            "G0 PASS: {} fixtures round-trip (spec + class_hash), 5 malformed subgraphs -> None, E1d distinct round-trip + normalizations hold, no panic",
            fixtures.len()
        );
    }
    ok
}

// ---------- G1: reification neutrality ----------

/// Fixture ruleset for G1: the seed rules plus two enriched-class rules
/// (arity-3 hyperedge creation and consumption) so the neutrality claim
/// covers the full pattern class, not just the v2 subset.
pub(crate) fn g1_specs() -> Vec<RuleSpec> {
    let mut v = seed_rule_specs();
    v.push(RuleSpec {
        priority: 0,
        n_vars: 2,
        var_labels: vec![Some(0), Some(1)],
        pedges: vec![vec![0, 1]],
        guards: vec![],
        deletes: vec![],
        relabels: vec![],
        creates: vec![3],
        redges: vec![vec![0, 1, 2]],

        distinct: vec![],

        effects: vec![],
    });
    v.push(RuleSpec {
        priority: 0,
        n_vars: 3,
        var_labels: vec![None, None, Some(3)],
        pedges: vec![vec![0, 1, 2]],
        guards: vec![],
        deletes: vec![2],
        relabels: vec![],
        creates: vec![],
        redges: vec![vec![0, 1]],

        distinct: vec![],

        effects: vec![],
    });
    v
}

pub fn gate1(seeds: &[u64], steps: usize) -> bool {
    let cfg = RunConfig::default();
    let caps = cfg.caps();
    let mut all_ok = true;

    for &seed in seeds {
        // Same seed world for both paths: identical construction sequence
        // => identical NodeIds => identical canonical binding order.
        let mut world_rng = ChaCha8Rng::seed_from_u64(seed);
        let mut base = HypergraphStore::new();
        seed_world(
            &mut base,
            &mut world_rng,
            cfg.world_nodes,
            cfg.world_edges,
            cfg.n_labels,
        );

        // ENCODED path: rules live in the RuleStore, decoded EVERY step.
        let mut rulestore = HypergraphStore::new();
        let mut seq = 0i64;
        for spec in g1_specs() {
            install_rule(&mut rulestore, &spec, &mut seq);
        }

        // DIRECT path: compiled once, ordered exactly as decode_all orders
        // (class_hash, seq) — the decoder's canonical enumeration applied to
        // the direct rule table, per the recon determinism caveat.
        let (direct_rules, report) = decode_all(&rulestore, &caps);
        assert_eq!(report.total, 0, "G1 fixtures must all decode");

        let mut wd = base.clone();
        let mut we = base.clone();
        let mut rng_d = ChaCha8Rng::seed_from_u64(seed ^ 0xABCD);
        let mut rng_e = ChaCha8Rng::seed_from_u64(seed ^ 0xABCD);

        let mut first_div: Option<usize> = None;
        for step in 0..steps {
            let bd = unlimited_budgets(&direct_rules);
            let _ = object_step(&mut wd, &direct_rules, &mut rng_d, &cfg, &bd);
            let (rules_e, _) = decode_all(&rulestore, &caps);
            let be = unlimited_budgets(&rules_e);
            let _ = object_step(&mut we, &rules_e, &mut rng_e, &cfg, &be);
            if wd.canonical_hash() != we.canonical_hash() {
                first_div = Some(step);
                break;
            }
        }
        match first_div {
            None => println!("G1 seed {seed}: PASS ({steps} steps bit-identical canonical hash)"),
            Some(s) => {
                println!("G1 seed {seed}: FAIL (first divergence at step {s})");
                all_ok = false;
            }
        }
    }
    if all_ok {
        println!(
            "G1 PASS: decode+apply == direct apply on {} seeds x {} steps",
            seeds.len(),
            steps
        );
    }
    all_ok
}

// ---------- G2: sustaining-vocabulary probe ----------

/// Hand-constructed churn cycle (existence proof, not reachability):
///   R1: A—B  =>  A—C, C—B   (creates C; +1 node, edge consumed)
///   R2: C—B  =>  delete C (dangling cleanup takes A—C), relabel B->D
///   R3: A, D (disconnected)  =>  relabel D->B, edge A—B
/// Net over a cycle: node and edge counts return; labels and structure keep
/// moving. R3 consumes D-labelled nodes, so it is throttled by R2's output —
/// no unbounded edge growth.
pub fn g2_specs() -> Vec<RuleSpec> {
    vec![
        RuleSpec {
            priority: 0,
            n_vars: 2,
            var_labels: vec![Some(0), Some(1)],
            pedges: vec![vec![0, 1]],
            guards: vec![],
            deletes: vec![],
            relabels: vec![],
            creates: vec![2],
            redges: vec![vec![0, 2], vec![2, 1]],

            distinct: vec![],

            effects: vec![],
        },
        RuleSpec {
            priority: 1,
            n_vars: 2,
            var_labels: vec![Some(2), Some(1)],
            pedges: vec![vec![0, 1]],
            guards: vec![],
            deletes: vec![0],
            relabels: vec![(1, 3)],
            creates: vec![],
            redges: vec![],

            distinct: vec![],

            effects: vec![],
        },
        RuleSpec {
            priority: 0,
            n_vars: 2,
            var_labels: vec![Some(0), Some(3)],
            pedges: vec![],
            guards: vec![],
            deletes: vec![],
            relabels: vec![(1, 1)],
            creates: vec![],
            redges: vec![vec![0, 1]],

            distinct: vec![],

            effects: vec![],
        },
    ]
}

pub fn gate2(seed: u64, steps: usize) -> bool {
    let cfg = RunConfig::default();
    let caps = cfg.caps();
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut world = HypergraphStore::new();
    seed_world(
        &mut world,
        &mut rng,
        cfg.world_nodes,
        cfg.world_edges,
        cfg.n_labels,
    );

    let mut rulestore = HypergraphStore::new();
    let mut seq = 0i64;
    for spec in g2_specs() {
        install_rule(&mut rulestore, &spec, &mut seq);
    }
    let (rules, report) = decode_all(&rulestore, &caps);
    assert_eq!(report.total, 0, "G2 ruleset must decode");

    let window = 100usize;
    let mut window_prod = 0usize;
    let mut dead_windows = 0usize;
    let mut hashes: Vec<u64> = Vec::new();
    for step in 0..steps {
        let budgets = unlimited_budgets(&rules);
        let oo = object_step(&mut world, &rules, &mut rng, &cfg, &budgets);
        window_prod += oo.productive_applied;
        if (step + 1) % window == 0 {
            if window_prod == 0 {
                dead_windows += 1;
            }
            window_prod = 0;
            hashes.push(world.canonical_hash());
        }
    }
    let tail: Vec<u64> = hashes.iter().rev().take(20).copied().collect();
    let mut distinct_tail = tail.clone();
    distinct_tail.sort_unstable();
    distinct_tail.dedup();

    let ok = dead_windows == 0 && distinct_tail.len() >= 10;
    println!(
        "G2 {}: {} steps, dead windows {}/{}, distinct canonical hashes in last 20 windows: {}/20, final world = ({} nodes, {} edges)",
        if ok { "PASS" } else { "FAIL" },
        steps,
        dead_windows,
        hashes.len(),
        distinct_tail.len(),
        world.node_count(),
        world.edge_count()
    );
    ok
}

// ---------- G4.4: the minimal sustainer (churnK) ----------

/// `churnK` — a **single-rule** sustaining set (k = 1): a node-neutral,
/// edge-neutral rewrite that both consumes and produces its own substrate.
///
///   `v0(label 0) —— v1(label 1)`  ⇒  `v0 → 1`, `v1 → 0`, edge re-emitted
///
/// The matched edge is consumed by engine semantics and re-emitted verbatim,
/// so node and edge counts are exactly conserved; the two endpoint labels
/// swap, so the rewrite is productive and the world's canonical hash cycles
/// with period 2. Every such edge in the world flips in parallel under
/// MaxParallel (bindings are node-disjoint).
///
/// This is the smallest possible autocatalytic set — one rule that
/// regenerates its own precondition — which is what E3 needs to contrast
/// against `churn3`'s distributed 3-rule cycle.
pub fn churn_k_specs() -> Vec<RuleSpec> {
    vec![RuleSpec {
        priority: 0,
        n_vars: 2,
        var_labels: vec![Some(0), Some(1)],
        pedges: vec![vec![0, 1]],
        guards: vec![],
        deletes: vec![],
        relabels: vec![(0, 1), (1, 0)],
        creates: vec![],
        redges: vec![vec![0, 1]],

        distinct: vec![],

        effects: vec![],
    }]
}

/// Verify a hand-built ruleset sustains activity, by G2's criterion:
/// productive rewrites in every window and a changing canonical hash.
fn sustains(specs: Vec<RuleSpec>, seed: u64, steps: usize, label: &str) -> bool {
    let cfg = RunConfig::default();
    let caps = cfg.caps();
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut world = HypergraphStore::new();
    seed_world(
        &mut world,
        &mut rng,
        cfg.world_nodes,
        cfg.world_edges,
        cfg.n_labels,
    );

    let mut rulestore = HypergraphStore::new();
    let mut seq = 0i64;
    for spec in &specs {
        install_rule(&mut rulestore, spec, &mut seq);
    }
    let (rules, report) = decode_all(&rulestore, &caps);
    assert_eq!(report.total, 0, "{label} ruleset must decode");
    let budgets = unlimited_budgets(&rules);

    let window = 100usize;
    let mut window_prod = 0usize;
    let mut dead_windows = 0usize;
    let mut n_windows = 0usize;
    // Hashes are sampled on CONSECUTIVE steps at the end of the run, not at
    // window boundaries: a period-2 attractor (churnK's label swap) sampled
    // every 100 steps always lands on the same phase and reads as frozen.
    // That aliasing is a measurement artifact, so the criterion looks at
    // adjacent steps instead.
    let tail_from = steps.saturating_sub(32);
    let mut tail_hashes: Vec<u64> = Vec::new();
    for step in 0..steps {
        let oo = object_step(&mut world, &rules, &mut rng, &cfg, &budgets);
        window_prod += oo.productive_applied;
        if (step + 1) % window == 0 {
            if window_prod == 0 {
                dead_windows += 1;
            }
            window_prod = 0;
            n_windows += 1;
        }
        if step >= tail_from {
            tail_hashes.push(world.canonical_hash());
        }
    }
    let mut distinct = tail_hashes.clone();
    distinct.sort_unstable();
    distinct.dedup();
    // Smallest p such that the tail is p-periodic (0 = no period <= 8 found).
    let period = (1..=8usize)
        .find(|&p| tail_hashes.len() > p && tail_hashes.windows(p + 1).all(|w| w[0] == w[p]))
        .unwrap_or(0);
    let ok = dead_windows == 0 && distinct.len() >= 2;
    println!(
        "  {label} (k={}): {} — {} steps, dead windows {}/{}, distinct hashes over last {} consecutive steps: {} (period {}), world=({}, {})",
        specs.len(),
        if ok { "SUSTAINS" } else { "DOES NOT SUSTAIN" },
        steps,
        dead_windows,
        n_windows,
        tail_hashes.len(),
        distinct.len(),
        if period > 0 { period.to_string() } else { "none<=8".into() },
        world.node_count(),
        world.edge_count()
    );
    ok
}

pub fn gate4_4(steps: usize) -> bool {
    println!("G4.4 — minimal sustainer construction:");
    let k1 = sustains(churn_k_specs(), 1, steps, "churnK");
    // Confirm the comparison target still sustains under the same criterion.
    let k3 = sustains(g2_specs(), 1, steps, "churn3");
    if k1 && k3 {
        println!("G4.4 PASS: minimal sustainer found at k=1; churn3 (k=3) also sustains — E3 has a real k contrast");
    } else if !k1 {
        println!("G4.4 FAIL: no k=1 sustainer; E3 degrades to a single-point observation");
    } else {
        println!("G4.4 FAIL: churn3 no longer sustains — investigate before E1");
    }
    k1 && k3
}

// ---------- G4.2: effect-gate completeness ----------

/// One effect-gate case: a two-node world (A:label0 —— B:label1) plus the
/// rule under test; assert the firing's productive/sterile classification.
fn effect_case(name: &str, spec: RuleSpec, want_productive: bool) -> bool {
    let mut cfg = RunConfig::default();
    cfg.world_ceiling = 100_000;
    cfg.edge_ceiling = 100_000;
    let caps = cfg.caps();

    let mut world = HypergraphStore::new();
    let a = world.add_node(NodeData::new(Label(0), 0));
    let b = world.add_node(NodeData::new(Label(1), 0));
    world.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[a, b])));

    let mut rulestore = HypergraphStore::new();
    let mut seq = 0i64;
    install_rule(&mut rulestore, &spec, &mut seq);
    let (rules, report) = decode_all(&rulestore, &caps);
    if report.total != 0 || rules.is_empty() {
        println!("  G4.2 FAIL [{name}]: fixture rule did not decode");
        return false;
    }
    let budgets = unlimited_budgets(&rules);
    let mut rng = ChaCha8Rng::seed_from_u64(1);
    let oo = object_step(&mut world, &rules, &mut rng, &cfg, &budgets);

    if oo.applied == 0 {
        println!("  G4.2 FAIL [{name}]: rule never fired (cannot score the gate)");
        return false;
    }
    let got_productive = oo.productive_applied > 0;
    let ok = got_productive == want_productive;
    println!(
        "  [{}] {name}: applied={} productive={} sterile={} (want {})",
        if ok { "ok" } else { "FAIL" },
        oo.applied,
        oo.productive_applied,
        oo.sterile_firings,
        if want_productive {
            "productive"
        } else {
            "sterile"
        }
    );
    ok
}

/// One quarantine case: the spec must fail decode against exactly validator
/// `want`, so it never reaches the object phase at all.
fn quarantine_case(name: &str, spec: RuleSpec, want: usize) -> bool {
    let caps = Caps::default();
    let mut rulestore = HypergraphStore::new();
    let mut seq = 0i64;
    let root = install_rule(&mut rulestore, &spec, &mut seq);
    let mut counters = [0usize; N_VALIDATORS];
    let got = decode_spec_counted(&rulestore, root, &caps, &mut counters);
    let mut want_counters = [0usize; N_VALIDATORS];
    want_counters[want] = 1;
    let ok = got.is_none() && counters == want_counters;
    println!(
        "  [{}] {name}: quarantined V{} (counters={:?})",
        if ok { "ok" } else { "FAIL" },
        want + 1,
        counters
    );
    ok
}

pub fn gate4_2() -> bool {
    println!("G4.2 — effect-gate completeness (F1):");
    let mut ok = true;

    // Edge-only DELETION: consume the matched edge, emit nothing. No label
    // or node change at all — invisible to a label-only gate.
    ok &= effect_case(
        "edge-only deletion",
        RuleSpec {
            priority: 0,
            n_vars: 2,
            var_labels: vec![Some(0), Some(1)],
            pedges: vec![vec![0, 1]],
            guards: vec![],
            deletes: vec![],
            relabels: vec![],
            creates: vec![],
            redges: vec![],

            distinct: vec![],

            effects: vec![],
        },
        true,
    );

    // Edge-only CREATION: disconnected LHS, emit an edge. Again no label or
    // node change.
    ok &= effect_case(
        "edge-only creation",
        RuleSpec {
            priority: 0,
            n_vars: 2,
            var_labels: vec![Some(0), Some(1)],
            pedges: vec![],
            guards: vec![],
            deletes: vec![],
            relabels: vec![],
            creates: vec![],
            redges: vec![vec![0, 1]],

            distinct: vec![],

            effects: vec![],
        },
        true,
    );

    // True no-op: relabel a node to the label it already has.
    ok &= effect_case(
        "relabel-to-same (weed)",
        RuleSpec {
            priority: 0,
            n_vars: 1,
            var_labels: vec![Some(0)],
            pedges: vec![],
            guards: vec![],
            deletes: vec![],
            relabels: vec![(0, 0)],
            creates: vec![],
            redges: vec![],

            distinct: vec![],

            effects: vec![],
        },
        false,
    );

    // Empty RHS effect: match a node, change nothing. Since V9 (non-vacuity),
    // this never reaches the object phase — it quarantines at decode instead
    // of firing sterile. The weed is caught earlier and cheaper; the sterile
    // classification is still pinned by the two cases above and below.
    ok &= quarantine_case(
        "empty RHS (weed, V9)",
        RuleSpec {
            priority: 0,
            n_vars: 1,
            var_labels: vec![Some(0)],
            pedges: vec![],
            guards: vec![],
            deletes: vec![],
            relabels: vec![],
            creates: vec![],
            redges: vec![],
            distinct: vec![],

            effects: vec![],
        },
        V9,
    );

    // Delete-and-recreate the IDENTICAL edge. The store ends
    // content-identical, so this is a weed rule dressed as edge activity.
    // NOTE: stricter than the plan's literal "edge created/deleted ⇒
    // productive" — see the deviation note in the findings.
    ok &= effect_case(
        "edge identity-churn (weed)",
        RuleSpec {
            priority: 0,
            n_vars: 2,
            var_labels: vec![Some(0), Some(1)],
            pedges: vec![vec![0, 1]],
            guards: vec![],
            deletes: vec![],
            relabels: vec![],
            creates: vec![],
            redges: vec![vec![0, 1]],

            distinct: vec![],

            effects: vec![],
        },
        false,
    );

    // Node creation and node deletion (the cases a label-only gate does see).
    ok &= effect_case(
        "node creation",
        RuleSpec {
            priority: 0,
            n_vars: 1,
            var_labels: vec![Some(0)],
            pedges: vec![],
            guards: vec![],
            deletes: vec![],
            relabels: vec![],
            creates: vec![2],
            redges: vec![],

            distinct: vec![],

            effects: vec![],
        },
        true,
    );
    ok &= effect_case(
        "node deletion",
        RuleSpec {
            priority: 0,
            n_vars: 1,
            var_labels: vec![Some(0)],
            pedges: vec![],
            guards: vec![],
            deletes: vec![0],
            relabels: vec![],
            creates: vec![],
            redges: vec![],

            distinct: vec![],

            effects: vec![],
        },
        true,
    );
    // The churnK sustainer must read productive (it is the E3 substrate).
    ok &= effect_case("churnK label swap", churn_k_specs().remove(0), true);

    println!(
        "G4.2 {}",
        if ok {
            "PASS: 7 firing cases classified + empty weed quarantines at V9"
        } else {
            "FAIL"
        }
    );
    ok
}

// ---------- G4.3: latency calibration ----------

/// Under flux with `churn3`, find the smallest influx `I` on the probe grid
/// for which `time_to_first_fission <= 200` and the seeded triple has not
/// frozen first. Prints the table and returns the calibrated `I`.
pub fn gate4_3(seeds: &[u64], probe_steps: usize) -> Option<i64> {
    println!("G4.3 — latency calibration (flux, churn3, target time_to_first_fission <= 200):");
    let grid: [i64; 6] = [5, 10, 20, 40, 80, 160];
    let mut calibrated: Option<i64> = None;

    for &i in &grid {
        let mut ttf: Vec<Option<usize>> = Vec::new();
        let mut froze_first = 0usize;
        for &seed in seeds {
            let mut cfg = RunConfig::default();
            cfg.economy = EconomyMode::Flux;
            cfg.seed_pop = "churn3".into();
            cfg.seed = seed;
            cfg.steps = probe_steps;
            cfg.log_every = 50;
            cfg.influx = i;
            cfg.abio = 0.0;
            cfg.p_dup = 0.0;
            cfg.p_del = 0.0;
            cfg.mu = 0.0; // calibration measures funding rate, not variation
            let res = run_phase('B', &cfg);
            if let (Some(f), Some(fz)) = (res.time_to_first_fission, res.time_to_freeze) {
                if fz < f {
                    froze_first += 1;
                }
            } else if res.time_to_first_fission.is_none() && res.time_to_freeze.is_some() {
                froze_first += 1;
            }
            ttf.push(res.time_to_first_fission);
        }
        let got: Vec<usize> = ttf.iter().filter_map(|x| *x).collect();
        let median = if got.is_empty() {
            None
        } else {
            let mut g = got.clone();
            g.sort_unstable();
            Some(g[g.len() / 2])
        };
        println!(
            "  I={:>3}: first fission in {}/{} seeds, median t_ff={}, froze-before-fission {}/{}",
            i,
            got.len(),
            seeds.len(),
            median.map(|v| v.to_string()).unwrap_or_else(|| "-".into()),
            froze_first,
            seeds.len()
        );
        // Criterion: among seeds that reproduce at all, funding must reach
        // the threshold within 200 steps, and no seed may freeze before it
        // fissions. Requiring EVERY seed to fission would be wrong — a seed
        // whose churn settles into a low duty cycle never reaches threshold,
        // which is a property of the world, not of the influx.
        if calibrated.is_none() {
            if let Some(m) = median {
                if m <= 200 && got.len() * 2 >= seeds.len() && froze_first == 0 {
                    calibrated = Some(i);
                }
            }
        }
    }

    match calibrated {
        Some(smallest) => {
            // Latency is insensitive to I across the whole grid (see table):
            // with a handful of rules, reward is capped by `fire_reward`, not
            // by `free`, so the pool is never the binding constraint at seed
            // scale. I is therefore chosen on the OTHER thing it sets — the
            // throughput carrying capacity N* ≈ I/rent — picking headroom
            // well above the 3-rule seed and well below `rule_cap`.
            let operating = 20;
            println!(
                "G4.3 PASS: latency target met from I = {smallest} upward; \
                 operating point I = {operating} (N* = I/rent = {operating} rules)"
            );
            Some(operating)
        }
        None => {
            println!(
                "G4.3 FAIL: no I on the grid gives fission within 200 steps without freezing \
                 first — latency and stability are in tension (a v4-level finding in its own right)"
            );
            None
        }
    }
}

// ---------- G-R0: RCF correctness (docs/ere_canonical_form.md §3) ----------

/// Apply a var renaming `p` (`p[old_var] = new_id`) to a spec. Test helper
/// mirroring `serialize_renamed` at the spec level. Hand-written literals
/// below cross-check it, so a bug here cannot green the gate by itself.
fn permute_spec(spec: &RuleSpec, p: &[u8]) -> RuleSpec {
    let n = spec.n_vars as usize;
    assert_eq!(p.len(), n);
    let mut inv = vec![0u8; n];
    for (old, &new) in p.iter().enumerate() {
        inv[new as usize] = old as u8;
    }
    let map = |v: u8| p[v as usize];
    RuleSpec {
        priority: spec.priority,
        n_vars: spec.n_vars,
        var_labels: (0..n)
            .map(|new| spec.var_labels[inv[new] as usize])
            .collect(),
        pedges: spec
            .pedges
            .iter()
            .map(|e| e.iter().map(|&r| map(r)).collect())
            .collect(),
        guards: spec
            .guards
            .iter()
            .map(|g| GuardSpec {
                var: map(g.var),
                key: g.key,
                op: g.op,
                val: g.val,
            })
            .collect(),
        deletes: spec.deletes.iter().map(|&v| map(v)).collect(),
        relabels: spec.relabels.iter().map(|&(v, l)| (map(v), l)).collect(),
        creates: spec.creates.clone(),
        redges: spec
            .redges
            .iter()
            .map(|e| {
                e.iter()
                    .map(|&r| if (r as usize) < n { map(r) } else { r })
                    .collect()
            })
            .collect(),
        distinct: spec
            .distinct
            .iter()
            .map(|&(a, b)| {
                let (x, y) = (map(a), map(b));
                (x.min(y), x.max(y))
            })
            .collect(),
        effects: spec
            .effects
            .iter()
            .map(|e| e.renamed(|r| if (r as usize) < n { map(r) } else { r }))
            .collect(),
    }
}

pub(crate) fn rcf_rich_base() -> RuleSpec {
    RuleSpec {
        priority: 2,
        n_vars: 4,
        var_labels: vec![Some(0), None, Some(2), None],
        pedges: vec![vec![0, 1], vec![2, 3]],
        guards: vec![GuardSpec {
            var: 0,
            key: 100,
            op: 0,
            val: PropertyValue::Float(0.5),
        }],
        deletes: vec![],
        relabels: vec![(1, 3)],
        creates: vec![1],
        redges: vec![vec![0, 1, 4], vec![2, 3]],
        distinct: vec![],

        effects: vec![],
    }
}

pub(crate) fn rcf_churnk_like() -> RuleSpec {
    RuleSpec {
        priority: 0,
        n_vars: 2,
        var_labels: vec![Some(0), Some(1)],
        pedges: vec![vec![0, 1]],
        guards: vec![],
        deletes: vec![],
        relabels: vec![(0, 1), (1, 0)],
        creates: vec![],
        redges: vec![vec![0, 1]],
        distinct: vec![],

        effects: vec![],
    }
}

pub(crate) fn rcf_guard_heavy() -> RuleSpec {
    RuleSpec {
        priority: 0,
        n_vars: 2,
        var_labels: vec![None, None],
        pedges: vec![vec![0, 1]],
        guards: vec![
            GuardSpec {
                var: 0,
                key: 100,
                op: 1,
                val: PropertyValue::Float(0.5),
            },
            GuardSpec {
                var: 1,
                key: 7,
                op: 2,
                val: PropertyValue::Int(3),
            },
        ],
        deletes: vec![],
        relabels: vec![],
        creates: vec![],
        redges: vec![vec![0, 1]],
        distinct: vec![],

        effects: vec![],
    }
}

/// Distinct-bearing base (E1d): triangle-ish rewrite with two pairs.
pub(crate) fn rcf_distinct_base() -> RuleSpec {
    RuleSpec {
        priority: 0,
        n_vars: 3,
        var_labels: vec![Some(0), Some(0), Some(0)],
        pedges: vec![vec![0, 1, 2]],
        guards: vec![],
        deletes: vec![],
        relabels: vec![(0, 1)],
        creates: vec![],
        redges: vec![],
        distinct: vec![(0, 1), (1, 2)],

        effects: vec![],
    }
}

pub fn gate_rcf() -> bool {
    let mut ok = true;
    let mut n_eq = 0u32;
    let mut n_ne = 0u32;
    let check_eq = |a: &RuleSpec, b: &RuleSpec, name: &str, ok: &mut bool, n: &mut u32| {
        *n += 1;
        if a.rcf_bytes() != b.rcf_bytes() || a.rcf_hash() != b.rcf_hash() {
            println!("G-R0 FAIL [must-collide]: {name}\n  a={a:?}\n  b={b:?}");
            *ok = false;
        }
    };
    let check_ne = |a: &RuleSpec, b: &RuleSpec, name: &str, ok: &mut bool, n: &mut u32| {
        *n += 1;
        if a.rcf_bytes() == b.rcf_bytes() || a.rcf_hash() == b.rcf_hash() {
            println!("G-R0 FAIL [must-distinct]: {name}\n  a={a:?}\n  b={b:?}");
            *ok = false;
        }
    };

    // ---- must-collide: hand-written var renamings ----
    {
        // C1: 2-var swap, labels + edges carried (the class_hash counterexample).
        let a = RuleSpec {
            priority: 0,
            n_vars: 2,
            var_labels: vec![Some(0), None],
            pedges: vec![vec![0, 1]],
            guards: vec![],
            deletes: vec![],
            relabels: vec![],
            creates: vec![],
            redges: vec![vec![0, 1]],
            distinct: vec![],

            effects: vec![],
        };
        let b = RuleSpec {
            priority: 0,
            n_vars: 2,
            var_labels: vec![None, Some(0)],
            pedges: vec![vec![1, 0]],
            guards: vec![],
            deletes: vec![],
            relabels: vec![],
            creates: vec![],
            redges: vec![vec![1, 0]],
            distinct: vec![],

            effects: vec![],
        };
        check_eq(&a, &b, "C1 hand 2-swap", &mut ok, &mut n_eq);
        // The legacy hash must NOT collide here (documents why RCF replaced it).
        if a.class_hash() == b.class_hash() {
            println!("G-R0 FAIL [legacy]: class_hash collides on C1 — expected divergence");
            ok = false;
        }
    }
    {
        // C2: hand 3-cycle with labels, guard, delete, relabel.
        let a = RuleSpec {
            priority: 1,
            n_vars: 3,
            var_labels: vec![Some(0), Some(1), None],
            pedges: vec![vec![0, 1, 2]],
            guards: vec![GuardSpec {
                var: 2,
                key: 9,
                op: 2,
                val: PropertyValue::Int(7),
            }],
            deletes: vec![],
            relabels: vec![(0, 5)],
            creates: vec![9],
            redges: vec![vec![1, 2, 3]],
            distinct: vec![],

            effects: vec![],
        };
        // p = [1,2,0]: old0->1, old1->2, old2->0
        let b = permute_spec(&a, &[1, 2, 0]);
        // Hand-verify one field so the helper cannot self-confirm alone:
        assert_eq!(b.var_labels, vec![None, Some(0), Some(1)]);
        assert_eq!(b.pedges, vec![vec![1, 2, 0]]);
        check_eq(&a, &b, "C2 hand-checked 3-cycle", &mut ok, &mut n_eq);
    }
    // ---- must-collide: helper renamings over rich bases ----
    {
        let base = rcf_rich_base();
        check_eq(
            &base,
            &permute_spec(&base, &[3, 2, 1, 0]),
            "C3 rich reversal",
            &mut ok,
            &mut n_eq,
        );
        check_eq(
            &base,
            &permute_spec(&base, &[1, 0, 2, 3]),
            "C4 rich swap01",
            &mut ok,
            &mut n_eq,
        );
        check_eq(
            &base,
            &permute_spec(&base, &[0, 1, 3, 2]),
            "C5 rich swap23",
            &mut ok,
            &mut n_eq,
        );
        check_eq(
            &base,
            &permute_spec(&base, &[1, 2, 3, 0]),
            "C6 rich 4-cycle",
            &mut ok,
            &mut n_eq,
        );
        check_eq(
            &base,
            &permute_spec(&base, &[1, 0, 3, 2]),
            "C7 rich double-swap",
            &mut ok,
            &mut n_eq,
        );
        let k = rcf_churnk_like();
        check_eq(
            &k,
            &permute_spec(&k, &[1, 0]),
            "C8 churnK-like swap",
            &mut ok,
            &mut n_eq,
        );
        let g = rcf_guard_heavy();
        check_eq(
            &g,
            &permute_spec(&g, &[1, 0]),
            "C9 guard-heavy swap",
            &mut ok,
            &mut n_eq,
        );
    }
    // ---- must-collide: reordering within unordered contexts ----
    {
        let a = rcf_rich_base();
        let mut b = a.clone();
        b.pedges.reverse();
        check_eq(&a, &b, "C10 LHS edge reorder", &mut ok, &mut n_eq);
        let mut c = a.clone();
        c.redges.reverse();
        check_eq(&a, &c, "C11 RHS edge reorder", &mut ok, &mut n_eq);
        let g = rcf_guard_heavy();
        let mut g2 = g.clone();
        g2.guards.reverse();
        check_eq(&g, &g2, "C12 guard reorder", &mut ok, &mut n_eq);
        let k = rcf_churnk_like();
        let mut k2 = k.clone();
        k2.relabels.reverse();
        check_eq(&k, &k2, "C13 relabel reorder", &mut ok, &mut n_eq);
        let d = RuleSpec {
            priority: 0,
            n_vars: 3,
            var_labels: vec![Some(3), Some(3), Some(3)],
            pedges: vec![vec![0, 1], vec![1, 2]],
            guards: vec![],
            deletes: vec![0, 2],
            relabels: vec![],
            creates: vec![],
            redges: vec![],
            distinct: vec![],

            effects: vec![],
        };
        let mut d2 = d.clone();
        d2.deletes.reverse();
        check_eq(&d, &d2, "C14 delete-list reorder", &mut ok, &mut n_eq);
        // ---- isolated-var renaming (var in no edge) ----
        let i1 = RuleSpec {
            priority: 0,
            n_vars: 2,
            var_labels: vec![Some(0), None],
            pedges: vec![],
            guards: vec![GuardSpec {
                var: 1,
                key: 4,
                op: 0,
                val: PropertyValue::Int(1),
            }],
            deletes: vec![],
            relabels: vec![],
            creates: vec![2],
            redges: vec![vec![0, 1]],
            distinct: vec![],

            effects: vec![],
        };
        check_eq(
            &i1,
            &permute_spec(&i1, &[1, 0]),
            "C15 isolated-var rename",
            &mut ok,
            &mut n_eq,
        );
        // ---- combined rename + outer reorder ----
        let mut cb = permute_spec(&a, &[2, 3, 0, 1]);
        cb.pedges.reverse();
        cb.redges.reverse();
        check_eq(&a, &cb, "C16 rename+reorder", &mut ok, &mut n_eq);
        // ---- create refs untouched by renaming ----
        let m = RuleSpec {
            priority: 0,
            n_vars: 2,
            var_labels: vec![Some(2), None],
            pedges: vec![],
            guards: vec![],
            deletes: vec![],
            relabels: vec![],
            creates: vec![0, 1],
            redges: vec![vec![0, 2], vec![1, 3], vec![2, 3]],
            distinct: vec![],

            effects: vec![],
        };
        check_eq(
            &m,
            &permute_spec(&m, &[1, 0]),
            "C17 create-ref rename",
            &mut ok,
            &mut n_eq,
        );
        // ---- independently constructed identical spec ----
        let j1 = rcf_churnk_like();
        let j2 = rcf_churnk_like();
        check_eq(&j1, &j2, "C18 identical construction", &mut ok, &mut n_eq);
    }
    // ---- must-collide: whole-region translation across stores ----
    {
        let caps = Caps::default();
        for (base, name) in [
            (rcf_rich_base(), "C19 rich"),
            (rcf_churnk_like(), "C20 churnK-like"),
        ] {
            let mut s1 = HypergraphStore::new();
            let mut q1 = 0i64;
            let r1 = install_rule(&mut s1, &base, &mut q1);
            let d1 = decode_spec(&s1, r1, &caps).expect("fixture must decode");
            let mut s2 = HypergraphStore::new();
            let mut q2 = 100i64;
            let decoy = RuleSpec {
                priority: -5,
                n_vars: 1,
                var_labels: vec![None],
                pedges: vec![],
                guards: vec![],
                deletes: vec![],
                relabels: vec![],
                creates: vec![],
                redges: vec![],
                distinct: vec![],

                effects: vec![],
            };
            install_rule(&mut s2, &decoy, &mut q2);
            let r2 = install_rule(&mut s2, &base, &mut q2);
            let d2 = decode_spec(&s2, r2, &caps).expect("fixture must decode in 2nd store");
            check_eq(&d1, &d2, name, &mut ok, &mut n_eq);
        }
    }
    // ---- must-collide: distinct pairs carried through renaming (E1d) ----
    {
        let t = rcf_distinct_base();
        check_eq(
            &t,
            &permute_spec(&t, &[2, 1, 0]),
            "C21 distinct reversal",
            &mut ok,
            &mut n_eq,
        );
        check_eq(
            &t,
            &permute_spec(&t, &[1, 2, 0]),
            "C22 distinct 3-cycle",
            &mut ok,
            &mut n_eq,
        );
        // Hand-check: reversal maps pairs {(0,1),(1,2)} -> {(1,2),(0,1)}.
        let tr = permute_spec(&t, &[2, 1, 0]);
        let mut got = tr.distinct.clone();
        got.sort_unstable();
        assert_eq!(got, vec![(0, 1), (1, 2)]);
    }
    // ---- must-collide: effects carried through renaming (E2) ----
    // (must-distinct effect cases D18-D20 live in the block below, next to
    // the value they vary — counted in the same gate.)
    {
        // The doc's chaining example: scale then subtract (order matters).
        let fx = RuleSpec {
            priority: 0,
            n_vars: 2,
            var_labels: vec![Some(0), None],
            pedges: vec![vec![0, 1]],
            guards: vec![],
            deletes: vec![],
            relabels: vec![],
            creates: vec![5],
            redges: vec![vec![0, 2]],
            distinct: vec![],
            effects: vec![
                EffectSpec {
                    target: 0,
                    key: 1,
                    op: EffectOp::ScaleFrom {
                        source: 0,
                        source_key: 1,
                        factor: 0.8,
                    },
                },
                EffectSpec {
                    target: 2,
                    key: 1,
                    op: EffectOp::Subtract {
                        value: PropertyValue::Int(100),
                    },
                },
            ],
        };
        check_eq(
            &fx,
            &permute_spec(&fx, &[1, 0]),
            "C23 effect rename",
            &mut ok,
            &mut n_eq,
        );
        // Hand-check: swap maps target 0->1, source 0->1, create ref 2 stays.
        let fr = permute_spec(&fx, &[1, 0]);
        assert!(matches!(
            fr.effects[0].op,
            EffectOp::ScaleFrom { source: 1, factor, .. } if factor == 0.8
        ));
        assert_eq!(fr.effects[0].target, 1);
        assert_eq!(fr.effects[1].target, 2);
        let mut rev = fx.clone();
        rev.effects.reverse();
        check_ne(&fx, &rev, "D19 effect order swapped", &mut ok, &mut n_ne);
        let mut chg = fx.clone();
        chg.effects[1].op = EffectOp::Add {
            value: PropertyValue::Int(100),
        };
        check_ne(&fx, &chg, "D20 effect op changed", &mut ok, &mut n_ne);
        let nofx = RuleSpec {
            effects: vec![],
            ..fx.clone()
        };
        check_ne(&fx, &nofx, "D18 effect dropped", &mut ok, &mut n_ne);
    }

    // ---- must-distinct ----
    {
        let base = RuleSpec {
            priority: 0,
            n_vars: 2,
            var_labels: vec![Some(0), None],
            pedges: vec![vec![0, 1]],
            guards: vec![],
            deletes: vec![],
            relabels: vec![],
            creates: vec![],
            redges: vec![vec![0, 1]],
            distinct: vec![],

            effects: vec![],
        };
        // D1: slot permutation, labels fixed (no renaming maps these).
        let mut s1 = base.clone();
        s1.pedges = vec![vec![1, 0]];
        check_ne(&base, &s1, "D1 LHS slot permutation", &mut ok, &mut n_ne);
        let mut s1b = base.clone();
        s1b.redges = vec![vec![1, 0]];
        check_ne(&base, &s1b, "D2 RHS slot permutation", &mut ok, &mut n_ne);
        // D3: var moved LHS-only -> both sides (drop the redge).
        let mut lhs_only = base.clone();
        lhs_only.redges = vec![];
        check_ne(
            &base,
            &lhs_only,
            "D3 LHS-only vs both-sides",
            &mut ok,
            &mut n_ne,
        );
        // D4/D5: label constraint add/remove.
        let mut add_lab = base.clone();
        add_lab.var_labels = vec![Some(0), Some(1)];
        check_ne(
            &base,
            &add_lab,
            "D4 add label constraint",
            &mut ok,
            &mut n_ne,
        );
        let mut rm_lab = base.clone();
        rm_lab.var_labels = vec![None, None];
        check_ne(
            &base,
            &rm_lab,
            "D5 remove label constraint",
            &mut ok,
            &mut n_ne,
        );
        // D6: priority change.
        let mut pri = base.clone();
        pri.priority = 7;
        check_ne(&base, &pri, "D6 priority change", &mut ok, &mut n_ne);
        // D7: disconnected pattern edge added.
        let extra = RuleSpec {
            priority: 0,
            n_vars: 3,
            var_labels: vec![Some(0), None, None],
            pedges: vec![vec![0, 1]],
            guards: vec![],
            deletes: vec![],
            relabels: vec![],
            creates: vec![],
            redges: vec![vec![0, 1]],
            distinct: vec![],

            effects: vec![],
        };
        let mut extra2 = extra.clone();
        extra2.pedges.push(vec![1, 2]);
        check_ne(
            &extra,
            &extra2,
            "D7 disconnected edge added",
            &mut ok,
            &mut n_ne,
        );
        // D8a/b/c: guard add / op change / threshold change.
        let mut gadd = base.clone();
        gadd.guards = vec![GuardSpec {
            var: 0,
            key: 4,
            op: 0,
            val: PropertyValue::Int(1),
        }];
        check_ne(&base, &gadd, "D8 guard added", &mut ok, &mut n_ne);
        let mut gop = gadd.clone();
        gop.guards[0].op = 1;
        check_ne(&gadd, &gop, "D9 guard op changed", &mut ok, &mut n_ne);
        let mut gval = gadd.clone();
        gval.guards[0].val = PropertyValue::Int(2);
        check_ne(
            &gadd,
            &gval,
            "D10 guard threshold changed",
            &mut ok,
            &mut n_ne,
        );
        // D11: survival status change.
        let mut del = base.clone();
        del.deletes = vec![1];
        del.redges = vec![];
        check_ne(&base, &del, "D11 delete added", &mut ok, &mut n_ne);
        // D12: create label change.
        let c1 = RuleSpec {
            priority: 0,
            n_vars: 1,
            var_labels: vec![Some(0)],
            pedges: vec![],
            guards: vec![],
            deletes: vec![],
            relabels: vec![],
            creates: vec![2],
            redges: vec![],
            distinct: vec![],

            effects: vec![],
        };
        let mut c2 = c1.clone();
        c2.creates = vec![3];
        check_ne(&c1, &c2, "D12 create label changed", &mut ok, &mut n_ne);
        // D13: relabel target change.
        let r1 = rcf_churnk_like();
        let mut r2 = r1.clone();
        r2.relabels = vec![(0, 1), (1, 1)];
        check_ne(&r1, &r2, "D13 relabel target changed", &mut ok, &mut n_ne);
        // D14: arity change with n_vars fixed.
        let ar3 = RuleSpec {
            priority: 0,
            n_vars: 3,
            var_labels: vec![None, None, None],
            pedges: vec![vec![0, 1]],
            guards: vec![],
            deletes: vec![],
            relabels: vec![],
            creates: vec![],
            redges: vec![],
            distinct: vec![],

            effects: vec![],
        };
        let mut ar3b = ar3.clone();
        ar3b.pedges = vec![vec![0, 1, 2]];
        check_ne(&ar3, &ar3b, "D14 arity changed", &mut ok, &mut n_ne);
        // D15/D16: distinct pairs participate in identity (E1d).
        let t = rcf_distinct_base();
        let mut t_add = t.clone();
        t_add.distinct.push((0, 2));
        t_add.distinct.sort_unstable();
        check_ne(&t, &t_add, "D15 distinct pair added", &mut ok, &mut n_ne);
        let mut t_chg = t.clone();
        t_chg.distinct = vec![(0, 1), (0, 2)];
        check_ne(&t, &t_chg, "D16 distinct pair changed", &mut ok, &mut n_ne);
        let mut t_drop = t.clone();
        t_drop.distinct = vec![];
        check_ne(&t, &t_drop, "D17 distinct dropped", &mut ok, &mut n_ne);
    }

    if ok {
        println!("G-R0 PASS: {n_eq} must-collide + {n_ne} must-distinct RCF fixtures hold; legacy class_hash diverges on C1 as required");
    } else {
        println!("G-R0 FAIL: see mismatches above ({n_eq} collide + {n_ne} distinct checked)");
    }
    ok
}

// ---------- G-R4: quarantine engagement (docs/ere_validators.md) ----------

/// Build the adversarial regions (one isolated violation per validator, V6
/// covered twice: relabel-range and distinct-range) in a single store.
/// Returns `(label, root, expected_validator)` per case. Every case must
/// decode to `None` incrementing exactly its own counter.
fn gr4_malformed(rs: &mut HypergraphStore, seq: &mut i64) -> Vec<(String, NodeId, usize)> {
    // ChurnK-like base: valid under V1..V9 (non-vacuous), the sabotage ground.
    let base = || RuleSpec {
        priority: 0,
        n_vars: 2,
        var_labels: vec![Some(0), Some(1)],
        pedges: vec![vec![0, 1]],
        guards: vec![],
        deletes: vec![],
        relabels: vec![(0, 1), (1, 0)],
        creates: vec![],
        redges: vec![vec![0, 1]],
        distinct: vec![],

        effects: vec![],
    };
    let mut out = Vec::new();

    // M1/V1: n_vars = 0.
    let m1 = install_rule(rs, &base(), seq);
    set_int(rs, m1, K_NVARS, 0);
    out.push(("V1 n_vars=0".to_string(), m1, V1));

    // M2/V2: PEdge arity beyond cap (5 > max_arity 3).
    let m2 = install_rule(rs, &base(), seq);
    {
        let mut props = vec![(K_ARITY, PropertyValue::Int(5))];
        for i in 0..5u32 {
            props.push((K_EP0 + i, PropertyValue::Int(0)));
        }
        let pe = add_rule_node(rs, PEDGE, props);
        add_role_edge(rs, m2, pe, HAS_PEDGE);
    }
    out.push(("V2 pedge arity 5 over cap".to_string(), m2, V2));

    // M3/V3: 5 individually-valid binary pedges (1 + 4 added) over the cap of 4.
    let m3 = install_rule(rs, &base(), seq);
    for _ in 0..4 {
        let pe = add_rule_node(
            rs,
            PEDGE,
            vec![
                (K_ARITY, PropertyValue::Int(2)),
                (K_EP0, PropertyValue::Int(0)),
                (K_EP0 + 1, PropertyValue::Int(1)),
            ],
        );
        add_role_edge(rs, m3, pe, HAS_PEDGE);
    }
    out.push(("V3 five pedges over cap".to_string(), m3, V3));

    // M4/V4: guard var out of range.
    let m4 = install_rule(rs, &base(), seq);
    {
        let gn = add_rule_node(
            rs,
            GUARD,
            vec![
                (K_GVAR, PropertyValue::Int(7)),
                (K_GKEY, PropertyValue::Int(1)),
                (K_GOP, PropertyValue::Int(0)),
                (K_GVAL, PropertyValue::Int(0)),
            ],
        );
        add_role_edge(rs, m4, gn, HAS_GUARD);
    }
    out.push(("V4 guard var out of range".to_string(), m4, V4));

    // M5/V5: delete var out of range.
    let m5 = install_rule(rs, &base(), seq);
    {
        let dn = add_rule_node(rs, DELETE, vec![(K_VAR_ID, PropertyValue::Int(7))]);
        add_role_edge(rs, m5, dn, HAS_DELETE);
    }
    out.push(("V5 delete var out of range".to_string(), m5, V5));

    // M6/V6: relabel var out of range (relabel-of-deleted is inert, not this).
    let m6 = install_rule(rs, &base(), seq);
    {
        let rn = add_rule_node(
            rs,
            RELABEL,
            vec![
                (K_VAR_ID, PropertyValue::Int(7)),
                (K_NEW_LABEL, PropertyValue::Int(0)),
            ],
        );
        add_role_edge(rs, m6, rn, HAS_RELABEL);
    }
    out.push(("V6 relabel var out of range".to_string(), m6, V6));

    // M6b/V6: DISTINCT pair var out of range (E1d; same counter, second arm).
    let m6b = install_rule(rs, &base(), seq);
    {
        let dn = add_rule_node(
            rs,
            DISTINCT,
            vec![
                (K_VAR_ID, PropertyValue::Int(0)),
                (K_VAR_ID2, PropertyValue::Int(7)),
            ],
        );
        add_role_edge(rs, m6b, dn, HAS_DISTINCT);
    }
    out.push(("V6 distinct var out of range".to_string(), m6b, V6));

    // M7/V7: create indices with a gap (0, 2).
    let m7 = install_rule(
        rs,
        &RuleSpec {
            priority: 0,
            n_vars: 1,
            var_labels: vec![None],
            pedges: vec![],
            guards: vec![],
            deletes: vec![],
            relabels: vec![],
            creates: vec![0, 1],
            redges: vec![],
            distinct: vec![],

            effects: vec![],
        },
        seq,
    );
    for cn in children(rs, m7, HAS_CREATE) {
        if get_int(rs, cn, K_CREATE_INDEX) == Some(1) {
            set_int(rs, cn, K_CREATE_INDEX, 2);
        }
    }
    out.push(("V7 create index gap".to_string(), m7, V7));

    // M8/V8: REdge ref to a deleted var (delete of var 1 is itself valid).
    let m8 = install_rule(rs, &base(), seq);
    {
        let dn = add_rule_node(rs, DELETE, vec![(K_VAR_ID, PropertyValue::Int(1))]);
        add_role_edge(rs, m8, dn, HAS_DELETE);
    }
    out.push(("V8 redge ref to deleted var".to_string(), m8, V8));

    // M9/V9: vacuous rule — no pedges, deletes, relabels, creates, or redges,
    // hence no possible store effect (pure match-volume load).
    let m9 = install_rule(
        rs,
        &RuleSpec {
            priority: 0,
            n_vars: 1,
            var_labels: vec![Some(0)],
            pedges: vec![],
            guards: vec![GuardSpec {
                var: 0,
                key: 1,
                op: 0,
                val: PropertyValue::Int(0),
            }],
            deletes: vec![],
            relabels: vec![],
            creates: vec![],
            redges: vec![],
            distinct: vec![],

            effects: vec![],
        },
        seq,
    );
    out.push(("V9 vacuous rule".to_string(), m9, V9));

    // M9b/V9: EFFECT target out of range (E2; same counter, second arm).
    let m9b = install_rule(rs, &base(), seq);
    {
        let en = add_rule_node(
            rs,
            EFFECT,
            vec![
                (K_ETARGET, PropertyValue::Int(9)),
                (K_EKEY, PropertyValue::Int(1)),
                (K_EOP, PropertyValue::Int(0)),
                (K_EVAL, PropertyValue::Int(1)),
            ],
        );
        add_role_edge(rs, m9b, en, HAS_EFFECT);
    }
    out.push(("V9 effect target out of range".to_string(), m9b, V9));

    out
}

pub fn gate_gr4() -> bool {
    let caps = Caps::default();
    let mut ok = true;

    // 1. Each adversarial region quarantines against exactly its validator.
    let mut rs = HypergraphStore::new();
    let mut seq = 0i64;
    let cases = gr4_malformed(&mut rs, &mut seq);
    assert_eq!(
        cases.len(),
        N_VALIDATORS + 2,
        "one adversarial case per validator (V6 and V9 twice)"
    );
    for (label, root, want) in &cases {
        let mut counters = [0usize; N_VALIDATORS];
        let got = decode_spec_counted(&rs, *root, &caps, &mut counters);
        if got.is_some() {
            println!("G-R4 FAIL [{label}]: decoded to Some (expected quarantine)");
            ok = false;
            continue;
        }
        for (i, c) in counters.iter().enumerate() {
            let want_c = usize::from(i == *want);
            if *c != want_c {
                println!(
                    "G-R4 FAIL [{label}]: counter V{} = {c} (expected {want_c})",
                    i + 1
                );
                ok = false;
            }
        }
    }

    // 2. Aggregate report over the adversarial store: total 11,
    // V6 == 2 (relabel + distinct arms), V9 == 2 (vacuous + effect arms).
    {
        let mut want = [1usize; N_VALIDATORS];
        want[V6] = 2;
        want[V9] = 2;
        let (rules, report) = decode_all(&rs, &caps);
        if !rules.is_empty() || report.total != N_VALIDATORS + 2 || report.by_validator != want {
            println!(
                "G-R4 FAIL [aggregate]: rules={} total={} by_validator={:?} (expected 0/11/{:?})",
                rules.len(),
                report.total,
                report.by_validator,
                want
            );
            ok = false;
        }
    }

    // 3. Ordinary valid specs engage no counter (zeros are reported, not absent).
    {
        let mut vs = HypergraphStore::new();
        let mut q = 0i64;
        for spec in seed_rule_specs() {
            install_rule(&mut vs, &spec, &mut q);
        }
        let (rules, report) = decode_all(&vs, &caps);
        if rules.len() != 4 || report.total != 0 || report.by_validator != [0; N_VALIDATORS] {
            println!(
                "G-R4 FAIL [clean]: rules={} total={} by_validator={:?} (expected 4/0/[0;9])",
                rules.len(),
                report.total,
                report.by_validator
            );
            ok = false;
        }
    }

    if ok {
        println!(
            "G-R4 PASS: 11 adversarial regions each engage exactly their validator (V6 and V9 twice); clean seed set reports [0;9]"
        );
    }
    ok
}

// ---------- D1/D2: determinism hazards (docs/ere_determinism.md) ----------

/// Two same-priority, same-pattern rules relabeling one node apart (the D1
/// sharp end: the priority tie is decided by rule_id alone), plus a
/// high-priority disjoint rule so priority-vs-insertion is also covered.
pub(crate) fn d1_tie_specs() -> (RuleSpec, RuleSpec, RuleSpec) {
    let mk = |new_label: u32| RuleSpec {
        priority: 0,
        n_vars: 2,
        var_labels: vec![Some(0), Some(1)],
        pedges: vec![vec![0, 1]],
        guards: vec![],
        deletes: vec![],
        relabels: vec![(0, new_label)],
        creates: vec![],
        redges: vec![],
        distinct: vec![],

        effects: vec![],
    };
    let hi = RuleSpec {
        priority: 5,
        n_vars: 1,
        var_labels: vec![Some(7)],
        pedges: vec![],
        guards: vec![],
        deletes: vec![0],
        relabels: vec![],
        creates: vec![],
        redges: vec![],
        distinct: vec![],

        effects: vec![],
    };
    (mk(9), mk(8), hi)
}

fn d1_tie_world() -> HypergraphStore {
    let mut w = HypergraphStore::new();
    let a = w.add_node(NodeData::new(Label(0), 0));
    let b = w.add_node(NodeData::new(Label(1), 0));
    w.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[a, b])));
    w
}

pub fn gate_d1() -> bool {
    let mut ok = true;
    let caps = Caps::default();
    let cfg = RunConfig::default();
    let (spec_a, spec_b, spec_hi) = d1_tie_specs();

    // Store X: install order A,B,Hi from seq 0.
    let mut sx = HypergraphStore::new();
    let mut qx = 0i64;
    install_rule(&mut sx, &spec_a, &mut qx);
    install_rule(&mut sx, &spec_b, &mut qx);
    install_rule(&mut sx, &spec_hi, &mut qx);
    // Store Y: different order + different seq base + delete/re-add cycle
    // (slot reuse), so ffi layout and install history both differ.
    let mut sy = HypergraphStore::new();
    let mut qy = 500i64;
    install_rule(&mut sy, &spec_hi, &mut qy);
    let decoy = install_rule(&mut sy, &rcf_churnk_like(), &mut qy);
    delete_rule(&mut sy, decoy);
    install_rule(&mut sy, &spec_a, &mut qy);
    install_rule(&mut sy, &spec_b, &mut qy);

    // Part 1: identical rule_id assignment (full content identity per id).
    let (dx, rx) = decode_all(&sx, &caps);
    let (dy, ry) = decode_all(&sy, &caps);
    if rx.total != 0 || ry.total != 0 || dx.len() != 3 || dy.len() != 3 {
        println!(
            "G-D1 FAIL [assign]: quarantines x={} y={} lens {}/{} (expected 0/0/3/3)",
            rx.total,
            ry.total,
            dx.len(),
            dy.len()
        );
        ok = false;
    } else {
        for (i, (a, b)) in dx.iter().zip(dy.iter()).enumerate() {
            if a.spec.rcf_bytes() != b.spec.rcf_bytes() {
                println!("G-D1 FAIL [assign]: rule_id {i} differs across insertion orders");
                ok = false;
            }
        }
    }

    // Part 2: identical dynamics over a priority-tie world (10 steps).
    let run = |rules: &[DecodedRule], seed: u64| -> (Vec<u64>, u64) {
        let mut w = d1_tie_world();
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let budgets = unlimited_budgets(rules);
        let mut hashes = Vec::new();
        let mut first_applied = 0usize;
        for step in 0..10 {
            let oo = object_step(&mut w, rules, &mut rng, &cfg, &budgets);
            if step == 0 {
                first_applied = oo.applied;
            }
            hashes.push(w.canonical_hash());
        }
        (hashes, first_applied as u64)
    };
    if ok {
        let (hx, ax) = run(&dx, 99);
        let (hy, ay) = run(&dy, 99);
        if ax != 1 || ay != 1 {
            println!(
                "G-D1 FAIL [tie]: first-step applied x={ax} y={ay} (expected 1 — tie not engaged)"
            );
            ok = false;
        } else if hx != hy {
            println!("G-D1 FAIL [tie]: trajectories diverge across insertion orders");
            ok = false;
        }
    }

    // Part 3: clone interchangeability — identical-spec copies in different
    // install positions still assign and run identically.
    {
        let mut se = HypergraphStore::new();
        let mut qe = 0i64;
        install_rule(&mut se, &spec_a, &mut qe);
        install_rule(&mut se, &spec_a, &mut qe);
        install_rule(&mut se, &spec_b, &mut qe);
        let mut sf = HypergraphStore::new();
        let mut qf = 0i64;
        install_rule(&mut sf, &spec_b, &mut qf);
        install_rule(&mut sf, &spec_a, &mut qf);
        install_rule(&mut sf, &spec_a, &mut qf);
        let (de, re) = decode_all(&se, &caps);
        let (df, rf) = decode_all(&sf, &caps);
        let seq_e: Vec<Vec<u8>> = de.iter().map(|d| d.spec.rcf_bytes()).collect();
        let seq_f: Vec<Vec<u8>> = df.iter().map(|d| d.spec.rcf_bytes()).collect();
        if re.total != 0 || rf.total != 0 || seq_e != seq_f {
            println!("G-D1 FAIL [clones]: spec sequences differ across install positions");
            ok = false;
        } else {
            let (he, _) = run(&de, 7);
            let (hf, _) = run(&df, 7);
            if he != hf {
                println!("G-D1 FAIL [clones]: clone trajectories diverge");
                ok = false;
            }
        }
    }

    if ok {
        println!("G-D1 PASS: id assignment + 10-step tie trajectories identical across insertion orders, seq bases, slot reuse, and clone positions");
    }
    ok
}

/// Golden RCF hashes (see docs/ere_determinism.md). Harvested from the
/// implementation; update deliberately, never silently.
pub const GOLDEN_RCF_CHURNK: u64 = 0x9b5814caf8c1197b;
pub const GOLDEN_RCF_RICH: u64 = 0x731e9bcaa2ee5ecf;
const D2_STREAM_SEED: u64 = 0xD1D2;

pub fn gate_d2() -> bool {
    let mut ok = true;
    let caps = Caps::default();
    let cfg = RunConfig::default();

    // Part 1: the decode + step workload draws nothing from the RNG stream.
    let mut adv = HypergraphStore::new();
    let mut q = 0i64;
    for spec in seed_rule_specs() {
        install_rule(&mut adv, &spec, &mut q);
    }
    install_rule(&mut adv, &rcf_churnk_like(), &mut q);
    let mut r1 = ChaCha8Rng::seed_from_u64(D2_STREAM_SEED);
    let first: u64 = r1.gen();
    let (rules, _report) = decode_all(&adv, &caps);
    let budgets = unlimited_budgets(&rules);
    let mut w = d1_tie_world();
    for _ in 0..20 {
        let _ = object_step(&mut w, &rules, &mut r1, &cfg, &budgets);
    }
    // RCF normalization also draws nothing (pure function — exercised here
    // so a future RNG-threaded normalization would trip the stream check
    // only if it also broke part 2's goldens; belt and suspenders).
    for d in &rules {
        let _ = d.spec.rcf_hash();
    }
    let second: u64 = r1.gen();
    let mut r2 = ChaCha8Rng::seed_from_u64(D2_STREAM_SEED);
    if first != r2.gen::<u64>() || second != r2.gen::<u64>() {
        println!("G-D2 FAIL [stream]: decode+steps consumed RNG draws");
        ok = false;
    }

    // Part 2: golden RCF values pin algorithm + hasher + fixtures.
    let churnk = rcf_churnk_like().rcf_hash();
    let rich = rcf_rich_base().rcf_hash();
    println!("  goldens: churnK={churnk:#018x} rich={rich:#018x}");
    if churnk != GOLDEN_RCF_CHURNK {
        println!("G-D2 FAIL [golden]: churnK {churnk:#018x} != {GOLDEN_RCF_CHURNK:#018x}");
        ok = false;
    }
    if rich != GOLDEN_RCF_RICH {
        println!("G-D2 FAIL [golden]: rich {rich:#018x} != {GOLDEN_RCF_RICH:#018x}");
        ok = false;
    }

    if ok {
        println!("G-D2 PASS: zero RNG draws across decode+20 steps; golden RCF hashes hold");
    }
    ok
}
