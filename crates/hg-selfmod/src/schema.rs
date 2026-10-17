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

// ---------- rule-space node labels ----------
pub const RULE_ROOT: u32 = 1000;
pub const PNODE: u32 = 1003;
pub const GUARD: u32 = 1005;
pub const RELABEL: u32 = 1006;
pub const CREATE: u32 = 1007;
pub const REDGE: u32 = 1008;
pub const PEDGE: u32 = 1009;
pub const DELETE: u32 = 1010;
/// Var-distinctness pair (E1d): carries two var ids (`K_VAR_ID`, `K_VAR_ID2`),
/// normalized `a < b`. One node per unordered pair.
pub const DISTINCT: u32 = 1011;
/// Property effect (E2): target/source refs and op payload on `K_E*` keys.
/// Declaration order is significant (sequential semantics) and preserved.
pub const EFFECT: u32 = 1012;

// ---------- rule-space edge roles (labels >= 2000) ----------
pub const HAS_PNODE: u32 = 2002;
pub const HAS_GUARD: u32 = 2004;
pub const HAS_RELABEL: u32 = 2005;
pub const HAS_CREATE: u32 = 2006;
pub const HAS_REDGE: u32 = 2007;
pub const HAS_PEDGE: u32 = 2008;
pub const HAS_DELETE: u32 = 2009;
pub const HAS_DISTINCT: u32 = 2010;
pub const HAS_EFFECT: u32 = 2011;
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
/// Second var id of a DISTINCT pair node (first is `K_VAR_ID`).
pub const K_VAR_ID2: u32 = 13;
/// EFFECT node keys (E2): target ref, property key, op code, value payload
/// (literal for set/add/subtract, Float factor for scale_from, Float
/// magnitude for jitter), source ref, source key.
pub const K_ETARGET: u32 = 14;
pub const K_EKEY: u32 = 15;
/// Op codes: 0 set, 1 copy_from, 2 scale_from, 3 add, 4 subtract,
/// 5 add_from_property, 6 subtract_from_property, 7 jitter.
pub const K_EOP: u32 = 16;
pub const K_EVAL: u32 = 17;
pub const K_ESRC: u32 = 18;
pub const K_ESRCKEY: u32 = 19;
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

#[derive(Clone, Debug, PartialEq)]
pub struct GuardSpec {
    pub var: u8,
    pub key: u32,
    /// 0 Gt, 1 Lt, 2 Eq, 3 Lte, 4 Gte, 5 Ne — the full `PropertyOp` set
    /// (E1-G completion; the energy-guard idiom is `gte`).
    pub op: u8,
    pub val: PropertyValue,
}

impl GuardSpec {
    pub fn property_op(&self) -> Option<PropertyOp> {
        match self.op {
            0 => Some(PropertyOp::Gt),
            1 => Some(PropertyOp::Lt),
            2 => Some(PropertyOp::Eq),
            3 => Some(PropertyOp::Lte),
            4 => Some(PropertyOp::Gte),
            5 => Some(PropertyOp::Ne),
            _ => None,
        }
    }
}

/// Generalized rule spec — the meeting point of encode/decode and the
/// engine-native compile. LHS pattern edges are consumed on apply (engine
/// rewrite semantics); edge preservation means re-emitting on the RHS.
/// RHS edge refs: `0..n_vars` = LHS vars (must survive), `n_vars..` = creates.
#[derive(Clone, Debug, PartialEq)]
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
    /// Var-distinctness pairs (E1d), normalized `a < b`, sorted, deduped by
    /// the decoder. Compiled to one 2-var `VarsDistinct` guard per pair.
    /// No mutation operator touches these yet (frozen trait — see
    /// `docs/ere_encoding_schema.md`); fission copies them verbatim.
    pub distinct: Vec<(u8, u8)>,
    /// Property effects (E2) in declaration order — order is significant
    /// (each effect's reads see earlier effects' writes) and is preserved,
    /// never sorted, by encode/decode/RCF. Compiled to `PropertyEffectDecl`s
    /// (names resolved by `compile_rule`). Refs are var ids (`< n_vars`) or
    /// create refs (`>= n_vars`). No mutation operator touches these yet
    /// (frozen trait, same doctrine as `distinct`).
    pub effects: Vec<EffectSpec>,
}

/// One encoded property effect. Mirrors `hg_rules::EffectOpDecl` with numeric
/// refs (var id or create ref) instead of names.
#[derive(Clone, Debug, PartialEq)]
pub struct EffectSpec {
    pub target: u8,
    pub key: u32,
    pub op: EffectOp,
}

#[derive(Clone, Debug, PartialEq)]
pub enum EffectOp {
    Set {
        value: PropertyValue,
    },
    CopyFrom {
        source: u8,
        source_key: u32,
    },
    ScaleFrom {
        source: u8,
        source_key: u32,
        factor: f64,
    },
    Add {
        value: PropertyValue,
    },
    Subtract {
        value: PropertyValue,
    },
    AddFromProperty {
        source: u8,
        source_key: u32,
    },
    SubtractFromProperty {
        source: u8,
        source_key: u32,
    },
    Jitter {
        magnitude: f64,
    },
}

impl EffectSpec {
    /// Rename refs by `rn`. The closure MUST map create refs (`>= n_vars`)
    /// to themselves — only var ids are renamed (callers pass a
    /// create-preserving closure; see `serialize_renamed`, `permute_spec`).
    /// Declaration order is untouched (caller preserves the vec order).
    pub fn renamed(&self, rn: impl Fn(u8) -> u8) -> EffectSpec {
        let r = &rn;
        EffectSpec {
            target: r(self.target),
            key: self.key,
            op: match &self.op {
                EffectOp::Set { value } => EffectOp::Set { value: *value },
                EffectOp::CopyFrom { source, source_key } => EffectOp::CopyFrom {
                    source: r(*source),
                    source_key: *source_key,
                },
                EffectOp::ScaleFrom {
                    source,
                    source_key,
                    factor,
                } => EffectOp::ScaleFrom {
                    source: r(*source),
                    source_key: *source_key,
                    factor: *factor,
                },
                EffectOp::Add { value } => EffectOp::Add { value: *value },
                EffectOp::Subtract { value } => EffectOp::Subtract { value: *value },
                EffectOp::AddFromProperty { source, source_key } => EffectOp::AddFromProperty {
                    source: r(*source),
                    source_key: *source_key,
                },
                EffectOp::SubtractFromProperty { source, source_key } => {
                    EffectOp::SubtractFromProperty {
                        source: r(*source),
                        source_key: *source_key,
                    }
                }
                EffectOp::Jitter { magnitude } => EffectOp::Jitter {
                    magnitude: *magnitude,
                },
            },
        }
    }
}

/// RCF domain separator ("RCF1"). Distinct from the legacy `class_hash`
/// separator so the two hashes are never confused. See
/// `docs/ere_canonical_form.md`.
pub const RCF_DOMAIN: u64 = 0x5243_4631u64;

/// Brute-force ceiling: complete canonical labeling enumerates `n_vars!`
/// renamings. Above this, `rcf_bytes` uses the deterministic greedy fallback
/// (documented heuristic — SB-R3 re-applies there).
pub const RCF_BRUTE_FORCE_MAX_VARS: usize = 8;

fn next_permutation(p: &mut [u8]) -> bool {
    if p.len() < 2 {
        return false;
    }
    let mut i = p.len() - 1;
    while i > 0 && p[i - 1] >= p[i] {
        i -= 1;
    }
    if i == 0 {
        return false;
    }
    let mut j = p.len() - 1;
    while p[j] <= p[i - 1] {
        j -= 1;
    }
    p.swap(i - 1, j);
    p[i..].reverse();
    true
}

fn for_each_perm(n: usize, f: &mut impl FnMut(&[u8])) {
    if n == 0 {
        f(&[]);
        return;
    }
    let mut p: Vec<u8> = (0..n as u8).collect();
    loop {
        f(&p);
        if !next_permutation(&mut p) {
            break;
        }
    }
}
impl RuleSpec {
    /// Content hash over the canonicalized spec (v2 `class_hash` generalized).
    /// LEGACY: index-sensitive, NOT renaming-invariant. Compat only — never
    /// gate novelty on this; use `rcf_hash`.
    /// FROZEN (E1d): ignores `distinct`/`effects` (and any future metadata) so
    /// pre-extension artifacts keep byte-identical order. Two specs differing
    /// only in distinct pairs share a class_hash and fall back to `K_SEQ`
    /// order — deterministic per history, arbitrary across histories; the
    /// real instrument (`rcf_hash`) separates them. See
    /// `docs/ere_determinism.md` D1.
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

impl RuleSpec {
    /// Canonical byte string modulo variable renaming (see
    /// `docs/ere_canonical_form.md` §2.2). `order[new_id] = old_var`.
    /// Complete for `n_vars <= RCF_BRUTE_FORCE_MAX_VARS` (lexicographic
    /// minimum over all renamings); deterministic greedy fallback above.
    /// RNG-free. Inner edge order is preserved (slots are positional).
    pub fn rcf_bytes(&self) -> Vec<u8> {
        let n = self.n_vars as usize;
        if n <= RCF_BRUTE_FORCE_MAX_VARS {
            let mut best: Option<Vec<u8>> = None;
            for_each_perm(n, &mut |order| {
                let cand = self.serialize_renamed(order);
                if best.as_ref().map_or(true, |b| cand < *b) {
                    best = Some(cand);
                }
            });
            best.unwrap_or_default()
        } else {
            self.serialize_renamed(&greedy_order(self))
        }
    }

    /// RCF hash: `FxHasher` over `RCF_DOMAIN || rcf_bytes`. Same hasher
    /// family as `canonical_hash`; deterministic, no per-process seed.
    pub fn rcf_hash(&self) -> u64 {
        use rustc_hash::FxHasher;
        use std::hash::{Hash, Hasher};
        let mut h = FxHasher::default();
        RCF_DOMAIN.hash(&mut h);
        self.rcf_bytes().hash(&mut h);
        h.finish()
    }

    fn serialize_renamed(&self, order: &[u8]) -> Vec<u8> {
        let n = self.n_vars as usize;
        debug_assert_eq!(order.len(), n);
        // inv[old_var] = new_id
        let mut inv = vec![0u8; n];
        for (new, &old) in order.iter().enumerate() {
            inv[old as usize] = new as u8;
        }
        let mut out = Vec::with_capacity(64);
        out.extend_from_slice(b"RCF1");
        out.extend_from_slice(&self.priority.to_le_bytes());
        out.push(self.n_vars);
        // var labels in new-id order
        for new in 0..n {
            match self.var_labels.get(order[new] as usize).copied().flatten() {
                Some(l) => {
                    out.push(0x01);
                    out.extend_from_slice(&l.to_le_bytes());
                }
                None => out.push(0x00),
            }
        }
        // LHS edges: refs remapped, inner order kept, outer sorted
        let mut pe: Vec<Vec<u8>> = self
            .pedges
            .iter()
            .map(|e| e.iter().map(|&r| inv[r as usize]).collect())
            .collect();
        pe.sort();
        push_u64(&mut out, pe.len() as u64);
        for e in &pe {
            out.push(e.len() as u8);
            out.extend_from_slice(e);
        }
        // guards: var remapped, records sorted
        let mut gs: Vec<Vec<u8>> = self
            .guards
            .iter()
            .map(|g| {
                let mut b = Vec::with_capacity(16);
                b.push(inv[g.var as usize]);
                b.extend_from_slice(&g.key.to_le_bytes());
                b.push(g.op);
                push_propval(&mut b, &g.val);
                b
            })
            .collect();
        gs.sort();
        push_u64(&mut out, gs.len() as u64);
        for g in &gs {
            out.extend_from_slice(g);
        }
        // deletes: remapped, sorted
        let mut d: Vec<u8> = self.deletes.iter().map(|&v| inv[v as usize]).collect();
        d.sort_unstable();
        push_u64(&mut out, d.len() as u64);
        out.extend_from_slice(&d);
        // relabels: var remapped, sorted
        let mut rl: Vec<(u8, u32)> = self
            .relabels
            .iter()
            .map(|&(v, l)| (inv[v as usize], l))
            .collect();
        rl.sort();
        push_u64(&mut out, rl.len() as u64);
        for (v, l) in &rl {
            out.push(*v);
            out.extend_from_slice(&l.to_le_bytes());
        }
        // creates: index-ordered, untouched by renaming
        push_u64(&mut out, self.creates.len() as u64);
        for l in &self.creates {
            out.extend_from_slice(&l.to_le_bytes());
        }
        // RHS edges: refs < n_vars remapped, create refs untouched
        let mut re: Vec<Vec<u8>> = self
            .redges
            .iter()
            .map(|e| {
                e.iter()
                    .map(|&r| if (r as usize) < n { inv[r as usize] } else { r })
                    .collect()
            })
            .collect();
        re.sort();
        push_u64(&mut out, re.len() as u64);
        for e in &re {
            out.push(e.len() as u8);
            out.extend_from_slice(e);
        }
        // Distinct pairs (E1d): vars remapped, each pair normalized a < b
        // (renaming can flip the order), outer sorted.
        let mut di: Vec<(u8, u8)> = self
            .distinct
            .iter()
            .map(|&(a, b)| {
                let (x, y) = (inv[a as usize], inv[b as usize]);
                (x.min(y), x.max(y))
            })
            .collect();
        di.sort();
        di.dedup();
        push_u64(&mut out, di.len() as u64);
        for (a, b) in &di {
            out.push(*a);
            out.push(*b);
        }
        // Effects (E2): declaration order PRESERVED (sequential semantics —
        // reordering can change behavior, so it must change the hash).
        push_u64(&mut out, self.effects.len() as u64);
        for e in &self.effects {
            let e = e.renamed(|r| if (r as usize) < n { inv[r as usize] } else { r });
            out.push(e.target);
            out.extend_from_slice(&e.key.to_le_bytes());
            match &e.op {
                EffectOp::Set { value } => {
                    out.push(0);
                    push_propval(&mut out, value);
                }
                EffectOp::CopyFrom { source, source_key } => {
                    out.push(1);
                    out.push(*source);
                    out.extend_from_slice(&source_key.to_le_bytes());
                }
                EffectOp::ScaleFrom {
                    source,
                    source_key,
                    factor,
                } => {
                    out.push(2);
                    out.push(*source);
                    out.extend_from_slice(&source_key.to_le_bytes());
                    out.extend_from_slice(&factor.to_bits().to_le_bytes());
                }
                EffectOp::Add { value } => {
                    out.push(3);
                    push_propval(&mut out, value);
                }
                EffectOp::Subtract { value } => {
                    out.push(4);
                    push_propval(&mut out, value);
                }
                EffectOp::AddFromProperty { source, source_key } => {
                    out.push(5);
                    out.push(*source);
                    out.extend_from_slice(&source_key.to_le_bytes());
                }
                EffectOp::SubtractFromProperty { source, source_key } => {
                    out.push(6);
                    out.push(*source);
                    out.extend_from_slice(&source_key.to_le_bytes());
                }
                EffectOp::Jitter { magnitude } => {
                    out.push(7);
                    out.extend_from_slice(&magnitude.to_bits().to_le_bytes());
                }
            }
        }
        out
    }
}

fn push_u64(out: &mut Vec<u8>, v: u64) {
    out.extend_from_slice(&v.to_le_bytes());
}

fn push_propval(out: &mut Vec<u8>, v: &PropertyValue) {
    match v {
        PropertyValue::Int(i) => {
            out.push(0x01);
            out.extend_from_slice(&i.to_le_bytes());
        }
        PropertyValue::Float(f) => {
            out.push(0x02);
            out.extend_from_slice(&f.to_bits().to_le_bytes());
        }
        PropertyValue::Bool(b) => {
            out.push(0x03);
            out.push(u8::from(*b));
        }
    }
}

/// Deterministic greedy var order for `n_vars > RCF_BRUTE_FORCE_MAX_VARS`
/// (heuristic — SB-R3 re-applies: may collide non-isomorphic specs).
/// Key per var: (label rank, lhs degree, rhs degree, guard count,
/// deleted flag, relabeled flag), ties by old id. Returns `order[new]=old`.
fn greedy_order(spec: &RuleSpec) -> Vec<u8> {
    let n = spec.n_vars as usize;
    let mut lhs_deg = vec![0usize; n];
    for e in &spec.pedges {
        for &r in e {
            if (r as usize) < n {
                lhs_deg[r as usize] += 1;
            }
        }
    }
    let mut rhs_deg = vec![0usize; n];
    for e in &spec.redges {
        for &r in e {
            if (r as usize) < n {
                rhs_deg[r as usize] += 1;
            }
        }
    }
    let mut guards = vec![0usize; n];
    for g in &spec.guards {
        if (g.var as usize) < n {
            guards[g.var as usize] += 1;
        }
    }
    let mut idx: Vec<u8> = (0..n as u8).collect();
    idx.sort_by_key(|&v| {
        let i = v as usize;
        (
            spec.var_labels.get(i).copied().flatten().is_some() as u8,
            spec.var_labels
                .get(i)
                .copied()
                .flatten()
                .unwrap_or(u32::MAX),
            lhs_deg[i],
            rhs_deg[i],
            guards[i],
            spec.deletes.contains(&v) as u8,
            spec.relabels.iter().any(|(w, _)| *w == v) as u8,
            v,
        )
    });
    idx
}
