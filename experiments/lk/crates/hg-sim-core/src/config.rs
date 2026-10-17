//! Simulation configuration. Schema stays additive (v4 §6.4): every new
//! section is Option<T> / #[serde(default)].

use hg_rules::StepMode;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SimConfig {
    pub experiment_name: String,
    #[serde(default)]
    pub run_id: String,
    #[serde(default = "default_out_dir")]
    pub out_dir: PathBuf,
    pub seed: u64,
    pub rules: RuleSetConfig,
    #[serde(default)]
    pub initial: InitialConditionConfig,
    #[serde(default)]
    pub engine: EngineConfig,
    #[serde(default)]
    pub evaluator: EvaluatorConfig,
    #[serde(default)]
    pub causal: CausalConfig,
    #[serde(default)]
    pub causal_language: CausalLanguageConfig,
    #[serde(default)]
    pub ei_coarse_graining: EiCoarseGrainingConfig,
    #[serde(default)]
    pub leakage: LeakageConfig,
    #[serde(default)]
    pub lk_rules: LkRulesConfig,
    #[serde(default)]
    pub campaign: CampaignConfig,
    #[serde(default)]
    pub relaxation: RelaxationConfig,
    /// Constant energy influx (v10 §3.1) — removes the seeded-budget
    /// scaffold. `skip_serializing_if` keeps a DISABLED block out of the
    /// serialized config entirely, so `config_hash` and every archived
    /// byte-identity regression are unchanged by this field's existence
    /// (the same principle the feature-gated `alife`/`cognition` fields
    /// use, achieved without a feature gate).
    #[serde(default, skip_serializing_if = "crate::influx::InfluxConfig::is_disabled")]
    pub influx: crate::influx::InfluxConfig,
    #[serde(default)]
    pub output: OutputOptions,
    /// The ALife layer. Feature-gated so that a build WITHOUT `alife` does not
    /// even carry the field — config serialization (and `config_hash`) is then
    /// byte-unchanged, and a `[alife]` TOML section is simply ignored. With the
    /// feature, this drives rule + classifier injection in the runner.
    #[cfg(feature = "alife")]
    #[serde(default)]
    pub alife: AlifeConfig,
    /// The learning layer's `[cognition]` section (docs/policy_config.md).
    /// Feature-gated on the same principle as `alife`: a build without
    /// `cognition` does not carry the field at all, so config
    /// serialization and `config_hash` are byte-unchanged and a
    /// `[cognition]` TOML section is simply ignored. `None` — the section
    /// absent — means no policy is ever constructed, which is what makes
    /// the leak detector's hash equality meaningful.
    #[cfg(feature = "cognition")]
    #[serde(default)]
    pub cognition: Option<hg_cognition::CognitionConfig>,
}

/// ALife run configuration (only present under the `alife` feature).
#[cfg(feature = "alife")]
#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct AlifeConfig {
    /// When true, the runner appends `hg_alife::alife_base_toml()` to the rule
    /// set and injects `HypergraphMotifClassifier` into the causal observer.
    #[serde(default)]
    pub enabled: bool,
    /// When true, mark exactly ONE initial substrate node with
    /// `carrier_marker = true` (scope §6.3), so `bootstrap_seed` has a node to
    /// sprout the first carrier from. The node is chosen deterministically from
    /// the run seed. Perturbation-seeding — permitted; per-generation injection
    /// is not (and is not expressible).
    #[serde(default)]
    pub seed_carrier_marker: bool,
    /// Initial energy (property key 1) written to every non-carrier node at run
    /// start (when `enabled`). Its PRIMARY job is to ensure the energy key
    /// EXISTS on substrate nodes: `add`/`subtract` effects are skipped on a
    /// missing key, so without this influx and harvest silently no-op on a
    /// freshly generated substrate. Default 0.0 = "start empty, let influx fill".
    #[serde(default)]
    pub initial_substrate_energy: f64,
    /// Override the `influx` rule's per-firing energy add-value (S3.5). `None`
    /// keeps the `alife_base.toml` default (2.0). This is THE swept parameter of
    /// S6.1: the turnover regime lives at a sub-sustain influx (~0.34) that a bare
    /// `alife_base.toml` cannot express. Exposing it as config (not an in-test
    /// patch) is what makes the S6.2 exit-criteria run reproducible through the
    /// production runner.
    #[serde(default)]
    pub influx_rate: Option<f64>,
    /// Override the BOOTSTRAP carrier's `replication_threshold` genome value
    /// (key 11, set by `bootstrap_seed`). `None` keeps the `alife_base.toml`
    /// default (15.0). This seeds an INITIAL CONDITION, not a physics constant:
    /// descendants inherit it via copy_from+jitter and, being node-funded, drift
    /// it toward the floor. The turnover attractor is entered from a low bootstrap
    /// value (~3.0); see docs/experiment5_scorecard.md.
    #[serde(default)]
    pub bootstrap_replication_threshold: Option<f64>,
    /// Override `child_node_cost` (S3.6), the energy a colonised target site must
    /// hold and pay to fund a birth. `None` keeps the `alife_base.toml` default
    /// (10.0). This is a SYMMETRIC SCALE: it drives four coupled values at once —
    /// the fundability guard (`target.energy ≥ c`), the target payment (`−c`), the
    /// child's birth energy (`0.8·c`), and the death recycle (`0.8·c`, matched to
    /// birth energy so the ledger balances). Because it scales birth and death in
    /// lockstep it does NOT reopen the turnover window (refuted by symmetry, docs
    /// §8) — exposed for completeness / stillbirth-floor experiments only. The
    /// stillbirth floor is c ≳ 0.7 (birth energy 0.8c must clear maintenance 0.5).
    #[serde(default)]
    pub child_node_cost: Option<f64>,
    /// When true, `harvest` and `metabolise` become GENOME-SCALED (deviation #4):
    /// harvest transfers `carrier.harvest_rate` (key 10) instead of a fixed 1.0, and
    /// metabolise drains `carrier.maintenance_cost` (key 12) instead of a fixed 0.5.
    /// The two previously-inert drift-null traits become coupled to fitness, so
    /// criterion 5's selection-vs-drift question can be re-examined on them. The
    /// bootstrap `maintenance_cost` is set to 0.5 (matching the fixed quantum) so the
    /// initial economy is unchanged; the site-floor guard becomes
    /// `site.energy ≥ carrier.harvest_rate` so a drifted-up harvest_rate can't drive a
    /// site negative. Off by default ⟹ the fixed-quantum economy (and every calibrated
    /// test) is untouched. Requires the `add_from_property`/`subtract_from_property`
    /// effect ops. See `docs/experiment5b_active_traits_preregistration.md`.
    #[serde(default)]
    pub genome_scaled_metabolism: bool,
    /// When true, `replicate` becomes PARENT-FUNDED instead of node-funded (§1
    /// option 2): the parent pays the child cost from its OWN energy, and the
    /// target site pays nothing (it only provides the unoccupied dispersal
    /// location). This reopens the node-funded decision on its merits: node-funded
    /// gave `replication_threshold` no opposing force, so it collapses to the floor
    /// (criterion-5 degeneracy). Parent-funding installs a real survival trade-off —
    /// a carrier must hold energy ≥ child cost to survive reproducing — giving
    /// `replication_threshold` an interior optimum near the child cost. It is NOT a
    /// hand-injected fitness term: the total energy per birth (and the ledger
    /// identity) is unchanged; only WHO pays moves from target to parent. Off by
    /// default ⟹ node-funded (every calibrated test unchanged). See
    /// `docs/experiment5c_parent_funded_preregistration.md`.
    #[serde(default)]
    pub parent_funded_replication: bool,
    /// Energy-proportional reproductive tax (only with `parent_funded_replication`).
    /// `Some(tau)`: on each birth, AFTER the parent pays the fixed child cost, the
    /// parent additionally loses fraction `tau` of its remaining energy to entropy
    /// (`parent.energy ×= 1 − tau`). This makes reproducing costly *in proportion to
    /// wealth*, so it "bites the energy-rich too": a reliably-rich carrier can no
    /// longer treat `replication_threshold` as a neutral (drifting) trait, because
    /// reproducing while overflowing now wastes real energy. Intended to close the
    /// residual degenerate basin of Experiment 5c (some rich lineages still collapse
    /// to the floor). `None` ⟹ exactly 5c (fixed cost only). See
    /// `docs/experiment5d_rich_penalty_preregistration.md`.
    #[serde(default)]
    pub repro_energy_tax: Option<f64>,
    /// Reproductive senescence (route B): `Some(cap)` makes each birth age the parent
    /// (`lifetime` key 21 += 1), and a carrier whose `lifetime ≥ cap` dies — a
    /// death route tied to reproduction RATE, decoupled from energy. This targets the
    /// residual deep drift sink of Experiment 5d: a carrier with an always-satisfied
    /// (≪ 0) threshold reproduces chronically, so it exhausts its lifetime cap fast and
    /// dies regardless of how well-fed it is — a cost no energy penalty can impose,
    /// because it depends on how OFTEN you reproduce, not on your wealth. `None` ⟹ no
    /// aging (energy-death only). Independent of the funding mode. See
    /// `docs/experiment5e_aging_preregistration.md`.
    #[serde(default)]
    pub reproductive_lifetime_cap: Option<f64>,
}

fn default_out_dir() -> PathBuf {
    PathBuf::from("output")
}

impl SimConfig {
    pub fn from_toml(content: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(content)
    }

    /// Deterministic hash of the configuration (manifest provenance).
    pub fn config_hash(&self) -> String {
        use rustc_hash::FxHasher;
        use std::hash::{Hash, Hasher};
        let ser = toml::to_string(self).unwrap_or_default();
        let mut h = FxHasher::default();
        ser.hash(&mut h);
        format!("{:016x}", h.finish())
    }

    /// Effective run id: explicit, or derived deterministically-shaped
    /// from experiment + seed + timestamp.
    pub fn effective_run_id(&self) -> String {
        if !self.run_id.is_empty() {
            return self.run_id.clone();
        }
        format!(
            "{}_{}_{}",
            self.experiment_name,
            self.seed,
            chrono::Utc::now().format("%Y%m%dT%H%M%S")
        )
    }

    /// The Run Parameters section content (report §2): the experimental
    /// configuration itself, distinct from the Manifest's bookkeeping.
    pub fn run_parameter_lines(&self) -> Vec<(String, String)> {
        vec![
            ("rules.source".into(), format!("{:?}", self.rules.source)),
            (
                "rules.name/path".into(),
                self.rules
                    .builtin_name
                    .clone()
                    .or(self.rules.file_path.as_ref().map(|p| p.display().to_string()))
                    .unwrap_or_else(|| "(inline)".into()),
            ),
            ("initial.type".into(), format!("{:?}", self.initial.condition_type)),
            ("initial.size".into(), format!("{}x{} / n={}", self.initial.width, self.initial.height, self.initial.node_count)),
            ("engine.step_mode".into(), format!("{:?}", self.engine.step_mode)),
            ("engine.max_steps".into(), self.engine.max_steps.to_string()),
            ("engine.window".into(), self.engine.metrics_window_interval.to_string()),
            ("engine.max_pairings_per_step".into(), self.engine.max_pairings_per_step.to_string()),
            ("engine.thread_count".into(), self.engine.thread_count.to_string()),
            ("evaluator.enabled".into(), self.evaluator.enabled.to_string()),
            ("causal.enabled".into(), self.causal.enabled.to_string()),
            ("seed".into(), self.seed.to_string()),
        ]
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RuleSetConfig {
    #[serde(default)]
    pub source: RuleSource,
    #[serde(default)]
    pub builtin_name: Option<String>,
    #[serde(default)]
    pub file_path: Option<PathBuf>,
    #[serde(default)]
    pub inline_toml: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RuleSource {
    #[default]
    Builtin,
    File,
    Inline,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InitialConditionConfig {
    #[serde(default)]
    pub condition_type: InitialConditionType,
    #[serde(default = "d_dim")]
    pub width: usize,
    #[serde(default = "d_dim")]
    pub height: usize,
    #[serde(default = "d_arity")]
    pub arity: usize,
    #[serde(default)]
    pub periodic: bool,
    #[serde(default)]
    pub defect_density: f64,
    #[serde(default = "d_nodes")]
    pub node_count: usize,
    #[serde(default = "d_edges")]
    pub edge_count: usize,
    #[serde(default = "d_arity")]
    pub max_arity: usize,
    #[serde(default = "d_labels")]
    pub label_range: u32,
    #[serde(default = "d_avg_degree")]
    pub avg_degree: f64,
}

fn d_dim() -> usize { 10 }
fn d_arity() -> usize { 2 }
fn d_nodes() -> usize { 30 }
fn d_edges() -> usize { 40 }
fn d_labels() -> u32 { 3 }
fn d_avg_degree() -> f64 { 2.0 }

impl Default for InitialConditionConfig {
    fn default() -> Self {
        Self {
            condition_type: InitialConditionType::default(),
            width: d_dim(),
            height: d_dim(),
            arity: d_arity(),
            periodic: false,
            defect_density: 0.0,
            node_count: d_nodes(),
            edge_count: d_edges(),
            max_arity: d_arity(),
            label_range: d_labels(),
            avg_degree: d_avg_degree(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum InitialConditionType {
    #[default]
    PureCrystal,
    CrystalWithDefects,
    WhiteNoise,
    SparseSoup,
    MinimalSeed,
    /// Periodic ring of overlapping 3-edges with exactly one mark —
    /// the substrate the glider rule needs (uses `node_count`).
    GliderRing,
    /// Triangulated torus (LK fixtures §3): `width` × `width` nodes,
    /// plus planted shortcut edges at density `defect_density` (seeded).
    /// The EXP-L2 initial condition.
    TriLattice,
    /// `TriLattice` with node (0,0) marked label 1 — the excitation-front
    /// reference dynamic's substrate (LK2 validation).
    TriLatticeExcited,
    Empty,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EngineConfig {
    #[serde(default)]
    pub step_mode: StepMode,
    #[serde(default = "d_max_steps")]
    pub max_steps: u64,
    #[serde(default = "d_window")]
    pub metrics_window_interval: u64,
    #[serde(default = "d_pairings")]
    pub max_pairings_per_step: usize,
    #[serde(default = "d_threads")]
    pub thread_count: usize,
}

fn d_max_steps() -> u64 { 10_000 }
fn d_window() -> u64 { 512 }
fn d_pairings() -> usize { 1000 }
fn d_threads() -> usize { num_cpus::get() }

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            step_mode: StepMode::MaxParallel,
            max_steps: d_max_steps(),
            metrics_window_interval: d_window(),
            max_pairings_per_step: d_pairings(),
            thread_count: d_threads(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EvaluatorConfig {
    #[serde(default = "d_true")]
    pub enabled: bool,
}

fn d_true() -> bool { true }

impl Default for EvaluatorConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CausalConfig {
    #[serde(default = "d_true")]
    pub enabled: bool,
    #[serde(default = "d_radius")]
    pub wl_radius: u8,
}

fn d_radius() -> u8 { 1 }

impl Default for CausalConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            wl_radius: 1,
        }
    }
}

/// `[causal_language]` — EI/CE calculator knobs
/// (hypersim_build_ei_calculator.md; not_nothing_v2_plan.md §6.5).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CausalLanguageConfig {
    #[serde(default = "d_ei_interval")]
    pub ei_interval_steps: u64,
    /// Below this many observed transitions EI is not emitted and no CE
    /// claim is made — the certainty table requires >= 1000 for
    /// Empirical (the plan's prose and TOML say 1000; its code sketch's
    /// 100 is overruled).
    #[serde(default = "d_ei_min")]
    pub ei_minimum_transitions: u64,
}

fn d_ei_interval() -> u64 { 1000 }
fn d_ei_min() -> u64 { 1000 }

impl Default for CausalLanguageConfig {
    fn default() -> Self {
        Self {
            ei_interval_steps: d_ei_interval(),
            ei_minimum_transitions: d_ei_min(),
        }
    }
}

/// `[ei_coarse_graining]` — which macro partitions to evaluate.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EiCoarseGrainingConfig {
    #[serde(default = "d_true")]
    pub try_structural: bool,
    #[serde(default = "d_true")]
    pub try_energetic: bool,
    #[serde(default = "d_true")]
    pub try_functional: bool,
    /// Carrier/substrate partition (Experiment 6). Off by default so runs that
    /// don't ask for it emit byte-identical EI output.
    #[serde(default)]
    pub try_carrier_substrate: bool,
    /// Exhaustive partition search — declared but unimplemented; off.
    #[serde(default)]
    pub try_optimal: bool,
}

impl Default for EiCoarseGrainingConfig {
    fn default() -> Self {
        Self {
            try_structural: true,
            try_energetic: true,
            try_functional: true,
            try_carrier_substrate: false,
            try_optimal: false,
        }
    }
}

/// `[leakage]` — locality & leakage track knobs (hypersim_leakage_plan_v1.md
/// Appendix B). All leakage quantities are window-timescale quantities;
/// the cadence is part of each metric's definition (same caveat as EI).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LeakageConfig {
    #[serde(default = "d_true")]
    pub enabled: bool,
    /// Align with the EI cadence to share windows.
    #[serde(default = "d_lk_cadence")]
    pub cadence_steps: u64,
    /// Detour census sample budget. NOTE the recorded sensitivity floor
    /// (docs/fixture_calibration.md): at 256 samples, sparse planted
    /// populations (p <= 1e-2) do NOT separate at 3 sigma; set to >= the
    /// edge count when sparse-regime sensitivity matters.
    #[serde(default = "d_lk_sample")]
    pub detour_sample_edges: usize,
    #[serde(default = "d_lk_rmax")]
    pub detour_r_max: usize,
    #[serde(default = "d_lk_sources")]
    pub ball_sources: usize,
    /// Cone analysis (LK2) event sampling budget (offline subcommand).
    #[serde(default = "d_lk_cone_events")]
    pub cone_sample_events: usize,
    #[serde(default = "d_lk_cone_delta")]
    pub cone_delta_steps: u64,
    /// Write causal_events.jsonl (required input of the cone analysis).
    #[serde(default)]
    pub cone_events_enabled: bool,
    /// Twin harness (LK3) defaults.
    #[serde(default = "d_lk_anchors")]
    pub twin_anchors: usize,
    #[serde(default = "d_lk_anchor_radius")]
    pub twin_anchor_radius: usize,
    /// Closure-defect estimator (LK4) minimum windows, else Unknown.
    #[serde(default = "d_lk_closure_min")]
    pub closure_min_windows: usize,
    /// Provenance pointer for the phase thresholds (v2 §2) — a document
    /// reference, deliberately not a number.
    #[serde(default = "d_lk_p_peak_source")]
    pub phase_p_peak_source: String,
    /// v6 §5 (A3): dump the raw per-window closure feature vectors to
    /// `closure_samples.json` for offline re-windowing (the window-floor
    /// characterization). Off by default ⟹ no extra output, byte-identical.
    #[serde(default)]
    pub dump_closure_samples: bool,
    /// `[leakage.tracked]` — exact lineage tracking of planted shortcuts
    /// (LK1R; EXP-L2 arms only).
    #[serde(default)]
    pub tracked: LeakageTrackedConfig,
}

/// `[leakage.tracked]` (plan v2 §4).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LeakageTrackedConfig {
    #[serde(default)]
    pub enabled: bool,
    /// Exact detours for the tracked set recomputed every this many
    /// steps (evaluated at window boundaries).
    #[serde(default = "d_lk_tracked_cadence")]
    pub recompute_cadence: u64,
}

fn d_lk_tracked_cadence() -> u64 { 500 }
fn d_lk_p_peak_source() -> String { "fixture_calibration.md".into() }

impl Default for LeakageTrackedConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            recompute_cadence: d_lk_tracked_cadence(),
        }
    }
}

fn d_lk_cadence() -> u64 { 500 }
fn d_lk_sample() -> usize { 256 }
fn d_lk_rmax() -> usize { 12 }
fn d_lk_sources() -> usize { 32 }
fn d_lk_cone_events() -> usize { 128 }
fn d_lk_cone_delta() -> u64 { 200 }
fn d_lk_anchors() -> usize { 64 }
fn d_lk_anchor_radius() -> usize { 3 }
fn d_lk_closure_min() -> usize { 200 }

impl Default for LeakageConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cadence_steps: d_lk_cadence(),
            detour_sample_edges: d_lk_sample(),
            detour_r_max: d_lk_rmax(),
            ball_sources: d_lk_sources(),
            cone_sample_events: d_lk_cone_events(),
            cone_delta_steps: d_lk_cone_delta(),
            cone_events_enabled: false,
            twin_anchors: d_lk_anchors(),
            twin_anchor_radius: d_lk_anchor_radius(),
            closure_min_windows: d_lk_closure_min(),
            phase_p_peak_source: d_lk_p_peak_source(),
            dump_closure_samples: false,
            tracked: LeakageTrackedConfig::default(),
        }
    }
}

/// `[lk_rules]` — the LK-R energy-metered rule family's scaffolding
/// (plan v2 §6). The rule CONSTANTS (θ, q, c, τ) live in the rule TOML
/// files; this section seeds the energy field and carries the values the
/// §6.2 audit needs. Disagreement between config and rule file shows up
/// as `audit_invariant_drift` — loudly, never silently.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LkRulesConfig {
    /// Master switch: initialize energy (key 1 = 0.0 everywhere, E₀ at
    /// the seed site) and run the audit.
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "d_lkr_theta")]
    pub theta: f64,
    #[serde(default = "d_lkr_q")]
    pub q: f64,
    /// Per-subdivision dissipation — the audit's `c`.
    #[serde(default = "d_lkr_c")]
    pub c: f64,
    #[serde(default = "d_lkr_tau")]
    pub tau: f64,
    /// Seed budget E₀ (placed at the seed site).
    #[serde(default = "d_lkr_e_total")]
    pub e_total: f64,
    /// "center" (torus midpoint for TriLattice; deterministic mid-index
    /// otherwise) or "corner" (first node).
    #[serde(default = "d_lkr_seed_placement")]
    pub seed_placement: String,
    /// v3 §3 generation cap. `Some(g)` initializes generation (key 6) =
    /// 0.0 on every IC node — required by the capped rule family's
    /// guard, which silently not-matches on a missing key and would fake
    /// a collapse result (v3 hazard 1). The guard VALUE lives in the
    /// rule TOML (rules/lk_pair_capped.toml), like the other constants;
    /// keep this in sync with it. `None` (default) changes nothing.
    #[serde(default)]
    pub g_max: Option<f64>,
    /// v4 §1.2: the priority order is a SWEPT AXIS, not a constant.
    /// "subdivide_first" (the v2 pinned order, prose name: pump regime)
    /// leaves the rule files untouched; "transport_first" remaps the LK
    /// rule priorities at load (transport above subdivide; fuse stays
    /// last in both regimes). Recorded in every manifest.
    #[serde(default = "d_lkr_priority")]
    pub priority: String,
    /// `[lk_rules.fuse]` — the v4 F2 fuse rule (filament reabsorption).
    #[serde(default)]
    pub fuse: LkFuseConfig,
}

fn d_lkr_priority() -> String { "subdivide_first".into() }

/// v4 §7: fuse configuration. The rule itself lives in
/// rules/lk_fuse.toml (embedded) and is APPENDED to the loaded rule set
/// when enabled; generation-guarded (gen ≥ 1) so planted/original
/// edges are non-removable by design (§1.4 — a v5 question, not a
/// knob). Churn is measured, never prevented (§7 anti-churn brake is
/// priority + conflict semantics).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LkFuseConfig {
    #[serde(default)]
    pub enabled: bool,
    /// Fuse-then-resubdivide at the same site within this many steps
    /// counts as one churn event (flag threshold pre-registered in F2
    /// notes before the first fuse run).
    #[serde(default = "d_churn_window")]
    pub churn_window_steps: u64,
    /// v5 §1.3 f_cap grammar: optional Guard `w.energy < f_cap`
    /// (STRICT `<`, pinned by FG5 — a child at exactly f_cap is not
    /// fusable). A fresh child carries exactly q, so with f_cap < q the
    /// guard is an AGING TIMER: t_age ≳ (q − f_cap)/τ steps of drainage
    /// before the node becomes fusable. `None` = unthrottled (the v4
    /// suppression limit). Counters wired from birth per the v4 rule.
    #[serde(default)]
    pub f_cap: Option<f64>,
    /// v6 §4 gate variable: `"none"` (unthrottled), `"f_cap"` (v5 energy
    /// gate — reads `f_cap`), `"countdown"` (v6 lifetime gate — fuse
    /// admits at `countdown == 0`, key 7, decremented by the `lk_tick`
    /// rule; subdivision endows the child `countdown_k`), or `"age"`
    /// (v8 §3.1 PASSIVE clock — fuse admits at
    /// `step − birth_step ≥ lifetime_k`, read from the creation step the
    /// store already records; no tick rule, no property, no back-action).
    /// The dispersed gate (§1.2) in both lifetime spellings. Default
    /// `"none"`: with `f_cap = None` this is the v5 unthrottled fuse,
    /// byte-identical.
    #[serde(default = "d_fuse_gate")]
    pub gate: String,
    /// v6 §4 countdown lifetime `k` endowed on each fresh child when
    /// `gate = "countdown"`. Swept {10,30,100,300}.
    #[serde(default = "d_countdown_k")]
    pub countdown_k: i64,
    /// v8 Appendix A lifetime `k` for `gate = "age"`: the fuse admits a
    /// node once `step − birth_step ≥ lifetime_k`. `None` falls back to
    /// `countdown_k`, so the two gates are swept on ONE number and a
    /// ticked/passive comparison pair cannot silently disagree on k.
    #[serde(default)]
    pub lifetime_k: Option<i64>,
    /// v6 §3 ledger mode: `"vanish"` (v4/v5 — fused energy leaves onto
    /// the ledger, audit `E_total + c·N_sub + Σ E_vanished = E₀`) or
    /// `"transfer"` (fused energy relocates to the lowest-generation
    /// surviving endpoint, audit `E_total + c·N_sub = E₀`). Default
    /// `"vanish"` (byte-identical to v5).
    #[serde(default = "d_ledger_mode")]
    pub ledger_mode: String,
}

fn d_churn_window() -> u64 { 20 }
fn d_fuse_gate() -> String { "none".into() }
fn d_countdown_k() -> i64 { 100 }
fn d_ledger_mode() -> String { "vanish".into() }

impl Default for LkFuseConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            churn_window_steps: d_churn_window(),
            f_cap: None,
            gate: d_fuse_gate(),
            countdown_k: d_countdown_k(),
            lifetime_k: None,
            ledger_mode: d_ledger_mode(),
        }
    }
}

fn d_lkr_theta() -> f64 { 100.0 }
fn d_lkr_q() -> f64 { 50.0 }
fn d_lkr_c() -> f64 { 10.0 }
fn d_lkr_tau() -> f64 { 25.0 }
fn d_lkr_e_total() -> f64 { 1.0e6 }
fn d_lkr_seed_placement() -> String { "center".into() }

impl Default for LkRulesConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            theta: d_lkr_theta(),
            q: d_lkr_q(),
            c: d_lkr_c(),
            tau: d_lkr_tau(),
            e_total: d_lkr_e_total(),
            seed_placement: d_lkr_seed_placement(),
            g_max: None,
            priority: d_lkr_priority(),
            fuse: LkFuseConfig::default(),
        }
    }
}

/// `[relaxation]` — v5 §2 (EXP-L15): start a run from a PERSISTED
/// snapshot instead of a generated initial condition. When `ic_source`
/// is set, the `[initial]` section is ignored for store construction
/// (its `width` is still read by the restoration check to build the
/// clean-lattice fixture), the LK energy field is NOT re-initialized
/// (the snapshot's energies are the physics), and the §6.2 audit
/// baseline becomes the LOADED state's energy sum (plus the fresh E₀
/// for arm (b)) — counters restart at zero with the run.
///
/// Hazard (v5 §9.1): the IC's provenance (source manifest params,
/// snapshot step) is copied into this run's manifest as
/// `ic_provenance`; a run at params that disagree with the source's is
/// a different experiment and must say so in the plan.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RelaxationConfig {
    /// A run directory containing `snapshots/` (or the snapshots
    /// directory itself). The LAST snapshot is the IC.
    #[serde(default)]
    pub ic_source: Option<PathBuf>,
    /// Arm (b): additionally seed a fresh `lk_rules.e_total` at the
    /// `seed_placement` site of the LOADED store (added to the site's
    /// existing energy, and to the audit baseline).
    #[serde(default)]
    pub concurrent_seed: bool,
}

/// `[campaign]` — campaign guardrails (plan v2 §7).
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CampaignConfig {
    #[serde(default)]
    pub abort: CampaignAbortConfig,
}

/// `[campaign.abort]`: k consecutive Exploding-classified windows
/// terminate the run (verdict ExplodingAbort) instead of eating the host.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CampaignAbortConfig {
    #[serde(default = "d_abort_windows")]
    pub exploding_windows: usize,
}

// v3 §1.3: burst exploders (binary_edge) show only 2 Exploding windows;
// k = 3 sails past them. k = 2 is the tested working value.
fn d_abort_windows() -> usize { 2 }

impl Default for CampaignAbortConfig {
    fn default() -> Self {
        Self {
            exploding_windows: d_abort_windows(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct OutputOptions {
    #[serde(default)]
    pub snapshots_enabled: bool,
    #[serde(default)]
    pub append_log_enabled: bool,
    /// Test mode (docs/reproducibility_policy.md): fixed run_id + timestamp
    /// so whole-file byte comparison is valid for test tooling.
    #[serde(default)]
    pub fixed_provenance: bool,
}
