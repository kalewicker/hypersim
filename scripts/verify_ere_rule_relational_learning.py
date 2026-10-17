#!/usr/bin/env python3
"""Independent paired learning/decision/native accounting, not Engine replay."""
import argparse
import math
from pathlib import Path
import verify_ere_rule_relational as native
from verify_ere_rule_choice_learning import close,next_draw

a=native.integrity
PROFILE='E2-rule-relational-learning-v1'
POLICY_SEEDS=[993748719001,993748719002,993748719003]
DEVELOPMENT=[c for c in native.CONTEXTS if c[0]+1!=c[1]]
RESERVED=[c for c in native.CONTEXTS if c[0]+1==c[1]]
MODES=['graph','blind']
DESIGN=dict(profile=PROFILE,policy_seeds=POLICY_SEEDS,native_seeds=native.SEEDS,
    development_combinations=DEVELOPMENT,reserved_update_combinations=RESERVED,policies=MODES,
    parameters_per_policy=16,updates_per_policy=12,samples_per_context=2,learning_rate=4,batch_denominator=12,
    baseline=0,horizon=24,training_cells=864,endpoint_cells=864,control_cells=54,total_cells=1782,
    maximum_native_attempts=42768,native_deadline_seconds=420,graph_blind_balanced_ceiling=1/24,
    minimum_ceiling_margin_per_seed=0.01,minimum_graph_minus_blind_gain_per_seed=0.01,
    minimum_reserved_expected_reward_gain_per_seed=0.01,require_every_reserved_joint_probability_gain=True,
    full_training_authorized=False,scientific_acceptance=False,independent_physics_replay=False,
    source_scope='ere-relational-learning-feature-scoped-v1')


def make_input(mode,observation):
    assert mode in MODES
    return dict(mode=mode,features=observation['features'] if mode=='graph' else [[1/3]*3 for _ in range(2)])


def logits(theta,inp):
    assert len(theta)==16 and all(type(t) is float and math.isfinite(t) for t in theta)
    assert inp['mode'] in MODES
    f=inp['features'];assert len(f)==2 and all(len(row)==3 for row in f)
    if inp['mode']=='blind':assert f==[[1/3]*3 for _ in range(2)]
    else:assert all(sum(row)==1 and set(row)<=set((0.,1.)) for row in f)
    rows=[[theta[a]+math.fsum(theta[4+a*3+k]*f[s][k] for k in range(3)) for a in range(4)] for s in range(2)]
    assert all(math.isfinite(t) for row in rows for t in row)
    assert all(math.isfinite(t-max(row)) for row in rows for t in row)
    return rows


def statistics(theta,inp,actions=None):
    rows=logits(theta,inp);p=[]
    for row in rows:
        maximum=max(row);w=[math.exp(t-maximum) for t in row];total=math.fsum(w);p.append([v/total for v in w])
    if actions is None:return p
    assert len(actions)==2 and all(type(i) is int and 0<=i<4 for i in actions)
    lp=math.fsum(rows[s][actions[s]]-max(rows[s])-math.log(math.fsum(math.exp(t-max(rows[s])) for t in rows[s])) for s in range(2))
    assert math.isfinite(lp);g=[0.]*16
    for s in range(2):
        for i in range(4):
            d=int(actions[s]==i)-p[s][i];g[i]+=d
            for k in range(3):g[4+i*3+k]+=d*inp['features'][s][k]
    return p,lp,g


def policy(theta,inp,draws):
    assert len(draws)==2 and all(type(d) is float and math.isfinite(d) and 0<=d<1 for d in draws)
    p=statistics(theta,inp);actions=[]
    for s,d in enumerate(draws):
        cumulative=0.;chosen=None
        for i,q in enumerate(p[s]):
            cumulative+=q
            if d<cumulative:chosen=i;break
        if chosen is None:chosen=max(i for i,q in enumerate(p[s]) if q>0)
        actions.append(chosen)
    p,lp,g=statistics(theta,inp,actions)
    return p,actions,lp,g


def update(mode,theta,samples):
    assert len(samples)==12 and all(sum(r['context']==c for r in samples)==2 for c in DEVELOPMENT)
    terms=[[] for _ in range(16)]
    for r in samples:
        d=r['decision'];assert d['mode']==mode and d['input']==make_input(mode,d['observation'])
        assert type(r['reward']) is float and math.isfinite(r['reward']) and 0<=r['reward']<=1
        _,actions,_,g=policy(theta,d['input'],d['draws']);assert actions==r['actions']
        for i,v in enumerate(g):terms[i].append(4*r['reward']*v/12)
    return [theta[i]+math.fsum(terms[i]) for i in range(16)]


def endpoint(mode,theta,rewards,observations):
    assert len(rewards)==144 and len(observations)==9
    assert all(type(r) is float and math.isfinite(r) and 0<=r<=1 for r in rewards)
    rows=[];balanced=[];reserved=[]
    for i,(c,o) in enumerate(zip(native.CONTEXTS,observations)):
        inp=make_input(mode,o);p=statistics(theta,inp)
        expected=math.fsum(p[0][x]*p[1][y]*rewards[i*16+x*4+y] for x in range(4) for y in range(4))
        correct=[native.CHOICES[s].index(c[s]) for s in range(2)]
        rows.append(dict(context=c,observation=o,input=inp,probabilities=p,correct_joint_probability=p[0][correct[0]]*p[1][correct[1]],expected_reward=expected,reserved=c in RESERVED))
        balanced.append(expected/9)
        if c in RESERVED:reserved.append(expected/3)
    return dict(mode=mode,theta=theta,contexts=rows,balanced_expected_reward=math.fsum(balanced),reserved_expected_reward=math.fsum(reserved))


def verify_transitions(transitions,rows):
    assert len(transitions)==36 and len(rows)==864
    policies=[];j=0;ti=0
    for seed in POLICY_SEEDS:
        state=seed;graph=[0.]*16;blind=[0.]*16
        for iteration in range(12):
            t=transitions[ti];ti+=1;batch=rows[j:j+24];j+=24
            assert t['policy_seed']==seed and t['iteration']==iteration and t['skipped'] is False
            assert [r['id'] for r in batch]==list(range(batch[0]['id'],batch[0]['id']+24))
            assert t['first_cell']==batch[0]['id'] and t['last_cell']==batch[-1]['id']
            close(t['before_graph'],graph);close(t['before_blind'],blind)
            drawn=[];samples={'graph':[],'blind':[]};k=0
            for c in DEVELOPMENT:
                for _ in range(2):
                    states=[];draws=[]
                    for _ in range(2):state,draw=next_draw(state);states.append(state);draws.append(draw)
                    drawn.append(dict(draws=draws,states=states))
                    for mode,theta in [('graph',graph),('blind',blind)]:
                        r=batch[k];k+=1;d=r['decision']
                        assert r['phase']=='training' and r['policy_seed']==seed and r['context']==c
                        assert r['native_seed']==native.SEEDS[(r['id']//2)%2]
                        assert d['mode']==mode and d['iteration']==iteration and d['draws']==draws and d['draw_states']==states
                        assert d['observation']==r['observed']
                        inp=make_input(mode,r['observed']);assert d['input']==inp
                        p,actions,lp,g=policy(theta,inp,draws);assert r['actions']==actions
                        expected=dict(mode=mode,iteration=iteration,observation=r['observed'],input=inp,draws=draws,draw_states=states,
                            theta=theta,probabilities=p,log_probability=lp,score_gradient=g)
                        close(d,expected);samples[mode].append(r)
                    assert batch[k-1]['native_seed']==batch[k-2]['native_seed']
            assert t['draws']==drawn
            graph=update('graph',graph,samples['graph']);blind=update('blind',blind,samples['blind'])
            close(t['after_graph'],graph);close(t['after_blind'],blind)
        policies.append((seed,graph,blind))
    return policies


def verify_comparisons(observed,policies,rows):
    assert len(observed)==3
    comparisons=[]
    for index,(seed,graph,blind) in enumerate(policies):
        selected=[r for r in rows if r['policy_seed']==seed]
        baseline=[r for r in selected if r['phase']=='baseline'];final=[r for r in selected if r['phase']=='final']
        assert len(baseline)==len(final)==144
        observations=[baseline[16*i]['observed'] for i in range(9)]
        for i in range(144):assert baseline[i]['observed']==final[i]['observed']==observations[i//16]
        base=[r['reward'] for r in baseline];end=[r['reward'] for r in final]
        before=endpoint('graph',[0.]*16,base,observations);after=endpoint('graph',graph,end,observations)
        blind_end=endpoint('blind',blind,end,observations);frozen=endpoint('graph',[0.]*16,end,observations)
        controls=[dict(context=r['context'],arm=r['arm'],reward=r['reward']) for r in selected if r['phase']=='control']
        assert len(controls)==18
        controls_pass=all(c['reward']==(9/24 if c['arm']=='reverse_paid' else 0.) for c in controls)
        graph_reward=after['balanced_expected_reward'];blind_reward=blind_end['balanced_expected_reward']
        reserved_gain=after['reserved_expected_reward']-before['reserved_expected_reward']
        assert controls_pass and base==end and before==frozen
        assert blind_reward<=1/24+2e-12
        assert all(c['probabilities']==blind_end['contexts'][0]['probabilities'] for c in blind_end['contexts'])
        row=dict(policy_seed=seed,baseline=before,final_graph=after,final_blind=blind_end,final_frozen=frozen,
            ceiling_margin=graph_reward-1/24,graph_minus_blind=graph_reward-blind_reward,reserved_gain=reserved_gain,
            ceiling_pass=graph_reward>=1/24+0.01,contrast_pass=graph_reward-blind_reward>=0.01,
            blind_bound_pass=blind_reward<=1/24+2e-12,reserved_gain_pass=reserved_gain>=0.01,
            every_reserved_joint_probability_gain=all(c['correct_joint_probability']>1/16 for c in after['contexts'] if c['reserved']),
            frozen_unchanged=before==frozen,native_matrix_unchanged=base==end,controls_pass=controls_pass,controls=controls)
        close(observed[index],row);comparisons.append(row)
    keys=('ceiling_pass','contrast_pass','blind_bound_pass','reserved_gain_pass','every_reserved_joint_probability_gain','frozen_unchanged','native_matrix_unchanged','controls_pass')
    return comparisons,all(all(c[k] for k in keys) for c in comparisons)


def schedule():
    for seed in POLICY_SEEDS:
        for c in native.CONTEXTS:
            for x in range(4):
                for y in range(4):yield seed,'baseline',None,None,c,'live_paid',[x,y]
        for iteration in range(12):
            for c in DEVELOPMENT:
                for _ in range(2):
                    for mode in MODES:yield seed,'training',mode,iteration,c,'live_paid',None
        for c in native.CONTEXTS:
            for x in range(4):
                for y in range(4):yield seed,'final',None,None,c,'live_paid',[x,y]
        for c in RESERVED:
            correct=[native.CHOICES[s].index(c[s]) for s in range(2)]
            for arm in native.ARMS[1:]:yield seed,'control',None,None,c,arm,correct


def audit(directory):
    directory=Path(directory);m=a.load(directory/'manifest.json')
    assert m['profile']==PROFILE and m['status']=='native_complete' and m['design']==DESIGN
    assert m['recorded_cells']==1782 and m['completed_updates']==72 and m['native_attempts']==42768
    assert m['failures']==[] and m['native_work_unknown'] is False and m['bounded_learning_authorized'] is True
    for k in ('training_admitted','full_training_authorized','scientific_acceptance','independent_physics_replay'):assert m[k] is False
    plan=list(schedule());expected={f'cell-{i:04}.json' for i in range(1782)}|{'manifest.json','transitions.json','comparisons.json'}
    expected|={f'decision-{i:04}.json' for i,(_,phase,*_) in enumerate(plan) if phase=='training'}
    assert {p.name for p in directory.glob('*.json')}==expected
    rows=[];phases={};fires=edits=0;reference=None
    for i,(seed,phase,mode,iteration,c,arm,actions) in enumerate(plan):
        r=a.load(directory/f'cell-{i:04}.json');n=r['native']
        assert set(r)=={'id','phase','policy_seed','context','actions','native_seed','decision','native'}
        assert type(r['id']) is int and r['id']==i and r['phase']==phase and r['policy_seed']==seed and r['context']==c
        assert r['native_seed']==native.SEEDS[(i//2)%2] and n['seed']==r['native_seed']
        assert n['context']==c and n['arm']==arm and n['actions']==r['actions']
        if actions is not None:assert r['actions']==actions
        if phase=='training':
            assert r['decision']['mode']==mode and r['decision']['iteration']==iteration
            assert a.load(directory/f'decision-{i:04}.json')=={k:v for k,v in r.items() if k!='native'}
        else:assert r['decision'] is None
        result=native.audit_receipt(n);fires+=result['non_meta_novel_fires'];edits+=result['paid_edits']
        ordinary=[(key,value['data']) for key,value in n['initial_inventory']['nodes'] if value['data']['label']<native.META]
        if reference is None:reference=ordinary
        assert ordinary==reference
        r.pop('native');r['observed']=n['encoded_observation'];r['arm']=arm;r['reward']=float(result['reward']);rows.append(r)
        phases[phase]=phases.get(phase,0)+1
    assert phases==dict(baseline=432,training=864,final=432,control=54)
    policies=verify_transitions(a.load(directory/'transitions.json'),[r for r in rows if r['phase']=='training'])
    comparisons,passed=verify_comparisons(a.load(directory/'comparisons.json'),policies,rows)
    assert type(m['learning_criterion_pass']) is bool and m['learning_criterion_pass']==passed
    return dict(accounting_pass=True,profile=PROFILE,cells=1782,native_steps_checked=42768,completed_updates=72,
        pre_emission_decisions_checked=864,phase_counts=phases,novel_fires=fires,paid_edits=edits,
        learning_criterion_pass=passed,comparisons=comparisons,training_admitted=False,
        full_training_authorized=False,scientific_acceptance=False,independent_physics_replay=False)


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('directory',type=Path)
    print(__import__('json').dumps(audit(p.parse_args().directory),indent=2))
