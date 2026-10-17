//! `GraphRuleSource` (plan Phase R1): reads rule regions from the world
//! snapshot at epoch boundaries, validates, compiles. The compile pipeline
//! of plan §2.4, minus engine integration.
//!
//! Determinism: enumeration via [`find_rule_roots`] (D1 content order);
//! no RNG anywhere (D2). Totality: every failure is counted (read-rejected
//! or quarantined), never fatal — "install | quarantine | count", the
//! plan's contract.
//!
//! Note the source reads the WORLD snapshot: in arm A2/A3 the rule regions
//! live inside the world store (that is the whole point of the track), so
//! one store carries both substrate and meta structure. For R3's identity
//! fixtures the regions are installed into the same store the engine steps.

use crate::compile::candidate_to_rule;
use crate::install::install_region;
use crate::rcf::rcf_hash;
use crate::schema::{find_rule_roots, read_region_with_footprint, CandidateRule};
use crate::validate::{validate, EngagementCounters, PriorityObservation};
use hg_core::{GraphSnapshot, HypergraphStore, NodeId, PropertyValue};
use hg_rules::source::{EpochPolicy, RuleSetDelta, RuleSource};

/// What one region contributed to the most recent consultation.
#[derive(Clone, Copy, Debug)]
pub struct RegionOutcome {
    /// Index into the delta's rule vector, if installed.
    pub installed_index: Option<usize>,
    /// RCF class of the region (computed even for quarantined regions, so
    /// quarantine accounting can be joined against novelty accounting).
    pub rcf_hash: u64,
}

pub struct GraphRuleSource {
    /// Engagement counters, cumulative across consultations (G-R4).
    pub counters: EngagementCounters,
    /// Distinct RCF classes seen since construction (novelty instrument).
    pub seen_classes: rustc_hash::FxHashSet<u64>,
    /// Count of first-time classes (never-before-seen RCFs).
    pub novel_classes: usize,
    last_outcomes: Vec<RegionOutcome>,
}

impl Default for GraphRuleSource {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphRuleSource {
    pub fn new() -> Self {
        Self {
            counters: EngagementCounters::default(),
            seen_classes: rustc_hash::FxHashSet::default(),
            novel_classes: 0,
            last_outcomes: Vec::new(),
        }
    }

    /// Outcomes of the most recent consultation.
    pub fn last_outcomes(&self) -> &[RegionOutcome] {
        &self.last_outcomes
    }

    /// Write-back path for meta-rewrites and fixtures: install
    /// CandidateRules as regions; returns the RULE_ROOT ids in order.
    pub fn install_candidates(s: &mut HypergraphStore, cands: &[CandidateRule]) -> Vec<NodeId> {
        cands.iter().map(|c| install_region(s, c)).collect()
    }
}

impl RuleSource for GraphRuleSource {
    fn rules(&mut self, snapshot: &GraphSnapshot<'_>, _step: u64) -> RuleSetDelta {
        let mut delta = RuleSetDelta::default();
        self.last_outcomes.clear();

        for root in find_rule_roots(snapshot) {
            match read_region_with_footprint(snapshot, root) {
                Err(rej) => {
                    // Walk-level structural failure: no rule formed. Most
                    // rejections count as V8 (the two-stage validator's
                    // second stage — the compile layer's walk audit);
                    // ArityOverflow is V3's predicate surfacing at the walk,
                    // so it maps there. Everything lands in the delta's
                    // read_rejected bucket regardless.
                    let vid = match rej {
                        crate::schema::ReadRejection::ArityOverflow => {
                            crate::validate::ValidatorId::V3
                        }
                        _ => crate::validate::ValidatorId::V8,
                    };
                    self.counters.record(vid);
                    delta.read_rejected += 1;
                    self.last_outcomes.push(RegionOutcome {
                        installed_index: None,
                        rcf_hash: 0,
                    });
                }
                Ok((cand, fp)) => {
                    // Priority malformation is invisible to the reader (it
                    // coerces); observe the raw property here so V5 fires.
                    let malformed = !matches!(
                        snapshot.get_node(root).and_then(|d| d.get_property(2)),
                        None | Some(PropertyValue::Int(_))
                    );
                    let h = rcf_hash(&cand);
                    match validate(&cand, &fp, &PriorityObservation { malformed }) {
                        crate::validate::Verdict::Quarantined(id) => {
                            self.counters.record(id);
                            delta.quarantined += 1;
                            self.last_outcomes.push(RegionOutcome {
                                installed_index: None,
                                rcf_hash: h,
                            });
                        }
                        crate::validate::Verdict::Valid => {
                            delta.rules.push(candidate_to_rule(&cand));
                            let idx = delta.rules.len() - 1;
                            self.last_outcomes.push(RegionOutcome {
                                installed_index: Some(idx),
                                rcf_hash: h,
                            });
                            if self.seen_classes.insert(h) {
                                self.novel_classes += 1;
                            }
                        }
                    }
                }
            }
        }
        delta
    }

    fn epoch_len(&self) -> EpochPolicy {
        // Consultation cadence is the driver's choice; the source itself
        // is always willing. A fixed cadence belongs to the harness config,
        // so the source reports Never and the driver consults explicitly.
        EpochPolicy::Never
    }
}
