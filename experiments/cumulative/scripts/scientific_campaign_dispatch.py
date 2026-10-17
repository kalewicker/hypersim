"""Scientific dispatch with exact authorization, durable intent and aggregate ceilings."""
import json
import math
from pathlib import Path
import subprocess
import time
from compact_evidence import Store,encode_json,publish,read_bounded,require,sha,sync_dir
from evidence_spool import RESERVATION,JournalView,usage
from scientific_campaign_controller import next_unit
from scientific_launch_binding import authorization,validate,CONTRACT,canonical
from audit_scientific_execution import recover
from audit_scientific_controller import audit_ledger


class ScientificDispatch:
    def __init__(self,controller,spool,binary,workspace,started):
        self.controller=controller;self.spool=spool;self.binary=Path(binary)
        self.workspace=Path(workspace).resolve();self.started=started
        state,_,n=controller.replay();require(n==0 and not spool.states(),'fresh launch only; no restart clock or retry')
        require(controller.root.resolve()==self.workspace/'controller' and spool.root.resolve()==self.workspace/'native_spool','fixed authorized campaign layout')
        self.a=controller.contract['authorization'];self.binary_hash=sha(read_bounded(self.binary,33554432))
        authorization(self.a,self.binary_hash,execute=True)
        require(self.a['campaign_id']==sha(str(self.workspace).encode()),'approval binds one campaign directory')
        require(spool.owner['cap_bytes']==CONTRACT['limits']['storage_bytes'],'authorized spool ceiling')

    def elapsed(self):return max(0,math.ceil((time.monotonic()-self.started)*1000))

    def dispatch(self,r):
        require(sha(read_bounded(self.binary,33554432))==self.binary_hash,'unchanged executable')
        state,_,_=self.controller.replay();unit=next_unit(self.controller.contract,state)
        require(r['execution_allowance']==unit['work_bound'],'exact native funding')
        if unit['unit']=='world':require(r['unit']=='world' and sha(canonical(r))==unit['request_sha256'],'exact next panel member')
        else:
            require(r['unit']=='assays' and r['range']==dict(start=unit['index'],end=unit['end']) and sha(encode_json(r['registry']))==state['registry']['sha256'],'entire frozen registry and next batch')
        limits=self.controller.contract['limits'];elapsed=self.elapsed()
        grant=min(CONTRACT['per_unit_ms'],limits['native_ms']-state['native_ms'],limits['workflow_ms']-elapsed)
        require(grant>0,'aggregate time exhausted')
        admission=self.spool.capacity(RESERVATION+33554432+1048576);outside=usage(self.workspace)-usage(self.spool.root)
        total=admission['charged_bytes']+outside;require(total<=limits['storage_bytes'],'whole campaign storage cap')
        # Validate all native input before publishing the intent. Provisional hashes
        # are replaced with the actual durable ledger hashes before invocation.
        e=dict(profile='cumulative-scientific-execution-binding-v1',request=r,authorization=self.a,controller=dict(owner_sha256=sha(read_bounded(self.controller.root/'contract.json',1048576)),intent_sha256='0'*64,request_sha256=sha(canonical(r)),work_bound=unit['work_bound']))
        validate(e,self.binary_hash,execute=True)
        self.controller.append('intent',dict(unit=unit,native_ms_bound=grant,elapsed_ms=elapsed,storage_charge_bytes=total))
        begin=time.monotonic()
        _,head,_=self.controller.replay();e['controller']['intent_sha256']=head
        audit_ledger(self.controller.root)
        ticket=self.spool.reserve(sha(encode_json(e)));root=self.spool.root/'requests'
        if not root.exists():root.mkdir();sync_dir(self.spool.root)
        path=root/(ticket['unit']+'.json');publish(path,encode_json(e))
        remaining=min(grant-math.ceil((time.monotonic()-begin)*1000),limits['workflow_ms']-self.elapsed())
        require(remaining>0,'deadline reached preparing native dispatch; retain intent')
        try:
            p=subprocess.run([str(self.binary),'--execute',str(path),ticket['directory']],capture_output=True,text=True,timeout=remaining/1000)
            exit_receipt=dict(returncode=p.returncode,stdout=p.stdout,stderr=p.stderr,timed_out=False)
        except subprocess.TimeoutExpired as ex:
            exit_receipt=dict(returncode=None,stdout=str(ex.stdout or ''),stderr=str(ex.stderr or ''),timed_out=True)
        publish(root/(ticket['unit']+'.receipt.json'),encode_json(exit_receipt))
        require(Path(ticket['directory']).is_dir(),'native failed before journal; retain full reservation')
        self.spool.archive(ticket['unit']);s=self.spool.states()[ticket['unit']]
        require(s['kind']=='released','exact compact evidence before receipt')
        store=Store(self.spool.root/'compact'/ticket['unit'],s['index_sha256'],s['seal_sha256']);store.verify_all()
        recovered=recover(JournalView(store),e,self.binary_hash)
        duration=math.ceil((time.monotonic()-begin)*1000);elapsed=self.elapsed()
        require(not exit_receipt['timed_out'] and exit_receipt['returncode']==0 and recovered['status']=='complete' and not recovered['staging'],'incomplete unit; never retry')
        require(duration<=grant and elapsed<limits['workflow_ms'],'unit or campaign time exhausted; retain full reservation')
        require(usage(self.workspace)+self.spool.capacity()['pending_reservation_bytes']<=limits['storage_bytes'],'whole campaign storage cap after archival')
        self.controller.append('receipt',dict(request_sha256=unit['request_sha256'],status='complete',work_lower=recovered['lower'],work_upper=recovered['upper'],native_ms=duration,elapsed_ms=elapsed,evidence_sha256=recovered['head']))
        return recovered
