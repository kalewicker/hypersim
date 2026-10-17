# Public repository and final project status

Date: 2026-10-17. Publication target: `kalewicker/hypersim` on GitHub.

The public repository contains the final workbench, isolated LK and cumulative
experiment sources, configurations, rule sets, contracts, tests, independent
checkers, reports and preservation receipts. It starts with a source snapshot.
The original artifact-bearing Git history and a final closeout commit remain
in the local repository on `a-track`; they are not ancestors of the public
branch. This preserves local history without importing old generated runs into
every public clone. Only the public snapshot branch is pushed.

Generated `output/`, `runs/`, build targets, Python caches, local assistant settings
and environment files are excluded. Vendored slotmap code and its existing
license/provenance files remain included. No experimental threshold, kernel or
scientific result was changed for publication.

## Research status

The [project-wide report](project_experimental_report_20261013.md) records the
closed cumulative-evolution programme and its audited negative result. The later,
separately versioned TA-1 work demonstrated supplied useful inheritance and
temporal development feasibility, then stopped at its mutation-retention gate:

- [Implementation plan and completed gates](transferable_adaptation_implementation_plan_20261016.md).
- [Gate0 authored inheritance](transferable_adaptation_gate0_findings_20261016.md).
- [Gate1 temporal development controls](transferable_adaptation_gate1_findings_20261017.md).
- [Final mutation-stress result](transferable_adaptation_mutation_stress_findings_20261017.md)
  and [validation receipt](transferable_adaptation_mutation_stress_validation_20261017.json).

Faithful copying passed all four mutation-stress development comparisons. Both
tested mutation kernels failed the fixed retention screen. Held-out transfer,
discovery and cumulative reuse remain unlaunched. The result does not establish
a universal ceiling for hypergraphs or Hypersim. Publishing the source does not
authorize a new campaign or override its stopping criteria.

## Evidence availability

Historical raw evidence was archived to a separately held flash drive on
October 14. The [archive guide](project_cleanup_20261014.md) and
[verification receipt](project_archive_verification_20261014.json) describe its
scope, hashes and restoration commands. Those archives are not hosted in this
public repository. Later October 16–17 TA-1 outputs are preserved locally and
are **not** covered by that earlier flash backup.

Relative `output/` and `runs/` links in findings describe original evidence
locations; they will not resolve in a fresh public clone. Dated inventories,
source/binary hashes, audit receipts and failure records are historical records,
not claims that the corresponding raw payload is included. Original scientific
checks require the matching evidence and frozen sources. Source availability and
rerun instructions do not substitute for independently checking those originals.

## Building and repeating development assays

Use Rust/Cargo for the workspaces and Python 3.11 or newer for the independent
checkers. The [user guide](user_guide.md) describes the workbench. Run the isolated
LK workspace through `./scripts/lk.sh`; see its
[integration guide](lk_workspace_integration.md). The cumulative workspace keeps
its LK path dependencies within this repository.

For the final TA-1 development panel, use fresh output directories:

```sh
cargo build --locked -p hg-sim-core --features evolving-minds --bin evolving-minds
target/debug/evolving-minds mutation-stress-register
RAYON_NUM_THREADS=4 target/debug/evolving-minds mutation-stress-development output/ta1_reproduction
python3 -B output/ta1_reproduction/checker.py output/ta1_reproduction --receipt output/ta1_reproduction_audit
```

This repeats exposed development conditions; it does not run the reserved
held-out panel. Existing destinations are refused. Keep every resulting case
and its original decision, including failed gates. A newly built binary has its
own recorded hash; a rerun is new evidence rather than the original sealed run.
The [mutation-stress implementation](transferable_adaptation_mutation_stress_implementation_20261017.md)
also documents the separate packaging/recovery probe and its fixed registration.

The last development validation passed 13 targeted Rust tests and 25 distinct
Python tests, plus full native replay of 154 main-panel publications. Some Python
regressions require generated fixture paths supplied through their documented
environment variables. Tests that skip missing fixtures are not equivalent to
the recorded full validation. No new scientific population was run to prepare
this public snapshot.
