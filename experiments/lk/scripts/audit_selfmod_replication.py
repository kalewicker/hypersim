#!/usr/bin/env python3
"""Audit fresh prefix selection plus unchanged v14 policy/finance verdicts."""
import argparse,copy,hashlib,json,pathlib,statistics,tarfile
from audit_selfmod_birth import State,load,rows,sha,require,new_totals,add_totals,score
from audit_selfmod_turnover import SwitchedState
from audit_selfmod_bounded import BoundedState,ARMS,extra_new,extra_add,capital,reproduction,adversarial
SEEDS=[131,137,149,157,163,173,181,191]
def fixed_success_count(records,arm):return sum(r['reproduction']['joint_success'] for r in records if r['arm']==arm)
def audit(out):
 m,summary,status,prov=(load(out/f) for f in ['manifest.json','summary.json','status.json','instrument_provenance.json']);pre=m['engineering_preflight'];seeds=[7,42] if pre else SEEDS;horizon=64 if pre else 2000
 configs={c['seed']:c for c in m['configs']}
 require(m['protocol']=='selfmod-v15-replication-v1' and m['seeds']==seeds and m['horizon']==horizon and m['arms']==ARMS and m['limit']==8 and m['prefix_limit']==256 and m['continuations_requested']==len(seeds)*8 and m['replication_threshold']==6 and not m['future_mutation'],'frozen design')
 require(sha(out/'selfmod-replicate')==prov['binary_sha256'] and sha(out/'source.tar.gz')==prov['source_archive_sha256'] and sha(out/'reference_provenance.json')==prov['reference_provenance_sha256'],'instrument hashes')
 with tarfile.open(out/'source.tar.gz') as tar:
  require(set(tar.getnames())==set(prov['sources_sha256']),'source archive inventory')
  for f,h in prov['sources_sha256'].items():require(hashlib.sha256(tar.extractfile(f).read()).hexdigest()==h,'source '+f)
  require(tar.extractfile('docs/selfmod_v15_replication_preregistration.md').read()==(out/'preregistration.md').read_bytes(),'frozen contract')
 ref=load(out/'reference_provenance.json')
 require(set(ref['reference_sha256'])=={str(p.relative_to(out)) for p in (out/'reference').rglob('*') if p.is_file()},'reference inventory')
 for f,h in ref['reference_sha256'].items():require(sha(out/f)==h,'reference '+f)
 prior=load(out/'reference/audit.json');require(prior['audit_passed'] and load(out/'reference/status.json')['state']=='complete','validated v14 reference')
 evidence=load(out/'v11_selected_mutation.json');require(sha(out/'v11_selected_mutation.json')==ref['selected_v11_evidence_sha256'] and m['prune_parameters']==evidence['parameters'],'named v11 variant')
 if not pre:
  require(load(out/'reference/replay_verification.json')['passed'],'v14 replay validation')
  require(load(out/'reference/v15_preflight/audit.json')['audit_passed'] and load(out/'reference/v15_preflight/status.json')['state']=='complete' and load(out/'reference/v15_preflight/summary.json')['historical_preflight_exact'],'historical v15 engineering parity')
  inventory=load(out/'fresh_seed_inventory.json');require(sha(out/'fresh_seed_inventory.json')==prov['fresh_seed_inventory_sha256'] and inventory['selected_seeds']==SEEDS and inventory['no_overlap'],'fresh seed inventory')
  for entry in inventory['prior_manifests'].values():require(not set(entry['seeds'])&set(SEEDS),'historical seed overlap')
 for cfg in configs.values():
  require(cfg['steps']==horizon and cfg['seed_pop']=='churn3' and cfg['copy_number']==1 and cfg['e0']==60 and cfg['fission_mode']=='full'
   and cfg['birth_cost']==50 and cfg['repro_threshold']==100 and cfg['rent']==1 and cfg['income_pool']==12 and cfg['income_mode']=='proportional'
   and cfg['influx']==20 and cfg['free_cap']==500 and cfg['creation_budget_init']==cfg['creation_budget_max']==200 and cfg['creation_refill']==1
   and cfg['tie_break']=='random' and cfg['match_cost_coeff']==0 and cfg['match_cap_per_rule']==64 and cfg['match_enum_budget']==50000
   and cfg['world_nodes']==200 and cfg['world_edges']==300 and cfg['world_ceiling']==500 and cfg['edge_ceiling']==1000 and cfg['node_explosion_cap']==20000
   and all(cfg[k]==0 for k in ['mu','abio','p_dup','p_del']),'unchanged operating point')
 records=[];prefix_steps=0;triggered=0;missing=[];forks=[]
 for seed in seeds:
  directory=out/f'seed_{seed}';initial=load(directory/'initial.json');require(initial['cfg']==configs[seed],'seed/config initialization');state=State(initial)
  require(len(state.balances)==3 and all(v==dict(capital=60,budget=200) for v in state.balances.values()),'ordinary initial endowment')
  require(state.cash()==dict(free=500,bound=180,cumulative_influx=680,cumulative_dissipated=0,audit_residual=0),'initial flux ledger')
  for row in rows(directory/'prefix.jsonl'):
   require(row['continuation_step'] is None and not any(b['gene']['member']==0 for b in row['births']),'first R1 trigger not skipped');state.step_row(row);prefix_steps+=1
  if (directory/'untriggered.json').exists():
   require(state.step==256 and load(directory/'untriggered.json')==dict(seed=seed,prefix_steps=256,reason='no natural R1 birth') and not (directory/'fork.json').exists(),'untriggered routing')
   missing.extend(dict(seed=seed,arm=arm,reason='untriggered') for arm in ARMS);continue
  fork=load(directory/'fork.json');require(fork['seed']==seed and state.step<256,'native trigger limit');state.object(fork['pending']);prefix_steps+=1;state.match_snapshot(fork['checkpoint']);triggered+=1
  births={}
  for genotype in ['faithful','prune']:
   birth=load(directory/f'{genotype}_birth.json');selected=birth['selection'];require(selected['arm']==genotype and selected['parent']==fork['faithful_selection']['parent'] and selected['child']==fork['faithful_selection']['child'],'paired selected identities')
   for key in ['absolute_step','roots','label_counts_before','world_nodes_before','world_edges_before','world_nodes','world_edges','world_hash','rng_word_pos','rng_probe','ledger']:
    require(birth['row'][key]==fork['faithful_birth'][key],'paid birth-phase pairing '+key)
   require(len(birth['row']['births'])==len(fork['faithful_birth']['births']),'paid sibling count')
   for b,original in zip(birth['row']['births'],fork['faithful_birth']['births']):
    for key in ['parent','child','parent_before','parent_after_native','child_tokens','inherited_budget','coin','operator','parameters']:require(b[key]==original[key],'paired native transfer/draw '+key)
   branch=copy.deepcopy(state);branch.finish(birth['row'],selected,evidence);branch.match_snapshot(birth['initial']);require(birth['initial']['cfg']==configs[seed],'post-birth config unchanged')
   if genotype=='faithful':require(birth['row']==fork['faithful_birth'] and selected==fork['faithful_selection'],'native faithful fork')
   births[genotype]=birth
  require(len(State(births['faithful']['initial']).balances)==len(State(births['prune']['initial']).balances),'population paired')
  n=len(State(births['faithful']['initial']).balances);forks.append(dict(seed=seed,birth_step=state.step,post_birth_population=n,cap_admissible=n<=8))
  for arm in ARMS:
   genotype,policy=arm.split('_');path=directory/arm;birth=births[genotype];selected=birth['selection']
   if policy in ['vacancy8','replace8'] and n>8:
    require(load(path/'inadmissible.json')==dict(seed=seed,arm=arm,population=n,limit=8,reason='initial population exceeds bound') and not (path/'rows.jsonl').exists(),'over-cap routing without cull')
    missing.append(dict(seed=seed,arm=arm,reason='initial population exceeds bound'));continue
   require(sha(path/'birth.json')==sha(directory/f'{genotype}_birth.json'),'policy common paid birth')
   branch=BoundedState(birth['initial'],policy) if policy in ['vacancy8','replace8'] else SwitchedState(birth['initial'],policy=='on');totals=new_totals();extra=extra_new();completed=0
   for row in rows(path/'rows.jsonl'):
    completed+=1;require(row['continuation_step']==completed,'fixed continuation order')
    if completed==horizon//2+1:
     extra['late_start_capital']=capital(branch);extra['late_min_capital']=extra['late_start_capital'];extra['late_max_capital']=extra['late_start_capital'];extra['late_start_recorded']=True
    if policy in ['on','off']:require('bounded' not in row,'native control schema')
    branch.step_row(row);add_totals(totals,row,selected,horizon);extra_add(extra,branch,row,horizon)
   require(completed==horizon,'full horizon');legacy=score(branch,selected,totals,completed);joint=reproduction(legacy,extra)
   require(legacy==load(path/'score.json') and joint==load(path/'reproduction.json'),'unchanged legacy/joint verdicts');branch.match_snapshot(load(path/'final.json'))
   if pre:
    for file in ['birth.json','rows.jsonl','score.json','reproduction.json','final.json']:
     reference=out/f'reference/seed_{seed}/{arm}/{file}';require(sha(path/file)==sha(reference) and sha(reference)==prior['evidence_sha256'][f'seed_{seed}/{arm}/{file}'],'v14 exact preflight '+file)
   records.append(dict(seed=seed,arm=arm,score=legacy,reproduction=joint))
 total=prefix_steps+len(records)*horizon
 require(len(records)+len(missing)==len(seeds)*8,'fixed requested panel coverage')
 require(summary==dict(protocol=m['protocol'],engineering_preflight=pre,triggered_seeds=triggered,continuations_completed=len(records),continuation_steps=len(records)*horizon,
  shared_prefix_steps=prefix_steps,scientific_advancing_steps=total,fixed_replication_denominator=len(seeds),missing=missing,results=records,historical_preflight_exact=pre,historical_results_pooled=False),'scientific accounting/fixed denominator')
 require(status==dict(state='complete',continuations_completed=len(records),scientific_advancing_steps=total),'completed status')
 aggregates={}
 for arm in ARMS:
  group=[r for r in records if r['arm']==arm];s=[r['score'] for r in group];j=[r['reproduction'] for r in group];rates=[v['lineage_per_copy_rate'] for v in s if v['lineage_per_copy_rate'] is not None];success=fixed_success_count(records,arm)
  aggregates[arm]=dict(requested=len(seeds),completed=len(group),missing=sum(r['arm']==arm for r in missing),fixed_denominator=len(seeds),legacy_established=sum(v['bounded_establishment'] for v in s),joint_success=success,
   replicated_joint_success=None if pre else success>=6,lineage_alive=sum(v['final_lineage_copies']>0 for v in s),cycle_persistent=sum(v['whole_cycle_persistent'] for v in s),enumeration_censored=sum(v['totals']['enumeration_skips']>0 for v in s),
   resource_gated=sum(v['totals']['resource_gates']>0 for v in s),reproduces_every_late_window=sum(all(v['metrics']['late_birth_windows']) for v in j),lineage_reproduces_late=sum(v['metrics']['late_lineage_births']>0 for v in j),
   retirement_financing_covered=sum(v['retirement_adjusted_late_surplus']>=0 for v in j),births=sum(v['metrics']['births'] for v in j),retirements=sum(v['metrics']['retirements'] for v in j),mean_per_copy_rate=statistics.mean(rates) if rates else None,
   max_boundary_population=max((v['metrics']['peak_boundary_population'] for v in j),default=None))
 return dict(audit_passed=True,engineering_preflight=pre,continuations_completed=len(records),scientific_advancing_steps=total,shared_prefix_steps=prefix_steps,continuation_steps=len(records)*horizon,
  fixed_replication_denominator=len(seeds),historical_results_pooled=False,aggregates=aggregates,forks=forks,missing=missing,records=records,auditor_sha256=sha(pathlib.Path(__file__)),
  scope='Independent native prefix/first-trigger, paired birth, money/gate/genealogy/RNG/cap/retirement, legacy/joint verdict and fixed-denominator reconstruction; native matcher/mutator/hash/ChaCha tested and replayed.',
  evidence_sha256={str(p.relative_to(out)):sha(p) for p in sorted(out.rglob('*')) if p.is_file() and p.name not in ['audit.json','replay_verification.json']})

def adversarial_replication(out):
 m=load(out/'manifest.json');eligible=[s for s in m['seeds'] if (out/f'seed_{s}/faithful_replace8/rows.jsonl').exists()]
 rejected=[]
 if eligible:rejected=adversarial(out,eligible[0])
 # Prefix trigger falsification must be rejected even without an admissible continuation.
 for seed in m['seeds']:
  path=out/f'seed_{seed}'
  if (path/'fork.json').exists():
   fork=load(path/'fork.json');birth=load(path/'faithful_birth.json');damaged=copy.deepcopy(birth['row']);damaged['births'][0]['parent']=-1
   try:State(fork['checkpoint']).finish(damaged,birth['selection'],load(out/'v11_selected_mutation.json'))
   except (ValueError,KeyError):rejected.append('fresh_trigger_parent')
   else:raise ValueError('accepted falsified fresh birth parent')
   break
 # Fixed denominator applies to all eight seeds even when only a subset is measured.
 synthetic=[dict(seed=s,arm='prune_replace8',reproduction=dict(joint_success=True)) for s in SEEDS[:5]]
 damaged=dict(fixed_denominator=5,replicated_joint_success=True)
 expected=dict(fixed_denominator=len(SEEDS),replicated_joint_success=fixed_success_count(synthetic,'prune_replace8')>=6)
 try:require(damaged==expected,'missing seeds promoted replication')
 except ValueError:rejected.append('missing_seeds_false_replication')
 else:raise ValueError('accepted shrunken replication denominator')
 return rejected

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('output',type=pathlib.Path);args=p.parse_args();report=audit(args.output);report['adversarial_corruptions_rejected']=adversarial_replication(args.output)
 with (args.output/'audit.json').open('x') as f:json.dump(report,f,indent=2);f.write('\n')
 print(json.dumps({k:report[k] for k in ['audit_passed','continuations_completed','scientific_advancing_steps','shared_prefix_steps','forks','missing','aggregates','adversarial_corruptions_rejected']},indent=2))
if __name__=='__main__':main()
