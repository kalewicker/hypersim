//! Persistence-only wire encoding for node properties.
//!
//! The domain [`PropertyValue`](crate::property::PropertyValue) is
//! `#[serde(untagged)]` — load-bearing, so TOML rule files can write bare
//! `100` / `0.8` / `true` literals. Untagged is fatal for bincode
//! (non-self-describing) in BOTH directions:
//!
//! * serialize succeeds but is **lossy** — the variant's raw content is
//!   written with no discriminant, so `Int(5)` and `Float(5.0)` are
//!   indistinguishable bytes on disk (the type is not persisted at all);
//! * deserialize **fails** (`DeserializeAnyNotSupported`) — untagged needs
//!   `deserialize_any`, which bincode cannot provide.
//!
//! This externally-tagged mirror is used ONLY where properties cross the
//! persistence boundary — the `#[serde(with = "crate::wire::properties")]`
//! on `NodeData.properties`. The domain type keeps `untagged` for TOML;
//! nothing here changes rule-file parsing.

use crate::graph::Properties;
use crate::property::PropertyValue;
use serde::{Deserialize, Serialize};

/// Externally-tagged mirror of `PropertyValue` — a discriminant is written
/// per value, so bincode round-trips `Int`/`Float`/`Bool` unambiguously.
#[derive(Serialize, Deserialize)]
enum PropertyValueWire {
    Int(i64),
    Float(f64),
    Bool(bool),
}

impl From<PropertyValue> for PropertyValueWire {
    fn from(v: PropertyValue) -> Self {
        match v {
            PropertyValue::Int(i) => PropertyValueWire::Int(i),
            PropertyValue::Float(f) => PropertyValueWire::Float(f),
            PropertyValue::Bool(b) => PropertyValueWire::Bool(b),
        }
    }
}

impl From<PropertyValueWire> for PropertyValue {
    fn from(v: PropertyValueWire) -> Self {
        match v {
            PropertyValueWire::Int(i) => PropertyValue::Int(i),
            PropertyValueWire::Float(f) => PropertyValue::Float(f),
            PropertyValueWire::Bool(b) => PropertyValue::Bool(b),
        }
    }
}

/// `#[serde(with = "…")]` module for `NodeData.properties`: serializes the
/// `(key, value)` pairs with tagged values so the on-disk bytes preserve
/// each property's variant.
pub mod properties {
    use super::{Properties, PropertyValue, PropertyValueWire};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(props: &Properties, s: S) -> Result<S::Ok, S::Error> {
        let wire: Vec<(u32, PropertyValueWire)> =
            props.iter().map(|(k, v)| (*k, (*v).into())).collect();
        wire.serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Properties, D::Error> {
        let wire = Vec::<(u32, PropertyValueWire)>::deserialize(d)?;
        Ok(wire
            .into_iter()
            .map(|(k, v)| (k, PropertyValue::from(v)))
            .collect())
    }
}

/// Tagged single values at compiled-rule persistence boundaries. Raw rule
/// declarations retain their existing untagged TOML representation.
pub mod value {
    use super::{PropertyValue, PropertyValueWire};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    pub fn serialize<S: Serializer>(v: &PropertyValue, s: S) -> Result<S::Ok, S::Error> {
        PropertyValueWire::from(*v).serialize(s)
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<PropertyValue, D::Error> {
        Ok(PropertyValueWire::deserialize(d)?.into())
    }
}
pub mod optional_value {
    use super::{PropertyValue, PropertyValueWire};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    pub fn serialize<S: Serializer>(v: &Option<PropertyValue>, s: S) -> Result<S::Ok, S::Error> {
        v.map(PropertyValueWire::from).serialize(s)
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Option<PropertyValue>, D::Error> {
        Ok(Option::<PropertyValueWire>::deserialize(d)?.map(Into::into))
    }
}
