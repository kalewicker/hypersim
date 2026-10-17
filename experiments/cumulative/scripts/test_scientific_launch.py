"""Zero-native authorization and synthetic accounting tests."""
import copy
from pathlib import Path
import tempfile
import time
import unittest
from unittest.mock import patch
from scientific_launch_binding import *
from scientific_campaign_controller import Controller,initial,next_unit,transition
from scientific_campaign_dispatch import ScientificDispatch
from evidence_spool import Spool


def preview(binary='a'*64,source='b'*64,campaign='c'*64):
    return dict(profile='cumulative-scientific-authorization-v1',mode='preview',approved=False,approval_reference=None,campaign_id=campaign,launch_sha256=sha(CONTRACT_RAW),panel_sha256=sha(PANEL_RAW),binary_sha256=binary,source_sha256=source,limits=copy.deepcopy(CONTRACT['limits']))


def envelope(r,a):
    return dict(profile='cumulative-scientific-execution-binding-v1',request=r,authorization=a,controller=dict(owner_sha256='d'*64,intent_sha256='e'*64,request_sha256=sha(canonical(r)),work_bound=r['execution_allowance']))


class Tests(unittest.TestCase):
    def test_preview_and_explicit_approval(self):
        a=preview();e=envelope(world_requests()[0],a)
        self.assertEqual(validate(e,'a'*64),project(e['request']))
        with self.assertRaises(ValueError):validate(e,'a'*64,True)
        # In-memory authorization schema check only; never persisted or executed.
        a.update(mode='launch',approved=True,approval_reference='unit-test-schema-only')
        self.assertEqual(validate(e,'a'*64,True),project(e['request']))
        a['approved']=1
        with self.assertRaises(ValueError):validate(e,'a'*64,True)

    def test_fixed_full_panel_owner(self):
        c=owner(preview());self.assertEqual(len(c['worlds']),188);validate_owner(c)
        c['worlds'][0]['world_id']='replacement'
        with self.assertRaises(ValueError):validate_owner(c)

    def test_mutated_approval_and_intent(self):
        e=envelope(world_requests()[0],preview())
        for key,value in [('binary_sha256','0'*64),('launch_sha256','0'*64),('panel_sha256','0'*64),('source_sha256','invalid'),('approved',True)]:
            x=copy.deepcopy(e);x['authorization'][key]=value
            with self.assertRaises(ValueError):validate(x,'a'*64)
        for key,value in [('request_sha256','0'*64),('intent_sha256','bad'),('work_bound',249),('work_bound',True)]:
            x=copy.deepcopy(e);x['controller'][key]=value
            with self.assertRaises(ValueError):validate(x,'a'*64)

    def test_nonzero_global_assay_offset(self):
        registry=dict(profile='cumulative-scientific-registry-v1',assays=[dict(id=i,role='intact',stages=[0]) for i in range(129)])
        r=request(registry=registry,start=128);p=validate(envelope(r,preview()),'a'*64)
        self.assertEqual((p['start'],p['end'],p['execution_allowance']),(128,129,20));self.assertEqual(p['items'][128]['id'],128)

    def test_pending_retains_all_worlds(self):
        c=owner(preview());s=initial(c)
        s=transition(c,s,dict(kind='intent',value=dict(unit=next_unit(c,s),native_ms_bound=30000,elapsed_ms=0,storage_charge_bytes=200000000)))
        with self.assertRaises(ValueError):next_unit(c,s)
        s=transition(c,s,dict(kind='close',value=dict(reason='operational_stop',decision_sha256=None)))
        self.assertEqual(s['rewrites_upper'],250);self.assertEqual(s['worlds'][0]['status'],'unknown')
        self.assertEqual(sum(w['status']=='not_started' for w in s['worlds']),187)

    def test_preview_cannot_acquire_dispatcher(self):
        with tempfile.TemporaryDirectory(dir='/private/tmp') as tmp:
            root=Path(tmp);binary=root/'binary';binary.write_bytes(b'nonexecutable test stub')
            a=preview(sha(binary.read_bytes()),campaign=sha(str(root).encode()))
            ctl=Controller.create(root/'controller',owner(a));spool=Spool.create(root/'native_spool',CONTRACT['limits']['storage_bytes'])
            with patch('scientific_campaign_dispatch.subprocess.run') as run:
                with self.assertRaises(ValueError):ScientificDispatch(ctl,spool,binary,root,time.monotonic())
                run.assert_not_called()
            self.assertEqual(ctl.replay()[2],0)

    def test_mocked_child_failure_cannot_retry(self):
        with tempfile.TemporaryDirectory(dir='/private/tmp') as tmp:
            root=Path(tmp);binary=root/'binary';binary.write_bytes(b'nonexecutable test stub')
            a=preview(sha(binary.read_bytes()),campaign=sha(str(root).encode()))
            a.update(mode='launch',approved=True,approval_reference='temporary mocked invocation only')
            ctl=Controller.create(root/'controller',owner(a));spool=Spool.create(root/'native_spool',CONTRACT['limits']['storage_bytes'])
            dispatch=ScientificDispatch(ctl,spool,binary,root,time.monotonic())
            def fail(*args,**kwargs):
                self.assertIsNotNone(Controller(ctl.root).replay()[0]['pending'])
                raise RuntimeError('mock failure before process creation')
            with patch('scientific_campaign_dispatch.subprocess.run',side_effect=fail) as run:
                with self.assertRaises(RuntimeError):dispatch.dispatch(world_requests()[0])
                with self.assertRaises(ValueError):dispatch.dispatch(world_requests()[0])
                self.assertEqual(run.call_count,1)
            self.assertEqual(ctl.replay()[0]['rewrites_upper'],250)


if __name__=='__main__':unittest.main(verbosity=2)
