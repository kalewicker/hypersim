//! **D2.3 — bare characterisation, the firewall gate.**
//!
//! Same template as C2 (`tests/c2_bare_characterisation.rs`), and the gate is
//! real: it killed `cyclic3` on its pre-registered substrate at 59 firings in
//! 8000 steps. A rule that is not live-and-perpetual at flat `n` is not a D2
//! rule, however good its alphabet looks over a short window.
//!
//! Arm E's design claim is that the degree sequence **CHANGES** — the opposite
//! of the claim C2 asserted for `swap` — so it is asserted here in the same
//! place, for the same reason: if it does not hold, the rule is not the one that
//! was pre-registered.
//!
//! ```text
//! cargo test --release -p hg-alphabet --test d2_3_bare_characterisation -- --ignored --nocapture
//! ```

use hg_core::{HypergraphStore, NodeId};
use hg_engine::Engine;
use hg_observe::{Observer, RewriteEvent, StepStats};
use hg_rules::{compile_rules, parse_rules_toml, StepMode};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use hg_alphabet::btrack;
use hg_alphabet::dtrack;
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
        let mut m = self.by_rule.lock().unwrap();
        let mut l = self.late.lock().unwrap();
        for ev in events {
            *m.entry(ev.rule_name.clone()).or_insert(0) += 1;
            if step >= self.late_from {
                *l.entry(ev.rule_name.clone()).or_insert(0) += 1;
            }
        }
        drop(m);
        drop(l);
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

/// Degree spread — reported because "the sequence changed" is a bit, and the
/// *shape* of the change is the physics. `rewire` moves incidences from low to
/// high, so max degree and isolated-node count are the quantities that say what
/// the rule is actually doing to the substrate.
fn degree_shape(store: &HypergraphStore) -> (usize, usize, usize) {
    let d = degree_sequence(store);
    let max = d.last().copied().unwrap_or(0);
    let min = d.first().copied().unwrap_or(0);
    let isolated = d.iter().filter(|&&x| x == 0).count();
    (min, max, isolated)
}

struct Characterisation {
    n0: usize,
    e0: usize,
    n_end: usize,
    e_end: usize,
    events: u64,
    n_flat: bool,
    e_flat: bool,
    degree_preserved: bool,
    late: BTreeMap<String, u64>,
}

fn characterise(label: &str, toml: &str, init: &Init, steps: u64) -> Characterisation {
    let mut store = init.build();
    let (n0, e0) = (store.node_count(), store.edge_count());
    let deg0 = degree_sequence(&store);

    let rules = compile_rules(&parse_rules_toml(toml).expect("parse")).expect("compile");
    let mut engine = Engine::new(rules, StepMode::MaxParallel, 1, 1_000_000);
    let by_rule = Arc::new(Mutex::new(BTreeMap::new()));
    let late = Arc::new(Mutex::new(BTreeMap::new()));
    engine.add_observer(Box::new(Counter {
        by_rule: Arc::clone(&by_rule),
        late: Arc::clone(&late),
        step: Arc::new(Mutex::new(0)),
        late_from: steps - steps / 10,
    }));

    let mut n_flat = true;
    let mut e_flat = true;
    let mut degree_preserved = true;
    let mut trajectory = Vec::new();
    let sample = (steps / 8).max(1);

    for step in 0..steps {
        engine.step(&mut store);
        if store.node_count() != n0 {
            n_flat = false;
        }
        if store.edge_count() != e0 {
            e_flat = false;
        }
        if degree_sequence(&store) != deg0 {
            degree_preserved = false;
        }
        if step % sample == 0 || step + 1 == steps {
            let (dmin, dmax, iso) = degree_shape(&store);
            trajectory.push((step, store.node_count(), store.edge_count(), components(&store), dmin, dmax, iso));
        }
    }

    // Taken ONCE each, before the struct literal — locking the same mutex twice
    // inside one statement deadlocks (the C2 harness had that bug live).
    let by_rule_snapshot = by_rule.lock().expect("counter poisoned").clone();
    let late_snapshot = late.lock().expect("counter poisoned").clone();
    let total_events: u64 = by_rule_snapshot.values().sum();

    eprintln!("\n  ===== {label} =====");
    let (dmin0, dmax0, iso0) = {
        let s = init.build();
        degree_shape(&s)
    };
    let (dmin, dmax, iso) = degree_shape(&store);
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
    eprintln!(
        "    degree shape: min/max/isolated  {dmin0}/{dmax0}/{iso0}  ->  {dmin}/{dmax}/{iso}"
    );
    eprintln!("    steps={steps} events={total_events}");
    eprintln!("    trajectory (step, n, e, comp, dmin, dmax, isolated):");
    for (s, n, e, k, lo, hi, is) in &trajectory {
        eprintln!("      t={s:<6} n={n:<5} e={e:<5} comp={k:<5} deg {lo}..{hi} isolated={is}");
    }
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

    Characterisation {
        n0,
        e0,
        n_end: store.node_count(),
        e_end: store.edge_count(),
        events: total_events,
        n_flat,
        e_flat,
        degree_preserved,
        late: late_snapshot,
    }
}

#[test]
#[ignore = "long run: D2.3 bare characterisation"]
fn d2_3_characterise_the_matched_treatment_bare() {
    eprintln!("\n################################################################################");
    eprintln!("#  D-TRACK / D2.3 — BARE CHARACTERISATION (physics first, firewall gate)");
    eprintln!("#  Pre-registration: docs/dtrack_preregistration.md @ 2901f56");
    eprintln!("################################################################################");

    // The control, re-characterised at the same horizon so the arms are read
    // against it rather than against a number quoted from the C-track.
    let swap = characterise(
        "CONTROL: swap on PureCrystal(6x6 arity3) — conserves the degree sequence",
        btrack::SWAP,
        &dtrack::bare_crystal(),
        8_000,
    );
    let rewire = characterise(
        "ARM E: rewire on PureCrystal(6x6 arity3) — degree-CHANGING",
        dtrack::REWIRE,
        &dtrack::bare_crystal(),
        8_000,
    );
    let swap_lab = characterise(
        "ARM F: swap on LabelledCrystal(6x6 arity3, 3 states, seed 42) — rule UNTOUCHED",
        btrack::SWAP,
        &dtrack::labelled_crystal(),
        8_000,
    );

    // Arm E′ — the ONE recorded post-hoc deviation. Arm E self-terminates
    // because it moves incidence in one direction always and so has an
    // absorbing state (max degree 6 -> 85: one node joins every edge, after
    // which no two node-disjoint 2-edges exist). E′ bounds the movement at both
    // ends using constants READ OFF the substrate's own initial degree range.
    // Run once; reported separately, never as the pre-registered arm.
    let rewire_b = characterise(
        "ARM E′ (RECORDED DEVIATION): rewire_bounded on PureCrystal(6x6 arity3)",
        dtrack::REWIRE_BOUNDED,
        &dtrack::bare_crystal(),
        8_000,
    );

    eprintln!("\n  ---- D2.2 predictions vs result ----");
    let arms: [(&str, &Characterisation, &str); 4] = [
        ("swap (control)", &swap, "preserved"),
        ("rewire (arm E)", &rewire, "CHANGES"),
        ("rewire_bounded (E')", &rewire_b, "CHANGES"),
        ("swap@labelled (arm F)", &swap_lab, "preserved"),
    ];
    eprintln!("    arm                    n flat  e flat  minted/event  degree seq  predicted   events    final-10%  GATE");
    for (name, c, predicted) in arms {
        let late: u64 = c.late.values().sum();
        eprintln!(
            "    {name:<22} {:<7} {:<7} {:<13.4} {:<11} {:<11} {:<9} {:<10} {}",
            c.n_flat,
            c.e_flat,
            (c.n_end as i64 - c.n0 as i64) as f64 / c.events.max(1) as f64,
            if c.degree_preserved { "preserved" } else { "CHANGED" },
            predicted,
            c.events,
            late,
            if c.events > 0 && c.n_flat && late > 0 { "PASS" } else { "FAIL" }
        );
    }

    // ---- Predictions committed for EVERY arm ----
    for (name, c, _) in arms {
        assert!(c.n_flat, "{name}: node count moved ({} -> {})", c.n0, c.n_end);
        assert_eq!(
            c.n_end as i64 - c.n0 as i64,
            0,
            "{name}: minted/event must be exactly 0.0000 as predicted"
        );
        assert!(c.e_flat, "{name}: edge count must be exactly flat ({} -> {})", c.e0, c.e_end);
    }

    // ---- Arm E's DESIGN CLAIM, asserted where C2 asserted `swap`'s ----
    assert!(
        !rewire.degree_preserved,
        "arm E exists to CHANGE the degree sequence — that is the single property separating it \
         from the `swap` control. If the sequence is preserved the rule is not the one that was \
         pre-registered and the arm answers nothing"
    );
    // …and the control's, so the contrast is pinned from both sides.
    assert!(
        swap.degree_preserved,
        "the `swap` control must still preserve the degree sequence exactly"
    );
    assert!(
        swap_lab.degree_preserved,
        "arm F leaves the rule untouched, so it must still preserve the degree sequence"
    );
    assert!(
        !rewire_b.degree_preserved,
        "arm E′ must also CHANGE the degree sequence, or it is not a treatment for `swap` at all"
    );

    // Arm E's gate FAILURE is pinned as a published result, not tolerated as a
    // flake. The D2.3 gate exists to kill rules like this — it killed `cyclic3`
    // at the C2 gate — and if it ever stops killing this one, it has stopped
    // working.
    assert!(
        rewire.late.values().sum::<u64>() == 0,
        "arm E self-terminated when measured, and that failure is the published D2.3 result"
    );
    assert!(
        rewire.events < 1_000,
        "arm E published as firing {} times in 8000 steps before freezing",
        rewire.events
    );

    eprintln!("\n################################################################################\n");
}
