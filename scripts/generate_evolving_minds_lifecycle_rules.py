#!/usr/bin/env python3
"""Lawful communication controls plus native prototype arbitration."""
import copy,hashlib,tomllib
from pathlib import Path
from generate_evolving_minds_family_rules import *
ENABLED,OFFER,LOCAL_NEXT=242016,242017,242018
FAMILY=BASE.with_name('family_v1.toml')
def family_rules():
 raw=FAMILY.read_bytes();assert hashlib.sha256(raw).hexdigest()=='fb3337159587ac09e42f6ab1d1e5d3a84c40eefc0ba6e2f782011f5e6bc38902';return tomllib.loads(raw.decode())['rules']
def lifecycle_rules():
 laws=copy.deepcopy(family_rules())
 for r in laws:
  if r['name'].startswith('development_wire_'):r['rhs']['property_effects'].append(effect('c',ENABLED,'set',True))
  if r['name'].startswith('prepare_'):r['lhs']['node_property_guards'].append(guard('c',ENABLED,'eq',True))
 for name,first,last in [('sham',True,True),('freeze',False,False),('rescue',False,True)]:
  labels={'r':ROOT,'g':GENE,'s':SENSOR,'c':CONNECTION,'a':ACTUATOR}
  links=[(OWN,('r','g')),(OWN,('r','c')),(OWN,('r','a')),(OWN,('r','s')),(SOURCE,('s','c')),(TARGET,('c','a'))]
  guards=[guard('g',IDENTITY,'eq',101),guard('g',OP,'eq',2)]
  compares=[compare(v,OWNER,'eq','r',BID) for v in ['g','s','c','a']]+[compare('s',DEST,'eq','g',SRC),compare('a',DEST,'eq','g',DEST)]
  laws.append(rule('communication_'+name,labels,links,guards,compares,[effect('c',ENABLED,'set',first),effect('c',ENABLED,'set',last),effect('a',SIGNAL,'set',False),effect('a',BUFFER,'set',False)]))
 return laws
def arbitration_rules():
 laws=family_rules();warm=copy.deepcopy([r for r in laws if r['name'] in ['development_halt','maintenance_full']]);birth=copy.deepcopy(next(r for r in laws if r['name']=='birth_faithful_1'))
 birth['name']='birth_global';birth['lhs']['node_property_guards'].append(guard('target',OFFER,'eq',True));warm.append(birth)
 local=copy.deepcopy(birth);local['name']='birth_local'
 for side in ['lhs','rhs']:
  local[side]['nodes'].remove('counter');local[side].get('node_labels',{}).pop('counter',None)
  for g in local[side].get('var_guards',[]):g['vars']=[v for v in g['vars'] if v!='counter']
 fx=local['rhs']['property_effects']
 for f in fx:
  if f.get('source_var')=='counter':f['source_var']='target';f['source_key']=LOCAL_NEXT
  if f['var']=='counter':f['var']='target';f['key']=LOCAL_NEXT;f['value']=8
 warm.append(local);return warm

def emit_rules(laws,header):return header+'\n'+'\n\n'.join('[[rules]]\n'+'\n'.join(emit(r,'rules')) for r in laws)+'\n'
def derive():return emit_rules(lifecycle_rules(),'# Full lifecycle lawful communication controls v1.')
def derive_arbitration():return emit_rules(arbitration_rules(),'# Native four-pass birth arbitration diagnostic v1.')
if __name__=='__main__':
 for name,text in [('lifecycle_v1.toml',derive()),('arbitration_v1.toml',derive_arbitration())]:
  BASE.with_name(name).write_text(text);print(name,hashlib.sha256(text.encode()).hexdigest())
