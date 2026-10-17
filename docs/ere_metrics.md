# ERE Metrics (R0 spec)

**Status:** specification. Every new field is `Option<T>` at the schema
level; `None` serializes as `unmeasured` (empty CSV cell) and is
distinguished from `0` in the report. Never silently convert unmeasured to
zero. Target home: `hg-schema::MetricsRow` (today: `crates/hg-schema/src/
metrics.rs:9-63` — ERE fields land as additive `Option` columns, CSV
byte-identical when unmeasured).

## New `MetricsRow` fields

```text
rule_region_nodes                  Option<usize>  live nodes in rule region(s)
rule_region_edges                  Option<usize>  live edges in rule region(s)
rules_installed                    Option<usize>  installed this window
rules_retired                      Option<usize>  retired this window
rules_quarantined                  Option<usize>  quarantined this window
quarantine_by_validator            Option<[usize; 9]>  V1..V9 counters (present incl. zeros)
rule_rcf_alive                     Option<usize>  distinct RCF classes installed
rule_rcf_novel_cumulative          Option<usize>  never-before-seen RCF classes
rule_rcf_novel_rate                Option<f64>    per-epoch novel RCF rate (OEE curve)
rule_survival_half_life            Option<f64>    by handle identity (gen-0 anchors), steps
rule_reuse_count_by_class          Option<map<RcfHash, usize>>  firings per RCF class
meta_rewrite_events                Option<usize>  meta-rewrites applied this window
meta_rewrite_site_entropy          Option<f64>    Shannon over edit-site WL class (M2 detector)
meta_edges_lost_to_cleanup         Option<usize>  must read 0 under protection (G-R5)
rule_replicator_kind               Option<RuleReplicatorKind>  None|Trivial|NonTrivial
epoch_len_effective                Option<u64>    actual steps between compiles
tpm_rule_set_stable                Option<bool>   G-R6 assertion result for the window
```

```rust
pub enum RuleReplicatorKind { None, Trivial, NonTrivial }
```

`Trivial` = the rule's only observable effect is reproducing its own
encoding. `NonTrivial` additionally requires ≥1 firing whose binding contains
no node with label ≥ `META_BASE` (target band; current code uses the
productive-effect gate as proxy — `runner.rs:410-460`). Every headline claim
is gated on `NonTrivial`.

## Formulas and thresholds (single source of truth)

| Constant | Value | Used by |
|---|---|---|
| `NOVELTY_SLOPE_THRESHOLD` | `1.0e-4` per step | novelty window open/close (`runner.rs:36`) |
| `FM_R7_ALARM` | `0.9` quarantined fraction | quarantine-capture alarm |
| `G_R1_VALID_MIN` | `0.40` | mutational-neutrality gate |
| `G_R1_NEAR_NEUT_MIN` | `0.10` | mutational-neutrality gate |
| `G_R1_CATAST_MAX` | `0.25` | mutational-neutrality gate |
| `G_R1_REDZONE` | `0.10` Valid floor | below: redesign encoding, don't tune loop |
| `G_R1_SAMPLE` | `10⁴` single-edit mutations over `≥20` seed rules | EXP-R01 |
| `G_R1_TRAJ_WINDOW` | `≥100` steps identical trajectory | NearNeutral definition |
| `G_R3_STEPS` | `10⁵` steps bit-identical (A1≡A0) | interpreter identity |
| `G_R2_STEPS` | `1000` steps behavioral identity per rule | expressivity round-trip |
| `FREEZE_STEPS` | `50` consecutive `productive_applied==0` | `time_to_freeze` |
| `EPOCH_SWEEP` | `{∞, 10⁴, 10³, 10², 10, 1}` | EXP-R08 |
| `RULE_REGION_CEIL` | config (default world-side `500` nodes) | FM-R3, logged binding/non-binding |

## Hazards

**M1 — metrics contamination by the rule region.** The rule region is ordinary
nodes/edges and enters counts, WL histograms, novelty, PI, EI by default.
**Required:** every substrate metric computed twice (meta-excluded and
meta-included), both reported; where one is meaningless the other is
`unmeasured` with a stated reason. No metric silently picks one. (Precedent:
`substrate.csv` split, `docs/metric_formulas.md` substrate variants.)

**M2 — point-source meta pump.** Expect meta-rewrites to concentrate at one
site (LK-track lesson). `meta_rewrite_site_entropy` exists to catch it:
entropy → 0 while events stay high = pump recurring at meta level = finding
about energy metering, not encoding.

## Current-code mapping

`hg-selfmod` `run.csv` (`runner.rs:547-552`) emits: `rule_count`,
`alive_rule_classes` (legacy hash — replace with `rule_rcf_alive`),
`M_rules`/`M_data` (cumulative registries — replace rule side with
`rule_rcf_novel_cumulative`), `decode_failures` (→ `rules_quarantined` +
per-Vk), `births/clonal/mutant/deaths`, `mean_tokens`, `lineage_depth_*`,
`data_hash`. Missing and unmeasured until ported: region node/edge counts,
per-class reuse, site entropy, half-life, replicator kind, epoch length,
TPM stability. The port keeps old column names where semantics are unchanged
so existing artifacts stay comparable; renamed/redefined columns are
versioned per `docs/schema_versions.md`.
