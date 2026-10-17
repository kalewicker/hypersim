# TA-1 Gate1 temporal transfer implementation

Date: 2026-10-16. Development-only; held-out evaluation and mutation stress
remain unlaunched. This milestone implements the temporal and causal-control
portion of [Gate1 in the plan](transferable_adaptation_implementation_plan_20261016.md).

Completed update: [the corrected preflight passes](transferable_adaptation_gate1_findings_20261017.md).
The original recovery integration failure and a later audit packaging failure
are retained with separate correction/continuation evidence.

## Native implementation

`TransferEnvironment` defines Shuffled, Alternating and Blocks4 resource timing,
with informative, permuted or withheld cues. Alternating switches each tick;
Blocks4 switches every four ticks. Every primary schedule contains32 true and32
false resource values. Phase is a deterministic independent domain of the world
seed. Informative cues track current resource; coin streams, prices, sites,
ring access and native physiology retain their previous meanings. This version
tests temporal transfer only. Common coordinate rotations are not counted as
transfer, and no spatial access intervention is claimed.

The schedule is compiled into the native512 world rules. It is bound through
the segment header, native state, checkpoint and law hash. New dynamics profile
`evolving-minds-ta1-temporal-v1`, streaming profile
`evolving-minds-ta1-temporal-streaming-v1` and codec `HGEMTA01` preserve the old
funded, handoff and Gate0 codec meanings. Resume restores store allocation and
engine state without replaying historical native execution. The new profile
rejects nonfaithful copying and incompatible handoff/retained/descendant policies.

Programs are edited before standardized native initialization. From the authored
Delayed program, g103 source assignments are Ancestor `[2]`, Useful `[0]`,
Reversion `[0,2]`, Sham `[0,0]` and Restoration `[0,2,0]`. Each intermediate
program and canonical byte sequence is recorded. These are laboratory
interventions, not mutations discovered or paid for by organisms. All resulting
genotypes have equal size, identical starting reserves and fresh developmental,
signal, environmental and ancestry state. Subsequent offspring pay full native
request, copying, development, maintenance and endowment costs.

Independent Python reference code derives schedules from the pattern and seed,
checks every world-rule payload, and reconstructs physical transitions, paid
requests, ledgers, ancestry and every checkpoint. It does not accept runner gate
flags as evidence. Useful/Sham/Restoration must have identical complete physical
outcomes; Ancestor/Reversion must also match. Recovery is matched to uninterrupted
Useful outcomes. A successful standardized founder is generation0, not a paid
transmission; three subsequent successful paid edges are required per family.

## Registration and entry points

The [corrected development contract](../experiments/evolving_minds/contracts/ta1_gate1_development_v2.md)
uses world17: ten primary causal-control cells over two temporal families, two
cue-withheld diagnostics and one fresh-process recovery control. This is13
logical cells,14 segments,832 ticks,38,272 advances and118 publications.

```sh
cargo build -p hg-sim-core --features evolving-minds --bin evolving-minds
target/debug/evolving-minds transfer-gate1-register
target/debug/evolving-minds transfer-gate1-evaluation-register
RAYON_NUM_THREADS=4 target/debug/evolving-minds transfer-gate1-development output/ta1_gate1_development_20261016 output/NEW_GATE1_DIRECTORY
python3 -B output/NEW_GATE1_DIRECTORY/checker.py output/NEW_GATE1_DIRECTORY --receipt output/NEW_GATE1_AUDIT_DIRECTORY
```

All evidence destinations must be new. The development writer freezes sources,
contract, intervention bytes, registration and binary hash before worlds advance,
then seals all native payloads. The audit writes a separate sealed receipt.

V1's main twelve cells completed, but its recovery control stopped at the generic
tick8 boundary instead of the contracted24. Its original native archive remains
Interrupted. V2 fixes this integration bug and adds a direct cutoff regression.
The required first argument binds the preserved v1 archive; its full inventory
is frozen before v2 execution. The independent audit checks original preservation
and equality of all twelve complete semantic histories plus the recovery prefix.
The v2 repetition is development verification, not another independent sample.
V1 executed35,696 advances and v2 reserves38,272:73,968 total across both attempts.
No environment, genotype, effect threshold or biological denominator changed.

The executed v2 bundle also retains its original auditor packaging bug. Its
successful verification used the proven local-variable-rename continuation,
with no additional native execution. Recheck that sealed result with a new
audit destination:

```sh
python3 -B scripts/audit_evolving_minds_transfer_gate1_continuation.py output/ta1_gate1_development_v2_20261016 output/ta1_gate1_audit_v2_20261016 output/NEW_CONTINUATION_RECEIPT
python3 -B scripts/check_evolving_minds_transfer_gate1_controls.py output/ta1_gate1_development_v2_20261016 output/ta1_gate1_audit_continuation_20261017
```

The current working checker and rebuilt binary include that packaging repair.
The original frozen checker, failed receipt and completed native evidence remain
unchanged. See the findings and validation receipt for source identities and
the AST restoration proof.

The prospective evaluation registration contains eight new seed blocks derived
from SHA256 of `hypersim-ta1-temporal-evaluation-v1-NNN` (first eight bytes,
little endian), with the same two primary families/five arms plus two diagnostics
per block:96 cells and282,624 advances. Each block is explicitly
`registered_unexposed`. Its scientific budget is unset, power adequacy and launch
flags are false, and there is no held-out execution command. Registration does
not execute schedules or use their outcomes. The two registrations must match
independent Python reconstruction before development exposure.

## Interpretation and stop boundary

Development requires >=1 additional active completed offspring and >=8 net
resource units in each primary family, plus paid useful persistence and exact
causal/recovery controls. Cue withholding is reported separately and is excluded
from these primary comparisons. All failed or censored routes stay in the
denominator; infrastructure failures yield incomplete evidence.

A failed development gate stops this version before held-out execution. A pass
establishes an engineering preflight on an exposed world. It does not establish
held-out transfer or evolutionary discovery. Mutation stress and its grammar-only
operator remain a distinct, later implementation boundary.
