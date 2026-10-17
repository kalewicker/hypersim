# Gate Formulas

One entry per gate: exact pass/fail predicate, fields read, and the
fixtures that exercise it. Constants trace to `docs/metric_formulas.md`.

Every gate is `Pass` / `Fail` / `Pending`. A gate is `Pending` until the
fields it reads are non-`None` and it has seen the windows it needs.
Science unlocks when **all** eight gates are `Pass`.

Gates read the per-window history of `MetricsRow` values, not a single row,
except where noted.

## 1. Fixture gate

**Reads**: the six-fixture truth table (offline, `hg-eval::fixtures`).
**Pass**: all six fixtures classify to their expected regime
(`docs/fixture_definitions.md`). **Fail**: any mismatch. **Pending**: never
(evaluated eagerly at run start; a live run inherits the compiled-in
result). Exercised by every fixture.

## 2. White-noise gate

**Reads**: `interestingness_score`, `regime`.
**Fail**: the current window is `WhiteNoiseSoup`/`SterileChaos` **and**
`interestingness_score ≥ interestingness_noise_max (0.5)` — noise is being
scored as interesting, which is the failure this gate guards against.
**Pass**: a noise window scores below 0.5, or the window is not noise and
`interestingness_score` is defined. **Pending**: `interestingness_score` is
`None`. Exercised by WhiteNoiseToy (must Pass by scoring low).

## 3. PI gate

**Reads**: `pi_normalized` history.
**Pending**: fewer than `pi_warmup_windows (4)` windows with a defined
`pi_normalized`. **Fail**: any window-to-window jump
`|pi_t − pi_{t-1}| > pi_jump_max (0.5)` (PI must be a stable proxy, not
noise). **Pass**: `pi_warmup_windows` seen and no oversized jump.
Exercised by FrozenGrid/StaticBlockToy (stable high PI ⇒ Pass) and
WhiteNoiseToy (stable low PI ⇒ also Pass; the gate checks stability, not
magnitude).

## 4. Generative gate

**Reads**: `regime` history, `new_type_bin_rate`.
**Pass**: `GenerativeCandidate` held for `sustain_windows (3)` consecutive
windows with `new_type_bin_rate > 0` throughout. **Fail**: a window is
`GenerativeCandidate` but novelty collapsed to 0 within the sustain window.
**Pending**: not yet sustained. Exercised by SlowGenerativeGrowth (Pass)
and UniformGrowthToy (never generative ⇒ stays Pending, not Fail).

## 5. Treadmill gate

**Reads**: `regime` history, `new_type_bin_rate`.
**Fail**: `GrowthTreadmill` persisted for `treadmill_stuck_windows (8)`
windows with `new_type_bin_rate = 0` throughout — the run is stuck growing
without inventing anything. **Pass**: not stuck (either not treadmill, or
novelty appeared). **Pending**: fewer than `treadmill_stuck_windows` seen.
Exercised by UniformGrowthToy (Fail — pure treadmill) and
SlowGenerativeGrowth (Pass — novelty every window).

## 6. Mobile gate

**Reads**: `active_persistent_structures`.
**Pass**: `active_persistent_structures ≥ 1` in the current window.
**Fail**: defined and `0`. **Pending**: `None`. Exercised by
MovingGliderToy (Pass) and StaticBlockToy (Fail — nothing moves).

## 7. Controller gate — **[reserved]**

**Reads**: nothing yet. **Always `Pending`** in the substrate-only
pipeline (no controller layer exists). Excluded from the completeness
gate's "must not be Pending" accounting; documented so its slot in the CSV
and report is defined. Becomes live only with `hg-cognition`.

## 8. Completeness gate

**Reads**: presence/absence of all **[computed]** metric fields for the
final window (`docs/metric_formulas.md`).
**Pass**: every [computed] field is non-`None`/non-`NaN` at least once in
the run — the pipeline produced a full row. **Fail**: a [computed] field
never populated. **Pending**: run not yet finished. [reserved] fields
(`ei_micro`, `ce_*` except where computed, `d_eff`, `spectral_dim`,
`diameter`) and the controller gate are excluded from this check by design.
Exercised by any full engine run.

## science_unlocked

`science_unlocked = all eight gates == Pass`. Because gate 7 (controller)
is permanently `Pending` in the substrate-only build, `science_unlocked`
is **structurally unreachable** in this phase — that is intended. The
substrate pipeline's success criterion is "every [computed] gate that can
Pass does Pass," surfaced in the report's Gate Status section, not the
`science_unlocked` flag. `ScienceUnlocked`/`ScienceLocked` events and the
flag exist and are wired; they simply do not fire until the cognition
layer supplies gate 7.
