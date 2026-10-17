#!/usr/bin/env python3
"""Reconstruct continuous v16 policy receipts, financing windows and v15 segment verdicts."""
import argparse,collections,copy,hashlib,json,pathlib,statistics,tarfile
from audit_selfmod_birth import load,rows,sha,require,new_totals,add_totals,score
from audit_selfmod_turnover import SwitchedState
from audit_selfmod_bounded import BoundedState,capital,extra_new,extra_add,reproduction
from run_selfmod_financing import SEEDS,ARMS

def flow(w):
 net=w['income']-w['rent']-w['retired']-w['starved']
 require(w['end_capital']-w['start_capital']==net,'window stock-flow')
 return net

def slope(values):
 if not values or any(v is None for v in values):return None
 middle=(len(values)-1)/2;den=sum((i-middle)**2 for i in range(len(values)))
 return sum((i-middle)*v for i,v in enumerate(values))/den if den else None

def validate_allocator(snapshot,sidecar,legacy_initial=False):
 require(set(sidecar)=={'format','world','rules'} and sidecar['format']==1,'allocator format')
 for store in ['world','rules']:
  require(len(sidecar[store])==2,'allocator stores')
  for name,order in zip(['nodes','edges'],sidecar[store]):
   vacant=[i for i,s in enumerate(snapshot[store][name]) if i>0 and s['value'] is None]
   require(len(order)==len(vacant) and len(set(order))==len(order) and set(order)==set(vacant),'allocator vacant coverage')
   if legacy_initial:require(order==sorted(vacant,reverse=True),'explicit canonical legacy restore order')

def window(start,state):
 c=capital(state)
 return dict(first_step=start,last_step=start+999,start_capital=c,end_capital=c,min_capital=c,max_capital=c,
  income=0,rent=0,retired=0,starved=0,capital_sum=0,copies_sum=0,birth_windows=[0]*4,lineage_birth_windows=[0]*4,
  lineage_productivity_windows=[0]*4,member_productivity_windows=[[0]*3 for _ in range(4)],
  retirement_phase_stocks=[],gates={k:0 for k in ['creation_blocked','node_ceiling_blocked','edge_ceiling_blocked','enumeration_skipped']})

def add_window(w,row,state):
 quarter=(row['continuation_step']-w['first_step'])//250
 income=sum(r['income'] for r in row['roots'] if r['lineage']);rent=sum(r['cost'] for r in row['roots'] if r['lineage']);starved=sum(r['capital_before'] for r in row['roots'] if r['lineage'] and not r['active'])
 phase_capital=w['end_capital']+income-rent-starved
 phase_copies=sum(r['lineage'] and r['active'] for r in row['roots'])
 w['income']+=income;w['rent']+=rent;w['starved']+=starved
 for birth,admission in zip(row['births'],row.get('bounded',{}).get('admissions',[None]*len(row['births']))):
  if birth['gene']['lineage']:
   phase_copies+=1
   if admission and admission['retired_tokens'] is not None:
    tokens=admission['retired_tokens'];w['retired']+=tokens;phase_capital-=tokens;phase_copies-=1
    w['retirement_phase_stocks'].append(dict(step=row['continuation_step'],parent=birth['parent'],child=birth['child'],capital=phase_capital,copies=phase_copies))
 require(phase_capital==capital(state),'birth/retirement phase capital')
 actual_copies=sum(state.genes[id_]['lineage'] for id_ in state.balances)
 require(phase_copies==actual_copies,'birth/retirement phase family copies')
 w['end_capital']=capital(state);w['min_capital']=min(w['min_capital'],w['end_capital']);w['max_capital']=max(w['max_capital'],w['end_capital']);w['capital_sum']+=w['end_capital'];w['copies_sum']+=actual_copies
 w['birth_windows'][quarter]+=len(row['births']);w['lineage_birth_windows'][quarter]+=sum(b['gene']['lineage'] for b in row['births'])
 for r in row['roots']:
  w['member_productivity_windows'][quarter][r['member']]+=r['fate']['productive']
  if r['lineage']:w['lineage_productivity_windows'][quarter]+=r['fate']['productive']
  if r['gate']:
   for key in w['gates']:w['gates'][key]+=bool(r['gate'][key])
 flow(w)

def finish_window(w,state):
 phase=[r['capital'] for r in w['retirement_phase_stocks']]
 w.update(adjusted_surplus=flow(w),mean_capital=w.pop('capital_sum')/1000,mean_copies=w.pop('copies_sum')/1000,
  final_lineage_copies=sum(state.genes[id_]['lineage'] for id_ in state.balances),final_member_copies=[sum(state.genes[id_]['member']==m for id_ in state.balances) for m in range(3)],
  retirement_phase_summary=dict(count=len(phase),mean=statistics.mean(phase) if phase else None,min=min(phase,default=None),max=max(phase,default=None)))
 w['enumeration_censored']=w['gates']['enumeration_skipped']>0
 return w

def audit(out):
 m=load(out/'manifest.json');pre=m['engineering_preflight'];seeds=[7,42] if pre else SEEDS;offset=64 if pre else 2000;steps=128 if pre else 8000
 require(m==dict(protocol='selfmod-v16-financing-v1',engineering_preflight=pre,seeds=seeds,arms=ARMS,offset=offset,extension_steps=steps,requested_trajectories=len(seeds)*8,max_parallel_processes=4,window_steps=1000,segment_steps=2000,prior_primary_changed=False,independent_replication=False),'frozen design')
 prov=load(out/'instrument_provenance.json');ip=load(out/'input_provenance.json')
 require(sha(out/'selfmod-finance')==prov['binary_sha256'] and sha(out/'source.tar.gz')==prov['source_archive_sha256'] and sha(out/'input_provenance.json')==prov['input_provenance_sha256'],'frozen instrument')
 with tarfile.open(out/'source.tar.gz') as tar:
  require(set(tar.getnames())==set(prov['sources_sha256']),'source inventory')
  for file,digest in prov['sources_sha256'].items():require(hashlib.sha256(tar.extractfile(file).read()).hexdigest()==digest,'source '+file)
  require(tar.extractfile('docs/selfmod_v16_financing_preregistration.md').read()==(out/'preregistration.md').read_bytes(),'frozen contract')
 require(set(ip['inputs_sha256'])=={str(p.relative_to(out)) for p in (out/'inputs').rglob('*') if p.is_file()},'input inventory')
 for file,digest in ip['inputs_sha256'].items():require(sha(out/file)==digest,'input '+file)
 prior=load(out/'inputs/audit.json');require(prior['audit_passed'] and load(out/'inputs/status.json')['state']=='complete','validated prior')
 if not pre:
  require(load(out/'inputs/replay_verification.json')['passed'] and load(out/'inputs/engineering_verification.json')['passed'],'prior replay/engineering checks')
  require(prior['aggregates']['prune_replace8']['joint_success']==5 and not prior['aggregates']['prune_replace8']['replicated_joint_success'],'prior primary remains 5/8 below threshold')
 records=[]
 for seed in seeds:
  for arm in ARMS:
   src=out/f'inputs/seed_{seed}/{arm}';dest=out/f'seed_{seed}/{arm}';policy=arm.split('_')[1]
   for f in ['birth.json','final.json','score.json','reproduction.json']:require(sha(src/f)==prior['evidence_sha256'][f'seed_{seed}/{arm}/{f}'],'validated prior '+f)
   initial=load(dest/'initial.json');require(sha(dest/'initial.json')==sha(src/'final.json'),'exact continuation initial state')
   validate_allocator(initial,load(dest/'initial_allocator.json'),legacy_initial=True)
   state=BoundedState(initial,policy) if policy in ['vacancy8','replace8'] else SwitchedState(initial,policy=='on');selected=load(src/'birth.json')['selection'];start=capital(state)
   require(initial['cfg']['steps']==offset and initial['world']['step']==load(src/'birth.json')['row']['absolute_step']+offset,'continuation clock/config')
   windows=[];segments=[];completed=0;w=None;totals=None;extra=None
   for row in rows(dest/'rows.jsonl'):
    local=completed%2000+1;global_step=offset+completed+1
    require(row['continuation_step']==global_step,'continuous row offset')
    if not pre:
     if completed%1000==0:w=window(global_step,state)
     if local==1:totals=new_totals();extra=extra_new()
     if local==1001:extra.update(late_start_capital=capital(state),late_min_capital=capital(state),late_max_capital=capital(state),late_start_recorded=True)
    state.step_row(row);completed+=1
    if not pre:
     add_window(w,row,state);local_row=dict(row,continuation_step=local);add_totals(totals,local_row,selected,2000);extra_add(extra,state,local_row,2000)
     if completed%1000==0:
      snap=load(dest/f'checkpoint_{global_step}.json');require(snap['cfg']==initial['cfg'],'checkpoint config unchanged');state.match_snapshot(snap);validate_allocator(snap,load(dest/f'checkpoint_{global_step}_allocator.json'));windows.append(finish_window(w,state))
     if local==2000:
      legacy=score(state,selected,totals,2000);joint=reproduction(legacy,extra);segments.append(dict(first_step=global_step-1999,last_step=global_step,score=legacy,reproduction=joint))
   require(completed==steps,'complete fixed horizon');final=load(dest/'final.json');require(final['cfg']==initial['cfg'],'full cfg preserved');state.match_snapshot(final)
   validate_allocator(final,load(dest/'final_allocator.json'))
   require(load(dest/'status.json')==dict(state='complete',offset=offset,steps_completed=steps),'branch completion')
   if pre:
    for part in ['a','b']:
     require(load(out/f'seed_{seed}/{arm}_split_{part}/status.json')['state']=='complete','split complete')
    require((dest/'rows.jsonl').read_bytes()==(out/f'seed_{seed}/{arm}_split_a/rows.jsonl').read_bytes()+(out/f'seed_{seed}/{arm}_split_b/rows.jsonl').read_bytes(),'exact split rows')
    require(sha(dest/'final.json')==sha(out/f'seed_{seed}/{arm}_split_b/final.json'),'exact split final')
    require(sha(dest/'final_allocator.json')==sha(out/f'seed_{seed}/{arm}_split_b/final_allocator.json'),'exact split allocator')
   report=dict(seed=seed,arm=arm,start_capital=start,final_capital=capital(state),net_capital_change=capital(state)-start,windows=windows,segments=segments)
   if not pre:
    report.update(capital_mean_slope_per_1000=slope([w['mean_capital'] for w in windows]),phase_capital_mean_slope_per_1000=slope([w['retirement_phase_summary']['mean'] for w in windows]),minimum_capital=min(w['min_capital'] for w in windows),
     positive_windows=sum(w['adjusted_surplus']>0 for w in windows),zero_windows=sum(w['adjusted_surplus']==0 for w in windows),negative_windows=sum(w['adjusted_surplus']<0 for w in windows),
     final_lineage_alive=windows[-1]['final_lineage_copies']>0,final_cycle_alive=all(windows[-1]['final_member_copies']),all_windows_culture_births=all(all(w['birth_windows']) for w in windows),all_windows_family_births=all(sum(w['lineage_birth_windows'])>0 for w in windows),enumeration_censored=any(w['enumeration_censored'] for w in windows))
   records.append(report);print(f'audited {seed} {arm}',flush=True)
 require(load(out/'status.json')==dict(state='complete',trajectories_completed=len(records),new_advancing_steps=len(records)*steps,excluded_split_steps=len(records)*128 if pre else 0),'fixed accounting')
 if pre:
  engineering=load(out/'engineering_verification.json');require(engineering['passed'] and engineering['branches']==16 and engineering['engineering_advancing_steps']==4096,'engineering accounting')
  for file,digest in engineering['evidence_sha256'].items():require(sha(out/file)==digest,'engineering receipt '+file)
 aggregates={}
 if not pre:
  for arm in ARMS:
   group=[r for r in records if r['arm']==arm]
   aggregates[arm]=dict(final_lineages_alive=sum(r['final_lineage_alive'] for r in group),final_cycles_alive=sum(r['final_cycle_alive'] for r in group),
    culture_births_every_quarter=sum(r['all_windows_culture_births'] for r in group),family_births_every_1000=sum(r['all_windows_family_births'] for r in group),
    segment_joint_counts=[sum(r['segments'][i]['reproduction']['joint_success'] for r in group) for i in range(4)],
    segment_establishment_counts=[sum(r['segments'][i]['score']['bounded_establishment'] for r in group) for i in range(4)],
    births=sum(sum(sum(w['birth_windows']) for w in r['windows']) for r in group),family_births=sum(sum(sum(w['lineage_birth_windows']) for w in r['windows']) for r in group),
    net_change_range=[min(r['net_capital_change'] for r in group),max(r['net_capital_change'] for r in group)],capital_mean_slopes=[r['capital_mean_slope_per_1000'] for r in group],enumeration_censored=sum(r['enumeration_censored'] for r in group))
 return dict(audit_passed=True,engineering_preflight=pre,trajectories_completed=len(records),new_diagnostic_advancing_steps=0 if pre else len(records)*steps,engineering_advancing_steps=4096 if pre else 0,
  prior_primary_changed=False,independent_replication=False,prior_prune_replace_joint=None if pre else 5,aggregates=aggregates,records=records,auditor_sha256=sha(pathlib.Path(__file__)),
  evidence_sha256={str(p.relative_to(out)):sha(p) for p in sorted(out.rglob('*')) if p.is_file() and p.name not in ['audit.json','replay_verification.json']})

def adversarial(out,report):
 rejected=[];base=out/'seed_7/faithful_replace8' if report['engineering_preflight'] else out/'seed_131/faithful_replace8';initial=load(base/'initial.json');row=next(rows(base/'rows.jsonl'))
 for name,edit in [('rent',lambda r:r['roots'][0].__setitem__('cost',999)),('rng',lambda r:r.__setitem__('rng_word_pos',r['rng_word_pos']+2)),('cap',lambda r:r['bounded'].__setitem__('limit',9))]:
  damaged=copy.deepcopy(row);edit(damaged)
  try:BoundedState(initial,'replace8').step_row(damaged)
  except (ValueError,KeyError):rejected.append(name)
  else:raise ValueError('accepted corrupted '+name)
 for name,edit in [('capital',lambda w:w.__setitem__('end_capital',w['end_capital']+1)),('retirement',lambda w:w.__setitem__('retired',w['retired']+1)),('income',lambda w:w.__setitem__('income',w['income']+1))]:
  original=dict(start_capital=100,end_capital=81,income=995,rent=0,retired=1014,starved=0) if report['engineering_preflight'] else copy.deepcopy(next(r for r in report['records'] if r['arm']=='prune_replace8')['windows'][0])
  flow(original);damaged=copy.deepcopy(original);edit(damaged)
  try:flow(damaged)
  except ValueError:rejected.append('window_'+name)
  else:raise ValueError('accepted corrupted window '+name)
 try:require(row['continuation_step']+1==(64 if report['engineering_preflight'] else 2000)+1,'offset')
 except ValueError:rejected.append('offset')
 else:raise ValueError('accepted offset corruption')
 damaged=copy.deepcopy(initial);damaged['ledger']['free']+=1
 try:BoundedState(damaged,'replace8').step_row(row)
 except ValueError:rejected.append('start_ledger')
 else:raise ValueError('accepted start ledger corruption')
 return rejected

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('output',type=pathlib.Path);args=p.parse_args();report=audit(args.output);report['adversarial_corruptions_rejected']=adversarial(args.output,report)
 with (args.output/'audit.json').open('x') as f:json.dump(report,f,indent=2);f.write('\n')
 print(json.dumps(dict(audit_passed=True,steps=report['new_diagnostic_advancing_steps'],aggregates=report['aggregates'],corruptions=report['adversarial_corruptions_rejected']),indent=2))
if __name__=='__main__':main()
