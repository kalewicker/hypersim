"""Freeze and execute one bounded live development controller integration."""
import json
from pathlib import Path
import shutil
import subprocess
import sys
import time
from run_readiness import ROOT, PACKAGE, FALSE, digest, write, source_paths
from run_campaign_adapter import custody
from compact_evidence import source_inventory, encode_json, publish, sha
from campaign_controller import Controller, PROFILE
from evidence_spool import Spool, WRITER_LIMIT
from live_campaign_dispatch import DevelopmentDispatch
from campaign_measurement import plan, evaluate
from audit_campaign_measurement import check


def run(out):
    started=time.monotonic();out.mkdir(parents=True,exist_ok=False)
    for name in ['commands','inputs']:(out/name).mkdir()
    c=json.loads((PACKAGE/'live_dispatch_contract.json').read_text())
    manifest=dict(profile=c['profile'],status='running',engineering_live_development_pass=False,native_work_lower=0,native_work_upper=0,**FALSE)
    write(out/'manifest.json',manifest);controller=None
    def preserve():
        custody();prior=json.loads((PACKAGE/'measurement_aggregation_contract.json').read_text())
        for ref in [prior[k] for k in ['parent','adapter_parent','storage_parent','reuse_native_parent']]+[c['measurement_parent'],c['parent']]:source_inventory(ROOT/ref['archive'],ref['seal_sha256'])
        pins=json.loads((ROOT/c['parent']['archive']/'source_sha256.json').read_text())
        if any(digest(ROOT/n)!=h for n,h in pins.items()):raise ValueError('historical source modified')
    def command(name,args):
        r=subprocess.run(args,capture_output=True,text=True,timeout=max(1,1200-(time.monotonic()-started)))
        write(out/'commands'/name,dict(exit_code=r.returncode,stdout=r.stdout,stderr=r.stderr))
        if r.returncode:raise ValueError(r.stderr)
        return r.stdout
    try:
        preserve();pins={str(p.relative_to(ROOT)):digest(p) for p in source_paths()};write(out/'source_sha256.json',pins)
        for name in pins:
            dest=out/'source_snapshots'/name;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(ROOT/name,dest)
        write(out/'contract.json',c);frozen=out/'source_snapshots/experiments/cumulative/scripts'
        command('dispatch_tests.json',[sys.executable,'-B',str(frozen/'test_live_campaign_dispatch.py')])
        command('controller_tests.json',[sys.executable,'-B',str(frozen/'test_campaign_controller.py')])
        parent=out/'measurement_parent';shutil.copytree(ROOT/c['measurement_parent']['archive'],parent)
        campaign=json.loads((parent/'campaign_contract.json').read_text());write(out/'campaign_contract.json',campaign)
        adapter=ROOT/c['adapter_parent']['archive'];ac=json.loads((adapter/'contract.json').read_text())
        binary=out/'cumulative-campaign';shutil.copy2(adapter/'cumulative-campaign',binary)
        source=dict(binary_sha256=digest(binary),source_sha256=digest(out/'source_sha256.json'))
        requests=[];worlds=[]
        for arm in c['world_order']:
            request=json.loads((adapter/'inputs'/('world_'+arm+'.json')).read_text())
            request.update(native_budget=1200,execution_allowance=c['world_reservations'][arm],journal_byte_allowance=WRITER_LIMIT,source_identity=source,source=None,fault=None)
            raw=encode_json(request);publish(out/'inputs'/('world_'+arm+'.json'),raw);requests.append(request)
            worlds.append(dict(world_id='development/'+arm,request_sha256=sha(raw),work_bound=c['world_reservations'][arm]))
        owner=dict(profile=PROFILE,purpose='saved_development',worlds=worlds,
                   limits=dict(rewrites=1200,native_ms=180000,workflow_ms=1200000,storage_bytes=268435456),
                   slots_per_world=10,batch_size=128,assay_work_bound=20,maximum_assays=78960,**FALSE)
        controller=Controller.create(out/'controller',owner);spool=Spool.create(out/'native_spool',268435456)
        bridge=DevelopmentDispatch(controller,spool,binary,ac,started,c['limits']['per_unit_ms'])
        cases=[];routes=[]
        for arm,request in zip(c['world_order'],requests):
            result=bridge.dispatch(request);recovered=result.pop('recovered');routes.append(result)
            cases.append(dict(name=arm,initial=recovered['initial']['state'],rows=recovered['rows'],final=recovered['checkpoint']['state']))
            write(out/'native_routes.json',routes)
            print(json.dumps(dict(phase=arm,work=recovered['here_upper'])),flush=True)
        measurement,registry=plan(cases,campaign);check(cases,campaign,measurement,registry)
        if measurement!=json.loads((parent/'measurement.json').read_text()) or registry!=json.loads((parent/'registry.json').read_text()):raise ValueError('saved fixed development measurement changed')
        publish(out/'measurement.json',encode_json(measurement));publish(out/'registry.json',encode_json(registry))
        controller.append('registry',dict(sha256=digest(out/'registry.json'),measurement_sha256=digest(out/'measurement.json'),count=len(registry['assays']),slots=len(measurement['samples'])))
        results=[]
        for start in range(0,len(registry['assays']),128):
            end=min(start+128,len(registry['assays']));request=json.loads((adapter/'inputs/assays_0.json').read_text())
            request.update(native_budget=1200,execution_allowance=20*(end-start),journal_byte_allowance=WRITER_LIMIT,source_identity=source,source=None,fault=None,
                           registry=dict(path=str(out/'registry.json'),sha256=digest(out/'registry.json')),range=dict(start=start,end=end))
            publish(out/'inputs'/f'assays_{start}.json',encode_json(request))
            result=bridge.dispatch(request,registry);recovered=result.pop('recovered');routes.append(result);results.extend(recovered['results']);write(out/'native_routes.json',routes)
        publish(out/'report.json',encode_json(evaluate(measurement,registry,results,cases,campaign)))
        controller.append('close',dict(reason='complete',decision_sha256=digest(out/'report.json')))
        audit=json.loads(command('independent_audit.json',[sys.executable,'-B',str(frozen/'audit_live_dispatch.py'),str(out)]));write(out/'audit.json',audit)
        write(out/'remaining_launch_requirements.json',dict(scientific_native_profile_implemented=False,campaign_preregistration_complete=False,
              completed='durable controller, actual monotonic deadlines, actual spool admission, native journal audit, complete measurement/registry/assay closure on fixed development worlds',
              remaining=['Implement versioned native admission restricted to the exact 47-block panel and independently audit that profile without starting scientific worlds.',
                         'Freeze the final scoped scientific contract and obtain explicit authorization for its proposed budget and single launch.'],**FALSE))
        preserve()
        if any(digest(ROOT/n)!=h for n,h in pins.items()):raise ValueError('source changed during live fixture')
        write(out/'preservation.json',dict(previous_archive_roots_preserved=17,previous_source_pins_preserved=174,historical_files_removed=0))
        manifest.update(audit,status='complete',source_files=len(pins),python_test_groups=13,elapsed_seconds=time.monotonic()-started,**FALSE);write(out/'manifest.json',manifest)
        if time.monotonic()-started>1200 or sum(p.stat().st_size for p in out.rglob('*') if p.is_file())>134217728:raise ValueError('workflow/output cap')
    except Exception as e:
        lower=0;upper=1200
        if controller is not None:
            try:
                state,_,_=controller.replay();lower=state['rewrites_lower'];upper=state['rewrites_upper']
                if not state['closed']:controller.append('close',dict(reason='operational_stop',decision_sha256=None))
            except Exception:pass  # Never rewrite or retry uncertain/torn controller evidence.
        manifest.update(status='failed',error=str(e),engineering_live_development_pass=False,native_work_lower=lower,native_work_upper=upper)
        write(out/'manifest.json',manifest);write(out/'output_sha256.json',{str(p.relative_to(out)):digest(p) for p in sorted(out.rglob('*')) if p.is_file() and p!=out/'output_sha256.json'})
        raise
    write(out/'output_sha256.json',{str(p.relative_to(out)):digest(p) for p in sorted(out.rglob('*')) if p.is_file()})
    print(json.dumps(dict(output=str(out),seal_sha256=digest(out/'output_sha256.json'),**manifest),indent=2))


if __name__=='__main__':run(Path(sys.argv[1]).resolve())
