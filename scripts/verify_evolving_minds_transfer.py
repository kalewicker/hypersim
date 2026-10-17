#!/usr/bin/env python3
"""Independently reconstruct the TA-1 Gate0 authored inheritance assay."""
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
PROFILE='evolving-minds-ta1-gate0-development-v1'
STREAM='evolving-minds-d0-handoff-streaming-v1'
FLAGS=dict(development_only=True,scientific_run_launched=False,gate_pass=False,training_admitted=False,evolution_enabled=False,reproductive_fairness_accepted=False,power_adequacy_established=False,ready_for_scientific_launch=False)
require=common.require
data=common.data
load=common.load
WORLDS=[17,23]

def cases():
    rows=[]
    for seed, arm, supplied, fault in [
        (17,'useful','Useful','Normal'), (17,'ancestor','Unchanged','Normal'),
        (23,'useful','Useful','Normal'), (23,'ancestor','Unchanged','Normal'),
        (17,'recovery','Useful','CleanStop')]:
        rows.append(dict(name=f'world{seed}_{arm}',arm=arm,config=dict(
            seed=seed,endowment=24,founders=['Delayed']*2,kernels=['Faithful']*2,
            home_offsets=[0,4],rotation=0,reversed=False,horizon=64,
            scheduler='semantic-lottery-v1'),handoff=dict(policy='paid-beneficial-edit-v1',
            supplied=supplied,descendant_kernel='Faithful'),fault=fault))
    return rows


def registration():
    return dict(config=dict(profile=PROFILE,development_only=True,worlds=WORLDS,
        horizon=64,required_faithful_active_transmissions=3,
        minimum_active_offspring_gain=1,minimum_net_resource_gain=8),cases=cases(),
        counts=dict(main_worlds=2,main_cells=4,initial_cells=5,segments=6,ticks=320,
                    native_advances_executed=14720,published_states=46),
        scientific_run_launched=False,transfer_established=False,
        discovery_established=False,cumulative_reuse_established=False,
        limits=dict(panel_bytes=536870912,native_seconds=3600,audit_seconds=3600,
                    summed_advance_seconds=2400))


def endpoints(result, useful):
    opportunity=result['opportunity']; members=opportunity['lifetimes']
    byid={m['id']:m for m in members}
    require(len(byid)==len(members) and {m['id'] for m in members if m['parent'] is None}=={1,2},
            'complete unique root/founder denominator')
    require(len(opportunity['ticks'])==64,'Gate0 full horizon')
    charges=['development_charge','attempt_charge','maintenance_charge','request_charge','copying_charge']
    harvested=sum(m['harvested'] for m in members)
    dissipated=sum(m[k] for m in members for k in charges)
    require(harvested==result['final_row']['harvested'] and
            dissipated==result['final_row']['dissipated'],'Gate0 net resource reconciliation')
    records=result['records']; paid={}
    for r in records:
        require(r['settled'] and r['parent'] in byid,'settled paid request denominator')
        if r['child_id'] is None:
            continue
        child=byid.get(r['child_id'])
        require(child is not None and child['parent']==r['parent'] and
                child['born_tick']==r['tick'] and child['program']==r['proposed_program'] and
                r['request_charge']==1 and r['copying_charge']==5 and r['refunded']==0 and
                child['initial_reserve']==24,'Gate0 paid native birth')
        require(child['id'] not in paid,'unique native paid child')
        paid[child['id']]=r
    require(set(paid)=={m['id'] for m in members if m['parent'] is not None},'every child has a paid birth')
    for m in members:
        require(m['attempts']==m['successes']+m['failures']==m['payable_action_opportunities'],
                'Gate0 paid action accounting')
        if m['parent'] is not None:
            require(byid[m['parent']]['born_tick']<m['born_tick'],'chronological ancestry')
    active=lambda m: m['completion_tick'] is not None and m['successes']>=1
    families=[]
    for founder in [1,2]:
        levels={};paths={};installations=[]
        for m in sorted(members,key=lambda m:(m['born_tick'],m['id'])):
            if m['parent'] is None: continue
            r=paid[m['id']]
            if m['parent']==founder:
                if useful:
                    # Physical replay also verifies this exact supplied proposal.
                    require(r['target_id']==103 and r['old_source']==2 and r['new_source']==0,
                            'known useful installation')
                    installations.append(m['id'])
                    if active(m): levels[m['id']]=0;paths[m['id']]=[m['id']]
            elif useful and m['parent'] in levels and active(m) and m['program']==byid[m['parent']]['program']:
                levels[m['id']]=levels[m['parent']]+1
                paths[m['id']]=paths[m['parent']]+[m['id']]
        best=min(levels,key=lambda i:(-levels[i],i)) if levels else None
        families.append(dict(founder=founder,installation_children=installations,
            max_active_faithful_transmissions=0 if best is None else levels[best],
            witness_path=[] if best is None else paths[best],
            persistence_pass=best is not None and levels[best]>=3))
    return dict(active_completed_offspring=sum(m['parent'] is not None and
        m['completion_tick'] is not None and m['attempts']>=1 for m in members),
        born_offspring=len(paid),harvested=harvested,dissipated=dissipated,
        net_resource_gain=harvested-dissipated,
        descendant_harvested=opportunity['primary_descendant_harvested'],families=families)


def decision(results):
    require(set(results)=={x['name'] for x in cases()},'fixed Gate0 result denominator')
    worlds=[]; failed=[]
    for seed in WORLDS:
        useful=endpoints(results[f'world{seed}_useful'],True)
        ancestor=endpoints(results[f'world{seed}_ancestor'],False)
        offspring=useful['active_completed_offspring']-ancestor['active_completed_offspring']
        resource=useful['net_resource_gain']-ancestor['net_resource_gain']
        checks=dict(active_offspring_gain=offspring>=1,net_resource_gain=resource>=8,
                    **{f"family{f['founder']}_persistence":f['persistence_pass'] for f in useful['families']})
        failed.extend(f'world{seed}:{k}' for k,v in checks.items() if not v)
        worlds.append(dict(world=seed,useful=useful,ancestor=ancestor,
            useful_minus_ancestor_active_offspring=offspring,
            useful_minus_ancestor_net_resource_gain=resource,checks=checks))
    # Recovery is a semantic equality control, not another observation.
    for key in ['opportunity','final_row','records']:
        require(results['world17_recovery'][key]==results['world17_useful'][key],
                'Gate0 fresh-process recovery exact '+key)
    return dict(gate0_feasibility_pass=not failed,failed_checks=failed,worlds=worlds,
        next_gate_automatically_authorized=False,transfer_established=False,
        discovery_established=False,cumulative_reuse_established=False,
        scientific_acceptance=False,sustained_claim=False,**FLAGS)


def verify(root):
    root=Path(root);start=time.monotonic();before=common.inventory(root)
    hashes=dict(before[0]); hashes.pop('output_sha256.json',None)
    seal=data(root/'output_sha256.json')
    require(seal==dict(profile=PROFILE,files_sha256=hashes,development_only=True,
                      scientific_run_launched=False),'complete sealed Gate0 payload')
    require(Path(__file__).read_bytes()==(root/'checker.py').read_bytes(),'executed frozen Gate0 checker')
    require(data(root/'registered_cases.json')==cases(),'fixed native handoff cases')
    require({p.name for p in root.iterdir() if p.is_dir()}=={x['name'] for x in cases()}|{'world17_recovery_resumed'},'fixed six-segment denominator')
    sources=data(root/'sources.json')
    require(data(root/'registration.json')==registration(),'fixed complete Gate0 registration')
    require(len((root/'binary_sha256.txt').read_text())==64 and all(c in '0123456789abcdef' for c in (root/'binary_sha256.txt').read_text()),'binary identity receipt')
    bindings=[('checker.py','scripts/verify_evolving_minds_transfer.py'),('stream_reference.py','scripts/verify_evolving_minds_streaming.py'),('admission_reference.py','scripts/verify_evolving_minds_admission.py'),('funding_reference.py','scripts/verify_evolving_minds_funding.py'),('opportunity_reference.py','scripts/evolving_minds_opportunity.py'),('shared_replay.py','scripts/evolving_minds_shared_replay.py'),('replay.py','scripts/verify_evolving_minds.py'),('d0_reference.py','scripts/verify_evolving_minds_d0.py'),('schedule_reference.py','scripts/verify_evolving_minds_choice.py'),('contract.md','experiments/evolving_minds/contracts/ta1_gate0_v1.md'),('base_rules.toml','rules/evolving_minds/admission_v1.toml')]
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
        if case['arm']=='ancestor':require(records and all(q['parent_program']==q['proposed_program'] for q in records),'unchanged paid control')
    for seed in WORLDS:
        for filename in ['schedule.json','rules.toml','initial.json','initial_lineage.json']:
            require((root/f'world{seed}_useful'/filename).read_bytes()==(root/f'world{seed}_ancestor'/filename).read_bytes(),'matched handoff founders/world/physics')
    require(data(root/'summary.json')==reports,'handoff report denominator')
    require(data(root/'execution_counts.json')==registration()['counts'],'handoff native execution denominator')
    m=data(root/'manifest.json');elapsed=(datetime.datetime.fromisoformat(m['end_time'])-datetime.datetime.fromisoformat(m['start_time'])).total_seconds()
    require(m['state']=='Completed' and m['experiment_name']==PROFILE and m['seed']==17 and m['thread_count']==4 and m['final_step']==m['max_steps']==14720 and elapsed<=3600 and sum(r['advance_seconds'] for r in resources.values())<=2400,'handoff root lifecycle/resources')
    status=data(root/'run_status.json');v.boundary(status,PROFILE);require(status['engineering_execution_complete'] and states==46,'handoff complete published state denominator')
    require(all(status.get(k) is False for k in ['transfer_established','discovery_established','cumulative_reuse_established']), 'Gate0 scientific claim boundary')
    require(time.monotonic()-start<=3600 and common.inventory(root)==before,'read-only bounded handoff audit')
    return dict(profile=PROFILE,**FLAGS,audit_pass=True,cells_verified=5,segments_verified=6,published_states_reconstructed=states,native_advances_executed=14720,audit_seconds=time.monotonic()-start,native_panel_elapsed_seconds=elapsed,results=results,resources=resources,decision=decision(results),derivations=derivations,source_sha256={p:hashlib.sha256(raw.encode()).hexdigest() for p,raw in sources.items()})

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('directory')
    parser.add_argument('--receipt',help='new audit directory outside the native input')
    args=parser.parse_args()
    if args.receipt is None:
        print(json.dumps(verify(args.directory),indent=2))
    else:
        root=Path(args.directory).resolve();destination=Path(args.receipt).resolve()
        require(destination!=root and root not in destination.parents,
                'audit destination outside immutable native input')
        destination.mkdir()
        manifest=dict(experiment_name=PROFILE+'-audit',state='Running',final_step=0,max_steps=0,
            start_time=datetime.datetime.now(datetime.timezone.utc).isoformat(),end_time=None)
        def put(name,value):
            with (destination/name).open('x') as f:
                json.dump(value,f,indent=2,sort_keys=True,allow_nan=False);f.write('\n')
        put('manifest.json',manifest)
        try:
            result=verify(root)
            result['native_seal_sha256']=hashlib.sha256((root/'output_sha256.json').read_bytes()).hexdigest()
            put('independent_replay.json',result)
            put('run_status.json',dict(profile=PROFILE+'-audit',audit_pass=True,
                engineering_execution_complete=True,gate0_feasibility_pass=result['decision']['gate0_feasibility_pass'],
                native_advances=0,transfer_established=False,discovery_established=False,
                cumulative_reuse_established=False,**FLAGS))
            manifest['state']='Completed'
            print(json.dumps(dict(audit_pass=True,decision=result['decision'],receipt=str(destination)),indent=2))
        except Exception as error:
            manifest['state']='Interrupted'
            put('failure.json',dict(error=repr(error),audit_pass=False,**FLAGS))
            raise
        finally:
            manifest['end_time']=datetime.datetime.now(datetime.timezone.utc).isoformat()
            (destination/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
            put('output_sha256.json',dict(profile=PROFILE+'-audit',files_sha256={
                p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(destination.iterdir())},
                development_only=True,scientific_run_launched=False))
