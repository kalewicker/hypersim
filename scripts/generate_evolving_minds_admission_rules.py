#!/usr/bin/env python3
"""Separate ecology laws: local tick identities, parallel escrow admission."""
import copy,hashlib,tomllib
from pathlib import Path
from generate_evolving_minds_family_rules import *
LOCAL=242018
REQUEST=242014
BASE_MUTATION=BASE.with_name('mutation_v1.toml')
def derive():
 raw=BASE_MUTATION.read_bytes();assert hashlib.sha256(raw).hexdigest()=='0656c1341373e1fa3bd31118d4e4b8865810478d7d89cf744a0548514312c474'
 laws=copy.deepcopy(tomllib.loads(raw.decode())['rules'])
 for r in laws:
  if r['name'].startswith(('reserve_','transaction_copy_')):
   for side in ['lhs','rhs']:
    t=r[side];t['nodes'].remove('counter');t.get('node_labels',{}).pop('counter',None)
    for guard_ in t.get('var_guards',[]):guard_['vars']=[v for v in guard_['vars'] if v!='counter']
   fx=r['rhs']['property_effects'];r['rhs']['property_effects']=[f for f in fx if f['var']!='counter']
   for f in r['rhs']['property_effects']:
    if f.get('source_var')=='counter':f['source_var']='target';f['source_key']=LOCAL
 laws.append(rule('identity_advance',{'site':SITE},[],effects=[effect('site',LOCAL,'add',8)]))
 return '# Parallel ecology admission with local tick identities v1.\n'+'\n\n'.join('[[rules]]\n'+'\n'.join(emit(r,'rules')) for r in laws)+'\n'
if __name__=='__main__':
 text=derive();BASE.with_name('admission_v1.toml').write_text(text);print(hashlib.sha256(text.encode()).hexdigest())
