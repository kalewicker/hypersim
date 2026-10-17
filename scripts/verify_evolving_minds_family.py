#!/usr/bin/env python3
"""Independent paid-copy, reset, lifecycle and event-journal audit."""
from collections import Counter
import argparse
import copy
import hashlib
import json
from pathlib import Path
import sys
import tomllib
import importlib.util

PROFILE='evolving-minds-d0-family-v1'
LAW_SHA='fb3337159587ac09e42f6ab1d1e5d3a84c40eefc0ba6e2f782011f5e6bc38902'
LENGTH,PARTS=242012,242013
PHASES=['environment_','world_','sense_0','sense_1']+[f'prepare_{i}_' for i in range(2,8)]+['commit_','development_','attempt_','settle_','maintenance_','birth_','death_']+['cleanup_']*26

def load(path,name):
    spec=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(spec);old=sys.dont_write_bytecode
    try:sys.dont_write_bytecode=True;spec.loader.exec_module(m)
    finally:sys.dont_write_bytecode=old
    return m

def cost(n):return 1.0+(11+n)/4
def expected_program(name,d):return dict(instructions=[dict(id=100,op='Halt',slot=0,source=0,initial=False)]) if name=='inactive' else d.program(name)
def boundary(status,req):
    req(status['profile']==PROFILE and status['development_only'] is True and status['engineering_execution_complete'] is True,'family completion')
    req(all(status[k] is False for k in ['scientific_run_launched','gate_pass','training_admitted','evolution_enabled']),'family scientific boundary')

def checkpoint(path,req):
    b=path.read_bytes();req(24<=len(b)<=16*1024*1024+24 and b[:8]==b'HGEMF001' and int.from_bytes(b[8:16],'little')==len(b)-24,'family checkpoint envelope')
    h=0xcbf29ce484222325
    for x in b[24:]:h=((h^x)*0x100000001b3)&((1<<64)-1)
    req(h==int.from_bytes(b[16:24],'little'),'family checkpoint checksum')

def verify_cell(directory,c,d,s,base,name,mode):
    req=c.require;p=expected_program(name,d);table=s.schedule(17,mode);req(json.loads((directory/'program.json').read_text())==p,'family program');req(json.loads((directory/'schedule.json').read_text())==table,'family schedule')
    raw=(directory/'rules.toml').read_bytes();req(raw.startswith(base),'family fixed physics');laws=tomllib.loads(raw.decode())['rules'];rules={r['name']:r for r in laws};req(len(rules)==len(laws)==len(tomllib.loads(base.decode())['rules'])+512,'family laws inventory')
    for site in range(8):
        for t in range(64):
            expected=dict(name=f'world_{site}_{t}',priority=0,lhs=dict(nodes=['site'],edges=[],node_labels=dict(site=c.SITE),node_property_guards=[dict(var='site',property=d.CLOCK,op='eq',value=t),dict(var='site',property=241009,op='eq',value=site)]),rhs=dict(nodes=['site'],edges=[],property_effects=[dict(var='site',key=k,op='set',value=v) for k,v in [(d.RESOURCE,table['resources'][t]),(d.CUE,table['cues'][t]),(d.COIN,table['coins'][site][t])]]+[dict(var='site',key=d.CLOCK,op='add',value=1)]))
            req(rules[expected['name']]==expected,'family world law identity')
    first=json.loads((directory/'initial.json').read_text());req(first['step']==first['tick']==0 and first['phase'] is None and not first['events'],'family initial clock');nodes,edges=c.graphs(first)
    sites={c.value(n,241009,'Int'):h for h,n in nodes.items() if n['label']==c.SITE};counter=next(h for h,n in nodes.items() if n['label']==c.COUNTER);root=next(h for h,n in nodes.items() if n['label']==c.ROOT)
    req(set(sites)==set(range(8)) and len(nodes)==20+len(p['instructions']) and all(n['generation']==0 for n in nodes.values()),'family initial inventory')
    class Ref(d.Reference):
        def material(self):return 2.0+len(self.p)/8+(len(self.parts)-2+len(self.wires))/4
    refs={1:Ref(p,table,64.0,c)};root_handles={1:root};ancestry={1:dict(id=1,parent=None,born_tick=0,death_tick=None,site=0,program=p)};dead={};site_reserves={i:32.0 for i in range(8)};influx=copy_sink=0.0;births=deaths=0;clock=0
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
        req(nodes[counter]['label']==c.COUNTER and nodes[counter]['energy']==0 and dict(nodes[counter]['properties'])=={c.NEXT:c.typed(len(ancestry)+1)},'family monotone birth counter')
        for bid in live():
            r=hydrate(bid);h=root_handles[bid];site=sites[ancestry[bid]['site']];members={k for k,n in nodes.items() if n['label'] not in [c.ROOT,c.SITE,c.DEAD,c.COUNTER] and c.value(n,c.OWNER,'Int')==bid};all_members[bid]=members
            req(nodes[h]['label']==c.ROOT and c.value(nodes[h],c.BID,'Int')==bid and c.value(nodes[h],c.PARENT,'Int')==(ancestry[bid]['parent'] or 0) and c.value(nodes[h],LENGTH,'Int')==len(r.p) and c.value(nodes[h],PARTS,'Int')==len(members),'family immutable ancestry/genome/count')
            subset=members|{h,site};small={k:copy.deepcopy(nodes[k]) for k in subset};small_edges={k:copy.deepcopy(e) for k,e in edges.items() if all(c.handle(n) in subset for n in e['nodes'])}
            small[h]['properties']=[x for x in small[h]['properties'] if x[0] not in [c.PARENT,LENGTH,PARTS]];c.set_property(small[h],c.BID,{'Int':1});small[site]['properties']=[x for x in small[site]['properties'] if x[0]!=c.OCCUPIED];c.set_property(small[site],241009,{'Int':0})
            for k in members:c.set_property(small[k],c.OWNER,{'Int':1})
            d.material(small,small_edges,r,c);expected|={c.edge_key(e) for e in small_edges.values()}
        for bid,snapshot in dead.items():
            h=root_handles[bid];members={k for k,n in nodes.items() if n['label'] not in [c.ROOT,c.SITE,c.DEAD,c.COUNTER] and c.value(n,c.OWNER,'Int')==bid};all_members[bid]=members
            n=copy.deepcopy(snapshot[h]);c.set_property(n,PARTS,{'Int':len(members)});req(nodes[h]==n,'family tombstone settlement/disassembly')
            for k in members:req(nodes[k]==snapshot[k],'family dead material changed');expected.add(key(d.OWN,h,k))
            for e in edges.values():
                endpoints={c.handle(k) for k in e['nodes']}
                if endpoints<=members:expected.add(c.edge_key(e))
        supported=set(sites.values())|{counter}|set(root_handles.values())|set().union(*all_members.values()) if all_members else set(sites.values())|{counter}|set(root_handles.values())
        req(set(nodes)==supported,'family orphan or hidden material')
        actual={c.edge_key(e) for e in edges.values()};req(len(actual)==len(edges) and actual==expected and all(len(e['nodes'])==2 and len({c.handle(h) for h in e['nodes']})==2 for e in edges.values()),'family global role/owner/scaffold grammar')
        req(len(nodes)<=256 and len(edges)<=768,'family material ceiling')
    material();req(c.total(nodes)==320.0,'family initial budgets')
    entries=[json.loads(x) for x in (directory/'journal.jsonl').read_text().splitlines()];req(len(entries)==2752,'family complete journal coverage');rows=[];work=0
    for index,entry in enumerate(entries):
        tick=index//43+1;phase=index%43;req(entry['tick']==tick and entry['pass']==phase and entry['phase']==PHASES[phase] and entry['step']==index+1,'family journal clock/pass')
        expected=[];winner=None;cleanup_bindings={}
        if phase==0:
            for i in range(8):
                if site_reserves[i]<=56:site_reserves[i]+=8;influx+=8;expected.append('environment_influx')
        elif phase==1:expected=[f'world_{i}_{clock}' for i in range(8)];clock+=1
        elif 2<=phase<=14:
            for bid in live():
                r=hydrate(bid);expected+=r.step(phase,tick);site_reserves[ancestry[bid]['site']]=r.site
        elif phase==15:
            candidates=[]
            for bid in live():
                r=refs[bid];home=ancestry[bid]['site'];length=len(r.p)
                if r.status!=2 or r.age<2 or r.reserve<12+cost(length):continue
                gs={c.value(n,d.POSITION,'Int'):h for h,n in nodes.items() if n['label']==c.GENE and c.value(n,c.OWNER,'Int')==bid}
                for target in [(home-1)%8,(home+1)%8]:
                    if target in {ancestry[b]['site'] for b in live()}:continue
                    binding={'r':root_handles[bid],'site':sites[home],'target':sites[target],'counter':counter,**{f'g{i}':gs[i] for i in range(length)}};bound=[binding[v] for v in rules[f'birth_faithful_{length}']['lhs']['nodes']];candidates.append((sorted(map(c.rank,bound)),bid,target,bound))
            if candidates:
                _,parent,target,bound=min(candidates);winner=(parent,target,bound);expected=[f'birth_faithful_{len(refs[parent].p)}']
        elif phase==16:
            for bid in live():
                r=refs[bid]
                if r.age>=16 or r.reserve<=0:
                    expected.append('death_age' if r.age>=16 else 'death_empty');site_reserves[ancestry[bid]['site']]+=r.reserve;r.reserve=0.0;ancestry[bid]['death_tick']=tick;deaths+=1
        else:
            for bid in dead:
                h=root_handles[bid];members=[k for k,n in nodes.items() if n['label'] not in [c.ROOT,c.DEAD,c.SITE,c.COUNTER] and c.value(n,c.OWNER,'Int')==bid]
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
            if phase>=17:req(bound==cleanup_bindings[bound[0]],'family canonical disassembly')
            binding=c.apply(nodes,edges,event,rules[event['rule']],index)
            if phase==15:
                parent,target,_=winner;bid=len(ancestry)+1;refs[parent].reserve-=12+cost(len(refs[parent].p));copy_sink+=cost(len(refs[parent].p));births+=1;refs[bid]=Ref(p,table,12.0,c);root_handles[bid]=binding['child'];ancestry[bid]=dict(id=bid,parent=parent,born_tick=tick,death_tick=None,site=target,program=p)
            elif phase==16:
                h=binding['r'];bid=c.value(nodes[h],c.BID,'Int');dead[bid]={k:copy.deepcopy(n) for k,n in nodes.items() if k==h or (n['label'] not in [c.ROOT,c.DEAD,c.SITE,c.COUNTER] and c.value(n,c.OWNER,'Int')==bid)}
            elif phase>=17 and event['rule']=='cleanup_root':
                bid=next(b for b,h in root_handles.items() if h==binding['r']);del root_handles[bid];del dead[bid]
        material();dissipated=copy_sink+sum(r.dissipated for r in refs.values());req(c.total(nodes)==320.0+influx-dissipated,'family conservation/copy ledger')
        stats=entry['stats'];req(stats['property_effects_skipped']==0 and stats['accepted_rewrites']==len(entry['events']) and stats['property_effects_applied']==sum(len(rules[e['rule']]['rhs'].get('property_effects',[])) for e in entry['events']),'family stats/effects')
        req(all(stats[field]==sum(len(e[field if field.startswith('edges') else 'created' if field=='nodes_created' else 'deleted']) for e in entry['events']) for field in ['nodes_created','nodes_deleted','edges_created','edges_deleted']),'family stats/material events')
        req(isinstance(entry['match_work'],int) and 0<=entry['match_work']<=1000000,'family work budget');work+=entry['match_work']
        if phase==42:
            req(not dead and all(not r.pending for r in refs.values()),'family completed-tick settlement');depth={}
            for bid,b in ancestry.items():depth[bid]=depth[b['parent']]+1 if b['parent'] else 0
            row=dict(tick=tick,engine_step=tick*43,live_organisms=len(live()),births=births,deaths=deaths,max_ancestry_depth=max(depth.values()),development_paid=sum(r.paid for r in refs.values()),development_failures=sum(r.dev_failed for r in refs.values()),attempts=sum(r.attempts for r in refs.values()),successes=sum(r.successes for r in refs.values()),failures=sum(r.failures for r in refs.values()),harvested=sum(r.harvested for r in refs.values()),influx=influx,dissipated=dissipated,total_energy=c.total(nodes),residual=0.0,match_work=work);rows.append(row);work=0
    final=json.loads((directory/'final.json').read_text());req(final['step']==2752 and final['tick']==64 and final['phase'] is None and not final['events'] and c.graphs(final)==(nodes,edges),'family exact final graph')
    req(json.loads((directory/'rows.json').read_text())==rows,'family reconstructed rows');req(json.loads((directory/'lineage.json').read_text())=={str(k):v for k,v in ancestry.items()},'family ancestry/copy identity')
    status=json.loads((directory/'run_status.json').read_text());boundary(status,req);req(status['program']==name and status['cue_mode']==mode and status['final_row']==rows[-1],'family cell summary')
    m=json.loads((directory/'manifest.json').read_text());req(m['state']=='Completed' and m['seed']==17 and m['max_steps']==m['final_step']==2752,'family cell manifest')
    req({p.name for p in directory.glob('*.checkpoint')}=={'initial.checkpoint','midpoint.checkpoint','final.checkpoint'},'family checkpoint coverage')
    for path in directory.glob('*.checkpoint'):checkpoint(path,req)
    req((directory/'journal.jsonl').stat().st_size<=16*1024*1024,'family journal budget')
    return status,dict(final_row=rows[-1],genotypes_verified=len(ancestry),faithful_births_verified=births)

def verify(root):
    root=Path(root);c=load(root/'replay.py','family_replay');d=load(root/'d0_reference.py','family_d0');s=load(root/'schedule_reference.py','family_schedule');req=c.require
    req(root.is_dir() and not root.is_symlink() and not any(p.is_symlink() for p in root.rglob('*')),'family evidence path');req(sum(p.stat().st_size for p in root.rglob('*') if p.is_file())<=128*1024*1024,'family panel output cap')
    base=(root/'base_rules.toml').read_bytes();req(hashlib.sha256(base).hexdigest()==LAW_SHA,'family law identity');cases=[('cue','Informative'),('cue','Permuted'),('cue','Withheld'),('inactive','Informative')];req({p.name for p in root.iterdir() if p.is_dir()}=={f'{m.lower()}_{n}' for n,m in cases},'family fixed four-cell denominator');results={};statuses=[]
    for name,mode in cases:status,result=verify_cell(root/f'{mode.lower()}_{name}',c,d,s,base,name,mode);statuses.append(status);results[f'{mode.lower()}_{name}']=result
    req(json.loads((root/'summary.json').read_text())==statuses,'family panel summary');r=results['informative_cue']['final_row'];req(r['max_ancestry_depth']>=3 and r['live_organisms']>0 and r['births']>=3 and r['deaths']>=3 and r['successes']==r['attempts'],'family engineering inheritance/turnover calibration')
    boundary(json.loads((root/'run_status.json').read_text()),req);m=json.loads((root/'manifest.json').read_text());req(m['state']=='Completed' and m['max_steps']==m['final_step']==11008,'family panel completion')
    sources=json.loads((root/'sources.json').read_text())
    for f,p in [('checker.py','scripts/verify_evolving_minds_family.py'),('d0_reference.py','scripts/verify_evolving_minds_d0.py'),('replay.py','scripts/verify_evolving_minds.py'),('schedule_reference.py','scripts/verify_evolving_minds_choice.py'),('base_rules.toml','rules/evolving_minds/family_v1.toml'),('contract.md','experiments/evolving_minds/contracts/d0_family_v1.md')]:req((root/f).read_text()==sources[p],'family frozen source binding')
    return dict(profile=PROFILE,audit_pass=True,development_only=True,scientific_run_launched=False,gate_pass=False,training_admitted=False,evolution_enabled=False,cells_verified=4,native_advances_verified=11008,source_sha256={k:hashlib.sha256(v.encode()).hexdigest() for k,v in sources.items()},cells=results)

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('directory',type=Path);print(json.dumps(verify(p.parse_args().directory),indent=2))
