#!/usr/bin/env python3
"""Read-only event replay and independent slot interpreter for bounded D0."""
import argparse
from collections import Counter
import hashlib
import importlib.util
import json
from pathlib import Path
import struct
import sys
import tomllib

PROFILE='evolving-minds-d0-development-v1'
LAW_SHA='af93878599c31468469f503d77a7393818e0b44efebb84dbd280f80db9ec7a5a'
STATE,SLOT=0xE1010000,0xE1010001
OWN,MAP,NEXT=range(0xE1010064,0xE1010067)
CURSOR,STATUS,POSITION,IDENTITY,DEST,SRC,INITIAL,USED,INPUT,BUFFER,HAS_ACT=range(242001,242012)
CLOCK,RESOURCE,CUE,COIN,_,_,ATTEMPT,CHOICE=range(241001,241009)
PHASES=['Environment','World','Sense','Sense']+['Signal']*7+['Development','Attempt','Settle','Maintenance']
OPS=['State','Actuator','Wire','Halt']

def load(path,name):
    spec=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(spec)
    old=sys.dont_write_bytecode
    try:sys.dont_write_bytecode=True;spec.loader.exec_module(m)
    finally:sys.dont_write_bytecode=old
    return m

def program(name):
    ops={
        'cue': [('Actuator',2,0,False),('Wire',2,0,False)],
        'coin': [('Actuator',2,0,False),('Wire',2,1,False)],
        'constant0': [('State',2,0,False),('Actuator',3,0,False),('Wire',3,2,False)],
        'constant1': [('State',2,0,True),('Actuator',3,0,False),('Wire',3,2,False)],
        'delayed_cue': [('State',2,0,False),('Actuator',3,0,False),('Wire',2,0,False),('Wire',3,2,False)],
        'missing_port': [('Actuator',2,0,False),('Wire',2,7,False)],
        'duplicate_writer': [('Actuator',2,0,False),('Wire',2,0,False),('Wire',2,1,False)],
        'duplicate_slot': [('State',2,0,False),('Actuator',2,0,False)],
        'second_actuator': [('Actuator',2,0,False),('Actuator',3,0,False)],
    }[name]+[('Halt',0,0,False)]
    return dict(instructions=[dict(id=100+i,op=o,slot=d,source=s,initial=v) for i,(o,d,s,v) in enumerate(ops)])

def canonical(p):
    return b'HG-D0-1'+bytes([len(p['instructions'])])+b''.join(struct.pack('<IBBBB',g['id'],OPS.index(g['op']),g['slot'],g['source'],g['initial']) for g in p['instructions'])

class Reference:
    """No native rules, handles, events or recorded metrics drive this model."""
    def __init__(self,p,table,reserve,c):
        self.p=p['instructions'];self.table=table;self.c=c
        self.parts={0:[c.SENSOR,False,False],1:[c.SENSOR,False,False]};self.wires=[]
        self.reserve=reserve;self.site=64.0;self.clock=0;self.resource=self.cue=self.coin=False
        self.cursor=self.status=self.age=0;self.pending=self.choice=False;self.has_act=False
        self.influx=self.dissipated=self.harvested=0.0
        self.paid=self.dev_failed=self.attempts=self.successes=self.failures=0
    def material(self):return 1+8+len(self.p)+len(self.parts)+len(self.wires)
    def step(self,phase,tick):
        c=self.c;rules=[]
        if phase==0:
            if self.site<=56:self.site+=8;self.influx+=8;rules=['environment_influx']
        elif phase==1:
            t=self.clock;self.resource=self.table['resources'][t];self.cue=self.table['cues'][t];self.coin=self.table['coins'][0][t];self.clock+=1;rules=[f'world_0_{t}']
        elif phase in [2,3]:
            slot=phase-2;self.parts[slot][1]=self.cue if slot==0 else self.coin;rules=[f'sense_{slot}']
        elif 4<=phase<=9:
            slot=phase-2
            for s,d in self.wires:
                if d==slot:self.parts[d][2]=self.parts[s][1];rules.append(f'prepare_{slot}_{self.parts[s][0]}_{self.parts[d][0]}')
        elif phase==10:
            for _,d in self.wires:self.parts[d][1]=self.parts[d][2];rules.append(f'commit_{self.parts[d][0]}')
        elif phase==11 and self.status<=1:
            if self.reserve<1:
                if self.status==0:self.status=1;rules=['development_wait']
            else:
                g=self.p[self.cursor];op=g['op'];d=g['slot'];s=g['source'];self.reserve-=1;self.dissipated+=1;self.paid+=1
                valid=True
                if op in ['State','Actuator']:
                    valid=d not in self.parts and not (op=='Actuator' and self.has_act)
                    if valid:
                        label=STATE if op=='State' else c.ACTUATOR;self.parts[d]=[label,g['initial'],g['initial']];self.has_act|=op=='Actuator';rules=[f'development_create_{OPS.index(op)}']
                elif op=='Wire':
                    valid=s in self.parts and d in self.parts and d!=s and self.parts[d][0]!=c.SENSOR and not any(b==d for _,b in self.wires)
                    if valid:self.wires.append((s,d));rules=[f'development_wire_{self.parts[s][0]}_{self.parts[d][0]}']
                else:rules=['development_halt']
                if valid:self.cursor+=1;self.status=2 if op=='Halt' else 0
                else:self.status=3;self.dev_failed+=1;rules=['development_fail']
        elif phase==12 and self.status==2 and self.has_act and self.reserve>=1:
            self.choice=next(p[1] for p in self.parts.values() if p[0]==c.ACTUATOR);self.pending=True;self.reserve-=1;self.dissipated+=1;self.attempts+=1;rules=['attempt_commit']
        elif phase==13 and self.pending:
            self.pending=False
            if self.choice==self.resource and self.site>=8 and self.reserve<=1000:self.site-=8;self.reserve+=8;self.harvested+=8;self.successes+=1;rules=['settle_success']
            else:self.failures+=1;rules=['settle_failure']
        elif phase==14:
            price=float(self.material());full=self.reserve>=price;spent=min(self.reserve,price);self.reserve-=spent;self.dissipated+=spent;self.age+=1;rules=['maintenance_full' if full else 'maintenance_exhausted']
        return rules
    def row(self,tick,work):
        return dict(tick=tick,engine_step=tick*15,cursor=self.cursor,status=self.status,material_nodes=self.material(),development_paid=self.paid,development_failures=self.dev_failed,attempts=self.attempts,successes=self.successes,failures=self.failures,harvested=self.harvested,influx=self.influx,dissipated=self.dissipated,total_energy=self.site+self.reserve,residual=0.0,match_work=work)

def material(nodes,edges,r,c):
    req=c.require
    roots=[h for h,n in nodes.items() if n['label']==c.ROOT];sites=[h for h,n in nodes.items() if n['label']==c.SITE]
    req(len(roots)==len(sites)==1,'D0 founder/site');root,site=roots[0],sites[0]
    parts={};slots={};genes={};wires=[]
    expected_props={root:{c.ENERGY:r.reserve,c.BID:1,c.AGE:r.age,CURSOR:r.cursor,STATUS:r.status,HAS_ACT:r.has_act,ATTEMPT:r.pending,CHOICE:r.choice,c.MAINT:float(r.material())},site:{c.ENERGY:r.site,CLOCK:r.clock,241009:0,RESOURCE:r.resource,CUE:r.cue,COIN:r.coin}}
    for h,n in nodes.items():
        req(n['energy']==0 and isinstance(n['generation'],int) and n['generation']>=0,'D0 hidden energy/generation')
        label=n['label']
        if label in [c.SENSOR,STATE,c.ACTUATOR]:
            slot=c.value(n,DEST,'Int');req(slot not in parts and slot in r.parts,'D0 slot bounds/uniqueness');parts[slot]=h;expected_label,signal,buffer=r.parts[slot];req(label==expected_label,'D0 material role')
            expected_props[h]={c.OWNER:1,DEST:slot,c.SIGNAL:signal}
            if label!=c.SENSOR:expected_props[h].update({BUFFER:buffer,INPUT:any(d==slot for _,d in r.wires)})
        elif label==SLOT:
            slot=c.value(n,DEST,'Int');req(slot not in slots and 0<=slot<8,'D0 slot map bounds');slots[slot]=h;expected_props[h]={c.OWNER:1,DEST:slot,USED:slot in r.parts}
        elif label==c.GENE:
            pos=c.value(n,POSITION,'Int');req(pos not in genes and 0<=pos<len(r.p),'D0 gene order');genes[pos]=h;g=r.p[pos];expected_props[h]={c.OWNER:1,POSITION:pos,IDENTITY:g['id'],c.OP:OPS.index(g['op']),DEST:g['slot'],SRC:g['source'],INITIAL:g['initial']}
        elif label==c.CONNECTION:wires.append(h);expected_props[h]={c.OWNER:1}
        else:req(label in [c.ROOT,c.SITE],'D0 unknown label')
    req(set(parts)==set(r.parts) and set(slots)==set(range(8)) and set(genes)==set(range(len(r.p))) and len(wires)==len(r.wires),'D0 material inventory')
    for h,n in nodes.items():req(len(n['properties'])==len(expected_props[h]) and dict(n['properties'])=={k:c.typed(v) for k,v in expected_props[h].items()},'D0 exact properties/genotype/upkeep/state')
    def key(label,a,b):return label,tuple(sorted([a,b]))
    expected={key(c.OCC,root,site)}|{key(OWN,root,h) for h in nodes if h not in [root,site]}
    expected|={key(MAP,slots[s],h) for s,h in parts.items()}
    expected|={key(NEXT,genes[i-1],genes[i]) for i in range(1,len(r.p))}
    actual={c.edge_key(e) for e in edges.values()};req(len(actual)==len(edges) and all(len(e['nodes'])==2 for e in edges.values()),'D0 duplicate/self/arity relation')
    pairs=[]
    for w in wires:
        src=[s for s,h in parts.items() if key(c.SOURCE,h,w) in actual];dst=[d for d,h in parts.items() if key(c.TARGET,w,h) in actual]
        req(len(src)==len(dst)==1,'D0 wire ports');pairs.append((src[0],dst[0]));expected|={key(c.SOURCE,parts[src[0]],w),key(c.TARGET,w,parts[dst[0]])}
    req(Counter(pairs)==Counter(r.wires) and actual==expected,'D0 role/ownership/writer/program relation grammar')
    req(len(nodes)<=40 and len(edges)<=64,'D0 expansion budget')
    req(c.total(nodes)==r.site+r.reserve,'D0 reserve reconstruction')

def checkpoint(path,req):
    b=path.read_bytes();req(24<=len(b)<=8*1024*1024+24 and b[:8]==b'HGEMD001','D0 checkpoint envelope');req(int.from_bytes(b[8:16],'little')==len(b)-24,'D0 checkpoint length')
    h=0xcbf29ce484222325
    for x in b[24:]:h=((h^x)*0x100000001b3)&((1<<64)-1)
    req(int.from_bytes(b[16:24],'little')==h,'D0 checkpoint checksum')

def boundary(status,req):
    req(status['profile']==PROFILE and status['engineering_execution_complete'] is True and status['development_only'] is True,'D0 engineering completion')
    req(all(status[k] is False for k in ['scientific_run_launched','gate_pass','training_admitted','evolution_enabled']),'D0 scientific boundary')

def verify_cell(directory,c,base,schedule_module,name,mode,reserve):
    req=c.require;p=program(name);req(json.loads((directory/'program.json').read_text())==p,'D0 program identity');req((directory/'genotype.bin').read_bytes()==canonical(p),'D0 canonical projection')
    table=schedule_module.schedule(17,mode);req(json.loads((directory/'schedule.json').read_text())==table,'D0 world streams')
    raw=(directory/'rules.toml').read_bytes();req(raw.startswith(base),'D0 fixed physical laws');laws=tomllib.loads(raw.decode())['rules'];rules={r['name']:r for r in laws};req(len(rules)==len(laws)==len(tomllib.loads(base.decode())['rules'])+512,'D0 rule inventory')
    # Independently check every scheduled world declaration, including unused sites.
    for site in range(8):
        for t in range(64):
            expected=dict(name=f'world_{site}_{t}',priority=0,lhs=dict(nodes=['site'],edges=[],node_labels=dict(site=c.SITE),node_property_guards=[dict(var='site',property=CLOCK,op='eq',value=t),dict(var='site',property=241009,op='eq',value=site)]),rhs=dict(nodes=['site'],edges=[],property_effects=[dict(var='site',key=k,op='set',value=v) for k,v in [(RESOURCE,table['resources'][t]),(CUE,table['cues'][t]),(COIN,table['coins'][site][t])]]+[dict(var='site',key=CLOCK,op='add',value=1)]))
            req(rules[expected['name']]==expected,'D0 world-law declaration')
    frames=[json.loads(s) for s in (directory/'trace.jsonl').read_text().splitlines()];req(len(frames)==361,'D0 full phase coverage');f=frames[0];req(f['step']==f['tick']==0 and f['phase'] is None and not f['events'],'D0 initial clock')
    nodes,edges=c.graphs(f);r=Reference(p,table,reserve,c);material(nodes,edges,r,c);req(all(n['generation']==0 for n in nodes.values()),'D0 initial generation');initial=reserve+64.0;rows=[];work=0;actions=[]
    for index,f in enumerate(frames[1:]):
        phase=index%15;tick=index//15+1;req(f['step']==index+1 and f['tick']==tick and f['phase']==PHASES[phase],'D0 clock/pass order')
        expected=r.step(phase,tick);req(Counter(e['rule'] for e in f['events'])==Counter(expected),'D0 mandatory interpreter/event coverage')
        engaged=set()
        for e in f['events']:
            hs=set(map(c.handle,e['bound']));req(not hs&engaged,'D0 conflicting rewrites');engaged|=hs;c.apply(nodes,edges,e,rules[e['rule']],index)
        after,after_edges=c.graphs(f);req(nodes==after and edges==after_edges,'D0 independent rewrite result');material(nodes,edges,r,c)
        req(c.total(nodes)==initial+r.influx-r.dissipated,'D0 conservation');req(isinstance(f['match_work'],int) and 0<=f['match_work']<=100000,'D0 matcher work bound');work+=f['match_work']
        req(f['stats']['property_effects_skipped']==0,'D0 skipped effects')
        req(f['stats']['accepted_rewrites']==len(f['events']) and all(f['stats'][field]==sum(len(e[event]) for e in f['events']) for field,event in [('nodes_created','created'),('nodes_deleted','deleted'),('edges_created','edges_created'),('edges_deleted','edges_deleted')]),'D0 native statistics/event accounting')
        if phase==12 and expected:actions.append(dict(tick=tick,choice=r.choice))
        if phase==14:rows.append(r.row(tick,work));work=0
    req(json.loads((directory/'rows.json').read_text())==rows,'D0 independently reconstructed rows')
    status=json.loads((directory/'run_status.json').read_text());boundary(status,req);req(status['program']==name and status['cue_mode']==mode and status['initial_reserve']==reserve and status['final_row']==rows[-1],'D0 cell identity/status')
    m=json.loads((directory/'manifest.json').read_text());req(m['state']=='Completed' and m['seed']==17 and m['final_step']==m['max_steps']==360,'D0 cell manifest')
    req({p.name for p in directory.glob('*.checkpoint')}=={'initial.checkpoint','partial.checkpoint','final.checkpoint'},'D0 checkpoint coverage')
    for p in directory.glob('*.checkpoint'):checkpoint(p,req)
    req((directory/'trace.jsonl').stat().st_size<=8*1024*1024,'D0 trace budget')
    return status,dict(final_row=rows[-1],actions=actions)

def verify(root):
    root=Path(root);c=load(root/'replay.py','d0_replay');s=load(root/'schedule_reference.py','d0_schedule');req=c.require
    req(root.is_dir() and not root.is_symlink() and not any(p.is_symlink() for p in root.rglob('*')),'D0 evidence path');req(sum(p.stat().st_size for p in root.rglob('*') if p.is_file())<=192*1024*1024,'D0 total output budget')
    base=(root/'base_rules.toml').read_bytes();req(hashlib.sha256(base).hexdigest()==LAW_SHA,'D0 law identity')
    cases=[(f'{m.lower()}_{p}',p,m,512.0) for m in ['Informative','Permuted','Withheld'] for p in ['cue','coin','constant0','constant1','delayed_cue']]+[(f'informative_{p}',p,'Informative',512.0) for p in ['missing_port','duplicate_writer','duplicate_slot','second_actuator']]+[('informative_cue_waiting','cue','Informative',0.0)]
    req({p.name for p in root.iterdir() if p.is_dir()}=={name for name,_,_,_ in cases},'D0 fixed 20-cell denominator');statuses=[];results={}
    for folder,p,m,reserve in cases:status,result=verify_cell(root/folder,c,base,s,p,m,reserve);statuses.append(status);results[folder]=result
    req(json.loads((root/'summary.json').read_text())==statuses,'D0 summary')
    for p in ['coin','constant0','constant1']:
        req(results[f'informative_{p}']==results[f'permuted_{p}']==results[f'withheld_{p}'],'D0 uninformed cue leakage')
    req(results['informative_cue']['final_row']['successes']==results['informative_cue']['final_row']['attempts']==22,'D0 useful cue control')
    req(len({results[f'informative_{p}']['final_row']['material_nodes'] for p in ['cue','constant0','delayed_cue']})==3,'D0 phenotype variation')
    req(results['informative_cue']['actions']!=results['informative_constant0']['actions'],'D0 action variation')
    for m in ['Informative','Permuted','Withheld']:
        table=s.schedule(17,m)
        req(all(a['choice']==table['cues'][a['tick']-2] for a in results[f'{m.lower()}_delayed_cue']['actions']),'D0 explicit relay delay')
    for p in ['missing_port','duplicate_writer','duplicate_slot','second_actuator']:
        row=results[f'informative_{p}']['final_row'];req(row['status']==3 and row['development_failures']==1 and row['development_paid']==row['cursor']+1 and row['attempts']==0,'D0 paid permanent failure')
    row=results['informative_cue_waiting']['final_row'];req(row['status']==1 and row['cursor']==row['development_paid']==row['attempts']==0,'D0 insufficient-energy waiting')
    boundary(json.loads((root/'run_status.json').read_text()),req);m=json.loads((root/'manifest.json').read_text());req(m['state']=='Completed' and m['final_step']==m['max_steps']==7200,'D0 panel completion')
    sources=json.loads((root/'sources.json').read_text())
    for filename,source in [('checker.py','scripts/verify_evolving_minds_d0.py'),('replay.py','scripts/verify_evolving_minds.py'),('schedule_reference.py','scripts/verify_evolving_minds_choice.py'),('base_rules.toml','rules/evolving_minds/d0_v1.toml'),('contract.md','experiments/evolving_minds/contracts/d0_development_v1.md')]:req((root/filename).read_text()==sources[source],'D0 frozen source binding')
    return dict(profile=PROFILE,audit_pass=True,development_only=True,scientific_run_launched=False,gate_pass=False,training_admitted=False,evolution_enabled=False,cells_verified=20,native_advances_verified=7200,source_sha256={k:hashlib.sha256(v.encode()).hexdigest() for k,v in sources.items()},cells=results)

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('directory',type=Path);print(json.dumps(verify(p.parse_args().directory),indent=2))
