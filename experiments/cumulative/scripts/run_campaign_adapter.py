"""Freeze and execute one finite development adapter panel; never retry a native route."""
import copy,json,os,re,shutil,subprocess,sys,time
from pathlib import Path
from run_readiness import ROOT,PACKAGE,FALSE,digest,write,source_paths
from run_durability import custody as older_custody
from audit_campaign_adapter import recover
PARENT=ROOT/'output/cumulative_campaign_planning_v2_20261009'
ROOTS={
'cumulative_durable_journal_readiness_v3_20261009':'8ae372d4e9f71cb34441453d2333dd524b0431b025028f20c7226c19f1709fe1',
'cumulative_campaign_planning_20261009':'bd30202bf798fc8f3416865a3b6288dea2c819dd955faac064cbda44b2f66b35',
'cumulative_campaign_planning_v2_20261009':'a82f42baf4d6f12ee3c7f576b6a47ca1d208df5a3f386de74ca185bc172a9dc9'}
def custody():
    older_custody()
    for name,root in ROOTS.items():
        p=ROOT/'output'/name
        if digest(p/'output_sha256.json')!=root:raise ValueError('historical root '+name)
        seal=json.loads((p/'output_sha256.json').read_text());actual={str(f.relative_to(p)) for f in p.rglob('*') if f.is_file() and f!=p/'output_sha256.json'}
        if actual!=set(seal) or any(digest(p/rel)!=h for rel,h in seal.items()):raise ValueError('historical custody '+name)
    pins=json.loads((PARENT/'source_sha256.json').read_text());export='experiments/cumulative/src/population/regulation/world/journal.rs'
    for rel,h in pins.items():
        if rel==export:
            if (ROOT/rel).read_bytes().replace(b'pub mod campaign;\n',b'',1)!=(PARENT/'source_snapshots'/rel).read_bytes():raise ValueError('old journal body changed')
        elif digest(ROOT/rel)!=h:raise ValueError('previous source changed '+rel)

def run(out):
    start=time.monotonic();out.mkdir(parents=True,exist_ok=False)
    for d in ['inputs','commands','journals','recovery']: (out/d).mkdir()
    c=json.loads((PACKAGE/'campaign_adapter_contract.json').read_text());limits=c['limits'];charged=0;lower=0;native_seconds=0;observed={}
    manifest={'profile':c['profile'],'status':'running','engineering_adapter_pass':False,'native_work_unknown':True,**FALSE};write(out/'manifest.json',manifest)
    def command(name,args,native=False,allow_failure=False):
        nonlocal native_seconds
        remain=limits['workflow_seconds']-(time.monotonic()-start)
        if native:remain=min(remain,limits['native_seconds']-native_seconds)
        if remain<=0:raise ValueError('aggregate deadline')
        before=time.monotonic()
        try:r=subprocess.run(args,cwd=ROOT,capture_output=True,text=True,timeout=remain,env={**os.environ,'PYTHONDONTWRITEBYTECODE':'1'})
        except subprocess.TimeoutExpired as e:
            write(out/'commands'/(name+'.json'),{'argv':args,'timed_out':True,'stdout':str(e.stdout or ''),'stderr':str(e.stderr or '')});raise
        elapsed=time.monotonic()-before
        if native:native_seconds+=elapsed
        write(out/'commands'/(name+'.json'),{'argv':args,'exit_code':r.returncode,'seconds':elapsed,'native_command':native,'stdout':r.stdout,'stderr':r.stderr});print(json.dumps({'phase':name,'exit_code':r.returncode}),flush=True)
        if r.returncode and not allow_failure:raise ValueError(name+': '+r.stderr[-1800:])
        return r
    try:
        custody();pins={str(p.relative_to(ROOT)):digest(p) for p in source_paths()};write(out/'source_sha256.json',pins)
        for rel in pins:
            dest=out/'source_snapshots'/rel;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(ROOT/rel,dest)
        write(out/'contract.json',c);write(out/'routes.json',c['routes'])
        # The Cargo lock is unchanged; preserve the already authenticated SHA source closure.
        old=ROOT/'output/cumulative_durable_journal_readiness_v3_20261009'
        for name in ['registry_sha256.json','registry_origin.json']:shutil.copy2(old/name,out/name)
        shutil.copytree(old/'registry_snapshots',out/'registry_snapshots')
        command('format',['cargo','fmt','--manifest-path',str(PACKAGE/'Cargo.toml'),'--check'])
        for name,count in [('readiness',7),('population',6),('lineage',5),('regulation',6),('integration',6),('durability',8)]:
            r=command('rust_'+name,['cargo','test','--offline','--locked','--manifest-path',str(PACKAGE/'Cargo.toml'),'--test',name,'--','--nocapture','--test-threads=1'],native=True)
            counters=[int(n) for n in re.findall(r'native_test_rewrites=(\d+)',r.stdout)]
            if len(counters)!=count:raise ValueError('missing test counters')
            charged+=sum(counters)
        if charged!=1765:raise ValueError('unchanged native regression ledger')
        lower=charged;write(out/'test_work.json',{'rust_test_groups':38,'native_rewrites':charged})
        command('build',['cargo','build','--offline','--locked','--manifest-path',str(PACKAGE/'Cargo.toml'),'--bin','cumulative-campaign'])
        binary=out/'cumulative-campaign';shutil.copy2(PACKAGE/'target/debug/cumulative-campaign',binary)
        identity={'binary_sha256':digest(binary),'source_sha256':digest(out/'source_sha256.json')}
        items=[{'id':i,'role':role,'stages':stages} for i,(role,stages) in enumerate(zip(c['roles'],c['assay_control_programs']))]
        items += [{'id':13+i,'role':c['roles'][i%13],'stages':[0,1,2,3,0,1,2,3]} for i in range(c['stress_assays'])]
        reg={'profile':c['profile'],'assays':items};write(out/'registry.json',reg)
        invalid=copy.deepcopy(reg);invalid['assays'][-1]['stages']=[5];write(out/'invalid_registry.json',invalid)
        write(out/'request_freeze.json',{'source_identity':identity,'registry_sha256':digest(out/'registry.json'),'contract_sha256':digest(out/'contract.json'),'before_native_routes':True,**FALSE})
        for route in c['routes']:
            name=route['name'];source=observed.get(route.get('source'))
            if source is not None:
                source_r=json.loads((out/'inputs'/(route['source']+'.json')).read_text())
                source=recover(out/'journals'/route['source'],source_r,digest(out/'inputs'/(route['source']+'.json')),c,items)
                if not route.get('rejection') and (source['unknown'] or source['staging']):raise ValueError('unsafe source before continuation dispatch')
            size=sum(p.stat().st_size for p in out.rglob('*') if p.is_file());spec=copy.deepcopy(c['world_spec']) if route['unit']=='world' else None
            if spec:
                spec['arm']=route.get('arm','endogenous')
                if spec['arm'] in ['external','frozen']:
                    for program in spec['programs']:program.update(enabled=0,mask=0)
            r={'profile':c['profile'],'purpose':'development','unit':route['unit'],'seed':c['development_seed'],'spec':spec,'registry':{'path':str(out/'registry.json'),'sha256':digest(out/'registry.json')} if route['unit']=='assays' else None,'range':route.get('range'),'source_identity':identity.copy(),'native_budget':6000,'execution_allowance':min(route.get('execution_allowance',6000),6000-charged),'journal_byte_allowance':min(33554432,limits['output_bytes']-size-2097152),'fault':route.get('fault'),'source':{'directory':str(out/'journals'/route['source']),'head':source['head']} if source else None}
            reject=route.get('rejection')
            if reject=='last_row_invalid':r['registry']={'path':str(out/'invalid_registry.json'),'sha256':digest(out/'invalid_registry.json')}
            if reject=='registry_hash':r['registry']['sha256']='0'*64
            if reject=='scientific_seed':r['seed']=c['rejected_scientific_seed']
            if reject=='source_identity':r['source_identity']['source_sha256']='0'*64
            if r['journal_byte_allowance']<=0 or r['execution_allowance']<0:raise ValueError('remaining development cap')
            inp=out/'inputs'/(name+'.json');write(inp,r)
            result=command(name,[str(binary),str(inp),str(out/'journals'/name)],native=True,allow_failure=bool(reject or r['fault']))
            if reject:
                if result.returncode!=1 or json.loads(result.stderr)['native_rewrites']!=0 or (out/'journals'/name).exists():raise ValueError('preflight refusal failed')
                report={'status':'rejected_before_native','native_rewrites':0}
            else:
                result=recover(out/'journals'/name,r,digest(inp),c,items,source);observed[name]=result;charged+=result['here_upper'];lower+=result['here_lower']
                report={k:result[k] for k in ['status','head','unknown','next','lower','upper','here_lower','here_upper']}
            write(out/'recovery'/(name+'.json'),report)
            manifest.update(native_work_lower=lower,native_work_upper=charged,native_work_unknown=lower!=charged,last_route=name);write(out/'manifest.json',manifest)
        frozen=out/'source_snapshots/experiments/cumulative/scripts'
        result=command('independent_audit',[sys.executable,'-B',str(frozen/'audit_campaign_adapter.py'),str(out)]);audit=json.loads(result.stdout);write(out/'audit.json',audit)
        command('adapter_corruption_tests',[sys.executable,'-B',str(frozen/'test_campaign_adapter.py'),str(out)])
        command('prior_durability_audit',[sys.executable,'-B',str(frozen/'audit_durability.py'),str(old)])
        command('prior_planning_rules',[sys.executable,'-B',str(frozen/'test_campaign_planning.py'),str(PARENT)])
        if any(digest(ROOT/p)!=h for p,h in pins.items()):raise ValueError('source changed during panel')
        custody();write(out/'preservation.json',{'all_eleven_prior_roots_unchanged':True,'all_parent_source_bodies_unchanged_except_additive_module_export':True,'all_pinned_sources_unchanged_during_run':True})
        manifest.update(audit,status='complete',native_seconds=native_seconds,elapsed_seconds=time.monotonic()-start,source_files=len(pins),rust_test_groups=38,python_corruption_checks=11,complete_campaign_measurement_adapter=False,whole_campaign_feasibility_pass=False);write(out/'manifest.json',manifest)
        size=sum(p.stat().st_size for p in out.rglob('*') if p.is_file())
        if size>limits['output_bytes']:raise ValueError('aggregate output cap')
    except Exception as e:
        manifest.update(status='failed',engineering_adapter_pass=False,error=str(e),native_work_lower=lower,native_work_upper=charged,native_work_unknown=True,elapsed_seconds=time.monotonic()-start);write(out/'manifest.json',manifest)
        write(out/'output_sha256.json',{str(p.relative_to(out)):digest(p) for p in sorted(out.rglob('*')) if p.is_file() and p!=out/'output_sha256.json'});raise
    write(out/'output_sha256.json',{str(p.relative_to(out)):digest(p) for p in sorted(out.rglob('*')) if p.is_file()})
    print(json.dumps({'output':str(out),'seal_sha256':digest(out/'output_sha256.json'),'payload_bytes':size,**audit},indent=2))
if __name__=='__main__':run(Path(sys.argv[1]).resolve())
