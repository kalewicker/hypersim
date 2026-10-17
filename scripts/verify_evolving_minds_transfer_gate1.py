#!/usr/bin/env python3
"""Independently reconstruct the TA-1 Gate1 authored inheritance assay."""
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
PROFILE='evolving-minds-ta1-gate1-development-v2'
STREAM='evolving-minds-ta1-temporal-streaming-v1'
FLAGS=dict(development_only=True,scientific_run_launched=False,gate_pass=False,training_admitted=False,evolution_enabled=False,reproductive_fairness_accepted=False,power_adequacy_established=False,ready_for_scientific_launch=False)
require=common.require
data=common.data
load=common.load
WORLDS=[17]

import copy
try:
    import d0_reference as d0
    import gate0_reference as gate0
except ModuleNotFoundError:
    import verify_evolving_minds_d0 as d0
    import verify_evolving_minds_transfer as gate0


def intervention(arm):
    sources={'ancestor':[2],'useful':[0],'reversion':[0,2],'sham':[0,0],
             'restoration':[0,2,0],'recovery':[0]}[arm]
    p=d0.program('delayed_cue');records=[]
    for source in sources:
        next(g for g in p['instructions'] if g['id']==103)['source']=source
        records.append(dict(source=source,program=copy.deepcopy(p),bytes=list(d0.canonical(p))))
    return ('Delayed' if sources[-1]==2 else 'Immediate'),records


def case(seed,pattern,mode,arm):
    program,records=intervention(arm)
    return dict(name=f'world{seed}_{pattern}_{mode}_{arm}',arm=arm,
        config=dict(seed=seed,endowment=24,founders=[program]*2,kernels=['Faithful']*2,
            home_offsets=[0,4],rotation=0,reversed=False,horizon=64,scheduler='semantic-lottery-v1'),
        environment=dict(pattern=pattern,cue_mode=mode),fault='CleanStop' if arm=='recovery' else 'Normal',
        interventions=records)


def cases():
    return [case(17,p,'Informative',arm) for p in ['Alternating','Blocks4']
            for arm in ['ancestor','useful','reversion','sham','restoration']]+[
            case(17,'Alternating','Withheld',arm) for arm in ['ancestor','useful']]+[
            case(17,'Alternating','Informative','recovery')]


def registration():
    return dict(profile=PROFILE,development_only=True,cases=cases(),
        counts=dict(main_worlds=1,primary_cells=10,diagnostic_cells=2,initial_cells=13,
                    segments=14,ticks=832,native_advances_executed=38272,published_states=118),
        minimum_active_offspring_gain=1,minimum_net_resource_gain=8,
        required_active_faithful_transmissions=3,transfer_established=False,scientific_run_launched=False,
        limits=dict(panel_bytes=536870912,native_seconds=3600,audit_seconds=3600,summed_advance_seconds=2400))


def evaluation_registration():
    blocks=[]
    for i in range(8):
        label=f'hypersim-ta1-temporal-evaluation-v1-{i:03}'
        seed=int.from_bytes(hashlib.sha256(label.encode()).digest()[:8],'little')
        cells=[case(seed,p,'Informative',arm) for p in ['Alternating','Blocks4']
               for arm in ['ancestor','useful','reversion','sham','restoration']]
        cells+=[case(seed,'Alternating','Withheld',arm) for arm in ['ancestor','useful']]
        blocks.append(dict(block=label,seed=seed,exposure='registered_unexposed',cases=cells))
    return dict(profile='evolving-minds-ta1-gate1-evaluation-proposal-v1',blocks=blocks,cells=96,
        native_advances=282624,published_states=864,launch_enabled=False,scientific_run_launched=False,
        transfer_established=False,budget_approved=False,power_adequacy_established=False,
        acceptance_proposal='all eight blocks in each primary temporal family meet development effect and persistence floors; exact sham/restoration and reversion controls; diagnostics excluded',
        scientific_budget=None,execution_command_available=False)


def endpoints(result):
    summary=gate0.endpoints(result,False)
    members=result['opportunity']['lifetimes'];byid={m['id']:m for m in members}
    active=lambda m:m['completion_tick'] is not None and m['successes']>=1
    families=[]
    for founder in [1,2]:
        levels={founder:0} if active(byid[founder]) else {};paths={founder:[founder]}
        for m in sorted(members,key=lambda m:(m['born_tick'],m['id'])):
            if m['parent'] in levels and active(m) and m['program']==byid[m['parent']]['program']:
                levels[m['id']]=levels[m['parent']]+1;paths[m['id']]=paths[m['parent']]+[m['id']]
        best=min(levels,key=lambda i:(-levels[i],i)) if levels else None
        families.append(dict(founder=founder,max_active_faithful_transmissions=0 if best is None else levels[best],
            witness_path=[] if best is None else paths[best],persistence_pass=best is not None and levels[best]>=3))
    summary['families']=families
    return summary


def decision(results):
    require(set(results)=={c['name'] for c in cases()},'fixed Gate1 result denominator')
    summaries={name:endpoints(r) for name,r in results.items()}
    families=[];failed=[]
    for pattern in ['Alternating','Blocks4']:
        names={arm:case(17,pattern,'Informative',arm)['name'] for arm in ['ancestor','useful','reversion','sham','restoration']}
        for candidate,reference in [('sham','useful'),('restoration','useful'),('reversion','ancestor')]:
            for key in ['opportunity','records','final_row']:
                require(results[names[candidate]][key]==results[names[reference]][key],'causal control exact '+pattern+':'+candidate+':'+key)
        useful=summaries[names['useful']];ancestor=summaries[names['ancestor']]
        offspring=useful['active_completed_offspring']-ancestor['active_completed_offspring']
        resource=useful['net_resource_gain']-ancestor['net_resource_gain']
        checks=dict(active_offspring_gain=offspring>=1,net_resource_gain=resource>=8,
            **{f"family{f['founder']}_persistence":f['persistence_pass'] for f in useful['families']})
        failed.extend(pattern+':'+k for k,v in checks.items() if not v)
        families.append(dict(pattern=pattern,useful=useful,ancestor=ancestor,checks=checks,
            useful_minus_ancestor_active_offspring=offspring,useful_minus_ancestor_net_resource_gain=resource))
    for key in ['opportunity','records','final_row']:
        require(results[case(17,'Alternating','Informative','recovery')['name']][key]==
                results[case(17,'Alternating','Informative','useful')['name']][key],'Gate1 recovery exact '+key)
    return dict(gate1_development_feasibility_pass=not failed,failed_checks=failed,primary_families=families,
        diagnostics={arm:summaries[case(17,'Alternating','Withheld',arm)['name']] for arm in ['ancestor','useful']},
        transfer_established=False,discovery_established=False,cumulative_reuse_established=False,
        next_gate_automatically_authorized=False,**FLAGS)


def verify(root):
    root=Path(root);start=time.monotonic();before=common.inventory(root)
    prior=data(root/'prior_attempt.json');prior_root=Path(prior['directory'])
    original=common.inventory(prior_root)
    require(original[0]==prior['files_sha256'] and prior['native_advances']==35696 and
            prior['combined_advances_with_v2']==73968 and prior['independent_replicates_added']==0,
            'preserved v1 engineering failure inventory')
    require(data(prior_root/'manifest.json')['state']=='Interrupted' and
            data(prior_root/'manifest.json')['final_step']==35328,'original interrupted v1 manifest')
    hashes=dict(before[0]); hashes.pop('output_sha256.json',None)
    seal=data(root/'output_sha256.json')
    require(seal==dict(profile=PROFILE,files_sha256=hashes,development_only=True,
                      scientific_run_launched=False),'complete sealed Gate1 payload')
    require(Path(__file__).read_bytes()==(root/'checker.py').read_bytes(),'executed frozen Gate1 checker')
    require(data(root/'registered_cases.json')==cases(),'fixed native handoff cases')
    require({p.name for p in root.iterdir() if p.is_dir()}=={x['name'] for x in cases()}|{'world17_recovery_resumed'},'fixed six-segment denominator')
    sources=data(root/'sources.json')
    require(data(root/'registration.json')==registration(),'fixed complete Gate1 registration')
    require(data(root/'evaluation_registration.json')==evaluation_registration(),'unexposed evaluation registration')
    require(len((root/'binary_sha256.txt').read_text())==64 and all(c in '0123456789abcdef' for c in (root/'binary_sha256.txt').read_text()),'binary identity receipt')
    bindings=[('checker.py','scripts/verify_evolving_minds_transfer_gate1.py'),('stream_reference.py','scripts/verify_evolving_minds_streaming.py'),('admission_reference.py','scripts/verify_evolving_minds_admission.py'),('funding_reference.py','scripts/verify_evolving_minds_funding.py'),('opportunity_reference.py','scripts/evolving_minds_opportunity.py'),('shared_replay.py','scripts/evolving_minds_shared_replay.py'),('replay.py','scripts/verify_evolving_minds.py'),('d0_reference.py','scripts/verify_evolving_minds_d0.py'),('schedule_reference.py','scripts/verify_evolving_minds_choice.py'),('contract.md','experiments/evolving_minds/contracts/ta1_gate1_development_v2.md'),('base_rules.toml','rules/evolving_minds/admission_v1.toml')]
    bindings.append(('gate0_reference.py','scripts/verify_evolving_minds_transfer.py'))
    for name,path in bindings:require((root/name).read_text()==sources[path],'handoff frozen source binding')
    v=load(root/'stream_reference.py','handoff_stream');a=load(root/'admission_reference.py','handoff_admission');c=load(root/'replay.py','handoff_replay');d=load(root/'d0_reference.py','handoff_d0');s=load(root/'schedule_reference.py','handoff_schedule');f=load(root/'funding_reference.py','handoff_funding');o=load(root/'opportunity_reference.py','handoff_observer');shared=load(root/'shared_replay.py','handoff_shared')
    modules=(a,c,d,s);base=(root/'base_rules.toml').read_bytes();laws=(root/'b24_base_rules.toml').read_bytes()
    require(hashlib.sha256(base).hexdigest()==a.LAW_SHA and tomllib.loads(laws.decode())==f.funded_laws(base,24),'independent handoff physics')
    results={};resources={};reports=[];states=0;derivations=[]
    for _registered_cell in cases():
        name=_registered_cell['name'];p=root/name;cfg=_registered_cell['config'];environment=_registered_cell['environment'];resumed=_registered_cell['fault']=='CleanStop'
        chain=v.inspect_segment(root/'world17_recovery_resumed' if resumed else p,funded_config=cfg,transfer_environment=environment)
        require(len(chain)==(2 if resumed else 1),'declared handoff recovery chain')
        for seg in chain:
            directory=seg['directory'];m=data(directory/'manifest.json');r=data(directory/'segment_report.json');status=data(directory/'run_status.json');v.boundary(status,STREAM)
            end=24 if directory==p and resumed else 64;begin=24 if directory.name.endswith('_resumed') else 0
            require(m['experiment_name']==STREAM and m['seed']==cfg['seed'] and m['thread_count']==4 and m['max_steps']==2944 and m['final_step']==end*46 and m['state']==r['state']==('Interrupted' if end==24 else 'Completed') and r['funded_config']==cfg and r.get('handoff_config') is None and r['transfer_environment']==environment and r.get('retained_policy') is None and r['start_tick']==begin and r['native_final_tick']==r['committed_final_tick']==end and r['fault']==('CleanStop' if end==24 else 'Normal') and status['engineering_execution_complete']==(end==64),'actual handoff lifecycle/configuration')
            resource=r['resources'];actual=sum(q.stat().st_size for q in directory.iterdir())
            require(resource['drained_history_empty'] and resource['advance_seconds']<=120 and resource['payload_bytes']==(directory/'ticks.jsonl').stat().st_size<=16*1024**2 and actual<=32*1024**2 and resource['checkpoints']==[[x['marker']['cursor']['tick'],x['checkpoint_bytes']] for x in seg['commits']] and max(x['checkpoint_bytes'] for x in seg['commits'])<=2*1024**2,'handoff segment resource caps')
            resources[directory.name]=dict(observed_total_bytes=actual,**resource)
        if resumed:reports.append(data(root/'world17_recovery_resumed/segment_report.json'))
        reports.append(data(p/'segment_report.json'))
        then=time.monotonic()
        final,physics,opportunity,count=shared.optimized_cell(chain,modules,laws,cfg,None,v,o,start,derivations,transfer_environment=environment)
        states+=count
        results[name]=dict(opportunity=opportunity,final_row=physics['final_row'],records=final['admission']['records'],audit_seconds=time.monotonic()-then)
        records=results[name]['records']
        require(all(q['parent_program']==q['proposed_program'] for q in records),'frozen genotype faithful copying')
    for pattern in ['Alternating','Blocks4']:
        for filename in ['schedule.json','rules.toml']:
            expected=(root/case(17,pattern,'Informative','ancestor')['name']/filename).read_bytes()
            require(all((root/case(17,pattern,'Informative',arm)['name']/filename).read_bytes()==expected
                        for arm in ['useful','reversion','sham','restoration']),'matched temporal native laws')
    require(data(root/'summary.json')==reports,'handoff report denominator')
    for cse in cases():
        old=prior_root/cse['name'];new=root/cse['name']
        for filename in ['schedule.json','rules.toml','initial.json','initial_lineage.json']:
            require((old/filename).read_bytes()==(new/filename).read_bytes(),'unchanged v1/v2 physics and initialization')
        previous=(old/'ticks.jsonl').read_bytes();current=(new/'ticks.jsonl').read_bytes()
        if cse['arm']=='recovery':
            require(len(previous.splitlines())==8 and current.startswith(previous),'preserved v1 recovery prefix')
        else:
            require(previous==current,'unchanged twelve complete v1/v2 semantic histories')
    require(data(root/'execution_counts.json')==registration()['counts'],'handoff native execution denominator')
    m=data(root/'manifest.json');elapsed=(datetime.datetime.fromisoformat(m['end_time'])-datetime.datetime.fromisoformat(m['start_time'])).total_seconds()
    require(m['state']=='Completed' and m['experiment_name']==PROFILE and m['seed']==17 and m['thread_count']==4 and m['final_step']==m['max_steps']==38272 and elapsed<=3600 and sum(r['advance_seconds'] for r in resources.values())<=2400,'handoff root lifecycle/resources')
    status=data(root/'run_status.json');v.boundary(status,PROFILE);require(status['engineering_execution_complete'] and states==118,'handoff complete published state denominator')
    require(all(status.get(k) is False for k in ['transfer_established','discovery_established','cumulative_reuse_established']), 'Gate1 scientific claim boundary')
    require(time.monotonic()-start<=3600 and common.inventory(root)==before and common.inventory(prior_root)==original,'read-only bounded handoff audit')
    return dict(profile=PROFILE,**FLAGS,audit_pass=True,cells_verified=13,segments_verified=14,published_states_reconstructed=states,native_advances_executed=38272,audit_seconds=time.monotonic()-start,native_panel_elapsed_seconds=elapsed,results=results,resources=resources,decision=decision(results),derivations=derivations,source_sha256={p:hashlib.sha256(raw.encode()).hexdigest() for p,raw in sources.items()})

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
                engineering_execution_complete=True,gate1_development_feasibility_pass=result['decision']['gate1_development_feasibility_pass'],
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
