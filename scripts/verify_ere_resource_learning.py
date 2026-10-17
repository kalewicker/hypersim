#!/usr/bin/env python3
"""Independent portable draws, actual credit, batch updates and full-denominator audit."""
import argparse
import gzip
import hashlib
import json
import math
from pathlib import Path
import struct
import verify_ere_resource_timed_bridge as prior

a=prior.a
SEEDS=[993748721101,993748721102,993748721103,993748721104]
MODES=['full','cues_masked'];PROFILE='E2-resource-on-policy-runner-v1';PREFLIGHT='E2-resource-on-policy-preflight-v1'
BRIDGE_ROOT='cc8161fbdb6b5475f5bb5eeef3865da6dcf6ae69e53a7231a84102925d0294ba'
FAILED_BRIDGE_ROOT='560ea51646703e3b3c10bd68e57b0e3637701796a1787767362b315316716e82'
ROOTS={**a.ROOTS,'adapter':prior.ADAPTER_ROOT,'failed_adapter':prior.FAILED_ADAPTER_ROOT,'bridge':BRIDGE_ROOT,'failed_bridge':FAILED_BRIDGE_ROOT}
COUNTS={**a.COUNTS,'adapter':688,'failed_adapter':681,'bridge':743,'failed_bridge':695}
MASK=(1<<64)-1
def draws(seed):
    state=seed
    for _ in range(800):
        state=(state+0x9e3779b97f4a7c15)&MASK;z=state
        z=((z^(z>>30))*0xbf58476d1ce4e5b9)&MASK;z=((z^(z>>27))*0x94d049bb133111eb)&MASK
        yield ((z^(z>>31))>>11)/(1<<53)
def draw_hashes():return {str(s):hashlib.sha256(b''.join(struct.pack('<d',d) for d in draws(s))).hexdigest() for s in SEEDS}
def schedule():
    result=[]
    def add(phase,model,seed,mode,update,state,draw,baseline=None):
        result.append(dict(id=len(result),phase=phase,model=model,seed=seed,mode=mode,update=update,state=state,draw=draw,baseline=baseline))
    for rep,seed in enumerate(SEEDS):
        for arm,mode in enumerate(MODES):
            stream=iter(draws(seed))
            for update in range(1,101):
                for state in range(8):add('training',rep*2+arm,seed,mode,update,state,next(stream))
    for rep,seed in enumerate(SEEDS):
        for arm,mode in enumerate(MODES):
            for state in range(8):
                for i in range(16):add('confirmation',rep*2+arm,seed,mode,None,state,(i+.5)/16)
    for state in range(8):
        for action in range(2):add('baseline',None,None,'cues_masked',None,state,[.25,.75][action],action)
    assert len(result)==7440
    return result
def update(theta,samples):
    assert len(theta)==4 and all(math.isfinite(t) and abs(t)<=8 for t in theta) and len(samples)==8
    contributions=[];gradient=[0.]*4
    for state,s in enumerate(samples):
        assert s['state']==state and s['action'] in [0,1] and 0<=s['utility']<=20 and math.isfinite(s['utility'])
        x=s['input'];assert len(x)==4 and x[0]==1 and x[1] in [0,1] and all(math.isfinite(v) and 0<=v<=1 for v in x)
        p=a.probability(theta,x)[0]
        credit=(s['utility']-9.5)/20 if x[1]==1 else 0
        g=[credit*v*((1 if s['action']==0 else 0)-p) for v in x];contributions.append(g)
        for i in range(4):gradient[i]+=g[i]/8
    norm=math.hypot(*gradient);scale=1/norm if norm>1 else 1
    clipped=[v*scale for v in gradient];after=[max(-8,min(8,t+g)) for t,g in zip(theta,clipped)]
    return dict(before=theta,after=after,contributions=contributions,mean_gradient=gradient,norm=norm,norm_scale=scale,clipped_gradient=clipped)
def compare_update(actual,expected):
    assert set(actual)==set(expected)
    for key,value in expected.items():a.close(actual[key],value)
def credit(receipt,identity):
    assert receipt['profile']==prior.NATIVE and receipt['completed_steps']==receipt['native_advances']==32 and receipt['seal_applications']==receipt['decision_calls']==1
    rows=receipt['timeline'];assert len(rows)==33 and [r['step'] for r in rows]==list(range(33))
    fires=sum(1 for r in rows[17:] for e in r['events'] if e['kind']=='world' and e['rcf']==identity)
    nodes=[r[1]['data'] for r in receipt['final_inventory']['nodes'] if r[1]['data']['label']==4293949190];assert len(nodes)==1
    values=[v for k,v in nodes[0]['properties'] if k==30471];assert len(values)==1 and set(values[0])=={'Int'}
    vouchers=values[0]['Int'];assert type(vouchers) is int and vouchers in [0,2,12] and vouchers==receipt['summary']['harvested_credits']
    assert 0<=fires+vouchers<=20
    return fires+vouchers
def load_gzip(path):
    with gzip.open(path,'rb') as stream:data=stream.read(16*1024*1024+1)
    assert len(data)<=16*1024*1024,'receipt decompression limit'
    return json.loads(data)
def references(root):
    return [a.load(root/f'inputs/cell-{i:04}.json') for i in range(16)],a.load(root/'inputs/allocation_audit.json')
def audit_cell(saved,slot,theta,refs,historical):
    assert saved['slot']==slot;policy=saved['policy'];r=saved['receipt']
    obs=a.projection(r['allocation_observation']['inventory']);x=prior.input_for(obs,slot['mode']);probs=a.probability(theta,x);action=int(slot['draw']>=probs[0]);request=['defend','harvest'][action] if obs['wallet']==1 else None
    assert policy['mode']==slot['mode'] and policy['draw']==slot['draw'] and policy['sampled_action']==action and policy['observation']==obs and policy['decision']==dict(step=13,request=request)
    a.close(policy['theta'],theta);a.close(policy['input'],x);a.close(policy['probabilities'],probs)
    index=slot['state']*2+action;old=refs[index];cell=historical['cells'][index]
    assert r['scenario']==old['scenario'] and r['resource']==old['resource'] and r['action']==request and r['classification']=='live_decision_bridge_engineering_fixture'
    for key in ['allocation_observation','initial_inventory','final_inventory','initialization','root','wallet','payoff','threat','defense','token','context','actions','seed','initial_fuel','initial_wallet_units','initial_defense_fuel','ordinary_nodes','observation_nodes','target_literals','initial_all_candidates','hazard_schedule','maintenance','summary']:
        assert r[key]==old[key],key
    for flag in ['scientific_acceptance','training_admitted','sustained_claim','independent_physics_replay']:assert r[flag] is False
    assert r['seeded_editor_rules']==prior.flags(old['seeded_editor_rules'],None) and r['seeded_editor_compiled']==prior.flags(old['seeded_editor_compiled'],None)
    d=r['decision_record'];before=prior.flags(old['table_catalogue'][old['timeline'][14]['pre_step_table']],None)
    assert d['step']==13 and d['decision']==policy['decision'] and d['callback_calls']==1 and d['graph_unchanged'] is True and d['wallet_revalidated']==obs['wallet']
    assert d['before_rules']==before and d['after_rules']==prior.flags(before,request)
    assert d['chosen_raw']==prior.flags(old['seeded_editor_rules'],request) and d['chosen_compiled']==prior.flags(old['seeded_editor_compiled'],request)
    assert d['rng_before']==d['rng_after']==old['timeline'][14]['rng_before']
    assert len(r['timeline'])==33;indices=[]
    for step,(row,reference) in enumerate(zip(r['timeline'],old['timeline'])):
        for key in ['step','state','events','paid_edits','genotype','installed']:assert row[key]==reference[key],(step,key)
        if step==0:continue
        for key in ['changed_nodes','rng_before','rng_after','compile_boundary','compile_report']:assert row[key]==reference[key],(step,key)
        assert not row['usage']['truncated'] and row['compile_boundary']==step-1
        idx=row['pre_step_table'];assert type(idx) is int and 0<=idx<len(r['table_catalogue']);indices.append(idx)
        assert r['table_catalogue'][idx]==prior.flags(old['table_catalogue'][reference['pre_step_table']],None if step<=13 else request)
    assert list(dict.fromkeys(indices))==list(range(len(r['table_catalogue'])))
    paid=[(row['step'],e['kind']) for row in r['timeline'] for e in row['events'] if e['kind'] in ['allocate_defend','allocate_harvest']]
    assert paid==([] if request is None else [(14,'allocate_'+request)])
    utility=credit(r,cell['measured']['identity']);assert utility==cell['utility']==saved['actual_utility']
    return dict(state=slot['state'],input=x,action=action,utility=utility)
def custody(root):
    root=Path(root);m=a.load(root/'manifest.json');maps={}
    expected={f'inputs/cell-{i:04}.json' for i in range(16)}|{f'inputs/state-{i}.json' for i in range(8)}|{'inputs/allocation_audit.json'}
    assert set(m['imported_files'])==expected
    for key,digest in ROOTS.items():
        p=root/f'inputs/{key}_seal.json';assert a.digest(p)==digest;maps[key]=a.load(p);assert len(maps[key])==COUNTS[key]
    for n,origin in m['imported_files'].items():assert a.digest(root/n)==maps[origin['archive']][origin['path']]
    for i in range(8):assert m['imported_files'][f'inputs/state-{i}.json']==dict(archive='bridge',path=f'native/cell-{i:04}.json')
    for i in range(16):assert m['imported_files'][f'inputs/cell-{i:04}.json']==dict(archive='primary',path=f'native/cell-{i:04}.json')
    assert m['imported_files']['inputs/allocation_audit.json']==dict(archive='completion_v4',path='audit.json')
    for n,h in m['sources'].items():assert a.digest(root/'source_snapshots'/n)==h
    for n,h in m['binaries'].items():assert a.digest(root/n)==h
    proposal=a.load(root/'source_snapshots/configs/ere_resource_learning_proposal_v1.json')
    assert proposal['seeds']==SEEDS and proposal['arms']==MODES and proposal['updates']==100 and proposal['states_per_update']==8
    assert proposal['initial_parameters']==[0.]*4 and proposal['learning_rate']==1 and proposal['reward_scale']==20 and proposal['funded_baseline']==9.5 and proposal['batch_gradient_denominator']==8
    assert proposal['gradient_norm_limit']==1 and proposal['parameter_absolute_limit']==8 and proposal['confirmation_draws_per_state']==16 and proposal['constant_baseline_cells']==16
    assert proposal['maximum_native_advances']==238080 and proposal['maximum_seal_applications']==7440 and proposal['maximum_optimizer_updates']==800 and proposal['launch_authorized'] is False
    assert proposal['learning_criteria']==dict(mean_full_funded_confirmation_utility_at_least=11.5,mean_full_minus_masked_confirmation_utility_strictly_positive=True,minimum_replicates_beating_always_harvest_and_paired_masked=3,replicate_denominator=4)
    assert a.load(root/'schedule.json')==schedule() and a.load(root/'draw_hashes.json')==draw_hashes()
    contract=a.load(root/'source_snapshots/configs/ere_resource_learning_runner_v1.json')
    assert contract['native_wall_seconds']==contract['audit_wall_seconds']==7200 and contract['campaign_output_limit_bytes']==1073741824 and contract['compressed_cell_reserve_bytes']==2097152 and contract['compressed_cell_limit_bytes']==1048576
    return m
def audit_preflight(root):
    root=Path(root);m=custody(root);assert m['profile']==PREFLIGHT and m['native_advances']==m['optimizer_updates']==0 and m['launch_authorized'] is False
    data=a.load(root/'arithmetic.json');assert data['classification']=='saved-receipt-arithmetic-only' and data['native_advances']==data['optimizer_updates']==0
    refs,historical=references(root);samples=[];raw=compressed=0
    for state in range(8):
        saved=a.load(root/f'inputs/state-{state}.json');actual=load_gzip(root/f'compressed-state-{state}.json.gz');assert actual==saved
        r=saved['receipt'];p=saved['policy'];utility=credit(r,historical['cells'][state*2]['measured']['identity'])
        assert r==a.load(root/f'inputs/state-{state}.json')['receipt']
        samples.append(dict(state=state,input=p['input'],action=p['sampled_action'],utility=utility))
        meta=data['cells'][state];assert meta['state']==state and meta['utility']==utility and meta['raw_bytes']==(root/f'inputs/state-{state}.json').stat().st_size and meta['compressed_bytes']==(root/f'compressed-state-{state}.json.gz').stat().st_size
        raw+=meta['raw_bytes'];compressed+=meta['compressed_bytes']
    assert data['samples']==samples;compare_update(data['update'],update(a.THETA,samples))
    a.close(data['clip_probe']['norm'],5.);a.close(data['clip_probe']['scale'],.2);a.close(data['clip_probe']['gradient'],[.6,.8,0,0]);a.close(data['project_probe'],[8,-8,3,0])
    return dict(profile=PREFLIGHT,engineering_pass=True,schedule_slots=7440,paired_draws_per_seed=800,seeds=4,saved_credit_cells=8,raw_fixture_bytes=raw,compressed_fixture_bytes=compressed,compressed_ratio=compressed/raw,projected_cell_storage_bytes=math.ceil(compressed/8*7440),native_advances=0,optimizer_updates=0,launch_authorized=False,scientific_acceptance=False,training_admitted=False,sustained_claim=False,independent_physics_replay=False)
def audit_campaign(root):
    root=Path(root);outer=custody(root);assert outer['launch_authorized'] is True
    out=root/'campaign';m=a.load(out/'manifest.json');assert m['status']=='complete' and m['native_work_unknown'] is False
    assert (m['completed_cells'],m['native_advances'],m['seal_applications'],m['optimizer_updates'])==(7440,238080,7440,800)
    refs,historical=references(root);theta=[[0.]*4 for _ in range(8)];samples=[];updates=0;means=[[] for _ in range(8)];baseline=[[] for _ in range(2)];compressed=0
    with (out/'ledger.jsonl').open() as stream:
        events=iter(stream)
        for slot in schedule():
            parameters=theta[slot['model']] if slot['model'] is not None else [0.]*4
            start=json.loads(next(events));assert start['kind']=='start' and start['id']==slot['id'];a.close(start['theta'],parameters)
            path=out/f'cells/cell-{slot["id"]:04}.json.gz';sample=audit_cell(load_gzip(path),slot,parameters,refs,historical)
            complete=json.loads(next(events));assert complete==dict(kind='complete',id=slot['id'],actual_utility=sample['utility'],compressed_bytes=path.stat().st_size)
            compressed+=path.stat().st_size;assert path.stat().st_size<=1048576
            if slot['phase']=='training':
                samples.append(sample)
                if slot['state']==7:
                    record=a.load(out/f'updates/update-{updates:04}.json');expected=update(parameters,samples)
                    assert record['index']==updates and record['model']==slot['model'] and record['update']==slot['update'] and len(record['samples'])==8
                    for actual,s in zip(record['samples'],samples):
                        assert actual['state']==s['state'] and actual['action']==s['action'] and actual['utility']==s['utility'];a.close(actual['input'],s['input'])
                    compare_update(record['arithmetic'],expected);theta[slot['model']]=expected['after'];samples=[]
                    event=json.loads(next(events));assert event['kind']=='update' and event['index']==updates and event['model']==slot['model'];a.close(event['after'],expected['after']);updates+=1
            elif slot['state']%2==0:
                (means[slot['model']] if slot['phase']=='confirmation' else baseline[slot['baseline']]).append(sample['utility'])
        assert next(events,None) is None,'extra/retried ledger events'
    assert updates==800 and not samples and all(len(v)==64 for v in means) and all(len(v)==4 for v in baseline)
    assert len(list((out/'cells').glob('*.json.gz')))==7440 and len(list((out/'updates').glob('*.json')))==800
    assert compressed==m['compressed_receipt_bytes'];a.close(a.load(out/'endpoints.json'),theta)
    per_model=[sum(v)/64 for v in means];base=[sum(v)/4 for v in baseline];assert base==[8.,11.]
    replicates=[dict(seed=seed,full=per_model[i*2],masked=per_model[i*2+1],paired_gap=per_model[i*2]-per_model[i*2+1]) for i,seed in enumerate(SEEDS)]
    full=sum(r['full'] for r in replicates)/4;masked=sum(r['masked'] for r in replicates)/4
    winners=sum(r['full']>11 and r['paired_gap']>0 for r in replicates)
    criteria=dict(mean_full_at_least_11_5=full>=11.5,mean_paired_gap_positive=full>masked,at_least_3_of_4_beat_both=winners>=3)
    for flag in ['scientific_acceptance','training_admitted','sustained_claim','independent_physics_replay']:assert m[flag] is False
    size=sum(p.stat().st_size for p in root.rglob('*') if p.is_file());assert size<=1073741824
    return dict(profile=PROFILE,accounting_pass=True,local_learning_pass=all(criteria.values()),criteria=criteria,replicates=replicates,full_mean=full,masked_mean=masked,paired_mean_gap=full-masked,always_defend_mean=8.,always_harvest_mean=11.,native_advances=238080,seal_applications=7440,optimizer_updates=800,scientific_acceptance=False,training_admitted=False,sustained_claim=False,independent_physics_replay=False)
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('archive',type=Path);p.add_argument('--campaign',action='store_true');p.add_argument('--seal',action='store_true');args=p.parse_args()
    if args.seal:a.verify_seal(args.archive)
    print(json.dumps((audit_campaign if args.campaign else audit_preflight)(args.archive),indent=2))
