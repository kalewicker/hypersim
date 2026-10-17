"""Frozen zero-native binding cases, including every world and complete registry tails."""
import copy
from scientific_launch_binding import *
from test_scientific_launch import envelope


def cases(a):
    result=[]
    def add(name,e,valid=True):result.append(dict(name=name,raw=canonical(e)+b'\n',valid=valid))
    for i,r in enumerate(world_requests()):add(f'world_{i:03}',envelope(r,a))
    roles=parse(ADMISSION_RAW)['roles']
    for count in [129,78960]:
        reg=dict(profile='cumulative-scientific-registry-v1',assays=[dict(id=i,role=roles[i%13],stages=[-1,0,1,2,3,4,0,1]) for i in range(count)])
        for start in [0,((count-1)//128)*128]:add(f'assays_{count}_{start}',envelope(request(registry=reg,start=start),a))
    good=envelope(world_requests()[0],a)
    for k,v in [('profile','wrong'),('mode','launch'),('approved',True),('approved',1),('approval_reference','invented'),('campaign_id','invalid'),('launch_sha256','0'*64),('panel_sha256','0'*64),('binary_sha256','0'*64),('source_sha256','invalid'),('limits',{**a['limits'],'rewrites':2000001})]:
        e=copy.deepcopy(good);e['authorization'][k]=v;add(f'bad_authorization_{k}_{len(result)}',e,False)
    for k,v in [('owner_sha256','invalid'),('intent_sha256',None),('request_sha256','0'*64),('work_bound',249),('work_bound',True),('extra',0)]:
        e=copy.deepcopy(good);e['controller'][k]=v;add(f'bad_controller_{k}_{len(result)}',e,False)
    for k,v in [('mode','execute'),('launch_authorized',True),('execution_allowance',249),('native_budget',2000001),('source',{}),('fault',{})]:
        e=copy.deepcopy(good);e['request'][k]=v;add(f'bad_request_{k}',e,False)
    e=copy.deepcopy(good);e['request']['world']['seed']^=1;add('wrong_scientific_seed',e,False)
    raw=canonical(good);result.append(dict(name='duplicate_authorization',raw=raw[:-1]+b',"authorization":{}}',valid=False))
    e=copy.deepcopy(good);e['extra']=None;add('extra_binding_field',e,False)
    reg=dict(profile='cumulative-scientific-registry-v1',assays=[dict(id=i,role='intact',stages=[0]) for i in range(129)])
    e=envelope(request(registry=reg),a);e['request']['registry']['assays'][-1]['stages']=[5]
    e['request']['registry_sha256']=sha(canonical(e['request']['registry']));e['controller']['request_sha256']=sha(canonical(e['request']));add('invalid_late_row_outside_batch',e,False)
    return result
