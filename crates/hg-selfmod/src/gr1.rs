//! G-R1 mutational-neutrality harness (EXP-R01).
//!
//! Protocol: `docs/ere_gr1_protocol.md`. Samples single-edit mutations from
//! the **production** kernel (`meta::mutate_once_with_op` — same draws, same
//! application as evolution), classifies each outcome, and evaluates the
//! pre-committed thresholds. No engine changes; trajectories reuse the
//! library-driver object step with unlimited budgets (the economy is not
//! what is being measured here).

use crate::decode::{decode_all, DecodedRule, N_VALIDATORS};
use crate::encode::install_rule;
use crate::gates::{
    churn_k_specs, d1_tie_specs, g0_fixtures, g1_specs, g2_specs, rcf_churnk_like, rcf_guard_heavy,
    rcf_rich_base,
};
use crate::meta::{mutate_once_with_op, N_OPS, OP_NAMES};
use crate::runner::{object_step, seed_rule_specs, seed_world, RunConfig};
use crate::schema::*;
use hg_core::{HypergraphStore, NodeId};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::collections::{HashMap, HashSet};

// ---------- constants (mirrored in docs/ere_gr1_protocol.md) ----------

/// Full-gate sample count (plan §6: 10^4).
pub const GR1_SAMPLES: usize = 10_000;
/// Trajectory length for NearNeutral / Catastrophic classification (plan: ≥100).
pub const GR1_TRAJ_STEPS: usize = 100;
/// Harness sampling stream (operator choice, params, seed pick).
pub const GR1_SEED: u64 = 0x5231_0000_0000_0001;
/// Fixture-world construction stream.
pub const GR1_WORLD_SEED: u64 = 0x5231_0000_0000_0002;
/// Per-trajectory object-step stream (same for parent and mutant).
pub const GR1_TRAJ_SEED: u64 = 0x5231_0000_0000_0003;
/// Fixture world size (small for speed; dynamics still exercised).
pub const GR1_WORLD_NODES: usize = 64;
pub const GR1_WORLD_EDGES: usize = 96;
/// Explosion definition: final > MULT × initial nodes or edges.
pub const GR1_EXPLODE_MULT: usize = 4;
/// How many RCF-identical (Neutral) pairs get trajectory spot-checks
/// (RCF-equality ⇒ behavioral-equality soundness check).
pub const GR1_NEUTRAL_SPOTCHECK: usize = 50;

// ---------- outcome classes ----------

/// Class indices (also index `per_op` columns).
pub const C_NEUTRAL: usize = 0;
pub const C_NEAR: usize = 1;
pub const C_DIVERGENT: usize = 2;
pub const C_QUARANTINED: usize = 3;
pub const C_CATASTROPHIC: usize = 4;
pub const N_CLASSES: usize = 5;

pub const CLASS_NAMES: [&str; N_CLASSES] = [
    "neutral",
    "near_neutral",
    "divergent",
    "quarantined",
    "catastrophic",
];

#[derive(Default)]
pub struct Gr1Report {
    pub n: usize,
    pub traj_steps: usize,
    pub n_seeds: usize,
    pub counts: [usize; N_CLASSES],
    pub per_op: Vec<[usize; N_CLASSES]>,
    pub per_validator: [usize; N_VALIDATORS],
    /// SB-R1 split: (valid, total) for at-cap vs off-cap parents.
    pub cap_valid: usize,
    pub cap_total: usize,
    pub off_valid: usize,
    pub off_total: usize,
    /// RCF-soundness spot-checks performed (all passed — else panic).
    pub neutral_spotchecks: usize,
}

impl Gr1Report {
    pub fn valid_frac(&self) -> f64 {
        (self.counts[C_NEUTRAL]
            + self.counts[C_NEAR]
            + self.counts[C_DIVERGENT]
            + self.counts[C_CATASTROPHIC]) as f64
            / self.n.max(1) as f64
    }
    pub fn near_frac(&self) -> f64 {
        self.counts[C_NEAR] as f64 / self.n.max(1) as f64
    }
    pub fn cata_frac(&self) -> f64 {
        self.counts[C_CATASTROPHIC] as f64 / self.n.max(1) as f64
    }
}

// ---------- seed set (≥20 rules, pairwise RCF-distinct, all decode) ----------

/// Named seed specs. Must stay ≥20 after RCF dedupe; every entry must decode
/// (asserted at build — a non-decoding seed is a test bug, fail loudly).
pub fn gr1_seed_set() -> Vec<(&'static str, RuleSpec)> {
    let mut v: Vec<(&'static str, RuleSpec)> = Vec::new();
    for (i, s) in seed_rule_specs().into_iter().enumerate() {
        let name: &'static str = Box::leak(format!("seed{i}").into_boxed_str());
        v.push((name, s));
    }
    v.push((
        "abio_minimal",
        RuleSpec {
            priority: 0,
            n_vars: 1,
            var_labels: vec![Some(0)],
            pedges: vec![],
            guards: vec![],
            deletes: vec![],
            relabels: vec![(0, 1)],
            creates: vec![],
            redges: vec![],

            distinct: vec![],

            effects: vec![],
        },
    ));
    for (i, s) in g2_specs().into_iter().enumerate() {
        let name: &'static str = Box::leak(format!("churn3_{i}").into_boxed_str());
        v.push((name, s));
    }
    for (i, s) in churn_k_specs().into_iter().enumerate() {
        let name: &'static str = Box::leak(format!("churnk_{i}").into_boxed_str());
        v.push((name, s));
    }
    for (i, s) in g0_fixtures().into_iter().enumerate() {
        let name: &'static str = Box::leak(format!("g0_{i}").into_boxed_str());
        v.push((name, s));
    }
    for (i, s) in g1_specs().into_iter().enumerate() {
        let name: &'static str = Box::leak(format!("g1_{i}").into_boxed_str());
        v.push((name, s));
    }
    v.push(("rich_base", rcf_rich_base()));
    v.push(("guard_heavy", rcf_guard_heavy()));
    v.push(("churnk_like", rcf_churnk_like()));
    let (tie_a, tie_b, tie_hi) = d1_tie_specs();
    v.push(("tie_a", tie_a));
    v.push(("tie_b", tie_b));
    v.push(("tie_hi", tie_hi));
    v.push((
        "edge_create",
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
    ));
    v.push((
        "edge_delete",
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
    ));
    // Dedupe by RCF (uniform-over-set must not oversample one class).
    let mut seen: HashSet<Vec<u8>> = HashSet::new();
    v.retain(|(_, s)| seen.insert(s.rcf_bytes()));
    // Every seed must decode (non-vacuous, within caps).
    let caps = Caps::default();
    for (name, s) in &v {
        let mut rs = HypergraphStore::new();
        let mut seq = 0i64;
        install_rule(&mut rs, s, &mut seq);
        let (rules, report) = decode_all(&rs, &caps);
        assert_eq!(
            report.total, 0,
            "G-R1 seed {name} must decode (quarantine={:?})",
            report.by_validator
        );
        assert_eq!(rules.len(), 1, "G-R1 seed {name} must decode to one rule");
    }
    assert!(
        v.len() >= 20,
        "G-R1 needs ≥20 RCF-distinct seeds, have {}",
        v.len()
    );
    v
}

fn parent_at_cap(spec: &RuleSpec, caps: &Caps) -> bool {
    spec.pedges.iter().any(|e| e.len() >= caps.max_arity)
        || spec.redges.iter().any(|e| e.len() >= caps.max_arity)
}

// ---------- trajectories ----------

struct Traj {
    hashes: Vec<u64>,
    productive: usize,
    final_nodes: usize,
    final_edges: usize,
    init_nodes: usize,
    init_edges: usize,
}

fn unlimited_budgets(rules: &[DecodedRule]) -> HashMap<NodeId, i64> {
    rules.iter().map(|d| (d.root, i64::MAX / 4)).collect()
}

fn fixture_world() -> HypergraphStore {
    let mut rng = ChaCha8Rng::seed_from_u64(GR1_WORLD_SEED);
    let mut w = HypergraphStore::new();
    seed_world(&mut w, &mut rng, GR1_WORLD_NODES, GR1_WORLD_EDGES, 4);
    w
}

fn run_trajectory(spec: &RuleSpec, caps: &Caps, cfg: &RunConfig, steps: usize) -> Option<Traj> {
    let mut rs = HypergraphStore::new();
    let mut seq = 0i64;
    install_rule(&mut rs, spec, &mut seq);
    let (rules, report) = decode_all(&rs, caps);
    if report.total != 0 || rules.len() != 1 {
        return None;
    }
    let mut w = fixture_world();
    let init_nodes = w.node_count();
    let init_edges = w.edge_count();
    let mut rng = ChaCha8Rng::seed_from_u64(GR1_TRAJ_SEED);
    let budgets = unlimited_budgets(&rules);
    let mut hashes = Vec::with_capacity(steps);
    let mut productive = 0usize;
    for _ in 0..steps {
        let oo = object_step(&mut w, &rules, &mut rng, cfg, &budgets);
        productive += oo.productive_applied;
        hashes.push(w.canonical_hash());
        // Early-out: explosion is already decided — running the remaining
        // steps would only burn match cost on a doomed world. The shortened
        // hash vector still differs from any non-exploded parent, and the
        // final counts keep the catastrophic classification exact.
        if w.node_count() > GR1_EXPLODE_MULT * init_nodes.max(1)
            || w.edge_count() > GR1_EXPLODE_MULT * init_edges.max(1)
        {
            break;
        }
    }
    Some(Traj {
        hashes,
        productive,
        final_nodes: w.node_count(),
        final_edges: w.edge_count(),
        init_nodes,
        init_edges,
    })
}

// ---------- main loop ----------

pub fn run_gr1(n_samples: usize, traj_steps: usize) -> Gr1Report {
    let caps = Caps::default();
    let cfg = RunConfig::default();
    let seeds = gr1_seed_set();
    let parent_rcfs: Vec<Vec<u8>> = seeds.iter().map(|(_, s)| s.rcf_bytes()).collect();
    let parent_cap: Vec<bool> = seeds.iter().map(|(_, s)| parent_at_cap(s, &caps)).collect();
    let mut rng = ChaCha8Rng::seed_from_u64(GR1_SEED);
    let mut rep = Gr1Report {
        n: n_samples,
        traj_steps,
        n_seeds: seeds.len(),
        per_op: vec![[0usize; N_CLASSES]; N_OPS as usize],
        ..Default::default()
    };
    let mut parent_trajs: HashMap<Vec<u8>, Traj> = HashMap::new();
    let mut neutral_checked = 0usize;

    for (done, _) in (0..n_samples).enumerate() {
        if done > 0 && done % 1000 == 0 {
            eprintln!("  gr1 progress: {done}/{n_samples}");
        }
        let si = (rng.gen::<u64>() % seeds.len() as u64) as usize;
        let cap = parent_cap[si];
        if cap {
            rep.cap_total += 1;
        } else {
            rep.off_total += 1;
        }
        let mut rs = HypergraphStore::new();
        let mut seq = 0i64;
        let root = install_rule(&mut rs, &seeds[si].1, &mut seq);
        let (_changed, op) = mutate_once_with_op(&mut rs, root, &mut rng, &caps);
        let (rules, report) = decode_all(&rs, &caps);
        if report.total > 0 {
            let v = report
                .by_validator
                .iter()
                .position(|&c| c > 0)
                .expect("total>0 implies a firing validator");
            rep.counts[C_QUARANTINED] += 1;
            rep.per_op[op][C_QUARANTINED] += 1;
            rep.per_validator[v] += 1;
            continue;
        }
        assert_eq!(
            rules.len(),
            1,
            "single-root store decoded to {} rules",
            rules.len()
        );
        // Valid from here on.
        if cap {
            rep.cap_valid += 1;
        } else {
            rep.off_valid += 1;
        }
        let mrcf = rules[0].spec.rcf_bytes();
        if mrcf == parent_rcfs[si] {
            rep.counts[C_NEUTRAL] += 1;
            rep.per_op[op][C_NEUTRAL] += 1;
            // RCF-soundness spot-check: RCF-identical must behave identically.
            if neutral_checked < GR1_NEUTRAL_SPOTCHECK {
                neutral_checked += 1;
                rep.neutral_spotchecks += 1;
                let pt = parent_trajs
                    .entry(parent_rcfs[si].clone())
                    .or_insert_with(|| {
                        run_trajectory(&seeds[si].1, &caps, &cfg, traj_steps)
                            .expect("seed must run")
                    });
                let mt = run_trajectory(&rules[0].spec, &caps, &cfg, traj_steps)
                    .expect("neutral mutant must run");
                assert_eq!(
                    pt.hashes, mt.hashes,
                    "RCF-identical mutant diverged from parent — RCF unsound"
                );
            }
            continue;
        }
        let pt = parent_trajs
            .entry(parent_rcfs[si].clone())
            .or_insert_with(|| {
                run_trajectory(&seeds[si].1, &caps, &cfg, traj_steps).expect("seed must run")
            });
        let mt =
            run_trajectory(&rules[0].spec, &caps, &cfg, traj_steps).expect("valid mutant must run");
        if pt.hashes == mt.hashes {
            rep.counts[C_NEAR] += 1;
            rep.per_op[op][C_NEAR] += 1;
        } else if mt.productive == 0
            || mt.final_nodes > GR1_EXPLODE_MULT * mt.init_nodes.max(1)
            || mt.final_edges > GR1_EXPLODE_MULT * mt.init_edges.max(1)
        {
            rep.counts[C_CATASTROPHIC] += 1;
            rep.per_op[op][C_CATASTROPHIC] += 1;
        } else {
            rep.counts[C_DIVERGENT] += 1;
            rep.per_op[op][C_DIVERGENT] += 1;
        }
    }
    rep
}

// ---------- gate ----------

pub fn print_gr1_report(rep: &Gr1Report) {
    println!(
        "G-R1: {} samples × {}-step trajectories over {} seeds (spotchecks {})",
        rep.n, rep.traj_steps, rep.n_seeds, rep.neutral_spotchecks
    );
    for (i, name) in CLASS_NAMES.iter().enumerate() {
        println!(
            "  {name:>14}: {:>6} ({:.3})",
            rep.counts[i],
            rep.counts[i] as f64 / rep.n.max(1) as f64
        );
    }
    println!(
        "  valid={:.3} (threshold ≥0.40) near={:.3} (≥0.10) catastrophic={:.3} (≤0.25)",
        rep.valid_frac(),
        rep.near_frac(),
        rep.cata_frac()
    );
    println!("  per-op (neutral/near/divergent/quarantined/catastrophic):");
    for (i, name) in OP_NAMES.iter().enumerate() {
        let r = rep.per_op[i];
        let tot: usize = r.iter().sum();
        println!(
            "    {name:>22}: {:>5} [{}/{}/{}/{}/{}]",
            tot, r[0], r[1], r[2], r[3], r[4]
        );
    }
    println!("  quarantine by validator: {:?}", rep.per_validator);
    println!(
        "  SB-R1 split: at-cap valid {}/{} ({:.3})  off-cap valid {}/{} ({:.3})",
        rep.cap_valid,
        rep.cap_total,
        rep.cap_valid as f64 / rep.cap_total.max(1) as f64,
        rep.off_valid,
        rep.off_total,
        rep.off_valid as f64 / rep.off_total.max(1) as f64
    );
}

pub fn gate_gr1() -> bool {
    let rep = run_gr1(GR1_SAMPLES, GR1_TRAJ_STEPS);
    print_gr1_report(&rep);
    let pass = rep.valid_frac() >= 0.40 && rep.near_frac() >= 0.10 && rep.cata_frac() <= 0.25;
    if pass {
        println!("G-R1 PASS: mutational neutrality thresholds hold — track proceeds");
    } else if rep.valid_frac() < 0.10 {
        println!("G-R1 FAIL (redesign): Valid < 0.10 — encoding not searchable; redesign E1, do not tune the loop");
    } else {
        println!("G-R1 FAIL (descope): thresholds unmet — track proceeds only with a smaller mutation kernel, recorded as an encoding finding");
    }
    pass
}
