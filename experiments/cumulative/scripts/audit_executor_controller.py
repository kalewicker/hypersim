"""Independent correspondence of controller funding, compact native evidence and measurement."""
import json
from pathlib import Path
import sys
from compact_evidence import Store,sha,encode_json,require,source_inventory
from audit_campaign_controller import audit_ledger
from audit_spool import ledger,Journal
from audit_scientific_executor import recover
from audit_campaign_measurement import check,independent_report


def audit(out):
    load=lambda p:json.loads((out/p).read_text())
    cr=(out/'contract.json').read_bytes();c=json.loads(cr);binary=sha((out/'cumulative-controlled-executor').read_bytes())
    pins=load('source_sha256.json');require(all(sha((out/'source_snapshots'/n).read_bytes())==h for n,h in pins.items()),'source closure')
    require(cr==(out/'source_snapshots/experiments/cumulative/executor_controller_contract.json').read_bytes(),'compiled route contract')
    source_inventory(out/'executor_parent',c['parent']['seal_sha256'])
    source_inventory(out/'measurement_parent',c['measurement_parent']['seal_sha256'])
    for i in range(188):
        refusal=load(f'commands/scientific_refusal_{i:03}.json')
        require(refusal['exit_code']==1 and json.loads(refusal['stderr'])['native_rewrites']==0 and 'scientific launch disabled' in json.loads(refusal['stderr'])['error'],'scientific execution refused before work')
    require(not (out/'forbidden_journal').exists(),'no scientific journal initialized')
    campaign=load('campaign_contract.json');require(campaign==load('measurement_parent/campaign_contract.json'),'unchanged measurement law')
    results=[];cases=[];work=0;durations=0;peak=0;native_charge=0
    for branch in ['complete','interrupted']:
        root=out/branch;summary=audit_ledger(root/'controller');owner=json.loads((root/'controller/contract.json').read_text())
        native_charge+=summary['accounted_native_ms']
        require(owner['purpose']=='saved_development' and owner['limits']==dict(rewrites=1200 if branch=='complete' else 250,native_ms=180000,workflow_ms=1200000,storage_bytes=268435456),'fixed branch funding')
        events=[json.loads(p.read_text())['event'] for p in sorted((root/'controller/events').iterdir())]
        intents=[e['value'] for e in events if e['kind']=='intent'];receipts=[e['value'] for e in events if e['kind']=='receipt']
        spool=root/'spool';states,_=ledger(spool)
        require(len(states)==len(intents)==(5 if branch=='complete' else 1),'fixed dispatch denominator')
        wanted_routes=c['routes'][:5] if branch=='complete' else [c['routes'][-1]]
        expected_worlds=[]
        for route in (c['routes'][:4] if branch=='complete' else [c['routes'][-1]]+c['routes'][1:4]):
            frozen=dict(profile='cumulative-controlled-executor-request-v1',purpose='development',contract_sha256=sha(cr),binary_sha256=binary,route=route)
            expected_worlds.append(dict(world_id='development/'+route['name'],request_sha256=sha(encode_json(frozen)),work_bound=route['execution_allowance']))
        require(owner['worlds']==expected_worlds,'all fixed worlds registered before work')
        for i,route in enumerate(wanted_routes):
            unit=f'u{i:06}';s=states[unit];frozen=json.loads((spool/'requests'/(unit+'.json')).read_text())
            wanted=dict(profile='cumulative-controlled-executor-request-v1',purpose='development',contract_sha256=sha(cr),binary_sha256=binary,route=route)
            require(frozen==wanted and s['request_sha256']==sha(encode_json(wanted)) and s['kind']=='released','exact native identity and lossless archive')
            store=Store(spool/'compact'/unit,s['index_sha256'],s['seal_sha256']);store.verify_all()
            recovered=recover(Journal(store),route,cr,binary)
            native=json.loads((spool/'requests'/(unit+'.receipt.json')).read_text())
            require(not native['timed_out'] and native['returncode']==(0 if branch=='complete' else 1),'one bounded native call')
            require(not recovered['staging'],'closed compact journal')
            if branch=='interrupted':
                require(recovered['status']=='unknown' and recovered['lower']==0 and recovered['upper']==12,'pending native reservation retained')
                continue
            r=load('complete/routes.json')[i];v=receipts[i]
            logical=sha(encode_json(wanted)) if i<4 else sha(encode_json(dict(registry_sha256=sha((out/'registry.json').read_bytes()),start=0,end=9)))
            require(intents[i]['unit']['request_sha256']==r['logical_request_sha256']==v['request_sha256']==logical,'controller/native identity binding')
            require(r['native_request_sha256']==s['request_sha256'] and r['unit']==unit,'exact route receipt')
            require(r['spool_admission']==s['admission'] and r['outside_spool_bytes']>=0 and r['total_storage_charge_bytes']==r['spool_admission']['charged_bytes']+r['outside_spool_bytes']==intents[i]['storage_charge_bytes']<=268435456,'whole workspace pre-dispatch charge')
            require(recovered['status']=='complete' and v['work_lower']==v['work_upper']==recovered['lower']==recovered['upper'] and v['evidence_sha256']==recovered['head'],'independent evidence before exact refund')
            require(v['native_ms']==r['native_archive_audit_ms']<=intents[i]['native_ms_bound']<=30000 and v['elapsed_ms']<1200000,'bounded actual dispatch time')
            work+=recovered['lower'];durations+=v['native_ms'];peak=max(peak,r['total_storage_charge_bytes'])
            if i<4:
                oldcr=(out/'executor_parent/contract.json').read_bytes();oldbinary=sha((out/'executor_parent/cumulative-scientific-executor').read_bytes())
                old=recover(out/'executor_parent/journals'/route['name'],json.loads(oldcr)['routes'][i],oldcr,oldbinary)
                require(recovered['results']==old['results'] and recovered['checkpoint']==old['checkpoint'],'exact original world/checkpoint equivalence')
                cases.append(dict(name=route['name'],initial=recovered['results'][0]['row']['before'],rows=[v['row'] for v in recovered['results']],final=recovered['checkpoint']['state']))
            else:results.extend(recovered['results'])
        if branch=='interrupted':
            require(len(receipts)==0 and summary['accounted_rewrites_upper']==250 and summary['accounted_rewrites_lower']==0 and summary['world_statuses']==['unknown','not_started','not_started','not_started'],'no refund, retry or dropped worlds')
            require(events[-1]==dict(kind='close',value=dict(reason='operational_stop',decision_sha256=None)),'unknown campaign is not a negative scientific result')
        else:
            require(summary['closed'] and summary['assays_completed']==9 and summary['accounted_rewrites_upper']==summary['accounted_rewrites_lower']==335,'complete fixed controller work')
            freeze=[e['value'] for e in events if e['kind']=='registry']
            require(freeze==[dict(sha256=sha((out/'registry.json').read_bytes()),measurement_sha256=sha((out/'measurement.json').read_bytes()),count=9,slots=40)],'one complete registry before assays')
            require(events[-1]==dict(kind='close',value=dict(reason='complete',decision_sha256=sha((out/'report.json').read_bytes()))),'content bound closure')
    measurement=load('measurement.json');registry=load('registry.json');check(cases,campaign,measurement,registry)
    require(measurement==load('measurement_parent/measurement.json') and registry==load('measurement_parent/registry.json'),'unchanged census and registry')
    report=independent_report(measurement,registry,results,cases,campaign)
    require(report==load('report.json')==load('measurement_parent/report.json'),'independent report and exact historical equivalence')
    require(work==335 and work+250<=c['limits']['native_rewrites'],'aggregate completed work plus full uncertain reservation')
    require(native_charge<=180000,'aggregate measured and uncertain native time charge')
    return dict(engineering_controller_bridge_pass=True,native_work_lower=work,native_work_upper=work+250,development_worlds=4,sampling_slots=40,assay_requests=9,retained_pairs=0,native_invocations=6,measured_native_archive_audit_ms=durations,maximum_workspace_admission_bytes=peak,interrupted_controller_upper=250,interrupted_native_upper=12,scientific_worlds_started=0,readiness_gate_complete=False,scientific_acceptance=False,evolution_demonstrated=False,campaign_launch_authorized=False)


if __name__=='__main__':print(json.dumps(audit(Path(sys.argv[1])),indent=2,sort_keys=True))
