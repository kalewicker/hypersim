#!/usr/bin/env python3
"""Independent pending-threat, paid native arbitration and continuity accounting."""
import argparse
from collections import Counter
import copy
import json
from pathlib import Path
import re
import verify_ere_rule_maintenance as previous
from verify_ere_rule_maintenance import (a,old,raw_editor,compiled,operation_catalogue,bounded,
    world_compiled,graph_inventory,replay,close,survival,check_binding)
import verify_ere_graph_encoder_confirmation as confirmation
PROFILE='E2-rule-prevention-transfer-panel-v1';NATIVE='E2-rule-prevention-transfer-native-v1'
META=old.META;ROLE=META+0x50;MARKER=0x7702;DEFENSE=META+0x7703;THREAT=META+0x7704;PENDING=0x7704
ARMS=['paid','exhaustion','unpaid','disabled','no_hazard'];FUELS=[4,2,0,4,4]
HAZARDS=[16,32,48,64];SEED=993748718001;DRAW=993748724001
CONTEXTS=old.CONTEXTS
CONDITIONS=['learned_paid','learned_exhaustion','learned_unpaid','learned_disabled','learned_no_hazard','masked_paid','blind_paid','oracle_paid']
NATIVE_ARMS=ARMS+['paid','paid','paid'];POLICY_INDICES=[0,0,0,0,0,3,1,None]
DESIGN=json.loads('''{"profile":"E2-rule-prevention-transfer-panel-v1","native_profile":"E2-rule-prevention-transfer-native-v1","classification":"frozen_controller_transfer_engineering","contexts":[[10,11],[10,21],[10,31],[20,11],[20,21],[20,31],[30,11],[30,21],[30,31]],"conditions":["learned_paid","learned_exhaustion","learned_unpaid","learned_disabled","learned_no_hazard","masked_paid","blind_paid","oracle_paid"],"native_arms":["paid","exhaustion","unpaid","disabled","no_hazard","paid","paid","paid"],"policy_indices":[0,0,0,0,0,3,1,null],"defense_fuel":[4,2,0,4,4,4,4,4],"repair_fuel":0,"birth_fuel":2,"seal_cost":1,"request_steps":[15,31,47,63],"threat_resolution_steps":[16,32,48,64],"damage_priority":300,"absorb_priority":400,"native_seed":993748718001,"draw_seed":993748724001,"horizon":96,"cells":72,"maximum_native_advances":6912,"maximum_nonadvancing_seal_applications":72,"native_deadline_seconds":180,"output_limit_bytes":268435456,"match_work_limit":50000,"match_result_limit":50000,"reference_interval":16,"optimizer_updates":0,"scientific_acceptance":false,"sustained_claim":false,"training_admitted":false,"independent_physics_replay":false}''')

def editor_catalogue(arm,actions):
    raw=old.editors('live_paid',actions)
    raw[0]['lhs_node_property_guards'][0]['value']=96
    for step in [15,31,47,63]:
        r=raw_editor(f'prevention_request_{step}',['token','threat'],dict(token=old.TOKEN,threat=THREAT),[],
            [('token',old.CLOCK,'Eq',step-1),('threat',PENDING,'Eq',0)],
            [('token',old.CLOCK,'Add',1),('threat',PENDING,'Set',1)],enabled=arm!='no_hazard')
        r['priority']=300;raw.append(r)
    for absorb in [False,True]:
        names=['a','b','root','token','threat']+(['defense'] if absorb else [])
        labels=dict(a=META+4,b=META+5,root=META+1,token=old.TOKEN,threat=THREAT)
        if absorb:labels['defense']=DEFENSE
        r=raw_editor('prevention_paid_absorb' if absorb else 'prevention_damage',names,labels,[['a','b','root']],
            [('a',MARKER,'Eq',1),('b',MARKER,'Eq',2),('threat',PENDING,'Eq',1)]+
                ([('defense',1,'Gte',1.),('defense',1,'Lte',4.)] if absorb else []),
            [('token',old.CLOCK,'Add',1),('threat',PENDING,'Set',0)]+([('defense',1,'Subtract',1.)] if absorb else []),
            enabled=not absorb or arm!='disabled')
        r['priority']=400 if absorb else 300
        r['lhs']['edge_labels']={'0':ROLE} if absorb else {}
        r['rhs']['edge_labels']={'0':ROLE} if absorb else {}
        raw.append(r)
    return raw,[compiled(r) for r in raw]

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
    fuel=0.;assert old.property_value(nodes[repair],1)=={'Float':fuel}
    defense=old.node_key(r['defense']);threat=old.node_key(r['threat']);defense_fuel=float(FUELS[ARMS.index(arm)]);pending=0
    assert r['initial_defense_fuel']==defense_fuel and nodes[defense]['label']==DEFENSE and nodes[threat]['label']==THREAT
    assert old.property_value(nodes[defense],1)=={'Float':defense_fuel} and old.property_value(nodes[threat],PENDING)=={'Int':0}
    assert not any(old.node_key(n) in [defense,threat] for e in edges.values() for n in e['nodes'])
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
    birth_fuel=2.;valid=True;installed=initial;touches=[];hazards=[];lesions=[];repairs=[];births=[];absorbed=[];issued=[];eligibility=[]
    schedule=[] if arm=='no_hazard' else HAZARDS
    assert r['hazard_schedule']==schedule
    for step,row in enumerate(rows):
        if step:
            prior=rows[step-1];pre_valid=valid
            due=step>1 and (step-1)%2==0
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
            expected_request=step in [x-1 for x in schedule]
            expected_absorb=bool(pending and valid and defense_fuel>=1. and arm!='disabled')
            expected_hazard=bool(pending and not expected_absorb)
            expected_repair=False
            assert bool(pending)==(step in schedule)
            assert sum(e['kind']=='request' for e in events)==int(expected_request)
            assert sum(e['kind']=='absorb' for e in events)==int(expected_absorb)
            slot=0 if step==1 else 1
            expected_birth=step in [1,5] and actions[slot]!=0
            assert row['paid_edits']==int(expected_birth)
            assert sum(e['kind']=='hazard' for e in events)==int(expected_hazard)
            assert sum(e['kind']=='encoded_repair' for e in events)==int(expected_repair)
            assert sum(e['kind']=='seeded_editor' for e in events)==int(not (expected_hazard or expected_absorb or expected_request))
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
                    if e['kind']=='hazard':assert name=='prevention_damage' and e['bound_nodes'][:3]==target and bound[3:]==[token,threat]
                    elif e['kind']=='absorb':assert name=='prevention_paid_absorb' and e['bound_nodes'][:3]==target and bound[3:]==[token,threat,defense]
                    elif e['kind']=='request':assert name==f'prevention_request_{step}' and bound==[token,threat]
                    elif e['kind']=='encoded_repair':raise AssertionError('repair is unfunded in prevention primitive')
                    else:
                        assert name==raw[slot+1 if expected_birth else 0]['name']
                        assert bound==([rk,literals[slot],token] if expected_birth else [token])
            previous_nodes=nodes;previous_edges=edges
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
            results_by_name={table[x['rule_id']]['name']:x['results'] for x in usage['rules']}
            assert results_by_name['prevention_damage']==int(bool(pending))
            if arm!='disabled':assert results_by_name['prevention_paid_absorb']==int(expected_absorb)
            for request_step in [15,31,47,63]:
                if arm!='no_hazard':assert results_by_name[f'prevention_request_{request_step}']==int(step==request_step)
            if pending:
                # Both candidates are demonstrably admitted to the native matcher.
                # Check the losing damage binding against the same live payload.
                damage_id=next(i for i,t in enumerate(table) if t['name']=='prevention_damage')
                selected=next(e for e in events if e['kind'] in ['absorb','hazard'])
                check_binding(table[damage_id],selected['bound_nodes'][:5],selected['matched_edges'],previous_nodes,
                    {k:e for k,e in previous_edges.items()})
                eligibility.append(dict(step=step,damage_bindings=1,absorb_bindings=int(expected_absorb),winner=selected['kind']))
            if expected_request:issued.append(step);pending=1
            if expected_absorb:absorbed.append(step);touches.append(step);defense_fuel-=1.;pending=0
            if expected_birth:labels[slot]=old.CHOICES[slot][actions[slot]];birth_fuel-=1.;births.append(step);touches.append(step)
            if expected_hazard:
                hazards.append(step);touches.append(step);pending=0
                if valid:lesions.append(step)
                valid=False
            if expected_repair:repairs.append(step);touches.append(step);fuel-=1.;valid=True
        else:assert row['events']==[] and row['paid_edits']==0
        assert row['genotype']==([old.candidate(True,r['context']),old.candidate(False,labels)] if valid else [])
        state=row['state'];assert state['literals']==labels and state['fuel']==birth_fuel and state['clock']==step and state['root_alive'] is True
        assert state['repair_fuel']==fuel and state['target_payload_label']==(ROLE if valid else 0)
        assert state['defense_fuel']==defense_fuel and state['pending_threat']==pending
        assert old.property_value(nodes[defense],1)=={'Float':defense_fuel} and old.property_value(nodes[threat],PENDING)=={'Int':pending}
        assert old.property_value(nodes[token],old.CLOCK)=={'Int':step} and old.property_value(nodes[token],1)=={'Float':birth_fuel}
        assert [old.property_value(nodes[k],0)['Int'] for k in literals]==labels and old.property_value(nodes[repair],1)=={'Float':fuel}
        assert {old.edge_key(x['id']):x['edge'] for x in state['world_edges']}=={k:e for k,e in edges.items() if all(old.node_key(n) in all_world for n in e['nodes'])}
        # Full completeness independently rechecked at every boundary, including quarantine.
        old.check_complete_material(dict(root=root,inventory=graph_inventory(nodes,edges,step)),valid)
    assert rows[-1]['installed']==installed
    close(graph_inventory(nodes,edges,96),r['final_inventory'])
    assert r['summary']==dict(birth_payments=len(births),hazard_events=len(hazards),repair_payments=len(repairs),absorbed_threats=len(absorbed),issued_threats=len(issued))
    measured=survival.cell(r,old.candidate(False,r['context'])['rcf'],touches)
    return dict(arm=arm,context=r['context'],birth_payments=len(births),issued_threats=len(issued),issued_steps=issued,
        threat_resolution_steps=[x['step'] for x in eligibility],eligibility=eligibility,absorbed_steps=absorbed,
        absorbed_threats=len(absorbed),hazard_events=len(hazards),hazard_steps=hazards,newly_damaging_lesions=len(lesions),
        repeated_erases=len(hazards)-len(lesions),repair_payments=len(repairs),initialization_payments=1,
        final_defense_fuel=defense_fuel,measured=measured)

def continuity(cell):
    m=cell['measured']
    def continuous(field,start):
        spells=m[field];return spells==[dict(start=start,end=None)]
    ordinary=m['ordinary_fire_steps']
    return dict(genotype_continuous_through96=continuous('genotype_spells',5),
        installed_continuous_through96=continuous('installed_spells',6),
        distinct_ordinary_use=bool(ordinary),
        use_before_and_after_each_threat=all(any(s<h for s in ordinary) and any(s>h for s in ordinary) for h in HAZARDS),
        class_lower_bounds_exceed16=all(m[k] is not None and m[k]['lower_bound']>16 for k in ['genotype_first_duration','installed_first_duration']))

def imports(out):
    previous.imports(out)
    checkpoints=a.load(out/'checkpoint_import.json')
    return confirmation.policies(checkpoints)

def decisions(out,policies,reference_directory=None):
    choices=a.load(out/('decision_preflight.json' if reference_directory else 'native/decisions.json'));assert len(choices)==9
    state=DRAW;schedule=[]
    for world,context in enumerate(CONTEXTS):
        d=choices[world];assert set(d)=={'context','observation','stream_before','stream_after','draws','policies'}
        assert d['context']==context and d['stream_before']==state
        draws=[]
        for _ in range(2):state,x=previous.next_draw(state);draws.append(x)
        assert d['stream_after']==state;close(d['draws'],draws)
        r=a.load(reference_directory/f'cell-{world*7:04}.json' if reference_directory else out/'native'/f'cell-{world*8:04}.json')
        o=previous.encoder.observe(r['initial_inventory']);close(d['observation'],o)
        assert len(d['policies'])==3
        acts={}
        for row,index in zip(d['policies'],[0,3,1]):
            mode,theta=policies[index];inp=previous.encoder.make_input(mode,o)
            prob=previous.encoder.statistics(theta,inp,derivatives=False)
            action=previous.encoder.sample(prob,draws)
            close(row,dict(policy_index=index,input=inp,probabilities=prob,actions=action))
            acts[index]=action
        for i,condition in enumerate(CONDITIONS):
            action=[old.CHOICES[s].index(context[s]) for s in range(2)] if i==7 else acts[POLICY_INDICES[i]]
            schedule.append(dict(id=world*8+i,world=world,context=context,condition=condition,arm=NATIVE_ARMS[i],actions=action,seed=SEED,horizon=96))
    return schedule

def audit(out):
    m=a.load(out/'manifest.json');assert m['profile']==PROFILE and m['design']==DESIGN
    for k in ['scientific_acceptance','sustained_claim','training_admitted','independent_physics_replay']:assert m[k] is False
    policies=imports(out);directory=out/'native'
    assert a.load(directory/'manifest.json')==dict(profile=PROFILE,status='complete',scheduled_cells=72,completed_cells=72,
        requested_advances=6912,native_work_unknown=False,optimizer_updates=0,training_admitted=False,scientific_acceptance=False,sustained_claim=False)
    slots=decisions(out,policies)
    close(a.load(out/'decision_preflight.json'),a.load(directory/'decisions.json'))
    assert a.load(directory/'schedule.json')==slots
    assert a.load(directory/'status.json')==[dict(s,status='complete') for s in slots]
    assert a.files(directory)=={f'cell-{i:04}.json' for i in range(72)}|{'decisions.json','schedule.json','status.json','manifest.json','executed_test_binary'}
    cells=[];groups={k:[] for k in CONDITIONS}
    for slot in slots:
        r=a.load(directory/f"cell-{slot['id']:04}.json")
        for key in ['context','actions','arm','seed','horizon']:assert r[key]==slot[key]
        d=a.load(directory/'decisions.json')[slot['world']]
        close(previous.encoder.observe(r['initial_inventory']),d['observation'])
        c=audit_receipt(r);c.update(id=slot['id'],condition=slot['condition'],actions=slot['actions'],world=slot['world'])
        c['continuity']=continuity(c);c['useful_continuity_pass']=all(c['continuity'].values()) and c['measured']['nontrivial']
        cells.append(c);groups[c['condition']].append(c)
    assert all(len(g)==9 for g in groups.values())
    counts={name:sum(c['useful_continuity_pass'] for c in group) for name,group in groups.items()}
    def exact(c,gen,installed):
        return c['measured']['genotype_first_duration']==dict(lower_bound=gen,observed_death_age=gen,right_censored=False) and c['measured']['installed_first_duration']==dict(lower_bound=installed,observed_death_age=installed,right_censored=False)
    flags=dict(
        learned_paid_full9=counts['learned_paid']==9 and all(c['absorbed_steps']==HAZARDS and c['hazard_steps']==[] and c['final_defense_fuel']==0. for c in groups['learned_paid']),
        oracle_paid_full9=counts['oracle_paid']==9 and all(c['absorbed_steps']==HAZARDS and c['hazard_steps']==[] and c['final_defense_fuel']==0. for c in groups['oracle_paid']),
        exhaustion_exact_full9=all(c['absorbed_steps']==[16,32] and c['hazard_steps']==[48,64] and exact(c,43,42) for c in groups['learned_exhaustion']),
        unpaid_exact_full9=all(c['absorbed_steps']==[] and c['hazard_steps']==HAZARDS and exact(c,11,10) for c in groups['learned_unpaid']),
        disabled_exact_full9=all(c['absorbed_steps']==[] and c['hazard_steps']==HAZARDS and c['final_defense_fuel']==4. and exact(c,11,10) for c in groups['learned_disabled']),
        no_hazard_exact_full9=counts['learned_no_hazard']==9 and all(c['issued_steps']==c['absorbed_steps']==c['hazard_steps']==[] and c['final_defense_fuel']==4. for c in groups['learned_no_hazard']),
        graph_over_masked=counts['learned_paid']>counts['masked_paid'],
        graph_over_blind=counts['learned_paid']>counts['blind_paid'],
        admitted_damage_bindings=all(all(x['damage_bindings']==1 for x in c['eligibility']) for c in cells),
        repair_unfunded=all(c['repair_payments']==0 for c in cells))
    cohorts={name:survival.cohort([c['measured'] for c in group],16.) for name,group in groups.items()}
    binary=a.digest(directory/'executed_test_binary');assert binary==m['prelaunch_binary_sha256']
    totals={key:sum(c[key] for c in cells) for key in ['birth_payments','issued_threats','absorbed_threats','hazard_events','newly_damaging_lesions','repeated_erases','repair_payments','initialization_payments']}
    return dict(profile=PROFILE,audit_pass=True,transfer_pass=all(flags.values()),flags=flags,cells=cells,
        useful_continuity_counts=counts,cohorts=cohorts,totals=totals,native_advances=6912,nonadvancing_seal_applications=72,
        optimizer_updates=0,executed_binary_sha256=binary,scientific_acceptance=False,sustained_claim=False,
        training_admitted=False,independent_physics_replay=False)

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('output',type=Path);p.add_argument('--seal',required=True);p.add_argument('--entries',type=int,required=True)
    args=p.parse_args();a.verify(args.output,expected=args.seal,count=args.entries)
    result=audit(args.output);close(a.load(args.output/'audit.json'),result)
    print(json.dumps({k:v for k,v in result.items() if k not in ['cells','cohorts']},indent=2))
if __name__=='__main__':main()
