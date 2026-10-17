# Two-paid-edit composition V1 — prospective engineering contract

Registered 2026-10-04 before new native execution. This implements the next task
from the small learning trial: verify two-edit composition before another learner
campaign. No optimizer, held-out learning evaluation, or full training is launched.

## Native task and fixed schedule

The ordinary triple has labels `[first,second,12]`. First is 10/20/30; second is
11/21/31, covering all nine pairs. Initialize a path-to-star core with those
labels and a reverse proposal with labels `[99,98,12]`. Both initialized bodies
are fully excluded from novelty by exact and zero-priority structural RCF.
There are two distinct proposal literal nodes and two distinct typed target
markers. Seeded native editors write slot0 at step1 and slot1 at step5; each
spends exactly one Float fuel and advances the native clock. Tick handles every
other step. No host writes occur during execution. Live compilation is Fixed(2).

Per slot choose index0 (no edit), or indices1–3 selecting that slot's three
labels. The full live panel covers every 4×4 action pair for every context/seed.
Six controls use both context-matching actions: frozen compilation, zero fuel,
disabled editor, preseeded complete proposal, one fuel, and reverse edit order.
Reverse order edits slot1 at step1 and slot0 at step5. Other seeded semantics
are unchanged. One fuel permits only the first edit. A preseeded proposal can
fire immediately but receives no novelty credit.

Order: seed, nine lexicographic context pairs, live action pairs in row-major
order, then the six controls in configuration order. Seeds 993748716001/002 are
new and fixed. Total 288 live cells + 108 controls = 396 cells, each 24 steps:
9,504 maximum attempts. Pooled 50k work/50k results, MaxParallel16, one shared
120-second native deadline. Save all outcomes before checking the predicted
causal pattern. No retry, seed replacement, culling, early stopping or warmup.
Native failures invalidate the panel and retain unknown partial work explicitly.

## Attribution, credit and predicted pattern

The first successful edit creates a partial genotype class at step1; it may be
installed at boundary2, but cannot match both ordinary labels. The second
successful edit creates another class at step5; it is installed at boundary6.
Only when both labels match should ordinary proposal firings occur, at steps
7,9,...,23: nine firings, reward9/24. Wrong labels, either edit alone, no edits,
frozen/unpaid/disabled/one-fuel controls, and initialized reuse receive zero.
Reverse order should produce the same final reward, without early credit.

Reward is reconstructed from actual strictly post-birth ordinary events and
the pre-step installed canonical table, not action correctness or cached summary.
Every initial class, including dormant and initialized lookalike bodies, is
excluded structurally. Full horizon and no-cap receipts are mandatory; invalid
work is not assigned a zero score. Partial and complete class lifetimes are
reported separately, including retirement, delayed installation and censoring.
Root survival is not class survival. No population half-life or sustained H-R1
claim is made; engineering counts are not scientific acceptance.

## Shared policy interface (no training)

Build a 12-logit selector shared across slots, not nine pair-specific lookup
rows. Each slot's observed label maps to one of three exposed category cues;
the same four-way softmax row is used for that category in either slot. The
joint action probability factorizes; joint log likelihood and score gradient
are the sum of the two component terms. Explicit draws remain separate from
physics RNG. This is seeded category normalization, not semantic discovery.

Before future training, declare six off-diagonal pairs for updates and three
diagonal pairs reserved from optimizer updates. Every pair is deliberately
exposed in this engineering panel. A later held-out-combination claim would
mean withheld from optimization, not unseen by developers or a new category.
No such claim or training result is produced here.

## Verification and provenance

Add versioned native/learning-interface modules and exports, leaving existing
fixture, learner, reward, compiler and auditor bodies unchanged. Independently
reconstruct both typed literal ownership paths, raw/compiled editor semantics,
paid changes, full graph ledger, epoch tables, budgets, class identities,
lifetimes, credit and shared-policy arithmetic. Rust tests check joint gradients
by finite differences, request refusals and the full native panel. Detached
tamper tests reject premature/partial credit, wrong targets, missing payment,
table drift, caps, action/context changes and shared-policy corruption.

Preserve the 758-entry learning-trial seal and older immutable archives. Use
the previously justified inactive Evolving Minds exclusions, with feature tree
checks before/after and complete active source pins. Verify historical active
drift is limited to the two additive exports. Freeze sources and binary before
native work; require a fresh output directory; seal failures as failures.
