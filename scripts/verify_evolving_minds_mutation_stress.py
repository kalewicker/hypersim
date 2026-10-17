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
PROFILE='evolving-minds-ta1-mutation-stress-development-v1'
STREAM='evolving-minds-ta1-mutation-stress-streaming-v1'
FLAGS=dict(development_only=True,scientific_run_launched=False,gate_pass=False,training_admitted=False,evolution_enabled=False,reproductive_fairness_accepted=False,power_adequacy_established=False,ready_for_scientific_launch=False)
require=common.require
data=common.data
load=common.load
WORLDS=[17,23]

try:
    import gate0_reference as gate0
except ModuleNotFoundError:
    import verify_evolving_minds_transfer as gate0
POLICY='grammar-source-stress-v1'
PROBE='evolving-minds-ta1-mutation-stress-probe-v1'


def cases(probe=False):
    rows=[]
    for seed in [17,23]:
        for pattern in ['Alternating','Blocks4']:
            for arm,program,kernel in [('ancestor','Delayed','Faithful'),('faithful','Immediate','Faithful'),('source','Immediate','SourceKernel'),('grammar','Immediate','GrammarSourceKernel')]:
                rows.append(dict(name=f'world{seed}_{pattern}_{arm}',arm=arm,
                    config=dict(seed=seed,endowment=24,founders=[program]*2,kernels=[kernel]*2,
                        home_offsets=[0,4],rotation=0,reversed=False,horizon=64,scheduler='semantic-lottery-v1'),
                    environment=dict(pattern=pattern,cue_mode='Informative'),fault='Normal'))
    import copy
    recovery=copy.deepcopy(rows[3]);recovery.update(name='world17_Alternating_recovery',arm='recovery',fault='CleanStop')
    return [rows[3],recovery] if probe else rows+[recovery]


def registration(probe=False):
    counts=dict(worlds=1,patterns=1,primary_cells=1,initial_cells=2,segments=3,ticks=128,native_advances_executed=5888,published_states=19) if probe else dict(worlds=2,patterns=2,primary_cells=16,initial_cells=17,segments=18,ticks=1088,native_advances_executed=50048,published_states=154)
    return dict(profile=PROBE if probe else PROFILE,development_only=True,cases=cases(probe),counts=counts,
        mutation_probability='3/4',minimum_paid_attempts_per_functional_root=4,minimum_success_fraction='3/4',
        required_functional_paid_transmissions=3,minimum_active_offspring_gain=1,minimum_net_resource_gain=8,
        grammar_advance_requires_all_four_world_pattern_cells=True,original_source_is_comparator_not_required_gate=True,
        scientific_run_launched=False,discovery_established=False,transfer_established=False,
        limits=dict(panel_bytes=536870912,native_seconds=3600,audit_seconds=3600,summed_advance_seconds=2400))


def summarize(result):
    summary=gate0.endpoints(result,False)
    members=result['opportunity']['lifetimes'];byid={m['id']:m for m in members}
    qualifies=lambda m:m['completion_tick'] is not None and m['attempts']>=4 and 4*m['successes']>=3*m['attempts']
    families=[]
    for founder in [1,2]:
        levels={founder:0} if qualifies(byid[founder]) else {};paths={founder:[founder]}
        for m in sorted(members,key=lambda m:(m['born_tick'],m['id'])):
            if m['parent'] in levels and qualifies(m):
                levels[m['id']]=levels[m['parent']]+1;paths[m['id']]=paths[m['parent']]+[m['id']]
        best=min(levels,key=lambda i:(-levels[i],i)) if levels else None
        path=[] if best is None else paths[best]
        families.append(dict(founder=founder,max_functional_paid_transmissions=0 if best is None else levels[best],
            witness_path=path,persistence_pass=best is not None and levels[best]>=3,
            changed_genotype_edges=sum(byid[a]['program']!=byid[b]['program'] for a,b in zip(path,path[1:]))))
    records=result['records']
    summary.update(families=families,qualified_root_ids=[m['id'] for m in members if qualifies(m)],
        requests=len(records),syntax_rejections=sum(not r['syntax_valid'] for r in records),
        unreachable_paid_births=sum(r['child_id'] is not None and not r['material_reachable'] for r in records),
        development_failures=sum(m['development_failures'] for m in members),
        total_refunds=sum(r['refunded'] for r in records))
    return summary


def decision(results,probe=False):
    require(set(results)=={c['name'] for c in cases(probe)},'fixed stress result denominator')
    for key in ['opportunity','records','final_row']:
        require(results['world17_Alternating_recovery'][key]==results['world17_Alternating_grammar'][key],
                'mutation stress fresh-process recovery exact '+key)
    if probe:
        return dict(probe_packaging_and_recovery_pass=True,development_screen_measured=False,
            grammar_retention_screen_pass=False,faithful_positive_control_pass=False,
            scientific_acceptance=False,next_gate_automatically_authorized=False,**FLAGS)
    summaries={name:summarize(r) for name,r in results.items()};groups=[]
    passes={arm:[] for arm in ['faithful','source','grammar']};failed=[]
    for seed in [17,23]:
        for pattern in ['Alternating','Blocks4']:
            key=f'world{seed}_{pattern}_';ancestor=summaries[key+'ancestor'];arms={}
            for arm in ['faithful','source','grammar']:
                r=summaries[key+arm]
                gain=r['active_completed_offspring']-ancestor['active_completed_offspring']
                net=r['net_resource_gain']-ancestor['net_resource_gain']
                checks=dict(active_offspring_gain=gain>=1,net_resource_gain=net>=8,
                    **{f"family{f['founder']}_persistence":f['persistence_pass'] for f in r['families']})
                passed=all(checks.values());passes[arm].append(passed)
                if arm in ['faithful','grammar']:failed.extend(key+arm+':'+c for c,v in checks.items() if not v)
                arms[arm]=dict(summary=r,checks=checks,retention_cell_pass=passed,
                    active_offspring_advantage=gain,net_resource_advantage=net)
            groups.append(dict(world=seed,pattern=pattern,ancestor=ancestor,arms=arms))
    return dict(development_screen_measured=True,faithful_positive_control_pass=all(passes['faithful']),
        original_source_retention_screen_pass=all(passes['source']),grammar_retention_screen_pass=all(passes['grammar']),
        development_advance_screen_pass=all(passes['faithful']) and all(passes['grammar']),failed_checks=failed,groups=groups,
        scientific_acceptance=False,transfer_established=False,discovery_established=False,
        cumulative_reuse_established=False,next_gate_automatically_authorized=False,**FLAGS)


def verify(root):
    root=Path(root);start=time.monotonic();before=common.inventory(root)
    probe=data(root/'registration.json')['profile']==PROBE
    profile=PROBE if probe else PROFILE
    expected=registration(probe);fixed_cases=cases(probe)
    hashes=dict(before[0]); hashes.pop('output_sha256.json',None)
    seal=data(root/'output_sha256.json')
    require(seal==dict(profile=profile,files_sha256=hashes,development_only=True,
                      scientific_run_launched=False),'complete sealed Gate0 payload')
    require(Path(__file__).read_bytes()==(root/'checker.py').read_bytes(),'executed frozen Gate0 checker')
    require(data(root/'registered_cases.json')==fixed_cases,'fixed native handoff cases')
    require({p.name for p in root.iterdir() if p.is_dir()}=={x['name'] for x in fixed_cases}|{'world17_recovery_resumed'},'fixed six-segment denominator')
    sources=data(root/'sources.json')
    require(data(root/'registration.json')==expected,'fixed complete Gate0 registration')
    require(len((root/'binary_sha256.txt').read_text())==64 and all(c in '0123456789abcdef' for c in (root/'binary_sha256.txt').read_text()),'binary identity receipt')
    bindings=[('checker.py','scripts/verify_evolving_minds_mutation_stress.py'),('stream_reference.py','scripts/verify_evolving_minds_streaming.py'),('admission_reference.py','scripts/verify_evolving_minds_admission.py'),('funding_reference.py','scripts/verify_evolving_minds_funding.py'),('opportunity_reference.py','scripts/evolving_minds_opportunity.py'),('shared_replay.py','scripts/evolving_minds_shared_replay.py'),('replay.py','scripts/verify_evolving_minds.py'),('d0_reference.py','scripts/verify_evolving_minds_d0.py'),('schedule_reference.py','scripts/verify_evolving_minds_choice.py'),('contract.md','experiments/evolving_minds/contracts/ta1_mutation_stress_v1.md'),('base_rules.toml','rules/evolving_minds/admission_v1.toml')]
    bindings.append(('gate0_reference.py','scripts/verify_evolving_minds_transfer.py'))
    for name,path in bindings:require((root/name).read_text()==sources[path],'handoff frozen source binding')
    v=load(root/'stream_reference.py','handoff_stream');a=load(root/'admission_reference.py','handoff_admission');c=load(root/'replay.py','handoff_replay');d=load(root/'d0_reference.py','handoff_d0');s=load(root/'schedule_reference.py','handoff_schedule');f=load(root/'funding_reference.py','handoff_funding');o=load(root/'opportunity_reference.py','handoff_observer');shared=load(root/'shared_replay.py','handoff_shared')
    modules=(a,c,d,s);base=(root/'base_rules.toml').read_bytes();laws=(root/'b24_base_rules.toml').read_bytes()
    require(hashlib.sha256(base).hexdigest()==a.LAW_SHA and tomllib.loads(laws.decode())==f.funded_laws(base,24),'independent handoff physics')
    results={};resources={};reports=[];states=0;derivations=[]
    certificates={}
    for cell in fixed_cases:
        name=cell['name'];p=root/name;cfg=cell['config'];environment=cell['environment'];resumed=cell['fault']=='CleanStop'
        chain=v.inspect_segment(root/'world17_recovery_resumed' if resumed else p,funded_config=cfg,transfer_environment=environment,mutation_stress_policy=POLICY)
        require(len(chain)==(2 if resumed else 1),'declared handoff recovery chain')
        for seg in chain:
            directory=seg['directory'];m=data(directory/'manifest.json');r=data(directory/'segment_report.json');status=data(directory/'run_status.json');v.boundary(status,STREAM)
            end=24 if directory==p and resumed else 64;begin=24 if directory.name.endswith('_resumed') else 0
            require(m['experiment_name']==STREAM and m['seed']==cfg['seed'] and m['thread_count']==4 and m['max_steps']==2944 and m['final_step']==end*46 and m['state']==r['state']==('Interrupted' if end==24 else 'Completed') and r['funded_config']==cfg and r.get('handoff_config') is None and r['transfer_environment']==environment and r['mutation_stress_policy']==POLICY and r.get('retained_policy') is None and r['start_tick']==begin and r['native_final_tick']==r['committed_final_tick']==end and r['fault']==('CleanStop' if end==24 else 'Normal') and status['engineering_execution_complete']==(end==64),'actual handoff lifecycle/configuration')
            resource=r['resources'];actual=sum(q.stat().st_size for q in directory.iterdir())
            require(resource['drained_history_empty'] and resource['advance_seconds']<=120 and resource['payload_bytes']==(directory/'ticks.jsonl').stat().st_size<=16*1024**2 and actual<=32*1024**2 and resource['checkpoints']==[[x['marker']['cursor']['tick'],x['checkpoint_bytes']] for x in seg['commits']] and max(x['checkpoint_bytes'] for x in seg['commits'])<=2*1024**2,'handoff segment resource caps')
            resources[directory.name]=dict(observed_total_bytes=actual,**resource)
        if resumed:reports.append(data(root/'world17_recovery_resumed/segment_report.json'))
        reports.append(data(p/'segment_report.json'))
        then=time.monotonic()
        final,physics,opportunity,count=shared.optimized_cell(chain,modules,laws,cfg,None,v,o,start,derivations,transfer_environment=environment,mutation_stress_policy=POLICY)
        states+=count
        results[name]=dict(opportunity=opportunity,final_row=physics['final_row'],records=final['admission']['records'],audit_seconds=time.monotonic()-then)
        records=results[name]['records']
        certificates[name]=[a.grammar_certificate(r,d) for r in records if r['operator']=='grammar_source']
        if cell['config']['kernels']==['GrammarSourceKernel']*2:
            require(all(r['syntax_valid'] for r in records),'grammar kernel only syntactically valid proposals')
    require(data(root/'grammar_certificates.json')==certificates,'complete grammar certificates including identity and failure routes')
    require(certificates['world17_Alternating_recovery']==certificates['world17_Alternating_grammar'],'exact recovery grammar certificates')
    if not probe:
        for seed in [17,23]:
            for pattern in ['Alternating','Blocks4']:
                for filename in ['schedule.json','rules.toml']:
                    reference=(root/f'world{seed}_{pattern}_faithful'/filename).read_bytes()
                    require(all((root/f'world{seed}_{pattern}_{arm}'/filename).read_bytes()==reference for arm in ['ancestor','source','grammar']),'matched stress schedules and physiology')
    require(data(root/'summary.json')==reports,'handoff report denominator')
    require(data(root/'execution_counts.json')==expected['counts'],'handoff native execution denominator')
    m=data(root/'manifest.json');elapsed=(datetime.datetime.fromisoformat(m['end_time'])-datetime.datetime.fromisoformat(m['start_time'])).total_seconds()
    require(m['state']=='Completed' and m['experiment_name']==profile and m['seed']==17 and m['thread_count']==4 and m['final_step']==m['max_steps']==expected['counts']['native_advances_executed'] and elapsed<=3600 and sum(r['advance_seconds'] for r in resources.values())<=2400,'handoff root lifecycle/resources')
    status=data(root/'run_status.json');v.boundary(status,profile);require(status['engineering_execution_complete'] and states==expected['counts']['published_states'],'handoff complete published state denominator')
    require(all(status.get(k) is False for k in ['transfer_established','discovery_established','cumulative_reuse_established']), 'Gate0 scientific claim boundary')
    require(time.monotonic()-start<=3600 and common.inventory(root)==before,'read-only bounded handoff audit')
    return dict(profile=profile,**FLAGS,audit_pass=True,cells_verified=expected['counts']['initial_cells'],segments_verified=expected['counts']['segments'],published_states_reconstructed=states,native_advances_executed=expected['counts']['native_advances_executed'],audit_seconds=time.monotonic()-start,native_panel_elapsed_seconds=elapsed,results=results,resources=resources,decision=decision(results,probe),derivations=derivations,source_sha256={p:hashlib.sha256(raw.encode()).hexdigest() for p,raw in sources.items()})

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('directory')
    parser.add_argument('--receipt',help='new audit directory outside the native input')
    args=parser.parse_args()
    if args.receipt is None:
        print(json.dumps(verify(args.directory),indent=2))
    else:
        root=Path(args.directory).resolve();destination=Path(args.receipt).resolve()
        audit_profile=data(root/'registration.json')['profile']+'-audit'
        require(destination!=root and root not in destination.parents,
                'audit destination outside immutable native input')
        destination.mkdir()
        manifest=dict(experiment_name=audit_profile,state='Running',final_step=0,max_steps=0,
            start_time=datetime.datetime.now(datetime.timezone.utc).isoformat(),end_time=None)
        def put(name,value):
            with (destination/name).open('x') as f:
                json.dump(value,f,indent=2,sort_keys=True,allow_nan=False);f.write('\n')
        put('manifest.json',manifest)
        try:
            result=verify(root)
            result['native_seal_sha256']=hashlib.sha256((root/'output_sha256.json').read_bytes()).hexdigest()
            put('independent_replay.json',result)
            put('run_status.json',dict(profile=audit_profile,audit_pass=True,
                engineering_execution_complete=True,grammar_retention_screen_pass=result['decision']['grammar_retention_screen_pass'],
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
            put('output_sha256.json',dict(profile=audit_profile,files_sha256={
                p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(destination.iterdir())},
                development_only=True,scientific_run_launched=False))
