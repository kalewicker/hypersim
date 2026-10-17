"""Resource and failure boundary tests; mocked invocation performs no native work."""
import copy
import json
from pathlib import Path
import tempfile
import time
import unittest
from unittest.mock import patch
from campaign_controller import Controller, FLAGS, PROFILE
from compact_evidence import encode_json,sha
from evidence_spool import Spool
from controlled_executor_dispatch import ControlledDispatch,request


class Tests(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory(dir='/private/tmp');self.root=Path(self.tmp.name)
        self.binary=self.root/'binary';self.binary.write_bytes(b'nonexecutable test stub')
        self.route=dict(name='endogenous',seed=2026101001,unit='world',execution_allowance=250,items=[],spec={},fault=None)
        self.cr=encode_json(dict(profile='cumulative-executor-controller-readiness-v1',campaign_launch_authorized=False,routes=[self.route],limits=dict(per_unit_ms=30000)))
        self.owner=dict(profile=PROFILE,purpose='saved_development',worlds=[dict(world_id='development/endogenous',request_sha256=sha(encode_json(request(self.route,self.cr,sha(self.binary.read_bytes())))),work_bound=250)],limits=dict(rewrites=1200,native_ms=180000,workflow_ms=1200000,storage_bytes=268435456),slots_per_world=10,batch_size=128,assay_work_bound=20,maximum_assays=78960,**FLAGS)
        self.controller=Controller.create(self.root/'controller',self.owner);self.spool=Spool.create(self.root/'spool',268435456)
        self.bridge=ControlledDispatch(self.controller,self.spool,self.binary,self.cr,self.root,time.monotonic())

    def tearDown(self):self.tmp.cleanup()

    def refused(self,route=None):
        with patch('controlled_executor_dispatch.invoke') as invoke:
            with self.assertRaises(ValueError):self.bridge.dispatch(route or self.route)
            invoke.assert_not_called()
        self.assertEqual(list((self.controller.root/'events').iterdir()),[])

    def test_clock(self):self.bridge.started-=1201;self.refused()

    def test_external_storage_is_charged(self):
        # Synthetic usage is explicitly a boundary test, not measured feasibility.
        from evidence_spool import usage
        def measured(p):return 268435456 if Path(p)==self.root else usage(p)
        with patch('controlled_executor_dispatch.usage',side_effect=measured):self.refused()

    def test_spool_limit(self):
        self.bridge.spool=Spool.create(self.root/'tiny',4194304);self.refused()

    def test_identity(self):
        for k,v in [('seed',1),('execution_allowance',249),('name','frozen'),('spec',{'ticks':17})]:
            r=copy.deepcopy(self.route);r[k]=v;self.refused(r)
        self.binary.write_bytes(b'changed');self.refused()

    def test_rewrite_cap(self):
        self.controller.contract['limits']['rewrites']=249;self.refused()

    def test_native_time_cap(self):
        state,head,count=self.controller.replay();state['native_ms']=180000
        with patch.object(self.controller,'replay',return_value=(state,head,count)):self.refused()

    def test_failure_is_durable_and_cannot_retry_or_restart(self):
        def failed(*args):
            state,_,count=Controller(self.controller.root).replay()
            self.assertEqual(count,1);self.assertEqual(state['rewrites_upper'],250)
            self.assertIsNotNone(state['pending']);raise RuntimeError('declared mock failure')
        with patch('controlled_executor_dispatch.invoke',side_effect=failed) as invoke:
            with self.assertRaises(RuntimeError):self.bridge.dispatch(self.route)
            with self.assertRaises(ValueError):self.bridge.dispatch(self.route)
            self.assertEqual(invoke.call_count,1)
        with self.assertRaises(ValueError):ControlledDispatch(self.controller,self.spool,self.binary,self.cr,self.root,time.monotonic())
        state=self.controller.append('close',dict(reason='operational_stop',decision_sha256=None))
        self.assertEqual(state['worlds'][0]['status'],'unknown');self.assertEqual(state['rewrites_upper'],250)


if __name__=='__main__':unittest.main(verbosity=2)
