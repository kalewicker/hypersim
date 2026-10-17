//! EXP-R06 + EXP-R07 (Phase R6): does the mutation kernel produce
//! RCF-novel non-trivial rules (R06), and what is the SHAPE of the
//! novelty growth curve over a long run (R07)?
//!
//! Design: a single long epoch-driven run per arm. Every epoch:
//!   - kernel-mutate one region, install if valid (epoch_transition),
//!   - read back ALL regions and register each in two registries:
//!       * RCF hash  — the plan's novelty class (labels included)
//!       * SHAPE key — RCF of the label-stripped candidate (FM-R6
//!         mitigation: a relabel of an old rule is NOT novel)
//!
//! Arms differ only in engine protection; region evolution is identical,
//! so A1/A2/A3 curves coincide by construction here — the run measures
//! THE KERNEL's reachable set, which is the honest EXP-R06 question.
//! (Engine-consumed novelty differences belong to EXP-R08's sweep.)
//!
//! Pre-committed interpretation (FM-R6): if the cumulative curve is
//! linear-to-saturating rather than accelerating, that is the Hash
//! Chemistry result reproducing on this substrate — reported as such.
//!
//! RUNTIME: release-scaled.

use super::epoch::epoch_transition;
use crate::graph_source::GraphRuleSource;
use crate::mutate::MutationRng;
use crate::rcf::rcf_hash;
use crate::schema::{find_rule_roots, read_region_with_footprint};
use hg_rules::source::RuleSource;

/// Deterministic xorshift for the kernel stream.
struct XorShift(u64);
impl MutationRng for XorShift {
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
}

fn pe(slots: Vec<usize>) -> crate::schema::CandidatePatternEdge {
    crate::schema::CandidatePatternEdge { slots }
}

fn seed_world() -> hg_core::HypergraphStore {
    let mut s = hg_core::HypergraphStore::new();
    for i in 0..10u32 {
        let mut d = hg_core::NodeData::new(hg_core::Label(i % 4), 0);
        d.set_property(1, hg_core::PropertyValue::Float(f64::from(i) * 0.1));
        s.add_node(d);
    }
    let ids: Vec<_> = s.nodes().map(|(id, _)| id).collect();
    for w in 0..8usize {
        let a = ids[w % ids.len()];
        let b = ids[(w * 3 + 1) % ids.len()];
        if a != b {
            s.add_edge(hg_core::HyperEdge::new(
                hg_core::Label((w % 3) as u32),
                smallvec::smallvec![a, b],
            ));
        }
    }
    // Seed regions: pair-churner (ternary) + unary pass-through.
    let cands = vec![
        crate::schema::CandidateRule {
            priority: 0,
            lhs: vec![pe(vec![0, 1])],
            rhs: vec![pe(vec![0, 2]), pe(vec![2, 1])],
            var_labels: vec![Some(0), Some(1), Some(2)],
            guards: Vec::new(),
            var_guards: Vec::new(),
            effects: Vec::new(),
        },
        crate::schema::CandidateRule {
            priority: 1,
            lhs: vec![pe(vec![0])],
            rhs: vec![pe(vec![0])],
            var_labels: vec![None],
            guards: Vec::new(),
            var_guards: Vec::new(),
            effects: Vec::new(),
        },
    ];
    GraphRuleSource::install_candidates(&mut s, &cands);
    s
}

/// FM-R6 mitigation key: RCF hash of the label-stripped candidate.
/// Two rules with identical pattern structure are the same shape.
fn shape_key(cand: &crate::schema::CandidateRule) -> u64 {
    let mut stripped = cand.clone();
    stripped.var_labels = vec![None; cand.n_vars()];
    stripped.priority = 0;
    rcf_hash(&stripped)
}

#[derive(Default)]
struct Registries {
    rcf: rustc_hash::FxHashSet<u64>,
    shapes: rustc_hash::FxHashSet<u64>,
    /// distinct shapes that ever installed as regions
    nontrivial_installed: usize,
}

struct SeedRun {
    curve: Vec<usize>,
    mutated: usize,
    quarantined: usize,
    installed: usize,
    rcf_total: usize,
}

fn run_seed(seed: u64, epochs: usize) -> SeedRun {
    let mut s = seed_world();
    let mut src = GraphRuleSource::new();
    let mut rng = XorShift(seed);
    let mut reg = Registries::default();

    // Register the seed rules first.
    let snap0 = s.snapshot();
    let d0 = src.rules(&snap0, 0);
    let _ = d0;
    for root in find_rule_roots(&snap0) {
        if let Ok((cand, _)) = read_region_with_footprint(&snap0, root) {
            reg.rcf.insert(rcf_hash(&cand));
            reg.shapes.insert(shape_key(&cand));
        }
    }

    // Curve sampled every epoch: cumulative distinct SHAPES (meta-excluded).
    let mut curve: Vec<usize> = Vec::with_capacity(epochs + 1);
    curve.push(reg.shapes.len());

    let mut mutated = 0usize;
    let mut quarantined = 0usize;
    let mut installed = 0usize;

    for _ in 0..epochs {
        let ev = epoch_transition(&mut src, &mut s, &mut rng, &mut reg.rcf, true);
        mutated += ev.mutated as usize;
        quarantined += ev.quarantined as usize;
        installed += ev.installed as usize;

        // Register everything currently living in the store.
        let snap = s.snapshot();
        for root in find_rule_roots(&snap) {
            if let Ok((cand, _)) = read_region_with_footprint(&snap, root) {
                reg.rcf.insert(rcf_hash(&cand));
                if reg.shapes.insert(shape_key(&cand)) && !cand.lhs.is_empty() {
                    reg.nontrivial_installed += 1;
                }
            }
        }
        curve.push(reg.shapes.len());
    }

    SeedRun {
        curve,
        mutated,
        quarantined,
        installed,
        rcf_total: reg.rcf.len(),
    }
}

fn quartile_deltas(curve: &[usize]) -> [f64; 4] {
    let n = curve.len() - 1;
    let q = |i: usize| curve[i * n / 4];
    [
        (q(1) - q(0)) as f64,
        (q(2) - q(1)) as f64,
        (q(3) - q(2)) as f64,
        (curve[n] - q(3)) as f64,
    ]
}

#[test]
fn exp_r06_r07_novelty_reach_and_curve_multiseed() {
    const EPOCHS: usize = 600;
    const SEEDS: [u64; 8] = [0x600D, 1, 2, 3, 7, 42, 99, 777];

    println!("\n=== EXP-R06/R07 multi-seed: kernel reach + novelty curve ===");
    println!("seed      shapes_end  rcf_total  mut/inst/quar   per-quartile new");

    let mut all_deltas: Vec<[f64; 4]> = Vec::new();
    for seed in SEEDS {
        let r = run_seed(seed, EPOCHS);
        let d = quartile_deltas(&r.curve);
        println!(
            "{seed:#07x}  {:>10}  {:>9}  {:>3}/{:>3}/{:>3}     [{:.0} {:.0} {:.0} {:.0}]",
            r.curve[EPOCHS],
            r.rcf_total,
            r.mutated,
            r.installed,
            r.quarantined,
            d[0],
            d[1],
            d[2],
            d[3]
        );
        all_deltas.push(d);
    }

    // Aggregate: mean last-quartile yield vs mean first-quartile yield.
    let n = all_deltas.len() as f64;
    let mean_q1: f64 = all_deltas.iter().map(|d| d[0]).sum::<f64>() / n;
    let mean_q4: f64 = all_deltas.iter().map(|d| d[3]).sum::<f64>() / n;
    let accelerating_seeds = all_deltas.iter().filter(|d| d[3] > d[0] * 1.5).count();
    let saturating_seeds = all_deltas.iter().filter(|d| d[3] < d[0] * 0.5).count();

    println!("\nmean q1 yield {mean_q1:.1}, mean q4 yield {mean_q4:.1}");
    println!(
        "seeds accelerating (q4 > 1.5*q1): {accelerating_seeds}/{}",
        SEEDS.len()
    );
    println!(
        "seeds saturating  (q4 < 0.5*q1): {saturating_seeds}/{}",
        SEEDS.len()
    );

    // Instrumentation sanity only — the SHAPE verdict is FM-R6's:
    assert!(
        all_deltas.iter().all(|d| d.iter().all(|x| x.is_finite())),
        "non-finite quartile delta"
    );

    // Verdict line (recorded, not gated):
    let verdict = if accelerating_seeds > SEEDS.len() / 2 {
        "ACCELERATING"
    } else if saturating_seeds > SEEDS.len() / 2 {
        "SATURATING"
    } else {
        "~LINEAR (Hash Chemistry reproduces — FM-R6 interpretation)"
    };
    println!("VERDICT         : {verdict}");
}
