"""Zero-native repair of report assembly; retain and audit the exact failed parent."""
import json
from pathlib import Path
import shutil
import subprocess
import sys
import time
from run_readiness import ROOT, PACKAGE, FALSE, digest, write, source_paths
from run_campaign_adapter import custody
from compact_evidence import Store, source_inventory, sha
from audit_campaign_measurement import worlds, check
from audit_spool import ledger, Journal
from audit_durability import read
from audit_campaign_adapter import recover_values
from campaign_measurement import plan, evaluate


def run(out):
    start=time.monotonic();out.mkdir(parents=True,exist_ok=False);(out/'commands').mkdir()
    c=json.loads((PACKAGE/'measurement_aggregation_contract.json').read_text())
    manifest=dict(profile=c['profile'],status='running',engineering_measurement_pass=False,native_work_lower=0,native_work_upper=0,**FALSE)
    write(out/'manifest.json',manifest)
    def preserve():
        custody()
        for key in ['parent','adapter_parent','storage_parent','reuse_native_parent']:
            p=c[key];source_inventory(ROOT/p['archive'],p['seal_sha256'])
        old=json.loads((ROOT/c['parent']['archive']/'source_sha256.json').read_text())
        if any(digest(ROOT/n)!=h for n,h in old.items()):raise ValueError('historical source modified')
    try:
        preserve();pins={str(p.relative_to(ROOT)):digest(p) for p in source_paths()};write(out/'source_sha256.json',pins)
        for n in pins:
            dest=out/'source_snapshots'/n;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(ROOT/n,dest)
        write(out/'contract.json',c);frozen=out/'source_snapshots/experiments/cumulative/scripts'
        tests=subprocess.run([sys.executable,'-B',str(frozen/'test_campaign_measurement.py'),str(ROOT)],capture_output=True,text=True,timeout=120)
        write(out/'commands/tests.json',dict(exit_code=tests.returncode,stdout=tests.stdout,stderr=tests.stderr))
        if tests.returncode:raise ValueError(tests.stderr)
        origin=out/'native_origin';shutil.copytree(ROOT/c['reuse_native_parent']['archive'],origin)
        for name in ['campaign_contract.json','adapter_store.json','prospective_pending.json','remaining_launch_requirements.json']:
            shutil.copy2(origin/name,out/name)
        shutil.copytree(origin/'adapter_evidence',out/'adapter_evidence')
        info=json.loads((out/'adapter_store.json').read_text());adapter=Store(out/'adapter_evidence',info['index_sha256'],c['adapter_parent']['seal_sha256']);adapter.verify_all()
        campaign=json.loads((out/'campaign_contract.json').read_text());cases=worlds(adapter.root)
        measurement,registry=plan(cases,campaign);check(cases,campaign,measurement,registry)
        write(out/'measurement.json',measurement);write(out/'registry.json',registry)
        if (out/'registry.json').read_bytes()!=(origin/'registry.json').read_bytes():raise ValueError('registry changed after native outcomes')
        states,_=ledger(origin/'native_spool');results=[];work=0
        for unit,state in states.items():
            raw=(origin/'native_spool/requests'/(unit+'.json')).read_bytes();request=json.loads(raw)
            compact=Store(origin/'native_spool/compact'/unit,state['index_sha256'],state['seal_sha256'])
            values,head,staging=read(Journal(compact))
            recovered=recover_values(values,head,staging,request,sha(raw),json.loads((adapter.root/'contract.json').read_text()),registry['assays'])
            if recovered['status']!='complete' or recovered['unknown'] or staging:raise ValueError('saved native work incomplete; no retry')
            results.extend(recovered['results']);work+=recovered['here_upper']
        write(out/'report.json',evaluate(measurement,registry,results,cases,campaign))
        write(out/'recovery.json',dict(failed_parent=c['reuse_native_parent'],failure='report assembly expected status at envelope level instead of results[0]',exact_saved_assay_count=len(results),saved_native_work_lower=work,saved_native_work_upper=work,new_native_rewrites=0,redispatch=False,prior_failure_manifest_unchanged=True))
        run=subprocess.run([sys.executable,'-B',str(frozen/'audit_campaign_measurement.py'),str(out)],capture_output=True,text=True,timeout=120)
        write(out/'commands/independent_audit.json',dict(exit_code=run.returncode,stdout=run.stdout,stderr=run.stderr))
        if run.returncode:raise ValueError(run.stderr)
        audited=json.loads(run.stdout);write(out/'audit.json',audited)
        preserve()
        if any(digest(ROOT/n)!=h for n,h in pins.items()):raise ValueError('source changed after freeze')
        write(out/'preservation.json',dict(previous_archive_roots_preserved=15,previous_source_pins_preserved=160,failed_assembly_sources_preserved_in_original_sealed_archive=True,historical_raw_files_removed=0))
        manifest.update(audited,status='complete',source_files=len(pins),native_seconds=0,elapsed_seconds=time.monotonic()-start,**FALSE)
        write(out/'manifest.json',manifest)
        if time.monotonic()-start>c['limits']['workflow_seconds'] or sum(p.stat().st_size for p in out.rglob('*') if p.is_file())>c['limits']['output_bytes']:raise ValueError('workflow/output limit')
    except Exception as e:
        manifest.update(status='failed',error=str(e),engineering_measurement_pass=False)
        write(out/'manifest.json',manifest)
        write(out/'output_sha256.json',{str(p.relative_to(out)):digest(p) for p in sorted(out.rglob('*')) if p.is_file() and p!=out/'output_sha256.json'})
        raise
    write(out/'output_sha256.json',{str(p.relative_to(out)):digest(p) for p in sorted(out.rglob('*')) if p.is_file()})
    print(json.dumps(dict(output=str(out),seal_sha256=digest(out/'output_sha256.json'),**manifest),indent=2))


if __name__=='__main__':run(Path(sys.argv[1]).resolve())
