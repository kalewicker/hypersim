//! Structural mutation operators over rule-subgraphs (plan §6).
//!
//! Determinism discipline: every `mutate_once` call draws EXACTLY
//! `1 + N_PARAM_DRAWS` u64s from the stream — one for operator choice, a
//! fixed block for parameters — regardless of which operator was chosen and
//! whether it turned out applicable. Draw count is path-independent, so a
//! seed reproduces bit-for-bit no matter how rule content evolved.
//!
//! Operators may produce malformed subgraphs (e.g. `drop_lhs_var` under a
//! PEdge that still references the var): that is allowed junk DNA — the
//! decoder filters it, the economy still charges it rent.

use crate::schema::*;
use crate::store_ext::*;
use hg_core::{HypergraphStore, NodeId, PropertyValue};
use rand::Rng;
use rand_chacha::ChaCha8Rng;

pub const N_OPS: u64 = 15;
pub const N_PARAM_DRAWS: usize = 5;

/// Operator names in `OPS` order (index-stable; used by the G-R1 harness
/// per-op breakdown — do not reorder without bumping the RCF/golden pins
/// that depend on kernel identity).
pub const OP_NAMES: [&str; N_OPS as usize] = [
    "jitter_guard",
    "mutate_label",
    "add_pnode_constraint",
    "drop_pnode_constraint",
    "add_lhs_var",
    "drop_lhs_var",
    "add_pedge",
    "drop_pedge",
    "add_rhs_create",
    "drop_rhs_create",
    "add_rhs_edge",
    "drop_rhs_edge",
    "toggle_delete",
    "grow_arity",
    "shrink_arity",
];

#[derive(Clone, Copy, Debug)]
enum Op {
    JitterGuard,
    MutateLabel,
    AddPnodeConstraint,
    DropPnodeConstraint,
    AddLhsVar,
    DropLhsVar,
    AddPedge,
    DropPedge,
    AddRhsCreate,
    DropRhsCreate,
    AddRhsEdge,
    DropRhsEdge,
    ToggleDelete,
    GrowArity,
    ShrinkArity,
}

const OPS: [Op; N_OPS as usize] = [
    Op::JitterGuard,
    Op::MutateLabel,
    Op::AddPnodeConstraint,
    Op::DropPnodeConstraint,
    Op::AddLhsVar,
    Op::DropLhsVar,
    Op::AddPedge,
    Op::DropPedge,
    Op::AddRhsCreate,
    Op::DropRhsCreate,
    Op::AddRhsEdge,
    Op::DropRhsEdge,
    Op::ToggleDelete,
    Op::GrowArity,
    Op::ShrinkArity,
];

fn pick(param: u64, n: usize) -> usize {
    (param % n.max(1) as u64) as usize
}

/// Draw an operator and its parameter block. ALWAYS consumes
/// `1 + N_PARAM_DRAWS` u64s, whatever is drawn and whether or not the caller
/// goes on to apply it — this is what keeps the stream position independent
/// of μ and of rule content.
fn draw_mutation(rng: &mut ChaCha8Rng) -> (Op, [u64; N_PARAM_DRAWS], usize) {
    let idx = (rng.gen::<u64>() % N_OPS) as usize;
    let op = OPS[idx];
    let mut p = [0u64; N_PARAM_DRAWS];
    for slot in p.iter_mut() {
        *slot = rng.gen::<u64>();
    }
    (op, p, idx)
}

/// Apply exactly one operator (possibly a no-op if inapplicable) to the rule
/// rooted at `root`. Returns true iff the subgraph was changed.
pub fn mutate_once(
    s: &mut HypergraphStore,
    root: NodeId,
    rng: &mut ChaCha8Rng,
    caps: &Caps,
) -> bool {
    let (changed, _) = mutate_once_with_op(s, root, rng, caps);
    changed
}

/// `mutate_once` that also reports the drawn operator's index into
/// `OPS`/`OP_NAMES` (for the G-R1 per-op breakdown). Same draws, same
/// application — the index is observation, not a new draw.
pub fn mutate_once_with_op(
    s: &mut HypergraphStore,
    root: NodeId,
    rng: &mut ChaCha8Rng,
    caps: &Caps,
) -> (bool, usize) {
    let (op, p, idx) = draw_mutation(rng);
    (apply_op(s, root, op, &p, caps), idx)
}

/// Outcome of one birth's variation draw.
pub struct BirthVariation {
    /// The μ-coin came up "mutate" — this is the plan's mutant/clonal split.
    pub coin_mutant: bool,
    /// The operator actually altered the subgraph. `coin_mutant && !changed`
    /// means an inapplicable operator produced an effectively-clonal child;
    /// the gap between the two is the *effective* variation rate.
    pub changed: bool,
}

/// Probabilistic mutate-at-birth (v4 §4).
///
/// **Draw order, fixed for every birth regardless of μ:** one f64 μ-coin,
/// then the full operator+parameter block via `draw_mutation`. Both are drawn
/// unconditionally, so a run at μ = 0 and a run at μ = 1 traverse the RNG
/// stream identically up to the point where their dynamics diverge. Changing
/// μ therefore changes the *decision*, never the stream alignment.
pub fn mutate_at_birth(
    s: &mut HypergraphStore,
    child: NodeId,
    rng: &mut ChaCha8Rng,
    caps: &Caps,
    mu: f64,
) -> BirthVariation {
    let coin: f64 = rng.gen();
    let (op, p, _) = draw_mutation(rng);
    if coin < mu {
        let changed = apply_op(s, child, op, &p, caps);
        BirthVariation {
            coin_mutant: true,
            changed,
        }
    } else {
        BirthVariation {
            coin_mutant: false,
            changed: false,
        }
    }
}

fn n_vars(s: &HypergraphStore, root: NodeId) -> i64 {
    get_int(s, root, K_NVARS).unwrap_or(1).max(1)
}

fn apply_op(
    s: &mut HypergraphStore,
    root: NodeId,
    op: Op,
    p: &[u64; N_PARAM_DRAWS],
    caps: &Caps,
) -> bool {
    if s.get_node(root).is_none() {
        return false;
    }
    let nv = n_vars(s, root);
    let labels = caps.n_labels as u64;
    match op {
        Op::JitterGuard => {
            let guards = children(s, root, HAS_GUARD);
            if guards.is_empty() {
                return false;
            }
            let gn = guards[pick(p[0], guards.len())];
            let Some(v) = s.get_node(gn).and_then(|d| d.get_property(K_GVAL)) else {
                return false;
            };
            let nvv = match v {
                PropertyValue::Float(x) => {
                    // uniform in [-0.1, 0.1] from a raw u64 draw
                    let u = (p[1] >> 11) as f64 / (1u64 << 53) as f64;
                    PropertyValue::Float(x + (u - 0.5) * 0.2)
                }
                PropertyValue::Int(x) => PropertyValue::Int(x + (p[1] % 3) as i64 - 1),
                b => b,
            };
            if let Some(d) = s.get_node_mut(gn) {
                d.set_property(K_GVAL, nvv);
            }
            true
        }
        Op::MutateLabel => {
            // Targets: create labels, relabel targets, pnode constraints.
            let mut targets: Vec<(NodeId, u32)> = Vec::new();
            for cn in children(s, root, HAS_CREATE) {
                targets.push((cn, K_CREATE_LABEL));
            }
            for rn in children(s, root, HAS_RELABEL) {
                targets.push((rn, K_NEW_LABEL));
            }
            for pn in children(s, root, HAS_PNODE) {
                targets.push((pn, K_LABEL_CONSTRAINT));
            }
            if targets.is_empty() {
                return false;
            }
            let (n, key) = targets[pick(p[0], targets.len())];
            set_int(s, n, key, (p[1] % labels) as i64);
            true
        }
        Op::AddPnodeConstraint => {
            let vid = (p[0] % nv as u64) as i64;
            let existing = children(s, root, HAS_PNODE)
                .into_iter()
                .find(|&pn| get_int(s, pn, K_VAR_ID) == Some(vid));
            match existing {
                Some(pn) => {
                    set_int(s, pn, K_LABEL_CONSTRAINT, (p[1] % labels) as i64);
                }
                None => {
                    let pn = add_rule_node(
                        s,
                        PNODE,
                        vec![
                            (K_VAR_ID, PropertyValue::Int(vid)),
                            (
                                K_LABEL_CONSTRAINT,
                                PropertyValue::Int((p[1] % labels) as i64),
                            ),
                        ],
                    );
                    add_role_edge(s, root, pn, HAS_PNODE);
                }
            }
            true
        }
        Op::DropPnodeConstraint => {
            let pnodes = children(s, root, HAS_PNODE);
            if pnodes.is_empty() {
                return false;
            }
            s.remove_node(pnodes[pick(p[0], pnodes.len())]);
            true
        }
        Op::AddLhsVar => {
            if nv as usize >= caps.lhs_max_nodes {
                return false;
            }
            set_int(s, root, K_NVARS, nv + 1);
            // Optionally (half the time) attach one binary PEdge to an
            // existing var so the new var is not isolated.
            if p[0] % 2 == 0 {
                let other = (p[1] % nv as u64) as i64;
                let pe = add_rule_node(
                    s,
                    PEDGE,
                    vec![
                        (K_ARITY, PropertyValue::Int(2)),
                        (K_EP0, PropertyValue::Int(nv)),
                        (K_EP0 + 1, PropertyValue::Int(other)),
                    ],
                );
                add_role_edge(s, root, pe, HAS_PEDGE);
            }
            true
        }
        Op::DropLhsVar => {
            // Drop the highest var id: decrement n_vars. Anything still
            // referencing it makes the rule malformed — allowed junk DNA.
            if nv <= 1 {
                return false;
            }
            set_int(s, root, K_NVARS, nv - 1);
            true
        }
        Op::AddPedge => {
            let pedges = children(s, root, HAS_PEDGE);
            if pedges.len() >= caps.lhs_max_edges {
                return false;
            }
            let arity = 2 + (p[0] % (caps.max_arity as u64 - 1)) as usize;
            let mut props = vec![(K_ARITY, PropertyValue::Int(arity as i64))];
            for i in 0..arity {
                props.push((
                    K_EP0 + i as u32,
                    PropertyValue::Int((p[1 + i.min(3)] % nv as u64) as i64),
                ));
            }
            let pe = add_rule_node(s, PEDGE, props);
            add_role_edge(s, root, pe, HAS_PEDGE);
            true
        }
        Op::DropPedge => {
            let pedges = children(s, root, HAS_PEDGE);
            if pedges.is_empty() {
                return false;
            }
            s.remove_node(pedges[pick(p[0], pedges.len())]);
            true
        }
        Op::AddRhsCreate => {
            let creates = children(s, root, HAS_CREATE);
            if creates.len() >= caps.rhs_max_create {
                return false;
            }
            // Next index = current count (decoder requires exactly 0..len;
            // if drop_rhs_create left a gap the rule is junk until repaired).
            let cn = add_rule_node(
                s,
                CREATE,
                vec![
                    (K_CREATE_INDEX, PropertyValue::Int(creates.len() as i64)),
                    (K_CREATE_LABEL, PropertyValue::Int((p[0] % labels) as i64)),
                ],
            );
            add_role_edge(s, root, cn, HAS_CREATE);
            true
        }
        Op::DropRhsCreate => {
            // Drop the highest-index create so indices stay 0..len (dangling
            // REdge refs to it still make the rule junk — allowed).
            let creates = children(s, root, HAS_CREATE);
            let Some((cn, _)) = creates
                .into_iter()
                .filter_map(|cn| get_int(s, cn, K_CREATE_INDEX).map(|k| (cn, k)))
                .max_by_key(|(_, k)| *k)
            else {
                return false;
            };
            s.remove_node(cn);
            true
        }
        Op::AddRhsEdge => {
            let n_creates = children(s, root, HAS_CREATE).len() as u64;
            let n_refs = nv as u64 + n_creates;
            let arity = 2 + (p[0] % (caps.max_arity as u64 - 1)) as usize;
            let mut props = vec![(K_ARITY, PropertyValue::Int(arity as i64))];
            for i in 0..arity {
                props.push((
                    K_EP0 + i as u32,
                    PropertyValue::Int((p[1 + i.min(3)] % n_refs) as i64),
                ));
            }
            let re = add_rule_node(s, REDGE, props);
            add_role_edge(s, root, re, HAS_REDGE);
            true
        }
        Op::DropRhsEdge => {
            let redges = children(s, root, HAS_REDGE);
            if redges.is_empty() {
                return false;
            }
            s.remove_node(redges[pick(p[0], redges.len())]);
            true
        }
        Op::ToggleDelete => {
            let vid = (p[0] % nv as u64) as i64;
            let existing = children(s, root, HAS_DELETE)
                .into_iter()
                .find(|&dn| get_int(s, dn, K_VAR_ID) == Some(vid));
            match existing {
                Some(dn) => s.remove_node(dn),
                None => {
                    let dn = add_rule_node(s, DELETE, vec![(K_VAR_ID, PropertyValue::Int(vid))]);
                    add_role_edge(s, root, dn, HAS_DELETE);
                }
            }
            true
        }
        Op::GrowArity | Op::ShrinkArity => {
            let mut edges: Vec<(NodeId, bool)> = Vec::new(); // (node, is_pedge)
            for pe in children(s, root, HAS_PEDGE) {
                edges.push((pe, true));
            }
            for re in children(s, root, HAS_REDGE) {
                edges.push((re, false));
            }
            if edges.is_empty() {
                return false;
            }
            let (en, is_pedge) = edges[pick(p[0], edges.len())];
            let Some(arity) = get_int(s, en, K_ARITY) else {
                return false;
            };
            if matches!(op, Op::GrowArity) {
                if arity as usize >= caps.max_arity {
                    return false;
                }
                let n_refs = if is_pedge {
                    nv as u64
                } else {
                    nv as u64 + children(s, root, HAS_CREATE).len() as u64
                };
                set_int(s, en, K_EP0 + arity as u32, (p[1] % n_refs) as i64);
                set_int(s, en, K_ARITY, arity + 1);
            } else {
                if arity <= 2 {
                    return false;
                }
                // Shrink: decrement arity; the stale K_EP prop beyond the new
                // arity is simply ignored by the decoder's read window.
                set_int(s, en, K_ARITY, arity - 1);
            }
            true
        }
    }
}

// `fission_mutated` (v3) is gone: fission now lives in `economy::funded_fission`,
// which owns the μ-coin, lineage depth, and token transfer together.
