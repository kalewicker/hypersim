#!/usr/bin/env python3
import copy
import os
from pathlib import Path
import time
import unittest
import verify_evolving_minds_descendant as p
import evolving_minds_shared_replay as shared

def fixture():
    out={}
    for case in p.cases():
        records=[];samples={};members=[dict(id=i,parent=None,founder=i) for i in [1,2]]
        for founder in [1,2]:
            parent=100+founder;child=200+founder
            members.extend([dict(id=parent,parent=founder,founder=founder),dict(id=child,parent=parent,founder=founder,completion_tick=15,requests=1)])
            r=dict(request_id=child,parent=parent,tick=10,operator='supplied_descendant',child_id=child,syntax_valid=True,material_reachable=True,parent_program={'a':0},proposed_program={'a':0 if case['arm']=='unchanged' else 1},request_charge=1,copying_charge=5,refunded=0)
            records.append(r);samples[str(founder)]=copy.deepcopy(r)
        out[case['name']]=dict(opportunity=dict(ticks=[dict(tick=i) for i in range(1,65)],lifetimes=members),records=records,target_state=dict(config=case['descendant'],targets=samples))
    return out

class DecisionTests(unittest.TestCase):
    def test_fixed_cases_and_source_denominator(self):
        cases=p.cases();self.assertEqual(len(cases),73)
        for seed in [601,607]:
            rows=[x for x in cases if x['config']['seed']==seed];self.assertEqual(len(rows),36)
            self.assertEqual({(x['descendant']['target_id'],x['descendant']['new_source']) for x in rows},{(None,None)}|{(i,s) for i in range(100,105) for s in range(1,8)})
    def test_engineering_exposure_never_admits_science(self):
        d=p.decision(fixture());self.assertTrue(d['early_exposure_screen_pass']);self.assertEqual(len(d['targets']),144)
        for k in ['scientific_acceptance','sustained_claim','training_admitted','power_adequacy_established','ready_for_scientific_launch']:self.assertFalse(d[k])
    def test_missing_route_retained_in_full_denominator(self):
        x=fixture();x['world607_g104_s7']['target_state']['targets'].pop('2');x['world607_g104_s7']['records'].pop()
        d=p.decision(x);self.assertFalse(d['early_exposure_screen_pass']);self.assertEqual(len(d['targets']),144);self.assertEqual(d['category_counts']['no_early_request'],1)
    def test_omitted_paid_target_not_reported_as_no_request(self):
        x=fixture();x['world607_g104_s7']['target_state']['targets'].pop('2')
        with self.assertRaisesRegex(ValueError,'omission'):p.decision(x)
    def test_paid_rejection_is_not_removed(self):
        x=fixture();r=x['world601_g104_s1']['records'][0];r.update(syntax_valid=False,material_reachable=False,refunded=29,copying_charge=0,child_id=None)
        d=p.decision(x);self.assertEqual(d['category_counts']['syntax_rejected'],1);self.assertEqual(d['substitution_denominator'],140)
    def test_late_request_not_replacement_and_costs_refused(self):
        x=fixture();x['world601_g103_s1']['target_state']['targets']['1']['tick']=25
        with self.assertRaisesRegex(ValueError,'early paid'):p.decision(x)
        x=fixture();x['world601_g103_s1']['records'][0]['request_charge']=0
        with self.assertRaisesRegex(ValueError,'settlement'):p.decision(x)
    def test_missing_cell_and_short_horizon_refused(self):
        x=fixture();x.pop('world607_g100_s1')
        with self.assertRaisesRegex(ValueError,'denominator'):p.decision(x)
        x=fixture();x['world607_g100_s1']['opportunity']['ticks'].pop()
        with self.assertRaisesRegex(ValueError,'full horizon'):p.decision(x)

@unittest.skipUnless(os.environ.get('EVOLVING_MINDS_DESCENDANT_FIXTURE'),'requires native exposed-world fixture')
class ReplayTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.root=Path(os.environ['EVOLVING_MINDS_DESCENDANT_FIXTURE']);cls.v=p.load(cls.root/'stream_reference.py','dt_stream');cls.a=p.load(cls.root/'admission_reference.py','dt_admission');cls.c=p.load(cls.root/'replay.py','dt_replay');cls.d=p.load(cls.root/'d0_reference.py','dt_d0');cls.s=p.load(cls.root/'schedule_reference.py','dt_schedule');cls.o=p.load(cls.root/'opportunity_reference.py','dt_observer');cls.modules=(cls.a,cls.c,cls.d,cls.s);cls.base=(cls.root/'b24_base_rules.toml').read_bytes();cls.directory=cls.root/'world17_recovery_resumed';cls.h=p.cases()[-1]['descendant'];cls.cfg=p.cases()[-1]['config'];cls.chain=cls.v.inspect_segment(cls.directory,funded_config=cls.cfg,descendant_config=cls.h)
    def refuse(self,edit):
        chain=copy.deepcopy(self.chain);commit=chain[0]['commits'][3];edit(chain,commit)
        with self.assertRaises(ValueError):self.v.physical_audit(chain[:1],commit,self.modules,self.base,funded_config=self.cfg,descendant_config=self.h)
        with self.assertRaises(ValueError):shared.optimized_cell(chain,self.modules,self.base,self.cfg,None,self.v,self.o,time.monotonic(),descendant_config=self.h)
    def test_original_and_shared_replay_agree_and_recovery(self):
        final,physics,o,count=shared.optimized_cell(self.chain,self.modules,self.base,self.cfg,None,self.v,self.o,time.monotonic(),descendant_config=self.h)
        self.assertEqual(count,10);self.assertEqual([t['tick'] for t in o['ticks']],list(range(1,65)))
        orig,ref=self.v.physical_audit(self.chain,self.chain[-1]['commits'][-1],self.modules,self.base,funded_config=self.cfg,descendant_config=self.h);self.assertEqual(final,orig);self.assertEqual(physics,ref)
    def test_omitted_target_and_wrong_family_refused(self):
        self.refuse(lambda chain,c:c['state']['native']['descendant']['targets'].pop('1'))
        def edit(chain,c):c['state']['native']['descendant']['targets']['2']=c['state']['native']['descendant']['targets']['1']
        self.refuse(edit)
    def test_retarget_and_proposal_bytes_refused(self):
        self.refuse(lambda chain,c:c['state']['native']['descendant']['targets']['1'].__setitem__('tick',25))
        self.refuse(lambda chain,c:c['state']['native']['descendant']['targets']['1']['proposed_bytes'].__setitem__(0,255))
    def test_intermediate_graph_and_omitted_boundary_refused(self):
        self.refuse(lambda chain,c:c['state']['frame'].__setitem__('nodes',[]))
        chain=copy.deepcopy(self.chain);chain[0]['commits'].pop(1)
        with self.assertRaises(ValueError):shared.optimized_cell(chain,self.modules,self.base,self.cfg,None,self.v,self.o,time.monotonic(),descendant_config=self.h)

if __name__=='__main__':unittest.main()
