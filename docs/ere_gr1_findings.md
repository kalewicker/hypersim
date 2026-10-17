# EXP-R01 / G-R1 findings — mutational neutrality of the RuleSpec kernel

**Run.** `hg-selfmod gate_gr1` (release), 10 000 samples × ≤100-step
trajectories over 23 RCF-distinct seeds, 50/50 RCF-soundness spotchecks
passed. Protocol: `docs/ere_gr1_protocol.md` (pre-committed thresholds).

## Outcome counts

| Class | n | frac | Threshold | Met |
|---|---|---|---|---|
| Neutral (RCF-identical) | 2932 | 0.293 | — | — |
| NearNeutral (distinct, identical 100-step trajectory) | 978 | **0.098** | ≥0.10 | **no (−0.002)** |
| Divergent (distinct, non-catastrophic) | 3169 | 0.317 | — | — |
| Quarantined (≥1 validator) | 1367 | 0.137 | — | — |
| Catastrophic (dead or explosive) | 1554 | 0.155 | ≤0.25 | yes |
| **Valid** (all non-quarantined) | 8633 | **0.863** | ≥0.40 | yes |

**Verdict: FAIL (descope branch).** Valid is far above its floor (0.86 vs
0.40) and catastrophic far below its cap (0.16 vs 0.25); NearNeutral misses
by 0.002 (978 vs 1000 — 22 samples in 10⁴). Binomial SE at p≈0.1 is ≈0.003,
so the threshold sits ~0.7 SE above the measurement: statistically
indistinguishable from the line, but a pre-committed line is a line. No
re-seed was run to shop the gate (that would be p-hacking the verdict; a
confirmatory seed is legitimate future work, labeled as such).

## Reading

1. **The encoding is searchable.** Valid = 0.86 clears redesign territory
   (0.10) by a factor of eight. Single edits usually keep rules working.
2. **The miss is narrow and located.** Per-op rows show where neutrality
   lives and dies:
   * Junk-DNA by design: `drop_lhs_var` → 557 quarantined (V8, dangling
     redge refs), `toggle_delete` → 383 quarantined (V8) — the decoder does
     its job; the mutators produce the junk they were designed to produce.
   * `add_pedge` → 523/656 catastrophic (0.80): bolting an edge onto a
     working rule usually detonates it. The single most dangerous op.
   * No-op-heavy: `drop_rhs_create` 475 neutral, `shrink_arity` 603 neutral
     (mostly inapplicable draws → effectively clonal, counted honestly).
   * Safest constructive ops: `add_lhs_var` 176 near, `add_pnode_constraint`
     111 near — extension without destruction.
3. **Validator engagement (live-run side):** V2 462, V8 756, V6 67, V9 82;
   V1/V3/V4/V5/V7 zero from this kernel (expected — no op touches those
   fields inconsistently; V4/V9 fire only via narrow paths). Not a G-R4
   problem (adversarial engagement already pinned); it characterizes the
   kernel's failure surface.
4. **SB-R1 split:** at-cap parents Valid 0.816 vs off-cap 0.877 — the arity
   ceiling drags validity by ≈0.06, modest and bounded, reported not hidden.
5. **RCF soundness:** 50/50 Neutral spotcheck trajectories identical —
   RCF-equality ⇒ behavioral-equality held everywhere sampled.

## Routing (pre-committed descope branch)

The outer loop (R5) is **descoped to a restricted kernel**, recorded as an
encoding finding: exclude `add_pedge` from the outer-loop kernel (or
rate-limit it hard — 80% catastrophic), keep the remaining 14 ops. The
restriction is about the *encoding's* failure surface, not a parameter
tweak. Additionally the kernel cannot evolve priority at all (no priority
op — §2 of the protocol); the outer loop inherits fixed priorities until a
deliberate kernel extension lands.

## Performance note

Release + explosion early-out (classification-exact, protocol §3): full gate
in minutes. Debug CI runs the 300-sample structural test only (~15 s).
