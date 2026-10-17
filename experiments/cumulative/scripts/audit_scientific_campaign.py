"""Read-only scientific ledger/native/measurement correspondence audit."""
import json
import sys
from pathlib import Path
from compact_evidence import Store,read_bounded,require,sha,encode_json
from audit_spool import ledger,Journal
from audit_scientific_controller import audit_ledger
from audit_scientific_execution import recover
from scientific_launch_binding import authorization,parse,canonical,PLANNING_RAW,PANEL_RAW,CONTRACT
from audit_campaign_measurement import check,independent_report
from plan_campaign import analyze_outcomes


def audit(out):
    load=lambda n:parse(read_bounded(out/n,33554432))
    a=load('authorization.json');binary=sha(read_bounded(out/'cumulative-scientific-launch',33554432));authorization(a,binary,True)
    pins=load('source_sha256.json');require(sha((out/'source_sha256.json').read_bytes())==a['source_sha256'],'approved complete source identity')
    require(all(sha(read_bounded(out/'source_snapshots'/n,33554432))==h for n,h in pins.items()),'frozen scientific sources')
    summary=audit_ledger(out/'controller');ownerraw=(out/'controller/contract.json').read_bytes()
    require(parse(ownerraw)['authorization']==a,'campaign authorization owner')
    entries=[p.read_bytes() for p in sorted((out/'controller/events').iterdir())]
    intents=[(sha(raw),parse(raw)['event']['value']) for raw in entries if parse(raw)['event']['kind']=='intent']
    receipts=[parse(raw)['event']['value'] for raw in entries if parse(raw)['event']['kind']=='receipt']
    states,_=ledger(out/'native_spool');require(len(states)==len(intents)==len(receipts),'all funded actions have complete audited receipts')
    require({p.name for p in (out/'native_spool/claims').iterdir()}=={h for h,_ in intents},'each durable intent consumed exactly once')
    cases=[];results=[];work=0
    for i,((head,intent),receipt) in enumerate(zip(intents,receipts)):
        unit=f'u{i:06}';s=states[unit];raw=(out/'native_spool/requests'/(unit+'.json')).read_bytes();e=parse(raw)
        require(e['authorization']==a and e['controller']['owner_sha256']==sha(ownerraw) and e['controller']['intent_sha256']==head and e['controller']['work_bound']==intent['unit']['work_bound'],'native authorization and actual durable intent')
        require(read_bounded(out/'native_spool/claims'/head,65)==(sha(canonical(e))+'\n').encode(),'atomic native intent claim')
        require(sha(raw)==s['request_sha256'] and s['kind']=='released','complete authenticated compact native unit')
        store=Store(out/'native_spool/compact'/unit,s['index_sha256'],s['seal_sha256']);store.verify_all()
        recovered=recover(Journal(store),e,binary)
        require(recovered['status']=='complete' and not recovered['staging'] and receipt['request_sha256']==intent['unit']['request_sha256'] and receipt['work_lower']==receipt['work_upper']==recovered['lower']==recovered['upper'] and receipt['evidence_sha256']==recovered['head'],'exact independently verified native receipt')
        exit_receipt=load('native_spool/requests/'+unit+'.receipt.json');require(exit_receipt['returncode']==0 and not exit_receipt['timed_out'],'one completed child invocation')
        r=e['request'];work+=recovered['lower']
        if r['unit']=='world':
            require(intent['unit']['request_sha256']==e['controller']['request_sha256'],'world request/intent identity')
            cases.append(dict(name=r['world']['world_id'],initial=recovered['results'][0]['row']['before'],rows=[v['row'] for v in recovered['results']],final=recovered['checkpoint']['state']))
        else:
            logical=sha(encode_json(dict(registry_sha256=sha(encode_json(r['registry'])),start=r['range']['start'],end=r['range']['end'])))
            require(logical==intent['unit']['request_sha256'] and r['registry']==load('registry.json'),'complete frozen assay identity')
            for v in recovered['results']:
                n=v['native']['results'][0];results.append(dict(request=v['request'],native=dict(results=[dict(status=n['status'],score=n['score'])])))
    panel=parse(PANEL_RAW);require([c['name'] for c in cases]==[w['world_id'] for w in panel['worlds']],'entire fixed panel in order')
    reg=load('registry.json');reg=dict(profile='cumulative-campaign-adapter-readiness-v1',assays=reg['assays'])
    measurement=load('measurement.json');planning=parse(PLANNING_RAW);check(cases,planning,measurement,reg)
    freezes=[parse(raw)['event']['value'] for raw in entries if parse(raw)['event']['kind']=='registry']
    require(freezes==[dict(sha256=sha((out/'registry.json').read_bytes()),measurement_sha256=sha((out/'measurement.json').read_bytes()),count=len(reg['assays']),slots=1880)],'exact measurement and registry frozen before assays')
    measured=independent_report(measurement,reg,results,cases,planning)
    outcomes=dict(blocks=47,worlds=[dict(block=w['block'],arm=w['arm'],status=v['status'],qualified=v['qualified']) for w,v in zip(panel['worlds'],measured['worlds'])])
    report=dict(profile=CONTRACT['profile'],measurement_report=measured,outcomes=outcomes,decision=analyze_outcomes(outcomes,planning),claim_scope=CONTRACT['claim_scope'])
    require(report==load('report.json'),'complete independently reconstructed report')
    require(parse(entries[-1])['event']==dict(kind='close',value=dict(reason='complete',decision_sha256=sha(encode_json(report)))),'scientific closure identity')
    require(summary['closed'] and summary['accounted_rewrites_lower']==summary['accounted_rewrites_upper']==work,'complete campaign work accounting')
    return dict(scientific_worlds=188,sampling_slots=1880,assay_requests=len(reg['assays']),native_rewrites=work,independent_campaign_audit_pass=True)


if __name__=='__main__':print(json.dumps(audit(Path(sys.argv[1]).resolve()),indent=2,sort_keys=True))
