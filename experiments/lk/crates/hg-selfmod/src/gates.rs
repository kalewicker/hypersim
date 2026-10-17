//! Gates G0–G2 (plan §8). Each blocks the next; G1 is the binding
//! acceptance test for the port (v2 F1 on the real engine).

use crate::decode::{decode_all, decode_spec, DecodedRule};
use crate::economy::{EconomyMode, IncomeMode};
use crate::encode::install_rule;
use crate::resolve::TieBreak;
use crate::runner::{
    object_step, run_phase, seed_rule_specs, seed_world, RunConfig, TruncationMode,
};
use crate::schema::*;
use crate::store_ext::*;
use hg_core::{HyperEdge, HypergraphStore, Label, NodeData, NodeId, PropertyValue};
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

fn g0_fixtures() -> Vec<RuleSpec> {
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

    if ok {
        println!(
            "G0 PASS: {} fixtures round-trip (spec + class_hash), 5 malformed subgraphs -> None, no panic",
            fixtures.len()
        );
    }
    ok
}

// ---------- G1: reification neutrality ----------

/// Fixture ruleset for G1: the seed rules plus two enriched-class rules
/// (arity-3 hyperedge creation and consumption) so the neutrality claim
/// covers the full pattern class, not just the v2 subset.
fn g1_specs() -> Vec<RuleSpec> {
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
        let (direct_rules, dfail) = decode_all(&rulestore, &caps);
        assert_eq!(dfail, 0, "G1 fixtures must all decode");

        let mut wd = base.clone();
        let mut we = base.clone();
        let mut rng_d = ChaCha8Rng::seed_from_u64(seed ^ 0xABCD);
        let mut rng_e = ChaCha8Rng::seed_from_u64(seed ^ 0xABCD);

        let mut first_div: Option<usize> = None;
        for step in 0..steps {
            let bd = unlimited_budgets(&direct_rules);
            let _ = object_step(&mut wd, &direct_rules, &mut rng_d, &cfg, &bd, &[]);
            let rules_e: (Vec<DecodedRule>, usize) = decode_all(&rulestore, &caps);
            let be = unlimited_budgets(&rules_e.0);
            let _ = object_step(&mut we, &rules_e.0, &mut rng_e, &cfg, &be, &[]);
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
    let (rules, dfail) = decode_all(&rulestore, &caps);
    assert_eq!(dfail, 0, "G2 ruleset must decode");

    let window = 100usize;
    let mut window_prod = 0usize;
    let mut dead_windows = 0usize;
    let mut hashes: Vec<u64> = Vec::new();
    for step in 0..steps {
        let budgets = unlimited_budgets(&rules);
        let oo = object_step(&mut world, &rules, &mut rng, &cfg, &budgets, &[]);
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
    let (rules, dfail) = decode_all(&rulestore, &caps);
    assert_eq!(dfail, 0, "{label} ruleset must decode");
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
        let oo = object_step(&mut world, &rules, &mut rng, &cfg, &budgets, &[]);
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
    let (rules, dfail) = decode_all(&rulestore, &caps);
    if dfail != 0 || rules.is_empty() {
        println!("  G4.2 FAIL [{name}]: fixture rule did not decode");
        return false;
    }
    let budgets = unlimited_budgets(&rules);
    let mut rng = ChaCha8Rng::seed_from_u64(1);
    let oo = object_step(&mut world, &rules, &mut rng, &cfg, &budgets, &[]);

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
        },
        false,
    );

    // Empty RHS effect: match a node, change nothing.
    ok &= effect_case(
        "empty RHS (weed)",
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
        },
        false,
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
        },
        true,
    );
    // The churnK sustainer must read productive (it is the E3 substrate).
    ok &= effect_case("churnK label swap", churn_k_specs().remove(0), true);

    println!(
        "G4.2 {}",
        if ok {
            "PASS: all 8 cases classified as specified"
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

// ---------- G5.3: the invasion gate ----------

/// A hand-built variant that is **strictly superior** to `churn3`'s member 2
/// (R3), for the invasion gate.
///
/// Resident R3 is `A(0), D(3)` with **no pattern edge** — two isolated
/// variables, so the matcher enumerates every (label-0, label-3) *pair* in
/// the world: O(n²) candidates for the same one productive effect. The
/// variant adds a pattern edge between them, so it enumerates only adjacent
/// pairs — dramatically fewer candidates — while producing the same
/// relabel (`D → B`) on the pairs it does match.
///
/// Under v4's flat income both cost the same and earn the same, so the
/// variant is invisible to selection. Under v5's match-proportional cost it
/// pays strictly less for a comparable productive yield, which is exactly the
/// "beneficial mutation" the gradient is supposed to be able to see. If this
/// cannot invade, no reachable mutation can either.
pub fn invader_spec() -> RuleSpec {
    RuleSpec {
        priority: 0,
        n_vars: 2,
        var_labels: vec![Some(0), Some(3)],
        pedges: vec![vec![0, 1]], // the whole difference: adjacency required
        guards: vec![],
        deletes: vec![],
        relabels: vec![(1, 1)],
        creates: vec![],
        redges: vec![vec![0, 1]],
    }
}

/// The v6 candidate invader: resident member R3 **plus a guard**, and nothing
/// else changed (§P0c, "resident member + guard excluding the non-productive
/// matches").
///
/// R3 is `A(label 0), D(label 3)` with no pattern edge, so it enumerates every
/// (label-0, label-3) pair. The guard `energy(var0) > theta` shrinks the
/// label-0 side of that product without touching the rewrite: same relabel
/// `D → B`, same emitted edge, same priority.
///
/// `var0` is the right place to put it. `churn3` never creates and never
/// deletes a label-0 node — R1 creates label 2, R2 deletes it, R3 relabels
/// label 3 → 1 — so var0 binds exactly the original seeded nodes for the whole
/// run, and every one of them carries `K_ENERGY` from `seed_world`. The guard
/// is therefore well-defined and its selectivity is stationary, which a guard
/// on the dynamically relabelled label-3 side would not be.
pub fn pareto_invader_spec(theta: f64) -> RuleSpec {
    RuleSpec {
        priority: 0,
        n_vars: 2,
        var_labels: vec![Some(0), Some(3)],
        pedges: vec![],
        guards: vec![GuardSpec {
            var: 0,
            key: K_ENERGY,
            op: 0,
            val: PropertyValue::Float(theta),
        }],
        deletes: vec![],
        relabels: vec![(1, 1)],
        creates: vec![],
        redges: vec![vec![0, 1]],
    }
}

/// Which variant the run seeds at `invasion_step`.
///
/// `clone` is the **neutral null**: R3 verbatim, no guard, no structural
/// change. It is behaviourally indistinguishable from the resident it joins, so
/// whatever share it reaches is the share an arriving copy reaches for free —
/// from its founding endowment and the population's reproduction dynamics
/// alone. Any real invader has to beat that, not merely beat zero.
pub fn invader_spec_for(cfg: &RunConfig) -> RuleSpec {
    let mut s = match cfg.invader_kind.as_str() {
        "guard" => pareto_invader_spec(cfg.invader_theta),
        "clone" => g2_specs().remove(2),
        _ => invader_spec(),
    };
    // `canonical_key` sorts by `Reverse(priority)` BEFORE `rule_id`, so raising
    // priority moves a rule to the head of the conflict-resolution queue
    // regardless of its class hash. Overriding it changes nothing about what
    // the rule does — same LHS, same guards, same RHS — so it isolates queue
    // position as a cause rather than a correlate.
    if cfg.invader_priority != i32::MIN {
        s.priority = cfg.invader_priority;
    }
    s
}

/// G6.2 — the Pareto construction gate.
///
/// v5's invasion gate specified "same productive effect, fewer match
/// candidates" but only ever checked the second clause. Its invader did **half
/// the productive work** of the resident, so its failure to invade was
/// uninterpretable: a fitness function that prefers the harder worker was
/// arguably right. This gate enforces the premise before the invasion run,
/// and refuses to admit a candidate that fails either clause.
///
/// The gate is scored by **substitution**, not by coresidence. A first
/// implementation added the candidate alongside its own template and compared
/// them in one world; that is circular. The two differ only in a guard, so
/// they contest the same conflict-resolution slots, and the resident's measured
/// rate falls precisely as the variant's rises — the more thoroughly a variant
/// suppresses the resident, the more easily it clears "variant ≥ resident".
/// On that design the resident's own rate swung 8× across candidates and v5's
/// invader scored a 1.08 ratio, contradicting v5's own measurement of 0.51.
///
/// Substitution asks the question the gate means to ask: in the same
/// ecological role, in the same worlds, does the variant do at least as much
/// productive work for strictly fewer candidates? Coresident rates are still
/// reported below, labelled, because the displacement they show is what P0c
/// has to survive.
pub fn gate6_2(
    seeds: &[u64],
    steps: usize,
    thetas: &[f64],
    invasion_step: usize,
    tie: TieBreak,
    coeff: f64,
) -> Option<f64> {
    let tie_name = match tie {
        TieBreak::RuleId => "ruleid",
        TieBreak::Merit => "merit",
        TieBreak::Random => "random",
        TieBreak::Jitter => "jitter",
    };
    println!(
        "G6.2 — Pareto construction gate (churn3, {} seeds x {steps} steps, \
         tie={tie_name} coeff={coeff})",
        seeds.len()
    );
    println!("  admit iff  productive/copy/step >= R3's  AND  candidates/copy/step < R3's,");
    println!(
        "  each measured in the SUBSTITUTION arm: the candidate takes R3's place in the cycle.\n"
    );

    // Control: unmodified churn3. R3's rates in its own population, kept
    // per seed so the variant can be scored PAIRED — the response is bimodal
    // in theta, so a difference in means is not on its own robust.
    let mut ctl: Vec<(f64, f64)> = Vec::new();
    let mut r_surv = 0usize;
    for &seed in seeds {
        let mut cfg = v6_base_cfg(seed, "churn3", steps);
        cfg.tie_break = tie;
        cfg.match_cost_coeff = coeff;
        let res = run_phase('B', &cfg);
        let r = crate::runner::per_copy_rates(&res)
            .get(&2)
            .copied()
            .unwrap_or((0.0, 0.0));
        ctl.push(r);
        if res.survived {
            r_surv += 1;
        }
    }
    let n = seeds.len() as f64;
    let r_prod = ctl.iter().map(|r| r.0).sum::<f64>() / n;
    let r_cand = ctl.iter().map(|r| r.1).sum::<f64>() / n;
    println!(
        "  CONTROL (unmodified churn3): R3 productive/copy/step = {r_prod:.4}, \
         candidates/copy/step = {r_cand:.1}, population survives {r_surv}/{}\n",
        seeds.len()
    );

    println!(
        "{:>22} {:>10} {:>8} {:>7} | {:>11} {:>8} {:>7} | {:>6} | {:>7} | {:>11}",
        "candidate",
        "prod/c/s",
        "vs R3",
        "paired",
        "cand/c/s",
        "vs R3",
        "paired",
        "surv",
        "coresid",
        "verdict"
    );

    let mut admitted: Option<f64> = None;
    let mut candidates: Vec<(String, String, f64)> =
        vec![("v5 (added pattern edge)".into(), "v5".into(), 0.0)];
    for &t in thetas {
        candidates.push((format!("guard energy>{t}"), "guard".into(), t));
    }

    for (label, kind, theta) in candidates {
        let (mut vp, mut vc, mut vs) = (0.0f64, 0.0f64, 0usize);
        let (mut win_prod, mut win_cand) = (0usize, 0usize);
        for (i, &seed) in seeds.iter().enumerate() {
            let mut cfg = v6_base_cfg(seed, "churn3", steps);
            cfg.substitute_member = 2;
            cfg.invader_kind = kind.clone();
            cfg.invader_theta = theta;
            cfg.tie_break = tie;
            cfg.match_cost_coeff = coeff;
            let res = run_phase('B', &cfg);
            let r = crate::runner::per_copy_rates(&res)
                .get(&2)
                .copied()
                .unwrap_or((0.0, 0.0));
            vp += r.0;
            vc += r.1;
            if r.0 >= ctl[i].0 {
                win_prod += 1;
            }
            if r.1 < ctl[i].1 {
                win_cand += 1;
            }
            if res.survived {
                vs += 1;
            }
        }
        let (vp, vc) = (vp / n, vc / n);

        // Secondary, labelled: the coresidence rate ratio, which is what the
        // invasion run will actually experience.
        let (mut cp, mut cr, mut cn) = (0.0f64, 0.0f64, 0.0f64);
        for &seed in seeds {
            let mut cfg = v6_base_cfg(seed, "churn3", steps);
            cfg.invasion_step = invasion_step;
            cfg.invader_kind = kind.clone();
            cfg.invader_theta = theta;
            let res = run_phase('B', &cfg);
            let rates = crate::runner::per_copy_rates(&res);
            if let (Some(r), Some(i)) = (rates.get(&2), rates.get(&INVADER_MEMBER_BASE)) {
                cp += i.0;
                cr += r.0;
                cn += 1.0;
            }
        }
        let cores = if cn > 0.0 && cr > 0.0 {
            cp / cr
        } else {
            f64::NAN
        };

        // Clause 0, added after the first scoring run and logged as a
        // deviation: the substitution arm must SURVIVE. v5's invader scored a
        // 1.11 work ratio while killing the population in 8/8 seeds — its rate
        // was a mean over the ~30 steps before collapse, which is not
        // comparable with a rate measured over a live 2000-step run. This is a
        // tightening of the gate, never a relaxation, and it is a measurement-
        // validity requirement rather than a preference about the answer.
        let c0 = vs == seeds.len();
        let c1 = vp >= r_prod;
        let c2 = vc < r_cand;
        let pass = c0 && c1 && c2;
        println!(
            "{:>22} {:>10.4} {:>8.4} {:>7} | {:>11.1} {:>8.4} {:>7} | {:>6} | {:>7.2} | {:>11}",
            label,
            vp,
            if r_prod > 0.0 { vp / r_prod } else { f64::NAN },
            format!("{win_prod}/{}", seeds.len()),
            vc,
            if r_cand > 0.0 { vc / r_cand } else { f64::NAN },
            format!("{win_cand}/{}", seeds.len()),
            format!("{vs}/{}", seeds.len()),
            cores,
            if !c0 {
                "reject:dies"
            } else if !c1 {
                "reject:work"
            } else if !c2 {
                "reject:cost"
            } else {
                "ADMIT"
            }
        );
        // Admission needs the pre-registered clauses on the means AND paired
        // support on a majority of seeds. Without the paired check a 2% edge on
        // a bimodal distribution would qualify as "Pareto-superior".
        if pass
            && win_prod * 2 > seeds.len()
            && win_cand * 2 > seeds.len()
            && admitted.is_none()
            && kind == "guard"
        {
            admitted = Some(theta);
        }
    }

    match admitted {
        Some(t) => println!(
            "\nG6.2 PASS: `guard energy>{t}` does at least R3's productive work for strictly fewer \
             candidates, in R3's own role. Admitted to G5.3."
        ),
        None => println!(
            "\nG6.2 FAIL: no candidate satisfies both clauses under substitution. Plan §P0c branch \
             (c): the efficiency axis is not expressible in this pattern vocabulary. Report it as a \
             finding about pattern-language expressiveness; do NOT proceed to P3 on the assumption \
             that economics is the issue."
        ),
    }
    admitted
}

/// P0c — G5.3 re-run with a G6.2-admitted invader, paired with the no-invader
/// control that G6.3 makes standing procedure.
///
/// The threshold is v5's, verbatim and unrelaxed: the invader's lineage share
/// must reach **≥ 2× its seed share in ≥ 6 of 8 seeds**. Only the invader's
/// construction differs from v5.
pub fn p0c_invasion(
    seeds: &[u64],
    steps: usize,
    invasion_step: usize,
    kind: &str,
    theta: f64,
    coeff: f64,
    tie: TieBreak,
) -> bool {
    let name = if kind == "guard" {
        format!("guard energy>{theta}")
    } else {
        "v5".into()
    };
    let tie_name = match tie {
        TieBreak::RuleId => "ruleid (v3-v6 canonical)",
        TieBreak::Merit => "merit (v7)",
        TieBreak::Random => "random (v7)",
        TieBreak::Jitter => "jitter (v8 hybrid)",
    };
    println!("P0c / G5.3 — invasion by `{name}`, introduced at step {invasion_step}");
    println!("  threshold (pre-registered in v5, unchanged): final share >= 2x seed share in >= 6/8 seeds");
    println!("  match_cost_coeff = {coeff}   tie_break = {tie_name}\n");
    // Share is a ratio, so it rises either because the invader grew or because
    // the population shrank around it. Absolute counts are reported alongside
    // so the two cannot be confused.
    println!(
        "{:>6} | {:>10} {:>11} {:>7} {:>8} | {:>9} {:>11} {:>11} | {:>8} {:>8}",
        "seed",
        "seed share",
        "final share",
        "ratio",
        "invades",
        "inv n_end",
        "R3 inv arm",
        "R3 ctl arm",
        "pop_end",
        "ctl pop"
    );

    let (mut invades, mut scorable, mut grew, mut displaced) = (0usize, 0usize, 0usize, 0usize);
    for &seed in seeds {
        // Paired no-invader control on identical configuration (G6.3).
        let mut ctlc = v6_base_cfg(seed, "churn3", steps);
        ctlc.match_cost_coeff = coeff;
        ctlc.tie_break = tie;
        let ctl = run_phase('B', &ctlc);
        let ctl_r3 = ctl.copy_number_final.get(2).copied().unwrap_or(0);

        let mut cfg = v6_base_cfg(seed, "churn3", steps);
        cfg.invasion_step = invasion_step;
        cfg.invader_kind = kind.to_string();
        cfg.invader_theta = theta;
        cfg.match_cost_coeff = coeff;
        cfg.tie_break = tie;
        let res = run_phase('B', &cfg);

        let ratio = if res.invasion_share_seed > 0.0 {
            res.invasion_share_final / res.invasion_share_seed
        } else {
            f64::NAN
        };
        let inv = ratio >= 2.0;
        if inv {
            invades += 1;
        }
        // The invader is seeded as exactly one copy. Its final count is the
        // only thing that distinguishes real reproductive success from a
        // share that rose because everything else died.
        let n_end = (res.invasion_share_final * res.final_rules as f64).round() as i64;
        if n_end > 1 {
            grew += 1;
        }
        scorable += 1;
        println!(
            "{:>6} | {:>10.4} {:>11.4} {:>7.2} {:>8} | {:>9} {:>11} {:>11} | {:>8} {:>8}",
            seed,
            res.invasion_share_seed,
            res.invasion_share_final,
            ratio,
            if inv { "YES" } else { "no" },
            n_end,
            res.copy_number_final.get(2).copied().unwrap_or(0),
            ctl_r3,
            res.final_rules,
            ctl.final_rules
        );
        if res.copy_number_final.get(2).copied().unwrap_or(0) < ctl_r3 {
            displaced += 1;
        }
    }

    let pass = invades >= 6;
    println!("\n  invaded in {invades}/{scorable} seeds (threshold: 6/8)");
    println!("  invader lineage actually GREW beyond its founding copy in {grew}/{scorable} seeds");
    println!("  R3 was REDUCED relative to its paired control in {displaced}/{scorable} seeds");
    if pass {
        println!(
            "P0c PASS (plan branch a): G5.3 passes on a properly constructed invader. v5's \
             diagnosis was wrong — the failure was invader construction, not the reproduction \
             criterion. Skip P3; P4 is unblocked."
        );
    } else {
        println!(
            "P0c FAIL (plan branch b): a Pareto-superior variant, verified superior by G6.2 and \
             competing against a resident that survives in the paired control, still cannot \
             invade. v5's diagnosis is CONFIRMED on a properly constructed test. P3 is indicated."
        );
    }
    pass
}

/// v8 — establishment-probability gate (v7 §4.1: recalibrate the null).
///
/// v5–v7's criterion ("final share ≥ 2× seed share in ≥ 6/8 seeds") demands
/// GROWTH from a single copy, but for a true neutral the dominant outcome
/// from n=1 is absorption at zero — v7 showed even a fair coin gives the
/// clone 0/8 on that reading. The calibrated question is comparative:
///
///   p_est(variant) vs p_est(neutral clone), binomially.
///
/// Each trial seeds ONE copy of `kind` at `invasion_step` into an otherwise
/// standard churn3 run and scores ESTABLISHMENT: the invader lineage holds
/// ≥ 2 copies at step 2000 AND the population survived (a dead population
/// establishes nothing). `trials` independent runs per cell, each with its
/// own seed, give a binomial count; the two cells are compared with a
/// one-sided Fisher-style 2×2 tally reported alongside.
///
/// Returns (p_est_variant, p_est_clone).
pub fn v8_establishment(
    trials: usize,
    steps: usize,
    invasion_step: usize,
    theta: f64,
    coeff: f64,
    tie: TieBreak,
    jitter_eps: f64,
    decay: f64,
    base: f64,
) -> (f64, f64) {
    let tie_name = match tie {
        TieBreak::RuleId => "ruleid",
        TieBreak::Merit => "merit",
        TieBreak::Random => "random",
        TieBreak::Jitter => "jitter",
    };
    println!(
        "v8 establishment — p_est(guard θ={theta}) vs p_est(neutral clone), \
         {trials} trials/cell, tie={tie_name} eps={jitter_eps} decay={decay} base={base} coeff={coeff}"
    );
    println!("  establish = invader holds >= 2 copies at end AND population alive\n");

    let cell = |kind: &str| -> (usize, Vec<usize>) {
        let mut est = 0usize;
        let mut copies: Vec<usize> = Vec::with_capacity(trials);
        for t in 0..trials {
            let mut cfg = v6_base_cfg(P0A_SEEDS[t % P0A_SEEDS.len()], "churn3", steps);
            // Trial independence: rotate the run seed across a wide space so
            // each trial sees a different trajectory, not just a different
            // invasion timing of the same one.
            cfg.seed = 1_000 + t as u64 * 7919;
            cfg.invasion_step = invasion_step;
            cfg.invader_kind = kind.to_string();
            cfg.invader_theta = theta;
            cfg.match_cost_coeff = coeff;
            cfg.tie_break = tie;
            cfg.jitter_eps = jitter_eps;
            cfg.merit_decay = decay;
            cfg.merit_base = base;
            let res = run_phase('B', &cfg);
            let n_end = if res.final_rules > 0 {
                (res.invasion_share_final * res.final_rules as f64).round() as usize
            } else {
                0
            };
            let established = res.survived && n_end >= 2;
            if established {
                est += 1;
            }
            copies.push(n_end);
        }
        (est, copies)
    };

    let (est_g, copies_g) = cell("guard");
    let (est_c, copies_c) = cell("clone");
    let p_g = est_g as f64 / trials as f64;
    let p_c = est_c as f64 / trials as f64;

    println!("{:>10} {:>6} {:>22}", "cell", "p_est", "copies per trial");
    println!("{:>10} {:>6.3} {:>22?}", "guard", p_g, copies_g);
    println!("{:>10} {:>6.3} {:>22?}", "clone", p_c, copies_c);
    let p_tail = fisher_one_sided(est_g, est_c, trials);
    println!(
        "\n  guard {est_g}/{trials}, clone {est_c}/{trials}; one-sided Fisher p = {p_tail:.4}"
    );
    if p_g > p_c {
        println!(
            "  v8 READ: guard establishes more often than the neutral null ({p_g:.2} vs {p_c:.2}).{}",
            if p_tail < 0.05 { " Significant at 0.05." } else { " Not significant at 0.05 — need more trials." }
        );
    } else {
        println!("  v8 READ: no fitness signal under this resolver (guard ≤ clone).");
    }
    (p_g, p_c)
}

/// One-sided Fisher exact for two equal-sized binomial cells, guard-dominant
/// direction: P(X >= a) for X ~ Hypergeometric(N=2T, K=a+c, draws=T). Each
/// specific 2x2 with first-row count x has probability
/// C(T, x)·C(T, k−x)/C(2T, k) — both row totals are T.
pub fn fisher_one_sided(est_g: usize, est_c: usize, trials: usize) -> f64 {
    let a = est_g as u64;
    let c = est_c as u64;
    let k = a + c;
    let t = trials as u64;
    let comb = |nn: u64, kk: u64| -> f64 {
        // C(nn, kk) via lgamma.
        (ln_gamma((nn + 1) as f64) - ln_gamma((kk + 1) as f64) - ln_gamma((nn - kk + 1) as f64))
            .exp()
    };
    let mut p_tail = 0.0f64;
    let mut x = a;
    while x <= k.min(t) {
        p_tail += comb(t, x) * comb(t, k - x) / comb(2 * t, k);
        x += 1;
    }
    p_tail
}

/// ln Γ(x) — Lanczos g=7, adequate for the small factorials used here.
fn ln_gamma(x: f64) -> f64 {
    const G: [f64; 9] = [
        0.99999999999980993,
        676.5203681218851,
        -1259.1392167224028,
        771.32342877765313,
        -176.61502916214059,
        12.507343278686905,
        -0.13857109526572012,
        9.9843695780195716e-6,
        1.5056327351493116e-7,
    ];
    if x < 0.5 {
        std::f64::consts::PI / (std::f64::consts::PI * x).sin() / ln_gamma(1.0 - x)
    } else {
        let x = x - 1.0;
        let mut a = G[0];
        let t = x + 7.5;
        for (i, g) in G.iter().enumerate().skip(1) {
            a += g / (x + i as f64);
        }
        0.5 * (2.0 * std::f64::consts::PI).ln() + (x + 0.5) * t.ln() - t + a.ln()
    }
}

/// Where each candidate variant lands in the rule ordering that conflict
/// resolution uses.
///
/// `Binding::canonical_key` is `(Reverse(priority), rule_id, sorted nodes)` and
/// `resolve_conflicts` accepts greedily in that order, so within a priority
/// class **every** binding of a lower-`rule_id` rule is considered before any
/// binding of a higher one: the earlier rule gets first claim on the whole
/// world and later rules see only what it left. `rule_id` is the index from
/// `decode_all`, which orders by `(class_hash, K_SEQ)` — a content hash of the
/// rule's own structure, which has nothing to do with how well the rule works.
///
/// If a variant's rank relative to R3 predicts its measured productivity better
/// than its guard selectivity does, then "efficiency" is not what is being
/// selected on.
pub fn v6_rule_order_probe(thetas: &[f64]) {
    println!("Rule-order probe — position in the conflict-resolution queue\n");
    let base = g2_specs();
    let r3 = base[2].clone();
    let r3h = r3.class_hash();
    println!("  churn3 residents (priority, class_hash):");
    for (i, s) in base.iter().enumerate() {
        println!(
            "    R{}  prio {:>2}  hash {:>20}",
            i + 1,
            s.priority,
            s.class_hash()
        );
    }
    println!(
        "\n  {:>22} {:>20} {:>14} {:>26}",
        "candidate", "class_hash", "vs R3", "first claim within prio 0"
    );
    let mut rows: Vec<(String, u64)> =
        vec![("v5 (pattern edge)".into(), invader_spec().class_hash())];
    for &t in thetas {
        rows.push((
            format!("guard energy>{t}"),
            pareto_invader_spec(t).class_hash(),
        ));
    }
    rows.push(("clone of R3".into(), r3h));
    for (label, h) in rows {
        let before = h < r3h;
        println!(
            "  {label:>22} {h:>20} {:>14} {:>26}",
            if h == r3h {
                "same".to_string()
            } else if before {
                "BEFORE".into()
            } else {
                "after".into()
            },
            if h == r3h {
                "ties, broken by K_SEQ"
            } else if before {
                "variant first"
            } else {
                "R3 first"
            }
        );
    }
    println!(
        "\n  R3's own hash: {r3h}. Priority is 0 for R1 and R3, 1 for R2, so R2 is always \
         considered first and R1/R3 contend by hash."
    );
}

/// G5.1 — no-gradient confirmation. Under v4's flat economy the per-step
/// variance of `net_income` across productive rules must be exactly 0.
pub fn gate5_1(seeds: &[u64], steps: usize) -> bool {
    println!("G5.1 — no-gradient confirmation (v4 flat economy, churn3):");
    let mut ok = true;
    for &seed in seeds {
        let mut cfg = RunConfig::default();
        cfg.economy = EconomyMode::Flux;
        cfg.seed_pop = "churn3".into();
        cfg.seed = seed;
        cfg.steps = steps;
        cfg.log_every = 50;
        cfg.mu = 0.0;
        cfg.abio = 0.0;
        cfg.p_dup = 0.0;
        cfg.p_del = 0.0;
        let res = run_phase('B', &cfg);
        let bad = res.net_income_var_max > 0.0;
        if bad {
            ok = false;
        }
        println!(
            "  seed {seed:>3}: mean net_income_var = {:.3e}, max = {:.3e}{}",
            res.net_income_var_mean,
            res.net_income_var_max,
            if bad { "   <-- NONZERO" } else { "" }
        );
    }
    if ok {
        println!(
            "G5.1 PASS: net_income variance is exactly 0 across productive rules on every seed \
             — fitness under v4's economy is binary, as the v5 reframe claims"
        );
    } else {
        println!(
            "G5.1 FAIL: variance is nonzero, so v4's fitness was NOT binary — the D-gradient \
             diagnosis is wrong at its root. Stop and diagnose before building the gradient."
        );
    }
    ok
}

/// G5.2 — the income pool must actually bind under proportional income.
pub fn gate5_2(seeds: &[u64], steps: usize, income_pool: i64, coeff: f64) -> bool {
    println!("G5.2 — pool-binds check (proportional income, pool={income_pool}, coeff={coeff}):");
    let mut ok = true;
    for &seed in seeds {
        let mut cfg = RunConfig::default();
        cfg.economy = EconomyMode::Flux;
        cfg.seed_pop = "churn3".into();
        cfg.seed = seed;
        cfg.steps = steps;
        cfg.log_every = 25;
        cfg.mu = 0.0;
        cfg.abio = 0.0;
        cfg.p_dup = 0.0;
        cfg.p_del = 0.0;
        cfg.income_mode = IncomeMode::Proportional;
        cfg.income_pool = income_pool;
        cfg.match_cost_coeff = coeff;
        let res = run_phase('B', &cfg);
        // Demand = productive firings x nominal reward; supply = pool.
        // If the pool binds, per-rule income is rationed and net_income
        // variance is nonzero (rules differ in productive share).
        let binds = res.net_income_var_mean > 0.0;
        if !binds {
            ok = false;
        }
        println!(
            "  seed {seed:>3}: mean net_income_var = {:.3e}, gini = {:.4}, survived={}{}",
            res.net_income_var_mean,
            res.income_gini_mean,
            res.survived,
            if binds { "" } else { "   <-- NO GRADIENT" }
        );
    }
    println!(
        "G5.2 {}",
        if ok {
            "PASS: income is rationed and differentiated — a gradient exists"
        } else {
            "FAIL: income undifferentiated; recalibrate income_pool / match_cost_coeff"
        }
    );
    ok
}

// ================= v6 =================

/// The v5 operating point (calibration doc §"Income and cost calibration"),
/// which every v6 selection experiment inherits. Isolated here so P0a, P0b,
/// P1, G6.2 and P0c cannot drift apart.
pub fn v6_base_cfg(seed: u64, pop: &str, steps: usize) -> RunConfig {
    let mut cfg = RunConfig::default();
    cfg.economy = EconomyMode::Flux;
    cfg.seed_pop = pop.into();
    cfg.seed = seed;
    cfg.steps = steps;
    cfg.log_every = 100;
    cfg.mu = 0.0;
    cfg.abio = 0.0;
    cfg.p_dup = 0.0;
    cfg.p_del = 0.0;
    cfg.income_mode = IncomeMode::Proportional;
    cfg.income_pool = 12;
    cfg.match_cost_coeff = 0.0005;
    cfg
}

fn member_name(m: i64) -> String {
    if m >= INVADER_MEMBER_BASE {
        "INVADER".to_string()
    } else if m < 0 {
        "?".to_string()
    } else {
        format!("R{}", m + 1)
    }
}

// ---------- G6.1: candidate fate accounting sums ----------

/// The five-way breakdown must sum exactly to `candidates_generated`, per rule
/// per step. `object_step` asserts this inline on every step of every run, so
/// this gate's job is to *exercise* the accounting across the configurations
/// v6 actually uses — including both truncation modes and the unlimited cap —
/// and to confirm the run-level totals close too.
pub fn gate6_1(seeds: &[u64], steps: usize) -> bool {
    println!("G6.1 — candidate fate accounting sums:");
    let arms: [(usize, TruncationMode, &str); 4] = [
        (8, TruncationMode::Canonical, "cap 8 canonical"),
        (64, TruncationMode::Canonical, "cap 64 canonical"),
        (64, TruncationMode::Random, "cap 64 random"),
        (0, TruncationMode::Canonical, "cap unlimited"),
    ];
    let mut ok = true;
    for (cap, mode, label) in arms {
        for &seed in seeds {
            let mut cfg = v6_base_cfg(seed, "churn3", steps);
            cfg.match_cap_per_rule = cap;
            cfg.truncation_mode = mode;
            let res = run_phase('B', &cfg);
            let f = res.fate;
            if !f.balances() {
                println!(
                    "  FAIL [{label}] seed {seed}: {} != {}+{}+{}+{}",
                    f.generated, f.truncated, f.rejected, f.sterile, f.productive
                );
                ok = false;
            }
            // Cross-check the aggregate against the independently maintained
            // per-member totals: a rule dropping out of `rule_roots` mid-step
            // would silently lose its fate otherwise.
            let mut per_member = crate::runner::FateCounts::default();
            for v in res.fate_per_member.values() {
                per_member.add(v);
            }
            if per_member != f {
                println!("  FAIL [{label}] seed {seed}: per-member total {per_member:?} != run total {f:?}");
                ok = false;
            }
            if seed == seeds[0] {
                let fr = f.fractions();
                println!(
                    "  {label:<18} seed {seed:>3}: generated={:>10}  trunc={:.4} conflict={:.4} \
                     sterile={:.4} productive={:.6}  accepted/step={:.2}",
                    f.generated, fr[0], fr[1], fr[2], fr[3], res.accepted_per_step
                );
            }
        }
    }
    println!(
        "G6.1 {}",
        if ok {
            "PASS: the five-way breakdown closes exactly, every rule every step, across all four \
             cap/ordering arms"
        } else {
            "FAIL: fate accounting does not close — every v6 efficiency number is suspect"
        }
    );
    ok
}

// ---------- P0a: basin characterisation ----------

/// The 24 world seeds P0a screens (calibration doc §P0a). The eight v4/v5
/// seeds lead, so every prior result stays locatable inside the new set.
pub const P0A_SEEDS: [u64; 24] = [
    1, 2, 3, 7, 42, 99, 123, 777, 4, 5, 6, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
];

/// Structural properties of a freshly seeded world, recorded so that
/// instability can be regressed on something rather than merely observed.
fn world_props(seed: u64, cfg: &RunConfig) -> (Vec<usize>, f64) {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut w = HypergraphStore::new();
    seed_world(
        &mut w,
        &mut rng,
        cfg.world_nodes,
        cfg.world_edges,
        cfg.n_labels,
    );
    let mut counts = vec![0usize; cfg.n_labels as usize];
    for (_, d) in w.nodes() {
        if (d.label.0 as usize) < counts.len() {
            counts[d.label.0 as usize] += 1;
        }
    }
    (counts, w.edge_count() as f64 / w.node_count().max(1) as f64)
}

/// P0a — which world seeds support the seed rulesets without perturbation.
///
/// Returns the baseline-stable seed list for `churn3`, which is the operative
/// list: every downstream v6 invasion experiment runs on `churn3`, so that is
/// what has to be quiet at baseline. `churnK` is measured on the same seeds and
/// reported for contrast (it is E3's k = 1 arm) but does not gate the list.
pub fn p0a_basin(seeds: &[u64], steps: usize) -> Vec<u64> {
    println!(
        "P0a — basin characterisation ({} seeds x {steps} steps, mu=0, no perturbation)",
        seeds.len()
    );
    println!("  baseline-stable := survives AND time_to_freeze = none  [pre-registered]\n");
    println!(
        "{:>5} {:>22} {:>7} {:>10} {:>10} {:>10} {:>9} | {:>10} {:>10}",
        "seed",
        "labels @0 (0/1/2/3)",
        "e/n",
        "c3_surv",
        "c3_freeze",
        "c3_conc",
        "c3_supp",
        "cK_surv",
        "cK_freeze"
    );

    let mut stable: Vec<u64> = Vec::new();
    let mut rows: Vec<(
        u64,
        bool,
        Option<usize>,
        f64,
        f64,
        bool,
        Option<usize>,
        Vec<usize>,
        f64,
    )> = Vec::new();
    for &seed in seeds {
        let c3 = run_phase('B', &v6_base_cfg(seed, "churn3", steps));
        let ck = run_phase('B', &v6_base_cfg(seed, "churnk", steps));
        let (labels, dens) = world_props(seed, &RunConfig::default());
        let is_stable = c3.survived && c3.time_to_freeze.is_none();
        if is_stable {
            stable.push(seed);
        }
        println!(
            "{:>5} {:>22} {:>7.3} {:>10} {:>10} {:>10.4} {:>9.4} | {:>10} {:>10}",
            seed,
            format!("{}/{}/{}/{}", labels[0], labels[1], labels[2], labels[3]),
            dens,
            if c3.survived { "yes" } else { "NO" },
            c3.time_to_freeze
                .map(|v| v.to_string())
                .unwrap_or_else(|| "none".into()),
            c3.candidate_concentration_mean,
            c3.node_support_frac_mean,
            if ck.survived { "yes" } else { "NO" },
            ck.time_to_freeze
                .map(|v| v.to_string())
                .unwrap_or_else(|| "none".into()),
        );
        rows.push((
            seed,
            c3.survived,
            c3.time_to_freeze,
            c3.candidate_concentration_mean,
            c3.node_support_frac_mean,
            ck.survived,
            ck.time_to_freeze,
            labels,
            dens,
        ));
    }

    let c3_surv = rows.iter().filter(|r| r.1).count();
    let c3_froze = rows.iter().filter(|r| r.2.is_some()).count();
    let ck_surv = rows.iter().filter(|r| r.5).count();
    let ck_froze = rows.iter().filter(|r| r.6.is_some()).count();
    println!(
        "\n  churn3: survived {c3_surv}/{}, froze {c3_froze}/{}  |  churnK: survived {ck_surv}/{}, froze {ck_froze}/{}",
        rows.len(), rows.len(), rows.len(), rows.len()
    );

    // Do the recorded world properties separate stable from unstable at all?
    let mean = |v: &[f64]| {
        if v.is_empty() {
            f64::NAN
        } else {
            v.iter().sum::<f64>() / v.len() as f64
        }
    };
    for (tag, want) in [("stable  ", true), ("unstable", false)] {
        let sel: Vec<_> = rows
            .iter()
            .filter(|r| (r.1 && r.2.is_none()) == want)
            .collect();
        if sel.is_empty() {
            continue;
        }
        println!(
            "  {tag} (n={:>2}): mean e/n = {:.3}, mean label-0 count = {:.1}, mean label-3 count = {:.1}, \
             mean concentration = {:.4}, mean node support = {:.4}",
            sel.len(),
            mean(&sel.iter().map(|r| r.8).collect::<Vec<_>>()),
            mean(&sel.iter().map(|r| r.7[0] as f64).collect::<Vec<_>>()),
            mean(&sel.iter().map(|r| r.7[3] as f64).collect::<Vec<_>>()),
            mean(&sel.iter().map(|r| r.3).collect::<Vec<_>>()),
            mean(&sel.iter().map(|r| r.4).collect::<Vec<_>>()),
        );
    }

    println!(
        "\n  BASELINE-STABLE SEEDS (churn3, n={}): {stable:?}",
        stable.len()
    );
    if stable.len() >= 8 {
        println!(
            "P0a PASS: {} stable seeds available; downstream experiments use the first 8, so a \
             death there is attributable to the manipulation.",
            stable.len()
        );
    } else {
        println!(
            "P0a FAIL: only {} stable seeds out of {} — the sustaining sets are far more fragile \
             than G2/G4.4 implied. The basin becomes the object of study; P0c cannot be run clean.",
            stable.len(),
            rows.len()
        );
    }
    stable
}

// ---------- P1: contention regime sweep ----------

/// P1 — is `match_cap_per_rule` load-bearing, and is canonical-order
/// truncation load-bearing?
///
/// The cap was introduced in v3 as a performance excursion and never swept, so
/// every result in v3–v5 is conditioned on it (ground rule 7). Two things are
/// being asked. First, whether the cap changes the dynamics qualitatively — if
/// it does, the substrate studied since v3 is a heavily aliased sample of the
/// true match space. Second, whether *canonical* ordering specifically matters,
/// by holding the cap fixed at 64 and replacing the canonically-first prefix
/// with a uniform subsample of identical size.
/// P1a — the cap's effect on the **rewriting dynamics alone**: no economy, no
/// reproduction, unlimited creation budgets, exactly G2's sustaining criterion.
///
/// This separates two claims that the full sweep would otherwise confound. If
/// the cap only changed how tokens flow, it would be an economic parameter. If
/// it changes whether the cycle rewrites at all, it is part of the substrate.
pub fn p1_object_only(seeds: &[u64], steps: usize) {
    println!("P1a — sustaining vs match cap, object dynamics only (no economy, churn3)");
    println!("  criterion is G2's: a window with zero productive applications is a dead window.\n");
    println!(
        "{:>10} {:>10} {:>9} {:>14} {:>14} {:>12} {:>10}",
        "cap", "order", "sustains", "dead windows", "prod/step", "first dead", "nodes"
    );
    let caps: [usize; 5] = [8, 64, 256, 1024, 0];
    let arms: Vec<(usize, TruncationMode)> = caps
        .iter()
        .map(|&c| (c, TruncationMode::Canonical))
        .chain(std::iter::once((64, TruncationMode::Random)))
        .collect();

    for (cap, mode) in arms {
        let (mut sustained, mut dead_tot, mut prod_tot, mut first_tot, mut nodes_tot) =
            (0usize, 0usize, 0f64, 0f64, 0f64);
        for &seed in seeds {
            let mut cfg = RunConfig::default();
            cfg.match_cap_per_rule = cap;
            cfg.truncation_mode = mode;
            let caps_s = cfg.caps();
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            let mut world = HypergraphStore::new();
            seed_world(
                &mut world,
                &mut rng,
                cfg.world_nodes,
                cfg.world_edges,
                cfg.n_labels,
            );
            let mut rs = HypergraphStore::new();
            let mut seq = 0i64;
            for spec in g2_specs() {
                install_rule(&mut rs, &spec, &mut seq);
            }
            let (rules, _) = decode_all(&rs, &caps_s);
            let budgets = unlimited_budgets(&rules);

            let window = 100usize;
            let (mut wprod, mut dead, mut nwin, mut total) = (0usize, 0usize, 0usize, 0usize);
            let mut first_dead: Option<usize> = None;
            for step in 0..steps {
                let oo = object_step(&mut world, &rules, &mut rng, &cfg, &budgets, &[]);
                wprod += oo.productive_applied;
                total += oo.productive_applied;
                if (step + 1) % window == 0 {
                    if wprod == 0 {
                        dead += 1;
                        first_dead.get_or_insert(step + 1 - window);
                    }
                    wprod = 0;
                    nwin += 1;
                }
            }
            if dead == 0 {
                sustained += 1;
            }
            dead_tot += dead;
            prod_tot += total as f64 / steps as f64;
            first_tot += first_dead.unwrap_or(steps) as f64;
            nodes_tot += world.node_count() as f64;
            let _ = nwin;
        }
        let n = seeds.len() as f64;
        println!(
            "{:>10} {:>10} {:>9} {:>14.1} {:>14.2} {:>12.0} {:>10.0}",
            if cap == 0 {
                "unlimited".to_string()
            } else {
                cap.to_string()
            },
            format!("{mode:?}").to_lowercase(),
            format!("{sustained}/{}", seeds.len()),
            dead_tot as f64 / n,
            prod_tot / n,
            first_tot / n,
            nodes_tot / n
        );
    }
    println!();
}

pub fn p1_contention(seeds: &[u64], steps: usize) {
    println!(
        "P1 — contention regime sweep (churn3, {} seeds x {steps} steps)",
        seeds.len()
    );
    println!(
        "  ALL prior selfmod results were produced in the `64 canonical` row (ground rule 7).\n"
    );

    let caps: [usize; 5] = [8, 64, 256, 1024, 0]; // 0 = unlimited
    let arms: Vec<(usize, TruncationMode)> = caps
        .iter()
        .map(|&c| (c, TruncationMode::Canonical))
        .chain(std::iter::once((64, TruncationMode::Random)))
        .collect();

    for &mu in &[0.0f64, 0.05] {
        println!("--- mu = {mu} ---");
        println!(
            "{:>10} {:>10} {:>7} {:>8} {:>13} {:>9} {:>9} {:>9} {:>10} {:>10} {:>9}",
            "cap",
            "order",
            "surv",
            "froze",
            "generated/st",
            "trunc",
            "conflict",
            "prod",
            "accept/st",
            "support",
            "conc8"
        );
        for (cap, mode) in &arms {
            let (mut surv, mut froze) = (0usize, 0usize);
            let (mut gen, mut acc, mut sup, mut conc) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
            let mut fr = [0.0f64; 4];
            for &seed in seeds {
                let mut cfg = v6_base_cfg(seed, "churn3", steps);
                cfg.mu = mu;
                cfg.match_cap_per_rule = *cap;
                cfg.truncation_mode = *mode;
                let r = run_phase('B', &cfg);
                if r.survived {
                    surv += 1;
                }
                if r.time_to_freeze.is_some() {
                    froze += 1;
                }
                gen += r.fate.generated as f64 / steps as f64;
                acc += r.accepted_per_step;
                sup += r.node_support_frac_mean;
                conc += r.candidate_concentration_mean;
                let f = r.fate.fractions();
                for i in 0..4 {
                    fr[i] += f[i];
                }
            }
            let n = seeds.len() as f64;
            println!(
                "{:>10} {:>10} {:>7} {:>8} {:>13.1} {:>9.4} {:>9.4} {:>9.6} {:>10.2} {:>10.4} {:>9.4}",
                if *cap == 0 { "unlimited".to_string() } else { cap.to_string() },
                format!("{mode:?}").to_lowercase(),
                format!("{surv}/{}", seeds.len()),
                format!("{froze}/{}", seeds.len()),
                gen / n,
                fr[0] / n,
                fr[1] / n,
                fr[3] / n,
                acc / n,
                sup / n,
                conc / n
            );
        }
        println!();
    }
    println!(
        "  Read: if the rows differ materially in survival, freeze rate or candidate volume, the \
         cap is load-bearing (branch a) and prior results are conditioned on an artifact of it. \
         If the two `64` rows differ, canonical ORDERING is separately load-bearing (branch c)."
    );
}

// ---------- P0b: which fate dominates ----------

/// Per-member candidate fate on a set of worlds. This is the measurement that
/// decides whether a Pareto invader is constructible at all (plan §P0b): a
/// guard can only exclude candidates that were going to be unproductive, and
/// *how* they were going to be unproductive determines whether excluding them
/// preserves productive work or merely reshuffles the draw.
pub fn p0b_fate_report(seeds: &[u64], steps: usize, pop: &str, cap: usize, mode: TruncationMode) {
    println!(
        "P0b — candidate fate breakdown ({pop}, {steps} steps, {} seeds)",
        seeds.len()
    );
    println!(
        "  cap={} truncation={mode:?}\n",
        if cap == 0 {
            "unlimited".into()
        } else {
            cap.to_string()
        }
    );
    println!(
        "{:>8} {:>14} {:>10} {:>10} {:>10} {:>12} {:>12}",
        "member", "generated/c/s", "trunc", "conflict", "sterile", "productive", "prod/c/s"
    );

    let mut agg: HashMap<i64, (crate::runner::FateCounts, i64)> = HashMap::new();
    for &seed in seeds {
        let mut cfg = v6_base_cfg(seed, pop, steps);
        cfg.match_cap_per_rule = cap;
        cfg.truncation_mode = mode;
        let res = run_phase('B', &cfg);
        for (m, f) in &res.fate_per_member {
            let e = agg
                .entry(*m)
                .or_insert((crate::runner::FateCounts::default(), 0));
            e.0.add(f);
            e.1 += res.copy_steps_per_member.get(m).copied().unwrap_or(0);
        }
    }

    let mut ms: Vec<i64> = agg.keys().copied().collect();
    ms.sort();
    let mut tot = crate::runner::FateCounts::default();
    for m in &ms {
        let (f, cs) = &agg[m];
        tot.add(f);
        let fr = f.fractions();
        let per = |x: i64| if *cs > 0 { x as f64 / *cs as f64 } else { 0.0 };
        println!(
            "{:>8} {:>14.1} {:>10.4} {:>10.4} {:>10.4} {:>12.6} {:>12.3}",
            member_name(*m),
            per(f.generated),
            fr[0],
            fr[1],
            fr[2],
            fr[3],
            per(f.productive)
        );
    }
    let fr = tot.fractions();
    println!(
        "\n  ALL: generated={} — truncated {:.2}%, lost to conflict {:.2}%, applied sterile {:.4}%, \
         applied productive {:.4}%",
        tot.generated, fr[0] * 100.0, fr[1] * 100.0, fr[2] * 100.0, fr[3] * 100.0
    );
    let dominant = if fr[0] >= fr[1] && fr[0] >= fr[2] {
        "TRUNCATION"
    } else if fr[1] >= fr[2] {
        "CONFLICT LOSS"
    } else {
        "STERILE APPLICATION"
    };
    println!("  Dominant non-productive fate: {dominant}");
    match dominant {
        "STERILE APPLICATION" => println!(
            "  => a guard excluding sterile matches preserves productive work exactly; \
             Pareto construction is straightforward."
        ),
        _ => println!(
            "  => the waste is upstream of application, so a tighter guard changes WHICH bindings \
             win the draw. Preserving productive count requires the guard to spare the region that \
             actually wins — constructible, but it has to be verified, not assumed (G6.2)."
        ),
    }
}

/// G5.3 diagnosis: per-member economics with the invader present.
/// Reports, for each lineage, the match candidates it generates, the
/// productive applications it wins, and the income/cost/net it realises —
/// the four numbers needed to tell "the invader was not actually superior"
/// from "the invader was superior and selection could not see it".
pub fn gate5_3_diagnose(seed: u64, steps: usize, invasion_step: usize) {
    use crate::economy::*;
    use crate::runner::{install_population, seed_population, seed_world};
    use std::collections::HashMap;

    let mut cfg = RunConfig::default();
    cfg.economy = EconomyMode::Flux;
    cfg.seed_pop = "churn3".into();
    cfg.seed = seed;
    cfg.mu = 0.0;
    cfg.income_mode = IncomeMode::Proportional;
    cfg.income_pool = 12;
    cfg.match_cost_coeff = 0.0005;
    let caps = cfg.caps();
    let pr = cfg.economy_params();

    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut world = HypergraphStore::new();
    let mut rs = HypergraphStore::new();
    let mut seq = 0i64;
    seed_world(
        &mut world,
        &mut rng,
        cfg.world_nodes,
        cfg.world_edges,
        cfg.n_labels,
    );
    let specs = seed_population(&cfg.seed_pop);
    install_population(&mut rs, &specs, &cfg, &mut seq);
    let mut led = Ledger::new(cfg.free_cap, total_bound(&rs));

    // member -> (candidates, productive, income, cost, tokens, copies)
    let mut acc: HashMap<i64, [f64; 6]> = HashMap::new();
    let mut window = 0usize;

    println!("G5.3 diagnosis — seed {seed}, invader introduced at step {invasion_step}");
    println!("  per-member per-step averages over each 200-step window\n");
    println!(
        "{:>6} {:>7} {:>11} {:>11} {:>9} {:>8} {:>8} {:>7}",
        "step", "member", "candidates", "productive", "income", "cost", "net", "copies"
    );

    for step in 0..steps {
        if step == invasion_step {
            let inv = invader_spec();
            let root = install_rule(&mut rs, &inv, &mut seq);
            set_tokens(&mut rs, root, cfg.e0);
            set_int(&mut rs, root, K_CREATION_BUDGET, cfg.creation_budget_init);
            set_int(&mut rs, root, K_DEPTH, 0);
            set_int(&mut rs, root, K_MEMBER, INVADER_MEMBER_BASE);
            set_int(&mut rs, root, K_LAST_FISSION, i64::MIN / 2);
            set_int(&mut rs, root, K_LAST_CANDIDATES, 0);
            led.cumulative_influx += cfg.e0;
        }
        led.influx(&pr);
        let mut costs: HashMap<NodeId, i64> = HashMap::new();
        rent_and_starve(&mut rs, &mut led, &pr, &mut costs);
        let (rules, _) = decode_all(&rs, &caps);
        let budgets: HashMap<NodeId, i64> = rule_roots(&rs)
            .into_iter()
            .map(|r| (r, get_int(&rs, r, K_CREATION_BUDGET).unwrap_or(0)))
            .collect();
        let oo = object_step(&mut world, &rules, &mut rng, &cfg, &budgets, &[]);
        for root in rule_roots(&rs) {
            let c = oo.candidates_per_rule.get(&root).copied().unwrap_or(0);
            set_int(&mut rs, root, K_LAST_CANDIDATES, c);
        }
        let mut income: HashMap<NodeId, i64> = HashMap::new();
        distribute_income(&mut rs, &oo.productive_per_rule, &mut led, &pr, &mut income);

        for root in rule_roots(&rs) {
            let m = get_int(&rs, root, K_MEMBER).unwrap_or(-1);
            let e = acc.entry(m).or_insert([0.0; 6]);
            e[0] += oo.candidates_per_rule.get(&root).copied().unwrap_or(0) as f64;
            e[1] += oo.productive_per_rule.get(&root).copied().unwrap_or(0) as f64;
            e[2] += income.get(&root).copied().unwrap_or(0) as f64;
            e[3] += costs.get(&root).copied().unwrap_or(0) as f64;
            e[4] = rule_tokens(&rs, root) as f64;
            e[5] += 1.0;
        }
        funded_fission(&mut rs, &mut rng, &pr, &caps, &mut seq, step as i64);
        refill_creation_budgets(&mut rs, &pr);
        window += 1;

        if window == 200 {
            let mut ms: Vec<i64> = acc.keys().copied().collect();
            ms.sort();
            for m in ms {
                let e = acc[&m];
                let copies = e[5] / 200.0;
                let per = |x: f64| {
                    if copies > 0.0 {
                        x / 200.0 / copies
                    } else {
                        0.0
                    }
                };
                println!(
                    "{:>6} {:>7} {:>11.1} {:>11.2} {:>9.2} {:>8.2} {:>8.2} {:>7.2}",
                    step + 1,
                    if m >= INVADER_MEMBER_BASE {
                        "INVADER".to_string()
                    } else {
                        format!("R{}", m + 1)
                    },
                    per(e[0]),
                    per(e[1]),
                    per(e[2]),
                    per(e[3]),
                    per(e[2]) - per(e[3]),
                    copies
                );
            }
            println!();
            acc.clear();
            window = 0;
        }
        if rule_roots(&rs).is_empty() {
            println!("  (population extinct at step {step})");
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Known values for the corrected one-sided Fisher tail. The original
    /// buggy version (wrong row totals) printed 0.0000 and 0.0208 here.
    #[test]
    fn test_fisher_known_values() {
        // 7/24 vs 2/24: normal approx puts this ~1.8 sigma out; exact ~0.07.
        let p = fisher_one_sided(7, 2, 24);
        assert!(
            (0.04..=0.12).contains(&p),
            "7/24 vs 2/24 should be borderline (~0.07), got {p}"
        );
        // A single success vs zero is exactly a coin-flip's worth of evidence.
        let p = fisher_one_sided(1, 0, 24);
        assert!(
            (p - 0.5).abs() < 1e-9,
            "1/24 vs 0/24 must be exactly 0.5, got {p}"
        );
    }

    /// Degenerate and symmetric cases.
    #[test]
    fn test_fisher_degenerate_cases() {
        // a = 0: the guard-dominant tail is the whole space.
        assert!((fisher_one_sided(0, 3, 24) - 1.0).abs() < 1e-9);
        // Symmetric 3-vs-3: tail includes the central mass; exact 0.6669...
        let p = fisher_one_sided(3, 3, 24);
        assert!(
            (0.66..=0.67).contains(&p),
            "3/24 vs 3/24 tail should be ~0.667, got {p}"
        );
        // Guard wins every trial, clone none: p = C(T,T)C(T,0)/C(2T,T) = 1/24!.
        let p = fisher_one_sided(24, 0, 24);
        let expected: f64 = 1.0 / (1..=24u64).map(|i| i as f64).product::<f64>();
        assert!(
            (p - expected).abs() < 1e-9,
            "all-vs-none should be 1/24!, got {p}"
        );
        // Zero trials edge: no evidence either way.
        assert!((fisher_one_sided(0, 0, 0) - 1.0).abs() < 1e-9);
    }

    /// ln_gamma against exact small factorials: ln(n!) = ln_gamma(n+1).
    #[test]
    fn test_ln_gamma_factorials() {
        let mut fact = 1.0f64;
        for n in 1..=10u64 {
            fact *= n as f64;
            let got = ln_gamma(n as f64 + 1.0);
            assert!(
                (got - fact.ln()).abs() < 1e-9,
                "ln({n}!) mismatch: {got} vs {}",
                fact.ln()
            );
        }
    }
}
