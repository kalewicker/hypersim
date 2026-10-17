# jq -s --slurpfile manifest OUT/manifest.json --slurpfile evaluations OUT/evaluations.json
# --slurpfile contrasts OUT/contrasts.json --slurpfile status OUT/status.json -f this_file OUT/rollouts/*.json
def close($a;$b): if $a==null or $b==null then $a==$b else ($a-$b|fabs)<1e-12 end;
def paid($r): [$r.ere_windows[].encoded_meta_rewrite_events]|add // 0;
def trace($r): [$r.ere_windows[].encoded_meta_operation_traces[]];
def comparable($s): $s|{world_hash_before,world_hash_after,world_rule_table,rng_before_compile,rng_after_compile,rng_after_step};
def complete($r): $r.error==null and $r.accounting_ok and $r.completed_steps==100 and ($r.step_observations|length)==100;
def intact($r): all($r.step_observations[];(.families|length)==4 and all(.families[];.valid)) and
  ($r.ere_windows[-1].encoded_meta_instruction_states|length)==1 and $r.ere_windows[-1].encoded_meta_instruction_states[0].valid;
def persists($r): (trace($r)[0].target_family) as $root |
  [$r.step_observations[].families[]|select(.anchor==$root)] as $states |
  $root!=null and ($states|length)>0 and ($states|length)==($r.step_observations|length) and
  all($states[];.valid and .active==true and .execution_bytes!=null and .execution_bytes==$states[0].execution_bytes);
def firstdiff($a;$b;$key): [range(0;([($a.step_observations|length),($b.step_observations|length)]|min)) as $i |
  select($a.step_observations[$i][$key]!=$b.step_observations[$i][$key])|$b.step_observations[$i].step][0];
def firstfire($r): [$r.ere_windows[]|select(([.structural_proposal_reuse_count_by_class[]]|add//0)>0)|.step_end][0];
def checks($a;$b;$n;$m): firstdiff($a;$b;"world_rule_table") as $table |
  firstdiff($a;$b;"world_hash_after") as $world | firstfire($b) as $fire |
  {
    complete_protected_and_frozen_controls: all([$a,$b,$n,$m][];complete(.)),
    one_paid_edit_no_static_editor: (paid($a)==1 and paid($b)==1),
    families_and_instructions_intact: all([$a,$b,$n,$m][];intact(.)),
    paid_edit_persists: (persists($a) and persists($b)),
    same_edit_world_table_rng_through_step10: (($a.step_observations|length)>=10 and ($b.step_observations|length)>=10 and
      all(range(0;10);comparable($a.step_observations[.])==comparable($b.step_observations[.]))),
    table_installed_at_step11: ($table==11 and $b.step_observations[10].world_rule_table!=$b.step_observations[0].world_rule_table),
    world_changes_after_compile: ($world!=null and $world>=11 and $world<=100),
    genuine_proposal_world_firing: ($fire!=null and $fire>=11 and $fire<=100 and $a.observation.structural_world_fires==0),
    no_edit_negative_identity: (($n.step_observations|length)==100 and ($m.step_observations|length)==100 and
      all(range(0;100);comparable($n.step_observations[.])==comparable($m.step_observations[.])) and
      all([$n,$m][];all(.step_observations[].families[];.active==false) and
        .observation.structural_world_fires==0 and paid(.)==0))
  };
. as $r | $manifest[0] as $m | $evaluations[0] as $e | $contrasts[0] as $c | $status[0] as $s |
{
  manifest: ($m.protocol=="E2-one-use-activation-persistence-v1" and $m.seed==984926978049 and
    $m.vocabulary_protocol=="E2-schema-preserving-vocabulary-v1" and $m.attempts==24 and $m.steps_per_attempt==100 and
    $m.requested_step_ceiling==2400 and $m.optimizer_evaluations==0 and $m.held_out_evaluations==0 and $m.old_gate_admission_reused==false),
  attempt_accounting: (($r|length)==24 and ($e|length)==24 and [$r[].evaluation]==[range(0;24)] and
    all(range(0;24);. as $i|$r[$i].seed==$m.seed+($i/6|floor) and $r[$i].arm==["A1","A2","A3"][($i%6/2|floor)] and
      $r[$i].edit==($i%2==1) and $r[$i].requested_steps==100 and
      ($r[$i]|{evaluation,seed,arm,edit,requested_steps,completed_steps,error,accounting_ok})==
      ($e[$i]|{evaluation,seed,arm,edit,requested_steps,completed_steps,error,accounting_ok}))),
  profiles: (all(range(0;24);. as $i|$r[$i] as $receipt|$receipt.effective_config as $cfg|
    $cfg==$m.planned_effective_configs[$i] and $cfg.seed==$receipt.seed and $cfg.ere.arm==$receipt.arm and
    $cfg.rules.inline_toml==$receipt.emission.core_toml and $cfg.ere.seed_families==$receipt.emission.families and
    ($cfg.ere.seed_families|length)==4 and all($cfg.ere.seed_families[];.active==false) and
    $cfg.ere.match_budget_policy=="Pooled" and $cfg.ere.encoding_version=="AlternativeFamiliesV3" and $cfg.ere.epoch.Fixed==10 and
    $cfg.ere.match_ceiling==50000 and $cfg.ere.rule_region_ceiling==500 and $cfg.ere.meta_seed_energy==0 and $cfg.ere.meta_rules_toml==null and
    $cfg.ere.measure_family_maintenance and $cfg.ere.measure_ei==false and $cfg.evaluator.enabled==false and $cfg.causal.enabled==false and
    $cfg.engine.max_steps==100 and $cfg.engine.metrics_window_interval==1 and $cfg.engine.step_mode=="MaxParallel" and
    $cfg.engine.max_pairings_per_step==16 and $cfg.engine.thread_count==1 and
    $cfg.initial.condition_type=="PureCrystal" and $cfg.initial.width==4 and $cfg.initial.height==4 and $cfg.initial.arity==2 and $cfg.initial.periodic and
    $cfg.ere.vocabulary.protocol==$m.vocabulary_protocol and ($cfg.ere.vocabulary.instructions|length)==1 and
    $cfg.ere.vocabulary.instructions[0].operation=="Activate" and $cfg.ere.vocabulary.instructions[0].fuel==(if $receipt.edit then 1 else 0 end))),
  matched_material: (all(range(0;4);. as $seed|$r[$seed*6].emission as $emission|
    all(range(0;6);. as $j|$r[$seed*6+$j].emission==$emission and
      ($r[$seed*6+$j].effective_config|del(.ere.arm,.ere.vocabulary.instructions[0].fuel))==
      ($r[$seed*6].effective_config|del(.ere.arm,.ere.vocabulary.instructions[0].fuel))))),
  measured_steps: (all($r[];. as $receipt|.step_observations as $t|.ere_windows as $w|
    .completed_steps!=null and .completed_steps<=100 and ($t|length)==.completed_steps and ($w|length)>=($t|length) and
    all(range(0;($t|length));. as $i|$t[$i].step==$i+1 and
      $t[$i].rng_before_compile==$t[$i].rng_after_compile and ($t[$i].rng_after_step|type)=="string" and
      $t[$i].match_work<=50000 and $t[$i].match_results<=50000 and $t[$i].match_ceiling_binding==false and
      all($t[$i].world_rule_table[];type=="string") and ($t[$i].families|length)==4 and
      $w[$i].step_start==$i and $w[$i].step_end==$i+1 and
      all(range(0;4);. as $f|$w[$i].seeded_family_maintenance[$f].checks==$i+1 and
        $w[$i].seeded_family_maintenance[$f].final_valid==$t[$i].families[$f].valid and
        $w[$i].seeded_family_maintenance[$f].valid_steps==([$t[0:$i+1][].families[$f]|select(.valid)]|length)) and
      (if $i==0 then true else $t[$i-1].world_hash_after==$t[$i].world_hash_before and $t[$i-1].rng_after_step==$t[$i].rng_before_compile end)))),
  paid_and_credit_accounting: (all($r[];.ere_windows as $w|trace(.) as $trace|paid(.) as $paid|
    .accounting_ok and $paid==($trace|length) and $paid==(if .edit then 1 else 0 end) and
    all($trace[];.operation=="Activate" and .step==1 and .fuel_before==1 and .fuel_after==0) and
    all($w[];.seeded_editor_rewrite_events==0 and .meta_rewrite_events==.ordinary_meta_rewrite_events and
      .encoded_meta_rewrite_events<=.ordinary_meta_rewrite_events and (.encoded_meta_instruction_states|length)==1 and
      .encoded_meta_instruction_states[0].valid and .encoded_meta_instruction_states[0].checks==.step_end and
      .encoded_meta_instruction_states[0].valid_steps==.step_end and .encoded_meta_instruction_states[0].fuel==0))),
  scores: (all($r[];.observation as $o|.ere_windows[-1] as $last|$last.seeded_family_maintenance as $f|
    $last.structural_proposal_reuse_count_by_class as $map|
    ([([$map[]|if .>1 then log else 0 end]|add//0)/(4*((401)|log)),1]|min) as $reuse|
    ([$f[].valid_steps]|add/400) as $time|([$f[]|select(.final_valid)]|length/4) as $final|
    .completed_steps==100 and $o.valid and $o.final_step==100 and $o.families==$f and $o.structural_reuse_by_class==$map and
    close($o.reuse_score;$reuse) and close($o.time_valid_fraction;$time) and close($o.final_valid_fraction;$final) and
    close($o.joint_score;$reuse*$time*$final) and $o.structural_world_fires==([$map[]]|add//0) and
    $o.meta_rewrite_events==([$r[.evaluation].ere_windows[].meta_rewrite_events]|add))),
  contrasts: (($c|length)==4 and all(range(0;4);. as $i|
    $r[$i*6+1] as $a|$r[$i*6+3] as $b|$r[$i*6] as $n|$r[$i*6+2] as $p|$r[$i*6+5] as $a3|
    checks($a;$b;$n;$p) as $checks|(trace($a3)[0].target_family) as $target|
    ([$a3.step_observations[9].families[]|select(.anchor==$target)|(.valid and .active==true)][0]) as $boundary|
    $c[$i].seed==$a.seed and $c[$i].protected_checks==$checks and $c[$i].protected_persistence_pass==all($checks[];.) and
    $c[$i].first_table_difference==firstdiff($a;$b;"world_rule_table") and $c[$i].first_world_difference==firstdiff($a;$b;"world_hash_after") and
    $c[$i].first_proposal_world_fire==firstfire($b) and $c[$i].a3_edited_family_valid_at_boundary==$boundary and
    $c[$i].a3_final_valid_families==([$a3.observation.families[]|select(.final_valid)]|length))),
  decision: ($s.protocol==$m.protocol and $s.attempts==24 and $s.complete==([$r[]|select(complete(.))]|length) and
    $s.errors==24-$s.complete and $s.actual_steps==([$r[].completed_steps]|add) and $s.actual_steps<=2400 and $s.requested_steps==2400 and
    $s.unmeasured_completed_step_attempts==0 and $s.optimizer_evaluation_steps==0 and $s.held_out_evaluation_steps==0 and
    $s.full_training_admitted==false and $s.protected_persistence_pass==($s.complete==24 and all($c[];.protected_persistence_pass)) and
    $s.status==(if $s.protected_persistence_pass then "protected_persistence_control_passed" else "incomplete_or_failed" end))
} as $checks |
{kind:"independent_persistence_receipt_audit_not_full_gate",checks:$checks,accounting_verified:all($checks[];.),
  limits:["Hashes are not stored full graphs", "Step/window observations are not event-level damage attribution", "A2 is engineered protection, not unprotected maintenance", "No training admission"]} |
if .accounting_verified then . else error("persistence receipt mismatch: \(.)") end
