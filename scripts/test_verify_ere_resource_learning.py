#!/usr/bin/env python3
import copy
import os
from pathlib import Path
import unittest
from unittest.mock import patch
import verify_ere_resource_learning as v

class LearningPreflight(unittest.TestCase):
    def root(self):return Path(os.environ['ERE_RESOURCE_LEARNING_PREFLIGHT'])
    def test_full_preflight(self):self.assertTrue(v.audit_preflight(self.root())['engineering_pass'])
    def test_schedule_and_draws(self):
        slots=v.schedule();self.assertEqual(len(slots),7440)
        for i,s in enumerate(slots):self.assertEqual(s['id'],i)
        for rep in range(4):self.assertEqual([s['draw'] for s in slots[rep*1600:rep*1600+800]],[s['draw'] for s in slots[rep*1600+800:rep*1600+1600]])
        original=v.a.load;bad=original(self.root()/'schedule.json');bad[0]['draw']=.5
        with patch.object(v.a,'load',side_effect=lambda p:bad if Path(p).name=='schedule.json' else original(p)):
            with self.assertRaises(AssertionError):v.audit_preflight(self.root())
    def test_arithmetic_corruptions(self):
        original=v.a.load;data=original(self.root()/'arithmetic.json')
        for field,value in [('after',[0]*4),('mean_gradient',[0]*4),('norm_scale',.5),('contributions',[[0]*4]*8)]:
            bad=copy.deepcopy(data);bad['update'][field]=value
            with patch.object(v.a,'load',side_effect=lambda p:bad if Path(p).name=='arithmetic.json' else original(p)):
                with self.assertRaises(AssertionError):v.audit_preflight(self.root())
        samples=copy.deepcopy(data['samples']);samples[1]['state']=0
        with self.assertRaises(AssertionError):v.update(v.a.THETA,samples)
        samples=copy.deepcopy(data['samples']);samples[1]['utility']=20
        changed=v.update(v.a.THETA,samples);self.assertEqual(changed['after'],v.update(v.a.THETA,data['samples'])['after'])
    def test_actual_credit_corruptions(self):
        root=self.root();refs,h=v.references(root);saved=v.a.load(root/'inputs/state-0.json');receipt=saved['receipt'];identity=h['cells'][0]['measured']['identity']
        for field,value in [('profile','wrong'),('completed_steps',31),('native_advances',0),('seal_applications',0)]:
            bad=copy.deepcopy(receipt);bad[field]=value
            with self.assertRaises(AssertionError):v.credit(bad,identity)
        bad=copy.deepcopy(receipt);bad['summary']['harvested_credits']=2
        with self.assertRaises(AssertionError):v.credit(bad,identity)
    def test_campaign_partial_is_not_complete(self):
        # Full admission cannot be obtained with zero cells or a partial batch.
        original=v.a.load;m=dict(status='complete',native_work_unknown=False,completed_cells=7,native_advances=224,seal_applications=7,optimizer_updates=0)
        with patch.object(v,'custody',return_value=dict(launch_authorized=True)),patch.object(v.a,'load',side_effect=lambda p:m if str(p).endswith('campaign/manifest.json') else original(p)):
            with self.assertRaises(AssertionError):v.audit_campaign(self.root())
    def test_policy_timing_corruptions(self):
        root=self.root();refs,h=v.references(root);saved=v.a.load(root/'inputs/state-0.json')
        cell=dict(slot=v.schedule()[0],policy=saved['policy'],receipt=saved['receipt'],actual_utility=8)
        # Use the recorded probe and draw on the same native graph, independently of a training slot.
        slot=copy.deepcopy(v.schedule()[0]);slot['draw']=.25;cell['slot']=slot
        self.assertEqual(v.audit_cell(cell,slot,v.a.THETA,refs,h)['utility'],8)
        for field,value in [('sampled_action',1),('draw',.75),('input',[1,1,0,0])]:
            bad=copy.deepcopy(cell);bad['policy'][field]=value
            with self.assertRaises(AssertionError):v.audit_cell(bad,slot,v.a.THETA,refs,h)
        bad=copy.deepcopy(cell);bad['receipt']['decision_record']['step']=14
        with self.assertRaises(AssertionError):v.audit_cell(bad,slot,v.a.THETA,refs,h)
if __name__=='__main__':unittest.main()
