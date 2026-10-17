#!/usr/bin/env python3
"""Independent catalogue accounting/membership audit; no fitness or physics."""
import argparse
import collections
import hashlib
import json
import shutil
import tempfile
from pathlib import Path

PROFILE = 'E2-complete-valid-production-grammar-v1'
OPS = ['AddSlot','RemoveSlot','RetargetSlot','AddVar','RemoveVar','AddPatternEdge',
       'RemovePatternEdge','AddLabel','RemoveLabel','ChangeLabel','ChangePriority','Activate','Deactivate']

def load(p):
    return json.loads(p.read_text())

def digest(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()

def audit(out,integrity=True):
    m=load(out/'manifest.json')
    assert m['profile']==PROFILE and m['status']=='complete'
    assert m['frozen_before_compilation'] and not m['training_admitted']
    assert m['engine_steps']==m['optimizer_rollouts']==0
    if integrity:
        assert digest(out/'executed_binary')==m['binary_sha256']
        assert digest(out/'catalogue.json')==m['catalogue_sha256']
        assert digest(out/'emission_checks.json')==m['emission_checks_sha256']
        for p,h in m['sources'].items():
            assert digest(out/'source_snapshots'/p)==h,p
        if (out/'output_sha256.json').exists():
            for p,h in load(out/'output_sha256.json').items():
                assert digest(out/p)==h,p
    c=load(out/'catalogue.json')
    assert c['profile']==PROFILE and c['params']==183 and c['label_values']==[0,1]
    assert c['engine_steps']==0 and not c['training_admitted']
    assert len(c['parents'])==168
    totals=collections.Counter()
    per_op=[0]*13
    masked_groups=0
    for p,parent in enumerate(c['parents']):
        assert parent['parent']==p and len(parent['groups'])==13
        operator_support=[]
        for op,group in enumerate(parent['groups']):
            assert group['operator']==op and group['name']==OPS[op] and len(group['roles'])==2
            support=[]
            for role,rs in enumerate(group['roles']):
                assert rs['role']==role
                entries=rs['productions']
                stats=rs['stats']
                assert all(k in ['Applied','NoOp','Inapplicable','RejectedPolicy','RejectedPostcondition','RejectedInput'] and isinstance(n,int) and n>=0 for k,n in stats.items())
                assert stats.get('Applied',0)==len(entries)
                totals.update(stats)
                per_op[op]+=len(entries)
                if entries:
                    support.append(role)
                else:
                    masked_groups+=1
                requests=set()
                for i,e in enumerate(entries):
                    assert e['index']==i and e['execution'] and e['rcf']
                    request=e['request']
                    if op<11:
                        assert isinstance(request,dict) or request=='ChangePriority'
                    assert (list(request)[0] if isinstance(request,dict) else request)==OPS[op]
                    packed=json.dumps(request,sort_keys=True)
                    assert packed not in requests
                    requests.add(packed)
                    if op in [7,9]:
                        assert request[OPS[op]]['value'] in [0,1]
                    if op>=11:
                        assert role==1 and len(entries)==1 and e['execution']!=parent['initial_execution']
            assert group['role_support']==support
            if support:
                operator_support.append(op)
        assert parent['operator_support']==operator_support and operator_support
        # Mode toggle has exactly one supported direction for every seed parent.
        assert len(parent['groups'][11]['roles'][1]['productions'])+len(parent['groups'][12]['roles'][1]['productions'])==1
    assert all(n>0 for n in per_op)
    rows=load(out/'emission_checks.json')
    assert len(rows)==64 and [r['emission_seed'] for r in rows]==list(range(64))
    world_hashes=set()
    for r in rows:
        p,op=r['parent'],r['operator']
        role=['Core','Proposal'].index(r['role'])
        group=c['parents'][p]['groups'][op]
        entries=group['roles'][role]['productions']
        e=entries[r['production_index']]
        assert r['grammar_profile']==PROFILE and r['admitted'] and r['status']=='Applied'
        assert r['production_count']==len(entries)
        assert r['signature']==e['execution'] and r['classes']['rcf']==e['rcf']
        assert r['operator_support']==c['parents'][p]['operator_support']
        assert r['role_support']==group['role_support']
        assert r['kernel_attempts']==1 and not r['resampled'] and not r['parent_fallback']
        k=r['kernel']
        assert k['request']==e['request'] and k['role']==r['role'] and k['status']=='Applied'
        assert k['committed'] and not k['training_admitted']
        assert k['advancing_engine_steps']==0 and not k['codec_reencoded'] and not k['metadata_recounted']
        assert k['before']['step']==k['staged']['step']==0
        assert r['root']==c['parents'][p]['root']
        world_hashes.add(r['initial_world_hash'])
    assert len(world_hashes)==1
    return {'audit_pass':True,'scope':'catalogue accounting and membership, not independent semantic decode or physics',
            'parents':168,'enumerated_requests':sum(totals.values()),'legal_productions':totals['Applied'],
            'compiler_outcomes':dict(totals),'productions_by_operator':dict(zip(OPS,per_op)),
            'masked_parent_operator_role_groups':masked_groups,'verified_emissions':64,
            'engine_steps':0,'optimizer_rollouts':0,'training_admitted':False}

def tamper_tests(out):
    cases=['mask','stats','request','signature','retry','fallback','parent','label','physics','duplicate']
    for case in cases:
        with tempfile.TemporaryDirectory(prefix='ere-grammar-tamper-') as tmp:
            dst=Path(tmp)
            for name in ['manifest.json','catalogue.json','emission_checks.json']:
                shutil.copyfile(out/name,dst/name)
            c=load(dst/'catalogue.json');rows=load(dst/'emission_checks.json');m=load(dst/'manifest.json')
            if case=='mask':c['parents'][0]['operator_support']=[]
            elif case=='stats':c['parents'][0]['groups'][10]['roles'][0]['stats']['Applied']+=1
            elif case=='request':rows[0]['kernel']['request']='invalid'
            elif case=='signature':rows[0]['signature']='bad'
            elif case=='retry':rows[0]['kernel_attempts']=2
            elif case=='fallback':rows[0]['parent_fallback']=True
            elif case=='parent':c['parents'].pop()
            elif case=='label':
                entry=next(e for p in c['parents'] for e in p['groups'][7]['roles'][0]['productions'])
                entry['request']['AddLabel']['value']=2
            elif case=='physics':m['engine_steps']=1
            elif case=='duplicate':
                rs=next(rs for p in c['parents'] for g in p['groups'] for rs in g['roles'] if len(rs['productions'])>1)
                rs['productions'][1]['request']=rs['productions'][0]['request']
            for name,obj in [('manifest.json',m),('catalogue.json',c),('emission_checks.json',rows)]:
                (dst/name).write_text(json.dumps(obj))
            try:
                audit(dst,integrity=False)
            except (AssertionError,IndexError,KeyError,ValueError):
                continue
            raise AssertionError(f'tamper accepted: {case}')
    return len(cases)

if __name__=='__main__':
    parser=argparse.ArgumentParser()
    parser.add_argument('output',type=Path)
    parser.add_argument('--tamper-tests',action='store_true')
    parser.add_argument('--seal',action='store_true')
    args=parser.parse_args()
    result=audit(args.output)
    if args.tamper_tests:
        result['tamper_rejections']=tamper_tests(args.output)
    if args.seal:
        (args.output/'audit.json').write_text(json.dumps(result,indent=2)+'\n')
        paths=sorted(p for p in args.output.rglob('*') if p.is_file() and p!=args.output/'output_sha256.json')
        (args.output/'output_sha256.json').write_text(json.dumps({str(p.relative_to(args.output)):digest(p) for p in paths},indent=2,sort_keys=True)+'\n')
        audit(args.output)
    print(json.dumps(result,indent=2))
