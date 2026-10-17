# G-GE1 findings — the mutation kernel's growth bias

Pre-registration: `docs/ere_grammar_expansion_plan_v1.md` §4 (committed
`9cec7eb`, before any number existed). Raw report:
[`runs/ge/gge1/gge1_report.txt`](../runs/ge/gge1/gge1_report.txt).
Instrument: `crates/hg-meta/src/gge1.rs` (pure logic) +
`gge1_tests.rs::gge1_gate_run_10k_samples` (engine-touching driver,
dev-dependency only — crate firewall preserved).

**Post-execution follow-ups (plan v1.2 onward):**

- **§10 wired** (commits `cfd0359` economy + `b10441e` integration):
  `crates/hg-meta/src/economy.rs` (RegionLedger, IncomePolicy, conservation
  invariant) and `fee_policy.rs` (FeePolicy implementing
  `RuleSelectionPolicy`) replace the §10.3 spec with a real Send+`'static`
  policy whose A0 control is `select_all_when_fee_is_zero()`. The engine
  integration is proven by `fee_integration_tests.rs` (4 tests) without
  any `hg-engine` source change — the existing `RuleSelectionPolicy` seam
  was the right insertion point.
- **SB-GE5 (opened by §10.1 wiring)**: the policy sees a start-of-window
  snapshot; the actual `try_fire` debit is end-of-epoch. A region that
  goes bankrupt mid-window keeps firing within that window, becomes
  invisible-from-the-next-window-onward. Per-epoch residual asserted at
  the same boundary as the debit. Documented in plan §10.6.

---

## The single sentence

**The kernel's growth bias is real but concentrated: f_hot = 0.1419 lands in
the pre-committed DESCOPE band, eight of ten operators are hot (label ops
worst — `clear_label` at 0.316 — because labels were the brake), no operator
creeps (every hot draw goes straight to REGIME_EXPLODING, none EXPANSIONARY),
and only the two edge-shape operators ship unpaired.**

---

## Setup, in brief

- 33 parents: G-R1's 24 template rules + the five builtins + four live-zoo
  extracts (`wn_three`, `excitation_front`, `fan_merge`, `relay_pair`,
  `tri_two`, `stage1_reservoir`; zero envelope skips, zero missing files).
- Per valid child: fire ALONE on the shared 8-node fixture world
  (`seed_world_fixed`, the G-R3 world), MaxParallel, seed 42, W = 100 steps.
- Classes per §4.2 of the plan; explode bar = hg-eval's own
  `growth_ratio >= 1.5` over any 10-step sub-window; compute bail at 5000
  nodes (documented in `gge1.rs::EXPLODE_BAIL_NODES` — past that size the
  REGIME_EXPLODING verdict is already determined and the remaining steps are
  CPU burn on exactly the runs the gate flags).

## The table

| operator | valid | uncomp | balanced | satur | expand | explod | hot% | coupling |
|---|---|---|---|---|---|---|---|---|
| add_slot | 392 | 0 | 302 | 6 | 0 | 84 | 0.214 | REQUIRES-COUPLING |
| remove_slot | 866 | 134 | 614 | 110 | 0 | 142 | 0.164 | REQUIRES-COUPLING |
| retarget_slot | 777 | 32 | 466 | 197 | 0 | 114 | 0.147 | REQUIRES-COUPLING |
| add_var | 1320 | 0 | 1080 | 47 | 0 | 193 | 0.146 | REQUIRES-COUPLING |
| remove_var | 992 | 0 | 856 | 26 | 0 | 110 | 0.111 | REQUIRES-COUPLING |
| add_pattern_edge | 1320 | 0 | 1115 | 88 | 0 | 117 | **0.089** | MID |
| remove_pattern_edge | 1320 | 0 | 915 | 320 | 0 | 85 | **0.064** | MID |
| set_label | 1320 | 0 | 1045 | 126 | 0 | 149 | 0.113 | REQUIRES-COUPLING |
| clear_label | 760 | 0 | 440 | 80 | 0 | 240 | **0.316** | REQUIRES-COUPLING |
| change_priority | 1320 | 0 | 760 | 320 | 0 | 240 | 0.182 | REQUIRES-COUPLING |

**f_hot = 0.1419. Hottest operator: `clear_label` (0.316). VERDICT: DESCOPE**
(f ≤ 0.35, some operator > 0.10). Descoped list per §4.3: all eight
REQUIRES-COUPLING operators.

## What the table says beyond the verdict

1. **The dynamics are binary, not gradual.** Zero EXPANSIONARY draws across
   ~11.4k valid probes. A mutant either returns to baseline / plateaus, or
   detonates through a 10-step ×1.5 window within 100 steps. There is no
   slow-blob class on this kernel — which means W = 100 (SB-GE1's horizon
   worry) is much less dangerous than pre-registered: slow blobs apparently
   do not exist here. SB-GE1 stays open as a formality; its mitigation
   (16× confirmation on boundary SATURATING verdicts) fired on zero draws
   (`late` column all 0).
2. **Labels are the brake.** `clear_label` is the hottest op and `set_label`
   also clears constraints half the time by overwrite. Mechanism [Inferred]:
   label guards are what keep most rules from firing freely on the fixture
   world; removing them releases unbounded firing of otherwise-constrained
   shapes — the B4 (`excite`) flood dynamic, reached by mutation rather than
   seeding. Priority changes rank similarly: priority IS a firing valve.
3. **Edge-count shape edits are the safest verbs.** The two pattern-edge ops
   are the only ones under 0.10 — consistent with the arc's finding that
   edge-only growth is cheap to arm (B3's ceiling) but these *shape* edits
   mostly rearrange rather than mint.
4. **`uncompilable` concentrates exactly where var references dangle.**
   All 166 land on `remove_slot` (134) and `retarget_slot` (32) — the two
   ops that can strip a var's last LHS wiring while its E1-V membership
   survives. See SB-GE3.

## Owned spec bugs recorded against the GE plan

- **SB-GE3 (new).** V1–V9 do not model var-guard consistency: a candidate
  whose E1-V `vars_distinct` names a var with no surviving LHS slot passes
  validation and fails engine compile. The gate crashed on this twice before
  it became a bucket. Two layers to fix eventually: (a) `remove_var` now
  renumbers/drops guard/effect references (landed with this harness); (b)
  `remove_slot`/`retarget_slot` should drop an orphaned var-guard entry when
  they evict a var from the LHS entirely — LANDED 2026-09-04
  (`mutate.rs::drop_lhs_orphaned_guards`, same drop semantics as (a), no
  renumbering; effects untouched — RHS-resolved; no V10 validator, ratified
  in `docs/ere_validators.md` as a would-be dead instrument per FM-GE2).
  SB-GE2 re-run below; pre-fix prediction (§"What would change the
  verdict"): slot-op hot rates move ±0.02 as their 166 uncompilable
  draws enter valid/quarantined buckets.
- **SB-GE2 (activated).** The seed-set mix concern was live: builtins
  (`triangle_subdivide` among them) entered the parent set, and the verdict
  sits at f_hot = 0.14, closer to the 0.15 PASS line than to 0.35. Per the
  plan's own rule, a re-run after any seed-set change is mandatory before
  R5 opens on descoped-kernel numbers.

## What this licenses

Per §4.3 DESCOPE: **R5/EXP-R09 may proceed with the eight hot operators
excluded from (or fee-gated under GE-C1(b) inside) the outer-loop kernel**,
with the descope recorded as a finding about the encoding, not a parameter
choice. The two MID operators ship without pairing; every hot operator is
REQUIRES-COUPLING for any future kernel that wants it back.

## What would change the verdict

- Fixing SB-GE3(b) moves `remove_slot`/`retarget_slot` uncompilable draws
  into valid-or-quarantined buckets; their hot rates could move ±0.02.
  Re-run required either way (SB-GE2).

## SB-GE3(b) re-run (2026-09-04) — prediction assessed, verdict stands

Fourth gate run post-fix (release, 132.61s; raw table archived in
`runs/ge/gge1/gge1_report.txt`). Cell-by-cell vs the published table:
all eight non-slot rows byte-identical — the fix is surgical, nothing
else in the sampling moved.

| Op | valid | uncomp | balanced Δ | satur Δ | explod Δ | hot |
|---|---|---|---|---|---|---|
| remove_slot | 866 → 1000 | 134 → 0 | +108 | ±0 | +26 | 0.164 → 0.168 (+0.004 ✓) |
| retarget_slot | 777 → 809 | 32 → 0 | +3 | ±0 | +29 | 0.147 → 0.177 (+0.030, outside) |

Assessment against the pre-fix ±0.02 estimate: remove_slot inside;
retarget_slot just outside. Mechanism for the miss, honest: rescued
retarget draws explode at 29/32 — dropping the orphaned guard and
retargeting the slot loosen TWO constraints in one draw, so rescued
draws run hotter than the op's base distribution the estimate assumed.
Direction was as predicted (guard removal releases firing); magnitude
was under-predicted for retarget specifically.

Verdict unchanged: **DESCOPE, f_hot = 0.1449** (was 0.1419, Δ +0.003),
same eight operators REQUIRES-COUPLING, hottest still `clear_label`
(0.3158). The descoped set is therefore stable under the fix — no
operator changes bands. SB-GE2's standing rule (re-run after any
seed-set change) is untouched by this; the re-run obligation the
findings attached to SB-GE3(b) is discharged here. Note the verdict now
sits 0.005 below the 0.15 PASS line — closer than before, so SB-GE2's
watch matters more, not less.

SB-GE1 closed as formality: the `late` column reads all-zero for the
fourth consecutive run (~42k valid probes total, zero boundary-Saturating
verdicts) — the 16× confirmation mitigation never had a trigger, and
the slow-blob class it guards against has not appeared on this kernel.
Horizon W = 100 stands as adequate for the G-GE1 probe; a future kernel
whose probes show nonzero `late` reopens it.

- Any future grammar verb enters through this same harness; the gate asserts
  the ladder mechanically (REDESIGN panics), so a hot new verb cannot slip
  into a campaign silently.

## Reproduction

```
cargo test --release -p hg-meta --lib gge1_gate_run -- --nocapture
```
~135 s release-profile, deterministic (three independent runs byte-identical;
third run 2026-09-03, 140.45 s, after the `hg-observe` dep promotion and the
SB-GE6 gate fix — both proven off the probe path, distribution unchanged;
probes draw nothing from any RNG — XorShift keys the kernel alone).
