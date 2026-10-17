#!/usr/bin/env python3
"""Independent native-ledger and held-out-update accounting, not physics replay."""
import argparse
import math
from pathlib import Path
import verify_ere_rule_composition_v2 as native
import verify_ere_rule_choice_learning as single

a=native.integrity
close=single.close
PROFILE='E2-rule-composition-learning-v1'
POLICY_SEEDS=[993748717001,993748717002,993748717003]
DEVELOPMENT=[c for c in native.CONTEXTS if c[0]+1!=c[1]]
RESERVED=[c for c in native.CONTEXTS if c[0]+1==c[1]]
DESIGN=dict(profile=PROFILE,policy_seeds=POLICY_SEEDS,native_seeds=native.SEEDS,
    development_combinations=DEVELOPMENT,reserved_update_combinations=RESERVED,
    updates_per_policy=12,samples_per_context=2,learning_rate=4,batch_denominator=12,
    baseline=0,horizon=24,training_cells=864,endpoint_cells=288,control_cells=54,
    total_cells=1206,maximum_native_attempts=28944,native_deadline_seconds=300,
    minimum_expected_reward_gain_per_run=0.01,require_every_reserved_joint_probability_gain=True,
    full_training_authorized=False,scientific_acceptance=False,independent_physics_replay=False,
    source_scope='ere-composition-learning-feature-scoped-v1')


def probabilities(theta,context):
    assert context in native.CONTEXTS
    return [single.probabilities(theta,context[0]),single.probabilities(theta,context[1]-1)]


def policy(theta,context,draws):
    assert context in native.CONTEXTS and len(draws)==2
    slots=[single.policy(theta,c,d) for c,d in zip([context[0],context[1]-1],draws)]
    return ([s[0] for s in slots],[s[1] for s in slots],math.fsum(s[2] for s in slots),
            [math.fsum(s[3][i] for s in slots) for i in range(12)])


def update(theta,samples):
    assert len(samples)==12
    assert all(sum(s['context']==c for s in samples)==2 for c in DEVELOPMENT)
    terms=[[] for _ in range(12)]
    for s in samples:
        assert type(s['reward']) is float and math.isfinite(s['reward']) and 0<=s['reward']<=1
        _,actions,_,gradient=policy(theta,s['context'],s['draws'])
        assert s['actions']==actions
        for i,g in enumerate(gradient):terms[i].append(4*s['reward']*g/12)
    return [theta[i]+math.fsum(terms[i]) for i in range(12)]


def endpoint(theta,rewards):
    assert len(rewards)==48 and all(type(r) is float and math.isfinite(r) and 0<=r<=1 for r in rewards)
    rows=[];terms=[]
    for i,c in enumerate(RESERVED):
        p=probabilities(theta,c)
        expected=math.fsum(p[0][x]*p[1][y]*rewards[i*16+x*4+y] for x in range(4) for y in range(4))
        correct=[native.CHOICES[j].index(c[j]) for j in range(2)]
        rows.append(dict(context=c,probabilities=p,correct_joint_probability=p[0][correct[0]]*p[1][correct[1]],expected_reward=expected))
        terms.append(expected/3)
    return dict(theta=theta,contexts=rows,mean_expected_reward=math.fsum(terms))


def verify_transitions(transitions,rows):
    assert len(transitions)==36 and len(rows)==864
    policies=[];j=0;t_index=0
    for seed in POLICY_SEEDS:
        state=seed;live=[0.]*12;reuse=[0.]*12
        for iteration in range(12):
            t=transitions[t_index];t_index+=1;batch=rows[j:j+24];j+=24
            assert t['policy_seed']==seed and t['iteration']==iteration and t['skipped'] is False
            assert [r['id'] for r in batch]==list(range(batch[0]['id'],batch[0]['id']+24))
            assert t['first_cell']==batch[0]['id'] and t['last_cell']==batch[-1]['id']
            close(t['before_live'],live);close(t['before_reuse'],reuse)
            drawn=[];live_samples=[];reuse_samples=[];k=0
            for c in DEVELOPMENT:
                for _ in range(2):
                    states=[];draws=[]
                    for _ in range(2):
                        state,draw=single.next_draw(state);states.append(state);draws.append(draw)
                    drawn.append(dict(draws=draws,states=states))
                    for name,theta,samples in [('live',live,live_samples),('reuse',reuse,reuse_samples)]:
                        r=batch[k];k+=1
                        assert r['phase']=='training' and r['policy_seed']==seed and r['iteration']==iteration
                        assert r['context']==c and r['policy']==name and r['draws']==draws and r['draw_states']==states
                        assert r['native_seed']==native.SEEDS[(r['id']//2)%2]
                        close(r['policy_theta'],theta)
                        p,actions,likelihood,gradient=policy(theta,c,draws)
                        assert r['actions']==actions
                        close(r['probabilities'],p);close(r['log_probability'],likelihood);close(r['score_gradient'],gradient)
                        samples.append(r)
                        if name=='reuse':assert r['reward']==0.
                    assert batch[k-1]['native_seed']==batch[k-2]['native_seed']
            # Exact recorded draws, not numerically approximate provenance.
            assert t['draws']==drawn
            live=update(live,live_samples);reuse=update(reuse,reuse_samples)
            close(t['after_live'],live);close(t['after_reuse'],reuse)
            assert reuse==[0.]*12
        policies.append((seed,live,reuse))
    return policies


def verify_comparisons(observed,policies,rows):
    assert len(observed)==3
    comparisons=[]
    for index,(seed,live,reuse) in enumerate(policies):
        selected=[r for r in rows if r['policy_seed']==seed]
        base=[r['reward'] for r in selected if r['phase']=='baseline']
        final=[r['reward'] for r in selected if r['phase']=='final']
        before=endpoint([0.]*12,base);after=endpoint(live,final)
        frozen=endpoint([0.]*12,final);after_reuse=endpoint(reuse,final)
        controls=[dict(context=r['context'],arm=r['arm'],reward=r['reward']) for r in selected if r['phase']=='control']
        assert len(controls)==18
        controls_pass=all(r['reward']==(9/24 if r['arm']=='reverse_paid' else 0.) for r in controls)
        # Native failure is not a negative learning outcome.
        assert controls_pass and before==frozen and reuse==[0.]*12
        gain=after['mean_expected_reward']-before['mean_expected_reward']
        row=dict(policy_seed=seed,baseline=before,final_live=after,final_reuse=after_reuse,final_frozen=frozen,
            expected_reward_gain=gain,gain_pass=gain>=0.01,
            every_reserved_joint_probability_gain=all(c['correct_joint_probability']>1/16 for c in after['contexts']),
            reuse_unchanged=reuse==[0.]*12,frozen_unchanged=before==frozen,controls_pass=controls_pass,controls=controls)
        close(observed[index],row);comparisons.append(row)
    passed=all(r['gain_pass'] and r['every_reserved_joint_probability_gain'] and r['reuse_unchanged'] and r['frozen_unchanged'] and r['controls_pass'] for r in comparisons)
    return comparisons,passed


def schedule():
    for seed in POLICY_SEEDS:
        for c in RESERVED:
            for x in range(4):
                for y in range(4):yield seed,'baseline',None,None,c,'live_paid',[x,y]
        for iteration in range(12):
            for c in DEVELOPMENT:
                for _ in range(2):
                    yield seed,'training','live',iteration,c,'live_paid',None
                    yield seed,'training','reuse',iteration,c,'preseeded_lure',None
        for c in RESERVED:
            for x in range(4):
                for y in range(4):yield seed,'final',None,None,c,'live_paid',[x,y]
        for c in RESERVED:
            correct=[native.CHOICES[j].index(c[j]) for j in range(2)]
            for arm in native.ARMS[1:]:yield seed,'control',None,None,c,arm,correct


def audit(directory):
    directory=Path(directory);m=a.load(directory/'manifest.json')
    assert m['profile']==PROFILE and m['status']=='native_complete' and m['design']==DESIGN
    assert m['recorded_cells']==1206 and m['completed_updates']==72 and m['native_attempts']==28944
    assert m['failures']==[] and m['native_work_unknown'] is False
    assert m['bounded_learning_authorized'] is True
    for key in ('training_admitted','full_training_authorized','scientific_acceptance','independent_physics_replay'):assert m[key] is False
    assert {p.name for p in directory.glob('*.json')}=={f'cell-{i:04}.json' for i in range(1206)}|{'manifest.json','transitions.json','comparisons.json'}
    rows=[];phases={};fires=edits=0
    for i,(seed,phase,name,iteration,c,arm,actions) in enumerate(schedule()):
        r=a.load(directory/f'cell-{i:04}.json');n=r['native']
        keys={'id','phase','policy_seed','context','actions','native_seed','policy','iteration','draws','draw_states','native'}
        if phase=='training':keys|={'policy_theta','probabilities','log_probability','score_gradient'}
        assert set(r)==keys
        assert type(r['id']) is int and r['id']==i and r['policy_seed']==seed
        assert r['phase']==phase and r['policy']==name and r['iteration']==iteration and r['context']==c
        assert r['native_seed']==native.SEEDS[(i//2)%2] and n['seed']==r['native_seed']
        assert n['context']==c and n['arm']==arm and n['actions']==r['actions']
        if actions is not None:assert r['actions']==actions
        if phase!='training':assert r['draws'] is None and r['draw_states'] is None
        checked=native.audit_receipt(n)
        fires+=checked['non_meta_novel_fires'];edits+=checked['paid_edits']
        r.pop('native');r['reward']=float(checked['reward']);r['arm']=arm;rows.append(r)
        phases[phase]=phases.get(phase,0)+1
    assert len(rows)==1206 and phases==dict(baseline=144,training=864,final=144,control=54)
    policies=verify_transitions(a.load(directory/'transitions.json'),[r for r in rows if r['phase']=='training'])
    comparisons,passed=verify_comparisons(a.load(directory/'comparisons.json'),policies,rows)
    assert type(m['learning_criterion_pass']) is bool and m['learning_criterion_pass']==passed
    return dict(accounting_pass=True,profile=PROFILE,cells=1206,native_steps_checked=28944,completed_updates=72,
        phase_counts=phases,novel_fires=fires,paid_edits=edits,learning_criterion_pass=passed,
        comparisons=comparisons,training_admitted=False,full_training_authorized=False,
        scientific_acceptance=False,independent_physics_replay=False)


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('directory',type=Path)
    print(__import__('json').dumps(audit(p.parse_args().directory),indent=2))
