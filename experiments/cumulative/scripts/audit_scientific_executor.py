"""Independent executor journal verification; never advances native worlds."""
import copy
import json
from pathlib import Path
import sys
from audit_readiness import require
from audit_durability import read, checkpoint, assay, reservation
from audit_world import rows, registered
from scientific_admission_cases import parse, check, sha


def recover(directory, route, contract_raw, binary_hash):
    c=parse(contract_raw);values,head,staging=read(directory)
    require(sum(p.stat().st_size for p in directory.iterdir())<=c['journal_byte_allowance'],'writer allowance')
    require(route in c['routes'] and route['seed']==2026101001,'fixed development route')
    require(values[0]==dict(kind='header',profile=c['profile'],purpose='development',contract_sha256=sha(contract_raw),route=route,binary_sha256=binary_hash),'executor header identity')
    world=route['unit']=='world';spec=route['spec'];items=route['items']
    end=spec['ticks'] if world else len(items)
    cp=None;work=0;index=0;pending=None;complete=False;observed=[]
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


def audit(out):
    cr=(out/'contract.json').read_bytes();c=parse(cr);binary=sha((out/'cumulative-scientific-executor').read_bytes())
    require(all(c[k] is False for k in ['campaign_launch_authorized','budget_approved','readiness_gate_complete','scientific_acceptance','evolution_demonstrated']),'science remains unlaunched')
    pins=parse((out/'source_sha256.json').read_bytes())
    require(all(sha((out/'source_snapshots'/n).read_bytes())==h for n,h in pins.items()),'source snapshots')
    require(cr==(out/'source_snapshots/experiments/cumulative/scientific_executor_contract.json').read_bytes(),'frozen executor contract')
    package=out/'source_snapshots/experiments/cumulative'
    boundary=parse((out/'boundary_cases.json').read_bytes());valid=0
    for case in boundary:
        raw=(out/'requests'/case['file']).read_bytes();receipt=parse((out/'commands'/case['name']).read_bytes())
        if case['mode']=='--check':
            wanted=check(raw,(package/'scientific_admission_contract.json').read_bytes(),(package/'scientific_panel.json').read_bytes(),(package/'campaign_planning_contract.json').read_bytes())
            require(receipt['exit_code']==0 and parse(receipt['stdout'].encode())==wanted,'shared independent admission')
            valid+=1
        else:
            require(receipt['exit_code']==1,'execution refusal')
            refusal=parse(receipt['stderr'].encode());require(refusal['native_rewrites']==0 and refusal['campaign_launch_authorized'] is False,'refusal before native work')
    require(not (out/'forbidden-journal').exists(),'no journal on scientific execution attempts')
    lower=upper=0;results={}
    sealraw=(out/'reference_seal.json').read_bytes();require(sha(sealraw)=='827890af64baf8cf8c25d67056a5006294c6c95fcddd888d03870c6d6287f794','known adapter reference seal')
    refpins=parse(sealraw)
    for p in (out/'reference').rglob('*'):
        if p.is_file():require(sha(p.read_bytes())==refpins[str(p.relative_to(out/'reference'))],'authenticated historical reference')
    for route in c['routes']:
        name=route['name'];r=recover(out/'journals'/name,route,cr,binary);cmd=parse((out/'commands'/(name+'.json')).read_bytes())
        require(not r['staging'],'no unaccounted staging')
        expected='unknown' if route['fault'] else 'complete'
        require(r['status']==expected and cmd['exit_code']==(1 if route['fault'] else 0),'fixed route outcome')
        native=parse((cmd['stderr'] if route['fault'] else cmd['stdout']).encode())
        require(native['native_rewrites']==r['lower'],'command/journal native work')
        lower+=r['lower'];upper+=r['upper'];results[name]=r
        if route['unit']=='world' and not route['fault']:
            ref,_,_=read(out/'reference/journals'/('world_'+name))
            require([v for v in ref if v['kind']=='result']==r['results'],'exact prior development transitions and checkpoints')
    require([v['native']['results'][0]['score'] for v in results['assays']['results']]==[0,7,7,22,22,-5,7,-10,-5,22,22,2,-10],'all thirteen authored chemistry controls')
    require(upper<=c['limits']['native_rewrites'],'aggregate conservative budget')
    return dict(engineering_executor_pass=True,development_routes=len(c['routes']),admission_requests_checked=valid,execution_refusals=len(boundary)-valid,native_work_lower=lower,native_work_upper=upper,uncertain_route_retained=True,scientific_worlds_started=0,campaign_controller_integration_complete=False,campaign_launch_authorized=False,readiness_gate_complete=False,scientific_acceptance=False,evolution_demonstrated=False)


if __name__=='__main__':print(json.dumps(audit(Path(sys.argv[1])),indent=2,sort_keys=True))
