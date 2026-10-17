//! `hg-meta` — ERE track (Endogenous Rule Encoding), phases R0–R2.
//!
//! Rules as first-class subgraph objects of the hypergraph they act on:
//! the E1 encoding schema (§3 of the plan), the rule canonical form (§4),
//! the validator battery (§5.4 as instantiated in docs/ere_validators.md),
//! the candidate compiler (G-R2 round-trip), and the offline
//! characterization harnesses (G-R1 mutational neutrality).
//!
//! Crate discipline: depends on `hg-core` + `hg-rules` ONLY. No engine, no
//! observe. Nothing here draws RNG; every ordering is content-based.

pub mod compile;
pub mod economy;
pub mod epoch;
#[cfg(test)]
mod expr_r07;
#[cfg(test)]
mod expr_r08;
#[cfg(test)]
mod expr_r09;
#[cfg(test)]
mod expr_r09_mech;
#[cfg(test)]
mod expr_r10;
#[cfg(test)]
mod expr_tests;
#[cfg(test)]
mod fee_integration_tests;
pub mod fee_policy;
pub mod gge1;
#[cfg(test)]
mod gge1_tests;
pub mod gge2;
pub mod gr1;
#[cfg(test)]
mod gr3;
#[cfg(test)]
mod gr4_tests;
pub mod graph_source;
pub mod install;
pub mod mutate;
#[cfg(test)]
mod outer;
pub mod protect;
#[cfg(test)]
mod r4_tests;
pub mod rcf;
pub mod schema;
pub mod sweep;
#[cfg(test)]
mod sweep_tests;
pub mod validate;

#[cfg(test)]
mod compile_tests;
#[cfg(test)]
mod gr1_gate;
#[cfg(test)]
mod gr1_tests;
#[cfg(test)]
mod rcf_tests;
#[cfg(test)]
mod schema_tests;
#[cfg(test)]
pub(crate) mod testutil;
#[cfg(test)]
mod validate_tests;
