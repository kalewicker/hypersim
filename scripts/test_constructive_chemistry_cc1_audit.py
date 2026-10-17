#!/usr/bin/env python3
"""Adversarial evidence tests: the independent checker must reject falsification."""
from copy import deepcopy
import importlib.util
import json
from pathlib import Path
import sys
import tempfile
import unittest
sys.dont_write_bytecode=True
ROOT=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('cc1audit',ROOT/'scripts/audit_constructive_chemistry_cc1.py')
a=importlib.util.module_from_spec(spec);spec.loader.exec_module(a)
CONTRACT=ROOT/'experiments/constructive_chemistry/contracts/cc1_atomic_fixture_v1.json'
NATIVE=Path(sys.argv[1]) if len(sys.argv)>1 and not sys.argv[1].startswith('-') else ROOT/'output/constructive_chemistry_cc1_development_20261005/native.json'
if len(sys.argv)>1 and not sys.argv[1].startswith('-'):sys.argv.pop(1)
class AuditorAdversarial(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.good=json.loads(NATIVE.read_bytes())
    def rejected(self,mutation):
        r=deepcopy(self.good);mutation(r)
        with tempfile.TemporaryDirectory() as folder:
            p=Path(folder)/'tampered.json';p.write_bytes(a.encoded(r))
            with self.assertRaises((ValueError,a.Rejection)):a.audit(CONTRACT,p)
    def test_native_evidence_passes(self):self.assertTrue(a.audit(CONTRACT,NATIVE)['gate_pass'])
    def test_missing_case(self):self.rejected(lambda r:r['variants'].pop())
    def test_wrong_rejection(self):self.rejected(lambda r:r['variants'][0]['rejected'].__setitem__(0,'InactiveFuel'))
    def test_rejection_allocator_drift(self):
        def change(r):
            for v in r['variants'][:2]:
                obs=v['attempts'][3]['after'];obs['exact']['allocator']['nodes']['next'][0]+=1;obs['exact_sha256']=a.digest(obs['exact'])
        self.rejected(change)
    def test_semantic_fuel_unpaid(self):
        def change(r):
            for v in r['variants'][:2]:
                next(n for n in v['final']['semantic']['nodes'] if n['id']==30)['activated']=True
        self.rejected(change)
    def test_missing_offer(self):self.rejected(lambda r:r['variants'][0]['attempts'].pop())
    def test_false_scientific_claim(self):self.rejected(lambda r:r.__setitem__('scientific_acceptance',True))
    def test_malformed_case_replaced_with_valid_graph(self):
        def change(r):
            normal=r['variants'][0]['initial']['exact']['state']
            for v in r['variants']:
                if v['id']=='malformed_missing_property':
                    v['raw_initial']=dict(graph=normal['graph'],revision=normal['revision'],frontier=normal['semantic_id_frontier'],reservoir=normal['reservoir'],heat=normal['heat'])
        self.rejected(change)
    def test_accepted_journal_omission(self):
        def change(r):
            for v in r['variants'][:2]:
                obs=v['attempts'][2]['after'];obs['exact']['state']['accepted_event_journal']=[];obs['exact_sha256']=a.digest(obs['exact'])
        self.rejected(change)
if __name__=='__main__':unittest.main()
