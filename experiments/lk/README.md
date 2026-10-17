# Hypersim

**A generalized hypergraph rewriting workbench.**

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

Every run produces exactly one CSV and one Markdown report; `manifest.json`
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
hg-sim-core    the ONLY crate that imports hg-engine — orchestrates
               everything behind the hg-app-api boundary
hg-gui         egui workbench: hg-app-api always; hg-sim-core only
               behind the "live" feature
```

Reserved, not yet built: `hg-alife`, `hg-cognition` (see the plan's
extension seams — `RuleSelectionPolicy`, `MotifClassifier`,
`property_effects` hook point are already in place).

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
