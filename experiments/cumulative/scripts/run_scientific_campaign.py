"""Single explicitly authorized scientific launch. Never resumes or retries a campaign."""
import json
from pathlib import Path
import shutil
import subprocess
import sys
import time
from compact_evidence import encode_json,publish,read_bounded,require,sha
from evidence_spool import Spool
from scientific_launch_binding import authorization,owner,request,world_requests,CONTRACT,PACKAGE,parse,PANEL_RAW,PLANNING_RAW
from scientific_campaign_controller import Controller
from scientific_campaign_dispatch import ScientificDispatch
from campaign_measurement import plan,evaluate
from audit_campaign_measurement import check,independent_report
from plan_campaign import analyze_outcomes
from run_readiness import source_paths,write


def launch(auth_path,binary_path,sources_path,out):
    started=time.monotonic();out=Path(out).resolve();binary_path=Path(binary_path)
    a=parse(read_bounded(Path(auth_path),1048576));authorization(a,sha(read_bounded(binary_path,33554432)),execute=True)
    source_raw=read_bounded(Path(sources_path),1048576);pins=parse(source_raw)
    require(sha(source_raw)==a['source_sha256'] and a['campaign_id']==sha(str(out).encode()),'authorized source and one campaign directory')
    source_root=PACKAGE.parents[1]
    require(set(pins)=={str(p.relative_to(source_root)) for p in source_paths()},'complete source closure, no omitted source files')
    for name,digest in pins.items():
        p=Path(name);require(not p.is_absolute() and '..' not in p.parts,'source path')
        require(sha(read_bounded(source_root/p,33554432))==digest,'source changed after authorization')
    # Authorization and complete source verification precede all campaign writes.
    out.mkdir(exist_ok=False);controller=None
    write(out/'manifest.json',dict(status='running',campaign_launch_authorized=True,scientific_acceptance=False,evolution_demonstrated=False,readiness_gate_complete=False))
    try:
        publish(out/'authorization.json',encode_json(a));publish(out/'source_sha256.json',source_raw)
        for name in pins:
            p=out/'source_snapshots'/name;p.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(source_root/name,p)
        binary=out/'cumulative-scientific-launch';shutil.copy2(binary_path,binary)
        controller=Controller.create(out/'controller',owner(a))
        spool=Spool.create(out/'native_spool',CONTRACT['limits']['storage_bytes'])
        bridge=ScientificDispatch(controller,spool,binary,out,started)
        cases=[];all_results=[];planning=parse(PLANNING_RAW)
        for r in world_requests():
            recovered=bridge.dispatch(r)
            cases.append(dict(name=r['world']['world_id'],initial=recovered['results'][0]['row']['before'],rows=[v['row'] for v in recovered['results']],final=recovered['checkpoint']['state']))
        measurement,registry=plan(cases,planning);check(cases,planning,measurement,registry)
        scientific_registry=dict(profile='cumulative-scientific-registry-v1',assays=registry['assays'])
        publish(out/'measurement.json',encode_json(measurement));publish(out/'registry.json',encode_json(scientific_registry))
        controller.append('registry',dict(sha256=sha(encode_json(scientific_registry)),measurement_sha256=sha(encode_json(measurement)),count=len(registry['assays']),slots=len(measurement['samples'])))
        for start in range(0,len(registry['assays']),128):
            recovered=bridge.dispatch(request(registry=scientific_registry,start=start))
            # Full traces remain in verified compact journals; retain only the
            # already audited score/status projection for global measurement.
            for v in recovered['results']:
                score=v['native']['results'][0]
                all_results.append(dict(request=v['request'],native=dict(results=[dict(status=score['status'],score=score['score'])])))
        report=evaluate(measurement,registry,all_results,cases,planning)
        require(report==independent_report(measurement,registry,all_results,cases,planning),'independent measurement report')
        # The reused measurement helper labels its wrapper development_only;
        # preserve that raw output and put the scientific inference in a new wrapper.
        panel=parse(PANEL_RAW)
        outcomes=dict(blocks=47,worlds=[dict(block=w['block'],arm=w['arm'],status=v['status'],qualified=v['qualified']) for w,v in zip(panel['worlds'],report['worlds'])])
        scientific=dict(profile=CONTRACT['profile'],measurement_report=report,outcomes=outcomes,decision=analyze_outcomes(outcomes,planning),claim_scope=CONTRACT['claim_scope'])
        require(bridge.elapsed()<CONTRACT['limits']['workflow_ms'],'workflow deadline before report closure')
        publish(out/'report.json',encode_json(scientific));controller.append('close',dict(reason='complete',decision_sha256=sha(encode_json(scientific))))
        remaining=CONTRACT['limits']['workflow_ms']/1000-(time.monotonic()-started)
        require(remaining>0,'workflow deadline before independent campaign audit')
        result=subprocess.run([sys.executable,'-B',str(PACKAGE/'scripts/audit_scientific_campaign.py'),str(out)],capture_output=True,text=True,timeout=remaining)
        publish(out/'audit_command.json',encode_json(dict(exit_code=result.returncode,stdout=result.stdout,stderr=result.stderr)))
        require(result.returncode==0,'independent scientific campaign audit failed')
        audited=parse(result.stdout.encode());publish(out/'audit.json',encode_json(audited))
        require(time.monotonic()-started<CONTRACT['limits']['workflow_ms']/1000,'campaign workflow deadline')
        write(out/'manifest.json',dict(status='complete',campaign_launch_authorized=True,scientific_acceptance=False,evolution_demonstrated=False,readiness_gate_complete=False,**audited))
    except Exception as ex:
        if controller is not None:
            try:
                state,_,_=controller.replay()
                if not state['closed']:controller.append('close',dict(reason='operational_stop',decision_sha256=None))
            except Exception:pass
        publish(out/'failure.json',encode_json(dict(error=str(ex),outcome='inconclusive_operational_failure',automatic_retry=False)))
        write(out/'manifest.json',dict(status='failed',error=str(ex),campaign_launch_authorized=True,scientific_acceptance=False,evolution_demonstrated=False,readiness_gate_complete=False))
        raise
    finally:
        publish(out/'output_sha256.json',encode_json({str(p.relative_to(out)):sha(p.read_bytes()) for p in sorted(out.rglob('*')) if p.is_file() and p!=out/'output_sha256.json'}))


if __name__=='__main__':
    require(len(sys.argv)==6 and sys.argv[1]=='--launch','usage: --launch AUTHORIZATION BINARY SOURCE_MANIFEST NEW_CAMPAIGN_DIRECTORY')
    launch(*sys.argv[2:])
