#!/usr/bin/env python3
"""Independent one-shot source proposal, escrow, native lifecycle audit."""
from collections import Counter
import argparse
import copy
import hashlib
import json
from pathlib import Path
import sys
import tomllib
import importlib.util

PROFILE='evolving-minds-d0-mutation-v1'
LAW_SHA='0656c1341373e1fa3bd31118d4e4b8865810478d7d89cf744a0548514312c474'
ESCROW=0xE1020000
REQUEST,TX=242014,242015
FUNDS,PLACE=ESCROW+100,ESCROW+101
LENGTH,PARTS=242012,242013
PHASES=['environment_','world_','sense_0','sense_1']+[f'prepare_{i}_' for i in range(2,8)]+['commit_','development_','attempt_','settle_','maintenance_','reserve_','transaction_','escrow_','death_']+['cleanup_']*26

def load(path,name):
    spec=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(spec);old=sys.dont_write_bytecode
    try:sys.dont_write_bytecode=True;spec.loader.exec_module(m)
    finally:sys.dont_write_bytecode=old
    return m

def cost(n):return 1.0+(11+n)/4
def expected_program(name,d):return dict(instructions=[dict(id=100,op='Halt',slot=0,source=0,initial=False)]) if name=='inactive' else d.program(name)
def boundary(status,req):
    req(status['mutation_mechanism_enabled'] is True,'mutation mechanism boundary')
    req(status['profile']==PROFILE and status['development_only'] is True and status['engineering_execution_complete'] is True,'family completion')
    req(all(status[k] is False for k in ['scientific_run_launched','gate_pass','training_admitted','evolution_enabled']),'family scientific boundary')

def checkpoint(path,req):
    b=path.read_bytes();req(24<=len(b)<=16*1024*1024+24 and b[:8]==b'HGEMM002' and int.from_bytes(b[8:16],'little')==len(b)-24,'family checkpoint envelope')
    h=0xcbf29ce484222325
    for x in b[24:]:h=((h^x)*0x100000001b3)&((1<<64)-1)
    req(h==int.from_bytes(b[16:24],'little'),'family checkpoint checksum')


def kernel_name(k):return dict(Faithful='faithful',SourceKernel='source',Coin='coin',Invalid='invalid',MissingPort='missing_port')[k]
def energy(nodes,c):return c.total(nodes)+sum(c.value(n,c.ENERGY,'Float') for n in nodes.values() if n['label']==ESCROW)
def typed_valid(p):
    gs=p['instructions']
    if not 1<=len(gs)<=8 or len({g['id'] for g in gs})!=len(gs) or gs[-1]['op']!='Halt':return False
    for i,g in enumerate(gs):
        op=g['op'];slot=g['slot'];src=g['source'];initial=g['initial']
        if not 0<=slot<=7 or not 0<=src<=7:return False
        if op=='State' and not(slot>=2 and src==0):return False
        if op=='Actuator' and not(slot>=2 and src==0 and not initial):return False
        if op=='Wire' and initial:return False
        if op=='Halt' and not(slot==src==0 and not initial and i==len(gs)-1):return False
    return True
def reachable(p):
    parts={0:'Sensor',1:'Sensor'};writers=set();act=False
    for g in p['instructions']:
        op=g['op'];dst=g['slot'];src=g['source']
        if op in ['State','Actuator']:
            if dst in parts or (op=='Actuator' and act):return False
            parts[dst]=op;act|=op=='Actuator'
        elif op=='Wire':
            if src not in parts or dst not in parts or src==dst or parts[dst]=='Sensor' or dst in writers:return False
            writers.add(dst)
    return True
class Sampler:
    def __init__(self,seed,kernel):self.state=seed^0xD0B17A5E;self.index=0;self.kernel=kernel
    def bounded(self,n,raw):
        mask=(1<<64)-1;threshold=((1<<64)-n)%n
        while True:
            self.state=(self.state+0x9e3779b97f4a7c15)&mask;z=self.state;z=((z^(z>>30))*0xbf58476d1ce4e5b9)&mask;z=((z^(z>>27))*0x94d049bb133111eb)&mask;z^=z>>31;self.index+=1;raw.append(z)
            if z>=threshold:return z%n
    def proposal(self,p):
        q=copy.deepcopy(p);raw=[];start=self.index;edit=None
        if self.kernel=='SourceKernel':
            if self.bounded(4,raw)!=0:
                ids=sorted(g['id'] for g in p['instructions']);target=ids[self.bounded(len(ids),raw)];old=next(g['source'] for g in p['instructions'] if g['id']==target);edit=(target,(old+1+self.bounded(7,raw))%8)
        elif self.kernel!='Faithful':
            op='Halt' if self.kernel=='Invalid' else 'Wire';target=min(g['id'] for g in p['instructions'] if g['op']==op);edit=(target,7 if self.kernel=='MissingPort' else 1)
        if edit:
            target,new=edit;g=next(g for g in q['instructions'] if g['id']==target);old=g['source'];g['source']=new;return q,target,old,new,raw,start
        return q,None,None,None,raw,start

def proposal_law(record,rules,d):
    name=f'transaction_copy_{len(record["proposed_program"]["instructions"])}' if record['syntax_valid'] else 'transaction_reject'
    # Always recover the immutable template, not a previous request's specialization.
    law=copy.deepcopy(rules['_templates'][name])
    if record['syntax_valid'] and record['target_id'] is not None:
        pos=next(i for i,g in enumerate(record['proposed_program']['instructions']) if g['id']==record['target_id'])
        effect=next(f for f in law['rhs']['property_effects'] if f['var']==f'cg{pos}' and f['key']==d.SRC)
        effect.pop('source_var');effect.pop('source_key');effect['op']='set';effect['value']=record['new_source']
    return law

def neighborhood(p,d):
    rows=[]
    for g in p['instructions']:
        for src in range(8):
            if src==g['source']:continue
            q=copy.deepcopy(p);next(x for x in q['instructions'] if x['id']==g['id'])['source']=src;valid=typed_valid(q)
            rows.append(dict(target_id=g['id'],old_source=g['source'],new_source=src,syntax_valid=valid,material_reachable=valid and reachable(q),proposed_bytes=list(d.canonical(q))))
    return dict(program=p,scope='grammar_projection_only',neighbors=rows)

def verify_cell(directory,c,d,s,base,name,mode,kernel,neighborhood=None,endowment=None):
    budget=12.0 if endowment is None else float(endowment)
    if endowment is not None:c.require(endowment in [12,24] and neighborhood is not None,"funding fixed budget/profile")
    req=c.require;p=expected_program(name,d);table=s.schedule(17,mode);req(json.loads((directory/'program.json').read_text())==p,'family program');req(json.loads((directory/'schedule.json').read_text())==table,'family schedule')
    raw=(directory/'rules.toml').read_bytes();req(raw.startswith(base),'family fixed physics');laws=tomllib.loads(raw.decode())['rules'];rules={r['name']:r for r in laws};req(len(rules)==len(laws)==len(tomllib.loads(base.decode())['rules'])+512,'family laws inventory')
    for site in range(8):
        for t in range(64):
            expected=dict(name=f'world_{site}_{t}',priority=0,lhs=dict(nodes=['site'],edges=[],node_labels=dict(site=c.SITE),node_property_guards=[dict(var='site',property=d.CLOCK,op='eq',value=t),dict(var='site',property=241009,op='eq',value=site)]),rhs=dict(nodes=['site'],edges=[],property_effects=[dict(var='site',key=k,op='set',value=v) for k,v in [(d.RESOURCE,table['resources'][t]),(d.CUE,table['cues'][t]),(d.COIN,table['coins'][site][t])]]+[dict(var='site',key=d.CLOCK,op='add',value=1)]))
            req(rules[expected['name']]==expected,'family world law identity')
    rules['_templates']=copy.deepcopy(rules)
    first=json.loads((directory/'initial.json').read_text());req(first['step']==first['tick']==0 and first['phase'] is None and not first['events'],'family initial clock');nodes,edges=c.graphs(first)
    sites={c.value(n,241009,'Int'):h for h,n in nodes.items() if n['label']==c.SITE};counter=next(h for h,n in nodes.items() if n['label']==c.COUNTER);root=next(h for h,n in nodes.items() if n['label']==c.ROOT)
    req(set(sites)==set(range(8)) and len(nodes)==20+len(p['instructions']) and all(n['generation']==0 for n in nodes.values()),'family initial inventory')
    class Ref(d.Reference):
        def material(self):return 2.0+len(self.p)/8+(len(self.parts)-2+len(self.wires))/4
    rng=Sampler(17,kernel);requests=[];pending=None
    refs={1:Ref(p,table,64.0,c)};root_handles={1:root};ancestry={1:dict(id=1,parent=None,born_tick=0,death_tick=None,site=0,program=p)};dead={};death_reserve={};death_cause={};site_reserves={i:32.0 for i in range(8)};influx=copy_sink=0.0;births=deaths=0;clock=0
    def hydrate(bid):
        r=refs[bid];site=ancestry[bid]['site'];r.site=site_reserves[site];r.clock=clock
        if clock:r.resource=table['resources'][clock-1];r.cue=table['cues'][clock-1];r.coin=table['coins'][site][clock-1]
        return r
    def live():return [bid for bid,b in ancestry.items() if b['death_tick'] is None]
    def material():
        expected=set();all_members={};occupied={ancestry[bid]['site'] for bid in live()}
        def key(label,a,b):return label,tuple(sorted([a,b]))
        for i in range(8):
            expected.add(key(c.NEIGHBOR,sites[i],sites[(i+1)%8]));n=nodes[sites[i]];props={c.ENERGY:site_reserves[i],d.CLOCK:clock,241009:i,d.RESOURCE:table['resources'][clock-1] if clock else False,d.CUE:table['cues'][clock-1] if clock else False,d.COIN:table['coins'][i][clock-1] if clock else False,c.OCCUPIED:i in occupied}
            req(n['label']==c.SITE and n['energy']==0 and len(n['properties'])==len(props) and dict(n['properties'])=={k:c.typed(v) for k,v in props.items()},'family physical site state')
        req(nodes[counter]['label']==c.COUNTER and nodes[counter]['energy']==0 and dict(nodes[counter]['properties'])=={c.NEXT:c.typed(len(ancestry)+1),REQUEST:c.typed(len(requests)+1)},'family monotone birth counter')
        for bid in live():
            r=hydrate(bid);h=root_handles[bid];site=sites[ancestry[bid]['site']];members={k for k,n in nodes.items() if n['label'] not in [c.ROOT,c.SITE,c.DEAD,c.COUNTER,ESCROW] and c.value(n,c.OWNER,'Int')==bid};all_members[bid]=members
            req(nodes[h]['label']==c.ROOT and c.value(nodes[h],c.BID,'Int')==bid and c.value(nodes[h],c.PARENT,'Int')==(ancestry[bid]['parent'] or 0) and c.value(nodes[h],LENGTH,'Int')==len(r.p) and c.value(nodes[h],PARTS,'Int')==len(members),'family immutable ancestry/genome/count')
            subset=members|{h,site};small={k:copy.deepcopy(nodes[k]) for k in subset};small_edges={k:copy.deepcopy(e) for k,e in edges.items() if all(c.handle(n) in subset for n in e['nodes'])}
            small[h]['properties']=[x for x in small[h]['properties'] if x[0] not in [c.PARENT,LENGTH,PARTS]];c.set_property(small[h],c.BID,{'Int':1});small[site]['properties']=[x for x in small[site]['properties'] if x[0]!=c.OCCUPIED];c.set_property(small[site],241009,{'Int':0})
            for k in members:c.set_property(small[k],c.OWNER,{'Int':1})
            d.material(small,small_edges,r,c);expected|={c.edge_key(e) for e in small_edges.values()}
        for bid,snapshot in dead.items():
            h=root_handles[bid];members={k for k,n in nodes.items() if n['label'] not in [c.ROOT,c.SITE,c.DEAD,c.COUNTER,ESCROW] and c.value(n,c.OWNER,'Int')==bid};all_members[bid]=members
            n=copy.deepcopy(snapshot[h]);c.set_property(n,PARTS,{'Int':len(members)});req(nodes[h]==n,'family tombstone settlement/disassembly')
            for k in members:req(nodes[k]==snapshot[k],'family dead material changed');expected.add(key(d.OWN,h,k))
            for e in edges.values():
                endpoints={c.handle(k) for k in e['nodes']}
                if endpoints<=members:expected.add(c.edge_key(e))
        supported=set(sites.values())|{counter}|set(root_handles.values())|set().union(*all_members.values()) if all_members else set(sites.values())|{counter}|set(root_handles.values())
        escrows={h for h,n in nodes.items() if n['label']==ESCROW}
        if pending:
            parent,target,eh=pending;r=requests[-1];req(escrows=={eh},'mutation escrow inventory')
            props={c.ENERGY:0.0 if r['settled'] else budget+cost(len(r['proposed_program']['instructions'])),c.OWNER:parent,LENGTH:len(r['proposed_program']['instructions']),REQUEST:r['request_id'],TX:0 if not r['settled'] else 1 if r['child_id'] else 2}
            req(nodes[eh]['energy']==0.0 and dict(nodes[eh]['properties'])=={k:c.typed(v) for k,v in props.items()},'mutation escrow reserve/status')
            expected.add(key(FUNDS,root_handles[parent],eh));expected.add(key(PLACE,eh,sites[target]));supported.add(eh)
        else:req(not escrows,'mutation hidden escrow')
        req(set(nodes)==supported,'mutation orphan or hidden material')
        actual={c.edge_key(e) for e in edges.values()};req(len(actual)==len(edges) and actual==expected and all(len(e['nodes'])==2 and len({c.handle(h) for h in e['nodes']})==2 for e in edges.values()),'family global role/owner/scaffold grammar')
        req(len(nodes)<=256 and len(edges)<=768,'family material ceiling')
    material();req(energy(nodes,c)==320.0,'family initial budgets')
    entries=[json.loads(x) for x in (directory/'journal.jsonl').read_text().splitlines()];req(len(entries)==2880,'family complete journal coverage');rows=[];work=0
    for index,entry in enumerate(entries):
        tick=index//45+1;phase=index%45;req(entry['tick']==tick and entry['pass']==phase and entry['phase']==PHASES[phase] and entry['step']==index+1,'family journal clock/pass')
        expected=[];winner=None;cleanup_bindings={}
        if phase==0:
            for i in range(8):
                if site_reserves[i]<=56:site_reserves[i]+=8;influx+=8;expected.append('environment_influx')
        elif phase==1:expected=[f'world_{i}_{clock}' for i in range(8)];clock+=1
        elif 2<=phase<=14:
            for bid in live():
                r=hydrate(bid);expected+=r.step(phase,tick);site_reserves[ancestry[bid]['site']]=r.site
        elif phase==15:
            if neighborhood is not None:
                for length in range(1,9):
                    key=f'reserve_{length}';rules[key]=copy.deepcopy(rules['_templates'][key])
                    if requests:rules[key]['lhs']['node_property_guards'].append(dict(var='r',property=c.BID,op='gte',value=2))
            candidates=[]
            for bid in live():
                r=refs[bid];home=ancestry[bid]['site'];length=len(r.p)
                if r.status!=2 or r.age<2 or r.reserve<1+budget+cost(length):continue
                if neighborhood is not None and requests and bid==1:continue
                gs={c.value(n,d.POSITION,'Int'):h for h,n in nodes.items() if n['label']==c.GENE and c.value(n,c.OWNER,'Int')==bid}
                for target in [(home-1)%8,(home+1)%8]:
                    if target in {ancestry[b]['site'] for b in live()}:continue
                    binding={'r':root_handles[bid],'site':sites[home],'target':sites[target],'counter':counter,**{f'g{i}':gs[i] for i in range(length)}};bound=[binding[v] for v in rules[f'reserve_{length}']['lhs']['nodes']];candidates.append(((length,sorted(map(c.rank,bound))),bid,target,bound))
            if candidates:
                _,parent,target,bound=min(candidates);winner=(parent,target,bound);expected=[f'reserve_{len(refs[parent].p)}']
        elif phase==16:
            if pending:
                r=requests[-1];name_settle=f'transaction_copy_{len(r["proposed_program"]["instructions"])}' if r['syntax_valid'] else 'transaction_reject';expected=[name_settle]
                law=proposal_law(r,rules,d);req(tomllib.loads(r['settlement_rule'])=={'rules':[law]},'mutation independently derived settlement law');rules[name_settle]=law
        elif phase==17:
            if pending:expected=['escrow_dispose']
        elif phase==18:
            for bid in live():
                r=refs[bid]
                if r.age>=16 or r.reserve<=0:
                    cause='death_age' if r.age>=16 else 'death_empty';expected.append(cause);death_reserve[bid]=r.reserve;death_cause[bid]=cause;site_reserves[ancestry[bid]['site']]+=r.reserve;r.reserve=0.0;ancestry[bid]['death_tick']=tick;deaths+=1
        else:
            for bid in dead:
                h=root_handles[bid];members=[k for k,n in nodes.items() if n['label'] not in [c.ROOT,c.DEAD,c.SITE,c.COUNTER,ESCROW] and c.value(n,c.OWNER,'Int')==bid]
                if members:
                    # Native canonical key orders equal-priority candidates by
                    # local rule ID before sorted handles (hg-observe Binding).
                    order=[c.SENSOR,c.GENE,c.CONNECTION,c.ACTUATOR,d.STATE,d.SLOT]
                    part=min(members,key=lambda k:(order.index(nodes[k]['label']),sorted(map(c.rank,[h,k]))));rule=f'cleanup_member_{nodes[part]["label"]}';cleanup_bindings[h]=[h,part]
                else:rule='cleanup_root';cleanup_bindings[h]=[h]
                expected.append(rule)
        req(Counter(e['rule'] for e in entry['events'])==Counter(expected),f'family mandatory mechanism/birth/cleanup coverage at step {index+1} pass {phase}: expected {expected}')
        if winner:req(list(map(c.handle,entry['events'][0]['bound']))==winner[2],'family canonical birth contention')
        engaged=set()
        for event in entry['events']:
            bound=list(map(c.handle,event['bound']));req(not set(bound)&engaged,'family conflicting events');engaged|=set(bound)
            if phase>=19:req(bound==cleanup_bindings[bound[0]],'family canonical disassembly')
            binding=c.apply(nodes,edges,event,rules[event['rule']],index)
            if phase==15:
                parent,target,_=winner;program=ancestry[parent]['program'];refs[parent].reserve-=1.0+budget+cost(len(refs[parent].p));copy_sink+=1.0
                enumerated=neighborhood is not None and not requests
                if enumerated:
                    proposed=copy.deepcopy(program);target_id=neighborhood['target_id'];new=neighborhood['new_source'];old=None
                    if target_id is not None:
                        g=next(g for g in proposed['instructions'] if g['id']==target_id);old=g['source'];g['source']=new
                    raw=[];draw_start=rng.index
                else:proposed,target_id,old,new,raw,draw_start=rng.proposal(program)
                valid=typed_valid(proposed);record=dict(request_id=len(requests)+1,tick=tick,parent=parent,site=target,operator=kernel_name(kernel),version=1,stream='splitmix64-source-v1:17',draw_start=draw_start,draw_end=rng.index,raw_draws=raw,target_id=target_id,old_source=old,new_source=new,parent_program=copy.deepcopy(program),proposed_program=proposed,parent_bytes=list(d.canonical(program)),proposed_bytes=list(d.canonical(proposed)),syntax_valid=valid,invalid_reason=None if valid else 'D0 typed parameter/order grammar',material_reachable=valid and reachable(proposed),request_charge=1.0,copying_charge=0.0,refunded=0.0,child_id=None,settled=False)
                if enumerated:
                    record['operator']='enumerated_source' if target_id is not None else 'unchanged_control';record['stream']='enumerated-source-v1'
                recorded=json.loads((directory/'mutations.json').read_text());req(len(recorded)>len(requests),'mutation omitted request');record['settlement_rule']=recorded[len(requests)]['settlement_rule'];requests.append(record);pending=(parent,target,binding['escrow'])
            elif phase==16:
                parent,target,_=pending;r=requests[-1];r['settled']=True
                if r['syntax_valid']:
                    bid=len(ancestry)+1;r['child_id']=bid;r['copying_charge']=cost(len(r['proposed_program']['instructions']));copy_sink+=r['copying_charge'];births+=1;refs[bid]=Ref(r['proposed_program'],table,budget,c);root_handles[bid]=binding['child'];ancestry[bid]=dict(id=bid,parent=parent,born_tick=tick,death_tick=None,site=target,program=copy.deepcopy(r['proposed_program']))
                else:
                    r['refunded']=budget+cost(len(r['proposed_program']['instructions']));refs[parent].reserve+=r['refunded']
            elif phase==17:pending=None
            elif phase==18:
                h=binding['r'];bid=c.value(nodes[h],c.BID,'Int');dead[bid]={k:copy.deepcopy(n) for k,n in nodes.items() if k==h or (n['label'] not in [c.ROOT,c.DEAD,c.SITE,c.COUNTER,ESCROW] and c.value(n,c.OWNER,'Int')==bid)}
            elif phase>=19 and event['rule']=='cleanup_root':
                bid=next(b for b,h in root_handles.items() if h==binding['r']);del root_handles[bid];del dead[bid]
        material();dissipated=copy_sink+sum(r.dissipated for r in refs.values());req(energy(nodes,c)==320.0+influx-dissipated,'family conservation/copy ledger')
        stats=entry['stats'];req(stats['property_effects_skipped']==0 and stats['accepted_rewrites']==len(entry['events']) and stats['property_effects_applied']==sum(len(rules[e['rule']]['rhs'].get('property_effects',[])) for e in entry['events']),'family stats/effects')
        req(all(stats[field]==sum(len(e[field if field.startswith('edges') else 'created' if field=='nodes_created' else 'deleted']) for e in entry['events']) for field in ['nodes_created','nodes_deleted','edges_created','edges_deleted']),'family stats/material events')
        req(isinstance(entry['match_work'],int) and 0<=entry['match_work']<=1000000,'family work budget');work+=entry['match_work']
        if phase==44:
            req(not dead and pending is None and all(not r.pending for r in refs.values()),'family completed-tick settlement');depth={}
            for bid,b in ancestry.items():depth[bid]=depth[b['parent']]+1 if b['parent'] else 0
            row=dict(tick=tick,engine_step=tick*45,live_organisms=len(live()),births=births,deaths=deaths,max_ancestry_depth=max(depth.values()),development_paid=sum(r.paid for r in refs.values()),development_failures=sum(r.dev_failed for r in refs.values()),attempts=sum(r.attempts for r in refs.values()),successes=sum(r.successes for r in refs.values()),failures=sum(r.failures for r in refs.values()),harvested=sum(r.harvested for r in refs.values()),influx=influx,dissipated=dissipated,total_energy=energy(nodes,c),residual=0.0,match_work=work);rows.append(row);work=0
    final=json.loads((directory/'final.json').read_text());req(final['step']==2880 and final['tick']==64 and final['phase'] is None and not final['events'] and c.graphs(final)==(nodes,edges),'family exact final graph')
    req(json.loads((directory/'rows.json').read_text())==rows,'family reconstructed rows');req(json.loads((directory/'lineage.json').read_text())=={str(k):v for k,v in ancestry.items()},'family ancestry/copy identity')
    req(json.loads((directory/'mutations.json').read_text())==requests,'mutation complete proposals/settlements')
    req(json.loads((directory/'mutation_state.json').read_text())==dict(kernel=kernel,seed=17,allocation_reversed=False,rng_state=rng.state,draw_index=rng.index,records=requests,pending=None),'mutation exact stream/history state')
    status=json.loads((directory/'run_status.json').read_text())
    if neighborhood is None:
        boundary(status,req);req(status['program']==name and status['cue_mode']==mode and status['kernel']==kernel and status['final_row']==rows[-1],'family cell summary')
    else:
        req(status['profile']==('evolving-minds-d0-neighborhood-v1' if endowment is None else 'evolving-minds-d0-funding-v1') and status['development_only'] is True and status['engineering_execution_complete'] is True and status['mutation_mechanism_enabled'] is True,'neighborhood completion')
        req(all(status[k] is False for k in ['scientific_run_launched','gate_pass','training_admitted','evolution_enabled','reproductive_fairness_accepted']),'neighborhood research boundary')
        req(status['recipe']==neighborhood and status['final_row']==rows[-1], 'neighborhood recipe/summary')
    if endowment is not None:req(status['endowment']==endowment and json.loads((directory/'endowment.json').read_text())==endowment,'funding cell reserve identity')
    m=json.loads((directory/'manifest.json').read_text());req(m['state']=='Completed' and m['seed']==17 and m['max_steps']==m['final_step']==2880,'family cell manifest')
    if neighborhood is None:
        req({p.name for p in directory.glob('*.checkpoint')}=={'initial.checkpoint','mutation.checkpoint','midpoint.checkpoint','final.checkpoint'},'family checkpoint coverage')
        for path in directory.glob('*.checkpoint'):checkpoint(path,req)
    else:
        req(not list(directory.glob('*.checkpoint')),'diagnostic checkpoint exclusion')
    req((directory/'journal.jsonl').stat().st_size<=16*1024*1024,'family journal budget')
    if neighborhood is not None:
        lifetimes={str(bid):dict(id=bid,born_tick=ancestry[bid]['born_tick'],death_tick=ancestry[bid]['death_tick'],death_cause=death_cause.get(bid),paid_construction_completed=r.status==2,development_paid=r.paid,development_failures=r.dev_failed,attempts=r.attempts,successes=r.successes,failures=r.failures,harvested=r.harvested,maintenance_paid=r.dissipated-r.paid-r.attempts,last_age=r.age,reserve_at_death=death_reserve.get(bid)) for bid,r in refs.items() if bid!=1}
        req(json.loads((directory/'lifetimes.json').read_text())==lifetimes,'neighborhood independent child lifetimes')
        req(requests and requests[0]['parent']==1 and sum(r['parent']==1 for r in requests)==1, 'single founder request/no fallback')
        first=requests[0];child=first['child_id'];reproduced=child is not None and any(r['parent']==child and r['child_id'] is not None for r in requests);alive=sum(b['death_tick'] is None for bid,b in ancestry.items() if bid!=1)
        outcome=dict(recipe=neighborhood,syntax_valid=first['syntax_valid'],material_reachable=first['material_reachable'],first_request_tick=first['tick'],first_child=lifetimes.get(str(child)),first_child_reproduced=reproduced,descendants=max(0,len(ancestry)-2),live_inherited_at_64=alive,max_inherited_depth=max(depth.values()),lineage_harvested=sum(r.harvested for bid,r in refs.items() if bid!=1),sustained_turnover=reproduced and alive>0,final_row=rows[-1])
        req(json.loads((directory/'outcome.json').read_text())==outcome,'neighborhood independently derived outcome')
        return status,outcome
    return status,dict(final_row=rows[-1],genotypes_verified=len(ancestry),committed_births_verified=births,requests_verified=len(requests),invalid_requests_verified=sum(not r["syntax_valid"] for r in requests),runtime_unreachable_commits=sum(r["syntax_valid"] and not r["material_reachable"] for r in requests),draws_verified=rng.index)

def verify(root):
    root=Path(root);c=load(root/'replay.py','family_replay');d=load(root/'d0_reference.py','family_d0');s=load(root/'schedule_reference.py','family_schedule');req=c.require
    req(root.is_dir() and not root.is_symlink() and not any(p.is_symlink() for p in root.rglob('*')),'family evidence path');req(sum(p.stat().st_size for p in root.rglob('*') if p.is_file())<=128*1024*1024,'family panel output cap')
    base=(root/'base_rules.toml').read_bytes();req(hashlib.sha256(base).hexdigest()==LAW_SHA,'mutation law identity');cases=[('cue','Faithful'),('cue','SourceKernel'),('constant1','SourceKernel'),('cue','Coin'),('cue','Invalid'),('cue','MissingPort')];req({p.name for p in root.iterdir() if p.is_dir()}=={f'{kernel_name(k)}_{n}' for n,k in cases},'mutation fixed six-cell denominator');results={};statuses=[]
    for name,kernel in cases:
        status,result=verify_cell(root/f'{kernel_name(kernel)}_{name}',c,d,s,base,name,'Informative',kernel);statuses.append(status);results[f'{kernel_name(kernel)}_{name}']=result
    req(json.loads((root/'summary.json').read_text())==statuses,'mutation panel summary')
    req(results['invalid_cue']['requests_verified']>0 and results['invalid_cue']['committed_births_verified']==0,'mutation rejection control')
    req(results['coin_cue']['committed_births_verified']>0 and results['missing_port_cue']['runtime_unreachable_commits']>0 and results['missing_port_cue']['final_row']['development_failures']>0,'mutation changed/runtime-failure controls')
    neighborhoods=[neighborhood(d.program(n),d) for n in ['cue','constant1']];req(json.loads((root/'neighborhoods.json').read_text())==neighborhoods,'mutation complete grammar neighborhoods')
    boundary(json.loads((root/'run_status.json').read_text()),req);m=json.loads((root/'manifest.json').read_text());req(m['state']=='Completed' and m['max_steps']==m['final_step']==17280,'mutation panel completion')
    sources=json.loads((root/'sources.json').read_text())
    for f,p in [('checker.py','scripts/verify_evolving_minds_mutation.py'),('d0_reference.py','scripts/verify_evolving_minds_d0.py'),('replay.py','scripts/verify_evolving_minds.py'),('schedule_reference.py','scripts/verify_evolving_minds_choice.py'),('base_rules.toml','rules/evolving_minds/mutation_v1.toml'),('contract.md','experiments/evolving_minds/contracts/d0_mutation_v1.md')]:req((root/f).read_text()==sources[p],'mutation frozen source binding')
    return dict(profile=PROFILE,audit_pass=True,development_only=True,scientific_run_launched=False,gate_pass=False,training_admitted=False,evolution_enabled=False,mutation_mechanism_enabled=True,cells_verified=6,native_advances_verified=17280,source_sha256={k:hashlib.sha256(v.encode()).hexdigest() for k,v in sources.items()},cells=results)

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('directory',type=Path);print(json.dumps(verify(p.parse_args().directory),indent=2))
