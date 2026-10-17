# Paid rule-edit choice adapter V1 — prospective engineering contract

Registered before native execution on 2026-10-04. This stage implements the
action/reward boundary recommended by the lifecycle fixture. It does not launch
training, change admission guards, or establish H-R1/sustained novelty.

## Design and fixed budget

The ordinary triple has labels `[context,11,12]`, with context fixed to 10, 20,
or 30. A core path-to-star rule is initialized; the reverse proposal is also
initialized but its first label literal is 99. The initialized family is fully
excluded from novelty using exact and zero-priority structural canonical forms.
Actions are index 0 (no edit), 1 (write 10), 2 (write 20), 3 (write 30).
The adapter selects one seeded editor template before execution. A native rewrite
at step 1 writes the proposal literal, spends one Float fuel, and advances the
native clock; the host does not write the graph during execution. This is
selection among seeded actions, not evolution of editor semantics.

Two new fixed seeds, 993748714001 and 993748714002, cover all four live-paid
actions in all three contexts. For every context/seed, the context-matching edit
also runs frozen-paid, live-unpaid, live-disabled, and preseeded-lure controls.
Order is seed, context, live actions 0–3, then these four controls. All 48 cells
run all 24 steps: 1,152 maximum attempts. No warmup, optimizer updates, culling,
replacement seeds, retry, outcome-selected action, or held-out evaluation.
One shared 60-second native deadline, pooled 50k work/50k results, deterministic
MaxParallel execution with at most 16 accepted rewrites per step. Failures retain
receipts, invalidate the panel, and are not assigned zero reward.

## Interface and reward

The context-indexed softmax uses 12 finite f64 logits, four per context.
Sampling accepts an explicit finite uniform draw in `[0,1)`; physics RNG remains
untouched. Log probabilities and score-function gradients are available for
future use, but there is no optimizer or training command in this stage.

Reward equals the number of ordinary-world rewrite events attributed to a class
that is structurally absent from all initial candidates, born during execution,
and installed in the actual pre-step table, divided by the fixed horizon 24.
Events must be strictly after birth. Credit is derived from the timeline, not
the requested action, expected answer, or cached summary. Paid edit provenance,
complete horizon, valid state, and no cap are required. Invalid/incomplete or
unrecognized receipts fail closed. Reward is an engineering objective only.

Expected pattern: each matching live edit has one birth at step 1, installation
boundary 2, and 11 credited firings at steps 3,5,...,23, reward 11/24.
Nonmatching paid edits still produce one class birth but zero world firings.
No-edit, unpaid, disabled, frozen, and preseeded reuse yield zero reward.
Frozen edits produce a genotype birth but no installed birth. Preseeded rules
fire without novelty credit. Lifetimes remain right-censored; sustained claims
and half-life estimates remain unavailable.

## Verification and limits

Preserve the original lifecycle module, verifier, preregistration, and archive
byte-for-byte. A versioned native fixture keeps its read-only recorder local and
reuses the old canonical/lifetime helpers. An independent Python auditor
reconstructs the typed graph, paid effects, installed rule identities, complete
edge/node ledger, budgets, compilation schedule, reward and policy arithmetic.
Detached tamper controls cover context/action, initial identities, reward,
post-birth attribution, caps, missing work, fuel, editor semantics and tables.
Rust tests exercise stable softmax, draw boundaries, score gradients against
finite differences, invalid requests, and the complete native panel.

All development contexts/actions are exposed and deliberately simple. A pass
validates the adapter, not learning, generalization, population safety, native
physics replay, scientific acceptance, or permission for a training campaign.
Only after this interface is verified should a separate tiny training contract
freeze update budget, evaluation schedule, controls, and comparison criterion.
