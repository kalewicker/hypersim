//! Rule-subgraph encoding vocabulary: labels, property keys, edge roles.
//!
//! Rules live in a SEPARATE `HypergraphStore` instance (the RuleStore), not
//! the world store. v3's meta-operators are procedural, so nothing matches
//! against rule-subgraphs yet; a separate store keeps the world pristine for
//! hg-observe and keeps hg-engine untouched. Same-store co-residence becomes
//! necessary only at the full-reflection milestone (meta-rules matched by the
//! same engine), which is out of scope for v3 — migrating this crate onto the
//! world store is a known, planned break.

use hg_core::{PropertyOp, PropertyValue};
use serde::{Deserialize, Serialize};

// ---------- rule-space node labels ----------
pub const RULE_ROOT: u32 = 1000;
pub const PNODE: u32 = 1003;
pub const GUARD: u32 = 1005;
pub const RELABEL: u32 = 1006;
pub const CREATE: u32 = 1007;
pub const REDGE: u32 = 1008;
pub const PEDGE: u32 = 1009;
pub const DELETE: u32 = 1010;

// ---------- rule-space edge roles (labels >= 2000) ----------
pub const HAS_PNODE: u32 = 2002;
pub const HAS_GUARD: u32 = 2004;
pub const HAS_RELABEL: u32 = 2005;
pub const HAS_CREATE: u32 = 2006;
pub const HAS_REDGE: u32 = 2007;
pub const HAS_PEDGE: u32 = 2008;
pub const HAS_DELETE: u32 = 2009;
pub const ROLE_BASE: u32 = 2000;

// ---------- property keys (rule space) ----------
pub const K_PRIORITY: u32 = 1;
pub const K_NVARS: u32 = 2;
pub const K_VAR_ID: u32 = 3;
pub const K_LABEL_CONSTRAINT: u32 = 4;
pub const K_GVAR: u32 = 5;
pub const K_GKEY: u32 = 6;
pub const K_GOP: u32 = 7; // 0 Gt, 1 Lt, 2 Eq (v2 convention)
pub const K_GVAL: u32 = 8;
pub const K_NEW_LABEL: u32 = 9;
pub const K_CREATE_INDEX: u32 = 10;
pub const K_CREATE_LABEL: u32 = 11;
pub const K_ARITY: u32 = 12;
/// Ordered endpoint refs of a PEdge/REdge: keys K_EP0 + i, i < arity.
pub const K_EP0: u32 = 20;
/// RuleRoot token count — the conserved resource.
pub const K_RESOURCE: u32 = 200;
/// RuleRoot creation counter (canonical-enumeration tie-break).
pub const K_SEQ: u32 = 201;
/// Per-lineage creation allowance (v4 F3). Depletes on NET node/edge
/// creation by this rule, refills at a slow fixed rate. Inherited by
/// offspring through the subgraph copy, so it is per-lineage by
/// construction — a grower exhausts its own allowance without locking
/// coresident churners out (v3 E1b's failure mode).
pub const K_CREATION_BUDGET: u32 = 202;
/// Fission genealogy depth: seeded rules 0, child = parent + 1.
pub const K_DEPTH: u32 = 203;
/// Which seed-population member this rule descends from (v5). Inherited
/// through the subgraph copy, so it labels a lineage, not an instance —
/// this is what makes copy number, stoichiometric deviation and invasion
/// share measurable.
pub const K_MEMBER: u32 = 204;
/// Step of this rule's last fission (v5 `refractory` fission mode).
pub const K_LAST_FISSION: u32 = 205;
/// Match candidates this rule generated on the PREVIOUS step. Match-
/// proportional cost is charged against this rather than the current step's
/// count, so the rent/starvation phase keeps its v4 position in the step
/// order and the gradient arm stays order-comparable to the v4 arm.
pub const K_LAST_CANDIDATES: u32 = 206;
/// Member id offset for a hand-seeded invader (G5.3), keeping its lineage
/// distinguishable from the resident it competes with.
pub const INVADER_MEMBER_BASE: i64 = 100;

// ---------- property keys (world space) ----------
pub const K_ENERGY: u32 = 100;

// ---------- size caps (experiment parameters, not schema limits) ----------
#[derive(Clone, Copy, Debug)]
pub struct Caps {
    pub lhs_max_nodes: usize,
    pub lhs_max_edges: usize,
    pub max_arity: usize,
    pub rhs_max_create: usize,
    /// Number of distinct world labels the mutators draw from.
    pub n_labels: u32,
}

impl Default for Caps {
    fn default() -> Self {
        Caps {
            lhs_max_nodes: 4,
            lhs_max_edges: 4,
            max_arity: 3,
            rhs_max_create: 3,
            n_labels: 4,
        }
    }
}

// ---------- the decoded intermediate form ----------

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GuardSpec {
    pub var: u8,
    pub key: u32,
    pub op: u8, // 0 Gt, 1 Lt, 2 Eq
    pub val: PropertyValue,
}

impl GuardSpec {
    pub fn property_op(&self) -> Option<PropertyOp> {
        match self.op {
            0 => Some(PropertyOp::Gt),
            1 => Some(PropertyOp::Lt),
            2 => Some(PropertyOp::Eq),
            _ => None,
        }
    }
}

/// Generalized rule spec — the meeting point of encode/decode and the
/// engine-native compile. LHS pattern edges are consumed on apply (engine
/// rewrite semantics); edge preservation means re-emitting on the RHS.
/// RHS edge refs: `0..n_vars` = LHS vars (must survive), `n_vars..` = creates.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RuleSpec {
    pub priority: i32,
    pub n_vars: u8,
    pub var_labels: Vec<Option<u32>>, // len == n_vars
    pub pedges: Vec<Vec<u8>>,         // ordered var refs, arity 2..=max
    pub guards: Vec<GuardSpec>,
    pub deletes: Vec<u8>, // sorted, deduped by decoder
    pub relabels: Vec<(u8, u32)>,
    pub creates: Vec<u32>,    // labels, by create index
    pub redges: Vec<Vec<u8>>, // ordered refs over survivors ∪ creates
}

impl RuleSpec {
    /// Content hash over the canonicalized spec (v2 `class_hash` generalized).
    pub fn class_hash(&self) -> u64 {
        use rustc_hash::FxHasher;
        use std::hash::{Hash, Hasher};
        let mut h = FxHasher::default();
        0x5e1f_a0d5u64.hash(&mut h);
        self.priority.hash(&mut h);
        self.n_vars.hash(&mut h);
        for vl in &self.var_labels {
            match vl {
                Some(l) => {
                    1u8.hash(&mut h);
                    l.hash(&mut h);
                }
                None => 0u8.hash(&mut h),
            }
        }
        let mut pe = self.pedges.clone();
        pe.sort();
        pe.len().hash(&mut h);
        for e in pe {
            e.hash(&mut h);
        }
        let mut gs: Vec<(u8, u32, u8, u64)> = self
            .guards
            .iter()
            .map(|g| {
                let mut vh = FxHasher::default();
                g.val.hash_into(&mut vh);
                (g.var, g.key, g.op, vh.finish())
            })
            .collect();
        gs.sort();
        gs.len().hash(&mut h);
        for g in gs {
            g.hash(&mut h);
        }
        let mut d = self.deletes.clone();
        d.sort();
        d.dedup();
        d.hash(&mut h);
        let mut rl = self.relabels.clone();
        rl.sort();
        rl.hash(&mut h);
        self.creates.hash(&mut h);
        let mut re = self.redges.clone();
        re.sort();
        re.len().hash(&mut h);
        for e in re {
            e.hash(&mut h);
        }
        h.finish()
    }
}
