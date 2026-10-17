#!/usr/bin/env python3
"""Saved-weight arithmetic only: never calls a native binary or fits a controller."""
import argparse
import json
import math
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time
import verify_ere_resource_learning as old

PROFILE = 'E2-resource-gradient-diagnostic-v1'
PARENT = 'ee9f26d77bbfa63b9cc66ce0562cff33d636b99ebd4ec54290405f1e1eba69b8'
ROOT = Path(__file__).resolve().parents[1]
OWN = ['configs/ere_resource_gradient_diagnostic_v1.json',
       'docs/ere_resource_gradient_diagnostic_v1.md',
       'scripts/ere_resource_gradient_diagnostic.py',
       'scripts/verify_ere_resource_gradient_diagnostic.py',
       'scripts/test_ere_resource_gradient_diagnostic.py',
       'scripts/recheck_ere_resource_gradient_diagnostic.py']
HELPERS = ['scripts/verify_ere_resource_learning.py',
           'scripts/verify_ere_resource_timed_bridge.py',
           'scripts/verify_ere_resource_adapter.py']

def save(path, value):
    path.write_text(json.dumps(value, indent=2, sort_keys=True, allow_nan=False)+'\n')

def inputs(row, mode):
    o = row['observation']
    return [1., o['wallet'], o['forecast'] if mode == 'full' else 0.,
            o['bounty']/12. if mode == 'full' else 0.]

def read_states(root):
    states = []
    historical = old.a.load(root/'inputs/allocation_audit.json')
    for state in range(8):
        receipts = [old.a.load(root/f'inputs/cell-{state*2+a:04}.json') for a in range(2)]
        assert receipts[0]['allocation_observation'] == receipts[1]['allocation_observation']
        o = old.a.projection(receipts[0]['allocation_observation']['inventory'])
        utility = []
        for action, r in enumerate(receipts):
            h = historical['cells'][state*2+action]
            identity = h['measured']['identity']
            fires = sum(e['kind']=='world' and e['rcf']==identity
                        for step in r['timeline'][17:] for e in step['events'])
            payoff = [node[1]['data'] for node in r['final_inventory']['nodes']
                      if node[1]['data']['label']==4293949190]
            assert len(payoff)==1
            credit = [v['Int'] for k,v in payoff[0]['properties'] if k==30471]
            assert len(credit)==1 and credit[0]==r['summary']['harvested_credits']
            utility.append(fires+credit[0]); assert utility[-1]==h['utility']
        states.append(dict(state=state, scenario=receipts[0]['scenario'],
                           resource=receipts[0]['resource'], observation=o, utility=utility))
    return states

def expected(theta, mode, states):
    assert mode in old.MODES
    assert len(theta)==4 and all(math.isfinite(t) and abs(t)<=8 for t in theta)
    rows=[]; gradient=[0.]*4; variance=[0.]*4; utility=0.
    for state in states:
        x=inputs(state,mode); z=sum(t*v for t,v in zip(theta,x)); p=1/(1+math.exp(-z))
        ud,uh=state['utility']; funded=state['observation']['wallet']==1
        eu=p*ud+(1-p)*uh
        g=[p*(1-p)*(ud-uh)*v/160 if funded else 0. for v in x]
        # Conditional variance of the original estimator under independent
        # Bernoulli action draws, not uncertainty over the deterministic world.
        c0=[(ud-9.5)/160*(1-p)*v if funded else 0. for v in x]
        c1=[(uh-9.5)/160*(-p)*v if funded else 0. for v in x]
        var=[max(0.,p*a*a+(1-p)*b*b-c*c) for a,b,c in zip(c0,c1,g)]
        for i in range(4):gradient[i]+=g[i];variance[i]+=var[i]
        if funded:utility+=eu/4
        best=0 if ud>uh else 1 if uh>ud else None
        rows.append(dict(state=state['state'], input=x, probabilities=[p,1-p],
                         logit=z, expected_utility=eu, expected_gradient=g,
                         conditional_variance=var, best_action=best,
                         signed_optimal_margin=(z if best==0 else -z) if funded and best is not None else None))
    norm=math.hypot(*gradient);noise=math.sqrt(sum(variance))
    return dict(theta=theta,mode=mode,states=rows,funded_expected_mean=utility,
                expected_gradient=gradient,conditional_variance=variance,
                gradient_norm=norm,noise_rms=noise,signal_to_noise=norm/noise if noise else None)

def finite_difference(theta,mode,states):
    result=[]
    for i in range(4):
        hi=theta.copy();lo=theta.copy();hi[i]+=1e-5;lo[i]-=1e-5
        result.append((expected(hi,mode,states)['funded_expected_mean']-
                       expected(lo,mode,states)['funded_expected_mean'])/2e-5)
    return result

def calculate(root):
    states=read_states(root);schedule=old.a.load(root/'inputs/parent_schedule.json')
    assert schedule==old.schedule()
    theta=[[0.]*4 for _ in range(8)];records=[];sums=[]
    for model in range(8):
        sums.append(dict(model=model,seed=old.SEEDS[model//2],mode=old.MODES[model%2],
                         expected_sum=[0.]*4,sampled_sum=[0.]*4,residual_squared=0.,
                         positive_alignment_steps=0,objective_decrease_steps=0,
                         norm_clip_steps=0,parameter_clip_steps=0))
    for index in range(800):
        r=old.a.load(root/f'inputs/update-{index:04}.json');model=index//100;mode=old.MODES[model%2]
        assert r['index']==index and r['model']==model and r['update']==index%100+1
        assert len(r['samples'])==8
        old.a.close(r['arithmetic']['before'],theta[model])
        for state,sample in enumerate(r['samples']):
            x=inputs(states[state],mode);old.a.close(sample['input'],x)
            slot=schedule[index*8+state];assert sample['state']==state
            action=int(slot['draw']>=old.a.probability(theta[model],x)[0])
            assert sample['action']==action and sample['utility']==states[state]['utility'][action]
        actual=old.update(theta[model],r['samples']);old.compare_update(r['arithmetic'],actual)
        before=expected(theta[model],mode,states)
        after=expected(r['arithmetic']['after'],mode,states)
        sample=actual['mean_gradient'];eg=before['expected_gradient']
        residual=[a-b for a,b in zip(sample,eg)];dot=sum(a*b for a,b in zip(sample,eg))
        sn=math.hypot(*sample);en=before['gradient_norm']
        change=after['funded_expected_mean']-before['funded_expected_mean']
        row=dict(index=index,model=model,update=r['update'],expected=before,
                 sampled_gradient=sample,residual=residual,
                 cosine_alignment=dot/(sn*en) if sn and en else None,
                 exact_expected_utility_after_recorded_update=after['funded_expected_mean'],
                 exact_expected_utility_change=change)
        records.append(row);aggregate=sums[model]
        for i in range(4):aggregate['expected_sum'][i]+=eg[i];aggregate['sampled_sum'][i]+=sample[i]
        aggregate['residual_squared']+=sum(v*v for v in residual)
        aggregate['positive_alignment_steps']+=dot>0
        aggregate['objective_decrease_steps']+=change<0
        aggregate['norm_clip_steps']+=actual['norm']>1
        projected=[t+g for t,g in zip(theta[model],actual['clipped_gradient'])]
        aggregate['parameter_clip_steps']+=any(abs(v)>8 for v in projected)
        # Evaluate the authenticated recorded weights, not a slightly drifting
        # reconstruction of Rust floating-point arithmetic.
        theta[model]=r['arithmetic']['after']
    old.a.close(old.a.load(root/'inputs/endpoints.json'),theta)
    for model,aggregate in enumerate(sums):
        aggregate['residual_rms']=math.sqrt(aggregate.pop('residual_squared')/100)
        aggregate['endpoint']=expected(theta[model],old.MODES[model%2],states)
    vector=old.a.load(root/'source_snapshots/configs/ere_resource_gradient_diagnostic_v1.json')['authored_capacity_vector']
    probes=[expected([0.]*4,mode,states) for mode in old.MODES]
    probes += [a['endpoint'] for a in sums]+[expected(vector,'full',states)]
    for probe in probes:
        probe['funded_utility_finite_difference']=finite_difference(probe['theta'],probe['mode'],states)
    return dict(profile=PROFILE,classification='saved-receipt-arithmetic-and-authored-capacity-only',
                states=states,probes=probes,models=sums,recorded_updates=records,
                funded_utility_gradient_to_update_gradient_scale=40,
                masked_constant_utility_formula='11 - 3*p_defend',
                masked_best_constant_upper_bound=11.,native_advances=0,optimizer_updates=0,
                controller_fitting=False,scientific_acceptance=False,training_admitted=False,
                sustained_claim=False,independent_physics_replay=False)

def imports():
    result={f'inputs/cell-{i:04}.json':f'inputs/cell-{i:04}.json' for i in range(16)}
    result.update({f'inputs/update-{i:04}.json':f'campaign/updates/update-{i:04}.json' for i in range(800)})
    for dest,origin in [('parent_manifest','manifest'),('parent_audit','audit'),
                        ('parent_schedule','schedule'),('parent_draw_hashes','draw_hashes')]:
        result[f'inputs/{dest}.json']=f'{origin}.json'
    result.update({'inputs/endpoints.json':'campaign/endpoints.json',
                   'inputs/proposal.json':'source_snapshots/configs/ere_resource_learning_proposal_v1.json',
                   'inputs/allocation_audit.json':'inputs/allocation_audit.json'})
    result.update({f'source_snapshots/{n}':f'source_snapshots/{n}' for n in HELPERS})
    return result

def run(parent,out):
    started=time.time();assert not out.exists() and out.is_relative_to(ROOT/'output')
    old.a.verify_seal(parent,PARENT,8968);pm=old.a.load(parent/'manifest.json')
    assert pm['status']=='complete' and pm['accounting_pass'] is True and pm['local_learning_pass'] is False
    assert all(old.a.digest(ROOT/n)==h for n,h in pm['sources'].items()),'parent-pinned live source drift'
    with tempfile.TemporaryDirectory(prefix='ere-gradient-diagnostic-') as path:
        stage=Path(path);(stage/'inputs').mkdir()
        shutil.copyfile(parent/'output_sha256.json',stage/'inputs/parent_seal.json')
        copied=imports()
        for local,origin in copied.items():
            dest=stage/local;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(parent/origin,dest)
        pins={}
        for n in OWN:
            dest=stage/'source_snapshots'/n;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(ROOT/n,dest);pins[n]=old.a.digest(dest)
        for n in HELPERS:pins[n]=old.a.digest(stage/'source_snapshots'/n)
        m=dict(profile=PROFILE,status='running',parent_root=PARENT,imported_files=copied,sources=pins,
               native_advances=0,optimizer_updates=0,controller_fitting=False,launch_authorized=False,
               scientific_acceptance=False,training_admitted=False,sustained_claim=False,independent_physics_replay=False)
        save(stage/'manifest.json',m);save(stage/'diagnostic.json',calculate(stage))
        env=dict(os.environ,PYTHONPATH='',PYTHONDONTWRITEBYTECODE='1',ERE_RESOURCE_GRADIENT_DIAGNOSTIC=str(stage))
        scripts=stage/'source_snapshots/scripts'
        for name,command in [('verification',[sys.executable,'-B',str(scripts/'verify_ere_resource_gradient_diagnostic.py'),str(stage)]),
                             ('tests',[sys.executable,'-B',str(scripts/'test_ere_resource_gradient_diagnostic.py'),'-v'])]:
            r=subprocess.run(command,cwd=stage,env=env,capture_output=True,text=True,timeout=60)
            save(stage/f'{name}.json',dict(command=command,returncode=r.returncode,stdout=r.stdout,stderr=r.stderr))
            assert r.returncode==0,r.stdout+r.stderr
            if name=='verification':save(stage/'audit.json',json.loads(r.stdout))
        old.a.verify_seal(parent,PARENT,8968)
        assert all(old.a.digest(ROOT/n)==h for n,h in pm['sources'].items())
        assert all(old.a.digest(ROOT/n)==h for n,h in pins.items())
        m.update(status='complete',diagnostic_pass=True,parent_preserved=True,
                 elapsed_seconds=time.time()-started)
        save(stage/'manifest.json',m)
        assert sum(p.stat().st_size for p in stage.rglob('*') if p.is_file())<134217728-1048576
        out.parent.mkdir(parents=True,exist_ok=True);shutil.copytree(stage,out)
    save(out/'output_sha256.json',{p.relative_to(out).as_posix():old.a.digest(p) for p in sorted(out.rglob('*')) if p.is_file()})
    return dict(output=str(out),seal_sha256=old.a.digest(out/'output_sha256.json'),
                entries=old.a.verify_seal(out),**old.a.load(out/'audit.json'))

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--parent',type=Path,default=ROOT/'output/ere_resource_learning_campaign_20261008')
    parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args();print(json.dumps(run(args.parent.resolve(),args.output.resolve()),indent=2))
