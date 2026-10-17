//! Locality & leakage track (LK) row shapes: `LeakageRow` (leakage.csv),
//! `ConeRow` (cone.csv), `TwinRow` (twin.csv), and the `ClaimType` tag
//! every row carries (hypersim_leakage_plan_v1.md §1, Appendix A).
//!
//! Pure data + serde only, per this crate's charter.

use serde::{Deserialize, Serialize};

/// Epistemic tag carried by every leakage-track row. Everything computed
/// by observers is `Measured` or `Estimated`; the closure-defect estimator
/// is `Inferred` (analysis layer only). `[Speculative]` never appears in
/// code or CSV by design.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ClaimType {
    Measured,
    #[default]
    Estimated,
    Inferred,
}

impl ClaimType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ClaimType::Measured => "measured",
            ClaimType::Estimated => "estimated",
            ClaimType::Inferred => "inferred",
        }
    }
}

/// Phase of the emergent layer w.r.t. the leakage instruments (v2 §1.5).
/// The detour channel is valid ONLY in `Sparse`; `Dense` means the
/// geometric background has dissolved and the detour channel is blind
/// there (the F-LK1 crossover). Thresholds are calibrated, not assumed
/// (docs/fixture_calibration.md p_peak section).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LeakagePhase {
    #[default]
    Sparse,
    Crossover,
    Dense,
}

impl LeakagePhase {
    pub fn as_str(&self) -> &'static str {
        match self {
            LeakagePhase::Sparse => "sparse",
            LeakagePhase::Crossover => "crossover",
            LeakagePhase::Dense => "dense",
        }
    }
}

/// Whether cone statistics may be read for a window (v2 §1.3 phase gate).
/// `OutOfPhase` rows carry NO cone numbers — emitting them anyway is the
/// `cone_emitted_out_of_phase` bug, not a judgment call.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ConeStatus {
    #[default]
    InPhase,
    OutOfPhase,
}

impl ConeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ConeStatus::InPhase => "in_phase",
            ConeStatus::OutOfPhase => "out_of_phase",
        }
    }
}

/// One row per leakage-emission window (leakage.csv). Detour values are
/// `f32::INFINITY` for wormholes (serialized as `inf`); percentile fields
/// are therefore finite only when the corresponding quantile edge has a
/// finite detour.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct LeakageRow {
    pub step: u64,
    pub detour_p50: f32,
    pub detour_p95: f32,
    pub wormhole_frac: f32,
    pub cut_exponent: Option<f32>,
    pub cut_fit_r2: Option<f32>,
    /// `cut_exponent - (d_eff - 1)/d_eff`; requires d_eff from the SAME
    /// window (never a stale value).
    pub area_law_defect: Option<f32>,
    /// Ball-growth dimension estimate (the d_eff estimator).
    pub d_bfs: Option<f32>,
    /// Spectral dimension, if that observer is active (reserved).
    pub d_spec: Option<f32>,
    /// `|d_bfs - d_spec|` when both present.
    pub dim_mismatch: Option<f32>,
    /// v2 §1.1: sampled estimate of the ABSOLUTE wormhole edge count
    /// (`wormhole_frac × E`). Fractions dilute under growth; counts
    /// don't — the dilution-confound guard.
    pub wormhole_count_est: Option<u32>,
    /// v2 §1.1: connected components of the (sampled) wormhole-edge
    /// subgraph. Edge count measures refinement of a handle; component
    /// count measures handles. Divergence is morphology signal.
    pub wormhole_components: Option<u32>,
    /// v2 §1.5: calibrated phase of this window (detour channel valid
    /// only in `sparse`).
    pub phase: LeakagePhase,
    /// v2 §1.3: whether cone statistics may be emitted for this window.
    pub cone_status: ConeStatus,
    /// LK-R energy audit (v2 §6.2), populated only when the metered rule
    /// family is active: current total energy over all nodes…
    pub e_total_now: Option<f64>,
    /// …cumulative subdivision-rule firings…
    pub n_subdivisions: Option<u64>,
    /// v4 §6.3 ledger: cumulative energy vanished with fused nodes.
    pub e_fused_total: Option<f64>,
    /// v4 §7: cumulative fuse-then-resubdivide-at-same-site events
    /// (within `churn_window_steps`). Measured, never prevented.
    pub churn_events: Option<u32>,
    /// v10 §3.2: cumulative energy injected by `[influx]`, as an
    /// ACCUMULATOR of what was actually written — never Φ·T, which would
    /// make the audit self-confirming (hazard §9.1). `None` when influx is
    /// disabled, which keeps the pre-v10 audit expression bit-identical.
    #[serde(default)]
    pub e_injected_total: Option<f64>,
    /// …and `e_total_now + c·n_subdivisions + e_fused_total − E₀`, which
    /// must be EXACTLY 0.0 (any drift is an accounting bug and halts
    /// interpretation).
    pub audit_residual: Option<f64>,
    /// v7 §1.1 INSTRUMENT-RANGE PRECONDITION, as a schema field rather
    /// than prose: is the wormhole census's detection domain actually
    /// reached this window?
    ///
    /// The census reports an edge as a wormhole only when its detour
    /// exceeds `detour_r_max`, and a filament's detour grows with its
    /// chain length — but the growth law is REGIME-DEPENDENT (measured
    /// in Q7-1: ≈ 2L+2 in the pump, where return paths must descend a
    /// second chain, vs ≈ L+3 in the swap regime, where the ambient
    /// lattice is intact). Rather than encode either formula, this is
    /// computed exactly: the detour of an edge on the LONGEST filament
    /// chain. Detour increases with chain length, so if that edge is not
    /// a wormhole then no filament edge is, and a zero census is
    /// `null (out-of-range)` — not a claim that no structure exists.
    ///
    /// `Some(true)` = domain reached (a zero is a real population zero);
    /// `Some(false)` = out of range (a zero says nothing about
    /// population); `None` = no filament present, or leakage inactive.
    pub census_in_range: Option<bool>,
    pub claim_type: ClaimType,
}

/// One row per sampled event (cone.csv, LK2 §1.3). All `estimated`.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct ConeRow {
    pub step: u64,
    pub event_id: u64,
    pub lhs_size: u32,
    /// Steps of causal future examined for this event.
    pub delta: u64,
    /// p95 of descendant graph distances from the event's site.
    pub cone_radius_p95: f32,
    /// Fraction of dynamics-channel descendants beyond v_eff*dt + slack.
    pub anomalous_frac_dynamics: f32,
    /// Same, for descendants reached through a cleanup-attributed causal
    /// edge (engine bookkeeping channel, reported separately by mandate).
    pub anomalous_frac_cleanup: f32,
    /// Descendants whose site nodes were absent from the reference
    /// snapshot (excluded from distances; the exclusion is recorded).
    pub excluded_descendants: u32,
    /// v2 §1.3: reported alongside every cone statistic — how many edges
    /// were censored from the reference metric for this row.
    pub censor_set_size: u32,
    /// v2 §1.3 phase gate: `out_of_phase` rows carry no cone numbers
    /// (the numeric fields are zero-filled and MUST NOT be read).
    pub cone_status: ConeStatus,
    pub claim_type: ClaimType,
}

/// One row per window of the tracked planted-shortcut lineage
/// (tracked.csv, v2 §4). Lineage membership is `measured`; the
/// detour-derived columns are `estimated`.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct TrackedRow {
    pub window: u64,
    pub step: u64,
    /// Edges currently in the tracked lineage (refinement inflates this).
    pub tracked_edge_count: u32,
    /// Connected components of the tracked-edge subgraph (the handle
    /// census; invariant under refinement of a single handle).
    pub tracked_components: u32,
    /// Tracked edges whose detour exceeds the cap (exact, not sampled).
    pub tracked_wormhole_count: u32,
    pub tracked_detour_p50: f32,
    pub tracked_detour_p95: f32,
    /// Longest path (in tracked edges) within a single tracked component —
    /// the throat-formation signal (V2 verdict).
    pub throat_length_max: u32,
    /// V3 signature: growth nucleus at a tracked far endpoint preceding
    /// the ambient front.
    pub far_nucleus_flag: bool,
    /// v3 §4 (EXP-L10): running max of energy at any tracked-edge
    /// endpoint ("mouth") — has transport actually delivered ≥ θ?
    /// Empty until any mouth carries an energy key (non-LK-R dynamics).
    pub mouth_energy_max: Option<f32>,
    /// First step at which far_nucleus_flag fired (empty until then).
    pub t_far_nucleus: Option<u64>,
    /// First step at which the ambient front reached the initial far-
    /// mouth distance (control arm's ambient arrival timing).
    pub t_ambient_front_at_far: Option<u64>,
    pub claim_type: ClaimType,
}

/// One row per window of a twin run (twin.csv, LK3 §6). All `estimated`.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct TwinRow {
    pub window: u64,
    pub step: u64,
    /// Fraction of surviving anchors whose radius-r_a WL ball profiles
    /// differ between branches (label-invariant by construction).
    pub damage_frac: f32,
    /// p95 of pre-perturbation graph distances from the perturbation site
    /// among damaged anchors (NaN when no anchor is damaged).
    pub front_radius: f32,
    /// Running slope of front_radius vs step.
    pub v_damage_running: f32,
    /// Anchors dropped because they were deleted in either branch.
    pub anchor_attrition: u32,
    pub claim_type: ClaimType,
}

/// One row per snapshot of the chain census (chains.csv, plan v3 §3 /
/// Appendix B): maximal paths of degree-2, generation > 0 nodes — the
/// primary EXP-L9 readout. All `estimated` (the census is exact on the
/// snapshot, but snapshots sample the trajectory).
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct ChainRow {
    pub snapshot_step: u64,
    pub chains_count: u32,
    /// Chain length = node count of the maximal path.
    pub chain_length_p50: f32,
    pub chain_length_p95: f32,
    pub chain_length_max: u32,
    pub claim_type: ClaimType,
}

/// One row per HANDLE (v5 §5, lifetimes.csv): a wormhole component —
/// connected component of the generation > 0 subgraph — identified per
/// the §1.1 handle-identity principle by its sorted gen-0 anchor set,
/// tracked offline across a run's persisted snapshots. All `estimated`:
/// birth/death are resolved only to snapshot cadence.
/// `resolution_bound_steps` carries the bound (the maximum
/// inter-snapshot gap); lifetime claims below 2× that bound are not
/// made (v5 §9.5). `handle_id` is RUN-SCOPED (anchor NodeIds are stable
/// within a run — gen-0 nodes are non-removable by the generation
/// guard — but not across runs); cross-run comparison uses lifetime
/// DISTRIBUTIONS, never ids.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct LifetimeRow {
    pub handle_id: String,
    /// First snapshot step at which the handle is observed.
    pub birth_step: u64,
    /// First snapshot step at which the handle is ABSENT after having
    /// been observed; empty = open at run end.
    pub death_step: Option<u64>,
    /// Max component node count over the handle's observations.
    pub max_length: u32,
    /// Max count of edges incident to component nodes (interior +
    /// anchor edges) over the handle's observations.
    pub max_edges: u32,
    /// Max gen-0 anchor count over the handle's observations.
    pub anchor_count: u32,
    pub resolution_bound_steps: u64,
    pub claim_type: ClaimType,
}

/// One row per (window, rule, guard) of the guard-engagement counters
/// (guards.csv, plan v4 §1.1). All `measured`. guard_id encodes the
/// guard family + declaration index: `prop{i}` (node_property_guards),
/// `cmp{i}` (node_property_compares), `deg{i}` (node_degree_compares).
/// vars_distinct is a binding constraint, not a counted guard.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct GuardRow {
    pub window: u64,
    pub rule: String,
    pub guard_id: String,
    pub evals: u64,
    pub rejects: u64,
    pub claim_type: ClaimType,
}
