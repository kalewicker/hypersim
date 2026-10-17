#!/usr/bin/env python3
"""Audit fixed v18 named variants, paid forks and unchanged v17 durability."""
import argparse,copy,hashlib,json,pathlib,tarfile
from audit_selfmod_birth import State,load,rows,sha,require,new_totals,add_totals,score
from audit_selfmod_turnover import SwitchedState
from audit_selfmod_bounded import BoundedState,capital,extra_new,extra_add,reproduction
from audit_selfmod_financing import window,add_window,finish_window,validate_allocator,slope,flow
from run_selfmod_variants import SEEDS,ARMS,VARIANTS,CONTRACT
from audit_selfmod_durability import members,family_copies,mark_floor,durability

def audit(out):
 m=load(out/'manifest.json');pre=m['engineering_preflight'];seeds=[7,42] if pre else SEEDS;horizon=2000 if pre else 10000
 expected=dict(protocol='selfmod-v18-variants-v1',engineering_preflight=pre,seeds=seeds,arms=ARMS,horizon=horizon,physical_scoring_horizon=10000,warmup_steps=2000,window_steps=1000,subwindow_steps=250,prefix_limit=256,limit=8,replication_threshold=6,stock_floor='rent_times_live_family_copies',future_mutation=False,requested_trajectories=len(seeds)*4,max_parallel_processes=4,configs=m['configs'],named_variants=CONTRACT['named_variants'])
 require(m==expected,'frozen protocol')
 prov=load(out/'instrument_provenance.json');ref=load(out/'reference_provenance.json')
 require(sha(out/'selfmod-variants')==prov['binary_sha256'] and sha(out/'source.tar.gz')==prov['source_archive_sha256'] and sha(out/'reference_provenance.json')==prov['reference_provenance_sha256'] and sha(out/'fresh_seed_inventory.json')==prov['seed_inventory_sha256'],'frozen instrument')
 with tarfile.open(out/'source.tar.gz') as tar:
  require(set(tar.getnames())==set(prov['sources_sha256']),'source inventory')
  for file,digest in prov['sources_sha256'].items():require(hashlib.sha256(tar.extractfile(file).read()).hexdigest()==digest,'source '+file)
  require(tar.extractfile('docs/selfmod_v18_variants_preregistration.md').read()==(out/'preregistration.md').read_bytes(),'frozen contract')
  require(tar.extractfile('docs/selfmod_v18_variants_contract.json').read()==(out/'contract.json').read_bytes(),'frozen machine contract')
 require(load(out/'contract.json')==CONTRACT and sha(out/'contract.json')==prov['contract_sha256'],'fixed named contract')
 require(set(ref['reference_sha256'])=={str(p.relative_to(out)) for p in (out/'reference').rglob('*') if p.is_file()},'reference inventory')
 for file,digest in ref['reference_sha256'].items():require(sha(out/file)==digest,'reference '+file)
 prior=load(out/'reference/v17_audit.json');require(prior['audit_passed'] and load(out/'reference/v17_status.json')['state']=='complete' and load(out/'reference/v17_replay_verification.json')['passed'] and load(out/'reference/v17_completion_verification.json')['passed'],'validated v17')
 require(prior['aggregates']['prune_replace8']['durable_success']==8 and prior['old_v15_prune_replace_joint']==5 and not prior['old_v15_primary_changed'],'historical verdicts unchanged')
 require(sha(out/'reference/v17_audit.json')==load(out/'reference/v17_completion_verification.json')['artifacts_sha256']['audit.json'],'v17 completion anchor')
 require(sha(out/'v17-reference-binary')==ref['reference_binary_sha256']==load(out/'reference/v17_instrument_provenance.json')['binary_sha256'],'frozen v17 reference binary')
 require(prov['sources_sha256']['scripts/audit_selfmod_durability.py']==load(out/'reference/v17_instrument_provenance.json')['sources_sha256']['scripts/audit_selfmod_durability.py'],'unchanged durability criterion implementation')
 evidences={}
 for genotype,identity in CONTRACT['named_variants'].items():
  evidence=load(out/(genotype+'_mutation.json'));require(all(evidence[k]==v for k,v in identity.items()),'fixed candidate identity')
  require(sha(out/(genotype+'_mutation.json'))==ref['named_v11_sha256'][genotype]==load(out/'reference/v11_audit.json')['evidence_sha256']['mutations/'+identity['id']+'.json'],'authenticated v11 candidate')
  evidences[genotype]=evidence
 base=load(out/'reference/v17_manifest.json')['configs'][0];require(m['configs']==[dict(base,seed=s,steps=10000) for s in seeds],'unchanged fresh operating point')
 inventory=load(out/'fresh_seed_inventory.json');require(inventory['selected_seeds']==SEEDS and inventory['engineering_only']==pre,'fresh inventory')
 if not pre:
  require(inventory['no_overlap'] and all(not set(v['seeds'])&set(SEEDS) for v in inventory['prior_manifests'].values()),'new independent seeds')
  require(load(out/'reference/engineering_audit.json')['audit_passed'] and load(out/'reference/engineering_engineering_verification.json')['passed'],'validated engineering')
  require(load(out/'reference/engineering_missing_case_verification.json')['passed'] and load(out/'reference/engineering_source_archive_verification.json')['passed'],'engineering rejection and archive checks')
 configs={c['seed']:c for c in m['configs']};records=[];missing=[];forks=[];prefix_steps=0;prefix_results=[]
 for seed in seeds:
  directory=out/f'seed_{seed}';initial=load(directory/'initial.json');require(initial['cfg']==configs[seed],'initial config');state=State(initial);validate_allocator(initial,load(directory/'initial_allocator.json'))
  require(state.step==0 and len(state.balances)==3 and all(v==dict(capital=60,budget=200) for v in state.balances.values()) and members(state)==[1,1,1],'ordinary fresh population')
  require(state.cash()==dict(free=500,bound=180,cumulative_influx=680,cumulative_dissipated=0,audit_residual=0),'ordinary initial flux ledger')
  for row in rows(directory/'prefix.jsonl'):
   require(row['continuation_step'] is None and not any(b['gene']['member']==0 for b in row['births']),'first R1 trigger not skipped');state.step_row(row);prefix_steps+=1
  if (directory/'untriggered.json').exists():
   require(state.step==256 and load(directory/'untriggered.json')==dict(seed=seed,triggered=False,prefix_steps=256) and not (directory/'fork.json').exists(),'untriggered routing');prefix_results.append(dict(seed=seed,triggered=False,prefix_steps=256));missing.extend(dict(seed=seed,arm=arm,reason='untriggered') for arm in ARMS);continue
  fork=load(directory/'fork.json');require(fork['seed']==seed and state.step<256,'trigger deadline');state.object(fork['pending']);prefix_steps+=1;state.match_snapshot(fork['checkpoint']);validate_allocator(fork['checkpoint'],fork['allocator']);prefix_results.append(dict(seed=seed,triggered=True,prefix_steps=state.step));births={}
  for genotype in VARIANTS:
   evidence=evidences.get(genotype)
   birth=load(directory/f'{genotype}_birth.json');selected=birth['selection'];require(selected['arm']==('faithful' if genotype=='faithful' else 'prune') and selected['parent']==fork['faithful_selection']['parent'] and selected['child']==fork['faithful_selection']['child'],'paired child identity')
   for key in ['absolute_step','roots','label_counts_before','world_nodes_before','world_edges_before','world_nodes','world_edges','world_hash','rng_word_pos','rng_probe','ledger']:require(birth['row'][key]==fork['faithful_birth'][key],'paired phase '+key)
   require(len(birth['row']['births'])==len(fork['faithful_birth']['births']),'siblings retained')
   for b,original in zip(birth['row']['births'],fork['faithful_birth']['births']):
    for key in ['parent','child','parent_before','parent_after_native','child_tokens','inherited_budget','coin','operator','parameters']:require(b[key]==original[key],'paired payment/draw '+key)
   branch=copy.deepcopy(state);branch.finish(birth['row'],selected,evidence);branch.match_snapshot(birth['initial']);require(birth['initial']==load(directory/f'{genotype}_initial.json') and birth['initial']['cfg']==configs[seed],'exact paid start');validate_allocator(birth['initial'],load(directory/f'{genotype}_initial_allocator.json'))
   if genotype=='faithful':require(birth['row']==fork['faithful_birth'] and selected==fork['faithful_selection'],'native faithful fork')
   births[genotype]=birth
  n=len(State(births['faithful']['initial']).balances);require(all(n==len(State(births[g]['initial']).balances) for g in VARIANTS),'paired population');forks.append(dict(seed=seed,birth_step=state.step,post_birth_population=n,cap_admissible=n<=8))
  for arm in ARMS:
   genotype,policy=arm.split('_');dest=directory/arm;birth=births[genotype];selected=birth['selection']
   if policy in ['vacancy8','replace8'] and n>8:
    require(load(dest/'inadmissible.json')==dict(seed=seed,arm=arm,population=n,limit=8) and not (dest/'rows.jsonl').exists(),'over-cap route without culling');missing.append(dict(seed=seed,arm=arm,reason='initial population exceeds bound'));continue
   require(sha(dest/'initial.json')==sha(directory/f'{genotype}_initial.json') and sha(dest/'initial_allocator.json')==sha(directory/f'{genotype}_initial_allocator.json'),'exact allocator-complete policy start')
   branch=BoundedState(birth['initial'],policy) if policy in ['vacancy8','replace8'] else SwitchedState(birth['initial'],policy=='on');windows=[];segments=[];completed=0
   for row in rows(dest/'rows.jsonl'):
    require(row['continuation_step']==completed+1,'fixed continuous offset');local=completed%2000+1
    if completed%1000==0:w=window(completed+1,branch);mark_floor(w,branch);w.update(family_creation_spend=0,culture_creation_spend=0,world_node_sum=0,peak_world_nodes=0)
    if local==1:totals=new_totals();extra=extra_new()
    if local==1001:extra.update(late_start_capital=capital(branch),late_min_capital=capital(branch),late_max_capital=capital(branch),late_start_recorded=True)
    branch.step_row(row);completed+=1;w['family_creation_spend']+=sum(r['spend'] for r in row['roots'] if r['lineage']);w['culture_creation_spend']+=sum(r['spend'] for r in row['roots']);w['world_node_sum']+=row['world_nodes'];w['peak_world_nodes']=max(w['peak_world_nodes'],row['world_nodes']);mark_floor(w,branch);add_window(w,row,branch);local_row=dict(row,continuation_step=local);add_totals(totals,local_row,selected,2000);extra_add(extra,branch,local_row,2000)
    if completed%1000==0:
     snap=load(dest/f'checkpoint_{completed}.json');require(snap['cfg']==configs[seed],'checkpoint physical config');branch.match_snapshot(snap);validate_allocator(snap,load(dest/f'checkpoint_{completed}_allocator.json'));w['mean_world_nodes']=w.pop('world_node_sum')/1000;windows.append(finish_window(w,branch))
    if local==2000:
     view=copy.copy(branch);view.cfg=dict(branch.cfg,steps=2000);legacy=score(view,selected,totals,2000);joint=reproduction(legacy,extra);segments.append(dict(first_step=completed-1999,last_step=completed,score=legacy,reproduction=joint))
   require(completed==horizon,'full fixed horizon');final=load(dest/'final.json');require(final['cfg']==configs[seed],'unchanged final physical config');branch.match_snapshot(final);validate_allocator(final,load(dest/'final_allocator.json'));require(load(dest/'status.json')==dict(state='complete',offset=0,steps_completed=horizon),'branch completion')
   primary=durability(selected,windows,completed==10000)
   if pre:
    for suffix in ['', '_allocator']:
     require(sha(dest/f'checkpoint_1000{suffix}.json')==sha(directory/f'{arm}_split_a/final{suffix}.json'),'split intermediate '+suffix)
     require(sha(dest/f'final{suffix}.json')==sha(directory/f'{arm}_split_b/final{suffix}.json'),'split final '+suffix)
    require((dest/'rows.jsonl').read_bytes()==(directory/f'{arm}_split_a/rows.jsonl').read_bytes()+(directory/f'{arm}_split_b/rows.jsonl').read_bytes(),'split rows')
   records.append(dict(seed=seed,arm=arm,durability=primary,windows=windows,segments=segments,final_lineage_copies=family_copies(branch),final_member_copies=members(branch),capital_mean_slope_per_1000=slope([w['mean_capital'] for w in windows[2:]]),phase_capital_mean_slope_per_1000=slope([w['retirement_phase_summary']['mean'] for w in windows[2:]])))
   print(f'audited {seed} {arm}',flush=True)
 reference_prefix_steps=0
 if pre:
  for seed in seeds:
   baseline=out/f'v17_seed_{seed}';directory=out/f'seed_{seed}';reference_prefix_steps+=load(baseline/'fork.json')['pending']['absolute_step']
   for f in ['initial.json','initial_allocator.json','prefix.jsonl','fork.json']+[g+'_'+v+'.json' for g in ['faithful','prune'] for v in ['birth','initial','initial_allocator']]:require(sha(directory/f)==sha(baseline/f),'frozen v17 exact prefix '+f)
  engineering=load(out/'engineering_verification.json');require(engineering['passed'] and engineering['frozen_v17_prefix_parity'] and engineering['reference_prefix_steps']==reference_prefix_steps and engineering['branches']==len(records),'engineering scope')
 require(load(out/'prefix_results.json')==prefix_results,'prefix accounting');require(len(records)+len(missing)==len(seeds)*4,'fixed panel coverage')
 summary=dict(trajectories_completed=len(records),continuation_steps=len(records)*horizon,shared_prefix_steps=prefix_steps,scientific_advancing_steps=0 if pre else len(records)*horizon+prefix_steps,engineering_advancing_steps=len(records)*horizon*2+prefix_steps+reference_prefix_steps if pre else 0,fixed_seed_denominator=len(seeds),missing=missing)
 require(load(out/'summary.json')==summary and load(out/'status.json')==dict(state='complete',trajectories_completed=len(records),continuation_steps=len(records)*horizon,shared_prefix_steps=prefix_steps),'summary/completion accounting')
 aggregates={}
 for arm in ARMS:
  group=[r for r in records if r['arm']==arm];success=sum(r['durability']['durable_success'] for r in group)
  aggregates[arm]=dict(requested=len(seeds),completed=len(group),missing=sum(r['arm']==arm for r in missing),fixed_seed_denominator=len(seeds),durable_success=success,durability_replicated=None if pre else success>=6,
   final_lineages_alive=sum(r['final_lineage_copies']>0 for r in group),final_cycles_alive=sum(all(r['final_member_copies']) for r in group),segment_joint_counts=[sum(r['segments'][i]['reproduction']['joint_success'] for r in group) for i in range(horizon//2000)],
   segment_establishment_counts=[sum(r['segments'][i]['score']['bounded_establishment'] for r in group) for i in range(horizon//2000)],culture_births=sum(sum(sum(w['birth_windows']) for w in r['windows']) for r in group),family_births=sum(sum(sum(w['lineage_birth_windows']) for w in r['windows']) for r in group),enumeration_censored=sum(any(w['enumeration_censored'] for w in r['windows']) for r in group))
 return dict(audit_passed=True,engineering_preflight=pre,scientific_advancing_steps=summary['scientific_advancing_steps'],engineering_advancing_steps=summary['engineering_advancing_steps'],continuation_steps=summary['continuation_steps'],shared_prefix_steps=prefix_steps,fixed_seed_denominator=len(seeds),old_v15_primary_changed=False,old_v15_prune_replace_joint=5,aggregates=aggregates,forks=forks,missing=missing,records=records,auditor_sha256=sha(pathlib.Path(__file__)),evidence_sha256={str(p.relative_to(out)):sha(p) for p in sorted(out.rglob('*')) if p.is_file() and p.name not in ['audit.json','replay_verification.json']})

def adversarial(out,report):
 rejected=[];eligible=[r['seed'] for r in report['forks'] if r['cap_admissible']];seed=eligible[0] if eligible else None
 if seed is not None:
  initial=load(out/f'seed_{seed}/faithful_replace8/initial.json');row=next(rows(out/f'seed_{seed}/faithful_replace8/rows.jsonl'))
  for name,edit in [('rent',lambda r:r['roots'][0].__setitem__('cost',999)),('rng',lambda r:r.__setitem__('rng_word_pos',r['rng_word_pos']+2)),('cap',lambda r:r['bounded'].__setitem__('limit',9))]:
   damaged=copy.deepcopy(row);edit(damaged)
   try:BoundedState(initial,'replace8').step_row(damaged)
   except (ValueError,KeyError):rejected.append(name)
   else:raise ValueError('accepted corrupted '+name)
  state=BoundedState(initial,'replace8')
  for actual in rows(out/f'seed_{seed}/faithful_replace8/rows.jsonl'):
   if actual['births']:
    for name,edit in [('birth_payment',lambda r:r['births'][0].__setitem__('child_tokens',999)),('retired_donor',lambda r:r['bounded']['admissions'][0].__setitem__('parent',-1))]:
     damaged=copy.deepcopy(actual);edit(damaged)
     try:copy.deepcopy(state).step_row(damaged)
     except (ValueError,KeyError):rejected.append(name)
     else:raise ValueError('accepted corrupted '+name)
    break
   state.step_row(actual)
  fork=load(out/f'seed_{seed}/fork.json');birth=load(out/f'seed_{seed}/faithful_birth.json');damaged=copy.deepcopy(birth['row']);damaged['births'][0]['parent']=-1
  try:State(fork['checkpoint']).finish(damaged,birth['selection'],load(out/'prune_mutation.json'))
  except (ValueError,KeyError):rejected.append('fresh_trigger_parent')
  else:raise ValueError('accepted false trigger parent')
  for name in ['candidate_spec','candidate_class','candidate_selection']:
   fork=load(out/f'seed_{seed}/fork.json');birth=load(out/f'seed_{seed}/matchless_birth.json');damaged=copy.deepcopy(birth)
   if name=='candidate_selection':damaged['selection']['changed_class']=False
   else:
    target=next(b for b in damaged['row']['births'] if b['child']==damaged['selection']['child'])
    if name=='candidate_spec':target['gene']['spec']=load(out/'extra_mutation.json')['decoded_spec']
    else:target['gene']['class_hash']=0
   try:State(fork['checkpoint']).finish(damaged['row'],damaged['selection'],load(out/'matchless_mutation.json'))
   except (ValueError,KeyError):rejected.append(name)
   else:raise ValueError('accepted false candidate '+name)
 synthetic=[dict(durable_success=True) for _ in range(5)]
 expected=dict(fixed_seed_denominator=8,durability_replicated=sum(r['durable_success'] for r in synthetic)>=6)
 damaged=dict(fixed_seed_denominator=5,durability_replicated=True)
 try:require(damaged==expected,'missing seeds promoted replication')
 except ValueError:rejected.append('missing_seeds_false_replication')
 else:raise ValueError('accepted false denominator/verdict')
 return rejected

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('output',type=pathlib.Path);args=p.parse_args();report=audit(args.output);report['adversarial_corruptions_rejected']=adversarial(args.output,report)
 with (args.output/'audit.json').open('x') as f:json.dump(report,f,indent=2);f.write('\n')
 print(json.dumps(dict(audit_passed=True,steps=report['scientific_advancing_steps'],prefix=report['shared_prefix_steps'],missing=report['missing'],aggregates=report['aggregates']),indent=2))
if __name__=='__main__':main()
