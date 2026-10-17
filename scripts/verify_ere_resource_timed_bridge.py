#!/usr/bin/env python3
"""Independent live-choice/custody/archival-equivalence audit; no physics replay."""
import argparse
import copy
import json
from pathlib import Path
import verify_ere_resource_adapter as a

PROFILE='E2-resource-timed-bridge-engineering-v1'
NATIVE='E2-resource-timed-bridge-native-v1'
MODES=a.MODES;THETA=a.THETA
REFUSALS=['expired','callback_error','wrong_step','unfunded_request']
MESSAGES=['expired bridge deadline','registered callback error','decision must bind live step13','request/funding mismatch']
ADAPTER_ROOT='37d364eae4cdee1d3c6f6481b247d7df93762ccd16358e65cb53563bb903b157'
FAILED_ADAPTER_ROOT='a34244da71bc81d656e1979b55ac0fa75ee0ec6cd4689ab0d1135028aec38c38'
def flags(table,request):
    result=copy.deepcopy(table)
    for r in result:
        if r['name'].startswith('allocation_'):
            r['enabled']=request is not None and (r['name']=='allocation_defend')==(request=='defend')
    return result
def schedule():
    slots=[]
    for mi,mode in enumerate(MODES):
        for state in range(8):
            slots.append(dict(id=len(slots),mode=mode,state=state,scenario=['hazard_low','hazard_high','safe_low','safe_high'][state//2],resource=['paid','unpaid'][state%2],draw=[.25,.75,.25,.75][mi]))
    slots.extend([dict(id=len(slots)+i,refusal=name) for i,name in enumerate(REFUSALS)])
    return slots
def input_for(o,mode):
    return [1.,o['wallet'],0. if mode in ['forecast_masked','cues_masked'] else o['forecast'],0. if mode in ['bounty_masked','cues_masked'] else o['bounty']/12.]
def audit_records(root):
    root=Path(root);slots=schedule();assert a.load(root/'native/schedule.json')==slots
    status=a.load(root/'native/status.json');assert len(status)==36
    historical=a.load(root/'inputs/allocation_audit.json');assert historical['audit_pass'] and historical['diagnostic_pass']
    advances=seals=decisions=payments=credits=0;utilities=[];prefixes={};observations={};paid_requests=[]
    for slot in slots:
        i=slot['id'];saved=a.load(root/f'native/cell-{i:04}.json');assert saved['slot']==slot and status[i]['id']==i
        if i>=32:
            fail=saved['failure'];expected=0 if i==32 else 13
            assert fail==dict(message=MESSAGES[i-32],native_advances=expected,seal_applications=int(expected>0),decision_calls=int(expected>0))
            assert status[i]==dict(id=i,status='expected_refusal',failure=fail)
            advances+=expected;seals+=int(expected>0);decisions+=int(expected>0);continue
        assert status[i]==dict(id=i,status='complete',native_advances=32,seal_applications=1)
        r=saved['receipt'];p=saved['policy'];assert r['profile']==NATIVE and r['classification']=='live_decision_bridge_engineering_fixture'
        assert r['scenario']==slot['scenario'] and r['resource']==slot['resource'] and r['completed_steps']==r['horizon']==32
        assert (r['native_advances'],r['seal_applications'],r['decision_calls'])==(32,1,1)
        for k in ['scientific_acceptance','training_admitted','sustained_claim','independent_physics_replay']:assert r[k] is False
        o=a.projection(r['allocation_observation']['inventory']);x=input_for(o,slot['mode']);prob=a.probability(THETA,x)
        sampled=int(slot['draw']>=prob[0]);request=['defend','harvest'][sampled] if o['wallet']==1 else None
        decision=dict(step=13,request=request)
        assert p['theta']==THETA and p['mode']==slot['mode'] and p['observation']==o and p['draw']==slot['draw'] and p['sampled_action']==sampled and p['decision']==decision
        a.close(p['input'],x);a.close(p['probabilities'],prob)
        assert r['action']==request
        reference_id=slot['state']*2+sampled
        old=a.load(root/f'inputs/cell-{reference_id:04}.json');cell=historical['cells'][reference_id]
        assert cell['id']==reference_id and cell['scenario']==slot['scenario'] and cell['resource']==slot['resource'] and cell['action']==old['action']==['defend','harvest'][sampled]
        for key in ['allocation_observation','initial_inventory','final_inventory','initialization','root','wallet','payoff','threat','defense','token','context','actions','seed','initial_fuel','initial_wallet_units','initial_defense_fuel','ordinary_nodes','observation_nodes','target_literals','initial_all_candidates','hazard_schedule','maintenance','summary']:
            assert r[key]==old[key],key
        assert r['seeded_editor_rules']==flags(old['seeded_editor_rules'],None)
        assert r['seeded_editor_compiled']==flags(old['seeded_editor_compiled'],None)
        d=r['decision_record'];assert d['step']==13 and d['decision']==decision and d['callback_calls']==1 and d['graph_unchanged'] is True and d['wallet_revalidated']==o['wallet']
        before=flags(old['table_catalogue'][old['timeline'][14]['pre_step_table']],None)
        assert d['before_rules']==before and d['after_rules']==flags(before,request)
        assert d['chosen_raw']==flags(old['seeded_editor_rules'],request) and d['chosen_compiled']==flags(old['seeded_editor_compiled'],request)
        assert d['rng_before']==d['rng_after']==old['timeline'][14]['rng_before']
        rows=r['timeline'];assert len(rows)==33 and [row['step'] for row in rows]==list(range(33))
        indices=[]
        for step,(row,reference) in enumerate(zip(rows,old['timeline'])):
            for key in ['step','state','events','paid_edits','genotype','installed']:
                assert row[key]==reference[key],(i,step,key)
            if step==0:continue
            for key in ['changed_nodes','rng_before','rng_after','compile_boundary','compile_report']:
                assert row[key]==reference[key],(i,step,key)
            assert row['compile_boundary']==step-1 and row['rng_before']==row['rng_after']
            assert not row['usage']['truncated']
            index=row['pre_step_table'];assert type(index) is int and 0<=index<len(r['table_catalogue']);indices.append(index)
            expected_table=flags(old['table_catalogue'][reference['pre_step_table']],None if step<=13 else request)
            assert r['table_catalogue'][index]==expected_table,(i,step,'runtime_table')
        assert list(dict.fromkeys(indices))==list(range(len(r['table_catalogue'])))
        prefix=[{k:row[k] for k in ['step','state','events','genotype','installed','paid_edits']} for row in rows[:14]]
        state=slot['state']
        if state in prefixes:assert prefixes[state]==prefix and observations[state]==r['allocation_observation']
        else:prefixes[state]=prefix;observations[state]=r['allocation_observation']
        allocations=[(row['step'],e['kind']) for row in rows for e in row['events'] if e['kind'] in ['allocate_defend','allocate_harvest']]
        assert allocations==([] if request is None else [(14,'allocate_'+request)])
        fires=[row['step'] for row in rows if 17<=row['step']<=32 for e in row['events'] if e['kind']=='world' and e['rcf']==cell['measured']['identity']]
        final_payoff=next(row[1]['data'] for row in r['final_inventory']['nodes'] if row[1]['data']['label']==4293949190)
        voucher=next(v['Int'] for k,v in final_payoff['properties'] if k==30471)
        assert voucher==r['summary']['harvested_credits']==cell['harvested_credits'] and fires==cell['post_resolution_fires']
        utility=len(fires)+voucher;assert utility==cell['utility']
        advances+=32;seals+=1;decisions+=1;payments+=len(allocations);credits+=voucher
        utilities.append(dict(id=i,state=state,mode=slot['mode'],sampled_action=sampled,request=request,utility=utility))
        if request is not None:paid_requests.append(request)
    assert len(prefixes)==8 and advances==1063 and seals==decisions==35 and payments==16 and credits==56
    assert paid_requests.count('defend')==paid_requests.count('harvest')==8
    return dict(profile=PROFILE,engineering_pass=True,successful_cells=32,expected_refusals=4,retained_slots=36,
        native_advances=advances,seal_applications=seals,decision_calls=decisions,allocation_payments=payments,harvested_credits=credits,
        prechoice_equivalent_states=8,cells=utilities,optimizer_updates=0,scientific_acceptance=False,training_admitted=False,sustained_claim=False,independent_physics_replay=False)
def audit(root):
    root=Path(root);m=a.load(root/'manifest.json');design=a.load(root/'source_snapshots/configs/ere_resource_timed_bridge_v1.json')
    assert design['profile']==m['profile']==PROFILE and design['native_profile']==NATIVE
    assert design['successful_cells']==32 and design['mode_draws']==[.25,.75,.25,.75] and design['modes']==MODES and design['theta']==THETA
    assert design['refusal_routes']==REFUSALS and design['expected_refusal_advances']==[0,13,13,13]
    assert design['maximum_native_advances']==1063 and design['maximum_seal_applications']==35 and design['horizon']==32
    assert design['observation_step']==13 and design['allocation_step']==14 and design['deadline_seconds']==60
    roots={**a.ROOTS,'adapter':ADAPTER_ROOT,'failed_adapter':FAILED_ADAPTER_ROOT};counts={**a.COUNTS,'adapter':688,'failed_adapter':681}
    maps={}
    for name,h in roots.items():
        p=root/f'inputs/{name}_seal.json';assert a.digest(p)==h;maps[name]=a.load(p);assert len(maps[name])==counts[name]
    assert set(m['imported_files'])=={f'inputs/cell-{i:04}.json' for i in range(16)}|{'inputs/allocation_audit.json','inputs/adapter_audit.json','inputs/adapter_manifest.json'}
    for n,origin in m['imported_files'].items():assert a.digest(root/n)==maps[origin['archive']][origin['path']]
    assert a.load(root/'inputs/adapter_manifest.json')['status']=='complete' and a.load(root/'inputs/adapter_audit.json')['engineering_pass']
    for n,h in m['sources'].items():assert a.digest(root/'source_snapshots'/n)==h
    assert a.digest(root/'test_binary')==m['binary_sha256'] and m['native_work_unknown'] is False
    for flag in ['optimizer_updates','scientific_acceptance','training_admitted','sustained_claim','independent_physics_replay']:assert m[flag]==design[flag]==0
    result=audit_records(root)
    assert m['native_advances']==result['native_advances'] and m['seal_applications']==result['seal_applications']
    return result
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('archive',type=Path);p.add_argument('--seal',action='store_true');args=p.parse_args()
    if args.seal:a.verify_seal(args.archive)
    print(json.dumps(audit(args.archive),indent=2))
