//! `PropertyValue` — typed node properties (v4 §3).
//!
//! Three variants; a single signed-integer type covers every realistic
//! count or flag. TOML's own lexer distinguishes int/float/bool, so these
//! deserialize via `#[serde(untagged)]` with no explicit type tag.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PropertyValue {
    Int(i64),
    Float(f64),
    Bool(bool),
}

impl PartialEq for PropertyValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (PropertyValue::Int(a), PropertyValue::Int(b)) => a == b,
            (PropertyValue::Float(a), PropertyValue::Float(b)) => a.to_bits() == b.to_bits(),
            (PropertyValue::Bool(a), PropertyValue::Bool(b)) => a == b,
            _ => false,
        }
    }
}
impl Eq for PropertyValue {}

/// Comparison operators for property guards.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PropertyOp {
    Eq,
    Ne,
    Lt,
    Gt,
    Lte,
    Gte,
}

impl PropertyValue {
    /// Fold into a deterministic byte tag + payload for hashing.
    pub fn hash_into<H: std::hash::Hasher>(&self, h: &mut H) {
        use std::hash::Hash;
        match self {
            PropertyValue::Int(v) => {
                0u8.hash(h);
                v.hash(h);
            }
            PropertyValue::Float(v) => {
                1u8.hash(h);
                v.to_bits().hash(h);
            }
            PropertyValue::Bool(v) => {
                2u8.hash(h);
                v.hash(h);
            }
        }
    }

    /// Compare a stored value against a threshold under `op`. Variant
    /// mismatch (v4 §3) returns `false` — no coercion, no panic.
    pub fn compare(&self, op: PropertyOp, threshold: &PropertyValue) -> bool {
        use std::cmp::Ordering;
        let ord: Option<Ordering> = match (self, threshold) {
            (PropertyValue::Int(a), PropertyValue::Int(b)) => Some(a.cmp(b)),
            (PropertyValue::Float(a), PropertyValue::Float(b)) => a.partial_cmp(b),
            (PropertyValue::Bool(a), PropertyValue::Bool(b)) => Some(a.cmp(b)),
            _ => return false, // variant mismatch
        };
        match ord {
            None => false, // e.g. NaN
            Some(o) => match op {
                PropertyOp::Eq => o == Ordering::Equal,
                PropertyOp::Ne => o != Ordering::Equal,
                PropertyOp::Lt => o == Ordering::Less,
                PropertyOp::Gt => o == Ordering::Greater,
                PropertyOp::Lte => o != Ordering::Greater,
                PropertyOp::Gte => o != Ordering::Less,
            },
        }
    }
}
