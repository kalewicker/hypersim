#!/usr/bin/env python3
"""Generic bounded faithful-copy ecology, derived from frozen D0 interpreter laws."""
import copy
import hashlib
from pathlib import Path
import tomllib
from generate_evolving_minds_choice_rules import emit
from generate_evolving_minds_d0_rules import *

BASE=Path(__file__).resolve().parents[1]/'rules/evolving_minds/d0_v1.toml'
GENOME_LEN,PARTS=242012,242013
COUNTER,DEAD=0xE1000006,0xE1000007
NEIGHBOR=0xE1000065
OCCUPIED,PARENT,NEXT_ID=240006,240008,240009

def copy_cost(length):return 1.0+0.25*(11+length)
def seed_upkeep(length):return 2.0+0.125*length

def derive():
    raw=BASE.read_bytes();assert hashlib.sha256(raw).hexdigest()=='af93878599c31468469f503d77a7393818e0b44efebb84dbd280f80db9ec7a5a'
    laws=copy.deepcopy(tomllib.loads(raw.decode())['rules'])
    for law in laws:
        if law['name'].startswith(('development_create_','development_wire_')):
            for fx in law['rhs']['property_effects']:
                if fx['key']==MAINT and fx['op']=='add':fx['value']=0.25
            law['rhs']['property_effects'].append(effect('r',PARTS,'add',1))
    for length in range(1,9):
        labels=dict(r=ROOT,site=SITE,target=SITE,counter=COUNTER)
        labels.update({f'g{i}':GENE for i in range(length)})
        # Traverse the ordered chain before checking the remaining owner spokes.
        # Binding every same-label gene spoke first creates a factorial search.
        links=[(OCC,('r','site')),(NEIGHBOR,('site','target')),(OWN,('r','g0'))]+[(NEXT,(f'g{i-1}',f'g{i}')) for i in range(1,length)]+[(OWN,('r',f'g{i}')) for i in range(1,length)]
        guards=[guard('r',STATUS,'eq',2),guard('r',GENOME_LEN,'eq',length),guard('r',AGE,'gte',2),guard('r',ENERGY,'gte',12.0+copy_cost(length)),guard('target',OCCUPIED,'eq',False)]
        compares=[]
        for i in range(length):guards.append(guard(f'g{i}',POSITION,'eq',i));compares.append(compare(f'g{i}',OWNER,'eq','r',BID))
        new=dict(child=ROOT);new.update({f'slot{i}':SLOT for i in range(8)});new.update({f'sensor{i}':SENSOR for i in range(2)});new.update({f'cg{i}':GENE for i in range(length)})
        newlinks=[(OCC,('child','target'))]+[(OWN,('child',v)) for v in new if v!='child']+[(MAP,(f'slot{i}',f'sensor{i}')) for i in range(2)]+[(NEXT,(f'cg{i-1}',f'cg{i}')) for i in range(1,length)]
        fx=[effect('r',ENERGY,'subtract',12.0+copy_cost(length)),effect('target',OCCUPIED,'set',True),cp('child',BID,'counter',NEXT_ID),cp('child',PARENT,'r',BID),effect('child',ENERGY,'set',12.0),effect('child',AGE,'set',0),effect('child',CURSOR,'set',0),effect('child',STATUS,'set',0),effect('child',HAS_ACT,'set',False),effect('child',ATTEMPT,'set',False),effect('child',CHOICE,'set',False),effect('child',GENOME_LEN,'set',length),effect('child',PARTS,'set',10+length),effect('child',MAINT,'set',seed_upkeep(length))]
        for i in range(8):fx += [cp(f'slot{i}',OWNER,'child',BID),effect(f'slot{i}',DEST,'set',i),effect(f'slot{i}',USED,'set',i<2)]
        for i in range(2):fx += [cp(f'sensor{i}',OWNER,'child',BID),effect(f'sensor{i}',DEST,'set',i),effect(f'sensor{i}',SIGNAL,'set',False)]
        for i in range(length):
            fx.append(cp(f'cg{i}',OWNER,'child',BID))
            for key in [POSITION,IDENTITY,OP,DEST,SRC,INITIAL]:fx.append(cp(f'cg{i}',key,f'g{i}',key))
        fx.append(effect('counter',NEXT_ID,'add',1))
        laws.append(rule(f'birth_faithful_{length}',labels,links,guards,compares,fx,new,newlinks))
    for age in [True,False]:
        guards=[guard('r',AGE,'gte',16)] if age else [guard('r',ENERGY,'lte',0.0)]
        law=rule('death_age' if age else 'death_empty',{'r':ROOT,'site':SITE},[(OCC,('r','site'))],guards,
                 effects=[dict(var='site',key=ENERGY,op='add_from_property',source_var='r',source_key=ENERGY),effect('r',ENERGY,'set',0.0),effect('site',OCCUPIED,'set',False)],priority=1 if age else 0)
        law['rhs']['edges']=[];law['rhs']['edge_labels']={};law['rhs']['node_labels']={'r':DEAD};laws.append(law)
    for label in [SENSOR,GENE,CONNECTION,ACTUATOR,STATE,SLOT]:
        law=rule(f'cleanup_member_{label}',{'r':DEAD,'p':label},[(OWN,('r','p'))],
                 [guard('r',PARTS,'gte',1)],[compare('p',OWNER,'eq','r',BID)],[effect('r',PARTS,'subtract',1)])
        law['rhs']['nodes']=['r'];law['rhs']['edges']=[];law['rhs']['edge_labels']={};laws.append(law)
    law=rule('cleanup_root',{'r':DEAD},[],[guard('r',PARTS,'eq',0),guard('r',ENERGY,'eq',0.0)])
    law['rhs']['nodes']=[];laws.append(law)
    return '# Bounded paid faithful D0 family laws, v1.\n'+'\n\n'.join('[[rules]]\n'+'\n'.join(emit(r,'rules')) for r in laws)+'\n'

if __name__=='__main__':
    target=BASE.with_name('family_v1.toml');text=derive();target.write_text(text);print(hashlib.sha256(text.encode()).hexdigest())
