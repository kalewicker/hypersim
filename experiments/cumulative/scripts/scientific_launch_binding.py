"""Exact scientific request/approval binding. Preview is never execution permission."""
import copy
from functools import lru_cache
from pathlib import Path
from scientific_admission_cases import parse,canonical,sha,check
from compact_evidence import require,encode_json
PACKAGE=Path(__file__).resolve().parents[1]
CONTRACT_RAW=(PACKAGE/'scientific_launch_contract.json').read_bytes()
CONTRACT=parse(CONTRACT_RAW)
ADMISSION_RAW=(PACKAGE/'scientific_admission_contract.json').read_bytes()
PANEL_RAW=(PACKAGE/'scientific_panel.json').read_bytes()
PLANNING_RAW=(PACKAGE/'campaign_planning_contract.json').read_bytes()


def authorization(a,binary_hash,execute=False):
    require(set(a)=={'profile','mode','approved','approval_reference','campaign_id','launch_sha256','panel_sha256','binary_sha256','source_sha256','limits'},'authorization fields')
    require(a['profile']=='cumulative-scientific-authorization-v1' and a['launch_sha256']==sha(CONTRACT_RAW) and a['panel_sha256']==sha(PANEL_RAW) and a['binary_sha256']==binary_hash,'authorization identity')
    require(canonical(a['limits'])==canonical(CONTRACT['limits']),'exact approved resource envelope')
    for k in ['source_sha256','campaign_id']:
        require(type(a[k]) is str and len(a[k])==64 and all(x in '0123456789abcdef' for x in a[k]),'authorization hash domain')
    require((a['mode']=='preview' and a['approved'] is False and a['approval_reference'] is None) or
            (a['mode']=='launch' and a['approved'] is True and type(a['approval_reference']) is str and 1<=len(a['approval_reference'].encode())<=1024),'explicit launch approval or inert preview')
    if execute:require(a['mode']=='launch' and a['approved'] is True,'launch authorization required; preview cannot execute')
    return a


def request(world=None,registry=None,start=0):
    c=parse(ADMISSION_RAW)
    r=dict(profile=c['profile'],mode='check_only',campaign_sha256=sha(ADMISSION_RAW),panel_sha256=sha(PANEL_RAW),unit='world' if world else 'assays',world=world,registry=registry,registry_sha256=None if world else sha(canonical(registry)),range=None if world else dict(start=start,end=min(start+128,len(registry['assays']))),native_budget=2000000,execution_allowance=c['world_reservations'][world['arm']] if world else 20*min(128,len(registry['assays'])-start),journal_byte_allowance=32505856,source=None,fault=None,launch_authorized=False)
    check(canonical(r),ADMISSION_RAW,PANEL_RAW,PLANNING_RAW)
    return r


@lru_cache(maxsize=1)
def _world_request_bytes():return tuple(canonical(request(world=w)) for w in parse(PANEL_RAW)['worlds'])


def world_requests():return [parse(raw) for raw in _world_request_bytes()]


def project(r):
    if r['unit']=='world':
        w=r['world'];spec={k:copy.deepcopy(v) for k,v in w.items() if k not in ['block','seed','world_id']};spec['name']=w['world_id']
        return dict(unit='world',spec=spec,seed=w['seed'],items=[],start=0,end=w['ticks'],execution_allowance=r['execution_allowance'])
    return dict(unit='assays',spec=None,seed=None,items=r['registry']['assays'],start=r['range']['start'],end=r['range']['end'],execution_allowance=r['execution_allowance'])


def validate(e,binary_hash,execute=False):
    require(set(e)=={'profile','request','authorization','controller'},'binding fields')
    require(e['profile']=='cumulative-scientific-execution-binding-v1','binding profile')
    r=e['request'];admitted=check(canonical(r),ADMISSION_RAW,PANEL_RAW,PLANNING_RAW)
    authorization(e['authorization'],binary_hash,execute)
    t=e['controller'];require(set(t)=={'owner_sha256','intent_sha256','request_sha256','work_bound'},'durable controller binding fields')
    for k in ['owner_sha256','intent_sha256']:
        require(type(t[k]) is str and len(t[k])==64 and all(x in '0123456789abcdef' for x in t[k]),'controller hash domain')
    require(t['request_sha256']==sha(canonical(r)) and type(t['work_bound']) is int and t['work_bound']==admitted['admitted']['work_bound'],'controller exact native request and funded work')
    return project(r)


def owner(a):
    authorization(a,a['binary_sha256'])
    return dict(profile='cumulative-scientific-controller-v1',purpose=a['mode'],authorization=a,
                worlds=[dict(world_id=r['world']['world_id'],request_sha256=sha(canonical(r)),work_bound=r['execution_allowance']) for r in world_requests()],
                limits=CONTRACT['limits'],slots_per_world=10,batch_size=128,assay_work_bound=20,maximum_assays=78960,
                readiness_gate_complete=False,scientific_acceptance=False,evolution_demonstrated=False,campaign_launch_authorized=a['approved'])


def validate_owner(c):
    require(canonical(c)==canonical(owner(c['authorization'])),'exact authorized owner and entire fixed panel')
