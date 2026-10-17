"""Publish the fixed durable-journal engineering panel; preserve partial/unsafe routes."""
import json,os,re,shutil,subprocess,sys,time,tomllib,tarfile
from pathlib import Path
from run_readiness import ROOT,PACKAGE,FALSE,digest,write,source_paths
from run_integration import preservation,HISTORICAL,PARENT as EDITOR,PARENT_ROOT as EDITOR_ROOT
from preservation_helpers import dependency_addition,strip_journal_main
from audit_durability import recover,summary,sha
PARENT=ROOT/'output/cumulative_regulated_world_lineage_readiness_20261009'
PARENT_ROOT='ea1921fb79c074683e699555536c904ad010799f90d88f86de191212f3d606b8'
FAILED=ROOT/'output/cumulative_durable_journal_readiness_20261009'
FAILED_ROOT='3f135cabdd1fa18fdb72a5b757ff26616ef9e1c91fc108293a23555385297786'
FAILED2=ROOT/'output/cumulative_durable_journal_readiness_v2_20261009'
FAILED2_ROOT='89cd6ffa5b241e0025a96b9719f8cc4698d2ebc242db74e63edde868f3550fcf'
def custody():
    preservation()
    for failed,root in [(FAILED,FAILED_ROOT),(FAILED2,FAILED2_ROOT)]:
        if digest(failed/'output_sha256.json')!=root:raise ValueError('failed dependency-inventory root')
        seal=json.loads((failed/'output_sha256.json').read_text());actual={str(p.relative_to(failed)) for p in failed.rglob('*') if p.is_file() and p!=failed/'output_sha256.json'}
        if actual!=set(seal) or any(digest(failed/rel)!=h for rel,h in seal.items()) or json.loads((failed/'manifest.json').read_text())['status']!='failed':raise ValueError('failed dependency-inventory custody/status')
    if digest(PARENT/'output_sha256.json')!=PARENT_ROOT:raise ValueError('integrated parent root')
    seal=json.loads((PARENT/'output_sha256.json').read_text());actual={str(p.relative_to(PARENT)) for p in PARENT.rglob('*') if p.is_file() and p!=PARENT/'output_sha256.json'}
    if actual!=set(seal) or any(digest(PARENT/rel)!=h for rel,h in seal.items()):raise ValueError('integrated parent full custody')
    m=json.loads((PARENT/'manifest.json').read_text())
    if m['status']!='complete' or not m['engineering_fixture_pass'] or any(m[f] is not False for f in FALSE):raise ValueError('integrated parent admission')
def guard():
    custody();pins=json.loads((PARENT/'source_sha256.json').read_text())
    allowed={'experiments/cumulative/'+rel for rel in ['README.md','Cargo.toml','Cargo.lock','src/main.rs','src/population/regulation/world.rs','scripts/run_population.py','scripts/run_lineage.py','scripts/run_regulation.py','scripts/run_integration.py']}
    changed=[rel for rel,h in pins.items() if digest(ROOT/rel)!=h]
    if not set(changed)<=allowed:raise ValueError('unexpected pinned source change '+str(changed))
    old=PARENT/'source_snapshots/experiments/cumulative'
    if (PACKAGE/'src/population/regulation/world.rs').read_bytes().replace(b'pub mod journal;\n',b'',1)!=(old/'src/population/regulation/world.rs').read_bytes():raise ValueError('original world body changed')
    if strip_journal_main((PACKAGE/'src/main.rs').read_bytes())!=(old/'src/main.rs').read_bytes():raise ValueError('original native dispatch changed')
    return changed,dependency_addition(PACKAGE,PARENT)
def run(out):
    start=time.monotonic();out.mkdir(parents=True,exist_ok=False)
    for name in ['inputs','commands','journals','recovery']: (out/name).mkdir()
    contract=json.loads((PACKAGE/'durability_contract.json').read_text());limits=contract['limits'];test_work=0;native_seconds=0;charged=0;recovered={}
    manifest={'profile':contract['profile'],'scope':contract['scope'],'status':'running','engineering_fixture_pass':False,'native_work_unknown':True,**FALSE};write(out/'manifest.json',manifest)
    def command(name,argv,native=False,allow_failure=False):
        nonlocal native_seconds
        remaining=limits['build_test_audit_seconds']-(time.monotonic()-start)
        if native:remaining=min(remaining,limits['native_seconds']-native_seconds)
        if remaining<=0:raise RuntimeError('aggregate workflow/native deadline')
        before=time.monotonic()
        try:r=subprocess.run(argv,cwd=ROOT,capture_output=True,text=True,timeout=remaining,env={**os.environ,'PYTHONDONTWRITEBYTECODE':'1'})
        except subprocess.TimeoutExpired as e:
            write(out/'commands'/(name+'.json'),{'argv':argv,'timed_out':True,'stdout':str(e.stdout or ''),'stderr':str(e.stderr or '')});raise
        elapsed=time.monotonic()-before
        if native:native_seconds+=elapsed
        write(out/'commands'/(name+'.json'),{'argv':argv,'exit_code':r.returncode,'seconds':elapsed,'stdout':r.stdout,'stderr':r.stderr})
        print(json.dumps({'phase':name,'exit_code':r.returncode}),flush=True)
        if r.returncode and not allow_failure:raise RuntimeError(name+' failed: '+r.stderr[-2000:])
        return r
    try:
        changed,added=guard();pins={str(p.relative_to(ROOT)):digest(p) for p in source_paths()};write(out/'source_sha256.json',pins)
        for rel in pins:
            dest=out/'source_snapshots'/rel;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(ROOT/rel,dest)
        write(out/'contract.json',contract);shutil.copy2(PACKAGE/'integration_contract.json',out/'world_contract.json');shutil.copy2(PARENT/'requests.json',out/'assay_requests.json');shutil.copy2(PARENT/'measurement.json',out/'measurement.json')
        encode=lambda v:json.dumps(v,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        parent_cases=json.loads((PARENT/'fixture.json').read_text())['cases'];parent_assays=json.loads((PARENT/'assays.json').read_text())['results']
        write(out/'parent_equivalence.json',{'parent_seal_sha256':PARENT_ROOT,'world_cases_sha256':sha(encode(parent_cases)),'measurement_sha256':sha(encode(json.loads((out/'measurement.json').read_text()))),'assay_results_sha256':sha(encode(parent_assays))});del parent_cases,parent_assays
        write(out/'historical_roots.json',{'roots':HISTORICAL,'editor':EDITOR_ROOT,'integrated':PARENT_ROOT,'failed_dependency_inventory':[FAILED_ROOT,FAILED2_ROOT],'allowed_source_changes':changed,'added_sha_dependencies':added,'original_cell_population_lineage_editor_and_world_bodies_unchanged':True})
        # Freeze the newly introduced registry source closure as well as its locked checksums.
        command('cargo_metadata',['cargo','metadata','--offline','--locked','--no-deps','--manifest-path',str(PACKAGE/'Cargo.toml'),'--format-version','1'])
        registry={};registry_paths={};registry_origin={}
        cache=Path(os.environ.get('CARGO_HOME',str(Path.home()/'.cargo')))/'registry/src'
        locked=tomllib.loads((PACKAGE/'Cargo.lock').read_text())['package']
        for p in locked:
            if p['name'] not in added:continue
            candidates=list(cache.glob('*/'+p['name']+'-'+p['version']))
            if len(candidates)!=1:raise ValueError('unique cached locked SHA dependency source '+p['name'])
            directory=candidates[0];archive=cache.parent/'cache'/directory.parent.name/(directory.name+'.crate')
            if digest(archive)!=p['checksum']:raise ValueError('cached registry package checksum '+p['name'])
            with tarfile.open(archive) as package_archive:
                for member in package_archive.getmembers():
                    if not member.isfile():continue
                    rel=Path(member.name).relative_to(directory.name)
                    if (directory/rel).read_bytes()!=package_archive.extractfile(member).read():raise ValueError('registry source differs from authenticated package '+p['name']+'/'+str(rel))
            destination=out/'registry_packages'/archive.name;destination.parent.mkdir(exist_ok=True);shutil.copy2(archive,destination)
            registry_origin[p['name']]={'version':p['version'],'source':p['source'],'package_checksum':p['checksum']}
            for file in directory.rglob('*'):
                if file.is_file():
                    rel=p['name']+'-'+p['version']+'/'+str(file.relative_to(directory));registry[rel]=digest(file);registry_paths[rel]=file
                    dest=out/'registry_snapshots'/rel;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(file,dest)
        write(out/'registry_origin.json',registry_origin)
        write(out/'registry_sha256.json',registry)
        command('format',['cargo','fmt','--manifest-path',str(PACKAGE/'Cargo.toml'),'--check'])
        for name,count in [('readiness',7),('population',6),('lineage',5),('regulation',6),('integration',6),('durability',8)]:
            r=command('rust_'+name,['cargo','test','--offline','--locked','--manifest-path',str(PACKAGE/'Cargo.toml'),'--test',name,'--','--nocapture','--test-threads=1'],native=True)
            counters=[int(x) for x in re.findall(r'native_test_rewrites=(\d+)',r.stdout)]
            if len(counters)!=count:raise ValueError('complete native test accounting '+name)
            test_work+=sum(counters)
            if test_work>limits['native_rewrites']:raise ValueError('native test cap')
        write(out/'test_work.json',{'native_rewrites':test_work,'rust_test_groups':38});charged=test_work
        command('build',['cargo','build','--offline','--locked','--manifest-path',str(PACKAGE/'Cargo.toml'),'--bin','cumulative-readiness']);binary=out/'cumulative-readiness';shutil.copy2(PACKAGE/'target/debug/cumulative-readiness',binary)
        identity={'binary_sha256':digest(binary),'source_sha256':digest(out/'source_sha256.json')};write(out/'identity.json',identity)
        world=json.loads((out/'world_contract.json').read_text());assays=json.loads((out/'assay_requests.json').read_text())
        write(out/'request_freeze.json',{'world_contract_sha256':digest(out/'world_contract.json'),'assay_requests_sha256':digest(out/'assay_requests.json'),'parent_equivalence_sha256':digest(out/'parent_equivalence.json'),'before_all_native_routes':True,'source_is_development':True,'outcomes_feed_back':False,**FALSE})
        for route in contract['routes']:
            name=route['name'];source=recovered.get(route.get('source'))
            # An independently verified source is required before every continuation dispatch.
            if source is not None:
                raw=(out/'inputs'/(route['source']+'.json')).read_bytes();source=recover(out/'journals'/route['source'],json.loads(raw),sha(raw));recovered[route['source']]=source
            size=sum(p.stat().st_size for p in out.rglob('*') if p.is_file())
            r={'profile':contract['profile'],'mode':route['mode'],'world_contract':world,'assay_requests':assays,'source_identity':identity,'native_budget':route.get('native_budget',6000),'execution_allowance':limits['native_rewrites']-charged,'journal_byte_allowance':min(limits['journal_bytes'],limits['output_bytes']-size-1048576),'fault':route.get('fault'),'source_dir':str(out/'journals'/route['source']) if source else None,'source_head_sha256':source['head'] if source else None,'replay':{'case':route['case'],'start_tick':route['start_tick']} if route['mode']=='world_replay' else None}
            input_path=out/'inputs'/(name+'.json');write(input_path,r)
            result=command(name,[str(binary),'journal-run',str(input_path),str(out/'journals'/name)],native=True,allow_failure=bool(route.get('fault') or route.get('expect_rejection')))
            if route.get('expect_rejection'):
                if result.returncode==0 or (out/'journals'/name).exists():raise ValueError('unsafe source was not refused before work')
                report={'status':'rejected_unsafe_source','native_work_here_lower':0,'native_work_here_upper':0,'native_work_unknown':False,**FALSE}
            else:
                restored=recover(out/'journals'/name,r,digest(input_path),source);recovered[name]=restored;report=summary(restored);charged+=restored['here_upper']
                if route.get('fault') and result.returncode==0:raise ValueError('injected interruption not retained')
            write(out/'recovery'/(name+'.json'),report)
        frozen=out/'source_snapshots/experiments/cumulative/scripts'
        command('independent_audit',[sys.executable,'-B',str(frozen/'audit_durability.py'),str(out),str(out/'audit.json')]);audit=json.loads((out/'audit.json').read_text())
        command('durability_auditor_tests',[sys.executable,'-B',str(frozen/'test_durability_audit.py'),str(out)])
        command('version_auditor_regression',[sys.executable,'-B',str(frozen/'test_versions_audit.py'),str(PARENT)])
        command('regulation_auditor_regression',[sys.executable,'-B',str(frozen/'test_regulation_audit.py'),str(EDITOR/'fixture.json'),str(EDITOR/'contract.json')])
        for name,script in [('capacity','test_audit.py'),('shared','test_population_audit.py')]:
            old=ROOT/'output'/HISTORICAL[name][0];command(name+'_auditor_regression',[sys.executable,'-B',str(frozen/script),str(old/'fixture.json'),str(old/'contract.json')])
        old=ROOT/'output'/HISTORICAL['lineage'][0];command('lineage_auditor_regression',[sys.executable,'-B',str(frozen/'test_lineage_audit.py'),str(old)])
        if {rel:digest(ROOT/rel) for rel in pins}!=pins or {rel:digest(p) for rel,p in registry_paths.items()}!=registry:raise ValueError('source changed during execution')
        custody();write(out/'preservation.json',{'all_six_historical_roots_unchanged':True,'failed_dependency_inventory_root_unchanged':True,'all_pinned_sources_unchanged':True,'source_files':len(pins),'added_registry_source_files':len(registry)})
        manifest.update(status='complete',engineering_fixture_pass=True,binary_sha256=digest(binary),source_files=len(pins),native_work_unknown=audit['native_work_unknown'],native_rewrites_lower_bound=audit['native_rewrites_lower_bound'],native_rewrites_upper_bound=audit['native_rewrites_upper_bound'],test_native_rewrites=test_work,native_seconds=native_seconds,elapsed_seconds=time.monotonic()-start,rust_test_groups=38,python_test_groups=54,samples=96,assays=33,durable_bounded_routes_engineering_complete=True,durable_campaign_routes_complete=False,independent_rng_replay=False,independent_allocator_replay=False)
        write(out/'manifest.json',manifest);size=sum(p.stat().st_size for p in out.rglob('*') if p.is_file())
        if size>limits['output_bytes']:raise ValueError('aggregate output cap')
    except Exception as e:
        manifest.update(status='failed',engineering_fixture_pass=False,error=str(e),test_native_rewrites=test_work,elapsed_seconds=time.monotonic()-start);write(out/'manifest.json',manifest)
        write(out/'output_sha256.json',{str(p.relative_to(out)):digest(p) for p in sorted(out.rglob('*')) if p.is_file() and p!=out/'output_sha256.json'});raise
    seal={str(p.relative_to(out)):digest(p) for p in sorted(out.rglob('*')) if p.is_file()};write(out/'output_sha256.json',seal)
    print(json.dumps({'output':str(out),'seal_sha256':digest(out/'output_sha256.json'),'sealed_entries':len(seal),'source_files':len(pins),'native_rewrites_lower_bound':audit['native_rewrites_lower_bound'],'native_rewrites_upper_bound':audit['native_rewrites_upper_bound'],'payload_bytes':size,'elapsed_seconds':manifest['elapsed_seconds'],'engineering_fixture_pass':True,**FALSE},indent=2))
if __name__=='__main__':
    if len(sys.argv)!=2:raise SystemExit('usage: run_durability.py NEW_OUTPUT_DIRECTORY')
    run(Path(sys.argv[1]).resolve())
