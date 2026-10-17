#!/usr/bin/env python3
import copy
import os
from pathlib import Path
import time
import unittest
import verify_evolving_minds_handoff as p
import evolving_minds_shared_replay as shared

def fixture():
    results={}
    for case in p.cases():
        members=[dict(id=i,parent=None,founder=i,completion_tick=5,attempts=4,successes=2,failures=2,payable_action_opportunities=4,requests=1) for i in [1,2]]
        members.extend(dict(id=i+2,parent=i,founder=i,completion_tick=10,attempts=4,successes=4,failures=0,payable_action_opportunities=4,requests=1) for i in [1,2])
        results[case['name']]=dict(opportunity=dict(ticks=list(range(64)),lifetimes=members,primary_descendant_harvested=100,descendant_denominator=2,descendants_completed=2),records=[dict(parent=i,child_id=i+2) for i in [1,2]],final_row=dict(max_ancestry_depth=1))
    return results

class DecisionTests(unittest.TestCase):
    def test_exposure_pass_never_admits_science(self):
        d=p.decision(fixture());self.assertTrue(d['handoff_exposure_screen_pass'])
        for key in ['scientific_acceptance','sustained_claim','ready_for_scientific_launch','scientific_run_launched','power_adequacy_established','training_admitted']:self.assertFalse(d[key])
    def test_no_request_world_retained_as_failure(self):
        r=fixture();r['world409_faithful']['opportunity']['lifetimes'][2]['requests']=0;d=p.decision(r);self.assertFalse(d['handoff_exposure_screen_pass']);self.assertEqual(len(d['worlds']),4)
    def test_source_failure_is_not_culled_or_replaced(self):
        r=fixture();r['world401_source']['opportunity']['lifetimes'][2]['requests']=0;d=p.decision(r);self.assertTrue(d['handoff_exposure_screen_pass']);self.assertEqual(d['worlds'][0]['arms']['source']['descendant_requests'],1)
    def test_missing_paid_handoff_not_rescued_by_yield(self):
        r=fixture();r['world403_faithful']['records']=[];r['world403_faithful']['opportunity']['primary_descendant_harvested']=100000;self.assertFalse(p.decision(r)['handoff_exposure_screen_pass'])
    def test_denominator_and_horizon_refused(self):
        r=fixture();r.pop('world407_source')
        with self.assertRaisesRegex(ValueError,'denominator'):p.decision(r)
        r=fixture();r['world407_source']['opportunity']['ticks'].pop()
        with self.assertRaisesRegex(ValueError,'full horizon'):p.decision(r)
    def test_missing_founder_and_paid_accounting_refused(self):
        r=fixture();r['world401_faithful']['opportunity']['lifetimes'][0]['parent']=99
        with self.assertRaisesRegex(ValueError,'founder denominator'):p.decision(r)
        r=fixture();r['world401_faithful']['opportunity']['lifetimes'][0]['attempts']=3
        with self.assertRaisesRegex(ValueError,'paid attempt'):p.decision(r)

@unittest.skipUnless(os.environ.get('EVOLVING_MINDS_HANDOFF_FIXTURE'),'requires preserved native handoff archive')
class ReplayTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.root=Path(os.environ['EVOLVING_MINDS_HANDOFF_FIXTURE']);cls.v=p.load(cls.root/'stream_reference.py','ht_stream');cls.a=p.load(cls.root/'admission_reference.py','ht_admission');cls.c=p.load(cls.root/'replay.py','ht_replay');cls.d=p.load(cls.root/'d0_reference.py','ht_d0');cls.s=p.load(cls.root/'schedule_reference.py','ht_schedule');cls.o=p.load(cls.root/'opportunity_reference.py','ht_observer');cls.modules=(cls.a,cls.c,cls.d,cls.s);cls.base=(cls.root/'b24_base_rules.toml').read_bytes();cls.case=p.cases()[0];cls.chain=cls.v.inspect_segment(cls.root/cls.case['name'],funded_config=cls.case['config'],handoff_config=cls.case['handoff'])
    def both_refuse(self,change):
        chain=copy.deepcopy(self.chain);commit=chain[0]['commits'][1];change(chain,commit)
        with self.assertRaises(ValueError):self.v.physical_audit(chain,commit,self.modules,self.base,funded_config=self.case['config'],handoff_config=self.case['handoff'])
        with self.assertRaises(ValueError):shared.optimized_cell(chain,self.modules,self.base,self.case['config'],None,self.v,self.o,time.monotonic(),handoff_config=self.case['handoff'])
    def test_admission_ast_preserved_and_full_result_agrees(self):
        _,receipt=shared.admission_function(self.a,(self.root/'admission_reference.py').read_text());self.assertEqual(receipt['original_ast_sha256'],receipt['restored_ast_sha256'])
        final,physics,opportunity,count=shared.optimized_cell(self.chain,self.modules,self.base,self.case['config'],None,self.v,self.o,time.monotonic(),handoff_config=self.case['handoff']);self.assertEqual(count,9)
        original,reference=self.v.physical_audit(self.chain,self.chain[0]['commits'][-1],self.modules,self.base,funded_config=self.case['config'],handoff_config=self.case['handoff']);self.assertEqual(final,original);self.assertEqual(physics,reference)
    def test_intermediate_graph_forgery_refused(self):self.both_refuse(lambda chain,commit:commit['state']['frame'].__setitem__('nodes',[]))
    def test_supplied_policy_forgery_refused(self):self.both_refuse(lambda chain,commit:commit['state']['native']['handoff_config'].__setitem__('supplied','Unchanged'))
    def test_supplied_bytes_and_metadata_forgery_refused(self):
        def edit(chain,commit):
            records=[r for b in commit['bundles'] for r in b['records']];self.assertTrue(records);records[0]['proposed_bytes'][0]=255
        self.both_refuse(edit)
        def meta(chain,commit):
            records=[r for b in commit['bundles'] for r in b['records']];records[0]['target_id']=102
        self.both_refuse(meta)
    def test_omitted_checkpoint_refused(self):
        chain=copy.deepcopy(self.chain);chain[0]['commits'].pop(1)
        with self.assertRaises(ValueError):shared.optimized_cell(chain,self.modules,self.base,self.case['config'],None,self.v,self.o,time.monotonic(),handoff_config=self.case['handoff'])
    def test_fresh_process_recovery_all_boundaries(self):
        case=p.cases()[-1];chain=self.v.inspect_segment(self.root/'world17_recovery_resumed',funded_config=case['config'],handoff_config=case['handoff']);final,physics,o,count=shared.optimized_cell(chain,self.modules,self.base,case['config'],None,self.v,self.o,time.monotonic(),handoff_config=case['handoff']);self.assertEqual(count,10);self.assertEqual(len(o['ticks']),64)

if __name__=='__main__':unittest.main()
