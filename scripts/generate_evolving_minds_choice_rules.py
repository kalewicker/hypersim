#!/usr/bin/env python3
"""Deterministic derivation of the separate authored choice-law profile."""
import copy
import hashlib
import json
from pathlib import Path
import tomllib

ROOT=Path(__file__).resolve().parents[1]
BASE=ROOT/'rules/evolving_minds/engineering_v1.toml'
STATE,CUE,COIN,SENSOR_COIN,POLICY,ATTEMPT,CHOICE=range(241002,241009)


def emit(table,path):
    lines=[]
    for k,v in table.items():
        if not isinstance(v,dict) and not (isinstance(v,list) and v and isinstance(v[0],dict)):
            lines.append(f'{json.dumps(k)} = {json.dumps(v)}')
    for k,v in table.items():
        sub=path+'.'+k
        if isinstance(v,dict):lines.extend(['['+sub+']',*emit(v,sub)])
        elif isinstance(v,list) and v and isinstance(v[0],dict):
            for row in v:lines.extend(['[['+sub+']]',*emit(row,sub)])
    return lines


def derive():
    raw=BASE.read_bytes()
    assert hashlib.sha256(raw).hexdigest()=='75d44aaeadb4977858fd5ed49cb4f3c222863f5b37218a521c28f1f4be5f607a'
    base=tomllib.loads(raw.decode())['rules']
    rules=[copy.deepcopy(r) for r in base if not r['name'].startswith(('sense_','signal_','action_')) and r['name']!='development_idle']
    development=next(r for r in rules if r['name']=='development_relay')
    development['lhs']['node_property_guards']=[g for g in development['lhs']['node_property_guards'] if not (g['var']=='g' and g['property']==240002)]
    birth=next(r for r in rules if r['name']=='birth_faithful')
    birth['rhs']['property_effects'] += [dict(var='child',key=POLICY,op='copy_from',source_var='g',source_key=240002),dict(var='child',key=ATTEMPT,op='set',value=False),dict(var='child',key=CHOICE,op='set',value=False),dict(var='cs',key=SENSOR_COIN,op='set',value=False)]
    sense=copy.deepcopy(next(r for r in base if r['name']=='sense_available'))
    sense['name']='sense_choice'
    sense['lhs']['node_property_guards']=[]
    sense['rhs']['property_effects']=[dict(var='s',key=240005,op='copy_from',source_var='site',source_key=CUE),dict(var='s',key=SENSOR_COIN,op='copy_from',source_var='site',source_key=COIN)]
    rules.append(sense)
    for policy in range(5):
        signal=copy.deepcopy(next(r for r in base if r['name']=='signal_relay'))
        signal['name']=f'signal_policy_{policy}'
        signal['lhs']['node_property_guards']=[dict(var='r',property=POLICY,op='eq',value=policy)]
        signal['rhs']['property_effects']=[dict(var='a',key=240005,op='set',value=policy==2)] if policy<3 else [dict(var='a',key=240005,op='copy_from',source_var='s',source_key=SENSOR_COIN if policy==3 else 240005)]
        rules.append(signal)
    attempt=copy.deepcopy(next(r for r in base if r['name']=='action_intake'))
    attempt['name']='attempt_commit'
    attempt['lhs']['node_property_guards']=[dict(var='r',property=POLICY,op='gte',value=1),dict(var='r',property=240004,op='gte',value=1),dict(var='r',property=ATTEMPT,op='eq',value=False),dict(var='r',property=1,op='gte',value=1.0)]
    attempt['rhs']['property_effects']=[dict(var='r',key=1,op='subtract',value=1.0),dict(var='r',key=CHOICE,op='copy_from',source_var='a',source_key=240005),dict(var='r',key=ATTEMPT,op='set',value=True)]
    rules.append(attempt)
    for success in [True,False]:
        settle=dict(name='settle_success' if success else 'settle_failure',priority=1 if success else 0,
            lhs=dict(nodes=['r','site'],edges=[['r','site']],node_labels=dict(r=3774873600,site=3774873601),edge_labels={'0':3774873700},var_guards=[dict(type='vars_distinct',vars=['r','site'])],node_property_guards=[dict(var='r',property=ATTEMPT,op='eq',value=True)]),
            rhs=dict(nodes=['r','site'],edges=[['r','site']],edge_labels={'0':3774873700},property_effects=[dict(var='r',key=ATTEMPT,op='set',value=False)]))
        if success:
            settle['lhs']['node_property_guards'] += [dict(var='r',property=1,op='lte',value=40.0),dict(var='site',property=1,op='gte',value=8.0)]
            settle['lhs']['node_property_compares']=[dict(var_a='r',key_a=CHOICE,op='eq',var_b='site',key_b=STATE)]
            settle['rhs']['property_effects'] += [dict(var='site',key=1,op='subtract',value=8.0),dict(var='r',key=1,op='add',value=8.0)]
        rules.append(settle)
    return '# Separate evolving-minds-choice-v1 laws; generated from frozen v1.\n'+ '\n\n'.join('[[rules]]\n'+'\n'.join(emit(r,'rules')) for r in rules)+'\n'


if __name__=='__main__':
    target=ROOT/'rules/evolving_minds/choice_v1.toml'
    text=derive()
    target.write_text(text)
    print(hashlib.sha256(text.encode()).hexdigest())
