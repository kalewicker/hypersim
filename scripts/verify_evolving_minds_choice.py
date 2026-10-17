#!/usr/bin/env python3
"""Independent read-only reconstruction of the authored choice panel."""
import argparse
import copy
import csv
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import sys
import tomllib

LAW_SHA='ccd2ffc2e1c59ee12b30680cc6c0c398e8870e35b28bdafa7dac29e30d806f6e'
CLOCK,RESOURCE,CUE,COIN,SENSOR_COIN,POLICY,ATTEMPT,CHOICE,SLOT=range(241001,241010)
PHASES='Environment World Sense Signal Development Attempt Settle Maintenance Birth Death Cleanup'.split()
CONTROLLERS=['Inactive','Constant0','Constant1','Random','Reactive']
MASK=(1<<64)-1


def mix64(x):
    x=(x+0x9e3779b97f4a7c15)&MASK
    x=((x^(x>>30))*0xbf58476d1ce4e5b9)&MASK
    x=((x^(x>>27))*0x94d049bb133111eb)&MASK
    return x^(x>>31)


def balanced(seed):
    order=list(range(64));state=seed
    for i in range(63,0,-1):
        n=i+1;threshold=((-n)&MASK)%n
        while True:
            state=(state+0x9e3779b97f4a7c15)&MASK;x=mix64(state)
            if x>=threshold:break
        j=x%n;order[i],order[j]=order[j],order[i]
    bits=[False]*64
    for i in order[:32]:bits[i]=True
    return bits


def schedule(seed,mode):
    resource=balanced(seed^0x5245534f55524345)
    cues=resource[:] if mode=='Informative' else balanced(seed^0x4355455f5045524d) if mode=='Permuted' else [False]*64
    coins=[[bool(mix64(seed^0x434f494e5f535452^(site<<32)^tick)&1) for tick in range(64)] for site in range(8)]
    return dict(seed=seed,mode=mode,resources=resource,cues=cues,coins=coins)


def load_replay(root):
    spec=importlib.util.spec_from_file_location('choice_reference_replay',root/'replay.py');c=importlib.util.module_from_spec(spec)
    old=sys.dont_write_bytecode
    try:
        sys.dont_write_bytecode=True;spec.loader.exec_module(c)
    finally:sys.dont_write_bytecode=old
    return c


def checkpoint(path,require):
    data=path.read_bytes()
    require(24<=len(data)<=32*1024*1024+24 and data[:8]==b'HGEMCH01','choice checkpoint envelope')
    require(int.from_bytes(data[8:16],'little')==len(data)-24,'choice checkpoint length')
    h=0xcbf29ce484222325
    for b in data[24:]:h=((h^b)*0x100000001b3)&MASK
    require(int.from_bytes(data[16:24],'little')==h,'choice checkpoint checksum')


def verify_arm(directory,c,base,controller,seed,mode):
    require=c.require;code=CONTROLLERS.index(controller)
    table=json.loads((directory/'schedule.json').read_text());require(table==schedule(seed,mode),'schedule derivation/stream identity')
    raw=(directory/'rules.toml').read_bytes();require(raw.startswith(base),'fixed physical-law identity')
    rules_list=tomllib.loads(raw.decode())['rules'];rules={r['name']:r for r in rules_list}
    require(len(rules)==len(rules_list)==len(tomllib.loads(base.decode())['rules'])+512,'rule inventory')
    for site in range(8):
        for tick in range(64):
            r=rules[f'world_{site}_{tick}'];lhs=r['lhs'];rhs=r['rhs']
            expected={'name':f'world_{site}_{tick}','priority':0,'lhs':{'nodes':['site'],'edges':[],'node_labels':{'site':c.SITE},'node_property_guards':[dict(var='site',property=CLOCK,op='eq',value=tick),dict(var='site',property=SLOT,op='eq',value=site)]},'rhs':{'nodes':['site'],'edges':[],'property_effects':[dict(var='site',key=k,op='set',value=v) for k,v in [(RESOURCE,table['resources'][tick]),(CUE,table['cues'][tick]),(COIN,table['coins'][site][tick])]]+[dict(var='site',key=CLOCK,op='add',value=1)]}}
            require(r==expected,'world law reads/writes')
    frames=[json.loads(x) for x in (directory/'trace.jsonl').read_text().splitlines()]
    require(len(frames)==705,'complete choice phase coverage')
    nodes,edges=c.graphs(frames[0]);sites={h for h,n in nodes.items() if n['label']==c.SITE};ordered=sorted(sites)
    scaffold={(c.NEIGHBOR,tuple(sorted([h,ordered[(i+1)%8]]))) for i,h in enumerate(ordered)}
    require(len(sites)==8 and len(nodes)==12 and len(edges)==11,'initial scaffold size')
    require(frames[0]['step']==frames[0]['tick']==0 and frames[0]['phase'] is None and not frames[0]['events'],'initial choice clock')
    for h,n in nodes.items():
        expected={}
        if n['label']==c.SITE:
            expected={c.ENERGY:32.0,c.OCCUPIED:h==ordered[0],CLOCK:0,SLOT:ordered.index(h),RESOURCE:False,CUE:False,COIN:False}
        elif n['label']==c.ROOT:
            expected={c.ENERGY:24.0,c.BID:1,c.PARENT:0,c.DEV:False,c.AGE:0,c.MAINT:3.0,POLICY:code,ATTEMPT:False,CHOICE:False}
        elif n['label']==c.SENSOR:expected={c.OWNER:1,c.SIGNAL:False,SENSOR_COIN:False}
        elif n['label']==c.GENE:expected={c.OWNER:1,c.OP:code}
        elif n['label']==c.COUNTER:expected={c.NEXT:2}
        require(n['generation']==0 and n['energy']==0 and dict(n['properties'])=={k:c.typed(v) for k,v in expected.items()},'initial material/state budgets')
    lineage={1:dict(birth_id=1,parent_id=None,born_tick=0,death_tick=None,opcode=code)}

    def material():
        normalized=copy.deepcopy(nodes)
        for n in normalized.values():
            if n['label']==c.GENE:
                require(c.value(n,c.OP,'Int')==code,'faithful controller genotype');c.set_property(n,c.OP,{'Int':1})
            if n['label']==c.ROOT:
                require(c.value(n,POLICY,'Int')==code,'controller identity');c.value(n,ATTEMPT,'Bool');c.value(n,CHOICE,'Bool')
        normalized_lineage={k:{**v,'opcode':1} for k,v in lineage.items()}
        return c.material(normalized,edges,normalized_lineage,sites,scaffold)

    material();initial=c.total(nodes);influx=dissipated=harvested=0.0
    births=deaths=developed=attempts=successes=failures=0
    rows=list(csv.DictReader(io.StringIO('\n'.join((directory/'metrics.csv').read_text().splitlines()[1:]))))
    require(len(rows)==64,'choice metric coverage');work=0
    for index,f in enumerate(frames[1:],1):
        tick=(index-1)//11+1;phase=PHASES[(index-1)%11]
        require(f['step']==index and f['tick']==tick and f['phase']==phase,'choice clock/phase')
        roots,owned,occupancy=material();before=copy.deepcopy(nodes);events=f['events']
        if phase=='Environment':eligible=[h for h in sites if c.value(nodes[h],c.ENERGY,'Float')<=56]
        elif phase=='World':eligible=list(sites)
        elif phase=='Sense':eligible=list(roots.values())
        elif phase=='Signal':eligible=[r for bid,r in roots.items() if c.ACTUATOR in owned[bid]]
        elif phase=='Development':eligible=[r for r in roots.values() if not c.value(nodes[r],c.DEV,'Bool') and c.value(nodes[r],c.ENERGY,'Float')>=2]
        elif phase=='Attempt':eligible=[r for bid,r in roots.items() if code>0 and c.ACTUATOR in owned[bid] and c.value(nodes[r],c.AGE,'Int')>=1 and c.value(nodes[r],c.ENERGY,'Float')>=1 and not c.value(nodes[r],ATTEMPT,'Bool')]
        elif phase=='Settle':eligible=[r for r in roots.values() if c.value(nodes[r],ATTEMPT,'Bool')]
        elif phase=='Maintenance':eligible=list(roots.values())
        elif phase=='Death':eligible=[r for r in roots.values() if c.value(nodes[r],c.AGE,'Int')>=16 or c.value(nodes[r],c.ENERGY,'Float')<=0]
        elif phase=='Cleanup':eligible=[h for h,n in nodes.items() if n['label']==c.DEAD]
        else:
            eligible=[];candidates=[];counter=next(h for h,n in nodes.items() if n['label']==c.COUNTER)
            for bid,r in roots.items():
                n=nodes[r]
                if not c.value(n,c.DEV,'Bool') or c.value(n,c.AGE,'Int')<2 or c.value(n,c.ENERGY,'Float')<32:continue
                site=next(s for s,x in occupancy.items() if x==r)
                for e in edges.values():
                    hs=list(map(c.handle,e['nodes']))
                    if e['label']==c.NEIGHBOR and site in hs:
                        target=next(h for h in hs if h!=site)
                        if target not in occupancy:candidates.append([r,owned[bid][c.GENE],site,target,counter])
            if candidates:
                require(len(events)==1 and list(map(c.handle,events[0]['bound']))==min(candidates,key=lambda hs:sorted(map(c.rank,hs))),'native birth contention')
            else:require(not events,'unsupported paid birth')
        if phase!='Birth':
            require(len(events)==len(eligible),'mandatory choice process coverage')
            bound_eligible=[h for e in events for h in map(c.handle,e['bound']) if h in eligible]
            require(sorted(bound_eligible)==sorted(eligible),'eligible process engagement')
        stats=f['stats'];require(stats['step']==index and stats['accepted_rewrites']==len(events) and stats['property_effects_skipped']==0,'choice native counters')
        require(stats['property_effects_applied']==sum(len(rules[e['rule']]['rhs'].get('property_effects',[])) for e in events),'choice applied effects')
        require(0<=f['match_work']<=100000 and len(events)<=stats['attempted_matches']<=10000,'choice matcher ceiling');work+=f['match_work']
        for key,field in [('nodes_created','created'),('nodes_deleted','deleted'),('edges_created','edges_created'),('edges_deleted','edges_deleted')]:
            require(stats[key]==sum(len(e[field]) for e in events),'choice material effect counters')
        for e in events:
            require(e['rule'] in rules and e['rule'].startswith(phase.lower()+'_'),'choice phase rule')
            if phase=='Settle':
                r=next(h for h in map(c.handle,e['bound']) if before[h]['label']==c.ROOT)
                site=next(s for s,x in occupancy.items() if x==r)
                success=c.value(before[r],CHOICE,'Bool')==c.value(before[site],RESOURCE,'Bool') and c.value(before[r],c.ENERGY,'Float')<=40 and c.value(before[site],c.ENERGY,'Float')>=8
                require(e['rule']==('settle_success' if success else 'settle_failure'),'wrong choice outcome/priority')
            if phase=='Maintenance':
                r=next(h for h in map(c.handle,e['bound']) if before[h]['label']==c.ROOT)
                dissipated+=min(c.value(before[r],c.ENERGY,'Float'),c.value(before[r],c.MAINT,'Float'))
            if e['rule']=='environment_influx':influx+=8
            if e['rule']=='development_relay':developed+=1;dissipated+=2
            if e['rule']=='attempt_commit':attempts+=1;dissipated+=1
            if e['rule']=='settle_success':successes+=1;harvested+=8
            if e['rule']=='settle_failure':failures+=1
            b=c.apply(nodes,edges,e,rules[e['rule']],index-1)
            if phase=='Birth':
                child=nodes[b['child']];bid=c.value(child,c.BID,'Int');parent=c.value(child,c.PARENT,'Int')
                require(bid not in lineage and parent in lineage and lineage[parent]['death_tick'] is None,'choice ancestry')
                lineage[bid]=dict(birth_id=bid,parent_id=parent,born_tick=tick,death_tick=None,opcode=code);births+=1;dissipated+=2
            if phase=='Death':
                bid=c.value(before[b['r']],c.BID,'Int');require(lineage[bid]['death_tick'] is None,'duplicate death');lineage[bid]['death_tick']=tick;deaths+=1
        require(c.graphs(f)==(nodes,edges),'independent native choice replay')
        material();require(c.total(nodes)==initial+influx-dissipated,'independent choice energy ledger')
        if phase=='Cleanup':
            require(attempts==successes+failures and all(not c.value(n,ATTEMPT,'Bool') for n in nodes.values() if n['label']==c.ROOT),'unsettled/free retry attempt')
            expected=dict(tick=tick,engine_step=index,live_organisms=sum(n['label']==c.ROOT for n in nodes.values()),births=births,deaths=deaths,developed=developed,attempts=attempts,successes=successes,failures=failures,harvested=harvested,influx=influx,dissipated=dissipated,total_energy=c.total(nodes),residual=0,match_work=work)
            require(set(rows[tick-1])==set(expected) and all(float(rows[tick-1][k])==v for k,v in expected.items()),'independent choice CSV');work=0
    require(json.loads((directory/'organisms.json').read_text())=={str(k):v for k,v in lineage.items()},'choice lineage artifact')
    status=json.loads((directory/'run_status.json').read_text());manifest=json.loads((directory/'manifest.json').read_text())
    require(status['controller']==controller and status['seed']==seed and status['cue_mode']==mode and status['final_row']==expected,'choice final status')
    require(status['profile']=='evolving-minds-choice-v1' and status['engineering_execution_complete'] is True and status['development_only'] is True and all(status[k] is False for k in ['scientific_run_launched','gate_pass','training_admitted','evolution_enabled']),'choice scientific boundary')
    require(manifest['state']=='Completed' and manifest['final_step']==manifest['max_steps']==704 and manifest['seed']==seed,'choice manifest')
    require({p.name for p in directory.glob('*.checkpoint')}=={'initial.checkpoint','midpoint.checkpoint','final.checkpoint'},'choice checkpoint coverage')
    for p in directory.glob('*.checkpoint'):checkpoint(p,require)
    require(sum(p.stat().st_size for p in directory.iterdir() if p.is_file())<=64*1024*1024,'choice output ceiling')
    depth={1:0}
    for bid,b in lineage.items():
        if bid!=1:depth[bid]=depth[b['parent_id']]+1
    return dict(controller=controller,seed=seed,cue_mode=mode,final_row=expected,max_ancestry_depth=max(depth.values()),cue_agreement=sum(a==b for a,b in zip(table['resources'],table['cues']))/64),rows,status


def verify(root):
    root=Path(root);c=load_replay(root);require=c.require
    require(root.is_dir() and not any(p.is_symlink() for p in root.rglob('*')),'choice evidence path')
    require(sum(p.stat().st_size for p in root.rglob('*') if p.is_file())<=256*1024*1024,'choice panel output ceiling')
    base=(root/'base_rules.toml').read_bytes();require(hashlib.sha256(base).hexdigest()==LAW_SHA,'choice base law identity')
    results={};statuses=[];metrics={}
    expected_dirs={f's{s}_{m.lower()}_{p.lower()}' for s in [17,29] for m in ['Informative','Permuted','Withheld'] for p in CONTROLLERS}
    require({p.name for p in root.iterdir() if p.is_dir()}==expected_dirs,'choice cell denominator')
    for seed in [17,29]:
        for mode in ['Informative','Permuted','Withheld']:
            for controller in CONTROLLERS:
                name=f's{seed}_{mode.lower()}_{controller.lower()}'
                result,rows,status=verify_arm(root/name,c,base,controller,seed,mode)
                results[name]=result;statuses.append(status);metrics[name]=rows
    require(json.loads((root/'summary.json').read_text())==statuses,'panel summary identity')
    for seed in [17,29]:
        for controller in ['inactive','constant0','constant1','random']:
            require(metrics[f's{seed}_informative_{controller}']==metrics[f's{seed}_permuted_{controller}']==metrics[f's{seed}_withheld_{controller}'],'uninformed control cue leakage')
    headroom=[]
    for seed in [17,29]:
        reactive=results[f's{seed}_informative_reactive']
        strongest=max(results[f's{seed}_informative_{p}']['final_row']['harvested'] for p in ['constant0','constant1','random'])
        gain=reactive['final_row']['harvested']-strongest
        require(gain>=16 and reactive['max_ancestry_depth']>=3 and reactive['final_row']['live_organisms']>0,'prospective information/viability calibration')
        a=metrics[f's{seed}_withheld_reactive'];b=metrics[f's{seed}_withheld_constant0']
        require(all({k:v for k,v in x.items() if k!='match_work'}=={k:v for k,v in y.items() if k!='match_work'} for x,y in zip(a,b)),'withheld reactive constant-choice control')
        headroom.append(dict(seed=seed,reactive_harvested=reactive['final_row']['harvested'],strongest_uninformed_harvested=strongest,margin=gain))
    status=json.loads((root/'run_status.json').read_text());manifest=json.loads((root/'manifest.json').read_text())
    require(status['engineering_execution_complete'] is True and status['development_only'] is True and all(status[k] is False for k in ['scientific_run_launched','gate_pass','training_admitted','evolution_enabled']),'panel scientific boundary')
    require(manifest['state']=='Completed' and manifest['final_step']==manifest['max_steps']==21120,'panel completion')
    return dict(profile='evolving-minds-choice-v1',audit_pass=True,development_only=True,scientific_run_launched=False,gate_pass=False,training_admitted=False,cells_verified=30,headroom=headroom,cells=results)


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('directory',type=Path)
    print(json.dumps(verify(parser.parse_args().directory),indent=2))
