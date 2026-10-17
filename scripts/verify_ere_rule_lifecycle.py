#!/usr/bin/env python3
"""Independent tiny-fixture receipt reconstruction, not Engine/RNG replay."""
import argparse
from collections import Counter
import copy
import itertools
from pathlib import Path
import struct

import audit_ere_complete_admission as integrity
from verify_ere_complete_witness import audit_inventory, check_complete_material, node_key, property_value

PROFILE = 'E2-rule-lifecycle-fixture-v1'
SEEDS = [993748713473, 993748713474, 993748713475]
ARMS = ['live_paid','live_replacement','frozen_paid','live_unpaid','live_disabled','preseeded_lure']
META = 0xFFF00000
TOKEN, CLOCK, TARGET = META+0x7600, 0x7600, 0x7601
OWNS = META+0x120


def candidate(core, label):
    c = dict(labels=[label,11,12], lhs=[[0,1],[1,2]] if core else [[0,1],[0,2]],
             rhs=[[0,1],[0,2]] if core else [[0,1],[1,2]], priority=int(not core))
    c['execution'] = encode(c, (0,1,2), False).hex()
    c['rcf'] = min(encode(c,p,True) for p in itertools.permutations(range(3))).hex()
    structural = dict(c,priority=0)
    c['structural_rcf'] = min(encode(structural,p,True) for p in itertools.permutations(range(3))).hex()
    return c


def encode(c, permutation, canonical):
    out = bytearray(b'E1RCF2'+struct.pack('<i',c['priority'])+bytes([3]))
    labels = [None]*3
    for old,new in enumerate(permutation): labels[new] = c['labels'][old]
    for label in labels: out.extend(bytes([label is not None])+struct.pack('<I',label or 0))
    for side in ('lhs','rhs'):
        edges = [[permutation[v] for v in edge] for edge in c[side]]
        if canonical: edges.sort()
        out.extend(struct.pack('<I',len(edges)))
        for edge in edges: out.extend(bytes([len(edge),*edge]))
    out.extend(struct.pack('<I',0))  # This fixture has no world property guards.
    return bytes(out)


def spells(rows, field, identity):
    result=[]; present=False
    for i,row in enumerate(rows):
        now=identity in {c['rcf'] for c in row[field]}
        if now and not present:result.append(dict(start=i,end=None))
        if not now and present:result[-1]['end']=i
        present=now
    return result


def summary(initial, rows):
    baseline={c['rcf'] for c in initial}; structural={c['structural_rcf'] for c in initial}
    novel=sorted({c['rcf'] for row in rows for c in row['genotype']
                  if c['rcf'] not in baseline and c['structural_rcf'] not in structural})
    classes=[]
    for identity in novel:
        gs=spells(rows,'genotype',identity);ins=spells(rows,'installed',identity)
        fires=[r['step'] for r in rows[1:] for e in r['events'] if e['kind']=='world' and e['rcf']==identity]
        classes.append(dict(rcf=identity,genotype_spells=gs,installed_spells=ins,
            first_birth=gs[0]['start'] if gs else None,first_installation=ins[0]['start'] if ins else None,
            first_non_meta_fire=fires[0] if fires else None,non_meta_fire_steps=fires,
            nontrivial=bool(fires),censor_boundary=24))
    edits=[row['step'] for row in rows[1:] if row['paid_edits']]
    interval=sum(b-a for a,b in zip(edits,edits[1:]))/(len(edits)-1) if len(edits)>1 else None
    return dict(novel_classes=classes,novel_class_count=len(classes),paid_edit_steps=edits,
        mean_edit_interarrival=interval,replacement_interval_unmeasured=interval is None,
        class_half_life=None,half_life_unmeasured=True,sustained_claim=False,
        scientific_acceptance=False,training_admitted=False)


def raw_editor(name, names, labels, edges, guards, effects, enabled=True):
    pattern=dict(nodes=names,edges=edges,node_labels=labels,edge_labels={'0':OWNS} if edges else {})
    return dict(name=name,priority=100 if len(names)==1 else 200,weight=1.,enabled=enabled,
        lhs=pattern,rhs=dict(nodes=names,edges=edges,node_labels={},edge_labels=pattern['edge_labels']),
        lhs_node_property_guards=[dict(var=v,property=k,op=op,value=value) for v,k,op,value in guards],
        lhs_node_property_compares=[],lhs_node_degree_compares=[],lhs_node_quantity_compares=[],
        lhs_var_guards=[],rhs_node_labels={},
        rhs_property_effects=[dict(var=v,key=k,op={op:dict(value=value)}) for v,k,op,value in effects],schedule=None)


def editors(arm):
    result=[raw_editor('lifecycle_seeded_tick',['token'],{'token':TOKEN},[],
        [('token',CLOCK,'Lt',24)],[('token',CLOCK,'Add',1)])]
    for clock,before,after in [(0,99,10),(8,10,99),(16,99,10)]:
        result.append(raw_editor(f'lifecycle_seeded_edit_{clock}',['root','literal','token'],
            dict(root=META+1,literal=META+6,token=TOKEN),[['root','literal']],
            [('root',8,'Eq',10),('literal',TARGET,'Eq',1),('literal',0,'Eq',before),
             ('token',CLOCK,'Eq',clock),('token',1,'Gte',1.),('token',1,'Lte',3.)],
            [('literal',0,'Set',after),('token',1,'Subtract',1.),('token',CLOCK,'Add',1)],
            enabled=arm!='live_disabled' and (clock==0 or arm=='live_replacement')))
    return result


def tagged(value):
    assert type(value) in (int,float)
    return {'Int' if type(value) is int else 'Float':value}


def compiled_editor(raw):
    names=raw['lhs']['nodes']; n=len(names)
    return dict(meta_visible=True,name=raw['name'],priority=raw['priority'],weight=1.,enabled=raw['enabled'],schedule=None,
        lhs_var_names=names,
        pattern=dict(node_label_ceiling=None,match_work_limit=None,match_result_limit=None,n_vars=n,
            edges=[dict(var_ixs=[names.index(v) for v in edge],label=OWNS,ordered=True) for edge in raw['lhs']['edges']],
            node_label_constraints=[raw['lhs']['node_labels'][name] for name in names],
            property_guards=[dict(var_ix=names.index(g['var']),key=g['property'],op=g['op'],value=tagged(g['value']))
                             for g in raw['lhs_node_property_guards']],
            property_compares=[],degree_compares=[],quantity_compares=[],distinct_pairs=[]),
        template=dict(n_rhs_vars=n,rhs_from_lhs=list(range(n)),rhs_new_labels=[0]*n,rhs_relabels=[None]*n,
            rhs_edges=[[[names.index(v) for v in edge],OWNS] for edge in raw['rhs']['edges']],lhs_deleted=[],
            property_effects=[dict(target_ix=names.index(e['var']),key=e['key'],op={next(iter(e['op'])):
                dict(value=tagged(next(iter(e['op'].values()))['value']))}) for e in raw['rhs_property_effects']]))


def edge_key(identity):return f'EdgeId({identity["idx"]}v{identity["version"]})'
def logical(edges):return Counter((e['label'],tuple(sorted(node_key(n) for n in e['nodes']))) for e in edges)
def set_property(data, key, value):
    rows=[row for row in data['properties'] if row[0]==key];assert len(rows)==1
    rows[0][1]=value


def check_budget(usage, arm):
    ids=[0,1]+[2+i for i,r in enumerate(editors(arm)) if r['enabled']]
    assert [r['rule_id'] for r in usage['rules']]==ids and usage['truncated'] is False
    work=results=0
    for rule in usage['rules']:
        assert rule['work_allowance']==50000-work and rule['result_allowance']==50000-results
        assert rule['truncated'] is False and 0<=rule['work']<=rule['work_allowance']
        assert 0<=rule['results']<=rule['result_allowance']
        work+=rule['work'];results+=rule['results']
    assert usage['work']==work and usage['results']==results


def audit_receipt(r):
    arm=r['arm'];assert arm in ARMS and r['seed'] in SEEDS
    assert r['profile']==PROFILE and r['classification']=='scripted_engineering_fixture'
    assert r['training_admitted'] is False and r['scientific_acceptance'] is False
    assert r['independent_physics_replay'] is False and r['horizon']==r['completed_steps']==24
    rows=r['timeline'];assert len(rows)==25 and [x['step'] for x in rows]==list(range(25))
    root,literal,token=[node_key(r[k]) for k in ('root','target_literal','token')]
    world=[node_key(n) for n in r['ordinary_nodes']];assert len(world)==len(set(world))==3
    fuel=0. if arm=='live_unpaid' else 3. if arm=='live_replacement' else 1.
    assert type(r['initial_fuel']) is float and r['initial_fuel']==fuel
    label=10 if arm=='preseeded_lure' else 99
    initial=[candidate(True,10),candidate(False,label)]
    assert r['initial_all_candidates']==initial
    assert r['seeded_editor_rules']==editors(arm)
    assert r['seeded_editor_compiled']==[compiled_editor(raw) for raw in editors(arm)]
    for which in ('initial_inventory','final_inventory'):
        audit_inventory({'inventory':r[which]})
        check_complete_material({'inventory':r[which],'root':r['root']},True)
    nodes={k:copy.deepcopy(v['data']) for k,v in r['initial_inventory']['nodes']}
    edges=dict(r['initial_inventory']['edges'])
    assert [nodes[n]['label'] for n in world]==[10,11,12]
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
                installed=[candidate(True,10),candidate(False,previous_literal)]
            assert previous['installed']==installed
            expected_edit=(arm not in ('live_unpaid','live_disabled','preseeded_lure') and step==1) or (
                arm=='live_replacement' and step in (9,17))
            expected_label=(10 if step==1 or step==17 else 99) if expected_edit else previous_literal
            assert row['paid_edits']==int(expected_edit)
            if expected_edit:fuel-=1.;edit_count+=1
            assert row['state']['literal']==expected_label
            check_budget(row['usage'],arm)
            assert row['rng_before']==row['rng_after']
            assert row['compile_boundary']==step-1
            report=row['compile_report']
            due=arm!='frozen_paid' and step>1 and (step-1)%2==0
            assert (report is not None)==due
            if report:
                assert report['step']==step-1 and report['quarantined']==0
                changed=step in ([3,11,19] if arm=='live_replacement' else [3] if arm=='live_paid' else [])
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
            expected_world = before_phase=='path' or installed[1]['labels'][0]==10
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
                    ei=1+[1,9,17].index(step) if expected_edit else 0
                    assert rid==2+ei and event['rule_name']==editors(arm)[ei]['name']
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
        assert row['genotype']==[candidate(True,10),candidate(False,label)]
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
    expected_novel=int(arm in ('live_paid','live_replacement','frozen_paid'))
    assert r['summary']['novel_class_count']==expected_novel
    if expected_novel:
        novel=r['summary']['novel_classes'][0]
        assert novel['first_birth']==1
        assert novel['non_meta_fire_steps']==([] if arm=='frozen_paid' else
            [3,5,7,9,19,21,23] if arm=='live_replacement' else list(range(3,24,2)))
    else:
        assert world_counts[candidate(False,10)['rcf']]==(12 if arm=='preseeded_lure' else 0)
    return dict(arm=arm,seed=r['seed'],native_steps=24,paid_edits=edit_count,
                novel_classes=expected_novel,non_meta_novel_fires=sum(len(c['non_meta_fire_steps']) for c in r['summary']['novel_classes']))


def audit_panel(directory):
    directory=Path(directory);m=integrity.load(directory/'manifest.json')
    assert m['status']=='native_complete' and m['recorded_cells']==18 and m['failures']==[]
    design=m['design'];assert design['profile']==PROFILE and design['seeds']==SEEDS and design['arms']==ARMS
    assert design['horizon']==24 and design['maximum_native_attempts']==432
    expected={f'{seed}-{arm}.json' for seed in SEEDS for arm in ARMS}
    assert {p.name for p in directory.glob('*.json')}==expected|{'manifest.json','native_assertions.json'}
    results=[]
    for seed in SEEDS:
        for arm in ARMS:
            row=integrity.load(directory/f'{seed}-{arm}.json');assert row['seed']==seed and row['arm']==arm
            results.append(audit_receipt(row))
    return dict(accounting_pass=True,cells=18,native_steps_checked=432,results=results,
        seeded_editor_motif=True,independent_physics_replay=False,training_admitted=False,scientific_acceptance=False)


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('directory',type=Path)
    args=parser.parse_args();print(__import__('json').dumps(audit_panel(args.directory),indent=2))
