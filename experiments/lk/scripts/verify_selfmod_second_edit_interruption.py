#!/usr/bin/env python3
"""Read-only verification of completed v19 stages; no full-assay verdict."""
import argparse,copy,hashlib,json,pathlib,tarfile
from audit_selfmod_birth import State,load,rows,sha,require
from audit_selfmod_financing import validate_allocator
from selfmod_second_edit_common import SecondState,reconstruct
from audit_selfmod_second_edit import pairing

def native_order(snapshot,child):
 def key(node):return (node['version']<<32)|node['idx']
 ids=[]
 for slot in snapshot['rules']['edges']:
  e=slot['value']
  if e is not None and e['label']==2002 and child in [key(n) for n in e['nodes']]:ids.extend(key(n) for n in e['nodes'] if key(n)!=child)
 result=[]
 for id_ in sorted(ids):
  slot=snapshot['rules']['nodes'][id_&0xffffffff];require(slot['version']==id_>>32,'native pnode version')
  result.append(dict(id=id_,variable=dict(slot['value']['properties'])[3]['Int']))
 return result

def verify(out):
 manifest=load(out/'manifest.json');contract=load(out/'contract.json');description=load(out/'composite_description.json');status=load(out/'status.json')
 require(status['state']=='failed' and status['trajectories_completed']==0 and 'seed_313' in status['error'] and 'var_labels' in status['error'],'specific interrupted failure retained')
 require(not manifest['engineering_preflight'] and manifest['seeds']==contract['seeds'] and len(manifest['seeds'])==8,'fixed scientific panel')
 prov=load(out/'instrument_provenance.json')
 for f,d in prov['artifacts_sha256'].items():require(sha(out/f)==d,'frozen artifact '+f)
 with tarfile.open(out/'source.tar.gz') as archive:
  require(set(archive.getnames())==set(prov['sources_sha256']),'frozen source inventory')
  for f,d in prov['sources_sha256'].items():require(hashlib.sha256(archive.extractfile(f).read()).hexdigest()==d,'frozen source '+f)
 ref=load(out/'reference_provenance.json')
 for f,d in ref['reference_sha256'].items():require(sha(out/f)==d,'preserved reference '+f)
 require(sha(out/'v18-reference-binary')==ref['reference_binary_sha256'],'reference binary')
 require(load(out/'fresh_seed_inventory.json')['no_overlap'],'original fresh inventory')
 evidence=load(out/'initial_prune_mutation.json');records=[];first_steps=0;search_steps=0
 for seed in manifest['seeds']:
  directory=out/f'seed_{seed}';prefix=directory/'initial_prefix';initial=load(prefix/'initial.json');state=State(initial);validate_allocator(initial,load(prefix/'initial_allocator.json'));count=0
  require(state.step==0 and initial['cfg']==next(c for c in manifest['configs'] if c['seed']==seed),'ordinary frozen initialization')
  for row in rows(prefix/'prefix.jsonl'):
   require(not any(b['gene']['member']==0 for b in row['births']),'first R1 birth');state.step_row(row);count+=1
  fork=load(prefix/'fork.json');state.object(fork['pending']);count+=1;first_steps+=count
  require(count<=256 and fork['seed']==seed,'first prefix deadline');state.match_snapshot(fork['checkpoint']);validate_allocator(fork['checkpoint'],fork['allocator'])
  for arm in ['faithful','prune']:
   born=load(prefix/(arm+'_birth.json'));pairing(born['row'],fork['faithful_birth']);branch=copy.deepcopy(state);branch.finish(born['row'],born['selection'],evidence);branch.match_snapshot(born['initial']);validate_allocator(born['initial'],load(prefix/(arm+'_initial_allocator.json')))
   require(born['initial']==load(prefix/(arm+'_initial.json')),'first paid fork')
  selected=load(prefix/'prune_birth.json')['selection'];est=directory/'establishment'
  for suffix in ['', '_allocator']:require(sha(est/f'initial{suffix}.json')==sha(prefix/f'prune_initial{suffix}.json'),'exact establishment input')
  readiness=reconstruct(est,selected,10000)['focal']
  if (directory/'readiness.json').exists():require(readiness==load(directory/'readiness.json'),'saved readiness verdict')
  record=dict(seed=seed,initial_prefix_steps=count,readiness=readiness,second_search_steps=0,second_paid_forks_verified=[],treatment_continuations=0)
  second=directory/'second_prefix'
  if second.exists():
   initial=load(second/'initial.json');require(initial==load(est/'final.json'),'ready search input');state=SecondState(initial,'replace8');validate_allocator(initial,load(second/'initial_allocator.json'));count=0
   for row in rows(second/'rows.jsonl'):
    state.object(row);require(not any(state.genes[id_]['lineage'] and b['capital']>=100 for id_,b in state.balances.items()),'first second target not skipped');state.finish(row);count+=1
   fork=load(second/'fork.json');state.object(fork['pending']);count+=1;search_steps+=count
   require(count==fork['search_steps'] and count<=256,'second prefix deadline');state.match_snapshot(fork['checkpoint']);validate_allocator(fork['checkpoint'],fork['allocator'])
   for arm in ['faithful','edited']:
    if not (second/(arm+'_birth.json')).exists():continue
    b=load(second/(arm+'_birth.json'));pairing(b['row'],fork['faithful_birth']);branch=copy.deepcopy(state);branch.finish_selected(b['row'],b['selection'],arm,contract,description);branch.match_snapshot(b['initial']);validate_allocator(b['initial'],load(second/(arm+'_initial_allocator.json')));record['second_paid_forks_verified'].append(arm)
   record['second_search_steps']=count;child=fork['faithful_selection']['child'];order=native_order(load(second/'faithful_initial.json'),child);index=contract['second_edit']['parameters'][0]%len(order)
   record['fixed_parameter_target']=dict(child=child,native_pnode_order=order,index=index,variable=order[index]['variable'])
  require(all(not (directory/arm).exists() for arm in ['faithful','edited']),'no treatment continuation')
  records.append(record);print(f'verified interrupted stages {seed}',flush=True)
 require(next(r for r in records if r['seed']==311)['fixed_parameter_target']['variable']==0 and next(r for r in records if r['seed']==313)['fixed_parameter_target']['variable']==1,'allocation order explains mismatch')
 require(not (out/'seed_313/second_prefix/edited_birth.json').exists(),'wrong genotype never admitted to branch')
 return dict(completed_stages_verified=True,full_assay_complete=False,assay_verdict=None,scientific_status='failed',fixed_seed_denominator=8,readiness_passes=sum(r['readiness']['durability']['durable_success'] for r in records),counts=dict(initial_prefix_steps=first_steps,establishment_steps=80000,second_prefix_steps=search_steps,continuation_steps=0,scientific_advancing_steps=first_steps+80000+search_steps),records=records,failure='fixed parameter selects variable 1 at seed313 because native pnode order differs',historical_verdicts_changed=False,frozen_artifacts_preserved=True,auditor_sha256=sha(pathlib.Path(__file__)))

if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('input',type=pathlib.Path);p.add_argument('output',type=pathlib.Path);a=p.parse_args();report=verify(a.input.resolve());a.output.mkdir(exist_ok=False);(a.output/'interrupted_run_verification.json').write_text(json.dumps(report,indent=2)+'\n')
