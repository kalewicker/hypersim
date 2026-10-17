#!/usr/bin/env python3
"""Standalone arithmetic/input/custody verification, not physics replay."""
import argparse
import hashlib
import json
import math
from pathlib import Path
import re

ROOTS = {
    'primary': 'bf034c9962c153f4465d8a831de0ad9107c48613f8b9c8635f2ec98ea04414a1',
    'correction': 'b8ddf8f15f4cb899c1b340031c3180143b3e5967855ecc462b16f385c0e32739',
    'completion_v3': 'cf4935ab7fee1f7ab66c85499dc304add8f14f0f7128f82f203d4ae1d1ebb483',
    'completion_v4': '303338086096ed6f6cee33774f93de5ba50a91ac143dc3b073a6fc3c8a795131',
}
COUNTS = dict(primary=341, correction=323, completion_v3=12, completion_v4=14)
MODES = ['full', 'forecast_masked', 'bounty_masked', 'cues_masked']
THETA = [0.07, -0.11, 0.13, -0.17]
def load(p):
    return json.loads(Path(p).read_text())
def digest(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def close(a, b, tol=2e-8):
    if isinstance(b, list):
        assert isinstance(a, list) and len(a) == len(b)
        for x, y in zip(a, b): close(x, y, tol)
    else:
        assert isinstance(a, (int, float)) and math.isfinite(a) and abs(a-b) <= tol, (a,b)
def verify_seal(p, expected=None, count=None):
    p = Path(p)
    if expected is not None: assert digest(p/'output_sha256.json') == expected
    entries = load(p/'output_sha256.json')
    if count is not None: assert len(entries) == count
    actual = {x.relative_to(p).as_posix() for x in p.rglob('*') if x.is_file() and x.name != 'output_sha256.json'}
    assert actual == set(entries), 'archive membership mismatch'
    for n,h in entries.items():
        dest = (p/n).resolve(); assert dest.is_relative_to(p.resolve()) and digest(dest) == h, n
    return len(entries)
def node_key(s):
    m = re.fullmatch(r'NodeId\((0|[1-9][0-9]*)v([1-9][0-9]*)\)', s)
    assert m; return int(m[1]), int(m[2])
def projection(raw):
    assert raw['step'] == 13 and type(raw['step']) is int
    nodes = {}; selected = {}
    for k, row in raw['nodes']:
        key = node_key(k); assert key not in nodes; nodes[key] = row
        label = row['data']['label']; assert type(label) is int and 0 <= label <= 2**32-1
        if label not in (4293949189,4293949188,4293949190): continue
        assert label not in selected and row['incident'] == []
        props = {}
        for p,v in row['data']['properties']:
            assert type(p) is int and p not in props; props[p] = v
        selected[label] = (key,props)
    assert len(selected) == 3
    seen = set()
    for k,row in raw['edges']:
        assert re.fullmatch(r'EdgeId\((0|[1-9][0-9]*)v([1-9][0-9]*)\)',k) and k not in seen;seen.add(k)
        for ref in row['nodes']:
            assert set(ref) == {'idx','version'} and all(type(ref[k]) is int for k in ref)
            key=(ref['idx'],ref['version']);assert key in nodes and ref['version']>0
            assert key not in [x[0] for x in selected.values()]
    def prop(label,p,kind):
        value=selected[label][1][p];assert set(value)=={kind}
        n=value[kind];assert type(n) in ((int,) if kind=='Int' else (float,int)) and math.isfinite(n)
        return n
    w=prop(4293949189,1,'Float');f=prop(4293949188,30469,'Int');b=prop(4293949190,30470,'Int')
    assert prop(4293949188,30468,'Int')==prop(4293949190,30471,'Int')==0
    assert w in [0,1] and f in [0,1] and b in [2,12]
    return dict(wallet=w,forecast=f,bounty=b)
def probability(t,x):
    z=sum(a*b for a,b in zip(t,x));p=1/(1+math.exp(-z));return [p,1-p]
def audit(root):
    root=Path(root);m=load(root/'manifest.json');design=load(root/'source_snapshots/configs/ere_resource_adapter_v1.json')
    assert design['modes']==MODES and design['arithmetic_probe_parameters']==THETA and design['states']==8 and design['source_cells']==16
    assert design['parameters']==4 and design['initial_parameters']==[0.]*4 and design['bounty_scale']==12
    assert design['finite_difference_epsilon']==1e-6 and design['derivative_tolerance']==2e-8
    for flag in ['native_advances','optimizer_updates','scientific_acceptance','training_admitted','sustained_claim','independent_physics_replay']:
        assert m[flag]==design[flag]==0
    maps={}
    for name,h in ROOTS.items():
        path=root/f'inputs/{name}_seal.json';assert digest(path)==h
        maps[name]=load(path);assert len(maps[name])==COUNTS[name]
    for local, origin in m['imported_files'].items():
        assert digest(root/local)==maps[origin['archive']][origin['path']]
    for n,h in m['sources'].items(): assert digest(root/'source_snapshots'/n)==h
    assert digest(root/'test_binary')==m['binary_sha256']
    prior=load(root/'inputs/allocation_audit.json');assert prior['audit_pass'] and prior['diagnostic_pass'] and prior['cells'].__len__()==16
    data=load(root/'adapter_records.json');assert data['theta']==THETA and len(data['records'])==32 and data['native_advances']==data['optimizer_updates']==0
    projections=[];utilities=[];derivatives=0;max_error=0.;inputs={mode:[] for mode in MODES}
    for state in range(8):
        pair=[load(root/f'inputs/cell-{state*2+a:04}.json') for a in range(2)]
        assert pair[0]['allocation_observation']==pair[1]['allocation_observation']
        o=projection(pair[0]['allocation_observation']['inventory']);projections.append(o)
        assert pair[0]['scenario']==['hazard_low','hazard_high','safe_low','safe_high'][state//2]
        assert pair[0]['resource']==['paid','unpaid'][state%2]
        u=[]
        for action,receipt in enumerate(pair):
            cell=prior['cells'][state*2+action]
            assert receipt['action']==cell['action']==['defend','harvest'][action]
            assert cell['scenario']==receipt['scenario'] and cell['resource']==receipt['resource'] and cell['id']==state*2+action
            fires=[r['step'] for r in receipt['timeline'] if 17<=r['step']<=32 for e in r['events'] if e['rcf']==cell['measured']['identity'] and e['kind']=='world']
            assert fires==cell['post_resolution_fires']
            credits=receipt['summary']['harvested_credits'];assert credits==cell['harvested_credits']
            u.append(len(fires)+credits);assert u[-1]==cell['utility']
        utilities.append(u)
        for mi,mode in enumerate(MODES):
            row=data['records'][state*4+mi];assert row['state']==state and row['mode']==mode and row['observation']==o
            x=[1.,o['wallet'],0. if mode in ['forecast_masked','cues_masked'] else o['forecast'],0. if mode in ['bounty_masked','cues_masked'] else o['bounty']/12.]
            inputs[mode].append(tuple(x));close(row['input'],x);close(row['utility'],u)
            p,q=probability(THETA,x);close(row['probabilities'],[p,q]);close(row['neutral'],[.5,.5]);assert row['boundary_sample']==1
            close(row['expected_utility'],p*u[0]+q*u[1]);g=[v*p*q*(u[0]-u[1]) for v in x];close(row['utility_gradient'],g)
            assert len(row['actions'])==2
            for action,record in enumerate(row['actions']):
                assert record['action']==record['sample']==action
                assert record['request']==(['defend','harvest'][action] if o['wallet']==1 else None)
                close(record['gradient'],[v*((1 if action==0 else 0)-p) for v in x])
                for i in range(4):
                    hi=THETA.copy();lo=THETA.copy();hi[i]+=1e-6;lo[i]-=1e-6
                    fd=(math.log(probability(hi,x)[action])-math.log(probability(lo,x)[action]))/2e-6
                    error=abs(record['gradient'][i]-fd);assert error<2e-8;max_error=max(max_error,error);derivatives+=1
            for i in range(4):
                hi=THETA.copy();lo=THETA.copy();hi[i]+=1e-6;lo[i]-=1e-6
                value=lambda t:sum(a*b for a,b in zip(probability(t,x),u))
                fd=(value(hi)-value(lo))/2e-6;error=abs(g[i]-fd);assert error<2e-8;max_error=max(max_error,error);derivatives+=1
    assert len({tuple(o.values()) for o in projections})==8
    unique={k:len(set(v)) for k,v in inputs.items()};assert unique==dict(full=8,forecast_masked=4,bounty_masked=4,cues_masked=2)
    assert utilities==[[8,2],[0,0],[8,12],[0,0],[8,10],[8,8],[8,20],[8,8]] and derivatives==384
    return dict(profile=design['profile'],engineering_pass=True,states=8,source_cells=16,score_records=64,utility_records=32,derivatives=derivatives,max_derivative_error=max_error,unique_inputs=unique,neutral_paid_mean=9.5,native_advances=0,optimizer_updates=0,scientific_acceptance=False,training_admitted=False,sustained_claim=False,independent_physics_replay=False)
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('archive',type=Path);p.add_argument('--seal',action='store_true');args=p.parse_args()
    if args.seal: verify_seal(args.archive)
    print(json.dumps(audit(args.archive),indent=2))
