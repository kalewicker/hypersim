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
PROFILE='evolving-minds-d0-paid-handoff-development-v1'
STREAM='evolving-minds-d0-handoff-streaming-v1'
FLAGS=dict(development_only=True,scientific_run_launched=False,gate_pass=False,training_admitted=False,evolution_enabled=False,reproductive_fairness_accepted=False,power_adequacy_established=False,ready_for_scientific_launch=False)
require=common.require
data=common.data
load=common.load
WORLDS=[401,403,407,409]

def cases():
    rows=[]
    def add(seed,arm,supplied,kernel,fault='Normal'):
        rows.append(dict(name=f'world{seed}_{arm}',arm=arm,config=dict(seed=seed,endowment=24,founders=['Delayed']*2,kernels=[kernel]*2,home_offsets=[0,4],rotation=0,reversed=False,horizon=64,scheduler='semantic-lottery-v1'),handoff=dict(policy='paid-beneficial-edit-v1',supplied=supplied,descendant_kernel=kernel),fault=fault))
    for seed in WORLDS:
        for arm,kernel in [('faithful','Faithful'),('source','SourceKernel')]:add(seed,arm,'Useful',kernel)
    for arm,supplied,fault in [('unchanged','Unchanged','Normal'),('invalid','Invalid','Normal'),('unreachable','Unreachable','Normal'),('recovery','Useful','CleanStop')]:add(17,arm,supplied,'Faithful',fault)
    return rows

def decision(results):
    require(set(results)=={x['name'] for x in cases()},'fixed handoff result denominator')
    worlds=[];failed=[]
    for seed in WORLDS:
        arms={}
        for arm in ['faithful','source']:
            name=f'world{seed}_{arm}';r=results[name];o=r['opportunity'];requests=r['records'];members=o['lifetimes']
            require(len(o['ticks'])==64,'handoff full horizon')
            require({m['id'] for m in members if m['parent'] is None}=={1,2},'handoff founder denominator')
            require(all(m['attempts']==m['successes']+m['failures']==m['payable_action_opportunities'] for m in members),'handoff paid attempt accounting')
            byid={m['id']:m for m in members};checks={}
            for founder in [1,2]:
                paid=[q for q in requests if q['parent']==founder]
                completed=any(q['child_id'] is not None and byid[q['child_id']]['completion_tick'] is not None for q in paid)
                exposed=any(m['parent'] is not None and m['founder']==founder and m['requests']>0 for m in members)
                checks[f'founder{founder}_paid_completed_handoff']=bool(paid) and completed
                if arm=='faithful':checks[f'founder{founder}_paid_descendant_request']=exposed
            failed.extend(f'{name}:{k}' for k,v in checks.items() if not v)
            arms[arm]=dict(checks=checks,primary_descendant_harvested=o['primary_descendant_harvested'],descendant_requests=sum(m['requests'] for m in members if m['parent'] is not None),descendant_denominator=o['descendant_denominator'],descendants_completed=o['descendants_completed'],max_depth=r['final_row']['max_ancestry_depth'])
        worlds.append(dict(world=seed,arms=arms,source_minus_faithful=arms['source']['primary_descendant_harvested']-arms['faithful']['primary_descendant_harvested']))
    return dict(handoff_exposure_screen_pass=not failed,failed_checks=failed,worlds=worlds,scientific_acceptance=False,sustained_claim=False,**FLAGS)

def verify(root):
    root=Path(root);start=time.monotonic();before=common.inventory(root)
    require(data(root/'registered_cases.json')==cases(),'fixed native handoff cases')
    require({p.name for p in root.iterdir() if p.is_dir()}=={x['name'] for x in cases()}|{'world17_recovery_resumed'},'fixed thirteen-segment denominator')
    sources=data(root/'sources.json')
    bindings=[('checker.py','scripts/verify_evolving_minds_handoff.py'),('stream_reference.py','scripts/verify_evolving_minds_streaming.py'),('admission_reference.py','scripts/verify_evolving_minds_admission.py'),('funding_reference.py','scripts/verify_evolving_minds_funding.py'),('opportunity_reference.py','scripts/evolving_minds_opportunity.py'),('shared_replay.py','scripts/evolving_minds_shared_replay.py'),('replay.py','scripts/verify_evolving_minds.py'),('d0_reference.py','scripts/verify_evolving_minds_d0.py'),('schedule_reference.py','scripts/verify_evolving_minds_choice.py'),('contract.md','experiments/evolving_minds/contracts/d0_paid_handoff_v1.md'),('base_rules.toml','rules/evolving_minds/admission_v1.toml')]
    for name,path in bindings:require((root/name).read_text()==sources[path],'handoff frozen source binding')
    v=load(root/'stream_reference.py','handoff_stream');a=load(root/'admission_reference.py','handoff_admission');c=load(root/'replay.py','handoff_replay');d=load(root/'d0_reference.py','handoff_d0');s=load(root/'schedule_reference.py','handoff_schedule');f=load(root/'funding_reference.py','handoff_funding');o=load(root/'opportunity_reference.py','handoff_observer');shared=load(root/'shared_replay.py','handoff_shared')
    modules=(a,c,d,s);base=(root/'base_rules.toml').read_bytes();laws=(root/'b24_base_rules.toml').read_bytes()
    require(hashlib.sha256(base).hexdigest()==a.LAW_SHA and tomllib.loads(laws.decode())==f.funded_laws(base,24),'independent handoff physics')
    results={};resources={};reports=[];states=0;derivations=[]
    for case in cases():
        name=case['name'];p=root/name;cfg=case['config'];h=case['handoff'];resumed=case['fault']=='CleanStop'
        chain=v.inspect_segment(root/'world17_recovery_resumed' if resumed else p,funded_config=cfg,handoff_config=h)
        require(len(chain)==(2 if resumed else 1),'declared handoff recovery chain')
        for seg in chain:
            directory=seg['directory'];m=data(directory/'manifest.json');r=data(directory/'segment_report.json');status=data(directory/'run_status.json');v.boundary(status,STREAM)
            end=24 if directory==p and resumed else 64;begin=24 if directory.name.endswith('_resumed') else 0
            require(m['experiment_name']==STREAM and m['seed']==cfg['seed'] and m['thread_count']==4 and m['max_steps']==2944 and m['final_step']==end*46 and m['state']==r['state']==('Interrupted' if end==24 else 'Completed') and r['funded_config']==cfg and r['handoff_config']==h and r.get('retained_policy') is None and r['start_tick']==begin and r['native_final_tick']==r['committed_final_tick']==end and r['fault']==('CleanStop' if end==24 else 'Normal') and status['engineering_execution_complete']==(end==64),'actual handoff lifecycle/configuration')
            resource=r['resources'];actual=sum(q.stat().st_size for q in directory.iterdir())
            require(resource['drained_history_empty'] and resource['advance_seconds']<=120 and resource['payload_bytes']==(directory/'ticks.jsonl').stat().st_size<=16*1024**2 and actual<=32*1024**2 and resource['checkpoints']==[[x['marker']['cursor']['tick'],x['checkpoint_bytes']] for x in seg['commits']] and max(x['checkpoint_bytes'] for x in seg['commits'])<=2*1024**2,'handoff segment resource caps')
            resources[directory.name]=dict(observed_total_bytes=actual,**resource)
        if resumed:reports.append(data(root/'world17_recovery_resumed/segment_report.json'))
        reports.append(data(p/'segment_report.json'))
        then=time.monotonic()
        final,physics,opportunity,count=shared.optimized_cell(chain,modules,laws,cfg,None,v,o,start,derivations,handoff_config=h)
        states+=count
        results[name]=dict(opportunity=opportunity,final_row=physics['final_row'],records=final['admission']['records'],audit_seconds=time.monotonic()-then)
        records=results[name]['records']
        if case['arm']=='unchanged':require(records and all(q['parent_program']==q['proposed_program'] for q in records),'unchanged paid control')
        if case['arm']=='invalid':require(records and all(not q['syntax_valid'] and q['refunded']==29 and q['copying_charge']==0 and q['child_id'] is None for q in records),'invalid full refund control')
        if case['arm']=='unreachable':require(records and all(q['syntax_valid'] and not q['material_reachable'] and q['copying_charge']==5 and q['refunded']==0 for q in records) and physics['final_row']['development_failures']>=2,'unreachable paid material control')
    for seed in WORLDS:
        for filename in ['schedule.json','rules.toml','initial.json','initial_lineage.json']:
            require((root/f'world{seed}_faithful'/filename).read_bytes()==(root/f'world{seed}_source'/filename).read_bytes(),'matched handoff founders/world/physics')
    require(data(root/'summary.json')==reports,'handoff report denominator')
    require(data(root/'execution_counts.json')==dict(fresh_worlds=4,main_arms_per_world=2,initial_cells=12,segments=13,ticks=768,native_advances_executed=35328),'handoff native execution denominator')
    m=data(root/'manifest.json');elapsed=(datetime.datetime.fromisoformat(m['end_time'])-datetime.datetime.fromisoformat(m['start_time'])).total_seconds()
    require(m['state']=='Completed' and m['experiment_name']==PROFILE and m['seed']==401 and m['thread_count']==4 and m['final_step']==m['max_steps']==35328 and elapsed<=3600 and sum(r['advance_seconds'] for r in resources.values())<=2400,'handoff root lifecycle/resources')
    status=data(root/'run_status.json');v.boundary(status,PROFILE);require(status['engineering_execution_complete'] and states==109,'handoff complete published state denominator')
    require(time.monotonic()-start<=3600 and common.inventory(root)==before,'read-only bounded handoff audit')
    return dict(profile=PROFILE,**FLAGS,audit_pass=True,cells_verified=12,segments_verified=13,published_states_reconstructed=states,native_advances_executed=35328,audit_seconds=time.monotonic()-start,native_panel_elapsed_seconds=elapsed,results=results,resources=resources,decision=decision(results),derivations=derivations,source_sha256={p:hashlib.sha256(raw.encode()).hexdigest() for p,raw in sources.items()})

if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('directory');args=parser.parse_args();print(json.dumps(verify(args.directory),indent=2))
