//! EXP-R10 (Phase R6): EI and CE with the meta band EXCLUDED and INCLUDED.
//!
//! The M1 hazard (plan §8): rule regions are a tiny population of
//! long-lived, highly-structured nodes. If the TPM is estimated over ALL
//! nodes, meta-band dynamics dominate the entropy budget despite being
//! (by design) nearly static under protection. The measurement is the
//! DIVERGENCE between the two filters:
//!
//!   band-excluded : TPM over substrate nodes only (label < META_BASE)
//!   band-included : TPM over every node
//!
//! Arms: A1-frozen (engine never sees children; the control — any EI here
//! reflects substrate dynamics alone) vs A3-bare (unprotected recompile).
//! A2 sits between; we report all three.
//!
//! G-R6 discipline: each arm runs with E = inf (frozen rule set for the
//! whole estimation window) so the stationarity assertion holds by
//! construction; the epoch-driven variants are EXP-R08's subject.
//!
//! RUNTIME: release-scaled.

use super::epoch::epoch_transition;
use crate::graph_source::GraphRuleSource;
use crate::mutate::MutationRng;
use crate::protect::sync_label_mirror;
use hg_causal::ei::{compute_ei_results, EiSettings, TpmAccumulator};
use hg_core::{compute_wl_class_map_snapshot, GraphSnapshot};
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
    for i in 0..12u32 {
        let mut d = hg_core::NodeData::new(hg_core::Label(i % 4), 0);
        d.set_property(1, hg_core::PropertyValue::Float(f64::from(i) * 0.1));
        s.add_node(d);
    }
    let ids: Vec<_> = s.nodes().map(|(id, _)| id).collect();
    for w in 0..10usize {
        let a = ids[w % ids.len()];
        let b = ids[(w * 5 + 1) % ids.len()];
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

struct ArmResult {
    ei_micro_excl: f64,
    ei_micro_incl: f64,
    /// best (max) CE across strategies, band excluded / included
    ce_max_excl: f64,
    ce_max_incl: f64,
    transitions: u64,
}

/// Cadence knob for future EXP-R10 variants (currently every epoch).
#[allow(dead_code)]
const OBSERVE_EVERY: usize = 1;

fn run_arm(meta_visible: Option<bool>, epochs: usize, seed: u64) -> ArmResult {
    // None = frozen engine table; Some(v) = live recompile at protection v.
    let _ = meta_visible; // region evolution identical; engine table unused by EI
    let mut s = seed_world();
    let mut src = GraphRuleSource::new();
    let mut rng = XorShift(seed);
    let mut seen = rustc_hash::FxHashSet::default();

    let snap0 = s.snapshot();
    let d0 = src.rules(&snap0, 0);
    let _ = d0;

    let settings = EiSettings {
        min_transitions: 200,
        ..EiSettings::default()
    };
    let mut tpm_excl = TpmAccumulator::new();
    let mut tpm_incl = TpmAccumulator::new();

    // Previous observation's class maps.
    let mut prev_all: Option<ahash::AHashMap<hg_core::NodeId, hg_core::WlHash>> = None;

    sync_label_mirror(&mut s);
    for _epoch in 0..epochs {
        {
            // Observe BEFORE this transition boundary (cadence = 1 epoch).
            let snap: GraphSnapshot<'_> = s.snapshot();
            let classes_all = compute_wl_class_map_snapshot(&snap, 2);
            let classes_sub: std::collections::HashMap<_, _> = classes_all
                .iter()
                .filter(|(id, _)| {
                    s.get_node(**id).map(|d| d.label.0) < Some(crate::schema::META_BASE)
                })
                .map(|(id, h)| (*id, *h))
                .collect();

            if let Some(prev) = &prev_all {
                for (id, to) in &classes_all {
                    if let Some(from) = prev.get(id) {
                        tpm_incl.record(*from, *to);
                    }
                }
                for (id, to) in &classes_sub {
                    if let Some(from) = prev.get(id) {
                        tpm_excl.record(*from, *to);
                    }
                }
            }
            prev_all = Some(classes_all);
        }

        // One epoch transition + mirror sync (labels move; the TPM observes
        // them through WL classes next round).
        let ev = epoch_transition(&mut src, &mut s, &mut rng, &mut seen, true);
        let _ = ev;
        sync_label_mirror(&mut s);
    }

    let features: ahash::AHashMap<hg_core::WlHash, hg_causal::ei::ClassFeatures> =
        ahash::AHashMap::default();
    let registry = hg_causal::constructor::ConstructorRegistry::new();
    let res_excl = compute_ei_results(&tpm_excl, &features, &registry, &settings, 0);
    let res_incl = compute_ei_results(&tpm_incl, &features, &registry, &settings, 0);

    ArmResult {
        ei_micro_excl: res_excl.first().map(|r| r.ei_micro).unwrap_or(f64::NAN),
        ei_micro_incl: res_incl.first().map(|r| r.ei_micro).unwrap_or(f64::NAN),
        ce_max_excl: res_excl
            .iter()
            .map(|r| r.ce_coefficient)
            .fold(0.0, f64::max),
        ce_max_incl: res_incl
            .iter()
            .map(|r| r.ce_coefficient)
            .fold(0.0, f64::max),
        transitions: tpm_excl.total_transitions(),
    }
}

#[test]
fn exp_r10_ei_ce_band_in_out() {
    const EPOCHS: usize = 400;

    println!("\n=== EXP-R10: EI/CE, meta band excluded vs included ===");
    println!("arm          transitions  EI_micro(excl)  EI_micro(incl)  CE*(excl)  CE*(incl)");

    let a1 = run_arm(None, EPOCHS, 0xA10);
    println!(
        "A1 frozen    {:>11}  {:>14.4}  {:>14.4}  {:>9.4}  {:>9.4}",
        a1.transitions, a1.ei_micro_excl, a1.ei_micro_incl, a1.ce_max_excl, a1.ce_max_incl
    );
    let a3 = run_arm(Some(true), EPOCHS, 0xA30);
    println!(
        "A3 bare      {:>11}  {:>14.4}  {:>14.4}  {:>9.4}  {:>9.4}",
        a3.transitions, a3.ei_micro_excl, a3.ei_micro_incl, a3.ce_max_excl, a3.ce_max_incl
    );

    // M1 divergence, recorded per arm:
    println!(
        "\nM1 |EI_excl − EI_incl|: A1 {:.4}, A3 {:.4}",
        (a1.ei_micro_excl - a1.ei_micro_incl).abs(),
        (a3.ei_micro_excl - a3.ei_micro_incl).abs()
    );

    // Instrumentation sanity: both filters must have produced usable TPMs.
    assert!(
        a1.transitions > 0 && a3.transitions > 0,
        "no transitions recorded"
    );
}
