//! Constructor registry: WL classes that persist while catalyzing tasks.

use crate::{Attribute, CertaintyClass, ConstructorRecord, ReplicatorKind};
use ahash::AHashMap;

#[derive(Default)]
pub struct ConstructorRegistry {
    constructors: AHashMap<u64, ConstructorRecord>,
}

impl ConstructorRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record one observed catalysis cycle for `pattern`.
    ///
    /// `confirmed`: the constructor's focal nodes survived into the post
    /// window with the SAME NodeIds (instance identity — a structurally
    /// identical pattern elsewhere is NOT this constructor surviving).
    /// `population_grew`: the pattern class's own count increased while
    /// persisting — the (trivial, structural) replication signal.
    pub fn record_cycle(
        &mut self,
        pattern: &Attribute,
        task_id: u64,
        confirmed: bool,
        population_grew: bool,
    ) {
        let entry = self
            .constructors
            .entry(pattern.wl_hash.0)
            .or_insert_with(|| ConstructorRecord {
                pattern: pattern.clone(),
                task_catalyzed: task_id,
                confirmed_cycles: 0,
                degraded_cycles: 0,
                is_replicator: false,
                replicator_kind: ReplicatorKind::None,
                certainty: CertaintyClass::Intuition,
                fidelity: 0.0,
            });

        if confirmed {
            entry.confirmed_cycles += 1;
        } else {
            entry.degraded_cycles += 1;
        }
        let total = entry.confirmed_cycles + entry.degraded_cycles;
        entry.fidelity = entry.confirmed_cycles as f32 / total.max(1) as f32;

        // Measured constructors cap at Empirical ("observed and
        // measured"). Law is "proven by rules or mathematics" — no amount
        // of observation earns it (the old fidelity>=0.9 escalation was
        // wrong twice over: category error, and fidelity was fake 1.00).
        if entry.confirmed_cycles >= 3 {
            entry.certainty = CertaintyClass::Empirical;
        }
        // Substrate-only detection can establish at most TRIVIAL
        // replication (the rule mechanically copying structure).
        // NonTrivial requires the ALife layer's bounded-individual
        // tracking and is never assigned here.
        if population_grew && entry.confirmed_cycles >= 2 {
            entry.replicator_kind = ReplicatorKind::Trivial;
            entry.is_replicator = true;
        }
    }

    pub fn get(&self, wl: u64) -> Option<&ConstructorRecord> {
        self.constructors.get(&wl)
    }

    /// Sorted by WL hash for stable display.
    pub fn all_sorted(&self) -> Vec<&ConstructorRecord> {
        let mut v: Vec<&ConstructorRecord> = self.constructors.values().collect();
        v.sort_by_key(|c| c.pattern.wl_hash.0);
        v
    }

    pub fn count(&self) -> usize {
        self.constructors.len()
    }

    pub fn replicator_count(&self) -> usize {
        self.constructors.values().filter(|c| c.is_replicator).count()
    }

    /// The Experiment-6 baseline: bounded self-copying individuals. On a
    /// substrate-only build this is always 0 by construction.
    pub fn non_trivial_replicator_count(&self) -> usize {
        self.constructors
            .values()
            .filter(|c| c.replicator_kind == ReplicatorKind::NonTrivial)
            .count()
    }
}
