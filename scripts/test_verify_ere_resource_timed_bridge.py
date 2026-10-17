#!/usr/bin/env python3
import copy
import json
import os
from pathlib import Path
import shutil
import tempfile
import unittest
from unittest.mock import patch
import verify_ere_resource_timed_bridge as v

def save(p,x):p.write_text(json.dumps(x))
def synthetic(root,source):
    """Construct arithmetic/accounting test data only; never reported as native."""
    shutil.copytree(source/'inputs',root/'inputs');(root/'native').mkdir()
    slots=v.schedule();save(root/'native/schedule.json',slots);statuses=[]
    for slot in slots:
        i=slot['id']
        if i>=32:
            steps=0 if i==32 else 13;failure=dict(message=v.MESSAGES[i-32],native_advances=steps,seal_applications=int(steps>0),decision_calls=int(steps>0))
            save(root/f'native/cell-{i:04}.json',dict(slot=slot,failure=failure));statuses.append(dict(id=i,status='expected_refusal',failure=failure));continue
        base=v.a.load(source/f'inputs/cell-{slot["state"]*2:04}.json');o=v.a.projection(base['allocation_observation']['inventory'])
        x=v.input_for(o,slot['mode']);prob=v.a.probability(v.THETA,x);sampled=int(slot['draw']>=prob[0]);request=['defend','harvest'][sampled] if o['wallet']==1 else None
        old=v.a.load(source/f'inputs/cell-{slot["state"]*2+sampled:04}.json');r=copy.deepcopy(old);decision=dict(step=13,request=request)
        before=v.flags(old['table_catalogue'][old['timeline'][14]['pre_step_table']],None)
        r.update(profile=v.NATIVE,classification='live_decision_bridge_engineering_fixture',action=request,native_advances=32,seal_applications=1,decision_calls=1,
            seeded_editor_rules=v.flags(old['seeded_editor_rules'],None),seeded_editor_compiled=v.flags(old['seeded_editor_compiled'],None),
            decision_record=dict(step=13,decision=decision,callback_calls=1,before_rules=before,after_rules=v.flags(before,request),
                chosen_raw=v.flags(old['seeded_editor_rules'],request),chosen_compiled=v.flags(old['seeded_editor_compiled'],request),
                graph_unchanged=True,rng_before=old['timeline'][14]['rng_before'],rng_after=old['timeline'][14]['rng_before'],wallet_revalidated=o['wallet']))
        r['table_catalogue']=[]
        for step,row in enumerate(r['timeline'][1:],1):
            table=v.flags(old['table_catalogue'][old['timeline'][step]['pre_step_table']],None if step<=13 else request)
            if table not in r['table_catalogue']:r['table_catalogue'].append(table)
            row['pre_step_table']=r['table_catalogue'].index(table)
        policy=dict(observation=o,input=x,mode=slot['mode'],theta=v.THETA,probabilities=prob,draw=slot['draw'],sampled_action=sampled,decision=decision)
        save(root/f'native/cell-{i:04}.json',dict(slot=slot,policy=policy,receipt=r));statuses.append(dict(id=i,status='complete',native_advances=32,seal_applications=1))
    save(root/'native/status.json',statuses)

class BridgeAudit(unittest.TestCase):
    def root(self):return Path(os.environ['ERE_RESOURCE_TIMED_BRIDGE_ARCHIVE'])
    def test_synthetic_complete(self):
        with tempfile.TemporaryDirectory(prefix='bridge-audit-') as path:
            root=Path(path);synthetic(root,self.root());r=v.audit_records(root)
            self.assertEqual((r['native_advances'],r['seal_applications'],r['retained_slots']),(1063,35,36))
    def test_prechoice_and_switch_corruptions(self):
        with tempfile.TemporaryDirectory(prefix='bridge-audit-') as path:
            root=Path(path);synthetic(root,self.root());original=v.a.load;data=original(root/'native/cell-0000.json')
            variants=[]
            for field,value in [('step',14),('callback_calls',2),('graph_unchanged',False),('wallet_revalidated',0)]:
                bad=copy.deepcopy(data);bad['receipt']['decision_record'][field]=value;variants.append(bad)
            bad=copy.deepcopy(data);next(r for r in bad['receipt']['decision_record']['before_rules'] if r['name']=='allocation_defend')['enabled']=True;variants.append(bad)
            bad=copy.deepcopy(data);bad['receipt']['decision_record']['after_rules'][0]['priority']=999;variants.append(bad)
            bad=copy.deepcopy(data);bad['receipt']['decision_record']['chosen_raw'][0]['enabled']=False;variants.append(bad)
            for bad in variants:
                with patch.object(v.a,'load',side_effect=lambda p:bad if Path(p).name=='cell-0000.json' and Path(p).parent.name=='native' else original(p)):
                    with self.assertRaises(AssertionError):v.audit_records(root)
    def test_policy_and_utility_corruptions(self):
        with tempfile.TemporaryDirectory(prefix='bridge-audit-') as path:
            root=Path(path);synthetic(root,self.root());original=v.a.load;data=original(root/'native/cell-0000.json');variants=[]
            for field,value in [('sampled_action',1),('probabilities',[.5,.5]),('draw',.75),('theta',[0,0,0,0]),('input',[1,1,0,0])]:
                bad=copy.deepcopy(data);bad['policy'][field]=value;variants.append(bad)
            bad=copy.deepcopy(data);bad['receipt']['summary']['harvested_credits']=999;variants.append(bad)
            bad=copy.deepcopy(data);bad['receipt']['timeline'][14]['events']=[];variants.append(bad)
            for bad in variants:
                with patch.object(v.a,'load',side_effect=lambda p:bad if Path(p).name=='cell-0000.json' and Path(p).parent.name=='native' else original(p)):
                    with self.assertRaises(AssertionError):v.audit_records(root)
    def test_fixed_schedule(self):
        slots=v.schedule();self.assertEqual(len(slots),36);self.assertEqual([s['id'] for s in slots],list(range(36)))
        self.assertEqual([s['refusal'] for s in slots[32:]],v.REFUSALS)
        self.assertEqual([s['state'] for s in slots[:32]],list(range(8))*4)
    def test_refusal_corruption(self):
        with tempfile.TemporaryDirectory(prefix='bridge-audit-') as path:
            root=Path(path);synthetic(root,self.root());original=v.a.load;bad=original(root/'native/cell-0035.json');bad['failure']['native_advances']=0
            with patch.object(v.a,'load',side_effect=lambda p:bad if Path(p).name=='cell-0035.json' else original(p)):
                with self.assertRaises(AssertionError):v.audit_records(root)
    def test_frozen_native_panel(self):
        if not (self.root()/'native/cell-0035.json').exists():self.skipTest('pre-native boundary; no physics fixture yet')
        self.assertTrue(v.audit_records(self.root())['engineering_pass'])
if __name__=='__main__':unittest.main()
