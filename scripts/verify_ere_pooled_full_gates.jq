# Read-only association of the classification resource-envelope receipt with
# task-specific pooled downstream gates. This is not a training admission hook.
def consistent:
  .samples > 0 and .valid + .quarantined == .samples and
  .neutral + .near_neutral + .divergent + .frozen + .exploding + .unmeasured == .valid;
def stratum($near):
  consistent and .unmeasured == 0 and .valid / .samples >= 0.40 and
  (($near | not) or .near_neutral / .samples >= 0.10) and
  (.frozen + .exploding) / .samples <= 0.25;

$gr1[0] as $one | $gr2[0] as $two | $gr2_header[0] as $header |
$gr3[0] as $three | $fixtures[0] as $f |
{
  receipt_profiles_match:
    ($one.protocol == "E2-alternatives-pooled-400k50k-v1" and
     $one.match_budget_policy == "Pooled" and $one.global_work == 400000 and
     $one.global_results == 50000 and
     all($header,$three,$f; .protocol == "E2-alternatives-pooled-v1" and .match_budget_policy == "Pooled") and
     $header.encoding == "AlternativeFamiliesV3" and
     $header.global_work == 2000000 and $header.global_results == 100000 and
     all($three,$f; .global_work == 50000 and .global_results == 50000)),
  gr0_verified: ($f.gr0 == true and $f.rcf_pairs == 40),
  gr1_verified:
    ($one.pass == true and $one.samples == 10000 and $one.seeds == 168 and
     $one.trajectory_steps == 100 and $one.unmeasured == 0 and
     $one.valid + $one.quarantined == $one.samples and
     $one.neutral + $one.near_neutral + $one.divergent + $one.catastrophic == $one.valid and
     $one.valid / $one.samples >= 0.40 and $one.near_neutral / $one.samples >= 0.10 and
     $one.catastrophic / $one.samples <= 0.25 and
     $one.neutral_soundness_checked >= 50 and $one.neutral_soundness_failures == 0 and
     $one.at_cap_samples > 0 and
     ($one.semantic.structural | stratum(true)) and
     ($one.semantic.active_parent_structural | stratum(true) and .samples >= 2000) and
     ($one.semantic.activation | stratum(false) and .samples >= 1000) and
     ($one.semantic.target_roles | keys | sort) == ["Core","Proposal"] and
     all($one.semantic.target_roles[]; consistent) and
     $one.semantic.mode_changes >= 100 and $one.semantic.mode_changes <= $one.semantic.activation.samples and
     $one.semantic.trajectory_changing_activation >= 20 and
     $one.semantic.trajectory_changing_activation <= $one.semantic.mode_changes and
     $one.resource_usage.budget_violations == 0 and
     $one.resource_usage.match_ceiling_steps == 0 and $one.resource_usage.graph_ceiling_steps == 0 and
     $one.resource_usage.peak_step_work <= 400000 and $one.resource_usage.peak_step_results <= 50000),
  gr2_verified:
    ($header.trajectory_steps == 1000 and $header.library_rules == 45 and
     ($two | length) == 45 and ([$two[] | [.file,.name]] | unique | length) == 45 and
     any($two[]; .status == "identical") and
     all($two[]; if .status == "identical" then .steps == 1000
         else (.status | startswith("outside-envelope:")) and .steps == 0 end)),
  gr3_verified:
    ($three.pass == true and $three.steps == 100000 and $three.completed_steps == 100000 and
     $three.a0_world_step == 100000 and $three.a1_world_step == 100000 and
     $three.graph_hashes_equal == true and $three.canonical_csv_equal == true and
     $three.canonical_csv_bytes > 0 and $three.rng_states_equal == true and
     $three.rng_consumed == true and $three.every_step_advanced == true and
     $three.match_ceiling_bound == false and $three.accepted_rewrites > 0 and
     $three.attempted_matches > 0 and $three.peak_step_work <= 50000 and $three.peak_step_results <= 50000),
  gr4_verified: ($f.gr4 == true and $f.quarantine_by_validator == [1,1,1,1,1,1,1,1,1]),
  gr5_verified:
    ($f.gr5 == true and $f.protection_steps == 10000 and $f.protection_completed_steps == 10000 and
     $f.protection_world_step == 10000 and $f.protection_accepted_rewrites > 0 and
     $f.meta_editor_events > 0 and $f.peak_step_work <= 50000 and $f.peak_step_results <= 50000),
  gr6_verified:
    ($f.gr6 == true and $f.changed_rule_set_stable == false and
     ($f | has("changed_meta_included_ei") and has("changed_meta_excluded_ei")) and
     $f.changed_meta_included_ei == null and $f.changed_meta_excluded_ei == null),
  fixture_aggregate_pass: ($f.pass == true)
} as $checks |
$checks + {
  all_gates_verified: all($checks[]; . == true),
  kind: "verified_gate_association_not_training_authorization",
  classification_protocol: $one.protocol,
  downstream_protocol: $header.protocol,
  supported_rules: ([$two[] | select(.status == "identical")] | length),
  outside_envelope_rules: ([$two[] | select(.status | startswith("outside-envelope:"))] | length),
  library_rules: ($two | length)
} |
if .all_gates_verified then . else error("pooled full gate verification failed: \(.)") end
