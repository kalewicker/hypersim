//! **E2.2 — bare characterisation, the firewall gate.**
//!
//! Per the D2.3 template. The gate is not a formality: it has killed two arms
//! already (`cyclic3` at 59 firings, `rewire` at 560).
//!
//! **The degree RANGE over time is the quantity to watch**, and E2.3 named the
//! failure mode in advance: self-balancing has its own absorbing state,
//! **degree uniformity**. If the rule drives every node to equal degree, no
//! pair satisfies `degree(c) > degree(a)` and it quiesces — arm E's failure
//! with the sign reversed. There is also a subtler pre-registered outcome: the
//! rule may stay live while contracting to `{4, 5}`, where every firing swaps a
//! 4 and a 5 and therefore **preserves the degree sequence**, collapsing the
//! treatment onto its own control.
//!
//! ```text
//! cargo test --release -p hg-alphabet --test e2_2_bare_characterisation -- --ignored --nocapture
//! ```

use hg_core::{HypergraphStore, NodeId};
use hg_engine::Engine;
use hg_observe::{Observer, RewriteEvent, StepStats};
use hg_rules::{compile_rules, parse_rules_toml, StepMode};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use hg_alphabet::dtrack;
use hg_alphabet::etrack;
use hg_alphabet::run::Init;

struct Counter {
    by_rule: Arc<Mutex<BTreeMap<String, u64>>>,
    late: Arc<Mutex<BTreeMap<String, u64>>>,
    step: Arc<Mutex<u64>>,
    late_from: u64,
}

impl Observer for Counter {
    fn on_step_end(&mut self, _s: &hg_core::GraphSnapshot<'_>, _st: &StepStats, events: &[RewriteEvent]) {
        let step = *self.step.lock().unwrap();
        {
            let mut m = self.by_rule.lock().unwrap();
            let mut l = self.late.lock().unwrap();
            for ev in events {
                *m.entry(ev.rule_name.clone()).or_insert(0) += 1;
                if step >= self.late_from {
                    *l.entry(ev.rule_name.clone()).or_insert(0) += 1;
                }
            }
        }
        *self.step.lock().unwrap() = step + 1;
    }
}

fn degree_sequence(store: &HypergraphStore) -> Vec<usize> {
    let mut d: Vec<usize> = store
        .nodes()
        .map(|(id, _)| store.edges_incident_to(id).len())
        .collect();
    d.sort_unstable();
    d
}

fn components(store: &HypergraphStore) -> usize {
    let ids: Vec<NodeId> = store.nodes().map(|(id, _)| id).collect();
    let index: std::collections::HashMap<NodeId, usize> =
        ids.iter().enumerate().map(|(i, &n)| (n, i)).collect();
    let mut parent: Vec<usize> = (0..ids.len()).collect();
    fn find(p: &mut [usize], mut x: usize) -> usize {
        while p[x] != x {
            p[x] = p[p[x]];
            x = p[x];
        }
        x
    }
    for (_, edge) in store.edges() {
        let mut it = edge.nodes.iter().filter_map(|n| index.get(n).copied());
        if let Some(first) = it.next() {
            for other in it {
                let (a, b) = (find(&mut parent, first), find(&mut parent, other));
                if a != b {
                    parent[a] = b;
                }
            }
        }
    }
    let mut roots: Vec<usize> = (0..ids.len()).map(|i| find(&mut parent, i)).collect();
    roots.sort_unstable();
    roots.dedup();
    roots.len()
}

struct Char {
    events: u64,
    late: u64,
    n_flat: bool,
    e_flat: bool,
    degree_preserved: bool,
    /// Steps on which the degree sequence differed from the PREVIOUS step. The
    /// E2.3 subtle outcome — live but asymptotically sequence-preserving — is
    /// invisible to a whole-run "did it ever change" bit, and this is what
    /// makes it visible.
    seq_changed_steps: u64,
    /// …restricted to the final tenth. If this is 0 while `events` is large,
    /// the rule is still firing but no longer changing the degree sequence.
    seq_changed_late: u64,
    range_end: (usize, usize),
}

fn characterise(label: &str, toml: &str, init: &Init, steps: u64) -> Char {
    let mut store = init.build();
    let (n0, e0) = (store.node_count(), store.edge_count());
    let deg0 = degree_sequence(&store);
    let late_from = steps - steps / 10;

    let rules = compile_rules(&parse_rules_toml(toml).expect("parse")).expect("compile");
    let mut engine = Engine::new(rules, StepMode::MaxParallel, 1, 1_000_000);
    let by_rule = Arc::new(Mutex::new(BTreeMap::new()));
    let late = Arc::new(Mutex::new(BTreeMap::new()));
    engine.add_observer(Box::new(Counter {
        by_rule: Arc::clone(&by_rule),
        late: Arc::clone(&late),
        step: Arc::new(Mutex::new(0)),
        late_from,
    }));

    let (mut n_flat, mut e_flat, mut degree_preserved) = (true, true, true);
    let (mut seq_changed_steps, mut seq_changed_late) = (0u64, 0u64);
    let mut prev = deg0.clone();
    let mut trajectory = Vec::new();
    let sample = (steps / 8).max(1);

    for step in 0..steps {
        engine.step(&mut store);
        let d = degree_sequence(&store);
        if store.node_count() != n0 {
            n_flat = false;
        }
        if store.edge_count() != e0 {
            e_flat = false;
        }
        if d != deg0 {
            degree_preserved = false;
        }
        if d != prev {
            seq_changed_steps += 1;
            if step >= late_from {
                seq_changed_late += 1;
            }
        }
        prev = d.clone();
        if step % sample == 0 || step + 1 == steps {
            let spread = d.last().copied().unwrap_or(0) - d.first().copied().unwrap_or(0);
            trajectory.push((
                step,
                store.node_count(),
                store.edge_count(),
                components(&store),
                d.first().copied().unwrap_or(0),
                d.last().copied().unwrap_or(0),
                spread,
                d.iter().filter(|&&x| x == 0).count(),
            ));
        }
    }

    let by_rule_snapshot = by_rule.lock().expect("counter poisoned").clone();
    let late_snapshot = late.lock().expect("counter poisoned").clone();
    let events: u64 = by_rule_snapshot.values().sum();
    let late_events: u64 = late_snapshot.values().sum();
    let d_end = degree_sequence(&store);

    eprintln!("\n  ===== {label} =====");
    eprintln!(
        "    n {} -> {} ({})   e {} -> {} ({})   degree sequence {}   components {} -> {}",
        n0,
        store.node_count(),
        if n_flat { "FLAT throughout" } else { "MOVED" },
        e0,
        store.edge_count(),
        if e_flat { "FLAT throughout" } else { "MOVED" },
        if degree_preserved { "PRESERVED exactly" } else { "CHANGED" },
        components(&init.build()),
        components(&store)
    );
    eprintln!("    steps={steps} events={events}  (final 10%: {late_events})");
    eprintln!("    trajectory (step, n, e, comp, deg min..max, SPREAD, isolated):");
    for (s, n, e, k, lo, hi, sp, iso) in &trajectory {
        eprintln!("      t={s:<6} n={n:<4} e={e:<4} comp={k:<3} deg {lo}..{hi}  spread={sp}  isolated={iso}");
    }
    eprintln!(
        "    degree-sequence CHANGES: {seq_changed_steps} of {steps} steps  ({seq_changed_late} in the final 10%)"
    );
    eprintln!("    liveness / perpetuality (fire counts only):");
    for (name, count) in &by_rule_snapshot {
        let l = late_snapshot.get(name).copied().unwrap_or(0);
        eprintln!(
            "      {name:<18} fired={count:<9} in final 10%: {l:<8} {}",
            if *count == 0 {
                "<< DEAD"
            } else if l == 0 {
                "<< SELF-TERMINATED"
            } else {
                "perpetual"
            }
        );
    }

    Char {
        events,
        late: late_events,
        n_flat,
        e_flat,
        degree_preserved,
        seq_changed_steps,
        seq_changed_late,
        range_end: (
            d_end.first().copied().unwrap_or(0),
            d_end.last().copied().unwrap_or(0),
        ),
    }
}

#[test]
#[ignore = "long run: E2.2 bare characterisation"]
fn e2_2_characterise_the_self_balancing_rule_bare() {
    eprintln!("\n################################################################################");
    eprintln!("#  E-TRACK / E2.2 — BARE CHARACTERISATION (physics first, firewall gate)");
    eprintln!("#  Pre-registration: docs/etrack_preregistration.md @ 17b0c02");
    eprintln!("#  Predicted: n/e flat, minted/event 0.0000, degree sequence CHANGES, degree");
    eprintln!("#  RANGE CONTRACTS toward uniformity. Named risk: uniformity is an absorbing state.");
    eprintln!("################################################################################");

    // The controls, re-run at the same horizon so the arm is read against
    // measured numbers rather than against numbers quoted from the D-track.
    let bounded = characterise(
        "CONTROL (D2 arm E′): rewire_bounded — degree-changing, substrate-derived constants",
        dtrack::REWIRE_BOUNDED,
        &dtrack::bare_crystal(),
        8_000,
    );
    let arm = characterise(
        "ARM G: rebalance — self-balancing, ZERO constants (the new grammar form)",
        etrack::REBALANCE,
        &dtrack::bare_crystal(),
        8_000,
    );

    eprintln!("\n  ---- E2.1 predictions vs result ----");
    eprintln!("    arm               n flat  e flat  degree seq  seq-changes (late)  events   final-10%  deg range  GATE");
    for (name, c) in [("rewire_bounded", &bounded), ("rebalance", &arm)] {
        eprintln!(
            "    {name:<17} {:<7} {:<7} {:<11} {:>6} ({:>5})       {:<8} {:<10} {}..{:<7} {}",
            c.n_flat,
            c.e_flat,
            if c.degree_preserved { "preserved" } else { "CHANGED" },
            c.seq_changed_steps,
            c.seq_changed_late,
            c.events,
            c.late,
            c.range_end.0,
            c.range_end.1,
            if c.events > 0 && c.n_flat && c.late > 0 { "PASS" } else { "FAIL" }
        );
    }

    // ---- Predictions committed for the arm ----
    assert!(arm.n_flat, "rebalance: node count must be exactly flat");
    assert!(arm.e_flat, "rebalance: edge count must be exactly flat");
    assert!(
        !arm.degree_preserved,
        "rebalance exists to CHANGE the degree sequence — that is the design claim, and if it \
         holds the sequence fixed it is not the rule that was pre-registered"
    );

    // ---- The E2.3 gate, read explicitly rather than left for a reader ----
    let live = arm.events > 0 && arm.late > 0;
    eprintln!("\n  ---- E2.3 gate reading ----");
    if !live {
        eprintln!(
            "    GATE KILL by degree uniformity: the rule self-terminated. Both MONOTONE\n\
             \x20   (`rewire`) and SELF-BALANCING (`rebalance`) degree dynamics have absorbing\n\
             \x20   states at fixed n and fixed e — a STRONGER structural claim than the one this\n\
             \x20   track set out to test. No E3 row."
        );
    } else if arm.seq_changed_late == 0 {
        eprintln!(
            "    LIVE BUT ASYMPTOTICALLY SEQUENCE-PRESERVING — the third pre-registered outcome.\n\
             \x20   The rule is still firing ({} in the final 10%) but the degree SEQUENCE stopped\n\
             \x20   moving. Self-balancing converges to conservation, so the treatment collapses\n\
             \x20   onto its own control. Reported as its own branch.",
            arm.late
        );
    } else {
        eprintln!(
            "    PASSES: live and perpetual at flat n, degree sequence still moving in the final\n\
             \x20   10% ({} changes). Proceeds to E3.",
            arm.seq_changed_late
        );
    }

    eprintln!("\n################################################################################\n");
}
