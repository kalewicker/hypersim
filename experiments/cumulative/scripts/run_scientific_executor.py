"""Freeze one bounded development execution. Never authorize scientific dispatch."""
import json
from pathlib import Path
import shutil
import subprocess
import sys
import time
from run_readiness import ROOT, PACKAGE, FALSE, digest, write, source_paths
from recheck_admission_boundary import recheck
from compact_evidence import source_inventory


def run(out):
    started=time.monotonic();out.mkdir(parents=True,exist_ok=False)
    for n in ['commands','requests','journals','reference']:(out/n).mkdir()
    cr=(PACKAGE/'scientific_executor_contract.json').read_bytes();c=json.loads(cr)
    m=dict(status='running',profile=c['profile'],native_work_lower=0,native_work_upper=0,engineering_executor_pass=False,**FALSE)
    write(out/'manifest.json',m)
    pins={str(p.relative_to(ROOT)):digest(p) for p in source_paths()}
    native_seconds=0;reserved=0
    def command(name,args,expected=0,native=False):
        nonlocal native_seconds
        remaining=1200-(time.monotonic()-started)
        timeout=min(remaining,180-native_seconds) if native else remaining
        if timeout<=0:raise ValueError('aggregate time cap')
        t=time.monotonic()
        try:r=subprocess.run(args,cwd=ROOT,capture_output=True,text=True,timeout=timeout)
        except subprocess.TimeoutExpired as e:
            write(out/'commands'/name,dict(timed_out=True,stdout=str(e.stdout),stderr=str(e.stderr),seconds=time.monotonic()-t));raise
        seconds=time.monotonic()-t
        if native:native_seconds+=seconds
        receipt=dict(exit_code=r.returncode,stdout=r.stdout,stderr=r.stderr,seconds=seconds)
        write(out/'commands'/name,receipt)
        if r.returncode!=expected:raise ValueError(name+': '+r.stderr)
        return receipt
    try:
        parent=ROOT/c['parent']['archive']
        write(out/'parent_recheck.json',recheck(parent,c['parent']['seal_sha256']))
        old=json.loads((parent/'source_sha256.json').read_text())
        if any(digest(ROOT/n)!=h for n,h in old.items()):raise ValueError('prior source changed')
        write(out/'source_sha256.json',pins)
        for n in pins:
            dest=out/'source_snapshots'/n;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(ROOT/n,dest)
        (out/'contract.json').write_bytes(cr)
        reference=ROOT/'output/cumulative_campaign_adapter_readiness_20261010'
        source_inventory(reference,'827890af64baf8cf8c25d67056a5006294c6c95fcddd888d03870c6d6287f794')
        shutil.copy2(reference/'output_sha256.json',out/'reference_seal.json')
        for route in c['routes'][:4]:shutil.copytree(reference/'journals'/('world_'+route['name']),out/'reference/journals'/('world_'+route['name']))
        command('format.json',['rustfmt','--check','--edition','2021','--config','skip_children=true',str(PACKAGE/'src/bin/cumulative-scientific-executor.rs'),str(PACKAGE/'src/bin/support/execution_journal.rs')])
        command('build.json',['cargo','build','--offline','--locked','--manifest-path',str(PACKAGE/'Cargo.toml'),'--bin','cumulative-scientific-executor'])
        command('rust_tests.json',['cargo','test','--offline','--locked','--manifest-path',str(PACKAGE/'Cargo.toml'),'--bin','cumulative-scientific-executor'])
        binary=out/'cumulative-scientific-executor';shutil.copy2(PACKAGE/'target/debug/cumulative-scientific-executor',binary)
        cases=[]
        for i in range(188):
            name=f'world_{i:03}.json';shutil.copy2(parent/'requests'/name,out/'requests'/name)
            cases.append(dict(name='check_'+name,file=name,mode='--check'))
        # All panel members plus a forged approval claim must fail before journal creation.
        write(out/'authorization_refusal_fixture.json',dict(launch_authorized=True,budget_approved=True,purpose='negative_test_only_not_user_authorization'))
        cases += [dict(name=f'execute_{i:03}.json',file=f'world_{i:03}.json',mode='--execute') for i in range(188)]
        write(out/'boundary_cases.json',cases)
        for case in cases:
            args=[str(binary),case['mode'],str(out/'requests'/case['file'])]
            if case['mode']=='--execute':args += [str(out/'authorization_refusal_fixture.json'),str(out/'forbidden-journal')]
            command(case['name'],args,expected=0 if case['mode']=='--check' else 1)
        # Freeze is complete before the first native call; unknown work is never retried.
        for route in c['routes']:
            reserved+=route['execution_allowance']
            if reserved>c['limits']['native_rewrites']:raise ValueError('aggregate reservation')
            m['native_work_upper']=reserved;write(out/'manifest.json',m)
            command(route['name']+'.json',[str(binary),'--development',route['name'],str(out/'journals'/route['name'])],expected=1 if route['fault'] else 0,native=True)
        # Existing journal rejection is zero-native and cannot overwrite completed evidence.
        refusal=command('existing_journal.json',[str(binary),'--development','endogenous',str(out/'journals/endogenous')],expected=1)
        if json.loads(refusal['stderr'])['native_rewrites']!=0:raise ValueError('existing journal advanced')
        frozen=out/'source_snapshots/experiments/cumulative/scripts'
        result=json.loads(command('audit.json',[sys.executable,'-B',str(frozen/'audit_scientific_executor.py'),str(out)])['stdout']);write(out/'audit.json',result)
        tests=json.loads(command('auditor_tests.json',[sys.executable,'-B',str(frozen/'test_scientific_executor.py'),str(out)])['stdout']);write(out/'auditor_tests.json',tests)
        if any(digest(ROOT/n)!=h for n,h in pins.items()):raise ValueError('source changed after freeze')
        write(out/'preservation.json',dict(previous_source_pins_preserved=len(old),historical_files_removed=0,parent_seal=c['parent']['seal_sha256']))
        m.update(result,status='complete',source_files=len(pins),native_seconds=native_seconds,elapsed_seconds=time.monotonic()-started);write(out/'manifest.json',m)
        if sum(p.stat().st_size for p in out.rglob('*') if p.is_file())>c['limits']['output_bytes']:raise ValueError('output cap')
    except Exception as e:
        m.update(status='failed',error=str(e),engineering_executor_pass=False);write(out/'manifest.json',m)
        raise
    finally:
        write(out/'output_sha256.json',{str(p.relative_to(out)):digest(p) for p in sorted(out.rglob('*')) if p.is_file() and p!=out/'output_sha256.json'})
    print(json.dumps(dict(output=str(out),seal_sha256=digest(out/'output_sha256.json'),**m),indent=2))


if __name__=='__main__':run(Path(sys.argv[1]).resolve())
