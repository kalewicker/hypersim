#!/usr/bin/env python3
"""Independently reconstruct the fixed paid supplied-edit diagnostic."""
import argparse
import datetime
import hashlib
import json
from pathlib import Path
import time
import tomllib
import sys
sys.dont_write_bytecode = True
# Loaded from the frozen archive when this checker is executed there.
try:
    import opportunity_reference as common
except ModuleNotFoundError:
    import evolving_minds_opportunity as common
PROFILE='evolving-minds-d0-paid-descendant-neighborhood-development-v1'
STREAM='evolving-minds-d0-paid-descendant-streaming-v1'
FLAGS=dict(scientific_acceptance=False,sustained_claim=False,development_only=True,scientific_run_launched=False,gate_pass=False,training_admitted=False,evolution_enabled=False,reproductive_fairness_accepted=False,power_adequacy_established=False,ready_for_scientific_launch=False)
require=common.require
data=common.data
load=common.load
WORLDS=[601,607]

def cases():
    rows=[]
    def add(seed,target,source,fault='Normal'):
        arm=f'g{target}_s{source}' if target is not None else 'unchanged'
        rows.append(dict(name=f'world{seed}_{arm}',arm=arm,config=dict(seed=seed,endowment=24,founders=['Delayed']*2,kernels=['Faithful']*2,home_offsets=[0,4],rotation=0,reversed=False,horizon=64,scheduler='semantic-lottery-v1'),descendant=dict(policy='first-paid-descendant-by24-v1',target_id=target,new_source=source),fault=fault))
    for seed in WORLDS:
        add(seed,None,None)
        for target in range(100,105):
            for source in range(1,8):add(seed,target,source)
    add(17,103,1,'CleanStop');rows[-1].update(name='world17_recovery',arm='recovery')
    return rows

def classify(result):
    o=result['opportunity'];records=result['records'];state=result['target_state']
    require([t['tick'] for t in o['ticks']]==list(range(1,65)),'descendant full horizon')
    require({m['id'] for m in o['lifetimes'] if m['parent'] is None}=={1,2},'descendant founder denominator')
    members={m['id']:m for m in o['lifetimes']};rows=[]
    for founder in [1,2]:
        sample=state['targets'].get(str(founder))
        family_requests=[q for q in records if q['parent']>2 and members[q['parent']]['founder']==founder and q['tick']<=24]
        first=min(family_requests,key=lambda q:(q['tick'],q['parent'])) if family_requests else None
        require((sample is None)==(first is None),'first paid target omission')
        if sample is None:
            rows.append(dict(founder=founder,category='no_early_request',completed=False,reproduced=False,target_child=None));continue
        require(0<sample['tick']<=24 and sample['parent']>2,'early paid target boundary')
        r=next(q for q in records if q['request_id']==sample['request_id'])
        require(r['request_id']==first['request_id'],'first paid target substitution')
        require(members[r['parent']]['founder']==founder and r['operator']=='supplied_descendant','paid target family provenance')
        child=members.get(r['child_id']);category='syntax_rejected' if not r['syntax_valid'] else 'paid_unreachable' if not r['material_reachable'] else 'reachable_unchanged' if r['parent_program']==r['proposed_program'] else 'reachable_edited'
        require(r['request_charge']==1 and r['refunded']==(29 if not r['syntax_valid'] else 0) and r['copying_charge']==(5 if r['syntax_valid'] else 0),'target paid settlement')
        rows.append(dict(founder=founder,category=category,request=r,completed=child is not None and child['completion_tick'] is not None,reproduced=child is not None and child['requests']>0,target_child=child))
    return rows

def decision(results):
    require(set(results)=={x['name'] for x in cases()},'fixed descendant result denominator')
    rows=[]
    for case in cases():
        require(results[case['name']]['target_state']['config']==case['descendant'],'target configuration denominator')
        if case['arm']=='recovery':continue
        for r in classify(results[case['name']]):rows.append(dict(world=case['config']['seed'],arm=case['arm'],**r))
    substitutions=[r for r in rows if r['arm']!='unchanged']
    totals={category:sum(r['category']==category for r in substitutions) for category in ['no_early_request','syntax_rejected','paid_unreachable','reachable_unchanged','reachable_edited']}
    missing=[dict(world=r['world'],arm=r['arm'],founder=r['founder']) for r in rows if r['category']=='no_early_request']
    neighborhoods=[]
    for world in WORLDS:
        for founder in [1,2]:
            panel=[r for r in substitutions if r['world']==world and r['founder']==founder]
            require(len(panel)==35,'fixed family source neighborhood')
            neighborhoods.append(dict(world=world,founder=founder,denominator=35,syntax_valid=sum(r['category'] not in ['syntax_rejected','no_early_request'] for r in panel),material_reachable=sum(r['category'].startswith('reachable_') for r in panel),completed=sum(r['completed'] for r in panel),reproducing=sum(r['reproduced'] for r in panel),no_early_request=sum(r['category']=='no_early_request' for r in panel)))
    return dict(neighborhoods=neighborhoods,main_cells=72,family_target_denominator=144,substitution_denominator=140,unchanged_target_denominator=4,
        early_exposure_screen_pass=not missing,missing_targets=missing,category_counts=totals,targets=rows,
        completed_substitution_children=sum(r['completed'] for r in substitutions),reproducing_substitution_children=sum(r['reproduced'] for r in substitutions),
        empirical_scientific_power_estimated=False,**FLAGS)

def verify(root):
    root=Path(root);start=time.monotonic();before=common.inventory(root)
    require(data(root/'registered_cases.json')==cases(),'fixed native descendant cases')
    require({p.name for p in root.iterdir() if p.is_dir()}=={x['name'] for x in cases()}|{'world17_recovery_resumed'},'fixed thirteen-segment denominator')
    sources=data(root/'sources.json')
    bindings=[('checker.py','scripts/verify_evolving_minds_descendant.py'),('stream_reference.py','scripts/verify_evolving_minds_streaming.py'),('admission_reference.py','scripts/verify_evolving_minds_admission.py'),('funding_reference.py','scripts/verify_evolving_minds_funding.py'),('opportunity_reference.py','scripts/evolving_minds_opportunity.py'),('shared_replay.py','scripts/evolving_minds_shared_replay.py'),('replay.py','scripts/verify_evolving_minds.py'),('d0_reference.py','scripts/verify_evolving_minds_d0.py'),('schedule_reference.py','scripts/verify_evolving_minds_choice.py'),('contract.md','experiments/evolving_minds/contracts/d0_paid_descendant_neighborhood_v1.md'),('base_rules.toml','rules/evolving_minds/admission_v1.toml')]
    for name,path in bindings:require((root/name).read_text()==sources[path],'descendant frozen source binding')
    v=load(root/'stream_reference.py','descendant_stream');a=load(root/'admission_reference.py','descendant_admission');c=load(root/'replay.py','descendant_replay');d=load(root/'d0_reference.py','descendant_d0');s=load(root/'schedule_reference.py','descendant_schedule');f=load(root/'funding_reference.py','descendant_funding');o=load(root/'opportunity_reference.py','descendant_observer');shared=load(root/'shared_replay.py','descendant_shared')
    modules=(a,c,d,s);base=(root/'base_rules.toml').read_bytes();laws=(root/'b24_base_rules.toml').read_bytes()
    require(hashlib.sha256(base).hexdigest()==a.LAW_SHA and tomllib.loads(laws.decode())==f.funded_laws(base,24),'independent descendant physics')
    results={};resources={};reports=[];states=0;derivations=[]
    for case in cases():
        name=case['name'];p=root/name;cfg=case['config'];h=case['descendant'];resumed=case['fault']=='CleanStop'
        chain=v.inspect_segment(root/'world17_recovery_resumed' if resumed else p,funded_config=cfg,descendant_config=h)
        require(len(chain)==(2 if resumed else 1),'declared descendant recovery chain')
        for seg in chain:
            directory=seg['directory'];m=data(directory/'manifest.json');r=data(directory/'segment_report.json');status=data(directory/'run_status.json');v.boundary(status,STREAM)
            end=24 if directory==p and resumed else 64;begin=24 if directory.name.endswith('_resumed') else 0
            require(m['experiment_name']==STREAM and m['seed']==cfg['seed'] and m['thread_count']==4 and m['max_steps']==2944 and m['final_step']==end*46 and m['state']==r['state']==('Interrupted' if end==24 else 'Completed') and r['funded_config']==cfg and r['descendant_config']==h and r.get('retained_policy') is None and r['start_tick']==begin and r['native_final_tick']==r['committed_final_tick']==end and r['fault']==('CleanStop' if end==24 else 'Normal') and status['engineering_execution_complete']==(end==64),'actual descendant lifecycle/configuration')
            resource=r['resources'];actual=sum(q.stat().st_size for q in directory.iterdir())
            require(resource['drained_history_empty'] and resource['advance_seconds']<=120 and resource['payload_bytes']==(directory/'ticks.jsonl').stat().st_size<=16*1024**2 and actual<=32*1024**2 and resource['checkpoints']==[[x['marker']['cursor']['tick'],x['checkpoint_bytes']] for x in seg['commits']] and max(x['checkpoint_bytes'] for x in seg['commits'])<=2*1024**2,'descendant segment resource caps')
            resources[directory.name]=dict(observed_total_bytes=actual,**resource)
        if resumed:reports.append(data(root/'world17_recovery_resumed/segment_report.json'))
        reports.append(data(p/'segment_report.json'))
        then=time.monotonic()
        final,physics,opportunity,count=shared.optimized_cell(chain,modules,laws,cfg,None,v,o,start,derivations,descendant_config=h)
        states+=count
        results[name]=dict(opportunity=opportunity,final_row=physics['final_row'],records=final['admission']['records'],target_state=chain[-1]['commits'][-1]['state']['native']['descendant'],audit_seconds=time.monotonic()-then)
        records=results[name]['records']
        print(json.dumps(dict(cell=name,verified_states=states)),file=sys.stderr,flush=True)
        classify(results[name])
    for seed in WORLDS:
        for case in cases():
            if case['config']['seed']!=seed:continue
            for filename in ['schedule.json','rules.toml','initial.json','initial_lineage.json']:
                require((root/f'world{seed}_unchanged'/filename).read_bytes()==(root/case['name']/filename).read_bytes(),'matched descendant initial world and physics')
    require(data(root/'summary.json')==reports,'descendant report denominator')
    require(data(root/'execution_counts.json')==dict(fresh_worlds=2,main_arms_per_world=36,initial_cells=73,segments=74,ticks=4672,native_advances_executed=214912),'descendant native execution denominator')
    m=data(root/'manifest.json');elapsed=(datetime.datetime.fromisoformat(m['end_time'])-datetime.datetime.fromisoformat(m['start_time'])).total_seconds()
    require(m['state']=='Completed' and m['experiment_name']==PROFILE and m['seed']==601 and m['thread_count']==4 and m['final_step']==m['max_steps']==214912 and elapsed<=3600 and sum(r['advance_seconds'] for r in resources.values())<=2400,'descendant root lifecycle/resources')
    status=data(root/'run_status.json');v.boundary(status,PROFILE);require(status['engineering_execution_complete'] and states==658,'descendant complete published state denominator')
    require(time.monotonic()-start<=3600 and common.inventory(root)==before,'read-only bounded descendant audit')
    return dict(profile=PROFILE,**FLAGS,audit_pass=True,cells_verified=73,segments_verified=74,published_states_reconstructed=states,native_advances_executed=214912,audit_seconds=time.monotonic()-start,native_panel_elapsed_seconds=elapsed,results=results,resources=resources,decision=decision(results),derivations=derivations,source_sha256={p:hashlib.sha256(raw.encode()).hexdigest() for p,raw in sources.items()})

if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('directory');args=parser.parse_args();print(json.dumps(verify(args.directory),indent=2))
