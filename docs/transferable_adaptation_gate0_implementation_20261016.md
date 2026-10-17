# TA-1 Gate 0 implementation

Date: 2026-10-16. Development-only authored inheritance assay.

Execution update: [Gate0 passed](transferable_adaptation_gate0_findings_20261016.md)
on both fixed development worlds; the result and validation are sealed.

The [implementation plan](transferable_adaptation_implementation_plan_20261016.md)
now has a fixed [Gate0 contract](../experiments/evolving_minds/contracts/ta1_gate0_v1.md).
This milestone composes the existing paid handoff lifecycle, with a new TA panel
profile and independent endpoints. Existing native checkpoint layouts and prices
retain their meanings; no additional live state requires a new codec here.

The panel contains Useful and Ancestor on exposed worlds17/23, plus a world17
fresh-process recovery control. Both arms start with identical Delayed founders;
Useful installs the known authored g103 edit only in paid founder offspring.
All later copies are faithful. This tests whether useful inheritance is attainable
before changing mutation or spending held-out worlds.

The decision requires an active completed-offspring gain of at least one and a
full-world net resource gain of at least eight per world. Net resource gain is
harvest less all native dissipation; offspring endowments are transfers. In each
founder family, installation is followed by three paid faithful transmissions
along a path whose members all complete development and perform successful work.
Birth counts, siblings, and installation itself cannot satisfy that path.

## Entry points

Build the opt-in binary:

```sh
cargo build -p hg-sim-core --features evolving-minds --bin evolving-minds
```

Print the fixed registration without executing any worlds:

```sh
target/debug/evolving-minds transfer-register
```

Execute the development fixture into a new directory:

```sh
RAYON_NUM_THREADS=4 target/debug/evolving-minds transfer-development output/NEW_TA_GATE0_DIRECTORY
```

Audit using the checker frozen by that binary and write a separate new receipt:

```sh
python3 -B output/NEW_TA_GATE0_DIRECTORY/checker.py output/NEW_TA_GATE0_DIRECTORY --receipt output/NEW_TA_GATE0_AUDIT_DIRECTORY
```

Parent output directories must exist; every evidence destination must be new.
The native archive freezes registration, contract, relevant sources, binary hash,
six segment histories, checkpoints, lifecycle manifests and a complete SHA256
seal. Independent replay reconstructs all46 published states and all14,720 native
advances. The audit is read-only with respect to native evidence; its receipt
has a separate manifest and seal. Execution success and Gate0 feasibility are
distinct. Transfer, discovery, cumulative reuse and scientific acceptance remain
false in this milestone.

## Validation

Targeted Rust tests cover fixed configuration, rejection of altered contracts,
native founder payment and faithful descendant identity. Python endpoint tests
cover functional paths, original denominators, net cost reconciliation, missing
paid births and exact recovery. Optional native tamper tests run with
`EVOLVING_MINDS_TRANSFER_FIXTURE` set to a generated Gate0 directory; they reject
omitted publications, forged intermediate graphs and free-copy proposals.

A failed Gate0 stops this version. It does not trigger longer horizons, changed
endowments, new development seeds, altered thresholds or Gate1 automatically.
