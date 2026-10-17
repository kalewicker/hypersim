"""Independent zero-native audit of generalized development journals and custody."""
import copy
import hashlib
import json
import sys
from pathlib import Path
from audit_readiness import require, FALSE_FLAGS
from audit_durability import read, checkpoint, assay, reservation
from audit_world import rows, registered
PROFILE='cumulative-campaign-adapter-readiness-v1'
sha=lambda b:hashlib.sha256(b).hexdigest()
load=lambda p:json.loads(p.read_text())

def registry(value,contract):
    require(set(value)=={'profile','assays'} and value['profile']==PROFILE and len(value['assays'])<=78960,'registry schema/cap')
    for i,a in enumerate(value['assays']):
        require(set(a)=={'id','role','stages'} and type(a['id']) is int and a['id']==i and a['role'] in contract['roles'],'global registry identity/role')
        require(1<=len(a['stages'])<=8 and all(type(s) is int and -1<=s<=4 for s in a['stages']),'complete global stage domain')
    return value['assays']

def mapped(request):
    result=copy.deepcopy(request);result['id']=0
    if result['role'] in ['padded_remove_later','padded_remove_both']:result['role']='intact'
    return result

def prepared(cp,contract):
    checkpoint(cp);w=cp['state']
    require(w['seed']==contract['development_seed'] and w['arm']=='endogenous' and w['tick']==15 and w['next_id']==32 and w['sites']==8 and w['parcels_per_tick']==8 and w['reservoir']==128 and w['initial_reservoir']==512 and w['initial_energy']==1024,'prepared world dimensions')
    for i,m in enumerate(w['cells']):
        s=m['state'];c=s['core'];p=s['program']
        require(p==s['baseline']=={'genes':[[j,j%4] for j in range(8)],'next_serial':8,'enabled':1,'mask':2},'fixed prepared program')
        require(s['attempts']==0 and c['rewrites']==0 and c['word_pos']=='0' and c['environment']==0,'prepared zero native history')
        require((s['alive'],m['site'],s['endowment'],s['grants'],s['initial_reservoir'],c['capital'],c['initial'],c['dissipated'],c['transferred'],c['parent'],c['generation'])==((True,i,64,3,48,55,112,33,24,None,0) if i<8 else (False,None,8,0,0,0,8,8,0,(i-8)%8,1)),'prepared per-cell ledger')
    expected=[{'parent':(i-8)%8,'child':i,'tick':1+(i-8)//8,'site':(i-8)%8,'copy_cost':11,'inherited':w['cells'][0]['state']['program'],'transmitted':w['cells'][0]['state']['program']} for i in range(8,32)]
    require(w['births']==expected,'prepared synthetic pedigree, not observed history')

def recover_values(values,head,staging,r,request_sha,contract,items,source=None):
    require(r['profile']==PROFILE and r['purpose']=='development' and r['seed']==contract['development_seed'],'development only')
    is_world=r['unit']!='assays';spec=contract['stress_spec'] if r['unit']=='stress' else r['spec']
    carried=source['lower'] if source else 0
    require(values[0]=={'kind':'header','request':r,'request_sha256':request_sha,'carried_work':carried},'exact frozen request header')
    cp=None;initial=None;world_rows=[];results=[];index=0 if is_world else r['range']['start'];work=carried;pending=None;terminal=None
    if source:
        require(not source['unknown'] and not source['staging'] and source['status']!='complete' and source['request']['source'] is None and source['head']==r['source']['head'],'safe one-hop committed source')
        for k in ['profile','purpose','unit','seed','spec','registry','range','source_identity','native_budget']:require(r[k]==source['request'][k],'continuation identity '+k)
        cp=copy.deepcopy(source['checkpoint']);initial=copy.deepcopy(source['initial']);world_rows=copy.deepcopy(source['rows']);results=copy.deepcopy(source['results']);index=source['next']
    else:require(r['source'] is None,'missing authenticated source')
    if not is_world:
        require(type(r['range']['start']) is int and type(r['range']['end']) is int and 0<=r['range']['start']<r['range']['end']<=len(items) and r['range']['end']-r['range']['start']<=128,'fixed global range')
    end=16 if r['unit']=='stress' else spec['ticks'] if is_world else r['range']['end']
    def bound():return reservation(cp['state'],spec) if is_world else 20
    for v in values[1:]:
        require(terminal is None,'no record after terminal');kind=v['kind']
        if kind=='initial':
            require(is_world and cp is None and pending is None and source is None and set(v)=={'kind','checkpoint'},'single initialization')
            cp=v['checkpoint'];checkpoint(cp)
            if r['unit']=='stress':prepared(cp,contract)
            else:registered(cp['state'],spec,r['seed'])
            initial=copy.deepcopy(cp);index=cp['state']['tick']
        elif kind=='intent':
            require(pending is None and index<end and (not is_world or cp is not None),'pending action domain')
            b=bound();require(v=={'kind':'intent','index':index,'work_before':work,'work_bound':b},'complete native intent')
            require(work+b<=r['native_budget'] and work-carried+b<=r['execution_allowance'],'pre-native funded reservation');pending=v
        elif kind=='result':
            require(pending is not None and v['index']==index,'result action identity')
            if is_world:
                require(set(v)=={'kind','index','row','checkpoint','native_rewrites','work_total'},'world result fields');checkpoint(v['checkpoint'])
                n,_=rows(cp['state'],[v['row']],v['checkpoint']['state'],spec);world_rows.append(v['row']);cp=v['checkpoint']
            else:
                require(set(v)=={'kind','index','request','native','native_rewrites','work_total'} and v['request']==items[index],'global request preserved')
                n=assay(v['native'],mapped(items[index]));results.append(v)
            require(type(v['native_rewrites']) is int and n==v['native_rewrites']<=pending['work_bound'] and v['work_total']==work+n,'actual native work counter')
            index+=1;work+=n;pending=None
        elif kind=='budget_stop':
            require(pending is None and index<end,'budget boundary');b=bound()
            require(v=={'kind':'budget_stop','index':index,'work_total':work,'reservation':b} and (work+b>r['native_budget'] or work-carried+b>r['execution_allowance']),'genuine pre-native cap stop');terminal='budget_stop'
        elif kind=='complete':
            require(pending is None and index==end and v=={'kind':'complete','work_total':work,'work_here':work-carried},'complete denominator/work');terminal='complete'
        else:raise ValueError('unknown adapter record')
    require(not staging or pending is not None,'staging requires uncertain action')
    upper=work+(pending['work_bound'] if pending else 0)
    require(upper<=r['native_budget'] and upper-carried<=r['execution_allowance'],'aggregate bounds')
    return {'request':r,'head':head,'staging':staging,'unknown':pending is not None,'status':terminal or 'partial','next':index,'lower':work,'upper':upper,'here_lower':work-carried,'here_upper':upper-carried,'checkpoint':cp,'initial':initial,'rows':world_rows,'results':results}

def recover(directory,r,request_sha,contract,items,source=None):
    values,head,staging=read(directory)
    require(sum(p.stat().st_size for p in directory.iterdir())<=r['journal_byte_allowance'],'journal allowance')
    return recover_values(values,head,staging,r,request_sha,contract,items,source)

def audit(package,sealed=False):
    contract=load(package/'contract.json');require(contract['profile']==PROFILE,'adapter contract')
    for f in FALSE_FLAGS:require(contract[f] is False,'no scientific admission')
    if sealed:
        seal=load(package/'output_sha256.json');actual={str(p.relative_to(package)) for p in package.rglob('*') if p.is_file() and p!=package/'output_sha256.json'}
        require(actual==set(seal) and all(not (package/p).is_symlink() and sha((package/p).read_bytes())==h for p,h in seal.items()),'immutable full archive seal')
    registry_pins=load(package/'registry_sha256.json')
    require(all(sha((package/'registry_snapshots'/p).read_bytes())==h for p,h in registry_pins.items()),'locked registry source closure')
    pins=load(package/'source_sha256.json')
    require(all(sha((package/'source_snapshots'/p).read_bytes())==h for p,h in pins.items()),'frozen full source closure')
    identity={'binary_sha256':sha((package/'cumulative-campaign').read_bytes()),'source_sha256':sha((package/'source_sha256.json').read_bytes())}
    raw=(package/'registry.json').read_bytes();items=registry(json.loads(raw),contract)
    expected=[{'id':i,'role':role,'stages':stages} for i,(role,stages) in enumerate(zip(contract['roles'],contract['assay_control_programs']))]
    expected += [{'id':13+i,'role':contract['roles'][i%13],'stages':[0,1,2,3,0,1,2,3]} for i in range(contract['stress_assays'])]
    require(items==expected,'fixed control and maximum-gene batch panel')
    routes=load(package/'routes.json');require(routes==contract['routes'],'fixed complete route denominator');observed={};charged=load(package/'test_work.json')['native_rewrites'];lower=charged;timing=sum(load(p)['seconds'] for p in (package/'commands').glob('rust_*.json'))
    require(charged==1765,'unchanged 38 native regression groups')
    for route in routes:
        name=route['name'];r=load(package/'inputs'/(name+'.json'));cmd=load(package/'commands'/(name+'.json'));timing+=cmd['seconds']
        require(r['execution_allowance']<=6000-charged,'aggregate funding')
        wanted_identity=identity.copy()
        if route.get('rejection')=='source_identity':wanted_identity['source_sha256']='0'*64
        require(r['source_identity']==wanted_identity,'frozen binary/source or declared identity rejection')
        if route.get('rejection'):
            expected_error={'pending_source':'unsafe unpaired intent','last_row_invalid':'complete global assay denominator','registry_hash':'registry identity','scientific_seed':'development request identity/domain/cap','source_identity':'continuation changed source_identity'}[route['rejection']]
            require(expected_error in json.loads(cmd['stderr'])['error'],'declared rejection reason')
            require(cmd['exit_code']==1 and json.loads(cmd['stderr'])['native_rewrites']==0 and not (package/'journals'/name).exists(),'preflight refusal before any native action')
            continue
        require(r['unit']==route['unit'] and r['fault']==route.get('fault') and r['range']==route.get('range'),'declared unit/fault/range')
        if r['unit']=='assays':require(r['registry']['sha256']==sha(raw),'whole global registry identity')
        if r['unit']=='world':
            wanted=copy.deepcopy(contract['world_spec']);wanted['arm']=route.get('arm','endogenous')
            if wanted['arm'] in ['external','frozen']:
                for program in wanted['programs']:program.update(enabled=0,mask=0)
            require(r['spec']==wanted,'matched declared world inputs')
        source=observed.get(route.get('source'));result=recover(package/'journals'/name,r,sha((package/'inputs'/(name+'.json')).read_bytes()),contract,items,source)
        require(result['status']==route['status'] and result['unknown']==route.get('unknown',False),'declared route status')
        observed[name]=result;charged+=result['here_upper'];lower+=result['here_lower']
        require(cmd['exit_code']==(1 if r['fault'] else 0),'declared native exit')
    require(charged<=6000 and timing<=300,'aggregate development work and command-time limits')
    reference=observed['world_endogenous'];continued=observed['world_continue']
    require(reference['rows']==continued['rows'] and reference['checkpoint']==continued['checkpoint'] and reference['lower']==continued['lower'],'byte-exact committed world continuation')
    a=observed['assays_0']['results']+observed['assays_1']['results']
    require([r['index'] for r in a]==list(range(142)),'global batch complete denominator')
    require([r['native'] for r in observed['assay_continue']['results']]==[r['native'] for r in a[:13]],'exact assay continuation')
    scores={r['request']['role']:r['native']['results'][0]['score'] for r in a[:13]}
    require(list(scores.values())==[0,7,7,22,22,-5,7,-10,-5,22,22,2,-10],'authored thirteen-role chemistry control')
    require(all(r['native']['results'][0]['status']=='complete' for r in a[:13]),'funded control')
    stress=[observed[f'stress_{i}'] for i in range(8)]
    require(all(s['here_lower']==48 and s['checkpoint']==stress[0]['checkpoint'] and s['rows']==stress[0]['rows'] for s in stress),'eight fixed identical maximum-dimension ticks')
    max_world=max((p.stat().st_size for route in routes if route['name'] in observed and observed[route['name']]['request']['unit']!='assays' for p in (package/'journals'/route['name']).iterdir()))
    max_assay=max(p.stat().st_size for name in ['assays_0','assays_1'] for p in (package/'journals'/name).iterdir())
    world_bytes=188*(2*16+4)*max_world;assay_bytes=78960*2*max_assay+1234*max_assay
    world_time=max(load(package/'commands'/f'stress_{i}.json')['seconds'] for i in range(8))*188*16
    assay_time=max(load(package/'commands'/f'assays_{i}.json')['seconds']/(128 if i==0 else 14) for i in range(2))*78960
    return {'profile':PROFILE,'engineering_adapter_pass':True,'routes':len(routes),'native_work_lower':lower,'native_work_upper':charged,'native_work_unknown':charged!=lower,'global_assays':142,'padded_control_later_gain_with_earlier':scores['intact']-scores['padded_remove_later'],'padded_control_later_gain_without_earlier':scores['padded_remove_earlier']-scores['padded_remove_both'],'stress_ticks':8,'stress_native_rewrites':384,'maximum_observed_world_record_bytes':max_world,'maximum_observed_assay_record_bytes':max_assay,'conservative_observed_size_projection_bytes':world_bytes+assay_bytes,'observed_time_projection_seconds':world_time+assay_time,'projection_is_certified_upper_bound':False,'all_worlds_development_only':True,'complete_campaign_measurement_adapter':False,'campaign_preregistration_complete':False,**{f:False for f in FALSE_FLAGS}}

if __name__=='__main__':
    print(json.dumps(audit(Path(sys.argv[1]),'--sealed' in sys.argv),indent=2,sort_keys=True))
