import copy
from pathlib import Path
import tempfile
import time
import unittest
from unittest.mock import patch
from campaign_controller import Controller, FLAGS, PROFILE
from compact_evidence import encode_json, sha
from evidence_spool import Spool, WRITER_LIMIT
from live_campaign_dispatch import DevelopmentDispatch


class DispatchTests(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory(dir='/private/tmp');self.root=Path(self.tmp.name)
        self.binary=self.root/'binary';self.binary.write_bytes(b'nonexecutable-test-only')
        self.request=dict(profile='cumulative-campaign-adapter-readiness-v1',purpose='development',seed=2026101001,
                          unit='world',spec={},registry=None,range=None,source=None,fault=None,native_budget=1200,execution_allowance=250,
                          journal_byte_allowance=WRITER_LIMIT,source_identity=dict(binary_sha256=sha(self.binary.read_bytes()),source_sha256='0'*64))
        c=dict(profile=PROFILE,purpose='saved_development',worlds=[dict(world_id='development',request_sha256=sha(encode_json(self.request)),work_bound=250)],
               limits=dict(rewrites=1200,native_ms=180000,workflow_ms=1200000,storage_bytes=268435456),
               slots_per_world=10,batch_size=128,assay_work_bound=20,maximum_assays=78960,**FLAGS)
        self.controller=Controller.create(self.root/'controller',c);self.spool=Spool.create(self.root/'spool',268435456)
        self.bridge=DevelopmentDispatch(self.controller,self.spool,self.binary,{},time.monotonic())

    def tearDown(self):self.tmp.cleanup()

    def test_native_failure_preserves_intent_and_no_second_dispatch(self):
        with patch('live_campaign_dispatch.run_development_unit',side_effect=RuntimeError('injected child failure')) as run:
            with self.assertRaises(RuntimeError):self.bridge.dispatch(self.request)
            self.assertEqual(run.call_count,1)
            s,_,_=self.controller.replay();self.assertEqual(s['rewrites_upper'],250);self.assertIsNotNone(s['pending'])
            with self.assertRaises(ValueError):self.bridge.dispatch(self.request)
            self.assertEqual(run.call_count,1)
        s=self.controller.append('close',dict(reason='operational_stop',decision_sha256=None))
        self.assertEqual(s['worlds'][0]['status'],'unknown')

    def test_durable_intent_exists_when_executor_is_called(self):
        def invoked(*args):
            state,_,count=Controller(self.controller.root).replay()
            self.assertEqual(count,1);self.assertIsNotNone(state['pending'])
            self.assertEqual(state['pending']['unit']['request_sha256'],sha(encode_json(self.request)))
            raise RuntimeError('stop before native work')
        with patch('live_campaign_dispatch.run_development_unit',side_effect=invoked):
            with self.assertRaises(RuntimeError):self.bridge.dispatch(self.request)
        with self.assertRaises(ValueError):DevelopmentDispatch(self.controller,self.spool,self.binary,{},time.monotonic())

    def test_real_clock_exhaustion_stops_before_intent(self):
        self.bridge.started=time.monotonic()-1201
        with patch('live_campaign_dispatch.run_development_unit') as run:
            with self.assertRaises(ValueError):self.bridge.dispatch(self.request)
            run.assert_not_called()
        self.assertEqual(self.controller.replay()[2],0)

    def test_actual_storage_admission_stops_before_intent(self):
        self.bridge.spool=Spool.create(self.root/'small',4*1024**2)
        with patch('live_campaign_dispatch.run_development_unit') as run:
            with self.assertRaises(ValueError):self.bridge.dispatch(self.request)
            run.assert_not_called()
        self.assertEqual(self.controller.replay()[2],0)

    def test_changed_identity_budget_seed_or_binary_never_dispatches(self):
        for change in [dict(seed=42),dict(purpose='scientific'),dict(execution_allowance=251),dict(spec=dict(ticks=15)),dict(journal_byte_allowance=1)]:
            r=copy.deepcopy(self.request);r.update(change)
            with patch('live_campaign_dispatch.run_development_unit') as run:
                with self.assertRaises(ValueError):self.bridge.dispatch(r)
                run.assert_not_called()
        self.binary.write_bytes(b'changed')
        with self.assertRaises(ValueError):self.bridge.dispatch(self.request)
        self.assertEqual(self.controller.replay()[2],0)


if __name__=='__main__':unittest.main(verbosity=2)
