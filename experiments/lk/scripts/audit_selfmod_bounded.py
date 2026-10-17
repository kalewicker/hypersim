#!/usr/bin/env python3
"""Reconstruct v14 bounded births, retirement capital and joint success."""
import argparse,copy,hashlib,json,pathlib,statistics,tarfile
from audit_selfmod_birth import State,canonical_spec,load,rows,sha,require,new_totals,add_totals,score,SEEDS
from audit_selfmod_turnover import SwitchedState
POLICIES=['on','off','vacancy8','replace8'];ARMS=[g+'_'+p for g in ['faithful','prune'] for p in POLICIES]
class BoundedState(State):
 def __init__(self,snapshot,policy):super().__init__(snapshot);self.policy=policy
 def finish(self,row):
  b=row['bounded'];eligible=[id_ for id_ in sorted(self.balances) if self.balances[id_]['capital']>=100]
  require(b['policy']==self.policy and b['limit']==8 and b['population_before']==len(self.balances) and len(self.balances)<=8 and b['eligible']==eligible,'bounded eligibility/cap')
  admitted=eligible if self.policy=='replace8' else eligible[:max(8-len(self.balances),0)]
  require([x['parent'] for x in row['births']]==admitted and b['blocked']==[x for x in eligible if x not in admitted],'admission order/vacancies')
  require(len(b['admissions'])==len(row['births']),'admission/birth pairing')
  for admission,birth in zip(b['admissions'],row['births']):
   parent,child=birth['parent'],birth['child'];stock=self.balances[parent];pg=self.genes[parent];gene=birth['gene'];n=len(self.balances)
   require(child not in self.genes and child not in self.balances,'root identity reused')
   require(birth['parent_before']==stock['capital'] and birth['parent_after_native']==stock['capital']-50 and birth['child_tokens']==50 and birth['inherited_budget']==stock['budget'],'normal funded payment/inheritance')
   require(not birth['cancelled'] and 0<=birth['coin']<1 and 0<=birth['operator']<15 and len(birth['parameters'])==5,'ordinary native draw block')
   require(gene['id']==child and gene['parent']==parent and gene['member']==pg['member'] and gene['lineage']==pg['lineage'] and gene['born']==self.step,'immutable birth genealogy')
   require(canonical_spec(gene['spec'])==canonical_spec(pg['spec']) and gene['class_hash']==pg['class_hash'] and gene['executable']==pg['executable'],'faithful subsequent class')
   stock['capital']-=50;self.balances[child]=dict(capital=50,budget=stock['budget']);self.genes[child]=gene;self.seq+=1
   retire=n==8 and self.policy=='replace8';returned=stock['capital'] if retire else None
   require(admission==dict(parent=parent,child=child,population_before=n,retired_tokens=returned),'retirement identity/capital')
   if retire:self.free+=stock['capital'];del self.balances[parent]
   require(len(self.balances)<=8,'post-admission cap')
  for stock in self.balances.values():stock['budget']=min(stock['budget']+self.cfg['creation_refill'],self.cfg['creation_budget_max'])
  require(row['stocks']==[dict(id=id_,**s) for id_,s in sorted(self.balances.items())],'native refill/boundary stocks')
  require(row['ledger']==self.cash(),'retirement ledger');require(row['rule_cap_binding']==(len(self.balances)>=self.cfg['rule_cap']),'original cap flag')
  require(row['rng_word_pos']==self.pos+14*len(row['births']),'birth-only RNG draws')
  if not row['births']:require(row['rng_probe']==self.probe,'RNG drift without birth')
  self.pos,self.probe=row['rng_word_pos'],row['rng_probe']

def extra_new():return dict(births=0,lineage_births=0,retirements=0,retired_tokens=0,late_lineage_retired=0,late_lineage_starved=0,
 late_birth_windows=[0]*4,late_lineage_births=0,late_start_capital=0,late_min_capital=0,late_max_capital=0,final_capital=0,peak_boundary_population=0,late_start_recorded=False)
def capital(s):return sum(v['capital'] for k,v in s.balances.items() if s.genes[k]['lineage'])
def extra_add(extra,state,row,horizon):
 step=row['continuation_step'];late=step>horizon//2;extra['births']+=len(row['births'])
 for b in row['births']:
  if b['gene']['lineage']:
   extra['lineage_births']+=1
   if late:extra['late_lineage_births']+=1
 if late:
  extra['late_birth_windows'][(step-horizon//2-1)//(horizon//8)]+=len(row['births'])
  extra['late_lineage_starved']+=sum(r['capital_before'] for r in row['roots'] if r['lineage'] and not r['active'])
 for admission in row.get('bounded',{}).get('admissions',[]):
  if admission['retired_tokens'] is not None:
   tokens=admission['retired_tokens'];extra['retirements']+=1;extra['retired_tokens']+=tokens
   if late and state.genes[admission['parent']]['lineage']:extra['late_lineage_retired']+=tokens
 extra['final_capital']=capital(state);extra['peak_boundary_population']=max(extra['peak_boundary_population'],len(state.balances))
 if late:
  extra['late_min_capital']=min(extra['late_min_capital'],extra['final_capital']);extra['late_max_capital']=max(extra['late_max_capital'],extra['final_capital'])
def reproduction(legacy,extra):
 t=legacy['totals'];surplus=t['late_lineage_income']-t['late_lineage_rent']-extra['late_lineage_retired']-extra['late_lineage_starved']
 require(extra['final_capital']-extra['late_start_capital']==surplus,'late lineage stock-flow including retirement/starvation')
 return dict(joint_success=legacy['bounded_establishment'] and surplus>=0 and all(extra['late_birth_windows']) and extra['late_lineage_births']>0,
  retirement_adjusted_late_surplus=surplus,metrics=extra)

def audit(out):
 m,summary,status,prov=(load(out/f) for f in ['manifest.json','summary.json','status.json','instrument_provenance.json']);pre=m['engineering_preflight'];seeds=[7,42] if pre else SEEDS;horizon=64 if pre else 2000
 require(m==dict(protocol='selfmod-v14-bounded-v1',engineering_preflight=pre,seeds=seeds,horizon=horizon,arms=ARMS,limit=8,continuations_requested=len(seeds)*8,prefix_steps=0,replication_threshold=6,future_mutation=False),'frozen design')
 require(sha(out/'selfmod-bounded')==prov['binary_sha256'] and sha(out/'source.tar.gz')==prov['source_archive_sha256'] and sha(out/'input_provenance.json')==prov['input_provenance_sha256'],'instrument/input hashes')
 with tarfile.open(out/'source.tar.gz') as tar:
  require(set(tar.getnames())==set(prov['sources_sha256']),'source inventory')
  for file,digest in prov['sources_sha256'].items():require(hashlib.sha256(tar.extractfile(file).read()).hexdigest()==digest,'source '+file)
  require(tar.extractfile('docs/selfmod_v14_bounded_preregistration.md').read()==(out/'preregistration.md').read_bytes(),'frozen contract')
 ip=load(out/'input_provenance.json');require(set(ip['inputs_sha256'])=={str(p.relative_to(out)) for p in (out/'inputs').rglob('*') if p.is_file()},'input inventory')
 for file,digest in ip['inputs_sha256'].items():require(sha(out/file)==digest,'input '+file)
 prior=load(out/'inputs/audit.json');require(prior['audit_passed'] and load(out/'inputs/status.json')['state']=='complete','validated v13')
 if not pre:require(load(out/'inputs/replay_verification.json')['passed'],'validated v13 replay')
 evidence=load(out/'inputs/v11_selected_mutation.json');records=[]
 for seed in seeds:
  input_dir=out/f'inputs/seed_{seed}';require(sha(input_dir/'fork.json')==prior['evidence_sha256'][f'inputs/seed_{seed}/fork.json'],'prior fork')
  fork=load(input_dir/'fork.json')
  for arm in ARMS:
   genotype,policy=arm.split('_');control=genotype+('_off' if policy=='off' else '_on');expected=input_dir/control;path=out/f'seed_{seed}/{arm}'
   for file in ['birth.json','rows.jsonl','score.json','final.json']:require(sha(expected/file)==prior['evidence_sha256'][f'seed_{seed}/{control}/{file}'],'validated input control '+file)
   birth=load(path/'birth.json');selected=birth['selection'];require(sha(path/'birth.json')==sha(expected/'birth.json'),'paid birth parity')
   initial=State(fork['checkpoint']);initial.finish(birth['row'],selected,evidence);initial.match_snapshot(birth['initial'])
   require(selected['arm']==genotype and selected['executable'] and not selected['cancelled'] and birth['initial']['cfg']==fork['checkpoint']['cfg'] and initial.cfg['steps']==horizon,'unaltered initial context')
   require(len(initial.balances)<=8,'cap does not delete initial residents')
   branch=BoundedState(birth['initial'],policy) if policy in ['vacancy8','replace8'] else SwitchedState(birth['initial'],policy=='on')
   totals=new_totals();extra=extra_new();completed=0
   for row in rows(path/'rows.jsonl'):
    completed+=1;require(row['continuation_step']==completed,'step order/denominator')
    if completed==horizon//2+1:
     extra['late_start_capital']=capital(branch);extra['late_min_capital']=extra['late_start_capital'];extra['late_max_capital']=extra['late_start_capital'];extra['late_start_recorded']=True
    if policy in ['on','off']:require('bounded' not in row,'control schema parity')
    branch.step_row(row);add_totals(totals,row,selected,horizon);extra_add(extra,branch,row,horizon)
   require(completed==horizon,'complete horizon');legacy=score(branch,selected,totals,completed);require(legacy==load(path/'score.json'),'legacy verdict')
   joint=reproduction(legacy,extra);require(joint==load(path/'reproduction.json'),'joint verdict/reproduction metrics');branch.match_snapshot(load(path/'final.json'))
   if policy in ['on','off']:
    for file in ['rows.jsonl','score.json','final.json']:require(sha(path/file)==sha(expected/file),'v13 byte-exact control '+file)
   records.append(dict(seed=seed,arm=arm,score=legacy,reproduction=joint))
 steps=len(records)*horizon
 require(summary==dict(protocol=m['protocol'],engineering_preflight=pre,continuations_completed=len(records),scientific_advancing_steps=steps,new_bounded_steps=steps//2,repeated_control_steps=steps//2,shared_prefix_steps=0,controls_byte_identical=True,duplicates_pooled=False,results=records),'summary accounting')
 require(status==dict(state='complete',continuations_completed=len(records),scientific_advancing_steps=steps),'complete status')
 aggregates={}
 for arm in ARMS:
  group=[r for r in records if r['arm']==arm];j=[r['reproduction'] for r in group];s=[r['score'] for r in group];rates=[v['lineage_per_copy_rate'] for v in s if v['lineage_per_copy_rate'] is not None]
  aggregates[arm]=dict(continuations=len(s),legacy_established=sum(v['bounded_establishment'] for v in s),joint_success=sum(v['joint_success'] for v in j),
   replicated_joint_success=None if pre else sum(v['joint_success'] for v in j)>=6,lineage_alive=sum(v['final_lineage_copies']>0 for v in s),cycle_persistent=sum(v['whole_cycle_persistent'] for v in s),
   enumeration_censored=sum(v['totals']['enumeration_skips']>0 for v in s),resource_gated=sum(v['totals']['resource_gates']>0 for v in s),
   reproduces_every_late_window=sum(all(v['metrics']['late_birth_windows']) for v in j),lineage_reproduces_late=sum(v['metrics']['late_lineage_births']>0 for v in j),
   retirement_financing_covered=sum(v['retirement_adjusted_late_surplus']>=0 for v in j),births=sum(v['metrics']['births'] for v in j),retirements=sum(v['metrics']['retirements'] for v in j),
   mean_per_copy_rate=statistics.mean(rates) if rates else None,max_boundary_population=max(v['metrics']['peak_boundary_population'] for v in j))
 return dict(audit_passed=True,engineering_preflight=pre,continuations_completed=len(records),scientific_advancing_steps=steps,new_bounded_steps=steps//2,repeated_control_steps=steps//2,
  controls_byte_identical=True,aggregates=aggregates,records=records,auditor_sha256=sha(pathlib.Path(__file__)),
  scope='Independent money, eligibility/cap/parent retirement, allowance, genealogy, RNG counts, legacy/joint verdict and late stock-flow reconstruction. Native matcher, mutator, effects, hash and ChaCha values tested/replayed.',
  evidence_sha256={str(p.relative_to(out)):sha(p) for p in sorted(out.rglob('*')) if p.is_file() and p.name not in ['audit.json','replay_verification.json']})

def adversarial(out,seed=None):
 seed=load(out/'manifest.json')['seeds'][0] if seed is None else seed;rejected=[]
 for policy in ['vacancy8','replace8']:
  path=out/f'seed_{seed}/faithful_{policy}';birth=load(path/'birth.json');row=next(rows(path/'rows.jsonl'))
  for label,edit in [
   ('rent',lambda r:r['roots'][0].__setitem__('cost',999)),('fate',lambda r:r['roots'][0]['fate'].__setitem__('productive',999)),
   ('cash',lambda r:r['ledger'].__setitem__('free',999)),('budget',lambda r:r['stocks'][0].__setitem__('budget',999)),
   ('rng',lambda r:r.__setitem__('rng_word_pos',r['rng_word_pos']+2)),('cap',lambda r:r['bounded'].__setitem__('limit',999)),
   ('eligibility',lambda r:r['bounded']['eligible'].append(-1)),('population',lambda r:r['bounded'].__setitem__('population_before',999))]:
   damaged=copy.deepcopy(row);edit(damaged)
   try:BoundedState(birth['initial'],policy).step_row(damaged)
   except (ValueError,KeyError):rejected.append(policy+'/'+label)
   else:raise ValueError('accepted corrupted '+policy+'/'+label)
 # Exercise retirement checks on an actual replacement phase, including its donor.
 path=out/f'seed_{seed}/faithful_replace8';state=BoundedState(load(path/'birth.json')['initial'],'replace8')
 for row in rows(path/'rows.jsonl'):
  before=copy.deepcopy(state)
  if any(a['retired_tokens'] is not None for a in row['bounded']['admissions']):
   idx=next(i for i,a in enumerate(row['bounded']['admissions']) if a['retired_tokens'] is not None)
   for label,edit in [
    ('retired_capital',lambda r:r['bounded']['admissions'][idx].__setitem__('retired_tokens',999)),
    ('retired_parent',lambda r:r['bounded']['admissions'][idx].__setitem__('parent',-1)),
    ('birth_payment',lambda r:r['births'][idx].__setitem__('child_tokens',999)),
    ('missing_birth',lambda r:r['births'].pop(idx))]:
    damaged=copy.deepcopy(row);edit(damaged)
    try:copy.deepcopy(before).step_row(damaged)
    except (ValueError,KeyError):rejected.append(label)
    else:raise ValueError('accepted corrupted '+label)
   break
  state.step_row(row)
 else:raise ValueError('preflight must exercise funded retirement')
 # Independently reconcile a deliberately falsified retirement withdrawal.
 legacy=load(path/'score.json');original=load(path/'reproduction.json');extra=copy.deepcopy(original['metrics']);extra['late_lineage_retired']+=50
 try:reproduction(legacy,extra)
 except ValueError:rejected.append('late_retirement_stock_flow')
 else:raise ValueError('accepted false late retirement withdrawal')
 # OFF cannot satisfy the ongoing-reproduction condition even if legacy establishment passes.
 offpath=out/f'seed_{seed}/faithful_off';offscore=load(offpath/'score.json');offextra=load(offpath/'reproduction.json')['metrics']
 damaged=copy.deepcopy(load(offpath/'reproduction.json'));damaged['joint_success']=True
 try:require(damaged==reproduction(offscore,offextra),'false OFF ongoing-reproduction verdict')
 except ValueError:rejected.append('off_false_ongoing_reproduction_verdict')
 else:raise ValueError('accepted false OFF ongoing reproduction')
 return rejected

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('output',type=pathlib.Path);args=p.parse_args();report=audit(args.output);report['adversarial_corruptions_rejected']=adversarial(args.output)
 with (args.output/'audit.json').open('x') as f:json.dump(report,f,indent=2);f.write('\n')
 print(json.dumps({k:report[k] for k in ['audit_passed','continuations_completed','scientific_advancing_steps','aggregates','adversarial_corruptions_rejected']},indent=2))
if __name__=='__main__':main()
