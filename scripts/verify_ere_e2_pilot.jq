# Read-only accounting/status audit. Does not simulate or rerun the optimizer.
# P1 signatures and P2 emission TV are recorded measurements, not recomputed here.
def close($a; $b): ($a - $b | fabs) < 1e-12;
def valid:
  .error == null and .observation.valid == true and
  .observation.ceiling_bound == false and .observation.final_step == .requested_steps and
  (.observation.structural_proposal_score | type == "number" and isfinite and . >= 0 and . <= 1);

$evaluations[0] as $e | $controls[0] as $c | $prechecks[0] as $p |
$status[0] as $s | $manifest[0] as $m | $theta[0] as $t |
[$e[] | select(.label == "stateability_baseline") | .observation.structural_proposal_score] as $baseline |
($baseline | add / length) as $mean |
($baseline | map(pow(. - $mean; 2)) | add / length | sqrt) as $sd |
([$e[] | select(.label == "forced_structural_active") | .observation.structural_proposal_score][0] -
 [$e[] | select(.label == "forced_priority_dormant") | .observation.structural_proposal_score][0] | fabs) as $discrimination |
[$e[18:] | range(0; length; 2) as $i |
  {generation: (($i / 8 | floor) + 1), pair: (($i % 8) / 2),
   valid: ([.[$i], .[$i + 1]] | all(.[]; valid))}] as $pairs |
($pairs | map(select(.valid)) | length) as $valid_pairs |
($p[0].measured >= 1) as $p1 |
($p[1].measured >= 0.05 and $valid_pairs == 20) as $p2 |
($discrimination > 2 * $sd and $discrimination > 1e-12) as $p3 |
($mean | [., 1 - .] | min) as $margin |
($margin > 0.02) as $p4 |
($c | all(.[]; .observation.valid == true and .observation.ceiling_bound == false and .observation.final_step == 1000)) as $controls_valid |
($c | map(select(.arm == "A3")) | all(.[];
  .observation.anchor_survival_fraction == 1 and .observation.cleanup_edges_lost == 0 and .observation.quarantine_decisions == 0)) as $intact |
{
  frozen_manifest:
    ($m.protocol == "E2-family-grammar-pilot-v1" and $m.parameters == 40 and $m.families == 4 and
     $m.seed == 972042076161 and $m.max_evaluations == 58 and $m.steps_per_eval == 1000 and
     $m.maximum_simulation_steps == 58000 and $m.precheck_generations == 5 and $m.precheck_population == 8 and
     $m.base_config.ere.match_ceiling == 50000 and $m.base_config.ere.rule_region_ceiling == 500 and
     $m.base_config.ere.match_budget_policy == "Pooled" and $m.base_config.ere.epoch.Fixed == 10 and
     $m.full_campaign == "not_launched"),
  complete_attempt_log:
    (($e | length) == 58 and [$e[].evaluation] == [range(0;58)] and
     all($e[]; .requested_steps == 1000 and (.theta | length) == 40 and all(.theta[]; isfinite)) and
     all($e[]; if .error == null then valid else .observation == null end)),
  no_hidden_or_missing_sections:
    (($c | length) == 12 and ($baseline | length) == 4 and ($pairs | length) == 20 and
     all(range(0;12); $e[.].label == "matched_control" and $e[.].seed == $m.seed + (. / 3 | floor) and
       $e[.].arm == ["A1","A2","A3"][. % 3]) and
     all(range(12;16); $e[.].label == "stateability_baseline" and $e[.].arm == "A3" and $e[.].seed == $m.seed + . - 12) and
     $e[16].label == "forced_structural_active" and $e[17].label == "forced_priority_dormant" and
     all(range(18;58); $e[.].label == (if . % 2 == 0 then "optimizer_probe_plus" else "optimizer_probe_minus" end) and
       $e[.].arm == "A3" and $e[.].seed == $m.seed + ((. - 18) / 8 | floor))),
  controls_match_log:
    ($c == [$e[] | select(.label == "matched_control") | {seed,arm,observation}]),
  repeated_baselines_match:
    (all(range(0;4); $e[12 + .].observation == $e[2 + . * 3].observation)),
  probe_l2_matches:
    (($t | length) == 40 and all($t[]; isfinite) and
     # f32 theta JSON uses short round-tripping decimals; jq computes in f64.
     ((($t | map(. * .) | add | sqrt) - $p[1].parameter_l2) | fabs) < 1e-7),
  precheck_consistency:
    (($p | map(.name)) == ["P1_CLASS_EXPRESSIBLE","P2_OPTIMIZER_TRAVERSAL","P3_OBJECTIVE_DISCRIMINATION","P4_NOT_SATURATED"] and
     $p[0].passed == $p1 and $p[1].passed == $p2 and $p[1].valid_pairs == $valid_pairs and
     $p[2].passed == $p3 and close($p[2].measured; $discrimination) and close($p[2].threshold; 2 * $sd) and
     $p[3].passed == $p4 and close($p[3].measured; $margin)),
  decision_consistency:
    ($s.stateability_pass == ($p1 and $p2 and $p3 and $p4) and $s.controls_valid == $controls_valid and
     $s.unprotected_intact == $intact and $s.ready_for_full_campaign_decision == ($s.stateability_pass and $controls_valid and $intact) and
     $s.pilot_evaluations_attempted == 58 and $s.maximum_steps_requested == 58000 and
     $s.pilot_probe_generations_completed == 5 and $s.pilot_probe_evaluations == 40 and
     $s.full_optimizer_generations_completed == 0 and $s.full_optimizer_evaluation_steps == 0 and
     $s.held_out_evaluation_steps == 0 and $s.status == "full_training_blocked_by_pilot")
} as $checks |
{
  kind: "read_only_pilot_accounting_audit_not_research_pass",
  checks: $checks, accounting_verified: all($checks[]; . == true),
  evaluations_attempted: ($e | length),
  complete_cap_free_evaluations: ([$e[] | select(valid)] | length),
  errors: [$e[] | select(.error != null) | {evaluation,label,seed,error}],
  known_completed_steps: ([$e[] | select(valid) | .observation.final_step] | add),
  requested_step_upper_bound: ([$e[].requested_steps] | add),
  failed_rollout_partial_steps: null,
  valid_probe_pairs: $valid_pairs,
  pairs_by_generation: ($pairs | group_by(.generation) | map({generation:.[0].generation, valid_pairs:([.[] | select(.valid)] | length)})),
  baseline_mean: $mean, baseline_population_sd: $sd,
  stateability_pass: ($p1 and $p2 and $p3 and $p4), unprotected_intact: $intact,
  full_training_launched: false,
  limits: ["No physics or optimizer rerun", "P1 signatures and P2 emission TV accepted as recorded", "No per-class fire maps saved for independent primary-score reconstruction", "Legacy meta_editor_events means all meta-touching rewrites, not paid-editor-only events"]
} |
if .accounting_verified then . else error("pilot accounting mismatch: \(.)") end
