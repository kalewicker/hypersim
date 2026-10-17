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
PROFILE='E2-defense-allocation-diagnostic-v1';NATIVE='E2-defense-allocation-native-v1'
META=old.META;ROLE=META+0x50;MARKER=0x7702;DEFENSE=META+0x7703;THREAT=META+0x7704;PENDING=0x7704
WALLET=META+0x7705;PAYOFF=META+0x7706;FORECAST=0x7705;BOUNTY=0x7706;CREDITS=0x7707
SCENARIOS=['hazard_low','hazard_high','safe_low','safe_high'];RESOURCES=['paid','unpaid'];ACTIONS=['defend','harvest']
HAZARDS=[16];SEED=993748718001
DESIGN=json.loads('''{"profile":"E2-defense-allocation-diagnostic-v1","native_profile":"E2-defense-allocation-native-v1","context":[10,11],"birth_actions":[1,1],"scenarios":["hazard_low","hazard_high","safe_low","safe_high"],"threat_expected":[true,true,false,false],"bounty":[2,12,2,12],"resources":["paid","unpaid"],"wallet_units":[1,0],"actions":["defend","harvest"],"horizon":32,"request_step":15,"resolution_step":16,"allocation_step":14,"utility_window":[17,32],"seed":993748718001,"cells":16,"maximum_native_advances":512,"maximum_nonadvancing_seal_applications":16,"native_deadline_seconds":60,"output_limit_bytes":134217728,"match_work_limit":50000,"match_result_limit":50000,"epoch":2,"birth_fuel":2,"seal_cost":1,"repair_fuel":0,"initial_defense_fuel":0,"damage_priority":300,"absorb_priority":400,"allocation_priority":500,"utility":"post-resolution novel ordinary firings + native harvested credits","optimizer_updates":0,"scientific_acceptance":false,"sustained_claim":false,"training_admitted":false,"independent_physics_replay":false}''')
def editor_catalogue(action):
    raw=old.editors('live_paid',[1,1]);raw[0]['lhs_node_property_guards'][0]['value']=32
    r=raw_editor('prevention_request_15',['token','threat'],dict(token=old.TOKEN,threat=THREAT),[],
        [('token',old.CLOCK,'Eq',14),('threat',PENDING,'Eq',0),('threat',FORECAST,'Eq',1)],
        [('token',old.CLOCK,'Add',1),('threat',PENDING,'Set',1)])
    r['priority']=300;raw.append(r)
    for absorb in [False,True]:
        names=['a','b','root','token','threat']+(['defense'] if absorb else [])
        labels=dict(a=META+4,b=META+5,root=META+1,token=old.TOKEN,threat=THREAT)
        if absorb:labels['defense']=DEFENSE
        r=raw_editor('prevention_paid_absorb' if absorb else 'prevention_damage',names,labels,[['a','b','root']],
            [('a',MARKER,'Eq',1),('b',MARKER,'Eq',2),('threat',PENDING,'Eq',1)]+([('defense',1,'Gte',1.),('defense',1,'Lte',1.)] if absorb else []),
            [('token',old.CLOCK,'Add',1),('threat',PENDING,'Set',0)]+([('defense',1,'Subtract',1.)] if absorb else []))
        r['priority']=400 if absorb else 300;r['lhs']['edge_labels']={'0':ROLE} if absorb else {};r['rhs']['edge_labels']={'0':ROLE} if absorb else {}
        raw.append(r)
    for bounty in [0,2,12]:
        defend=bounty==0;target='defense' if defend else 'payoff'
        guards=[('token',old.CLOCK,'Eq',13),('wallet',1,'Gte',1.),('wallet',1,'Lte',1.)]
        guards+= [(target,1,'Eq',0.)] if defend else [(target,BOUNTY,'Eq',bounty),(target,CREDITS,'Eq',0)]
        effects=[(target,1 if defend else CREDITS,'Add',1. if defend else bounty),('wallet',1,'Subtract',1.),('token',old.CLOCK,'Add',1)]
        r=raw_editor('allocation_defend' if defend else f'allocation_harvest_{bounty}',['token','wallet',target],
            dict(token=old.TOKEN,wallet=WALLET,**{target:DEFENSE if defend else PAYOFF}),[],guards,effects,
            enabled=action==('defend' if defend else 'harvest'))
        r['priority']=500;raw.append(r)
    return raw,[compiled(r) for r in raw]

def audit_receipt(r):
    scenario=r['scenario'];resource=r['resource'];action=r['action']
    assert scenario in SCENARIOS and resource in RESOURCES and action in ACTIONS and r['context']==[10,11] and r['seed']==SEED
    forecast=int(scenario.startswith('hazard'));bounty=12 if scenario.endswith('high') else 2
    wallet_units=1. if resource=='paid' else 0.;initial_wallet=wallet_units;credits=0;allocated=[]
    actions=r['actions'];assert actions==[1,1]
    assert r['profile']==NATIVE and r['classification']=='scripted_engineering_fixture' and r['maintenance'] is True
    assert r['horizon']==r['completed_steps']==32 and r['initial_fuel']==2.
    for k in ['training_admitted','scientific_acceptance','sustained_claim','independent_physics_replay']:assert r[k] is False
    rows=r['timeline'];assert [x['step'] for x in rows]==list(range(33))
    indices=[row['pre_step_table'] for row in rows[1:]]
    assert all(type(i) is int and 0<=i<len(r['table_catalogue']) for i in indices)
    assert list(dict.fromkeys(indices))==list(range(len(r['table_catalogue'])))
    raw,editors=editor_catalogue(action);assert r['seeded_editor_rules']==raw and r['seeded_editor_compiled']==editors
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
    defense=old.node_key(r['defense']);threat=old.node_key(r['threat']);defense_fuel=0.;pending=0
    assert r['initial_defense_fuel']==defense_fuel and nodes[defense]['label']==DEFENSE and nodes[threat]['label']==THREAT
    assert old.property_value(nodes[defense],1)=={'Float':defense_fuel} and old.property_value(nodes[threat],PENDING)=={'Int':0}
    wallet=old.node_key(r['wallet']);payoff=old.node_key(r['payoff'])
    assert nodes[wallet]['label']==WALLET and nodes[payoff]['label']==PAYOFF
    assert r['initial_wallet_units']==initial_wallet and old.property_value(nodes[wallet],1)=={'Float':wallet_units}
    assert old.property_value(nodes[threat],FORECAST)=={'Int':forecast}
    assert old.property_value(nodes[payoff],BOUNTY)=={'Int':bounty} and old.property_value(nodes[payoff],CREDITS)=={'Int':0}
    assert not any(old.node_key(n) in [defense,threat,wallet,payoff] for e in edges.values() for n in e['nodes'])
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
    schedule=HAZARDS if forecast else []
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
            expected_absorb=bool(pending and valid and defense_fuel>=1.)
            expected_hazard=bool(pending and not expected_absorb)
            expected_repair=False
            expected_allocation=step==14 and wallet_units==1.
            if step==14:
                close(r['allocation_observation'],dict(step=13,wallet=r['wallet'],threat=r['threat'],payoff=r['payoff'],
                    wallet_units=wallet_units,forecast=forecast,bounty=bounty,inventory=graph_inventory(nodes,edges,13)))
            assert bool(pending)==(step in schedule)
            assert sum(e['kind']=='request' for e in events)==int(expected_request)
            assert sum(e['kind']=='absorb' for e in events)==int(expected_absorb)
            slot=0 if step==1 else 1
            expected_birth=step in [1,5] and actions[slot]!=0
            assert row['paid_edits']==int(expected_birth)
            assert sum(e['kind']=='hazard' for e in events)==int(expected_hazard)
            assert sum(e['kind']=='encoded_repair' for e in events)==int(expected_repair)
            assert sum(e['kind']=='allocate_defend' for e in events)==int(expected_allocation and action=='defend')
            assert sum(e['kind']=='allocate_harvest' for e in events)==int(expected_allocation and action=='harvest')
            assert sum(e['kind']=='seeded_editor' for e in events)==int(not (expected_hazard or expected_absorb or expected_request or expected_allocation))
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
                    elif e['kind']=='allocate_defend':assert name=='allocation_defend' and bound==[token,wallet,defense]
                    elif e['kind']=='allocate_harvest':assert name==f'allocation_harvest_{bounty}' and bound==[token,wallet,payoff]
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
            assert results_by_name['prevention_paid_absorb']==int(expected_absorb)
            assert results_by_name['prevention_request_15']==int(expected_request)
            if action=='defend':assert results_by_name['allocation_defend']==int(expected_allocation)
            else:
                for offer in [2,12]:assert results_by_name[f'allocation_harvest_{offer}']==int(expected_allocation and bounty==offer)
            if pending:
                # Both candidates are demonstrably admitted to the native matcher.
                # Check the losing damage binding against the same live payload.
                damage_id=next(i for i,t in enumerate(table) if t['name']=='prevention_damage')
                selected=next(e for e in events if e['kind'] in ['absorb','hazard'])
                check_binding(table[damage_id],selected['bound_nodes'][:5],selected['matched_edges'],previous_nodes,
                    {k:e for k,e in previous_edges.items()})
                eligibility.append(dict(step=step,damage_bindings=1,absorb_bindings=int(expected_absorb),winner=selected['kind']))
            if expected_allocation:
                allocated.append(step);wallet_units-=1.
                if action=='defend':defense_fuel+=1.
                else:credits+=bounty
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
        assert state['wallet_units']==wallet_units and state['forecast']==forecast and state['bounty']==bounty and state['harvested_credits']==credits
        assert old.property_value(nodes[wallet],1)=={'Float':wallet_units}
        assert old.property_value(nodes[threat],FORECAST)=={'Int':forecast}
        assert old.property_value(nodes[payoff],BOUNTY)=={'Int':bounty} and old.property_value(nodes[payoff],CREDITS)=={'Int':credits}
        harvest_spent=len(allocated) if action=='harvest' else 0
        assert wallet_units+defense_fuel+len(absorbed)+harvest_spent==initial_wallet,'unit ledger conservation'
        assert old.property_value(nodes[defense],1)=={'Float':defense_fuel} and old.property_value(nodes[threat],PENDING)=={'Int':pending}
        assert old.property_value(nodes[token],old.CLOCK)=={'Int':step} and old.property_value(nodes[token],1)=={'Float':birth_fuel}
        assert [old.property_value(nodes[k],0)['Int'] for k in literals]==labels and old.property_value(nodes[repair],1)=={'Float':fuel}
        assert {old.edge_key(x['id']):x['edge'] for x in state['world_edges']}=={k:e for k,e in edges.items() if all(old.node_key(n) in all_world for n in e['nodes'])}
        # Full completeness independently rechecked at every boundary, including quarantine.
        old.check_complete_material(dict(root=root,inventory=graph_inventory(nodes,edges,step)),valid)
    assert rows[-1]['installed']==installed
    close(graph_inventory(nodes,edges,32),r['final_inventory'])
    assert r['summary']==dict(birth_payments=len(births),hazard_events=len(hazards),repair_payments=len(repairs),absorbed_threats=len(absorbed),issued_threats=len(issued),allocation_payments=len(allocated),harvested_credits=credits)
    measured=survival.cell(r,old.candidate(False,r['context'])['rcf'],touches)
    post_fires=[s for s in measured['ordinary_fire_steps'] if 17<=s<=32]
    return dict(scenario=scenario,resource=resource,action=action,context=r['context'],utility=len(post_fires)+credits,
        post_resolution_fires=post_fires,harvested_credits=credits,allocation_payments=len(allocated),final_wallet_units=wallet_units,birth_payments=len(births),issued_threats=len(issued),issued_steps=issued,
        threat_resolution_steps=[x['step'] for x in eligibility],eligibility=eligibility,absorbed_steps=absorbed,
        absorbed_threats=len(absorbed),hazard_events=len(hazards),hazard_steps=hazards,newly_damaging_lesions=len(lesions),
        repeated_erases=len(hazards)-len(lesions),repair_payments=len(repairs),initialization_payments=1,
        final_defense_fuel=defense_fuel,measured=measured)

def audit(out):
    manifest=a.load(out/'manifest.json');assert manifest['profile']==PROFILE and manifest['design']==DESIGN
    for k in ['scientific_acceptance','sustained_claim','training_admitted','independent_physics_replay']:assert manifest[k] is False
    directory=out/'native'
    assert a.load(directory/'manifest.json')==dict(profile=NATIVE,status='complete',scheduled_cells=16,completed_cells=16,
        requested_advances=512,native_work_unknown=False,optimizer_updates=0,training_admitted=False,scientific_acceptance=False,sustained_claim=False)
    slots=[dict(id=i,scenario=s,resource=r,action=act,horizon=32,seed=SEED) for i,(s,r,act) in enumerate((s,r,act) for s in SCENARIOS for r in RESOURCES for act in ACTIONS)]
    assert a.load(directory/'schedule.json')==slots and a.load(directory/'status.json')==[dict(s,status='complete') for s in slots]
    assert a.files(directory)=={f'cell-{i:04}.json' for i in range(16)}|{'schedule.json','status.json','manifest.json','executed_test_binary'}
    cells=[]
    for slot in slots:
        r=a.load(directory/f"cell-{slot['id']:04}.json")
        for k in ['scenario','resource','action','horizon','seed']:assert r[k]==slot[k]
        c=audit_receipt(r);c['id']=slot['id'];cells.append(c)
    # Paired actions have identical action-free preallocation observations.
    for i in range(0,16,2):assert a.load(directory/f'cell-{i:04}.json')['allocation_observation']==a.load(directory/f'cell-{i+1:04}.json')['allocation_observation']
    observations=[a.load(directory/f'cell-{i:04}.json')['allocation_observation'] for i in range(0,16,2)]
    legacy=[previous.encoder.observe(o['inventory']) for o in observations]
    task_inputs=[dict(wallet_units=o['wallet_units'],forecast=o['forecast'],bounty=o['bounty']) for o in observations]
    legacy_alias=all(o==legacy[0] for o in legacy)
    assert len({(o['wallet_units'],o['forecast'],o['bounty']) for o in task_inputs})==8
    paid=[[c['utility'] for c in cells if c['scenario']==s and c['resource']=='paid'] for s in SCENARIOS]
    unpaid=[[c['utility'] for c in cells if c['scenario']==s and c['resource']=='unpaid'] for s in SCENARIOS]
    best=[ACTIONS[0] if p[0]>p[1] else ACTIONS[1] if p[1]>p[0] else None for p in paid]
    defend=sum(p[0] for p in paid)/4;harvest=sum(p[1] for p in paid)/4;adaptive=sum(max(p) for p in paid)/4
    flags=dict(paid_table_exact=paid==[[8,2],[8,12],[8,10],[8,20]],unpaid_table_exact=unpaid==[[0,0],[0,0],[8,8],[8,8]],
        unique_state_dependent_choice=best==['defend','harvest','harvest','harvest'],
        adaptive_above_constants=adaptive>max(defend,harvest) and [adaptive,defend,harvest]==[12.5,8.,11.],
        no_unpaid_value_creation=all(c['allocation_payments']==c['harvested_credits']==c['absorbed_threats']==0 for c in cells if c['resource']=='unpaid'),
        admitted_damage=all(all(e['damage_bindings']==1 for e in c['eligibility']) for c in cells),
        repair_unfunded=all(c['repair_payments']==0 for c in cells))
    binary=a.digest(directory/'executed_test_binary');assert binary==manifest['prelaunch_binary_sha256']
    totals={key:sum(c[key] for c in cells) for key in ['birth_payments','issued_threats','absorbed_threats','hazard_events','newly_damaging_lesions','repeated_erases','repair_payments','initialization_payments','allocation_payments','harvested_credits']}
    return dict(profile=PROFILE,audit_pass=True,diagnostic_pass=all(flags.values()),flags=flags,cells=cells,paid_utilities=paid,unpaid_utilities=unpaid,
        best_paid_actions=best,adaptive_mean=adaptive,always_defend_mean=defend,always_harvest_mean=harvest,totals=totals,
        task_observations=task_inputs,existing_encoder_observation_alias=legacy_alias,
        native_advances=512,nonadvancing_seal_applications=16,optimizer_updates=0,executed_binary_sha256=binary,
        scientific_acceptance=False,sustained_claim=False,training_admitted=False,independent_physics_replay=False)
def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('output',type=Path);p.add_argument('--seal',required=True);p.add_argument('--entries',type=int,required=True)
    args=p.parse_args();a.verify(args.output,expected=args.seal,count=args.entries)
    result=audit(args.output);close(a.load(args.output/'audit.json'),result)
    print(json.dumps({k:v for k,v in result.items() if k!='cells'},indent=2))
if __name__=='__main__':main()
