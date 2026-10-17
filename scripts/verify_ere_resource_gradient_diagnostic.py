#!/usr/bin/env python3
"""Independent two-action enumeration and saved-update audit; no producer math."""
import argparse
import json
import math
from pathlib import Path
import verify_ere_resource_learning as old

PARENT='ee9f26d77bbfa63b9cc66ce0562cff33d636b99ebd4ec54290405f1e1eba69b8'
PROFILE='E2-resource-gradient-diagnostic-v1'

def near(actual,expected):
    old.a.close(actual,expected,2e-8)

def features(state,mode):
    o=state['observation']
    assert mode in ['full','cues_masked']
    return [1.,o['wallet'],o['forecast'] if mode=='full' else 0.,o['bounty']/12 if mode=='full' else 0.]

def prob(theta,x):
    assert len(theta)==len(x)==4 and all(math.isfinite(v) for v in theta+x)
    z=math.fsum(t*v for t,v in zip(theta,x))
    return z,1/(1+math.exp(-z))

def objective(theta,mode,states):
    total=0.
    for s in states:
        if s['observation']['wallet']==1:
            _,p=prob(theta,features(s,mode));u=s['utility'];total+=(p*u[0]+(1-p)*u[1])/4
    return total

def analytic(theta,mode,states,baseline=9.5):
    result=[];gradient=[0.]*4;variance=[0.]*4
    for state in states:
        x=features(state,mode);z,p=prob(theta,x);funded=state['observation']['wallet']==1
        u=state['utility'];contrib=[]
        for action in [0,1]:
            score=[v*((1 if action==0 else 0)-p) for v in x]
            contrib.append([v*(u[action]-baseline)/160 if funded else 0. for v in score])
        g=[p*contrib[0][i]+(1-p)*contrib[1][i] for i in range(4)]
        var=[max(0.,p*(contrib[0][i]-g[i])**2+(1-p)*(contrib[1][i]-g[i])**2) for i in range(4)]
        for i in range(4):gradient[i]+=g[i];variance[i]+=var[i]
        best=0 if u[0]>u[1] else 1 if u[1]>u[0] else None
        result.append(dict(state=state['state'],input=x,probabilities=[p,1-p],logit=z,
                           expected_utility=p*u[0]+(1-p)*u[1],expected_gradient=g,
                           conditional_variance=var,best_action=best,
                           signed_optimal_margin=(z if best==0 else -z) if funded and best is not None else None))
    noise=math.sqrt(sum(variance));norm=math.hypot(*gradient)
    return dict(theta=theta,mode=mode,states=result,funded_expected_mean=objective(theta,mode,states),
                expected_gradient=gradient,conditional_variance=variance,gradient_norm=norm,
                noise_rms=noise,signal_to_noise=norm/noise if noise else None)

def verify_expected(saved,theta,mode,states):
    expected=analytic(theta,mode,states)
    assert set(saved)==set(expected) or set(saved)==set(expected)|{'funded_utility_finite_difference'}
    near(saved['theta'],theta)
    assert saved['mode']==mode and len(saved['states'])==8
    for actual,row in zip(saved['states'],expected['states']):
        assert set(actual)==set(row) and actual['state']==row['state'] and actual['best_action']==row['best_action']
        for k in ['input','probabilities','logit','expected_utility','expected_gradient','conditional_variance']:near(actual[k],row[k])
        if row['signed_optimal_margin'] is None:assert actual['signed_optimal_margin'] is None
        else:near(actual['signed_optimal_margin'],row['signed_optimal_margin'])
    for k in ['funded_expected_mean','expected_gradient','conditional_variance','gradient_norm','noise_rms','signal_to_noise']:near(saved[k],expected[k])
    return expected

def rank(rows):
    rows=[r.copy() for r in rows];pivot=0
    for col in range(4):
        index=next((i for i in range(pivot,len(rows)) if abs(rows[i][col])>1e-10),None)
        if index is None:continue
        rows[pivot],rows[index]=rows[index],rows[pivot];v=rows[pivot][col]
        rows[pivot]=[x/v for x in rows[pivot]]
        for i in range(len(rows)):
            if i!=pivot:
                factor=rows[i][col];rows[i]=[a-factor*b for a,b in zip(rows[i],rows[pivot])]
        pivot+=1
    return pivot

def reconstruct_states(root):
    historical=old.a.load(root/'inputs/allocation_audit.json');states=[]
    for state in range(8):
        utilities=[];observation=None;meta=None
        for action in [0,1]:
            r=old.a.load(root/f'inputs/cell-{2*state+action:04}.json');h=historical['cells'][2*state+action]
            assert r['completed_steps']==r['horizon']==32 and len(r['timeline'])==33
            assert [row['step'] for row in r['timeline']]==list(range(33))
            assert r['action']==h['action']==['defend','harvest'][action]
            o=old.a.projection(r['allocation_observation']['inventory'])
            if observation is not None:assert observation==o and meta==(r['scenario'],r['resource'])
            observation=o;meta=(r['scenario'],r['resource'])
            fires=[row['step'] for row in r['timeline'] if 17<=row['step']<=32
                   for event in row['events'] if event['kind']=='world' and event['rcf']==h['measured']['identity']]
            assert fires==h['post_resolution_fires']
            nodes=[row[1]['data'] for row in r['final_inventory']['nodes'] if row[1]['data']['label']==4293949190]
            assert len(nodes)==1
            props=[v for k,v in nodes[0]['properties'] if k==30471]
            assert len(props)==1 and set(props[0])=={'Int'} and type(props[0]['Int']) is int
            credits=props[0]['Int'];assert credits==r['summary']['harvested_credits']==h['harvested_credits']
            utility=len(fires)+credits;assert utility==h['utility'] and 0<=utility<=20
            utilities.append(utility)
        assert observation['wallet']==1-state%2
        states.append(dict(state=state,scenario=meta[0],resource=meta[1],observation=observation,utility=utilities))
    return states

def custody(root):
    m=old.a.load(root/'manifest.json');assert m['profile']==PROFILE and m['parent_root']==PARENT
    assert old.a.digest(root/'inputs/parent_seal.json')==PARENT
    parent=old.a.load(root/'inputs/parent_seal.json');assert len(parent)==8968
    wanted={f'inputs/cell-{i:04}.json':f'inputs/cell-{i:04}.json' for i in range(16)}
    wanted.update({f'inputs/update-{i:04}.json':f'campaign/updates/update-{i:04}.json' for i in range(800)})
    wanted.update({'inputs/parent_manifest.json':'manifest.json','inputs/parent_audit.json':'audit.json',
                   'inputs/parent_schedule.json':'schedule.json','inputs/parent_draw_hashes.json':'draw_hashes.json',
                   'inputs/endpoints.json':'campaign/endpoints.json',
                   'inputs/proposal.json':'source_snapshots/configs/ere_resource_learning_proposal_v1.json',
                   'inputs/allocation_audit.json':'inputs/allocation_audit.json'})
    helpers=['scripts/verify_ere_resource_learning.py','scripts/verify_ere_resource_timed_bridge.py','scripts/verify_ere_resource_adapter.py']
    wanted.update({f'source_snapshots/{n}':f'source_snapshots/{n}' for n in helpers})
    assert m['imported_files']==wanted
    for local,origin in wanted.items():assert old.a.digest(root/local)==parent[origin]
    own=['configs/ere_resource_gradient_diagnostic_v1.json','docs/ere_resource_gradient_diagnostic_v1.md',
         'scripts/ere_resource_gradient_diagnostic.py','scripts/verify_ere_resource_gradient_diagnostic.py',
         'scripts/test_ere_resource_gradient_diagnostic.py','scripts/recheck_ere_resource_gradient_diagnostic.py']
    assert set(m['sources'])==set(own+helpers)
    for n,h in m['sources'].items():assert old.a.digest(root/'source_snapshots'/n)==h
    pm=old.a.load(root/'inputs/parent_manifest.json');pa=old.a.load(root/'inputs/parent_audit.json')
    assert pm['status']=='complete' and pm['accounting_pass'] is True and pm['local_learning_pass'] is False
    assert pa['accounting_pass'] is True and pa['local_learning_pass'] is False
    assert (pm['native_advances'],pm['optimizer_updates'])==(238080,800)
    assert old.a.load(root/'inputs/parent_schedule.json')==old.schedule()
    assert old.a.load(root/'inputs/parent_draw_hashes.json')==old.draw_hashes()
    proposal=old.a.load(root/'inputs/proposal.json')
    assert proposal['funded_baseline']==9.5 and proposal['reward_scale']==20 and proposal['batch_gradient_denominator']==8
    assert proposal['learning_rate']==proposal['gradient_norm_limit']==1 and proposal['parameter_absolute_limit']==8
    config=old.a.load(root/'source_snapshots/configs/ere_resource_gradient_diagnostic_v1.json')
    assert config['parent_root']==PARENT and config['parent_entries']==8968
    assert config['states']==8 and config['actions']==2 and config['models']==8 and config['recorded_updates']==800
    assert config['authored_capacity_vector']==[-1.,-1.,4.,-4.]
    assert config['finite_difference_epsilon']==1e-5 and config['derivative_tolerance']==2e-8
    for record in [m,config]:
        for flag in ['native_advances','optimizer_updates','controller_fitting','launch_authorized','scientific_acceptance','training_admitted','sustained_claim','independent_physics_replay']:assert record[flag]==0
    return m

def audit(root):
    root=Path(root);custody(root);data=old.a.load(root/'diagnostic.json');states=reconstruct_states(root)
    assert data['profile']==PROFILE and data['classification']=='saved-receipt-arithmetic-and-authored-capacity-only'
    assert data['states']==states and len(data['probes'])==11 and len(data['models'])==8 and len(data['recorded_updates'])==800
    for flag in ['native_advances','optimizer_updates','controller_fitting','scientific_acceptance','training_admitted','sustained_claim','independent_physics_replay']:assert data[flag]==0
    assert data['funded_utility_gradient_to_update_gradient_scale']==40
    assert data['masked_constant_utility_formula']=='11 - 3*p_defend' and data['masked_best_constant_upper_bound']==11
    endpoints=old.a.load(root/'inputs/endpoints.json');probes=data['probes'];max_error=0.;checks=0
    design=[([0.]*4,mode) for mode in old.MODES]+[(t,old.MODES[i%2]) for i,t in enumerate(endpoints)]+[([-1.,-1.,4.,-4.],'full')]
    for probe,(theta,mode) in zip(probes,design):
        assert len(theta)==4 and all(abs(t)<=8 for t in theta)
        expected=verify_expected(probe,theta,mode,states)
        for i in range(4):
            hi=theta.copy();lo=theta.copy();hi[i]+=1e-5;lo[i]-=1e-5
            fd=(objective(hi,mode,states)-objective(lo,mode,states))/2e-5
            near(probe['funded_utility_finite_difference'][i],fd)
            error=abs(fd-40*expected['expected_gradient'][i]);assert error<2e-8
            max_error=max(max_error,error);checks+=1
        near(expected['expected_gradient'],analytic(theta,mode,states,baseline=0.)['expected_gradient'])
    schedule=old.a.load(root/'inputs/parent_schedule.json');chains=[[0.]*4 for _ in range(8)]
    totals=[dict(expected_sum=[0.]*4,sampled_sum=[0.]*4,residual_squared=0.,positive_alignment_steps=0,
                 objective_decrease_steps=0,norm_clip_steps=0,parameter_clip_steps=0) for _ in range(8)]
    for index,row in enumerate(data['recorded_updates']):
        record=old.a.load(root/f'inputs/update-{index:04}.json');model=index//100;mode=old.MODES[model%2];theta=chains[model]
        assert row['index']==record['index']==index and row['model']==record['model']==model and row['update']==record['update']==index%100+1
        assert len(record['samples'])==8
        contributions=[];gradient=[0.]*4
        for state,sample in enumerate(record['samples']):
            x=features(states[state],mode);_,p=prob(theta,x);near(sample['input'],x)
            action=int(schedule[index*8+state]['draw']>=p)
            assert sample['state']==state and sample['action']==action and sample['utility']==states[state]['utility'][action]
            c=[(sample['utility']-9.5)/20*v*((1 if action==0 else 0)-p) if x[1]==1 else 0. for v in x]
            contributions.append(c)
            for i in range(4):gradient[i]+=c[i]/8
        norm=math.hypot(*gradient);scale=min(1.,1/norm) if norm else 1.
        clipped=[g*scale for g in gradient];raw_after=[t+g for t,g in zip(theta,clipped)]
        after=[min(8,max(-8,v)) for v in raw_after]
        expected_arithmetic=dict(before=theta,after=after,contributions=contributions,
                                 mean_gradient=gradient,norm=norm,norm_scale=scale,clipped_gradient=clipped)
        assert set(record['arithmetic'])==set(expected_arithmetic)
        for key,value in expected_arithmetic.items():near(record['arithmetic'][key],value)
        expected=verify_expected(row['expected'],theta,mode,states);eg=expected['expected_gradient']
        residual=[a-b for a,b in zip(gradient,eg)];near(row['sampled_gradient'],gradient);near(row['residual'],residual)
        dot=math.fsum(a*b for a,b in zip(gradient,eg));denom=math.hypot(*gradient)*math.hypot(*eg)
        if denom:near(row['cosine_alignment'],dot/denom)
        else:assert row['cosine_alignment'] is None
        utility_after=objective(record['arithmetic']['after'],mode,states)
        change=utility_after-expected['funded_expected_mean']
        near(row['exact_expected_utility_after_recorded_update'],utility_after);near(row['exact_expected_utility_change'],change)
        t=totals[model]
        for i in range(4):t['expected_sum'][i]+=eg[i];t['sampled_sum'][i]+=gradient[i]
        t['residual_squared']+=sum(v*v for v in residual)
        t['positive_alignment_steps']+=dot>0;t['objective_decrease_steps']+=change<0
        t['norm_clip_steps']+=norm>1;t['parameter_clip_steps']+=any(abs(v)>8 for v in raw_after)
        chains[model]=record['arithmetic']['after']
    near(endpoints,chains);models=[]
    for i,(saved,t) in enumerate(zip(data['models'],totals)):
        assert saved['model']==i and saved['seed']==old.SEEDS[i//2] and saved['mode']==old.MODES[i%2]
        near(saved['expected_sum'],t['expected_sum']);near(saved['sampled_sum'],t['sampled_sum'])
        near(saved['residual_rms'],math.sqrt(t['residual_squared']/100))
        for k in ['positive_alignment_steps','objective_decrease_steps','norm_clip_steps','parameter_clip_steps']:assert saved[k]==t[k]
        assert saved['endpoint']==probes[i+2]
        if not t['norm_clip_steps'] and not t['parameter_clip_steps']:
            near(saved['sampled_sum'],endpoints[i])
        models.append(dict(model=i,seed=saved['seed'],mode=saved['mode'],
                           exact_funded_mean=saved['endpoint']['funded_expected_mean'],
                           hazard_low_defend_probability=saved['endpoint']['states'][0]['probabilities'][0],
                           hazard_low_signed_margin=saved['endpoint']['states'][0]['signed_optimal_margin'],
                           endpoint_gradient_norm=saved['endpoint']['gradient_norm'],
                           endpoint_signal_to_noise=saved['endpoint']['signal_to_noise'],
                           expected_sum=saved['expected_sum'],sampled_sum=saved['sampled_sum'],
                           residual_rms=saved['residual_rms'],positive_alignment_steps=saved['positive_alignment_steps'],
                           objective_decrease_steps=saved['objective_decrease_steps'],
                           norm_clip_steps=saved['norm_clip_steps'],parameter_clip_steps=saved['parameter_clip_steps']))
    capacity=probes[-1];margins=[r['signed_optimal_margin'] for r in capacity['states'] if r['input'][1]==1]
    assert all(v>0 for v in margins)
    paid=[s for s in states if s['observation']['wallet']==1]
    ranks={mode:rank([features(s,mode) for s in paid]) for mode in old.MODES}
    assert ranks==dict(full=3,cues_masked=1)
    best_constant=max(sum(s['utility'][action] for s in paid)/4 for action in [0,1]);assert best_constant==11
    assert len({tuple(features(s,'cues_masked')) for s in paid})==1
    assert sum(p.stat().st_size for p in root.rglob('*') if p.is_file())<134217728
    return dict(profile=PROFILE,diagnostic_pass=True,parent_root=PARENT,states=8,source_receipts=16,
                recorded_updates_checked=800,sampled_actions_checked=6400,models=models,
                derivative_checks=checks,max_derivative_error=max_error,feature_ranks=ranks,
                authored_capacity_funded_mean=capacity['funded_expected_mean'],
                authored_capacity_minimum_signed_margin=min(margins),masked_expected_utility_upper_bound=best_constant,
                exact_full_endpoint_mean=sum(m['exact_funded_mean'] for m in models[::2])/4,
                exact_masked_endpoint_mean=sum(m['exact_funded_mean'] for m in models[1::2])/4,
                native_advances=0,optimizer_updates=0,controller_fitting=False,
                scientific_acceptance=False,training_admitted=False,sustained_claim=False,independent_physics_replay=False)

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('archive',type=Path);args=p.parse_args()
    print(json.dumps(audit(args.archive),indent=2,allow_nan=False))
