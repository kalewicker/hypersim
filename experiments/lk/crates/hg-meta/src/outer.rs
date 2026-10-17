//! EXP-R09: the ERE outer loop under F5′ (ERE plan §10, §12).
//!
//! Pre-registration: `docs/exp_r09_f5prime_findings.md` (written before any
//! number below existed). Name-collision note: `expr_r09.rs` + its findings
//! doc are the GE-track fee-gated loop; THIS module is the ERE-plan EXP-R09
//! (evolutionary outer loop, G_θ under F5′).
//!
//! G_θ: θ ∈ R^13, five softmax groups emitting E1-expressible seed rule
//! regions over the substrate alphabet {0,1,2,3}. Inner loop: A2-protected,
//! full 10-op kernel, no fee economy (ERE plan v1 has none — kept pure).
//! F5′: firing-mass concentration over NonTrivial-gated RCF classes.
//!
//! Suite split: cheap unit tests + P1′/P2′ (analytic, no engine) run in the
//! default suite. The eval-backed battery (P3′/P4′, campaign, controls —
//! 44 inner evals) is `#[ignore]`d for explicit release runs:
//! `cargo test --release -p hg-meta --lib outer -- --ignored --nocapture`.
//! Debug pace would bury the fast suite; precedent for slow expr drivers
//! is release-scaling (expr_r07 notes), this one just makes it explicit.

use crate::epoch::{epoch_transition, SharedFireCounter};
use crate::graph_source::GraphRuleSource;
use crate::mutate::MutationRng;
use crate::protect::{compile_delta, sync_label_mirror};
use crate::rcf::rcf_hash;
use crate::schema::{
    find_rule_roots, read_region_with_footprint, CandidatePatternEdge, CandidateRule,
};
use crate::validate::{validate, PriorityObservation, Verdict};
use hg_engine::Engine;
use hg_rules::compiled::compile_rules;
use hg_rules::source::RuleSource;
use hg_rules::StepMode;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use rustc_hash::FxHashMap;

// ---------------------------------------------------------------------------
// G_θ — harness-scale generative grammar
// ---------------------------------------------------------------------------

/// Production groups: (offset, width, choices-as-f64).
const G_NSEEDS: (usize, usize) = (0, 3); // n_seeds ∈ {1,2,3}
const G_NLHS: (usize, usize) = (3, 2); // n_lhs_edges ∈ {1,2}
const G_NRHS: (usize, usize) = (5, 2); // n_rhs_edges ∈ {1,2}
const G_NVARS: (usize, usize) = (7, 2); // n_vars ∈ {2,3}
const G_LABEL: (usize, usize) = (9, 4); // label per var ∈ {0,1,2,3}
pub const THETA_DIM: usize = 13;

const NSEEDS: [usize; 3] = [1, 2, 3];
const NLHS: [usize; 2] = [1, 2];
const NRHS: [usize; 2] = [1, 2];
const NVARS: [usize; 2] = [2, 3];
const LABELS: [u32; 4] = [0, 1, 2, 3];

/// Descoped OFF θ (stated in pre-reg): arity fixed 2 — the only arity the
/// substrate speaks, so a 3-slot pattern edge could never match (dead
/// production, would read as selection signal). Guards/var_guards/effects/
/// priority likewise fixed off/zero.
const ARITY: usize = 2;
const MAX_TRIES_PER_SEED: usize = 16;
const FALLBACK_P1_ABORT_FRAC: f64 = 0.5;

fn softmax(logits: &[f32]) -> Vec<f64> {
    let m = logits.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let ex: Vec<f64> = logits.iter().map(|l| ((l - m) as f64).exp()).collect();
    let z: f64 = ex.iter().sum();
    ex.iter().map(|e| e / z).collect()
}

fn sample_idx<R: Rng>(rng: &mut R, probs: &[f64]) -> usize {
    let mut u = rng.gen::<f64>();
    for (i, p) in probs.iter().enumerate() {
        u -= p;
        if u <= 0.0 {
            return i;
        }
    }
    probs.len() - 1
}

/// Concatenated per-group probabilities — the induced allocation (kept as
/// the documented readout of what θ MEANS; traversal math uses group_tv).
#[allow(dead_code)]
pub fn allocation(theta: &[f32]) -> Vec<f64> {
    [G_NSEEDS, G_NLHS, G_NRHS, G_NVARS, G_LABEL]
        .iter()
        .flat_map(|(o, w)| softmax(&theta[*o..*o + *w]))
        .collect()
}

/// Mean per-group total variation (groups are separate distributions;
/// a flat concat-TV would overweight the 4-wide label group).
pub fn group_tv(a: &[f32], b: &[f32]) -> f64 {
    let mut tot = 0.0;
    for (o, w) in [G_NSEEDS, G_NLHS, G_NRHS, G_NVARS, G_LABEL] {
        let pa = softmax(&a[o..o + w]);
        let pb = softmax(&b[o..o + w]);
        tot += 0.5
            * pa.iter()
                .zip(pb.iter())
                .map(|(x, y)| (x - y).abs())
                .sum::<f64>();
    }
    tot / 5.0
}

fn pe(slots: Vec<usize>) -> CandidatePatternEdge {
    CandidatePatternEdge { slots }
}

/// The fallback template (GE `seed_world_n` rule 1 — splitter, known valid
/// and NonTrivial; the copy rule next to it is the identity trap F5′ gates
/// out, so it must NOT be the fallback).
fn fallback_candidate() -> CandidateRule {
    CandidateRule {
        priority: 0,
        lhs: vec![pe(vec![0, 1])],
        rhs: vec![pe(vec![0, 2]), pe(vec![2, 1])],
        var_labels: vec![Some(0), Some(1), Some(2)],
        guards: Vec::new(),
        var_guards: Vec::new(),
        effects: Vec::new(),
    }
}

fn sample_candidate<R: Rng>(rng: &mut R, theta: &[f32]) -> CandidateRule {
    let nl = NLHS[sample_idx(rng, &softmax(&theta[G_NLHS.0..G_NLHS.0 + G_NLHS.1]))];
    let nr = NRHS[sample_idx(rng, &softmax(&theta[G_NRHS.0..G_NRHS.0 + G_NRHS.1]))];
    let nv = NVARS[sample_idx(rng, &softmax(&theta[G_NVARS.0..G_NVARS.0 + G_NVARS.1]))];
    let lp = softmax(&theta[G_LABEL.0..G_LABEL.0 + G_LABEL.1]);
    let var_labels: Vec<Option<u32>> = (0..nv)
        .map(|_| Some(LABELS[sample_idx(rng, &lp)]))
        .collect();
    let mut edge = || pe((0..ARITY).map(|_| rng.gen_range(0..nv)).collect());
    CandidateRule {
        priority: 0,
        lhs: (0..nl).map(|_| edge()).collect(),
        rhs: (0..nr).map(|_| edge()).collect(),
        var_labels,
        guards: Vec::new(),
        var_guards: Vec::new(),
        effects: Vec::new(),
    }
}

/// FM-R4 gate at the candidate level: an exact identity (lhs ≡ rhs as
/// shape multisets) rewrites nothing and would farm firing mass as a no-op
/// (the GE seed-rule-2 copy trap, 4002 free firings).
pub fn nontrivial(cand: &CandidateRule) -> bool {
    // SB-R5 corollary: a both-sides var's label is a match constraint, not
    // a write — sides sharing the same vars can never differ in labels, so
    // same-shape-multiset ⟺ identity, and the comparison below is exact
    // (inner slots AND outer edge multiset both sorted: the engine matches
    // endpoints as unordered multisets, SB-R4).
    let mut l: Vec<Vec<usize>> = cand
        .lhs
        .iter()
        .map(|e| {
            let mut s = e.slots.clone();
            s.sort();
            s
        })
        .collect();
    let mut r: Vec<Vec<usize>> = cand
        .rhs
        .iter()
        .map(|e| {
            let mut s = e.slots.clone();
            s.sort();
            s
        })
        .collect();
    l.sort();
    r.sort();
    l != r
}

/// Sample one VALID seed (scratch-install → read-back → validate, so seeds
/// are reader-canonical by construction). Returns the candidate plus
/// whether the fallback fired.
pub fn valid_seed<R: Rng>(rng: &mut R, theta: &[f32]) -> (CandidateRule, bool) {
    for _ in 0..MAX_TRIES_PER_SEED {
        let cand = sample_candidate(rng, theta);
        let mut scratch = hg_core::HypergraphStore::new();
        GraphRuleSource::install_candidates(&mut scratch, &[cand]);
        let snap = scratch.snapshot();
        let roots = find_rule_roots(&snap);
        if roots.len() != 1 {
            continue;
        }
        let Ok((back, fp)) = read_region_with_footprint(&snap, roots[0]) else {
            continue;
        };
        if matches!(
            validate(&back, &fp, &PriorityObservation { malformed: false }),
            Verdict::Valid
        ) {
            return (back, false);
        }
    }
    (fallback_candidate(), true)
}

pub fn sample_n_seeds<R: Rng>(rng: &mut R, theta: &[f32]) -> (Vec<CandidateRule>, usize) {
    let n = NSEEDS[sample_idx(rng, &softmax(&theta[G_NSEEDS.0..G_NSEEDS.0 + G_NSEEDS.1]))];
    let mut out = Vec::with_capacity(n);
    let mut fallbacks = 0;
    for _ in 0..n {
        let (c, fb) = valid_seed(rng, theta);
        out.push(c);
        fallbacks += usize::from(fb);
    }
    (out, fallbacks)
}

// ---------------------------------------------------------------------------
// Substrate — `expr_r09::seed_world_n` substrate half, duplicated: that
// helper is private to its module (the file's own shape_key precedent).
// Node labels i%4 ∈ {0,1,2,3}, edge labels w%3 ∈ {0,1,2}, all arity 2.
// ---------------------------------------------------------------------------

fn seed_substrate_n(n: usize) -> hg_core::HypergraphStore {
    let n = n.max(3);
    let mut s = hg_core::HypergraphStore::new();
    for i in 0..n as u32 {
        let mut d = hg_core::NodeData::new(hg_core::Label(i % 4), 0);
        d.set_property(1, hg_core::PropertyValue::Float(f64::from(i) * 0.1));
        s.add_node(d);
    }
    let ids: Vec<_> = s.nodes().map(|(id, _)| id).collect();
    for w in 0..n.saturating_sub(2) {
        let a = ids[w % ids.len()];
        let b = ids[(w * 3 + 1) % ids.len()];
        if a != b {
            s.add_edge(hg_core::HyperEdge::new(
                hg_core::Label((w % 3) as u32),
                smallvec::smallvec![a, b],
            ));
        }
    }
    s
}

// ---------------------------------------------------------------------------
// Inner eval — A2-protected, full kernel, no economy, F5′ out
// ---------------------------------------------------------------------------

pub const INNER_EPOCHS: usize = 30;
pub const INNER_STEPS: usize = 10;
pub const BURN_IN: usize = 3;
/// M1-Pro cap: total region nodes (FM-R3). Over-cap epochs skip the install
/// and log bound — never silently absorb.
pub const REGION_NODE_CAP: usize = 384;
pub const SEED_NODES: usize = 24;

pub struct EvalOutcome {
    pub f5prime: f64,
    pub top_share: f64,
    pub scored_fires: u64,
    pub n_classes: usize,
    pub bound_epochs: usize,
    pub extinct_at: Option<usize>,
    pub fallbacks: usize,
    pub dropped: u64,
}

struct SplitMix(u64);
impl MutationRng for SplitMix {
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }
}

pub fn run_inner(theta: &[f32], eval_seed: u64) -> EvalOutcome {
    let mut rng = ChaCha8Rng::seed_from_u64(eval_seed);
    let mut mut_rng = SplitMix(eval_seed ^ 0x51F5_0001);
    let mut s = seed_substrate_n(SEED_NODES);
    let (seeds, fallbacks) = sample_n_seeds(&mut rng, theta);
    GraphRuleSource::install_candidates(&mut s, &seeds);
    let mut src = GraphRuleSource::new();
    let mut seen = rustc_hash::FxHashSet::default();
    let snap = s.snapshot();
    let _ = src.rules(&snap, 0);

    let mut class_fires: FxHashMap<u64, u64> = FxHashMap::default();
    let mut bound_epochs = 0usize;
    let mut extinct_at = None;
    let mut dropped = 0u64;

    for epoch in 0..INNER_EPOCHS {
        // 1. Consult.
        let snap = s.snapshot();
        let mut pairs: Vec<(hg_core::NodeId, CandidateRule, u64, bool)> = Vec::new();
        let mut region_nodes = 0usize;
        for root in find_rule_roots(&snap) {
            let Ok((cand, fp)) = read_region_with_footprint(&snap, root) else {
                continue;
            };
            region_nodes += fp.nodes;
            if matches!(
                validate(&cand, &fp, &PriorityObservation { malformed: false }),
                Verdict::Valid
            ) {
                let h = rcf_hash(&cand);
                let nt = nontrivial(&cand);
                pairs.push((root, cand, h, nt));
            }
        }
        if pairs.is_empty() {
            extinct_at = Some(epoch);
            break;
        }
        let roots: Vec<hg_core::NodeId> = pairs.iter().map(|(r, _, _, _)| *r).collect();
        let cands: Vec<CandidateRule> = pairs.iter().map(|(_, c, _, _)| c.clone()).collect();

        // 2. Protected table + step window with fire counter.
        let table = compile_rules(&compile_delta(&cands, false)).expect("valid regions compile");
        let mut engine = Engine::new(table, StepMode::MaxParallel, 42, 10_000);
        let counter = SharedFireCounter::new();
        engine.add_observer(Box::new(counter.clone()));
        sync_label_mirror(&mut s);
        for _ in 0..INNER_STEPS {
            let _ = engine.step(&mut s);
            sync_label_mirror(&mut s);
        }

        // 3. Attribute rule_id → root (consultation order) → RCF class.
        // Nonzero dropped is a DRIVER bug (stale table), loud here.
        let counts = counter.0.lock().expect("uncontended").drain();
        for (rid, n) in &counts {
            match roots.get(*rid) {
                Some(root) => {
                    if epoch >= BURN_IN {
                        if let Some((_, _, h, nt)) = pairs.iter().find(|(r, _, _, _)| r == root) {
                            if *nt {
                                *class_fires.entry(*h).or_default() += *n;
                            }
                        }
                    }
                }
                None => dropped += *n,
            }
        }

        // 4. Endogenous transition (FM-R3 cap: skip install when bound).
        if region_nodes > REGION_NODE_CAP {
            bound_epochs += 1;
        } else {
            epoch_transition(&mut src, &mut s, &mut mut_rng, &mut seen, false);
        }
    }

    let total: u64 = class_fires.values().sum();
    let (f5prime, top_share) = if total == 0 {
        (0.0, 0.0)
    } else {
        let sq: u64 = class_fires.values().map(|c| c * c).sum();
        let top = *class_fires.values().max().unwrap_or(&0);
        (sq as f64 / total as f64, top as f64 / total as f64)
    };
    EvalOutcome {
        f5prime,
        top_share,
        scored_fires: total,
        n_classes: class_fires.len(),
        bound_epochs,
        extinct_at,
        fallbacks,
        dropped,
    }
}

// ---------------------------------------------------------------------------
// Optimizer — the `hg-train::optim::OpenAiEs` form, inlined: hg-train's
// dev-dependency weight (hg-sim-core → hg-engine and friends) is
// unjustified for ~40 lines. Behavior pinned by `es_climbs_quadratic`.
// ---------------------------------------------------------------------------

pub fn rank_shape(fits: &[f32]) -> Vec<f32> {
    let mut idx: Vec<usize> = (0..fits.len()).collect();
    idx.sort_by(|&a, &b| fits[a].total_cmp(&fits[b]).then_with(|| a.cmp(&b)));
    let n = idx.len();
    let mut out = vec![0.0f32; n];
    if n < 2 {
        return out;
    }
    for (rank, &i) in idx.iter().enumerate() {
        out[i] = rank as f32 / (n - 1) as f32 - 0.5;
    }
    out
}

pub fn es_update(
    theta: &mut [f32],
    eps: &[Vec<f32>],
    plus: &[f32],
    minus: &[f32],
    sigma: f32,
    alpha: f32,
) {
    assert_eq!(eps.len(), plus.len());
    assert_eq!(eps.len(), minus.len());
    let flat: Vec<f32> = plus
        .iter()
        .zip(minus.iter())
        .flat_map(|(p, m)| [*p, *m])
        .collect();
    let shaped = rank_shape(&flat);
    let scale = alpha / (eps.len() as f32 * sigma);
    for (p, e) in eps.iter().enumerate() {
        let d = shaped[2 * p] - shaped[2 * p + 1];
        if d == 0.0 {
            continue;
        }
        for (t, ei) in theta.iter_mut().zip(e) {
            *t += scale * d * ei;
        }
    }
}

pub fn standard_normal_vec<R: Rng>(rng: &mut R, n: usize) -> Vec<f32> {
    // Box–Muller from a uniform stream (hg-train draws N(0,1) the same way
    // in effect; exact stream need not match — only the update FORM is
    // shared, pinned by the quadratic test).
    (0..n)
        .map(|_| {
            let u1 = rng.gen::<f64>().max(1e-12);
            let u2 = rng.gen::<f64>();
            ((-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()) as f32
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Cheap unit tests — default suite
// ---------------------------------------------------------------------------

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn softmax_uniform_at_zero() {
        let p = softmax(&[0.0, 0.0, 0.0]);
        assert!(p.iter().all(|x| (x - 1.0 / 3.0).abs() < 1e-9));
    }

    #[test]
    fn group_tv_zero_at_same_theta_far_apart() {
        let z = vec![0.0f32; THETA_DIM];
        assert_eq!(group_tv(&z, &z), 0.0);
        let mut far = vec![0.0f32; THETA_DIM];
        far[0] = 2.5;
        far[2] = -2.5;
        let tv = group_tv(&z, &far);
        assert!(tv > 0.05, "far probe must clear the P1′ bar, got {tv}");
    }

    #[test]
    fn nontrivial_gate_identity_vs_splitter_vs_perm() {
        // Exact identity (GE seed-rule-2 form): trivial.
        let ident = CandidateRule {
            priority: 0,
            lhs: vec![pe(vec![0])],
            rhs: vec![pe(vec![0])],
            var_labels: vec![None],
            guards: Vec::new(),
            var_guards: Vec::new(),
            effects: Vec::new(),
        };
        assert!(!nontrivial(&ident));
        // Splitter: nontrivial.
        assert!(nontrivial(&fallback_candidate()));
        // Slot-permuted copy: SB-R4 says permutation is void → trivial.
        let perm = CandidateRule {
            priority: 0,
            lhs: vec![pe(vec![0, 1])],
            rhs: vec![pe(vec![1, 0])],
            var_labels: vec![Some(0), Some(1)],
            guards: Vec::new(),
            var_guards: Vec::new(),
            effects: Vec::new(),
        };
        assert!(!nontrivial(&perm));
        // Edge-count change (deletion): nontrivial.
        let del = CandidateRule {
            priority: 0,
            lhs: vec![pe(vec![0, 1]), pe(vec![0, 1])],
            rhs: vec![pe(vec![0, 1])],
            var_labels: vec![Some(0), Some(1)],
            guards: Vec::new(),
            var_guards: Vec::new(),
            effects: Vec::new(),
        };
        assert!(nontrivial(&del));
    }

    #[test]
    fn fallback_template_is_valid() {
        let cand = fallback_candidate();
        let mut scratch = hg_core::HypergraphStore::new();
        GraphRuleSource::install_candidates(&mut scratch, &[cand]);
        let snap = scratch.snapshot();
        let roots = find_rule_roots(&snap);
        assert_eq!(roots.len(), 1);
        let (back, fp) = read_region_with_footprint(&snap, roots[0]).expect("reads");
        assert!(
            matches!(
                validate(&back, &fp, &PriorityObservation { malformed: false }),
                Verdict::Valid
            ),
            "fallback must validate — else every P1′ seed is suspect"
        );
    }

    #[test]
    fn es_climbs_quadratic() {
        // Synthetic: f(θ) = −‖θ − t‖². ES must move θ toward t.
        // Pins the inlined update FORM (not hg-train's stream).
        let target = [1.0f32, -0.5, 0.25, 0.0, 0.0, 0.0, 0.0];
        let d0 = target.iter().map(|x| x * x).sum::<f32>().sqrt();
        let mut theta = vec![0.0f32; 7];
        let mut rng = ChaCha8Rng::seed_from_u64(0xE5);
        let fit = |th: &[f32]| {
            -th.iter()
                .zip(target.iter())
                .map(|(a, b)| (a - b) * (a - b))
                .sum::<f32>()
        };
        for _ in 0..30 {
            let eps: Vec<Vec<f32>> = (0..6).map(|_| standard_normal_vec(&mut rng, 7)).collect();
            let (plus, minus): (Vec<f32>, Vec<f32>) = eps
                .iter()
                .map(|e| {
                    let p: Vec<f32> = theta.iter().zip(e).map(|(t, x)| t + 0.5 * x).collect();
                    let m: Vec<f32> = theta.iter().zip(e).map(|(t, x)| t - 0.5 * x).collect();
                    (fit(&p), fit(&m))
                })
                .unzip();
            es_update(&mut theta, &eps, &plus, &minus, 0.5, 0.1);
        }
        let d1 = theta
            .iter()
            .zip(target.iter())
            .map(|(a, b)| (a - b) * (a - b))
            .sum::<f32>()
            .sqrt();
        assert!(d1 < d0, "ES must approach the target: {d0:.3} → {d1:.3}");
    }

    #[test]
    fn sampled_seeds_validate_or_fall_back() {
        // Grammar health at θ=0: fallback fraction must clear P1′'s bar.
        let theta = vec![0.0f32; THETA_DIM];
        let mut rng = ChaCha8Rng::seed_from_u64(0xE901);
        let mut fb = 0usize;
        for _ in 0..24 {
            let (_, f) = valid_seed(&mut rng, &theta);
            fb += usize::from(f);
        }
        let frac = fb as f64 / 24.0;
        assert!(
            frac < FALLBACK_P1_ABORT_FRAC,
            "fallback fraction {frac:.2} ≥ P1′ abort bar — grammar unstateable"
        );
        println!("seed fallback fraction at θ=0: {fb}/24");
    }
}

// ---------------------------------------------------------------------------
// Prechecks P1′/P2′ (cheap, analytic — default suite) + eval battery
// (P3′/P4′, campaign, controls — #[ignore], explicit release runs)
// ---------------------------------------------------------------------------

/// The far-end probe: ±2.5 spread favoring {3 seeds, 1 lhs, 2 rhs, 2 vars,
/// labels 0/3}. What makes it "far" is the induced allocation TV, not the
/// norm (P-2 discipline: state the traversal in allocation units).
pub fn theta_far() -> Vec<f32> {
    vec![
        -2.5, 0.0, 2.5, // n_seeds → 3
        2.5, -2.5, // n_lhs → 1
        -2.5, 2.5, // n_rhs → 2
        2.5, -2.5, // n_vars → 2
        2.5, -2.5, -2.5, 2.5, // labels → 0/3
    ]
}

pub const P_TV_BAR: f64 = 0.05; // TRAVERSAL_TV_MIN mirror
pub const ES_SIGMA: f32 = 0.5;
pub const ES_ALPHA: f32 = 0.1;
pub const ES_GENS: usize = 3;
pub const ES_PAIRS: usize = 4;

#[cfg(test)]
mod precheck_tests {
    use super::*;

    #[test]
    fn exp09_p1_expressiveness() {
        // P1′: the class can express a difference (analytic TV) AND seeds
        // are real (fallback fraction below the abort bar).
        let zero = vec![0.0f32; THETA_DIM];
        let tv = group_tv(&zero, &theta_far());
        assert!(
            tv >= P_TV_BAR,
            "P1′ ABORT: far probe moves allocation by TV={tv:.4} < {P_TV_BAR} — the class is inexpressive, campaign VOID (finding, §10.4)"
        );
        let mut rng = ChaCha8Rng::seed_from_u64(0xE9B1);
        let mut fb = 0usize;
        for _ in 0..24 {
            let (_, f) = valid_seed(&mut rng, &zero);
            fb += usize::from(f);
        }
        let frac = fb as f64 / 24.0;
        assert!(
            frac < FALLBACK_P1_ABORT_FRAC,
            "P1′ ABORT: fallback fraction {frac:.2} — grammar mostly-invalid, campaign VOID (finding, §10.4)"
        );
        println!("P1′ PASS: TV={tv:.4} ≥ {P_TV_BAR}, fallback {fb}/24");
    }

    #[test]
    fn exp09_p2_traversal() {
        // P2′: σ-scale perturbations move the induced allocation.
        let zero = vec![0.0f32; THETA_DIM];
        let mut rng = ChaCha8Rng::seed_from_u64(0xE9A7);
        let mut tvs = Vec::new();
        for _ in 0..8 {
            let e = standard_normal_vec(&mut rng, THETA_DIM);
            let moved: Vec<f32> = zero
                .iter()
                .zip(e.iter())
                .map(|(t, x)| t + ES_SIGMA * x)
                .collect();
            tvs.push(group_tv(&zero, &moved));
        }
        let mean = tvs.iter().sum::<f64>() / tvs.len() as f64;
        assert!(
            mean >= P_TV_BAR,
            "P2′ ABORT: mean σ-traversal TV={mean:.4} < {P_TV_BAR} — optimizer cannot move the class, campaign VOID (finding, §10.4)"
        );
        println!("P2′ PASS: mean traversal TV={mean:.4} ≥ {P_TV_BAR}");
    }
}

// ---------------------------------------------------------------------------
// Eval battery — #[ignore], explicit release runs (44 inner evals total).
// `cargo test --release -p hg-meta --lib outer::battery -- --ignored --nocapture`
// ---------------------------------------------------------------------------

#[cfg(test)]
mod battery {
    use super::*;

    fn mean_se(xs: &[f64]) -> (f64, f64) {
        let n = xs.len() as f64;
        let m = xs.iter().sum::<f64>() / n;
        let v = xs.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / (n - 1.0).max(1.0);
        (m, (v / n).sqrt())
    }

    #[test]
    #[ignore]
    fn exp09_p34_discrimination_saturation() {
        println!("=== EXP-R09 P3′/P4′ probes (discrimination + saturation) ===");
        let mut rng = ChaCha8Rng::seed_from_u64(0xEA01);
        let probes: Vec<(&str, Vec<f32>)> = vec![
            ("zero", vec![0.0f32; THETA_DIM]),
            ("far", theta_far()),
            ("rand1", standard_normal_vec(&mut rng, THETA_DIM)),
            ("rand2", standard_normal_vec(&mut rng, THETA_DIM)),
        ];
        let eval_seeds = [0xEB01u64, 0xEB02];
        let mut fits = Vec::new();
        let mut bound_total = 0usize;
        let mut dropped_total = 0u64;
        let mut extinct_total = 0usize;
        for (name, th) in &probes {
            for s in eval_seeds {
                let o = run_inner(th, s);
                println!(
                    "  probe {name:6} seed {s:#X}: F5′={:.4} top={:.3} fires={} classes={} bound={} extinct={:?} fb={} dropped={}",
                    o.f5prime, o.top_share, o.scored_fires, o.n_classes,
                    o.bound_epochs, o.extinct_at, o.fallbacks, o.dropped
                );
                fits.push(o.f5prime as f32);
                bound_total += o.bound_epochs;
                dropped_total += o.dropped;
                extinct_total += usize::from(o.extinct_at.is_some());
            }
        }
        assert_eq!(
            dropped_total, 0,
            "DRIVER BUG: {dropped_total} attributed firings missed the consultation order (stale table)"
        );
        let (mn, mx) = fits
            .iter()
            .fold((f32::INFINITY, f32::NEG_INFINITY), |(a, b), &f| {
                (a.min(f), b.max(f))
            });
        assert!(
            mx - mn > 0.0 && mx > 0.0,
            "P3′ ABORT: F5′ range [{mn:.4}, {mx:.4}] — objective blind (all zero or all tied), campaign VOID (finding, §10.4)"
        );
        let total_epochs = (probes.len() * eval_seeds.len()) as f64 * INNER_EPOCHS as f64;
        let bound_frac = bound_total as f64 / total_epochs;
        assert!(
            bound_frac < 0.5,
            "P4′ ABORT: cap bound on {bound_frac:.2} of probe epochs — F5′ would measure the ceiling, campaign VOID (finding, §10.4)"
        );
        println!(
            "P3′ PASS: F5′ range [{mn:.4}, {mx:.4}]; P4′ PASS: bound {bound_frac:.3}, extinct {extinct_total}/8, dropped 0"
        );
    }

    #[test]
    #[ignore]
    fn exp09_campaign_and_controls() {
        println!(
            "=== EXP-R09 campaign: G={ES_GENS} × pairs={ES_PAIRS}, σ={ES_SIGMA}, α={ES_ALPHA} ==="
        );
        let mut theta = vec![0.0f32; THETA_DIM];
        let mut es_rng = ChaCha8Rng::seed_from_u64(0xEC00);
        // Best by pair-mean fitness (recomputed for the report, not trusted
        // across tests — no inter-test state).
        let mut best_theta = theta.clone();
        let mut best_fit = f32::NEG_INFINITY;
        let mut dropped_total = 0u64;
        for g in 0..ES_GENS {
            let eps: Vec<Vec<f32>> = (0..ES_PAIRS)
                .map(|_| standard_normal_vec(&mut es_rng, THETA_DIM))
                .collect();
            let mut plus = Vec::with_capacity(ES_PAIRS);
            let mut minus = Vec::with_capacity(ES_PAIRS);
            for (p, e) in eps.iter().enumerate() {
                let cand = |sign: f32| {
                    theta
                        .iter()
                        .zip(e.iter())
                        .map(|(t, x)| t + sign * ES_SIGMA * x)
                        .collect()
                };
                let (tp, tm): (Vec<f32>, Vec<f32>) = (cand(1.0), cand(-1.0));
                // ONE shared eval seed per pair (pre-reg budget: 8 evals/gen):
                // the seed-lottery component cancels in the antithetic
                // difference (hg-train form).
                let s0 = 0xEC10 + (g as u64) * 8 + (p as u64);
                let fp = run_inner(&tp, s0).f5prime as f32;
                let fm = run_inner(&tm, s0).f5prime as f32;
                plus.push(fp);
                minus.push(fm);
                if fp > best_fit {
                    best_fit = fp;
                    best_theta = tp;
                }
                if fm > best_fit {
                    best_fit = fm;
                    best_theta = tm;
                }
            }
            es_update(&mut theta, &eps, &plus, &minus, ES_SIGMA, ES_ALPHA);
            let pm: f32 = plus.iter().sum::<f32>() / plus.len() as f32;
            let mm: f32 = minus.iter().sum::<f32>() / minus.len() as f32;
            println!("  gen {g}: plus-mean {pm:.4} minus-mean {mm:.4} best {best_fit:.4}");
        }
        // Controls × 4 fresh seeds: champion / untrained / random-resample.
        let ctrl_seeds = [0xED01u64, 0xED02, 0xED03, 0xED04];
        let mut r_rng = ChaCha8Rng::seed_from_u64(0xED00);
        let (mut ch, mut un, mut ra) = (Vec::new(), Vec::new(), Vec::new());
        for s in ctrl_seeds {
            let o = run_inner(&best_theta, s);
            dropped_total += o.dropped;
            ch.push(o.f5prime);
            println!(
                "  champion seed {s:#X}: F5′={:.4} top={:.3} fires={} classes={} extinct={:?}",
                o.f5prime, o.top_share, o.scored_fires, o.n_classes, o.extinct_at
            );
            let o = run_inner(&[0.0f32; THETA_DIM], s);
            dropped_total += o.dropped;
            un.push(o.f5prime);
            println!(
                "  untrained seed {s:#X}: F5′={:.4} top={:.3} fires={} classes={} extinct={:?}",
                o.f5prime, o.top_share, o.scored_fires, o.n_classes, o.extinct_at
            );
            let rt: Vec<f32> = standard_normal_vec(&mut r_rng, THETA_DIM)
                .iter()
                .map(|x| x / (THETA_DIM as f32).sqrt())
                .collect();
            let o = run_inner(&rt, s);
            dropped_total += o.dropped;
            ra.push(o.f5prime);
            println!(
                "  random   seed {s:#X}: F5′={:.4} top={:.3} fires={} classes={} extinct={:?}",
                o.f5prime, o.top_share, o.scored_fires, o.n_classes, o.extinct_at
            );
        }
        assert_eq!(dropped_total, 0, "DRIVER BUG: stale-table attribution");
        for v in ch.iter().chain(un.iter()).chain(ra.iter()) {
            assert!(v.is_finite(), "F5′ must be finite");
        }
        let (cm, cse) = mean_se(&ch);
        let (um, use_) = mean_se(&un);
        let (rm, rse) = mean_se(&ra);
        let diff = cm - um;
        let dse = (cse * cse + use_ * use_).sqrt();
        println!("=== EXP-R09 verdict ===");
        println!("  champion  mean {cm:.4} ± {cse:.4}");
        println!("  untrained mean {um:.4} ± {use_:.4}");
        println!("  random    mean {rm:.4} ± {rse:.4}");
        println!("  champion − untrained = {diff:.4} ± {dse:.4}");
    }
}
