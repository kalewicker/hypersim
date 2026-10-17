//! Rule Canonical Form (plan §4): normal form modulo variable renaming and
//! node relabeling, so novelty counts distinguish *rules*, not copies.
//!
//! §4.2 algorithm:
//! 1. extract region → `CandidateRule` (the reader's job);
//! 2. drop node identity — vars are slots, pattern edges are (side, slots);
//! 3. WL-style refinement over the rule's own bipartite var/edge graph,
//!    initial colors from (side-membership multiset, label constraint,
//!    degree);
//! 4. order vars by refined color; residual ties broken lexicographically on
//!    the sorted multiset of incident (side, edge-color, slot_k) triples,
//!    then by stable index — RNG-free;
//! 5. emit bytes: priority, LHS edges sorted, RHS edges sorted, label
//!    constraints; hash with a fixed-seed FxHasher.
//!
//! Incompleteness (SB-R3): WL refinement does not separate all
//! non-isomorphic graphs, so RCF can collide two genuinely distinct rules.
//! It cannot separate identical ones — RCF UNDER-REPORTS novelty, which is
//! the safe direction, stated in every novelty result.

use crate::schema::CandidateRule;

/// A side tag for an incidence triple.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Side {
    L,
    R,
}

impl Side {
    fn byte(self) -> u8 {
        match self {
            Side::L => 0,
            Side::R => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Incidence {
    side: Side,
    /// Index into the working edge list (0..n_edges), NOT the original order
    /// in the candidate (which may vary by insertion order).
    edge: usize,
    slot_k: usize,
}

/// One var's refinement signature at a round.
type Sig = Vec<(u64, u8, usize)>;

/// The canonical byte string before hashing — exposed so fixtures can assert
/// on structure, not just the final u64.
pub fn rcf_bytes(rule: &CandidateRule) -> Vec<u8> {
    // ---- collect the bipartite incidence structure ----
    // Edge list first (LHS then RHS, in candidate order — edge ORDER within
    // a side does not survive emission, which sorts rows), incidences per var.
    let mut edges: Vec<(Side, Vec<usize>)> = Vec::new();
    let mut incidences: Vec<(usize, Incidence)> = Vec::new();
    for (side, es) in [(Side::L, &rule.lhs), (Side::R, &rule.rhs)] {
        for e in es.iter() {
            let eid = edges.len();
            edges.push((side, e.slots.clone()));
            for (k, v) in e.slots.iter().enumerate() {
                incidences.push((
                    *v,
                    Incidence {
                        side,
                        edge: eid,
                        slot_k: k,
                    },
                ));
            }
        }
    }
    let n_vars = rule.n_vars();

    // ---- initial colors: (side-multiset, label constraint, degree) ----
    // Full label fidelity lives in the Option<u32> component; the byte
    // multiset alone would coarsen labels > 255.
    let init_full = |v: usize| -> (Sig, Option<u32>) {
        let mut sides: Sig = incidences
            .iter()
            .filter(|(vi, _)| *vi == v)
            .map(|(_, i)| (i.side.byte() as u64, 0, 0))
            .collect();
        sides.sort();
        let deg = sides.len() as u64;
        sides.push((deg, 0, 0));
        (sides, rule.var_labels.get(v).copied().flatten())
    };

    let sigs: Vec<(Sig, Option<u32>)> = (0..n_vars).map(init_full).collect();
    let mut color = dense_rank_pair(&sigs);

    // ---- WL refinement to stability ----
    loop {
        // Edge color = min var color on the edge (bipartite WL).
        let edge_color: Vec<u64> = edges
            .iter()
            .map(|(_, vs)| vs.iter().map(|v| color[*v]).min().unwrap_or(u64::MAX))
            .collect();
        let mut next_sigs: Vec<(Sig, Option<u32>)> = Vec::with_capacity(n_vars);
        for v in 0..n_vars {
            let mut s: Sig = incidences
                .iter()
                .filter(|(vi, _)| *vi == v)
                .map(|(_, i)| (edge_color[i.edge], i.side.byte(), i.slot_k))
                .collect();
            s.sort();
            next_sigs.push((s, rule.var_labels.get(v).copied().flatten()));
        }
        let new_color = dense_rank_pair(&next_sigs);
        let changed = new_color != color;
        color = new_color;
        if !changed {
            break;
        }
    }

    // ---- var renumbering: sort by (color, final incident-triple multiset) ----
    let edge_color: Vec<u64> = edges
        .iter()
        .map(|(_, vs)| vs.iter().map(|v| color[*v]).min().unwrap_or(u64::MAX))
        .collect();
    let mut var_order: Vec<usize> = (0..n_vars).collect();
    var_order.sort_by_cached_key(|v| {
        let mut triples: Sig = incidences
            .iter()
            .filter(|(vi, _)| *vi == *v)
            .map(|(_, i)| (edge_color[i.edge], i.side.byte(), i.slot_k))
            .collect();
        triples.sort();
        (color[*v], triples, rule.var_labels[*v])
    });
    let mut rename = vec![usize::MAX; n_vars];
    for (new, old) in var_order.iter().enumerate() {
        rename[*old] = new;
    }

    // ---- emit bytes ----
    let mut out = Vec::with_capacity(32 + edges.len() * 12);
    out.extend_from_slice(&rule.priority.to_le_bytes());
    out.push(b'|');
    emit_side(&mut out, &rule.lhs, &rename);
    out.push(b'|');
    emit_side(&mut out, &rule.rhs, &rename);
    out.push(b'|');
    let mut labels: Vec<(usize, u32)> = rule
        .var_labels
        .iter()
        .enumerate()
        .filter_map(|(v, l)| l.map(|x| (rename[v], x)))
        .collect();
    labels.sort();
    for (v, l) in labels {
        out.extend_from_slice(&(v as u64).to_le_bytes());
        out.extend_from_slice(&l.to_le_bytes());
        out.push(b',');
    }
    out
}

fn emit_side(out: &mut Vec<u8>, side: &[crate::schema::CandidatePatternEdge], rename: &[usize]) {
    let mut rows: Vec<Vec<usize>> = side
        .iter()
        .map(|e| e.slots.iter().map(|s| rename[*s]).collect())
        .collect();
    rows.sort();
    for r in rows {
        for s in r {
            out.extend_from_slice(&(s as u64).to_le_bytes());
            out.push(b'.');
        }
        out.push(b';');
    }
}

/// Dense ranking over (signature, label) pairs. Equal signatures share a
/// color; distinct get consecutive — fully deterministic.
fn dense_rank_pair(sigs: &[(Sig, Option<u32>)]) -> Vec<u64> {
    let mut order: Vec<usize> = (0..sigs.len()).collect();
    order.sort_by(|a, b| sigs[*a].cmp(&sigs[*b]));
    let mut colors = vec![0u64; sigs.len()];
    let mut next = 0u64;
    for w in order.windows(2) {
        colors[w[0]] = next;
        if sigs[w[0]] != sigs[w[1]] {
            next += 1;
        }
    }
    if let Some(&last) = order.last() {
        colors[last] = next;
    }
    colors
}

/// RCF hash — fixed-seed FxHasher over [`rcf_bytes`], matching the
/// workspace's `class_hash` convention.
pub fn rcf_hash(rule: &CandidateRule) -> u64 {
    use rustc_hash::FxHasher;
    use std::hash::{Hash, Hasher};
    let bytes = rcf_bytes(rule);
    let mut h = FxHasher::default();
    0x5E1F_A0D5_E7E5_0001u64.hash(&mut h);
    bytes.hash(&mut h);
    h.finish()
}
