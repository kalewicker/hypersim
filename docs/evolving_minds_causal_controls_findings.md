# Evolving Minds causal-control engineering findings

**Roadmap update (2026-10-01):** The [plan review](evolving_minds_plan_review_2026-10-01.md) revises the next-work sequence to choice-interface calibration before a larger D0 language. Results below remain the historical engineering receipt.

Date: 2026-09-29. The next engineering step is implemented and independently
verified: causal probes of the authored relay's signal-to-intake path.

The opt-in CLI now accepts `interventions NEW_DIRECTORY`. It persists a default
two-tick BuildRelay baseline, then independently resumes its final checkpoint
for four diagnostic forks. Semantic targets are resolved by stable birth ID 1,
owner and unique material label. Every fork restores the same engine RNG and
allocator state. No scientific campaign, evolutionary search or training runs.

## Matched preparation and measured outcomes

All arms reset the actuator signal to false so a stale actuator cannot bypass
the signal phase. Energy, maintenance prices, node/edge inventory and incidence
remain identical at the intervention boundary. Additional investigator edits
are recorded explicitly; they do not consume or create metabolic energy.

| Arm | Additional edit | Native signal events | Native intake events | Acquired energy | Lifecycle-valid state |
| --- | --- | ---: | ---: | ---: | --- |
| sham | None | 1 | 1 | 8 | Yes |
| signal_clamp | Sensor signal false | 1 | 0 | 0 | Yes |
| foreign_owner | Connection owner 999 | 0 | 0 | 0 | No |
| owner_rescue | Owner 999, then restored to 1 | 1 | 1 | 8 | Yes |

The false signal still traverses the relay but does not engage intake. Foreign
ownership prevents the native relay rule from matching; normal material
validation also rejects that state. Repair restores the sham's complete native
phase history exactly. Total metabolic reserve is conserved in all probe
phases. These results calibrate a known authored mechanism; the repair is an
investigator intervention, not autonomous adaptation.

## Evidence and implementation

Verified receipt: [verification.json](../output/evolving_minds_causal_controls_verified_20260929/verification.json).
The frozen independent checker passes all four arms, and its 52 source-inventory
entries matched the live files at verification. The archive contains baseline
traces/checkpoints, probe frames/events, edit provenance, validity classification,
fixed contract, status, manifests, frozen checkers and SHA-256 integrity receipt.

- Runner: `crates/hg-sim-core/src/evolving_minds/interventions.rs`.
- Fixed scope: [causal_controls_v1.md](../experiments/evolving_minds/contracts/causal_controls_v1.md).
- Independent checker: `scripts/verify_evolving_minds_interventions.py`.
- Native tests: `crates/hg-sim-core/tests/evolving_minds_interventions.rs`.
- Tamper tests: `scripts/test_verify_evolving_minds_interventions.py`.

Two new native tests pass: matched budgets/semantic targeting, ownership
rejection and exact rescue, repeated fork independence, and unsupported/corrupt
checkpoint rejection. The existing nine Evolving Minds integration tests pass.
Nine Python controls pass, covering clean read-only verification, wrong targets,
unmatched initial energy, omitted native events, false owner admission, false
outcomes, missing phases, omitted rescue edits and false effect counters.
The all-features compile check passes.

The first development archive `output/evolving_minds_causal_controls_20260929`
is retained. Its frozen checker rejected the edit-provenance encoding because
standalone property values serialize as scalar JSON while graph properties
carry type tags. The revised checker handles the declared scalar edit schema;
the new verified archive freezes that correction. The physical laws and
interventions did not change, and the older archive is not the acceptance receipt.

## Reproduce

Run from the repository root with a fresh output path:

```sh
cargo run -p hg-sim-core --features evolving-minds --bin evolving-minds -- interventions output/evolving_minds_new_controls
python3 -B output/evolving_minds_new_controls/checker.py output/evolving_minds_new_controls
cargo test -p hg-sim-core --features evolving-minds --test evolving_minds_interventions --test evolving_minds
EVOLVING_MINDS_INTERVENTIONS_FIXTURE=output/evolving_minds_new_controls python3 -B scripts/test_verify_evolving_minds_interventions.py
```

## Remaining boundary

These forks run only Signal and Action at engine steps 19/20; environmental tick
remains 2. They are not complete ecological continuations, and their stores are
never admitted back into the lifecycle runner. There is no later Sense phase
that would overwrite the sensor clamp. This restricted checker reconstructs
the baseline, edits and native effects; it is not a general matcher auditor.

The next implementation step is the bounded multi-instruction developmental
schema, with independently checked copying and mutation provenance. Useful
resource choices and fair encoding controls still need a separate ecology
revision. H1/H2 acceptance remains incomplete; evolution, learning and memory
claims remain outside this engineering receipt.
