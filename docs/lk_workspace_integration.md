# Preserved LK experiment workspace

The full source experiment is available at `experiments/lk`. This is a separate
Cargo workspace inside the repository. It retains its own dependency versions,
engine, schemas, rules, and configuration defaults. Main-project commands still
use the root workspace; LK commands use the launcher below. No runtime types or
engine implementations cross the workspace boundary.

## Running it

From the main repository:

```sh
./scripts/lk.sh run --locked --release -p hypersim-cli -- fixtures
./scripts/lk.sh run --locked --release -p hypersim-cli -- run --config configs/example_binary_edge.toml
./scripts/lk.sh test --locked -p hg-sim-core --lib
./scripts/lk.sh test --locked -p hg-sim-core --test lk_fuse_gates --test b1_age_gates
./scripts/lk.sh test --locked -p hg-engine --test age_grammar_b0 --test tick_cost_a0
```

The launcher changes into the experiment directory so historical relative rule
and output paths resolve correctly. It places build products in
`experiments/lk/target` and ordinary relative run outputs inside that workspace.
It works from any current directory. The same commands can be issued directly
from `experiments/lk` using `cargo`.

## Snapshot and provenance

`experiments/lk/provenance.json` records the source branch and commit, the dirty
source status, and SHA-256 checksums of every imported file. The snapshot includes
the source checkout's uncommitted influx implementation and guard-shadow audit.
The original `../hypersim-lk` worktree was left intact. This is a source snapshot,
not a live synchronization mechanism; edits in either workspace do not propagate
to the other.

The source worktree contains research beyond LK, including a divergent ERE and
self-modification implementation. Those sources are retained together because
extracting only their LK dependencies would change the experiment's engine.
The two workspaces deliberately retain duplicate package names. Do not add LK
crates as members or path dependencies of the main workspace.

## Historical artifacts and expensive experiments

Historical `runs/`, `output/`, and training campaign directories were not copied.
The original LK run archive alone occupies approximately 6.8 GB and remains at
`../hypersim-lk/runs`. Findings and preregistrations are preserved under
`experiments/lk/docs`; references to historical run files refer to that original
archive. Archive-dependent audits such as `g0_guard_shadow` and archived-snapshot
round trips require their corresponding historical files to be copied into the
same relative paths in the preserved workspace before running. Missing archives
are not evidence of a null result.

Do not use an unrestricted workspace-wide test run as a quick smoke check:
some historical tests scan archives or launch substantial research campaigns.
The targeted commands above exercise the library and locality gate behavior
without launching the ignored long experiments. Run a named ignored experiment
only deliberately, after reading its source and preregistration.

## Future shared-engine integration

A future port should reconcile rule vocabulary, schema changes, and simulation
hooks separately, with old-config serialization and deterministic replay checks.
The preserved workspace provides an unchanged reference implementation for that
work. This integration itself changes neither main-project dynamics nor the
scientific findings of the experiment.

## Validation and known source issue

The imported files were compared byte-for-byte with the source and checked
against the provenance hashes. Cargo metadata confirms a separate workspace.
The main project's pre-existing tracked patch was checked for exact preservation.

The imported `hg-sim-core` library suite currently has one pre-existing failure:
`influx_accumulator_beats_the_formula`. On this machine the accumulated value is
`400000.0` and graph energy sums to `400000.00000000006`; the test asserts exact
floating-point equality. The same named test fails against the original source
checkout. The source and its scientific assertion are preserved unchanged here;
this integration does not certify the influx audit's claim of exact equality.

The preserved CLI, `hg-meta`, `hg-selfmod`, and `hg-train` compile with all
features enabled. All six CLI fixture classifications pass, and a 16-step
headless sample produces a CSV, report, and completed manifest. The main
workspace's seven leakage flag tests, five leakage run tests, and ten native
ERE integration tests pass. Age/countdown engine checks pass; the archival
round-trip test reports success without an archive, so that result is not
claimed as historical archive coverage.

The source's new `e_injected_total` CSV field also precedes `audit_residual`.
Several historical tests still parse column index 18 as the residual; with
influx disabled that column is now empty. `lk_fuse_gates`'s
`fg1_audit_exact_and_fg2_no_churn` and the two `b1_age_gates` audit checks fail
at that stale positional read. This schema/test mismatch is preserved from the
source snapshot. These suites are useful diagnostics but are not currently
all green; parse by header when repairing them in a separate follow-up.

Final targeted results: `b1_age_gates` passes 4 tests (including determinism)
and fails the 2 stale CSV audit readers; `lk_fuse_gates` passes 3 and fails
1 stale CSV reader; the influx library passes 5 and fails its exact-sum test.
The two engine suites pass 4 runnable checks with 3 long experiments ignored.
