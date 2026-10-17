#!/usr/bin/env python3
"""Independent contextual paid-choice graph/reward accounting; no physics replay."""
import argparse
from collections import Counter
import copy
import math
from pathlib import Path
import verify_ere_rule_lifecycle as shared
from verify_ere_rule_lifecycle import (integrity, candidate, summary, raw_editor,
    compiled_editor, audit_inventory, check_complete_material, node_key,
    property_value, logical, edge_key, set_property, META, TOKEN, CLOCK, TARGET, OWNS)

PROFILE='E2-rule-choice-adapter-v1'
SEEDS=[993748714001,993748714002]
CONTEXTS=[10,20,30]
ACTIONS=[None,10,20,30]
ARMS=['live_paid','frozen_paid','live_unpaid','live_disabled','preseeded_lure']


def editors(arm,action):
    result=shared.editors(arm)
    result[1]['rhs_property_effects'][0]['op']['Set']['value']=ACTIONS[action] or 99
    for i,r in enumerate(result[1:],1):
        r['enabled']=arm!='live_disabled' and action!=0 and i==1
    return result


def check_budget(usage,arm,action):
    ids=[0,1]+[2+i for i,r in enumerate(editors(arm,action)) if r['enabled']]
    assert [r['rule_id'] for r in usage['rules']]==ids and usage['truncated'] is False
    work=results=0
    for rule in usage['rules']:
        assert rule['work_allowance']==50000-work and rule['result_allowance']==50000-results
        assert rule['truncated'] is False and 0<=rule['work']<=rule['work_allowance']
        assert 0<=rule['results']<=rule['result_allowance']
        work+=rule['work'];results+=rule['results']
    assert usage['work']==work and usage['results']==results


def schedule():
    for seed in SEEDS:
        for context in CONTEXTS:
            for action in range(4):yield seed,context,'live_paid',action
            for arm in ARMS[1:]:yield seed,context,arm,ACTIONS.index(context)


def filename(seed,context,arm,action):return f'{seed}-{context}-{arm}-{action}.json'

def audit_receipt(r):
    arm=r['arm'];assert arm in ARMS and r['seed'] in SEEDS
    context=r['context'];action=r['action']
    assert type(context) is int and context in CONTEXTS and type(action) is int and 0<=action<4
    choice=ACTIONS[action]
    paid=arm in ('live_paid','frozen_paid') and action!=0
    assert r['profile']==PROFILE and r['classification']=='scripted_engineering_fixture'
    assert r['training_admitted'] is False and r['scientific_acceptance'] is False
    assert r['independent_physics_replay'] is False and r['horizon']==r['completed_steps']==24
    rows=r['timeline'];assert len(rows)==25 and [x['step'] for x in rows]==list(range(25))
    root,literal,token=[node_key(r[k]) for k in ('root','target_literal','token')]
    world=[node_key(n) for n in r['ordinary_nodes']];assert len(world)==len(set(world))==3
    fuel=0. if arm=='live_unpaid' else 1.
    assert type(r['initial_fuel']) is float and r['initial_fuel']==fuel
    label=context if arm=='preseeded_lure' else 99
    initial=[candidate(True,context),candidate(False,label)]
    assert r['initial_all_candidates']==initial
    assert r['seeded_editor_rules']==editors(arm,action)
    assert r['seeded_editor_compiled']==[compiled_editor(raw) for raw in editors(arm,action)]
    for which in ('initial_inventory','final_inventory'):
        audit_inventory({'inventory':r[which]})
        check_complete_material({'inventory':r[which],'root':r['root']},True)
    nodes={k:copy.deepcopy(v['data']) for k,v in r['initial_inventory']['nodes']}
    edges=dict(r['initial_inventory']['edges'])
    assert [nodes[n]['label'] for n in world]==[context,11,12]
    assert {n for n,data in nodes.items() if data['label']<META}==set(world)
    assert nodes[token]['label']==TOKEN and nodes[literal]['label']==META+6
    assert property_value(nodes[literal],TARGET)=={'Int':1}
    assert property_value(nodes[literal],0)=={'Int':label}
    assert property_value(nodes[token],CLOCK)=={'Int':0}
    assert property_value(nodes[token],1)=={'Float':fuel}
    # Establish that the marked literal belongs to proposal variable0, not core.
    def link(role,start,slot):
        es=[e for e in edges.values() if e['label']==role and node_key(e['nodes'][0])==start]
        assert len(es)==1
        return node_key(es[0]['nodes'][slot])
    allele=link(META+0x43,root,1);left=link(META+0x40,allele,1)
    members=[e for e in edges.values() if e['label']==META+0x41 and node_key(e['nodes'][0])==left]
    vars0={link(META+0x50,node_key(e['nodes'][1]),1) for e in members}
    assert len(vars0)==1 and link(META+0x70,next(iter(vars0)),1)==literal
    installed=initial
    world_counts=Counter();edit_count=0
    for step,row in enumerate(rows):
        if step:
            previous=rows[step-1]
            previous_literal=previous['state']['literal']
            previous_graph=copy.deepcopy(edges)
            if arm!='frozen_paid' and (step-1)%2==0:
                installed=[candidate(True,context),candidate(False,previous_literal)]
            assert previous['installed']==installed
            expected_edit=paid and step==1
            expected_label=choice if expected_edit else previous_literal
            assert row['paid_edits']==int(expected_edit)
            if expected_edit:fuel-=1.;edit_count+=1
            assert row['state']['literal']==expected_label
            check_budget(row['usage'],arm,action)
            assert row['rng_before']==row['rng_after']
            assert row['compile_boundary']==step-1
            report=row['compile_report']
            due=arm!='frozen_paid' and step>1 and (step-1)%2==0
            assert (report is not None)==due
            if report:
                assert report['step']==step-1 and report['quarantined']==0
                changed=step==3 and paid and arm=='live_paid'
                assert report['changed'] is changed
                assert report['installed']==report['retired']==int(changed)
            changes={node_key(n['id']):n for n in row['changed_nodes']}
            assert len(changes)==len(row['changed_nodes'])
            assert set(changes)==({token,literal} if expected_edit else {token})
            expected_nodes={token:copy.deepcopy(nodes[token])}
            set_property(expected_nodes[token],CLOCK,{'Int':step})
            set_property(expected_nodes[token],1,{'Float':fuel})
            if expected_edit:
                expected_nodes[literal]=copy.deepcopy(nodes[literal]);set_property(expected_nodes[literal],0,{'Int':expected_label})
            for key,change in changes.items():
                assert change['before']==nodes[key] and change['after']==expected_nodes[key]
                nodes[key]=change['after']
            events=row['events'];assert row['stats']['accepted_rewrites']==len(events)
            assert row['stats']['nodes_created']==row['stats']['nodes_deleted']==0
            assert row['stats']['property_effects_skipped']==0
            assert row['stats']['property_effects_applied']==(3 if expected_edit else 1)
            assert row['stats']['step']==step
            ordinary_before=[e for e in previous_graph.values() if all(node_key(n) in world for n in e['nodes'])]
            def motif(which):
                return Counter([(0,tuple(sorted((world[0],world[1])))),
                                (0,tuple(sorted((world[1] if which=='path' else world[0],world[2]))))])
            before_phase='path' if logical(ordinary_before)==motif('path') else 'star'
            assert logical(ordinary_before)==motif(before_phase)
            expected_world = before_phase=='path' or installed[1]['labels'][0]==context
            assert sum(e['kind']=='world' for e in events)==int(expected_world)
            assert sum(e['kind']=='seeded_editor' for e in events)==1
            deleted=added=0
            for event in events:
                assert event['nodes_created']==event['nodes_deleted']==event['cleanup_neighbors']==[]
                rid=event['rule_id'];bound=[node_key(n) for n in event['bound_nodes']]
                removed=event['edges_deleted'];created=event['edges_created']
                assert event['matched_edges']==[e['id'] for e in removed]
                if event['kind']=='world':
                    assert rid in (0,1) and bound==world and len(set(bound))==3
                    assert event['rcf']==installed[rid]['rcf']
                    assert rid==(0 if before_phase=='path' else 1)
                    assert len(removed)==len(created)==2
                    target='star' if before_phase=='path' else 'path'
                    assert logical([x['edge'] for x in removed])==motif(before_phase)
                    assert logical([x['edge'] for x in created])==motif(target)
                    world_counts[installed[rid]['rcf']]+=1
                else:
                    assert event['rcf'] is None
                    ei=1 if expected_edit else 0
                    assert rid==2+ei and event['rule_name']==editors(arm,action)[ei]['name']
                    assert bound==([root,literal,token] if expected_edit else [token])
                    assert len(removed)==len(created)==int(expected_edit)
                    if expected_edit:
                        assert removed[0]['edge']==created[0]['edge']==dict(label=OWNS,nodes=[r['root'],r['target_literal']])
                for edge in removed:
                    key=edge_key(edge['id']);assert edges.pop(key)==edge['edge'];deleted+=1
                for edge in created:
                    key=edge_key(edge['id']);assert key not in edges;edges[key]=edge['edge'];added+=1
            assert row['stats']['edges_deleted']==deleted and row['stats']['edges_created']==added
            label=expected_label
        else:
            assert row['events']==[] and row['paid_edits']==0
        assert row['genotype']==[candidate(True,context),candidate(False,label)]
        assert row['state']['fuel']==fuel and type(row['state']['fuel']) is float
        assert row['state']['clock']==step and row['state']['root_alive'] is True
        observed={edge_key(x['id']):x['edge'] for x in row['state']['world_edges']}
        actual={k:e for k,e in edges.items() if all(node_key(n) in world for n in e['nodes'])}
        assert observed==actual
    assert rows[-1]['installed']==installed
    final_nodes={k:v['data'] for k,v in r['final_inventory']['nodes']}
    assert final_nodes==nodes and dict(r['final_inventory']['edges'])==edges
    assert r['final_inventory']['step']==24 and r['initial_inventory']['step']==0
    assert r['summary']==summary(initial,rows)
    expected_novel=int(paid)
    assert r['summary']['novel_class_count']==expected_novel
    if expected_novel:
        novel=r['summary']['novel_classes'][0]
        assert novel['first_birth']==1
        assert novel['non_meta_fire_steps']==([] if arm=='frozen_paid' else
            list(range(3,24,2)) if choice==context else [])
    else:
        assert world_counts[candidate(False,context)['rcf']]==(12 if arm=='preseeded_lure' else 0)
    fires=sum(len(c['non_meta_fire_steps']) for c in r['summary']['novel_classes'])
    assert r['adapter_reward']==fires/24
    return dict(arm=arm,context=context,action=action,reward=fires/24,seed=r['seed'],native_steps=24,paid_edits=edit_count,
                novel_classes=expected_novel,non_meta_novel_fires=sum(len(c['non_meta_fire_steps']) for c in r['summary']['novel_classes']))


def audit_policy(receipt):
    assert receipt['training_updates']==0 and receipt['training_admitted'] is False
    theta=receipt['theta'];assert len(theta)==12 and all(type(x) is float and math.isfinite(x) for x in theta)
    assert len(receipt['contexts'])==3
    for i,context in enumerate(CONTEXTS):
        r=receipt['contexts'][i];assert r['context']==context
        logits=theta[4*i:4*i+4];maximum=max(logits)
        weights=[math.exp(x-maximum) for x in logits];total=math.fsum(weights)
        p=[x/total for x in weights]
        assert len(r['probabilities'])==4 and all(abs(a-b)<1e-14 for a,b in zip(p,r['probabilities']))
        assert len(r['actions'])==4
        for action,a in enumerate(r['actions']):
            assert a['action']==action
            assert abs(a['log_probability']-(logits[action]-maximum-math.log(total)))<1e-13
            gradient=[0.]*12
            for j in range(4):gradient[4*i+j]=int(j==action)-p[j]
            assert len(a['score_gradient'])==12
            assert all(abs(x-y)<1e-14 for x,y in zip(gradient,a['score_gradient']))
            assert a['sample_midpoint']==action


def audit_panel(directory):
    directory=Path(directory);m=integrity.load(directory/'manifest.json')
    assert m['status']=='native_complete' and m['recorded_cells']==48 and m['failures']==[]
    design=m['design']
    expected_design=dict(profile=PROFILE,contexts=CONTEXTS,actions=ACTIONS,seeds=SEEDS,arms=ARMS,
        horizon=24,cells=48,maximum_native_attempts=1152,native_deadline_seconds=60,
        match_work_limit=50000,match_result_limit=50000,reward_denominator=24,
        training_updates=0,scientific_acceptance=False,training_admitted=False)
    assert design==expected_design
    expected={filename(*cell) for cell in schedule()}
    assert {p.name for p in directory.glob('*.json')}==expected|{'manifest.json','native_assertions.json','policy.json'}
    assertions=integrity.load(directory/'native_assertions.json')
    assert assertions==dict(passed=True,cells=48,native_attempts=1152,advancing_steps=1152,
                           training_updates=0,scientific_acceptance=False)
    policy=integrity.load(directory/'policy.json');audit_policy(policy)
    results=[]
    for seed,context,arm,action in schedule():
        r=integrity.load(directory/filename(seed,context,arm,action))
        assert (r['seed'],r['context'],r['arm'],r['action'])==(seed,context,arm,action)
        results.append(audit_receipt(r))
    return dict(accounting_pass=True,cells=48,native_steps_checked=1152,results=results,
        novel_fires=sum(r['non_meta_novel_fires'] for r in results),
        seeded_editor_motif=True,independent_physics_replay=False,training_updates=0,
        training_admitted=False,scientific_acceptance=False)


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('directory',type=Path)
    args=parser.parse_args();print(__import__('json').dumps(audit_panel(args.directory),indent=2))
