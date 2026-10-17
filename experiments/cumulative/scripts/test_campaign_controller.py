import copy
import json
from pathlib import Path
import tempfile
import unittest
from campaign_controller import Controller, FLAGS, PROFILE, initial, next_unit, transition
from audit_campaign_controller import audit_ledger


def contract():
    return dict(profile=PROFILE,purpose='synthetic_full_scale',worlds=[dict(world_id='fixed/0',request_sha256='a'*64,work_bound=250)],
                limits=dict(rewrites=2000000,native_ms=10800000,workflow_ms=21600000,storage_bytes=8589934592),
                slots_per_world=10,batch_size=128,assay_work_bound=20,maximum_assays=78960,**FLAGS)


def intent(c,s):
    return dict(unit=next_unit(c,s),native_ms_bound=10,elapsed_ms=s['elapsed_ms'],storage_charge_bytes=123006976)


def receipt(s,status='complete'):
    return dict(request_sha256=s['pending']['unit']['request_sha256'],status=status,work_lower=2,work_upper=2,
                native_ms=1,elapsed_ms=s['elapsed_ms']+1,evidence_sha256='b'*64)


def apply(c,s,k,v):return transition(c,s,dict(kind=k,value=v))


class ControllerTests(unittest.TestCase):
    def test_scientific_purpose_and_claims_refused(self):
        for change in [dict(purpose='scientific'),dict(campaign_launch_authorized=True),dict(slots_per_world=9)]:
            c=contract();c.update(change)
            with self.assertRaises(ValueError):initial(c)

    def test_fixed_identity_order_and_typed_caps(self):
        c=contract();s=initial(c)
        for mutate in [lambda v:v['unit'].update(request_sha256='c'*64),lambda v:v['unit'].update(index=1),
                       lambda v:v.update(native_ms_bound=True),lambda v:v.update(elapsed_ms=-1)]:
            v=intent(c,s);mutate(v)
            with self.assertRaises(ValueError):apply(c,s,'intent',v)

    def test_all_four_pre_dispatch_resource_limits(self):
        for name,value in [('rewrites',249),('native_ms',9),('workflow_ms',1),('storage_bytes',1)]:
            c=contract();c['limits'][name]=value;s=initial(c);v=intent(c,s)
            if name=='workflow_ms':v['elapsed_ms']=1
            with self.assertRaises(ValueError):apply(c,s,'intent',v)

    def test_pending_restart_never_redispatches(self):
        with tempfile.TemporaryDirectory(dir='/private/tmp') as tmp:
            ctl=Controller.create(Path(tmp)/'controller',contract());s=ctl.append('intent',intent(ctl.contract,initial(ctl.contract)))
            reopened=Controller(ctl.root);other,_,_=reopened.replay();self.assertEqual(other,s)
            with self.assertRaises(ValueError):next_unit(ctl.contract,other)
            end=reopened.append('close',dict(reason='operational_stop',decision_sha256=None))
            self.assertEqual(end['rewrites_upper'],250);self.assertEqual(end['worlds'][0]['status'],'unknown')
            self.assertEqual(audit_ledger(ctl.root)['accounted_rewrites_upper'],250)
            with self.assertRaises(ValueError):reopened.append('close',dict(reason='operational_stop',decision_sha256=None))

    def test_unknown_receipt_keeps_reservation_and_stops(self):
        c=contract();s=initial(c);s=apply(c,s,'intent',intent(c,s));s=apply(c,s,'receipt',receipt(s,'unknown'))
        self.assertEqual((s['rewrites_lower'],s['rewrites_upper'],s['native_ms']),(2,250,10))
        with self.assertRaises(ValueError):next_unit(c,s)
        with self.assertRaises(ValueError):apply(c,s,'close',dict(reason='complete',decision_sha256='c'*64))

    def test_receipt_identity_overrun_and_clock(self):
        c=contract();s=initial(c);s=apply(c,s,'intent',intent(c,s))
        for update in [dict(request_sha256='d'*64),dict(work_upper=251),dict(work_upper=3),dict(native_ms=11),dict(work_lower=True),dict(elapsed_ms=-1)]:
            r=receipt(s);r.update(update)
            with self.assertRaises(ValueError):apply(c,s,'receipt',r)

    def test_registry_barrier_and_entire_denominator_funding(self):
        c=contract();s=initial(c);r=dict(sha256='c'*64,measurement_sha256='d'*64,count=129,slots=10)
        with self.assertRaises(ValueError):apply(c,s,'registry',r)
        s=apply(c,s,'intent',intent(c,s));s=apply(c,s,'receipt',receipt(s))
        small=copy.deepcopy(c);small['limits']['rewrites']=2581
        with self.assertRaises(ValueError):apply(small,s,'registry',r)
        s=apply(c,s,'registry',r)
        with self.assertRaises(ValueError):apply(c,s,'registry',r)
        s=apply(c,s,'intent',intent(c,s));self.assertEqual(s['pending']['unit']['end'],128)
        s=apply(c,s,'receipt',receipt(s));self.assertEqual(next_unit(c,s)['index'],128)
        self.assertEqual(next_unit(c,s)['end'],129)
        with self.assertRaises(ValueError):apply(c,s,'close',dict(reason='complete',decision_sha256='a'*64))

    def test_torn_and_changed_chains_fail_closed(self):
        for corrupt in ['torn','hash']:
            with tempfile.TemporaryDirectory(dir='/private/tmp') as tmp:
                ctl=Controller.create(Path(tmp)/'controller',contract());ctl.append('intent',intent(ctl.contract,initial(ctl.contract)))
                if corrupt=='torn':(ctl.root/'events/.staging-00000001.json').write_text('{')
                else:
                    p=ctl.root/'events/00000000.json';v=json.loads(p.read_text());v['previous']='0'*64;p.write_text(json.dumps(v))
                with self.assertRaises(ValueError):ctl.replay()
                with self.assertRaises(ValueError):audit_ledger(ctl.root)


if __name__=='__main__':unittest.main(verbosity=2)
