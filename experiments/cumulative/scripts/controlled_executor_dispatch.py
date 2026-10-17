"""One-shot controller bridge for compiled development routes. No scientific launch."""
import json
import math
from pathlib import Path
import subprocess
import time
from compact_evidence import Store, encode_json, publish, read_bounded, require, sha, sync_dir
from campaign_controller import next_unit
from evidence_spool import RESERVATION, JournalView, usage
from audit_scientific_executor import recover


def request(route, contract_raw, binary_hash):
    return dict(profile='cumulative-controlled-executor-request-v1',purpose='development',
                contract_sha256=sha(contract_raw),binary_sha256=binary_hash,route=route)


def invoke(spool, frozen, binary, timeout):
    """One invocation; even failed or partial journals are retained and compacted."""
    ticket=spool.reserve(sha(encode_json(frozen)))
    root=spool.root/'requests'
    if not root.exists():root.mkdir();sync_dir(spool.root)
    publish(root/(ticket['unit']+'.json'),encode_json(frozen))
    try:
        r=subprocess.run([str(binary),'--development',frozen['route']['name'],ticket['directory']],capture_output=True,text=True,timeout=timeout)
        receipt=dict(returncode=r.returncode,stdout=r.stdout,stderr=r.stderr,timed_out=False)
    except subprocess.TimeoutExpired as e:
        receipt=dict(returncode=None,stdout=str(e.stdout or ''),stderr=str(e.stderr or ''),timed_out=True)
    publish(root/(ticket['unit']+'.receipt.json'),encode_json(receipt))
    require(Path(ticket['directory']).is_dir(),'no native journal; preserve reservation and stop')
    spool.archive(ticket['unit'])
    return ticket,receipt


class ControlledDispatch:
    def __init__(self,controller,spool,binary,contract_raw,workspace,started):
        state,_,count=controller.replay()
        require(count==0 and controller.contract['purpose']=='saved_development','fresh development controller only; no clock reset or restart')
        require(not spool.states(),'fresh spool required')
        self.controller=controller;self.spool=spool;self.binary=Path(binary)
        self.contract_raw=contract_raw;self.contract=json.loads(contract_raw)
        require(self.contract['profile']=='cumulative-executor-controller-readiness-v1','compiled route contract profile')
        require(self.contract['campaign_launch_authorized'] is False,'development only')
        self.workspace=Path(workspace).resolve();self.started=started
        require(self.spool.root.resolve().is_relative_to(self.workspace) and self.controller.root.resolve().is_relative_to(self.workspace),'single accounted workspace')
        self.binary_hash=sha(read_bounded(self.binary,32*1024**2))

    def elapsed(self):return max(0,math.ceil((time.monotonic()-self.started)*1000))

    def dispatch(self,route,registry=None):
        state,_,_=self.controller.replay();unit=next_unit(self.controller.contract,state)
        require(route in self.contract['routes'] and route['seed']==2026101001,'exact compiled development route')
        require(sha(read_bounded(self.binary,32*1024**2))==self.binary_hash,'unchanged native binary')
        frozen=request(route,self.contract_raw,self.binary_hash);native_hash=sha(encode_json(frozen))
        require(route['execution_allowance']==unit['work_bound'],'native and controller reservation identity')
        if unit['unit']=='world':
            require(route['unit']=='world' and native_hash==unit['request_sha256'] and registry is None,'exact next world')
        else:
            require(route['unit']=='assays' and registry is not None and unit['index']==0 and unit['end']==len(route['items']),'fixed complete development batch')
            require(sha(encode_json(registry))==state['registry']['sha256'] and registry['assays']==route['items'],'measured frozen registry equals compiled assay route')
        limits=self.controller.contract['limits'];elapsed=self.elapsed()
        grant=min(self.contract['limits']['per_unit_ms'],limits['native_ms']-state['native_ms'],limits['workflow_ms']-elapsed)
        require(grant>0,'native/workflow time exhausted')
        spool_admission=self.spool.capacity(RESERVATION+1048576)
        outside=usage(self.workspace)-usage(self.spool.root)
        total=spool_admission['charged_bytes']+outside
        require(total<=limits['storage_bytes'],'whole workspace storage exhausted before native intent')
        self.controller.append('intent',dict(unit=unit,native_ms_bound=grant,elapsed_ms=elapsed,storage_charge_bytes=total))
        begin=time.monotonic()
        ticket,exit_receipt=invoke(self.spool,frozen,self.binary,grant/1000)
        s=self.spool.states()[ticket['unit']]
        require(s['kind']=='released','verified compact evidence required')
        store=Store(self.spool.root/'compact'/ticket['unit'],s['index_sha256'],s['seal_sha256']);store.verify_all()
        recovered=recover(JournalView(store),route,self.contract_raw,self.binary_hash)
        duration=math.ceil((time.monotonic()-begin)*1000);elapsed=self.elapsed()
        require(not exit_receipt['timed_out'] and exit_receipt['returncode']==0 and recovered['status']=='complete' and not recovered['staging'],'unknown execution; full intent remains charged and cannot retry')
        require(duration<=grant and elapsed<limits['workflow_ms'],'native/archive/audit or workflow deadline exceeded; retain intent')
        require(usage(self.workspace)+self.spool.capacity()['pending_reservation_bytes']<=limits['storage_bytes'],'post-dispatch whole workspace cap')
        self.controller.append('receipt',dict(request_sha256=unit['request_sha256'],status='complete',work_lower=recovered['lower'],work_upper=recovered['upper'],native_ms=duration,elapsed_ms=elapsed,evidence_sha256=recovered['head']))
        return dict(unit=ticket['unit'],logical_request_sha256=unit['request_sha256'],native_request_sha256=native_hash,spool_admission=spool_admission,outside_spool_bytes=outside,total_storage_charge_bytes=total,native_archive_audit_ms=duration,recovered=recovered)
