//! Regime classification — 11 variants (v4 §7).

use serde::{Deserialize, Serialize};

/// The regime a run window is classified into. Variant order is the
/// classifier's priority order for the two novelty-distinguished regimes:
/// `ActivePersistentStructure` sits immediately before `GenerativeCandidate`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Regime {
    Frozen,
    StaticOrdered,
    GrowthTreadmill,
    WhiteNoiseSoup,
    SterileChaos,
    Exploding,
    FragmentedIsolation,
    Monoculture,
    /// It moves (a persistent structure whose constituent nodes change),
    /// but does not necessarily invent new structure types — a strictly
    /// weaker claim than `GenerativeCandidate`.
    ActivePersistentStructure,
    GenerativeCandidate,
    #[default]
    Unknown,
}

impl Regime {
    pub fn as_str(&self) -> &'static str {
        match self {
            Regime::Frozen => "Frozen",
            Regime::StaticOrdered => "StaticOrdered",
            Regime::GrowthTreadmill => "GrowthTreadmill",
            Regime::WhiteNoiseSoup => "WhiteNoiseSoup",
            Regime::SterileChaos => "SterileChaos",
            Regime::Exploding => "Exploding",
            Regime::FragmentedIsolation => "FragmentedIsolation",
            Regime::Monoculture => "Monoculture",
            Regime::ActivePersistentStructure => "ActivePersistentStructure",
            Regime::GenerativeCandidate => "GenerativeCandidate",
            Regime::Unknown => "Unknown",
        }
    }
}
