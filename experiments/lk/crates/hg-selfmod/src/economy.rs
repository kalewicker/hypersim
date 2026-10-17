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
use std::collections::{HashMap, HashSet};

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

/// How a productive rule's income is determined (v5 §3.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IncomeMode {
    /// v4: flat `fire_reward` per productive rule per step, capped by `free`.
    /// Fitness is binary — every productive rule nets exactly
    /// `fire_reward − rent`, so there is no differential reproduction.
    Flat,
    /// v5: a fixed per-step stream shared in proportion to productive
    /// firings. Rules compete for a bounded pool and cannot mint.
    Proportional,
}

impl std::str::FromStr for IncomeMode {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, ()> {
        match s {
            "flat" => Ok(IncomeMode::Flat),
            "proportional" | "prop" => Ok(IncomeMode::Proportional),
            _ => Err(()),
        }
    }
}

/// Reproduction economics (v5 §3.3). v4 behaviour is `Full`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FissionMode {
    /// Parent pays `birth_cost` from its own tokens (v4 control).
    Full,
    /// Parent and child split the parent's tokens, parent never dropped
    /// below `split_floor`.
    Split,
    /// Parent pays a reduced cost; child starts correspondingly poorer.
    Partial,
    /// As `Full`, plus a cooldown before the same rule may fission again.
    Refractory,
}

impl std::str::FromStr for FissionMode {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, ()> {
        match s {
            "full" => Ok(FissionMode::Full),
            "split" => Ok(FissionMode::Split),
            "partial" => Ok(FissionMode::Partial),
            "refractory" => Ok(FissionMode::Refractory),
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
    // ---- v5 ----
    pub income_mode: IncomeMode,
    /// Per-step income stream shared under `Proportional` (v5 §3.1a).
    pub income_pool: i64,
    /// Cost per match candidate (v5 §3.1b). Charged against the PREVIOUS
    /// step's candidate count so rent keeps its v4 position in the step
    /// order. Zero reproduces v4 exactly.
    pub match_cost_coeff: f64,
    pub fission_mode: FissionMode,
    pub split_floor: i64,
    pub partial_cost: i64,
    pub refractory_steps: i64,
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

/// Per-rule metabolic cost: `base_rent + match_cost_coeff × candidates`,
/// where `candidates` is the rule's PREVIOUS-step match count (v5 §3.1b).
/// Using the previous step keeps this phase in its v4 position in the step
/// order, so the gradient arm stays order-comparable to the v4 arm.
pub fn rule_cost(s: &HypergraphStore, root: NodeId, pr: &EconomyParams) -> i64 {
    let cand = get_int(s, root, K_LAST_CANDIDATES).unwrap_or(0).max(0);
    pr.rent + (pr.match_cost_coeff * cand as f64).round() as i64
}

/// Cost + starvation. Runs BEFORE the object phase (v3 §7 step order).
/// Records each surviving rule's charge into `costs` so `net_income` can be
/// formed once income is distributed.
pub fn rent_and_starve(
    s: &mut HypergraphStore,
    led: &mut Ledger,
    pr: &EconomyParams,
    costs: &mut HashMap<NodeId, i64>,
) -> u32 {
    let mut deaths = 0u32;
    for root in rule_roots(s) {
        let t = rule_tokens(s, root);
        let due = rule_cost(s, root, pr);
        if t < due {
            // Cannot pay: the rule starves. Its remaining tokens are
            // recycled (death returns capital in both economies); the
            // shortfall is simply never collected.
            led.recycle(t);
            delete_rule(s, root);
            deaths += 1;
        } else {
            set_tokens(s, root, t - due);
            led.collect_rent(due, pr);
            costs.insert(root, due);
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
    let mut counts: HashMap<NodeId, u32> = HashMap::new();
    for &r in productive {
        counts.insert(r, 1);
    }
    let mut income = HashMap::new();
    distribute_income(s, &counts, led, pr, &mut income);
}

/// Pay productive rules, recording each rule's income into `income`.
///
/// * `Flat` (v4): every productive rule receives `fire_reward`, capped by
///   `free`. Income does not depend on how much a rule did — fitness is
///   binary, which is what G5.1 measures.
/// * `Proportional` (v5): a fixed stream `income_pool` is shared in
///   proportion to productive firings,
///   `income_i = floor(pool × productive_i / Σ productive_j)`, with the
///   integer remainder handed out one token at a time in canonical
///   (sorted-key) order. Rules compete for a bounded stream and cannot
///   mint, so the anti-Goodhart property is preserved.
pub fn distribute_income(
    s: &mut HypergraphStore,
    productive_per_rule: &HashMap<NodeId, u32>,
    led: &mut Ledger,
    pr: &EconomyParams,
    income: &mut HashMap<NodeId, i64>,
) {
    // Canonical order: determinism when the pool runs out or a remainder
    // has to be broken (v4 fixed exactly this latent nondeterminism).
    let mut roots: Vec<NodeId> = productive_per_rule
        .iter()
        .filter(|(r, c)| **c > 0 && s.get_node(**r).is_some())
        .map(|(r, _)| *r)
        .collect();
    roots.sort_by_key(|&r| key_u64(r));
    if roots.is_empty() {
        return;
    }

    match pr.income_mode {
        IncomeMode::Flat => {
            for root in roots {
                let pay = pr.fire_reward.min(led.free);
                if pay > 0 {
                    set_tokens(s, root, rule_tokens(s, root) + pay);
                    led.free -= pay;
                    *income.entry(root).or_insert(0) += pay;
                }
            }
        }
        IncomeMode::Proportional => {
            let pool = pr.income_pool.min(led.free).max(0);
            if pool == 0 {
                return;
            }
            let total: i64 = roots.iter().map(|r| productive_per_rule[r] as i64).sum();
            if total <= 0 {
                return;
            }
            let mut handed = 0i64;
            let mut shares: Vec<(NodeId, i64)> = Vec::with_capacity(roots.len());
            for &root in &roots {
                let p = productive_per_rule[&root] as i64;
                let share = pool * p / total; // floor
                shares.push((root, share));
                handed += share;
            }
            // Deterministic remainder: one token each, canonical order.
            let mut remainder = pool - handed;
            for (_, share) in shares.iter_mut() {
                if remainder <= 0 {
                    break;
                }
                *share += 1;
                remainder -= 1;
            }
            for (root, share) in shares {
                if share > 0 {
                    set_tokens(s, root, rule_tokens(s, root) + share);
                    led.free -= share;
                    *income.entry(root).or_insert(0) += share;
                }
            }
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
    step: i64,
) -> FissionOutcome {
    funded_fission_inner(
        s,
        rng,
        pr,
        caps,
        seq,
        step,
        &mut |s| rule_roots(s).len() < pr.rule_cap,
        &mut |_, _, _| {},
    )
}

/// Diagnostic birth genealogy from the same native operation and RNG path.
pub fn funded_fission_recorded(
    s: &mut HypergraphStore,
    rng: &mut ChaCha8Rng,
    pr: &EconomyParams,
    caps: &Caps,
    seq: &mut i64,
    step: i64,
) -> (FissionOutcome, Vec<(NodeId, NodeId)>) {
    let mut events = Vec::new();
    let outcome = funded_fission_inner(
        s,
        rng,
        pr,
        caps,
        seq,
        step,
        &mut |s| rule_roots(s).len() < pr.rule_cap,
        &mut |_, parent, child| {
            events.push((parent, child));
        },
    );
    (outcome, events)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BoundedBirth {
    pub parent: NodeId,
    pub child: NodeId,
    pub population_before: usize,
    pub retired_tokens: Option<i64>,
}

/// Experimental admission/parent retirement, sharing every native birth operation.
pub fn funded_fission_bounded_recorded(
    s: &mut HypergraphStore,
    rng: &mut ChaCha8Rng,
    pr: &EconomyParams,
    caps: &Caps,
    seq: &mut i64,
    step: i64,
    limit: usize,
    replace: bool,
    ledger: &mut Ledger,
) -> (FissionOutcome, Vec<BoundedBirth>) {
    assert!(limit > 0 && limit < pr.rule_cap && rule_roots(s).len() <= limit);
    let mut events = Vec::new();
    let outcome = funded_fission_inner(
        s,
        rng,
        pr,
        caps,
        seq,
        step,
        &mut |s| replace || rule_roots(s).len() < limit,
        &mut |s, parent, child| {
            let population_before = rule_roots(s).len() - 1;
            let retired_tokens = if replace && population_before == limit {
                let capital = rule_tokens(s, parent);
                ledger.free += capital;
                delete_rule(s, parent);
                Some(capital)
            } else {
                None
            };
            assert!(rule_roots(s).len() <= limit);
            events.push(BoundedBirth {
                parent,
                child,
                population_before,
                retired_tokens,
            });
        },
    );
    (outcome, events)
}

fn funded_fission_inner(
    s: &mut HypergraphStore,
    rng: &mut ChaCha8Rng,
    pr: &EconomyParams,
    caps: &Caps,
    seq: &mut i64,
    step: i64,
    admit: &mut impl FnMut(&HypergraphStore) -> bool,
    on_birth: &mut impl FnMut(&mut HypergraphStore, NodeId, NodeId),
) -> FissionOutcome {
    let mut out = FissionOutcome {
        clonal: 0,
        mutant: 0,
        noop: 0,
    };
    for root in rule_roots(s) {
        if !admit(s) {
            break;
        }
        if s.get_node(root).is_none() {
            continue;
        }
        if rule_tokens(s, root) < pr.repro_threshold {
            continue;
        }
        if pr.fission_mode == FissionMode::Refractory {
            let last = get_int(s, root, K_LAST_FISSION).unwrap_or(i64::MIN / 2);
            if step - last < pr.refractory_steps {
                continue;
            }
        }
        // Token split is an internal transfer: no ledger movement.
        let parent_tokens = rule_tokens(s, root);
        let (parent_after, child_tokens) = match pr.fission_mode {
            FissionMode::Full | FissionMode::Refractory => {
                (parent_tokens - pr.birth_cost, pr.birth_cost)
            }
            FissionMode::Partial => (parent_tokens - pr.partial_cost, pr.partial_cost),
            FissionMode::Split => {
                // Halve, but never drop the parent below its floor; the
                // child gets whatever the parent actually gave up.
                let half = parent_tokens / 2;
                let keep = half.max(pr.split_floor).min(parent_tokens);
                (keep, parent_tokens - keep)
            }
        };
        if child_tokens <= 0 || parent_after < 0 {
            continue;
        }

        let child = duplicate_rule(s, root);
        set_int(s, child, K_SEQ, *seq);
        *seq += 1;
        set_int(
            s,
            child,
            K_DEPTH,
            get_int(s, root, K_DEPTH).unwrap_or(0) + 1,
        );
        set_int(s, child, K_LAST_FISSION, step);
        set_int(s, root, K_LAST_FISSION, step);
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
        set_tokens(s, root, parent_after);
        set_tokens(s, child, child_tokens);
        on_birth(s, root, child);
    }
    out
}

/// Forced duplication for E-stoich (v5 §5.1): clone every live rule whose
/// `K_MEMBER` is in `members`, with no mutation and no token cost beyond the
/// normal split, at a chosen step. This perturbs cycle stoichiometry without
/// involving the reproduction economy or the mutation operators at all.
pub fn forced_duplicate(s: &mut HypergraphStore, members: &[i64], seq: &mut i64, step: i64) -> u32 {
    let mut n = 0u32;
    for root in rule_roots(s) {
        let m = get_int(s, root, K_MEMBER).unwrap_or(-1);
        if !members.contains(&m) {
            continue;
        }
        let tokens = rule_tokens(s, root);
        let half = tokens / 2;
        let child = duplicate_rule(s, root);
        set_int(s, child, K_SEQ, *seq);
        *seq += 1;
        set_int(
            s,
            child,
            K_DEPTH,
            get_int(s, root, K_DEPTH).unwrap_or(0) + 1,
        );
        set_int(s, child, K_LAST_FISSION, step);
        set_tokens(s, root, tokens - half);
        set_tokens(s, child, half);
        n += 1;
    }
    n
}
