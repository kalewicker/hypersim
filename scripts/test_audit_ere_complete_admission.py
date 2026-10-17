#!/usr/bin/env python3
"""Development tests; no Engine work or experiment admission."""
import copy
import json
from pathlib import Path
import tempfile
import unittest
import audit_ere_complete_admission as a

REPO = Path(__file__).resolve().parents[1]

class Tests(unittest.TestCase):
    def test_registered_request_and_refusals(self):
        q = a.load(REPO/'configs/ere_complete_bounded_request_v1.json')
        a.request(q)
        for field, bad in [('horizon',10000), ('work',400000), ('evaluations',2), ('optimizer_generations',20), ('parameters',[]), ('wall_seconds',901), ('physics_seed',-1), ('max_pairings',True)]:
            x=copy.deepcopy(q); x[field]=bad
            with self.assertRaises(AssertionError): a.request(x)
        q['unknown']=True
        with self.assertRaises(AssertionError): a.request(q)

    def test_safe_paths_and_duplicate_json(self):
        for name in ('', '../x', '/x', 'a/../b', 'a//b', 'a\\b', 'a\nb'):
            self.assertFalse(a.safe(name))
        self.assertTrue(a.safe('source_snapshots/a.rs'))
        with tempfile.TemporaryDirectory() as tmp:
            p=Path(tmp)/'x'; p.write_text('{"a":1,"a":2}')
            with self.assertRaises(AssertionError): a.load(p)

    def test_exact_seal_and_symlink(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp); (root/'nested').mkdir(); (root/'nested/x').write_text('test')
            a.seal(root); self.assertEqual(a.verify(root),1)
            (root/'extra').write_text('extra')
            with self.assertRaises(AssertionError): a.verify(root)
            (root/'extra').unlink(); (root/'nested/x').write_text('changed')
            with self.assertRaises(AssertionError): a.verify(root)
            (root/'link').symlink_to(root/'nested/x')
            with self.assertRaises(AssertionError): a.files(root)

    def test_real_pinned_archives_read_only(self):
        paths=a.load(REPO/'configs/ere_complete_receipts_v1.json')
        for name,(seal,sha,count) in a.SPECS.items():
            self.assertEqual(a.verify(REPO/'configs'/paths[name],seal,sha,count),count)

    def test_existing_native_accounting_and_tampered_score(self):
        runtime=REPO/'output/ere_runtime50k_signal_panel_20260930'
        with (runtime/'rollouts.jsonl').open() as stream:
            row=json.loads(next(stream))
        q=a.load(REPO/'configs/ere_complete_bounded_request_v1.json')
        q['physics_seed']=row['evaluation']['physics_seed']
        q['emission_seed']=row['emission']['emission_seed']
        row=copy.deepcopy(row); row['engineering_only']=True
        catalogue=a.load(REPO/'output/ere_valid_production_grammar_20260929/catalogue.json')
        self.assertEqual(a.evaluation(row,catalogue,q,a.helper(runtime)),(100,100,True))
        row['evaluation']['score']=.1234
        with self.assertRaises(AssertionError): a.evaluation(row,catalogue,q,a.helper(runtime))

if __name__ == '__main__': unittest.main()
