#!/usr/bin/env python3
"""Independent native-ledger, sampling, update, denominator and endpoint accounting."""
import argparse
import math
from pathlib import Path
import verify_ere_rule_choice as native

a=native.integrity
PROFILE='E2-rule-choice-learning-v2'
POLICY_SEEDS=[993748715001,993748715002,993748715003]
UPDATES=12
MASK=(1<<64)-1
DESIGN=dict(profile=PROFILE,policy_seeds=POLICY_SEEDS,native_seeds=native.SEEDS,
    contexts=native.CONTEXTS,actions=native.ACTIONS,updates_per_policy=12,samples_per_context=2,
    learning_rate=1.0,baseline=0.0,horizon=24,training_cells=432,evaluation_cells=108,
    total_cells=540,maximum_native_attempts=12960,native_deadline_seconds=180,
    minimum_expected_reward_gain_per_run=0.05,require_every_context_probability_gain=True,
    full_training_authorized=False,scientific_acceptance=False,
    source_scope='ere-learning-feature-scoped-v2')


def close(actual,expected):
    if isinstance(expected,dict):
        assert isinstance(actual,dict) and set(actual)==set(expected)
        for key,value in expected.items():close(actual[key],value)
    elif isinstance(expected,list):
        assert isinstance(actual,list) and len(actual)==len(expected)
        for x,y in zip(actual,expected):close(x,y)
    elif type(expected) is float:
        assert type(actual) is float and math.isfinite(actual) and abs(actual-expected)<=2e-12
    else:
        assert type(actual) is type(expected) and actual==expected


def next_draw(state):
    state=(state+0x9e3779b97f4a7c15)&MASK
    z=state;z=((z^(z>>30))*0xbf58476d1ce4e5b9)&MASK
    z=((z^(z>>27))*0x94d049bb133111eb)&MASK;z=z^(z>>31)
    return state,(z>>11)/9007199254740992


def probabilities(theta,context):
    offset=native.CONTEXTS.index(context)*4;row=theta[offset:offset+4]
    assert len(theta)==12 and all(type(x) is float and math.isfinite(x) for x in theta)
    maximum=max(row);weights=[math.exp(x-maximum) for x in row];total=math.fsum(weights)
    return [x/total for x in weights]


def policy(theta,context,draw):
    p=probabilities(theta,context);cumulative=0.;action=None
    assert type(draw) is float and 0<=draw<1
    for i,probability in enumerate(p):
        cumulative+=probability
        if draw<cumulative:action=i;break
    if action is None:action=max(i for i,x in enumerate(p) if x>0)
    offset=native.CONTEXTS.index(context)*4
    gradient=[0.]*12
    for i in range(4):gradient[offset+i]=int(i==action)-p[i]
    row=theta[offset:offset+4];maximum=max(row)
    likelihood=row[action]-maximum-math.log(math.fsum(math.exp(x-maximum) for x in row))
    return p,action,likelihood,gradient


def update(theta,samples):
    assert len(samples)==6
    assert all(sum(s['context']==c for s in samples)==2 for c in native.CONTEXTS)
    terms=[[] for _ in range(12)]
    for s in samples:
        assert 0<=s['reward']<=1 and math.isfinite(s['reward'])
        _,action,_,gradient=policy(theta,s['context'],s['draw'])
        assert action==s['action']
        for i,g in enumerate(gradient):terms[i].append(s['reward']*g/2)
    return [theta[i]+math.fsum(terms[i]) for i in range(12)]


def endpoint(theta,rewards):
    assert len(rewards)==12
    contexts=[];expected=[]
    for i,context in enumerate(native.CONTEXTS):
        p=probabilities(theta,context)
        conditional=math.fsum(x*y for x,y in zip(p,rewards[4*i:4*i+4]))
        contexts.append(dict(context=context,probabilities=p,
            matching_edit_probability=p[native.ACTIONS.index(context)],expected_reward=conditional))
        expected.append(conditional/3)
    return dict(theta=theta,contexts=contexts,mean_expected_reward=math.fsum(expected))


def verify_transitions(transitions,training_rows):
    assert len(transitions)==36 and len(training_rows)==432
    output=[];j=0;transition_index=0
    for seed in POLICY_SEEDS:
        live=[0.]*12;reuse=[0.]*12;state=seed
        for iteration in range(UPDATES):
            t=transitions[transition_index];transition_index+=1
            batch=training_rows[j:j+12];j+=12
            assert [r['id'] for r in batch]==list(range(batch[0]['id'],batch[0]['id']+12))
            assert t['first_cell']==batch[0]['id'] and t['last_cell']==batch[-1]['id']
            assert t['policy_seed']==seed and t['iteration']==iteration and t['skipped'] is False
            close(t['before_live'],live);close(t['before_reuse'],reuse)
            draws=[];samples_live=[];samples_reuse=[];k=0
            for context in native.CONTEXTS:
                for _ in range(2):
                    state,draw=next_draw(state);draws.append(dict(draw=draw,state=state))
                    for name,theta,samples in [('live',live,samples_live),('reuse',reuse,samples_reuse)]:
                        r=batch[k];k+=1
                        assert r['phase']=='training' and r['policy_seed']==seed and r['iteration']==iteration
                        assert r['policy']==name and r['context']==context and r['draw_state']==state
                        # Draw identity is exact; approximate numerical tolerance is only for policy arithmetic.
                        assert r['draw']==draw
                        close(r['policy_theta'],theta)
                        p,action,likelihood,gradient=policy(theta,context,draw)
                        assert r['action']==action
                        close(r['probabilities'],p);close(r['log_probability'],likelihood);close(r['score_gradient'],gradient)
                        samples.append(r)
                        if name=='reuse':assert r['reward']==0.
            close(t['draws'],draws)
            live=update(live,samples_live);reuse=update(reuse,samples_reuse)
            close(t['after_live'],live);close(t['after_reuse'],reuse)
            assert reuse==[0.]*12 and t['after_reuse']==[0.]*12
        output.append((seed,live,reuse))
    return output


def audit(directory):
    directory=Path(directory);m=a.load(directory/'manifest.json')
    assert m['profile']==PROFILE and m['status']=='native_complete' and m['design']==DESIGN
    assert m['recorded_cells']==540 and m['completed_updates']==72 and m['native_attempts']==12960
    assert m['failures']==[] and m['native_work_unknown'] is False
    assert m['bounded_learning_authorized'] is True and m['full_training_authorized'] is False
    assert m['training_admitted'] is False and m['scientific_acceptance'] is False and m['independent_physics_replay'] is False
    expected={f'cell-{i:04}.json' for i in range(540)}|{'manifest.json','transitions.json','comparisons.json'}
    assert {p.name for p in directory.glob('*.json')}==expected
    compact=[];phases={};i=0
    for seed in POLICY_SEEDS:
        schedule=[]
        for context in native.CONTEXTS:
            for action in range(4):schedule.append(('baseline',None,None,context,'live_paid',action))
        for iteration in range(12):
            for context in native.CONTEXTS:
                for _ in range(2):
                    schedule.append(('training','live',iteration,context,'live_paid',None))
                    schedule.append(('training','reuse',iteration,context,'preseeded_lure',None))
        for context in native.CONTEXTS:
            for action in range(4):schedule.append(('final',None,None,context,'live_paid',action))
        for context in native.CONTEXTS:
            for arm in native.ARMS[1:]:schedule.append(('control',None,None,context,arm,native.ACTIONS.index(context)))
        for phase,name,iteration,context,arm,action in schedule:
            r=a.load(directory/f'cell-{i:04}.json');n=r['native']
            assert type(r['id']) is int and r['id']==i and r['policy_seed']==seed
            assert r['phase']==phase and r['policy']==name and r['iteration']==iteration and r['context']==context
            assert r['native_seed']==native.SEEDS[i%2] and n['seed']==r['native_seed']
            assert n['arm']==arm and n['context']==context and n['action']==r['action']
            if action is not None:assert r['action']==action
            result=native.audit_receipt(n)
            r.pop('native');r['reward']=float(result['reward']);compact.append(r)
            phases[phase]=phases.get(phase,0)+1;i+=1
            if phase!='training':
                assert r['draw'] is None and r['draw_state'] is None
                assert not any(k in r for k in ('policy_theta','probabilities','log_probability','score_gradient'))
            if phase=='control':assert r['reward']==0.
    assert phases==dict(baseline=36,training=432,final=36,control=36)
    transitions=a.load(directory/'transitions.json')
    policies=verify_transitions(transitions,[r for r in compact if r['phase']=='training'])
    observed=a.load(directory/'comparisons.json');assert len(observed)==3
    comparisons=[]
    for index,(seed,live,reuse) in enumerate(policies):
        rows=[r for r in compact if r['policy_seed']==seed]
        base=[r['reward'] for r in rows if r['phase']=='baseline']
        final=[r['reward'] for r in rows if r['phase']=='final']
        before=endpoint([0.]*12,base);after=endpoint(live,final);after_reuse=endpoint(reuse,final)
        gain=after['mean_expected_reward']-before['mean_expected_reward']
        controls=[dict(context=r['context'],arm=native.ARMS[1:][j%4],reward=r['reward'])
                  for j,r in enumerate(r for r in rows if r['phase']=='control')]
        row=dict(policy_seed=seed,baseline=before,final_live=after,final_reuse=after_reuse,
            expected_reward_gain=gain,every_context_probability_gain=all(c['matching_edit_probability']>0.25 for c in after['contexts']),
            gain_pass=gain>=0.05,reuse_unchanged=reuse==[0.]*12,controls=controls)
        close(observed[index],row);comparisons.append(row)
    passed=all(r['gain_pass'] and r['every_context_probability_gain'] and r['reuse_unchanged'] for r in comparisons)
    assert type(m['learning_criterion_pass']) is bool and m['learning_criterion_pass']==passed
    return dict(accounting_pass=True,profile=PROFILE,cells=540,native_steps_checked=12960,
        completed_updates=72,phase_counts=phases,learning_criterion_pass=passed,
        comparisons=comparisons,training_admitted=False,full_training_authorized=False,
        scientific_acceptance=False,independent_physics_replay=False)


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('directory',type=Path)
    args=parser.parse_args();print(__import__('json').dumps(audit(args.directory),indent=2))
