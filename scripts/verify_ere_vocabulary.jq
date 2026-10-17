# jq -s --slurpfile manifest OUTPUT/manifest.json --slurpfile evaluations
# OUTPUT/evaluations.json --slurpfile status OUTPUT/status.json -f this_file OUTPUT/rollouts/*.json
# Arithmetic audit, not a new gate or simulation replay.
def close($a;$b): if $a==null or $b==null then $a==$b else ($a-$b|fabs)<1e-12 end;
def reuse($m): [$m[]|if .>1 then log else 0 end]|add // 0;
def world_meta($v): $v.world_rule_meta_events // $v.destructive_world_meta_events;
. as $r | $manifest[0] as $m | $evaluations[0] as $e | $status[0] as $s |
"E2-schema-preserving-vocabulary-v1" as $protocol |
{
  manifest: ($m.protocol==$protocol and $m.seed==980632010753 and $m.attempts==12 and
    $m.steps_per_attempt==1000 and $m.requested_step_ceiling==12000 and
    $m.optimizer_evaluations==0 and $m.held_out_evaluations==0 and $m.old_gate_admission_reused==false),
  attempts: (($r|length)==12 and ($e|length)==12 and [$r[].evaluation]==[range(0;12)] and
    all(range(0;12);. as $i|$e[$i].evaluation==$i and $e[$i].seed==$m.seed+($i/3|floor) and
      $e[$i].arm==["A1","A2","A3"][$i%3] and $e[$i].requested_steps==1000 and
      $e[$i].completed_steps==$r[$i].completed_steps and $e[$i].error==$r[$i].error and
      $e[$i].observation==$r[$i].observation and $e[$i].accounting_ok==$r[$i].accounting_ok and
      $e[$i].encoded_events==$r[$i].encoded_events and world_meta($e[$i])==world_meta($r[$i]))),
  profiles: (all(range(0;12);. as $i|$r[$i].effective_config as $c|
    $c==$m.planned_effective_configs[$i] and $c.seed==$e[$i].seed and $c.ere.arm==$e[$i].arm and
    $c.engine.max_steps==1000 and $c.engine.max_pairings_per_step==16 and
    $c.engine.thread_count==1 and $c.engine.metrics_window_interval==100 and $c.engine.step_mode=="MaxParallel" and
    $c.ere.encoding_version=="AlternativeFamiliesV3" and $c.ere.match_budget_policy=="Pooled" and
    $c.ere.epoch.Fixed==10 and $c.ere.match_ceiling==50000 and $c.ere.rule_region_ceiling==500 and
    $c.ere.meta_seed_energy==0 and $c.ere.meta_rules_toml==null and $c.ere.measure_family_maintenance==true and
    $c.ere.measure_ei==false and $c.evaluator.enabled==false and $c.causal.enabled==false and
    $c.initial.condition_type=="PureCrystal" and $c.initial.width==4 and $c.initial.height==4 and
    $c.initial.arity==2 and $c.initial.periodic==true and $c.ere.vocabulary.protocol==$protocol and
    [$c.ere.vocabulary.instructions[].operation]==["Activate","Deactivate","Transpose","Transpose"] and
    all($c.ere.vocabulary.instructions[];.fuel==32))),
  matched_initial_material: (all(range(0;4);. as $i|
    $r[$i*3].emission==$r[$i*3+1].emission and $r[$i*3].emission==$r[$i*3+2].emission and
    ($r[$i*3].effective_config|del(.ere.arm))==($r[$i*3+1].effective_config|del(.ere.arm)) and
    ($r[$i*3].effective_config|del(.ere.arm))==($r[$i*3+2].effective_config|del(.ere.arm)))),
  measured_windows: (all($r[];. as $receipt|.ere_windows as $w|
    .partial_error_history_retained==true and .completed_steps!=null and .completed_steps>=0 and .completed_steps<=1000 and
    ($w|length)>0 and $w[-1].step_end==.completed_steps and
    all(range(0;($w|length));. as $i|$w[$i].step_start==(if $i==0 then 0 else $w[$i-1].step_end end) and
      $w[$i].step_end>=$w[$i].step_start and $w[$i].step_end-$w[$i].step_start<=100 and
      $w[$i].encoded_meta_vocabulary_protocol==$protocol and
      $w[$i].seeded_editor_rewrite_events==0 and $w[$i].meta_rewrite_events==$w[$i].ordinary_meta_rewrite_events and
      $w[$i].encoded_meta_rewrite_events<= $w[$i].ordinary_meta_rewrite_events and
      $w[$i].encoded_meta_rewrite_events==($w[$i].encoded_meta_operation_traces|length) and
      ($w[$i].seeded_family_maintenance|length)==4 and
      all($w[$i].seeded_family_maintenance[];.checks==$w[$i].step_end and .valid_steps<=.checks and .final_valid==(.final_failure==null)) and
      all($w[$i].encoded_meta_instruction_states[];.checks==$w[$i].step_end and .valid_steps<=.checks)) and
    $receipt.instruction_states==$w[-1].encoded_meta_instruction_states)),
  paid_events: (all($r[];.ere_windows as $w|([$w[].encoded_meta_rewrite_events]|add) as $paid|
    .encoded_events==$paid and world_meta(.)==([$w[].ordinary_meta_rewrite_events]|add)-$paid and
    all($w[].encoded_meta_operation_traces[];.fuel_before-.fuel_after==1 and .fuel_after>=0 and .fuel_before<=32) and
    ($w[-1].encoded_meta_instruction_states|length)==4 and
    all($w[-1].encoded_meta_instruction_states[];.valid and .valid_steps==.checks and .fuel>=0 and .fuel<=32) and
    ([$w[-1].encoded_meta_instruction_states[].fuel]|add)+$paid==128 and .accounting_ok==true and
    all($w[-1].encoded_meta_instruction_states[];. as $state |
      [$w[].encoded_meta_operation_traces[]|select(.instruction==$state.instruction)] as $trace |
      $state.fuel+($trace|length)==32 and
      all(range(0;($trace|length));. as $i|$trace[$i].operation==$state.operation and
        $trace[$i].fuel_before==32-$i and $trace[$i].fuel_after==31-$i)) and
    all($w[].encoded_meta_operation_traces[];. as $trace|
      any($w[-1].encoded_meta_instruction_states[];.instruction==$trace.instruction)))),
  scores: (all($r[];.ere_windows as $w|.observation as $o|
    if $o==null then .error!=null else
      $w[-1].seeded_family_maintenance as $f | $w[-1].structural_proposal_reuse_count_by_class as $map |
      ([reuse($map)/(4*((4000+1)|log)),1]|min) as $reuse |
      ([$f[].valid_steps]|add/(4*$o.final_step)) as $t |
      ([$f[]|select(.final_valid)]|length/4) as $final |
      .completed_steps==1000 and $o.final_step==1000 and $o.valid==true and ($w|length)==10 and
      all($w[];.match_ceiling_binding==false and .rule_region_ceiling_binding==false) and
      $o.families==$f and $o.structural_reuse_by_class==$map and
      close($o.reuse_score;$reuse) and close($o.time_valid_fraction;$t) and close($o.final_valid_fraction;$final) and
      close($o.joint_score;$reuse*$t*$final) and $o.structural_world_fires==([$map[]]|add//0) and
      $o.seeded_editor_events==0 and $o.ordinary_meta_events==([$w[].ordinary_meta_rewrite_events]|add) and
      $o.meta_rewrite_events==$o.ordinary_meta_events and $o.quarantine_decisions==([$w[].rules_quarantined]|add) and
      $o.cleanup_edges_lost==([$w[].meta_edges_lost_to_cleanup]|add)
    end)),
  decision: ($s.protocol==$protocol and $s.attempts==12 and
    $s.complete==([$e[]|select(.completed_steps==1000 and .error==null)]|length) and $s.errors==12-$s.complete and
    $s.actual_steps==([$e[].completed_steps]|add) and $s.actual_steps<=12000 and $s.requested_steps==12000 and
    $s.optimizer_evaluation_steps==0 and $s.held_out_evaluation_steps==0 and $s.full_training_admitted==false and
    $s.status==(if $s.complete==12 then "diagnostic_complete" else "diagnostic_incomplete_or_failed" end))
} as $checks |
{kind:"independent_arithmetic_audit_not_gate",checks:$checks,accounting_verified:all($checks[];.),
  limits:["No graph snapshot replay", "Primitive preservation verified separately in contract tests", "No full G-R or stateability admission"]} |
if .accounting_verified then . else error("vocabulary receipt mismatch: \(.)") end
