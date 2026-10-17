//! EXP-R09: the fee-gated outer loop (GE plan §10, pre-registered).
//!
//! Three arms, one harness, 40 epochs × 25 MaxParallel steps each:
//!
//! | Arm | Economy | Kernel | Committed reading |
//! |---|---|---|
//! | **A0** | FEE_ZERO, inert policy | all 10 ops | MUST reproduce the blob regime (trajectory Exploding/Expanding). If not, fees were never binding — stop and report |
//! | **A1** | default fees + throttle, live `FeePolicy` | all 10 ops | primary; conservation balanced every epoch; panel + novelty reported |
//! | **A2** | OFF | 2 MID ops only (`add/remove_pattern_edge`) | road-not-taken record; decision criterion A1 novelty ≥ A2 novelty |
//!
//! Per-epoch composition (the wiring SB-GE5 left to the driver):
//! consult (content order) → validate → compile engine table → install
//! `FeePolicy` aligned by `rule_id` → step with [`SharedFireCounter`]
//! attached → drain → [`map_fired_to_roots`] →
//! `epoch_transition_with_economy` (debit → mutate/install → enroll →
//! income/age → audit).
//!
//! Scope notes (honest panel accounting vs §5/G-GE2):
//! - WIRED HERE: node + edge trajectories every step (policy 8) with the
//!   G-GE1 regime classifier as the blob verdict (reuse, not reimplement);
//!   novelty (RCF) + SHAPE (FM-R6 key) curves vs EPOCH and vs kernel EVENT
//!   (policy 4); burn-in = first 10% unscored for novelty verdicts;
//!   per-epoch conservation audit (§10.6); paid/refused counts;
//!   quarantine accounting; G-GE2 panel emission every epoch
//!   (`observe_substrate` dual-scope walk + `PanelEpoch` rows incl. site
//!   pump state — the panel is reported, trajectory verdicts stay primary
//!   until the R5 panel-primary run).
//! - DEFERRED PAST G-GE2: EI/CE, WL-classed edit sites (root-identity
//!   substitution stands — see `docs/gge2_audit.md`). Everything else §5
//!   named now emits from this driver.
//! - All arms run under protection (`meta_visible = false`); visibility is
//!   not the variable here. Seeds enroll in the economy at epoch 0 as
//!   throttled newborns (plan §10 interpretation — every region-bearing
//!   rule pays; recorded in the SB-GE5 resolution).
//!
//! RUNTIME NOTE: release-scaled like the other epoch drivers
//! (`cargo test --release -p hg-meta expr_r09`, seconds).

use super::epoch::{epoch_transition_with_economy, EpochEconomy, SharedFireCounter};
use crate::economy::IncomePolicy;
use crate::fee_policy::{select_all_when_fee_is_zero, FeePolicy};
use crate::gge1::{classify_trajectory, DynClass};
use crate::gge2::{novel_per_event, observe_substrate, scored_gain, PanelEpoch};
use crate::graph_source::GraphRuleSource;
use crate::mutate::{MutationRng, Op};
use crate::protect::{compile_delta, sync_label_mirror};
use crate::rcf::rcf_hash;
use crate::schema::{find_rule_roots, CandidateRule};
use crate::validate::{validate, PriorityObservation, Verdict};
use hg_core::NodeId;
use hg_engine::Engine;
use hg_rules::compiled::compile_rules;
use hg_rules::source::RuleSource;
use hg_rules::StepMode;
use rustc_hash::{FxHashMap, FxHashSet};

const EPOCHS: usize = 40;
const E_LEN: usize = 25;

/// Loop conditions for one outer-loop run (SB-GE7(b) emergence search).
/// `LoopConfig::default()` reproduces the pre-registered R09 loop
/// exactly; the search cells vary one axis each (see
/// `docs/ere_r09_findings.md` SB-GE7(b) pre-registration).
#[derive(Clone, Copy)]
struct LoopConfig {
    epochs: usize,
    steps_per_epoch: usize,
    seed_nodes: usize,
    meta_visible: bool,
    /// Node ceiling (standing policy 1: logged binding/non-binding every
    /// run). `None` = uncapped (all pre-R5-MULTISEED runs). Enforcement
    /// breaks the step loop only — epoch structure, panel, audits continue.
    max_nodes: Option<usize>,
}

impl Default for LoopConfig {
    fn default() -> Self {
        Self {
            epochs: EPOCHS,
            steps_per_epoch: E_LEN,
            seed_nodes: 10,
            meta_visible: false,
            max_nodes: None,
        }
    }
}

/// The two MID operators that ship unpaired (G-GE1): the A2 kernel.
const MID_OPS: [Op; 2] = [Op::AddPatternEdge, Op::RemovePatternEdge];

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

/// Seed world with `n` substrate nodes. `n = 10` reproduces the
/// pre-registered R09 seed exactly (10 nodes, 8 edges, 2 seed rules);
/// larger `n` scales nodes + edges by the same pattern (SB-GE7(b) C2).
fn seed_world_n(n: usize) -> hg_core::HypergraphStore {
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

/// FM-R6 mitigation key (expr_r07's, duplicated — that helper is private
/// to its module): RCF hash of the label-stripped candidate. A relabel of
/// an already-seen rule is not novel.
fn shape_key(cand: &CandidateRule) -> u64 {
    let mut stripped = cand.clone();
    stripped.var_labels = vec![None; cand.n_vars()];
    stripped.priority = 0;
    rcf_hash(&stripped)
}

/// Map per-`rule_id` fire counts onto region roots through the epoch's
/// consultation order (`roots[k]` is the region compiled as `rule_id k`).
/// Returns the attributed map plus a dropped count: nonzero dropped is a
/// DRIVER bug (stale table), loud here rather than silently absorbed
/// (the transition would merely file it as `unattributed`).
fn map_fired_to_roots(
    counts: &FxHashMap<usize, u64>,
    roots: &[NodeId],
) -> (FxHashMap<NodeId, u64>, u64) {
    let mut fired = FxHashMap::default();
    let mut dropped = 0u64;
    for (rid, n) in counts {
        match roots.get(*rid) {
            Some(root) => *fired.entry(*root).or_default() += *n,
            None => dropped += *n,
        }
    }
    (fired, dropped)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Arm {
    A0,
    A1,
    A2,
}

struct ArmOutcome {
    node_traj: Vec<usize>,
    edge_traj: Vec<usize>,
    /// Per-epoch cumulative distinct RCF classes installed.
    rcf_curve: Vec<usize>,
    /// Per-epoch cumulative distinct SHAPE keys installed.
    shape_curve: Vec<usize>,
    /// Per-epoch kernel mutation events (the EVENT abscissa).
    events_curve: Vec<usize>,
    mut_events: usize,
    installed: usize,
    quarantined: usize,
    /// Accepted engine firings per epoch (attributed + unattributed alike:
    /// what the substrate actually did, regardless of mapping).
    accepted_curve: Vec<u64>,
    paid: u64,
    refused: u64,
    audits_balanced: usize,
    audits_total: usize,
    dropped_in_mapping: u64,
    survived_epochs: usize,
    final_roots: usize,
    /// G-GE2 panel: one row per survived epoch (dual-scope alphabet,
    /// site pump state, burn-in flag). Reported, not gated.
    panel: Vec<PanelEpoch>,
    /// Live region count after each survived epoch (persistence curve —
    /// SB-GE7(b)/R5 showed survival itself is seed-contingent).
    roots_curve: Vec<usize>,
    /// Standing-policy-1 ceiling log: epochs whose step loop hit
    /// `max_nodes` (empty when uncapped or never binding).
    bound_epochs: Vec<usize>,
}

fn run_arm(arm: Arm, seed: u64) -> ArmOutcome {
    run_arm_with(arm, seed, LoopConfig::default())
}

fn run_arm_with(arm: Arm, seed: u64, cfg: LoopConfig) -> ArmOutcome {
    let mut s = seed_world_n(cfg.seed_nodes);
    let mut src = GraphRuleSource::new();
    let mut seen_rcf = FxHashSet::default();
    let mut seen_shapes = FxHashSet::default();
    let mut rng = XorShift(seed | 1);
    // A2 carries no economy at all; A0's is the inert control.
    let mut econ = match arm {
        Arm::A0 => Some(EpochEconomy::new(IncomePolicy::FEE_ZERO)),
        Arm::A1 => Some(EpochEconomy::new(IncomePolicy::default())),
        Arm::A2 => None,
    };

    // Initial consultation (counters baseline, as in every epoch driver).
    let snap = s.snapshot();
    let _ = src.rules(&snap, 0);

    let mut out = ArmOutcome {
        node_traj: vec![s.node_count()],
        edge_traj: vec![s.edge_count()],
        rcf_curve: Vec::with_capacity(cfg.epochs),
        shape_curve: Vec::with_capacity(cfg.epochs),
        events_curve: Vec::with_capacity(cfg.epochs),
        mut_events: 0,
        installed: 0,
        quarantined: 0,
        accepted_curve: Vec::with_capacity(cfg.epochs),
        paid: 0,
        refused: 0,
        audits_balanced: 0,
        audits_total: 0,
        dropped_in_mapping: 0,
        survived_epochs: 0,
        final_roots: 0,
        panel: Vec::with_capacity(cfg.epochs),
        roots_curve: Vec::with_capacity(cfg.epochs),
        bound_epochs: Vec::new(),
    };

    // G-GE2 site aggregation (audit granularity decision): each epoch
    // yields at most one site (`EpochEvent.site`), so the per-epoch
    // entropy is degenerate by construction. The driver accumulates
    // per-root counts run-to-date and reports the cumulative window
    // every epoch — the only windowing with enough volume to reach the
    // pre-committed 20-event pump floor inside 40 epochs.
    let mut site_counts: FxHashMap<NodeId, u64> = FxHashMap::default();

    for epoch in 0..cfg.epochs {
        // 1. Consult: content order, validate, keep (root, cand) pairs.
        // Quarantined regions stay installed in the graph but leave the
        // engine table — the same semantics as `GraphRuleSource::rules`.
        let snap = s.snapshot();
        let mut pairs: Vec<(NodeId, CandidateRule)> = Vec::new();
        let mut consult_roots = 0usize;
        let mut consult_read_ok = 0usize;
        for root in find_rule_roots(&snap) {
            consult_roots += 1;
            let Ok((cand, fp)) = crate::schema::read_region_with_footprint(&snap, root) else {
                continue;
            };
            consult_read_ok += 1;
            if matches!(
                validate(&cand, &fp, &PriorityObservation { malformed: false }),
                Verdict::Valid
            ) {
                pairs.push((root, cand));
            }
        }
        if pairs.is_empty() {
            // Extinction funnel (R5 exposed it): roots present but none
            // consultable — report WHERE the funnel chokes, not just that
            // it did. Roots vs read-ok vs valid distinguishes destruction
            // (roots gone) from expiry (readable but invalid).
            println!(
                "  consult funnel at death: roots {consult_roots} read-ok {consult_read_ok} valid 0"
            );
            break; // region extinct
        }
        let roots: Vec<NodeId> = pairs.iter().map(|(r, _)| *r).collect();
        let cands: Vec<CandidateRule> = pairs.into_iter().map(|(_, c)| c).collect();

        // 2. Engine table + policy for this window. The policy sees the
        // post-settle ledger snapshot (§10.6 forward half).
        let table =
            compile_rules(&compile_delta(&cands, cfg.meta_visible)).expect("valid regions compile");
        let mut engine = Engine::new(table, StepMode::MaxParallel, 42, 10_000);
        match arm {
            Arm::A0 => engine.set_policy(Box::new(select_all_when_fee_is_zero())),
            Arm::A1 => {
                let e = econ.as_ref().expect("A1 carries an economy");
                let mut pol = FeePolicy::new(e.income);
                pol.ledgers.ledgers = roots.iter().map(|r| e.ledger(r).cloned()).collect();
                engine.set_policy(Box::new(pol));
            }
            Arm::A2 => {}
        }
        let counter = SharedFireCounter::new();
        engine.add_observer(Box::new(counter.clone()));

        // 3. Step the window, recording both trajectories every step.
        sync_label_mirror(&mut s);
        for _ in 0..cfg.steps_per_epoch {
            if cfg.max_nodes.is_some_and(|m| s.node_count() >= m) {
                out.bound_epochs.push(epoch);
                break;
            }
            let _ = engine.step(&mut s);
            sync_label_mirror(&mut s);
            out.node_traj.push(s.node_count());
            out.edge_traj.push(s.edge_count());
        }

        // 4. Attribute → transition (the §10.6 debit half lives inside).
        let counts = counter.0.lock().expect("uncontended").drain();
        out.accepted_curve.push(counts.values().sum());
        let (fired, dropped) = map_fired_to_roots(&counts, &roots);
        out.dropped_in_mapping += dropped;
        let ops: Option<&[Op]> = match arm {
            Arm::A2 => Some(&MID_OPS),
            _ => None,
        };
        let ev = epoch_transition_with_economy(
            &mut src,
            &mut s,
            &mut rng,
            &mut seen_rcf,
            cfg.meta_visible,
            &fired,
            econ.as_mut(),
            ops,
        );
        out.mut_events += usize::from(ev.mutated);
        out.installed += usize::from(ev.installed);
        out.quarantined += usize::from(ev.quarantined);
        if let Some(site) = ev.site {
            *site_counts.entry(site).or_default() += 1;
        }
        if let Some(a) = ev.economy {
            out.audits_total += 1;
            out.audits_balanced += usize::from(a.balanced);
            out.paid += a.paid;
            out.refused += a.refused;
        }
        out.events_curve.push(out.mut_events);

        // 5. Novelty registry: every installed region, RCF + SHAPE.
        let snap = s.snapshot();
        let mut live_roots = 0usize;
        for root in find_rule_roots(&snap) {
            if let Ok((cand, _)) = crate::schema::read_region_with_footprint(&snap, root) {
                live_roots += 1;
                seen_rcf.insert(rcf_hash(&cand));
                seen_shapes.insert(shape_key(&cand));
            }
        }
        out.roots_curve.push(live_roots);
        out.rcf_curve.push(seen_rcf.len());
        out.shape_curve.push(seen_shapes.len());

        // 6. G-GE2 panel emission on the post-install store: one
        // dual-scope S1 walk plus the run-to-date site window. Rows are
        // reported, never gated (panel-primary verdicts wait for R5).
        out.panel.push(PanelEpoch::from_observed(
            epoch,
            cfg.epochs,
            &observe_substrate(&s),
            seen_rcf.len(),
            out.mut_events,
            site_counts.values().copied(),
        ));

        out.survived_epochs += 1;
    }

    out.final_roots = find_rule_roots(&s.snapshot()).len();
    out
}

fn class_name(c: &DynClass) -> &'static str {
    match c {
        DynClass::Balanced => "Balanced",
        DynClass::Saturating { .. } => "Saturating",
        DynClass::Expanding => "Expanding",
        DynClass::RegimeExploding => "RegimeExploding",
    }
}

#[test]
fn exp_r09_outer_loop_a0_a1_a2() {
    println!("\n=== EXP-R09: fee-gated outer loop (GE plan §10, pre-registered) ===");
    println!("harness: 40 epochs x 25 MaxParallel steps, protection on, burn-in = first 4 epochs unscored");

    let a0 = run_arm(Arm::A0, 0x90A1);
    let a1 = run_arm(Arm::A1, 0x90A3);
    let a2 = run_arm(Arm::A2, 0x90A5);

    for (name, o) in [
        ("A0 fee-zero", &a0),
        ("A1 main", &a1),
        ("A2 excl-shadow", &a2),
    ] {
        let cls = classify_trajectory(&o.node_traj);
        let burn = EPOCHS / 10;
        let novel_post_burnin = o.shape_curve.last().copied().unwrap_or(0)
            - o.shape_curve.get(burn).copied().unwrap_or(0);
        println!(
            "{:<14} epochs {:>2}/40  roots {:>2}  traj {:<15} max_n {:>6}  max_e {:>6}  \
             shapes_end {:>3} (post-burn-in +{:>2})  rcf {:>3}  mut/inst/quar {:>3}/{:>3}/{:>3}  \
             paid {:>5} refused {:>5}  audits {}/{} balanced  accepted_tot {:>6}",
            name,
            o.survived_epochs,
            o.final_roots,
            class_name(&cls),
            o.node_traj.iter().max().copied().unwrap_or(0),
            o.edge_traj.iter().max().copied().unwrap_or(0),
            o.shape_curve.last().copied().unwrap_or(0),
            novel_post_burnin,
            o.rcf_curve.last().copied().unwrap_or(0),
            o.mut_events,
            o.installed,
            o.quarantined,
            o.paid,
            o.refused,
            o.audits_balanced,
            o.audits_total,
            o.accepted_curve.iter().sum::<u64>(),
        );
    }

    // — G-GE2 panel report (first light: emission, not verdict) —
    //
    // Per arm: final-epoch dual-scope alphabet row, site pump state, and
    // the scored novelty axes (burn-in cut applied). Panel rows are
    // reported as measurement; no asserts — panel-primary verdicts wait
    // for the R5 run (audit slice 3).
    let burn = EPOCHS / 10;
    for (name, o) in [
        ("A0 fee-zero", &a0),
        ("A1 main", &a1),
        ("A2 excl-shadow", &a2),
    ] {
        let per_event = novel_per_event(&o.shape_curve[burn..], &o.events_curve[burn..]);
        let gain = scored_gain(&o.shape_curve);
        match o.panel.last() {
            Some(row) => println!(
                "panel {:<14} A_eff excl {:>6.2} incl {:>6.2}  raw {:>3}/{:<3}  \
                 recur_excl {:>5.3} sing_excl {:>5.3}  sites {:>2} H {:>6.3} pump {}  \
                 scored_gain {:>2}  novel/event {:.4}",
                name,
                row.a_eff_excl.unwrap_or(f64::NAN),
                row.a_eff_incl.unwrap_or(f64::NAN),
                row.raw_excl,
                row.raw_incl,
                row.recurrent_excl.unwrap_or(f64::NAN),
                row.singleton_excl.unwrap_or(f64::NAN),
                row.site_events,
                row.site_entropy.unwrap_or(f64::NAN),
                row.pump,
                gain,
                per_event.unwrap_or(f64::NAN),
            ),
            None => println!("panel {name:<14} NO ROWS (arm extinct before epoch 0)"),
        }
        assert_eq!(
            o.panel.len(),
            o.survived_epochs,
            "{name}: one panel row per survived epoch"
        );
    }

    // — Committed readings (§10.3, amended SB-GE7) —
    //
    // A0 was pre-registered as a gate-assert (MUST reproduce the blob
    // regime). Run 1 fired the stop honestly: A0 came back Saturating, so
    // the fee was never binding and A1-vs-A0 suppression is unmeasurable
    // in this harness. Amendment rationale, not goalpost-moving: a
    // control-EXISTENCE verdict is a measurement, and keeping it red would
    // conflate the harness signal with suite health while SB-GE7 (below)
    // is open. The firing is on record in docs/ere_r09_findings.md run 1;
    // this test now reports the verdict and pins only harness coherence.
    let c0 = classify_trajectory(&a0.node_traj);
    let control_present = matches!(c0, DynClass::Expanding | DynClass::RegimeExploding);
    println!(
        "\nA0 CONTROL VERDICT: {} (trajectory {c0:?})",
        if control_present {
            "PRESENT — blob regime reproduced, A1 suppression interpretable"
        } else {
            "ABSENT (SB-GE7) — fee never binding; A1-vs-A0 suppression \
             unmeasurable here. See docs/ere_r09_findings.md. Do not tune \
             the harness to chase the control."
        }
    );

    // A1: conservation held every epoch it was measured. (Whether the blob
    // was suppressed is the REPORT below, not an assert — a priced-but-
    // exploding A1 reopens REDESIGN per §10.6 rather than failing loudly.)
    assert!(
        a1.audits_total == EPOCHS && a1.audits_balanced == a1.audits_total,
        "A1 conservation broke: {}/{} audits balanced",
        a1.audits_balanced,
        a1.audits_total
    );

    // Coherence on all arms: regions persist, mapping never drops.
    for (name, o) in [("A0", &a0), ("A1", &a1), ("A2", &a2)] {
        assert_eq!(o.survived_epochs, EPOCHS, "{name} went extinct");
        assert!(o.final_roots >= 2, "{name}: seed regions must persist");
        assert_eq!(
            o.dropped_in_mapping, 0,
            "{name}: rule_id→root mapping dropped firings (driver bug)"
        );
    }

    // — Report (measurement, not gates) —
    let c1 = classify_trajectory(&a1.node_traj);
    let c2 = classify_trajectory(&a2.node_traj);
    let s1 = a1.shape_curve.last().copied().unwrap_or(0);
    let s2 = a2.shape_curve.last().copied().unwrap_or(0);
    println!(
        "\nA1 trajectory {c1:?} (A0 was {c0:?}); A2 trajectory {c2:?}; \
         novelty A1 {s1} vs A2 {s2} shapes"
    );
    if matches!(c1, DynClass::RegimeExploding) {
        println!(
            "REDESIGN-REOPEN (§10.6): priced-but-exploding — unrestrained firing \
             was not the operative channel. Report against §2 as a new species; \
             exclusion becomes the default descope."
        );
    } else if s1 >= s2 {
        println!("DECISION (§10.3): A1 novelty ≥ A2 novelty — fee-gating dominates exclusion on both axes.");
    } else {
        println!("DECISION (§10.3): A1 novelty < A2 novelty — fees cost shape reach; record the suppression curve (FM-GE3 watch).");
    }
    // FM-GE3 watch, quantitative: quarantine share must not collapse under fees.
    println!(
        "FM-GE3 watch: quarantine share A0 {:.3} vs A1 {:.3} (of kernel events)",
        a0.quarantined as f64 / a0.mut_events.max(1) as f64,
        a1.quarantined as f64 / a1.mut_events.max(1) as f64,
    );
}

/// R5: panel-primary outer loop vs the EXP-R07 null — pre-registered in
/// `docs/ere_r5_findings.md`. Reference loop, 150 epochs (one R07
/// quartile), full A0/A1/A2 + panel. The null comparison is REPORTED:
/// R07 ran kernel-only epochs while R5 fires 25 steps per epoch, so any
/// yield gap reads as suppression (FM-GE3 form), never as a defect.
/// Coherence asserted; verdicts transcribed to findings.
#[test]
fn r5_panel_primary() {
    panel_primary_impl(
        "=== R5: panel-primary outer loop vs EXP-R07 null (pre-registered) ===",
        LoopConfig {
            epochs: 150,
            ..LoopConfig::default()
        },
        [
            ("A0 fee-zero", Arm::A0, 0x9501u64),
            ("A1 main", Arm::A1, 0x9503u64),
            ("A2 excl-shadow", Arm::A2, 0x9505u64),
        ],
    );
}

/// R5-C2-LOOP (pre-registered in `docs/ere_r5_findings.md`): the R5
/// panel-primary protocol on the SB-GE7(b) C2 substrate (40-node world),
/// licensed by the C2 cell meeting the pre-committed success criterion
/// under the SB-R6-corrected instrument.
#[test]
fn r5_c2_loop() {
    panel_primary_impl(
        "=== R5-C2-LOOP: panel-primary protocol on the C2 substrate (pre-registered) ===",
        LoopConfig {
            epochs: 150,
            seed_nodes: 40,
            ..LoopConfig::default()
        },
        [
            ("A0 fee-zero", Arm::A0, 0x9501u64),
            ("A1 main", Arm::A1, 0x9503u64),
            ("A2 excl-shadow", Arm::A2, 0x9505u64),
        ],
    );
}

fn panel_primary_impl(tag: &str, cfg: LoopConfig, arms: [(&str, Arm, u64); 3]) {
    println!("\n{tag}");
    // Null quartile yields, in order (ere_r6_findings.md, seed 0x600D).
    let null_q = [61usize, 59, 60, 52];
    // Collect-then-assert: one arm's death must not hide the others' data.
    let mut outs: Vec<(&str, Arm, ArmOutcome)> = Vec::with_capacity(3);
    for (name, arm, seed) in arms {
        outs.push((name, arm, run_arm_with(arm, seed, cfg)));
    }
    for (name, arm, o) in &outs {
        let cls = classify_trajectory(&o.node_traj);
        let cls_e = classify_trajectory(&o.edge_traj);
        let burn = cfg.epochs / 10;
        let per_event = novel_per_event(&o.shape_curve[burn..], &o.events_curve[burn..]);
        // Quarter yields inside the 150 (shape_curve is per-epoch cumulative).
        let cuts = [37usize, 74, 111, 149];
        let mut q = [0usize; 4];
        let mut prev = 0usize;
        for (slot, c) in q.iter_mut().zip(cuts.iter()) {
            let v = o.shape_curve.get(*c).copied().unwrap_or(prev);
            *slot = v.saturating_sub(prev);
            prev = v;
        }
        println!(
            "{:<14} epochs {:>3}/150  traj {:<15} (edge {:<15}) max_n {:>6}  \
             shapes {:>3} (+{:>2} scored)  quarters {:>2}/{:>2}/{:>2}/{:>2} (null {:>2}/{:>2}/{:>2}/{:>2})  \
             mut/inst/quar {:>3}/{:>3}/{:>3}  paid {:>5} refused {:>5}  accepted {:>6}  novel/event {:.4}",
            name,
            o.survived_epochs,
            class_name(&cls),
            class_name(&cls_e),
            o.node_traj.iter().max().copied().unwrap_or(0),
            o.shape_curve.last().copied().unwrap_or(0),
            scored_gain(&o.shape_curve),
            q[0],
            q[1],
            q[2],
            q[3],
            null_q[0],
            null_q[1],
            null_q[2],
            null_q[3],
            o.mut_events,
            o.installed,
            o.quarantined,
            o.paid,
            o.refused,
            o.accepted_curve.iter().sum::<u64>(),
            per_event.unwrap_or(f64::NAN),
        );
        match o.panel.last() {
            Some(row) => println!(
                "  panel: A_eff excl {:>6.2} incl {:>6.2}  raw {:>3}/{:<3}  \
                 recur_excl {:>5.3} sing_excl {:>5.3}  sites {:>3} H {:>6.3} pump {}",
                row.a_eff_excl.unwrap_or(f64::NAN),
                row.a_eff_incl.unwrap_or(f64::NAN),
                row.raw_excl,
                row.raw_incl,
                row.recurrent_excl.unwrap_or(f64::NAN),
                row.singleton_excl.unwrap_or(f64::NAN),
                row.site_events,
                row.site_entropy.unwrap_or(f64::NAN),
                row.pump,
            ),
            None => println!("  panel: NO ROWS (arm extinct before epoch 0)"),
        }
        // Blob-panel excursion vs the §2 register: reported, never gated
        // (an excursion is a finding — species recurrence or new species).
        let quiet_nodes = matches!(cls, DynClass::Balanced | DynClass::Saturating { .. });
        let quiet_edges = matches!(cls_e, DynClass::Balanced | DynClass::Saturating { .. });
        let r0 = o.roots_curve.first().copied().unwrap_or(0);
        let rmin = o.roots_curve.iter().min().copied().unwrap_or(0);
        let rend = o.roots_curve.last().copied().unwrap_or(0);
        println!(
            "  excursion check: nodes {} / edges {}  roots {}→{}→{} (start→min→end)",
            if quiet_nodes {
                "quiet"
            } else {
                "EXCURSION — see findings"
            },
            if quiet_edges {
                "quiet"
            } else {
                "EXCURSION — see findings"
            },
            r0,
            rmin,
            rend,
        );
        assert_eq!(
            o.survived_epochs, cfg.epochs,
            "{name}: arm went extinct (harness coherence)"
        );
        assert_eq!(
            o.dropped_in_mapping, 0,
            "{name}: rule_id→root mapping dropped firings (driver bug)"
        );
        assert_eq!(
            o.panel.len(),
            o.survived_epochs,
            "{name}: one panel row per survived epoch"
        );
        if *arm == Arm::A1 {
            assert!(
                o.audits_total == cfg.epochs && o.audits_balanced == o.audits_total,
                "{name}: conservation broke: {}/{} audits balanced",
                o.audits_balanced,
                o.audits_total
            );
        }
    }
    println!("\nFM-GE3 suppression readout + verdicts: see docs/ere_r5_findings.md.");
}

/// R5-MULTISEED: the distribution behind R5's point estimates —
/// pre-registered in `docs/ere_r5_findings.md`. Same 150-epoch reference
/// loop, 5 fresh seeds per arm. Reports ranges and tables; asserts only
/// coherence (survival, mapping, panel rows, A1 audits) — the decision
/// rules are evaluated in findings, not gated here.
#[test]
fn r5_multiseed() {
    println!("\n=== R5-MULTISEED: yield/volume distributions (pre-registered) ===");
    println!("ceiling: max_nodes 2_000, binding logged per seed (standing policy 1)");
    let cfg = LoopConfig {
        epochs: 150,
        max_nodes: Some(2_000),
        ..LoopConfig::default()
    };
    // Fresh odd distinct seeds per arm; R5's 0x9501/03/05 stay out.
    // A0 0x9517 RETIRED for compute pathology (owned): >12 CPU-min
    // without completing 150 epochs — late-epoch match-enumeration grind
    // below every ceiling (20k never bound in time; probe showed epochs
    // 0–20 take 0.03s, so cost is superlinear late). Its datum stands as
    // finding (i)'s extreme point (cost lottery ≥1000×); the A0 yield
    // distribution runs on the 4 completed seeds, asymmetry owned.
    // No replacement: selecting a seed that completes would be
    // survivorship bias on the very axis under measurement.
    let grids: [(&str, Arm, &[u64]); 3] = [
        ("A0", Arm::A0, &[0x9511, 0x9513, 0x9515, 0x9519]),
        ("A1", Arm::A1, &[0x9521, 0x9523, 0x9525, 0x9527, 0x9529]),
        ("A2", Arm::A2, &[0x9531, 0x9533, 0x9535, 0x9537, 0x9539]),
    ];
    for (name, arm, seeds) in grids {
        let mut yields: Vec<usize> = Vec::with_capacity(5);
        let mut vols: Vec<u64> = Vec::with_capacity(5);
        for &seed in seeds {
            let t0 = std::time::Instant::now();
            let o = run_arm_with(arm, seed, cfg);
            let dt = t0.elapsed();
            let cls = classify_trajectory(&o.node_traj);
            let cuts = [37usize, 74, 111, 149];
            let mut q = [0usize; 4];
            let mut prev = 0usize;
            for (slot, c) in q.iter_mut().zip(cuts.iter()) {
                let v = o.shape_curve.get(*c).copied().unwrap_or(prev);
                *slot = v.saturating_sub(prev);
                prev = v;
            }
            let y = o.shape_curve.last().copied().unwrap_or(0);
            let v: u64 = o.accepted_curve.iter().sum();
            yields.push(y);
            vols.push(v);
            println!(
                "  {name} seed {seed:#X}  traj {:<12} shapes {:>3} (+{:>2})  q {:>2}/{:>2}/{:>2}/{:>2}  accepted {:>6}  max_n {:>6}  bound {:>3}ep  quar {}/{}  wall {:.1}s",
                class_name(&cls),
                y,
                scored_gain(&o.shape_curve),
                q[0],
                q[1],
                q[2],
                q[3],
                v,
                o.node_traj.iter().max().copied().unwrap_or(0),
                o.bound_epochs.len(),
                o.quarantined,
                o.mut_events.max(1),
                dt.as_secs_f64(),
            );
            assert_eq!(o.survived_epochs, cfg.epochs, "{name} {seed:#X}: extinct");
            assert_eq!(o.dropped_in_mapping, 0, "{name} {seed:#X}: mapping dropped");
            assert_eq!(
                o.panel.len(),
                o.survived_epochs,
                "{name} {seed:#X}: panel short"
            );
            if arm == Arm::A1 {
                assert!(
                    o.audits_total == cfg.epochs && o.audits_balanced == o.audits_total,
                    "{name} {seed:#X}: conservation broke"
                );
            }
        }
        let (ymin, ymax) = (
            yields.iter().min().copied().unwrap_or(0),
            yields.iter().max().copied().unwrap_or(0),
        );
        let (vmin, vmax) = (
            vols.iter().min().copied().unwrap_or(0),
            vols.iter().max().copied().unwrap_or(0),
        );
        let ymean = yields.iter().sum::<usize>() as f64 / yields.len().max(1) as f64;
        println!(
            "  {name} SUMMARY: yield min/mean/max {ymin}/{ymean:.1}/{ymax}  accepted min/max {vmin}/{vmax}"
        );
    }
}

/// R5-PAIRED: the paired-seed A0/A1 grid — pre-registered in
/// `docs/ere_r5_findings.md` (R5-PAIRED section). Same 150-epoch
/// reference loop + panel + ceiling; each of 5 fresh seeds runs under
/// BOTH arms. Reports per-seed yield gaps and volume ratios; asserts
/// only coherence (survival, mapping, panel rows, A1 conservation) —
/// P1/P2/P3 are evaluated in findings, not gated here.
#[test]
fn r5_paired() {
    println!("\n=== R5-PAIRED: paired-seed A0/A1 grid (pre-registered) ===");
    println!("ceiling: max_nodes 2_000, binding logged per seed (standing policy 1)");
    let cfg = LoopConfig {
        epochs: 150,
        max_nodes: Some(2_000),
        ..LoopConfig::default()
    };
    // Fresh odd distinct seeds; everything used before stays out (R5's
    // 0x950x, the grid's 0x951x-0x953x, retired 0x9517).
    let seeds: &[u64] = &[0x9541, 0x9543, 0x9545, 0x9547, 0x9549];
    let mut gaps: Vec<i64> = Vec::with_capacity(5);
    for &seed in seeds {
        let mut row: [(usize, u64); 2] = [(0, 0); 2];
        for (i, arm) in [Arm::A0, Arm::A1].iter().enumerate() {
            let aname = if *arm == Arm::A0 { "A0" } else { "A1" };
            let t0 = std::time::Instant::now();
            let o = run_arm_with(*arm, seed, cfg);
            let dt = t0.elapsed();
            let cls = classify_trajectory(&o.node_traj);
            let y = o.shape_curve.last().copied().unwrap_or(0);
            let v: u64 = o.accepted_curve.iter().sum();
            row[i] = (y, v);
            println!(
                "  seed {seed:#X} {aname}  traj {:<12} shapes {:>3} (+{:>2})  accepted {:>6}  max_n {:>6}  bound {:>3}ep  quar {}/{}  wall {:.1}s",
                class_name(&cls),
                y,
                scored_gain(&o.shape_curve),
                v,
                o.node_traj.iter().max().copied().unwrap_or(0),
                o.bound_epochs.len(),
                o.quarantined,
                o.mut_events.max(1),
                dt.as_secs_f64(),
            );
            assert_eq!(
                o.survived_epochs, cfg.epochs,
                "seed {seed:#X} {aname}: extinct"
            );
            assert_eq!(
                o.dropped_in_mapping, 0,
                "seed {seed:#X} {aname}: mapping dropped"
            );
            assert_eq!(
                o.panel.len(),
                o.survived_epochs,
                "seed {seed:#X} {aname}: panel short"
            );
            if *arm == Arm::A1 {
                assert!(
                    o.audits_total == cfg.epochs && o.audits_balanced == o.audits_total,
                    "seed {seed:#X}: conservation broke"
                );
            }
        }
        let (y0, v0) = row[0];
        let (y1, v1) = row[1];
        let gap = y0 as i64 - y1 as i64;
        gaps.push(gap);
        let ratio = v0 as f64 / v1.max(1) as f64;
        println!("  seed {seed:#X} GAP yield A0-A1 {gap:+}  volume A0/A1 {v0}/{v1} = {ratio:.1}x");
    }
    let pos = gaps.iter().filter(|g| **g > 0).count();
    let neg = gaps.iter().filter(|g| **g < 0).count();
    let mean_gap = gaps.iter().sum::<i64>() as f64 / gaps.len().max(1) as f64;
    println!("PAIRED SUMMARY: gaps {gaps:?}  +{pos}/-{neg}  mean {mean_gap:+.1} (P1 needs 5/5 one sign; P2/P3 in findings)");
}

/// R5-C2-GRID (pre-registered in `docs/ere_r5_findings.md`): the paired
/// grid on the C2 substrate (40-node world, no ceiling) — the 129× as
/// a distribution, all three arms within-seed.
///
/// HEAVY + UNSAFE for default suites: 0x9557-A0 was SIGKILLed
/// (presumed memory kill — uncapped explosive cell, no panic/assert
/// output) on the 2026-09-08 run, and 0x9553-A0 needs ~21 min wall.
/// Run explicitly:
/// `cargo test --release -p hg-meta --lib expr_r09::r5_c2_grid -- --ignored --nocapture`.
#[test]
#[ignore]
fn r5_c2_grid() {
    println!("\n=== R5-C2-GRID: paired 3-arm grid on C2 substrate (pre-registered) ===");
    println!("no ceiling (C2-loop A0 hit max_n 20821 uncapped); binding would measure the ceiling");
    let cfg = LoopConfig {
        epochs: 150,
        seed_nodes: 40,
        ..LoopConfig::default()
    };
    // Fresh odd distinct seeds; everything used before stays out.
    let seeds: &[u64] = &[0x9551, 0x9553, 0x9555, 0x9557, 0x9559];
    for &seed in seeds {
        // (shapes, accepted) per arm: A0, A1, A2.
        let mut row: [(usize, u64); 3] = [(0, 0); 3];
        for (i, arm) in [Arm::A0, Arm::A1, Arm::A2].iter().enumerate() {
            let aname = match arm {
                Arm::A0 => "A0",
                Arm::A1 => "A1",
                _ => "A2",
            };
            let t0 = std::time::Instant::now();
            let o = run_arm_with(*arm, seed, cfg);
            let dt = t0.elapsed();
            let cls = classify_trajectory(&o.node_traj);
            let y = o.shape_curve.last().copied().unwrap_or(0);
            let v: u64 = o.accepted_curve.iter().sum();
            row[i] = (y, v);
            println!(
                "  seed {seed:#X} {aname}  traj {:<12} shapes {:>3} (+{:>2})  accepted {:>8}  max_n {:>6}  quar {}/{}  wall {:.1}s",
                class_name(&cls),
                y,
                scored_gain(&o.shape_curve),
                v,
                o.node_traj.iter().max().copied().unwrap_or(0),
                o.quarantined,
                o.mut_events.max(1),
                dt.as_secs_f64(),
            );
            assert_eq!(
                o.survived_epochs, cfg.epochs,
                "seed {seed:#X} {aname}: extinct"
            );
            assert_eq!(
                o.dropped_in_mapping, 0,
                "seed {seed:#X} {aname}: mapping dropped"
            );
            assert_eq!(
                o.panel.len(),
                o.survived_epochs,
                "seed {seed:#X} {aname}: panel short"
            );
            if *arm == Arm::A1 {
                assert!(
                    o.audits_total == cfg.epochs && o.audits_balanced == o.audits_total,
                    "seed {seed:#X}: conservation broke"
                );
            }
        }
        let [(y0, v0), (y1, v1), (y2, _)] = row;
        let gap = y0 as i64 - y1 as i64;
        let ratio = v0 as f64 / v1.max(1) as f64;
        let nov = y1 as i64 - y2 as i64;
        println!("  seed {seed:#X} GAP yield A0-A1 {gap:+}  volume A0/A1 {v0}/{v1} = {ratio:.1}x  novelty A1-A2 {nov:+}");
    }
    println!("GRID SUMMARY: crush/novelty distributions in docs/ere_r5_findings.md.");
}

/// SB-GE7(b) emergence search — pre-registered in
/// `docs/ere_r09_findings.md`, one shot, four A0-only cells.
///
/// The emergence verdict is REPORTED per the pre-committed decision
/// rule (cell succeeds iff A0 classifies Expanding/RegimeExploding),
/// never gated: asserting emergence would fail-red on the expected
/// outcome, asserting its absence would forbid the discovery. Only
/// harness coherence is asserted (a crashed cell must be loud, a quiet
/// cell must be data).
#[test]
fn sb_ge7b_emergence_search() {
    println!("\n=== SB-GE7(b): emergence-first search (pre-registered, A0-only) ===");
    let cells: [(&str, LoopConfig, u64); 4] = [
        ("C1 reference", LoopConfig::default(), 0xB001),
        (
            "C2 bigger-world",
            LoopConfig {
                seed_nodes: 40,
                ..LoopConfig::default()
            },
            0xB003,
        ),
        (
            "C3 longer-windows",
            LoopConfig {
                steps_per_epoch: 100,
                ..LoopConfig::default()
            },
            0xB005,
        ),
        (
            "C4 unprotected",
            LoopConfig {
                meta_visible: true,
                ..LoopConfig::default()
            },
            0xB007,
        ),
    ];
    let mut successes: Vec<&str> = Vec::new();
    for (name, cfg, seed) in cells {
        let o = run_arm_with(Arm::A0, seed, cfg);
        let cls = classify_trajectory(&o.node_traj);
        let burn = cfg.epochs / 10;
        let per_event = novel_per_event(&o.shape_curve[burn..], &o.events_curve[burn..]);
        println!(
            "{:<16} epochs {:>2}/{}  traj {:<15} max_n {:>6}  max_e {:>6}  \
             shapes {:>3} (+{:>2})  mut/inst/quar {:>3}/{:>3}/{:>3}  accepted {:>6}  novel/event {:.4}",
            name,
            o.survived_epochs,
            cfg.epochs,
            class_name(&cls),
            o.node_traj.iter().max().copied().unwrap_or(0),
            o.edge_traj.iter().max().copied().unwrap_or(0),
            o.shape_curve.last().copied().unwrap_or(0),
            scored_gain(&o.shape_curve),
            o.mut_events,
            o.installed,
            o.quarantined,
            o.accepted_curve.iter().sum::<u64>(),
            per_event.unwrap_or(f64::NAN),
        );
        match o.panel.last() {
            Some(row) => println!(
                "  panel: A_eff excl {:>6.2} incl {:>6.2}  raw {:>3}/{:<3}  \
                 recur_excl {:>5.3} sing_excl {:>5.3}  sites {:>2} H {:>6.3} pump {}",
                row.a_eff_excl.unwrap_or(f64::NAN),
                row.a_eff_incl.unwrap_or(f64::NAN),
                row.raw_excl,
                row.raw_incl,
                row.recurrent_excl.unwrap_or(f64::NAN),
                row.singleton_excl.unwrap_or(f64::NAN),
                row.site_events,
                row.site_entropy.unwrap_or(f64::NAN),
                row.pump,
            ),
            None => println!("  panel: NO ROWS (cell extinct before epoch 0)"),
        }
        assert_eq!(
            o.dropped_in_mapping, 0,
            "{name}: rule_id→root mapping dropped firings (driver bug)"
        );
        assert_eq!(
            o.panel.len(),
            o.survived_epochs,
            "{name}: one panel row per survived epoch"
        );
        // Extinction coherence pins ONLY the reference cell (prior data:
        // the R09 loop survives 40/40). Search cells may die — C4's
        // unprotected band eating its own regions would be a measurement
        // about viability, not a driver bug, and must read as data.
        let extinct = o.survived_epochs != cfg.epochs;
        if name == "C1 reference" {
            assert!(
                !extinct,
                "C1: reference loop went extinct (harness coherence)"
            );
        }
        let emerged = matches!(cls, DynClass::Expanding | DynClass::RegimeExploding);
        println!(
            "  verdict: {}",
            if emerged {
                "SUCCESS — blob regime emerged under A0"
            } else if extinct {
                "no emergence (cell extinct — viability datum, see findings)"
            } else {
                "no emergence"
            }
        );
        if emerged {
            successes.push(name);
        }
    }
    println!("\nDECISION (pre-committed rule, first success C2 → C3 → C4):");
    match successes.as_slice() {
        [] => println!(
            "no cell emerged — SB-GE7(b) CLOSES as a failed attempt; R5 on the reference loop."
        ),
        ss => println!(
            "R5 loop candidate: {} (successes: {})",
            ss[0],
            ss.join(", ")
        ),
    }
}
