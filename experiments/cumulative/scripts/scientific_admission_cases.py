"""Independent typed admission oracle and fixed zero-native request corpus."""
import copy
import hashlib
import json

sha=lambda b:hashlib.sha256(b).hexdigest()
canonical=lambda v:json.dumps(v,sort_keys=True,separators=(',',':'),ensure_ascii=False,allow_nan=False).encode()


def parse(raw):
    if len(raw)>33554432:raise ValueError('request byte cap')
    def unique(pairs):
        result={}
        for k,v in pairs:
            if k in result:raise ValueError('duplicate key')
            result[k]=v
        return result
    def bad(v):raise ValueError('nonfinite')
    return json.loads(raw,object_pairs_hook=unique,parse_constant=bad)


def check(raw,contract_raw,panel_raw,planning_raw):
    r=parse(raw);c=parse(contract_raw);p=parse(panel_raw)
    def require(v):
        if not v:raise ValueError('independent admission refusal')
    require(c['panel_sha256']==sha(panel_raw) and c['planning_contract_sha256']==sha(planning_raw))
    keys={'profile','mode','campaign_sha256','panel_sha256','unit','world','registry','registry_sha256','range','native_budget','execution_allowance','journal_byte_allowance','source','fault','launch_authorized'}
    require(type(r) is dict and set(r)==keys)
    require(r['profile']==c['profile'] and r['mode']=='check_only' and r['campaign_sha256']==sha(contract_raw) and r['panel_sha256']==sha(panel_raw))
    require(r['launch_authorized'] is False and r['source'] is None and r['fault'] is None)
    require(type(r['native_budget']) is int and r['native_budget']==2000000 and type(r['journal_byte_allowance']) is int and r['journal_byte_allowance']==32505856)
    require(type(r['execution_allowance']) is int)
    if r['unit']=='world':
        require(r['registry'] is None and r['registry_sha256'] is None and r['range'] is None)
        matches=[i for i,w in enumerate(p['worlds']) if canonical(w)==canonical(r['world'])];require(len(matches)==1)
        w=p['worlds'][matches[0]];bound=c['world_reservations'][w['arm']];require(r['execution_allowance']==bound)
        detail=dict(unit='world',panel_index=matches[0],world_id=w['world_id'],seed=w['seed'],work_bound=bound)
    elif r['unit']=='assays':
        reg=r['registry'];span=r['range'];require(r['world'] is None and type(span) is dict and set(span)=={'start','end'})
        require(type(reg) is dict and set(reg)=={'profile','assays'} and reg['profile']==c['registry_profile'] and r['registry_sha256']==sha(canonical(reg)))
        rows=reg['assays'];require(type(rows) is list and 1<=len(rows)<=78960)
        for i,row in enumerate(rows):
            require(type(row) is dict and set(row)=={'id','role','stages'} and type(row['id']) is int and row['id']==i and row['role'] in c['roles'])
            require(type(row['stages']) is list and 1<=len(row['stages'])<=8 and all(type(s) is int and -1<=s<=4 for s in row['stages']))
        start,end=span['start'],span['end'];require(type(start) is int and type(end) is int and 0<=start<end<=len(rows) and start%128==0 and end==min(start+128,len(rows)))
        bound=(end-start)*20;require(r['execution_allowance']==bound)
        detail=dict(unit='assays',registry_count=len(rows),registry_sha256=r['registry_sha256'],range=span,work_bound=bound)
    else:raise ValueError('unit')
    return dict(profile=c['profile'],request_valid=True,execution_enabled=False,launch_authorized=False,native_rewrites=0,
                campaign_sha256=sha(contract_raw),panel_sha256=sha(panel_raw),admitted=detail,request_sha256=sha(raw))


def corpus(contract_raw,panel_raw):
    c=parse(contract_raw);panel=parse(panel_raw);cases=[]
    base=dict(profile=c['profile'],mode='check_only',campaign_sha256=sha(contract_raw),panel_sha256=sha(panel_raw),unit='world',world=None,
              registry=None,registry_sha256=None,range=None,native_budget=2000000,execution_allowance=250,journal_byte_allowance=32505856,source=None,fault=None,launch_authorized=False)
    def add(name,r,valid):cases.append(dict(name=name,raw=canonical(r)+b'\n',valid=valid,mode='--check'))
    for i,w in enumerate(panel['worlds']):
        r={**base,'world':w,'execution_allowance':c['world_reservations'][w['arm']]};add(f'world_{i:03}',r,True)
        r=copy.deepcopy(r);r['world']['seed']^=1;add(f'wrong_seed_{i:03}',r,False)
    good={**base,'world':copy.deepcopy(panel['worlds'][0])}
    for field,value in [('mode','execute'),('launch_authorized',True),('native_budget',2000001),('execution_allowance',249),('execution_allowance',True),('journal_byte_allowance',32505857),('source',{}),('fault',{}),('campaign_sha256','0'*64),('panel_sha256','0'*64),('extra',None),('unit','stress')]:
        r=copy.deepcopy(good);r[field]=value;add('bad_'+field+'_'+str(len(cases)),r,False)
    for field,value in [('arm','external'),('block',True),('seed',2026101001),('ticks',15),('capital',40.0),('programs',[dict(stages=[0,1],enabled=1,mask=31)]),('extra',None)]:
        r=copy.deepcopy(good);r['world'][field]=value;add('bad_world_'+field,r,False)
    r=copy.deepcopy(good);del r['world']['world_id'];add('missing_world_id',r,False)
    cases.append(dict(name='execute_cli_refused',raw=canonical(good),valid=False,mode='--execute'))
    cases.append(dict(name='duplicate_top_key',raw=canonical(good)[:-1]+b',"launch_authorized":false}',valid=False,mode='--check'))
    raw=canonical(good).replace(b'"capital":40',b'"capital":40,"capital":40')
    cases.append(dict(name='duplicate_nested_key',raw=raw,valid=False,mode='--check'))
    cases.append(dict(name='trailing_json',raw=canonical(good)+b' {}',valid=False,mode='--check'))
    def assay(n,start=0):
        reg=dict(profile=c['registry_profile'],assays=[dict(id=i,role=c['roles'][i%13],stages=[0,1,2]) for i in range(n)])
        end=min(start+128,n)
        return {**base,'world':None,'unit':'assays','registry':reg,'registry_sha256':sha(canonical(reg)),'range':dict(start=start,end=end),'execution_allowance':20*(end-start)}
    def rehash(r):r['registry_sha256']=sha(canonical(r['registry']));return r
    for n,start in [(13,0),(129,0),(129,128),(78960,0),(78960,78848)]:add(f'assays_{n}_{start}',assay(n,start),True)
    for label,change in [
        ('late_role',lambda r:r['registry']['assays'][-1].update(role='invented')),
        ('late_id',lambda r:r['registry']['assays'][-1].update(id=0)),
        ('late_bool_stage',lambda r:r['registry']['assays'][-1].update(stages=[True])),
        ('empty_program',lambda r:r['registry']['assays'][-1].update(stages=[])),
        ('too_many_genes',lambda r:r['registry']['assays'][-1].update(stages=[0]*9)),
        ('outside_domain',lambda r:r['registry']['assays'][-1].update(stages=[5])),
        ('fractional_id',lambda r:r['registry']['assays'][0].update(id=0.0)),
        ('unknown_field',lambda r:r['registry']['assays'][-1].update(extra=1))]:
        r=assay(129);change(r);add(label,rehash(r),False)
    r=assay(78960);r['registry']['assays'][-1]['stages']=[-2];add('maximum_late_corruption',rehash(r),False)
    r=assay(78961);add('registry_over_cap',r,False)
    r=assay(129);r['range']=dict(start=1,end=129);r['execution_allowance']=2560;add('nonaligned_range',r,False)
    r=assay(129);r['range']['end']=127;r['execution_allowance']=2540;add('short_nonfinal_batch',r,False)
    r=assay(129);r['range']['start']=False;add('boolean_range',r,False)
    r=assay(129);r['registry_sha256']='0'*64;add('wrong_registry_hash',r,False)
    r=assay(13);r['registry']['assays'][0]['stages']=[-1,0,1,2,3,4];add('complete_stage_domain',rehash(r),True)
    cases.append(dict(name='encoded_byte_cap',raw=b' '*(33554432+1),valid=False,mode='--check'))
    return cases
