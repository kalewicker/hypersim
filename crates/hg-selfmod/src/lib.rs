//! hg-selfmod: the v3 self-modification experiment — rules reified as
//! first-class subgraphs in a SEPARATE RuleStore instance, decoded each step
//! into engine-native `CompiledRule`s and applied to the world store by
//! hg-engine's own matcher/conflict/apply path. Conserved-token economy
//! (Phase A regulation, Phase B metabolic selection) over the rule
//! population. See docs/selfmod_v3_recon.md and hg_selfmod_v3_plan.md.
//!
//! Rule-space location: a separate `HypergraphStore` (not the world store) —
//! nothing matches against rule-subgraphs in v3, the world stays pristine
//! for hg-observe, and hg-engine stays untouched. Same-store co-residence is
//! the (out-of-scope) full-reflection milestone's planned break.

pub mod decode;
pub mod economy;
pub mod encode;
pub mod gates;
pub mod gr1;
pub mod gr2;
pub mod meta;
pub mod runner;
pub mod schema;
pub mod store_ext;
