# Usage: jq -s --slurpfile evaluations ... --slurpfile manifest ...
# --slurpfile controls ... --slurpfile distributions ... --slurpfile prechecks ...
# --slurpfile status ... -f this_file rollouts/*.json
# Pure arithmetic receipt audit: no simulation or optimizer execution.
def close($a;$b): if $a==null or $b==null then $a==$b else ($a-$b|fabs)<1e-12 end;
def reuse($map): [$map[]|if .>1 then log else 0 end]|add // 0;
def complete: .error==null and .observation.valid==true and .completed_steps==1000;
def intact: .valid==true and .time_valid_fraction==1 and .final_valid_fraction==1 and .quarantine_decisions==0 and .cleanup_edges_lost==0;

. as $receipts | $evaluations[0] as $e | $manifest[0] as $m | $controls[0] as $c |
$distributions[0] as $d | $prechecks[0] as $p | $status[0] as $s |
([$e[18:]|range(0;length;2) as $i|[.[$i],.[$i+1]]|select(all(.[];complete))]|length) as $pairs |
([$e[]|select(.label=="stateability_baseline")|.observation.joint_score]) as $baseline |
(if all($baseline[];type=="number") then ($baseline|add/4) else null end) as $mean |
(if $mean!=null then ($baseline|map(pow(.-$mean;2))|add/4|sqrt) else null end) as $sd |
([$e[]|select(.label=="forced_structural_active")|.observation.joint_score][0]) as $a |
([$e[]|select(.label=="forced_priority_dormant")|.observation.joint_score][0]) as $b |
(if $a!=null and $b!=null then ($a-$b|fabs) else null end) as $delta |
(if $mean!=null then ([$mean,1-$mean]|min) else null end) as $margin |
([$d.forced_execution|keys[]|select($d.initial_execution[.]==null)]|length) as $distinct |
(([$d.initial_genotype,$d.final_genotype]|map(keys)|add|unique) as $keys |
  [$keys[]|(($d.initial_genotype[.]//0)-($d.final_genotype[.]//0)|fabs)]|add/128) as $tv |
[$distinct>=1,($tv>=0.05 and $pairs==20),($delta!=null and $sd!=null and $delta>2*$sd and $delta>1e-12),($margin!=null and $margin>0.02)] as $passed |
($c|all(.[];.observation.valid==true)) as $controls_valid |
($c|map(select(.arm=="A3"))) as $a3 |
($c|map(select(.arm=="A2"))) as $a2 |
((all($a3[];.observation|intact)) and any($a3[];.observation.structural_world_fires>0)) as $maintained |
(all($a2[];(.observation|intact) and .observation.seeded_editor_events>0)) as $protected |
{
  manifest_profile: ($m.protocol=="E2-maintenance-reuse-pilot-v1" and $m.seed==976337043457 and
    $m.max_evaluations==58 and $m.maximum_simulation_steps==58000 and $m.steps_per_eval==1000 and
    $m.parameters==40 and $m.families==4 and $m.full_campaign=="not_launched" and
    $m.base_config.ere.match_ceiling==50000 and $m.base_config.ere.rule_region_ceiling==500 and
    $m.base_config.ere.measure_family_maintenance==true and $m.base_config.ere.measure_ei==false),
  attempt_accounting: (($e|length)==58 and ($receipts|length)==58 and
    [$e[].evaluation]==[range(0;58)] and [$receipts[].evaluation]==[range(0;58)] and
    all($e[];.requested_steps==1000 and .completed_steps!=null and .completed_steps>=0 and .completed_steps<=1000)),
  section_seeds_and_arms: (
    all(range(0;12);$e[.].label=="matched_control" and $e[.].seed==$m.seed+(. / 3|floor) and $e[.].arm==["A1","A2","A3"][.%3]) and
    all(range(12;16);$e[.].label=="stateability_baseline" and $e[.].seed==$m.seed+.-12 and $e[.].arm=="A3") and
    $e[16].label=="forced_structural_active" and $e[17].label=="forced_priority_dormant" and
    all(range(18;58);$e[.].label==(if .%2==0 then "optimizer_probe_plus" else "optimizer_probe_minus" end) and
      $e[.].seed==$m.seed+((.-18)/8|floor) and $e[.].arm=="A3")),
  raw_receipts_match_log: (all(range(0;58);$receipts[.].observation==$e[.].observation and
    $receipts[.].error==$e[.].error and $receipts[.].completed_steps==$e[.].completed_steps)),
  effective_runtime_profiles: (all(range(0;58);. as $i | $receipts[$i].effective_config as $cfg |
    $cfg.seed==$e[$i].seed and $cfg.engine.max_steps==1000 and $cfg.engine.max_pairings_per_step==16 and
    $cfg.engine.step_mode=="MaxParallel" and $cfg.engine.thread_count==1 and $cfg.engine.metrics_window_interval==100 and
    $cfg.ere.arm==$e[$i].arm and $cfg.ere.encoding_version=="AlternativeFamiliesV3" and
    $cfg.ere.match_budget_policy=="Pooled" and $cfg.ere.match_ceiling==50000 and $cfg.ere.rule_region_ceiling==500 and
    $cfg.ere.epoch.Fixed==10 and $cfg.ere.meta_seed_energy==32 and $cfg.ere.measure_family_maintenance==true and
    $cfg.ere.measure_ei==false and $cfg.causal.enabled==false and $cfg.evaluator.enabled==false and
    $cfg.initial.width==4 and $cfg.initial.height==4 and $cfg.initial.periodic==true and $cfg.initial.arity==2 and
    $cfg.initial.condition_type=="PureCrystal")),
  matched_control_material: (all(range(0;4);. as $i |
    $receipts[$i*3].emission==$receipts[$i*3+1].emission and $receipts[$i*3].emission==$receipts[$i*3+2].emission and
    ($receipts[$i*3].effective_config|del(.ere.arm,.ere.meta_rules_toml))==($receipts[$i*3+1].effective_config|del(.ere.arm,.ere.meta_rules_toml)) and
    ($receipts[$i*3].effective_config|del(.ere.arm,.ere.meta_rules_toml))==($receipts[$i*3+2].effective_config|del(.ere.arm,.ere.meta_rules_toml)))),
  scores_reconstructed: (all($receipts[];
    if .error!=null then .observation==null and .ere_windows==[]
    else .ere_windows as $w | $w[-1] as $last | .observation as $o |
      $last.seeded_family_maintenance as $f | $last.structural_proposal_reuse_count_by_class as $map |
      ((reuse($map))/(4*((1000*.effective_config.engine.max_pairings_per_step/4+1)|log))) as $unclipped |
      ([$unclipped,1]|min) as $r |
      ([$f[].valid_steps]|add/4000) as $t | ([$f[]|select(.final_valid)]|length/4) as $final |
      .completed_steps==1000 and $o.final_step==1000 and $o.valid==true and ($w|length)==10 and ($f|length)==4 and
      [$f[].family_index]==[0,1,2,3] and all($f[];.checks==1000 and .valid_steps<=.checks and .final_valid==(.final_failure==null)) and
      all(range(0;10);. as $i | $w[$i].step_start==$i*100 and $w[$i].step_end==($i+1)*100 and
        $w[$i].match_ceiling_binding==false and $w[$i].rule_region_ceiling_binding==false and
        $w[$i].meta_rewrite_events==$w[$i].seeded_editor_rewrite_events+$w[$i].ordinary_meta_rewrite_events and
        all($w[$i].seeded_family_maintenance[];.checks==$w[$i].step_end)) and
      $o.families==$f and $o.structural_reuse_by_class==$map and
      close($o.reuse_score;$r) and close($o.time_valid_fraction;$t) and close($o.final_valid_fraction;$final) and close($o.joint_score;$r*$t*$final) and
      $o.structural_world_fires==([$map[]]|add//0) and
      $o.seeded_editor_events==([$w[].seeded_editor_rewrite_events]|add) and
      $o.ordinary_meta_events==([$w[].ordinary_meta_rewrite_events]|add) and
      $o.meta_rewrite_events==([$w[].meta_rewrite_events]|add) and $o.meta_rewrite_events==$o.seeded_editor_events+$o.ordinary_meta_events and
      $o.quarantine_decisions==([$w[].rules_quarantined]|add) and $o.cleanup_edges_lost==([$w[].meta_edges_lost_to_cleanup]|add)
    end)),
  controls_and_baseline_repeats: ($c==[$e[]|select(.label=="matched_control")|{seed,arm,observation}] and
    all(range(0;4);$e[12+.].observation==$e[2+.*3].observation)),
  recorded_distribution_counts: (all($d[];([.[]]|add)==64)),
  prechecks_reconstructed: (($p|length)==4 and [$p[].passed]==$passed and $p[0].measured==$distinct and
    close($p[1].measured;$tv) and $p[1].valid_pairs==$pairs and close($p[2].measured;$delta) and
    close($p[2].threshold;(if $sd==null then null else 2*$sd end)) and close($p[3].measured;$margin)),
  decision_reconstructed: ($s.stateability_pass==all($passed[];.) and $s.controls_valid==$controls_valid and
    $s.unprotected_maintenance_pass==$maintained and $s.protected_instrument_control_pass==$protected and
    $s.ready_for_full_campaign_decision==(all($passed[];.) and $controls_valid and $maintained and $protected) and
    $s.known_completed_steps==([$e[].completed_steps]|add) and $s.evaluations_attempted==58 and
    $s.requested_step_upper_bound==58000 and $s.unmeasured_completed_step_attempts==0 and
    $s.full_optimizer_generations_completed==0 and $s.full_optimizer_evaluation_steps==0 and $s.held_out_evaluation_steps==0)
} as $checks |
{kind:"independent_arithmetic_audit_not_research_pass",checks:$checks,
 accounting_verified:all($checks[];.),evaluations_attempted:($e|length),
 complete_evaluations:([$e[]|select(complete)]|length),errors:[$e[]|select(.error!=null)|{evaluation,completed_steps,error}],
 known_completed_steps:([$e[].completed_steps]|add),requested_step_upper_bound:58000,
 valid_probe_pairs:$pairs,baseline_mean:$mean,baseline_population_sd:$sd,emission_tv:$tv,
 stateability_pass:all($passed[];.),unprotected_maintenance_pass:$maintained,
 limits:["No physics rerun", "Validity counters audited arithmetically, not reconstructed from saved graph snapshots", "No partial ERE history retained on halted rollouts"]} |
if .accounting_verified then . else error("maintenance receipt mismatch: \(.)") end
