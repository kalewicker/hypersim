//! Event-level causal graph export (LK2 input): one record per accepted
//! rewrite, with parent links to the most recent earlier event that
//! affected each node the rewrite read. Influence that reaches a node
//! only through DANGLING-EDGE CLEANUP (the engine's documented
//! locality-of-effect exception, v4 §8.1) is tagged so the cone analysis
//! can report that channel separately — otherwise engine bookkeeping
//! masquerades as substrate non-locality.
//!
//! Dependency rule (operationalization, frozen here): event B at a later
//! step depends on event A iff some node B *read* (its bound nodes; for
//! creation rules, its created nodes) was last *affected* by A. A node is
//! affected by A when A bound it, created it, or created an edge touching
//! it (dynamics channel), or when A cleanup-deleted an edge touching it
//! (cleanup channel). Within a step no dependencies exist: candidates are
//! collected from the pre-step snapshot and applied node-disjointly.
//!
//! Exported as `causal_events.jsonl` (one JSON object per line) so the
//! offline cone analysis can stream it.

use ahash::AHashMap;
use hg_core::GraphSnapshot;
use hg_observe::{Observer, RewriteEvent, StepStats};
use serde::{Deserialize, Serialize};
use slotmap::Key;
use std::sync::{Arc, Mutex};

/// One accepted rewrite, in causal-graph form. Node identities are the
/// slotmap key data (`as_ffi`), stable within a run and matchable against
/// snapshot node ids by the offline analysis.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EventRecord {
    pub id: u64,
    /// Step the rewrite executed in (post-increment step numbering: the
    /// step counter value AFTER the engine finished the step).
    pub step: u64,
    pub rule: String,
    /// LHS size = number of bound nodes (dispersion-probe bin key).
    pub lhs_size: u32,
    /// The event's site: bound nodes (or created nodes for creation
    /// rules, which have no bound nodes).
    pub site: Vec<u64>,
    /// (parent event id, reached only via cleanup). When both channels
    /// link the same parent, dynamics wins (cleanup = false).
    pub parents: Vec<(u64, bool)>,
}

#[derive(Default)]
pub struct EventGraphLog {
    pub records: Vec<EventRecord>,
}

impl EventGraphLog {
    /// Serialize as JSONL (one record per line, id order).
    pub fn to_jsonl(&self) -> String {
        let mut out = String::new();
        for r in &self.records {
            out.push_str(&serde_json::to_string(r).expect("EventRecord serializes"));
            out.push('\n');
        }
        out
    }

    pub fn from_jsonl(content: &str) -> Result<Self, serde_json::Error> {
        let mut records = Vec::new();
        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }
            records.push(serde_json::from_str(line)?);
        }
        Ok(Self { records })
    }
}

/// Observer that builds the event graph live. Shares the log through an
/// `Arc<Mutex<_>>` so the runner (which hands ownership of the observer
/// to the engine's chain) can still dump it at run end.
pub struct CausalEventObserver {
    log: Arc<Mutex<EventGraphLog>>,
    /// node key -> (last affecting event id, affected ONLY via cleanup).
    last_affect: AHashMap<u64, (u64, bool)>,
    next_id: u64,
}

impl CausalEventObserver {
    pub fn new() -> (Self, Arc<Mutex<EventGraphLog>>) {
        let log = Arc::new(Mutex::new(EventGraphLog::default()));
        (
            Self {
                log: Arc::clone(&log),
                last_affect: AHashMap::new(),
                next_id: 0,
            },
            Arc::clone(&log),
        )
    }
}

impl Observer for CausalEventObserver {
    fn on_step_end(
        &mut self,
        snapshot: &GraphSnapshot<'_>,
        stats: &StepStats,
        events: &[RewriteEvent],
    ) {
        if events.is_empty() {
            return;
        }
        let step = stats.step;
        let mut new_records = Vec::with_capacity(events.len());
        for event in events {
            let id = self.next_id;
            self.next_id += 1;

            let read_nodes: Vec<u64> = if event.binding.bound_nodes.is_empty() {
                Vec::new() // creation rules read nothing
            } else {
                event
                    .binding
                    .bound_nodes
                    .iter()
                    .map(|n| n.data().as_ffi())
                    .collect()
            };
            let site: Vec<u64> = if event.binding.bound_nodes.is_empty() {
                event.nodes_created.iter().map(|n| n.data().as_ffi()).collect()
            } else {
                read_nodes.clone()
            };

            // Parents: last affector per read node; dynamics beats cleanup
            // when the same parent is reachable both ways.
            let mut parents: AHashMap<u64, bool> = AHashMap::new();
            for n in &read_nodes {
                if let Some(&(pid, cleanup)) = self.last_affect.get(n) {
                    parents
                        .entry(pid)
                        .and_modify(|c| *c = *c && cleanup)
                        .or_insert(cleanup);
                }
            }
            let mut parents: Vec<(u64, bool)> = parents.into_iter().collect();
            parents.sort_unstable();

            new_records.push(EventRecord {
                id,
                step,
                rule: event.rule_name.clone(),
                lhs_size: event.binding.bound_nodes.len() as u32,
                site,
                parents,
            });

            // Record this event's affected nodes. Dynamics channel first…
            for n in event
                .binding
                .bound_nodes
                .iter()
                .chain(event.nodes_created.iter())
            {
                if snapshot.get_node(*n).is_some() {
                    self.last_affect.insert(n.data().as_ffi(), (id, false));
                }
            }
            for e in &event.edges_created {
                if let Some(edge) = snapshot.get_edge(*e) {
                    for n in &edge.nodes {
                        self.last_affect.insert(n.data().as_ffi(), (id, false));
                    }
                }
            }
            // …then cleanup, which must NOT overwrite a dynamics affect
            // from this same event.
            for n in &event.cleanup_neighbors {
                if snapshot.get_node(*n).is_some() {
                    let key = n.data().as_ffi();
                    match self.last_affect.get(&key) {
                        Some(&(eid, false)) if eid == id => {}
                        _ => {
                            self.last_affect.insert(key, (id, true));
                        }
                    }
                }
            }
            // Deleted nodes can never be read again (generational ids are
            // not reused): drop them to bound memory.
            for n in &event.nodes_deleted {
                self.last_affect.remove(&n.data().as_ffi());
            }
        }
        self.log.lock().expect("event log lock").records.extend(new_records);
    }
}
