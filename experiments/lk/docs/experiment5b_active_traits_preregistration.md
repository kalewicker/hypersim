# Experiment 5b — pre-registration: activating harvest_rate & maintenance_cost

**Committed 2026-07-15, BEFORE running the genome-scaled economy.** Implements plan
deviation #4 (genome-scaled harvest/maintenance, deferred until "the traits' selective effect
is wanted"), using the new `add_from_property`/`subtract_from_property` effect ops. Motivated
by the criterion-5 validation: `harvest_rate` and `maintenance_cost` were the *inert drift
null* precisely because no rule read them. Coupling them tests whether **any** trait, once
under selection here, shifts distinguishably from drift — i.e. whether criterion 5 can be met
non-degenerately. Forbidden move still binds: **do not tune to manufacture a clean result.**

## 1. The change (config-gated, off by default)

`AlifeConfig.genome_scaled_metabolism = true` rewrites two rules via `apply_alife_overrides`:
- **metabolise**: `carrier.energy −= carrier.maintenance_cost` (was fixed 0.5).
- **harvest**: transfer `carrier.harvest_rate` (site −rate, carrier +rate; was fixed 1.0),
  with the site-floor guard changed to `site.energy ≥ carrier.harvest_rate` so a drifted-up
  rate can't drive a site negative.
- **bootstrap**: `maintenance_cost` seeded at 0.5 (= old quantum) so the **initial economy is
  identical**; `harvest_rate` already 1.0. Divergence appears only as genomes jitter.

Off by default ⟹ the fixed-quantum economy and every calibrated test are byte-unchanged
(exit-criterion 7). Jitter remains UNCLAMPED (the S2 decision), so an active trait can drift
past its nominal bounds — including `maintenance_cost < 0`.

## 2. Fixed parameters
Turnover regime (`configs/experiment5.toml`): seed sweep {12345, 1, 7, 99999, 2024, 42},
influx 0.34, bootstrap repl_threshold 3.0, 10 000 steps, `genome_scaled_metabolism = true`.
Drift reference = the criterion-5 fixed-economy run, where these two traits were pure drift
(sign-MIXED across seeds, |Δ| envelope ≈ 4.5).

## 3. Predictions (committed)

- **P1 — `maintenance_cost` collapses degenerately AND is destructive.** Nothing opposes a
  lower metabolic drain (lower maintenance = live longer = more reproduction, at zero cost),
  so `maintenance_cost` shifts DIRECTIONALLY DOWN, sign-consistent across seeds (unlike its
  sign-mixed drift in the fixed economy). As the drain → 0 (or negative, unclamped), the
  energy-≤0 death mechanism weakens, so **turnover collapses toward immortal saturation**
  (deaths/step decline over the run). This is the §1 degenerate attractor again — now on a
  trait whose collapse removes the mortality the whole regime depends on.
- **P2 — `harvest_rate` is under selection with a resource externality.** Higher rate grabs
  more energy (selected up) but depletes the shared site faster (a real trade-off under
  contention). Prediction: `harvest_rate` shifts directionally (sign-consistent, beyond the
  drift envelope), most likely UP; whether it settles at a stable interior value or runs to a
  boundary is the open question P3 turns on.
- **P3 — criterion-5 verdict.** Activating these traits most likely **adds more degenerate
  free directions** (maintenance→floor; harvest→up) rather than a stabilized trade-off, so
  criterion 5 stays degenerate. The one way it becomes NON-degenerate: a trait settling at a
  stable INTERIOR optimum with directional selection distinguishable from drift.

## 4. Falsification / honest outcomes

- **Non-degenerate selection found (the interesting positive):** if `harvest_rate` (or
  `maintenance_cost`) converges to a stable interior value — not a boundary — with a
  reproducible directional signal beyond the drift envelope, criterion 5 is met
  non-degenerately. Report it plainly (it would be the first non-degenerate selection in the
  project).
- **Turnover survives with maintenance→0:** if deaths persist despite `maintenance_cost`
  collapsing (some other mortality route dominates), P1's destructive claim is wrong; report.
- **Economy breaks (sites negative / population explosion / extinction at bootstrap):** a
  mechanism bug, not a finding — diagnose the guard/ops before any selection claim.
- **Determinism:** the genome-scaled run must be reproducible at a fixed seed.

## 5. Decision table

| outcome | interpretation |
|---|---|
| maintenance↓ to floor, turnover collapses; harvest↑ unbounded | **P1+P3 confirmed:** active traits are more degenerate free directions; criterion 5 stays degenerate; deviation #4 does not rescue it — a genuine, if deflating, finding. |
| a trait settles at a stable interior optimum, selection > drift | **Non-degenerate selection** — criterion 5 met for real; the headline result. |
| turnover survives, both traits drift-like | activating them changed little; report and reconsider whether the coupling is strong enough. |
