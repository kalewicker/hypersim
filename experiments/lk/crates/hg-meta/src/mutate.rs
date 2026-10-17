//! Mutation kernel over CandidateRule (plan §6, G-R1).
//!
//! Single-edit mutations only — the kernel whose viability distribution
//! answers v10's error catastrophe at the encoding level. Each mutation is
//! total (returns a modified clone or Nothing) and deterministic given the
//! RNG stream handed to it; the harness owns seeding.
//!
//! Kernel (one edit each):
//! - AddSlot / RemoveSlot      — attach/detach a var to/from an edge
//! - RetargetSlot              — move a slot to another var
//! - AddVar                    — new var wired into one existing edge
//! - RemoveVar                 — drop a var and every reference to it
//! - AddPatternEdge / RemovePatternEdge
//! - SetLabel / ClearLabel     — add/remove a label constraint
//! - ChangePriority            — ±1..3

use crate::schema::{CandidatePatternEdge, CandidateRule};

/// One sampled mutation application. `Nothing` means the sampled parameters
/// were out of range for this rule (e.g. RemoveSlot on an arity-0 edge);
/// counting Nothing as its own class keeps sampling honest — it is not a
/// Valid outcome.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Applied {
    Edited,
    Nothing,
}

/// Uniform-ish small value from the harness RNG, kept as u64 so the kernel
/// itself stays RNG-agnostic (D2: no RNG types in hg-meta's public API).
pub trait MutationRng {
    fn next_u64(&mut self) -> u64;
    /// Draw in [0, n).
    fn below(&mut self, n: usize) -> usize {
        if n == 0 {
            0
        } else {
            (self.next_u64() % n as u64) as usize
        }
    }
}

impl<M: MutationRng + ?Sized> MutationRng for &mut M {
    fn next_u64(&mut self) -> u64 {
        (**self).next_u64()
    }
}

/// The ten single-edit operators, in dispatch order. G-GE1 (GE plan §4)
/// attributes growth bias PER OPERATOR, which requires knowing which op a
/// draw used; the plain [`mutate`] entry point stays available for the
/// published G-R1 path.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Op {
    AddSlot,
    RemoveSlot,
    RetargetSlot,
    AddVar,
    RemoveVar,
    AddPatternEdge,
    RemovePatternEdge,
    SetLabel,
    ClearLabel,
    ChangePriority,
}

impl Op {
    /// Dispatch order == historical match-arm order of `mutate`, pinned by
    /// `gge1_tests::dispatch_order_matches_mutate` so the G-R1 sampling
    /// distribution cannot silently drift.
    pub const ALL: [Op; 10] = [
        Op::AddSlot,
        Op::RemoveSlot,
        Op::RetargetSlot,
        Op::AddVar,
        Op::RemoveVar,
        Op::AddPatternEdge,
        Op::RemovePatternEdge,
        Op::SetLabel,
        Op::ClearLabel,
        Op::ChangePriority,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Op::AddSlot => "add_slot",
            Op::RemoveSlot => "remove_slot",
            Op::RetargetSlot => "retarget_slot",
            Op::AddVar => "add_var",
            Op::RemoveVar => "remove_var",
            Op::AddPatternEdge => "add_pattern_edge",
            Op::RemovePatternEdge => "remove_pattern_edge",
            Op::SetLabel => "set_label",
            Op::ClearLabel => "clear_label",
            Op::ChangePriority => "change_priority",
        }
    }

    fn from_index(i: usize) -> Self {
        Self::ALL[i % Self::ALL.len()]
    }
}

/// Apply ONE uniformly-chosen single-edit mutation.
pub fn mutate(rule: &CandidateRule, rng: &mut dyn MutationRng) -> (CandidateRule, Applied) {
    mutate_op(rule, rng.below(10), rng)
}

/// Apply a SPECIFIC operator (by dispatch index). Same draws as `mutate`
/// for the same op choice — the split is purely attributional.
pub fn mutate_op(
    rule: &CandidateRule,
    op: usize,
    rng: &mut dyn MutationRng,
) -> (CandidateRule, Applied) {
    match Op::from_index(op) {
        Op::AddSlot => add_slot(rule, rng),
        Op::RemoveSlot => remove_slot(rule, rng),
        Op::RetargetSlot => retarget_slot(rule, rng),
        Op::AddVar => add_var_wired(rule, rng),
        Op::RemoveVar => remove_var(rule, rng),
        Op::AddPatternEdge => add_pattern_edge(rule, rng),
        Op::RemovePatternEdge => remove_pattern_edge(rule, rng),
        Op::SetLabel => set_label(rule, rng),
        Op::ClearLabel => clear_label(rule, rng),
        Op::ChangePriority => change_priority(rule, rng),
    }
}

fn pick_side<'a>(
    rule: &'a CandidateRule,
    rng: &mut dyn MutationRng,
) -> (&'a Vec<CandidatePatternEdge>, bool) {
    if rule.lhs.is_empty() && rule.rhs.is_empty() {
        return (&rule.lhs, true);
    }
    // Weight sides by edge count so edits distribute across both sides even
    // when unbalanced (LHS-heavy rules are the common case post-V1).
    let total = rule.lhs.len() + rule.rhs.len();
    let k = rng.below(total.max(1));
    if k < rule.lhs.len() {
        (&rule.lhs, true)
    } else {
        (&rule.rhs, false)
    }
}

fn add_slot(rule: &CandidateRule, rng: &mut dyn MutationRng) -> (CandidateRule, Applied) {
    let mut r = rule.clone();
    let n_vars = r.n_vars();
    if n_vars == 0 {
        return (r, Applied::Nothing);
    }
    // Pick a side by edge count, then an edge within it.
    let (edges, is_lhs) = {
        let total = r.lhs.len() + r.rhs.len();
        if total == 0 {
            return (r, Applied::Nothing);
        }
        pick_side(&r, rng)
    };
    if edges.is_empty() {
        return (r, Applied::Nothing);
    }
    let ei = rng.below(edges.len());
    let v = rng.below(n_vars);
    let target = if is_lhs { &mut r.lhs } else { &mut r.rhs };
    target[ei].slots.push(v);
    (r, Applied::Edited)
}

fn remove_slot(rule: &CandidateRule, rng: &mut dyn MutationRng) -> (CandidateRule, Applied) {
    let mut r = rule.clone();
    let (edges, is_lhs) = pick_side(&r, rng);
    let with_slots: Vec<usize> = edges
        .iter()
        .enumerate()
        .filter(|(_, e)| !e.slots.is_empty())
        .map(|(i, _)| i)
        .collect();
    if with_slots.is_empty() {
        return (r, Applied::Nothing);
    }
    let ei = with_slots[rng.below(with_slots.len())];
    let si = rng.below(edges[ei].slots.len());
    let target = if is_lhs { &mut r.lhs } else { &mut r.rhs };
    target[ei].slots.remove(si);
    // SB-GE3(b): evicting a slot can orphan a var's last LHS wiring while
    // its guard entries survive — the engine rejects those at compile.
    drop_lhs_orphaned_guards(&mut r);
    (r, Applied::Edited)
}

fn retarget_slot(rule: &CandidateRule, rng: &mut dyn MutationRng) -> (CandidateRule, Applied) {
    let mut r = rule.clone();
    let n_vars = r.n_vars();
    if n_vars < 2 {
        return (r, Applied::Nothing);
    }
    let (edges, is_lhs) = pick_side(&r, rng);
    let with_slots: Vec<usize> = edges
        .iter()
        .enumerate()
        .filter(|(_, e)| !e.slots.is_empty())
        .map(|(i, _)| i)
        .collect();
    if with_slots.is_empty() {
        return (r, Applied::Nothing);
    }
    let ei = with_slots[rng.below(with_slots.len())];
    let si = rng.below(edges[ei].slots.len());
    let new_v = rng.below(n_vars);
    let target = if is_lhs { &mut r.lhs } else { &mut r.rhs };
    target[ei].slots[si] = new_v;
    // SB-GE3(b): same orphaning hazard as remove_slot (see above).
    drop_lhs_orphaned_guards(&mut r);
    (r, Applied::Edited)
}

/// New var appended and immediately wired into one existing edge slot.
fn add_var_wired(rule: &CandidateRule, rng: &mut dyn MutationRng) -> (CandidateRule, Applied) {
    let mut r = rule.clone();
    let (edges, is_lhs) = pick_side(&r, rng);
    if edges.is_empty() {
        return (r, Applied::Nothing);
    }
    let ei = rng.below(edges.len());
    let new_v = r.n_vars();
    let target = if is_lhs { &mut r.lhs } else { &mut r.rhs };
    target[ei].slots.push(new_v);
    r.var_labels.push(None);
    (r, Applied::Edited)
}

fn remove_var(rule: &CandidateRule, rng: &mut dyn MutationRng) -> (CandidateRule, Applied) {
    let mut r = rule.clone();
    let n = r.n_vars();
    if n == 0 {
        return (r, Applied::Nothing);
    }
    let victim = rng.below(n);
    // Renumber all slots > victim down by one; drop references == victim.
    for side in [&mut r.lhs, &mut r.rhs] {
        for e in side.iter_mut() {
            e.slots.retain(|s| *s != victim);
            for s in e.slots.iter_mut() {
                if *s > victim {
                    *s -= 1;
                }
            }
        }
    }
    r.var_labels.remove(victim);
    // E1-G / E1-V / E2-lite carry var references too (post-R5 kernels hit
    // these on every TOML-extracted seed): drop entries naming the victim,
    // renumber survivors. Guards on a removed var are simply gone with it —
    // the same semantics remove_var applies to slots and labels.
    r.guards.retain(|g| g.var != victim);
    for g in r.guards.iter_mut() {
        if g.var > victim {
            g.var -= 1;
        }
    }
    r.var_guards
        .retain(|vg| vg.vars.iter().all(|v| *v != victim));
    for vg in r.var_guards.iter_mut() {
        for v in vg.vars.iter_mut() {
            if *v > victim {
                *v -= 1;
            }
        }
    }
    r.effects
        .retain(|e| e.target != victim && e.source.is_none_or(|s| s != victim));
    for e in r.effects.iter_mut() {
        if e.target > victim {
            e.target -= 1;
        }
        if let Some(s) = e.source.as_mut() {
            if *s > victim {
                *s -= 1;
            }
        }
    }
    (r, Applied::Edited)
}

/// SB-GE3(b): drop guard entries orphaned by slot edits.
///
/// `remove_slot` / `retarget_slot` never change var indices, but they can
/// evict a var's last LHS wiring while its E1-G / E1-V entries survive;
/// the engine rejects those at compile (`InvalidGuard("... not in LHS")`,
/// `compiled.rs`). Drop entries naming vars with no surviving LHS slot —
/// the same semantics [`remove_var`] applies to slots and labels
/// (orphaned guards are gone with their wiring). No renumbering: slot
/// edits keep every index stable. E2-lite effects resolve against RHS
/// indices, so they are untouched.
fn drop_lhs_orphaned_guards(r: &mut CandidateRule) {
    use std::collections::BTreeSet;
    let wired: BTreeSet<usize> = r.lhs.iter().flat_map(|e| e.slots.iter().copied()).collect();
    r.guards.retain(|g| wired.contains(&g.var));
    r.var_guards
        .retain(|vg| vg.vars.iter().all(|v| wired.contains(v)));
}

fn add_pattern_edge(rule: &CandidateRule, rng: &mut dyn MutationRng) -> (CandidateRule, Applied) {
    let mut r = rule.clone();
    let arity = 1 + rng.below(crate::schema::MAX_ARITY.min(3));
    let n_vars = r.n_vars();
    if n_vars == 0 {
        return (r, Applied::Nothing);
    }
    let slots: Vec<usize> = (0..arity).map(|_| rng.below(n_vars)).collect();
    // V6 forbids repeated vars within one edge; sample without replacement
    // by retrying (bounded, deterministic).
    let mut slots_dedup: Vec<usize> = Vec::with_capacity(arity);
    for s in slots {
        if !slots_dedup.contains(&s) {
            slots_dedup.push(s);
        }
    }
    if slots_dedup.is_empty() {
        return (r, Applied::Nothing);
    }
    // Half the time wire into the LHS, else RHS — but V1 needs a non-empty
    // LHS, so bias toward LHS when LHS is empty.
    let lhs_target = r.lhs.is_empty() || rng.below(2) == 0;
    if lhs_target {
        r.lhs.push(CandidatePatternEdge { slots: slots_dedup });
    } else {
        r.rhs.push(CandidatePatternEdge { slots: slots_dedup });
    }
    (r, Applied::Edited)
}

fn remove_pattern_edge(
    rule: &CandidateRule,
    rng: &mut dyn MutationRng,
) -> (CandidateRule, Applied) {
    let mut r = rule.clone();
    // Never empty the LHS entirely (V1 would quarantine): only remove from
    // a side with ≥2 edges, preferring RHS.
    if !r.rhs.is_empty() && (r.lhs.len() <= 1 || rng.below(2) == 0) {
        let ei = rng.below(r.rhs.len());
        r.rhs.remove(ei);
        return (r, Applied::Edited);
    }
    if r.lhs.len() >= 2 {
        let ei = rng.below(r.lhs.len());
        r.lhs.remove(ei);
        return (r, Applied::Edited);
    }
    (r, Applied::Nothing)
}

/// Set a label constraint on some var (existing constraint overwritten —
/// change-label folds into set via uniform var choice).
fn set_label(rule: &CandidateRule, rng: &mut dyn MutationRng) -> (CandidateRule, Applied) {
    let mut r = rule.clone();
    let n = r.n_vars();
    if n == 0 {
        return (r, Applied::Nothing);
    }
    let v = rng.below(n);
    // Small world labels dominate real rule sets: draw 0..8.
    r.var_labels[v] = Some(rng.below(8) as u32);
    (r, Applied::Edited)
}

fn clear_label(rule: &CandidateRule, rng: &mut dyn MutationRng) -> (CandidateRule, Applied) {
    let mut r = rule.clone();
    let constrained: Vec<usize> = r
        .var_labels
        .iter()
        .enumerate()
        .filter_map(|(v, l)| l.map(|_| v))
        .collect();
    if constrained.is_empty() {
        return (r, Applied::Nothing);
    }
    let v = constrained[rng.below(constrained.len())];
    r.var_labels[v] = None;
    (r, Applied::Edited)
}

fn change_priority(rule: &CandidateRule, rng: &mut dyn MutationRng) -> (CandidateRule, Applied) {
    let mut r = rule.clone();
    let delta = (rng.below(7) as i32) - 3; // [-3, 3], 0 possible but rare-harmless
    r.priority = r.priority.saturating_add(delta);
    (r, Applied::Edited)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{CandidateGuard, CandidateVarGuard};
    use hg_core::PropertyValue;

    /// Scripted RNG: yields the script in order (overrun panics — a test
    /// bug, loud by design).
    struct SeqRng {
        vs: Vec<u64>,
        i: usize,
    }
    impl MutationRng for SeqRng {
        fn next_u64(&mut self) -> u64 {
            let v = self.vs[self.i];
            self.i += 1;
            v
        }
    }

    /// Two vars, each with exactly one LHS slot; var 1 carries an E1-G
    /// guard and shares an E1-V entry with var 0.
    fn guarded_pair() -> CandidateRule {
        CandidateRule {
            priority: 0,
            lhs: vec![
                CandidatePatternEdge { slots: vec![0] },
                CandidatePatternEdge { slots: vec![1] },
            ],
            rhs: vec![CandidatePatternEdge { slots: vec![0] }],
            var_labels: vec![None, None],
            guards: vec![CandidateGuard {
                var: 1,
                key: 0,
                op: 0,
                val: PropertyValue::Int(0),
            }],
            var_guards: vec![CandidateVarGuard { vars: vec![0, 1] }],
            effects: vec![],
        }
    }

    #[test]
    fn remove_slot_drops_guards_orphaned_from_lhs() {
        // Draws: side k=0 → LHS; edge idx 1 → var 1's only LHS slot; slot 0.
        let mut rng = SeqRng {
            vs: vec![0, 1, 0],
            i: 0,
        };
        let (out, applied) = remove_slot(&guarded_pair(), &mut rng);
        assert_eq!(applied, Applied::Edited);
        assert!(out.lhs[1].slots.is_empty());
        assert!(out.guards.is_empty(), "orphaned E1-G entry must go");
        assert!(out.var_guards.is_empty(), "orphaned E1-V entry must go");
    }

    #[test]
    fn drop_helper_keeps_guards_on_wired_vars_and_effects() {
        let mut r = guarded_pair();
        // Manually orphan var 1 the way a slot edit would.
        r.lhs[1].slots.clear();
        // A guard on the still-wired var 0 must survive the sweep.
        r.guards.push(CandidateGuard {
            var: 0,
            key: 1,
            op: 2,
            val: PropertyValue::Int(3),
        });
        drop_lhs_orphaned_guards(&mut r);
        assert_eq!(r.guards.len(), 1);
        assert_eq!(r.guards[0].var, 0);
        assert!(r.var_guards.is_empty());
    }

    #[test]
    fn slot_ops_never_leave_guard_refs_off_the_lhs() {
        // Property sweep: every slot-op draw from the guarded pair must
        // end with all guard refs LHS-wired (or the op a Nothing).
        for seed in 0..32u64 {
            for (oi, op) in [
                remove_slot as fn(&CandidateRule, &mut dyn MutationRng) -> (CandidateRule, Applied),
                retarget_slot
                    as fn(&CandidateRule, &mut dyn MutationRng) -> (CandidateRule, Applied),
            ]
            .into_iter()
            .enumerate()
            {
                // Fresh script per op: remove_slot draws 3, retarget 4.
                let mut rng = SeqRng {
                    vs: vec![seed, seed / 2 + 1, seed % 3, seed + 1 + oi as u64],
                    i: 0,
                };
                let (out, _) = op(&guarded_pair(), &mut rng);
                let wired: std::collections::BTreeSet<usize> = out
                    .lhs
                    .iter()
                    .flat_map(|e| e.slots.iter().copied())
                    .collect();
                assert!(out.guards.iter().all(|g| wired.contains(&g.var)));
                assert!(out
                    .var_guards
                    .iter()
                    .all(|vg| vg.vars.iter().all(|v| wired.contains(v))));
            }
        }
    }
}
