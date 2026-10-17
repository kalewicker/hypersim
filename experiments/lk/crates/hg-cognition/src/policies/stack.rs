//! `StimulusDriver` and `PolicyStack` — reservoir input injection through
//! the existing seam (NN plan §5.5).

use crate::select::partition_candidates;
use hg_core::GraphSnapshot;
use hg_observe::{Binding, RuleSelectionPolicy};

/// Gates paired creation rules against an input tape.
///
/// Given rules `inject_one` / `inject_zero`, both scheduled
/// `every_n_steps n = W`, the driver admits exactly the creation binding
/// matching `tape[step / W]` and drops the other. This is reservoir input
/// injection with **no engine change and no new schedule mode** — the
/// symbol window is the rules' own schedule period, and the seam does the
/// rest.
///
/// **The one sanctioned exception to the creation exemption**
/// (`docs/policy_config.md` §3.3). It is 0 parameters and never learned, so
/// no optimizer can discover creation-starvation as a strategy: the only
/// component allowed to drop a creation binding is one that cannot be
/// trained to.
///
/// Creation bindings from rules *outside* the gated pair pass through
/// untouched — the driver gates its own pair, not creation in general.
pub struct StimulusDriver {
    rule_zero: usize,
    rule_one: usize,
    tape: Vec<bool>,
    window: u64,
}

impl StimulusDriver {
    /// `window` is the symbol period in steps, matching the paired rules'
    /// `every_n_steps`.
    pub fn new(rule_zero: usize, rule_one: usize, tape: Vec<bool>, window: u64) -> Self {
        assert!(window > 0, "StimulusDriver window must be > 0");
        assert!(!tape.is_empty(), "StimulusDriver tape must be non-empty");
        Self {
            rule_zero,
            rule_one,
            tape,
            window,
        }
    }

    /// The symbol in force at `step`. The tape wraps: a rollout longer than
    /// the tape replays it, which keeps memory-capacity estimates
    /// well-defined at any horizon rather than silently running out of
    /// input mid-rollout.
    pub fn symbol_at(&self, step: u64) -> bool {
        let idx = (step / self.window) as usize % self.tape.len();
        self.tape[idx]
    }

    pub fn tape(&self) -> &[bool] {
        &self.tape
    }

    pub fn window(&self) -> u64 {
        self.window
    }
}

impl RuleSelectionPolicy for StimulusDriver {
    fn select(&mut self, snapshot: &GraphSnapshot<'_>, candidates: &[Binding]) -> Vec<usize> {
        let want_one = self.symbol_at(snapshot.step());
        let admit = if want_one { self.rule_one } else { self.rule_zero };
        let drop = if want_one { self.rule_zero } else { self.rule_one };

        (0..candidates.len())
            .filter(|&i| {
                let b = &candidates[i];
                // Only gated-pair creation bindings are ever dropped.
                !(b.is_creation() && b.rule_id == drop && admit != drop)
            })
            .collect()
    }
}

/// Applies `driver` to creation bindings and `learned` to the rest.
///
/// The two policies never see each other's partition: the driver decides
/// which creation bindings survive, the learned policy decides which
/// pattern matches survive, and neither can reach into the other's half.
/// That separation is what makes the creation exemption hold even when a
/// learned policy is active.
pub struct PolicyStack {
    driver: Box<dyn RuleSelectionPolicy>,
    learned: Box<dyn RuleSelectionPolicy>,
}

impl PolicyStack {
    pub fn new(driver: Box<dyn RuleSelectionPolicy>, learned: Box<dyn RuleSelectionPolicy>) -> Self {
        Self { driver, learned }
    }
}

impl RuleSelectionPolicy for PolicyStack {
    fn select(&mut self, snapshot: &GraphSnapshot<'_>, candidates: &[Binding]) -> Vec<usize> {
        let (creation, matches) = partition_candidates(candidates);

        // Each half runs on the FULL candidate slice so that indices stay
        // valid and so the learned policy's pool-level features (candidate
        // count, rule share) describe the real step rather than a filtered
        // view of it. Its verdict is then intersected with the pattern
        // matches, and the driver's with the creations.
        let driver_keep = self.driver.select(snapshot, candidates);
        let learned_keep = self.learned.select(snapshot, candidates);

        let creation_set: rustc_hash::FxHashSet<usize> = creation.into_iter().collect();
        let match_set: rustc_hash::FxHashSet<usize> = matches.into_iter().collect();

        let mut out: Vec<usize> = driver_keep
            .into_iter()
            .filter(|i| creation_set.contains(i))
            .chain(learned_keep.into_iter().filter(|i| match_set.contains(i)))
            .collect();
        out.sort_unstable();
        out.dedup();
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policies::{MlpScorer, RandomSubset};
    use crate::testkit::{candidate_pool, creation, matched, node_ids, ring_store};
    use crate::Parameterized;
    use hg_core::HypergraphStore;

    fn store_at_step(step: u64) -> HypergraphStore {
        let mut s = ring_store(32, 2);
        s.set_step(step);
        s
    }

    #[test]
    fn driver_admits_the_taped_symbol_and_drops_its_partner() {
        // tape = [true, false], window = 10.
        let mut d = StimulusDriver::new(0, 1, vec![true, false], 10);
        let ids_store = ring_store(8, 0);
        let ids = node_ids(&ids_store);

        // step 0 -> symbol true -> admit rule 1 (one), drop rule 0 (zero).
        let store = store_at_step(0);
        let snap = store.snapshot();
        let cands = vec![creation(0, 0), creation(1, 0), matched(2, 0, &ids[0..1])];
        let picked = d.select(&snap, &cands);
        assert_eq!(picked, vec![1, 2], "zero-rule creation dropped at step 0");

        // step 10 -> tape index 1 -> symbol false -> admit rule 0.
        let store = store_at_step(10);
        let snap = store.snapshot();
        let picked = d.select(&snap, &cands);
        assert_eq!(picked, vec![0, 2], "one-rule creation dropped at step 10");
    }

    #[test]
    fn driver_leaves_ungated_creations_alone() {
        let mut d = StimulusDriver::new(0, 1, vec![true], 4);
        let store = store_at_step(0);
        let snap = store.snapshot();
        // Rule 7 is a seeding rule, nothing to do with the stimulus pair.
        let cands = vec![creation(0, 0), creation(1, 0), creation(7, 0)];
        let picked = d.select(&snap, &cands);
        assert!(picked.contains(&2), "unrelated creation rule must survive");
        assert!(!picked.contains(&0));
    }

    #[test]
    fn driver_tape_wraps() {
        let d = StimulusDriver::new(0, 1, vec![true, false, false], 5);
        assert!(d.symbol_at(0));
        assert!(!d.symbol_at(5));
        assert!(!d.symbol_at(10));
        assert!(d.symbol_at(15), "tape should wrap at index 3");
        assert!(d.symbol_at(19), "still inside window 3");
    }

    #[test]
    fn stack_gives_creations_to_the_driver_and_matches_to_the_learned_policy() {
        let store = store_at_step(0);
        let snap = store.snapshot();
        let ids = node_ids(&store);
        let mut cands = vec![creation(0, 0), creation(1, 0)];
        cands.extend((0..30).map(|i| matched(2, 0, &ids[i..i + 1])));

        let driver = StimulusDriver::new(0, 1, vec![true], 4);
        let learned = RandomSubset::new(Some(5), 3);
        let mut stack = PolicyStack::new(Box::new(driver), Box::new(learned));

        let picked = stack.select(&snap, &cands);
        // Driver's verdict on creations: rule 1 admitted, rule 0 dropped.
        assert!(picked.contains(&1));
        assert!(!picked.contains(&0));
        // Learned policy's budget applies to the matches only.
        let n_match = picked.iter().filter(|&&i| !cands[i].is_creation()).count();
        assert_eq!(n_match, 5);
    }

    #[test]
    fn stack_output_is_sorted_and_deduplicated() {
        let store = store_at_step(0);
        let snap = store.snapshot();
        let cands = candidate_pool(&store, 2, 20, 2, 2);
        let mut mlp = MlpScorer::new(2, 8, 4, Some(6), 1);
        mlp.set_params(&vec![0.1; mlp.n_params()]);
        let mut stack = PolicyStack::new(
            Box::new(StimulusDriver::new(0, 1, vec![true], 4)),
            Box::new(mlp),
        );
        let picked = stack.select(&snap, &cands);
        assert!(picked.windows(2).all(|w| w[0] < w[1]));
    }
}
