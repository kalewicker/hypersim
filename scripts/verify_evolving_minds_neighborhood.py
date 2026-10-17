#!/usr/bin/env python3
"""Independent complete-denominator paid source-edit calibration audit."""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import time
import tomllib
import importlib.util
import sys

PROFILE='evolving-minds-d0-neighborhood-v1'
def req(ok,why):
 if not ok:raise ValueError(why)
def load(p,n):
 spec=importlib.util.spec_from_file_location(n,p);m=importlib.util.module_from_spec(spec);old=sys.dont_write_bytecode
 try:sys.dont_write_bytecode=True;spec.loader.exec_module(m)
 finally:sys.dont_write_bytecode=old
 return m
def read(p):return json.loads(p.read_text())
def recipes(d):
 result=[]
 for name in ['cue','delayed_cue']:
  result.append(dict(program=name,target_id=None,new_source=None))
  for g in sorted(d.program(name)['instructions'],key=lambda x:x['id']):
   result.extend(dict(program=name,target_id=g['id'],new_source=source) for source in range(8) if source!=g['source'])
 return result
def name(r):return r['program']+'_unchanged' if r['target_id'] is None else f"{r['program']}_g{r['target_id']}_s{r['new_source']}"
def verify(root):
 start=time.monotonic();root=Path(root);req(root.is_dir() and not root.is_symlink() and not any(p.is_symlink() for p in root.rglob('*')),'neighborhood evidence path');req(sum(p.stat().st_size for p in root.rglob('*') if p.is_file())<=256*1024*1024,'neighborhood panel ceiling')
 sources=read(root/'sources.json')
 for f,p in [('checker.py','scripts/verify_evolving_minds_neighborhood.py'),('mutation_reference.py','scripts/verify_evolving_minds_mutation.py'),('d0_reference.py','scripts/verify_evolving_minds_d0.py'),('replay.py','scripts/verify_evolving_minds.py'),('schedule_reference.py','scripts/verify_evolving_minds_choice.py'),('base_rules.toml','rules/evolving_minds/mutation_v1.toml'),('contract.md','experiments/evolving_minds/contracts/d0_neighborhood_v1.md')]:req((root/f).read_text()==sources[p],'frozen source binding')
 modules=[load(root/f,n) for f,n in [('mutation_reference.py','nbr_mut'),('replay.py','nbr_replay'),('d0_reference.py','nbr_d0'),('schedule_reference.py','nbr_schedule')]];a,c,d,s=modules;base=(root/'base_rules.toml').read_bytes();req(hashlib.sha256(base).hexdigest()==a.LAW_SHA,'fixed physiology law identity')
 cells=recipes(d);req(len(cells)==58 and read(root/'registered_cells.json')==cells and {p.name for p in root.iterdir() if p.is_dir()}=={name(r) for r in cells},'complete fixed 58-cell denominator')
 after=copy.deepcopy([r for r in tomllib.loads(base.decode())['rules'] if r['name'].startswith('reserve_')])
 for law in after:law['lhs']['node_property_guards'].append(dict(var='r',property=c.BID,op='gte',value=2))
 outcomes=[]
 for recipe in cells:
  p=root/name(recipe);req(read(p/'recipe.json')==recipe,'cell recipe');req(tomllib.loads((p/'descendant_reserve.toml').read_text())==dict(rules=after),'native descendant eligibility law');resources=read(p/'resources.json');req(0<=resources['advance_seconds']<=120 and resources['native_advances']==2880,'native resource bounds')
  for f in p.iterdir():
   if f.suffix=='.json':req(f.stat().st_size<=4*1024*1024,'cell JSON ceiling')
  _,outcome=a.verify_cell(p,c,d,s,base,recipe['program'],'Informative','Faithful',neighborhood=recipe);outcomes.append(outcome)
 req(read(root/'summary.json')==outcomes,'complete outcomes inventory')
 status=read(root/'run_status.json');req(status['profile']==PROFILE and status['engineering_execution_complete'] is True and status['development_only'] is True and status['mutation_mechanism_enabled'] is True and all(status[k] is False for k in ['scientific_run_launched','gate_pass','training_admitted','evolution_enabled','reproductive_fairness_accepted']),'root research boundary');m=read(root/'manifest.json');req(m['state']=='Completed' and m['seed']==17 and m['max_steps']==m['final_step']==58*2880,'complete panel manifest')
 summaries={}
 for program,n in [('cue',21),('delayed_cue',35)]:
  group=[o for o in outcomes if o['recipe']['program']==program];control=next(o for o in group if o['recipe']['target_id'] is None);edited=[o for o in group if o['recipe']['target_id'] is not None];req(len(edited)==n,'fixed edit denominator')
  predicates=dict(syntax_valid=lambda o:o['syntax_valid'],grammar_reachable=lambda o:o['material_reachable'],paid_construction=lambda o:o['first_child'] is not None and o['first_child']['paid_construction_completed'],first_child_reproduced=lambda o:o['first_child_reproduced'],sustained_turnover=lambda o:o['sustained_turnover'])
  counts={k:sum(f(o) for o in edited) for k,f in predicates.items()};weighted={k:0.25*float(f(control))+0.75*counts[k]/n for k,f in predicates.items()}
  improvements=[dict(recipe=o['recipe'],descendant_delta=o['descendants']-control['descendants'],live_delta=o['live_inherited_at_64']-control['live_inherited_at_64'],harvest_delta=o['lineage_harvested']-control['lineage_harvested']) for o in edited]
  summaries[program]=dict(edit_denominator=n,counts=counts,source_kernel_mass_on_this_diagnostic=weighted,unchanged_control=control,all_control_differences=improvements)
 return dict(profile=PROFILE,audit_pass=True,development_only=True,scientific_run_launched=False,gate_pass=False,training_admitted=False,evolution_enabled=False,reproductive_fairness_accepted=False,cells_verified=58,edit_denominator=56,native_advances_verified=167040,scope='one exposed seed; supplied founder exclusion; faithful descendants; no selection',audit_seconds=time.monotonic()-start,source_sha256={k:hashlib.sha256(v.encode()).hexdigest() for k,v in sources.items()},programs=summaries)
if __name__=='__main__':
 parser=argparse.ArgumentParser();parser.add_argument('directory');print(json.dumps(verify(parser.parse_args().directory),indent=2))
