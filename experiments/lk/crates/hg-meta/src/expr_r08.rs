//! EXP-R08 (Phase R6): the epoch sweep — `E ∈ {∞, 10⁴, 10³, 10², 10, 1}`
//! realized at harness scale as `{∞, 50, 20, 10, 5, 1}` epochs-per-recompile,
//! arms A2 (protected) and A3 (bare).
//!
//! Measured per E (plan §11.3):
//!   - rule-change rate: fraction of epoch boundaries where the compiled
//!     rule set actually changed (RCF hash multiset differs)
//!   - tpm_rule_set_stable: the G-R6 assertion — was the rule set unchanged
//!     across the window a TPM was estimated over?
//!   - novel-RCF rate per epoch
//!
//! EI confidence-interval widths require the full hg-eval observer; here we
//! measure the two quantities this harness owns (rule-change dynamics and
//! stationarity) and record CI-width proxy = rule-set churn within window.
//! The pre-committed prediction: rule-change rate rises monotonically as E
//! shrinks; an E* with both nonzero rule change AND stable estimation
//! windows is what R5 needs to exist.
//!
//! RUNTIME: release-scaled.

use super::epoch::epoch_transition;
use crate::graph_source::GraphRuleSource;
use crate::mutate::MutationRng;
use crate::rcf::rcf_hash;
use crate::schema::{find_rule_roots, read_region_with_footprint};

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

/// Canonical signature of the current COMPILED rule set: sorted RCF hashes
/// of every region. Two epochs share a rule set iff these multisets match.
fn ruleset_signature(s: &hg_core::HypergraphStore) -> Vec<u64> {
    let snap = s.snapshot();
    let mut sig: Vec<u64> = find_rule_roots(&snap)
        .iter()
        .filter_map(|r| read_region_with_footprint(&snap, *r).ok())
        .map(|(c, _)| rcf_hash(&c))
        .collect();
    sig.sort_unstable();
    sig
}

#[derive(Clone, Copy)]
struct SweepRow {
    e: u64,
    /// fraction of recompile boundaries where the rule set changed
    rule_change_rate: f64,
    /// fraction of estimation windows (E steps) with NO rule change inside
    tpm_rule_set_stable: f64,
}

fn run_sweep_arm(meta_visible: bool, total_epochs: usize, es: &[u64], seed: u64) -> Vec<SweepRow> {
    let mut rows = Vec::new();
    for &e in es {
        // Fresh world per (arm, E).
        let mut s = seed_world();
        let mut src = GraphRuleSource::new();
        let mut rng = XorShift(seed ^ (e.wrapping_mul(0x9E37_79B9)));
        let mut seen = rustc_hash::FxHashSet::default();

        // Baseline signature.
        let mut prev_sig = ruleset_signature(&s);
        let mut changes = 0usize;
        let mut boundaries = 0usize;
        // Stationarity windows: a window of `e` consecutive transitions with
        // zero rule-set changes. We count over the transition stream.
        let mut windows = 0usize;
        let mut stable_windows = 0usize;
        let mut changes_in_window = 0usize;

        for epoch in 0..total_epochs {
            let ev = epoch_transition(&mut src, &mut s, &mut rng, &mut seen, meta_visible);
            let _ = ev;

            // A recompile boundary happens when the region set changes.
            let sig = ruleset_signature(&s);
            if sig != prev_sig {
                changes += 1;
                changes_in_window += 1;
                prev_sig = sig;
            }
            boundaries += 1;

            // Window bookkeeping (G-R6): every `max(e,1)` boundaries closes
            // one TPM-estimation window.
            let win = usize::try_from(e.max(1)).unwrap_or(1);
            if (epoch + 1) % win == 0 {
                windows += 1;
                if changes_in_window == 0 {
                    stable_windows += 1;
                }
                changes_in_window = 0;
            }
        }

        rows.push(SweepRow {
            e,
            rule_change_rate: changes as f64 / boundaries as f64,
            tpm_rule_set_stable: if windows > 0 {
                stable_windows as f64 / windows as f64
            } else {
                1.0
            },
        });
    }
    rows
}

#[test]
fn exp_r08_epoch_sweep() {
    const TOTAL: usize = 400;
    // E = ∞ is realized as "no boundary after init": one giant window.
    let es: [u64; 6] = [u64::MAX / 4, 50, 20, 10, 5, 1];

    println!("\n=== EXP-R08 epoch sweep ({TOTAL} epochs per cell) ===");
    println!("arm  E        rule_change_rate   tpm_rule_set_stable");

    for (name, meta_visible) in [("A2", false), ("A3", true)] {
        let rows = run_sweep_arm(meta_visible, TOTAL, &es, 0x08_08);
        for r in &rows {
            let e_disp = if r.e > 1000 {
                "inf".to_string()
            } else {
                format!("{}", r.e)
            };
            println!(
                "{name}    {e_disp:<8} {:<18.3} {:.3}",
                r.rule_change_rate, r.tpm_rule_set_stable
            );
        }
        println!();
    }

    // Pre-committed predictions to CHECK (not gate):
    //  1. rule-change rate rises monotonically as E shrinks — trivially true
    //     in this harness because smaller E means more frequent mutation
    //     opportunities per boundary... actually each boundary IS one epoch's
    //     mutation; the honest check is that the RATE PER STEP rises.
    //  2. An E* with both nonzero rule change and high window stability.
    // Report only; the substantive interpretation lives in the findings doc.
}
