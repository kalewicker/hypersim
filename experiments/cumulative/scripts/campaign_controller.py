"""Durable campaign scheduling rehearsal. No native executor or scientific admission.

Receipts are supplied by a verified evidence adapter or explicitly synthetic fixture.
Timing/storage values exercise accounting; live clock/spool enforcement is a separate
dispatch boundary. A surviving intent is never automatically dispatched again.
"""
import copy
import fcntl
import json
from pathlib import Path
from compact_evidence import encode_json, publish, read_bounded, require, sha

PROFILE = 'cumulative-campaign-controller-rehearsal-v1'
FLAGS = {k:False for k in ['readiness_gate_complete','scientific_acceptance','evolution_demonstrated','campaign_launch_authorized']}


def integer(v):
    return type(v) is int and v >= 0


def identity(v):
    return type(v) is str and len(v)==64 and all(c in '0123456789abcdef' for c in v)


def initial(contract):
    require(contract['profile']==PROFILE and contract['purpose'] in ['saved_development','synthetic_full_scale'], 'rehearsal only; scientific dispatch unavailable')
    require(all(contract.get(k) is False for k in FLAGS), 'scientific flags closed')
    require(set(contract['limits'])=={'rewrites','native_ms','workflow_ms','storage_bytes'}, 'fixed resource dimensions')
    require(all(integer(v) and v>0 for v in contract['limits'].values()), 'positive resource ceilings')
    worlds=contract['worlds']
    require(0<len(worlds)<=188 and len({w['world_id'] for w in worlds})==len(worlds), 'unique fixed world denominator')
    require(all(identity(w['request_sha256']) and integer(w['work_bound']) and w['work_bound']>0 for w in worlds), 'frozen world request/reservation')
    require(contract['slots_per_world']==10 and contract['batch_size']==128 and contract['assay_work_bound']==20 and contract['maximum_assays']==78960, 'frozen census/batch bounds')
    return dict(phase='worlds',worlds=[dict(world_id=w['world_id'],status='not_started',evidence=None) for w in worlds],
                pending=None,registry=None,next_world=0,next_assay=0,rewrites_lower=0,rewrites_upper=0,
                native_ms=0,elapsed_ms=0,peak_storage_bytes=0,closed=False,reason=None,decision_sha256=None,**FLAGS)


def next_unit(c,s):
    require(not s['closed'] and s['pending'] is None, 'closed or unresolved intent; never redispatch')
    if s['phase']=='worlds':
        w=c['worlds'][s['next_world']]
        return dict(unit='world',index=s['next_world'],request_sha256=w['request_sha256'],work_bound=w['work_bound'])
    if s['phase']=='assays':
        end=min(s['next_assay']+128,s['registry']['count'])
        request=dict(registry_sha256=s['registry']['sha256'],start=s['next_assay'],end=end)
        return dict(unit='assays',index=s['next_assay'],request_sha256=sha(encode_json(request)),work_bound=20*(end-s['next_assay']),end=end)
    raise ValueError('no schedulable unit until registry/closure')


def transition(c,s,event):
    """Pure validated state transition; caller durably publishes intent before dispatch."""
    s=copy.deepcopy(s);require(not s['closed'], 'terminal campaign cannot change')
    kind=event['kind'];v=event['value'];limits=c['limits']
    if kind=='intent':
        unit=next_unit(c,s)
        require(set(v)=={'unit','native_ms_bound','elapsed_ms','storage_charge_bytes'} and v['unit']==unit, 'exact next frozen request; no skip or seed replacement')
        require(all(integer(v[k]) for k in ['native_ms_bound','elapsed_ms','storage_charge_bytes']), 'typed pre-dispatch accounting')
        require(s['elapsed_ms']<=v['elapsed_ms']<limits['workflow_ms'], 'workflow deadline or clock reversal')
        require(s['rewrites_upper']+unit['work_bound']<=limits['rewrites'], 'aggregate rewrite reservation')
        require(v['native_ms_bound']>0 and s['native_ms']+v['native_ms_bound']<=limits['native_ms'], 'aggregate native time reservation')
        require(v['storage_charge_bytes']<=limits['storage_bytes'], 'storage admission before dispatch')
        s['pending']=copy.deepcopy(v);s['rewrites_upper']+=unit['work_bound'];s['native_ms']+=v['native_ms_bound']
        s['elapsed_ms']=v['elapsed_ms'];s['peak_storage_bytes']=max(s['peak_storage_bytes'],v['storage_charge_bytes'])
        if unit['unit']=='world':s['worlds'][unit['index']]['status']='pending'
    elif kind=='receipt':
        require(s['pending'] is not None, 'receipt requires durable intent')
        p=s['pending'];unit=p['unit']
        require(set(v)=={'request_sha256','status','work_lower','work_upper','native_ms','elapsed_ms','evidence_sha256'}, 'receipt fields')
        require(v['request_sha256']==unit['request_sha256'] and identity(v['evidence_sha256']), 'receipt bound to exact request and evidence')
        require(v['status'] in ['complete','unknown'], 'no outcome-based retry status')
        require(all(integer(v[k]) for k in ['work_lower','work_upper','native_ms','elapsed_ms']), 'typed measured resource bounds')
        require(v['work_lower']<=v['work_upper']<=unit['work_bound'] and v['native_ms']<=p['native_ms_bound'], 'receipt exceeds funded work/time')
        require(v['elapsed_ms']>=s['elapsed_ms'], 'monotone receipt clock')
        require(v['status']!='complete' or v['work_lower']==v['work_upper'], 'complete work must be exact')
        s['rewrites_lower']+=v['work_lower']
        # Unknown receipt retains the full pre-dispatch charge, even if a lower
        # bound is visible. It cannot create funding for another speculative unit.
        if v['status']=='complete':
            s['rewrites_upper']-=unit['work_bound']-v['work_upper'];s['native_ms']-=p['native_ms_bound']-v['native_ms']
        s['elapsed_ms']=v['elapsed_ms'];s['pending']=None
        if unit['unit']=='world':
            s['worlds'][unit['index']].update(status=v['status'],evidence=v['evidence_sha256']);s['next_world']+=1
            if s['next_world']==len(c['worlds']):s['phase']='registry'
        else:
            s['next_assay']=unit['end']
            if s['next_assay']==s['registry']['count']:s['phase']='decision'
        if v['status']=='unknown' or v['elapsed_ms']>=limits['workflow_ms']:
            s['phase']='stopped';s['reason']='unknown_execution' if v['status']=='unknown' else 'workflow_deadline'
    elif kind=='registry':
        require(s['phase']=='registry' and s['pending'] is None, 'registry only after every fixed world is terminal')
        require(set(v)=={'sha256','measurement_sha256','count','slots'} and identity(v['sha256']) and identity(v['measurement_sha256']), 'frozen registry/measurement identity')
        require(integer(v['count']) and v['count']<=c['maximum_assays'] and type(v['slots']) is int and v['slots']==len(c['worlds'])*10, 'complete measurement denominator')
        require(s['rewrites_upper']+v['count']*20<=limits['rewrites'], 'fund entire frozen assay denominator before first batch')
        s['registry']=copy.deepcopy(v);s['phase']='assays' if v['count'] else 'decision'
    elif kind=='close':
        require(set(v)=={'reason','decision_sha256'} and v['reason'] in ['complete','operational_stop'], 'explicit closure reason')
        if v['reason']=='complete':
            require(s['phase']=='decision' and s['pending'] is None and identity(v['decision_sha256']), 'complete evidence before decision closure')
        else:require(v['decision_sha256'] is None, 'incomplete campaign has no negative/positive decision')
        if s['pending'] is not None and s['pending']['unit']['unit']=='world':
            s['worlds'][s['pending']['unit']['index']]['status']='unknown'
        s.update(closed=True,phase='closed',reason=v['reason'],decision_sha256=v['decision_sha256'])
    else:raise ValueError('unknown controller transition')
    return s


class Controller:
    def __init__(self,root):
        self.root=Path(root)
        self.contract=json.loads(read_bounded(self.root/'contract.json',1048576))
        initial(self.contract)

    @classmethod
    def create(cls,root,contract):
        initial(contract);root=Path(root);root.mkdir(exist_ok=False)
        (root/'events').mkdir();(root/'lock').touch(exist_ok=False)
        publish(root/'contract.json',encode_json(contract))
        return cls(root)

    def replay(self):
        require({p.name for p in self.root.iterdir()}=={'contract.json','events','lock'}, 'unexpected controller files')
        state=initial(self.contract);head=sha(read_bounded(self.root/'contract.json',1048576));paths=sorted((self.root/'events').iterdir())
        require(len(paths)<=2000, 'controller event bound')
        for i,p in enumerate(paths):
            require(p.name==f'{i:08}.json', 'torn or missing publication; no automatic repair')
            raw=read_bounded(p,16384);entry=json.loads(raw)
            require(set(entry)=={'sequence','previous','event'} and type(entry['sequence']) is int and entry['sequence']==i and entry['previous']==head, 'controller hash chain')
            state=transition(self.contract,state,entry['event']);head=sha(raw)
        return state,head,len(paths)

    def append(self,kind,value):
        return self.append_many([(kind,value)])

    def append_many(self,events):
        """Publish rehearsal events under one lock; each event remains independently durable."""
        read_bounded(self.root/'lock',0)
        with (self.root/'lock').open('rb') as lock:
            fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
            state,head,n=self.replay()
            for kind,value in events:
                require(n<2000, 'event capacity before action')
                event=dict(kind=kind,value=value);updated=transition(self.contract,state,event)
                raw=encode_json(dict(sequence=n,previous=head,event=event))
                publish(self.root/'events'/f'{n:08}.json',raw)
                state=updated;head=sha(raw);n+=1
            return state
