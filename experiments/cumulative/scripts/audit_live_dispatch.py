"""Independent audit of live development controller/spool/native correspondence."""
import json
from pathlib import Path
import sys
from compact_evidence import Store, sha, source_inventory, require, encode_json
from audit_campaign_controller import audit_ledger
from audit_spool import ledger, Journal
from audit_durability import read
from audit_campaign_adapter import recover_values, recover
from audit_campaign_measurement import check, independent_report


def audit(package):
    load=lambda n:json.loads((package/n).read_text())
    c=load('contract.json');parent=package/'measurement_parent';source_inventory(parent,c['measurement_parent']['seal_sha256'])
    info=json.loads((parent/'adapter_store.json').read_text());adapter=Store(parent/'adapter_evidence',info['index_sha256'],c['adapter_parent']['seal_sha256']);adapter.verify_all()
    ac=json.loads((adapter.root/'contract.json').read_text());campaign=load('campaign_contract.json')
    require(campaign==json.loads((parent/'campaign_contract.json').read_text()), 'unchanged prospective law')
    controller=package/'controller';summary=audit_ledger(controller);owner=json.loads((controller/'contract.json').read_text())
    require(owner['purpose']=='saved_development' and owner['limits']==dict(rewrites=1200,native_ms=180000,workflow_ms=1200000,storage_bytes=268435456),'fixed live development funding')
    events=[json.loads(p.read_text())['event'] for p in sorted((controller/'events').iterdir())]
    intents=[e['value'] for e in events if e['kind']=='intent'];receipts=[e['value'] for e in events if e['kind']=='receipt']
    spool=package/'native_spool';states,admissions=ledger(spool);routes=load('native_routes.json')
    require(len(states)==len(routes)==len(intents)==len(receipts)==5,'four world units and one complete assay batch')
    require(summary['closed'] and summary['worlds']==4 and summary['assays_completed']==9,'complete development controller denominator')
    registry=load('registry.json');cases=[];results=[];work=0;expected_owner=[]
    for index,route in enumerate(routes):
        unit=route['unit'];state=states[unit];raw=(spool/'requests'/(unit+'.json')).read_bytes();request=json.loads(raw)
        require(unit==f'u{index:06}' and state['kind']=='released' and not (spool/'raw'/unit).exists(),'fresh ordered native units compacted')
        require(sha(raw)==state['request_sha256']==route['native_request_sha256'],'spool/native request identity')
        require(request['source_identity']==dict(binary_sha256=sha((package/'cumulative-campaign').read_bytes()),source_sha256=sha((package/'source_sha256.json').read_bytes())),'binary and source identity')
        require(request['seed']==2026101001 and request['purpose']=='development' and request['source'] is None and request['fault'] is None and request['native_budget']==1200 and request['journal_byte_allowance']==32505856,'fixed development native route')
        if index<4:
            arm=c['world_order'][index];refraw=(adapter.root/'inputs'/('world_'+arm+'.json')).read_bytes();reference=json.loads(refraw)
            require(request['unit']=='world' and request['spec']==reference['spec'] and request['registry'] is None and request['range'] is None and request['execution_allowance']==c['world_reservations'][arm],'unchanged matched world')
            expected_owner.append(dict(world_id='development/'+arm,request_sha256=sha(raw),work_bound=c['world_reservations'][arm]))
            logical=sha(raw);items=[]
        else:
            require(request['unit']=='assays' and request['spec'] is None and request['range']==dict(start=0,end=9) and request['execution_allowance']==180,'full fixed global batch')
            require(request['registry']['sha256']==sha((package/'registry.json').read_bytes()),'frozen live registry bytes')
            logical=sha(encode_json(dict(registry_sha256=request['registry']['sha256'],start=0,end=9)));items=registry['assays']
        require(intents[index]['unit']['request_sha256']==logical==route['logical_request_sha256'],'controller intent matches native unit')
        require(intents[index]['storage_charge_bytes']==route['admission']['charged_bytes']==state['admission']['charged_bytes'],'actual pre-dispatch spool charge')
        grant=intents[index]['native_ms_bound'];duration=route['native_and_archive_ms']
        require(0<grant<=30000 and 0<=duration<=grant and receipts[index]['native_ms']==duration,'measured native/archive timing within actual grant')
        require(receipts[index]['elapsed_ms']>=intents[index]['elapsed_ms'] and receipts[index]['elapsed_ms']<1200000,'actual monotonic workflow bound')
        compact=Store(spool/'compact'/unit,state['index_sha256'],state['seal_sha256']);compact.verify_all();values,head,staging=read(Journal(compact))
        recovered=recover_values(values,head,staging,request,sha(raw),ac,items)
        require(recovered['status']=='complete' and not recovered['unknown'] and not staging,'complete independently audited native unit')
        exit_receipt=json.loads((spool/'requests'/(unit+'.receipt.json')).read_text())
        require(exit_receipt['returncode']==0 and not exit_receipt['timed_out'],'successful single native invocation')
        require(receipts[index]['evidence_sha256']==head and receipts[index]['request_sha256']==logical and receipts[index]['work_lower']==receipts[index]['work_upper']==recovered['here_upper'],'audited native evidence before accepting exact receipt')
        work+=recovered['here_upper']
        if index<4:
            old=recover(adapter.root/'journals'/('world_'+arm),reference,sha(refraw),ac,[])
            require(recovered['rows']==old['rows'] and recovered['checkpoint']==old['checkpoint'],'exact saved/new world and allocator checkpoint equivalence')
            cases.append(dict(name=arm,initial=recovered['initial']['state'],rows=recovered['rows'],final=recovered['checkpoint']['state']))
        else:results.extend(recovered['results'])
    require(owner['worlds']==expected_owner,'all world requests frozen before dispatch')
    measurement=load('measurement.json');check(cases,campaign,measurement,registry)
    require(measurement==json.loads((parent/'measurement.json').read_text()) and registry==json.loads((parent/'registry.json').read_text()),'unchanged complete 40-slot measurement and registry')
    report=independent_report(measurement,registry,results,cases,campaign)
    require(report==load('report.json')==json.loads((parent/'report.json').read_text()),'independent score decision and saved equivalence')
    freeze=next(e['value'] for e in events if e['kind']=='registry')
    require(freeze==dict(sha256=sha((package/'registry.json').read_bytes()),measurement_sha256=sha((package/'measurement.json').read_bytes()),count=9,slots=40),'complete registry frozen once before assays')
    require(events[-1]==dict(kind='close',value=dict(reason='complete',decision_sha256=sha((package/'report.json').read_bytes()))),'content-bound closure')
    require(summary['accounted_rewrites_lower']==summary['accounted_rewrites_upper']==work==335,'exact aggregate funding and native evidence')
    require(all(load('manifest.json')[k] is False for k in ['readiness_gate_complete','scientific_acceptance','evolution_demonstrated','campaign_launch_authorized']), 'no scientific admission')
    return dict(engineering_live_development_pass=True,native_work_lower=work,native_work_upper=work,development_worlds=4,sampling_slots=40,assay_requests=9,
                native_units=5,retained_pairs=0,measured_native_and_archive_ms=sum(r['native_and_archive_ms'] for r in routes),
                maximum_admission_charge_bytes=max(admissions),saved_world_checkpoint_equivalence=True,scientific_worlds_executed=0,
                live_development_dispatch_integrated=True,scientific_native_profile_implemented=False)


if __name__=='__main__':print(json.dumps(audit(Path(sys.argv[1])),indent=2,sort_keys=True))
