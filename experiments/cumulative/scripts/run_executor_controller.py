"""One frozen complete development campaign and one interrupted attempt; no retries."""
import json
from pathlib import Path
import shutil
import subprocess
import sys
import time
from run_readiness import ROOT,PACKAGE,FALSE,digest,write,source_paths
from compact_evidence import encode_json,publish,sha,source_inventory
from campaign_controller import Controller,PROFILE
from evidence_spool import Spool
from controlled_executor_dispatch import ControlledDispatch,request
from campaign_measurement import plan,evaluate
from audit_campaign_measurement import check


def run(out):
    started=time.monotonic();out.mkdir(parents=True,exist_ok=False);(out/'commands').mkdir()
    cr=(PACKAGE/'executor_controller_contract.json').read_bytes();c=json.loads(cr)
    m=dict(status='running',profile=c['profile'],engineering_controller_bridge_pass=False,native_work_lower=0,native_work_upper=0,**FALSE)
    write(out/'manifest.json',m);controllers=[]
    def command(name,args,expected=0):
        remaining=1200-(time.monotonic()-started)
        if remaining<=0:raise ValueError('workflow deadline')
        r=subprocess.run(args,capture_output=True,text=True,timeout=remaining)
        write(out/'commands'/name,dict(exit_code=r.returncode,stdout=r.stdout,stderr=r.stderr))
        if r.returncode!=expected:raise ValueError(name+': '+r.stderr)
        return r.stdout
    def owner(routes,limit,binary):
        return dict(profile=PROFILE,purpose='saved_development',worlds=[dict(world_id='development/'+r['name'],request_sha256=sha(encode_json(request(r,cr,digest(binary)))),work_bound=r['execution_allowance']) for r in routes],limits=dict(rewrites=limit,native_ms=180000,workflow_ms=1200000,storage_bytes=268435456),slots_per_world=10,batch_size=128,assay_work_bound=20,maximum_assays=78960,**FALSE)
    try:
        for key,destination in [('parent','executor_parent'),('measurement_parent','measurement_parent')]:
            ref=c[key];source_inventory(ROOT/ref['archive'],ref['seal_sha256']);shutil.copytree(ROOT/ref['archive'],out/destination)
        old=json.loads((out/'executor_parent/source_sha256.json').read_text())
        if any(digest(ROOT/n)!=h for n,h in old.items()):raise ValueError('historical source changed')
        pins={str(p.relative_to(ROOT)):digest(p) for p in source_paths()};write(out/'source_sha256.json',pins)
        for n in pins:
            p=out/'source_snapshots'/n;p.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(ROOT/n,p)
        (out/'contract.json').write_bytes(cr);frozen=out/'source_snapshots/experiments/cumulative/scripts'
        command('boundary_tests.json',[sys.executable,'-B',str(frozen/'test_controlled_executor_dispatch.py')])
        command('format.json',['rustfmt','--check','--edition','2021','--config','skip_children=true',str(PACKAGE/'src/bin/cumulative-controlled-executor.rs')])
        command('rust_tests.json',['cargo','test','--offline','--locked','--manifest-path',str(PACKAGE/'Cargo.toml'),'--bin','cumulative-controlled-executor'])
        command('build.json',['cargo','build','--offline','--locked','--manifest-path',str(PACKAGE/'Cargo.toml'),'--bin','cumulative-controlled-executor'])
        binary=out/'cumulative-controlled-executor';shutil.copy2(PACKAGE/'target/debug/cumulative-controlled-executor',binary)
        write(out/'refused_authorization.json',dict(purpose='negative_test_only_not_user_authorization',launch_authorized=True))
        for i in range(188):
            command(f'scientific_refusal_{i:03}.json',[str(binary),'--execute',str(out/'executor_parent/requests'/f'world_{i:03}.json'),str(out/'refused_authorization.json'),str(out/'forbidden_journal')],expected=1)
        campaign=json.loads((out/'measurement_parent/campaign_contract.json').read_text());write(out/'campaign_contract.json',campaign)
        root=out/'complete';root.mkdir();controller=Controller.create(root/'controller',owner(c['routes'][:4],1200,binary));controllers.append(controller)
        bridge=ControlledDispatch(controller,Spool.create(root/'spool',268435456),binary,cr,out,started)
        cases=[];routes=[]
        for route in c['routes'][:4]:
            result=bridge.dispatch(route);recovered=result.pop('recovered');routes.append(result);write(root/'routes.json',routes)
            cases.append(dict(name=route['name'],initial=recovered['results'][0]['row']['before'],rows=[r['row'] for r in recovered['results']],final=recovered['checkpoint']['state']))
            print(json.dumps(dict(phase=route['name'],work=recovered['lower'])),flush=True)
        measurement,registry=plan(cases,campaign);check(cases,campaign,measurement,registry)
        if registry['assays']!=c['routes'][4]['items']:raise ValueError('measured registry differs from preregistered development route')
        publish(out/'measurement.json',encode_json(measurement));publish(out/'registry.json',encode_json(registry))
        controller.append('registry',dict(sha256=digest(out/'registry.json'),measurement_sha256=digest(out/'measurement.json'),count=len(registry['assays']),slots=len(measurement['samples'])))
        result=bridge.dispatch(c['routes'][4],registry);recovered=result.pop('recovered');routes.append(result);write(root/'routes.json',routes)
        publish(out/'report.json',encode_json(evaluate(measurement,registry,recovered['results'],cases,campaign)))
        controller.append('close',dict(reason='complete',decision_sha256=digest(out/'report.json')))
        # Separate preregistered failure route; this is not a retry of any world.
        root=out/'interrupted';root.mkdir();fault_controller=Controller.create(root/'controller',owner([c['routes'][-1]]+c['routes'][1:4],250,binary));controllers.append(fault_controller)
        bridge=ControlledDispatch(fault_controller,Spool.create(root/'spool',268435456),binary,cr,out,started)
        try:bridge.dispatch(c['routes'][-1])
        except ValueError as e:
            if 'unknown execution' not in str(e):raise
            write(root/'expected_refusal.json',dict(error=str(e)))
        else:raise ValueError('declared interrupted route unexpectedly complete')
        try:bridge.dispatch(c['routes'][-1])
        except ValueError as e:write(root/'retry_refusal.json',dict(error=str(e)))
        else:raise ValueError('unknown action retried')
        fault_controller.append('close',dict(reason='operational_stop',decision_sha256=None))
        audited=json.loads(command('independent_audit.json',[sys.executable,'-B',str(frozen/'audit_executor_controller.py'),str(out)]));write(out/'audit.json',audited)
        if any(digest(ROOT/n)!=h for n,h in pins.items()):raise ValueError('source changed after freeze')
        write(out/'preservation.json',dict(previous_source_pins_preserved=len(old),historical_files_removed=0,parent_seal=c['parent']['seal_sha256']))
        m.update(audited,status='complete',source_files=len(pins),python_test_groups=7,rust_test_groups=3,elapsed_seconds=time.monotonic()-started);write(out/'manifest.json',m)
        if time.monotonic()-started>1200 or sum(p.stat().st_size for p in out.rglob('*') if p.is_file())>134217728:raise ValueError('workflow/output cap')
    except Exception as e:
        lower=upper=0
        for ctl in controllers:
            try:
                s,_,_=ctl.replay();lower+=s['rewrites_lower'];upper+=s['rewrites_upper']
                if not s['closed']:ctl.append('close',dict(reason='operational_stop',decision_sha256=None))
            except Exception:upper=1500
        m.update(status='failed',error=str(e),engineering_controller_bridge_pass=False,native_work_lower=lower,native_work_upper=upper);write(out/'manifest.json',m)
        raise
    finally:write(out/'output_sha256.json',{str(p.relative_to(out)):digest(p) for p in sorted(out.rglob('*')) if p.is_file() and p!=out/'output_sha256.json'})
    print(json.dumps(dict(output=str(out),seal_sha256=digest(out/'output_sha256.json'),**m),indent=2))


if __name__=='__main__':run(Path(sys.argv[1]).resolve())
