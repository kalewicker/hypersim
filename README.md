# Hypersim

**A generalized hypergraph rewriting workbench.**

**Research closeout — 2026-10-13:** The current experimental programme is closed
following the final independently audited cumulative-evolution campaign's valid
negative result. Read the [detailed project experimental report](docs/project_experimental_report_20261013.md)
for the full research history, positive capabilities, failed gates and limits.
The [documentation index](docs/README.md) and
[complete document/evidence catalogue](docs/project_experimental_inventory_20261013.md)
provide the supporting record. The workbench and preserved experiments remain
available; historical proposed continuations are not active campaign authorizations.

**Final follow-up — 2026-10-17:** TA-1 demonstrated authored useful inheritance
and passed its temporal development controls. Its subsequent mutation-stress
panel passed independent replay but failed the registered retention gate;
faithful copying passed, while both tested mutation kernels failed. TA-1 stops
before held-out transfer or discovery. Read the
[mutation-stress findings](docs/transferable_adaptation_mutation_stress_findings_20261017.md)
and [public repository notes](docs/public_repository_notes_20261017.md).

**Evidence storage — 2026-10-14:** Generated run evidence has been archived and
fully restore-verified on the flash drive at `/Volumes/B/Hypersim-archives-2026-10-14`.
The checkout now retains source, configuration, documentation and Git history.
See the [archive and restoration guide](docs/project_cleanup_20261014.md) before
using historical artifact links or running archive-dependent checks.
The public repository contains source, tests, contracts and documentation.
Raw runs, build products, local settings and the earlier artifact-bearing Git
history are excluded; the full historical Git repository remains preserved
locally. The later TA-1 raw evidence remains local and is not part of the
October 14 flash archives. A fresh clone can rerun the documented development
assays, but does not contain the original evidence payloads.

Hypersim is a research tool for defining, running, observing, and
understanding arbitrary hypergraph rewriting systems. It integrates:

1. **A rewriting engine** — any rule set expressible in TOML (sparse
   edge-label constraints, typed property guards, scheduled creation
   rules), deterministic under a fixed seed regardless of thread count.
2. **A scientific evaluator** — 11-regime classification, type-histogram
   diversity/novelty metrics, a persistence-based PI proxy, an
   interestingness score with penalties, and an eight-gate engine grounded
   in a six-fixture truth table.
3. **A causal language layer** — WL-hash attributes, a task lattice,
   a constructor registry with fidelity/replicator tracking, effective
   information over class transitions, and a certainty-tagged knowledge
   graph with a closed predicate vocabulary.

Every ordinary CLI run produces one CSV and one Markdown report; `manifest.json`
is the durable, authoritative record of run status. Same configuration and
seed always produce the same result — the cross-thread-count determinism
test compares CSVs byte-for-byte at 1, 2, and 8 threads.

**Start with the [User Guide](docs/user_guide.md)** — building, running,
rule authoring, reading outputs, and extension points.

Built against `hypersim_implementation_plan_v4.md` (the canonical spec).
The nine Phase 0 specification documents live in `docs/`.

## Quick start

```sh
# Run the six-fixture evaluator truth table
cargo run -p hypersim-cli -- fixtures

# Headless run: one CSV + one report + manifest.json per run
cargo run --release -p hypersim-cli -- run --config configs/example_binary_edge.toml

# Overrides
cargo run --release -p hypersim-cli -- run --config configs/example_crystal.toml \
    --steps 1024 --seed 7 --threads 8 --out-dir runs

# GUI workbench (live simulation backend; the mock-data shell is
# hg-gui's default feature set, exercised by its tests)
cargo run --release -p hypersim

# Logs: span-based tracing, attributable per run/window
RUST_LOG=info cargo run -p hypersim-cli -- run --config configs/example_binary_edge.toml
```

## LK experiment

The related experiment is preserved as an isolated workspace in
[`experiments/lk`](experiments/lk). Run it with `./scripts/lk.sh`; see the
[integration guide](docs/lk_workspace_integration.md) for commands, provenance,
and historical archive requirements.

## Crate architecture (v4 §2)

```
hg-core        the hypergraph: store, snapshot, generators, WL hash,
               PropertyValue (no internal deps)
hg-schema      frozen data shapes: Regime (11), GateStatus/GateName,
               MetricsRow, RunSummary (serde only)
hg-observe     engine extension seams: Observer, RewriteEvent,
               ObserverChain, Binding, RuleSelectionPolicy
hg-rules       TOML grammar -> compiled rules (sparse edge_labels,
               creation-rule schedules, typed guards)
hg-engine      matcher (non-injective by default), conflict resolution
               (priority via Reverse<i32>), apply, step loop
hg-parallel    rayon support behind hg-engine's "parallel" feature
hg-output      CSV/manifest/binary formats — deliberately NO internal deps
hg-eval        window metrics, regime classifier, gates, fixtures
hg-causal      task lattice, constructors, EI, knowledge graph
hg-report      the 11-section report (depends on hg-schema only)
hg-app-api     SimCommand, SimEvent/SimStatusEvent, SimProgress,
               SimController trait, GUI DTOs, MockSimController
hg-sim-core    ordinary application orchestration behind hg-app-api;
               specialized research drivers also use engine APIs
hg-gui         egui workbench: hg-app-api always; hg-sim-core only
               behind the "live" feature
```

Additional implemented research packages include `hg-alife`, `hg-alphabet`,
`hg-cognition`, `hg-train`, `hg-selfmod` and `hg-meta`. See the
[architecture map](docs/project_architecture_and_experiment_status.md) for their
boundaries and the experimental report for their scientific status. The isolated
LK and cumulative workspaces retain distinct source and experiment identities.

## Layout

- `crates/`, `bins/` — the workspace (see above)
- `docs/` — the [User Guide](docs/user_guide.md) plus the nine Phase 0 specification documents
- `rules/` — rule sets: the five builtins (embedded into `hg-rules` at
  compile time) plus `wolfram_analog.toml` / `crystal_battery.toml`,
  loadable via `rules.source = "File"`
- `configs/` — example run configurations
- `output/`, `runs/` — run artifacts (CSV, report.md, manifest.json)

## Testing

```sh
cargo test --workspace
```

Notable suites: canonical-hash construction-order invariance and WL
property tests (hg-core), the six-fixture truth table and white-noise
interestingness invariant (hg-eval), the §8.2 conflict-set invariant and
cross-thread determinism (hg-engine), manifest durability with zero event
listeners (hg-output, hg-sim-core), and a 50k-step run producing a
non-empty task lattice and a constructor candidate (hg-sim-core).
