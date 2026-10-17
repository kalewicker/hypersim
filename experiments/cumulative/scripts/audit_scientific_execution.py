"""Independent executor journal verification; never advances native worlds."""
import copy
import json
from pathlib import Path
import sys
from audit_readiness import require
from audit_durability import read, checkpoint, assay, reservation
from audit_world import rows, registered
from scientific_admission_cases import parse, check, sha


def recover(directory,envelope,binary_hash):
    from scientific_launch_binding import validate
    route=validate(envelope,binary_hash,execute=True)
    values,head,staging=read(directory)
    require(sum(p.stat().st_size for p in directory.iterdir())<=32505856,'writer allowance')
    require(values[0]==dict(kind='header',profile='cumulative-scientific-execution-binding-v1',binding=envelope),'exact scientific identity and authorization header')
    world=route['unit']=='world';spec=route['spec'];items=route['items']
    end=route['end']
    cp=None;work=0;index=route['start'];pending=None;complete=False;observed=[]
    for v in values[1:]:
        require(not complete,'no records after completion')
        kind=v['kind']
        if kind=='initial':
            require(world and cp is None and index==0 and pending is None and set(v)=={'kind','checkpoint'},'one initialization')
            cp=v['checkpoint'];checkpoint(cp);registered(cp['state'],spec,route['seed'])
        elif kind=='intent':
            require(pending is None and index<end and (not world or cp is not None),'intent domain')
            b=reservation(cp['state'],spec) if world else 20
            require(v==dict(kind='intent',index=index,work_before=work,work_bound=b) and work+b<=route['execution_allowance'],'pre-native funding')
            pending=b
        elif kind=='result':
            require(pending is not None and type(v['index']) is int and v['index']==index,'result identity')
            if world:
                require(set(v)=={'kind','index','row','checkpoint','native_rewrites','work_total'},'world fields')
                checkpoint(v['checkpoint']);n,_=rows(cp['state'],[v['row']],v['checkpoint']['state'],spec);cp=v['checkpoint']
            else:
                require(set(v)=={'kind','index','request','native','native_rewrites','work_total'} and v['request']==items[index],'assay fields and global identity')
                local=copy.deepcopy(items[index]);local['id']=0
                if local['role'] in ['padded_remove_later','padded_remove_both']:local['role']='intact'
                n=assay(v['native'],local)
            require(type(v['native_rewrites']) is int and type(v['work_total']) is int and v['native_rewrites']==n<=pending and v['work_total']==work+n,'independent work counter')
            observed.append(v);work+=n;index+=1;pending=None
        elif kind=='complete':
            require(index==end and pending is None and v==dict(kind='complete',work_total=work,work_here=work),'complete fixed denominator')
            complete=True
        else:raise ValueError('unknown record')
    return dict(status='complete' if complete else 'unknown' if pending is not None else 'partial',lower=work,upper=work+(pending or 0),head=head,staging=staging,results=observed,checkpoint=cp)

