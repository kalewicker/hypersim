"""Independent controller ledger checks. Does not import the controller/reducer."""
import json
from pathlib import Path
import sys
from compact_evidence import sha, encode_json, read_bounded, require, source_inventory


def audit_ledger(root):
    raw=read_bounded(root/'contract.json',1048576);c=json.loads(raw);head=sha(raw)
    require(c['profile']=='cumulative-scientific-controller-v1' and c['purpose'] in ['preview','launch'], 'no scientific executor')
    from scientific_launch_binding import validate_owner
    validate_owner(c)
    limits=c['limits'];cursor=0;assay_cursor=0;registry=None;active=None;closed=False;stopped=False
    low=high=native=elapsed=peak=0;statuses=['not_started']*len(c['worlds']);events=list(sorted((root/'events').iterdir()))
    require(len(events)<=2000 and all(c[f] is False for f in ['readiness_gate_complete','scientific_acceptance','evolution_demonstrated']), 'bounded rehearsal')
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

