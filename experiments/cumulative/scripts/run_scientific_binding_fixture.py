"""Freeze and test scientific bindings without executing any native world or assay."""
import json
from pathlib import Path
import shutil
import subprocess
import sys
import time
from run_readiness import ROOT,PACKAGE,FALSE,digest,write,source_paths
from compact_evidence import source_inventory,require
from scientific_launch_binding import CONTRACT,owner,sha
from scientific_binding_cases import cases
from test_scientific_launch import preview
from scientific_campaign_controller import Controller,next_unit


def run(out):
    started=time.monotonic();out.mkdir(parents=True,exist_ok=False)
    for name in ['commands','bindings']:(out/name).mkdir()
    m=dict(status='running',native_rewrites=0,engineering_scientific_binding_pass=False,**FALSE);write(out/'manifest.json',m)
    def command(name,args,expected=0):
        remaining=1200-(time.monotonic()-started)
        if remaining<=0:raise ValueError('fixture deadline')
        p=subprocess.run(args,capture_output=True,text=True,timeout=remaining);write(out/'commands'/name,dict(exit_code=p.returncode,stdout=p.stdout,stderr=p.stderr))
        if p.returncode!=expected:raise ValueError(name+': '+p.stderr)
        return p.stdout
    try:
        parent=ROOT/CONTRACT['parent']['archive'];source_inventory(parent,CONTRACT['parent']['seal_sha256'])
        old=json.loads((parent/'source_sha256.json').read_text())
        require(all(digest(ROOT/n)==h for n,h in old.items()),'prior sources unchanged')
        pins={str(p.relative_to(ROOT)):digest(p) for p in source_paths()};write(out/'source_sha256.json',pins)
        for n in pins:
            p=out/'source_snapshots'/n;p.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(ROOT/n,p)
        shutil.copytree(parent,out/'parent');frozen=out/'source_snapshots/experiments/cumulative/scripts'
        command('python_tests.json',[sys.executable,'-B',str(frozen/'test_scientific_launch.py')])
        command('format.json',['rustfmt','--check','--edition','2021','--config','skip_children=true',str(PACKAGE/'src/bin/cumulative-scientific-launch.rs'),str(PACKAGE/'src/bin/support/scientific_binding.rs')])
        command('rust_tests.json',['cargo','test','--offline','--locked','--manifest-path',str(PACKAGE/'Cargo.toml'),'--bin','cumulative-scientific-launch'])
        command('build.json',['cargo','build','--offline','--locked','--manifest-path',str(PACKAGE/'Cargo.toml'),'--bin','cumulative-scientific-launch'])
        binary=out/'cumulative-scientific-launch';shutil.copy2(PACKAGE/'target/debug/cumulative-scientific-launch',binary)
        proposed=ROOT/'output/cumulative_scientific_campaign_20261011'
        a=preview(digest(binary),digest(out/'source_sha256.json'),sha(str(proposed).encode()));write(out/'authorization_preview.json',a)
        corpus=cases(a)
        for c in corpus:(out/'bindings'/(c['name']+'.json')).write_bytes(c['raw'])
        for c in corpus:
            path=out/'bindings'/(c['name']+'.json')
            command('inspect_'+c['name']+'.json',[str(binary),'--inspect',str(path)],0 if c['valid'] else 1)
            if c['valid']:command('execute_'+c['name']+'.json',[str(binary),'--execute',str(path),str(out/'forbidden_journal')],1)
        command('driver_refusal.json',[sys.executable,'-B',str(frozen/'run_scientific_campaign.py'),'--launch',str(out/'authorization_preview.json'),str(binary),str(out/'source_sha256.json'),str(out/'forbidden_campaign')],1)
        ctl=Controller.create(out/'preview_controller',owner(a));state,_,_=ctl.replay()
        ctl.append('intent',dict(unit=next_unit(ctl.contract,state),native_ms_bound=30000,elapsed_ms=0,storage_charge_bytes=200000000))
        ctl.append('close',dict(reason='operational_stop',decision_sha256=None))
        write(out/'preview_controller_scope.json',dict(synthetic_accounting_only=True,native_rewrites=0,no_scientific_execution=True))
        result=json.loads(command('independent_audit.json',[sys.executable,'-B',str(frozen/'audit_scientific_binding_fixture.py'),str(out)]));write(out/'audit.json',result)
        require(all(digest(ROOT/n)==h for n,h in pins.items()),'source freeze preserved')
        write(out/'preservation.json',dict(previous_source_pins_preserved=len(old),historical_files_removed=0))
        m.update(result,status='complete',python_test_groups=7,rust_test_groups=4,elapsed_seconds=time.monotonic()-started);write(out/'manifest.json',m)
        require(sum(p.stat().st_size for p in out.rglob('*') if p.is_file())<134217728,'fixture output cap')
    except Exception as e:
        m.update(status='failed',error=str(e),engineering_scientific_binding_pass=False);write(out/'manifest.json',m);raise
    finally:write(out/'output_sha256.json',{str(p.relative_to(out)):digest(p) for p in sorted(out.rglob('*')) if p.is_file() and p!=out/'output_sha256.json'})
    print(json.dumps(dict(output=str(out),seal_sha256=digest(out/'output_sha256.json'),**m),indent=2))


if __name__=='__main__':run(Path(sys.argv[1]).resolve())
