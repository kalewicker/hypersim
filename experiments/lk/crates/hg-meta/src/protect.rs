//! Arm A2/A3: epoch-driven recompile with the meta_visible protection
//! scheme (plan §7.2, §11).
//!
//! Protection WITHOUT touching the matcher (ground rule 1): the matcher's
//! property guards read node PROPERTIES; labels are not properties. So the
//! guard `label < META_BASE` is implemented as a mirrored property:
//!
//!   - [`sync_label_mirror`] copies every world node's label into property
//!     key [`K_LABEL_MIRROR`] (Int). Called once per step by the driver —
//!     O(n) over the snapshot, deterministic.
//!   - With `meta_visible = false` (A2, default), [`candidate_to_rule_gated`]
//!     injects a `NodePropertyGuard { var, K_LABEL_MIRROR, Lt,
//!     Int(META_BASE) }` on EVERY LHS var at compile time. Ordinary rules
//!     then cannot bind any meta-band node — the rule region is invisible
//!     to them. This is exactly the plan's "implicit label < META_BASE
//!     guard on every LHS var", realized through existing machinery.
//!   - With `meta_visible = true` (A3), no guards are injected: rules can
//!     match rule regions, and persistence must be earned.

use crate::compile::interned_names;
use crate::schema::{CandidatePatternEdge, CandidateRule, META_BASE};
use hg_core::{HypergraphStore, PropertyValue};
use hg_rules::rule::Rule;

/// Property key carrying the mirrored label. SB-R6 (EXP-R09 battery): this
/// was key 0, colliding with schema `K_LABEL_VALUE` (the LABEL_LIT value
/// key) — `sync_label_mirror` writes EVERY node including LABEL_LITs, so
/// one sync overwrote each HASLABEL value with the lit node's own label
/// and V4-quarantined every labeled region at the next consult. Key 3 is
/// free (substrate energy is 1, priority is 2, guard/effect blocks are
/// 10+); the reader ignores non-`K_LABEL_VALUE` props on lit nodes, so the
/// extra mirror prop there is harmless.
pub const K_LABEL_MIRROR: u32 = 3;

/// Rewrite every node's K_LABEL_MIRROR to its current label. Deterministic;
/// no RNG; total. Returns the number of nodes touched (metrics row).
pub fn sync_label_mirror(s: &mut HypergraphStore) -> usize {
    let updates: Vec<(hg_core::NodeId, i64)> =
        s.nodes().map(|(id, d)| (id, d.label.0 as i64)).collect();
    for (id, lab) in &updates {
        if let Some(d) = s.get_node_mut(*id) {
            d.set_property(K_LABEL_MIRROR, PropertyValue::Int(*lab));
        }
    }
    updates.len()
}

/// Compile with the protection gate. `meta_visible = false` injects the
/// label<META_BASE guard on every LHS var (A2); `true` compiles bare (A3).
///
/// The injected guards ride in `lhs_node_property_guards`, so compilation,
/// matching, conflict resolution, and GuardStats all behave exactly as for
/// hand-written guarded rules. Nothing downstream can tell the difference.
pub fn candidate_to_rule_gated(cand: &CandidateRule, meta_visible: bool) -> Rule {
    let mut rule = crate::compile::candidate_to_rule(cand);
    if !meta_visible {
        // SB-GE6: inject under the INTERNED names (compile.rs::interned_names),
        // never raw indices — the two coincide only when the reader returns
        // vars in identity order, which its RHS-before-LHS content sort does
        // not guarantee. Unwired vars (empty name) are skipped: the compiler
        // discarded them, so a guard on them would name a var absent from LHS.
        let (names, _) = interned_names(cand);
        let wired = wired_lhs_vars(cand);
        for (v, wired_v) in wired.iter().enumerate() {
            if !wired_v || names[v].is_empty() {
                continue;
            }
            rule.lhs_node_property_guards
                .push(hg_rules::NodePropertyGuard {
                    var: names[v].clone(),
                    property: K_LABEL_MIRROR,
                    op: hg_core::PropertyOp::Lt,
                    value: PropertyValue::Int(META_BASE as i64),
                });
        }
    }
    rule
}

fn wired_lhs_vars(cand: &CandidateRule) -> Vec<bool> {
    let mut m = vec![false; cand.n_vars()];
    for e in &cand.lhs {
        for s in &e.slots {
            m[*s] = true;
        }
    }
    m
}

/// Convenience: compile a whole delta under an arm's visibility setting.
pub fn compile_delta(rules: &[CandidateRule], meta_visible: bool) -> Vec<Rule> {
    rules
        .iter()
        .map(|c| candidate_to_rule_gated(c, meta_visible))
        .collect()
}

#[allow(unused)]
fn _type_anchor(_e: CandidatePatternEdge) {}
