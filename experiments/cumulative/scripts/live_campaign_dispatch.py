"""One-process development dispatcher for the sealed controller and native adapter.

Reopened controllers cannot acquire a fresh clock epoch or dispatch again. A crash
leaves a funded intent; evidence can be audited and the attempt closed separately.
"""
import math
import time
from pathlib import Path
from compact_evidence import Store, encode_json, read_bounded, require, sha
from campaign_controller import next_unit
from evidence_spool import RESERVATION, WRITER_LIMIT, JournalView, run_development_unit
from audit_durability import read
from audit_campaign_adapter import recover_values, registry as validate_registry


class DevelopmentDispatch:
    def __init__(self,controller,spool,binary,adapter_contract,started,per_unit_ms=30000):
        state,_,count=controller.replay()
        require(count==0 and controller.contract['purpose']=='saved_development', 'fresh development controller only; no restart dispatch')
        require(type(per_unit_ms) is int and 0<per_unit_ms<=300000, 'unit time ceiling')
        require(not spool.states(), 'fresh owned native spool')
        self.controller=controller;self.spool=spool;self.binary=Path(binary)
        self.adapter=adapter_contract;self.started=started;self.per_unit_ms=per_unit_ms

    def elapsed(self):
        return max(0,math.ceil((time.monotonic()-self.started)*1000))

    def dispatch(self,request,registry=None):
        state,_,_=self.controller.replay();unit=next_unit(self.controller.contract,state)
        limits=self.controller.contract['limits'];raw=encode_json(request)
        require(request['profile']=='cumulative-campaign-adapter-readiness-v1' and request['purpose']=='development' and request['seed']==2026101001, 'development native identity only')
        require(request['fault'] is None and request['source'] is None and request['journal_byte_allowance']==WRITER_LIMIT, 'fresh fixed native route and writer allowance')
        require(request['native_budget']==limits['rewrites'] and request['execution_allowance']==unit['work_bound'], 'exact funded native allowance')
        require(sha(read_bounded(self.binary,32*1024**2))==request['source_identity']['binary_sha256'], 'executing binary hash')
        items=[]
        if unit['unit']=='world':
            require(request['unit']=='world' and sha(raw)==unit['request_sha256'] and registry is None, 'exact frozen world request')
        else:
            require(request['unit']=='assays' and request['range']==dict(start=unit['index'],end=unit['end']), 'exact frozen assay range')
            require(sha(read_bounded(Path(request['registry']['path']),32*1024**2))==request['registry']['sha256']==state['registry']['sha256'], 'live registry file identity')
            require(registry is not None and sha(encode_json(registry))==state['registry']['sha256'], 'canonical full registry supplied to native audit')
            items=validate_registry(registry,self.adapter)
        elapsed=self.elapsed()
        grant=min(self.per_unit_ms,limits['native_ms']-state['native_ms'],limits['workflow_ms']-elapsed)
        require(grant>0, 'real native/workflow clock exhausted before dispatch')
        admission=self.spool.capacity(RESERVATION+1048576)
        self.controller.append('intent',dict(unit=unit,native_ms_bound=grant,elapsed_ms=elapsed,storage_charge_bytes=admission['charged_bytes']))
        # Nothing may invoke the child before the controller's durable intent.
        begin=time.monotonic()
        ticket,frozen,exit_receipt=run_development_unit(self.spool,request,self.binary,grant/1000)
        duration=math.ceil((time.monotonic()-begin)*1000)
        require(frozen==request and ticket['request_sha256']==sha(raw), 'spool preserves frozen request')
        spooled=self.spool.states()[ticket['unit']]
        require(spooled['kind']=='released', 'durable compact evidence before result admission')
        store=Store(self.spool.root/'compact'/ticket['unit'],spooled['index_sha256'],spooled['seal_sha256']);store.verify_all()
        values,head,staging=read(JournalView(store))
        recovered=recover_values(values,head,staging,frozen,ticket['request_sha256'],self.adapter,items)
        require(not exit_receipt['timed_out'] and exit_receipt['returncode']==0 and recovered['status']=='complete' and not recovered['unknown'] and not staging, 'incomplete native unit; retain intent and never retry')
        require(duration<=grant, 'native plus archival time exceeded grant; preserve unknown accounting')
        receipt=dict(request_sha256=unit['request_sha256'],status='complete',work_lower=recovered['here_lower'],work_upper=recovered['here_upper'],
                     native_ms=duration,elapsed_ms=self.elapsed(),evidence_sha256=head)
        self.controller.append('receipt',receipt)
        return dict(unit=ticket['unit'],logical_request_sha256=unit['request_sha256'],native_request_sha256=ticket['request_sha256'],
                    admission=admission,native_and_archive_ms=duration,recovered=recovered)
