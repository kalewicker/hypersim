#!/usr/bin/env python3
"""Independent two-literal native graph/credit reconstruction, not Engine replay."""
import argparse
from collections import Counter
import copy
import itertools
import math
from pathlib import Path
import verify_ere_rule_lifecycle as shared
from verify_ere_rule_lifecycle import (integrity, summary, raw_editor, compiled_editor,
    audit_inventory, check_complete_material, node_key, property_value,
    logical, edge_key, set_property, META, OWNS)

PROFILE='E2-rule-composition-v1'
AUDITOR_PROFILE='E2-rule-composition-receipt-auditor-v2'
SEEDS=[993748716001,993748716002]
CONTEXTS=[[a,b] for a in (10,20,30) for b in (11,21,31)]
CHOICES=[[None,10,20,30],[None,11,21,31]]
ARMS=['live_paid','frozen_paid','live_unpaid','live_disabled','preseeded_lure','one_fuel','reverse_paid']
TOKEN,CLOCK,TARGET=META+0x7700,0x7700,0x7701
DESIGN=dict(profile=PROFILE,contexts=CONTEXTS,choices=CHOICES,seeds=SEEDS,arms=ARMS,
    development_combinations=[c for c in CONTEXTS if c[0]+1!=c[1]],
    reserved_update_combinations=[c for c in CONTEXTS if c[0]+1==c[1]],
    edit_steps=[1,5],horizon=24,cells=396,maximum_native_attempts=9504,
    native_deadline_seconds=120,match_work_limit=50000,match_result_limit=50000,
    reward_denominator=24,training_updates=0,scientific_acceptance=False,training_admitted=False)


def candidate(core,labels):
    c=dict(labels=[*labels,12],lhs=[[0,1],[1,2]] if core else [[0,1],[0,2]],
        rhs=[[0,1],[0,2]] if core else [[0,1],[1,2]],priority=int(not core))
    c['execution']=shared.encode(c,(0,1,2),False).hex()
    c['rcf']=min(shared.encode(c,p,True) for p in itertools.permutations(range(3))).hex()
    structural=dict(c,priority=0)
    c['structural_rcf']=min(shared.encode(structural,p,True) for p in itertools.permutations(range(3))).hex()
    return c


def editors(arm,actions):
    result=[raw_editor('composition_seeded_tick',['token'],{'token':TOKEN},[],
        [('token',CLOCK,'Lt',24)],[('token',CLOCK,'Add',1)])]
    order=[1,0] if arm=='reverse_paid' else [0,1]
    for stage,slot in enumerate(order):
        clock=[0,4][stage];before=[99,98][slot];after=CHOICES[slot][actions[slot]] or before
        result.append(raw_editor(f'composition_seeded_edit_{clock}_slot{slot}',
            ['root','literal','token'],dict(root=META+1,literal=META+6,token=TOKEN),[['root','literal']],
            [('root',8,'Eq',10),('literal',TARGET,'Eq',slot+1),('literal',0,'Eq',before),
             ('token',CLOCK,'Eq',clock),('token',1,'Gte',1.),('token',1,'Lte',2.)],
            [('literal',0,'Set',after),('token',1,'Subtract',1.),('token',CLOCK,'Add',1)],
            enabled=arm!='live_disabled' and actions[slot]!=0))
    return result


def check_budget(usage,arm,actions):
    ids=[0,1]+[2+i for i,r in enumerate(editors(arm,actions)) if r['enabled']]
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
            for actions in itertools.product(range(4),repeat=2):yield seed,context,'live_paid',list(actions)
            correct=[CHOICES[i].index(x) for i,x in enumerate(context)]
            for arm in ARMS[1:]:yield seed,context,arm,correct


def filename(seed,context,arm,actions):return f'{seed}-{context[0]}-{context[1]}-{arm}-{actions[0]}-{actions[1]}.json'

def audit_receipt(r):
    arm=r['arm'];assert arm in ARMS and r['seed'] in SEEDS
    context=r['context'];actions=r['actions']
    assert context in CONTEXTS and len(context)==2 and all(type(x) is int for x in context)
    assert type(actions) is list and len(actions)==2 and all(type(x) is int and 0<=x<4 for x in actions)
    order=[1,0] if arm=='reverse_paid' else [0,1]
    assert r['profile']==PROFILE and r['classification']=='scripted_engineering_fixture'
    assert r['training_admitted'] is False and r['scientific_acceptance'] is False
    assert r['independent_physics_replay'] is False and r['horizon']==r['completed_steps']==24
    rows=r['timeline'];assert len(rows)==25 and [x['step'] for x in rows]==list(range(25))
    root,token=[node_key(r[k]) for k in ('root','token')]
    literals=[node_key(n) for n in r['target_literals']]
    assert len(literals)==len(set(literals))==2 and root not in literals and token not in literals
    world=[node_key(n) for n in r['ordinary_nodes']];assert len(world)==len(set(world))==3
    fuel=0. if arm=='live_unpaid' else 1. if arm=='one_fuel' else 2.
    assert type(r['initial_fuel']) is float and r['initial_fuel']==fuel
    labels=context.copy() if arm=='preseeded_lure' else [99,98]
    initial=[candidate(True,context),candidate(False,labels)]
    assert r['initial_all_candidates']==initial
    assert r['seeded_editor_rules']==editors(arm,actions)
    assert r['seeded_editor_compiled']==[compiled_editor(raw) for raw in editors(arm,actions)]
    for which in ('initial_inventory','final_inventory'):
        audit_inventory({'inventory':r[which]})
        check_complete_material({'inventory':r[which],'root':r['root']},True)
    nodes={k:copy.deepcopy(v['data']) for k,v in r['initial_inventory']['nodes']}
    edges=dict(r['initial_inventory']['edges'])
    assert [nodes[n]['label'] for n in world]==[context[0],context[1],12]
    assert {n for n,data in nodes.items() if data['label']<META}==set(world)
    assert nodes[token]['label']==TOKEN
    for slot,literal in enumerate(literals):
        assert nodes[literal]['label']==META+6
        assert property_value(nodes[literal],TARGET)=={'Int':slot+1}
        assert property_value(nodes[literal],0)=={'Int':labels[slot]}
    assert property_value(nodes[token],CLOCK)=={'Int':0}
    assert property_value(nodes[token],1)=={'Float':fuel}
    # Establish both typed literal targets belong to the proposal, not the core.
    def link(role,start,slot):
        es=[e for e in edges.values() if e['label']==role and node_key(e['nodes'][0])==start]
        assert len(es)==1
        return node_key(es[0]['nodes'][slot])
    allele=link(META+0x43,root,1);left=link(META+0x40,allele,1)
    right=link(META+0x40,allele,2)
    def side_tuples(side):
        patterns=[node_key(e['nodes'][1]) for e in edges.values()
                  if e['label']==META+0x41 and node_key(e['nodes'][0])==side]
        assert len(patterns)==len(set(patterns))==2
        assert all(nodes[p]['label']==META+4 for p in patterns)
        return [(link(META+0x50,p,1),link(META+0x51,p,1)) for p in patterns]
    lhs,rhs=side_tuples(left),side_tuples(right)
    left0={pair[0] for pair in lhs};left1={pair[1] for pair in lhs}
    right0={pair[0] for pair in rhs};right1={pair[1] for pair in rhs}
    assert len(left0)==1 and len(left1)==2
    middle=left1 & right0
    assert len(middle)==1
    v0=next(iter(left0));v1=next(iter(middle))
    last=left1-{v1};assert len(last)==1
    v2=next(iter(last));assert len({v0,v1,v2})==3
    assert Counter(lhs)==Counter([(v0,v1),(v0,v2)])
    assert Counter(rhs)==Counter([(v0,v1),(v1,v2)])
    assert right0=={v0,v1} and right1=={v1,v2}
    for slot,variable in enumerate([v0,v1,v2]):
        assert nodes[variable]['label']==META+5
        label_node=link(META+0x70,variable,1)
        assert nodes[label_node]['label']==META+6
        assert property_value(nodes[label_node],0)=={'Int':[*labels,12][slot]}
        if slot<2:assert label_node==literals[slot]
    installed=initial
    world_counts=Counter();edit_count=0;edit_steps=[]
    for step,row in enumerate(rows):
        if step:
            previous=rows[step-1]
            previous_literals=previous['state']['literals']
            previous_graph=copy.deepcopy(edges)
            if arm!='frozen_paid' and (step-1)%2==0:
                installed=[candidate(True,context),candidate(False,previous_literals)]
            assert previous['installed']==installed
            slot=order[0 if step==1 else 1]
            expected_edit=step in (1,5) and arm not in ('preseeded_lure','live_disabled') and actions[slot]!=0 and fuel>=1.
            expected_labels=previous_literals.copy()
            if expected_edit:expected_labels[slot]=CHOICES[slot][actions[slot]]
            assert row['paid_edits']==int(expected_edit)
            if expected_edit:fuel-=1.;edit_count+=1;edit_steps.append(step)
            assert row['state']['literals']==expected_labels
            check_budget(row['usage'],arm,actions)
            assert row['rng_before']==row['rng_after']
            assert row['compile_boundary']==step-1
            report=row['compile_report']
            due=arm!='frozen_paid' and step>1 and (step-1)%2==0
            assert (report is not None)==due
            if report:
                assert report['step']==step-1 and report['quarantined']==0
                changed=step-2 in edit_steps
                assert report['changed'] is changed
                assert report['installed']==report['retired']==int(changed)
            changes={node_key(n['id']):n for n in row['changed_nodes']}
            assert len(changes)==len(row['changed_nodes'])
            assert set(changes)==({token,literals[slot]} if expected_edit else {token})
            expected_nodes={token:copy.deepcopy(nodes[token])}
            set_property(expected_nodes[token],CLOCK,{'Int':step})
            set_property(expected_nodes[token],1,{'Float':fuel})
            if expected_edit:
                literal=literals[slot]
                expected_nodes[literal]=copy.deepcopy(nodes[literal]);set_property(expected_nodes[literal],0,{'Int':expected_labels[slot]})
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
            expected_world = before_phase=='path' or installed[1]['labels'][:2]==context
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
                    ei=(1 if step==1 else 2) if expected_edit else 0
                    assert rid==2+ei and event['rule_name']==editors(arm,actions)[ei]['name']
                    assert bound==([root,literals[slot],token] if expected_edit else [token])
                    assert len(removed)==len(created)==int(expected_edit)
                    if expected_edit:
                        assert removed[0]['edge']==created[0]['edge']==dict(label=OWNS,nodes=[r['root'],r['target_literals'][slot]])
                for edge in removed:
                    key=edge_key(edge['id']);assert edges.pop(key)==edge['edge'];deleted+=1
                for edge in created:
                    key=edge_key(edge['id']);assert key not in edges;edges[key]=edge['edge'];added+=1
            assert row['stats']['edges_deleted']==deleted and row['stats']['edges_created']==added
            labels=expected_labels
        else:
            assert row['events']==[] and row['paid_edits']==0
        assert row['genotype']==[candidate(True,context),candidate(False,labels)]
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
    expected_novel=len(edit_steps)
    assert r['summary']['novel_class_count']==expected_novel
    classes=r['summary']['novel_classes']
    assert sorted(c['first_birth'] for c in classes)==edit_steps
    productive=arm in ('live_paid','reverse_paid') and labels==context and len(edit_steps)==2
    for c in classes:
        birth=c['first_birth'];later=[s for s in edit_steps if s>birth]
        end=later[0] if later else None
        assert c['genotype_spells']==[dict(start=birth,end=end)]
        assert c['installed_spells']==([] if arm=='frozen_paid' else [dict(start=birth+1,end=end+1 if end is not None else None)])
        assert c['non_meta_fire_steps']==(list(range(7,24,2)) if productive and birth==5 else [])
    if arm=='preseeded_lure':
        assert world_counts[candidate(False,context)['rcf']]==12 and not classes
    fires=sum(len(c['non_meta_fire_steps']) for c in r['summary']['novel_classes'])
    assert r['adapter_reward']==fires/24
    return dict(arm=arm,context=context,actions=actions,reward=fires/24,seed=r['seed'],native_steps=24,paid_edits=edit_count,world_fires=sum(world_counts.values()),
                novel_classes=expected_novel,non_meta_novel_fires=sum(len(c['non_meta_fire_steps']) for c in r['summary']['novel_classes']))


def audit_policy(receipt):
    assert receipt['training_updates']==0 and receipt['training_admitted'] is False
    theta=receipt['theta'];assert len(theta)==12 and all(type(x) is float and math.isfinite(x) for x in theta)
    assert len(receipt['contexts'])==9
    for context,r in zip(CONTEXTS,receipt['contexts']):
        assert r['context']==context
        probabilities=[]
        for label in [context[0],context[1]-1]:
            offset=[10,20,30].index(label)*4;logits=theta[offset:offset+4];maximum=max(logits)
            weights=[math.exp(x-maximum) for x in logits];total=math.fsum(weights)
            probabilities.append([x/total for x in weights])
        assert len(r['probabilities'])==2
        for actual,expected in zip(r['probabilities'],probabilities):
            assert len(actual)==4 and all(abs(x-y)<1e-14 for x,y in zip(actual,expected))
        assert len(r['actions'])==16
        for actions,entry in zip(itertools.product(range(4),repeat=2),r['actions']):
            assert entry['actions']==entry['sample_midpoints']==list(actions)
            likelihood=0.;gradient=[0.]*12
            for slot,label in enumerate([context[0],context[1]-1]):
                offset=[10,20,30].index(label)*4;logits=theta[offset:offset+4];maximum=max(logits)
                likelihood+=logits[actions[slot]]-maximum-math.log(math.fsum(math.exp(x-maximum) for x in logits))
                for j in range(4):gradient[offset+j]+=int(j==actions[slot])-probabilities[slot][j]
            assert abs(entry['log_probability']-likelihood)<1e-13
            assert len(entry['score_gradient'])==12 and all(abs(x-y)<1e-14 for x,y in zip(entry['score_gradient'],gradient))


def audit_panel(directory):
    directory=Path(directory);m=integrity.load(directory/'manifest.json')
    assert m['status']=='native_complete' and m['recorded_cells']==396 and m['failures']==[]
    assert m['design']==DESIGN
    expected={filename(*cell) for cell in schedule()}
    assert {p.name for p in directory.glob('*.json')}==expected|{'manifest.json','native_assertions.json','policy.json'}
    assertions=integrity.load(directory/'native_assertions.json')
    assert assertions==dict(passed=True,cells=396,native_attempts=9504,advancing_steps=9504,
        training_updates=0,scientific_acceptance=False)
    audit_policy(integrity.load(directory/'policy.json'))
    results=[]
    for seed,context,arm,actions in schedule():
        r=integrity.load(directory/filename(seed,context,arm,actions))
        assert (r['seed'],r['context'],r['arm'],r['actions'])==(seed,context,arm,actions)
        results.append(audit_receipt(r))
    return dict(accounting_pass=True,auditor_profile=AUDITOR_PROFILE,new_native_steps=0,cells=396,native_steps_checked=9504,results=results,
        novel_fires=sum(r['non_meta_novel_fires'] for r in results),training_updates=0,
        independent_physics_replay=False,training_admitted=False,scientific_acceptance=False)


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('directory',type=Path)
    args=parser.parse_args();print(__import__('json').dumps(audit_panel(args.directory),indent=2))
