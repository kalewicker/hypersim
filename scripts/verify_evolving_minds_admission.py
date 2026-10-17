#!/usr/bin/env python3
"""Independent semantic lottery, parallel escrow and full lifecycle audit."""
from collections import Counter
import argparse
import copy
import hashlib
import json
from pathlib import Path
import sys
import tomllib
import importlib.util

PROFILE='evolving-minds-d0-admission-v1'
LAW_SHA='66a3b76760eb55398830bcf8e409c9b5675b1051b56239329bee0dc7c43a5edd'
ESCROW=0xE1020000
REQUEST,TX=242014,242015
FUNDS,PLACE=ESCROW+100,ESCROW+101
LENGTH,PARTS,LOCAL=242012,242013,242018
PHASES=['environment_','world_','identity_','sense_0','sense_1']+[f'prepare_{i}_' for i in range(2,8)]+['commit_','development_','attempt_','settle_','maintenance_','reserve_','transaction_','escrow_','death_']+['cleanup_']*26

def load(path,name):
    spec=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(spec);old=sys.dont_write_bytecode
    try:sys.dont_write_bytecode=True;spec.loader.exec_module(m)
    finally:sys.dont_write_bytecode=old
    return m

def cost(n):return 1.0+(11+n)/4
def expected_program(name,d):return dict(instructions=[dict(id=100,op='Halt',slot=0,source=0,initial=False)]) if name=='inactive' else d.program(name)
def boundary(status,req,profile=PROFILE):
    req(status['profile']==profile and status['development_only'] is True and status['engineering_execution_complete'] is True,'family completion')
    req(all(status[k] is False for k in ['scientific_run_launched','gate_pass','training_admitted','evolution_enabled','reproductive_fairness_accepted']),'family scientific boundary')

def checkpoint(path,req):
    b=path.read_bytes();req(24<=len(b)<=16*1024*1024+24 and b[:8]==b'HGEMA001' and int.from_bytes(b[8:16],'little')==len(b)-24,'family checkpoint envelope')
    h=0xcbf29ce484222325
    for x in b[24:]:h=((h^x)*0x100000001b3)&((1<<64)-1)
    req(h==int.from_bytes(b[16:24],'little'),'family checkpoint checksum')


def kernel_name(k):return dict(Faithful='faithful',SourceKernel='source',Coin='coin',Invalid='invalid',MissingPort='missing_port',GrammarSourceKernel='grammar_source')[k]
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
def grammar_candidates(p):
    rows=[]
    for g in sorted(p['instructions'],key=lambda x:x['id']):
        for source in range(8):
            if source==g['source']:continue
            q=copy.deepcopy(p);next(x for x in q['instructions'] if x['id']==g['id'])['source']=source
            if typed_valid(q):rows.append((g['id'],source,q))
    return rows

def grammar_certificate(r,d):
    import struct
    candidates=grammar_candidates(r['parent_program'])
    h=hashlib.sha256(b'HG-TA-GRAMMAR-CANDIDATES-1')
    for identity,source,p in candidates:h.update(struct.pack('<IB',identity,source));h.update(d.canonical(p))
    gate=r['raw_draws'][0]%4!=0
    index=next((i for i,(identity,source,_) in enumerate(candidates) if identity==r['target_id'] and source==r['new_source']),None)
    if bool(gate and candidates)!=(index is not None):raise ValueError('grammar certificate selected alternative')
    n=len(candidates)
    return dict(request_id=r['request_id'],candidate_count=n,candidate_sha256=h.hexdigest(),
        mutation_gate_selected=gate,selected_index=index,identity_probability_numerator=1,
        identity_probability_denominator=4 if n else 1,each_candidate_probability_numerator=3 if n else 0,
        each_candidate_probability_denominator=4*n if n else 1)

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
        elif self.kernel=='GrammarSourceKernel':
            if self.bounded(4,raw)!=0:
                candidates=grammar_candidates(p)
                if candidates:
                    target,new,_=candidates[self.bounded(len(candidates),raw)];edit=(target,new)
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

MASK=(1<<64)-1
def mix(x):
    z=(x+0x9e3779b97f4a7c15)&MASK;z=((z^(z>>30))*0xbf58476d1ce4e5b9)&MASK;z=((z^(z>>27))*0x94d049bb133111eb)&MASK;return z^(z>>31)
def logical(site,recipe):return (site-recipe['rotation'])%8
def lottery(seed,tick,offers):
    offers=sorted(offers,key=lambda o:(o['parent'],o['home'],o['target'],o['length']));stream=mix(seed^0x41444d495353494f^tick);state=stream;raw=[];order=list(range(len(offers)))
    for i in range(len(offers)-1,0,-1):
        n=i+1;threshold=((1<<64)-n)%n
        while True:
            state=(state+0x9e3779b97f4a7c15)&MASK;z=state;z=((z^(z>>30))*0xbf58476d1ce4e5b9)&MASK;z=((z^(z>>27))*0x94d049bb133111eb)&MASK;z^=z>>31;raw.append(z)
            if z>=threshold:break
        order[i],order[z%n]=order[z%n],order[i]
    parents=set();sites=set();selected=[]
    for i in order:
        o=offers[i]
        if o['parent'] in parents or o['home'] in sites or o['target'] in sites:continue
        parents.add(o['parent']);sites.update([o['home'],o['target']]);selected.append(i)
    return dict(tick=tick,stream_seed=stream,offers=offers,raw_draws=raw,permutation=order,selected=selected)
def specialize(law,name,parent,target=None,request=None):
    law=copy.deepcopy(law);law['name']=name
    for var,key,v in [('r',240007,parent),('target',241009,target),('escrow',REQUEST,request)]:
        if v is not None:law['lhs']['node_property_guards'].append(dict(var=var,property=key,op='eq',value=v))
    return law

FUNDED_PROFILE='evolving-minds-d0-funded-admission-v1'
def validate_funded_config(cfg,stress=False):
    def require(ok):
        if not ok:raise ValueError('funded configuration envelope')
    require(set(cfg)=={'seed','endowment','founders','kernels','home_offsets','rotation','reversed','horizon','scheduler'})
    require(type(cfg['seed']) is int and 0<=cfg['seed']<2**64 and cfg['endowment'] in [12,24] and type(cfg['endowment']) is int)
    require(len(cfg['founders'])==len(cfg['kernels'])==len(cfg['home_offsets'])==2 and all(x in ['Delayed','Immediate','Coin'] for x in cfg['founders']) and all(x in (['Faithful','SourceKernel','Coin','Invalid','MissingPort']+(['GrammarSourceKernel'] if stress else [])) for x in cfg['kernels']))
    require(all(type(x) is int and 0<=x<8 for x in cfg['home_offsets']) and len(set(cfg['home_offsets']))==2 and type(cfg['rotation']) is int and 0<=cfg['rotation']<8 and type(cfg['reversed']) is bool and type(cfg['horizon']) is int and cfg['horizon']==64 and cfg['scheduler']=='semantic-lottery-v1')
def funded_program(program,d):
    p=copy.deepcopy(d.program('delayed_cue'));next(g for g in p['instructions'] if g['id']==103)['source']={'Delayed':2,'Immediate':0,'Coin':1}[program];return p
def temporal_schedule(seed, environment, reference):
    if set(environment)!={'pattern','cue_mode'} or environment['pattern'] not in ['Shuffled','Alternating','Blocks4'] or environment['cue_mode'] not in ['Informative','Permuted','Withheld']:
        raise ValueError('temporal environment envelope')
    table=reference.schedule(seed,environment['cue_mode'])
    phase=bool(mix(seed ^ 0x5441315f50484153)&1)
    if environment['pattern']!='Shuffled':
        width=1 if environment['pattern']=='Alternating' else 4
        table['resources']=[bool((t//width)%2)^phase for t in range(64)]
    if environment['cue_mode']=='Informative':table['cues']=list(table['resources'])
    return table

def verify_cell(directory,c,d,s,base,recipe,checkpoints=True,final_tick=64,funded_config=None,retained_policy=None,handoff_config=None,descendant_config=None,transfer_environment=None,mutation_stress_policy=None):
    req=c.require;budget=12.0 if funded_config is None else float(funded_config['endowment']);seed=17 if funded_config is None else funded_config['seed']
    if funded_config is not None:
        validate_funded_config(funded_config,stress=mutation_stress_policy is not None);req(recipe==dict(case='faithful_disjoint',rotation=funded_config['rotation'],reversed=funded_config['reversed']),'funded coordinate adapter');req(json.loads((directory/'funded_config.json').read_text())==funded_config,'funded config identity')
    if mutation_stress_policy is not None:
        req(mutation_stress_policy=='grammar-source-stress-v1' and transfer_environment is not None and
            retained_policy is None and handoff_config is None and descendant_config is None and funded_config is not None,'stress policy binding')
        req(funded_config['endowment']==24 and funded_config['kernels'][0]==funded_config['kernels'][1] and
            funded_config['kernels'][0] in ['Faithful','SourceKernel','GrammarSourceKernel'] and
            (funded_config['founders']==['Immediate']*2 or (funded_config['founders']==['Delayed']*2 and funded_config['kernels']==['Faithful']*2)),'stress founder/kernel envelope')
    retained={};descendant_targets={}
    if descendant_config is not None:
        req(funded_config is not None and retained_policy is None and handoff_config is None,'descendant proposal policy conflict')
        req(set(descendant_config)=={'policy','target_id','new_source'} and descendant_config['policy']=='first-paid-descendant-by24-v1' and funded_config['endowment']==24 and funded_config['founders']==['Delayed']*2 and funded_config['kernels']==['Faithful']*2,'descendant configuration binding')
        target,new=descendant_config['target_id'],descendant_config['new_source']
        req((target is None and new is None) or (type(target) is int and target in range(100,105) and type(new) is int and new in range(1,8)),'descendant fixed source substitution')
    if handoff_config is not None:
        req(funded_config is not None and retained_policy is None and set(handoff_config)=={'policy','supplied','descendant_kernel'} and handoff_config['policy']=='paid-beneficial-edit-v1' and handoff_config['supplied'] in ['Useful','Unchanged','Invalid','Unreachable'] and handoff_config['descendant_kernel'] in ['Faithful','SourceKernel'] and funded_config['founders']==['Delayed']*2 and funded_config['kernels']==[handoff_config['descendant_kernel']]*2,'handoff config binding')
    if retained_policy is not None:
        req(retained_policy=='founder-first-proposal-v1' and funded_config is not None,'retained policy binding')
    ps=[d.program('cue'),d.program('delayed_cue' if recipe['case']=='mixed_competing' else 'cue')] if funded_config is None else [funded_program(n,d) for n in funded_config['founders']];table=s.schedule(seed,'Informative') if transfer_environment is None else temporal_schedule(seed,transfer_environment,s);req(json.loads((directory/'recipe.json').read_text())==recipe,'admission recipe');req(json.loads((directory/'schedule.json').read_text())==table,'family schedule')
    raw=(directory/'rules.toml').read_bytes();req(raw.startswith(base),'family fixed physics');laws=tomllib.loads(raw.decode())['rules'];rules={r['name']:r for r in laws};req(len(rules)==len(laws)==len(tomllib.loads(base.decode())['rules'])+512,'family laws inventory')
    for site in range(8):
        for t in range(64):
            expected=dict(name=f'world_{site}_{t}',priority=0,lhs=dict(nodes=['site'],edges=[],node_labels=dict(site=c.SITE),node_property_guards=[dict(var='site',property=d.CLOCK,op='eq',value=t),dict(var='site',property=241009,op='eq',value=site)]),rhs=dict(nodes=['site'],edges=[],property_effects=[dict(var='site',key=k,op='set',value=v) for k,v in [(d.RESOURCE,table['resources'][t]),(d.CUE,table['cues'][t]),(d.COIN,table['coins'][site][t])]]+[dict(var='site',key=d.CLOCK,op='add',value=1)]))
            req(rules[expected['name']]==expected,'family world law identity')
    rules['_templates']=copy.deepcopy(rules)
    first=json.loads((directory/'initial.json').read_text());req(first['step']==first['tick']==0 and first['phase'] is None and not first['events'],'family initial clock');nodes,edges=c.graphs(first)
    sites={c.value(n,241009,'Int'):h for h,n in nodes.items() if n['label']==c.SITE};root_handles={c.value(n,c.BID,'Int'):h for h,n in nodes.items() if n['label']==c.ROOT}
    req(set(sites)==set(range(8)) and len(nodes)==30+sum(len(p['instructions']) for p in ps) and set(root_handles)=={1,2} and not any(n['label']==c.COUNTER for n in nodes.values()) and all(n['generation']==0 for n in nodes.values()),'family initial inventory')
    class Ref(d.Reference):
        def material(self):return 2.0+len(self.p)/8+(len(self.parts)-2+len(self.wires))/4
    requests=[];pending={};rounds=[];identity_clock=0
    homes=[recipe['rotation'],(recipe['rotation']+(2 if recipe['case']=='mixed_competing' else 4))%8] if funded_config is None else [(h+funded_config['rotation'])%8 for h in funded_config['home_offsets']]
    refs={i+1:Ref(p,table,64.0,c) for i,p in enumerate(ps)};ancestry={i+1:dict(id=i+1,parent=None,born_tick=0,death_tick=None,site=homes[i],program=p) for i,p in enumerate(ps)};dead={};site_reserves={i:32.0 for i in range(8)};influx=copy_sink=0.0;births=deaths=0;clock=0
    def hydrate(bid):
        r=refs[bid];site=ancestry[bid]['site'];r.site=site_reserves[site];r.clock=clock
        if clock:r.resource=table['resources'][clock-1];r.cue=table['cues'][clock-1];r.coin=table['coins'][site][clock-1]
        return r
    def live():return [bid for bid,b in ancestry.items() if b['death_tick'] is None]
    def material():
        expected=set();all_members={};occupied={ancestry[bid]['site'] for bid in live()}
        def key(label,a,b):return label,tuple(sorted([a,b]))
        for i in range(8):
            expected.add(key(c.NEIGHBOR,sites[i],sites[(i+1)%8]));n=nodes[sites[i]];props={c.ENERGY:site_reserves[i],d.CLOCK:clock,241009:i,d.RESOURCE:table['resources'][clock-1] if clock else False,d.CUE:table['cues'][clock-1] if clock else False,d.COIN:table['coins'][i][clock-1] if clock else False,c.OCCUPIED:i in occupied,LOCAL:3+logical(i,recipe)+8*identity_clock}
            req(n['label']==c.SITE and n['energy']==0 and len(n['properties'])==len(props) and dict(n['properties'])=={k:c.typed(v) for k,v in props.items()},'family physical site state')
        for bid in live():
            r=hydrate(bid);h=root_handles[bid];site=sites[ancestry[bid]['site']];members={k for k,n in nodes.items() if n['label'] not in [c.ROOT,c.SITE,c.DEAD,c.COUNTER,ESCROW] and c.value(n,c.OWNER,'Int')==bid};all_members[bid]=members
            req(nodes[h]['label']==c.ROOT and c.value(nodes[h],c.BID,'Int')==bid and c.value(nodes[h],c.PARENT,'Int')==(ancestry[bid]['parent'] or 0) and c.value(nodes[h],LENGTH,'Int')==len(r.p) and c.value(nodes[h],PARTS,'Int')==len(members),'family immutable ancestry/genome/count')
            subset=members|{h,site};small={k:copy.deepcopy(nodes[k]) for k in subset};small_edges={k:copy.deepcopy(e) for k,e in edges.items() if all(c.handle(n) in subset for n in e['nodes'])}
            small[h]['properties']=[x for x in small[h]['properties'] if x[0] not in [c.PARENT,LENGTH,PARTS]];c.set_property(small[h],c.BID,{'Int':1});small[site]['properties']=[x for x in small[site]['properties'] if x[0] not in [c.OCCUPIED,LOCAL]];c.set_property(small[site],241009,{'Int':0})
            for k in members:c.set_property(small[k],c.OWNER,{'Int':1})
            d.material(small,small_edges,r,c);expected|={c.edge_key(e) for e in small_edges.values()}
        for bid,snapshot in dead.items():
            h=root_handles[bid];members={k for k,n in nodes.items() if n['label'] not in [c.ROOT,c.SITE,c.DEAD,c.COUNTER,ESCROW] and c.value(n,c.OWNER,'Int')==bid};all_members[bid]=members
            n=copy.deepcopy(snapshot[h]);c.set_property(n,PARTS,{'Int':len(members)});req(nodes[h]==n,'family tombstone settlement/disassembly')
            for k in members:req(nodes[k]==snapshot[k],'family dead material changed');expected.add(key(d.OWN,h,k))
            for e in edges.values():
                endpoints={c.handle(k) for k in e['nodes']}
                if endpoints<=members:expected.add(c.edge_key(e))
        supported=set(sites.values())|set(root_handles.values())|set().union(*all_members.values()) if all_members else set(sites.values())|set(root_handles.values())
        escrows={h for h,n in nodes.items() if n['label']==ESCROW}
        req(escrows=={eh for _,_,eh in pending.values()},'admission escrow inventory')
        for rid,(parent,target,eh) in pending.items():
            r=next(r for r in requests if r['request_id']==rid)
            props={c.ENERGY:0.0 if r['settled'] else budget+cost(len(r['proposed_program']['instructions'])),c.OWNER:parent,LENGTH:len(r['proposed_program']['instructions']),REQUEST:rid,TX:0 if not r['settled'] else 1 if r['child_id'] else 2}
            req(nodes[eh]['energy']==0.0 and dict(nodes[eh]['properties'])=={k:c.typed(v) for k,v in props.items()},'admission escrow reserve/status')
            expected.add(key(FUNDS,root_handles[parent],eh));expected.add(key(PLACE,eh,sites[target]));supported.add(eh)
        req(set(nodes)==supported,'mutation orphan or hidden material')
        actual={c.edge_key(e) for e in edges.values()};req(len(actual)==len(edges) and actual==expected and all(len(e['nodes'])==2 and len({c.handle(h) for h in e['nodes']})==2 for e in edges.values()),'family global role/owner/scaffold grammar')
        req(len(nodes)<=256 and len(edges)<=768,'family material ceiling')
    material();req(energy(nodes,c)==384.0,'family initial budgets')
    entries=[json.loads(x) for x in (directory/'journal.jsonl').read_text().splitlines()];req(len(entries)==final_tick*46,'family complete journal coverage');rows=[];work=0
    for index,entry in enumerate(entries):
        tick=index//46+1;phase=index%46;req(entry['tick']==tick and entry['pass']==phase and entry['phase']==PHASES[phase] and entry['step']==index+1,'family journal clock/pass')
        expected=[];winners={};cleanup_bindings={}
        if phase==0:
            for i in range(8):
                if site_reserves[i]<=56:site_reserves[i]+=8;influx+=8;expected.append('environment_influx')
        elif phase==1:expected=[f'world_{i}_{clock}' for i in range(8)];clock+=1
        elif phase==2:expected=['identity_advance']*8;identity_clock+=1
        elif 3<=phase<=15:
            for bid in live():
                r=hydrate(bid);expected+=r.step(phase-1,tick);site_reserves[ancestry[bid]['site']]=r.site
        elif phase==16:
            offers=[]
            for bid in live():
                r=refs[bid];home=ancestry[bid]['site'];length=len(r.p)
                if r.status!=2 or r.age<2 or r.reserve<1+budget+cost(length):continue
                for target in [(home-1)%8,(home+1)%8]:
                    if target not in {ancestry[b]['site'] for b in live()}:offers.append(dict(parent=bid,home=logical(home,recipe),target=logical(target,recipe),length=length))
            round_=lottery(seed,tick,offers);rounds.append(round_)
            for i in round_['selected']:
                o=round_['offers'][i];parent=o['parent'];target=(o['target']+recipe['rotation'])%8;length=o['length'];name=f'reserve_selected_{parent}_{target}'
                law=specialize(rules[f'reserve_{length}'],name,parent,target);rules[name]=law;gs={c.value(n,d.POSITION,'Int'):h for h,n in nodes.items() if n['label']==c.GENE and c.value(n,c.OWNER,'Int')==parent};binding={'r':root_handles[parent],'site':sites[ancestry[parent]['site']],'target':sites[target],**{f'g{i}':gs[i] for i in range(length)}};bound=[binding[v] for v in law['lhs']['nodes']];winners[name]=(parent,target,bound);expected.append(name)
        elif phase==17:
            for rid in sorted(pending):
                r=next(r for r in requests if r['request_id']==rid);name=f'transaction_selected_{rid}';law=proposal_law(r,rules,d);law=specialize(law,name,r['parent'],r['site'],rid);rules[name]=law;expected.append(name);r['settlement_rule']=name
        elif phase==18:expected=['escrow_dispose']*len(pending)
        elif phase==19:
            for bid in live():
                r=refs[bid]
                if r.age>=16 or r.reserve<=0:
                    expected.append('death_age' if r.age>=16 else 'death_empty');site_reserves[ancestry[bid]['site']]+=r.reserve;r.reserve=0.0;ancestry[bid]['death_tick']=tick;deaths+=1
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

        early_parents={}
        if descendant_config is not None and phase==16 and tick<=24:
            for parent,_,_ in winners.values():
                founder=parent
                while ancestry[founder]['parent']:founder=ancestry[founder]['parent']
                if parent!=founder and str(founder) not in descendant_targets:
                    early_parents[founder]=min(parent,early_parents.get(founder,parent))
        engaged=set()
        for event in entry['events']:
            bound=list(map(c.handle,event['bound']));req(not set(bound)&engaged,'family conflicting events');engaged|=set(bound)
            if phase>=20:req(bound==cleanup_bindings[bound[0]],'family canonical disassembly')
            binding=c.apply(nodes,edges,event,rules[event['rule']],index)
            if phase==16:
                parent,target,want=winners[event['rule']];req(bound==want,'admission native selected binding');program=ancestry[parent]['program'];refs[parent].reserve-=1+budget+cost(len(refs[parent].p));copy_sink+=1.0
                rid=3+logical(target,recipe)+8*tick;founder=parent
                while ancestry[founder]['parent']:founder=ancestry[founder]['parent']
                kernel=(('Invalid' if founder==1 else 'SourceKernel') if recipe['case']=='mixed_settlement' else 'Faithful') if funded_config is None else funded_config['kernels'][founder-1];stream=mix(seed^0x4d55544154494f4e^rid);rng=Sampler(stream,kernel)
                if retained_policy is not None and (parent!=founder or str(founder) in retained):kernel='Faithful';rng=Sampler(stream,kernel)
                if handoff_config is not None and parent==founder:kernel='Faithful';rng=Sampler(stream,kernel)
                proposed,target_id,old,new,raw,draw_start=rng.proposal(program);valid=typed_valid(proposed);record=dict(request_id=rid,tick=tick,parent=parent,site=target,operator=kernel_name(kernel),version=1,stream=f'splitmix64-request-local-v1:{stream}',draw_start=0,draw_end=rng.index,raw_draws=raw,target_id=target_id,old_source=old,new_source=new,parent_program=copy.deepcopy(program),proposed_program=proposed,parent_bytes=list(d.canonical(program)),proposed_bytes=list(d.canonical(proposed)),syntax_valid=valid,invalid_reason=None if valid else 'D0 typed parameter/order grammar',material_reachable=valid and reachable(proposed),request_charge=1.0,copying_charge=0.0,refunded=0.0,child_id=None,settled=False,settlement_rule='')
                if retained_policy is not None and parent==founder:
                    first=retained.get(str(founder))
                    if first is None:retained[str(founder)]=copy.deepcopy(record)
                    else:
                        req(record['parent_program']==first['parent_program'],'retained founder genome unchanged')
                        for key in ['proposed_program','proposed_bytes','target_id','old_source','new_source','syntax_valid','invalid_reason','material_reachable']:record[key]=copy.deepcopy(first[key])
                        record.update(operator='retained_founder',stream=f"retained-founder-v1:{first['request_id']}",raw_draws=[],draw_start=0,draw_end=0)
                if (handoff_config is not None or descendant_config is not None) and parent==founder:
                    req(program==funded_program('Delayed',d),'handoff unchanged founder genome')
                    q=copy.deepcopy(program);supplied=handoff_config['supplied'] if descendant_config is None else 'Useful';edit=None
                    if supplied in ['Useful','Unreachable']:edit=(103,0 if supplied=='Useful' else 7)
                    elif supplied=='Invalid':edit=(next(g['id'] for g in q['instructions'] if g['op']=='Halt'),1)
                    target_id=old=new=None
                    if edit is not None:
                        target_id,new=edit;g=next(g for g in q['instructions'] if g['id']==target_id);old=g['source'];g['source']=new
                    valid=typed_valid(q)
                    record.update(operator='supplied_founder',stream='paid-beneficial-edit-v1:'+supplied,draw_start=0,draw_end=0,raw_draws=[],target_id=target_id,old_source=old,new_source=new,proposed_program=q,proposed_bytes=list(d.canonical(q)),syntax_valid=valid,invalid_reason=None if valid else 'D0 typed parameter/order grammar',material_reachable=valid and reachable(q))
                if descendant_config is not None and early_parents.get(founder)==parent:
                    req(str(founder) not in descendant_targets and tick<=24 and program==funded_program('Immediate',d),'first early descendant parent identity')
                    q=copy.deepcopy(program);target_id=descendant_config['target_id'];new=descendant_config['new_source'];old=0 if target_id is not None else None
                    if target_id is not None:next(g for g in q['instructions'] if g['id']==target_id)['source']=new
                    valid=typed_valid(q)
                    label=f'{target_id}:{new}' if target_id is not None else 'unchanged:unchanged'
                    record.update(operator='supplied_descendant',stream='first-paid-descendant-by24-v1:'+label,draw_start=0,draw_end=0,raw_draws=[],target_id=target_id,old_source=old,new_source=new,proposed_program=q,proposed_bytes=list(d.canonical(q)),syntax_valid=valid,invalid_reason=None if valid else 'D0 typed parameter/order grammar',material_reachable=valid and reachable(q))
                    descendant_targets[str(founder)]=copy.deepcopy(record)
                requests.append(record);requests.sort(key=lambda r:r['request_id']);pending[rid]=(parent,target,binding['escrow'])
            elif phase==17:
                rid=c.value(nodes[binding['escrow']],REQUEST,'Int');parent,target,_=pending[rid];r=next(r for r in requests if r['request_id']==rid);r['settled']=True
                if r['syntax_valid']:
                    bid=rid;r['child_id']=bid;r['copying_charge']=cost(len(r['proposed_program']['instructions']));copy_sink+=r['copying_charge'];births+=1;refs[bid]=Ref(r['proposed_program'],table,budget,c);root_handles[bid]=binding['child'];ancestry[bid]=dict(id=bid,parent=parent,born_tick=tick,death_tick=None,site=target,program=copy.deepcopy(r['proposed_program']))
                else:r['refunded']=budget+cost(len(r['proposed_program']['instructions']));refs[parent].reserve+=r['refunded']
            elif phase==18:
                rid=next(rid for rid,(_,_,h) in pending.items() if h==binding['escrow']);del pending[rid]
            elif phase==19:
                h=binding['r'];bid=c.value(nodes[h],c.BID,'Int');dead[bid]={k:copy.deepcopy(n) for k,n in nodes.items() if k==h or (n['label'] not in [c.ROOT,c.DEAD,c.SITE,c.COUNTER,ESCROW] and c.value(n,c.OWNER,'Int')==bid)}
            elif phase>=20 and event['rule']=='cleanup_root':
                bid=next(b for b,h in root_handles.items() if h==binding['r']);del root_handles[bid];del dead[bid]
        material();dissipated=copy_sink+sum(r.dissipated for r in refs.values());req(energy(nodes,c)==384.0+influx-dissipated,'family conservation/copy ledger')
        stats=entry['stats'];req(stats['property_effects_skipped']==0 and stats['accepted_rewrites']==len(entry['events']) and stats['property_effects_applied']==sum(len(rules[e['rule']]['rhs'].get('property_effects',[])) for e in entry['events']),'family stats/effects')
        req(all(stats[field]==sum(len(e[field if field.startswith('edges') else 'created' if field=='nodes_created' else 'deleted']) for e in entry['events']) for field in ['nodes_created','nodes_deleted','edges_created','edges_deleted']),'family stats/material events')
        req(isinstance(entry['match_work'],int) and 0<=entry['match_work']<=1000000,'family work budget');work+=entry['match_work']
        if phase==45:
            req(not dead and not pending and all(not r.pending for r in refs.values()),'family completed-tick settlement');depth={}
            for bid,b in ancestry.items():depth[bid]=depth[b['parent']]+1 if b['parent'] else 0
            row=dict(tick=tick,engine_step=tick*46,live_organisms=len(live()),births=births,deaths=deaths,max_ancestry_depth=max(depth.values()),development_paid=sum(r.paid for r in refs.values()),development_failures=sum(r.dev_failed for r in refs.values()),attempts=sum(r.attempts for r in refs.values()),successes=sum(r.successes for r in refs.values()),failures=sum(r.failures for r in refs.values()),harvested=sum(r.harvested for r in refs.values()),influx=influx,dissipated=dissipated,total_energy=energy(nodes,c),residual=0.0,match_work=work);rows.append(row);work=0
    final=json.loads((directory/'final.json').read_text());req(final['step']==final_tick*46 and final['tick']==final_tick and final['phase'] is None and not final['events'] and c.graphs(final)==(nodes,edges),'family exact final graph')
    req(json.loads((directory/'rows.json').read_text())==rows,'family reconstructed rows');req(json.loads((directory/'lineage.json').read_text())=={str(k):v for k,v in ancestry.items()},'family ancestry/copy identity')
    req(json.loads((directory/'admission_state.json').read_text())==dict(recipe=recipe,rounds=rounds,records=requests),'admission complete inventory/lottery/proposal/settlement history')
    status=json.loads((directory/'run_status.json').read_text());boundary(status,req,PROFILE if funded_config is None else FUNDED_PROFILE);req(status['recipe']==recipe and status['requests']==len(requests) and status['final_row']==(rows[-1] if rows else None),'admission cell summary')
    m=json.loads((directory/'manifest.json').read_text());req(m['state']=='Completed' and m['seed']==seed and m['max_steps']==m['final_step']==final_tick*46,'family cell manifest')
    if checkpoints:
        req({p.name for p in directory.glob('*.checkpoint')}=={'initial.checkpoint','admission.checkpoint','midpoint.checkpoint','final.checkpoint'},'family checkpoint coverage')
        for path in directory.glob('*.checkpoint'):checkpoint(path,req)
    req((directory/'journal.jsonl').stat().st_size<=16*1024*1024,'family journal budget')
    return status,dict(**(dict(descendant=dict(config=descendant_config,targets=descendant_targets)) if descendant_config is not None else {}),**(dict(handoff_config=handoff_config) if handoff_config is not None else {}),**(dict(retained=dict(policy=retained_policy,samples=retained)) if retained_policy is not None else {}),final_row=rows[-1] if rows else None,genotypes_verified=len(ancestry),committed_births_verified=births,requests_verified=len(requests),invalid_requests_verified=sum(not r['syntax_valid'] for r in requests),parallel_admission_ticks=[r['tick'] for r in rounds if len(r['selected'])>1],mixed_length_parallel_ticks=[r['tick'] for r in rounds if len({r['offers'][i]['length'] for i in r['selected']})>1],mixed_validity_parallel_ticks=[tick for tick in range(1,65) if len({r['syntax_valid'] for r in requests if r['tick']==tick})>1])

def verify(root):
    root=Path(root);c=load(root/'replay.py','admission_replay');d=load(root/'d0_reference.py','admission_d0');s=load(root/'schedule_reference.py','admission_schedule');req=c.require
    req(root.is_dir() and not root.is_symlink() and not any(p.is_symlink() for p in root.rglob('*')),'admission evidence path');req(sum(p.stat().st_size for p in root.rglob('*') if p.is_file())<=256*1024*1024,'admission output cap')
    base=(root/'base_rules.toml').read_bytes();req(hashlib.sha256(base).hexdigest()==LAW_SHA,'admission law identity')
    recipes=[dict(case=case,rotation=rotation,reversed=reversed_) for case in ['faithful_disjoint','mixed_competing','mixed_settlement'] for rotation in [0,3] for reversed_ in [False,True]]
    def name(r):return f"{r['case']}_r{r['rotation']}_{'reverse' if r['reversed'] else 'forward'}"
    req({p.name for p in root.iterdir() if p.is_dir()}=={name(r) for r in recipes},'admission fixed twelve-cell denominator');results={};statuses=[]
    for r in recipes:
        status,result=verify_cell(root/name(r),c,d,s,base,r);statuses.append(status);results[name(r)]=result
    req(json.loads((root/'summary.json').read_text())==statuses,'admission root summary');boundary(json.loads((root/'run_status.json').read_text()),req);m=json.loads((root/'manifest.json').read_text());req(m['state']=='Completed' and m['max_steps']==m['final_step']==35328,'admission root manifest')
    # Allocation independence compares semantics, leaving matcher work and graph handles diagnostic.
    for case in ['faithful_disjoint','mixed_competing','mixed_settlement']:
        for rotation in [0,3]:
            paths=[root/name(dict(case=case,rotation=rotation,reversed=x)) for x in [False,True]]
            for f in ['lineage.json','admission_state.json','rows.json']:
                vals=[json.loads((p/f).read_text()) for p in paths]
                if f=='admission_state.json':
                    for v in vals:v.pop('recipe')
                if f=='rows.json':
                    for v in vals:
                        for row in v:row.pop('match_work')
                req(vals[0]==vals[1],'admission allocation semantic equality')
    # Coin schedules are tied to physical sites; only the faithful cue recipe claims rotation equivalence.
    for f in ['lineage.json','admission_state.json','rows.json']:
        vals=[]
        for rot in [0,3]:
            v=json.loads((root/f'faithful_disjoint_r{rot}_forward'/f).read_text())
            if f=='lineage.json':
                for b in v.values():b['site']=(b['site']-rot)%8
            elif f=='admission_state.json':
                v.pop('recipe')
                for r in v['records']:r['site']=(r['site']-rot)%8
            else:
                for row in v:row.pop('match_work')
            vals.append(v)
        req(vals[0]==vals[1],'admission faithful rotation equivalence')
    req(results['faithful_disjoint_r0_forward']['parallel_admission_ticks'],'admission disjoint parallel coverage');req(results['mixed_settlement_r0_forward']['mixed_validity_parallel_ticks'],'admission concurrent invalid/valid coverage')
    calibration=json.loads((root/'lottery_calibration.json').read_text());expected=[];wins={1:0,2:0}
    for seed in range(32):
        for rotation in range(8):
            for reversed_ in [False,True]:
                for competing in [False,True]:
                    physical=[dict(parent=1,home=rotation,target=(rotation+1)%8,length=3),dict(parent=2,home=(rotation+(2 if competing else 4))%8,target=(rotation+(1 if competing else 5))%8,length=3)]
                    if reversed_:physical.reverse()
                    offers=[dict(o,home=(o['home']-rotation)%8,target=(o['target']-rotation)%8) for o in physical];rnd=lottery(seed,1,offers);expected.append(dict(seed=seed,rotation=rotation,reversed=reversed_,competing=competing,physical_offers=physical,round=rnd));req(len(rnd['selected'])==(1 if competing else 2),'admission calibration conflict coverage')
                    if competing and rotation==0 and not reversed_:wins[rnd['offers'][rnd['selected'][0]]['parent']]+=1
    req(calibration==expected and len(calibration)==1024,'admission lottery calibration full inventory')
    sources=json.loads((root/'sources.json').read_text())
    for f,p in [('checker.py','scripts/verify_evolving_minds_admission.py'),('d0_reference.py','scripts/verify_evolving_minds_d0.py'),('replay.py','scripts/verify_evolving_minds.py'),('schedule_reference.py','scripts/verify_evolving_minds_choice.py'),('mutation_reference.py','scripts/verify_evolving_minds_mutation.py'),('base_rules.toml','rules/evolving_minds/admission_v1.toml'),('contract.md','experiments/evolving_minds/contracts/d0_admission_v1.md')]:req((root/f).read_text()==sources[p],'admission frozen source binding')
    return dict(profile=PROFILE,audit_pass=True,development_only=True,scientific_run_launched=False,gate_pass=False,training_admitted=False,evolution_enabled=False,reproductive_fairness_accepted=False,cells_verified=12,native_advances_verified=35328,lottery_cases_verified=1024,competing_seed_winners=wins,source_sha256={k:hashlib.sha256(v.encode()).hexdigest() for k,v in sources.items()},cells=results)
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('directory',type=Path);print(json.dumps(verify(p.parse_args().directory),indent=2))
