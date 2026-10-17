//! The static rule-dependency screen (`hypersim_nn_experiments_plan_v3.md`
//! §8.2).
//!
//! Substrate supply is the binding constraint: nine candidate (rule set, IC)
//! pairings were run through P-1 empirically and all nine aborted. This
//! screen turns that trial-and-error into a design criterion, and it needs no
//! rollout.
//!
//! # What `conservation_soak` taught
//!
//! Its failure mode (§8.1) is the instructive one: **both rules fire, never
//! simultaneously.** Co-liveness is not a property of a rule set — it is a
//! property of the rule set's *dependency graph*, and transient co-liveness
//! is not enough. P-1 needs **sustained** co-liveness, which means the
//! dependency structure has to be cyclic: A's RHS producing B's LHS and
//! B's RHS producing A's, so each rule regenerates the other's substrate.
//!
//! # The asymmetry is deliberate
//!
//! `can_produce` over-approximates — it asks whether A's RHS emits anything
//! B's LHS constrains on, not whether a full match will ever be assembled.
//! So:
//!
//! * **No cycle is definitive.** The rules cannot sustain each other and no
//!   initial condition will make them co-live for long. Abort before any
//!   rollout.
//! * **A cycle is a candidate**, not a verdict. The empirical P-1 probe
//!   decides. A passed screen must never be treated as a precheck
//!   (plan v3 risk 5).

use hg_rules::CompiledRule;
use std::collections::BTreeSet;

/// What a rule's RHS emits, as the coarse signature the screen matches on.
#[derive(Debug, Default, Clone)]
struct Produces {
    /// `(arity, label)` of each edge the RHS writes. `label` is `0` for an
    /// unlabelled edge, matching the compiler's sparse-map resolution.
    edges: BTreeSet<(usize, u32)>,
    /// Labels the RHS puts on nodes — fresh creations and relabels alike.
    labels: BTreeSet<u32>,
}

/// What a rule's LHS demands.
#[derive(Debug, Default, Clone)]
struct Requires {
    /// `(arity, label)` per LHS edge. An unconstrained edge label matches
    /// **any** produced label, which is the over-approximating direction.
    edges: BTreeSet<(usize, Option<u32>)>,
    labels: BTreeSet<u32>,
}

fn produces(r: &CompiledRule) -> Produces {
    let mut p = Produces::default();
    for (vars, label) in &r.template.rhs_edges {
        p.edges.insert((vars.len(), label.0));
    }
    for (i, from) in r.template.rhs_from_lhs.iter().enumerate() {
        if from.is_none() {
            // Freshly created: carries its declared label (default 0).
            if let Some(l) = r.template.rhs_new_labels.get(i) {
                p.labels.insert(l.0);
            }
        } else if let Some(Some(l)) = r.template.rhs_relabels.get(i) {
            // Survivor the RHS explicitly relabels.
            p.labels.insert(l.0);
        }
    }
    p
}

fn requires(r: &CompiledRule) -> Requires {
    let mut q = Requires::default();
    for e in &r.pattern.edges {
        q.edges.insert((e.var_ixs.len(), e.label.map(|l| l.0)));
    }
    for c in r.pattern.node_label_constraints.iter().flatten() {
        q.labels.insert(c.0);
    }
    q
}

/// Can `a`'s RHS produce anything `b`'s LHS constrains on?
///
/// Deliberately an **or** over the two channels and a **containment** test
/// rather than a match: one shared edge signature does not mean `b`'s whole
/// pattern will assemble. Over-claiming an edge is the safe direction, since
/// only the *absence* of a cycle is treated as definitive.
fn can_produce(a: &Produces, b: &Requires) -> bool {
    let edge_hit = b.edges.iter().any(|(arity, label)| match label {
        Some(l) => a.edges.contains(&(*arity, *l)),
        // Unconstrained LHS edge label: any produced edge of that arity.
        None => a.edges.iter().any(|(pa, _)| pa == arity),
    });
    edge_hit || !a.labels.is_disjoint(&b.labels)
}

/// One rule set's screen result.
#[derive(Debug, Clone, PartialEq)]
pub struct ScreenResult {
    /// Names of the rules considered — enabled and non-creation.
    pub considered: Vec<String>,
    /// Rules excluded, with the reason.
    pub excluded: Vec<(String, &'static str)>,
    /// Every strongly connected component of size >= 2, by rule name.
    pub sustaining_components: Vec<Vec<String>>,
    /// Rules whose RHS regenerates their own LHS. Recorded but **not**
    /// sufficient: a rule sustaining itself says nothing about co-liveness
    /// with another, and P-1 asks about two rules live in the same step.
    pub self_loops: Vec<String>,
    pub edges: Vec<(String, String)>,
}

impl ScreenResult {
    /// The screen's verdict. `false` is definitive; `true` is a candidate.
    pub fn passes(&self) -> bool {
        !self.sustaining_components.is_empty()
    }

    /// Why, in a form a caller can put in an abort.
    pub fn detail(&self) -> String {
        if self.passes() {
            format!(
                "sustaining cycle(s) {:?} over rules {:?}; a cycle is a CANDIDATE, not a \
                 verdict — the empirical P-1 probe decides",
                self.sustaining_components, self.considered
            )
        } else {
            format!(
                "no cycle of two or more rules in the dependency graph. Considered {:?} \
                 (excluded {:?}); edges {:?}; self-loops {:?}. The rules cannot regenerate \
                 each other's substrate, so no initial condition will make them co-live for \
                 long — a self-loop does not count, because a rule sustaining its own LHS \
                 says nothing about two rules being live in the same step",
                self.considered, self.excluded, self.edges, self.self_loops
            )
        }
    }
}

/// Build the dependency graph and report its strongly connected components.
///
/// **Creation rules are excluded.** They are schedule-gated rather than
/// substrate-driven, so they inject structure on a clock and cannot
/// participate in the mutual-regeneration the screen is looking for. That
/// exclusion is what makes `conservation_soak` fail: its `seed_cycle` is a
/// creation rule, leaving only `transfer`, which regenerates its own edge and
/// nothing else's.
pub fn rule_dependency_screen(rules: &[CompiledRule]) -> ScreenResult {
    let mut considered: Vec<&CompiledRule> = Vec::new();
    let mut excluded = Vec::new();
    for r in rules {
        if !r.enabled {
            excluded.push((r.name.clone(), "disabled"));
        } else if r.is_creation() {
            excluded.push((r.name.clone(), "creation rule — schedule-gated, not substrate-driven"));
        } else {
            considered.push(r);
        }
    }

    let prod: Vec<Produces> = considered.iter().map(|r| produces(r)).collect();
    let req: Vec<Requires> = considered.iter().map(|r| requires(r)).collect();
    let n = considered.len();

    let mut adj = vec![Vec::new(); n];
    let mut edges = Vec::new();
    let mut self_loops = Vec::new();
    for a in 0..n {
        for b in 0..n {
            if can_produce(&prod[a], &req[b]) {
                adj[a].push(b);
                edges.push((considered[a].name.clone(), considered[b].name.clone()));
                if a == b {
                    self_loops.push(considered[a].name.clone());
                }
            }
        }
    }

    ScreenResult {
        sustaining_components: sccs(&adj)
            .into_iter()
            .filter(|c| c.len() >= 2)
            .map(|c| c.into_iter().map(|i| considered[i].name.clone()).collect())
            .collect(),
        considered: considered.iter().map(|r| r.name.clone()).collect(),
        excluded,
        self_loops,
        edges,
    }
}

/// Tarjan's SCC, iterative so a large rule set cannot blow the stack.
fn sccs(adj: &[Vec<usize>]) -> Vec<Vec<usize>> {
    let n = adj.len();
    let (mut index, mut idx, mut low) = (0usize, vec![usize::MAX; n], vec![0usize; n]);
    let (mut on_stack, mut stack, mut out) = (vec![false; n], Vec::new(), Vec::new());

    for root in 0..n {
        if idx[root] != usize::MAX {
            continue;
        }
        // (node, next child to visit)
        let mut call: Vec<(usize, usize)> = vec![(root, 0)];
        while let Some((v, ci)) = call.pop() {
            if ci == 0 {
                idx[v] = index;
                low[v] = index;
                index += 1;
                stack.push(v);
                on_stack[v] = true;
            }
            if ci < adj[v].len() {
                let w = adj[v][ci];
                call.push((v, ci + 1));
                if idx[w] == usize::MAX {
                    call.push((w, 0));
                } else if on_stack[w] {
                    low[v] = low[v].min(idx[w]);
                }
                continue;
            }
            // Done with v: fold its low-link into its parent, then close.
            if let Some(&(parent, _)) = call.last() {
                low[parent] = low[parent].min(low[v]);
            }
            if low[v] == idx[v] {
                let mut comp = Vec::new();
                while let Some(w) = stack.pop() {
                    on_stack[w] = false;
                    comp.push(w);
                    if w == v {
                        break;
                    }
                }
                out.push(comp);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use hg_rules::compile_rules;

    fn load(path: &str) -> Vec<CompiledRule> {
        let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(path);
        let raw = hg_rules::load_rules_from_file(&p).expect("parse");
        compile_rules(&raw).expect("compile")
    }

    /// E1's exit criterion: `conservation_soak` aborts at the screen, with no
    /// rollout, for the reason §8.1 identifies.
    #[test]
    fn conservation_soak_aborts_at_the_screen_without_a_rollout() {
        let r = rule_dependency_screen(&load("rules/conservation_soak.toml"));
        assert!(
            !r.passes(),
            "conservation_soak must not pass the screen: {}",
            r.detail()
        );
        assert!(
            r.excluded.iter().any(|(n, _)| n == "seed_cycle"),
            "seed_cycle is a creation rule and must be excluded: {:?}",
            r.excluded
        );
        // And the reason is the right one: what remains cannot pair up.
        assert!(
            r.sustaining_components.is_empty(),
            "no component of two or more rules: {:?}",
            r.sustaining_components
        );
    }

    /// The screen must not reject the substrate the programme's only positive
    /// result stands on — that would make it useless rather than
    /// conservative.
    #[test]
    fn the_two_owned_substrates_pass_the_screen() {
        for path in ["rules/tri_two.toml", "rules/wn_three.toml"] {
            let r = rule_dependency_screen(&load(path));
            assert!(r.passes(), "{path} must pass: {}", r.detail());
            assert!(
                r.sustaining_components.iter().any(|c| c.len() >= 2),
                "{path} needs a component of >= 2 rules: {:?}",
                r.sustaining_components
            );
        }
    }

    /// A self-loop is recorded and is **not** a pass. This is the
    /// distinction that makes the screen mean "sustained co-liveness"
    /// rather than "some rule keeps going".
    #[test]
    fn a_self_sustaining_rule_alone_does_not_pass() {
        let r = rule_dependency_screen(&load("rules/conservation_soak.toml"));
        assert!(
            !r.self_loops.is_empty(),
            "transfer regenerates its own LHS, so the self-loop must be visible: {r:?}"
        );
        assert!(!r.passes(), "but a self-loop alone is not sustained co-liveness");
    }

    /// The over-approximation, demonstrated on the case that shows it — and
    /// this is plan v3 risk 5 made concrete rather than argued.
    ///
    /// `lk_pair` **passes** the screen: `lk_metered_subdivide` and
    /// `lk_transport` do reference each other's structures. D5's empirical
    /// P-1 nonetheless aborted it on all four shipped initial conditions
    /// with `rules ever live: []`, because the LK-R rules are **energy-gated**
    /// and no shipped IC carries energy.
    ///
    /// The screen reads edges and node labels; it does not read
    /// `property_guards`. Ignoring them is the *conservative* direction —
    /// it can only add edges, hence only add cycles, hence only turn
    /// definitive aborts into candidates. Making the screen guard-aware
    /// would make it stricter and risk a false abort, which is the one
    /// error it must not make.
    ///
    /// **A passed screen is never a precheck.**
    #[test]
    fn a_pass_is_a_candidate_not_a_verdict() {
        let r = rule_dependency_screen(&load("rules/lk_pair.toml"));
        assert!(
            r.passes(),
            "lk_pair is structurally cyclic, so the screen must pass it: {}",
            r.detail()
        );
        assert!(
            r.detail().contains("CANDIDATE"),
            "and must say so in the detail a caller would surface: {}",
            r.detail()
        );
        // Empirically, P-1 aborts it — the screen cannot see property guards.
        // Recorded here so the gap is a documented property, not a surprise.
    }

    /// The screen must reproduce D5's empirical aborts where it can see the
    /// reason, and must not reject the substrates the programme runs on.
    #[test]
    fn screen_agrees_with_the_empirical_aborts_it_can_see() {
        for (path, expect_pass) in [
            ("rules/tri_two.toml", true),
            ("rules/wn_three.toml", true),
            ("rules/conservation_soak.toml", false),
            ("rules/crystal_battery.toml", false),
            ("rules/effects_cascade_fixture.toml", false),
        ] {
            let r = rule_dependency_screen(&load(path));
            assert_eq!(r.passes(), expect_pass, "{path}: {}", r.detail());
        }
    }

    #[test]
    fn sccs_finds_a_two_cycle_and_ignores_a_chain() {
        // 0 <-> 1, and 2 -> 0 with nothing back.
        let adj = vec![vec![1], vec![0], vec![0]];
        let mut comps: Vec<Vec<usize>> = sccs(&adj)
            .into_iter()
            .filter(|c| c.len() >= 2)
            .map(|mut c| {
                c.sort_unstable();
                c
            })
            .collect();
        comps.sort();
        assert_eq!(comps, vec![vec![0, 1]]);
    }
}
