"""Freeze, fund and seal the bounded development measurement milestone."""
import json
from pathlib import Path
import shutil
import subprocess
import sys
import time
from run_readiness import ROOT, PACKAGE, FALSE, digest, write, source_paths
from run_campaign_adapter import custody
from compact_evidence import Store, source_inventory, pack, sha
from evidence_spool import Spool, JournalView, run_development_unit
from audit_durability import read
from audit_campaign_adapter import recover_values
from audit_campaign_measurement import worlds, check
from campaign_measurement import plan, evaluate
from plan_campaign import panel


def run(out):
    started=time.monotonic();out.mkdir(parents=True,exist_ok=False)
    (out/'commands').mkdir()
    c=json.loads((PACKAGE/'measurement_contract.json').read_text())
    manifest=dict(profile=c['profile'],status='running',engineering_measurement_pass=False,native_work_lower=0,native_work_upper=0,**FALSE)
    write(out/'manifest.json',manifest);charged=0;native_seconds=0
    def preserve():
        custody()
        for key in ['parent','adapter_parent','storage_parent']:
            p=c[key];source_inventory(ROOT/p['archive'],p['seal_sha256'])
        previous=json.loads((ROOT/c['parent']['archive']/'source_sha256.json').read_text())
        if any(digest(ROOT/n)!=h for n,h in previous.items()):raise ValueError('historical source modified')
    try:
        preserve()
        pins={str(p.relative_to(ROOT)):digest(p) for p in source_paths()};write(out/'source_sha256.json',pins)
        for n in pins:
            dest=out/'source_snapshots'/n;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(ROOT/n,dest)
        write(out/'contract.json',c)
        campaign=json.loads((PACKAGE/'campaign_planning_contract.json').read_text());write(out/'campaign_contract.json',campaign)
        frozen=out/'source_snapshots/experiments/cumulative/scripts'
        tests=subprocess.run([sys.executable,'-B',str(frozen/'test_campaign_measurement.py'),str(ROOT)],capture_output=True,text=True,timeout=120)
        write(out/'commands/tests.json',dict(exit_code=tests.returncode,stdout=tests.stdout,stderr=tests.stderr))
        if tests.returncode:raise ValueError(tests.stderr)
        adapter=ROOT/c['adapter_parent']['archive']
        pack(adapter,out/'adapter_evidence',c['adapter_parent']['seal_sha256'])
        identity=dict(index_sha256=digest(out/'adapter_evidence/index.json'));write(out/'adapter_store.json',identity)
        store=Store(out/'adapter_evidence',identity['index_sha256'],c['adapter_parent']['seal_sha256']);store.verify_all()
        cases=worlds(store.root);measurement,registry=plan(cases,campaign);check(cases,campaign,measurement,registry)
        write(out/'measurement.json',measurement);write(out/'registry.json',registry)
        proposed=panel(campaign,47)
        write(out/'prospective_pending.json',dict(panel=proposed,samples=[dict(world_id=w['world_id'],census=t,slot=s,status='not_started') for w in proposed['worlds'] for t in campaign['design']['census_ticks'] for s in range(2)]))
        write(out/'remaining_launch_requirements.json',dict(campaign_preregistration_complete=False,requirements=[
            'Versioned scientific native profile restricted to the fixed 47-block panel; current binary permits only development seed 2026101001.',
            'Complete campaign controller with immutable world identities, global measurement registry, aggregate native/time/storage accounting, and unknown outcomes on interruption.',
            'Freeze final campaign authorization and resource ceilings before any scientific world; current 250-rewrite allowance is development-only.',
            'Freeze claim scope: replicated worlds at the fixed diagnostic environment, or a separately preregistered independent confirmation environment; do not treat a changed assay seed as independent confirmation.'
        ],**FALSE))
        # Fixed denominator and complete registry are durable before the first assay.
        if len(registry['assays'])*20>c['limits']['native_rewrites']:raise ValueError('complete assay reservation exceeds development funding')
        spool=Spool.create(out/'native_spool',c['limits']['spool_cap_bytes'])
        binary=out/'cumulative-campaign';shutil.copy2(adapter/'cumulative-campaign',binary)
        base=json.loads((adapter/'inputs/assays_0.json').read_text())
        base.update(native_budget=c['limits']['native_rewrites'],registry=dict(path=str(out/'registry.json'),sha256=digest(out/'registry.json')),source=None,fault=None)
        base['source_identity']=dict(binary_sha256=digest(binary),source_sha256=digest(out/'source_sha256.json'))
        adapter_contract=json.loads((adapter/'contract.json').read_text());results=[]
        for start in range(0,len(registry['assays']),128):
            request={**base,'range':dict(start=start,end=min(start+128,len(registry['assays']))),'execution_allowance':c['limits']['native_rewrites']-charged}
            begin=time.monotonic()
            ticket,frozen_request,receipt=run_development_unit(spool,request,binary,c['limits']['native_seconds']-native_seconds)
            native_seconds+=time.monotonic()-begin
            state=spool.states()[ticket['unit']];compact=Store(spool.root/'compact'/ticket['unit'],state['index_sha256'],state['seal_sha256'])
            values,head,staging=read(JournalView(compact))
            recovered=recover_values(values,head,staging,frozen_request,ticket['request_sha256'],adapter_contract,registry['assays'])
            charged+=recovered['here_upper']
            if receipt['returncode'] or receipt['timed_out'] or recovered['status']!='complete' or recovered['unknown']:raise ValueError('incomplete assay batch; no retry')
            results.extend(recovered['results'])
        write(out/'report.json',evaluate(measurement,registry,results,cases,campaign))
        audited=subprocess.run([sys.executable,'-B',str(frozen/'audit_campaign_measurement.py'),str(out)],capture_output=True,text=True,timeout=120)
        write(out/'commands/independent_audit.json',dict(exit_code=audited.returncode,stdout=audited.stdout,stderr=audited.stderr))
        if audited.returncode:raise ValueError(audited.stderr)
        audit=json.loads(audited.stdout);write(out/'audit.json',audit)
        preserve()
        if any(digest(ROOT/n)!=h for n,h in pins.items()):raise ValueError('source changed after freeze')
        write(out/'preservation.json',dict(previous_archive_roots_preserved=14,previous_source_pins_preserved=160,historical_raw_files_removed=0))
        manifest.update(audit,status='complete',source_files=len(pins),native_seconds=native_seconds,elapsed_seconds=time.monotonic()-started,**FALSE)
        if charged>c['limits']['native_rewrites'] or native_seconds>c['limits']['native_seconds'] or time.monotonic()-started>c['limits']['workflow_seconds']:raise ValueError('bounded execution limit')
        write(out/'manifest.json',manifest)
        if sum(p.stat().st_size for p in out.rglob('*') if p.is_file())>c['limits']['output_bytes']:raise ValueError('output cap')
    except Exception as e:
        manifest.update(status='failed',error=str(e),engineering_measurement_pass=False,native_work_lower=charged,native_work_upper=c['limits']['native_rewrites'],native_work_unknown=True)
        write(out/'manifest.json',manifest)
        write(out/'output_sha256.json',{str(p.relative_to(out)):digest(p) for p in sorted(out.rglob('*')) if p.is_file() and p!=out/'output_sha256.json'})
        raise
    write(out/'output_sha256.json',{str(p.relative_to(out)):digest(p) for p in sorted(out.rglob('*')) if p.is_file()})
    print(json.dumps(dict(output=str(out),seal_sha256=digest(out/'output_sha256.json'),**manifest),indent=2))


if __name__=='__main__':run(Path(sys.argv[1]).resolve())
