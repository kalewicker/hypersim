//! Constant energy influx (plan v10 §3.1) — the removal of the
//! **seeded-budget** scaffold.
//!
//! DESIGN ORDER, RECORDED (v10 §1.1, applying v8 §1.1 prospectively for
//! the first time). The design order for any new capability is (a) derive
//! it from state, (b) compute it engine-side, (c) express it as a rule
//! with characterized back-action. Influx takes **(b)**, deliberately and
//! before the fact rather than after: a per-node energy injection written
//! as a single-node-LHS rule is `lk_tick` again — a rule that binds
//! everywhere, costs nothing, and therefore wins scheduler conflicts
//! against the physics it is supposed to observe. v8 spent a whole round
//! removing exactly that instrument. So influx is not a rule, claims no
//! bindings, draws no RNG, and is applied by the runner before matching.
//!
//! THE ACCUMULATOR (v10 §3.2, hazard 1). The extended audit invariant is
//!
//! ```text
//! E_total(t) + c·N_sub(t) + Σ E_fused(t) = E₀ + E_injected(t)
//! ```
//!
//! and `E_injected` **must be an accumulator, never a formula**. Writing
//! it as Φ·T assumes the per-step injection actually equalled Φ, which is
//! precisely what the audit exists to check — a formula-based term makes
//! the audit self-confirming, which is worse than having no audit. So
//! [`inject`] returns the sum of the deltas it actually wrote, the caller
//! adds that to a running total, and the invariant closes against the
//! total.
//!
//! This is not a hypothetical distinction. Under `constant_total` each of
//! the N targets receives Φ/N, and Σ(Φ/N) over N targets is **not** Φ in
//! binary floating point for most (Φ, N) — with N changing every step as
//! the graph grows, the discrepancy accumulates without bound. A formula
//! term would drift; the accumulator closes to exactly 0.0.
//! `influx_accumulator_beats_the_formula` pins the gap.
//!
//! DETERMINISM (IG4). No draws: targets come from `SlotMap` iteration
//! order, which is slot order and reproducible across processes given the
//! same insertion/removal sequence — the same property `ids` already
//! relies on elsewhere in the runner. Deltas within a mode are equal, so
//! the accumulated sum does not depend on visit order either.

use hg_core::{HypergraphStore, NodeId, PropertyValue};
use serde::{Deserialize, Serialize};

/// Energy is property key 1 throughout the LK family.
const ENERGY: u32 = 1;

fn default_mode() -> String {
    "constant_total".to_string()
}

/// `[influx]` (v10 Appendix A). Disabled by default, and skipped entirely
/// from serialization when disabled, so `config_hash` and every archived
/// byte-identity regression are unchanged by this feature's existence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InfluxConfig {
    #[serde(default)]
    pub enabled: bool,
    /// `constant_total` | `fixed_sites` | `per_node`.
    #[serde(default = "default_mode")]
    pub mode: String,
    /// Energy per step. Under `constant_total` and `fixed_sites` this is
    /// the TOTAL entering the graph each step; under `per_node` it is the
    /// amount each node receives, so the total is Φ·N and grows with the
    /// graph (v10 §3.1 — the positive-feedback variant, run last).
    #[serde(default)]
    pub phi: f64,
    /// `fixed_sites` only: indices into the live-node list. Empty means
    /// "the run's LK seed site(s)", i.e. pump continuity with the quench
    /// removed.
    #[serde(default)]
    pub sites: Vec<usize>,
}

impl Default for InfluxConfig {
    fn default() -> Self {
        Self { enabled: false, mode: default_mode(), phi: 0.0, sites: Vec::new() }
    }
}

impl InfluxConfig {
    /// Serialization gate: a disabled influx block must not appear in the
    /// serialized config at all, so adding this feature does not perturb
    /// the config hash of any pre-v10 run.
    pub fn is_disabled(&self) -> bool {
        !self.enabled
    }

    pub fn validate(&self) -> Result<(), String> {
        if !self.enabled {
            return Ok(());
        }
        if !matches!(self.mode.as_str(), "constant_total" | "fixed_sites" | "per_node") {
            return Err(format!(
                "influx.mode '{}' unknown (constant_total | fixed_sites | per_node)",
                self.mode
            ));
        }
        if !self.phi.is_finite() || self.phi < 0.0 {
            return Err(format!("influx.phi must be finite and >= 0 (got {})", self.phi));
        }
        Ok(())
    }
}

/// What one step's injection actually did. `injected` is the quantity the
/// audit accumulates; the rest exists so an injection that silently
/// reached nothing cannot masquerade as a working one.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct InfluxOutcome {
    /// Sum of the deltas actually written this step. THE accumulator input.
    pub injected: f64,
    /// Nodes that received energy.
    pub targets: usize,
    /// Live nodes skipped because they carry no energy property. Under the
    /// LK ICs every node is zeroed into existence with one, so this is
    /// expected to be 0; it is counted rather than assumed.
    pub skipped: usize,
}

/// Inject one step's energy, before matching. Returns the amount actually
/// written — never the amount requested.
///
/// `seed_sites` is used only by `fixed_sites` with an empty `sites` list.
pub fn inject(
    store: &mut HypergraphStore,
    cfg: &InfluxConfig,
    seed_sites: &[NodeId],
) -> InfluxOutcome {
    let mut out = InfluxOutcome::default();
    if !cfg.enabled || cfg.phi == 0.0 {
        return out;
    }

    // Live, energy-bearing nodes in SlotMap order (deterministic).
    let mut live: Vec<NodeId> = Vec::new();
    for (id, d) in store.nodes() {
        if matches!(d.get_property(ENERGY), Some(PropertyValue::Float(_))) {
            live.push(id);
        } else {
            out.skipped += 1;
        }
    }
    if live.is_empty() {
        return out;
    }

    let targets: Vec<NodeId> = match cfg.mode.as_str() {
        "constant_total" | "per_node" => live,
        "fixed_sites" => {
            if cfg.sites.is_empty() {
                // Pump continuity: the run's own seed placement.
                seed_sites.iter().copied().filter(|id| store.get_node(*id).is_some()).collect()
            } else {
                cfg.sites.iter().filter_map(|&i| live.get(i).copied()).collect()
            }
        }
        // validate() rejects anything else before the run starts.
        _ => return out,
    };
    if targets.is_empty() {
        return out;
    }

    // `constant_total` and `fixed_sites` divide Φ across their targets, so
    // Φ is the total entering the graph and is independent of graph size —
    // which is what makes Φ a clean control variable (v10 §3.1) and what
    // makes the p = Φ/c prediction placement-independent (§3.3).
    // `per_node` gives Φ to each, so the total is Φ·N.
    let delta = match cfg.mode.as_str() {
        "per_node" => cfg.phi,
        _ => cfg.phi / targets.len() as f64,
    };

    for id in &targets {
        if let Some(d) = store.get_node_mut(*id) {
            let existing = match d.get_property(ENERGY) {
                Some(PropertyValue::Float(v)) => v,
                _ => 0.0,
            };
            d.set_property(ENERGY, PropertyValue::Float(existing + delta));
            // Accumulate what was WRITTEN, one target at a time — never
            // `phi`, never `delta * targets.len()`.
            out.injected += delta;
            out.targets += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use hg_core::{Label, NodeData};

    fn store_with(n: usize) -> HypergraphStore {
        let mut s = HypergraphStore::new();
        for _ in 0..n {
            let id = s.add_node(NodeData::new(Label(0), 0));
            s.get_node_mut(id).unwrap().set_property(ENERGY, PropertyValue::Float(0.0));
        }
        s
    }

    fn total_energy(s: &HypergraphStore) -> f64 {
        s.nodes()
            .filter_map(|(_, d)| match d.get_property(ENERGY) {
                Some(PropertyValue::Float(v)) => Some(v),
                _ => None,
            })
            .sum()
    }

    /// The accumulator is load-bearing, demonstrated rather than asserted:
    /// over a run whose node count changes, Σ(actual deltas) and the
    /// formula Φ·T disagree, and it is the accumulator that tracks the
    /// graph's real energy. This is hazard §9.1 made into a number.
    #[test]
    fn influx_accumulator_beats_the_formula() {
        let cfg = InfluxConfig {
            enabled: true,
            mode: "constant_total".into(),
            phi: 1000.0,
            sites: vec![],
        };
        let mut s = store_with(3);
        let mut accumulated = 0.0f64;
        let steps = 400;
        for t in 0..steps {
            // Grow the graph, exactly the condition under which Φ·T fails.
            if t % 7 == 0 {
                let id = s.add_node(NodeData::new(Label(0), t));
                s.get_node_mut(id).unwrap().set_property(ENERGY, PropertyValue::Float(0.0));
            }
            accumulated += inject(&mut s, &cfg, &[]).injected;
        }
        let formula = cfg.phi * steps as f64;
        let actual = total_energy(&s);

        // The accumulator matches the graph EXACTLY; the formula does not.
        assert_eq!(
            accumulated, actual,
            "accumulator {accumulated} != graph energy {actual} — the audit's \
             injected term does not track what entered the graph"
        );
        assert_ne!(
            formula, actual,
            "Φ·T happened to equal the graph energy here, so this test is not \
             exercising the drift it exists to demonstrate — pick a Φ/N that \
             does not divide exactly"
        );
        println!(
            "accumulator={accumulated:.6} graph={actual:.6} formula={formula:.6} \
             drift={:.3e}",
            formula - actual
        );
    }

    #[test]
    fn constant_total_is_independent_of_graph_size() {
        let cfg =
            InfluxConfig { enabled: true, mode: "constant_total".into(), phi: 64.0, sites: vec![] };
        for n in [1usize, 2, 4, 8, 1024] {
            let mut s = store_with(n);
            let o = inject(&mut s, &cfg, &[]);
            assert_eq!(o.targets, n);
            assert_eq!(o.injected, 64.0, "constant_total injected {} at n={n}", o.injected);
        }
    }

    #[test]
    fn per_node_total_scales_with_graph_size() {
        let cfg = InfluxConfig { enabled: true, mode: "per_node".into(), phi: 2.0, sites: vec![] };
        for n in [1usize, 4, 256] {
            let mut s = store_with(n);
            let o = inject(&mut s, &cfg, &[]);
            assert_eq!(o.injected, 2.0 * n as f64, "per_node should be Φ·N at n={n}");
        }
    }

    #[test]
    fn fixed_sites_defaults_to_the_seed_and_keeps_phi_as_the_total() {
        let cfg =
            InfluxConfig { enabled: true, mode: "fixed_sites".into(), phi: 500.0, sites: vec![] };
        let mut s = store_with(16);
        let seed = s.nodes().map(|(id, _)| id).nth(7).unwrap();
        let o = inject(&mut s, &cfg, &[seed]);
        assert_eq!(o.targets, 1);
        assert_eq!(o.injected, 500.0);
        assert_eq!(total_energy(&s), 500.0);
        let got = match s.get_node(seed).unwrap().get_property(ENERGY) {
            Some(PropertyValue::Float(v)) => v,
            _ => panic!("seed lost its energy property"),
        };
        assert_eq!(got, 500.0, "all of Φ should land on the seed site");
    }

    #[test]
    fn disabled_influx_writes_nothing() {
        let cfg = InfluxConfig::default();
        let mut s = store_with(8);
        let o = inject(&mut s, &cfg, &[]);
        assert_eq!(o, InfluxOutcome::default());
        assert_eq!(total_energy(&s), 0.0);
    }

    #[test]
    fn unknown_mode_is_rejected_before_the_run() {
        let cfg = InfluxConfig {
            enabled: true,
            mode: "sprinkle".into(),
            phi: 1.0,
            sites: vec![],
        };
        assert!(cfg.validate().is_err());
        assert!(InfluxConfig::default().validate().is_ok());
    }
}
