//! Token economy: selectable **stock** (v2/v3 conserved pool) or **flux**
//! (driven, dissipative flow). Integer arithmetic throughout.
//!
//! # Flow audit (asserted every step, both modes)
//!
//! ```text
//! cumulative_influx == bound_in_rules + free + cumulative_dissipated
//! ```
//!
//! `cumulative_influx` is seeded with the initial endowment (starting `free`
//! plus every seeded rule's `e0`), so the identity holds from step 0 with no
//! special case. In **stock** mode influx-per-step and dissipation are both
//! zero, so the audit degenerates to v3's `free + Σ tokens == total` — one
//! assert covers both arms, and the stock arm stays bit-comparable to v3.
//!
//! # Where tokens go
//!
//! | event | stock | flux |
//! |---|---|---|
//! | influx | — | `+I` to `free` each step, overflow above `free_cap` dissipated |
//! | rent | recycled to `free` | **dissipated** (this is what makes it a flow) |
//! | starvation | tokens → `free` | tokens → `free` (death recycles capital in both) |
//! | reward | `free` → rule, capped by `free` | same; source is replenished |
//! | fission | parent → child (internal transfer) | same |
//!
//! Rules can never mint tokens in either mode: they compete for an
//! externally-supplied stream. The anti-Goodhart property that motivated
//! conservation survives the switch to flux.

use crate::meta::mutate_at_birth;
use crate::schema::*;
use crate::store_ext::*;
use hg_core::{HypergraphStore, NodeId};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EconomyMode {
    Stock,
    Flux,
}

impl std::str::FromStr for EconomyMode {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, ()> {
        match s {
            "stock" => Ok(EconomyMode::Stock),
            "flux" => Ok(EconomyMode::Flux),
            _ => Err(()),
        }
    }
}

pub fn rule_tokens(s: &HypergraphStore, root: NodeId) -> i64 {
    get_int(s, root, K_RESOURCE).unwrap_or(0)
}

pub fn set_tokens(s: &mut HypergraphStore, root: NodeId, t: i64) {
    set_int(s, root, K_RESOURCE, t);
}

pub fn total_bound(s: &HypergraphStore) -> i64 {
    rule_roots(s).iter().map(|&r| rule_tokens(s, r)).sum()
}

#[derive(Clone, Copy, Debug)]
pub struct EconomyParams {
    pub mode: EconomyMode,
    /// Stock mode: the conserved total. Flux mode: unused except as the
    /// initial endowment source.
    pub total: i64,
    pub e0: i64,
    pub birth_cost: i64,
    pub rent: i64,
    pub fire_reward: i64,
    pub repro_threshold: i64,
    /// Flux influx per step.
    pub influx: i64,
    /// Flux free-pool cap; overflow dissipated. Without this, a frozen world
    /// accumulates unbounded fuel and would resurrect any population,
    /// destroying the freeze signal.
    pub free_cap: i64,
    pub p_dup: f64,
    pub p_del: f64,
    pub abio: f64,
    pub rule_cap: usize,
    /// Variation rate: probability an offspring is mutated rather than a
    /// faithful clone.
    pub mu: f64,
    pub creation_refill: i64,
    pub creation_budget_max: i64,
}

/// The token ledger. All movement goes through here so the audit cannot be
/// bypassed by a caller that forgets to record a flow.
#[derive(Clone, Copy, Debug)]
pub struct Ledger {
    pub free: i64,
    pub cumulative_influx: i64,
    pub cumulative_dissipated: i64,
}

impl Ledger {
    /// Seed the ledger with the initial endowment: starting free pool plus
    /// whatever the seeded rules already hold.
    pub fn new(initial_free: i64, initial_bound: i64) -> Self {
        Ledger {
            free: initial_free,
            cumulative_influx: initial_free + initial_bound,
            cumulative_dissipated: 0,
        }
    }

    /// External influx (flux mode only). Overflow above `free_cap` is
    /// counted as both influx and dissipation, so the audit holds exactly.
    pub fn influx(&mut self, pr: &EconomyParams) {
        if pr.mode != EconomyMode::Flux || pr.influx <= 0 {
            return;
        }
        self.cumulative_influx += pr.influx;
        self.free += pr.influx;
        if self.free > pr.free_cap {
            let overflow = self.free - pr.free_cap;
            self.free = pr.free_cap;
            self.cumulative_dissipated += overflow;
        }
    }

    /// Rent payment: dissipated under flux, recycled under stock.
    fn collect_rent(&mut self, amount: i64, pr: &EconomyParams) {
        match pr.mode {
            EconomyMode::Stock => self.free += amount,
            EconomyMode::Flux => self.cumulative_dissipated += amount,
        }
    }

    /// Death recycles capital in both modes.
    fn recycle(&mut self, amount: i64) {
        self.free += amount;
    }

    pub fn audit_residual(&self, bound: i64) -> i64 {
        self.cumulative_influx - (bound + self.free + self.cumulative_dissipated)
    }
}

// ---------- Phase A (stock only): fitness-blind regulation ----------

fn f01(rng: &mut ChaCha8Rng) -> f64 {
    use rand::Rng;
    rng.gen::<f64>()
}

fn below(rng: &mut ChaCha8Rng, n: u64) -> u64 {
    use rand::Rng;
    if n == 0 {
        0
    } else {
        rng.gen::<u64>() % n
    }
}

/// Phase A meta step: stochastic, fitness-blind birth/death gated by
/// resource, plus the genesis term. Stock semantics (v3 unchanged).
pub fn meta_step_a(
    s: &mut HypergraphStore,
    rng: &mut ChaCha8Rng,
    led: &mut Ledger,
    pr: &EconomyParams,
    caps: &Caps,
    seq: &mut i64,
) -> (u32, u32) {
    use crate::encode::install_rule;
    use crate::meta::mutate_once;

    let mut births = 0u32;
    let mut deaths = 0u32;

    if pr.abio > 0.0 && led.free >= pr.birth_cost {
        let roll = f01(rng);
        if roll < pr.abio * (led.free as f64 / pr.total as f64) {
            let v_lab = below(rng, caps.n_labels as u64) as u32;
            let r_lab = below(rng, caps.n_labels as u64) as u32;
            let spec = RuleSpec {
                priority: 0,
                n_vars: 1,
                var_labels: vec![Some(v_lab)],
                pedges: vec![],
                guards: vec![],
                deletes: vec![],
                relabels: vec![(0, r_lab)],
                creates: vec![],
                redges: vec![],

                distinct: vec![],

                effects: vec![],
            };
            let root = install_rule(s, &spec, seq);
            set_tokens(s, root, pr.birth_cost);
            set_int(s, root, K_CREATION_BUDGET, pr.creation_budget_max);
            set_int(s, root, K_DEPTH, 0);
            led.free -= pr.birth_cost;
            births += 1;
        }
    }

    let roots = rule_roots(s);
    if roots.is_empty() {
        return (births, deaths);
    }
    let root = roots[below(rng, roots.len() as u64) as usize];
    let roll = f01(rng);
    if roll < pr.p_dup {
        if rule_roots(s).len() < pr.rule_cap && led.free >= pr.birth_cost {
            let child = duplicate_rule(s, root);
            set_int(s, child, K_SEQ, *seq);
            *seq += 1;
            set_tokens(s, child, pr.birth_cost); // explicit — never inherit
            led.free -= pr.birth_cost;
            births += 1;
        }
    } else if roll < pr.p_dup + pr.p_del {
        led.recycle(rule_tokens(s, root));
        delete_rule(s, root);
        deaths += 1;
    } else {
        mutate_once(s, root, rng, caps);
    }
    (births, deaths)
}

// ---------- Phase B: metabolism ----------

/// Rent + starvation. Runs BEFORE the object phase (v3 §7 step order).
pub fn rent_and_starve(s: &mut HypergraphStore, led: &mut Ledger, pr: &EconomyParams) -> u32 {
    let mut deaths = 0u32;
    for root in rule_roots(s) {
        let t = rule_tokens(s, root);
        if t < pr.rent {
            led.recycle(t);
            delete_rule(s, root);
            deaths += 1;
        } else {
            set_tokens(s, root, t - pr.rent);
            led.collect_rent(pr.rent, pr);
        }
    }
    deaths
}

/// Flat `fire_reward` per PRODUCTIVE rule per step (not per match — rewards
/// relevance, resists match-volume farming; effect-gating closes the weed-rule
/// channel). Capped by `free`, so reward stays resource-limited and
/// competitive. Deterministic: roots processed in sorted order.
pub fn reward_productive(
    s: &mut HypergraphStore,
    productive: &HashSet<NodeId>,
    led: &mut Ledger,
    pr: &EconomyParams,
) {
    let mut roots: Vec<NodeId> = productive.iter().copied().collect();
    roots.sort_by_key(|&r| key_u64(r));
    for root in roots {
        if s.get_node(root).is_none() {
            continue; // starved earlier this step
        }
        let pay = pr.fire_reward.min(led.free);
        if pay > 0 {
            let t = rule_tokens(s, root);
            set_tokens(s, root, t + pay);
            led.free -= pay;
        }
    }
}

/// Per-lineage creation budgets refill a little each step, capped.
pub fn refill_creation_budgets(s: &mut HypergraphStore, pr: &EconomyParams) {
    if pr.creation_refill <= 0 {
        return;
    }
    for root in rule_roots(s) {
        let b = get_int(s, root, K_CREATION_BUDGET).unwrap_or(pr.creation_budget_max);
        let nb = (b + pr.creation_refill).min(pr.creation_budget_max);
        set_int(s, root, K_CREATION_BUDGET, nb);
    }
}

pub struct FissionOutcome {
    pub clonal: u32,
    pub mutant: u32,
    /// Mutant-coin births whose operator turned out inapplicable (an
    /// effectively-clonal child). The gap between `mutant` and
    /// `mutant - noop` is the effective variation rate.
    pub noop: u32,
}

/// Funded fission: a rule at `repro_threshold` pays `birth_cost` from ITS OWN
/// tokens to the child (an internal transfer — no ledger movement).
/// Mutate-at-birth is now probabilistic (v4 §4): with probability μ exactly
/// one structural operator is applied, else the child is a faithful clone.
pub fn funded_fission(
    s: &mut HypergraphStore,
    rng: &mut ChaCha8Rng,
    pr: &EconomyParams,
    caps: &Caps,
    seq: &mut i64,
) -> FissionOutcome {
    let mut out = FissionOutcome {
        clonal: 0,
        mutant: 0,
        noop: 0,
    };
    for root in rule_roots(s) {
        if rule_roots(s).len() >= pr.rule_cap {
            break;
        }
        if s.get_node(root).is_none() {
            continue;
        }
        if rule_tokens(s, root) >= pr.repro_threshold {
            let child = duplicate_rule(s, root);
            set_int(s, child, K_SEQ, *seq);
            *seq += 1;
            let depth = get_int(s, root, K_DEPTH).unwrap_or(0);
            set_int(s, child, K_DEPTH, depth + 1);
            // Draw order is fixed regardless of μ (see meta::mutate_at_birth).
            let var = mutate_at_birth(s, child, rng, caps, pr.mu);
            if var.coin_mutant {
                out.mutant += 1;
                if !var.changed {
                    out.noop += 1;
                }
            } else {
                out.clonal += 1;
            }
            set_tokens(s, root, rule_tokens(s, root) - pr.birth_cost);
            set_tokens(s, child, pr.birth_cost);
        }
    }
    out
}
