//! **C2 — bare characterisation, the firewall gate.**
//!
//! A C-track rule is characterised as physics — node/edge trajectory, degree
//! sequence, connectivity, liveness, perpetuality — and entered in the zoo
//! **before** any alphabet comparison and before any other track may use it.
//!
//! *Gate:* a rule that is not live-and-perpetual at flat `n` is not a C-track
//! rule, however good its alphabet looks over a short window. Three of the four
//! zoo candidates the B-track probed died exactly here.
//!
//! ```text
//! cargo test --release -p hg-alphabet --test c2_bare_characterisation -- --include-ignored --nocapture
//! ```

use hg_core::{HypergraphStore, NodeId};
use hg_engine::Engine;
use hg_observe::{Observer, RewriteEvent, StepStats};
use hg_rules::{compile_rules, parse_rules_toml, StepMode};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use hg_alphabet::btrack;
use hg_alphabet::run::Init;

struct Counter {
    by_rule: Arc<Mutex<BTreeMap<String, u64>>>,
    /// Firings in the LAST tenth of the run — the perpetuality test. A rule
    /// that fired a lot early and nothing late has self-terminated, which is
    /// how `influx` died.
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
        *self.step.lock().unwrap() = step + 1;
    }
}

/// Degree multiset, sorted — the quantity `swap` claims to preserve exactly.
fn degree_sequence(store: &HypergraphStore) -> Vec<usize> {
    let mut d: Vec<usize> = store
        .nodes()
        .map(|(id, _)| store.edges_incident_to(id).len())
        .collect();
    d.sort_unstable();
    d
}

/// Connected components by union-find over incidences.
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

struct Characterisation {
    n0: usize,
    e0: usize,
    n_end: usize,
    e_end: usize,
    events: u64,
    steps_run: u64,
    n_flat: bool,
    e_flat: bool,
    degree_preserved: bool,
    comp0: usize,
    comp_end: usize,
    by_rule: BTreeMap<String, u64>,
    late: BTreeMap<String, u64>,
    trajectory: Vec<(u64, usize, usize, usize)>,
}

fn characterise(label: &str, toml: &str, init: &Init, steps: u64) -> Characterisation {
    let mut store = init.build();
    let (n0, e0) = (store.node_count(), store.edge_count());
    let deg0 = degree_sequence(&store);
    let comp0 = components(&store);

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
    let mut steps_run = 0;
    let sample = (steps / 8).max(1);

    for step in 0..steps {
        engine.step(&mut store);
        steps_run = step + 1;
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
            trajectory.push((step, store.node_count(), store.edge_count(), components(&store)));
        }
    }

    // Taken ONCE each, before the struct literal. Locking the same mutex twice
    // inside one struct literal deadlocks: the first `MutexGuard` is a temporary
    // that lives until the end of the enclosing STATEMENT, and `std::sync::Mutex`
    // is not reentrant. That bug was live here and hung the run.
    let by_rule_snapshot = by_rule.lock().expect("counter poisoned").clone();
    let late_snapshot = late.lock().expect("counter poisoned").clone();
    let total_events: u64 = by_rule_snapshot.values().sum();

    let c = Characterisation {
        n0,
        e0,
        n_end: store.node_count(),
        e_end: store.edge_count(),
        events: total_events,
        steps_run,
        n_flat,
        e_flat,
        degree_preserved,
        comp0,
        comp_end: components(&store),
        by_rule: by_rule_snapshot,
        late: late_snapshot,
        trajectory,
    };

    eprintln!("\n  ===== {label} =====");
    eprintln!(
        "    n {} -> {} ({})   e {} -> {} ({})   degree sequence {}   components {} -> {}",
        c.n0,
        c.n_end,
        if c.n_flat { "FLAT throughout" } else { "MOVED" },
        c.e0,
        c.e_end,
        if c.e_flat { "FLAT throughout" } else { "MOVED" },
        if c.degree_preserved { "PRESERVED exactly" } else { "changed" },
        c.comp0,
        c.comp_end
    );
    eprintln!("    steps={} events={}", c.steps_run, c.events);
    eprintln!("    trajectory (step, n, e, components):");
    for (s, n, e, k) in &c.trajectory {
        eprintln!("      t={s:<6} n={n:<6} e={e:<6} comp={k}");
    }
    eprintln!("    liveness / perpetuality (fire counts only):");
    for (name, count) in &c.by_rule {
        let late = c.late.get(name).copied().unwrap_or(0);
        eprintln!(
            "      {name:<18} fired={count:<9} in final 10%: {late:<8} {}",
            if *count == 0 {
                "<< DEAD"
            } else if late == 0 {
                "<< SELF-TERMINATED"
            } else {
                "perpetual"
            }
        );
    }
    c
}

#[test]
#[ignore = "long run: C2 bare characterisation"]
fn c2_characterise_the_gap_rules_bare() {
    eprintln!("\n################################################################################");
    eprintln!("#  C-TRACK / C2 — BARE CHARACTERISATION (physics first, firewall gate)");
    eprintln!("#  Pre-registration: docs/ctrack_preregistration.md");
    eprintln!("################################################################################");

    // Long run: the bounded_churn byproduct (36 nodes / 24 EDGES after 2400
    // steps) is why the edge trajectory is mandatory and why it must be long.
    let swap = characterise(
        "swap on PureCrystal(6x6 arity3)",
        btrack::SWAP,
        &Init::PureCrystal { w: 6, h: 6, arity: 3, seed_label1: false },
        8_000,
    );
    let cyc = characterise(
        "cyclic3 on LabelledCrystal(6x6 arity3, 3 states, seed 42) — PRE-REGISTERED",
        btrack::CYCLIC3,
        &Init::LabelledCrystal { w: 6, h: 6, arity: 3, states: 3, seed: 42, periodic: false },
        8_000,
    );
    // POST-HOC deviation, recorded as one: the substrate (not the rule) is
    // corrected for two mechanism defects — an open boundary is a sink for
    // waves, and 6x6 has no room for a spiral core. Run once.
    let cyc_t = characterise(
        "cyclic3 on LabelledCrystal(24x24 arity3, 3 states, seed 42, PERIODIC) — post-hoc deviation",
        btrack::CYCLIC3,
        &Init::LabelledCrystal { w: 24, h: 24, arity: 3, states: 3, seed: 42, periodic: true },
        8_000,
    );

    eprintln!("\n  ---- C1 predictions vs result ----");
    let arms: [(&str, &Characterisation); 3] = [
        ("swap", &swap),
        ("cyclic3@6x6", &cyc),
        ("cyclic3@torus", &cyc_t),
    ];
    eprintln!("    arm            n flat  e flat  deg preserved  minted/event  events   final-10%  GATE");
    for (name, c) in arms {
        let late: u64 = c.late.values().sum();
        eprintln!(
            "    {name:<14} {:<7} {:<7} {:<14} {:<13.4} {:<8} {:<10} {}",
            c.n_flat,
            c.e_flat,
            c.degree_preserved,
            (c.n_end as i64 - c.n0 as i64) as f64 / c.events.max(1) as f64,
            c.events,
            late,
            if c.events > 0 && c.n_flat && late > 0 { "PASS" } else { "FAIL" }
        );
    }

    // ---- Predictions that held for EVERY arm, asserted ----
    for (name, c) in arms {
        assert!(c.n_flat, "{name}: node count moved ({} -> {})", c.n0, c.n_end);
        assert_eq!(
            c.n_end as i64 - c.n0 as i64,
            0,
            "{name}: minted/event must be exactly 0.0000 as predicted"
        );
        assert!(c.e_flat, "{name}: edge count must be exactly flat");
    }

    // ---- `swap` PASSES, and its distinguishing structural claim holds ----
    assert!(swap.events > 0 && swap.late.values().sum::<u64>() > 0, "swap must be perpetual");
    assert!(
        swap.degree_preserved,
        "swap must preserve the whole degree sequence exactly — that is the mechanism, and if it \
         does not hold the rule is not the one that was pre-registered"
    );

    // ---- `cyclic3` FAILS the gate on its PRE-REGISTERED substrate ----
    // Pinned as a result, not tolerated as a flake: the C2 gate exists to kill
    // rules like this, and if it ever stops killing this one the gate has
    // stopped working.
    assert!(
        cyc.late.values().sum::<u64>() == 0,
        "cyclic3@6x6 self-terminated when measured, and that failure is the published C2 result"
    );
    assert!(
        cyc.events < 100,
        "cyclic3@6x6 published as firing {} times in 8000 steps before freezing",
        cyc.events
    );

    eprintln!(
        "\n  C2 GATE: `swap` PASSES. `cyclic3` FAILS on its pre-registered 6x6 open substrate\n\
         \x20  ({} events in 8000 steps, none in the final 10%). The torus arm is a recorded\n\
         \x20  post-hoc substrate deviation and is reported separately, never as the pre-registered\n\
         \x20  result.",
        cyc.events
    );
    eprintln!("\n################################################################################\n");
}
