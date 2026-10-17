#!/usr/bin/env python3
import copy
import json
import os
from pathlib import Path
import time
import unittest
import evolving_minds_failure_audit as analysis
import evolving_minds_shared_replay as shared


class SharedReplayTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.root=Path(os.environ['EVOLVING_MINDS_DEVELOPMENT_FIXTURE'])
        cls.pilot=analysis.common.load(cls.root/'checker.py','shared_test_pilot')
        cls.v=analysis.common.load(cls.root/'stream_reference.py','shared_test_stream')
        cls.a=analysis.common.load(cls.root/'admission_reference.py','shared_test_admission')
        cls.c=analysis.common.load(cls.root/'replay.py','shared_test_replay')
        cls.d=analysis.common.load(cls.root/'d0_reference.py','shared_test_d0')
        cls.s=analysis.common.load(cls.root/'schedule_reference.py','shared_test_schedule')
        cls.o=analysis.common.load(cls.root/'opportunity_reference.py','shared_test_observer')
        cls.modules=(cls.a,cls.c,cls.d,cls.s)
        cls.cfg=cls.pilot.cases()[2]['config']
        cls.chain=cls.v.inspect_segment(cls.root/'world101_retained',funded_config=cls.cfg,retained_policy=cls.pilot.POLICY)
        cls.base=(cls.root/'b24_base_rules.toml').read_bytes()
        cls.baseline=analysis.common.data(cls.root/'independent_replay.json')['results']['world101_retained']

    def both_refuse(self, change, pattern=None):
        chain=copy.deepcopy(self.chain);commit=chain[0]['commits'][1];change(chain,commit)
        for method in ['original','optimized']:
            with self.subTest(method=method):
                with self.assertRaises(ValueError):
                    if method=='original':
                        self.v.physical_audit(chain,commit,self.modules,self.base,funded_config=self.cfg,retained_policy=self.pilot.POLICY)
                    else:
                        shared.optimized_cell(chain,self.modules,self.base,self.cfg,self.pilot.POLICY,self.v,self.o,time.monotonic())

    def test_admission_ast_restores_every_original_statement(self):
        _,receipt=shared.admission_function(self.a,(self.root/'admission_reference.py').read_text())
        self.assertEqual(receipt['original_ast_sha256'],receipt['restored_ast_sha256'])
        self.assertEqual(receipt['callback_sites'],2)

    def test_pilot_ast_restores_all_nonreplay_checks(self):
        _,receipt=shared.pilot_function(self.pilot,(self.root/'checker.py').read_text(),shared.optimized_cell)
        self.assertEqual(receipt['original_ast_sha256'],receipt['restored_ast_sha256'])

    def test_full_result_matches_original_with_every_checkpoint(self):
        _,physics,opportunity,count=shared.optimized_cell(self.chain,self.modules,self.base,self.cfg,self.pilot.POLICY,self.v,self.o,time.monotonic())
        self.assertEqual(count,9)
        self.assertEqual(dict(opportunity=opportunity,final_row=physics['final_row'],retained=physics['retained']),self.baseline)

    def test_intermediate_graph_forgery_refused_by_both(self):
        def edit(chain,commit):
            frame=commit['state']['frame']
            # Change the published graph while keeping its native ledger intact.
            nodes,_=self.c.graphs(frame)
            self.assertTrue(nodes)
            frame['nodes']=[]
        self.both_refuse(edit)

    def test_intermediate_ledger_forgery_refused_by_both(self):
        self.both_refuse(lambda chain,commit:commit['state']['native']['ledger'].__setitem__('influx',-1))

    def test_intermediate_row_and_ledger_forgery_refused_by_both(self):
        def edit(chain,commit):
            commit['bundles'][-1]['row']['harvested']+=8
            commit['state']['native']['ledger']['harvested']+=8
        self.both_refuse(edit)

    def test_retained_bytes_forgery_refused_by_both(self):
        self.both_refuse(lambda chain,commit:commit['state']['native']['retained']['samples']['1']['proposed_bytes'].__setitem__(0,255))

    def test_deleted_retained_sample_refused_by_both(self):
        self.both_refuse(lambda chain,commit:commit['state']['native']['retained']['samples'].pop('1'))

    def test_omitted_checkpoint_refused(self):
        chain=copy.deepcopy(self.chain);chain[0]['commits'].pop(1)
        with self.assertRaisesRegex(ValueError,'nine shared checkpoints'):
            shared.optimized_cell(chain,self.modules,self.base,self.cfg,self.pilot.POLICY,self.v,self.o,time.monotonic())

    def test_complete_route_payment_and_child_denominator(self):
        records=[r for line in (self.root/'world101_retained'/'ticks.jsonl').read_text().splitlines() for r in json.loads(line)['records']]
        immediate=self.a.funded_program('Immediate',self.d)
        result=analysis.decompose(records,self.baseline['opportunity'],immediate)
        self.assertEqual(result['requests'],5)
        self.assertEqual(result['descendant_requests'],0)
        self.assertEqual(sum(result['categories'].values()),5)
        self.assertEqual(len(result['routes']),5)
        with self.assertRaisesRegex(ValueError,'request denominator'):
            analysis.decompose(records[:-1],self.baseline['opportunity'],immediate)

    def test_false_child_route_refused(self):
        records=[r for line in (self.root/'world101_retained'/'ticks.jsonl').read_text().splitlines() for r in json.loads(line)['records']]
        records[0]['child_id']=987654
        with self.assertRaises(ValueError):
            analysis.decompose(records,self.baseline['opportunity'],self.a.funded_program('Immediate',self.d))


if __name__=='__main__':unittest.main()
