#!/usr/bin/env python3
"""Independent configuration-bound native funded admission and recovery audit."""
import argparse,copy,hashlib,json,time,tomllib
from pathlib import Path
import importlib.util,sys
PROFILE='evolving-minds-d0-funded-streaming-v1'
FAULTS=['CleanStop','TornPayload','TornCheckpoint','BeforeCommit','AfterCommit','Quota']
FAULT_NAMES=['clean_stop','torn_payload','torn_checkpoint','before_commit','after_commit','quota']
def req(ok,why):
 if not ok:raise ValueError(why)
def data(p):return json.loads(Path(p).read_text())
def load(p,name):
 spec=importlib.util.spec_from_file_location(name,p);m=importlib.util.module_from_spec(spec);old=sys.dont_write_bytecode
 try:sys.dont_write_bytecode=True;spec.loader.exec_module(m)
 finally:sys.dont_write_bytecode=old
 return m
def cases():
 rows=[]
 for name,p in [('delayed_b24','Delayed'),('immediate_b24','Immediate'),('coin_b24','Coin')]:rows.append(dict(name=name,config=dict(seed=17,endowment=24,founders=[p,p],kernels=['Faithful','Faithful'],home_offsets=[0,4],rotation=0,reversed=False,horizon=64,scheduler='semantic-lottery-v1')))
 def add(name,i,**changes):
  c=copy.deepcopy(rows[i]['config']);c.update(changes);rows.append(dict(name=name,config=c))
 add('source_b24',0,kernels=['SourceKernel','SourceKernel']);add('mixed_reject_b24',1,kernels=['Invalid','Faithful']);add('mixed_unreachable_b24',1,kernels=['MissingPort','Faithful']);add('immediate_b12',1,endowment=12);add('source_s23_b24',3,seed=23);add('immediate_r3_forward',1,rotation=3);add('immediate_r3_reverse',8,reversed=True)
 return rows
def denominator(root,registered):
 expected={f"baseline_{r['name']}" for r in registered}|{f'{prefix}_{n}' for n in FAULT_NAMES for prefix in ['fault','recovered']}
 req(len(registered)==10 and data(root/'registered_cases.json')==registered and data(root/'registered_faults.json')==FAULTS and {p.name for p in root.iterdir() if p.is_dir()}==expected,'complete fixed funded panel denominator')
def summary(evidence):
 ancestry=evidence['lineage'];records=evidence['admission']['records'];rounds=evidence['admission']['rounds']
 origins={}
 for k,b in sorted(ancestry.items(),key=lambda kv:int(kv[0])):origins[int(k)]=int(k) if b['parent'] is None else origins[b['parent']]
 per={}
 for founder in [1,2]:
  rs=[r for r in records if origins[r['parent']]==founder];bs=[b for k,b in ancestry.items() if int(k)>2 and origins[int(k)]==founder]
  per[str(founder)]=dict(requests=len(rs),founder_requests=sum(r['parent']==founder for r in rs),births=sum(r['child_id'] is not None for r in rs),invalid_requests=sum(not r['syntax_valid'] for r in rs),unreachable_births=sum(r['syntax_valid'] and not r['material_reachable'] for r in rs),live_descendants_at_64=sum(b['death_tick'] is None for b in bs),request_charge=sum(r['request_charge'] for r in rs),copy_charge=sum(r['copying_charge'] for r in rs),refunds=sum(r['refunded'] for r in rs))
 return dict(final_row=evidence['rows'][-1],eligible_parent_ticks=sum(len({o['parent'] for o in rnd['offers']}) for rnd in rounds),offers=sum(len(rnd['offers']) for rnd in rounds),admitted_requests=len(records),parallel_admission_ticks=[r['tick'] for r in rounds if len(r['selected'])>1],per_founder=per)
def exact(a,b):
 req(a==b,'recovery chain exact native events/graphs/rows/lineage/proposals')
def verify(root,extra_segment=None):
 start=time.monotonic();root=Path(root);req(root.is_dir() and not root.is_symlink() and not any(p.is_symlink() for p in root.rglob('*')),'funded panel path');req(sum(p.stat().st_size for p in root.rglob('*') if p.is_file())<=512*1024*1024,'funded panel ceiling')
 sources=data(root/'sources.json');bindings=[('checker.py','scripts/verify_evolving_minds_funded_admission.py'),('stream_reference.py','scripts/verify_evolving_minds_streaming.py'),('admission_reference.py','scripts/verify_evolving_minds_admission.py'),('funding_reference.py','scripts/verify_evolving_minds_funding.py'),('replay.py','scripts/verify_evolving_minds.py'),('d0_reference.py','scripts/verify_evolving_minds_d0.py'),('schedule_reference.py','scripts/verify_evolving_minds_choice.py'),('base_rules.toml','rules/evolving_minds/admission_v1.toml'),('contract.md','experiments/evolving_minds/contracts/d0_funded_admission_v1.md')]
 for f,p in bindings:req((root/f).read_text()==sources[p],'frozen funded source binding')
 v=load(root/'stream_reference.py','funded_storage_ref');a=load(root/'admission_reference.py','funded_adm_ref');f=load(root/'funding_reference.py','funded_law_ref');c=load(root/'replay.py','funded_replay');d=load(root/'d0_reference.py','funded_d0');s=load(root/'schedule_reference.py','funded_schedule');modules=[a,c,d,s]
 base=(root/'base_rules.toml').read_bytes();req(hashlib.sha256(base).hexdigest()==a.LAW_SHA,'fixed original admission law identity');funded=(root/'b24_base_rules.toml').read_bytes();req(tomllib.loads(funded.decode())==f.funded_laws(base,24),'independently derived parent-funded law fields');bases={12:base,24:funded}
 registered=cases();denominator(root,registered)
 configs={f"baseline_{r['name']}":r['config'] for r in registered};fault_cfg=next(r['config'] for r in registered if r['name']=='mixed_reject_b24')
 for n in FAULT_NAMES:
  for prefix in ['fault','recovered']:configs[f'{prefix}_{n}']=fault_cfg
 reports=[];results={};full={};count=0;native_advances=0
 for name,cfg in configs.items():
  a.validate_funded_config(cfg);p=root/name;chain=v.inspect_segment(p,funded_config=cfg);seg=chain[-1];header=seg['header'];report=data(p/'segment_report.json');manifest=data(p/'manifest.json');status=data(p/'run_status.json');v.boundary(status,PROFILE)
  tick=seg['marker']['cursor']['tick'];faulted=name.startswith('fault_');expected_fault=FAULTS[FAULT_NAMES.index(name.removeprefix('fault_'))] if faulted else 'Normal'
  req(report['profile']==PROFILE and report['funded_config']==cfg and report['recipe']==header['recipe'] and header['fault']==report['fault']==expected_fault and report['committed_final_tick']==tick and report['start_tick']==header['start_tick'] and manifest['seed']==cfg['seed'] and manifest['experiment_name']==PROFILE and manifest['max_steps']==2944 and manifest['final_step']==tick*46 and report['state']==manifest['state']==('Interrupted' if faulted else 'Completed') and status['engineering_execution_complete'] is (not faulted),'funded actual manifest/report scope')
  r=report['resources'];req(r['drained_history_empty'] is True and r['retained_entries_peak']==46 and r['retained_rows_peak']==r['retained_rounds_peak']==1 and r['retained_requests_peak']<=8 and r['retained_lineage_peak']<=16 and r['payload_bytes']==(p/'ticks.jsonl').stat().st_size and all(r[k]>=0 for k in ['advance_seconds','checkpoint_seconds','io_seconds','resume_seconds']) and r['advance_seconds']<=120,'bounded funded retained history/resources')
  req(r['checkpoints']==[[x['marker']['cursor']['tick'],x['checkpoint_bytes']] for x in seg['commits']],'funded checkpoint inventory')
  native_tick=report['native_final_tick'];req(header['start_tick']<=tick<=native_tick<=64,'native/committed horizon');native_advances+=(native_tick-header['start_tick'])*46
  if faulted:
   if expected_fault=='Quota':req(native_tick>tick and report['stop_reason']=='declared segment payload quota' and r['payload_bytes']<=262144,'quota censoring')
   else:req(native_tick==dict(CleanStop=8,TornPayload=9,TornCheckpoint=16,BeforeCommit=16,AfterCommit=16)[expected_fault] and tick==(16 if expected_fault=='AfterCommit' else 8),'funded fixed interrupted cut')
   if expected_fault=='BeforeCommit':req((p/'commit_0016.json.pending').exists() and not (p/'commit_0016.json').exists(),'pending commit not published')
   if expected_fault=='TornCheckpoint':req((p/'checkpoint_0016.bin.pending').exists() and not (p/'commit_0016.json').exists(),'torn checkpoint not published')
   if expected_fault=='TornPayload':req((p/'ticks.jsonl').read_bytes()[-1:]!=b'\n','torn record retained')
   if expected_fault=='AfterCommit':req(data(p/'manifest_at_crash.json')['final_step']==8*46,'stale manifest evidence')
  else:req(tick==native_tick==64,'full funded horizon')
  evidence=None
  for commit in seg['commits']:
   native=commit['state']['native'];req(native['funded_config']==cfg and native['schedule']==data(chain[0]['directory']/'schedule.json') and native['program']==a.funded_program(cfg['founders'][0],d),'funded live identity binding')
   law=(p/'rules.toml').read_bytes();h=0xcbf29ce484222325
   for byte in law:h=((h^byte)*0x100000001b3)&((1<<64)-1)
   req(native['law_identity']==h,'funded native law checksum')
   evidence,_=v.physical_audit(chain,commit,modules,bases[cfg['endowment']],funded_config=cfg);count+=1
  reports.append(report)
  if not faulted:
   full[name]=evidence;results[name]=summary(evidence)
 req(data(root/'summary.json')==reports,'complete funded segment summaries');status=data(root/'run_status.json');v.boundary(status,PROFILE);req(status['engineering_execution_complete'] is True,'funded panel engineering completion');m=data(root/'manifest.json');req(m['state']=='Completed' and m['experiment_name']==PROFILE and m['max_steps']==m['final_step']==47104,'funded complete-chain manifest')
 req(data(root/'execution_counts.json')==dict(native_advances_executed=native_advances,complete_committed_chain_advances=47104,complete_chains=16,segments=22,unpublished_or_repeated_tail_advances=native_advances-47104),'actual native execution count')
 baseline=full['baseline_mixed_reject_b24']
 for n in FAULT_NAMES:exact(full[f'recovered_{n}'],baseline)
 # Allocation has different graph handles and matcher work; compare semantic outcomes.
 x,y=copy.deepcopy(full['baseline_immediate_r3_forward']),copy.deepcopy(full['baseline_immediate_r3_reverse'])
 for v_ in [x,y]:
  v_.pop('journal');v_['admission'].pop('recipe')
  for row in v_['rows']:row.pop('match_work')
 req(x==y,'funded allocation semantic identity')
 req(results['baseline_immediate_b24']['parallel_admission_ticks'],'funded concurrent paid admissions');rej=results['baseline_mixed_reject_b24']['per_founder']['1'];req(rej['founder_requests']>1 and rej['invalid_requests']==rej['requests'] and rej['births']==0 and rej['refunds']==29*rej['requests'],'founder repeated paid rejection without exclusion/fallback')
 req(results['baseline_mixed_unreachable_b24']['final_row']['development_failures']>0,'funded paid development failure')
 extra=False
 if extra_segment is not None:
  chain=v.inspect_segment(extra_segment,funded_config=fault_cfg);commit=chain[-1]['commits'][-1];req(commit['marker']['cursor']['tick']==64,'fresh-process continuation horizon');evidence,_=v.physical_audit(chain,commit,modules,bases[24],funded_config=fault_cfg);exact(evidence,baseline);extra=True
 return dict(profile=PROFILE,audit_pass=True,development_only=True,scientific_run_launched=False,gate_pass=False,training_admitted=False,evolution_enabled=False,reproductive_fairness_accepted=False,baseline_cells_verified=10,complete_chains_verified=16,segments_verified=22,interrupted_prefixes_verified=6,published_states_reconstructed=count,unique_committed_chain_advances_verified=47104,native_advances_executed=native_advances,fresh_process_continuation_verified=extra,audit_seconds=time.monotonic()-start,source_sha256={k:hashlib.sha256(x.encode()).hexdigest() for k,x in sources.items()},results=results)
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('directory');p.add_argument('--extra-segment');args=p.parse_args();print(json.dumps(verify(args.directory,args.extra_segment),indent=2))
