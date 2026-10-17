//! Certainty-tagged knowledge graph of claims.

use crate::{CertaintyClass, Claim, Predicate};
use ahash::AHashMap;

#[derive(Default)]
pub struct KnowledgeGraph {
    claims: AHashMap<u64, Claim>,
    by_subject_predicate: AHashMap<(String, Predicate), Vec<u64>>,
    next_id: u64,
    conflicts: Vec<String>,
}

impl KnowledgeGraph {
    pub fn new() -> Self {
        Self {
            claims: AHashMap::new(),
            by_subject_predicate: AHashMap::new(),
            next_id: 1,
            conflicts: Vec::new(),
        }
    }

    pub fn add_claim(
        &mut self,
        subject: String,
        predicate: Predicate,
        object: String,
        certainty: CertaintyClass,
        step: u64,
    ) -> u64 {
        // Functional predicates (one object per subject) get conflict
        // detection; multi-valued ones (persists/emerges) do not.
        let functional = matches!(predicate, Predicate::TransformsInto);
        if functional {
            if let Some(ids) = self
                .by_subject_predicate
                .get(&(subject.clone(), predicate))
            {
                for id in ids {
                    if let Some(existing) = self.claims.get(id) {
                        if existing.object != object {
                            self.conflicts.push(format!(
                                "step {step}: {subject} {} {object} conflicts with earlier {} {}",
                                predicate.as_str(),
                                predicate.as_str(),
                                existing.object
                            ));
                        }
                    }
                }
            }
        }

        let id = self.next_id;
        self.next_id += 1;
        self.by_subject_predicate
            .entry((subject.clone(), predicate))
            .or_default()
            .push(id);
        self.claims.insert(
            id,
            Claim {
                id,
                subject,
                predicate,
                object,
                certainty,
                step,
            },
        );
        id
    }

    pub fn claim_count(&self) -> usize {
        self.claims.len()
    }

    pub fn claims(&self) -> impl Iterator<Item = &Claim> {
        self.claims.values()
    }

    pub fn conflicts(&self) -> &[String] {
        &self.conflicts
    }

    /// Claims sorted by id (stable display order).
    pub fn claims_sorted(&self) -> Vec<&Claim> {
        let mut v: Vec<&Claim> = self.claims.values().collect();
        v.sort_by_key(|c| c.id);
        v
    }
}
