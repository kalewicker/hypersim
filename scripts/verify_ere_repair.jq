# jq -s --slurpfile manifest OUT/manifest.json --slurpfile contrasts OUT/contrasts.json
# --slurpfile status OUT/status.json -f scripts/verify_ere_repair.jq OUT/rollouts/*.json
def close($a;$b): ($a-$b|fabs)<1e-12;
def complete: .error==null and .accounting_ok and .completed_steps==100 and (.step_observations|length)==100;
def traces: [.ere_windows[].encoded_meta_operation_traces[]];
def decision($g):
  {all_six_complete:all($g[];complete),
   combined_all_final_valid:($g[3].observation.final_valid_fraction==1),
   combined_measured_recovery:(([$g[3].observation.families[].recovery_transitions]|add)>0),
   combined_paid_repair:any($g[3]|traces[];.operation=="Repair"),
   combined_greater_valid_time:($g[3].observation.time_valid_fraction>$g[0].observation.time_valid_fraction),
   combined_proposal_world_fires:($g[3].observation.structural_world_fires>0),
   protected_references_intact:all([4,5][];$g[.].observation.time_valid_fraction==1 and $g[.].observation.final_valid_fraction==1)};
. as $r|$manifest[0] as $m|$contrasts[0] as $c|$status[0] as $s|
{
 protocol_budget:($m.protocol=="E2-witness-repair-v2" and $m.seed==984927043585 and $m.attempts==24 and
   $m.steps_per_attempt==100 and $m.requested_step_ceiling==2400 and $m.optimizer_evaluations==0 and $m.held_out_evaluations==0 and
   $m.old_gate_admission_reused==false and ($r|length)==24 and [$r[].evaluation]==[range(0;24)]),
 profiles_material:all(range(0;24);. as $i|$r[$i] as $a|$a.effective_config as $cfg|$cfg.ere.vocabulary as $v|
   $a.seed==$m.seed+($i/6|floor) and $a.condition==$m.conditions[$i%6] and $a.requested_steps==100 and
   $cfg==$m.planned_effective_configs[$i] and $cfg.ere.protocol==null and $v.protocol==$m.protocol and
   $cfg.ere.encoding_version=="AlternativeFamiliesV3" and $cfg.ere.match_budget_policy=="Pooled" and $cfg.ere.epoch.Fixed==10 and
   $cfg.ere.match_ceiling==50000 and $cfg.ere.rule_region_ceiling==500 and $cfg.ere.meta_seed_energy==0 and
   $cfg.ere.meta_rules_toml==null and $cfg.ere.measure_family_maintenance and $cfg.ere.measure_ei==false and
   $cfg.engine.step_mode=="MaxParallel" and $cfg.engine.thread_count==1 and $cfg.engine.max_pairings_per_step==16 and
   $cfg.engine.max_steps==100 and $cfg.engine.metrics_window_interval==1 and $cfg.evaluator.enabled==false and $cfg.causal.enabled==false and
   $cfg.initial.condition_type=="PureCrystal" and $cfg.initial.width==4 and $cfg.initial.height==4 and $cfg.initial.periodic and $cfg.initial.arity==2 and
   $cfg.ere.arm==(if $i%6<4 then "A3" else "A2" end) and
   ($v.instructions|map(.operation))==["WitnessActivate","Seal","Seal","Repair","Repair"] and
   ($v.instructions|map(.fuel))==[1,(if ([1,3,5]|index($i%6))!=null then 16 else 0 end),(if ([1,3,5]|index($i%6))!=null then 16 else 0 end),
      (if ([2,3,5]|index($i%6))!=null then 32 else 0 end),(if ([2,3,5]|index($i%6))!=null then 32 else 0 end)] and
   $cfg.ere.seed_families==$a.emission.families and ($a.emission.families|length)==4 and all($a.emission.families[];.active==false) and
   $cfg.rules.inline_toml==$a.emission.core_toml and $a.emission==$r[($i/6|floor)*6].emission and
   ($cfg|del(.experiment_name,.ere.arm,.ere.vocabulary.instructions[].fuel))==
     ($r[($i/6|floor)*6].effective_config|del(.experiment_name,.ere.arm,.ere.vocabulary.instructions[].fuel))),
 completed_measurements:all($r[];. as $a|.step_observations as $t|.ere_windows as $w|
   .completed_steps!=null and ($t|length)==.completed_steps and ($w|length)==.completed_steps and .completed_steps<=100 and
   all(range(0;($t|length));. as $i|$t[$i].step==$i+1 and $t[$i].rng_before_compile==$t[$i].rng_after_compile and
     $t[$i].match_ceiling_binding==false and $t[$i].match_work<=50000 and $t[$i].match_results<=50000 and
     all($t[$i].world_rule_table[];type=="string") and ($t[$i].families|length)==4 and
     $w[$i].step_start==$i and $w[$i].step_end==$i+1 and $w[$i].rule_region_ceiling_binding==false and
     (if $i==0 then true else $t[$i-1].rng_after_step==$t[$i].rng_before_compile and $t[$i-1].world_hash_after==$t[$i].world_hash_before end) and
     all(range(0;4);. as $f|$w[$i].seeded_family_maintenance[$f] as $state|
       $state.checks==$i+1 and $state.final_valid==$t[$i].families[$f].valid and
       $state.valid_steps==([$t[0:$i+1][].families[$f]|select(.valid)]|length) and
       $state.recovery_transitions==([range(1;$i+1) as $j|select($t[$j-1].families[$f].valid==false and $t[$j].families[$f].valid)]|length)))),
 fuel_and_attribution:all($r[];. as $a|traces as $t|
   (.ere_windows[0].encoded_meta_instruction_states|map(.instruction)) as $ids|
   $a.effective_config.ere.vocabulary.instructions as $init|
   all(.ere_windows[];. as $w|.seeded_editor_rewrite_events==0 and .meta_rewrite_events==.ordinary_meta_rewrite_events and
      .encoded_meta_rewrite_events==(.encoded_meta_operation_traces|length) and .encoded_meta_rewrite_events<=.ordinary_meta_rewrite_events and
      (.encoded_meta_instruction_states|length)==5 and
      all(range(0;5);. as $i|$w.encoded_meta_instruction_states[$i] as $state|
        $state.instruction==$ids[$i] and $state.valid and $state.checks==$w.step_end and $state.valid_steps==$w.step_end and
        $state.operation==$init[$i].operation and $state.fuel==$init[$i].fuel-([$t[]|select(.instruction==$ids[$i] and .step<=$w.step_end)]|length))) and
   all(range(0;5);. as $i|[$t[]|select(.instruction==$ids[$i])] as $events|
      all(range(0;($events|length));. as $j|$events[$j].operation==$init[$i].operation and
        $events[$j].fuel_before==$init[$i].fuel-$j and $events[$j].fuel_after==$init[$i].fuel-$j-1 and $events[$j].fuel_after>=0)) and
   all($t[];.instruction as $id|($ids|index($id))!=null) and .accounting_ok),
 scores:all($r[];.observation as $o|.ere_windows[-1] as $last|$last.seeded_family_maintenance as $f|
   $last.structural_proposal_reuse_count_by_class as $map|
   ([([$map[]|if .>1 then log else 0 end]|add//0)/(4*(401|log)),1]|min) as $reuse|
   ([$f[].valid_steps]|add/400) as $time|([$f[]|select(.final_valid)]|length/4) as $final|
   .completed_steps==100 and $o.valid and $o.families==$f and $o.structural_reuse_by_class==$map and
   close($o.reuse_score;$reuse) and close($o.time_valid_fraction;$time) and close($o.final_valid_fraction;$final) and
   close($o.joint_score;$reuse*$time*$final) and $o.structural_world_fires==([$map[]]|add//0)),
 contrasts: (($c|length)==4 and all(range(0;4);. as $i|$r[$i*6:$i*6+6] as $g|decision($g) as $check|
    $c[$i].seed==$g[0].seed and $c[$i].checks==$check and $c[$i].combined_control_pass==all($check[];.) and
    $c[$i].conditions==[$g[]|{condition,observation,accounting_ok,error}])),
 status:($s.protocol==$m.protocol and $s.attempts==24 and $s.complete==([$r[]|select(complete)]|length) and $s.errors==24-$s.complete and
    $s.actual_steps==([$r[].completed_steps]|add) and $s.actual_steps<=2400 and $s.requested_steps==2400 and
    $s.optimizer_evaluation_steps==0 and $s.held_out_evaluation_steps==0 and $s.full_training_admitted==false and
    $s.combined_control_pass==all($c[];.combined_control_pass) and $s.status==(if $s.combined_control_pass then "bounded_combined_control_passed" else "incomplete_or_failed" end))
} as $checks|
{kind:"independent_witness_repair_receipt_audit_not_full_gate",checks:$checks,accounting_verified:all($checks[];.)}|
if .accounting_verified then . else error("witness repair receipt mismatch: \(.)") end
