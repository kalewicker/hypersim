"""Independent controller ledger checks. Does not import the controller/reducer."""
import json
from pathlib import Path
import sys
from compact_evidence import sha, encode_json, read_bounded, require, source_inventory


def audit_ledger(root):
    raw=read_bounded(root/'contract.json',1048576);c=json.loads(raw);head=sha(raw)
    require(c['profile']=='cumulative-campaign-controller-rehearsal-v1' and c['purpose'] in ['saved_development','synthetic_full_scale'], 'no scientific executor')
    limits=c['limits'];cursor=0;assay_cursor=0;registry=None;active=None;closed=False;stopped=False
    low=high=native=elapsed=peak=0;statuses=['not_started']*len(c['worlds']);events=list(sorted((root/'events').iterdir()))
    require(len(events)<=2000 and all(c[f] is False for f in ['readiness_gate_complete','scientific_acceptance','evolution_demonstrated','campaign_launch_authorized']), 'bounded rehearsal')
    for i,p in enumerate(events):
        require(not closed and p.name==f'{i:08}.json','no event gap or mutation after closure')
        raw=read_bounded(p,16384);e=json.loads(raw)
        require(e['sequence']==i and type(e['sequence']) is int and e['previous']==head,'independent event chain');head=sha(raw)
        kind=e['event']['kind'];v=e['event']['value']
        if kind=='intent':
            require(active is None and not stopped, 'no duplicate or unknown retry')
            if cursor<len(c['worlds']):
                w=c['worlds'][cursor];unit=dict(unit='world',index=cursor,request_sha256=w['request_sha256'],work_bound=w['work_bound'])
                statuses[cursor]='pending'
            else:
                require(registry is not None and assay_cursor<registry['count'],'assays require frozen complete registry')
                end=min(assay_cursor+128,registry['count'])
                ref=dict(registry_sha256=registry['sha256'],start=assay_cursor,end=end)
                unit=dict(unit='assays',index=assay_cursor,end=end,request_sha256=sha(encode_json(ref)),work_bound=20*(end-assay_cursor))
            require(v['unit']==unit,'exact registered order/range')
            require(all(type(v[k]) is int and v[k]>=0 for k in ['native_ms_bound','elapsed_ms','storage_charge_bytes']), 'typed admission')
            require(elapsed<=v['elapsed_ms']<limits['workflow_ms'] and v['native_ms_bound']>0,'pre-dispatch time')
            high+=unit['work_bound'];native+=v['native_ms_bound'];peak=max(peak,v['storage_charge_bytes']);elapsed=v['elapsed_ms']
            require(high<=limits['rewrites'] and native<=limits['native_ms'] and peak<=limits['storage_bytes'],'all funding before execution')
            active=v
        elif kind=='receipt':
            require(active is not None and v['request_sha256']==active['unit']['request_sha256'],'exact pending request')
            require(v['status'] in ['complete','unknown'] and all(type(v[k]) is int and v[k]>=0 for k in ['work_lower','work_upper','native_ms','elapsed_ms']), 'typed receipt')
            require(0<=v['work_lower']<=v['work_upper']<=active['unit']['work_bound'] and v['native_ms']<=active['native_ms_bound'] and v['elapsed_ms']>=elapsed, 'receipt within bounds')
            require(len(v['evidence_sha256'])==64,'evidence reference')
            low+=v['work_lower'];elapsed=v['elapsed_ms']
            if v['status']=='complete':
                require(v['work_lower']==v['work_upper'],'exact complete work')
                high-=active['unit']['work_bound']-v['work_upper'];native-=active['native_ms_bound']-v['native_ms']
            else:stopped=True
            if active['unit']['unit']=='world':statuses[cursor]=v['status'];cursor+=1
            else:assay_cursor=active['unit']['end']
            if elapsed>=limits['workflow_ms']:stopped=True
            active=None
        elif kind=='registry':
            require(cursor==len(c['worlds']) and active is None and registry is None and not stopped,'one registry after all worlds')
            require(v['slots']==10*cursor and type(v['count']) is int and 0<=v['count']<=78960,'full census denominator')
            require(high+20*v['count']<=limits['rewrites'],'whole assay denominator funded before batching');registry=v
        elif kind=='close':
            require(v['reason'] in ['complete','operational_stop'],'closure class')
            if v['reason']=='complete':
                require(not stopped and active is None and registry is not None and assay_cursor==registry['count'] and len(v['decision_sha256'])==64,'complete closure denominator')
            else:
                require(v['decision_sha256'] is None,'no decision on partial campaign')
                if active is not None and active['unit']['unit']=='world':statuses[cursor]='unknown'
            closed=True
        else:raise ValueError('unknown event')
    return dict(events=len(events),closed=closed,worlds=len(statuses),world_statuses=statuses,assays_completed=assay_cursor,
                accounted_rewrites_lower=low,accounted_rewrites_upper=high,accounted_native_ms=native,peak_storage_bytes=peak,
                sampling_slots=len(statuses)*10,head=head)


def audit(package):
    c=json.loads((package/'contract.json').read_text());summaries=json.loads((package/'controller_summaries.json').read_text())
    results={name:audit_ledger(package/'controllers'/name) for name in c['routes']}
    require(results==summaries,'published independent controller summaries')
    require(results['saved_development']['worlds']==4 and results['saved_development']['assays_completed']==9,'real saved evidence denominator')
    require(results['full_scale']['worlds']==188 and results['full_scale']['assays_completed']==78960 and results['full_scale']['closed'],'full synthetic capacity rehearsal')
    require(results['interrupted']['world_statuses'][0]=='unknown' and results['interrupted']['world_statuses'][1:]==['not_started']*187,'interruption preserves fixed unknown denominator')
    require(results['interrupted']['accounted_rewrites_upper']==250,'unknown work stays reserved')
    draft=json.loads((package/'final_campaign_draft.json').read_text())
    require(draft['status']=='draft_not_authorized' and draft['claim_scope']['environmental_generalization'] is False and draft['campaign_launch_authorized'] is False,'bounded claim and no launch')
    require(draft['panel']['world_count']==188 and draft['panel']['sampling_slots']==1880,'fixed proposed panel')
    parent=package/'measurement_parent';source_inventory(parent,c['parent']['seal_sha256'])
    require(draft['panel']==json.loads((parent/'prospective_pending.json').read_text())['panel'] and draft['prospective_contract']==json.loads((parent/'campaign_contract.json').read_text()), 'unchanged seeds, criteria and inference')
    require(draft['proposed_budget']==c['virtual_resource_ceilings'] and draft['budget_approved'] is False and draft['live_dispatch_integrated'] is False, 'proposed ceilings are not authorization')
    full_contract=json.loads((package/'controllers/full_scale/contract.json').read_text())
    bounds={'endogenous':250,'external':240,'nonheritable':250,'frozen':202}
    wanted=[dict(world_id=w['world_id'],request_sha256=sha(encode_json(w)),work_bound=bounds[w['arm']]) for w in draft['panel']['worlds']]
    require(full_contract['worlds']==wanted and full_contract['limits']==c['virtual_resource_ceilings'], 'exact full-scale frozen identities and reservations')
    for name,registry_path,measurement_path,decision_path in [
        ('full_scale',package/'synthetic_registry.json',package/'synthetic_measurement.json',package/'synthetic_decision.json'),
        ('saved_development',parent/'registry.json',parent/'measurement.json',parent/'report.json')]:
        events=[json.loads(p.read_text())['event'] for p in sorted((package/'controllers'/name/'events').iterdir())]
        freezes=[e['value'] for e in events if e['kind']=='registry']
        require(len(freezes)==1 and freezes[0]['sha256']==sha(registry_path.read_bytes()) and freezes[0]['measurement_sha256']==sha(measurement_path.read_bytes()), 'content-bound measurement/registry barrier')
        items=json.loads(registry_path.read_text())['assays']
        require(len(items)==freezes[0]['count'] and all(a['id']==i for i,a in enumerate(items)), 'complete actual request inventory')
        require(events[-1]['value']['decision_sha256']==sha(decision_path.read_bytes()), 'decision bound to saved or labeled synthetic report')
    require(all(json.loads((package/'manifest.json').read_text())[f] is False for f in ['readiness_gate_complete','scientific_acceptance','evolution_demonstrated','campaign_launch_authorized']), 'closed scientific flags')
    # Re-run the already frozen measurement auditor in the publisher; bind that
    # successful receipt here to the exact retained parent, never a synthetic score.
    receipt=json.loads((package/'commands/parent_recheck.json').read_text())
    require(receipt['exit_code']==0 and json.loads(receipt['stdout'])['seal_sha256']==c['parent']['seal_sha256'],'authenticated saved measurement audit')
    refs=json.loads((package/'saved_evidence_bindings.json').read_text())
    saved_events=[json.loads(p.read_text())['event'] for p in sorted((package/'controllers/saved_development/events').iterdir())]
    receipt_events=[e['value'] for e in saved_events if e['kind']=='receipt']
    require(len(refs)==len(receipt_events)==5,'complete saved evidence references')
    from compact_evidence import Store
    from audit_campaign_adapter import recover, recover_values
    from audit_durability import read
    from audit_spool import Journal,ledger
    info=json.loads((parent/'adapter_store.json').read_text());store=Store(parent/'adapter_evidence',info['index_sha256'],json.loads((parent/'contract.json').read_text())['adapter_parent']['seal_sha256']);store.verify_all()
    ac=json.loads((store.root/'contract.json').read_text());observed=[]
    for arm in ['endogenous','external','nonheritable','frozen']:
        raw=(store.root/'inputs'/('world_'+arm+'.json')).read_bytes();r=recover(store.root/'journals'/('world_'+arm),json.loads(raw),sha(raw),ac,[])
        observed.append(dict(request_sha256=sha(raw),evidence_sha256=r['head'],work=r['here_upper']))
    origin=parent/'native_origin';states,_=ledger(origin/'native_spool');require(len(states)==1,'saved batch identity')
    unit,state=next(iter(states.items()));raw=(origin/'native_spool/requests'/(unit+'.json')).read_bytes()
    st=Store(origin/'native_spool/compact'/unit,state['index_sha256'],state['seal_sha256']);vals,head,staging=read(Journal(st))
    items=json.loads((parent/'registry.json').read_text())['assays'];r=recover_values(vals,head,staging,json.loads(raw),sha(raw),ac,items)
    logical=sha(encode_json(dict(registry_sha256=sha((parent/'registry.json').read_bytes()),start=0,end=9)))
    observed.append(dict(request_sha256=logical,evidence_sha256=head,work=r['here_upper']))
    require(refs==observed,'saved observations independently recovered')
    for ref,event in zip(refs,receipt_events):
        require(event['request_sha256']==ref['request_sha256'] and event['evidence_sha256']==ref['evidence_sha256'] and event['work_lower']==event['work_upper']==ref['work'],'saved work never replaced by synthetic receipt')
    return dict(engineering_controller_rehearsal_pass=True,native_rewrites=0,scientific_worlds_executed=0,
                saved_worlds=4,saved_assays=9,full_scale_worlds=188,full_scale_assay_requests=78960,
                full_scale_batches=617,scientific_sampling_slots_pending=1880,production_dispatch_integrated=False)


if __name__=='__main__':print(json.dumps(audit(Path(sys.argv[1])),indent=2,sort_keys=True))
