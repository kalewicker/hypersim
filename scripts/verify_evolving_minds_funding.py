#!/usr/bin/env python3
"""Independent complete-denominator parent-funded offspring calibration audit."""
import argparse,copy,hashlib,importlib.util,json,sys,time,tomllib
from pathlib import Path
PROFILE='evolving-minds-d0-funding-v1'
def req(ok,why):
 if not ok:raise ValueError(why)
def load(p,n):
 spec=importlib.util.spec_from_file_location(n,p);m=importlib.util.module_from_spec(spec);old=sys.dont_write_bytecode
 try:sys.dont_write_bytecode=True;spec.loader.exec_module(m)
 finally:sys.dont_write_bytecode=old
 return m
def read(p):return json.loads(p.read_text())
def recipes(d):
 rows=[]
 for b in [12,24]:
  for name in ['cue','delayed_cue']:
   rows.append(dict(endowment=b,recipe=dict(program=name,target_id=None,new_source=None)))
   for g in sorted(d.program(name)['instructions'],key=lambda g:g['id']):
    rows.extend(dict(endowment=b,recipe=dict(program=name,target_id=g['id'],new_source=src)) for src in range(8) if src!=g['source'])
 return rows
def name(r):
 q=r['recipe'];suffix='_unchanged' if q['target_id'] is None else f"_g{q['target_id']}_s{q['new_source']}"
 return f"b{r['endowment']}_{q['program']}{suffix}"
def funded_laws(base,budget):
 req(budget in [12,24],'fixed endowment');laws=copy.deepcopy(tomllib.loads(base.decode()))
 for law in laws['rules']:
  n=law['name']
  if n.startswith('reserve_'):
   length=int(n.removeprefix('reserve_'));fee=1+(11+length)/4
   guards=[g for g in law['lhs']['node_property_guards'] if g['var']=='r' and g['property']==1];req(len(guards)==1 and guards[0]==dict(var='r',property=1,op='gte',value=13+fee),'legacy affordability guard');guards[0]['value']=1+budget+fee
   for var,op,old,new in [('r','subtract',13+fee,1+budget+fee),('escrow','set',12+fee,budget+fee)]:
    effects=[e for e in law['rhs']['property_effects'] if e['var']==var and e['key']==1];req(len(effects)==1 and effects[0]==dict(var=var,key=1,op=op,value=old),'legacy request deposit');effects[0]['value']=new
  elif n.startswith('transaction_copy_'):
   length=int(n.removeprefix('transaction_copy_'));fee=1+(11+length)/4
   guards=[g for g in law['lhs']['node_property_guards'] if g['var']=='escrow' and g['property']==1];req(len(guards)==1 and guards[0]==dict(var='escrow',property=1,op='eq',value=12+fee),'legacy copy escrow guard');guards[0]['value']=budget+fee
   effects=[e for e in law['rhs']['property_effects'] if e['var']=='child' and e['key']==1];req(len(effects)==1 and effects[0]==dict(var='child',key=1,op='set',value=12),'legacy newborn reserve');effects[0]['value']=budget
 return laws
def denominator(root,cells):
 req(len(cells)==116 and read(root/'registered_cells.json')==cells and {p.name for p in root.iterdir() if p.is_dir()}=={name(r) for r in cells},'complete fixed 116-cell denominator')
def verify(root):
 start=time.monotonic();root=Path(root);req(root.is_dir() and not root.is_symlink() and not any(p.is_symlink() for p in root.rglob('*')),'funding evidence path');req(sum(p.stat().st_size for p in root.rglob('*') if p.is_file())<=512*1024*1024,'funding panel ceiling')
 sources=read(root/'sources.json')
 for f,p in [('checker.py','scripts/verify_evolving_minds_funding.py'),('mutation_reference.py','scripts/verify_evolving_minds_mutation.py'),('d0_reference.py','scripts/verify_evolving_minds_d0.py'),('replay.py','scripts/verify_evolving_minds.py'),('schedule_reference.py','scripts/verify_evolving_minds_choice.py'),('base_rules.toml','rules/evolving_minds/mutation_v1.toml'),('contract.md','experiments/evolving_minds/contracts/d0_funding_v1.md')]:req((root/f).read_text()==sources[p],'frozen source binding')
 a,c,d,s=[load(root/f,n) for f,n in [('mutation_reference.py','fund_mut'),('replay.py','fund_replay'),('d0_reference.py','fund_d0'),('schedule_reference.py','fund_schedule')]];base=(root/'base_rules.toml').read_bytes();req(hashlib.sha256(base).hexdigest()==a.LAW_SHA,'immutable legacy physiology law identity')
 cells=recipes(d);denominator(root,cells);bases={}
 for budget in [12,24]:
  b=(root/f'b{budget}_base_rules.toml').read_bytes();req(tomllib.loads(b.decode())==funded_laws(base,budget),'independently derived funding physiology');bases[budget]=b
  if budget==12:req(b==base,'unchanged baseline law bytes')
 outcomes=[]
 for registered in cells:
  recipe=registered['recipe'];budget=registered['endowment'];p=root/name(registered)
  req(read(p/'recipe.json')==recipe and read(p/'endowment.json')==budget,'cell recipe and endowment')
  after=copy.deepcopy([r for r in tomllib.loads(bases[budget].decode())['rules'] if r['name'].startswith('reserve_')])
  for law in after:law['lhs']['node_property_guards'].append(dict(var='r',property=c.BID,op='gte',value=2))
  req(tomllib.loads((p/'descendant_reserve.toml').read_text())==dict(rules=after),'funded descendant eligibility law')
  resources=read(p/'resources.json');req(0<=resources['advance_seconds']<=120 and resources['native_advances']==2880,'native resource bounds')
  for f in p.iterdir():
   if f.suffix=='.json':req(f.stat().st_size<=4*1024*1024,'cell JSON ceiling')
  _,outcome=a.verify_cell(p,c,d,s,bases[budget],recipe['program'],'Informative','Faithful',neighborhood=recipe,endowment=budget);outcomes.append(dict(endowment=budget,outcome=outcome))
 req(read(root/'summary.json')==outcomes,'complete outcomes inventory')
 status=read(root/'run_status.json');req(status['profile']==PROFILE and status['engineering_execution_complete'] is True and status['development_only'] is True and status['mutation_mechanism_enabled'] is True and all(status[k] is False for k in ['scientific_run_launched','gate_pass','training_admitted','evolution_enabled','reproductive_fairness_accepted']),'root research boundary');m=read(root/'manifest.json');req(m['state']=='Completed' and m['seed']==17 and m['max_steps']==m['final_step']==116*2880,'complete panel manifest')
 groups={}
 for budget in [12,24]:
  for program,n in [('cue',21),('delayed_cue',35)]:
   group=[x['outcome'] for x in outcomes if x['endowment']==budget and x['outcome']['recipe']['program']==program];control=next(o for o in group if o['recipe']['target_id'] is None);edited=[o for o in group if o['recipe']['target_id'] is not None];req(len(edited)==n,'fixed edit denominator')
   predicates=dict(syntax_valid=lambda o:o['syntax_valid'],grammar_reachable=lambda o:o['material_reachable'],paid_construction=lambda o:o['first_child'] is not None and o['first_child']['paid_construction_completed'],first_child_reproduced=lambda o:o['first_child_reproduced'],sustained_turnover=lambda o:o['sustained_turnover'])
   counts={k:sum(f(o) for o in edited) for k,f in predicates.items()};weighted={k:.25*float(f(control))+.75*counts[k]/n for k,f in predicates.items()}
   changes=[dict(recipe=o['recipe'],descendant_delta=o['descendants']-control['descendants'],live_delta=o['live_inherited_at_64']-control['live_inherited_at_64'],harvest_delta=o['lineage_harvested']-control['lineage_harvested']) for o in edited]
   groups[f'b{budget}_{program}']=dict(edit_denominator=n,counts=counts,source_kernel_mass_on_this_diagnostic=weighted,unchanged_control=control,all_control_differences=changes)
 paired=[]
 for i in range(58):
  old,new=outcomes[i]['outcome'],outcomes[i+58]['outcome'];req(old['recipe']==new['recipe'],'paired budgets')
  paired.append(dict(recipe=old['recipe'],descendant_delta=new['descendants']-old['descendants'],live_delta=new['live_inherited_at_64']-old['live_inherited_at_64'],harvest_delta=new['lineage_harvested']-old['lineage_harvested'],first_request_tick_b12=old['first_request_tick'],first_request_tick_b24=new['first_request_tick']))
 return dict(profile=PROFILE,audit_pass=True,development_only=True,scientific_run_launched=False,gate_pass=False,training_admitted=False,evolution_enabled=False,reproductive_fairness_accepted=False,cells_verified=116,edited_cells_verified=112,native_advances_verified=334080,scope='one exposed seed; parent-paid endowments 12/24; supplied founder exclusion; faithful descendants; no selection',audit_seconds=time.monotonic()-start,source_sha256={k:hashlib.sha256(v.encode()).hexdigest() for k,v in sources.items()},programs=groups,all_budget_differences=paired)
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('directory');print(json.dumps(verify(p.parse_args().directory),indent=2))
