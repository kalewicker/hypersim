#!/usr/bin/env python3
"""Independent CC-1 dictionary semantics + native arena/receipt auditor (stdlib only).

The semantic model has no generational arena, native calls, graph patch API, or
Rust validator dependency. It constructs the frozen fixtures and interprets an
independent script for each declared operation, then compares every observation.
"""
import argparse
from collections import Counter
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import sys
import time
sys.dont_write_bytecode = True
SITE, BEAD, BOND = 0xE2000000, 0xE2000001, 0xE2000002
ADJ, PLACE, ROLE_A, ROLE_B = range(0xE2000064, 0xE2000068)
IDENT, SITE_INDEX, SPECIES, ACTIVE, KIND, PORT_A, PORT_B = range(243001, 243008)
I64, U64 = 2**63-1, 2**64-1
FLAGS = ['training_admitted', 'scientific_acceptance', 'sustained_claim', 'evolution_enabled',
         'scientific_run_launched', 'ready_for_scientific_launch', 'reproductive_fairness_accepted', 'power_adequacy_established']

def require(value, message):
    if not value:
        raise ValueError(message)

def encoded(v):
    return json.dumps(v, sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()

def digest(v):
    return hashlib.sha256(encoded(v)).hexdigest()

class Rejection(Exception):
    pass

def reject(reason):
    raise Rejection(reason)

class Model:
    def __init__(self, contract, name):
        def resolve(n):
            f = deepcopy(contract['fixtures'][n])
            result = resolve(f.pop('base')) if 'base' in f else {}
            result.update(f)
            return result
        f = resolve(name)
        omitted = set(f.get('omit_beads', []))
        self.beads = {}
        for raw in f['beads']:
            if raw['id'] in omitted:
                continue
            b = deepcopy(raw)
            b['activated'] = f.get('set_activation', {}).get(str(b['id']), b['activated'])
            b['site'] = f.get('set_site', {}).get(str(b['id']), b['site'])
            b['species'] = 'ABCDF'.index(b['species'])
            b['generation'] = 0
            self.beads[b['id']] = b
        self.bonds = {int(i): dict(a=a, b=b, generation=0) for i, (a,b) in zip(f.get('initial_bond_ids', []), f.get('initial_bonds', []))}
        self.revision = int(f.get('initial_revision', 0))
        self.frontier = int(f.get('semantic_id_frontier', max([1,2,3]+list(self.beads)+list(self.bonds))+1))
        self.heat = int(f.get('initial_heat', 0))
        self.reservoir = [int(f.get('reservoir_active_fuel', 0)), int(f.get('reservoir_inactive_fuel', 0))]
        self.baseline = self.totals()

    def totals(self):
        material = [sum(b['species']==s for b in self.beads.values()) for s in range(5)]
        world_energy = 4*sum(b['activated'] for b in self.beads.values()) + 2*len(self.bonds)
        total_material = material[:]
        total_material[4] += sum(self.reservoir)
        energy = world_energy + 4*self.reservoir[0] + self.heat
        return total_material, energy

    def invariant(self):
        if self.heat < 0 or min(self.reservoir) < 0:
            reject('LedgerRange')
        m,e = self.totals()
        if m != self.baseline[0]:
            reject('MaterialBalance')
        if e != self.baseline[1]:
            reject('EnergyBalance')

    def snapshot(self):
        nodes = [dict(id=i+1,label=SITE,generation=0,site=i,species=None,activated=None) for i in range(3)]
        nodes += [dict(id=i,label=BEAD,**{k:b[k] for k in ('generation','site','species','activated')}) for i,b in self.beads.items()]
        nodes += [dict(id=i,label=BOND,generation=b['generation'],site=None,species=None,activated=None) for i,b in self.bonds.items()]
        edges = [dict(label=ADJ,nodes=[1,2]),dict(label=ADJ,nodes=[2,3])]
        edges += [dict(label=PLACE,nodes=sorted([i,b['site']+1])) for i,b in self.beads.items()]
        edges += [dict(label=role,nodes=sorted([i,b[endpoint]])) for i,b in self.bonds.items() for role,endpoint in [(ROLE_A,'a'),(ROLE_B,'b')]]
        material, energy = self.totals()
        return dict(nodes=sorted(nodes,key=lambda n:n['id']),edges=sorted(edges,key=lambda e:encoded(e)),revision=self.revision,
                    frontier=self.frontier,reservoir=self.reservoir[:],heat=self.heat,
                    world_material=[str(sum(b['species']==s for b in self.beads.values())) for s in range(5)],
                    total_material=list(map(str,material)),world_energy=str(4*sum(b['activated'] for b in self.beads.values())+2*len(self.bonds)),total_energy=str(energy))

    def prepare(self, action):
        # Priority is part of the frozen typed API. Overflow checks precede writes.
        if self.revision == U64:
            reject('ArithmeticOverflow')
        kind, params = action
        next_state = deepcopy(self)
        if kind == 'bond':
            a,b,f = params
            if any(i not in self.beads for i in params):
                reject('MissingHandle')
            if len(set(params)) != 3:
                reject('Distinctness')
            if [self.beads[i]['species'] for i in params] != [0,1,4]:
                reject('Species')
            if not self.beads[f]['activated']:
                reject('InactiveFuel')
            occupied = {x[k] for x in self.bonds.values() for k in ['a','b']}
            if a in occupied or b in occupied:
                reject('PortOccupied')
            if len({self.beads[i]['site'] for i in params}) != 1:
                reject('Nonlocal')
            if self.frontier == I64 or self.heat > I64-2:
                reject('ArithmeticOverflow')
            next_state.beads[f]['activated'] = False
            next_state.bonds[self.frontier] = dict(a=a,b=b,generation=self.revision+1)
            next_state.frontier += 1
            next_state.heat += 2
        elif kind == 'in':
            lane = 0 if params else 1
            if self.reservoir[lane] == 0:
                reject('ReservoirExhausted')
            if sum(b['site']==0 for b in self.beads.values()) == 8:
                reject('Capacity')
            if len(self.beads) >= 16:
                reject('ResourceCap')
            if self.frontier == I64:
                reject('ArithmeticOverflow')
            next_state.reservoir[lane] -= 1
            next_state.beads[self.frontier] = dict(id=self.frontier,site=0,species=4,activated=params,generation=self.revision+1)
            next_state.frontier += 1
        elif kind == 'out':
            if params not in self.beads:
                reject('MissingHandle')
            b = self.beads[params]
            if b['species'] != 4:
                reject('Species')
            if b['site'] != 0:
                reject('Boundary')
            lane = 0 if b['activated'] else 1
            if self.reservoir[lane] == I64:
                reject('ArithmeticOverflow')
            next_state.reservoir[lane] += 1
            del next_state.beads[params]
        else:
            raise ValueError('unknown reference action')
        next_state.revision += 1
        next_state.invariant()
        return next_state

    def mutant(self, name):
        m = self.prepare(('bond',[10,20,30])) if name in ['missing_heat','double_fuel_credit'] else deepcopy(self)
        if name == 'extra_unpaid_bead':
            m.beads[m.frontier] = dict(id=m.frontier,site=0,species=4,activated=False,generation=m.revision)
            m.frontier += 1
        elif name == 'missing_reservoir_material_debit':
            funded = self.prepare(('out',30))
            m = funded.prepare(('in',True))
            m.reservoir = funded.reservoir[:]
        elif name == 'missing_heat':
            m.heat = 0
        elif name == 'double_fuel_credit':
            m.beads[30]['activated'] = True
        elif name == 'negative_heat':
            m.heat = -1
        else:
            raise ValueError('unknown reference mutant')
        return m

def semantic_action(a):
    kind, p = next(iter(a.items()))
    if kind == 'FormBond':
        return ('bond',[p[k]['semantic_id'] for k in ('a','b','fuel')])
    if kind == 'TransferFuelIn':
        return ('in',p['activated'])
    if kind == 'TransferFuelOut':
        return ('out',p['fuel']['semantic_id'])
    raise ValueError('unknown native action')

def reference_program(contract, case):
    m = Model(contract,case['fixture'])
    records = []
    def record(phase, decision, action=None, reason=None, before=None):
        records.append(dict(phase=phase,decision=decision,action=action,reason=reason,before=before or m.snapshot(),after=m.snapshot()))
    def action(a):
        nonlocal m
        before = m.snapshot()
        try:
            m = m.prepare(a)
            record('prepare_commit','accepted',a,before=before)
        except Rejection as e:
            record('prepare_commit','rejected',a,str(e),before)
    def prepared(a):
        nxt = m.prepare(a)
        record('prepare','prepared',a)
        return nxt
    op = case['operation']
    if op.startswith('prepare_both_commit_first_then'):
        a,b = [('bond',x) for x in case['order']]
        p,q = prepared(a),prepared(b)
        before = m.snapshot();m=p;record('commit','accepted',a,before=before)
        record('commit','rejected',b,'StaleState')
        action(b)
    elif op in ['form_bond','form_bond_with_missing_handle']:
        a = ('bond',case['binding'])
        if case['layer'] == 'commit_fault':
            m.prepare(a)  # valid full staged transition; fault prevents publication
            record('fault_staging','rejected',a,'InjectedFault')
        else:
            action(a)
    elif op == 'transfer_in_then_out':
        ident = m.frontier
        action(('in',case['activated']))
        action(('out',ident))
    elif op == 'transfer_fuel_in':
        action(('in',case['activated']))
    elif op in ['transfer_bead_out','transfer_fuel_out']:
        action(('out',case['semantic_id']))
    elif op == 'prepare_then_intervening_bond_then_commit_and_reprepare':
        a = ('bond',case['binding']);prepared(a)
        action(('bond',case['intervening']))
        record('commit','rejected',a,'StaleState');action(a)
    elif op == 'prepare_then_transfer_out_and_in_then_try_old_handle':
        a = ('bond',case['binding']);prepared(a)
        action(('out',30));action(('in',True))
        record('commit','rejected',a,'StaleState');action(a)
    elif op == 'commit_prepared_in_distinct_world':
        m.prepare(('bond',[10,20,30]));record('cross_world_commit','rejected',('bond',[10,20,30]),'WorldIdentity')
    elif op in ['remove_node_without_full_incident_footprint','replace_activation_bool_with_int']:
        # Explicit generic edit and actual footprint/type evidence checked separately.
        record('generic_stage','rejected',reason='UnexpectedCleanup' if op.startswith('remove') else 'PropertyType')
    elif op == 'validate_mutant_poststate_without_publication':
        mutant = m.mutant(case['mutation'])
        try:
            mutant.invariant()
            raise ValueError('mutant unexpectedly balanced')
        except Rejection as e:
            record('invariant_check','rejected',reason=str(e))
    else:
        raise ValueError('reference script missing '+op)
    return m, records

# Decode tagged full arena wire independently. Never reconstruct a native graph.
def decode_raw(raw):
    graph = raw['graph']
    revision = raw['revision'];frontier=raw['frontier'];reservoir=raw['reservoir'];heat=raw['heat']
    def bad(test):
        if not test:
            reject('MalformedInitialState')
    bad(type(revision) is int and 0<=revision<=U64 and type(frontier) is int and 0<frontier<=I64)
    bad(all(type(i) is int and 0<=i<=I64 for i in reservoir) and type(heat) is int and 0<=heat<=I64)
    bad(graph['step']==revision and len(graph['nodes'])<=33 and len(graph['edges'])<=65)
    handles={};nodes={};property_maps={}
    for index, slot in enumerate(graph['nodes'][1:],1):
        n=slot['value']
        if n is None:
            continue
        properties=n['properties'];label=n['label']
        expected={SITE:{IDENT:'Int',SITE_INDEX:'Int'},BEAD:{IDENT:'Int',SPECIES:'Int',ACTIVE:'Bool'},BOND:{IDENT:'Int',KIND:'Int',PORT_A:'Int',PORT_B:'Int'}}.get(label)
        bad(expected is not None)
        keys=[p[0] for p in properties];bad(len(keys)==len(set(keys)) and set(keys)==set(expected))
        props={}
        for k, value in properties:
            bad(list(value)==[expected[k]])
            value=value[expected[k]]
            bad(type(value) is (bool if expected[k]=='Bool' else int))
            props[k]=value
        ident=props[IDENT];bad(0<ident<frontier and ident not in nodes)
        bad(n['energy']==0 and str(n['energy']) not in ['-0.0','-0'] and type(n['generation']) is int and 0<=n['generation']<=revision)
        handles[(index,slot['version'])]=ident;property_maps[ident]=props
        site=props.get(SITE_INDEX)
        species=props.get(SPECIES);activated=props.get(ACTIVE)
        if label==SITE:
            bad(site in [0,1,2] and ident==site+1 and n['generation']==0)
        if label==BEAD:
            bad(species in range(5) and (not activated or species==4))
        if label==BOND:
            bad(all(props[k]==0 for k in [KIND,PORT_A,PORT_B]))
        nodes[ident]=dict(id=ident,label=label,generation=n['generation'],species=species,activated=activated,site=site)
    bad(sorted(n['site'] for n in nodes.values() if n['label']==SITE)==[0,1,2])
    seen=set();edges=[];placements={};roles={};adj=set();incidents={k:[] for k in handles};labels={}
    for index, slot in enumerate(graph['edges'][1:],1):
        e=slot['value']
        if e is None:
            continue
        endpoints=[(h['idx'],h['version']) for h in e['nodes']]
        bad(len(endpoints)==2 and endpoints[0]!=endpoints[1] and all(h in handles for h in endpoints))
        pair=sorted(handles[h] for h in endpoints);label=e['label']
        bad((label,tuple(pair)) not in seen);seen.add((label,tuple(pair)))
        ns=[nodes[i] for i in pair]
        if label==ADJ:
            bad(all(n['label']==SITE for n in ns));adj.add(tuple(sorted(n['site'] for n in ns)))
        elif label==PLACE:
            bead=[n for n in ns if n['label']==BEAD];site=[n for n in ns if n['label']==SITE]
            bad(len(bead)==len(site)==1 and bead[0]['id'] not in placements);placements[bead[0]['id']]=site[0]['site']
        elif label in [ROLE_A,ROLE_B]:
            bond=[n for n in ns if n['label']==BOND];bead=[n for n in ns if n['label']==BEAD]
            bad(len(bond)==len(bead)==1 and bead[0]['species']==(0 if label==ROLE_A else 1))
            key=(bond[0]['id'],label);bad(key not in roles);roles[key]=bead[0]['id']
        else:
            reject('MalformedInitialState')
        key=dict(idx=index,version=slot['version'])
        for h in endpoints:
            incidents[h].append(key)
        labels.setdefault(str(label),[]).append(key)
        edges.append(dict(label=label,nodes=pair))
    bad(adj=={(0,1),(1,2)})
    bead_ids={n['id'] for n in nodes.values() if n['label']==BEAD}
    bad(set(placements)==bead_ids and len(bead_ids)<=16 and all(n<=8 for n in Counter(placements.values()).values()))
    used=set();bond_ids={n['id'] for n in nodes.values() if n['label']==BOND}
    for i in bond_ids:
        bad((i,ROLE_A) in roles and (i,ROLE_B) in roles)
        a,b=roles[i,ROLE_A],roles[i,ROLE_B]
        bad(a!=b and a not in used and b not in used and placements[a]==placements[b]);used.update([a,b])
    for i,s in placements.items():nodes[i]['site']=s
    bad(min(reservoir)>=0 and heat>=0)
    # Native secondary indexes are checked against independent edge scans.
    for h, expected in incidents.items():
        slot=graph['node_to_edges'][h[0]]
        bad(slot['version']==h[1] and sorted(slot['value'],key=lambda e:(e['idx'],e['version']))==expected)
    bad(graph['label_index']==labels)
    material=[sum(n['species']==s for n in nodes.values() if n['label']==BEAD) for s in range(5)]
    total=material[:];total[4]+=sum(reservoir)
    energy=4*sum(n['activated'] for n in nodes.values() if n['label']==BEAD)+2*len(bond_ids)
    return dict(nodes=sorted(nodes.values(),key=lambda n:n['id']),edges=sorted(edges,key=lambda e:encoded(e)),revision=revision,frontier=frontier,
                reservoir=reservoir,heat=heat,world_material=list(map(str,material)),total_material=list(map(str,total)),world_energy=str(energy),total_energy=str(energy+4*reservoir[0]+heat))

def check_malformed_recipe(raw, model, mutation):
    # Verify which defect was actually supplied, not merely that some defect exists.
    snapshot=model.snapshot();expected_nodes={}
    for n in snapshot['nodes']:
        props=[[IDENT,{'Int':n['id']}]]
        if n['label']==SITE:props += [[SITE_INDEX,{'Int':n['site']}]]
        elif n['label']==BEAD:props += [[SPECIES,{'Int':n['species']}],[ACTIVE,{'Bool':n['activated']}]]
        else:props += [[KIND,{'Int':0}],[PORT_A,{'Int':0}],[PORT_B,{'Int':0}]]
        expected_nodes[n['id']]=dict(label=n['label'],generation=n['generation'],energy=0.0,properties=props)
    expected_edges=deepcopy(snapshot['edges']);frontier=model.frontier;reservoir=model.reservoir[:]
    if mutation=='missing_property':
        expected_nodes[30]['properties']=[p for p in expected_nodes[30]['properties'] if p[0]!=ACTIVE]
    elif mutation=='wrong_property_type':
        expected_nodes[30]['properties'][-1]=[ACTIVE,{'Int':1}]
    elif mutation=='duplicate_property':
        expected_nodes[30]['properties'].append([ACTIVE,{'Bool':True}])
    elif mutation=='dangling_endpoint':
        expected_edges.append(dict(label=PLACE,nodes=[0,30]))
    elif mutation=='extra_untyped_edge':
        expected_edges.append(dict(label=7,nodes=[30,30]))
    elif mutation=='overoccupied_site':
        for i in range(frontier,frontier+6):
            expected_nodes[i]=dict(label=BEAD,generation=0,energy=0.0,properties=[[IDENT,{'Int':i}],[SPECIES,{'Int':4}],[ACTIVE,{'Bool':False}]])
            expected_edges.append(dict(label=PLACE,nodes=[1,i]))
        frontier+=6
    elif mutation=='negative_reservoir':
        reservoir[0]=-1
    else:raise ValueError('unknown malformed recipe')
    nodes={};handles={}
    for idx,slot in enumerate(raw['graph']['nodes']):
        if slot['value'] is not None:
            n=slot['value'];ident=next(p[1]['Int'] for p in n['properties'] if p[0]==IDENT)
            require(ident not in nodes,'malformed recipe duplicate semantic ID')
            nodes[ident]=n;handles[idx,slot['version']]=ident
    edges=[dict(label=slot['value']['label'],nodes=sorted(handles.get((h['idx'],h['version']),0) for h in slot['value']['nodes']))
           for slot in raw['graph']['edges'] if slot['value'] is not None]
    require(nodes==expected_nodes and sorted(edges,key=encoded)==sorted(expected_edges,key=encoded),'declared malformed recipe mismatch')
    require(raw['revision']==model.revision and raw['frontier']==frontier and raw['reservoir']==reservoir and raw['heat']==model.heat,'malformed initial private fields')

def check_observation(obs, expected):
    require(obs['semantic']==expected,'semantic reference state mismatch')
    require(obs['exact_sha256']==digest(obs['exact']),'exact state hash mismatch')
    require(len(encoded(obs['exact']))<=1_048_576,'exact state cap')
    state=obs['exact']['state']
    raw=dict(graph=state['graph'],revision=state['revision'],frontier=state['semantic_id_frontier'],reservoir=state['reservoir'],heat=state['heat'])
    require(decode_raw(raw)==expected,'full native graph differs from semantic projection')
    require(state['baseline_material']==expected['total_material'] and state['baseline_energy']==expected['total_energy'],'immutable baselines')
    for arena in ['nodes','edges']:
        slots=state['graph'][arena];alloc=obs['exact']['allocator'][arena]
        free=[i for i,s in enumerate(slots[1:],1) if s['value'] is None]
        require(alloc['isolated'] is False and alloc['next']==[len(slots),2] and sorted(alloc['free'][0])==free and alloc['free'][1]==[],'allocator inventory/frontier')
        require(len(alloc['free'][0])==len(set(alloc['free'][0])),'duplicate free slot')
        require(all((s['version']%2==1)==(s['value'] is not None) for s in slots[1:]),'generational slot occupation')
    return state

def projection(s):
    beads=[{k:n[k] for k in ('id','species','activated','site')} for n in s['nodes'] if n['label']==BEAD]
    bonds={n['id']:{} for n in s['nodes'] if n['label']==BOND}
    for edge in s['edges']:
        if edge['label'] in [ROLE_A,ROLE_B]:
            i=next(i for i in edge['nodes'] if i in bonds);other=next(i2 for i2 in edge['nodes'] if i2!=i)
            bonds[i]['a' if edge['label']==ROLE_A else 'b']=other
    return dict(beads=beads,bonds=sorted(bonds.values(),key=lambda b:(b['a'],b['b'])),reservoir=s['reservoir'],heat=s['heat'],world_energy=s['world_energy'],total_energy=s['total_energy'],material=s['total_material'])

def audit(contract_path, native_path):
    started=time.monotonic();c=json.loads(Path(contract_path).read_bytes());r=json.loads(Path(native_path).read_bytes())
    require(r['contract_sha256']==hashlib.sha256(Path(contract_path).read_bytes()).hexdigest(),'contract binding')
    require(r['cases']==41 and len(r['variants'])==82,'fixed denominator')
    require(r['status']=='executed_pending_independent_audit' and r['cap_failure'] is None,'complete uncapped execution')
    require(r['physical_time'] is None and r['rng'] is None and r['seeds']==[] and r['independent_scientific_worlds']==0,'scope boundary')
    for f in FLAGS:require(r.get(f) is False,'explicit false '+f)
    accepted=rejected=prepared=observations=0;case_results=[];finals={}
    for index,case in enumerate(c['cases']):
        pair=r['variants'][index*2:index*2+2]
        require([v['workers'] for v in pair]==[1,4] and all(v['id']==case['id'] for v in pair),'frozen case/worker order')
        require({k:v for k,v in pair[0].items() if k!='workers'}=={k:v for k,v in pair[1].items() if k!='workers'},'same-order exact worker identity')
        require(sum(len(encoded(v)) for v in pair)<=2*c['caps']['artifact_bytes_per_case'],'artifact cap')
        for v in pair:
            require(time.monotonic()-started<300,'audit time cap')
            expected=case['expected']
            if 'accepted' in expected:require(v['accepted']==expected['accepted'],'accepted count')
            if 'rejected' in expected:require(v['rejected']==expected['rejected'],'ordered frozen rejections')
            if 'rejection' in expected:require(v['rejected']==[expected['rejection']],'frozen rejection')
            if case['layer']=='initial_admission':
                check_malformed_recipe(v['raw_initial'],Model(c,case['fixture']),case['mutation'])
                try:
                    decode_raw(v['raw_initial']);raise ValueError('malformed admitted by independent validator')
                except Rejection as e:
                    require(str(e)=='MalformedInitialState','malformed reason')
                require(v['accepted']==0 and not v['attempts'],'no malformed execution');rejected+=1
                continue
            initial=Model(c,case['fixture']);check_observation(v['initial'],initial.snapshot());observations+=1
            final,program=reference_program(c,case);attempts=v['attempts']
            require(len(attempts)==len(program)<=32,'complete offer script')
            journal=[];previous=v['initial']
            for actual,ref in zip(attempts,program):
                require(actual['phase']==ref['phase'] and actual['decision']==ref['decision'],'decision/phase mismatch')
                if ref['reason']:require(actual['reason']==ref['reason'],'reference rejection mismatch')
                if ref['action']:require(semantic_action(actual['action'])==ref['action'],'proposal selection/order')
                require(actual['before']==previous,'exact trajectory continuity')
                bs=check_observation(actual['before'],ref['before']);after=check_observation(actual['after'],ref['after']);observations+=2
                previous=actual['after']
                if actual['decision']!='accepted':
                    require(actual['before']['exact']==actual['after']['exact'],'rejection/preparation changed native graph/allocator/ledger/journal')
                    rejected+=actual['decision']=='rejected';prepared+=actual['decision']=='prepared'
                else:
                    accepted+=1;event=actual['event'];journal.append(event)
                    require(event['revision']==ref['after']['revision'] and event['event_id']==case['id']+':'+str(event['revision']),'semantic event identity')
                    require(event['before_sha256']==actual['before']['exact_sha256'],'event before binding')
                    require(event['action']==actual['action'],'event proposal binding')
                    delta={}
                    for arena in ['nodes','edges']:
                        left=bs['graph'][arena];right=after['graph'][arena]
                        delta[arena]=[dict(slot_index=i,before=left[i] if i<len(left) else None,after=right[i] if i<len(right) else None)
                                      for i in range(1,max(len(left),len(right))) if (left[i] if i<len(left) else None)!=(right[i] if i<len(right) else None)]
                    delta['step']=dict(before=bs['graph']['step'],after=after['graph']['step'])
                    require(event['graph_delta']==delta,'exact tagged graph/property delta')
                    require(event['heat_delta']==ref['after']['heat']-ref['before']['heat'],'heat delta')
                    require(event['reservoir_delta']==[a-b for a,b in zip(ref['after']['reservoir'],ref['before']['reservoir'])],'reservoir delta')
                    allocation=ref['before']['frontier'] if ref['action'][0] in ['in','bond'] else None
                    require(event['allocated_semantic_id']==allocation,'semantic allocation')
                require(after['accepted_event_journal']==journal,'accepted journal mismatch')
                require(after['world_identity']==case['id'] and after['law']==c['law_version'] and after['profile']==c['profile'],'immutable identity/law/profile')
            require(v['final']==previous,'complete final exact state/journal')
            check_observation(v['final'],final.snapshot());observations+=1
            require(v['accepted']==sum(x['decision']=='accepted' for x in program),'independent accepted count')
            require(v['rejected']==[x['reason'] for x in program if x['decision']=='rejected'],'independent rejection list')
            if case['layer']=='invariant_check':
                mutant=initial.mutant(case['mutation']);obs=v['diagnostic']['mutant']
                require(obs['semantic']==mutant.snapshot() and obs['exact_sha256']==digest(obs['exact']),'actual mutant/reference')
                state=obs['exact']['state']
                require(state['baseline_material']==list(map(str,initial.baseline[0])) and state['baseline_energy']==str(initial.baseline[1]),'mutant baseline must remain immutable')
                raw=dict(graph=state['graph'],revision=state['revision'],frontier=state['semantic_id_frontier'],reservoir=state['reservoir'],heat=max(0,state['heat']))
                adjusted=deepcopy(mutant.snapshot());adjusted['heat']=max(0,state['heat']);adjusted['total_energy']=str(int(adjusted['total_energy'])+adjusted['heat']-state['heat'])
                require(decode_raw(raw)==adjusted,'full native mutant differs from reference')
                require(v['diagnostic']['validation']==expected['rejection'],'mutant predicate decision')
            if case['id']=='deleted_and_reused_fuel_handle':
                old=v['diagnostic']['old_handle']['node'];new=v['diagnostic']['new_handle']['node']
                require(old['idx']==new['idx'] and new['version']==old['version']+2,'same arena index/new generation')
            if case['id']=='cross_world_commit':
                require(v['diagnostic']['other_before']==v['diagnostic']['other_after'],'cross-world rejection identity')
            finals[case['id']]=final.snapshot()
        case_results.append(dict(id=case['id'],variants=2,pass_exact=True))
    require(projection(finals['disjoint_forward'])==projection(finals['disjoint_reverse']),'reversed disjoint physical composition')
    require(len(encoded(r))<=c['caps']['total_output_bytes'] and r['native_seconds']<300,'native output/time cap')
    return dict(schema=1,profile='constructive-chemistry-cc1-independent-audit-v1',gate_pass=True,engineering_only=True,cases=41,worker_variants=82,
                malformed_variants=14,accepted_commits=accepted,rejected_attempts=rejected,prepared_offers=prepared,observations_checked=observations,
                same_order_worker_exact_identity=True,reversed_disjoint_physical_identity=True,native_allocator_rejection_identity=True,
                independent_dictionary_reference=True,case_results=case_results,audit_seconds=time.monotonic()-started,
                contract_sha256=r['contract_sha256'],native_sha256=hashlib.sha256(Path(native_path).read_bytes()).hexdigest(),
                **{f:False for f in FLAGS})

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('contract');p.add_argument('native');p.add_argument('--output')
    args=p.parse_args()
    result=audit(args.contract,args.native)
    output=json.dumps(result,indent=2,sort_keys=True)+'\n'
    if args.output:
        with open(args.output,'x') as f:f.write(output)
    else:print(output,end='')
