#!/usr/bin/env python3
"""Independent fixed-panel graph, binding, payment and first-spell audit, not physics replay."""
import argparse
from collections import Counter
import copy
import json
from pathlib import Path
import re
import audit_ere_complete_admission as a
import verify_ere_rule_relational as old
from verify_ere_rule_lifecycle import raw_editor,compiled_editor
from verify_ere_complete_long_gates import check_binding
from verify_ere_complete_witness import ROLES
import verify_ere_graph_encoder as encoder
from verify_ere_rule_choice_learning import next_draw,close
import ere_rule_survival_metrics as survival

PROFILE='E2-rule-maintenance-panel-v1';NATIVE='E2-rule-maintenance-v1'
META=old.META;ROLE=META+0x50;MARKER=0x7702
CONTEXTS=old.CONTEXTS
ARMS=['learned_protected','learned_exposed_repair','learned_exhaustion','learned_unpaid','learned_frozen','learned_no_hazard','oracle_exposed_repair']
FUELS=[8,8,2,0,8,8,8];HAZARDS=[16,32,48,64];SEED=993748718001;DRAW=993748723001
SOURCE='ere_graph_encoder_native_confirmation_20261007'
SOURCE_SEAL='2da2e2889dd687ba613beea610ab9fb93670dced83285e38519d2e4142c2408d'
EXACT_SEAL='4fbeff3d0fdd8c1579bd709530610c4c11efd122347692c5b43e9b9c874521e5'
DESIGN=dict(profile=PROFILE,native_profile=NATIVE,contexts=CONTEXTS,arms=ARMS,repair_fuel=FUELS,
    initial_birth_fuel=2,seal_initialization_cost=1,target_role=ROLE,
    target='proposal left first pattern E_SLOT0, ordered sealed endpoints [pattern,variable,root]',
    native_seed=SEED,draw_seed=DRAW,horizon=96,hazard_steps=HAZARDS,birth_steps=[1,5],reference_exposure_interval=16,
    cells=63,maximum_native_advances=6048,maximum_nonadvancing_seal_applications=63,native_deadline_seconds=180,
    output_limit_bytes=268435456,match_work_limit=50000,match_result_limit=50000,epoch=2,optimizer_updates=0,
    training_admitted=False,scientific_acceptance=False,sustained_claim=False,independent_physics_replay=False)


def compiled(raw,ordered=True,visible=True,ceiling=None):
    c=compiled_editor(raw);c['meta_visible']=visible;c['pattern']['node_label_ceiling']=ceiling
    for i,e in enumerate(c['pattern']['edges']):
        e['label']=raw['lhs']['edge_labels'].get(str(i));e['ordered']=ordered
    for i,e in enumerate(c['template']['rhs_edges']):e[1]=raw['rhs']['edge_labels'].get(str(i),0)
    return c


def editor_catalogue(arm,actions):
    raw=old.editors('live_paid',actions)
    raw[0]['lhs_node_property_guards'][0]['value']=96
    if arm!='learned_no_hazard':
        for step in HAZARDS:
            r=raw_editor(f'maintenance_hazard_{step}',['a','b','root','token'],
                dict(a=META+4,b=META+5,root=META+1,token=old.TOKEN),[['a','b','root']],
                [('a',MARKER,'Eq',1),('b',MARKER,'Eq',2),('token',old.CLOCK,'Eq',step-1)],
                [('token',old.CLOCK,'Add',1)])
            r['priority']=300;r['lhs']['edge_labels']={};r['rhs']['edge_labels']={};raw.append(r)
    return raw,[compiled(r,visible=not (r['name'].startswith('maintenance_hazard') and arm=='learned_protected'),
        ceiling=META if r['name'].startswith('maintenance_hazard') and arm=='learned_protected' else None) for r in raw]


def operation_catalogue(op):
    result=[]
    for ix,role in enumerate(ROLES):
        pairs=[(META+1,META+8)] if ix<2 else [(META+2,META+4),(META+3,META+4)] if ix==2 else [(META+5,META+6)] if ix==11 else [(META+5,META+7)] if ix==12 else [(META+4,META+5)]
        for al,bl in pairs:
            av='root' if ix<2 else 'a';names=['op','root','b'] if ix<2 else ['op','root','a','b']
            r=raw_editor(f'complete_encoded_meta_{op}_{ix}_{al}',names,
                dict(op=META+9,root=META+1,**({} if ix<2 else {'a':al}),b=bl),
                [['root',av,'b'],[av,'b'] if op=='Seal' else [av,'b','root']],
                [('op',8,'Eq',11),('op',10,'Eq',3 if op=='Seal' else 6),('op',1,'Gte',1.),
                 ('op',1,'Lte',32.),('root',8,'Eq',10)],[('op',1,'Subtract',1.)])
            r['priority']=100;r['lhs']['edge_labels']={'0':META+0x100+ix,'1':role if op=='Seal' else 0}
            r['rhs']['edges']=[['root',av,'b'],[av,'b','root']];r['rhs']['edge_labels']={'0':META+0x100+ix,'1':role}
            c=compiled(r);c['pattern']['degree_compares']=[dict(var_ix=0,op='Eq',value=0,offset=None)]
            result.append(c)
    return result


def world_compiled(candidate,name):
    assert re.fullmatch('e1_[0-9a-f]{16}',name)
    names=['v0','v1','v2'];c=candidate
    r=raw_editor(name,names,dict(zip(names,c['labels'])),[[names[i] for i in e] for e in c['lhs']],[],[])
    r['priority']=c['priority'];r['lhs']['edge_labels']={};r['rhs']['edge_labels']={}
    r['rhs']['edges']=[[names[i] for i in e] for e in c['rhs']]
    return compiled(r,ordered=False,visible=False,ceiling=META)


def bounded(table):
    result=copy.deepcopy(table)
    for c in result:c['pattern']['match_work_limit']=c['pattern']['match_result_limit']=50000
    return result


def graph_inventory(nodes,edges,step):
    incident={k:[] for k in nodes};labels={}
    for key,e in edges.items():
        labels.setdefault(e['label'],[]).append(key)
        for n in e['nodes']:incident[old.node_key(n)].append(key)
    return dict(step=step,nodes=sorted([k,dict(data=d,incident=sorted(incident[k]))] for k,d in nodes.items()),
        edges=sorted([k,e] for k,e in edges.items()),label_indexes=sorted([l,sorted(keys)] for l,keys in labels.items()))


def replay(events,table,nodes,edges):
    before_nodes=nodes;before_edges=edges;nodes=dict(nodes);edges=dict(edges);used=set();totals=Counter()
    for event in events:
        rid=event['rule_id'];assert type(rid) is int and 0<=rid<len(table)
        rule=table[rid];assert rule['name']==event['rule_name'] and rule['enabled']
        assert event['nodes_created']==event['nodes_deleted']==event['cleanup_neighbors']==[]
        b=event['bound_nodes'];matched=event['matched_edges'];check_binding(rule,b,matched,before_nodes,before_edges)
        for g in rule['pattern']['degree_compares']:
            assert g==dict(var_ix=0,op='Eq',value=0,offset=None)
            assert not any(old.node_key(b[0]) in map(old.node_key,e['nodes']) for e in before_edges.values())
        keys=set(map(old.node_key,b));assert not keys & used,'MaxParallel bindings overlap';used|=keys
        t=rule['template'];assert t['lhs_deleted']==[] and all(i is not None for i in t['rhs_from_lhs'])
        assert t['rhs_relabels']==[None]*len(b)
        assert event['matched_edges']==[e['id'] for e in event['edges_deleted']]
        for e in event['edges_deleted']:assert edges.pop(old.edge_key(e['id']))==e['edge'];totals['edges_deleted']+=1
        rhs=[b[i] for i in t['rhs_from_lhs']]
        assert len(event['edges_created'])==len(t['rhs_edges'])
        for e,(indices,label) in zip(event['edges_created'],t['rhs_edges']):
            assert e['edge']==dict(label=label,nodes=[rhs[i] for i in indices])
            key=old.edge_key(e['id']);assert key not in edges;edges[key]=e['edge'];totals['edges_created']+=1
        for effect in t['property_effects']:
            key=old.node_key(rhs[effect['target_ix']]);nodes[key]=copy.deepcopy(nodes[key])
            op,value=next(iter(effect['op'].items()));value=value['value'];before=old.property_value(nodes[key],effect['key'])
            assert op in ['Add','Subtract','Set'] and before.keys()==value.keys()
            typ=next(iter(value));after=value if op=='Set' else {typ:before[typ]+(1 if op=='Add' else -1)*value[typ]}
            old.set_property(nodes[key],effect['key'],after);totals['property_effects_applied']+=1
    return nodes,edges,totals


def audit_receipt(r):
    arm=r['arm'];assert arm in ARMS and r['context'] in CONTEXTS and r['seed']==SEED
    actions=r['actions'];assert len(actions)==2 and all(type(x) is int and 0<=x<4 for x in actions)
    assert r['profile']==NATIVE and r['classification']=='scripted_engineering_fixture' and r['maintenance'] is True
    assert r['horizon']==r['completed_steps']==96 and r['initial_fuel']==2.
    for k in ['training_admitted','scientific_acceptance','sustained_claim','independent_physics_replay']:assert r[k] is False
    rows=r['timeline'];assert [x['step'] for x in rows]==list(range(97))
    indices=[row['pre_step_table'] for row in rows[1:]]
    assert all(type(i) is int and 0<=i<len(r['table_catalogue']) for i in indices)
    assert list(dict.fromkeys(indices))==list(range(len(r['table_catalogue'])))
    raw,editors=editor_catalogue(arm,actions);assert r['seeded_editor_rules']==raw and r['seeded_editor_compiled']==editors
    operations=operation_catalogue('Seal')+operation_catalogue('RepairSealed')
    p=r['initialization'];assert p['step']==0 and p['paid_applications']==1 and p['role']==ROLE and p['rng_unchanged'] is True
    assert p['after_inventory']==r['initial_inventory']
    root=r['root'];rk=old.node_key(root);token=old.node_key(r['token']);literals=list(map(old.node_key,r['target_literals']))
    repair=old.node_key(p['repair_instruction']);seal=old.node_key(p['seal_instruction']);target=p['endpoints']
    assert target[2]==root and len(set(map(old.node_key,target)))==3
    for inv in [p['before_inventory'],r['initial_inventory'],r['final_inventory']]:old.audit_inventory(dict(inventory=inv))
    old.check_complete_material(dict(inventory=p['before_inventory'],root=root),True)
    pre=p['before_inventory'];nodes={k:copy.deepcopy(v['data']) for k,v in pre['nodes']};edges=dict(pre['edges'])
    assert old.property_value(nodes[seal],1)=={'Float':1.}
    fuel=float(FUELS[ARMS.index(arm)]);assert old.property_value(nodes[repair],1)=={'Float':fuel}
    for key,opcode in [(seal,3),(repair,6)]:
        assert nodes[key]['label']==META+9 and old.property_value(nodes[key],8)=={'Int':11} and old.property_value(nodes[key],10)=={'Int':opcode}
    # Resolve target and both editable literals from live proposal graph paths.
    def link(role,n,slot):
        es=[e for key,e in sorted(edges.items(),key=lambda item:int(re.fullmatch(r'EdgeId\((\d+)v(\d+)\)',item[0])[1])) if e['label']==role and e['nodes'][0]==n]
        assert len(es)==(2 if role==META+0x41 else 1)
        return es[0]['nodes'][slot]
    allele=link(META+0x43,root,1);left=link(META+0x40,allele,1)
    assert link(META+0x41,left,1)==target[0] and link(ROLE,target[0],1)==target[1]
    for slot in range(2):
        variable=link(ROLE+slot,target[0],1);literal=link(META+0x70,variable,1)
        assert old.node_key(literal)==literals[slot] and old.property_value(nodes[literals[slot]],old.TARGET)=={'Int':slot+1}
    assert old.property_value(nodes[rk],9)=={'Int':1}
    # Decode both initialized graph bodies independently; complete_material
    # validates ownership/witnesses but does not itself decode rule semantics.
    for core,role,labels0 in [(True,META+0x42,r['context']),(False,META+0x43,[99,98])]:
        ar=link(role,root,1);ak=old.node_key(ar)
        assert nodes[ak]['label']==META+8 and old.property_value(nodes[ak],2)=={'Int':int(not core)}
        lhs=link(META+0x40,ar,1);rhs=link(META+0x40,ar,2)
        pairs=[]
        for side in [lhs,rhs]:
            patterns=[e['nodes'][1] for e in edges.values() if e['label']==META+0x41 and e['nodes'][0]==side]
            assert len(patterns)==2
            pairs.append([(old.node_key(link(ROLE,p,1)),old.node_key(link(ROLE+1,p,1))) for p in patterns])
        variables=set(v for ps in pairs for pair in ps for v in pair);assert len(variables)==3
        bylabel={}
        for vk in variables:
            n=next(n for e in edges.values() for n in e['nodes'] if old.node_key(n)==vk)
            lit=old.node_key(link(META+0x70,n,1));label=old.property_value(nodes[lit],0)['Int']
            assert label not in bylabel;bylabel[label]=vk
        expected=old.candidate(core,labels0);assert set(bylabel)==set(expected['labels'])
        for side,ps in zip(['lhs','rhs'],pairs):
            assert Counter(ps)==Counter(tuple(bylabel[expected['labels'][i]] for i in pair) for pair in expected[side])
        assert not any(e['label']==META+0x71 and old.node_key(e['nodes'][0]) in variables for e in edges.values())
    chosen=copy.deepcopy(operation_catalogue('Seal')[4]);chosen['pattern']['match_work_limit']=chosen['pattern']['match_result_limit']=50000
    assert p['compiled_rule']==chosen and p['original_payload'] in p['binding']['matched_edges']
    b=p['binding'];matched=b['matched_edges'];bound=b['bound_nodes']
    event=dict(rule_id=0,rule_name=chosen['name'],bound_nodes=bound,matched_edges=matched,nodes_created=[],nodes_deleted=[],cleanup_neighbors=[],
        edges_deleted=[dict(id=i,edge=edges[old.edge_key(i)]) for i in matched],edges_created=[])
    after=dict(p['after_inventory']['edges']);removed={old.edge_key(i) for i in matched}
    added=[(k,e) for k,e in after.items() if k not in set(edges)-removed]
    # Created witness first, sealed payload second; infer identities from exact templates.
    for indices,label in chosen['template']['rhs_edges']:
        wanted=dict(label=label,nodes=[bound[i] for i in indices]);matches=[(k,e) for k,e in added if e==wanted];assert len(matches)==1
        key,e=matches[0];m=re.fullmatch(r'EdgeId\((\d+)v(\d+)\)',key);assert m
        event['edges_created'].append(dict(id=dict(idx=int(m[1]),version=int(m[2])),edge=e))
    nodes,edges,_=replay([event],[chosen],nodes,edges)
    close(graph_inventory(nodes,edges,0),p['after_inventory'])
    all_world=list(map(old.node_key,r['observation_nodes']));world=list(map(old.node_key,r['ordinary_nodes']))
    assert len(set(all_world))==7 and [nodes[k]['label'] for k in all_world]==old.ORDINARY_LABELS
    assert {k for k,v in nodes.items() if v['label']<META}==set(all_world)
    assert [nodes[k]['label'] for k in world]==r['context']+[12] and len(set(world))==3
    labels=[99,98];initial=[old.candidate(True,r['context']),old.candidate(False,labels)]
    assert r['initial_all_candidates']==initial
    birth_fuel=2.;valid=True;installed=initial;touches=[];hazards=[];lesions=[];repairs=[];births=[]
    schedule=[] if arm=='learned_no_hazard' else HAZARDS
    assert r['hazard_schedule']==schedule
    for step,row in enumerate(rows):
        if step:
            prior=rows[step-1];pre_valid=valid
            due=arm!='learned_frozen' and step>1 and (step-1)%2==0
            previous_installed=installed
            if due:installed=[old.candidate(True,r['context']),old.candidate(False,labels)] if valid else []
            assert prior['installed']==installed
            table=r['table_catalogue'][row['pre_step_table']]
            assert len(table)==len(installed)+len(operations)+len(editors)
            assert table[len(installed):]==bounded(operations+editors)
            for c,t in zip(installed,table):assert t==bounded([world_compiled(c,t['name'])])[0]
            report=row['compile_report'];assert (report is not None)==due
            if report:
                changed=previous_installed!=installed
                assert report['step']==step-1 and report['quarantined']==int(not pre_valid)
                assert report['validated_regions']==2+int(pre_valid) and report['changed'] is changed
                assert report['installed']==int(changed and bool(installed)) and report['retired']==int(changed and bool(previous_installed))
                assert len(report['classes'])==len(installed)
            assert row['compile_boundary']==step-1 and row['rng_before']==row['rng_after']
            events=row['events'];assert row['stats']['step']==step and row['stats']['accepted_rewrites']==len(events)
            expected_hazard=step in schedule and arm!='learned_protected'
            expected_repair=not valid and fuel>=1. and not expected_hazard
            slot=0 if step==1 else 1
            expected_birth=step in [1,5] and actions[slot]!=0
            assert row['paid_edits']==int(expected_birth)
            assert sum(e['kind']=='hazard' for e in events)==int(expected_hazard)
            assert sum(e['kind']=='encoded_repair' for e in events)==int(expected_repair)
            assert sum(e['kind']=='seeded_editor' for e in events)==int(not expected_hazard)
            ordinary=[e for e in edges.values() if all(old.node_key(n) in world for n in e['nodes'])]
            path=Counter([(0,tuple(sorted((world[0],world[1])))),(0,tuple(sorted((world[1],world[2]))))])
            star=Counter([(0,tuple(sorted((world[0],world[1])))),(0,tuple(sorted((world[0],world[2]))))])
            phase=old.logical(ordinary);assert phase in [path,star]
            expected_world=bool(installed) and (phase==path or installed[1]['labels'][:2]==r['context'])
            assert sum(e['kind']=='world' for e in events)==int(expected_world)
            for e in events:
                rid=e['rule_id'];name=e['rule_name'];bound=list(map(old.node_key,e['bound_nodes']))
                if e['kind']=='world':
                    assert rid<len(installed) and e['rcf']==installed[rid]['rcf'] and bound==world and len(set(bound))==3
                    assert rid==(0 if phase==path else 1)
                else:
                    assert e['rcf'] is None
                    if e['kind']=='hazard':assert name==f'maintenance_hazard_{step}' and e['bound_nodes'][:3]==target
                    elif e['kind']=='encoded_repair':assert name==operations[14+4]['name'] and bound[0]==repair and e['bound_nodes'][1:]==[root,target[0],target[1]]
                    else:
                        assert name==raw[slot+1 if expected_birth else 0]['name']
                        assert bound==([rk,literals[slot],token] if expected_birth else [token])
            previous_nodes=nodes
            nodes,edges,totals=replay(events,table,nodes,edges)
            changes={old.node_key(x['id']):x for x in row['changed_nodes']};assert len(changes)==len(row['changed_nodes'])
            assert set(changes)=={k for k in nodes if nodes[k]!=previous_nodes[k]}
            for k,x in changes.items():assert x['before']==previous_nodes[k] and x['after']==nodes[k]
            for k in ['edges_created','edges_deleted','property_effects_applied']:assert row['stats'][k]==totals[k]
            for k in ['nodes_created','nodes_deleted','property_effects_skipped']:assert row['stats'][k]==0
            usage=row['usage'];assert usage['truncated'] is False
            assert [x['rule_id'] for x in usage['rules']]==[i for i,t in enumerate(table) if t['enabled']]
            work=results=0
            for x in usage['rules']:
                assert x['truncated'] is False and x['work_allowance']==50000-work and x['result_allowance']==50000-results
                assert 0<=x['work']<=x['work_allowance'] and 0<=x['results']<=x['result_allowance']
                work+=x['work'];results+=x['results']
            assert usage['work']==work and usage['results']==results
            if expected_birth:labels[slot]=old.CHOICES[slot][actions[slot]];birth_fuel-=1.;births.append(step);touches.append(step)
            if expected_hazard:
                hazards.append(step);touches.append(step)
                if valid:lesions.append(step)
                valid=False
            if expected_repair:repairs.append(step);touches.append(step);fuel-=1.;valid=True
        else:assert row['events']==[] and row['paid_edits']==0
        assert row['genotype']==([old.candidate(True,r['context']),old.candidate(False,labels)] if valid else [])
        state=row['state'];assert state['literals']==labels and state['fuel']==birth_fuel and state['clock']==step and state['root_alive'] is True
        assert state['repair_fuel']==fuel and state['target_payload_label']==(ROLE if valid else 0)
        assert old.property_value(nodes[token],old.CLOCK)=={'Int':step} and old.property_value(nodes[token],1)=={'Float':birth_fuel}
        assert [old.property_value(nodes[k],0)['Int'] for k in literals]==labels and old.property_value(nodes[repair],1)=={'Float':fuel}
        assert {old.edge_key(x['id']):x['edge'] for x in state['world_edges']}=={k:e for k,e in edges.items() if all(old.node_key(n) in all_world for n in e['nodes'])}
        # Full completeness independently rechecked at every boundary, including quarantine.
        old.check_complete_material(dict(root=root,inventory=graph_inventory(nodes,edges,step)),valid)
    assert rows[-1]['installed']==installed
    close(graph_inventory(nodes,edges,96),r['final_inventory'])
    assert r['summary']==dict(birth_payments=len(births),hazard_events=len(hazards),repair_payments=len(repairs))
    measured=survival.cell(r,old.candidate(False,r['context'])['rcf'],touches)
    return dict(arm=arm,context=r['context'],birth_payments=len(births),hazard_opportunities=len(schedule),blocked_opportunities=4 if arm=='learned_protected' else 0,
        hazard_events=len(hazards),newly_damaging_lesions=len(lesions),repeated_erases=len(hazards)-len(lesions),
        repair_payments=len(repairs),initialization_payments=1,final_repair_fuel=fuel,
        restoration_steps=repairs,measured=measured)


def imports(out):
    assert a.digest(out/'confirmation_source_seal.json')==SOURCE_SEAL
    prior=a.load(out/'confirmation_source_seal.json');assert len(prior)==759
    assert a.digest(out/'checkpoint_import.json')==prior['checkpoint_import.json']
    assert a.digest(out/'exact_source_seal.json')==EXACT_SEAL
    exact=a.load(out/'exact_source_seal.json');assert len(exact)==445
    assert a.digest(out/'checkpoint_import.json')==exact['checkpoints.json']
    checkpoints=a.load(out/'checkpoint_import.json');assert [x['checkpoint'] for x in checkpoints]==[12,100]
    return checkpoints[1]['final_graph']['theta']


def audit(out):
    manifest=a.load(out/'manifest.json');assert manifest['profile']==PROFILE and manifest['design']==DESIGN
    theta=imports(out);directory=out/'native';m=a.load(directory/'manifest.json')
    assert m==dict(profile=PROFILE,status='complete',scheduled_cells=63,completed_cells=63,requested_advances=6048,native_work_unknown=False,
        optimizer_updates=0,training_admitted=False,scientific_acceptance=False,sustained_claim=False)
    decisions=a.load(directory/'decisions.json');assert len(decisions)==9
    state=DRAW;schedule=[];cells=[];groups={arm:[] for arm in ARMS}
    for world,c in enumerate(CONTEXTS):
        d=decisions[world];assert d['context']==c and d['stream_before']==state
        draws=[]
        for _ in range(2):state,x=next_draw(state);draws.append(x)
        assert d['stream_after']==state;close(d['draws'],draws)
        r=a.load(directory/f'cell-{world*7:04}.json');o=encoder.observe(r['initial_inventory']);inp=encoder.make_input('graph',o)
        close(d['observation'],o);close(d['input'],inp)
        prob=encoder.statistics(theta,inp,derivatives=False);close(d['probabilities'],prob)
        actions=encoder.sample(prob,draws);assert d['actions']==actions
        for i,arm in enumerate(ARMS):
            index=world*7+i;action=[old.CHOICES[s].index(c[s]) for s in range(2)] if arm=='oracle_exposed_repair' else actions
            slot=dict(id=index,world=world,context=c,arm=arm,actions=action,seed=SEED,horizon=96);schedule.append(slot)
            r=a.load(directory/f'cell-{index:04}.json')
            for k in ['context','arm','actions','seed','horizon']:assert r[k]==slot[k]
            close(encoder.observe(r['initial_inventory']),o)
            result=audit_receipt(r);result['id']=index;cells.append(result);groups[arm].append(result['measured'])
    assert a.load(directory/'schedule.json')==schedule
    assert a.load(directory/'status.json')==[dict(slot,status='complete') for slot in schedule]
    assert a.files(directory)=={f'cell-{i:04}.json' for i in range(63)}|{'decisions.json','schedule.json','status.json','manifest.json','executed_test_binary'}
    binary=a.digest(directory/'executed_test_binary');assert binary==manifest['prelaunch_binary_sha256']
    cohorts={arm:survival.cohort(group,16.) for arm,group in groups.items()}
    totals={key:sum(c[key] for c in cells) for key in ['birth_payments','hazard_opportunities','blocked_opportunities','hazard_events','newly_damaging_lesions','repeated_erases','repair_payments','initialization_payments']}
    # Mechanism capacity and learned continuity are explicitly distinct.
    learned=[c for c in cells if c['arm']=='learned_exposed_repair'];oracle=[c for c in cells if c['arm']=='oracle_exposed_repair']
    return dict(profile=PROFILE,audit_pass=True,cells=63,native_advances=6048,optimizer_updates=0,totals=totals,
        learned_target_births=sum(bool(c['measured']['genotype_spells']) for c in learned),
        learned_nontrivial_worlds=sum(c['measured']['nontrivial'] for c in learned),
        learned_paid_restoration_pass=all(c['restoration_steps']==[17,33,49,65] for c in learned),
        oracle_capacity_pass=all(c['restoration_steps']==[17,33,49,65] and c['measured']['nontrivial'] for c in oracle),
        cohorts=cohorts,cell_measurements=cells,executed_binary_sha256=binary,
        scientific_acceptance=False,sustained_claim=False,training_admitted=False,independent_physics_replay=False)


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('output',type=Path);p.add_argument('--seal',required=True);p.add_argument('--entries',type=int,required=True)
    args=p.parse_args();a.verify(args.output,expected=args.seal,count=args.entries)
    result=audit(args.output);close(a.load(args.output/'audit.json'),result)
    print(json.dumps({k:v for k,v in result.items() if k not in ['cell_measurements','cohorts']},indent=2))


if __name__=='__main__':main()
