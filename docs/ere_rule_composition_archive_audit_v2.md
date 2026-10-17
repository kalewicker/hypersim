# Composition receipts V2 — archive-only target-identity correction

Registered 2026-10-04 before this supplemental audit. No native work, optimizer
updates or physics retry is permitted. Use the already sealed V1 development
archive, root `8dff01746ecc81db6e3c3e1c3036e0de9cdf8af4050857d1150a8cbbe11ede71`,
617 entries, containing all 396 native outcomes and 9,504 completed steps.
Its root lifecycle manifest remains failed because its original frozen auditor
rejected the second-target proof. That failure must not be edited or resealed.

Diagnosis: `E_SLOT_BASE+k` is the endpoint position **within a pattern edge**,
not the global variable number. In the proposal's left star, edges `(v0,v1)` and
`(v0,v2)` share their slot0 variable but have distinct slot1 variables. V1
incorrectly required every left pattern's slot1 to identify v1. Native edits
selected the intended two literals; the independent target proof was wrong.

V2 changes only that proof. Independently establish both sides' two binary
pattern tuples. The unique shared left-slot0 endpoint is v0; the unique
intersection of left-slot1 and right-slot0 endpoints is v1; the remaining
left-slot1 endpoint is v2. Require exactly the typed left tuples `(v0,v1)` and
`(v0,v2)` and right tuples `(v0,v1)` and `(v1,v2)`. Prove literal targets through
the corresponding typed E_HASLABEL relations, check variable/pattern/literal
labels and properties, and require the untouched third literal to remain 12.
No allocation ID, collection iteration order, or presentation-order hint defines
variable identity. This is a stronger proper graph proof, not an identity waiver.

All original contexts, choices, controls, seeds, horizon, typed edit semantics,
ledger reconstruction, class lifetimes, paid provenance, budget checks and
expected reward counts remain unchanged. Native receipt profile and design
remain `E2-rule-composition-v1`; auditor version is separately
`E2-rule-composition-receipt-auditor-v2`. No native source or old auditor body is
changed. Copy the corrected checker and its tests into a fresh sealed supplement,
pin all original inputs against both archive snapshots and live active sources,
verify the original and historical root seals before/after, and run only Python
accounting/tamper tests plus non-mutating feature checks. Never invoke Cargo
tests or the native binary from this supplement.

In addition to the original four Python groups and 24 native/four policy tamper
controls, add a structural regression proving that slot1 names two different
variables across the left patterns while the chosen target remains the intended
middle variable. Report the original failure and corrected accounting separately.
Zero training updates, no generalization claim, no physics replay, and unchanged
scientific/full-training admission boundaries remain mandatory.
