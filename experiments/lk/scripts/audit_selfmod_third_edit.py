#!/usr/bin/env python3
"""Independent reconstruction of three native paid forks and third-descendant durability."""
import argparse,copy,hashlib,json,pathlib,tarfile
from audit_selfmod_birth import State,load,rows,sha,require,graph_stocks
from audit_selfmod_bounded import BoundedState
from audit_selfmod_financing import validate_allocator
from audit_selfmod_second_edit_semantic import pairing
from selfmod_second_edit_common import SecondState,reconstruct,descendants
from selfmod_second_edit_semantic import verify_target as verify_second
from selfmod_third_edit_common import ThirdState,verify_target
from run_selfmod_third_edit import CONTRACT,SEEDS,ARMS

def initial_prefix(prefix,evidence,config,seed):
 initial=load(prefix/'initial.json');require(initial['cfg']==config,'fixed initial config');state=State(initial);validate_allocator(initial,load(prefix/'initial_allocator.json'));require(state.step==0 and len(state.balances)==3 and all(v==dict(capital=60,budget=200) for v in state.balances.values()),'ordinary initialization');n=0
 for r in rows(prefix/'prefix.jsonl'):
  require(r['continuation_step'] is None and not any(b['gene']['member']==0 for b in r['births']),'first funded R1');state.step_row(r);n+=1
 if (prefix/'untriggered.json').exists():require(n==256 and load(prefix/'untriggered.json')==dict(seed=seed,triggered=False,prefix_steps=256) and not (prefix/'fork.json').exists(),'initial missing marker');return n,None
 fork=load(prefix/'fork.json');require(n<256 and fork['seed']==seed,'initial deadline');state.object(fork['pending']);n+=1;state.match_snapshot(fork['checkpoint']);validate_allocator(fork['checkpoint'],fork['allocator'])
 for arm in ['faithful','prune']:
  b=load(prefix/(arm+'_birth.json'));s=b['selection'];require(s['arm']==arm and (s['parent'],s['child'])==(fork['faithful_selection']['parent'],fork['faithful_selection']['child']),'actual initial selected child');pairing(b['row'],fork['faithful_birth']);branch=copy.deepcopy(state);branch.finish(b['row'],s,evidence);branch.match_snapshot(b['initial']);require(b['initial']==load(prefix/(arm+'_initial.json')),'exact initial saved fork');validate_allocator(b['initial'],load(prefix/(arm+'_initial_allocator.json')))
  if arm=='faithful':require(b['row']==fork['faithful_birth'] and s==fork['faithful_selection'],'native initial phase')
 return n,load(prefix/'prune_birth.json')['selection']

def exact_start(stage,source):
 for suffix in ['', '_allocator']:require(sha(stage/f'initial{suffix}.json')==sha(source.with_name(source.stem+suffix+'.json')),'exact initial/allocator stage transition')

def search(prefix,prior,offset,contract,description,founder=None):
 exact_start(prefix,prior);initial=load(prefix/'initial.json');state=ThirdState(initial,'replace8') if founder is not None else SecondState(initial,'replace8');validate_allocator(initial,load(prefix/'initial_allocator.json'));n=0
 def eligible():
  ids=descendants(state.genes,founder) if founder is not None else {id_ for id_,g in state.genes.items() if g['lineage']}
  return [id_ for id_,s in state.balances.items() if id_ in ids and s['capital']>=100]
 for r in rows(prefix/'rows.jsonl'):
  require(r['continuation_step']==offset+n+1,'search continuity');state.object(r);require(not eligible(),'no funded target skipped');state.finish(r);n+=1
 if (prefix/'untriggered.json').exists():require(n==256 and load(prefix/'untriggered.json')==dict(triggered=False,search_steps=256) and not (prefix/'fork.json').exists(),'missing search marker');return n,None
 fork=load(prefix/'fork.json');require(n<256 and fork['search_steps']==n+1 and fork['pending']['continuation_step']==offset+n+1,'first qualifying step');state.object(fork['pending']);n+=1;state.match_snapshot(fork['checkpoint']);validate_allocator(fork['checkpoint'],fork['allocator'])
 if founder is not None:require(fork['founder']==founder,'immutable second founder')
 for arm in ARMS:
  b=load(prefix/(arm+'_birth.json'));s=b['selection'];require((s['parent'],s['child'])==(fork['faithful_selection']['parent'],fork['faithful_selection']['child']) and s['parent']==fork['parent'],'actual paired native target');pairing(b['row'],fork['faithful_birth']);branch=copy.deepcopy(state)
  if founder is None:branch.finish_selected(b['row'],s,arm,contract,description)
  else:branch.finish_selected(b['row'],s,arm,contract,description,founder)
  branch.match_snapshot(b['initial']);require(b['initial']==load(prefix/(arm+'_initial.json')),'exact saved paid fork');validate_allocator(b['initial'],load(prefix/(arm+'_initial_allocator.json')))
  if arm=='faithful':require(b['row']==fork['faithful_birth'] and s==fork['faithful_selection'],'native faithful paired phase')
 check=verify_second if founder is None else verify_target;check(load(prefix/'edited_birth.json'),load(prefix/'faithful_birth.json'),load(prefix/'edited_targeting.json'),contract)
 return n,fork

def audit(out):
 m=load(out/'manifest.json');pre=m['engineering_preflight'];seeds=[7,42] if pre else SEEDS;horizon=2000 if pre else 10000
 require(m==dict(protocol=CONTRACT['protocol'],engineering_preflight=pre,seeds=seeds,arms=ARMS,horizon=horizon,contract=CONTRACT,configs=m['configs'],max_parallel_processes=4,requested_trajectories=len(seeds)*2),'fixed manifest');require(load(out/'contract.json')==CONTRACT,'frozen contract');prov=load(out/'instrument_provenance.json')
 for f,h in prov['artifacts_sha256'].items():require(sha(out/f)==h,'frozen artifact '+f)
 with tarfile.open(out/'source.tar.gz') as t:
  require(set(t.getnames())==set(prov['sources_sha256']),'source inventory')
  for f,h in prov['sources_sha256'].items():require(hashlib.sha256(t.extractfile(f).read()).hexdigest()==h,'frozen source '+f)
  for source,dest in [('docs/selfmod_v20_third_edit_contract.json','contract.json'),('docs/selfmod_v20_third_edit_preregistration.md','preregistration.md')]:require(t.extractfile(source).read()==(out/dest).read_bytes(),'frozen preregistration')
 ref=load(out/'reference_provenance.json');require(set(ref['reference_sha256'])=={str(f.relative_to(out)) for f in (out/'reference').rglob('*') if f.is_file()},'reference inventory')
 for f,h in ref['reference_sha256'].items():require(sha(out/f)==h,'authenticated reference '+f)
 prior=load(out/'reference/v19b_audit.json');done=load(out/'reference/v19b_completion_verification.json');require(prior['audit_passed'] and done['passed'] and load(out/'reference/v19b_status.json')['state']=='complete' and load(out/'reference/v19b_replay_verification.json')['passed'],'validated v19b');require(sha(out/'reference/v19b_audit.json')==done['artifacts_sha256']['audit.json'],'completion anchor');require(all(prior['aggregates'][a]['durable_second_families']==8 for a in ARMS),'prior verdicts unchanged');require(sha(out/'v19b-reference-binary')==ref['reference_binary_sha256']==load(out/'reference/v19b_instrument_provenance.json')['artifacts_sha256']['selfmod-second-edit-semantic'],'earlier-stage frozen executable');require(prov['sources_sha256']['scripts/audit_selfmod_durability.py']==load(out/'reference/v19b_instrument_provenance.json')['sources_sha256']['scripts/audit_selfmod_durability.py'],'unchanged primary criterion');require(load(out/'reference/v19_failed_status.json')['state']=='failed','prior failure retained')
 second_contract=load(out/'reference/v19b_contract.json');require(CONTRACT['second_composite_spec']==second_contract['composite_spec'] and CONTRACT['second_composite_class_hash']==second_contract['composite_class_hash'],'same twice-edited platform');evidences={}
 for role in ['initial_prune','second_edit','third_edit']:
  e=load(out/(role+'_mutation.json'));require(all(e[k]==v for k,v in CONTRACT[role].items()) and sha(out/(role+'_mutation.json'))==load(out/'reference/v11_audit.json')['evidence_sha256']['mutations/'+e['id']+'.json'],'exact v11 provenance '+role);evidences[role]=e
 description=load(out/'composite_description.json');require(description==dict(composite_spec=CONTRACT['composite_spec'],composite_class_hash=CONTRACT['composite_class_hash'],executable=True,object_steps=0),'fixed third class');second_description=dict(composite_spec=CONTRACT['second_composite_spec'],composite_class_hash=CONTRACT['second_composite_class_hash'],executable=True,object_steps=0)
 base=load(out/'reference/v19b_manifest.json')['configs'][0];require(m['configs']==[dict(base,seed=s,steps=10000) for s in seeds],'unaltered physical config');inventory=load(out/'fresh_seed_inventory.json');require(inventory['selected_seeds']==SEEDS and inventory['engineering_only']==pre,'fixed seed inventory')
 if not pre:
  require(inventory['no_overlap'] and all(not set(SEEDS)&set(v['seeds']) for v in inventory['prior_manifests'].values()),'fresh seed panel')
  for f in ['audit.json','engineering_verification.json','source_archive_verification.json','missing_case_verification.json','test_verification.json']:require(load(out/'reference'/('engineering_'+f)).get('passed',load(out/'reference'/('engineering_'+f)).get('audit_passed',False)),'preflight gate '+f)
 records=[];readiness=[];forks=[];routes=[];counts=dict(initial_prefix_steps=0,establishment_steps=0,second_prefix_steps=0,second_establishment_steps=0,third_prefix_steps=0,reference_steps=0)
 for seed,config in zip(seeds,m['configs']):
  d=out/f'seed_{seed}';n,selected=initial_prefix(d/'initial_prefix',evidences['initial_prune'],config,seed);counts['initial_prefix_steps']+=n;kind='initial_untriggered'
  if selected is not None:
   kind='inadmissible'
   if len(graph_stocks(load(d/'initial_prefix/prune_initial.json')))<=8:
    exact_start(d/'establishment',d/'initial_prefix/prune_initial.json');r1=reconstruct(d/'establishment',selected,10000)['focal'];counts['establishment_steps']+=10000;require(r1==load(d/'readiness.json'),'exact first readiness');item=dict(seed=seed,first=r1,second=None);readiness.append(item);kind='not_ready'
    if r1['durability']['durable_success']:
     n,f=search(d/'second_prefix',d/'establishment/final.json',10000,second_contract,second_description);counts['second_prefix_steps']+=n;kind='second_untriggered'
     if f is not None:
      second=load(d/'second_prefix/edited_birth.json')['selection'];exact_start(d/'second_establishment',d/'second_prefix/edited_initial.json');r2=reconstruct(d/'second_establishment',second,10000,second['child'])['focal'];counts['second_establishment_steps']+=10000;require(r2==load(d/'second_readiness.json'),'exact second descendant-family readiness');item['second']=r2;kind='second_not_ready'
      if r2['durability']['durable_success']:
       n,f=search(d/'third_prefix',d/'second_establishment/final.json',20000,CONTRACT,description,second['child']);counts['third_prefix_steps']+=n;kind='third_untriggered'
       if f is not None:
        kind='triggered';forks.append(dict(seed=seed,second_founder=second['child'],search_steps=n,parent=f['parent'],child=f['faithful_selection']['child']))
        if pre:
         counts['reference_steps']+=n;require(list(rows(d/'v19b_third_reference/rows.jsonl'))==list(rows(d/'third_prefix/rows.jsonl'))+[f['faithful_birth']],'native faithful third phase parity')
         for suffix in ['', '_allocator']:require(sha(d/f'v19b_third_reference/final{suffix}.json')==sha(d/f'third_prefix/faithful_initial{suffix}.json'),'native faithful third final')
        for arm in ARMS:
         b=load(d/f'third_prefix/{arm}_birth.json');sel=b['selection'];dest=d/arm;exact_start(dest,d/f'third_prefix/{arm}_initial.json');result=reconstruct(dest,sel,horizon,sel['child']);middle=reconstruct(dest,second,horizon,second['child'])['focal'];records.append(dict(seed=seed,arm=arm,founder=sel['child'],second_founder=second['child'],third_family=result['focal'],second_family=middle,original_family=result['original']))
         if pre:
          require((dest/'rows.jsonl').read_bytes()==(d/(arm+'_split_a/rows.jsonl')).read_bytes()+(d/(arm+'_split_b/rows.jsonl')).read_bytes(),'exact split rows')
          for suffix in ['', '_allocator']:
           require(sha(dest/f'checkpoint_1000{suffix}.json')==sha(d/(arm+f'_split_a/final{suffix}.json')),'split intermediate');require(sha(dest/f'final{suffix}.json')==sha(d/(arm+f'_split_b/final{suffix}.json')),'split final')
         print(f'audited {seed} {arm}',flush=True)
  route=dict(seed=seed,kind=kind);require(load(d/'routing.json')==route,'fixed missing/triggered routing');routes.append(route)
  if kind!='triggered':require(all(not (d/a).exists() for a in ARMS),'no omitted branch admitted')
  for path in ['establishment','second_prefix','second_establishment','third_prefix']:
   allowed={'initial_untriggered':[],'inadmissible':[],'not_ready':['establishment'],'second_untriggered':['establishment','second_prefix'],'second_not_ready':['establishment','second_prefix','second_establishment'],'third_untriggered':['establishment','second_prefix','second_establishment','third_prefix'],'triggered':['establishment','second_prefix','second_establishment','third_prefix']}[kind];require((d/path).exists()==(path in allowed),'no unauthorized extra stage')
 advancing=sum(v for k,v in counts.items() if k!='reference_steps')+len(records)*horizon;counts.update(continuation_steps=len(records)*horizon,scientific_advancing_steps=0 if pre else advancing,engineering_advancing_steps=advancing+counts['reference_steps']+len(records)*horizon if pre else 0,trajectories_completed=len(records),fixed_seed_denominator=len(seeds),routing=routes);require(counts==load(out/'summary.json') and load(out/'status.json')==dict(state='complete',trajectories_completed=len(records)),'complete bounded accounting');require(pre or advancing<=326144,'science bound')
 aggregates={}
 for arm in ARMS:
  group=[r for r in records if r['arm']==arm];n=sum(r['third_family']['durability']['durable_success'] for r in group);aggregates[arm]=dict(durable_third_families=n,replicated=None if pre else n>=6,second_family_durable=sum(r['second_family']['durability']['durable_success'] for r in group),original_family_durable=sum(r['original_family']['durability']['durable_success'] for r in group),third_family_alive=sum(r['third_family']['final_family_copies']>0 for r in group),culture_alive=sum(all(r['third_family']['final_member_copies']) for r in group),completed=len(group),missing=len(seeds)-len(group),fixed_seed_denominator=len(seeds),enumeration_censored=sum(any(w['enumeration_censored'] for w in r['third_family']['windows']) for r in group))
 return dict(audit_passed=True,engineering_preflight=pre,counts=counts,first_readiness_passes=sum(r['first']['durability']['durable_success'] for r in readiness),second_readiness_passes=sum(r['second'] is not None and r['second']['durability']['durable_success'] for r in readiness),aggregates=aggregates,readiness=readiness,records=records,forks=forks,historical_verdicts_changed=False,auditor_sha256=sha(pathlib.Path(__file__)),evidence_sha256={str(f.relative_to(out)):sha(f) for f in sorted(out.rglob('*')) if f.is_file() and f.name not in ['audit.json','replay_verification.json']})

def adversarial(out,report):
 rejected=[]
 if not report['forks']:return rejected
 seed=report['forks'][0]['seed'];d=out/f'seed_{seed}';fork=load(d/'third_prefix/fork.json');born=load(d/'third_prefix/edited_birth.json');faithful=load(d/'third_prefix/faithful_birth.json');target=load(d/'third_prefix/edited_targeting.json');founder=fork['founder'];description=load(out/'composite_description.json')
 for name in ['genotype','paying_parent','payment','retirement','rng','wrong_founder']:
  b=copy.deepcopy(born);f=founder
  if name=='paying_parent':b['selection']['parent']=-1
  elif name=='rng':b['row']['rng_word_pos']+=2
  elif name=='wrong_founder':f=next(g['id'] for g in fork['checkpoint']['genes'] if not g['lineage'])
  else:
   birth=next(x for x in b['row']['births'] if x['child']==b['selection']['child'])
   if name=='genotype':birth['gene']['spec']=CONTRACT['second_composite_spec']
   elif name=='payment':birth['child_tokens']=999
   else:next(a for a in b['row']['bounded']['admissions'] if a['child']==birth['child'])['retired_tokens']=999
  try:ThirdState(fork['checkpoint'],'replace8').finish_selected(b['row'],b['selection'],'edited',CONTRACT,description,f)
  except (ValueError,KeyError):rejected.append(name)
  else:raise ValueError('accepted '+name)
 for name in ['variable','index','original_parameters','effective_parameters','order','child','draw','extra_material']:
  t=copy.deepcopy(target);b=copy.deepcopy(born)
  if name=='variable':t['target_variable']=0
  elif name=='index':t['target_index']=1
  elif name=='original_parameters':t['original_parameters'][0]+=1
  elif name=='effective_parameters':t['effective_parameters'][0]=1
  elif name=='order':t['native_pnode_order'][0]['variable']=0
  elif name=='child':t['child']=-1
  elif name=='draw':t['scheduler_draws']=1
  else:next(n for n in b['initial']['rules']['nodes'] if n['value'] is not None)['value']['energy']=999
  try:verify_target(b,faithful,t,CONTRACT)
  except (ValueError,KeyError,StopIteration):rejected.append(name)
  else:raise ValueError('accepted '+name)
 initial=load(d/'edited/initial.json');first=next(rows(d/'edited/rows.jsonl'))
 for name,edit in [('rent',lambda r:r['roots'][0].__setitem__('cost',999)),('cap',lambda r:r['bounded'].__setitem__('limit',9)),('lineage',lambda r:r['roots'][0].__setitem__('lineage',not r['roots'][0]['lineage']))]:
  bad=copy.deepcopy(first);edit(bad)
  try:BoundedState(initial,'replace8').step_row(bad)
  except (ValueError,KeyError):rejected.append(name)
  else:raise ValueError('accepted '+name)
 state=BoundedState(initial,'replace8')
 for r in rows(d/'edited/rows.jsonl'):
  if r['births']:
   bad=copy.deepcopy(r);bad['births'][0]['gene']['class_hash']=0
   try:copy.deepcopy(state).step_row(bad)
   except (ValueError,KeyError):rejected.append('future_genotype')
   else:raise ValueError('accepted future genotype')
   break
  state.step_row(r)
 return rejected

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('output',type=pathlib.Path);a=p.parse_args();report=audit(a.output);report['adversarial_corruptions_rejected']=adversarial(a.output,report)
 with (a.output/'audit.json').open('x') as f:json.dump(report,f,indent=2);f.write('\n')
 print(json.dumps({k:report[k] for k in ['audit_passed','counts','first_readiness_passes','second_readiness_passes','aggregates']},indent=2))
if __name__=='__main__':main()
