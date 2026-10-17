# Project documentation and evidence catalogue — 2026-10-13

**Storage update — 2026-10-14:** This is the pre-cleanup historical inventory.
Its evidence-directory links now name restoration locations. The evidence is
stored in fully verified flash-drive archives; see the
[archive/restoration guide](project_cleanup_20261014.md). The JSON inventory is
preserved as the original snapshot rather than regenerated without its evidence.

[Read the experimental report](project_experimental_report_20261013.md).

This catalogue indexes **505 first-party Markdown files**, **301 immediate evidence directories**, and **2843 metadata files**.

Scope: Markdown outside build, vendor and evidence trees; named lifecycle/audit metadata at directory depths 0–3 below `output`, `runs`, isolated LK equivalents and the two `crates/hg-train/runs_stage2` roots. Nested evidence payloads and external historical LK archives are not recursively inventoried. No simulation, replay, or historical audit was launched. Counts describe files, not independent experiments.

[Machine inventory and SHA-256 identities](project_experimental_inventory_20261013.json). [Generator](../scripts/build_project_closeout_inventory.py). Raw statuses are preserved: a complete lifecycle is not scientific acceptance. A missing status means no recognized direct metadata field, not a negative result.

## ALife

- [Experiment 5 — Pre-registration (written before the S4 run)](../docs/experiment5_preregistration.md) — `docs/experiment5_preregistration.md`
- [Experiment 5 — S6 gate report (scorecard)](../docs/experiment5_scorecard.md) — `docs/experiment5_scorecard.md`
- [Experiment 5b — pre-registration: activating harvest_rate & maintenance_cost](../docs/experiment5b_active_traits_preregistration.md) — `docs/experiment5b_active_traits_preregistration.md`
- [Experiment 5b — findings: activating harvest_rate & maintenance_cost](../docs/experiment5b_findings.md) — `docs/experiment5b_findings.md`
- [Experiment 5c — findings: parent-funded replication (§1 option 2)](../docs/experiment5c_findings.md) — `docs/experiment5c_findings.md`
- [Experiment 5c — pre-registration: parent-funded replication (§1 option 2)](../docs/experiment5c_parent_funded_preregistration.md) — `docs/experiment5c_parent_funded_preregistration.md`
- [Experiment 5d — findings: energy-proportional reproductive tax](../docs/experiment5d_findings.md) — `docs/experiment5d_findings.md`
- [Experiment 5d — pre-registration: an energy-proportional reproductive tax](../docs/experiment5d_rich_penalty_preregistration.md) — `docs/experiment5d_rich_penalty_preregistration.md`
- [Experiment 5e — pre-registration: reproductive senescence (rate-based aging)](../docs/experiment5e_aging_preregistration.md) — `docs/experiment5e_aging_preregistration.md`
- [Experiment 5e — findings: reproductive senescence, and a fundamental limit](../docs/experiment5e_findings.md) — `docs/experiment5e_findings.md`
- [Experiment 5 — Pre-registration (written before the S4 run)](../experiments/lk/docs/experiment5_preregistration.md) — `experiments/lk/docs/experiment5_preregistration.md`
- [Experiment 5 — S6 gate report (scorecard)](../experiments/lk/docs/experiment5_scorecard.md) — `experiments/lk/docs/experiment5_scorecard.md`
- [Experiment 5b — pre-registration: activating harvest_rate & maintenance_cost](../experiments/lk/docs/experiment5b_active_traits_preregistration.md) — `experiments/lk/docs/experiment5b_active_traits_preregistration.md`
- [Experiment 5b — findings: activating harvest_rate & maintenance_cost](../experiments/lk/docs/experiment5b_findings.md) — `experiments/lk/docs/experiment5b_findings.md`
- [Experiment 5c — findings: parent-funded replication (§1 option 2)](../experiments/lk/docs/experiment5c_findings.md) — `experiments/lk/docs/experiment5c_findings.md`
- [Experiment 5c — pre-registration: parent-funded replication (§1 option 2)](../experiments/lk/docs/experiment5c_parent_funded_preregistration.md) — `experiments/lk/docs/experiment5c_parent_funded_preregistration.md`
- [Experiment 5d — findings: energy-proportional reproductive tax](../experiments/lk/docs/experiment5d_findings.md) — `experiments/lk/docs/experiment5d_findings.md`
- [Experiment 5d — pre-registration: an energy-proportional reproductive tax](../experiments/lk/docs/experiment5d_rich_penalty_preregistration.md) — `experiments/lk/docs/experiment5d_rich_penalty_preregistration.md`
- [Experiment 5e — pre-registration: reproductive senescence (rate-based aging)](../experiments/lk/docs/experiment5e_aging_preregistration.md) — `experiments/lk/docs/experiment5e_aging_preregistration.md`
- [Experiment 5e — findings: reproductive senescence, and a fundamental limit](../experiments/lk/docs/experiment5e_findings.md) — `experiments/lk/docs/experiment5e_findings.md`

## Architecture, substrate, locality and supporting contracts

- [Hypersim documentation](../docs/README.md) — `docs/README.md`
- [A-track / A1 — event-signature definition and the radius selection rule](../docs/atrack_a1_preregistration.md) — `docs/atrack_a1_preregistration.md`
- [A-track findings — the substrate's realised event alphabet](../docs/atrack_findings.md) — `docs/atrack_findings.md`
- [Locality as a Balance, Not a Geometry](../docs/attractor_writeup.md) — `docs/attractor_writeup.md`
- [B-track findings — what bounds the alphabet, and what the A-track could not resolve](../docs/btrack_findings.md) — `docs/btrack_findings.md`
- [B-track pre-registration — rule-count scaling, horizon de-confounding](../docs/btrack_preregistration.md) — `docs/btrack_preregistration.md`
- [Campaign output — artifacts, layout, and the contamination firewall](../docs/campaign_output.md) — `docs/campaign_output.md`
- [C-track findings — writing rules into the zoo's measured gap](../docs/ctrack_findings.md) — `docs/ctrack_findings.md`
- [C-track pre-registration — the gap rules, their mechanisms, and their predicted behaviour](../docs/ctrack_preregistration.md) — `docs/ctrack_preregistration.md`
- [D-track findings — node-budget contention, and the conservation control](../docs/dtrack_findings.md) — `docs/dtrack_findings.md`
- [D-track pre-registration — node-budget contention, and the conservation control](../docs/dtrack_preregistration.md) — `docs/dtrack_preregistration.md`
- [E-track findings — the grammar hole is closed, and the qualifier comes off](../docs/etrack_findings.md) — `docs/etrack_findings.md`
- [E-track pre-registration — closing the grammar hole](../docs/etrack_preregistration.md) — `docs/etrack_preregistration.md`
- [Experimental status and continuation review — 2026-09-26](../docs/experimental_status_review_2026-09-26.md) — `docs/experimental_status_review_2026-09-26.md`
- [Fitness formulas — the learning layer's single source of truth](../docs/fitness_formulas.md) — `docs/fitness_formulas.md`
- [LK0 Fixture Calibration (EXP-L1)](../docs/fixture_calibration.md) — `docs/fixture_calibration.md`
- [Evaluator Fixture Definitions](../docs/fixture_definitions.md) — `docs/fixture_definitions.md`
- [Gate Formulas](../docs/gate_formulas.md) — `docs/gate_formulas.md`
- [Instrument registry — engagement counters and predicted ranges](../docs/instrument_registry.md) — `docs/instrument_registry.md`
- [Closure-Defect Spec (LK4)](../docs/leakage_closure_spec.md) — `docs/leakage_closure_spec.md`
- [Locality & Leakage — Experiment Findings (LK track)](../docs/leakage_experiment_findings.md) — `docs/leakage_experiment_findings.md`
- [Preserved LK experiment workspace](../docs/lk_workspace_integration.md) — `docs/lk_workspace_integration.md`
- [Metric Formulas](../docs/metric_formulas.md) — `docs/metric_formulas.md`
- [Policy configuration — `(cognition)` and `(training)`](../docs/policy_config.md) — `docs/policy_config.md`
- [Hypersim Architecture and Experiment Status](../docs/project_architecture_and_experiment_status.md) — `docs/project_architecture_and_experiment_status.md`
- [Hypersim: experimental programme and closeout report](../docs/project_experimental_report_20261013.md) — `docs/project_experimental_report_20261013.md`
- [Property Registry](../docs/property_registry.md) — `docs/property_registry.md`
- [Reproducibility Policy Specification](../docs/reproducibility_policy.md) — `docs/reproducibility_policy.md`
- [Rewrite Contract Specification](../docs/rewrite_contract.md) — `docs/rewrite_contract.md`
- [Rule TOML Grammar Specification](../docs/rule_grammar.md) — `docs/rule_grammar.md`
- [Run State Machine Specification](../docs/run_state_machine.md) — `docs/run_state_machine.md`
- [Schema Versioning Specification](../docs/schema_versions.md) — `docs/schema_versions.md`
- [Hypersim storage cleanup — 2026-10-01](../docs/storage_cleanup_2026-10-01.md) — `docs/storage_cleanup_2026-10-01.md`
- [Hypersim User Guide](../docs/user_guide.md) — `docs/user_guide.md`
- [Cumulative evolution: completed campaign and readiness history](../experiments/cumulative/README.md) — `experiments/cumulative/README.md`
- [Hypersim](../experiments/lk/README.md) — `experiments/lk/README.md`
- [Locality as a Balance, Not a Geometry](../experiments/lk/docs/attractor_writeup.md) — `experiments/lk/docs/attractor_writeup.md`
- [Measurement Back-Action, Quantified — v8 / EXP-L18](../experiments/lk/docs/back_action_v8.md) — `experiments/lk/docs/back_action_v8.md`
- [Campaign output — artifacts, layout, and the contamination firewall](../experiments/lk/docs/campaign_output.md) — `experiments/lk/docs/campaign_output.md`
- [Q7-1 — Census-Zero Resolution (chain census over EXP-L17)](../experiments/lk/docs/chain_census_l17.md) — `experiments/lk/docs/chain_census_l17.md`
- [Q7-2 — Tick-Blocking Audit (directional contention)](../experiments/lk/docs/contention_audit_v7.md) — `experiments/lk/docs/contention_audit_v7.md`
- [Fitness formulas — the learning layer's single source of truth](../experiments/lk/docs/fitness_formulas.md) — `experiments/lk/docs/fitness_formulas.md`
- [LK0 Fixture Calibration (EXP-L1)](../experiments/lk/docs/fixture_calibration.md) — `experiments/lk/docs/fixture_calibration.md`
- [Evaluator Fixture Definitions](../experiments/lk/docs/fixture_definitions.md) — `experiments/lk/docs/fixture_definitions.md`
- [Gate Formulas](../experiments/lk/docs/gate_formulas.md) — `experiments/lk/docs/gate_formulas.md`
- [Grammar Audit — v6 Phase A0 (single-node LHS feasibility)](../experiments/lk/docs/grammar_audit_v6.md) — `experiments/lk/docs/grammar_audit_v6.md`
- [Grammar Audit — v8 Phase B0 (age guard feasibility)](../experiments/lk/docs/grammar_audit_v8.md) — `experiments/lk/docs/grammar_audit_v8.md`
- [G0.1 — The guard-shadow audit (v10 §2.1, F-LK33)](../experiments/lk/docs/guard_shadow_audit_v10.md) — `experiments/lk/docs/guard_shadow_audit_v10.md`
- [Instrument registry — engagement counters and predicted ranges](../experiments/lk/docs/instrument_registry.md) — `experiments/lk/docs/instrument_registry.md`
- [Closure-Defect Spec (LK4)](../experiments/lk/docs/leakage_closure_spec.md) — `experiments/lk/docs/leakage_closure_spec.md`
- [Locality & Leakage — Experiment Findings (LK track)](../experiments/lk/docs/leakage_experiment_findings.md) — `experiments/lk/docs/leakage_experiment_findings.md`
- [Metric Formulas](../experiments/lk/docs/metric_formulas.md) — `experiments/lk/docs/metric_formulas.md`
- [Policy configuration — `(cognition)` and `(training)`](../experiments/lk/docs/policy_config.md) — `experiments/lk/docs/policy_config.md`
- [Property Registry](../experiments/lk/docs/property_registry.md) — `experiments/lk/docs/property_registry.md`
- [Reproducibility Policy Specification](../experiments/lk/docs/reproducibility_policy.md) — `experiments/lk/docs/reproducibility_policy.md`
- [Rewrite Contract Specification](../experiments/lk/docs/rewrite_contract.md) — `experiments/lk/docs/rewrite_contract.md`
- [Rule TOML Grammar Specification](../experiments/lk/docs/rule_grammar.md) — `experiments/lk/docs/rule_grammar.md`
- [Run State Machine Specification](../experiments/lk/docs/run_state_machine.md) — `experiments/lk/docs/run_state_machine.md`
- [Schema Versioning Specification](../experiments/lk/docs/schema_versions.md) — `experiments/lk/docs/schema_versions.md`
- [Hypersim User Guide](../experiments/lk/docs/user_guide.md) — `experiments/lk/docs/user_guide.md`
- [hg-alife — Implementation Plan (Build Spec)](../experiments/lk/hg_alife_implementation_plan.md) — `experiments/lk/hg_alife_implementation_plan.md`
- [hg-alife — Scope & Architecture](../experiments/lk/hg_alife_scope_and_architecture.md) — `experiments/lk/hg_alife_scope_and_architecture.md`
- [Hypersim Implementation Plan v4](../experiments/lk/hypersim_implementation_plan_v4.md) — `experiments/lk/hypersim_implementation_plan_v4.md`
- [Hypersim Substrate Characterization Report](../experiments/lk/hypersim_substrate_characterization_report.md) — `experiments/lk/hypersim_substrate_characterization_report.md`

## Causal emergence

- [Experiment 6 — findings: causal emergence across the replication-onset boundary](../docs/experiment6_findings.md) — `docs/experiment6_findings.md`
- [Experiment 6 — pre-registration: causal emergence across the replication-onset boundary](../docs/experiment6_preregistration.md) — `docs/experiment6_preregistration.md`
- [Experiment 6b — findings: birth/death-aware CE fixes the identity macro](../docs/experiment6b_findings.md) — `docs/experiment6b_findings.md`
- [Experiment 6b — pre-registration: birth/death-aware CE (fixing the identity macro)](../docs/experiment6b_preregistration.md) — `docs/experiment6b_preregistration.md`
- [Experiment 6c — findings: population-level causal emergence](../docs/experiment6c_findings.md) — `docs/experiment6c_findings.md`
- [Experiment 6c — pre-registration: population-level causal emergence](../docs/experiment6c_population_ce_preregistration.md) — `docs/experiment6c_population_ce_preregistration.md`
- [Experiment 6 — findings: causal emergence across the replication-onset boundary](../experiments/lk/docs/experiment6_findings.md) — `experiments/lk/docs/experiment6_findings.md`
- [Experiment 6 — pre-registration: causal emergence across the replication-onset boundary](../experiments/lk/docs/experiment6_preregistration.md) — `experiments/lk/docs/experiment6_preregistration.md`
- [Experiment 6b — findings: birth/death-aware CE fixes the identity macro](../experiments/lk/docs/experiment6b_findings.md) — `experiments/lk/docs/experiment6b_findings.md`
- [Experiment 6b — pre-registration: birth/death-aware CE (fixing the identity macro)](../experiments/lk/docs/experiment6b_preregistration.md) — `experiments/lk/docs/experiment6b_preregistration.md`
- [Experiment 6c — findings: population-level causal emergence](../experiments/lk/docs/experiment6c_findings.md) — `experiments/lk/docs/experiment6c_findings.md`
- [Experiment 6c — pre-registration: population-level causal emergence](../experiments/lk/docs/experiment6c_population_ce_preregistration.md) — `experiments/lk/docs/experiment6c_population_ce_preregistration.md`

## Cognition

- [Cognition Stage 0a / 0b / 1 — diagnostics findings](../docs/cognition_stage0_findings.md) — `docs/cognition_stage0_findings.md`
- [Cognition Stage 2 — evolved rule-selection bias](../docs/cognition_stage2_findings.md) — `docs/cognition_stage2_findings.md`
- [Cognition Stages 4 & 5 — task lattice and effective information](../docs/cognition_stage45_findings.md) — `docs/cognition_stage45_findings.md`
- [D-phase closeout — selection-learning is stateable, and it took five instrument fixes to find out](../experiments/lk/docs/cognition_dphase_closeout_findings.md) — `experiments/lk/docs/cognition_dphase_closeout_findings.md`
- [E0 — the quantizer audit](../experiments/lk/docs/cognition_e0_quantizer_audit.md) — `experiments/lk/docs/cognition_e0_quantizer_audit.md`
- [E2 — the D6 re-run under N-1, and what score normalization changed](../experiments/lk/docs/cognition_e2_n1_rerun_findings.md) — `experiments/lk/docs/cognition_e2_n1_rerun_findings.md`
- [E3 — Stage 0b″ and Stage 5′](../experiments/lk/docs/cognition_e3_stage0b2_stage5prime_findings.md) — `experiments/lk/docs/cognition_e3_stage0b2_stage5prime_findings.md`
- [E4 — authored substrates, and Stage 8 replication](../experiments/lk/docs/cognition_e4_stage8_findings.md) — `experiments/lk/docs/cognition_e4_stage8_findings.md`
- [E5 — Stage 4′ (task lattice) and Stage 6 (phenotype contrast)](../experiments/lk/docs/cognition_e5_stage4prime_stage6_findings.md) — `experiments/lk/docs/cognition_e5_stage4prime_stage6_findings.md`
- [E-phase closeout — plan v3, complete](../experiments/lk/docs/cognition_ephase_closeout_findings.md) — `experiments/lk/docs/cognition_ephase_closeout_findings.md`
- [Cognition Stage 0a / 0b / 1 — diagnostics findings](../experiments/lk/docs/cognition_stage0_findings.md) — `experiments/lk/docs/cognition_stage0_findings.md`
- [Cognition Stage 2 — evolved rule-selection bias](../experiments/lk/docs/cognition_stage2_findings.md) — `experiments/lk/docs/cognition_stage2_findings.md`
- [Stage 2′ — F4/F5 stateability, and the programme's go/no-go](../experiments/lk/docs/cognition_stage2_prime_findings.md) — `experiments/lk/docs/cognition_stage2_prime_findings.md`
- [Stage 3′ — `MlpScorer` under the objectives that cleared Stage 2′](../experiments/lk/docs/cognition_stage3_prime_findings.md) — `experiments/lk/docs/cognition_stage3_prime_findings.md`
- [Cognition Stages 4 & 5 — task lattice and effective information](../experiments/lk/docs/cognition_stage45_findings.md) — `experiments/lk/docs/cognition_stage45_findings.md`
- [Stage 0b′ and Stage 1′ — the separation and memory verdicts under r2](../experiments/lk/docs/cognition_stage_0b1_prime_findings.md) — `experiments/lk/docs/cognition_stage_0b1_prime_findings.md`
- [Stage R — the re-analysis passes](../experiments/lk/docs/cognition_stage_r_findings.md) — `experiments/lk/docs/cognition_stage_r_findings.md`

## Constructive chemistry

- [CC-0: bounded constructive chemistry compatibility specification v1](../docs/constructive_chemistry_cc0_compatibility_spec_v1.md) — `docs/constructive_chemistry_cc0_compatibility_spec_v1.md`
- [CC-0 chemistry compatibility specification — 2026-10-05](../docs/constructive_chemistry_cc0_findings_20261005.md) — `docs/constructive_chemistry_cc0_findings_20261005.md`
- [CC-1: atomic constructive chemistry engineering fixture](../docs/constructive_chemistry_cc1_findings_20261005.md) — `docs/constructive_chemistry_cc1_findings_20261005.md`
- [Constructive chemistry and higher-order rewriting: Hypersim adaptation](../docs/constructive_chemistry_higher_order_hypersim_plan_20261005.md) — `docs/constructive_chemistry_higher_order_hypersim_plan_20261005.md`

## ERE and grammar

- [ERE Rule Canonical Form (R0 spec)](../docs/ere_canonical_form.md) — `docs/ere_canonical_form.md`
- [Direct categorical adapter: implementation and engineering verification](../docs/ere_categorical_adapter_implementation_20261001.md) — `docs/ere_categorical_adapter_implementation_20261001.md`
- [Direct categorical adapter v1: engineering contract](../docs/ere_categorical_adapter_v1.md) — `docs/ere_categorical_adapter_v1.md`
- [Historical/archive-only accounting policy v1](../docs/ere_categorical_comparison_archive_audit_v1.md) — `docs/ere_categorical_comparison_archive_audit_v1.md`
- [Memory-bounded auditor: packaging-only engineering continuation](../docs/ere_categorical_comparison_bounded_auditor_continuation_v2.md) — `docs/ere_categorical_comparison_bounded_auditor_continuation_v2.md`
- [Categorical comparison: memory-bounded auditor findings](../docs/ere_categorical_comparison_bounded_auditor_findings_20261002.md) — `docs/ere_categorical_comparison_bounded_auditor_findings_20261002.md`
- [Categorical comparison: memory-bounded supplemental auditor contract](../docs/ere_categorical_comparison_bounded_auditor_v1.md) — `docs/ere_categorical_comparison_bounded_auditor_v1.md`
- [Memory-bounded auditor: source-stage verification correction](../docs/ere_categorical_comparison_bounded_auditor_verification_v3.md) — `docs/ere_categorical_comparison_bounded_auditor_verification_v3.md`
- [Final engineering verifier: explicit import correction](../docs/ere_categorical_comparison_bounded_auditor_verification_v4.md) — `docs/ere_categorical_comparison_bounded_auditor_verification_v4.md`
- [Categorical comparison: completed native development fixture](../docs/ere_categorical_comparison_development_20261002.md) — `docs/ere_categorical_comparison_development_20261002.md`
- [Categorical comparison harness and accounting auditor v1](../docs/ere_categorical_comparison_harness_v1.md) — `docs/ere_categorical_comparison_harness_v1.md`
- [Categorical comparison: historical corrected accounting complete](../docs/ere_categorical_comparison_historical_audit_findings_20261003.md) — `docs/ere_categorical_comparison_historical_audit_findings_20261003.md`
- [Categorical-versus-ES harness and auditor implementation](../docs/ere_categorical_comparison_implementation_20261002.md) — `docs/ere_categorical_comparison_implementation_20261002.md`
- [Categorical comparison: launch bridge implementation and verification](../docs/ere_categorical_comparison_launch_bridge_findings_20261002.md) — `docs/ere_categorical_comparison_launch_bridge_findings_20261002.md`
- [Categorical comparison: prospective storage-only launch bridge](../docs/ere_categorical_comparison_launch_bridge_v1.md) — `docs/ere_categorical_comparison_launch_bridge_v1.md`
- [Ordered-sum supplemental audit: tests pass, final live-source check blocked](../docs/ere_categorical_comparison_ordered_audit_findings_20261003.md) — `docs/ere_categorical_comparison_ordered_audit_findings_20261003.md`
- [Postmeasurement ordered-sum accounting correction v1](../docs/ere_categorical_comparison_ordered_audit_v1.md) — `docs/ere_categorical_comparison_ordered_audit_v1.md`
- [Categorical versus ES comparison: preregistration receipt](../docs/ere_categorical_comparison_preregistration_20261001.md) — `docs/ere_categorical_comparison_preregistration_20261001.md`
- [Categorical versus ES scientific trial: native complete, frozen audit failed](../docs/ere_categorical_comparison_scientific_findings_20261003.md) — `docs/ere_categorical_comparison_scientific_findings_20261003.md`
- [Categorical versus ES: prospective native direction comparison](../docs/ere_categorical_comparison_v1.md) — `docs/ere_categorical_comparison_v1.md`
- [Categorical-feasibility native development pilot: complete, no improvement claim](../docs/ere_categorical_feasible_development_findings_20261003.md) — `docs/ere_categorical_feasible_development_findings_20261003.md`
- [Categorical-feasibility development harness: zero-work preflight complete](../docs/ere_categorical_feasible_development_preflight_findings_20261003.md) — `docs/ere_categorical_feasible_development_preflight_findings_20261003.md`
- [Feasibility-aware categorical native development fixture v1](../docs/ere_categorical_feasible_development_v1.md) — `docs/ere_categorical_feasible_development_v1.md`
- [Feasibility-aware categorical proposal: isolated adapter engineering complete](../docs/ere_categorical_feasible_findings_20261003.md) — `docs/ere_categorical_feasible_findings_20261003.md`
- [Feasibility-aware categorical proposal v1: offline engineering contract](../docs/ere_categorical_feasible_v1.md) — `docs/ere_categorical_feasible_v1.md`
- [Feasibility-aware categorical proposal v2: aligned risk estimand](../docs/ere_categorical_feasible_v2.md) — `docs/ere_categorical_feasible_v2.md`
- [Complete-witness checkpoint/continuation findings](../docs/ere_checkpoint_continuation_findings.md) — `docs/ere_checkpoint_continuation_findings.md`
- [Complete-witness checkpoint/continuation verification](../docs/ere_checkpoint_continuation_v1.md) — `docs/ere_checkpoint_continuation_v1.md`
- [Coherent frozen auditor: profiling and optimization findings](../docs/ere_coherent_auditor_optimization_findings.md) — `docs/ere_coherent_auditor_optimization_findings.md`
- [Coherent auditor optimization: development diagnostic protocol](../docs/ere_coherent_auditor_optimization_v1.md) — `docs/ere_coherent_auditor_optimization_v1.md`
- [Coherent G-R1 closure campaign findings](../docs/ere_coherent_closure_campaign_findings_20260929.md) — `docs/ere_coherent_closure_campaign_findings_20260929.md`
- [Coherent G-R1 closure campaign](../docs/ere_coherent_closure_campaign_v1.md) — `docs/ere_coherent_closure_campaign_v1.md`
- [Coherent auditor per-draw overhead: findings](../docs/ere_coherent_draw_optimization_findings.md) — `docs/ere_coherent_draw_optimization_findings.md`
- [Coherent per-draw auditor overhead: prospective development protocol](../docs/ere_coherent_draw_optimization_v1.md) — `docs/ere_coherent_draw_optimization_v1.md`
- [Full-scale coherent follow-up: engineering findings](../docs/ere_coherent_full_scale_engineering_findings_20260929.md) — `docs/ere_coherent_full_scale_engineering_findings_20260929.md`
- [Full-scale coherent follow-up: engineering scope](../docs/ere_coherent_full_scale_engineering_v1.md) — `docs/ere_coherent_full_scale_engineering_v1.md`
- [Complete-profile receipt-bound admission and bounded runner](../docs/ere_complete_admission_runner_findings_20260930.md) — `docs/ere_complete_admission_runner_findings_20260930.md`
- [Receipt-bound complete-profile admission and bounded runner v1](../docs/ere_complete_admission_runner_v1.md) — `docs/ere_complete_admission_runner_v1.md`
- [Fixed-champion confirmation findings](../docs/ere_complete_champion_confirmation_findings_20261001.md) — `docs/ere_complete_champion_confirmation_findings_20261001.md`
- [Fixed-champion confirmation v1](../docs/ere_complete_champion_confirmation_v1.md) — `docs/ere_complete_champion_confirmation_v1.md`
- [Coherent harness and frozen auditor: development findings](../docs/ere_complete_coherent_assay_implementation.md) — `docs/ere_complete_coherent_assay_implementation.md`
- [Complete-profile coherent mutation assay: prospective version 1](../docs/ere_complete_coherent_assay_v1.md) — `docs/ere_complete_coherent_assay_v1.md`
- [Complete coherent full assay: preparation and launch status](../docs/ere_complete_coherent_draw_assay_preflight_20260929.md) — `docs/ere_complete_coherent_draw_assay_preflight_20260929.md`
- [Complete coherent draw-local full assay: prospective version 1](../docs/ere_complete_coherent_draw_assay_v1.md) — `docs/ere_complete_coherent_draw_assay_v1.md`
- [Complete coherent full assay: launch authorization](../docs/ere_complete_coherent_draw_full_authorization_20260929.md) — `docs/ere_complete_coherent_draw_full_authorization_20260929.md`
- [Complete coherent draw-local full trial: findings](../docs/ere_complete_coherent_draw_full_trial_findings_20260929.md) — `docs/ere_complete_coherent_draw_full_trial_findings_20260929.md`
- [Integrated draw-local coherent resource pilot: findings](../docs/ere_complete_coherent_draw_resource_pilot_findings.md) — `docs/ere_complete_coherent_draw_resource_pilot_findings.md`
- [Complete coherent draw-local resource pilot: prospective version 1](../docs/ere_complete_coherent_draw_resource_pilot_v1.md) — `docs/ere_complete_coherent_draw_resource_pilot_v1.md`
- [Isolated coherent assay: implementation and all-parent smoke](../docs/ere_complete_coherent_isolated_assay_implementation.md) — `docs/ere_complete_coherent_isolated_assay_implementation.md`
- [Complete-profile isolated coherent mutation assay: prospective version 1](../docs/ere_complete_coherent_isolated_assay_v1.md) — `docs/ere_complete_coherent_isolated_assay_v1.md`
- [Complete-profile coherent kernel: implementation and development findings](../docs/ere_complete_coherent_kernel_implementation.md) — `docs/ere_complete_coherent_kernel_implementation.md`
- [Complete-profile coherent mutation kernel: prospective development contract](../docs/ere_complete_coherent_kernel_v1.md) — `docs/ere_complete_coherent_kernel_v1.md`
- [Integrated optimized coherent resource pilot: findings](../docs/ere_complete_coherent_optimized_resource_pilot_findings.md) — `docs/ere_complete_coherent_optimized_resource_pilot_findings.md`
- [Complete coherent optimized resource pilot: prospective version 1](../docs/ere_complete_coherent_optimized_resource_pilot_v1.md) — `docs/ere_complete_coherent_optimized_resource_pilot_v1.md`
- [Complete coherent isolated resource pilot: findings](../docs/ere_complete_coherent_resource_pilot_findings.md) — `docs/ere_complete_coherent_resource_pilot_findings.md`
- [Complete coherent isolated resource pilot: prospective version 1](../docs/ere_complete_coherent_resource_pilot_v1.md) — `docs/ere_complete_coherent_resource_pilot_v1.md`
- [Direction assay launch authorization](../docs/ere_complete_direction_assay_authorization_20261001.md) — `docs/ere_complete_direction_assay_authorization_20261001.md`
- [Fixed-parameter update-direction assay: full scientific findings](../docs/ere_complete_direction_assay_findings_20261001.md) — `docs/ere_complete_direction_assay_findings_20261001.md`
- [Update-direction reliability: implementation and development verification](../docs/ere_complete_direction_implementation_20261001.md) — `docs/ere_complete_direction_implementation_20261001.md`
- [Fixed-parameter update-direction reliability assay](../docs/ere_complete_direction_v1.md) — `docs/ere_complete_direction_v1.md`
- [Feasibility-aware training findings](../docs/ere_complete_feasibility_training_findings_20261001.md) — `docs/ere_complete_feasibility_training_findings_20261001.md`
- [Feasibility-aware short training: prospective contract](../docs/ere_complete_feasibility_training_v1.md) — `docs/ere_complete_feasibility_training_v1.md`
- [Complete-profile gates: authorized fresh run](../docs/ere_complete_gate_fresh_run_findings.md) — `docs/ere_complete_gate_fresh_run_findings.md`
- [Complete-profile deterministic gate harness: findings](../docs/ere_complete_gate_harness_findings.md) — `docs/ere_complete_gate_harness_findings.md`
- [Complete-profile deterministic gate harness v1](../docs/ere_complete_gate_harness_v1.md) — `docs/ere_complete_gate_harness_v1.md`
- [Full complete-profile G-R1 trial authorization](../docs/ere_complete_gr1_full_authorization_20260927.md) — `docs/ere_complete_gr1_full_authorization_20260927.md`
- [Full complete-profile G-R1 mutation trial](../docs/ere_complete_gr1_full_trial_findings.md) — `docs/ere_complete_gr1_full_trial_findings.md`
- [Complete-profile G-R1: harness, frozen auditor and development verification](../docs/ere_complete_gr1_implementation.md) — `docs/ere_complete_gr1_implementation.md`
- [Complete-profile G-R1 mutation classification: prospective contract](../docs/ere_complete_gr1_v1.md) — `docs/ere_complete_gr1_v1.md`
- [Full complete-profile long-gate trial authorization](../docs/ere_complete_long_gate_full_authorization_20260927.md) — `docs/ere_complete_long_gate_full_authorization_20260927.md`
- [Complete-profile full G-R3/G-R5 trial findings](../docs/ere_complete_long_gate_full_trial_findings.md) — `docs/ere_complete_long_gate_full_trial_findings.md`
- [Complete-profile long gates: implementation and development evidence](../docs/ere_complete_long_gate_implementation.md) — `docs/ere_complete_long_gate_implementation.md`
- [Complete-profile G-R3/G-R5 long-horizon preregistration v1](../docs/ere_complete_long_gates_v1.md) — `docs/ere_complete_long_gates_v1.md`
- [Budget-matched minibatch experiment: implementation and development verification](../docs/ere_complete_minibatch_implementation_20261001.md) — `docs/ere_complete_minibatch_implementation_20261001.md`
- [Minibatch trial launch authorization](../docs/ere_complete_minibatch_trial_authorization_20261001.md) — `docs/ere_complete_minibatch_trial_authorization_20261001.md`
- [Budget-matched minibatch training: full-trial findings](../docs/ere_complete_minibatch_trial_findings_20261001.md) — `docs/ere_complete_minibatch_trial_findings_20261001.md`
- [Budget-matched single-emission versus minibatch training](../docs/ere_complete_minibatch_v1.md) — `docs/ere_complete_minibatch_v1.md`
- [Complete-profile optimizer signal findings](../docs/ere_complete_optimizer_signal_findings_20260929.md) — `docs/ere_complete_optimizer_signal_findings_20260929.md`
- [Complete-profile optimizer signal diagnostic v1](../docs/ere_complete_optimizer_signal_v1.md) — `docs/ere_complete_optimizer_signal_v1.md`
- [Complete-witness profile: admission status and next gates](../docs/ere_complete_profile_gate_status.md) — `docs/ere_complete_profile_gate_status.md`
- [Complete-profile scaling trial findings](../docs/ere_complete_scaling_trial_findings_20260930.md) — `docs/ere_complete_scaling_trial_findings_20260930.md`
- [Complete-profile 50k/50k scaling trial v1](../docs/ere_complete_scaling_trial_v1.md) — `docs/ere_complete_scaling_trial_v1.md`
- [v2 engineering correction](../docs/ere_complete_scaling_trial_v2.md) — `docs/ere_complete_scaling_trial_v2.md`
- [100-step training continuation v2](../docs/ere_complete_short_training_continuation_v2.md) — `docs/ere_complete_short_training_continuation_v2.md`
- [Reliability fix and 100-step training findings](../docs/ere_complete_short_training_findings_20260930.md) — `docs/ere_complete_short_training_findings_20260930.md`
- [Complete-family 100-step training v1](../docs/ere_complete_short_training_v1.md) — `docs/ere_complete_short_training_v1.md`
- [Complete witness codec: implementation and findings](../docs/ere_complete_witness_findings.md) — `docs/ere_complete_witness_findings.md`
- [Complete witness ownership codec: frozen bounded contract](../docs/ere_complete_witness_v1.md) — `docs/ere_complete_witness_v1.md`
- [ERE saved-receipt coverage, seed variation, and attainable-performance review](../docs/ere_coverage_headroom_findings_20261003.md) — `docs/ere_coverage_headroom_findings_20261003.md`
- [Defense allocation accounting correction v2](../docs/ere_defense_allocation_accounting_v2.md) — `docs/ere_defense_allocation_accounting_v2.md`
- [Allocation accounting completion v3: historical regression packaging](../docs/ere_defense_allocation_completion_v3.md) — `docs/ere_defense_allocation_completion_v3.md`
- [Allocation completion v4: full historical input set](../docs/ere_defense_allocation_completion_v4.md) — `docs/ere_defense_allocation_completion_v4.md`
- [Defense allocation diagnostic: findings, failures and extension map](../docs/ere_defense_allocation_findings_20261007.md) — `docs/ere_defense_allocation_findings_20261007.md`
- [Defense allocation diagnostic: prospective contract v1](../docs/ere_defense_allocation_v1.md) — `docs/ere_defense_allocation_v1.md`
- [ERE Determinism Hazards D1/D2 (R0 spec, RuleSpec implementation)](../docs/ere_determinism.md) — `docs/ere_determinism.md`
- [E1.1 shared-pattern redesign preregistration](../docs/ere_e11_redesign.md) — `docs/ere_e11_redesign.md`
- [E2-aware grammar and bounded pilot](../docs/ere_e2_grammar_pilot.md) — `docs/ere_e2_grammar_pilot.md`
- [E2 maintenance-and-reuse pilot v1](../docs/ere_e2_maintenance_pilot.md) — `docs/ere_e2_maintenance_pilot.md`
- [E2 pooled 400k/50k classification trial](../docs/ere_e2_pooled_400k50k_trial.md) — `docs/ere_e2_pooled_400k50k_trial.md`
- [E2 pooled matching: frozen implementation and verification plan](../docs/ere_e2_pooled_budget.md) — `docs/ere_e2_pooled_budget.md`
- [E2 pooled full gate verification](../docs/ere_e2_pooled_full_gates.md) — `docs/ere_e2_pooled_full_gates.md`
- [E2 alternative-rule families: preregistration and implementation record](../docs/ere_e2_semantic_redesign.md) — `docs/ere_e2_semantic_redesign.md`
- [One-use activation persistence: results and implementation](../docs/ere_edit_persistence_findings.md) — `docs/ere_edit_persistence_findings.md`
- [One-use activation persistence control v1](../docs/ere_edit_persistence_v1.md) — `docs/ere_edit_persistence_v1.md`
- [ERE Encoding Schema (R0 spec)](../docs/ere_encoding_schema.md) — `docs/ere_encoding_schema.md`
- [ERE Failure Register (R0 spec)](../docs/ere_failure_register.md) — `docs/ere_failure_register.md`
- [EXP-R01 / G-R1 findings — mutational neutrality of the RuleSpec kernel](../docs/ere_gr1_findings.md) — `docs/ere_gr1_findings.md`
- [G-R1 Mutational-Neutrality Protocol (EXP-R01)](../docs/ere_gr1_protocol.md) — `docs/ere_gr1_protocol.md`
- [EXP-R02 / G-R2 findings — expressivity envelope of the RuleSpec encoding](../docs/ere_gr2_findings.md) — `docs/ere_gr2_findings.md`
- [G-R2 Expressivity Protocol (EXP-R02)](../docs/ere_gr2_protocol.md) — `docs/ere_gr2_protocol.md`
- [Frozen encoder native confirmation, prospective v1](../docs/ere_graph_encoder_confirmation_v1.md) — `docs/ere_graph_encoder_confirmation_v1.md`
- [Exact-gradient graph encoder diagnostic: findings, 2026-10-07](../docs/ere_graph_encoder_exact_gradient_findings_20261007.md) — `docs/ere_graph_encoder_exact_gradient_findings_20261007.md`
- [E2-graph-encoder-exact-gradient-v1: prospective archive-reuse diagnostic](../docs/ere_graph_encoder_exact_v1.md) — `docs/ere_graph_encoder_exact_v1.md`
- [Raw-graph encoder: engineering findings and architecture](../docs/ere_graph_encoder_findings_20261005.md) — `docs/ere_graph_encoder_findings_20261005.md`
- [Native raw-graph encoder learning: findings and next boundary](../docs/ere_graph_encoder_learning_findings_20261007.md) — `docs/ere_graph_encoder_learning_findings_20261007.md`
- [Native raw-graph encoder learning: prospective V1 contract](../docs/ere_graph_encoder_learning_v1.md) — `docs/ere_graph_encoder_learning_v1.md`
- [Frozen encoder native confirmation and survival measurements](../docs/ere_graph_encoder_native_confirmation_findings_20261007.md) — `docs/ere_graph_encoder_native_confirmation_findings_20261007.md`
- [Raw-graph encoder: prospective engineering contract](../docs/ere_graph_encoder_v1.md) — `docs/ere_graph_encoder_v1.md`
- [Allocator identity isolation: implementation and verification](../docs/ere_identity_isolation_implementation.md) — `docs/ere_identity_isolation_implementation.md`
- [Prospective allocator identity isolation v1](../docs/ere_identity_isolation_v1.md) — `docs/ere_identity_isolation_v1.md`
- [Label-separated ecology trial findings](../docs/ere_label_separated_findings.md) — `docs/ere_label_separated_findings.md`
- [Label-separated repeated-damage ecology v1](../docs/ere_label_separated_v1.md) — `docs/ere_label_separated_v1.md`
- [Schema-preserving meta vocabulary v1](../docs/ere_meta_vocabulary_v1.md) — `docs/ere_meta_vocabulary_v1.md`
- [ERE Metrics (R0 spec)](../docs/ere_metrics.md) — `docs/ere_metrics.md`
- [Native ERE implementation and gate status](../docs/ere_native_implementation.md) — `docs/ere_native_implementation.md`
- [Complete-witness repeated-damage experiment findings](../docs/ere_repeated_damage_findings.md) — `docs/ere_repeated_damage_findings.md`
- [Complete-witness repeated-damage ecology v1](../docs/ere_repeated_damage_v1.md) — `docs/ere_repeated_damage_v1.md`
- [Repeated-self-pair feasibility v1: findings](../docs/ere_repeated_edge_feasibility_findings_20260929.md) — `docs/ere_repeated_edge_feasibility_findings_20260929.md`
- [Repeated-self-pair feasibility v1: engineering kernel revision](../docs/ere_repeated_edge_feasibility_v1.md) — `docs/ere_repeated_edge_feasibility_v1.md`
- [Resource adapter corrected accounting v2](../docs/ere_resource_adapter_accounting_v2.md) — `docs/ere_resource_adapter_accounting_v2.md`
- [Resource-token observation/action adapter findings — 2026-10-08](../docs/ere_resource_adapter_findings_20261008.md) — `docs/ere_resource_adapter_findings_20261008.md`
- [Resource-token adapter engineering v1](../docs/ere_resource_adapter_v1.md) — `docs/ere_resource_adapter_v1.md`
- [Fixed graph-cue contrast adapter: zero-native preflight v1](../docs/ere_resource_contrast_adapter_v1.md) — `docs/ere_resource_contrast_adapter_v1.md`
- [Fixed cue-contrast adapter — engineering preflight passes](../docs/ere_resource_contrast_preflight_findings_20261009.md) — `docs/ere_resource_contrast_preflight_findings_20261009.md`
- [Resource-allocation gradient diagnostic — 2026-10-09](../docs/ere_resource_gradient_diagnostic_findings_20261009.md) — `docs/ere_resource_gradient_diagnostic_findings_20261009.md`
- [Saved-receipt gradient and feature-margin diagnostic v1](../docs/ere_resource_gradient_diagnostic_v1.md) — `docs/ere_resource_gradient_diagnostic_v1.md`
- [Bounded resource-allocation learning campaign — complete, local criterion fails](../docs/ere_resource_learning_campaign_findings_20261008.md) — `docs/ere_resource_learning_campaign_findings_20261008.md`
- [Bounded learning runner/auditor preflight — 2026-10-08](../docs/ere_resource_learning_preflight_findings_20261008.md) — `docs/ere_resource_learning_preflight_findings_20261008.md`
- [Bounded resource-allocation learning proposal v1 — unlaunched](../docs/ere_resource_learning_proposal_v1.md) — `docs/ere_resource_learning_proposal_v1.md`
- [Bounded on-policy learning runner and zero-native preflight v1](../docs/ere_resource_learning_runner_v1.md) — `docs/ere_resource_learning_runner_v1.md`
- [Live resource-allocation bridge findings — 2026-10-08](../docs/ere_resource_timed_bridge_findings_20261008.md) — `docs/ere_resource_timed_bridge_findings_20261008.md`
- [Corrected timing-bridge preflight v2](../docs/ere_resource_timed_bridge_preflight_v2.md) — `docs/ere_resource_timed_bridge_preflight_v2.md`
- [Live resource-allocation timing bridge v1](../docs/ere_resource_timed_bridge_v1.md) — `docs/ere_resource_timed_bridge_v1.md`
- [Paid rule-edit choice adapter: implementation and findings](../docs/ere_rule_choice_adapter_findings_20261004.md) — `docs/ere_rule_choice_adapter_findings_20261004.md`
- [Paid rule-edit choice adapter V1 — prospective engineering contract](../docs/ere_rule_choice_adapter_v1.md) — `docs/ere_rule_choice_adapter_v1.md`
- [Tiny paid-edit learning trial: findings and implementation](../docs/ere_rule_choice_learning_findings_20261004.md) — `docs/ere_rule_choice_learning_findings_20261004.md`
- [Tiny paid-edit learning trial V1 — prospective contract](../docs/ere_rule_choice_learning_v1.md) — `docs/ere_rule_choice_learning_v1.md`
- [Tiny paid-edit learning trial V2 — zero-native-work source-scope correction](../docs/ere_rule_choice_learning_v2.md) — `docs/ere_rule_choice_learning_v2.md`
- [Composition receipts V2 — archive-only target-identity correction](../docs/ere_rule_composition_archive_audit_v2.md) — `docs/ere_rule_composition_archive_audit_v2.md`
- [Two-paid-edit composition: implementation and findings](../docs/ere_rule_composition_findings_20261004.md) — `docs/ere_rule_composition_findings_20261004.md`
- [Shared two-paid-edit learning: implementation and findings](../docs/ere_rule_composition_learning_findings_20261004.md) — `docs/ere_rule_composition_learning_findings_20261004.md`
- [Shared two-paid-edit learning trial V1](../docs/ere_rule_composition_learning_v1.md) — `docs/ere_rule_composition_learning_v1.md`
- [Two-paid-edit composition V1 — prospective engineering contract](../docs/ere_rule_composition_v1.md) — `docs/ere_rule_composition_v1.md`
- [In-run rule birth and lifecycle: completed bounded fixture](../docs/ere_rule_lifecycle_fixture_findings_20261003.md) — `docs/ere_rule_lifecycle_fixture_findings_20261003.md`
- [Bounded in-run rule-birth/lifecycle fixture v1](../docs/ere_rule_lifecycle_fixture_v1.md) — `docs/ere_rule_lifecycle_fixture_v1.md`
- [Newly born rule maintenance: recovery passes, continuity gap remains](../docs/ere_rule_maintenance_findings_20261007.md) — `docs/ere_rule_maintenance_findings_20261007.md`
- [Post-birth maintenance prospective contract v1](../docs/ere_rule_maintenance_v1.md) — `docs/ere_rule_maintenance_v1.md`
- [Paid prevention primitive: implementation and findings](../docs/ere_rule_prevention_primitive_findings_20261007.md) — `docs/ere_rule_prevention_primitive_findings_20261007.md`
- [Paid prevention primitive: prospective contract v1](../docs/ere_rule_prevention_primitive_v1.md) — `docs/ere_rule_prevention_primitive_v1.md`
- [Frozen-controller paid-prevention transfer: findings and extension map](../docs/ere_rule_prevention_transfer_findings_20261007.md) — `docs/ere_rule_prevention_transfer_findings_20261007.md`
- [Frozen-controller paid-prevention transfer: prospective v1](../docs/ere_rule_prevention_transfer_v1.md) — `docs/ere_rule_prevention_transfer_v1.md`
- [Relation-sensitive task and graph-conditioned policy: findings](../docs/ere_rule_relational_findings_20261004.md) — `docs/ere_rule_relational_findings_20261004.md`
- [Paired relational learning: findings and extension map](../docs/ere_rule_relational_learning_findings_20261004.md) — `docs/ere_rule_relational_learning_findings_20261004.md`
- [Paired graph-aware / graph-blind learning V1](../docs/ere_rule_relational_learning_v1.md) — `docs/ere_rule_relational_learning_v1.md`
- [Relation-sensitive task and graph-conditioned interface V1](../docs/ere_rule_relational_v1.md) — `docs/ere_rule_relational_v1.md`
- [Rule survival measurement v1](../docs/ere_rule_survival_measurement_v1.md) — `docs/ere_rule_survival_measurement_v1.md`
- [Complete-profile 50k/50k signal confirmation](../docs/ere_runtime50k_signal_panel_findings_20260930.md) — `docs/ere_runtime50k_signal_panel_findings_20260930.md`
- [Complete-profile 50k/50k signal confirmation v1](../docs/ere_runtime50k_signal_panel_v1.md) — `docs/ere_runtime50k_signal_panel_v1.md`
- [Sampler-to-reward mapping: read-only diagnostic](../docs/ere_sampler_reward_mapping_20261001.md) — `docs/ere_sampler_reward_mapping_20261001.md`
- [Paid sealed-payload repair: implementation and findings](../docs/ere_sealed_repair_findings.md) — `docs/ere_sealed_repair_findings.md`
- [Paid sealed-payload label repair v1](../docs/ere_sealed_repair_v1.md) — `docs/ere_sealed_repair_v1.md`
- [Paid sealed-payload label repair: harness revision 2](../docs/ere_sealed_repair_v2.md) — `docs/ere_sealed_repair_v2.md`
- [Paid sealed-payload label repair: characterization revision 3](../docs/ere_sealed_repair_v3.md) — `docs/ere_sealed_repair_v3.md`
- [Next experiment: maintenance of a newly born useful rule](../docs/ere_sustained_rule_maintenance_next_v1.md) — `docs/ere_sustained_rule_maintenance_next_v1.md`
- [Valid-production optimizer signal panel](../docs/ere_valid_optimizer_signal_panel_findings_20260929.md) — `docs/ere_valid_optimizer_signal_panel_findings_20260929.md`
- [Valid-production optimizer signal panel v1](../docs/ere_valid_optimizer_signal_panel_v1.md) — `docs/ere_valid_optimizer_signal_panel_v1.md`
- [Complete-profile validated production grammar](../docs/ere_valid_production_grammar_implementation.md) — `docs/ere_valid_production_grammar_implementation.md`
- [Complete-profile validated production grammar v1](../docs/ere_valid_production_grammar_v1.md) — `docs/ere_valid_production_grammar_v1.md`
- [ERE Validators (R0 spec)](../docs/ere_validators.md) — `docs/ere_validators.md`
- [Witness repair: adversarial test findings](../docs/ere_witness_adversarial_findings.md) — `docs/ere_witness_adversarial_findings.md`
- [Witness repair adversarial engineering controls v1](../docs/ere_witness_adversarial_tests_v1.md) — `docs/ere_witness_adversarial_tests_v1.md`
- [Witness-backed damage prevention and paid repair: implementation and results](../docs/ere_witness_repair_findings.md) — `docs/ere_witness_repair_findings.md`
- [Witness-backed sealing and paid repair v1: frozen contract](../docs/ere_witness_repair_v1.md) — `docs/ere_witness_repair_v1.md`
- [Witness-backed sealing and paid repair v2: frozen contract](../docs/ere_witness_repair_v2.md) — `docs/ere_witness_repair_v2.md`
- [Corrected witness-backed damage prevention and paid repair v2](../docs/ere_witness_repair_v2_findings.md) — `docs/ere_witness_repair_v2_findings.md`
- [Grammar Expansion Without Blobs — Sub-track Plan v1 (GE)](../experiments/lk/docs/ere_grammar_expansion_plan_v1.md) — `experiments/lk/docs/ere_grammar_expansion_plan_v1.md`
- [EXP-R09 findings — the fee-gated outer loop (GE plan §10)](../experiments/lk/docs/ere_r09_findings.md) — `experiments/lk/docs/ere_r09_findings.md`
- [R5 findings — panel-primary outer loop vs the EXP-R07 null](../experiments/lk/docs/ere_r5_findings.md) — `experiments/lk/docs/ere_r5_findings.md`
- [ERE Phase R6 — Measurement Pass (first results)](../experiments/lk/docs/ere_r6_findings.md) — `experiments/lk/docs/ere_r6_findings.md`
- [ERE Validators V1–V9 — Phase R0 deliverable](../experiments/lk/docs/ere_validators.md) — `experiments/lk/docs/ere_validators.md`
- [EXP-R09 findings — the ERE outer loop under F5′ (ERE plan §10, §12)](../experiments/lk/docs/exp_r09_f5prime_findings.md) — `experiments/lk/docs/exp_r09_f5prime_findings.md`
- [G-GE1 findings — the mutation kernel's growth bias](../experiments/lk/docs/gge1_findings.md) — `experiments/lk/docs/gge1_findings.md`
- [G-GE2 audit — blob-panel status vs GE plan §5 (2026-09-04)](../experiments/lk/docs/gge2_audit.md) — `experiments/lk/docs/gge2_audit.md`
- [G-GE2 findings — first panel light (slice 3, 2026-09-04)](../experiments/lk/docs/gge2_findings.md) — `experiments/lk/docs/gge2_findings.md`

## Evolving Minds

- [Parallel birth admission in the full D0 ecology](../docs/evolving_minds_admission_findings.md) — `docs/evolving_minds_admission_findings.md`
- [Evolving Minds causal-control engineering findings](../docs/evolving_minds_causal_controls_findings.md) — `docs/evolving_minds_causal_controls_findings.md`
- [Authored resource-choice engineering findings](../docs/evolving_minds_choice_findings.md) — `docs/evolving_minds_choice_findings.md`
- [Evolving Minds, constructive chemistry, and higher-order rewriting in Hypersim](../docs/evolving_minds_constructive_chemistry_integrated_plan_20261005.md) — `docs/evolving_minds_constructive_chemistry_integrated_plan_20261005.md`
- [Bounded D0 development: implementation and engineering verification](../docs/evolving_minds_d0_findings.md) — `docs/evolving_minds_d0_findings.md`
- [Descendant behavior and inherited-class propagation — 2026-10-05](../docs/evolving_minds_descendant_behavior_findings_20261005.md) — `docs/evolving_minds_descendant_behavior_findings_20261005.md`
- [Paid descendant mutation-route diagnosis — 2026-10-04](../docs/evolving_minds_descendant_route_findings_20261004.md) — `docs/evolving_minds_descendant_route_findings_20261004.md`
- [Evolving Minds: first engineering implementation](../docs/evolving_minds_engineering_implementation.md) — `docs/evolving_minds_engineering_implementation.md`
- [Matched-pilot failure diagnosis and verified shared replay audit](../docs/evolving_minds_failure_audit_optimization_findings_20261004.md) — `docs/evolving_minds_failure_audit_optimization_findings_20261004.md`
- [Paid faithful D0 inheritance: engineering findings](../docs/evolving_minds_family_findings.md) — `docs/evolving_minds_family_findings.md`
- [Configurable funded admission and live recovery](../docs/evolving_minds_funded_admission_findings_20261003.md) — `docs/evolving_minds_funded_admission_findings_20261003.md`
- [Parent-funded offspring calibration](../docs/evolving_minds_funding_findings_20261003.md) — `docs/evolving_minds_funding_findings_20261003.md`
- [Evolving Minds H1/H2 review and paid variation calibration](../docs/evolving_minds_h1_h2_decision_20261003.md) — `docs/evolving_minds_h1_h2_decision_20261003.md`
- [Conditional H3a proposal: useful inherited source variation](../docs/evolving_minds_h3a_conditional_proposal_20261003.md) — `docs/evolving_minds_h3a_conditional_proposal_20261003.md`
- [Evolving adaptive organisms in Hypersim](../docs/evolving_minds_hypersim_implementation_plan.md) — `docs/evolving_minds_hypersim_implementation_plan.md`
- [D0 lifecycle controls and birth arbitration](../docs/evolving_minds_lifecycle_fairness_findings.md) — `docs/evolving_minds_lifecycle_fairness_findings.md`
- [Matched five-arm development pilot: verified negative calibration](../docs/evolving_minds_matched_development_findings_20261004.md) — `docs/evolving_minds_matched_development_findings_20261004.md`
- [One-shot source mutation and native birth escrow: engineering findings](../docs/evolving_minds_mutation_findings.md) — `docs/evolving_minds_mutation_findings.md`
- [Descendant yield, lifetime opportunity and conditional power](../docs/evolving_minds_opportunity_accounting_findings_20261004.md) — `docs/evolving_minds_opportunity_accounting_findings_20261004.md`
- [Paid descendant source-neighborhood calibration — 2026-10-05](../docs/evolving_minds_paid_descendant_neighborhood_findings_20261005.md) — `docs/evolving_minds_paid_descendant_neighborhood_findings_20261005.md`
- [Paid supplied-edit handoff diagnostic — 2026-10-04](../docs/evolving_minds_paid_handoff_findings_20261004.md) — `docs/evolving_minds_paid_handoff_findings_20261004.md`
- [Evolving Minds plan review](../docs/evolving_minds_plan_review_2026-10-01.md) — `docs/evolving_minds_plan_review_2026-10-01.md`
- [Retained first-request comparator: implementation and engineering findings](../docs/evolving_minds_retained_proposal_findings_20261004.md) — `docs/evolving_minds_retained_proposal_findings_20261004.md`
- [Evolving Minds live checkpoints and segmented evidence](../docs/evolving_minds_streaming_findings.md) — `docs/evolving_minds_streaming_findings.md`

## Final cumulative evolution

- [Cumulative campaign adapter and capacity benchmark — 2026-10-10](../docs/cumulative_campaign_adapter_findings_20261010.md) — `docs/cumulative_campaign_adapter_findings_20261010.md`
- [Durable campaign controller rehearsal — 2026-10-10](../docs/cumulative_campaign_controller_findings_20261010.md) — `docs/cumulative_campaign_controller_findings_20261010.md`
- [Complete campaign measurement wiring — 2026-10-10](../docs/cumulative_campaign_measurement_findings_20261010.md) — `docs/cumulative_campaign_measurement_findings_20261010.md`
- [Campaign sizing and prospective acceptance — 2026-10-09](../docs/cumulative_campaign_planning_findings_20261009.md) — `docs/cumulative_campaign_planning_findings_20261009.md`
- [Durable journal and continuation readiness — 2026-10-09](../docs/cumulative_durable_journal_readiness_findings_20261009.md) — `docs/cumulative_durable_journal_readiness_findings_20261009.md`
- [Cumulative editor regulation readiness — 2026-10-09](../docs/cumulative_editor_regulation_readiness_findings_20261009.md) — `docs/cumulative_editor_regulation_readiness_findings_20261009.md`
- [Executor/controller integration — 2026-10-11](../docs/cumulative_executor_controller_findings_20261011.md) — `docs/cumulative_executor_controller_findings_20261011.md`
- [Cumulative evolution: lineage and intervention measurement](../docs/cumulative_lineage_intervention_readiness_findings_20261009.md) — `docs/cumulative_lineage_intervention_readiness_findings_20261009.md`
- [Live development controller integration — 2026-10-10](../docs/cumulative_live_controller_findings_20261010.md) — `docs/cumulative_live_controller_findings_20261010.md`
- [Cumulative evolution: first integrated readiness fixture](../docs/cumulative_readiness_findings_20261009.md) — `docs/cumulative_readiness_findings_20261009.md`
- [Regulated shared-world and versioned-lineage readiness — 2026-10-09](../docs/cumulative_regulated_world_lineage_readiness_findings_20261009.md) — `docs/cumulative_regulated_world_lineage_readiness_findings_20261009.md`
- [Frozen scientific admission checks — 2026-10-11](../docs/cumulative_scientific_admission_findings_20261011.md) — `docs/cumulative_scientific_admission_findings_20261011.md`
- [Scientific execution bindings — 2026-10-11](../docs/cumulative_scientific_binding_findings_20261011.md) — `docs/cumulative_scientific_binding_findings_20261011.md`
- [Fixed cumulative-evolution campaign — 2026-10-13](../docs/cumulative_scientific_campaign_findings_20261013.md) — `docs/cumulative_scientific_campaign_findings_20261013.md`
- [Native executor integration — 2026-10-11](../docs/cumulative_scientific_executor_findings_20261011.md) — `docs/cumulative_scientific_executor_findings_20261011.md`
- [Scientific campaign launch review — 2026-10-11](../docs/cumulative_scientific_launch_review_20261011.md) — `docs/cumulative_scientific_launch_review_20261011.md`
- [Cumulative evolution: shared-population readiness continuation](../docs/cumulative_shared_population_readiness_findings_20261009.md) — `docs/cumulative_shared_population_readiness_findings_20261009.md`
- [Bounded evidence spool integration — 2026-10-10](../docs/cumulative_spool_integration_findings_20261010.md) — `docs/cumulative_spool_integration_findings_20261010.md`
- [Cumulative evidence storage validation — 2026-10-10](../docs/cumulative_storage_validation_findings_20261010.md) — `docs/cumulative_storage_validation_findings_20261010.md`
- [A final bounded test of cumulative evolution in a self-modifying hypergraph](../docs/final_cumulative_complexity_experiment_proposal_20261009.md) — `docs/final_cumulative_complexity_experiment_proposal_20261009.md`

## Machine-profile companion contracts

- [Evolving Minds causal controls v1](../experiments/evolving_minds/contracts/causal_controls_v1.md) — `experiments/evolving_minds/contracts/causal_controls_v1.md`
- [Authored resource-choice engineering contract v1](../experiments/evolving_minds/contracts/choice_v1.md) — `experiments/evolving_minds/contracts/choice_v1.md`
- [D0 parallel ecology admission v1](../experiments/evolving_minds/contracts/d0_admission_v1.md) — `experiments/evolving_minds/contracts/d0_admission_v1.md`
- [EM-16: descendant behavior and inherited-class propagation v1](../experiments/evolving_minds/contracts/d0_descendant_behavior_propagation_v1.md) — `experiments/evolving_minds/contracts/d0_descendant_behavior_propagation_v1.md`
- [Paid handoff descendant route diagnosis v1](../experiments/evolving_minds/contracts/d0_descendant_route_diagnosis_v1.md) — `experiments/evolving_minds/contracts/d0_descendant_route_diagnosis_v1.md`
- [D0 development v1 engineering contract](../experiments/evolving_minds/contracts/d0_development_v1.md) — `experiments/evolving_minds/contracts/d0_development_v1.md`
- [Matched-pilot failure decomposition and shared replay audit v1](../experiments/evolving_minds/contracts/d0_failure_audit_optimization_v1.md) — `experiments/evolving_minds/contracts/d0_failure_audit_optimization_v1.md`
- [D0 faithful family v1: prospective engineering contract](../experiments/evolving_minds/contracts/d0_family_v1.md) — `experiments/evolving_minds/contracts/d0_family_v1.md`
- [Funded semantic admission and live continuation v1](../experiments/evolving_minds/contracts/d0_funded_admission_v1.md) — `experiments/evolving_minds/contracts/d0_funded_admission_v1.md`
- [Parent-funded offspring reserve calibration v1](../experiments/evolving_minds/contracts/d0_funding_v1.md) — `experiments/evolving_minds/contracts/d0_funding_v1.md`
- [D0 full lifecycle communication controls and birth arbitration v1](../experiments/evolving_minds/contracts/d0_lifecycle_fairness_v1.md) — `experiments/evolving_minds/contracts/d0_lifecycle_fairness_v1.md`
- [Matched five-arm development pilot v1](../experiments/evolving_minds/contracts/d0_matched_development_v1.md) — `experiments/evolving_minds/contracts/d0_matched_development_v1.md`
- [D0 one-shot source mutation and birth escrow v1](../experiments/evolving_minds/contracts/d0_mutation_v1.md) — `experiments/evolving_minds/contracts/d0_mutation_v1.md`
- [Paid inherited source-neighborhood calibration v1](../experiments/evolving_minds/contracts/d0_neighborhood_v1.md) — `experiments/evolving_minds/contracts/d0_neighborhood_v1.md`
- [Descendant endpoint and opportunity accounting v1](../experiments/evolving_minds/contracts/d0_opportunity_accounting_v1.md) — `experiments/evolving_minds/contracts/d0_opportunity_accounting_v1.md`
- [Paid descendant source neighborhood v1](../experiments/evolving_minds/contracts/d0_paid_descendant_neighborhood_v1.md) — `experiments/evolving_minds/contracts/d0_paid_descendant_neighborhood_v1.md`
- [Paid handoff audit continuation v1](../experiments/evolving_minds/contracts/d0_paid_handoff_audit_continuation_v1.md) — `experiments/evolving_minds/contracts/d0_paid_handoff_audit_continuation_v1.md`
- [Paid supplied-edit handoff diagnostic v1](../experiments/evolving_minds/contracts/d0_paid_handoff_v1.md) — `experiments/evolving_minds/contracts/d0_paid_handoff_v1.md`
- [Retained first-request comparator v1](../experiments/evolving_minds/contracts/d0_retained_proposal_v1.md) — `experiments/evolving_minds/contracts/d0_retained_proposal_v1.md`
- [Live-state checkpoint and segmented evidence v1](../experiments/evolving_minds/contracts/d0_streaming_v1.md) — `experiments/evolving_minds/contracts/d0_streaming_v1.md`
- [Evolving Minds H0/H1 engineering fixture v1](../experiments/evolving_minds/contracts/engineering_v1.md) — `experiments/evolving_minds/contracts/engineering_v1.md`

## Project specifications

- [Hypersim](../README.md) — `README.md`
- [hg-alife — Implementation Plan (Build Spec)](../hg_alife_implementation_plan.md) — `hg_alife_implementation_plan.md`
- [hg-alife — Scope & Architecture](../hg_alife_scope_and_architecture.md) — `hg_alife_scope_and_architecture.md`
- [Hypersim Implementation Plan v4](../hypersim_implementation_plan_v4.md) — `hypersim_implementation_plan_v4.md`
- [Hypersim Substrate Characterization Report](../hypersim_substrate_characterization_report.md) — `hypersim_substrate_characterization_report.md`

## Self-modification

- [selfmod v3 findings — VF2 port + enriched-vocabulary selection test](../docs/selfmod_v3_findings.md) — `docs/selfmod_v3_findings.md`
- [selfmod v3 — Phase 0 reconnaissance](../docs/selfmod_v3_recon.md) — `docs/selfmod_v3_recon.md`
- [selfmod v4 — calibration record](../docs/selfmod_v4_calibration.md) — `docs/selfmod_v4_calibration.md`
- [selfmod v4 findings — flux economy, variation-rate sweep, persistence vs novelty](../docs/selfmod_v4_findings.md) — `docs/selfmod_v4_findings.md`
- [selfmod v10 — re-baseline under the corrected instrument](../experiments/lk/docs/selfmod_v10_findings.md) — `experiments/lk/docs/selfmod_v10_findings.md`
- [Selfmod v11 — single-mutation spectrum findings](../experiments/lk/docs/selfmod_v11_findings.md) — `experiments/lk/docs/selfmod_v11_findings.md`
- [Selfmod v11 — single-mutation spectrum](../experiments/lk/docs/selfmod_v11_spectrum_preregistration.md) — `experiments/lk/docs/selfmod_v11_spectrum_preregistration.md`
- [Selfmod v12 — controlled child establishment](../experiments/lk/docs/selfmod_v12_birth_findings.md) — `experiments/lk/docs/selfmod_v12_birth_findings.md`
- [Selfmod v12 — controlled R1 child establishment](../experiments/lk/docs/selfmod_v12_birth_preregistration.md) — `experiments/lk/docs/selfmod_v12_birth_preregistration.md`
- [Selfmod v13 — subsequent fission limits child establishment](../experiments/lk/docs/selfmod_v13_turnover_findings.md) — `experiments/lk/docs/selfmod_v13_turnover_findings.md`
- [Selfmod v13 — subsequent fission and competitive establishment](../experiments/lk/docs/selfmod_v13_turnover_preregistration.md) — `experiments/lk/docs/selfmod_v13_turnover_preregistration.md`
- [Selfmod v14 — funded parent replacement at eight copies](../experiments/lk/docs/selfmod_v14_bounded_findings.md) — `experiments/lk/docs/selfmod_v14_bounded_findings.md`
- [Selfmod v14 — bounded funded reproduction](../experiments/lk/docs/selfmod_v14_bounded_preregistration.md) — `experiments/lk/docs/selfmod_v14_bounded_preregistration.md`
- [Selfmod v15 — fresh-seed bounded reproduction replication](../experiments/lk/docs/selfmod_v15_replication_findings.md) — `experiments/lk/docs/selfmod_v15_replication_findings.md`
- [Selfmod v15 — fresh-seed replication of bounded funded reproduction](../experiments/lk/docs/selfmod_v15_replication_preregistration.md) — `experiments/lk/docs/selfmod_v15_replication_preregistration.md`
- [Selfmod v16 — extended financing diagnostic](../experiments/lk/docs/selfmod_v16_financing_findings.md) — `experiments/lk/docs/selfmod_v16_financing_findings.md`
- [Selfmod v16 — continuous financing diagnostic](../experiments/lk/docs/selfmod_v16_financing_preregistration.md) — `experiments/lk/docs/selfmod_v16_financing_preregistration.md`
- [Selfmod v17: fresh prospective durability test](../experiments/lk/docs/selfmod_v17_durability_findings.md) — `experiments/lk/docs/selfmod_v17_durability_findings.md`
- [Selfmod v17 — prospective funded-reproduction durability](../experiments/lk/docs/selfmod_v17_durability_preregistration.md) — `experiments/lk/docs/selfmod_v17_durability_preregistration.md`
- [Selfmod v18 — named variants under funded parent replacement](../experiments/lk/docs/selfmod_v18_variants_findings.md) — `experiments/lk/docs/selfmod_v18_variants_findings.md`
- [Selfmod v18 — prospective named R1 variant comparison](../experiments/lk/docs/selfmod_v18_variants_preregistration.md) — `experiments/lk/docs/selfmod_v18_variants_preregistration.md`
- [Selfmod v19 — interrupted second-edit assay](../experiments/lk/docs/selfmod_v19_second_edit_findings.md) — `experiments/lk/docs/selfmod_v19_second_edit_findings.md`
- [Selfmod v19 — one further edit in an established prune family](../experiments/lk/docs/selfmod_v19_second_edit_preregistration.md) — `experiments/lk/docs/selfmod_v19_second_edit_preregistration.md`
- [Selfmod v19b — a further edit survives funded reproduction](../experiments/lk/docs/selfmod_v19b_second_edit_findings.md) — `experiments/lk/docs/selfmod_v19b_second_edit_findings.md`
- [Selfmod v19b — semantic constraint removal at a second funded birth](../experiments/lk/docs/selfmod_v19b_second_edit_preregistration.md) — `experiments/lk/docs/selfmod_v19b_second_edit_preregistration.md`
- [Selfmod v20 — the remaining label constraint marks a durability boundary](../experiments/lk/docs/selfmod_v20_third_edit_findings.md) — `experiments/lk/docs/selfmod_v20_third_edit_findings.md`
- [Selfmod v20 — third edited birth on the prune platform](../experiments/lk/docs/selfmod_v20_third_edit_preregistration.md) — `experiments/lk/docs/selfmod_v20_third_edit_preregistration.md`
- [Selfmod v21 — a reversible third changed birth remains durable](../experiments/lk/docs/selfmod_v21_reversible_edit_findings.md) — `experiments/lk/docs/selfmod_v21_reversible_edit_findings.md`
- [Selfmod v21 — reversible third edit while retaining the label1 role](../experiments/lk/docs/selfmod_v21_reversible_edit_preregistration.md) — `experiments/lk/docs/selfmod_v21_reversible_edit_preregistration.md`
- [Selfmod v22 — reversible third families maintain function to 50,000 steps](../experiments/lk/docs/selfmod_v22_maintenance_findings.md) — `experiments/lk/docs/selfmod_v22_maintenance_findings.md`
- [Selfmod v22 — 50,000-step maintenance of the two label1-retaining states](../experiments/lk/docs/selfmod_v22_maintenance_preregistration.md) — `experiments/lk/docs/selfmod_v22_maintenance_preregistration.md`
- [Selfmod v23 — fresh serial panel maintains restored families for 50,000 steps](../experiments/lk/docs/selfmod_v23_fresh50k_findings.md) — `experiments/lk/docs/selfmod_v23_fresh50k_findings.md`
- [Selfmod v23 — fresh matched 50,000-step serial panel](../experiments/lk/docs/selfmod_v23_fresh50k_preregistration.md) — `experiments/lk/docs/selfmod_v23_fresh50k_preregistration.md`
- [V23 reached-pair replay and financing supplement](../experiments/lk/docs/selfmod_v23_reached_replay_findings.md) — `experiments/lk/docs/selfmod_v23_reached_replay_findings.md`
- [V23 reached-pair replay and financing supplement](../experiments/lk/docs/selfmod_v23_reached_replay_preregistration.md) — `experiments/lk/docs/selfmod_v23_reached_replay_preregistration.md`
- [V24 fixed-checkpoint conflict attribution findings](../experiments/lk/docs/selfmod_v24_conflict_attribution_findings.md) — `experiments/lk/docs/selfmod_v24_conflict_attribution_findings.md`
- [V24 fixed-checkpoint conflict attribution engineering assay](../experiments/lk/docs/selfmod_v24_conflict_attribution_preregistration.md) — `experiments/lk/docs/selfmod_v24_conflict_attribution_preregistration.md`
- [selfmod v3 findings — VF2 port + enriched-vocabulary selection test](../experiments/lk/docs/selfmod_v3_findings.md) — `experiments/lk/docs/selfmod_v3_findings.md`
- [selfmod v3 — Phase 0 reconnaissance](../experiments/lk/docs/selfmod_v3_recon.md) — `experiments/lk/docs/selfmod_v3_recon.md`
- [selfmod v4 — calibration record](../experiments/lk/docs/selfmod_v4_calibration.md) — `experiments/lk/docs/selfmod_v4_calibration.md`
- [selfmod v4 findings — flux economy, variation-rate sweep, persistence vs novelty](../experiments/lk/docs/selfmod_v4_findings.md) — `experiments/lk/docs/selfmod_v4_findings.md`
- [selfmod v5 — calibration record](../experiments/lk/docs/selfmod_v5_calibration.md) — `experiments/lk/docs/selfmod_v5_calibration.md`
- [selfmod v5 findings — fitness gradient, and why efficiency still cannot be selected](../experiments/lk/docs/selfmod_v5_findings.md) — `experiments/lk/docs/selfmod_v5_findings.md`
- [selfmod v6 — calibration record](../experiments/lk/docs/selfmod_v6_calibration.md) — `experiments/lk/docs/selfmod_v6_calibration.md`
- [selfmod v6 — findings](../experiments/lk/docs/selfmod_v6_findings.md) — `experiments/lk/docs/selfmod_v6_findings.md`
- [selfmod v7 — the tie-break fix](../experiments/lk/docs/selfmod_v7_findings.md) — `experiments/lk/docs/selfmod_v7_findings.md`
- [selfmod v8 — the calibrated null and the jitter hybrid](../experiments/lk/docs/selfmod_v8_findings.md) — `experiments/lk/docs/selfmod_v8_findings.md`
- [selfmod v9 — decayed merit: entry with defense](../experiments/lk/docs/selfmod_v9_findings.md) — `experiments/lk/docs/selfmod_v9_findings.md`

## Validation

- [H2 — the replication-vs-duration confound, resolved: it was DURATION](../docs/validation_h2_confound_split_findings.md) — `docs/validation_h2_confound_split_findings.md`
- [V-track — findings: pre-experiment validation of the Experiment-5/6 stack](../docs/validation_v1_findings.md) — `docs/validation_v1_findings.md`
- [hypersim — Pre-Experiment Validation Plan (V-track)](../docs/validation_v1_plan.md) — `docs/validation_v1_plan.md`
- [F1 — WL depth-selection pre-registration](../docs/validation_v2_f1_depth_preregistration.md) — `docs/validation_v2_f1_depth_preregistration.md`
- [F-track — findings: the V1 negative was a resolution artifact](../docs/validation_v2_findings.md) — `docs/validation_v2_findings.md`
- [hypersim — V2 / F-track: fix plan following the V1 validation findings](../docs/validation_v2_fix_plan.md) — `docs/validation_v2_fix_plan.md`
- [Multi-seed F2 replication — pre-registration](../docs/validation_v2_multiseed_preregistration.md) — `docs/validation_v2_multiseed_preregistration.md`
- [F6 — pre-registrations for the record](../docs/validation_v2_preregistrations.md) — `docs/validation_v2_preregistrations.md`
- [V3 / G-track — findings: the sterile-token control](../docs/validation_v3_sterile_token_findings.md) — `docs/validation_v3_sterile_token_findings.md`
- [V3 / G-track — G1 structural results + G2 pre-registration (committed BEFORE any CE)](../docs/validation_v3_sterile_token_preregistration.md) — `docs/validation_v3_sterile_token_preregistration.md`
- [V4 / H-track — H0 findings: the churn rung is a dead end for CE; hand off to H4](../docs/validation_v4_h0_findings.md) — `docs/validation_v4_h0_findings.md`
- [V4 / H-track — H0.1: the CE-stationarity criterion (committed BEFORE measurement)](../docs/validation_v4_h0_stationarity_criterion.md) — `docs/validation_v4_h0_stationarity_criterion.md`
- [V5 / J-track — J0.1: the site-ablation definition (committed BEFORE running J0.2)](../docs/validation_v5_j0_ablation_definition.md) — `docs/validation_v5_j0_ablation_definition.md`
- [V5 / J-track — J0 findings: the carrier resolution was substrate-borrowed → J2](../docs/validation_v5_j0_findings.md) — `docs/validation_v5_j0_findings.md`
- [V5 / J-track — J2 findings: a churn-stable instrument exists (metabolic, not geometric)](../docs/validation_v5_j2_findings.md) — `docs/validation_v5_j2_findings.md`
- [V5 / J-track — J2.1: the variable-persistence criterion (committed BEFORE J2.2)](../docs/validation_v5_j2_variable_persistence_criterion.md) — `docs/validation_v5_j2_variable_persistence_criterion.md`
- [V6 / K1 — turnover-vs-persistence criterion for arm B (PRE-COMMITTED)](../docs/validation_v6_k1_turnover_criterion.md) — `docs/validation_v6_k1_turnover_criterion.md`
- [V6 / K2 — multi-seed arm B: pre-registration](../docs/validation_v6_k2_multiseed_preregistration.md) — `docs/validation_v6_k2_multiseed_preregistration.md`
- [V6 / K3 — isolating the presence-specific signal from the always-on metabolic one](../docs/validation_v6_k3_signal_isolation_preregistration.md) — `docs/validation_v6_k3_signal_isolation_preregistration.md`
- [V6 / K-track — hardening the verdict: it is sustained PRESENCE, it holds across seeds,](../docs/validation_v6_k_track_findings.md) — `docs/validation_v6_k_track_findings.md`
- [V7 / L1.1 — how a *stationary* carrier is sustained (mechanism committed before build)](../docs/validation_v7_l1_sustain_mechanism.md) — `docs/validation_v7_l1_sustain_mechanism.md`
- [V7 / L2.1 — what an instrument must satisfy to measure the presence-vs-motility question](../docs/validation_v7_l2_instrument_criteria.md) — `docs/validation_v7_l2_instrument_criteria.md`
- [V7 / L-track — presence versus motility: the question is outside the instrument's domain,](../docs/validation_v7_l_track_findings.md) — `docs/validation_v7_l_track_findings.md`
- [V8 / M2 — arm-C truncated-window reading, and the null-spread guard (PRE-COMMITTED)](../docs/validation_v8_m2_preregistration.md) — `docs/validation_v8_m2_preregistration.md`
- [V8 / M3.1 — the inert-carrier control and the expectation table (COMMITTED BEFORE ANY INSTRUMENT WORK)](../docs/validation_v8_m3_inert_carrier_control.md) — `docs/validation_v8_m3_inert_carrier_control.md`
- [V8 / M4.2 — multi-seed arm B′: pre-registration](../docs/validation_v8_m4_2_preregistration.md) — `docs/validation_v8_m4_2_preregistration.md`
- [V8 / M4.3 — guard audit: which V-track criteria have a firing test?](../docs/validation_v8_m4_3_guard_audit.md) — `docs/validation_v8_m4_3_guard_audit.md`
- [V8 / M4.4 — powering the selection question: pre-registration](../docs/validation_v8_m4_4_preregistration.md) — `docs/validation_v8_m4_4_preregistration.md`
- [V8 / M-track — the churn substrate is measurable, the empirical leg survives, and the](../docs/validation_v8_m_track_findings.md) — `docs/validation_v8_m_track_findings.md`
- [V-track closeout / C1 — the resolution diagnostic: channel or instrument?](../docs/validation_v9_c1_resolution_preregistration.md) — `docs/validation_v9_c1_resolution_preregistration.md`
- [V-track closeout / C4 — confirmatory arm-C sweep, and the last guard gap](../docs/validation_v9_c4_preregistration.md) — `docs/validation_v9_c4_preregistration.md`
- [V9 / N0 — the burn-in criterion (COMMITTED BEFORE ANY CHURN MEASUREMENT IN THIS TRACK)](../docs/validation_v9_n0_burnin_criterion.md) — `docs/validation_v9_n0_burnin_criterion.md`
- [V9 / N0 v2 — the burn-in criterion, recalibrated against the trusted baseline](../docs/validation_v9_n0_burnin_criterion_v2.md) — `docs/validation_v9_n0_burnin_criterion_v2.md`
- [V9 / N2.1 — Phase 2 expectation table, with honest provenance](../docs/validation_v9_n2_expectation_table.md) — `docs/validation_v9_n2_expectation_table.md`
- [V9 / N3 — scope discipline: what the metabolic re-run cannot mean](../docs/validation_v9_n3_scope_discipline.md) — `docs/validation_v9_n3_scope_discipline.md`
- [V9 / N-track — the metabolic instrument is validated, and it answers a *different* question](../docs/validation_v9_n_track_findings.md) — `docs/validation_v9_n_track_findings.md`
- [V9 / Phase 1 — validating the metabolic instrument: pre-registration](../docs/validation_v9_phase1_preregistration.md) — `docs/validation_v9_phase1_preregistration.md`
- [V-track closeout — the coarseness belongs to the CHANNEL. The V-track is complete.](../docs/validation_vtrack_closeout_findings.md) — `docs/validation_vtrack_closeout_findings.md`
- [H2 — the replication-vs-duration confound, resolved: it was DURATION](../experiments/lk/docs/validation_h2_confound_split_findings.md) — `experiments/lk/docs/validation_h2_confound_split_findings.md`
- [V-track — findings: pre-experiment validation of the Experiment-5/6 stack](../experiments/lk/docs/validation_v1_findings.md) — `experiments/lk/docs/validation_v1_findings.md`
- [hypersim — Pre-Experiment Validation Plan (V-track)](../experiments/lk/docs/validation_v1_plan.md) — `experiments/lk/docs/validation_v1_plan.md`
- [F1 — WL depth-selection pre-registration](../experiments/lk/docs/validation_v2_f1_depth_preregistration.md) — `experiments/lk/docs/validation_v2_f1_depth_preregistration.md`
- [F-track — findings: the V1 negative was a resolution artifact](../experiments/lk/docs/validation_v2_findings.md) — `experiments/lk/docs/validation_v2_findings.md`
- [hypersim — V2 / F-track: fix plan following the V1 validation findings](../experiments/lk/docs/validation_v2_fix_plan.md) — `experiments/lk/docs/validation_v2_fix_plan.md`
- [Multi-seed F2 replication — pre-registration](../experiments/lk/docs/validation_v2_multiseed_preregistration.md) — `experiments/lk/docs/validation_v2_multiseed_preregistration.md`
- [F6 — pre-registrations for the record](../experiments/lk/docs/validation_v2_preregistrations.md) — `experiments/lk/docs/validation_v2_preregistrations.md`
- [V3 / G-track — findings: the sterile-token control](../experiments/lk/docs/validation_v3_sterile_token_findings.md) — `experiments/lk/docs/validation_v3_sterile_token_findings.md`
- [V3 / G-track — G1 structural results + G2 pre-registration (committed BEFORE any CE)](../experiments/lk/docs/validation_v3_sterile_token_preregistration.md) — `experiments/lk/docs/validation_v3_sterile_token_preregistration.md`
- [V4 / H-track — H0 findings: the churn rung is a dead end for CE; hand off to H4](../experiments/lk/docs/validation_v4_h0_findings.md) — `experiments/lk/docs/validation_v4_h0_findings.md`
- [V4 / H-track — H0.1: the CE-stationarity criterion (committed BEFORE measurement)](../experiments/lk/docs/validation_v4_h0_stationarity_criterion.md) — `experiments/lk/docs/validation_v4_h0_stationarity_criterion.md`
- [V5 / J-track — J0.1: the site-ablation definition (committed BEFORE running J0.2)](../experiments/lk/docs/validation_v5_j0_ablation_definition.md) — `experiments/lk/docs/validation_v5_j0_ablation_definition.md`
- [V5 / J-track — J0 findings: the carrier resolution was substrate-borrowed → J2](../experiments/lk/docs/validation_v5_j0_findings.md) — `experiments/lk/docs/validation_v5_j0_findings.md`
- [V5 / J-track — J2 findings: a churn-stable instrument exists (metabolic, not geometric)](../experiments/lk/docs/validation_v5_j2_findings.md) — `experiments/lk/docs/validation_v5_j2_findings.md`
- [V5 / J-track — J2.1: the variable-persistence criterion (committed BEFORE J2.2)](../experiments/lk/docs/validation_v5_j2_variable_persistence_criterion.md) — `experiments/lk/docs/validation_v5_j2_variable_persistence_criterion.md`
- [V6 / K1 — turnover-vs-persistence criterion for arm B (PRE-COMMITTED)](../experiments/lk/docs/validation_v6_k1_turnover_criterion.md) — `experiments/lk/docs/validation_v6_k1_turnover_criterion.md`
- [V6 / K2 — multi-seed arm B: pre-registration](../experiments/lk/docs/validation_v6_k2_multiseed_preregistration.md) — `experiments/lk/docs/validation_v6_k2_multiseed_preregistration.md`
- [V6 / K3 — isolating the presence-specific signal from the always-on metabolic one](../experiments/lk/docs/validation_v6_k3_signal_isolation_preregistration.md) — `experiments/lk/docs/validation_v6_k3_signal_isolation_preregistration.md`
- [V6 / K-track — hardening the verdict: it is sustained PRESENCE, it holds across seeds,](../experiments/lk/docs/validation_v6_k_track_findings.md) — `experiments/lk/docs/validation_v6_k_track_findings.md`
- [V7 / L1.1 — how a *stationary* carrier is sustained (mechanism committed before build)](../experiments/lk/docs/validation_v7_l1_sustain_mechanism.md) — `experiments/lk/docs/validation_v7_l1_sustain_mechanism.md`
- [V7 / L2.1 — what an instrument must satisfy to measure the presence-vs-motility question](../experiments/lk/docs/validation_v7_l2_instrument_criteria.md) — `experiments/lk/docs/validation_v7_l2_instrument_criteria.md`
- [V7 / L-track — presence versus motility: the question is outside the instrument's domain,](../experiments/lk/docs/validation_v7_l_track_findings.md) — `experiments/lk/docs/validation_v7_l_track_findings.md`
- [V8 / M2 — arm-C truncated-window reading, and the null-spread guard (PRE-COMMITTED)](../experiments/lk/docs/validation_v8_m2_preregistration.md) — `experiments/lk/docs/validation_v8_m2_preregistration.md`
- [V8 / M3.1 — the inert-carrier control and the expectation table (COMMITTED BEFORE ANY INSTRUMENT WORK)](../experiments/lk/docs/validation_v8_m3_inert_carrier_control.md) — `experiments/lk/docs/validation_v8_m3_inert_carrier_control.md`
- [V8 / M4.2 — multi-seed arm B′: pre-registration](../experiments/lk/docs/validation_v8_m4_2_preregistration.md) — `experiments/lk/docs/validation_v8_m4_2_preregistration.md`
- [V8 / M4.3 — guard audit: which V-track criteria have a firing test?](../experiments/lk/docs/validation_v8_m4_3_guard_audit.md) — `experiments/lk/docs/validation_v8_m4_3_guard_audit.md`
- [V8 / M4.4 — powering the selection question: pre-registration](../experiments/lk/docs/validation_v8_m4_4_preregistration.md) — `experiments/lk/docs/validation_v8_m4_4_preregistration.md`
- [V8 / M-track — the churn substrate is measurable, the empirical leg survives, and the](../experiments/lk/docs/validation_v8_m_track_findings.md) — `experiments/lk/docs/validation_v8_m_track_findings.md`
- [V-track closeout / C1 — the resolution diagnostic: channel or instrument?](../experiments/lk/docs/validation_v9_c1_resolution_preregistration.md) — `experiments/lk/docs/validation_v9_c1_resolution_preregistration.md`
- [V-track closeout / C4 — confirmatory arm-C sweep, and the last guard gap](../experiments/lk/docs/validation_v9_c4_preregistration.md) — `experiments/lk/docs/validation_v9_c4_preregistration.md`
- [V9 / N0 — the burn-in criterion (COMMITTED BEFORE ANY CHURN MEASUREMENT IN THIS TRACK)](../experiments/lk/docs/validation_v9_n0_burnin_criterion.md) — `experiments/lk/docs/validation_v9_n0_burnin_criterion.md`
- [V9 / N0 v2 — the burn-in criterion, recalibrated against the trusted baseline](../experiments/lk/docs/validation_v9_n0_burnin_criterion_v2.md) — `experiments/lk/docs/validation_v9_n0_burnin_criterion_v2.md`
- [V9 / N2.1 — Phase 2 expectation table, with honest provenance](../experiments/lk/docs/validation_v9_n2_expectation_table.md) — `experiments/lk/docs/validation_v9_n2_expectation_table.md`
- [V9 / N3 — scope discipline: what the metabolic re-run cannot mean](../experiments/lk/docs/validation_v9_n3_scope_discipline.md) — `experiments/lk/docs/validation_v9_n3_scope_discipline.md`
- [V9 / N-track — the metabolic instrument is validated, and it answers a *different* question](../experiments/lk/docs/validation_v9_n_track_findings.md) — `experiments/lk/docs/validation_v9_n_track_findings.md`
- [V9 / Phase 1 — validating the metabolic instrument: pre-registration](../experiments/lk/docs/validation_v9_phase1_preregistration.md) — `experiments/lk/docs/validation_v9_phase1_preregistration.md`
- [V-track closeout — the coarseness belongs to the CHANNEL. The V-track is complete.](../experiments/lk/docs/validation_vtrack_closeout_findings.md) — `experiments/lk/docs/validation_vtrack_closeout_findings.md`

## Immediate evidence packages

Direct lifecycle fields are listed when present; all inspected metadata, including nested records and audit fields, is in the JSON inventory.

| Package | Direct lifecycle records | Inspected metadata files in subtree |
|---|---|---:|
| [output/binary_edge_demo_42_20260707T114454](../output/binary_edge_demo_42_20260707T114454) | manifest.json: Completed | 1 |
| [output/binary_edge_demo_42_20260707T223631](../output/binary_edge_demo_42_20260707T223631) | manifest.json: Completed | 1 |
| [output/binary_edge_demo_42_20260707T231413](../output/binary_edge_demo_42_20260707T231413) | manifest.json: Completed | 1 |
| [output/constructive_chemistry_cc0_20261005](../output/constructive_chemistry_cc0_20261005) | manifest.json: Completed; run_status.json: no top-level status/state | 2 |
| [output/constructive_chemistry_cc1_20261005](../output/constructive_chemistry_cc1_20261005) | manifest.json: Failed | 2 |
| [output/constructive_chemistry_cc1_completion_20261005](../output/constructive_chemistry_cc1_completion_20261005) | manifest.json: Completed | 1 |
| [output/constructive_chemistry_cc1_development_20261005](../output/constructive_chemistry_cc1_development_20261005) | No direct lifecycle record found | 1 |
| [output/constructive_chemistry_cc1_verified_20261005](../output/constructive_chemistry_cc1_verified_20261005) | manifest.json: Completed | 2 |
| [output/cumulative_campaign_adapter_readiness_20261010](../output/cumulative_campaign_adapter_readiness_20261010) | manifest.json: complete | 2 |
| [output/cumulative_campaign_controller_20261010](../output/cumulative_campaign_controller_20261010) | manifest.json: complete | 5 |
| [output/cumulative_campaign_measurement_20261010](../output/cumulative_campaign_measurement_20261010) | manifest.json: failed | 1 |
| [output/cumulative_campaign_measurement_v2_20261010](../output/cumulative_campaign_measurement_v2_20261010) | manifest.json: complete | 3 |
| [output/cumulative_campaign_planning_20261009](../output/cumulative_campaign_planning_20261009) | manifest.json: failed | 1 |
| [output/cumulative_campaign_planning_v2_20261009](../output/cumulative_campaign_planning_v2_20261009) | manifest.json: complete | 2 |
| [output/cumulative_durable_journal_readiness_20261009](../output/cumulative_durable_journal_readiness_20261009) | manifest.json: failed | 1 |
| [output/cumulative_durable_journal_readiness_v2_20261009](../output/cumulative_durable_journal_readiness_v2_20261009) | manifest.json: failed | 1 |
| [output/cumulative_durable_journal_readiness_v3_20261009](../output/cumulative_durable_journal_readiness_v3_20261009) | manifest.json: complete | 2 |
| [output/cumulative_editor_regulation_readiness_20261009](../output/cumulative_editor_regulation_readiness_20261009) | manifest.json: complete | 2 |
| [output/cumulative_executor_controller_20261011](../output/cumulative_executor_controller_20261011) | manifest.json: complete | 9 |
| [output/cumulative_lineage_development_unfunded_20261009](../output/cumulative_lineage_development_unfunded_20261009) | manifest.json: failed | 1 |
| [output/cumulative_lineage_intervention_readiness_20261009](../output/cumulative_lineage_intervention_readiness_20261009) | manifest.json: complete | 4 |
| [output/cumulative_live_controller_20261010](../output/cumulative_live_controller_20261010) | manifest.json: complete | 5 |
| [output/cumulative_readiness_20261009](../output/cumulative_readiness_20261009) | manifest.json: complete | 2 |
| [output/cumulative_regulated_world_lineage_readiness_20261009](../output/cumulative_regulated_world_lineage_readiness_20261009) | manifest.json: complete | 2 |
| [output/cumulative_scientific_admission_20261011](../output/cumulative_scientific_admission_20261011) | manifest.json: complete | 6 |
| [output/cumulative_scientific_binding_20261011](../output/cumulative_scientific_binding_20261011) | manifest.json: complete | 8 |
| [output/cumulative_scientific_campaign_20261011](../output/cumulative_scientific_campaign_20261011) | manifest.json: complete | 2 |
| [output/cumulative_scientific_executor_20261011](../output/cumulative_scientific_executor_20261011) | manifest.json: complete | 3 |
| [output/cumulative_shared_population_readiness_20261009](../output/cumulative_shared_population_readiness_20261009) | manifest.json: complete | 2 |
| [output/cumulative_spool_integration_20261010](../output/cumulative_spool_integration_20261010) | manifest.json: complete | 2 |
| [output/cumulative_storage_validation_20261010](../output/cumulative_storage_validation_20261010) | manifest.json: complete | 1 |
| [output/ere_categorical_adapter_engineering_20261001](../output/ere_categorical_adapter_engineering_20261001) | manifest.json: failed_engineering | 1 |
| [output/ere_categorical_adapter_engineering_v2_20261001](../output/ere_categorical_adapter_engineering_v2_20261001) | manifest.json: complete | 2 |
| [output/ere_categorical_comparison_bounded_auditor_20261002](../output/ere_categorical_comparison_bounded_auditor_20261002) | manifest.json: failed_engineering | 1 |
| [output/ere_categorical_comparison_bounded_auditor_v2_20261002](../output/ere_categorical_comparison_bounded_auditor_v2_20261002) | manifest.json: failed_engineering | 1 |
| [output/ere_categorical_comparison_bounded_auditor_v3_20261002](../output/ere_categorical_comparison_bounded_auditor_v3_20261002) | manifest.json: failed_engineering | 1 |
| [output/ere_categorical_comparison_bounded_auditor_v4_20261002](../output/ere_categorical_comparison_bounded_auditor_v4_20261002) | manifest.json: complete | 1 |
| [output/ere_categorical_comparison_development_20261002](../output/ere_categorical_comparison_development_20261002) | manifest.json: complete | 2 |
| [output/ere_categorical_comparison_historical_audit_20261003](../output/ere_categorical_comparison_historical_audit_20261003) | manifest.json: complete | 2 |
| [output/ere_categorical_comparison_launch_bridge_preflight_20261002](../output/ere_categorical_comparison_launch_bridge_preflight_20261002) | manifest.json: zero_work_preflight | 2 |
| [output/ere_categorical_comparison_launch_bridge_verification_20261002](../output/ere_categorical_comparison_launch_bridge_verification_20261002) | manifest.json: complete | 1 |
| [output/ere_categorical_comparison_ordered_audit_20261003](../output/ere_categorical_comparison_ordered_audit_20261003) | manifest.json: failed | 1 |
| [output/ere_categorical_comparison_ordered_audit_v2_20261003](../output/ere_categorical_comparison_ordered_audit_v2_20261003) | manifest.json: failed | 1 |
| [output/ere_categorical_comparison_preflight_20261002](../output/ere_categorical_comparison_preflight_20261002) | manifest.json: zero_work_preflight | 2 |
| [output/ere_categorical_comparison_preregistration_20261001](../output/ere_categorical_comparison_preregistration_20261001) | manifest.json: preregistered_not_implemented | 1 |
| [output/ere_categorical_comparison_scientific_20261003](../output/ere_categorical_comparison_scientific_20261003) | manifest.json: complete | 1 |
| [output/ere_categorical_feasible_development_20261003](../output/ere_categorical_feasible_development_20261003) | manifest.json: complete | 2 |
| [output/ere_categorical_feasible_development_preflight_20261003](../output/ere_categorical_feasible_development_preflight_20261003) | manifest.json: zero_work_preflight | 2 |
| [output/ere_categorical_feasible_engineering_20261003](../output/ere_categorical_feasible_engineering_20261003) | manifest.json: failed_engineering | 1 |
| [output/ere_categorical_feasible_engineering_v2_20261003](../output/ere_categorical_feasible_engineering_v2_20261003) | manifest.json: complete | 2 |
| [output/ere_checkpoint_continuation_20260926](../output/ere_checkpoint_continuation_20260926) | manifest.json: no top-level status/state; status.json: complete | 2 |
| [output/ere_closure_preserved_audit_20260929](../output/ere_closure_preserved_audit_20260929) | No direct lifecycle record found | 1 |
| [output/ere_closure_replay_20260929](../output/ere_closure_replay_20260929) | No direct lifecycle record found | 0 |
| [output/ere_coherent_auditor_optimization_20260928](../output/ere_coherent_auditor_optimization_20260928) | No direct lifecycle record found | 6 |
| [output/ere_coherent_draw_optimization_20260928](../output/ere_coherent_draw_optimization_20260928) | No direct lifecycle record found | 4 |
| [output/ere_complete_admission_preflight_20260930](../output/ere_complete_admission_preflight_20260930) | manifest.json: ready_for_bounded_execution; status.json: ready_for_bounded_execution | 3 |
| [output/ere_complete_bounded_engineering_20260930](../output/ere_complete_bounded_engineering_20260930) | manifest.json: complete; status.json: complete | 3 |
| [output/ere_complete_champion_confirmation_20261001](../output/ere_complete_champion_confirmation_20261001) | manifest.json: complete | 2 |
| [output/ere_complete_coherent_closure_full_20260929](../output/ere_complete_coherent_closure_full_20260929) | manifest.json: no top-level status/state; run_status.json: complete | 3 |
| [output/ere_complete_coherent_draw_full_20260929](../output/ere_complete_coherent_draw_full_20260929) | manifest.json: no top-level status/state; run_status.json: failed_or_unmeasured | 2 |
| [output/ere_complete_coherent_draw_resource_pilot_20260928](../output/ere_complete_coherent_draw_resource_pilot_20260928) | manifest.json: no top-level status/state; run_status.json: complete | 3 |
| [output/ere_complete_coherent_isolated_smoke_20260928](../output/ere_complete_coherent_isolated_smoke_20260928) | manifest.json: no top-level status/state; run_status.json: complete | 3 |
| [output/ere_complete_coherent_optimized_resource_pilot_20260928](../output/ere_complete_coherent_optimized_resource_pilot_20260928) | manifest.json: no top-level status/state; run_status.json: complete | 3 |
| [output/ere_complete_coherent_resource_pilot_20260928](../output/ere_complete_coherent_resource_pilot_20260928) | manifest.json: no top-level status/state; run_status.json: complete | 3 |
| [output/ere_complete_coherent_smoke_20260928](../output/ere_complete_coherent_smoke_20260928) | manifest.json: no top-level status/state; run_status.json: failed_or_unmeasured | 2 |
| [output/ere_complete_coherent_smoke_v2_20260928](../output/ere_complete_coherent_smoke_v2_20260928) | manifest.json: no top-level status/state; run_status.json: failed_or_unmeasured | 2 |
| [output/ere_complete_coherent_smoke_v3_20260928](../output/ere_complete_coherent_smoke_v3_20260928) | manifest.json: no top-level status/state; run_status.json: complete | 3 |
| [output/ere_complete_direction_assay_20261001](../output/ere_complete_direction_assay_20261001) | manifest.json: complete | 2 |
| [output/ere_complete_direction_development_20261001](../output/ere_complete_direction_development_20261001) | manifest.json: complete | 2 |
| [output/ere_complete_feasibility_training_20261001](../output/ere_complete_feasibility_training_20261001) | manifest.json: complete | 2 |
| [output/ere_complete_gates_20260926](../output/ere_complete_gates_20260926) | manifest.json: no top-level status/state | 1 |
| [output/ere_complete_gates_fresh_20260926](../output/ere_complete_gates_fresh_20260926) | manifest.json: no top-level status/state; run_status.json: no top-level status/state | 2 |
| [output/ere_complete_gates_preflight_20260926](../output/ere_complete_gates_preflight_20260926) | No direct lifecycle record found | 0 |
| [output/ere_complete_gates_preflight_v2_20260926](../output/ere_complete_gates_preflight_v2_20260926) | No direct lifecycle record found | 0 |
| [output/ere_complete_gr1_full_20260927](../output/ere_complete_gr1_full_20260927) | manifest.json: no top-level status/state; run_status.json: complete | 3 |
| [output/ere_complete_gr1_smoke_20260927](../output/ere_complete_gr1_smoke_20260927) | manifest.json: no top-level status/state; run_status.json: failed_or_unmeasured | 2 |
| [output/ere_complete_gr1_smoke_complete_20260927](../output/ere_complete_gr1_smoke_complete_20260927) | manifest.json: no top-level status/state; run_status.json: complete | 3 |
| [output/ere_complete_gr1_smoke_final_20260927](../output/ere_complete_gr1_smoke_final_20260927) | manifest.json: no top-level status/state; run_status.json: complete | 3 |
| [output/ere_complete_gr1_smoke_v2_20260927](../output/ere_complete_gr1_smoke_v2_20260927) | manifest.json: no top-level status/state; run_status.json: complete | 3 |
| [output/ere_complete_gr1_smoke_verified_20260927](../output/ere_complete_gr1_smoke_verified_20260927) | manifest.json: no top-level status/state; run_status.json: complete | 3 |
| [output/ere_complete_long_gates_full_20260927](../output/ere_complete_long_gates_full_20260927) | manifest.json: no top-level status/state; run_status.json: complete | 4 |
| [output/ere_complete_long_gates_smoke_20260926](../output/ere_complete_long_gates_smoke_20260926) | manifest.json: no top-level status/state; run_status.json: failed_or_unmeasured | 3 |
| [output/ere_complete_long_gates_smoke_final_20260926](../output/ere_complete_long_gates_smoke_final_20260926) | manifest.json: no top-level status/state; run_status.json: complete | 4 |
| [output/ere_complete_long_gates_smoke_v2_20260926](../output/ere_complete_long_gates_smoke_v2_20260926) | manifest.json: no top-level status/state; run_status.json: failed_or_unmeasured | 3 |
| [output/ere_complete_long_gates_smoke_v3_20260926](../output/ere_complete_long_gates_smoke_v3_20260926) | manifest.json: no top-level status/state; run_status.json: complete | 4 |
| [output/ere_complete_long_gates_smoke_v4_20260926](../output/ere_complete_long_gates_smoke_v4_20260926) | manifest.json: no top-level status/state; run_status.json: complete | 4 |
| [output/ere_complete_minibatch_development_20261001](../output/ere_complete_minibatch_development_20261001) | manifest.json: complete | 2 |
| [output/ere_complete_minibatch_trial_20261001](../output/ere_complete_minibatch_trial_20261001) | manifest.json: complete | 2 |
| [output/ere_complete_optimizer_signal_20260929](../output/ere_complete_optimizer_signal_20260929) | manifest.json: complete | 2 |
| [output/ere_complete_scaling_trial_20260930](../output/ere_complete_scaling_trial_20260930) | manifest.json: incomplete | 1 |
| [output/ere_complete_scaling_trial_v2_20260930](../output/ere_complete_scaling_trial_v2_20260930) | manifest.json: incomplete | 1 |
| [output/ere_complete_short_training_20260930](../output/ere_complete_short_training_20260930) | manifest.json: incomplete | 1 |
| [output/ere_complete_short_training_continuation_20260930](../output/ere_complete_short_training_continuation_20260930) | manifest.json: complete | 2 |
| [output/ere_complete_witness_20260926](../output/ere_complete_witness_20260926) | manifest.json: no top-level status/state; status.json: complete_bounded_engineering_assay | 2 |
| [output/ere_coverage_headroom_review_20261003](../output/ere_coverage_headroom_review_20261003) | manifest.json: complete | 1 |
| [output/ere_defense_allocation_20261007](../output/ere_defense_allocation_20261007) | manifest.json: failed | 3 |
| [output/ere_defense_allocation_accounting_v2_20261007](../output/ere_defense_allocation_accounting_v2_20261007) | manifest.json: failed | 2 |
| [output/ere_defense_allocation_completion_v3_20261007](../output/ere_defense_allocation_completion_v3_20261007) | manifest.json: failed | 2 |
| [output/ere_defense_allocation_completion_v4_20261007](../output/ere_defense_allocation_completion_v4_20261007) | manifest.json: complete | 2 |
| [output/ere_e11_20260925](../output/ere_e11_20260925) | No direct lifecycle record found | 0 |
| [output/ere_e11_training_20260925](../output/ere_e11_training_20260925) | manifest.json: no top-level status/state; status.json: blocked_by_prerequisite | 2 |
| [output/ere_e11_verified_20260925](../output/ere_e11_verified_20260925) | No direct lifecycle record found | 0 |
| [output/ere_e2_20260925](../output/ere_e2_20260925) | No direct lifecycle record found | 0 |
| [output/ere_e2_diagnosis_20260925](../output/ere_e2_diagnosis_20260925) | No direct lifecycle record found | 0 |
| [output/ere_e2_grammar_pilot_20260926](../output/ere_e2_grammar_pilot_20260926) | manifest.json: no top-level status/state; status.json: full_training_blocked_by_pilot | 2 |
| [output/ere_e2_maintenance_pilot_20260926](../output/ere_e2_maintenance_pilot_20260926) | manifest.json: no top-level status/state; status.json: full_training_blocked_by_maintenance_pilot | 2 |
| [output/ere_e2_training_20260925](../output/ere_e2_training_20260925) | manifest.json: no top-level status/state; status.json: blocked_by_prerequisite | 2 |
| [output/ere_e2p_20260926](../output/ere_e2p_20260926) | No direct lifecycle record found | 0 |
| [output/ere_e2p_400k50k_20260926](../output/ere_e2p_400k50k_20260926) | No direct lifecycle record found | 0 |
| [output/ere_e2p_400k50k_probe_20260926](../output/ere_e2p_400k50k_probe_20260926) | No direct lifecycle record found | 0 |
| [output/ere_e2p_gates_20260926](../output/ere_e2p_gates_20260926) | No direct lifecycle record found | 0 |
| [output/ere_e2p_probe_20260926](../output/ere_e2p_probe_20260926) | No direct lifecycle record found | 0 |
| [output/ere_edit_persistence_20260926](../output/ere_edit_persistence_20260926) | manifest.json: no top-level status/state; status.json: protected_persistence_control_passed | 2 |
| [output/ere_graph_encoder_engineering_20261005](../output/ere_graph_encoder_engineering_20261005) | manifest.json: complete | 2 |
| [output/ere_graph_encoder_exact_gradient_20261007](../output/ere_graph_encoder_exact_gradient_20261007) | manifest.json: complete | 2 |
| [output/ere_graph_encoder_exact_recheck_20261007](../output/ere_graph_encoder_exact_recheck_20261007) | No direct lifecycle record found | 0 |
| [output/ere_graph_encoder_learning_trial_20261007](../output/ere_graph_encoder_learning_trial_20261007) | manifest.json: complete | 3 |
| [output/ere_graph_encoder_native_confirmation_20261007](../output/ere_graph_encoder_native_confirmation_20261007) | manifest.json: complete | 3 |
| [output/ere_graph_encoder_native_confirmation_recheck_20261007](../output/ere_graph_encoder_native_confirmation_recheck_20261007) | No direct lifecycle record found | 0 |
| [output/ere_identity_isolation_20260928](../output/ere_identity_isolation_20260928) | manifest.json: no top-level status/state | 2 |
| [output/ere_label_separated_20260926](../output/ere_label_separated_20260926) | manifest.json: no top-level status/state; status.json: complete | 2 |
| [output/ere_meta_vocabulary_20260926](../output/ere_meta_vocabulary_20260926) | manifest.json: no top-level status/state; status.json: diagnostic_complete | 2 |
| [output/ere_native_v1_20260925](../output/ere_native_v1_20260925) | No direct lifecycle record found | 0 |
| [output/ere_native_v2_20260925](../output/ere_native_v2_20260925) | No direct lifecycle record found | 0 |
| [output/ere_repeated_damage_20260926](../output/ere_repeated_damage_20260926) | manifest.json: no top-level status/state; status.json: complete | 2 |
| [output/ere_resource_adapter_20261008](../output/ere_resource_adapter_20261008) | manifest.json: failed | 1 |
| [output/ere_resource_adapter_accounting_v2_20261008](../output/ere_resource_adapter_accounting_v2_20261008) | manifest.json: complete | 2 |
| [output/ere_resource_contrast_preflight_20261009](../output/ere_resource_contrast_preflight_20261009) | manifest.json: complete | 3 |
| [output/ere_resource_gradient_diagnostic_20261009](../output/ere_resource_gradient_diagnostic_20261009) | manifest.json: complete | 2 |
| [output/ere_resource_learning_campaign_20261008](../output/ere_resource_learning_campaign_20261008) | manifest.json: complete | 3 |
| [output/ere_resource_learning_preflight_20261008](../output/ere_resource_learning_preflight_20261008) | manifest.json: complete | 2 |
| [output/ere_resource_timed_bridge_20261008](../output/ere_resource_timed_bridge_20261008) | manifest.json: failed | 1 |
| [output/ere_resource_timed_bridge_preflight_v2_20261008](../output/ere_resource_timed_bridge_preflight_v2_20261008) | manifest.json: complete | 3 |
| [output/ere_rule_choice_adapter_development_20261004](../output/ere_rule_choice_adapter_development_20261004) | manifest.json: complete | 3 |
| [output/ere_rule_choice_learning_trial_20261004](../output/ere_rule_choice_learning_trial_20261004) | manifest.json: failed | 1 |
| [output/ere_rule_choice_learning_trial_v2_20261004](../output/ere_rule_choice_learning_trial_v2_20261004) | manifest.json: complete | 3 |
| [output/ere_rule_composition_archive_audit_v2_20261004](../output/ere_rule_composition_archive_audit_v2_20261004) | manifest.json: complete | 2 |
| [output/ere_rule_composition_development_20261004](../output/ere_rule_composition_development_20261004) | manifest.json: failed | 2 |
| [output/ere_rule_composition_learning_trial_20261004](../output/ere_rule_composition_learning_trial_20261004) | manifest.json: complete | 3 |
| [output/ere_rule_lifecycle_fixture_development_20261003](../output/ere_rule_lifecycle_fixture_development_20261003) | manifest.json: complete | 3 |
| [output/ere_rule_maintenance_closure_v2_20261007](../output/ere_rule_maintenance_closure_v2_20261007) | manifest.json: complete | 1 |
| [output/ere_rule_maintenance_preflight_20261007](../output/ere_rule_maintenance_preflight_20261007) | No direct lifecycle record found | 0 |
| [output/ere_rule_maintenance_preflight_v2_20261007](../output/ere_rule_maintenance_preflight_v2_20261007) | No direct lifecycle record found | 0 |
| [output/ere_rule_maintenance_recheck_20261007](../output/ere_rule_maintenance_recheck_20261007) | manifest.json: failed | 1 |
| [output/ere_rule_maintenance_trial_20261007](../output/ere_rule_maintenance_trial_20261007) | manifest.json: complete | 4 |
| [output/ere_rule_prevention_primitive_20261007](../output/ere_rule_prevention_primitive_20261007) | manifest.json: complete | 4 |
| [output/ere_rule_prevention_recheck_20261007](../output/ere_rule_prevention_recheck_20261007) | manifest.json: complete | 1 |
| [output/ere_rule_prevention_transfer_20261007](../output/ere_rule_prevention_transfer_20261007) | manifest.json: complete | 4 |
| [output/ere_rule_prevention_transfer_recheck_20261007](../output/ere_rule_prevention_transfer_recheck_20261007) | manifest.json: complete | 1 |
| [output/ere_rule_relational_development_20261004](../output/ere_rule_relational_development_20261004) | manifest.json: complete | 3 |
| [output/ere_rule_relational_learning_trial_20261004](../output/ere_rule_relational_learning_trial_20261004) | manifest.json: complete | 3 |
| [output/ere_rule_survival_measurement_20261007](../output/ere_rule_survival_measurement_20261007) | manifest.json: complete | 1 |
| [output/ere_rule_survival_measurement_recheck_20261007](../output/ere_rule_survival_measurement_recheck_20261007) | No direct lifecycle record found | 0 |
| [output/ere_runtime50k_signal_panel_20260930](../output/ere_runtime50k_signal_panel_20260930) | manifest.json: complete | 6 |
| [output/ere_sampler_reward_mapping_20261001](../output/ere_sampler_reward_mapping_20261001) | manifest.json: complete | 1 |
| [output/ere_sealed_repair_20260926](../output/ere_sealed_repair_20260926) | manifest.json: no top-level status/state; status.json: aborted_harness_failure | 2 |
| [output/ere_sealed_repair_v2_20260926](../output/ere_sealed_repair_v2_20260926) | manifest.json: no top-level status/state; status.json: failed_composite_damage_quarantine_control | 2 |
| [output/ere_sealed_repair_v3_20260926](../output/ere_sealed_repair_v3_20260926) | manifest.json: no top-level status/state; status.json: complete_characterization_with_failed_original_quarantine_gate | 2 |
| [output/ere_training_20260925](../output/ere_training_20260925) | manifest.json: no top-level status/state; status.json: blocked_by_prerequisite | 2 |
| [output/ere_v2_diagnosis_20260925](../output/ere_v2_diagnosis_20260925) | No direct lifecycle record found | 0 |
| [output/ere_valid_optimizer_signal_panel_20260929](../output/ere_valid_optimizer_signal_panel_20260929) | manifest.json: complete | 4 |
| [output/ere_valid_production_grammar_20260929](../output/ere_valid_production_grammar_20260929) | manifest.json: complete | 2 |
| [output/ere_witness_adversarial_20260926](../output/ere_witness_adversarial_20260926) | manifest.json: no top-level status/state | 1 |
| [output/ere_witness_repair_20260926](../output/ere_witness_repair_20260926) | manifest.json: no top-level status/state; status.json: bounded_combined_control_passed | 2 |
| [output/ere_witness_repair_v2_20260926](../output/ere_witness_repair_v2_20260926) | manifest.json: no top-level status/state; status.json: bounded_combined_control_passed | 2 |
| [output/evolving_minds_admission_development_20261003](../output/evolving_minds_admission_development_20261003) | manifest.json: Completed; run_status.json: no top-level status/state | 26 |
| [output/evolving_minds_admission_resume_20261003](../output/evolving_minds_admission_resume_20261003) | manifest.json: Completed; run_status.json: no top-level status/state | 2 |
| [output/evolving_minds_admission_verified_20261003](../output/evolving_minds_admission_verified_20261003) | manifest.json: Completed; run_status.json: no top-level status/state | 26 |
| [output/evolving_minds_arbitration_development_20261002](../output/evolving_minds_arbitration_development_20261002) | manifest.json: Interrupted; run_status.json: no top-level status/state | 120 |
| [output/evolving_minds_arbitration_verified_20261002](../output/evolving_minds_arbitration_verified_20261002) | manifest.json: Completed; run_status.json: no top-level status/state | 130 |
| [output/evolving_minds_arbitration_verified_v2_20261002](../output/evolving_minds_arbitration_verified_v2_20261002) | manifest.json: Completed; run_status.json: no top-level status/state | 130 |
| [output/evolving_minds_causal_controls_20260929](../output/evolving_minds_causal_controls_20260929) | manifest.json: Completed; run_status.json: no top-level status/state | 4 |
| [output/evolving_minds_causal_controls_verified_20260929](../output/evolving_minds_causal_controls_verified_20260929) | manifest.json: Completed; run_status.json: no top-level status/state | 4 |
| [output/evolving_minds_choice_development_20261001](../output/evolving_minds_choice_development_20261001) | manifest.json: Completed; run_status.json: no top-level status/state | 62 |
| [output/evolving_minds_choice_resume_20261001](../output/evolving_minds_choice_resume_20261001) | No direct lifecycle record found | 0 |
| [output/evolving_minds_choice_verified_20261001](../output/evolving_minds_choice_verified_20261001) | manifest.json: Completed; run_status.json: no top-level status/state | 62 |
| [output/evolving_minds_d0_development_20261002](../output/evolving_minds_d0_development_20261002) | manifest.json: Completed; run_status.json: no top-level status/state | 42 |
| [output/evolving_minds_d0_resume_20261002](../output/evolving_minds_d0_resume_20261002) | manifest.json: Completed; run_status.json: no top-level status/state | 2 |
| [output/evolving_minds_d0_verified_20261002](../output/evolving_minds_d0_verified_20261002) | manifest.json: Completed; run_status.json: no top-level status/state | 42 |
| [output/evolving_minds_descendant_audit_20261004](../output/evolving_minds_descendant_audit_20261004) | run_status.json: no top-level status/state | 5 |
| [output/evolving_minds_descendant_behavior_20261005](../output/evolving_minds_descendant_behavior_20261005) | manifest.json: Completed; run_status.json: no top-level status/state | 2 |
| [output/evolving_minds_descendant_behavior_metadata_20261005](../output/evolving_minds_descendant_behavior_metadata_20261005) | manifest.json: Completed; run_status.json: no top-level status/state | 2 |
| [output/evolving_minds_descendant_development_20261004](../output/evolving_minds_descendant_development_20261004) | manifest.json: Completed; run_status.json: no top-level status/state | 150 |
| [output/evolving_minds_descendant_routes_20261004](../output/evolving_minds_descendant_routes_20261004) | manifest.json: Completed; run_status.json: no top-level status/state | 2 |
| [output/evolving_minds_engineering_final_20260929](../output/evolving_minds_engineering_final_20260929) | No direct lifecycle record found | 4 |
| [output/evolving_minds_engineering_final_resume_20260929](../output/evolving_minds_engineering_final_resume_20260929) | No direct lifecycle record found | 0 |
| [output/evolving_minds_engineering_resume_20260929](../output/evolving_minds_engineering_resume_20260929) | No direct lifecycle record found | 0 |
| [output/evolving_minds_engineering_smoke_20260929](../output/evolving_minds_engineering_smoke_20260929) | No direct lifecycle record found | 4 |
| [output/evolving_minds_engineering_verified_20260929](../output/evolving_minds_engineering_verified_20260929) | No direct lifecycle record found | 4 |
| [output/evolving_minds_failure_audit_optimization_20261004](../output/evolving_minds_failure_audit_optimization_20261004) | manifest.json: Completed; run_status.json: no top-level status/state | 2 |
| [output/evolving_minds_family_compat_resume_20261002](../output/evolving_minds_family_compat_resume_20261002) | manifest.json: Completed; run_status.json: no top-level status/state | 2 |
| [output/evolving_minds_family_compat_resume_20261003](../output/evolving_minds_family_compat_resume_20261003) | manifest.json: Completed; run_status.json: no top-level status/state | 2 |
| [output/evolving_minds_family_development_20261002](../output/evolving_minds_family_development_20261002) | manifest.json: Completed; run_status.json: no top-level status/state | 10 |
| [output/evolving_minds_family_resume_20261002](../output/evolving_minds_family_resume_20261002) | manifest.json: Completed; run_status.json: no top-level status/state | 2 |
| [output/evolving_minds_family_verified_20261002](../output/evolving_minds_family_verified_20261002) | manifest.json: Completed; run_status.json: no top-level status/state | 10 |
| [output/evolving_minds_family_verified_v2_20261002](../output/evolving_minds_family_verified_v2_20261002) | manifest.json: Completed; run_status.json: no top-level status/state | 10 |
| [output/evolving_minds_funded_admission_development_20261003](../output/evolving_minds_funded_admission_development_20261003) | manifest.json: Completed; run_status.json: no top-level status/state | 46 |
| [output/evolving_minds_funded_admission_process_resume_20261003](../output/evolving_minds_funded_admission_process_resume_20261003) | manifest.json: Completed; run_status.json: no top-level status/state | 2 |
| [output/evolving_minds_funded_legacy_stream_compatibility_20261003](../output/evolving_minds_funded_legacy_stream_compatibility_20261003) | manifest.json: Completed; run_status.json: no top-level status/state | 2 |
| [output/evolving_minds_funding_development_20261003](../output/evolving_minds_funding_development_20261003) | manifest.json: Completed; run_status.json: no top-level status/state | 234 |
| [output/evolving_minds_handoff_audit_continuation_20261004](../output/evolving_minds_handoff_audit_continuation_20261004) | manifest.json: Completed; run_status.json: no top-level status/state | 2 |
| [output/evolving_minds_handoff_development_20261004](../output/evolving_minds_handoff_development_20261004) | manifest.json: Completed; run_status.json: no top-level status/state | 28 |
| [output/evolving_minds_lifecycle_development_20261002](../output/evolving_minds_lifecycle_development_20261002) | manifest.json: Completed; run_status.json: no top-level status/state | 10 |
| [output/evolving_minds_lifecycle_resume_20261002](../output/evolving_minds_lifecycle_resume_20261002) | manifest.json: Completed; run_status.json: no top-level status/state | 2 |
| [output/evolving_minds_lifecycle_verified_20261002](../output/evolving_minds_lifecycle_verified_20261002) | manifest.json: Completed; run_status.json: no top-level status/state | 10 |
| [output/evolving_minds_lifecycle_verified_v2_20261002](../output/evolving_minds_lifecycle_verified_v2_20261002) | manifest.json: Completed; run_status.json: no top-level status/state | 10 |
| [output/evolving_minds_matched_development_20261004](../output/evolving_minds_matched_development_20261004) | manifest.json: Completed; run_status.json: no top-level status/state | 42 |
| [output/evolving_minds_mutation_compat_resume_20261003](../output/evolving_minds_mutation_compat_resume_20261003) | manifest.json: Completed; run_status.json: no top-level status/state | 2 |
| [output/evolving_minds_mutation_development_20261002](../output/evolving_minds_mutation_development_20261002) | manifest.json: Completed; run_status.json: no top-level status/state | 14 |
| [output/evolving_minds_mutation_legacy_resume_20261002](../output/evolving_minds_mutation_legacy_resume_20261002) | manifest.json: Completed; run_status.json: no top-level status/state | 2 |
| [output/evolving_minds_mutation_resume_20261002](../output/evolving_minds_mutation_resume_20261002) | manifest.json: Completed; run_status.json: no top-level status/state | 2 |
| [output/evolving_minds_mutation_resume_v2_20261002](../output/evolving_minds_mutation_resume_v2_20261002) | manifest.json: Completed; run_status.json: no top-level status/state | 2 |
| [output/evolving_minds_mutation_verified_20261002](../output/evolving_minds_mutation_verified_20261002) | manifest.json: Completed; run_status.json: no top-level status/state | 14 |
| [output/evolving_minds_mutation_verified_v2_20261002](../output/evolving_minds_mutation_verified_v2_20261002) | manifest.json: Completed; run_status.json: no top-level status/state | 14 |
| [output/evolving_minds_neighborhood_auditor_continuation_20261003](../output/evolving_minds_neighborhood_auditor_continuation_20261003) | No direct lifecycle record found | 0 |
| [output/evolving_minds_neighborhood_development_20261003](../output/evolving_minds_neighborhood_development_20261003) | manifest.json: Completed; run_status.json: no top-level status/state | 118 |
| [output/evolving_minds_opportunity_accounting_20261004](../output/evolving_minds_opportunity_accounting_20261004) | manifest.json: Completed | 1 |
| [output/evolving_minds_retained_development_20261004](../output/evolving_minds_retained_development_20261004) | manifest.json: Completed; run_status.json: no top-level status/state | 18 |
| [output/evolving_minds_retained_funded_compatibility_20261004](../output/evolving_minds_retained_funded_compatibility_20261004) | manifest.json: Completed; run_status.json: no top-level status/state | 2 |
| [output/evolving_minds_retained_process_resume_20261004](../output/evolving_minds_retained_process_resume_20261004) | manifest.json: Completed; run_status.json: no top-level status/state | 2 |
| [output/evolving_minds_streaming_development_20261003](../output/evolving_minds_streaming_development_20261003) | manifest.json: Completed; run_status.json: no top-level status/state | 32 |
| [output/evolving_minds_streaming_process_resume_20261003](../output/evolving_minds_streaming_process_resume_20261003) | manifest.json: Completed; run_status.json: no top-level status/state | 2 |
| [output/evolving_minds_streaming_verified_20261003](../output/evolving_minds_streaming_verified_20261003) | manifest.json: Completed; run_status.json: no top-level status/state | 32 |
| [output/gui_binary_edge_42_20260708T034718](../output/gui_binary_edge_42_20260708T034718) | manifest.json: Completed | 1 |
| [output/gui_binary_edge_42_20260708T062454](../output/gui_binary_edge_42_20260708T062454) | manifest.json: Completed | 1 |
| [output/gui_binary_edge_42_20260708T094619](../output/gui_binary_edge_42_20260708T094619) | manifest.json: Completed | 1 |
| [output/gui_binary_edge_42_20260708T204034](../output/gui_binary_edge_42_20260708T204034) | manifest.json: Completed | 1 |
| [output/gui_glider_42_20260708T214818](../output/gui_glider_42_20260708T214818) | manifest.json: Completed | 1 |
| [output/gui_glider_42_20260708T214843](../output/gui_glider_42_20260708T214843) | manifest.json: Completed | 1 |
| [output/gui_glider_42_20260708T215018](../output/gui_glider_42_20260708T215018) | manifest.json: Interrupted | 1 |
| [output/gui_glider_42_20260708T225527](../output/gui_glider_42_20260708T225527) | manifest.json: Completed | 1 |
| [output/gui_glider_42_20260708T234251](../output/gui_glider_42_20260708T234251) | manifest.json: Completed | 1 |
| [output/gui_glider_42_20260708T234541](../output/gui_glider_42_20260708T234541) | manifest.json: Completed | 1 |
| [output/gui_glider_42_20260708T235306](../output/gui_glider_42_20260708T235306) | manifest.json: Completed | 1 |
| [output/gui_glider_42_20260709T092139](../output/gui_glider_42_20260709T092139) | manifest.json: Completed | 1 |
| [output/gui_growth_treadmill_42_20260708T204249](../output/gui_growth_treadmill_42_20260708T204249) | manifest.json: Completed | 1 |
| [output/gui_growth_treadmill_42_20260708T214551](../output/gui_growth_treadmill_42_20260708T214551) | manifest.json: Completed | 1 |
| [output/gui_live_42_20260707T231815](../output/gui_live_42_20260707T231815) | manifest.json: Completed | 1 |
| [output/gui_tetrahedral_42_20260708T214838](../output/gui_tetrahedral_42_20260708T214838) | manifest.json: Completed | 1 |
| [output/gui_triangle_subdivide_42_20260709T092214](../output/gui_triangle_subdivide_42_20260709T092214) | manifest.json: Completed | 1 |
| [runs/atrack](../runs/atrack) | No direct lifecycle record found | 0 |
| [runs/btrack](../runs/btrack) | No direct lifecycle record found | 0 |
| [runs/crystal_test_42_20260707T171502](../runs/crystal_test_42_20260707T171502) | manifest.json: Running | 1 |
| [runs/crystal_test_42_20260707T194546](../runs/crystal_test_42_20260707T194546) | manifest.json: Completed | 1 |
| [runs/crystal_test_42_20260707T194829](../runs/crystal_test_42_20260707T194829) | manifest.json: Completed | 1 |
| [runs/ctrack](../runs/ctrack) | No direct lifecycle record found | 0 |
| [runs/expL_v2_archive](../runs/expL_v2_archive) | No direct lifecycle record found | 0 |
| [runs/exp_l10](../runs/exp_l10) | No direct lifecycle record found | 31 |
| [runs/exp_l11](../runs/exp_l11) | No direct lifecycle record found | 6 |
| [runs/exp_l12](../runs/exp_l12) | No direct lifecycle record found | 10 |
| [runs/exp_l13](../runs/exp_l13) | No direct lifecycle record found | 12 |
| [runs/exp_l14](../runs/exp_l14) | No direct lifecycle record found | 12 |
| [runs/exp_l14_16sq](../runs/exp_l14_16sq) | No direct lifecycle record found | 12 |
| [runs/exp_l15](../runs/exp_l15) | No direct lifecycle record found | 4 |
| [runs/exp_l16](../runs/exp_l16) | No direct lifecycle record found | 95 |
| [runs/exp_l7r](../runs/exp_l7r) | No direct lifecycle record found | 5 |
| [runs/exp_l9](../runs/exp_l9) | No direct lifecycle record found | 25 |
| [runs/exp_l9b](../runs/exp_l9b) | No direct lifecycle record found | 25 |
| [runs/p0_gate_reaudit](../runs/p0_gate_reaudit) | No direct lifecycle record found | 9 |
| [runs/q0_retrodiction](../runs/q0_retrodiction) | No direct lifecycle record found | 3 |
| [experiments/lk/output/selfmod_v11_preflight_20260926](../experiments/lk/output/selfmod_v11_preflight_20260926) | manifest.json: no top-level status/state; status.json: complete | 2 |
| [experiments/lk/output/selfmod_v11_preflight_v2_20260926](../experiments/lk/output/selfmod_v11_preflight_v2_20260926) | manifest.json: no top-level status/state; status.json: complete | 3 |
| [experiments/lk/output/selfmod_v11_spectrum_20260926](../experiments/lk/output/selfmod_v11_spectrum_20260926) | manifest.json: no top-level status/state; status.json: complete | 4 |
| [experiments/lk/output/selfmod_v12_birth_20260926](../experiments/lk/output/selfmod_v12_birth_20260926) | manifest.json: no top-level status/state; status.json: complete | 3 |
| [experiments/lk/output/selfmod_v12_birth_preflight_20260926](../experiments/lk/output/selfmod_v12_birth_preflight_20260926) | manifest.json: no top-level status/state; status.json: complete | 3 |
| [experiments/lk/output/selfmod_v12_birth_preflight_v2_20260926](../experiments/lk/output/selfmod_v12_birth_preflight_v2_20260926) | manifest.json: no top-level status/state; status.json: complete | 3 |
| [experiments/lk/output/selfmod_v12_birth_v2_20260926](../experiments/lk/output/selfmod_v12_birth_v2_20260926) | manifest.json: no top-level status/state; status.json: complete | 4 |
| [experiments/lk/output/selfmod_v13_turnover_20260927](../experiments/lk/output/selfmod_v13_turnover_20260927) | manifest.json: no top-level status/state; status.json: complete | 5 |
| [experiments/lk/output/selfmod_v13_turnover_preflight_20260927](../experiments/lk/output/selfmod_v13_turnover_preflight_20260927) | manifest.json: no top-level status/state; status.json: complete | 6 |
| [experiments/lk/output/selfmod_v13_turnover_preflight_v2_20260927](../experiments/lk/output/selfmod_v13_turnover_preflight_v2_20260927) | manifest.json: no top-level status/state; status.json: complete | 6 |
| [experiments/lk/output/selfmod_v13_turnover_v2_20260927](../experiments/lk/output/selfmod_v13_turnover_v2_20260927) | manifest.json: no top-level status/state; status.json: complete | 7 |
| [experiments/lk/output/selfmod_v14_bounded_20260927](../experiments/lk/output/selfmod_v14_bounded_20260927) | manifest.json: no top-level status/state; status.json: complete | 7 |
| [experiments/lk/output/selfmod_v14_bounded_preflight_20260927](../experiments/lk/output/selfmod_v14_bounded_preflight_20260927) | manifest.json: no top-level status/state; status.json: complete | 6 |
| [experiments/lk/output/selfmod_v15_replication_20260927](../experiments/lk/output/selfmod_v15_replication_20260927) | manifest.json: no top-level status/state; status.json: complete | 10 |
| [experiments/lk/output/selfmod_v15_replication_preflight_20260927](../experiments/lk/output/selfmod_v15_replication_preflight_20260927) | manifest.json: no top-level status/state; status.json: complete | 6 |
| [experiments/lk/output/selfmod_v16_financing_20260927](../experiments/lk/output/selfmod_v16_financing_20260927) | manifest.json: no top-level status/state; status.json: complete | 72 |
| [experiments/lk/output/selfmod_v16_financing_preflight_20260927](../experiments/lk/output/selfmod_v16_financing_preflight_20260927) | manifest.json: no top-level status/state; status.json: complete | 54 |
| [experiments/lk/output/selfmod_v17_durability_20260927](../experiments/lk/output/selfmod_v17_durability_20260927) | manifest.json: no top-level status/state; status.json: complete | 73 |
| [experiments/lk/output/selfmod_v17_durability_preflight_20260927](../experiments/lk/output/selfmod_v17_durability_preflight_20260927) | status.json: failed | 5 |
| [experiments/lk/output/selfmod_v17_durability_preflight_v2_20260927](../experiments/lk/output/selfmod_v17_durability_preflight_v2_20260927) | manifest.json: no top-level status/state; status.json: complete | 55 |
| [experiments/lk/output/selfmod_v18_variants_20260928](../experiments/lk/output/selfmod_v18_variants_20260928) | manifest.json: no top-level status/state; status.json: complete | 37 |
| [experiments/lk/output/selfmod_v18_variants_preflight_20260928](../experiments/lk/output/selfmod_v18_variants_preflight_20260928) | manifest.json: no top-level status/state; status.json: complete | 27 |
| [experiments/lk/output/selfmod_v19_interruption_verification_20260928](../experiments/lk/output/selfmod_v19_interruption_verification_20260928) | No direct lifecycle record found | 0 |
| [experiments/lk/output/selfmod_v19_second_edit_20260928](../experiments/lk/output/selfmod_v19_second_edit_20260928) | manifest.json: no top-level status/state; status.json: failed | 10 |
| [experiments/lk/output/selfmod_v19_second_edit_preflight_20260928](../experiments/lk/output/selfmod_v19_second_edit_preflight_20260928) | manifest.json: no top-level status/state; status.json: complete | 17 |
| [experiments/lk/output/selfmod_v19_semantic_target_engineering_20260928](../experiments/lk/output/selfmod_v19_semantic_target_engineering_20260928) | No direct lifecycle record found | 0 |
| [experiments/lk/output/selfmod_v19b_second_edit_20260928](../experiments/lk/output/selfmod_v19b_second_edit_20260928) | manifest.json: no top-level status/state; status.json: complete | 29 |
| [experiments/lk/output/selfmod_v19b_second_edit_preflight_20260928](../experiments/lk/output/selfmod_v19b_second_edit_preflight_20260928) | manifest.json: no top-level status/state; status.json: complete | 17 |
| [experiments/lk/output/selfmod_v20_third_edit_20260928](../experiments/lk/output/selfmod_v20_third_edit_20260928) | manifest.json: no top-level status/state; status.json: complete | 34 |
| [experiments/lk/output/selfmod_v20_third_edit_preflight_20260928](../experiments/lk/output/selfmod_v20_third_edit_preflight_20260928) | manifest.json: no top-level status/state; status.json: complete | 20 |
| [experiments/lk/output/selfmod_v20_third_edit_preflight_v2_20260928](../experiments/lk/output/selfmod_v20_third_edit_preflight_v2_20260928) | manifest.json: no top-level status/state; status.json: complete | 21 |
| [experiments/lk/output/selfmod_v21_reversible_edit_20260928](../experiments/lk/output/selfmod_v21_reversible_edit_20260928) | manifest.json: no top-level status/state; status.json: complete | 37 |
| [experiments/lk/output/selfmod_v21_reversible_edit_preflight_20260928](../experiments/lk/output/selfmod_v21_reversible_edit_preflight_20260928) | manifest.json: no top-level status/state; status.json: complete | 21 |
| [experiments/lk/output/selfmod_v22_maintenance_20260928](../experiments/lk/output/selfmod_v22_maintenance_20260928) | manifest.json: no top-level status/state; status.json: complete | 25 |
| [experiments/lk/output/selfmod_v22_maintenance_preflight_20260928](../experiments/lk/output/selfmod_v22_maintenance_preflight_20260928) | manifest.json: no top-level status/state; status.json: complete | 18 |
| [experiments/lk/output/selfmod_v22_maintenance_preflight_v2_20260928](../experiments/lk/output/selfmod_v22_maintenance_preflight_v2_20260928) | manifest.json: no top-level status/state; status.json: complete | 19 |
| [experiments/lk/output/selfmod_v23_fresh50k_20260928](../experiments/lk/output/selfmod_v23_fresh50k_20260928) | manifest.json: no top-level status/state; status.json: complete | 33 |
| [experiments/lk/output/selfmod_v23_fresh50k_preflight_20260928](../experiments/lk/output/selfmod_v23_fresh50k_preflight_20260928) | manifest.json: no top-level status/state; status.json: complete | 21 |
| [experiments/lk/output/selfmod_v23_reached_replay_20261009](../experiments/lk/output/selfmod_v23_reached_replay_20261009) | No direct lifecycle record found | 1 |
| [experiments/lk/output/selfmod_v24_conflict_attribution_20261009](../experiments/lk/output/selfmod_v24_conflict_attribution_20261009) | manifest.json: no top-level status/state; status.json: complete | 12 |
| [crates/hg-train/runs_stage2/runs](../crates/hg-train/runs_stage2/runs) | No direct lifecycle record found | 0 |

## Documentation snapshot limits

Historical documents retain superseded recommendations and date-specific counts. Use the experimental report for current programme closure. Duplicate documents across root and isolated LK are provenance copies, not independent findings. Historical LK runs were not all copied with its source; see the [integration record](lk_workspace_integration.md).
