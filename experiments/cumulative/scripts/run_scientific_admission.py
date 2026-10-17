"""Freeze and execute only the scientific admission checker, never scientific worlds."""
import json
from pathlib import Path
import shutil
import subprocess
import sys
import time
from run_readiness import ROOT, PACKAGE, FALSE, digest, write, source_paths
from run_campaign_adapter import custody
from compact_evidence import source_inventory
from scientific_admission_cases import corpus


def run(out):
    started=time.monotonic();out.mkdir(parents=True,exist_ok=False)
    for name in ['commands','requests','receipts','check_working_directory']:(out/name).mkdir()
    cr=(PACKAGE/'scientific_admission_contract.json').read_bytes();c=json.loads(cr)
    manifest=dict(profile=c['profile'],status='running',engineering_scientific_admission_pass=False,native_rewrites=0,**FALSE)
    write(out/'manifest.json',manifest)
    def preserve():
        custody();prior=json.loads((PACKAGE/'measurement_aggregation_contract.json').read_text())
        references=[prior[k] for k in ['parent','adapter_parent','storage_parent','reuse_native_parent']]
        references += [json.loads((PACKAGE/'controller_contract.json').read_text())['parent'],json.loads((PACKAGE/'live_dispatch_contract.json').read_text())['parent'],c['parent']]
        for ref in references:source_inventory(ROOT/ref['archive'],ref['seal_sha256'])
        old=json.loads((ROOT/c['parent']['archive']/'source_sha256.json').read_text())
        if any(digest(ROOT/n)!=h for n,h in old.items()):raise ValueError('prior source modified')
    def command(name,args):
        r=subprocess.run(args,cwd=ROOT,capture_output=True,text=True,timeout=max(1,1200-(time.monotonic()-started)))
        write(out/'commands'/name,dict(exit_code=r.returncode,stdout=r.stdout,stderr=r.stderr))
        if r.returncode:raise ValueError(r.stderr)
        return r.stdout
    try:
        preserve();pins={str(p.relative_to(ROOT)):digest(p) for p in source_paths()};write(out/'source_sha256.json',pins)
        for name in pins:
            dest=out/'source_snapshots'/name;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(ROOT/name,dest)
        (out/'contract.json').write_bytes(cr)
        parent=out/'live_parent';shutil.copytree(ROOT/c['parent']['archive'],parent)
        frozen=out/'source_snapshots/experiments/cumulative/scripts'
        command('parent_recheck.json',[sys.executable,'-B',str(parent/'source_snapshots/experiments/cumulative/scripts/recheck_live_dispatch.py'),str(parent),c['parent']['seal_sha256']])
        command('format.json',['rustfmt','--check','--edition','2021',str(PACKAGE/'src/bin/cumulative-scientific-admission.rs')])
        command('rust_tests.json',['cargo','test','--offline','--locked','--manifest-path',str(PACKAGE/'Cargo.toml'),'--bin','cumulative-scientific-admission'])
        command('build.json',['cargo','build','--offline','--locked','--manifest-path',str(PACKAGE/'Cargo.toml'),'--bin','cumulative-scientific-admission'])
        binary=out/'cumulative-scientific-admission';shutil.copy2(PACKAGE/'target/debug/cumulative-scientific-admission',binary);binary_hash=digest(binary)
        cases=corpus(cr,(PACKAGE/'scientific_panel.json').read_bytes())
        index=[]
        # Freeze every case, including maximum-size and malformed requests, before checking any.
        for case in cases:
            p=out/'requests'/(case['name']+'.json');p.write_bytes(case['raw'])
            index.append(dict(name=case['name'],mode=case['mode'],valid=case['valid'],request_sha256=digest(p)))
        write(out/'cases.json',index)
        for i,case in enumerate(cases):
            r=subprocess.run([str(binary),case['mode'],str(out/'requests'/(case['name']+'.json'))],cwd=out/'check_working_directory',capture_output=True,text=True,timeout=30)
            write(out/'receipts'/(case['name']+'.json'),dict(binary_sha256=binary_hash,exit_code=r.returncode,stdout=r.stdout,stderr=r.stderr))
            if r.returncode!=(0 if case['valid'] else 1):raise ValueError('unexpected native admission verdict '+case['name']+': '+r.stderr)
            if (i+1)%100==0:print(json.dumps(dict(phase='admission_checks',completed=i+1,total=len(cases),native_rewrites=0)),flush=True)
        audited=json.loads(command('independent_audit.json',[sys.executable,'-B',str(frozen/'audit_scientific_admission.py'),str(out)]));write(out/'audit.json',audited)
        write(out/'remaining_launch_requirements.json',dict(native_admission_validator_implemented=True,scientific_execution_adapter_implemented=False,
            remaining=['Wire the shared Rust admission validator into a versioned scientific execution entry point and its controller/journal auditor; preserve exact identities and fail closed before native work.',
                       'Finalize the scoped scientific contract and obtain explicit authorization for the proposed resource ceilings and one fixed launch.'],
            admission_is_not_execution_permission=True,**FALSE))
        preserve()
        if any(digest(ROOT/n)!=h for n,h in pins.items()):raise ValueError('source changed after freeze')
        write(out/'preservation.json',dict(previous_archive_roots_preserved=18,previous_source_pins_preserved=180,historical_files_removed=0))
        manifest.update(audited,status='complete',source_files=len(pins),rust_test_groups=3,elapsed_seconds=time.monotonic()-started,**FALSE);write(out/'manifest.json',manifest)
        if time.monotonic()-started>1200 or sum(p.stat().st_size for p in out.rglob('*') if p.is_file())>134217728:raise ValueError('workflow/output cap')
    except Exception as e:
        manifest.update(status='failed',error=str(e),engineering_scientific_admission_pass=False);write(out/'manifest.json',manifest)
        write(out/'output_sha256.json',{str(p.relative_to(out)):digest(p) for p in sorted(out.rglob('*')) if p.is_file() and p!=out/'output_sha256.json'})
        raise
    write(out/'output_sha256.json',{str(p.relative_to(out)):digest(p) for p in sorted(out.rglob('*')) if p.is_file()})
    print(json.dumps(dict(output=str(out),seal_sha256=digest(out/'output_sha256.json'),**manifest),indent=2))


if __name__=='__main__':run(Path(sys.argv[1]).resolve())
