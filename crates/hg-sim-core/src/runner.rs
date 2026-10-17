//! Headless run pipeline: engine loop, window cadence, CSV/manifest/report
//! output, shared-state publication, event emission.
//!
//! Exit criterion (v4 §13 Phase 10): a live headless run produces one CSV,
//! one report, and a manifest with correct final state, regardless of
//! whether anything was listening on the event channels.

use crate::config::{InitialConditionType, RuleSource, SimConfig};
use hg_app_api::events::{send_lossy, send_status};
use hg_app_api::{
    ClaimRow, CompactEdge, CompactGraphSnapshot, CompactNode, ConstructorRow, SharedState,
    SimEvent, SimStatusEvent, TaskLatticeRow,
};
use hg_causal::CausalObserver;
use hg_core::HypergraphStore;
use hg_engine::Engine;
use hg_eval::{fixture_gate_status, run_truth_table, EvaluatorObserver};
use hg_output::{AppendLogWriter, CsvWriter, RunManifest, SnapshotWriter};
use hg_rules::{compile_rules, CompiledRule};
use hg_schema::{GateName, GateStatus, MetricsRow, Regime, RunSummary};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SimError {
    #[error("rule error: {0}")]
    Rules(#[from] hg_rules::ParseError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("manifest error: {0}")]
    Manifest(#[from] hg_output::ManifestError),
    #[error("config error: {0}")]
    Config(String),
    #[cfg(feature = "hg-meta")]
    #[error("ERE halted after {completed_steps} completed world steps: {reason}")]
    EreHalt { completed_steps: u64, reason: String },
}

pub struct RunArtifacts {
    pub run_dir: PathBuf,
    pub csv_path: PathBuf,
    pub report_path: PathBuf,
    pub manifest_path: PathBuf,
    pub final_step: u64,
    pub canonical_hash: u64,
    pub summary: RunSummary,
    /// Every window's row, in order. Already assembled in memory for the
    /// report; returning it costs a move and is what lets a training
    /// rollout read the evaluator's output without a CSV round-trip.
    pub history: Vec<MetricsRow>,
    /// Node count of the store as built, before step 0.
    pub n_nodes_initial: usize,
    /// Window counts per `Regime`, indexed by discriminant
    /// (docs/fitness_formulas.md §3.1). All-zero when the evaluator is off.
    pub regime_occupancy: [u64; 11],
    /// The constructor registry at run end, in the registry's own sorted
    /// order. Empty when the causal layer is off.
    pub constructors: Vec<ConstructorSummary>,
    /// The task lattice at run end. Empty when the causal layer is off.
    pub tasks: Vec<TaskSummary>,
    /// Longest path in the task lattice at run end (Stage 4).
    pub task_lattice_depth: usize,
}

/// The three constructor facts F3 needs (docs/fitness_formulas.md §4),
/// lifted out of `hg_causal::ConstructorRecord` so the fitness layer does
/// not need the causal crate's full record type.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ConstructorSummary {
    pub wl_hash: u64,
    pub fidelity: f32,
    /// Reported, never rewarded in v1 (docs/fitness_formulas.md §4).
    pub is_replicator: bool,
    pub confirmed_cycles: u32,
}

/// One task-lattice entry, lifted out of `hg_causal::Task` the same way
/// [`ConstructorSummary`] is lifted out of `ConstructorRecord`.
///
/// Stage 4 asks for task-lattice **depth** and **composition rate** — "A→C
/// entries appearing after A→B and B→C are both present". Depth is already
/// on every `MetricsRow`; composition rate is a query over the task set as
/// a whole, and needs the (input, output) pairs rather than a scalar. The
/// WL hashes are enough to compute it, so the causal crate's `Attribute`
/// does not have to cross this boundary.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TaskSummary {
    pub input_wl: u64,
    pub output_wl: u64,
    /// `TaskStatus::Confirmed` — three or more witnesses.
    pub confirmed: bool,
    pub witness_count: u32,
}

/// Per-window probe closure: sees the snapshot the evaluator just saw and
/// returns a feature row (the reservoir readout's state vector).
pub type ProbeFn = Box<dyn FnMut(&hg_core::GraphSnapshot<'_>, u64) -> Vec<f32> + Send>;

/// Everything that differs between a scientific run and a training rollout.
///
/// This exists so there is exactly ONE step loop. The NN plan's "no forked
/// physics" invariant is not a convention to be respected — `run_rollout`
/// and `run_controlled` call the same function with different values of
/// this struct, and the only branches it introduces are around writing to
/// disk. There is no code path a rollout can take that a run cannot.
#[derive(Default)]
struct RunHooks {
    /// True for a training rollout: no run directory, no manifest, no CSV,
    /// no report, no binary artifacts. Metrics come back in memory.
    rollout: bool,
    /// Installed into the engine at the `RuleSelectionPolicy` seam.
    policy: Option<Box<dyn hg_observe::RuleSelectionPolicy>>,
    /// Called at each window boundary; its rows land in `probe_rows`.
    probe: Option<ProbeFn>,
    probe_rows: Vec<Vec<f32>>,
    #[cfg(feature = "hg-meta")]
    ere_capture: Option<Vec<hg_schema::EreMetricsRow>>,
    #[cfg(feature = "hg-meta")]
    ere_step_capture: Option<Vec<hg_schema::ere::EreStepObservation>>,
    /// Diagnostic-only initialization, called exactly once before stepping.
    /// All subsequent environment dynamics must be native engine rules.
    #[cfg(feature = "hg-meta")]
    ere_environment_setup: Option<EreEnvironmentSetup>,
}

#[cfg(feature = "hg-meta")]
pub type EreEnvironmentSetup = Box<
    dyn FnOnce(&mut HypergraphStore, &mut Engine) -> Result<Vec<CompiledRule>, String> + Send,
>;

impl RunHooks {
    /// A rollout writes nothing. Every disk-touching branch in `run_inner`
    /// is gated on this, and only on this.
    fn writes_disk(&self) -> bool {
        !self.rollout
    }
}

pub fn load_rules(config: &SimConfig) -> Result<Vec<CompiledRule>, SimError> {
    #[cfg_attr(not(feature = "alife"), allow(unused_mut))]
    let mut raw = match config.rules.source {
        RuleSource::Builtin => {
            let name = config.rules.builtin_name.as_deref().ok_or_else(|| {
                SimError::Config("rules.source = Builtin requires builtin_name".into())
            })?;
            hg_rules::builtin_rules(name)?
        }
        RuleSource::File => {
            let path = config.rules.file_path.as_ref().ok_or_else(|| {
                SimError::Config("rules.source = File requires file_path".into())
            })?;
            hg_rules::load_rules_from_file(path)?
        }
        RuleSource::Inline => {
            let toml_str = config.rules.inline_toml.as_deref().ok_or_else(|| {
                SimError::Config("rules.source = Inline requires inline_toml".into())
            })?;
            hg_rules::parse_rules_toml(toml_str)?
        }
    };
    // ALife biology is APPENDED to the configured substrate rules (so base
    // rule ids are preserved and the biology gets higher ids). Gated by both
    // the feature and the config flag: with the feature off, or `enabled`
    // false, `raw` is exactly the base and behaviour is bit-identical.
    #[cfg(feature = "alife")]
    if config.alife.enabled {
        let mut biology = hg_rules::parse_rules_toml(hg_alife::alife_base_toml())?;
        apply_alife_overrides(&mut biology, &config.alife);
        raw.append(&mut biology);
    }
    // v4 §7: the fuse rule is APPENDED when enabled (same pattern as the
    // ALife biology) so base rule ids are preserved.
    if config.lk_rules.enabled && config.lk_rules.fuse.enabled {
        let mut fuse = hg_rules::parse_rules_toml(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../rules/lk_fuse.toml"
        )))?;
        // v5 §1.3 f_cap grammar: optional Guard w.energy < f_cap,
        // STRICT `<` (FG5 pins the boundary: a child at exactly f_cap
        // is not fusable). Appended after the declared guards so the
        // generation guard keeps its prop0 counter id.
        if let Some(f_cap) = config.lk_rules.fuse.f_cap {
            for r in fuse.iter_mut() {
                if r.name == "lk_fuse" {
                    r.lhs_node_property_guards.push(hg_rules::NodePropertyGuard {
                        var: "w".into(),
                        property: 1,
                        op: hg_core::PropertyOp::Lt,
                        value: hg_core::PropertyValue::Float(f_cap),
                    });
                }
            }
        }
        raw.append(&mut fuse);
    }
    let mut compiled = compile_rules(&raw)?;
    // v4 §1.2: the priority order is a swept axis. "transport_first"
    // remaps the LK rule priorities at load; fuse stays last in both
    // regimes (F2 anti-churn brake). Only LK-named rules are touched.
    if config.lk_rules.enabled {
        match config.lk_rules.priority.as_str() {
            "subdivide_first" => {}
            "transport_first" => {
                for r in compiled.iter_mut() {
                    match r.name.as_str() {
                        "lk_metered_subdivide" => r.priority = 0,
                        "lk_transport" => r.priority = 1,
                        "lk_fuse" => r.priority = -1,
                        _ => {}
                    }
                }
            }
            other => {
                return Err(SimError::Config(format!(
                    "unknown lk_rules.priority '{other}' \
                     (subdivide_first | transport_first)"
                )))
            }
        }
    }
    Ok(compiled)
}

/// Apply the S6 config overrides to the parsed biology rules in place, mirroring
/// the in-test `rules_with` patchers so that a config file reproduces the exact
/// regime the sweeps discovered. Each override is `Option`: `None` leaves the
/// `alife_base.toml` default untouched, so `enabled = true` with no overrides is
/// bit-identical to the pre-S6 behaviour (and the existing alife tests still hold).
#[cfg(feature = "alife")]
pub fn apply_alife_overrides(biology: &mut Vec<hg_rules::Rule>, alife: &crate::config::AlifeConfig) {
    use hg_core::PropertyValue;
    use hg_rules::EffectOpDecl;
    for r in biology.iter_mut() {
        match r.name.as_str() {
            // influx: the energy add-value on the substrate energy key (S3.5).
            "influx" if alife.influx_rate.is_some() => {
                let v = alife.influx_rate.unwrap();
                for eff in &mut r.rhs_property_effects {
                    if eff.key == hg_alife::keys::ENERGY {
                        if let EffectOpDecl::Add { value } = &mut eff.op {
                            *value = PropertyValue::Float(v);
                        }
                    }
                }
            }
            // bootstrap_seed: the founding carrier's replication_threshold genome.
            "bootstrap_seed" if alife.bootstrap_replication_threshold.is_some() => {
                let v = alife.bootstrap_replication_threshold.unwrap();
                for eff in &mut r.rhs_property_effects {
                    if eff.var == "carrier" && eff.key == hg_alife::keys::REPLICATION_THRESHOLD {
                        if let EffectOpDecl::Set { value } = &mut eff.op {
                            *value = PropertyValue::Float(v);
                        }
                    }
                }
            }
            // child_node_cost: a SYMMETRIC SCALE across four coupled values — the
            // fundability guard, the target payment, the child birth energy (0.8c),
            // and the death recycle (0.8c). replicate carries the first three;
            // die_and_recycle carries the recycle. See AlifeConfig::child_node_cost.
            "replicate" if alife.child_node_cost.is_some() => {
                let c = alife.child_node_cost.unwrap();
                let birth = 0.8 * c;
                for gd in &mut r.lhs_node_property_guards {
                    if gd.var == "target" && gd.property == hg_alife::keys::ENERGY {
                        gd.value = PropertyValue::Float(c);
                    }
                }
                for eff in &mut r.rhs_property_effects {
                    if eff.var == "target" && eff.key == hg_alife::keys::ENERGY {
                        if let EffectOpDecl::Subtract { value } = &mut eff.op {
                            *value = PropertyValue::Float(c);
                        }
                    }
                    if eff.var == "child" && eff.key == hg_alife::keys::ENERGY {
                        if let EffectOpDecl::Set { value } = &mut eff.op {
                            *value = PropertyValue::Float(birth);
                        }
                    }
                }
            }
            "die_and_recycle" if alife.child_node_cost.is_some() => {
                let birth = 0.8 * alife.child_node_cost.unwrap();
                for eff in &mut r.rhs_property_effects {
                    if eff.var == "site" && eff.key == hg_alife::keys::ENERGY {
                        if let EffectOpDecl::Add { value } = &mut eff.op {
                            *value = PropertyValue::Float(birth);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // Genome-scaled metabolism (deviation #4): swap the fixed harvest/metabolise
    // quanta for property-valued amounts read from the carrier's genome, coupling
    // harvest_rate and maintenance_cost to fitness. A separate pass because it edits
    // guards + effects across three rules.
    if alife.genome_scaled_metabolism {
        use hg_core::PropertyOp;
        use hg_rules::NodePropertyCompare;
        let (energy, hrate, mcost) = (
            hg_alife::keys::ENERGY,
            hg_alife::keys::HARVEST_RATE,
            hg_alife::keys::MAINTENANCE_COST,
        );
        for r in biology.iter_mut() {
            match r.name.as_str() {
                // Keep the initial economy identical: metabolise reads maintenance_cost,
                // so bootstrap it to the old fixed quantum (0.5), not 1.0.
                "bootstrap_seed" => {
                    for eff in &mut r.rhs_property_effects {
                        if eff.var == "carrier" && eff.key == mcost {
                            if let EffectOpDecl::Set { value } = &mut eff.op {
                                *value = PropertyValue::Float(0.5);
                            }
                        }
                    }
                }
                // metabolise: subtract carrier.maintenance_cost instead of 0.5.
                "metabolise" => {
                    for eff in &mut r.rhs_property_effects {
                        if eff.var == "carrier" && eff.key == energy {
                            eff.op = EffectOpDecl::SubtractFromProperty {
                                source_var: "carrier".into(),
                                source_key: mcost,
                            };
                        }
                    }
                }
                // harvest: transfer carrier.harvest_rate (site −rate, carrier +rate) and
                // make the site-floor guard scale so the site can't go negative.
                "harvest" => {
                    for eff in &mut r.rhs_property_effects {
                        if eff.var == "site" && eff.key == energy {
                            eff.op = EffectOpDecl::SubtractFromProperty {
                                source_var: "carrier".into(),
                                source_key: hrate,
                            };
                        }
                        if eff.var == "carrier" && eff.key == energy {
                            eff.op = EffectOpDecl::AddFromProperty {
                                source_var: "carrier".into(),
                                source_key: hrate,
                            };
                        }
                    }
                    // Replace the constant site-floor guard (site.energy ≥ 1.0) with a
                    // property compare (site.energy ≥ carrier.harvest_rate).
                    r.lhs_node_property_guards
                        .retain(|g| !(g.var == "site" && g.property == energy));
                    r.lhs_node_property_compares.push(NodePropertyCompare {
                        var_a: "site".into(),
                        key_a: energy,
                        op: PropertyOp::Gte,
                        var_b: "carrier".into(),
                        key_b: hrate,
                        offset: None,
                    });
                }
                _ => {}
            }
        }
    }

    // Parent-funded replication (§1 option 2): move the child cost from the target
    // site to the parent carrier. The parent now pays from its own energy, so a
    // carrier must hold energy ≥ child cost to survive reproducing — installing the
    // survival trade-off that gives `replication_threshold` an interior optimum. Total
    // energy per birth (ledger identity) is unchanged; only the payer moves.
    if alife.parent_funded_replication {
        use hg_rules::{EffectOpDecl, PropertyEffectDecl};
        let energy = hg_alife::keys::ENERGY;
        for r in biology.iter_mut() {
            if r.name != "replicate" {
                continue;
            }
            // The child cost = the current target energy-subtract amount (may already
            // have been patched by child_node_cost).
            let cost = r
                .rhs_property_effects
                .iter()
                .find_map(|eff| match (&eff.var[..], eff.key, &eff.op) {
                    ("target", k, EffectOpDecl::Subtract { value: PropertyValue::Float(v) })
                        if k == energy => Some(*v),
                    _ => None,
                })
                .expect("replicate has a target energy payment to move to the parent");
            // Remove the target's payment + its energy-affordability guard (the target
            // no longer pays; it only supplies the unoccupied dispersal location).
            r.rhs_property_effects.retain(|eff| {
                !(eff.var == "target"
                    && eff.key == energy
                    && matches!(eff.op, EffectOpDecl::Subtract { .. }))
            });
            r.lhs_node_property_guards
                .retain(|g| !(g.var == "target" && g.property == energy));
            // The parent funds the child from its own energy.
            r.rhs_property_effects.push(PropertyEffectDecl {
                var: "parent".into(),
                key: energy,
                op: EffectOpDecl::Subtract { value: PropertyValue::Float(cost) },
            });
            // Energy-proportional reproductive tax (5d): AFTER the fixed cost, the
            // parent loses fraction `tau` of its remaining energy to entropy
            // (parent.energy ×= 1 − tau). Absolute cost scales with wealth, so it
            // bites the energy-rich — pushed last so it reads the post-cost energy.
            if let Some(tau) = alife.repro_energy_tax {
                r.rhs_property_effects.push(PropertyEffectDecl {
                    var: "parent".into(),
                    key: energy,
                    op: EffectOpDecl::ScaleFrom {
                        source_var: "parent".into(),
                        source_key: energy,
                        factor: 1.0 - tau,
                    },
                });
            }
        }
    }

    // Reproductive senescence (route B / 5e): each birth ages the parent, and a carrier
    // dies once its `lifetime` reaches the cap — a death route tied to reproduction RATE,
    // not energy. Adds a `die_of_age` rule (cloned from die_and_recycle with a lifetime
    // guard). Aims at the deep drift sink 5d proved immune to energy costs.
    if let Some(cap) = alife.reproductive_lifetime_cap {
        use hg_core::{PropertyOp, PropertyValue};
        use hg_rules::{EffectOpDecl, NodePropertyGuard, PropertyEffectDecl};
        let (energy, lifetime) = (hg_alife::keys::ENERGY, hg_alife::keys::LIFETIME);
        let zero = || PropertyValue::Float(0.0);
        let mut die_of_age = None;
        for r in biology.iter_mut() {
            match r.name.as_str() {
                // Founder starts un-aged.
                "bootstrap_seed" => r.rhs_property_effects.push(PropertyEffectDecl {
                    var: "carrier".into(),
                    key: lifetime,
                    op: EffectOpDecl::Set { value: zero() },
                }),
                // Each birth: the child is born un-aged; the parent ages by one.
                "replicate" => {
                    r.rhs_property_effects.push(PropertyEffectDecl {
                        var: "child".into(),
                        key: lifetime,
                        op: EffectOpDecl::Set { value: zero() },
                    });
                    r.rhs_property_effects.push(PropertyEffectDecl {
                        var: "parent".into(),
                        key: lifetime,
                        op: EffectOpDecl::Add { value: PropertyValue::Float(1.0) },
                    });
                }
                // Clone the energy-death rule; swap its guard for a lifetime cap.
                "die_and_recycle" => {
                    let mut clone = r.clone();
                    clone.name = "die_of_age".into();
                    clone
                        .lhs_node_property_guards
                        .retain(|g| !(g.var == "carrier" && g.property == energy));
                    clone.lhs_node_property_guards.push(NodePropertyGuard {
                        var: "carrier".into(),
                        property: lifetime,
                        op: PropertyOp::Gte,
                        value: PropertyValue::Float(cap),
                    });
                    die_of_age = Some(clone);
                }
                _ => {}
            }
        }
        if let Some(rule) = die_of_age {
            biology.push(rule);
        }
    }
}

/// Load a relaxation IC (v5 §2): the LAST persisted snapshot under
/// `source` (a run directory containing `snapshots/`, or the snapshots
/// directory itself), plus a provenance string assembled from the source
/// run's manifest (v5 §9.1: the IC's params travel with the new run).
fn load_relaxation_ic(source: &std::path::Path) -> Result<(HypergraphStore, String), SimError> {
    let (snaps_dir, run_dir) = if source.join("snapshots").is_dir() {
        (source.join("snapshots"), source.to_path_buf())
    } else {
        (
            source.to_path_buf(),
            source.parent().map(|p| p.to_path_buf()).unwrap_or_default(),
        )
    };
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&snaps_dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "bin"))
        .collect();
    paths.sort();
    let last = paths.last().ok_or_else(|| {
        SimError::Config(format!(
            "relaxation.ic_source {} contains no snapshots",
            source.display()
        ))
    })?;
    let (step, payload) = hg_output::SnapshotReader::read_snapshot(last)?;
    let store: HypergraphStore =
        bincode::deserialize(&payload).map_err(std::io::Error::other)?;
    let src_manifest = RunManifest::load(&run_dir).ok();
    let provenance = match src_manifest {
        Some(m) => format!(
            "{} @ step {step} | source run_id={} seed={} priority={} config_hash={}",
            source.display(),
            m.run_id,
            m.seed,
            m.priority.as_deref().unwrap_or("-"),
            m.config_hash,
        ),
        None => format!("{} @ step {step} | source manifest missing", source.display()),
    };
    Ok((store, provenance))
}

pub fn build_initial_store(config: &SimConfig) -> HypergraphStore {
    build_initial_store_with_tracked(config).0
}

/// `build_initial_store`, also returning the planted shortcut EdgeIds
/// for TriLattice* initial conditions (the TrackedEdgeObserver's lineage
/// seed, LK1R; empty for every other IC).
pub fn build_initial_store_with_tracked(
    config: &SimConfig,
) -> (HypergraphStore, Vec<hg_core::EdgeId>) {
    let mut store = HypergraphStore::new();
    let mut planted: Vec<hg_core::EdgeId> = Vec::new();
    let ic = &config.initial;
    match ic.condition_type {
        InitialConditionType::PureCrystal => {
            hg_core::generate_crystal(&mut store, ic.width, ic.height, ic.arity, ic.periodic, config.seed);
        }
        InitialConditionType::CrystalWithDefects => {
            hg_core::generate_crystal_with_defects(
                &mut store, ic.width, ic.height, ic.arity, ic.periodic, ic.defect_density, config.seed,
            );
        }
        InitialConditionType::WhiteNoise => {
            hg_core::generate_white_noise(&mut store, ic.node_count, ic.edge_count, ic.max_arity, ic.label_range, config.seed);
        }
        InitialConditionType::SparseSoup => {
            hg_core::generate_sparse_soup(&mut store, ic.node_count, ic.avg_degree, ic.max_arity, ic.label_range, config.seed);
        }
        InitialConditionType::MinimalSeed => {
            hg_core::generate_minimal_seed(&mut store, ic.arity, 0, config.seed);
        }
        InitialConditionType::GliderRing => {
            hg_core::generate_glider_ring(&mut store, ic.node_count);
        }
        InitialConditionType::TriLattice | InitialConditionType::TriLatticeExcited => {
            // LK fixture generator: triangulated torus + planted
            // shortcuts at density defect_density (0.0 = clean lattice).
            let (sw, shortcuts) = hg_eval::fixtures::locality::small_world_with_shortcuts(
                ic.width,
                ic.defect_density,
                config.seed,
            );
            store = sw;
            planted = shortcuts;
            if ic.condition_type == InitialConditionType::TriLatticeExcited {
                // One excitation mark at the deterministic first node.
                let first = store.nodes().next().map(|(id, _)| id);
                if let Some(data) = first.and_then(|id| store.get_node_mut(id)) {
                    data.label = hg_core::Label(1);
                }
            }
        }
        InitialConditionType::Empty => {}
    }
    // ALife substrate energy: ensure the energy key exists (and set its initial
    // value) on every non-carrier node, so influx/harvest can act on the
    // generated substrate. Feature- and flag-gated.
    #[cfg(feature = "alife")]
    if config.alife.enabled {
        let e = hg_core::PropertyValue::Float(config.alife.initial_substrate_energy);
        // `occupied` (key 22) is the NAC surrogate for birth-dispersal: every
        // substrate node starts UNoccupied so `target.occupied == false` can
        // match. bootstrap_seed flips the founding site true. See genome.rs.
        let unoccupied = hg_core::PropertyValue::Bool(false);
        let ids: Vec<hg_core::NodeId> = store.nodes().map(|(id, _)| id).collect();
        for id in ids {
            if let Some(d) = store.get_node_mut(id) {
                if d.label != hg_alife::CARRIER_LABEL {
                    d.set_property(hg_alife::keys::ENERGY, e);
                    d.set_property(hg_alife::keys::OCCUPIED, unoccupied);
                }
            }
        }
    }
    // ALife (scope §6.3): plant exactly one carrier_marker on the generated
    // substrate so bootstrap_seed can sprout the first carrier. Feature- and
    // flag-gated, so a no-feature / flag-off build is untouched.
    #[cfg(feature = "alife")]
    if config.alife.seed_carrier_marker {
        seed_one_carrier_marker(&mut store, config.seed);
    }
    (store, planted)
}

/// Mark exactly one node with `carrier_marker = true`, chosen deterministically
/// from `seed` via a local ChaCha stream (never thread_rng; independent of the
/// engine's RNG, matching how the generators seed themselves). No-op on an
/// empty store.
#[cfg(feature = "alife")]
fn seed_one_carrier_marker(store: &mut HypergraphStore, seed: u64) {
    use rand::{Rng, SeedableRng};
    let ids: Vec<hg_core::NodeId> = store.nodes().map(|(id, _)| id).collect();
    if ids.is_empty() {
        return;
    }
    // XOR a distinct constant so the pick is not correlated with the
    // generator's own use of `seed`.
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed ^ 0x00CA_111E);
    let pick = ids[rng.gen_range(0..ids.len())];
    if let Some(d) = store.get_node_mut(pick) {
        d.set_property(
            hg_alife::keys::CARRIER_MARKER,
            hg_core::PropertyValue::Bool(true),
        );
    }
}

/// Forwarding wrapper that lets the engine own an observer while the runner keeps
/// a shared handle to read its accumulated state after the run (mirrors the
/// `CausalEventObserver` shared-log pattern). Used to pull ecology/lineage metrics
/// for `ecology.csv`. Feature-gated so the no-feature build never sees it.
#[cfg(feature = "alife")]
struct AlifeObserverHandle {
    inner: std::sync::Arc<std::sync::Mutex<hg_alife::AlifeObserver>>,
}

#[cfg(feature = "alife")]
impl hg_observe::Observer for AlifeObserverHandle {
    fn on_step_end(
        &mut self,
        snap: &hg_core::GraphSnapshot<'_>,
        stats: &hg_observe::StepStats,
        events: &[hg_observe::RewriteEvent],
    ) {
        self.inner.lock().expect("alife observer lock").on_step_end(snap, stats, events);
    }
}

/// Run to completion on the calling thread. `shared`/`events` are optional:
/// headless runs pass None and everything still lands on disk.
pub fn run_headless(
    config: &SimConfig,
    shared: Option<&SharedState>,
    event_tx: Option<&crossbeam_channel::Sender<SimEvent>>,
    status_tx: Option<&crossbeam_channel::Sender<SimStatusEvent>>,
) -> Result<RunArtifacts, SimError> {
    run_controlled(config, shared, event_tx, status_tx, None)
}

/// ERE training and probes reuse run_inner, including its evaluator cadence,
/// ceilings and failure handling. Only disk writes are disabled.
#[cfg(feature = "hg-meta")]
pub fn run_ere_rollout(config: &SimConfig) -> Result<RunArtifacts, SimError> {
    let mut hooks = RunHooks { rollout: true, ..RunHooks::default() };
    run_inner(config, None, None, None, None, &mut hooks)
}

/// Same run loop, retaining measured windows and the trailing partial window
/// on an ERE ceiling halt. Does not resume, retry or apply a partial match pool.
#[cfg(feature = "hg-meta")]
pub fn run_ere_diagnostic(config: &SimConfig) -> (Result<RunArtifacts, SimError>, Vec<hg_schema::EreMetricsRow>) {
    let mut hooks = RunHooks { rollout: true, ere_capture: Some(Vec::new()), ..RunHooks::default() };
    let result = run_inner(config, None, None, None, None, &mut hooks);
    (result, hooks.ere_capture.take().unwrap())
}

/// Same shared loop, with opt-in read-only completed-step observations. Errors
/// retain measured windows/steps without retries or filling missing observations.
#[cfg(feature = "hg-meta")]
pub fn run_ere_instrumented_diagnostic(config: &SimConfig) -> (Result<RunArtifacts, SimError>,
    Vec<hg_schema::EreMetricsRow>, Vec<hg_schema::ere::EreStepObservation>) {
    if config.ere.is_none() {return (Err(SimError::Config("step diagnostic requires explicit ERE".into())),Vec::new(),Vec::new());}
    let mut hooks = RunHooks {rollout:true,ere_capture:Some(Vec::new()),ere_step_capture:Some(Vec::new()),..RunHooks::default()};
    let result=run_inner(config,None,None,None,None,&mut hooks);
    (result,hooks.ere_capture.take().unwrap(),hooks.ere_step_capture.take().unwrap())
}

/// Opt-in experiment seam, not a training/config vocabulary admission. Setup
/// may seed a fixture and register read-only observers at step zero only. Its
/// returned exogenous rules are a separately accounted static-editor suffix;
/// they use the same matcher, conflicts, budgets and apply path as other rules.
/// There is no per-step callback and no RNG or step advance during setup.
#[cfg(feature = "hg-meta")]
pub fn run_ere_environment_diagnostic(
    config: &SimConfig,
    setup: EreEnvironmentSetup,
) -> (
    Result<RunArtifacts, SimError>, Vec<hg_schema::EreMetricsRow>,
    Vec<hg_schema::ere::EreStepObservation>,
) {
    let admitted = config.ere.as_ref().is_some_and(|e| {
        matches!(e.arm, crate::ere::EreArm::A2 | crate::ere::EreArm::A3)
            && e.vocabulary.as_ref().is_some_and(|v| v.protocol == hg_meta::COMPLETE_PROTOCOL)
            && e.meta_rules_toml.is_none()
            && e.epoch == hg_rules::EpochPolicy::Fixed(1.try_into().unwrap())
    });
    if !admitted {
        return (Err(SimError::Config("environment diagnostic requires complete witness A2/A3, epoch one and no static editor".into())), Vec::new(), Vec::new());
    }
    let mut hooks = RunHooks {
        rollout: true, ere_capture: Some(Vec::new()), ere_step_capture: Some(Vec::new()),
        ere_environment_setup: Some(setup), ..RunHooks::default()
    };
    let result = run_inner(config, None, None, None, None, &mut hooks);
    (result, hooks.ere_capture.take().unwrap(), hooks.ere_step_capture.take().unwrap())
}

/// `run_headless` plus cooperative control: stop and pause requests are
/// honoured at step boundaries with proper manifest transitions
/// (Running -> Paused -> Running, or -> Interrupted on stop). This is what
/// keeps manifest.json truthful when a GUI stops or closes a live run.
pub fn run_controlled(
    config: &SimConfig,
    shared: Option<&SharedState>,
    event_tx: Option<&crossbeam_channel::Sender<SimEvent>>,
    status_tx: Option<&crossbeam_channel::Sender<SimStatusEvent>>,
    control: Option<&hg_app_api::RunControl>,
) -> Result<RunArtifacts, SimError> {
    let mut hooks = RunHooks::default();
    run_inner(config, shared, event_tx, status_tx, control, &mut hooks)
}

/// The one step loop. `run_controlled` and `run_rollout` both land here;
/// see [`RunHooks`] for what may differ between them.
fn run_inner(
    config: &SimConfig,
    shared: Option<&SharedState>,
    event_tx: Option<&crossbeam_channel::Sender<SimEvent>>,
    status_tx: Option<&crossbeam_channel::Sender<SimStatusEvent>>,
    control: Option<&hg_app_api::RunControl>,
    hooks: &mut RunHooks,
) -> Result<RunArtifacts, SimError> {
    // An explicit run_id always wins (it is already deterministic);
    // fixed_provenance only replaces the timestamped fallback.
    let run_id = if config.output.fixed_provenance && config.run_id.is_empty() {
        format!("{}_{}_TEST", config.experiment_name, config.seed)
    } else {
        config.effective_run_id()
    };
    let created = if config.output.fixed_provenance {
        "1970-01-01T00:00:00Z".to_string()
    } else {
        chrono::Utc::now().to_rfc3339()
    };
    let run_dir = config.out_dir.join(&run_id);
    if hooks.writes_disk() {
        std::fs::create_dir_all(&run_dir)?;
    }

    // v5 §2: relaxation IC loaded FIRST so its provenance is in the
    // manifest from creation (hazard §9.1).
    let relaxation_ic = config
        .relaxation
        .ic_source
        .as_ref()
        .map(|src| load_relaxation_ic(src))
        .transpose()?;
    let relaxation_active = relaxation_ic.is_some();

    // Span-based context (v4 §12): every log line below is attributable
    // to this run even with sim/layout/GUI threads interleaved.
    let run_span = tracing::info_span!("run", run_id = %run_id, seed = config.seed);
    let _run_guard = run_span.enter();
    tracing::info!(max_steps = config.engine.max_steps, threads = config.engine.thread_count, "run starting");

    // Manifest: durable status record, transitions committed to disk.
    let mut manifest = RunManifest::default();
    manifest.run_id = run_id.clone();
    manifest.experiment_name = config.experiment_name.clone();
    manifest.seed = config.seed;
    manifest.thread_count = config.engine.thread_count;
    manifest.step_mode = format!("{:?}", config.engine.step_mode);
    manifest.hypersim_version = env!("CARGO_PKG_VERSION").to_string();
    manifest.target = std::env::consts::ARCH.to_string();
    manifest.max_steps = config.engine.max_steps;
    manifest.config_hash = config.config_hash();
    manifest.classifier_calibration = config
        .leakage
        .enabled
        .then(|| hg_observe::locality::CLASSIFIER_CALIBRATION.to_string());
    manifest.priority = config
        .lk_rules
        .enabled
        .then(|| config.lk_rules.priority.clone());
    manifest.children_born_in_dead_zone = config
        .lk_rules
        .enabled
        .then(|| config.lk_rules.q <= 2.0 * config.lk_rules.tau);
    manifest.f_cap = (config.lk_rules.enabled && config.lk_rules.fuse.enabled)
        .then_some(config.lk_rules.fuse.f_cap)
        .flatten();
    manifest.ic_provenance = relaxation_ic.as_ref().map(|(_, p)| p.clone());
    manifest.start_time = created.clone();
    // A rollout has no durable status record because it has no artifacts
    // whose status could be in question. Every manifest transition below is
    // `if let Some(m)`, so the state machine is simply absent rather than
    // faked with a second, in-memory implementation.
    let mut manifest = hooks
        .writes_disk()
        .then(|| manifest.create(&run_dir))
        .transpose()?;

    // Exactly one CSV per run.
    let csv_path = run_dir.join("metrics.csv");
    let mut csv: Option<CsvWriter<MetricsRow>> = hooks
        .writes_disk()
        .then(|| CsvWriter::create(&csv_path, 1, &run_id, config.seed, &created))
        .transpose()?;

    // Leakage track (LK1): one leakage.csv per run, same pattern.
    let mut locality = config.leakage.enabled.then(|| {
        hg_observe::LocalityObserver::new(
            hg_observe::LeakageSettings {
                cadence_steps: config.leakage.cadence_steps,
                detour_sample_edges: config.leakage.detour_sample_edges,
                detour_r_max: config.leakage.detour_r_max,
                ball_sources: config.leakage.ball_sources,
            },
            config.seed,
        )
    });
    // v4 §1.1: guards.csv writer, created lazily on the first window
    // with any guard-carrying rule (empty for guard-free rule sets).
    let mut guards_csv: Option<CsvWriter<hg_schema::GuardRow>> = None;
    let mut leakage_csv: Option<CsvWriter<hg_schema::LeakageRow>> = (config.leakage.enabled
        && hooks.writes_disk())
    .then(|| CsvWriter::create(&run_dir.join("leakage.csv"), 1, &run_id, config.seed, &created))
    .transpose()?;

    // Optional binary outputs (docs/schema_versions.md): per-window store
    // snapshots (HGSS) and an append-log of MetricsRow records (HGAL).
    let mut snapshots = if config.output.snapshots_enabled && hooks.writes_disk() {
        Some(SnapshotWriter::new(&run_dir.join("snapshots"))?)
    } else {
        None
    };
    let mut append_log = if config.output.append_log_enabled && hooks.writes_disk() {
        Some(AppendLogWriter::create(&run_dir.join("metrics.hgal"))?)
    } else {
        None
    };

    // Engine + science.
    let rules = load_rules(config)?;
    // The unambiguous which-rules-actually-ran check: name and count as
    // loaded, not as configured.
    tracing::info!(
        rule_count = rules.len(),
        source = ?config.rules.source,
        name = config
            .rules
            .builtin_name
            .as_deref()
            .or(config.rules.file_path.as_ref().and_then(|p| p.to_str()))
            .unwrap_or("(inline)"),
        "rules loaded"
    );
    let mut engine = Engine::new(
        rules,
        config.engine.step_mode,
        config.seed,
        config.engine.max_pairings_per_step,
    );
    // The RuleSelectionPolicy seam (v4 §6.1). `None` leaves
    // `Engine::policy` unset, so the policy block in the step loop is not
    // entered at all — which is what makes "policy machinery installed but
    // inert" byte-identical to a build that never had it.
    if let Some(policy) = hooks.policy.take() {
        engine.set_policy(policy);
    }
    // Event-level causal export (LK2): the observer lives in the engine's
    // chain (only it sees per-step RewriteEvents); the shared handle lets
    // the runner dump causal_events.jsonl at run end.
    let event_log = config.leakage.cone_events_enabled.then(|| {
        let (obs, log) = hg_causal::CausalEventObserver::new();
        engine.add_observer(Box::new(obs));
        log
    });
    let (mut store, planted_shortcuts) = match relaxation_ic {
        // Relaxation: the snapshot IS the initial state; no planted
        // lineage seed (tracking starts at plant time, not resume time).
        Some((s, _)) => (s, Vec::new()),
        None => build_initial_store_with_tracked(config),
    };

    // LK-R scaffolding (plan v2 §6): initialize the energy field (key 1)
    // to 0.0 everywhere — transport guards and `add` effects need the key
    // to EXIST — and place the seed budget E₀. On the (vertex-transitive)
    // TriLattice torus every placement is equivalent; "center" = the
    // deterministic mid-index node, "corner" = the first.
    let mut lkr_seed_sites: Vec<hg_core::NodeId> = Vec::new();
    if config.lk_rules.enabled {
        let zero = hg_core::PropertyValue::Float(0.0);
        let ids: Vec<hg_core::NodeId> = store.nodes().map(|(id, _)| id).collect();
        // Fresh ICs only: a relaxation snapshot's energies/generations
        // ARE the physics — zeroing them would erase the state under
        // measurement (v5 §2).
        if !relaxation_active {
            for &id in &ids {
                if let Some(d) = store.get_node_mut(id) {
                    d.set_property(1, zero);
                    // v3 §1.4: generation (key 6) = 0.0 on every IC node when
                    // the cap is in play — the capped guard silently
                    // not-matches on a missing key (hazard 1).
                    if config.lk_rules.g_max.is_some() {
                        d.set_property(6, zero);
                    }
                }
            }
        }
        // v3 §4 (EXP-L10) placements construct the encounter EXP-L2
        // never had. Mouth endpoints of the planted shortcuts, sorted
        // for determinism.
        let mouths: Vec<hg_core::NodeId> = {
            let mut m: Vec<hg_core::NodeId> = planted_shortcuts
                .iter()
                .filter_map(|e| store.get_edge(*e))
                .flat_map(|e| e.nodes.iter().copied())
                .collect();
            m.sort_unstable();
            m.dedup();
            m
        };
        let seed_id = match config.lk_rules.seed_placement.as_str() {
            "corner" => ids.first().copied(),
            "center" => ids.get(ids.len() / 2).copied(),
            // Seed ON a mouth of the first planted shortcut (the
            // deterministic minimum): transport reaches the handle at
            // step 1 by construction.
            "at_tracked_mouth" => {
                if mouths.is_empty() {
                    return Err(SimError::Config(
                        "seed_placement = at_tracked_mouth requires planted shortcuts \
                         (initial.defect_density > 0 with tracked ICs)"
                            .into(),
                    ));
                }
                mouths.first().copied()
            }
            // Control: the node FARTHEST (BFS) from every mouth — same
            // position class as any node on the vertex-transitive torus
            // (no boundary; the v2 note's "distance-to-boundary" is
            // vacuous here, documented adaptation), guaranteed no
            // adjacent handle, and it gives the ambient arrival timing.
            "matched_offset" => {
                if mouths.is_empty() {
                    return Err(SimError::Config(
                        "seed_placement = matched_offset requires planted shortcuts".into(),
                    ));
                }
                let mut dist: std::collections::HashMap<hg_core::NodeId, u64> =
                    mouths.iter().map(|&m| (m, 0)).collect();
                let mut queue: std::collections::VecDeque<hg_core::NodeId> =
                    mouths.iter().copied().collect();
                while let Some(v) = queue.pop_front() {
                    let d = dist[&v];
                    for e in store.edges_incident_to(v).to_vec() {
                        if let Some(edge) = store.get_edge(e) {
                            for nb in edge.nodes.clone() {
                                if let std::collections::hash_map::Entry::Vacant(en) =
                                    dist.entry(nb)
                                {
                                    en.insert(d + 1);
                                    queue.push_back(nb);
                                }
                            }
                        }
                    }
                }
                ids.iter()
                    .copied()
                    .max_by_key(|id| (dist.get(id).copied().unwrap_or(0), std::cmp::Reverse(*id)))
            }
            other => {
                return Err(SimError::Config(format!(
                    "unknown lk_rules.seed_placement '{other}' \
                     (center | corner | at_tracked_mouth | matched_offset)"
                )))
            }
        };
        // Relaxation arm (a) places nothing; arm (b) (concurrent_seed)
        // ADDS a fresh E₀ at the placement site of the LOADED store
        // (v5 §2 — old throats decay while new growth runs). Fresh runs:
        // existing is the just-zeroed 0.0, so `existing + e_total` is
        // bit-identical to the old overwrite.
        if !relaxation_active || config.relaxation.concurrent_seed {
            if let Some(seed_id) = seed_id {
                if let Some(d) = store.get_node_mut(seed_id) {
                    let existing = match d.get_property(1) {
                        Some(hg_core::PropertyValue::Float(v)) => v,
                        _ => 0.0,
                    };
                    d.set_property(
                        1,
                        hg_core::PropertyValue::Float(existing + config.lk_rules.e_total),
                    );
                }
                lkr_seed_sites.push(seed_id);
            }
        }
    }
    // v5 §2: the audit baseline. Fresh runs: E₀ from config. Relaxation
    // runs: the LOADED state's energy sum (incl. any concurrent seed
    // just placed) — the invariant E + c·N_sub + Σ E_fused = E₀' holds
    // exactly with counters restarting at zero.
    let audit_e0 = if config.lk_rules.enabled && relaxation_active {
        store
            .nodes()
            .filter_map(|(_, d)| match d.get_property(1) {
                Some(hg_core::PropertyValue::Float(v)) => Some(v),
                _ => None,
            })
            .sum()
    } else {
        config.lk_rules.e_total
    };
    // N_subdivisions counter for the §6.2 audit invariant.
    let lkr_sub_count = config.lk_rules.enabled.then(|| {
        let count = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
        engine.add_observer(Box::new(hg_observe::RuleFireCounter {
            rule_name: "lk_metered_subdivide".into(),
            count: std::sync::Arc::clone(&count),
        }));
        count
    });
    // v4 §6.3 fused-energy ledger: Σ vanished energy, part of the audit.
    let lkr_fused = config.lk_rules.enabled.then(|| {
        let sum = std::sync::Arc::new(std::sync::Mutex::new(0.0f64));
        engine.add_observer(Box::new(hg_observe::EnergyVanishedCounter {
            sum: std::sync::Arc::clone(&sum),
        }));
        sum
    });
    // v4 §7 churn counter (measured, never prevented).
    let lkr_churn = (config.lk_rules.enabled && config.lk_rules.fuse.enabled).then(|| {
        let count = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
        engine.add_observer(Box::new(hg_observe::ChurnCounter::new(
            config.lk_rules.fuse.churn_window_steps,
            std::sync::Arc::clone(&count),
        )));
        count
    });

    // Exact lineage tracking of the planted shortcut population (LK1R):
    // per-step events only exist inside the engine's chain, so the
    // observer lives there and the runner keeps a shared handle to drain
    // tracked.csv rows at run end.
    let tracked = (config.leakage.tracked.enabled && !planted_shortcuts.is_empty()).then(|| {
        let obs = hg_observe::TrackedEdgeObserver::new(
            planted_shortcuts.clone(),
            config.leakage.detour_r_max,
            lkr_seed_sites.clone(),
        )
        .with_cadence(config.leakage.tracked.recompute_cadence);
        let shared = std::sync::Arc::new(std::sync::Mutex::new(obs));
        engine.add_observer(Box::new(hg_observe::SharedTracked(std::sync::Arc::clone(
            &shared,
        ))));
        shared
    });
    // Exploding early-abort gate (plan v2 §7), wired at the run level (a
    // run is a campaign cell; there is no campaign runner yet).
    let mut abort_gate = hg_eval::campaign::AbortGate::new(config.campaign.abort.exploding_windows);
    let mut exploding_abort_at: Option<u64> = None;

    let window_len = config.engine.metrics_window_interval.max(1);
    let fixture_gate = if config.evaluator.enabled {
        fixture_gate_status()
    } else {
        GateStatus::Pending
    };
    let mut evaluator = config
        .evaluator
        .enabled
        .then(|| EvaluatorObserver::new(window_len, fixture_gate));
    let mut causal = config.causal.enabled.then(|| {
        let observer = CausalObserver::new();
        // ALife: let the causal layer SEE carriers by classifying the
        // carrier-labelled WL class (scope §3.2). Off ⟹ NullMotifClassifier,
        // i.e. unchanged. Feature- and flag-gated.
        #[cfg(feature = "alife")]
        let observer = if config.alife.enabled {
            observer.with_classifier(Box::new(hg_alife::HypergraphMotifClassifier::new()))
        } else {
            observer
        };
        observer.with_ei_settings(hg_causal::EiSettings {
            interval_steps: config.causal_language.ei_interval_steps,
            min_transitions: config.causal_language.ei_minimum_transitions,
            // Transitions are recorded window-to-window (where WL classes
            // are already computed) — the cadence is part of the
            // measurement's definition and is carried in every EiResult.
            cadence_steps: config.engine.metrics_window_interval,
            try_structural: config.ei_coarse_graining.try_structural,
            try_energetic: config.ei_coarse_graining.try_energetic,
            try_functional: config.ei_coarse_graining.try_functional,
            try_carrier_substrate: config.ei_coarse_graining.try_carrier_substrate,
            try_optimal: config.ei_coarse_graining.try_optimal,
        })
    });

    // ALife ecology/lineage observer (S5). Feature- and flag-gated; the engine
    // owns a forwarding handle and the runner keeps the shared `Arc` to dump
    // `ecology.csv` at run end. Sampled at the metrics window cadence.
    #[cfg(feature = "alife")]
    let alife_observer = config.alife.enabled.then(|| {
        let obs = std::sync::Arc::new(std::sync::Mutex::new(hg_alife::AlifeObserver::new(window_len)));
        engine.add_observer(Box::new(AlifeObserverHandle { inner: obs.clone() }));
        obs
    });
    // Substrate-only metrics (S0.2, scope §9 Q3): a substrate-dedicated
    // WindowComputer + a paired full-graph one, so `substrate.csv` shows the
    // carrier-pollution gap (full graph reads Exploding on a carrier bloom; the
    // substrate subgraph does not). Feature- and flag-gated; a SEPARATE file, so
    // `metrics.csv` is byte-identical without the feature.
    #[cfg(feature = "alife")]
    let mut substrate_metrics = config.alife.enabled.then(|| {
        (
            hg_eval::WindowComputer::new(window_len),
            hg_eval::WindowComputer::new(window_len),
            Vec::<hg_schema::SubstrateRow>::new(),
        )
    });

    // The DEGENERATE_OCCUPANCY denominator (docs/fitness_formulas.md §6.3),
    // read once before step 0.
    #[cfg(feature = "hg-meta")]
    let mut ere_runtime = config.ere.as_ref().map(|cfg| {
        if config.causal.enabled { return Err(SimError::Config("ERE uses independent stationary TPMs; disable the legacy cumulative causal observer".into())); }
        crate::ere::EreRuntime::install(cfg, &mut store, &mut engine).map_err(SimError::Config)
    }).transpose()?;
    #[cfg(feature = "hg-meta")]
    if let Some(setup) = hooks.ere_environment_setup.take() {
        let rng = engine.rng_state_probe();
        let rules = setup(&mut store, &mut engine).map_err(SimError::Config)?;
        if store.step() != 0 || engine.rng_state_probe() != rng {
            return Err(SimError::Config("environment setup must not advance step or engine RNG".into()));
        }
        ere_runtime.as_mut().ok_or_else(|| SimError::Config("environment requires ERE".into()))?
            .install_diagnostic_environment(rules, &mut engine).map_err(SimError::Config)?;
    }
    #[cfg(feature = "hg-meta")]
    let mut ere_records: Vec<hg_schema::EreMetricsRow> = Vec::new();
    let n_nodes_initial = store.node_count();

    if let Some(m) = manifest.as_mut() {
        m.start()?;
    }
    if let Some(s) = shared {
        s.progress.set_max_steps(config.engine.max_steps);
        s.progress.set_run_state_u8(hg_output::RunState::Running.as_u8());
    }

    // Closure-defect samples (LK4): one per leakage emission, requires
    // the evaluator (regime/activity) and the shared WL class map.
    let mut closure_samples: Vec<hg_causal::WindowSample> = Vec::new();

    let mut window_id: u64 = 0;
    let mut window_accepted: usize = 0;
    let mut window_attempted: usize = 0;
    let mut per_step_accepted: Vec<usize> = Vec::with_capacity(window_len as usize);
    let mut history: Vec<MetricsRow> = Vec::new();
    let mut prev_regime = Regime::Unknown;

    let thread_pool = rayon::ThreadPoolBuilder::new()
        .num_threads(config.engine.thread_count.max(1))
        .build()
        .map_err(|e| SimError::Config(e.to_string()))?;

    let mut interrupted = false;
    // Speed throttle bookkeeping (control.speed_steps_per_sec) and the
    // advisory sim_hz published to SimProgress once per window.
    let mut last_step_at = std::time::Instant::now();
    let mut window_started_at = std::time::Instant::now();
    let loop_result = thread_pool.install(|| -> Result<(), SimError> {
        for step in 0..config.engine.max_steps {
            if let Some(ctrl) = control {
                if ctrl.stop_requested() {
                    interrupted = true;
                    break;
                }
                if ctrl.pause_requested() {
                    if let Some(m) = manifest.as_mut() {
                        m.pause()?;
                    }
                    if let Some(s) = shared {
                        s.progress.set_run_state_u8(hg_output::RunState::Paused.as_u8());
                    }
                    while ctrl.pause_requested() && !ctrl.stop_requested() {
                        std::thread::sleep(std::time::Duration::from_millis(10));
                    }
                    if ctrl.stop_requested() {
                        interrupted = true;
                        break;
                    }
                    if let Some(m) = manifest.as_mut() {
                        m.resume()?;
                    }
                    if let Some(s) = shared {
                        s.progress.set_run_state_u8(hg_output::RunState::Running.as_u8());
                    }
                    // Don't count paused wall time against pacing or hz.
                    last_step_at = std::time::Instant::now();
                    window_started_at = std::time::Instant::now();
                }
                // Speed throttle: pace to the requested steps/sec by
                // sleeping off the remainder of this step's time slot.
                // Uncapped (the default) skips this entirely.
                let speed = ctrl.speed_steps_per_sec();
                if speed.is_finite() && speed > 0.0 {
                    let slot = std::time::Duration::from_secs_f32(1.0 / speed);
                    let elapsed = last_step_at.elapsed();
                    if elapsed < slot {
                        std::thread::sleep(slot - elapsed);
                    }
                }
                last_step_at = std::time::Instant::now();
            }

            #[cfg(feature = "hg-meta")]
            let ere_probe_before = hooks.ere_step_capture.as_ref().map(|_|(
                store.step(),format!("{:016x}",crate::ere::substrate_hash(&store)),engine.rng_state_probe()));
            #[cfg(feature = "hg-meta")]
            let mut ere_probe_after_compile = None;
            #[cfg(feature = "hg-meta")]
            if let Some(ere) = ere_runtime.as_mut() {
                ere.before_step(&store, &mut engine).map_err(|reason| {
                    if config.ere.as_ref().is_some_and(|c|c.measure_family_maintenance) {
                        SimError::EreHalt {completed_steps:store.step(),reason}
                    } else {SimError::Config(reason)}
                })?;
                if ere_probe_before.is_some() {ere_probe_after_compile=Some(engine.rng_state_probe());}
            }
            let stats = engine.step(&mut store);
            #[cfg(feature = "hg-meta")]
            if let Some(ere) = ere_runtime.as_mut() {
                ere.note_match_ceiling(engine.match_ceiling_binding());
                ere.after_step(&store, &stats);
                if let (Some((previous_step,world_before,rng_before)),Some(rng_after),Some(records)) =
                    (ere_probe_before,ere_probe_after_compile,hooks.ere_step_capture.as_mut()) {
                    if store.step()>previous_step {records.push(ere.observe_completed_step(&store,&engine,
                        world_before,rng_before,rng_after));}
                }
                if engine.match_ceiling_binding() {
                    if config.ere.as_ref().is_some_and(|c|c.measure_family_maintenance) {
                        return Err(SimError::EreHalt {completed_steps:store.step(),
                            reason:"match ceiling binding; partial candidate pool was not applied".into()});
                    }
                    return Err(SimError::Config("ERE match ceiling is binding; partial candidate pool was not applied".into()));
                }
                if config.ere.as_ref().is_some_and(|c| c.arm == crate::ere::EreArm::A2)
                    && !ere.protection_intact(&store) {
                    return Err(SimError::Config("G-R5 failed: protected anchor loss or meta-edge cleanup".into()));
                }
            }
            window_accepted += stats.accepted_rewrites;
            window_attempted += stats.attempted_matches;
            per_step_accepted.push(stats.accepted_rewrites);

            if let Some(s) = shared {
                s.progress.set_step(step + 1);
            }

            // A window closes every window_len steps — and at the final
            // step, so a trailing partial window is emitted rather than
            // silently dropped (its per-step ratios use the actual length).
            let is_last_step = step + 1 == config.engine.max_steps;
            let window_boundary = (step + 1) % window_len == 0;
            if !window_boundary && !is_last_step {
                continue;
            }

            let snapshot = store.snapshot();
            let _window_span = tracing::debug_span!("window", window = window_id).entered();
            // One WL class map per window, shared by evaluator and causal
            // layer (review finding: each recomputed the full-graph pass).
            let classes = (evaluator.is_some() || causal.is_some())
                .then(|| hg_core::compute_wl_class_map_snapshot(&snapshot, 1));
            let features = evaluator.as_mut().zip(classes.as_ref()).map(|(e, cls)| {
                e.process_window_with_classes(&snapshot, cls)
            });
            if let (Some(c), Some(cls)) = (causal.as_mut(), classes.as_ref()) {
                c.process_window_with_classes(&snapshot, window_id, cls);
            }
            engine.observers_mut().on_window_complete(&snapshot, window_id);

            // v4 §1.1: guard-engagement counters, drained per window into
            // guards.csv (leakage-track instrument). guard_id = family +
            // declaration index; the CSV is created on first nonempty drain
            // so guard-free runs stay byte-identical.
            if config.leakage.enabled && hooks.writes_disk() {
                for (rule_name, gs) in engine.drain_guard_stats() {
                    let w = match guards_csv.as_mut() {
                        Some(w) => w,
                        None => guards_csv.insert(CsvWriter::create(
                            &run_dir.join("guards.csv"),
                            1,
                            &run_id,
                            config.seed,
                            &created,
                        )?),
                    };
                    let families = [
                        ("prop", &gs.property_guards),
                        ("cmp", &gs.property_compares),
                        ("deg", &gs.degree_compares),
                    ];
                    for (prefix, counts) in families {
                        for (i, &(evals, rejects)) in counts.iter().enumerate() {
                            w.write_row(&hg_schema::GuardRow {
                                window: window_id,
                                rule: rule_name.clone(),
                                guard_id: format!("{prefix}{i}"),
                                evals,
                                rejects,
                                claim_type: hg_schema::ClaimType::Measured,
                            })?;
                        }
                    }
                    w.flush()?;
                }
            }

            // Substrate-only metrics this window (S0.2). Same per-window d_eff
            // seed the full-graph path uses, so the two d_eff values are
            // comparable. Threads its OWN WindowComputer ⟹ substrate growth/
            // novelty are never contaminated by carrier births.
            #[cfg(feature = "alife")]
            if let Some((sub_wc, full_wc, rows)) = substrate_metrics.as_mut() {
                let deff_seed = config.seed ^ window_id.wrapping_mul(0x9E37_79B9_7F4A_7C15);
                let (_f, sub_regime) =
                    hg_eval::substrate_regime(sub_wc, &snapshot, hg_alife::CARRIER_LABEL);
                let s = hg_eval::substrate_structural(&snapshot, hg_alife::CARRIER_LABEL, deff_seed);
                let full_regime = hg_eval::classify(&full_wc.observe(&snapshot));
                rows.push(hg_schema::SubstrateRow {
                    window_id,
                    step_end: step + 1,
                    substrate_n_nodes: s.n_nodes,
                    substrate_n_edges: s.n_edges,
                    substrate_mean_degree: s.mean_degree,
                    substrate_d_eff: s.d_eff,
                    substrate_regime: hg_schema::SubstrateRow::regime_str(sub_regime),
                    full_n_nodes: snapshot.node_count(),
                    full_regime: hg_schema::SubstrateRow::regime_str(full_regime),
                });
            }

            let mut row = assemble_row(
                window_id,
                step,
                per_step_accepted.len() as u64,
                &store,
                window_accepted,
                window_attempted,
                &per_step_accepted,
                evaluator.as_ref(),
                features.as_ref(),
                causal.as_ref(),
            );
            // Effective dimension (ball-growth fit): seeded per window so
            // source sampling is reproducible at any thread count. The
            // ball BFS pass is SHARED with the leakage instruments (LK1
            // cost budget: one pass feeds both d_eff and cut scaling).
            // When the evaluator is on, the source count is pinned to
            // DEFF_SOURCES so d_eff values never depend on [leakage].
            let need_geometry = evaluator.is_some() || locality.is_some();
            let mut d_eff = None;
            let mut profiles = None;
            if need_geometry && snapshot.node_count() >= 8 {
                let deff_seed =
                    config.seed ^ window_id.wrapping_mul(0x9E37_79B9_7F4A_7C15);
                let n_sources = if evaluator.is_some() {
                    hg_eval::geometry::DEFF_SOURCES
                } else {
                    config.leakage.ball_sources
                };
                let sources =
                    hg_eval::geometry::sample_sources(&snapshot, deff_seed, n_sources);
                let p = hg_observe::ball_profiles(
                    &snapshot,
                    &sources,
                    hg_eval::geometry::DEFF_R_MAX,
                );
                d_eff = hg_eval::geometry::dimension_from_profiles(
                    &p,
                    snapshot.node_count(),
                );
                profiles = Some(p);
            }
            if evaluator.is_some() {
                row.d_eff = d_eff;
            }
            if let (Some(lk), Some(profiles)) = (locality.as_mut(), profiles.as_ref()) {
                if lk.due(step + 1) {
                    let settings = lk.settings;
                    let mut comp = hg_observe::compute_leakage(
                        &snapshot,
                        step + 1,
                        lk.seed_for(step + 1),
                        &settings,
                        profiles,
                        hg_eval::geometry::DEFF_R_MAX,
                        hg_eval::geometry::DEFF_SATURATION_FRAC,
                        hg_eval::geometry::DEFF_MIN_FIT_POINTS,
                        d_eff,
                    );
                    // LK-R energy audit (v2 §6.2): the exact invariant
                    // E_total(t) + c·N_subdivisions(t) = E₀. All rule
                    // constants are integers in f64, so the residual is
                    // exactly 0.0 or the accounting is broken.
                    if let Some(counter) = lkr_sub_count.as_ref() {
                        let e_now: f64 = snapshot
                            .nodes()
                            .filter_map(|(_, d)| match d.get_property(1) {
                                Some(hg_core::PropertyValue::Float(v)) => Some(v),
                                _ => None,
                            })
                            .sum();
                        let n_sub = counter.load(std::sync::atomic::Ordering::Relaxed);
                        let e_fused = lkr_fused
                            .as_ref()
                            .map(|m| *m.lock().expect("vanished lock"))
                            .unwrap_or(0.0);
                        comp.row.e_total_now = Some(e_now);
                        comp.row.n_subdivisions = Some(n_sub);
                        comp.row.e_fused_total = Some(e_fused);
                        comp.row.churn_events = lkr_churn
                            .as_ref()
                            .map(|c| c.load(std::sync::atomic::Ordering::Relaxed) as u32);
                        // v4 §6.3 extended invariant, exact:
                        // E_total + c·N_sub + Σ E_fused = E₀.
                        comp.row.audit_residual = Some(
                            e_now + config.lk_rules.c * n_sub as f64 + e_fused - audit_e0,
                        );
                    }
                    if let Some(w) = leakage_csv.as_mut() {
                        w.write_row(&comp.row)?;
                        w.flush()?;
                    }
                    // Window-aligned closure features (cadence shared
                    // with the leakage emission; no extra measurement
                    // pass). wl_class_count is the same-window distinct
                    // WL class count from the shared class map.
                    if let (Some(cls), Some(f), Some(ev)) =
                        (classes.as_ref(), features.as_ref(), evaluator.as_ref())
                    {
                        let distinct: std::collections::HashSet<u64> =
                            cls.values().map(|h| h.0).collect();
                        closure_samples.push(hg_causal::WindowSample {
                            macro_v: hg_causal::closure::features::macro_features(
                                d_eff,
                                comp.row.cut_exponent.map(|c| c as f64),
                                comp.row.wormhole_frac as f64,
                                comp.row.wormhole_components,
                                comp.row.area_law_defect.map(|d| d as f64),
                                comp.row.dim_mismatch.map(|d| d as f64),
                                Some(f.activity),
                                Some(distinct.len()),
                                ev.state.current_regime,
                            ),
                            micro_v: hg_causal::closure::features::micro_features(
                                &snapshot,
                                cls,
                                &comp.detour_histogram,
                            ),
                            // v3 §1.2: the clock baseline's features —
                            // window index + this window's phase label.
                            clock_v: hg_causal::closure::clock_features(
                                closure_samples.len(),
                                comp.row.phase,
                            ),
                        });
                    }
                    lk.record(comp);
                }
            }
            // Per-window probe (NN plan §3): the reservoir readout's state
            // vector. Runs on the same snapshot the evaluator just saw, so
            // probe rows and metric rows are window-aligned by
            // construction.
            if let Some(probe) = hooks.probe.as_mut() {
                let row = probe(&snapshot, window_id);
                hooks.probe_rows.push(row);
            }

            #[cfg(feature = "hg-meta")]
            if let Some(ere) = ere_runtime.as_mut() {
                let excluded = MetricsRow { step_start: row.step_start, step_end: row.step_end,
                    window_id: row.window_id, ..MetricsRow::default() };
                let ere_row = ere.close_window(&store, row.clone(), excluded);
                if let Some(rows) = hooks.ere_capture.as_mut() { rows.push(ere_row.clone()); }
                row.ei_micro = None; row.ei_macro = None; row.ce_best = None;
                row.ce_structural = None; row.ce_energetic = None; row.ce_functional = None;
                row.ere = Some(Box::new(ere_row.clone()));
                ere_records.push(ere_row);
            }
            if let Some(w) = csv.as_mut() {
                w.write_row(&row)?;
                w.flush()?;
            }
            if let Some(sw) = snapshots.as_mut() {
                // hg-output takes pre-serialized bytes (v4 §2.2): the
                // orchestrator decides that a snapshot is a bincoded store.
                let payload = bincode::serialize(&store).map_err(std::io::Error::other)?;
                sw.write_snapshot(step + 1, &payload)?;
            }
            if let Some(al) = append_log.as_mut() {
                let payload = bincode::serialize(&row).map_err(std::io::Error::other)?;
                al.append(&payload)?;
                al.flush()?;
            }
            history.push(row.clone());

            // Publish to shared state (RwLock: scientifically load-bearing).
            if let Some(s) = shared {
                s.metrics_history.write().rows.push(row.clone());
                publish_graph(s, &store);
                if let Some(ev) = evaluator.as_ref() {
                    // Incremental publication: append the latest window's
                    // entries. Wholesale clones made cumulative publish
                    // cost O(windows²) and grew the write-lock hold time
                    // every window.
                    let mut es = s.eval_state.write();
                    es.current_regime = ev.state.current_regime;
                    if let Some(r) = ev.state.regime_history.last() {
                        es.regime_history.push(*r);
                    }
                    let have = es.pi_history.len();
                    if have < ev.state.pi_history.len() {
                        es.pi_history.extend_from_slice(&ev.state.pi_history[have..]);
                    }
                    if let Some(i) = ev.state.interestingness_history.last() {
                        es.interestingness_history.push(*i);
                    }
                    es.gate_statuses = ev.state.gate_statuses.clone(); // 8 entries
                    es.science_unlocked = ev.state.science_unlocked;
                }
                if let Some(c) = causal.as_ref() {
                    *s.knowledge_graph.write() = project_kg(c);
                }
            }

            // Lossy events: drop-on-full, never panic.
            if let Some(tx) = event_tx {
                send_lossy(tx, SimEvent::WindowComplete {
                    window: window_id,
                    metrics: Box::new(row.clone()),
                });
                if let Some(ev) = evaluator.as_ref() {
                    let regime = ev.state.current_regime;
                    if regime != prev_regime {
                        send_lossy(tx, SimEvent::RegimeChanged {
                            window: window_id,
                            old: prev_regime,
                            new: regime,
                        });
                        prev_regime = regime;
                    }
                    for (gate, status) in &ev.state.gate_statuses {
                        send_lossy(tx, SimEvent::GateUpdated { gate: *gate, status: *status });
                    }
                }
            }

            // Advisory throughput display (SimProgress is atomics-only).
            if let Some(s) = shared {
                let secs = window_started_at.elapsed().as_secs_f32();
                if secs > 0.0 {
                    s.progress.set_sim_hz(per_step_accepted.len() as f32 / secs);
                }
            }
            window_started_at = std::time::Instant::now();

            // Exploding early-abort (v2 §7): k consecutive Exploding
            // windows terminate the cell AFTER this window's outputs
            // landed, so the verdict carries its window trace.
            if let Some(ev) = evaluator.as_ref() {
                if abort_gate.observe(ev.state.current_regime) {
                    exploding_abort_at = Some(window_id);
                    tracing::warn!(
                        window = window_id,
                        consecutive = abort_gate.consecutive(),
                        "ExplodingAbort: terminating cell"
                    );
                    interrupted = true;
                }
            }

            window_id += 1;
            window_accepted = 0;
            window_attempted = 0;
            per_step_accepted.clear();
            if exploding_abort_at.is_some() {
                break;
            }
        }
        Ok(())
    });
    if let Err(e) = loop_result {
        // The manifest is the authoritative status record: a failed run
        // must not be left looking Running (v4 §9.1).
        if let Some(m) = manifest.as_mut() {
            let _ = m.seal_interrupted(store.step());
        }
        if let Some(s) = shared {
            s.progress.set_run_state_u8(hg_output::RunState::Interrupted.as_u8());
        }
        if let Some(tx) = status_tx {
            send_status(tx, SimStatusEvent::ErrorOccurred { message: e.to_string() });
        }
        #[cfg(feature = "hg-meta")]
        if let Some(ere) = ere_runtime.as_mut() {
            let trailing = ere.close_window(&store, MetricsRow::default(), MetricsRow::default());
            if let Some(rows) = hooks.ere_capture.as_mut() { rows.push(trailing.clone()); }
            ere_records.push(trailing);
            if hooks.writes_disk() {
                let _ = crate::ere::write_records(&run_dir, &ere_records);
            }
        }
        return Err(e);
    }

    let final_step = store.step();
    tracing::info!(final_step, nodes = store.node_count(), edges = store.edge_count(), "run complete");
    let snapshot = store.snapshot();
    engine.observers_mut().on_run_end(&snapshot);

    // Closure defect (LK4): estimated over the run's leakage windows —
    // computed before the summary so its lines and flags land in the
    // report, written to closure.json below with the other artifacts.
    let closure_settings = hg_causal::ClosureSettings {
        min_windows: config.leakage.closure_min_windows,
        ..Default::default()
    };
    let closure_report = (!closure_samples.is_empty())
        .then(|| hg_causal::estimate_closure(&closure_samples, &closure_settings));
    // v3 §5.3 phase-stratified CD: Sparse-only window segments (mask via
    // the clock phase one-hot; index 1 = the Sparse slot). Expected
    // Unknown below the window floor at short era lengths — recorded as
    // such, not suppressed.
    let closure_sparse_report = (!closure_samples.is_empty()).then(|| {
        let mask: Vec<bool> = closure_samples
            .iter()
            .map(|s| s.clock_v.get(1).copied() == Some(1.0))
            .collect();
        hg_causal::closure::estimate_stratum(
            &closure_samples,
            &mask,
            "sparse_only",
            &closure_settings,
        )
    });

    // Completeness gate (docs/gate_formulas.md #8).
    if let Some(ev) = evaluator.as_mut() {
        let complete = history.iter().any(|r| {
            r.pi_normalized.is_some()
                && r.type_histogram_entropy.is_some()
                && r.regime.is_some()
                && r.interestingness_score.is_some()
                && r.mean_degree.is_some()
        });
        ev.gates_mut().finalize_completeness(complete);
    }

    // Durable status first: the manifest is the authoritative record and
    // must reflect the true final state even if report rendering fails.
    if let Some(m) = manifest.as_mut() {
        if interrupted {
            m.seal_interrupted(final_step)?;
        } else {
            m.seal_completed(final_step)?;
        }
    }
    if let Some(s) = shared {
        if let Some(m) = manifest.as_ref() {
            s.progress.set_run_state_u8(m.state().as_u8());
        }
    }

    // Assemble the RunSummary (report content) with the sealed end time.
    let mut summary = build_summary(
        config,
        &run_id,
        &created,
        final_step,
        &history,
        evaluator.as_ref(),
        causal.as_ref(),
    );
    if let Some(m) = manifest.as_ref() {
        summary.provenance.state = m.state().as_str().to_string();
        summary.provenance.end_time = m.end_time.clone();
    }

    // Exploding early-abort verdict (v2 §7): recorded, flagged, and the
    // manifest already sealed Interrupted above.
    if let Some(w) = exploding_abort_at {
        summary.suspicious_flags.push("exploding_abort_triggered".into());
        summary.observations.push(format!(
            "ExplodingAbort: {} consecutive Exploding windows — cell terminated at window {w} \
             (the substrate result stands; the host does not pay for it)",
            config.campaign.abort.exploding_windows
        ));
    }

    // Locality & leakage report content + suspicious flags (LK5).
    if let Some(lk) = locality.as_ref() {
        summary.leakage_rows = lk.rows.clone();
        summary
            .suspicious_flags
            .extend(leakage_flags(&lk.rows, closure_report.as_ref()));
        if let Some(cr) = closure_report.as_ref() {
            summary.leakage_summary.push((
                "closure defect".into(),
                format!(
                    "[Inferred] CD = (raw {:.4}, clock {}) bits/window over {} windows \
                     ({} folds, certainty {}) — interpret via CD_clock; CD_raw alone \
                     may not be quoted (v3 §1.2)",
                    cr.cd_raw_bits,
                    cr.cd_clock_bits
                        .map(|c| format!("{c:.4}"))
                        .unwrap_or_else(|| "n/a".into()),
                    cr.n_windows,
                    cr.cv_folds,
                    cr.certainty
                ),
            ));
        }
        if config.leakage.cone_events_enabled {
            summary.leakage_summary.push((
                "cone analysis".into(),
                "causal_events.jsonl written — run `hypersim-cli cone --run-dir <dir>`".into(),
            ));
        }
        // LK5R additions (plan v2 §8): tracked morphology, energy audit,
        // LK-R gate parameters, and the v2 flags.
        let tracked_rows: Vec<hg_schema::TrackedRow> = tracked
            .as_ref()
            .map(|t| t.lock().expect("tracked lock").rows.clone())
            .unwrap_or_default();
        summary.suspicious_flags.extend(leakage_flags_v2(
            &tracked_rows,
            &lk.rows,
            config.leakage.closure_min_windows,
            config.lk_rules.enabled,
        ));
        if let (Some(first), Some(last)) = (tracked_rows.first(), tracked_rows.last()) {
            summary.leakage_summary.push((
                "tracked morphology".into(),
                format!(
                    "[Estimated] planted lineage {} → {} edges, {} → {} handles, \
                     wormholes {} → {}, throat_length_max {} → {}, far_nucleus {}",
                    first.tracked_edge_count,
                    last.tracked_edge_count,
                    first.tracked_components,
                    last.tracked_components,
                    first.tracked_wormhole_count,
                    last.tracked_wormhole_count,
                    first.throat_length_max,
                    last.throat_length_max,
                    if tracked_rows.iter().any(|r| r.far_nucleus_flag) {
                        "SEEN (V3 candidate — rerun with alternate priority before claiming)"
                    } else {
                        "never seen"
                    },
                ),
            ));
        }
        if config.lk_rules.enabled {
            if let Some(last) = lk.rows.iter().rev().find(|r| r.audit_residual.is_some()) {
                summary.leakage_summary.push((
                    "energy audit".into(),
                    format!(
                        "[Measured] E_total {} + c·N_sub {}·{} = {} vs E₀ {} — residual {} \
                         (must be exactly 0.0)",
                        last.e_total_now.unwrap_or(f64::NAN),
                        config.lk_rules.c,
                        last.n_subdivisions.unwrap_or(0),
                        last.e_total_now.unwrap_or(f64::NAN)
                            + config.lk_rules.c * last.n_subdivisions.unwrap_or(0) as f64,
                        audit_e0,
                        last.audit_residual.unwrap_or(f64::NAN),
                    ),
                ));
            }
            summary.leakage_summary.push((
                "LK-R scaffolding".into(),
                format!(
                    "θ={} q={} c={} τ={} E₀={} seed={} — boundedness and sustainedness are \
                     IMPOSED by this scaffolding (dissipation + diffusion seeding); every \
                     claim is conditional on the sustained bounded structural phase \
                     (plan v2 §6.4)",
                    config.lk_rules.theta,
                    config.lk_rules.q,
                    config.lk_rules.c,
                    config.lk_rules.tau,
                    config.lk_rules.e_total,
                    config.lk_rules.seed_placement,
                ),
            ));
        }
    }

    // v5 §2 / Appendix B restoration_check (relaxation runs only): the
    // terminal state vs the clean-lattice fixture at [initial] width,
    // compared by whole-graph structural WL hash — label-invariant
    // (NodeIds differ by construction), property/energy content excluded
    // (the restored lattice is energy-DEAD; the claim is geometric).
    if relaxation_active {
        let fixture = hg_eval::fixtures::locality::small_world_with_shortcuts(
            config.initial.width,
            0.0,
            config.seed,
        )
        .0;
        let h_run = hg_eval::restoration::structural_wl_hash(&store);
        let h_fix = hg_eval::restoration::structural_wl_hash(&fixture);
        summary.leakage_summary.push((
            "restoration_check".into(),
            format!(
                "[Measured] {} — final {:016x} ({} nodes / {} edges) vs clean {}\u{b2} \
                 lattice {:016x} ({} nodes / {} edges)",
                if h_run == h_fix { "RESTORED" } else { "NOT_RESTORED" },
                h_run,
                store.node_count(),
                store.edge_count(),
                config.initial.width,
                h_fix,
                fixture.node_count(),
                fixture.edge_count(),
            ),
        ));
    }

    // ---- Artifacts. Everything below this line touches disk, and a
    // rollout skips all of it. The SUMMARY above is still assembled
    // either way: it is derived from the same history and the same
    // evaluator/causal state, so a rollout's science is computed
    // identically and simply not serialized.
    let report_path = run_dir.join("report.md");
    if hooks.writes_disk() {
    // 11-section report.
    hg_report::write_report(&summary, &report_path)?;
    // Raw-TPM dump for the independent EI verifier (verify_ei.py):
    // recomputing EI/CE from these counts with zero shared code is the
    // strongest check the calculator gets.
    if let Some(c) = causal.as_ref() {
        let dump = serde_json::to_string_pretty(&c.ei_dump())
            .map_err(std::io::Error::other)?;
        std::fs::write(run_dir.join("ei_dump.json"), dump)?;
    }
    // Event-level causal graph (LK2 cone-analysis input).
    if let Some(log) = event_log.as_ref() {
        let content = log.lock().expect("event log lock").to_jsonl();
        std::fs::write(run_dir.join("causal_events.jsonl"), content)?;
    }
    // Tracked planted-shortcut lineage (LK1R): tracked.csv.
    if let Some(shared) = tracked.as_ref() {
        let rows = shared.lock().expect("tracked lock").rows.clone();
        let mut w: CsvWriter<hg_schema::TrackedRow> =
            CsvWriter::create(&run_dir.join("tracked.csv"), 1, &run_id, config.seed, &created)?;
        for row in &rows {
            w.write_row(row)?;
        }
        w.flush()?;
    }
    if let Some(cr) = closure_report.as_ref() {
        let json = serde_json::to_string_pretty(cr).map_err(std::io::Error::other)?;
        std::fs::write(run_dir.join("closure.json"), json)?;
    }
    if let Some(cr) = closure_sparse_report.as_ref() {
        let json = serde_json::to_string_pretty(cr).map_err(std::io::Error::other)?;
        std::fs::write(run_dir.join("closure_sparse_only.json"), json)?;
    }
    // ALife ecology.csv: a SEPARATE file (like leakage.csv) written only for
    // ALife runs, so `metrics.csv` is byte-identical without the feature/flag.
    #[cfg(feature = "alife")]
    if let Some(obs) = alife_observer.as_ref() {
        let rows = obs.lock().expect("alife observer lock").ecology_rows();
        if !rows.is_empty() {
            let mut w: CsvWriter<hg_alife::EcologyRow> =
                CsvWriter::create(&run_dir.join("ecology.csv"), 1, &run_id, config.seed, &created)?;
            for r in &rows {
                w.write_row(r)?;
            }
            w.flush()?;
        }
    }
    // Substrate-only metrics (S0.2): another SEPARATE file, same discipline.
    #[cfg(feature = "alife")]
    if let Some((_, _, rows)) = substrate_metrics.as_ref() {
        if !rows.is_empty() {
            let mut w: CsvWriter<hg_schema::SubstrateRow> =
                CsvWriter::create(&run_dir.join("substrate.csv"), 1, &run_id, config.seed, &created)?;
            for r in rows {
                w.write_row(r)?;
            }
            w.flush()?;
        }
    }
    #[cfg(feature = "hg-meta")]
    if ere_runtime.is_some() { crate::ere::write_records(&run_dir, &ere_records)?; }
    } // end artifacts block

    // Regime occupancy, indexed by discriminant per
    // docs/fitness_formulas.md §3.1. Taken from the evaluator's own
    // regime_history rather than by re-parsing the CSV's regime strings —
    // a rollout has no CSV, and a string round-trip is a silent
    // reindexing hazard the moment a variant is renamed.
    let mut regime_occupancy = [0u64; 11];
    if let Some(ev) = evaluator.as_ref() {
        for r in &ev.state.regime_history {
            regime_occupancy[regime_index(*r)] += 1;
        }
    }

    let constructors: Vec<ConstructorSummary> = causal
        .as_ref()
        .map(|c| {
            c.constructors
                .all_sorted()
                .into_iter()
                .map(|r| ConstructorSummary {
                    wl_hash: r.pattern.wl_hash.0,
                    fidelity: r.fidelity,
                    is_replicator: r.is_replicator,
                    confirmed_cycles: r.confirmed_cycles,
                })
                .collect()
        })
        .unwrap_or_default();

    let tasks: Vec<TaskSummary> = causal
        .as_ref()
        .map(|c| {
            c.lattice
                .tasks()
                .map(|t| TaskSummary {
                    input_wl: t.input.wl_hash.0,
                    output_wl: t.output.wl_hash.0,
                    confirmed: matches!(t.status, hg_causal::TaskStatus::Confirmed),
                    witness_count: t.witness_count,
                })
                .collect()
        })
        .unwrap_or_default();
    let task_lattice_depth = causal.as_ref().map(|c| c.lattice.depth()).unwrap_or(0);

    if let Some(tx) = status_tx {
        send_status(tx, SimStatusEvent::RunCompleted {
            summary: Box::new(summary.clone()),
        });
        if let Some(ev) = evaluator.as_ref() {
            if ev.gates().science_unlocked() {
                send_status(tx, SimStatusEvent::ScienceUnlocked);
            } else {
                send_status(tx, SimStatusEvent::ScienceLocked {
                    reason: "controller gate reserved (substrate-only build)".into(),
                });
            }
        }
    }

    Ok(RunArtifacts {
        manifest_path: run_dir.join("manifest.json"),
        csv_path,
        report_path,
        run_dir,
        final_step,
        canonical_hash: store.canonical_hash(),
        summary,
        history,
        n_nodes_initial,
        regime_occupancy,
        constructors,
        tasks,
        task_lattice_depth,
    })
}

/// Run the shared loop with a policy installed and nothing written to disk.
///
/// Crate-internal: `run_rollout` (behind the `cognition` feature) is the
/// public door. Kept here rather than in `rollout.rs` so that `RunHooks`
/// stays private to this module — there must be no way to construct a
/// disk-suppressed run except through the two sanctioned entry points.
#[cfg(feature = "cognition")]
pub(crate) fn run_rollout_inner(
    config: &SimConfig,
    policy: Box<dyn hg_observe::RuleSelectionPolicy>,
    probe: Option<ProbeFn>,
) -> Result<(RunArtifacts, Vec<Vec<f32>>), SimError> {
    let mut hooks = RunHooks {
        rollout: true,
        policy: Some(policy),
        probe,
        probe_rows: Vec::new(),
        ..RunHooks::default()
    };
    let artifacts = run_inner(config, None, None, None, None, &mut hooks)?;
    Ok((artifacts, std::mem::take(&mut hooks.probe_rows)))
}

/// `Regime` discriminant as an array index (docs/fitness_formulas.md §3.1).
///
/// Written as an exhaustive match rather than an `as usize` cast so that
/// adding a variant is a compile error here instead of a silent reindexing
/// of every published campaign CSV.
pub fn regime_index(r: Regime) -> usize {
    match r {
        Regime::Frozen => 0,
        Regime::StaticOrdered => 1,
        Regime::GrowthTreadmill => 2,
        Regime::WhiteNoiseSoup => 3,
        Regime::SterileChaos => 4,
        Regime::Exploding => 5,
        Regime::FragmentedIsolation => 6,
        Regime::Monoculture => 7,
        Regime::ActivePersistentStructure => 8,
        Regime::GenerativeCandidate => 9,
        Regime::Unknown => 10,
    }
}

/// Per-run leakage suspicious flags (LK5 anti-gaslight layer). Public so
/// the pathological-input tests exercise exactly the shipping logic.
pub fn leakage_flags(
    rows: &[hg_schema::LeakageRow],
    closure: Option<&hg_causal::ClosureReport>,
) -> Vec<String> {
    let mut flags = Vec::new();
    // cut_fit_r2_below_threshold: a poor cut fit means cut_exponent and
    // area_law_defect are not to be read at face value.
    const CUT_R2_MIN: f32 = 0.8;
    if rows.iter().any(|r| r.cut_fit_r2.is_some_and(|r2| r2 < CUT_R2_MIN)) {
        flags.push(format!("cut_fit_r2_below_threshold (< {CUT_R2_MIN})"));
    }
    // Anti-gaslight self-check: the detour instrument, as configured in
    // THIS binary, must still flag a known-leaky fixture. Scaled
    // instantiation (n = 1024, d = 3, cap 8) keeps it cheap; the
    // full-size calibration lives in the LK0 suite.
    let expander = hg_eval::fixtures::locality::expander(1024, 3, 42);
    let census = hg_observe::detour_census(&expander.snapshot(), 42, 64, 8);
    if census.wormhole_frac <= 0.5 {
        flags.push("expander_fixture_not_flagged_leaky".into());
    }
    if closure.is_some_and(|c| c.pathology_negative_cd) {
        flags.push("closure_defect_negative".into());
    }
    // audit_invariant_drift (v2 §6.2): any nonzero residual is an
    // accounting bug and HALTS INTERPRETATION of the whole run.
    if rows
        .iter()
        .any(|r| r.audit_residual.is_some_and(|res| res != 0.0))
    {
        flags.push("audit_invariant_drift".into());
    }
    flags
}

/// LK5R flags over the tracked lineage + LK-R era (plan v2 §8). Public
/// for the pathological-input tests.
pub fn leakage_flags_v2(
    tracked_rows: &[hg_schema::TrackedRow],
    leakage_rows: &[hg_schema::LeakageRow],
    closure_min_windows: usize,
    lk_rules_active: bool,
) -> Vec<String> {
    let mut flags = Vec::new();
    // wormhole_edges_diverge_from_components_unremarked: throat
    // morphology (edge count pulling away from handle count) present —
    // the flag FORCES the report to remark on it (v2 hazard 2).
    const DIVERGENCE_RATIO: f32 = 3.0;
    if tracked_rows.iter().any(|r| {
        r.tracked_components > 0
            && r.tracked_edge_count as f32 / r.tracked_components as f32 >= DIVERGENCE_RATIO
    }) {
        flags.push("wormhole_edges_diverge_from_components_unremarked".into());
    }
    // quench_before_min_windows: G2 violated in a science run — the
    // structural era (n_subdivisions still growing) ended before the
    // closure-defect window floor was reachable.
    if lk_rules_active {
        let active_windows = leakage_rows
            .windows(2)
            .filter(|w| w[0].n_subdivisions != w[1].n_subdivisions)
            .count()
            + 1;
        let quenched = leakage_rows
            .windows(2)
            .any(|w| w[0].n_subdivisions == w[1].n_subdivisions);
        if quenched && active_windows < closure_min_windows {
            flags.push("quench_before_min_windows".into());
        }
    }
    flags
}

#[allow(clippy::too_many_arguments)]
fn assemble_row(
    window_id: u64,
    step: u64,
    window_len: u64,
    store: &HypergraphStore,
    accepted: usize,
    attempted: usize,
    per_step: &[usize],
    evaluator: Option<&EvaluatorObserver>,
    features: Option<&hg_eval::WindowFeatures>,
    causal: Option<&CausalObserver>,
) -> MetricsRow {
    let mut row = MetricsRow {
        window_id,
        step_start: step + 1 - window_len,
        step_end: step,
        n_nodes: store.node_count(),
        n_edges: store.edge_count(),
        accepted_rewrites: accepted,
        attempted_matches: attempted,
        accepted_per_step: accepted as f64 / window_len as f64,
        ..MetricsRow::default()
    };

    // activity_cv: σ/μ of per-step accepted counts; 0 when μ = 0.
    let n = per_step.len().max(1) as f64;
    let mean = per_step.iter().sum::<usize>() as f64 / n;
    row.activity_cv = Some(if mean > 0.0 {
        let var = per_step.iter().map(|&x| (x as f64 - mean).powi(2)).sum::<f64>() / n;
        var.sqrt() / mean
    } else {
        0.0
    });

    if let Some(f) = features {
        row.mean_degree = f.mean_degree;
        row.activity = Some(f.activity);
        row.entropy_rate = Some(f.entropy_rate);
        row.pi_normalized = f.pi_normalized;
        row.pi_sample_size = Some(f.pi_sample_size);
        row.pi_sample_strategy = Some(f.pi_sample_strategy.to_string());
        row.type_histogram_entropy = Some(f.type_histogram_entropy);
        row.dominant_type_fraction = Some(f.dominant_type_fraction);
        row.type_histogram_jsd = Some(f.type_histogram_jsd);
        row.new_type_bin_rate = Some(f.new_type_bin_rate);
        row.active_persistent_structures = Some(f.active_persistent_structures);
        row.static_persistent_motifs = Some(f.static_persistent_motifs);
        row.static_repeated_motifs = Some(f.static_repeated_motifs);
        row.new_signatures = Some(f.new_signatures.len());
    }
    if let Some(ev) = evaluator {
        row.regime = Some(ev.state.current_regime.as_str().to_string());
        row.interestingness_score = ev.state.interestingness_history.last().copied();
        row.archive_size = Some(ev.archive().size());
        row.archive_capacity = Some(ev.archive().capacity());
        row.archive_evictions = Some(ev.archive().evictions());
        for (gate, status) in ev.state.gate_statuses.iter() {
            let s = Some(status.as_str().to_string());
            match gate {
                GateName::Fixture => row.gate_fixture = s,
                GateName::WhiteNoise => row.gate_white_noise = s,
                GateName::Pi => row.gate_pi = s,
                GateName::Generative => row.gate_generative = s,
                GateName::Treadmill => row.gate_treadmill = s,
                GateName::Mobile => row.gate_mobile = s,
                GateName::Controller => row.gate_controller = s,
                GateName::Completeness => row.gate_completeness = s,
            }
        }
        row.science_unlocked = Some(ev.state.science_unlocked);
    }
    if let Some(c) = causal {
        row.task_lattice_depth = Some(c.lattice.depth());
        row.constructor_count = Some(c.constructors.count());
        row.replicator_count = Some(c.constructors.replicator_count());
        for r in &c.latest_ei {
            match r.strategy {
                hg_causal::CoarseStrategy::Structural => {
                    row.ce_structural = Some(r.ce_coefficient)
                }
                hg_causal::CoarseStrategy::Energetic => {
                    row.ce_energetic = Some(r.ce_coefficient)
                }
                hg_causal::CoarseStrategy::Functional => {
                    row.ce_functional = Some(r.ce_coefficient)
                }
                // Experiment-6 CE is surfaced via EiResultRow (summary) + ei_dump.json,
                // not a metrics.csv column — adding one would break the bit-identical
                // schema for every non-Exp-6 run.
                hg_causal::CoarseStrategy::CarrierSubstrate => {}
                hg_causal::CoarseStrategy::Optimal => {}
            }
        }
        if let Some(best) = c
            .latest_ei
            .iter()
            .max_by(|a, b| a.ce_coefficient.total_cmp(&b.ce_coefficient))
        {
            row.ei_micro = Some(best.ei_micro);
            row.ei_macro = Some(best.ei_macro);
            row.ce_best = Some(best.ce_coefficient);
            row.ce_best_strategy = Some(best.strategy.as_str().to_string());
            // M(t) fills only on the window that actually emitted (blank
            // otherwise, like d_eff) — latest_ei carries between emissions.
            if best.step == window_id {
                row.wl_class_count = Some(best.class_registry_size as usize);
            }
        }
    }
    row
}

/// Public for tests: the GUI-snapshot sampling policy is a regression
/// surface (a bad sample renders every big run as the same blob).
pub fn publish_graph(shared: &SharedState, store: &HypergraphStore) {
    const GUI_NODE_CAP: usize = 5000;
    let mut compact = CompactGraphSnapshot {
        step: store.step(),
        nodes: Vec::new(),
        edges: Vec::new(),
    };

    // Over the cap, sample by BFS so the GUI gets a CONTIGUOUS fragment
    // with its real local structure. The old first-N-in-slotmap-order
    // sample was structure-destroying for splitting rules: an early
    // node's edges are continually replaced by edges to new midpoints,
    // so with the midpoints outside the cap the sampled subgraph lost
    // exactly the connectivity that distinguishes a tree from a mesh
    // (every capped run rendered as the same de-edged blob).
    let over_cap = store.node_count() > GUI_NODE_CAP;
    let mut members: std::collections::HashSet<hg_core::NodeId> =
        std::collections::HashSet::with_capacity(GUI_NODE_CAP.min(store.node_count()));
    if over_cap {
        let mut queue: std::collections::VecDeque<hg_core::NodeId> = Default::default();
        'fill: for (seed, _) in store.nodes() {
            if members.contains(&seed) {
                continue; // already swept up by an earlier component
            }
            members.insert(seed);
            queue.push_back(seed);
            while let Some(nid) = queue.pop_front() {
                if members.len() >= GUI_NODE_CAP {
                    break 'fill;
                }
                for edge_id in store.edges_incident_to(nid) {
                    if let Some(edge) = store.get_edge(*edge_id) {
                        for &nb in &edge.nodes {
                            if members.len() >= GUI_NODE_CAP {
                                break 'fill;
                            }
                            if !members.contains(&nb) && store.get_node(nb).is_some() {
                                members.insert(nb);
                                queue.push_back(nb);
                            }
                        }
                    }
                }
            }
        }
    }
    let included = |id: &hg_core::NodeId| !over_cap || members.contains(id);

    for (id, data) in store.nodes() {
        if !included(&id) {
            continue;
        }
        compact.nodes.push(CompactNode {
            id,
            label: data.label,
            generation: data.generation,
            energy: data.energy,
        });
    }
    // Only edges fully inside the sample: the published value is a valid
    // subgraph, not a set of dangling references the canvas drops.
    for (id, edge) in store.edges() {
        if edge.nodes.iter().all(&included) {
            compact.edges.push(CompactEdge {
                id,
                label: edge.label,
                nodes: edge.nodes.to_vec(),
            });
        }
    }
    *shared.compact_graph.write() = compact;
}

fn project_kg(c: &CausalObserver) -> hg_app_api::KnowledgeGraphView {
    hg_app_api::KnowledgeGraphView {
        tasks: c
            .lattice
            .tasks()
            .map(|t| TaskLatticeRow {
                id: t.id,
                input_hash: t.input.wl_hash.0,
                output_hash: t.output.wl_hash.0,
                status: format!("{:?}", t.status),
                certainty: format!("{:?}", t.certainty),
                witness_count: t.witness_count,
            })
            .collect(),
        constructors: c
            .constructors
            .all_sorted()
            .iter()
            .map(|r| ConstructorRow {
                wl_hash: r.pattern.wl_hash.0,
                fidelity: r.fidelity,
                confirmed_cycles: r.confirmed_cycles,
                is_replicator: r.is_replicator,
                replicator_kind: r.replicator_kind.as_str().to_string(),
                certainty: format!("{:?}", r.certainty),
            })
            .collect(),
        claims: c
            .knowledge
            .claims_sorted()
            .iter()
            .map(|cl| ClaimRow {
                subject: cl.subject.clone(),
                predicate: cl.predicate.as_str().to_string(),
                object: cl.object.clone(),
                certainty: format!("{:?}", cl.certainty),
                step: cl.step,
            })
            .collect(),
        ei_results: c
            .latest_ei
            .iter()
            .map(|r| hg_app_api::EiResultRow {
                strategy: r.strategy.as_str().to_string(),
                ei_micro: r.ei_micro,
                ei_macro: r.ei_macro,
                ce_coefficient: r.ce_coefficient,
                transition_count: r.transition_count,
                cadence_steps: r.cadence_steps,
                class_registry_size: r.class_registry_size,
            })
            .collect(),
    }
}

fn build_summary(
    config: &SimConfig,
    run_id: &str,
    created: &str,
    final_step: u64,
    history: &[MetricsRow],
    evaluator: Option<&EvaluatorObserver>,
    causal: Option<&CausalObserver>,
) -> RunSummary {
    let mut s = RunSummary::new();
    s.provenance.run_id = run_id.to_string();
    s.provenance.experiment_name = config.experiment_name.clone();
    s.provenance.seed = config.seed;
    s.provenance.thread_count = config.engine.thread_count;
    s.provenance.step_mode = format!("{:?}", config.engine.step_mode);
    s.provenance.hypersim_version = env!("CARGO_PKG_VERSION").to_string();
    s.provenance.state = "Completed".into();
    s.provenance.start_time = created.to_string();
    s.provenance.max_steps = config.engine.max_steps;
    s.provenance.final_step = final_step;
    s.provenance.config_hash = config.config_hash();
    s.run_parameters = config.run_parameter_lines();
    s.final_metrics = history.last().cloned();

    if let Some(ev) = evaluator {
        // Regime summary: windows per regime, in first-seen order.
        let mut counts: Vec<(Regime, u64)> = Vec::new();
        for r in &ev.state.regime_history {
            match counts.iter_mut().find(|(cr, _)| cr == r) {
                Some((_, c)) => *c += 1,
                None => counts.push((*r, 1)),
            }
        }
        s.regime_summary = counts;
        s.gate_statuses = ev.state.gate_statuses.clone();
        s.science_unlocked = ev.state.science_unlocked;
        if !s.science_unlocked {
            s.science_locked_reason =
                Some("controller gate reserved (substrate-only build)".into());
        }
        s.fixture_truth_table = run_truth_table()
            .into_iter()
            .map(|o| (o.name.to_string(), o.regime.as_str().to_string(), o.passed))
            .collect();

        let last = ev.state.features_history.last();
        s.evaluator_details = vec![
            (
                "PI Normalization".into(),
                match last {
                    Some(f) => format!(
                        "pi_normalized = {} over {} nodes ({})",
                        f.pi_normalized.map(|v| format!("{v:.4}")).unwrap_or("n/a".into()),
                        f.pi_sample_size,
                        f.pi_sample_strategy
                    ),
                    None => "no windows completed".into(),
                },
            ),
            (
                "Type Histogram Dynamics".into(),
                match last {
                    Some(f) => format!(
                        "entropy {:.4}, dominant fraction {:.4}, JSD {:.4}, {} bins",
                        f.type_histogram_entropy,
                        f.dominant_type_fraction,
                        f.type_histogram_jsd,
                        f.n_type_bins
                    ),
                    None => "no windows completed".into(),
                },
            ),
            (
                "Static vs Active Persistence".into(),
                match last {
                    Some(f) => format!(
                        "{} static, {} active persistent structures",
                        f.static_persistent_motifs, f.active_persistent_structures
                    ),
                    None => "no windows completed".into(),
                },
            ),
            (
                "Novelty Archive Stats".into(),
                format!(
                    "{} signatures / capacity {}, {} evictions",
                    ev.archive().size(),
                    ev.archive().capacity(),
                    ev.archive().evictions()
                ),
            ),
        ];

        // Suspicious flags (report §9).
        let mut flags = Vec::new();
        for w in ev.state.pi_history.windows(2) {
            if (w[1] - w[0]).abs() > hg_eval::thresholds::PI_JUMP_MAX {
                flags.push(format!("pi_normalized jumped {:.3} -> {:.3}", w[0], w[1]));
            }
        }
        if history.iter().any(|r| r.n_nodes == 0) {
            flags.push("graph emptied out mid-run".into());
        }
        s.suspicious_flags = flags;

        // Observations + next actions (report §10, §11).
        if let Some((regime, count)) = s.regime_summary.iter().max_by_key(|(_, c)| *c) {
            s.observations.push(format!(
                "dominant regime: {} ({count} of {} windows)",
                regime.as_str(),
                ev.state.regime_history.len()
            ));
        }
        if s.gate_statuses.iter().any(|(g, st)| *g == GateName::Treadmill && *st == GateStatus::Fail) {
            s.observations.push("run is stuck in a growth treadmill".into());
            s.recommended_actions.push("add a novelty-introducing rule or raise label diversity".into());
        }
        if s.gate_statuses.iter().any(|(g, st)| *g == GateName::Generative && *st == GateStatus::Pass) {
            s.observations.push("sustained generative behavior observed".into());
            s.recommended_actions.push("extend max_steps and re-run to test persistence".into());
        }
    }

    if let Some(c) = causal {
        s.causal_summary = vec![
            ("tasks".into(), c.lattice.task_count().to_string()),
            ("task lattice depth".into(), c.lattice.depth().to_string()),
            ("claims".into(), c.knowledge.claim_count().to_string()),
            ("claim conflicts".into(), c.knowledge.conflicts().len().to_string()),
        ];
        // EI/CE block — a statistical estimate from finite transitions,
        // not a direct readout, hence the [Estimated] tag.
        if c.latest_ei.is_empty() {
            s.causal_summary.push((
                "EI/CE".into(),
                format!(
                    "not yet emitted — {} transitions observed (minimum {}, interval {} steps)",
                    c.tpm.total_transitions(),
                    c.ei_settings.min_transitions,
                    c.ei_settings.interval_steps
                ),
            ));
        } else {
            for r in &c.latest_ei {
                s.causal_summary.push((
                    format!("EI/CE ({})", r.strategy.as_str()),
                    format!(
                        "[Estimated] EI_micro {:.4} bits, EI_macro {:.4} bits, CE {:.4}, \
determinism {:.4}, degeneracy {:.4}, {} transitions (window cadence, {} steps), \
{} micro / {} macro classes",
                        r.ei_micro,
                        r.ei_macro,
                        r.ce_coefficient,
                        r.determinism,
                        r.degeneracy,
                        r.transition_count,
                        r.cadence_steps,
                        r.micro_classes,
                        r.macro_classes
                    ),
                ));
            }
            // M(t): the registry-size curve, with a qualitative verdict.
            if let Some(&(last_step, final_size)) = c.registry_size_history.last() {
                let saturated_at = c
                    .registry_size_history
                    .iter()
                    .find(|&&(_, size)| size == final_size)
                    .map(|&(w, _)| w);
                let growth = match saturated_at {
                    Some(w) if w < last_step => {
                        format!("saturated at {final_size} classes by window {w}")
                    }
                    _ => format!("still growing at run end ({final_size} classes)"),
                };
                s.causal_summary
                    .push(("WL class registry M(t)".into(), growth));
            }
            if c.ei_micro_history.len() >= 2 {
                s.causal_summary.push((
                    "EI_micro trajectory".into(),
                    c.ei_micro_history
                        .iter()
                        .map(|v| format!("{v:.4}"))
                        .collect::<Vec<_>>()
                        .join(" -> "),
                ));
            }
        }
        s.constructor_ecology = c
            .constructors
            .all_sorted()
            .iter()
            .map(|r| {
                (
                    format!("{:016x}", r.pattern.wl_hash.0),
                    format!(
                        "fidelity {:.2}, {} confirmed / {} degraded cycles, replicator: {}, certainty {:?}",
                        r.fidelity,
                        r.confirmed_cycles,
                        r.degraded_cycles,
                        r.replicator_kind.as_str(),
                        r.certainty
                    ),
                )
            })
            .collect();
    }
    s
}
