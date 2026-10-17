#!/usr/bin/env python3
"""Frozen-policy native confirmation accounting; independent probabilities/ledgers."""
import argparse
import json
import math
from pathlib import Path
import verify_ere_graph_encoder_exact as exact

a=exact.a;e=exact.e;p=exact.prior;close=exact.close
PROFILE='E2-graph-encoder-native-confirmation-v1'
SOURCE='ere_graph_encoder_exact_gradient_20261007'
SEAL='4fbeff3d0fdd8c1579bd709530610c4c11efd122347692c5b43e9b9c874521e5'
POLICIES=['learned_graph','trained_blind','initial_graph','learned_graph_masked']
DRAW_SEED=993748722001
DESIGN=dict(profile=PROFILE,source_archive=SOURCE,source_root_seal=SEAL,source_entries=445,checkpoint=100,policies=POLICIES,
    contexts=p.native.CONTEXTS,native_seeds=p.native.SEEDS,policy_draw_seed=DRAW_SEED,matrix_cells=288,control_cells=36,
    decision_cells=72,total_cells=396,horizon=24,maximum_native_attempts=9504,native_deadline_seconds=180,output_cap_bytes=134217728,
    optimizer_updates=0,minimum_graph_reward=1/24+0.01,maximum_trained_blind_reward=1/24+2e-12,minimum_graph_control_gap=0.01,
    minimum_reserved_gain=0.01,require_every_reserved_probability_gain=True,require_each_native_seed=True,
    sampled_outcomes_are_descriptive=True,fresh_execution_on_exposed_worlds=True,training_admitted=False,scientific_acceptance=False,
    sustained_claim=False,independent_physics_replay=False)


def imports(out):
    assert a.digest(out/'source_output_sha256.json')==SEAL
    seal=a.load(out/'source_output_sha256.json');assert len(seal)==445
    assert a.digest(out/'checkpoint_import.json')==seal['checkpoints.json']
    assert a.digest(out/'matrix_import.json')==seal['input.json']
    c=a.load(out/'checkpoint_import.json');assert [r['checkpoint'] for r in c]==[12,100]
    assert c[1]['diagnostic_criterion_pass'] is True
    data=a.load(out/'matrix_import.json');assert [w['context'] for w in data['worlds']]==p.native.CONTEXTS
    provenance=dict(source_archive=SOURCE,source_root_seal=SEAL,checkpoint=100,
        imports={name:dict(source=old,sha256=seal[old]) for name,old in [('checkpoint_import.json','checkpoints.json'),('matrix_import.json','input.json')]})
    assert a.load(out/'import_provenance.json')==provenance
    return c,data


def policies(checkpoints):
    assert len(checkpoints)==2 and [c['checkpoint'] for c in checkpoints]==[12,100]
    c=checkpoints[1];g=c['final_graph']['theta'];b=c['final_blind']['theta'];initial=e.initialize()
    for mode,t in [('graph',g),('blind',b)]:e.statistics(t,e.make_input(mode,c['baseline']['contexts'][0]['observation']),derivatives=False)
    return [('graph',g),('blind',b),('graph',initial),('blind',g)]


def schedule():
    for seed in p.native.SEEDS:
        for c in p.native.CONTEXTS:
            for x in range(4):
                for y in range(4):yield 'matrix',seed,c,'live_paid',[x,y],None
    for seed in p.native.SEEDS:
        for c in p.RESERVED:
            actions=[p.native.CHOICES[s].index(c[s]) for s in range(2)]
            for arm in p.native.ARMS[1:]:yield 'control',seed,c,arm,actions,None
    for seed in p.native.SEEDS:
        for c in p.native.CONTEXTS:
            for policy in POLICIES:yield 'decision',seed,c,'live_paid',None,policy


def decision(policy,mode,theta,o,draws,states):
    inp=e.make_input(mode,o);prob=e.statistics(theta,inp,derivatives=False)
    return dict(policy=policy,mode=mode,observation=o,input=inp,theta=theta,draws=draws,draw_states=states,probabilities=prob),e.sample(prob,draws)


def verify_decision(wrapper,policy,mode,theta,o,draws,states):
    d,actions=decision(policy,mode,theta,o,draws,states);close(wrapper['decision'],d);assert wrapper['actions']==actions


def verify_bindings(r,i,phase,seed,c,arm,actions):
    assert set(r)=={'id','phase','native_seed','context','arm','actions','decision','native'}
    assert type(r['id']) is int and r['id']==i and r['phase']==phase and r['native_seed']==seed and r['context']==c and r['arm']==arm
    n=r['native'];assert n['seed']==seed and n['context']==c and n['arm']==arm and n['actions']==r['actions']
    if actions is not None:assert r['actions']==actions


def comparison(seed,policy_vectors,matrix,controls,base,checkpoints,observations):
    assert len(policy_vectors)==4 and len(controls)==18
    endpoints=[p.endpoint(mode,t,matrix,observations) for mode,t in policy_vectors]
    reward=[r['balanced_expected_reward'] for r in endpoints];g=reward[0]
    gain=endpoints[0]['reserved_expected_reward']-endpoints[2]['reserved_expected_reward']
    # Byte-pinned source endpoints may differ from independent fsum arithmetic
    # only within the unchanged2e-12 tolerance, never at threshold decisions.
    close(endpoints[0],checkpoints[1]['final_graph']);close(endpoints[1],checkpoints[1]['final_blind'])
    flags=dict(graph_reward_pass=g>=1/24+0.01,trained_blind_bound_pass=reward[1]<=1/24+2e-12,
        trained_blind_gap_pass=g-reward[1]>=0.01,initial_gap_pass=g-reward[2]>=0.01,same_theta_mask_gap_pass=g-reward[3]>=0.01,
        reserved_gain_pass=gain>=0.01,every_reserved_probability_gain=all(r['correct_joint_probability']>b['correct_joint_probability']
            for r,b in zip(endpoints[0]['contexts'],endpoints[2]['contexts']) if r['reserved']),
        controls_pass=all(c['reward']==(9/24 if c['arm']=='reverse_paid' else 0.) for c in controls),native_matrix_unchanged=matrix==base,
        archived_graph_endpoint_match=True,archived_blind_endpoint_match=True)
    return dict(native_seed=seed,policies=[dict(policy=name,endpoint=end) for name,end in zip(POLICIES,endpoints)],
        graph_minus_trained_blind=g-reward[1],graph_minus_initial=g-reward[2],graph_minus_same_theta_mask=g-reward[3],reserved_gain=gain,
        flags=flags,confirmation_pass=all(flags.values()),controls=controls)


def verify_comparisons(records,vectors,matrices,controls,base,checkpoints,observations):
    assert len(records)==2 and len(matrices)==len(controls)==2
    result=[]
    for i,seed in enumerate(p.native.SEEDS):
        row=comparison(seed,vectors,matrices[i],controls[i],base,checkpoints,observations);close(records[i],row);result.append(row)
    return result,all(r['confirmation_pass'] for r in result)


def audit(out):
    checkpoints,data=imports(out);vectors=policies(checkpoints);base=[r for w in data['worlds'] for r in w['rewards']]
    obs=[e.observe(w['raw']) for w in data['worlds']];directory=out/'native';m=a.load(directory/'manifest.json')
    assert m['profile']==PROFILE and m['status']=='native_complete' and m['design']==DESIGN
    for k,v in dict(recorded_cells=396,native_attempts=9504,advancing_steps=9504,optimizer_updates=0,failures=[],native_work_unknown=False,
        training_admitted=False,scientific_acceptance=False,sustained_claim=False,independent_physics_replay=False).items():assert m[k]==v and type(m[k]) is type(v)
    plan=list(schedule());expected={f'cell-{i:04}.json' for i in range(396)}|{f'decision-{i:04}.json' for i in range(324,396)}|{'manifest.json','comparisons.json','sampled.json','executed_test_binary'}
    assert a.files(directory)==expected
    matrices=[[],[]];controls=[[],[]];sampled=[];state=DRAW_SEED;draws=states=None;fires=edits=0
    for i,(phase,seed,c,arm,actions,policy) in enumerate(plan):
        r=a.load(directory/f'cell-{i:04}.json');verify_bindings(r,i,phase,seed,c,arm,actions);n=r['native']
        result,o=p.audit_native(n);fires+=result['non_meta_novel_fires'];edits+=result['paid_edits'];j=p.native.SEEDS.index(seed)
        assert o==obs[p.native.CONTEXTS.index(c)]
        reward=float(result['reward'])
        if phase=='decision':
            pi=POLICIES.index(policy)
            if pi==0:
                draws=[];states=[]
                for _ in range(2):state,d=p.old.next_draw(state);draws.append(d);states.append(state)
            mode,t=vectors[pi];verify_decision(r,policy,mode,t,o,draws,states)
            assert a.load(directory/f'decision-{i:04}.json')=={k:v for k,v in r.items() if k!='native'}
            assert reward==matrices[j][p.native.CONTEXTS.index(c)*16+r['actions'][0]*4+r['actions'][1]]
            sampled.append(dict(id=i,native_seed=seed,context=c,policy=policy,actions=r['actions'],reward=reward))
        else:
            assert r['decision'] is None and r['actions']==actions
            if phase=='matrix':matrices[j].append(reward)
            else:controls[j].append(dict(context=c,arm=arm,reward=reward))
    comparisons,passed=verify_comparisons(a.load(directory/'comparisons.json'),vectors,matrices,controls,base,checkpoints,obs)
    close(a.load(directory/'sampled.json'),sampled);assert type(m['confirmation_pass']) is bool and m['confirmation_pass']==passed
    samples={name:dict(cells=18,mean_reward=math.fsum(r['reward'] for r in sampled if r['policy']==name)/18,
        positive_cells=sum(r['reward']>0 for r in sampled if r['policy']==name)) for name in POLICIES}
    return dict(accounting_pass=True,profile=PROFILE,cells=396,native_attempts=9504,advancing_steps=9504,pre_emission_decisions_checked=72,
        optimizer_updates=0,paid_edits=edits,non_meta_novel_fires=fires,confirmation_pass=passed,comparisons=comparisons,sampled_outcomes=samples,
        training_admitted=False,scientific_acceptance=False,sustained_claim=False,independent_physics_replay=False)


if __name__=='__main__':
    p_args=argparse.ArgumentParser(description=__doc__);p_args.add_argument('directory',type=Path)
    print(json.dumps(audit(p_args.parse_args().directory),indent=2))
