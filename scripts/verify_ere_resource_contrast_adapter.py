#!/usr/bin/env python3
"""Independent signed-cue, funding, derivative and custody audit; no physics."""
import argparse
import json
import math
from pathlib import Path
import verify_ere_resource_learning as old

PROFILE='E2-resource-cue-contrast-preflight-v1'
ADAPTER='E2-resource-cue-contrast-adapter-v1'
CAMPAIGN='ee9f26d77bbfa63b9cc66ce0562cff33d636b99ebd4ec54290405f1e1eba69b8'
DIAGNOSTIC='36d512dbd096d6d7efb6cd1bff33c1e67c9b51200e6a7f601d6c2f15de406ec2'
PROBES=[[0.]*4,[.07,-.11,.13,-.17]]
MODES=['full','forecast_masked','bounty_masked','cues_masked']
EXTRA=['configs/ere_resource_contrast_adapter_v1.json','docs/ere_resource_contrast_adapter_v1.md',
       'crates/hg-train/src/ere/resource_contrast_adapter.rs','crates/hg-train/tests/ere_resource_contrast_adapter.rs',
       'scripts/run_ere_resource_contrast_adapter.py','scripts/verify_ere_resource_contrast_adapter.py',
       'scripts/test_verify_ere_resource_contrast_adapter.py','scripts/recheck_ere_resource_contrast_adapter.py']
EXPORT='pub mod resource_contrast_adapter;\n'

def features(o,mode):
    assert o['wallet'] in [0,1] and o['forecast'] in [0,1] and o['bounty'] in [2,12] and mode in MODES
    f=0 if mode in ['forecast_masked','cues_masked'] else 2*o['forecast']-1
    b=0 if mode in ['bounty_masked','cues_masked'] else (o['bounty']-7)/5
    return [1.,f,b,f*b]
def probabilities(t,x):
    assert len(t)==len(x)==4 and all(math.isfinite(v) and abs(v)<=8 for v in t)
    assert x[0]==1 and x[1] in [-1,0,1] and x[2] in [-1,0,1] and x[3]==x[1]*x[2]
    p=1/(1+math.exp(-sum(a*b for a,b in zip(t,x))));return [p,1-p]
def close(actual,expected):old.a.close(actual,expected,2e-8)

def custody(root):
    root=Path(root);m=old.a.load(root/'manifest.json')
    assert m['profile']==PROFILE
    maps={}
    for key,expected,count in [('campaign',CAMPAIGN,8968),('diagnostic',DIAGNOSTIC,838)]:
        path=root/f'inputs/{key}_seal.json';assert old.a.digest(path)==expected
        maps[key]=old.a.load(path);assert len(maps[key])==count
    wanted={f'inputs/cell-{i:04}.json':dict(archive='campaign',path=f'inputs/cell-{i:04}.json') for i in range(16)}
    wanted['inputs/allocation_audit.json']=dict(archive='campaign',path='inputs/allocation_audit.json')
    for key in maps:
        for kind in ['manifest','audit']:
            wanted[f'inputs/{key}_{kind}.json']=dict(archive=key,path=f'{kind}.json')
    assert m['imported_files']==wanted
    for local,origin in wanted.items():assert old.a.digest(root/local)==maps[origin['archive']][origin['path']]
    parent=old.a.load(root/'inputs/campaign_manifest.json');diag=old.a.load(root/'inputs/diagnostic_manifest.json')
    assert parent['status']=='complete' and parent['accounting_pass'] is True and parent['local_learning_pass'] is False
    assert diag['status']=='complete' and diag['diagnostic_pass'] is True
    assert old.a.load(root/'inputs/diagnostic_audit.json')['diagnostic_pass'] is True
    assert set(m['sources'])==set(parent['sources'])|set(EXTRA)
    for n,h in m['sources'].items():assert old.a.digest(root/'source_snapshots'/n)==h
    for n,h in parent['sources'].items():
        path=root/'source_snapshots'/n
        if n=='crates/hg-train/src/ere.rs':
            text=path.read_text();assert text.count(EXPORT)==1
            import hashlib
            assert hashlib.sha256(text.replace(EXPORT,'',1).encode()).hexdigest()==h
        else:assert old.a.digest(path)==h
    assert old.a.digest(root/'test_binary')==m['binary_sha256']
    config=old.a.load(root/'source_snapshots/configs/ere_resource_contrast_adapter_v1.json')
    assert config['profile']==PROFILE and config['campaign_root']==CAMPAIGN and config['diagnostic_root']==DIAGNOSTIC
    assert config['parameters']==4 and config['initial_parameters']==[0.]*4
    assert config['arithmetic_probes']==PROBES and config['modes']==MODES
    assert config['forecast_domain']==[0,1] and config['bounty_domain']==[2,12] and config['wallet_domain']==[0,1]
    assert config['batch_denominator']==8 and config['funded_baseline']==9.5 and config['reward_scale']==20
    assert config['finite_difference_epsilon']==1e-6 and config['derivative_tolerance']==2e-8
    assert config['output_limit_bytes']==134217728
    for item in [config,m]:
        for flag in ['native_advances','optimizer_updates','controller_fitting','launch_authorized','scientific_acceptance','training_admitted','sustained_claim','independent_physics_replay']:assert item[flag]==0
    return m

def native_states(root):
    history=old.a.load(root/'inputs/allocation_audit.json');result=[]
    for state in range(8):
        obs=None;utility=[]
        for action in range(2):
            r=old.a.load(root/f'inputs/cell-{state*2+action:04}.json');h=history['cells'][state*2+action]
            assert r['completed_steps']==32 and len(r['timeline'])==33
            assert [row['step'] for row in r['timeline']]==list(range(33))
            o=old.a.projection(r['allocation_observation']['inventory'])
            if obs is not None:assert obs==o
            obs=o
            fires=[row['step'] for row in r['timeline'][17:] for event in row['events'] if event['kind']=='world' and event['rcf']==h['measured']['identity']]
            assert fires==h['post_resolution_fires']
            payoffs=[row[1]['data'] for row in r['final_inventory']['nodes'] if row[1]['data']['label']==4293949190]
            assert len(payoffs)==1
            props=[v for k,v in payoffs[0]['properties'] if k==30471]
            assert len(props)==1 and set(props[0])=={'Int'} and type(props[0]['Int']) is int
            credits=props[0]['Int'];assert credits==r['summary']['harvested_credits']==h['harvested_credits']
            assert r['action']==h['action']==['defend','harvest'][action]
            utility.append(len(fires)+credits);assert utility[-1]==h['utility']
        assert obs['wallet']==1-state%2
        result.append((obs,utility))
    return result

def audit(root):
    root=Path(root);custody(root);data=old.a.load(root/'contrast_records.json');states=native_states(root)
    assert data['profile']==ADAPTER and data['classification']=='saved-input-contrast-arithmetic-only'
    for flag in ['native_advances','optimizer_updates','controller_fitting','scientific_acceptance','training_admitted','sustained_claim','independent_physics_replay']:assert data[flag]==0
    assert len(data['records'])==64 and len(data['batches'])==8
    derivatives=0;max_error=0.;unique={mode:set() for mode in MODES};credits_by_wallet=[0,0]
    for state,(o,u) in enumerate(states):
        for pi,t in enumerate(PROBES):
            for mi,mode in enumerate(MODES):
                r=data['records'][(state*2+pi)*4+mi];x=features(o,mode);p,q=probabilities(t,x)
                assert r['state']==state and r['probe']==pi and r['theta']==t and r['mode']==mode and r['observation']==o
                assert r['funded']==(o['wallet']==1) and r['utility']==u and r['boundary_sample']==1
                close(r['input'],x);close(r['probabilities'],[p,q]);unique[mode].add(tuple(x))
                close(r['expected_utility'],p*u[0]+q*u[1]);close(r['utility_gradient'],[v*p*q*(u[0]-u[1]) for v in x])
                assert len(r['scores'])==len(r['contributions'])==len(r['requests'])==2
                for action in [0,1]:
                    score=[v*((1 if action==0 else 0)-p) for v in x]
                    close(r['scores'][action],score)
                    expected=[v*(u[action]-9.5)/20 if o['wallet']==1 else 0. for v in score]
                    close(r['contributions'][action],expected);credits_by_wallet[int(o['wallet'])]+=1
                    for i in range(4):
                        hi=t.copy();lo=t.copy();hi[i]+=1e-6;lo[i]-=1e-6
                        fd=(math.log(probabilities(hi,x)[action])-math.log(probabilities(lo,x)[action]))/2e-6
                        error=abs(fd-score[i]);assert error<2e-8;max_error=max(max_error,error);derivatives+=1
                for i in range(4):
                    hi=t.copy();lo=t.copy();hi[i]+=1e-6;lo[i]-=1e-6
                    value=lambda theta:sum(a*b for a,b in zip(probabilities(theta,x),u))
                    fd=(value(hi)-value(lo))/2e-6
                    error=abs(fd-r['utility_gradient'][i]);assert error<2e-8;max_error=max(max_error,error);derivatives+=1
                for saved,draw in zip(r['requests'],[.25,.75]):
                    action=int(draw>=p);request=['defend','harvest'][action] if o['wallet']==1 else None
                    assert saved['feature_profile']==ADAPTER and saved['theta']==t and saved['mode']==mode and saved['observation']==o
                    assert saved['funded']==(o['wallet']==1) and saved['draw']==draw and saved['sampled_action']==action
                    close(saved['input'],x);close(saved['probabilities'],[p,q])
                    assert saved['decision']==dict(step=13,request=request)
    paid=states[::2];full=[features(o,'full') for o,u in paid]
    gram=[[sum(a*b for a,b in zip(x,y)) for y in full] for x in full]
    assert gram==[[4 if i==j else 0 for j in range(4)] for i in range(4)]
    assert {mode:len(values) for mode,values in unique.items()}==dict(full=4,forecast_masked=2,bounty_masked=2,cues_masked=1)
    full_signs=0;directions=[]
    for pi,t in enumerate(PROBES):
        for mi,mode in enumerate(MODES):
            batch=data['batches'][pi*4+mi];g=[0.]*4;mean=0.
            assert batch['probe']==pi and batch['theta']==t and batch['mode']==mode
            for o,u in states:
                x=features(o,mode);p,q=probabilities(t,x)
                if o['wallet']==1:
                    mean+=(p*u[0]+q*u[1])/4
                    for i in range(4):g[i]+=x[i]*p*q*(u[0]-u[1])/160
            close(batch['expected_batch_gradient'],g);close(batch['funded_expected_mean'],mean)
            assert len(batch['directions'])==4
            for state,(row,(o,u)) in enumerate(zip(batch['directions'],paid)):
                x=features(o,mode);p,q=probabilities(t,x);actual=sum(a*b for a,b in zip(x,g));pred=p*q*(u[0]-u[1])/40
                close(row['input'],x);close(row['actual'],actual);close(row['orthogonal_prediction'],pred)
                if mode=='full':
                    close(actual,pred);assert actual*(u[0]-u[1])>0;full_signs+=1
                    directions.append(dict(probe=pi,state=state*2,direction=actual))
    assert derivatives==768 and full_signs==8 and credits_by_wallet==[64,64]
    assert sum(p.stat().st_size for p in root.rglob('*') if p.is_file())<134217728
    return dict(profile=PROFILE,engineering_pass=True,states=8,source_receipts=16,arithmetic_records=64,
                score_records=128,actor_contribution_records=128,pure_requests=128,derivatives=derivatives,
                max_derivative_error=max_error,unique_actor_inputs={k:len(v) for k,v in unique.items()},
                full_funded_gram=gram,full_funded_rank=4,masked_funded_rank=1,
                full_logit_sign_checks=full_signs,full_expected_logit_directions=directions,
                actor_squared_norms=dict(full=4,forecast_masked=2,bounty_masked=2,cues_masked=1),
                native_advances=0,optimizer_updates=0,controller_fitting=False,launch_authorized=False,
                scientific_acceptance=False,training_admitted=False,sustained_claim=False,independent_physics_replay=False)

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('archive',type=Path);args=p.parse_args()
    print(json.dumps(audit(args.archive),indent=2))
