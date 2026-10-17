# V8 / M4.3 — guard audit: which V-track criteria have a firing test?

Per V8 rule 5 and §6.1: three guards in the program's history passed while the very condition they
guard against was present, so a guard with no test in which it *rejects* is untested. This audits
every acceptance criterion in the V-track family (F/G/H/J/K/L/M) for whether a test exists in which
it fires — i.e. rejects a case it should reject.

## The §6.1 pattern (the three prior silent passes)

1. **satiation < repl_threshold sterility** (S3, pre-V) — bootstrap satiation was 12 < threshold 15,
   making replication impossible; the "working economy" tests all passed anyway because they did not
   check births. Caught only when a births counter was added.
2. **WL depth-1 degeneracy** (V-track → F-track) — the depth-1 partition collapsed carriers to a
   single class (K = 1); the original V-track CE-negative "passed" as a real negative. Caught when
   F1 varied depth.
3. **F0 combinatorial-support bar** (L-track → M2.3) — `C(M,k) > 100` accepted arm D at 3.78×10²
   while its realised permutation null had sd = 0. Caught when arm D's z came back `n/a`.

All three share a shape: **the guard's accept condition was a proxy that can be satisfied while the
real condition it stands for is false.** The audit below asks, for each live criterion, whether a
test drives it into rejection.

## Audit table

| criterion / guard | where | what it rejects | firing test exists? | notes |
|---|---|---|---|---|
| **null-spread guard** (`sd>0 ∧ max−min>0 ∧ ≥10 distinct`) | M2.3, `m2_arm_c_and_null_guard.rs` | a permutation null that cannot vary | **YES** — asserts arm D is *rejected* while the old bar accepts it (`m2` firing test) | the M2.3 replacement itself; the model for the rest |
| **F0 support bar** (`C(M,k)>100`) | F0, used everywhere | a partition too coarse to permute | **YES, retroactively** — M2.3 audit shows it *fails to reject* arm D, which is its firing (negative) test | retained as necessary-not-sufficient; superseded by the null-spread guard |
| **H0.1 stationarity** (TWTF / M(t) / T/M / persistence) | H0.1, `m1_h0_full_criterion.rs` | a moving substrate whose partition ephemeralises | **YES** — growth_treadmill (TWTF 0.66) and triangle_subdivide (0.98) are *rejected*; bounded churn passes | the growing proxies are the built-in firing cases |
| **second stationarity witness** (window-to-window Jaccard) | M1.2, `m1_h0_full_criterion.rs` | a partition that recurs by TWTF but replaces its class set | **YES** — growing proxies score 0.004–0.07 and are *rejected* | added precisely because TWTF has a demonstrated blind spot |
| **M3.1 inert-carrier gate** (`inert≈0 ∧ D≫0`) | M3.1, `m3_metabolic_dynamics.rs` | a measure that scores carrier-ness rather than dynamics | **YES** — candidates 1 & 2 are *rejected* on the C row (lifetime-normalised); the inert control is asserted genuinely inert | the gate's whole point is a zero case |
| **detectability** (`k≥2 ∧ k<M ∧ p<α ∧ support ∧ spread`) | F2 onward | a non-boundary / degenerate partition | **YES** — arm C (`p=0.93`) and arm D (`degenerate`) are *rejected* in `l2`/`m2` | multiple negatives across the arc |
| **K2/M4.2 inclusion** (`transitions ∧ k ∧ sustained ∧ spread`) | K2, M4.2 | a seed whose test is ill-posed | **PARTIAL** — the `sustained` and `k≥2` clauses reject seeds in K2's own run; the **new spread clause fired 0 times** on B′ | spread clause has no *positive* firing case among sustained seeds yet — see gap 1 |
| **M4.4 null-validity** (inert traits ~50/50) | M4.4 | a drift null that is itself biased | **NO dedicated firing case** — the real run's null happened to be clean (43%/53%) | see gap 2 |
| **firewall** (bare energy vectors byte-identical / conservation) | H1, L1.2, M4.1 | a substrate rule that back-reacts or is non-conservative | **YES** — M4.1 asserts conservation and per-step quantum bounds on 5 imposed gradients; would reject a leaky rule | the assertions are the firing test |

## Gaps found (recorded, not silently accepted)

**Gap 1 — the null-spread inclusion clause (M4.2) has no positive firing case among *sustained*
seeds.** On arm B′ all 30 sustained seeds had non-degenerate nulls, so the clause never excluded
anyone. It *does* have a firing test in the abstract (the M2.3 arm-D rejection), but no
established-population seed has yet exercised it. This is acceptable — the clause exists to catch
arm-D-like cases and there are none among live B′ populations — but it means the clause is currently
**inert in this application**, and that is stated rather than presented as a passed check.

**Gap 2 — M4.4's null-validity check has no dedicated firing test.** The check (inert traits must be
~50/50) passed on the real run because the null happened to be clean. There is no constructed case in
which it *rejects*. Lower priority than Gap 1 because the check reads two numbers that are printed in
full, so a future biased null would be visible; but by the rule-5 standard it is an untested guard.

**Not a gap:** the `diffuse` firewall (M4.1) and the H0.1/second-witness stationarity guards (M1) all
have live firing cases in their own harnesses.

## Consequence

The M2.3 null-spread guard — the one guard this track *added* — is the best-tested of the family: it
was born from a silent pass and ships with the exact rejection that pass should have produced. The
two gaps above are logged for a future track; neither blocks any current claim, because in both the
underlying numbers are reported in full and were inspected.
