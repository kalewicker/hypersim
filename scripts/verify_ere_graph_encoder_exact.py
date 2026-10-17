#!/usr/bin/env python3
"""Independent forward-AD exact-gradient accounting; no new native execution."""
import argparse
import json
import math
from pathlib import Path
import verify_ere_graph_encoder_learning as prior

a=prior.a; e=prior.encoder; close=prior.close
PROFILE='E2-graph-encoder-exact-gradient-v1'
SOURCE='ere_graph_encoder_learning_trial_20261007'
SEAL='232178828b8de4fc2c33e30f6a3ccc7b25d489a5deeb7aad788367a1d2e9cd5b'
DESIGN=dict(profile=PROFILE,source_archive=SOURCE,source_root_seal=SEAL,source_entries=2920,matrix_policy_seed=prior.POLICY_SEEDS[0],
    baseline_cells=144,control_cells=18,development_contexts=prior.DEVELOPMENT,reserved_contexts=prior.RESERVED,
    actions_per_context=16,objective_denominator=6,modes=prior.MODES,parameters_per_policy=110,initialization_seed=e.SEED,
    initialization_scale=0.15,updates_per_mode=100,learning_rate=4.,checkpoints=[12,100],arithmetic_tolerance=2e-12,
    finite_difference_epsilon=1e-6,finite_difference_tolerance=2e-8,blind_ceiling=1/24,ceiling_margin=0.01,contrast_margin=0.01,
    reserved_gain_margin=0.01,maximum_native_attempts=0,adaptive_updates=False,training_admitted=False,scientific_acceptance=False,
    independent_physics_replay=False,sustained_claim=False)


def import_matrix(out):
    assert a.digest(out/'source_output_sha256.json')==SEAL
    seal=a.load(out/'source_output_sha256.json');assert len(seal)==2920
    ids=list(range(144))+list(range(576,594))
    assert a.files(out/'inputs')=={f'cell-{i:04}.json' for i in ids}
    plan=list(prior.schedule());worlds=[];controls=[];provenance=[];steps=0
    for i in ids:
        name=f'cell-{i:04}.json';path=out/'inputs'/name
        assert a.digest(path)==seal['native/'+name]
        r=a.load(path);seed,phase,mode,iteration,c,arm,actions=plan[i];n=r['native']
        assert set(r)=={'id','phase','policy_seed','context','actions','native_seed','decision','native'}
        assert type(r['id']) is int and r['id']==i and r['phase']==phase and r['policy_seed']==seed
        assert r['context']==c and r['actions']==actions and r['decision'] is None
        assert r['native_seed']==prior.native.SEEDS[(i//2)%2]==n['seed'] and n['context']==c and n['arm']==arm and n['actions']==actions
        audited,o=prior.audit_native(n);steps+=24
        if phase=='baseline':
            if i%16==0:worlds.append(dict(context=c,raw=n['initial_inventory'],rewards=[]))
            assert worlds[-1]['context']==c and worlds[-1]['raw']==n['initial_inventory']
            worlds[-1]['rewards'].append(float(audited['reward']))
        else:controls.append(dict(context=c,arm=arm,reward=float(audited['reward'])))
        provenance.append(dict(source='native/'+name,sha256=a.digest(path)))
    assert len(worlds)==9 and len(controls)==18
    return dict(worlds=worlds,controls=controls),dict(source_archive=SOURCE,source_root_seal=SEAL,receipts=provenance,imported_native_steps_checked=steps,new_native_attempts=0)


def development(data):
    assert [w['context'] for w in data['worlds']]==prior.native.CONTEXTS
    return [dict(context=w['context'],observation=e.observe(w['raw']),rewards=w['rewards']) for w in data['worlds'] if w['context'] in prior.DEVELOPMENT]


def objective(mode,theta,worlds):
    assert mode in prior.MODES and len(worlds)==6 and [w['context'] for w in worlds]==prior.DEVELOPMENT
    terms=[];grad=[[] for _ in range(110)]
    for w in worlds:
        rewards=w['rewards'];assert len(rewards)==16 and all(type(r) is float and math.isfinite(r) and 0<=r<=1 for r in rewards)
        inp=e.make_input(mode,w['observation']);z=e.logits(theta,inp);p=[]
        for row in z:
            maximum=max(t.v for t in row);total=math.fsum(math.exp(t.v-maximum) for t in row)
            p.append([math.exp(t.v-maximum)/total for t in row])
        # Forward AD of the complete expected objective; no Rust backward code.
        expectations=[[math.fsum(p[s][a]*z[s][a].g[j] for a in range(4)) for j in range(110)] for s in range(2)]
        for x in range(4):
            for y in range(4):
                weight=p[0][x]*p[1][y]*rewards[4*x+y]/6;terms.append(weight)
                for j in range(110):grad[j].append(weight*(z[0][x].g[j]-expectations[0][j]+z[1][y].g[j]-expectations[1][j]))
    return math.fsum(terms),[math.fsum(g) for g in grad]


def verify_transition(t,iteration,mode,theta,worlds):
    assert set(t)=={'iteration','mode','before','objective','gradient','after'}
    assert type(t['iteration']) is int and t['iteration']==iteration and t['mode']==mode
    close(t['before'],theta);value,g=objective(mode,theta,worlds)
    close(t['objective'],value);close(t['gradient'],g)
    after=[v+4*d for v,d in zip(theta,g)];close(t['after'],after)
    # Verify each recorded update, then carry the recorded Rust state forward.
    return t['after']


def comparison(checkpoint,graph,blind,data):
    rewards=[r for w in data['worlds'] for r in w['rewards']];obs=[e.observe(w['raw']) for w in data['worlds']]
    result=prior.comparison(prior.POLICY_SEEDS[0],0,graph,blind,rewards,rewards,obs,data['controls'])
    result.pop('policy_seed');result['checkpoint']=checkpoint
    result['diagnostic_criterion_pass']=all(result[k] for k in prior.KEYS)
    return result


def verify_checkpoints(records,states,data):
    assert len(records)==2 and set(states)=={12,100}
    result=[]
    for r,k in zip(records,[12,100]):
        expected=comparison(k,*states[k],data);close(r,expected);result.append(expected)
    return result


def audit(out):
    data,provenance=import_matrix(out);assert a.load(out/'input.json')==data and a.load(out/'input_provenance.json')==provenance
    worlds=development(data);records=a.load(out/'transitions.json');assert len(records)==200
    theta={mode:e.initialize() for mode in prior.MODES};states={}
    for iteration in range(100):
        for j,mode in enumerate(prior.MODES):theta[mode]=verify_transition(records[iteration*2+j],iteration,mode,theta[mode],worlds)
        if iteration+1 in [12,100]:states[iteration+1]=(theta['graph'].copy(),theta['blind'].copy())
    checkpoints=verify_checkpoints(a.load(out/'checkpoints.json'),states,data)
    expected=dict(profile=PROFILE,completed_updates=200,native_attempts=0,checkpoints=[dict(checkpoint=c['checkpoint'],diagnostic_criterion_pass=c['diagnostic_criterion_pass']) for c in checkpoints],
        training_admitted=False,scientific_acceptance=False,independent_physics_replay=False,sustained_claim=False)
    assert a.load(out/'optimizer_result.json')==expected
    m=a.load(out/'manifest.json');assert m['design']==DESIGN and m['sources']
    assert m['classification']=='offline_archive_reuse_optimizer_diagnostic' and m['maximum_native_attempts']==0
    for k in ('training_admitted','scientific_acceptance','independent_physics_replay','sustained_claim'):assert m[k] is False
    return dict(accounting_pass=True,profile=PROFILE,completed_updates=200,new_native_attempts=0,
        imported_receipts_checked=162,imported_native_steps_checked=3888,checkpoint_results=expected['checkpoints'],checkpoints=checkpoints,
        training_admitted=False,scientific_acceptance=False,independent_physics_replay=False,sustained_claim=False)


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('directory',type=Path)
    print(json.dumps(audit(p.parse_args().directory),indent=2))
