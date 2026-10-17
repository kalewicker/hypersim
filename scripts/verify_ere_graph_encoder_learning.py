#!/usr/bin/env python3
"""Independent native encoder-learning ledger/forward-AD/update accounting."""
import argparse
import json
import math
from pathlib import Path
import verify_ere_graph_encoder as encoder

old=encoder.previous;native=old.native;a=old.a;close=old.close
PROFILE='E2-graph-encoder-learning-v1'
INCIDENTAL={
    'ere_rule_relational_learning_trial_20261004':{'.DS_Store':'c6e45f89821173f05534c41a6fdb5ede9acb4b688595bd305c70c52ab0887562'},
    'ere_rule_relational_development_20261004':{'.DS_Store':'1b3db65d4a1f2f6d1d9e5ab3d5d85b3c28e38aa17e91638a881aa81fc3ba1b78'},
    'ere_rule_composition_development_20261004':{'.DS_Store':'fbf21b3ca241344d9904c0630c295162aedb3198fd5e66e021c1f12217fc3f30'},
    'ere_rule_choice_learning_trial_v2_20261004':{'.DS_Store':'e9839136be2a311ce258595c04ea3c5781febdc3c0818d537333d2db1e7a5874'},
    'ere_rule_choice_adapter_development_20261004':{'.DS_Store':'e95110d2c718f21f2efa6e69bfff474144ccc4cf0cd6fbc64aa6b13fd694dee7'},
    'ere_rule_lifecycle_fixture_development_20261003':{'.DS_Store':'665ca8acfcfb15c4fbae30f46abd376e60ec05d1d906bec9abc7ffa2d5ee6fd4'}}
POLICY_SEEDS=[993748721001,993748721002,993748721003]
DEVELOPMENT=old.DEVELOPMENT;RESERVED=old.RESERVED;MODES=old.MODES
POSITIVE_THETAS=[
    [-0.6608964527051322,0.42868730111578657,-0.0009963506465326816,0.23320550223587838,-0.2923549148213898,-0.1647095040149254,-0.2038320338688169,1.0725015286111028,-0.2803673831237244,-0.363446844371592,-0.386319866621485,0.6745730185507594,-0.28924950257580706,-0.393826747168228,-0.22949613141210964,0.8565283808162159],
    [-0.7833765011282057,0.15927194778481651,0.28576511633798357,0.3383394370054055,-0.22692701033897494,-0.24505920959376212,-0.31139028119546847,0.9897186896731762,-0.3825524404105463,-0.44789430147781323,-0.34749366010078886,1.0825633319442847,-0.4493045555055122,-0.41529801923341214,-0.45495168193997626,1.2085891381787937],
    [-0.3942903508320628,0.0018732767775542433,0.225235637224086,0.16718143683042255,-0.10126309083256886,-0.14371738819443286,-0.14930987180506108,0.3568891668946645,-0.1807526551966802,-0.17426323492043,-0.12184476771338841,0.5214823457717513,-0.174401940834277,-0.1337813083487072,-0.1970123023806383,0.497975047559768]]
DESIGN=dict(old.DESIGN,profile=PROFILE,policy_seeds=POLICY_SEEDS,parameters_per_policy=110,
    initialization_seed=encoder.SEED,initialization_scale=0.15,positive_control_source=encoder.INPUT_NAME,
    positive_control_root_seal=encoder.INPUT_SEAL,positive_control_thetas=POSITIVE_THETAS,
    source_scope='ere-graph-encoder-learning-feature-scoped-v1',historical_incidental_files=INCIDENTAL)
KEYS=('ceiling_pass','contrast_pass','blind_bound_pass','reserved_gain_pass','every_reserved_joint_probability_gain',
    'frozen_unchanged','uniform_unchanged','native_matrix_unchanged','positive_control_unchanged','positive_control_pass','controls_pass')


def verify_incidental(archive,observed):
    assert observed==INCIDENTAL.get(archive,{}),'unregistered incidental file or changed bytes'


def policy(theta,inp,draws):
    p=encoder.statistics(theta,inp,derivatives=False);actions=encoder.sample(p,draws)
    p,lp,g=encoder.statistics(theta,inp,actions);return p,actions,lp,g


def update(mode,theta,samples):
    assert len(samples)==12 and all(sum(r['context']==c for r in samples)==2 for c in DEVELOPMENT)
    terms=[[] for _ in range(110)]
    for r in samples:
        d=r['decision'];assert d['mode']==mode and d['input']==encoder.make_input(mode,d['observation'])
        assert type(r['reward']) is float and math.isfinite(r['reward']) and 0<=r['reward']<=1
        _,actions,_,g=policy(theta,d['input'],d['draws']);assert r['actions']==actions
        for i,v in enumerate(g):terms[i].append(4*r['reward']*v/12)
    result=[theta[i]+math.fsum(t) for i,t in enumerate(terms)];encoder.statistics(result,samples[0]['decision']['input'],derivatives=False);return result


def endpoint(mode,theta,rewards,observations):
    assert len(rewards)==144 and len(observations)==9 and all(type(r) is float and math.isfinite(r) and 0<=r<=1 for r in rewards)
    contexts=[];balanced=[];reserved=[]
    for i,(c,o) in enumerate(zip(native.CONTEXTS,observations)):
        inp=encoder.make_input(mode,o);p=encoder.statistics(theta,inp,derivatives=False)
        expected=math.fsum(p[0][x]*p[1][y]*rewards[i*16+x*4+y] for x in range(4) for y in range(4))
        correct=[native.CHOICES[s].index(c[s]) for s in range(2)]
        contexts.append(dict(context=c,observation=o,input=inp,probabilities=p,correct_joint_probability=p[0][correct[0]]*p[1][correct[1]],expected_reward=expected,reserved=c in RESERVED))
        balanced.append(expected/9)
        if c in RESERVED:reserved.append(expected/3)
    return dict(mode=mode,theta=theta,contexts=contexts,balanced_expected_reward=math.fsum(balanced),reserved_expected_reward=math.fsum(reserved))


def verify_transitions(transitions,rows):
    assert len(transitions)==36 and len(rows)==864;policies=[];j=ti=0
    for si,seed in enumerate(POLICY_SEEDS):
        state=seed;graph=encoder.initialize();blind=graph.copy()
        for iteration in range(12):
            t=transitions[ti];ti+=1;batch=rows[j:j+24];j+=24
            first=si*594+144+iteration*24
            assert t['policy_seed']==seed and t['iteration']==iteration and t['skipped'] is False
            assert [r['id'] for r in batch]==list(range(first,first+24)) and t['first_cell']==first and t['last_cell']==first+23
            close(t['before_graph'],graph);close(t['before_blind'],blind)
            drawn=[];samples={'graph':[],'blind':[]};k=0
            for c in DEVELOPMENT:
                for _ in range(2):
                    draws=[];states=[]
                    for _ in range(2):state,draw=old.next_draw(state);draws.append(draw);states.append(state)
                    drawn.append(dict(draws=draws,states=states))
                    for mode,theta in [('graph',graph),('blind',blind)]:
                        r=batch[k];k+=1;d=r['decision']
                        assert r['phase']=='training' and r['policy_seed']==seed and r['context']==c and r['native_seed']==native.SEEDS[(r['id']//2)%2]
                        assert d['mode']==mode and d['iteration']==iteration and d['draws']==draws and d['draw_states']==states and d['observation']==r['observed']
                        inp=encoder.make_input(mode,r['observed']);assert d['input']==inp
                        p,actions,lp,g=policy(theta,inp,draws);assert r['actions']==actions
                        close(d,dict(mode=mode,iteration=iteration,observation=r['observed'],input=inp,draws=draws,draw_states=states,theta=theta,probabilities=p,log_probability=lp,score_gradient=g))
                        samples[mode].append(r)
                    assert batch[k-2]['native_seed']==batch[k-1]['native_seed']
            assert t['draws']==drawn
            graph=update('graph',graph,samples['graph']);blind=update('blind',blind,samples['blind'])
            close(t['after_graph'],graph);close(t['after_blind'],blind)
        policies.append((seed,graph,blind))
    return policies


def comparison(seed,index,graph,blind,base,end,observations,controls):
    initial=encoder.initialize();before=endpoint('graph',initial,base,observations);before_blind=endpoint('blind',initial,base,observations)
    after=endpoint('graph',graph,end,observations);blind_end=endpoint('blind',blind,end,observations)
    frozen=endpoint('graph',initial,end,observations);frozen_blind=endpoint('blind',initial,end,observations)
    uniform_base=endpoint('graph',[0.]*110,base,observations);uniform_final=endpoint('graph',[0.]*110,end,observations)
    # Derive the old positive readout from raw adjacency independently of policy
    # input: it is separate frozen evaluation, never part of an encoder decision.
    hand=[]
    for o in observations:
        adjacency=o['adjacency'];anchor=o['labels'].index(12);dist={anchor:0};queue=[anchor]
        for i in queue:
            for j,connected in enumerate(adjacency[i]):
                if connected and j not in dist:dist[j]=dist[i]+1;queue.append(j)
        features=[[float(dist.get(o['labels'].index(label))==2-s) for label in native.CHOICES[s][1:]] for s in range(2)]
        assert all(sum(row)==1 for row in features);hand.append(dict(features=features,marginals=encoder.LABELS))
    positive_base=old.endpoint('graph',POSITIVE_THETAS[index],base,hand);positive_final=old.endpoint('graph',POSITIVE_THETAS[index],end,hand)
    g=after['balanced_expected_reward'];b=blind_end['balanced_expected_reward'];gain=after['reserved_expected_reward']-before['reserved_expected_reward']
    return dict(policy_seed=seed,baseline=before,baseline_blind=before_blind,final_graph=after,final_blind=blind_end,final_frozen=frozen,final_frozen_blind=frozen_blind,
        uniform_base=uniform_base,uniform_final=uniform_final,positive_control_base=positive_base,positive_control_final=positive_final,
        ceiling_margin=g-1/24,graph_minus_blind=g-b,reserved_gain=gain,ceiling_pass=g>=1/24+0.01,contrast_pass=g-b>=0.01,
        blind_bound_pass=b<=1/24+2e-12,reserved_gain_pass=gain>=0.01,
        every_reserved_joint_probability_gain=all(c['correct_joint_probability']>o['correct_joint_probability'] for c,o in zip(after['contexts'],before['contexts']) if c['reserved']),
        frozen_unchanged=before==frozen and before_blind==frozen_blind,uniform_unchanged=uniform_base==uniform_final,native_matrix_unchanged=base==end,
        positive_control_unchanged=positive_base==positive_final,positive_control_pass=positive_final['balanced_expected_reward']>=1/24+0.01,
        controls_pass=all(c['reward']==(9/24 if c['arm']=='reverse_paid' else 0.) for c in controls),controls=controls)


def verify_comparisons(observed,policies,rows):
    assert len(observed)==3;comparisons=[]
    for i,(seed,graph,blind) in enumerate(policies):
        selected=[r for r in rows if r['policy_seed']==seed];base=[r for r in selected if r['phase']=='baseline'];end=[r for r in selected if r['phase']=='final']
        assert len(base)==len(end)==144;observations=[base[16*j]['observed'] for j in range(9)]
        assert all(base[j]['observed']==end[j]['observed']==observations[j//16] for j in range(144))
        controls=[dict(context=r['context'],arm=r['arm'],reward=r['reward']) for r in selected if r['phase']=='control'];assert len(controls)==18
        row=comparison(seed,i,graph,blind,[r['reward'] for r in base],[r['reward'] for r in end],observations,controls)
        assert all(row[k] for k in ('frozen_unchanged','uniform_unchanged','native_matrix_unchanged','positive_control_unchanged','positive_control_pass','controls_pass','blind_bound_pass'))
        assert all(c['probabilities']==row['final_blind']['contexts'][0]['probabilities'] for c in row['final_blind']['contexts'])
        close(observed[i],row);comparisons.append(row)
    return comparisons,all(all(c[k] for k in KEYS) for c in comparisons)


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


def audit_native(receipt):
    result=native.audit_receipt(receipt);observed=encoder.observe(receipt['initial_inventory'])
    assert receipt['raw_graph_observation']==observed
    return result,observed


def audit(directory):
    directory=Path(directory);m=a.load(directory/'manifest.json')
    assert m['profile']==PROFILE and m['status']=='native_complete' and m['design']==DESIGN
    assert m['recorded_cells']==1782 and m['completed_updates']==72 and m['native_attempts']==42768
    assert m['failures']==[] and m['native_work_unknown'] is False and m['bounded_learning_authorized'] is True
    for key in ('training_admitted','full_training_authorized','scientific_acceptance','independent_physics_replay'):assert m[key] is False
    plan=list(schedule());expected={f'cell-{i:04}.json' for i in range(1782)}|{'manifest.json','transitions.json','comparisons.json'}
    expected|={f'decision-{i:04}.json' for i,(_,phase,*_) in enumerate(plan) if phase=='training'}
    assert {p.name for p in directory.glob('*.json')}==expected
    rows=[];phases={};fires=edits=0;reference=None
    for i,(seed,phase,mode,iteration,c,arm,actions) in enumerate(plan):
        r=a.load(directory/f'cell-{i:04}.json');n=r['native']
        assert set(r)=={'id','phase','policy_seed','context','actions','native_seed','decision','native'}
        assert type(r['id']) is int and r['id']==i and r['phase']==phase and r['policy_seed']==seed and r['context']==c
        assert r['native_seed']==native.SEEDS[(i//2)%2] and n['seed']==r['native_seed'] and n['context']==c and n['arm']==arm and n['actions']==r['actions']
        if actions is not None:assert r['actions']==actions
        if phase=='training':
            assert r['decision']['mode']==mode and r['decision']['iteration']==iteration
            assert a.load(directory/f'decision-{i:04}.json')=={k:v for k,v in r.items() if k!='native'}
        else:assert r['decision'] is None
        result,observed=audit_native(n);fires+=result['non_meta_novel_fires'];edits+=result['paid_edits']
        ordinary=[(key,value['data']) for key,value in n['initial_inventory']['nodes'] if value['data']['label']<native.META]
        if reference is None:reference=ordinary
        assert ordinary==reference
        r.pop('native');r.update(observed=observed,arm=arm,reward=float(result['reward']));rows.append(r);phases[phase]=phases.get(phase,0)+1
    assert phases==dict(baseline=432,training=864,final=432,control=54)
    policies=verify_transitions(a.load(directory/'transitions.json'),[r for r in rows if r['phase']=='training'])
    comparisons,passed=verify_comparisons(a.load(directory/'comparisons.json'),policies,rows)
    assert type(m['learning_criterion_pass']) is bool and m['learning_criterion_pass']==passed
    return dict(accounting_pass=True,profile=PROFILE,cells=1782,native_steps_checked=42768,completed_updates=72,pre_emission_decisions_checked=864,
        phase_counts=phases,novel_fires=fires,paid_edits=edits,learning_criterion_pass=passed,comparisons=comparisons,
        training_admitted=False,full_training_authorized=False,scientific_acceptance=False,independent_physics_replay=False)


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('directory',type=Path)
    print(json.dumps(audit(p.parse_args().directory),indent=2))
