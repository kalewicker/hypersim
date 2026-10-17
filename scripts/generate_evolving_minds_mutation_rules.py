#!/usr/bin/env python3
"""Derive native admission, escrow settlement and copying from faithful v1."""
import copy,hashlib,tomllib
from pathlib import Path
from generate_evolving_minds_family_rules import *
ESCROW=0xE1020000
REQUEST,TX=242014,242015
FUNDS,PLACE=ESCROW+100,ESCROW+101
BASE_FAMILY=BASE.with_name('family_v1.toml')
def derive():
 raw=BASE_FAMILY.read_bytes();assert hashlib.sha256(raw).hexdigest()=='fb3337159587ac09e42f6ab1d1e5d3a84c40eefc0ba6e2f782011f5e6bc38902'
 original=tomllib.loads(raw.decode())['rules'];laws=[copy.deepcopy(r) for r in original if not r['name'].startswith('birth_')]
 for length in range(1,9):
  template=copy.deepcopy(next(r for r in original if r['name']==f'birth_faithful_{length}'))
  admission=copy.deepcopy(template);admission['name']=f'reserve_{length}'
  for g in admission['lhs']['node_property_guards']:
   if g['var']=='r' and g['property']==ENERGY:g['value']+=1.0
  labels={v:admission['lhs']['node_labels'][v] for v in admission['lhs']['nodes']};links=[(admission['lhs']['edge_labels'][str(i)],tuple(e)) for i,e in enumerate(admission['lhs']['edges'])]
  admission['rhs']={'nodes':admission['lhs']['nodes']+['escrow'],'edges':admission['lhs']['edges']+[['r','escrow'],['escrow','target']],'node_labels':{'escrow':ESCROW},'edge_labels':{**admission['lhs']['edge_labels'],str(len(links)):FUNDS,str(len(links)+1):PLACE},'property_effects':[effect('r',ENERGY,'subtract',13.0+copy_cost(length)),effect('escrow',ENERGY,'set',12.0+copy_cost(length)),cp('escrow',OWNER,'r',BID),effect('escrow',GENOME_LEN,'set',length),cp('escrow',REQUEST,'counter',REQUEST),effect('escrow',TX,'set',0),effect('counter',REQUEST,'add',1)]}
  laws.append(admission)
  commit=template;commit['name']=f'transaction_copy_{length}'
  for side in ['lhs','rhs']:
   g=commit[side];start=len(g['edges']);g['nodes'].append('escrow');g['edges'] += [['r','escrow'],['escrow','target']];g.setdefault('node_labels',{})['escrow']=ESCROW;g['edge_labels'].update({str(start):FUNDS,str(start+1):PLACE})
  commit['lhs']['node_property_guards']=[g for g in commit['lhs']['node_property_guards'] if not(g['var']=='r' and g['property']==ENERGY)]+[guard('escrow',ENERGY,'eq',12.0+copy_cost(length)),guard('escrow',TX,'eq',0)]
  commit['lhs']['node_property_compares'].append(compare('escrow',OWNER,'eq','r',BID))
  commit['rhs']['property_effects']=[f for f in commit['rhs']['property_effects'] if not(f['var']=='r' and f['key']==ENERGY)]+[effect('escrow',ENERGY,'set',0.0),effect('escrow',TX,'set',1)]
  laws.append(commit)
 reject=rule('transaction_reject',{'r':ROOT,'escrow':ESCROW,'target':SITE},[(FUNDS,('r','escrow')),(PLACE,('escrow','target'))],[guard('escrow',TX,'eq',0)],[compare('escrow',OWNER,'eq','r',BID)],[dict(var='r',key=ENERGY,op='add_from_property',source_var='escrow',source_key=ENERGY),effect('escrow',ENERGY,'set',0.0),effect('escrow',TX,'set',2)])
 laws.append(reject)
 dispose=rule('escrow_dispose',{'escrow':ESCROW},[],[guard('escrow',ENERGY,'eq',0.0),guard('escrow',TX,'gte',1)]);dispose['rhs']['nodes']=[];laws.append(dispose)
 return '# One-shot source mutation and native escrow v1.\n'+'\n\n'.join('[[rules]]\n'+'\n'.join(emit(r,'rules')) for r in laws)+'\n'
if __name__=='__main__':
 text=derive();BASE.with_name('mutation_v1.toml').write_text(text);print(hashlib.sha256(text.encode()).hexdigest())
