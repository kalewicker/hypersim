# Read-only paired receipt verification. Bind old, trial, audit and replay using
# jq --slurpfile; historical receipts are never edited or promoted.
def consistent:
  .valid + .quarantined == .samples and
  .neutral + .near_neutral + .divergent + .frozen + .exploding + .unmeasured == .valid;
def stratum($near):
  consistent and .samples > 0 and .unmeasured == 0 and
  .valid / .samples >= 0.40 and
  (($near | not) or .near_neutral / .samples >= 0.10) and
  (.frozen + .exploding) / .samples <= 0.25;
def shifted($outcome): .unmeasured -= 1 | .[$outcome] += 1;

$old[0] as $baseline | $trial[0] as $new | $audit[0] as $original | $replay[0] as $probe |
$new.previously_unmeasured_classifications as $changes |
([1136,2518,3183,4554,6859,7127,8380,8679] | map(tostring) | sort) as $ids |
(reduce ($changes | to_entries[]) as $change ($baseline;
  ($original.cases[] | select((.sample_index_zero_based | tostring) == $change.key)) as $case |
  ($case.core_seed_index / 4 | floor | tostring) as $shape |
  $change.value as $outcome |
  .unmeasured -= 1 |
  if $outcome == "frozen" or $outcome == "exploding"
  then .catastrophic += 1 else .[$outcome] += 1 end |
  .by_operator.AddPatternEdge |= shifted($outcome) |
  .by_seed_shape[$shape] |= shifted($outcome) |
  .semantic.structural |= shifted($outcome) |
  .semantic.active_parent_structural |= shifted($outcome) |
  .semantic.target_roles[$case.target_role] |= shifted($outcome)
)) as $expected |
{
  distinct_protocol_and_fixed_envelope:
    ($new.protocol == "E2-alternatives-pooled-400k50k-v1" and
     $baseline.protocol == "E2-alternatives-pooled-v1" and
     $new.match_budget_policy == "Pooled" and
     $new.global_work == 400000 and $new.global_results == 50000 and
     $new.rng_seed == $original.rng_seed and $new.samples == 10000 and
     $new.seeds == 168 and $new.trajectory_steps == 100),
  exactly_eight_completed_classifications:
    (($changes | keys | sort) == $ids and
     all($changes[]; . == "near_neutral" or . == "divergent" or . == "frozen" or . == "exploding")),
  all_other_receipt_fields_unchanged:
    (($expected | del(.protocol, .pass)) ==
     ($new | del(.protocol, .pass, .global_work, .global_results, .rng_seed, .kind,
       .resource_usage, .previously_unmeasured_classifications, .elapsed_seconds))),
  recomputed_acceptance:
    ($new.pass == true and $new.unmeasured == 0 and
     $new.valid + $new.quarantined == $new.samples and
     $new.neutral + $new.near_neutral + $new.divergent + $new.catastrophic == $new.valid and
     $new.valid / $new.samples >= 0.40 and $new.near_neutral / $new.samples >= 0.10 and
     $new.catastrophic / $new.samples <= 0.25 and
     $new.neutral_soundness_checked >= 50 and $new.neutral_soundness_failures == 0 and
     $new.at_cap_samples > 0 and
     ($new.semantic.structural | stratum(true)) and
     ($new.semantic.active_parent_structural | stratum(true) and .samples >= 2000) and
     ($new.semantic.activation | stratum(false) and .samples >= 1000) and
     ($new.semantic.target_roles | keys | sort) == ["Core","Proposal"] and
     all($new.semantic.target_roles[]; consistent and .samples > 0) and
     $new.semantic.mode_changes >= 100 and
     $new.semantic.mode_changes <= $new.semantic.activation.samples and
     $new.semantic.trajectory_changing_activation >= 20 and
     $new.semantic.trajectory_changing_activation <= $new.semantic.mode_changes),
  full_assay_caps_respected:
    ($new.resource_usage.budget_violations == 0 and
     $new.resource_usage.match_ceiling_steps == 0 and
     $new.resource_usage.graph_ceiling_steps == 0 and
     $new.resource_usage.peak_step_work <= 400000 and
     $new.resource_usage.peak_step_results <= 50000),
  replay_correctness:
    ($probe.protocol == $new.protocol and $probe.correctness_pass == true and
     $probe.targeted_cases == 8 and $probe.rescued_cases == 8 and
     $probe.remaining_bounded_cases == 0 and
     ([$probe.cases[].sample_index_zero_based | tostring] | sort) == $ids and
     all($probe.cases[]; .correctness_pass and .budgets_respected and
       .split_reproduced and .pooled_completed_prefix_matches_oracle and
       .split_completed_prefix_matches_oracle and .pooled.completed_steps == 100))
} |
. + {verification_pass: all(.[]; . == true)} |
if .verification_pass then . else error("400k/50k receipt verification failed: \(.)") end
