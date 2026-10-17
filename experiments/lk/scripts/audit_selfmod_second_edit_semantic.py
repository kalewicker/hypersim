#!/usr/bin/env python3
"""Independently reconstruct readiness, bounded second births and descendant durability."""
import argparse,copy,hashlib,json,pathlib,tarfile
from audit_selfmod_birth import State,load,rows,sha,require,graph_stocks
from audit_selfmod_bounded import BoundedState
from audit_selfmod_financing import validate_allocator
from selfmod_second_edit_common import SecondState,reconstruct,descendants
from run_selfmod_second_edit_semantic import CONTRACT,SEEDS,ARMS
from selfmod_second_edit_semantic import verify_target

def pairing(actual,reference):
 for key in ['absolute_step','continuation_step','roots','stocks','ledger','world_hash','world_nodes_before','world_edges_before','world_nodes','world_edges','label_counts_before','rng_word_pos','rng_probe','rule_cap_binding','bounded']:
  if key in reference:require(actual[key]==reference[key],'paired complete phase '+key)
 require(len(actual['births'])==len(reference['births']),'paired siblings')
 for a,b in zip(actual['births'],reference['births']):
  for k in ['parent','child','parent_before','parent_after_native','child_tokens','inherited_budget','coin','operator','parameters','cancelled']:require(a[k]==b[k],'paired native payment/draw '+k)

def audit(out):
 m=load(out/'manifest.json');pre=m['engineering_preflight'];seeds=[7,42] if pre else SEEDS;horizon=2000 if pre else 10000
 require(m==dict(protocol=CONTRACT['protocol'],engineering_preflight=pre,seeds=seeds,arms=ARMS,horizon=horizon,contract=CONTRACT,configs=m['configs'],max_parallel_processes=4,requested_trajectories=len(seeds)*2),'fixed protocol')
 prov=load(out/'instrument_provenance.json')
 for f,d in prov['artifacts_sha256'].items():require(sha(out/f)==d,'frozen artifact '+f)
 require(load(out/'contract.json')==CONTRACT,'fixed contract')
 with tarfile.open(out/'source.tar.gz') as t:
  require(set(t.getnames())==set(prov['sources_sha256']),'source inventory')
  for f,d in prov['sources_sha256'].items():require(hashlib.sha256(t.extractfile(f).read()).hexdigest()==d,'frozen source '+f)
  require(t.extractfile('docs/selfmod_v19b_second_edit_preregistration.md').read()==(out/'preregistration.md').read_bytes(),'frozen prose')
  require(t.extractfile('docs/selfmod_v19b_second_edit_contract.json').read()==(out/'contract.json').read_bytes(),'frozen JSON')
 ref=load(out/'reference_provenance.json')
 require(set(ref['reference_sha256'])=={str(p.relative_to(out)) for p in (out/'reference').rglob('*') if p.is_file()},'reference inventory')
 for f,d in ref['reference_sha256'].items():require(sha(out/f)==d,'authenticated reference '+f)
 prior=load(out/'reference/v18_audit.json');require(prior['audit_passed'] and load(out/'reference/v18_status.json')['state']=='complete' and load(out/'reference/v18_replay_verification.json')['passed'],'validated v18')
 require(load(out/'reference/v19_failed_status.json')['state']=='failed' and load(out/'reference/v19_failed_status.json')['trajectories_completed']==0,'preserved failed v19')
 require(load(out/'reference/v19_interrupted_run_verification.json')['completed_stages_verified'] and not load(out/'reference/v19_interrupted_run_verification.json')['full_assay_complete'],'interrupted stages only')
 correction=load(out/'reference/v19_semantic_target_verification.json');require(correction['passed'] and correction['object_steps']==0 and correction['both_native_orders_verified'] and correction['new_scientific_assay_started']==False,'zero-step semantic correction provenance')
 require(sha(out/'reference/v18_audit.json')==load(out/'reference/v18_completion_verification.json')['artifacts_sha256']['audit.json'],'v18 completion anchor')
 require(sha(out/'v18-reference-binary')==ref['reference_binary_sha256']==load(out/'reference/v18_instrument_provenance.json')['binary_sha256'],'frozen v18 binary')
 require(prior['aggregates']['prune_replace8']['durable_success']==8 and prior['old_v15_prune_replace_joint']==5,'historical verdicts preserved')
 require(prov['sources_sha256']['scripts/audit_selfmod_durability.py']==load(out/'reference/v18_instrument_provenance.json')['sources_sha256']['scripts/audit_selfmod_durability.py'],'unchanged durability implementation')
 evidences={}
 for role in ['initial_prune','second_edit']:
  e=load(out/(role+'_mutation.json'));require(all(e[k]==v for k,v in CONTRACT[role].items()),'named mutation identity');require(sha(out/(role+'_mutation.json'))==ref['mutation_sha256'][role]==load(out/'reference/v11_audit.json')['evidence_sha256']['mutations/'+e['id']+'.json'],'v11 authentication');evidences[role]=e
 description=load(out/'composite_description.json');require(description['composite_spec']==CONTRACT['composite_spec'] and description['executable'] and description['object_steps']==0 and description['composite_class_hash']==CONTRACT['composite_class_hash'] and description['composite_class_hash']!=CONTRACT['initial_prune']['mutant_class_hash'],'fixed changed composite')
 base=load(out/'reference/v18_manifest.json')['configs'][0];require(m['configs']==[dict(base,seed=s,steps=10000) for s in seeds],'unchanged physical configs');configs={c['seed']:c for c in m['configs']}
 inventory=load(out/'fresh_seed_inventory.json');require(inventory['selected_seeds']==SEEDS and inventory['engineering_only']==pre,'fixed inventory')
 if not pre:
  require(inventory['no_overlap'] and all(not set(x['seeds'])&set(SEEDS) for x in inventory['prior_manifests'].values()),'fresh seeds')
  for f in ['audit.json','engineering_verification.json','missing_case_verification.json','source_archive_verification.json','semantic_target_verification.json']:
   e=load(out/'reference'/('engineering_'+f));require(e.get('passed',e.get('audit_passed',False)),'validated engineering '+f)
 records=[];ready_records=[];routing=[];first_steps=0;establishment_steps=0;search_steps=0;reference_steps=0;forks=[]
 for seed in seeds:
  directory=out/f'seed_{seed}';prefix=directory/'initial_prefix';initial=load(prefix/'initial.json');require(initial['cfg']==configs[seed],'fresh physical config');state=State(initial);validate_allocator(initial,load(prefix/'initial_allocator.json'))
  require(state.step==0 and len(state.balances)==3 and all(v==dict(capital=60,budget=200) for v in state.balances.values()),'ordinary initialization')
  count=0
  for row in rows(prefix/'prefix.jsonl'):
   require(row['continuation_step'] is None and not any(b['gene']['member']==0 for b in row['births']),'first native R1 birth');state.step_row(row);count+=1
  if (prefix/'untriggered.json').exists():
   require(count==256 and load(prefix/'untriggered.json')==dict(seed=seed,triggered=False,prefix_steps=256) and not (prefix/'fork.json').exists(),'initial untriggered routing');first_steps+=count;route=dict(seed=seed,kind='initial_untriggered');require(load(directory/'routing.json')==route,'route');routing.append(route);continue
  fork=load(prefix/'fork.json');require(count<256 and fork['seed']==seed,'initial deadline');state.object(fork['pending']);count+=1;first_steps+=count;state.match_snapshot(fork['checkpoint']);validate_allocator(fork['checkpoint'],fork['allocator'])
  for arm in ['faithful','prune']:
   b=load(prefix/(arm+'_birth.json'));sel=b['selection'];require(sel['arm']==arm and (sel['parent'],sel['child'])==(fork['faithful_selection']['parent'],fork['faithful_selection']['child']),'initial paired target');pairing(b['row'],fork['faithful_birth']);branch=copy.deepcopy(state);branch.finish(b['row'],sel,evidences['initial_prune']);branch.match_snapshot(b['initial']);require(b['initial']==load(prefix/(arm+'_initial.json')),'initial fork');validate_allocator(b['initial'],load(prefix/(arm+'_initial_allocator.json')))
   if arm=='faithful':require(b['row']==fork['faithful_birth'] and sel==fork['faithful_selection'],'initial faithful phase')
  n=len(graph_stocks(load(prefix/'prune_initial.json')))
  if n>8:
   route=dict(seed=seed,kind='inadmissible');require(load(directory/'routing.json')==route and not (directory/'establishment').exists(),'no initial cull');routing.append(route);continue
  selected=load(prefix/'prune_birth.json')['selection'];est=directory/'establishment'
  for suffix in ['', '_allocator']:require(sha(est/f'initial{suffix}.json')==sha(prefix/f'prune_initial{suffix}.json'),'exact establishment start')
  readiness=reconstruct(est,selected,10000)['focal'];establishment_steps+=10000;require(readiness==load(directory/'readiness.json'),'independent readiness verdict');ready_records.append(dict(seed=seed,readiness=readiness))
  if pre:
   old=directory/'v18_reference';reference_steps+=len(list(rows(old/'prefix/prefix.jsonl')))+1+10000
   for f in ['initial.json','initial_allocator.json','prefix.jsonl','fork.json']+[g+'_'+part+'.json' for g in ['faithful','prune'] for part in ['birth','initial','initial_allocator']]:require(sha(prefix/f)==sha(old/'prefix'/f),'v18 prefix parity '+f)
   for f in ['rows.jsonl','final.json','final_allocator.json']+[f'checkpoint_{s}{suffix}.json' for s in range(1000,10001,1000) for suffix in ['', '_allocator']]:require(sha(est/f)==sha(old/'establishment'/f),'v18 readiness parity '+f)
  if not readiness['durability']['durable_success']:
   route=dict(seed=seed,kind='not_ready');require(load(directory/'routing.json')==route and not (directory/'second_prefix').exists(),'failed readiness retained');routing.append(route);continue
  second=directory/'second_prefix';initial=load(second/'initial.json');require(initial==load(est/'final.json') and sha(second/'initial_allocator.json')==sha(est/'final_allocator.json'),'exact ready checkpoint');state=SecondState(initial,'replace8');count=0
  for row in rows(second/'rows.jsonl'):
   require(row['continuation_step']==10001+count,'second search offset');state.object(row);require(not any(state.genes[id_]['lineage'] and balance['capital']>=100 for id_,balance in state.balances.items()),'no eligible family birth skipped');state.finish(row);count+=1
  if (second/'untriggered.json').exists():
   require(count==256 and load(second/'untriggered.json')==dict(triggered=False,search_steps=256) and not (second/'fork.json').exists(),'second deadline');search_steps+=count;route=dict(seed=seed,kind='second_untriggered');require(load(directory/'routing.json')==route,'second missing route');routing.append(route);continue
  fork=load(second/'fork.json');require(count<256 and fork['search_steps']==count+1 and fork['pending']['continuation_step']==10001+count,'first search trigger');state.object(fork['pending']);count+=1;search_steps+=count;state.match_snapshot(fork['checkpoint']);validate_allocator(fork['checkpoint'],fork['allocator'])
  verify_target(load(second/'edited_birth.json'),load(second/'faithful_birth.json'),load(second/'edited_targeting.json'),CONTRACT)
  for arm in ARMS:
   b=load(second/(arm+'_birth.json'));sel=b['selection'];require((sel['parent'],sel['child'])==(fork['faithful_selection']['parent'],fork['faithful_selection']['child']) and sel['parent']==fork['parent'],'paired second child');pairing(b['row'],fork['faithful_birth']);branch=copy.deepcopy(state);branch.finish_selected(b['row'],sel,arm,CONTRACT,description);branch.match_snapshot(b['initial']);require(b['initial']==load(second/(arm+'_initial.json')),'exact second paid state');validate_allocator(b['initial'],load(second/(arm+'_initial_allocator.json')))
   if arm=='faithful':require(b['row']==fork['faithful_birth'] and sel==fork['faithful_selection'],'native faithful second fork')
   dest=directory/arm
   for suffix in ['', '_allocator']:require(sha(dest/f'initial{suffix}.json')==sha(second/f'{arm}_initial{suffix}.json'),'exact second continuation start')
   scored=reconstruct(dest,sel,horizon,sel['child']);records.append(dict(seed=seed,arm=arm,founder=sel['child'],second_family=scored['focal'],original_family=scored['original']))
   if pre:
    require((dest/'rows.jsonl').read_bytes()==(directory/(arm+'_split_a/rows.jsonl')).read_bytes()+(directory/(arm+'_split_b/rows.jsonl')).read_bytes(),'split exact rows')
    for suffix in ['', '_allocator']:
     require(sha(dest/f'checkpoint_1000{suffix}.json')==sha(directory/(arm+f'_split_a/final{suffix}.json')),'split intermediate');require(sha(dest/f'final{suffix}.json')==sha(directory/(arm+f'_split_b/final{suffix}.json')),'split final')
   print(f'audited {seed} {arm}',flush=True)
  if pre:
   ref=directory/'v18_reference/second_search';reference_steps+=count;require(list(rows(ref/'rows.jsonl'))==list(rows(second/'rows.jsonl'))+[fork['faithful_birth']],'native bounded search and faithful birth parity')
   for suffix in ['', '_allocator']:require(sha(ref/f'final{suffix}.json')==sha(second/f'faithful_initial{suffix}.json'),'v18 faithful second state')
  route=dict(seed=seed,kind='triggered');require(load(directory/'routing.json')==route,'triggered route');routing.append(route);forks.append(dict(seed=seed,search_steps=count,parent=fork['parent'],child=fork['faithful_selection']['child']))
 for route in routing:
  if route['kind']!='triggered':require(all(not (out/f"seed_{route['seed']}"/arm).exists() for arm in ARMS),'missing worlds have no treatment branches')
 counts=dict(initial_prefix_steps=first_steps,establishment_steps=establishment_steps,second_prefix_steps=search_steps,continuation_steps=len(records)*horizon,scientific_advancing_steps=0 if pre else first_steps+establishment_steps+search_steps+len(records)*horizon,engineering_advancing_steps=first_steps+establishment_steps+search_steps+reference_steps+len(records)*horizon*2 if pre else 0,reference_steps=reference_steps,trajectories_completed=len(records),fixed_seed_denominator=len(seeds),routing=routing)
 require(counts==load(out/'summary.json') and load(out/'status.json')==dict(state='complete',trajectories_completed=len(records)),'scope/step/denominator accounting')
 if pre:require(load(out/'engineering_verification.json')==dict(passed=True,exact_splits=len(records),frozen_v18_initial_readiness_second_birth_parity=True,engineering_advancing_steps=counts['engineering_advancing_steps'],excluded_from_science=True),'engineering scope')
 aggregates={}
 for arm in ARMS:
  group=[r for r in records if r['arm']==arm];n=sum(r['second_family']['durability']['durable_success'] for r in group)
  aggregates[arm]=dict(durable_second_families=n,replicated=None if pre else n>=6,original_family_durable=sum(r['original_family']['durability']['durable_success'] for r in group),second_family_alive=sum(r['second_family']['final_family_copies']>0 for r in group),original_family_alive=sum(r['original_family']['final_family_copies']>0 for r in group),culture_alive=sum(all(r['second_family']['final_member_copies']) for r in group),completed=len(group),missing=len(seeds)-len(group),fixed_seed_denominator=len(seeds),enumeration_censored=sum(not r['second_family']['durability']['uncensored_measured'] for r in group) if not pre else None)
 return dict(audit_passed=True,engineering_preflight=pre,counts=counts,readiness_passes=sum(r['readiness']['durability']['durable_success'] for r in ready_records),aggregates=aggregates,readiness=ready_records,forks=forks,records=records,historical_verdicts_changed=False,auditor_sha256=sha(pathlib.Path(__file__)),evidence_sha256={str(p.relative_to(out)):sha(p) for p in sorted(out.rglob('*')) if p.is_file() and p.name not in ['audit.json','replay_verification.json']})

def adversarial(out,report):
 rejected=[]
 if report['forks']:
  seed=report['forks'][0]['seed'];directory=out/f'seed_{seed}';fork=load(directory/'second_prefix/fork.json');b=load(directory/'second_prefix/edited_birth.json')
  for name in ['second_genotype','second_parent','second_payment','second_retirement','second_rng']:
   damaged=copy.deepcopy(b)
   if name=='second_parent':damaged['selection']['parent']=-1
   elif name=='second_rng':damaged['row']['rng_word_pos']+=2
   else:
    target=next(x for x in damaged['row']['births'] if x['child']==damaged['selection']['child'])
    if name=='second_genotype':target['gene']['spec']=CONTRACT['initial_prune']['decoded_spec']
    elif name=='second_payment':target['child_tokens']=999
    else:next(a for a in damaged['row']['bounded']['admissions'] if a['child']==target['child'])['retired_tokens']=999
   try:SecondState(fork['checkpoint'],'replace8').finish_selected(damaged['row'],damaged['selection'],'edited',CONTRACT,load(out/'composite_description.json'))
   except (ValueError,KeyError):rejected.append(name)
   else:raise ValueError('accepted corrupted '+name)
  initial=load(directory/'edited/initial.json');r=next(rows(directory/'edited/rows.jsonl'))
  for name,edit in [('rent',lambda x:x['roots'][0].__setitem__('cost',999)),('cap',lambda x:x['bounded'].__setitem__('limit',9)),('lineage_flag',lambda x:x['roots'][0].__setitem__('lineage',not x['roots'][0]['lineage']))]:
   bad=copy.deepcopy(r);edit(bad)
   try:BoundedState(initial,'replace8').step_row(bad)
   except (ValueError,KeyError):rejected.append(name)
   else:raise ValueError('accepted corrupted '+name)
  state=BoundedState(initial,'replace8')
  for actual in rows(directory/'edited/rows.jsonl'):
   if actual['births']:
    bad=copy.deepcopy(actual);bad['births'][0]['gene']['class_hash']=0
    try:copy.deepcopy(state).step_row(bad)
    except (ValueError,KeyError):rejected.append('future_genotype')
    else:raise ValueError('accepted corrupted future genotype')
    break
   state.step_row(actual)
  faithful=load(directory/'second_prefix/faithful_birth.json');target=load(directory/'second_prefix/edited_targeting.json')
  for name in ['semantic_variable','semantic_index','semantic_original_parameters','semantic_effective_parameters','semantic_order','semantic_child','semantic_rng','semantic_extra_material']:
   damaged=copy.deepcopy(target);edited=copy.deepcopy(b)
   if name=='semantic_variable':damaged['target_variable']=1
   elif name=='semantic_index':damaged['target_index']=1-damaged['target_index']
   elif name=='semantic_original_parameters':damaged['original_parameters'][0]+=1
   elif name=='semantic_effective_parameters':damaged['effective_parameters'][0]=1-damaged['effective_parameters'][0]
   elif name=='semantic_order':damaged['native_pnode_order'].reverse()
   elif name=='semantic_child':damaged['child']=-1
   elif name=='semantic_rng':damaged['scheduler_draws']=1
   else:
    node=next(n for n in edited['initial']['rules']['nodes'] if n['value'] is not None and n['value']['label']==1000);node['value']['energy']=999
   try:verify_target(edited,faithful,damaged,CONTRACT)
   except (ValueError,KeyError,StopIteration):rejected.append(name)
   else:raise ValueError('accepted corrupted '+name)
 return rejected

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('output',type=pathlib.Path);args=p.parse_args();report=audit(args.output);report['adversarial_corruptions_rejected']=adversarial(args.output,report)
 with (args.output/'audit.json').open('x') as f:json.dump(report,f,indent=2);f.write('\n')
 print(json.dumps(dict(audit_passed=True,counts=report['counts'],readiness_passes=report['readiness_passes'],aggregates=report['aggregates']),indent=2))
if __name__=='__main__':main()
