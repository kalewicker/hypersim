#!/usr/bin/env python3
"""Development controls only. No new scientific Engine trajectories."""
import copy
from pathlib import Path
import unittest
import audit_ere_complete_scaling as s

REPO=Path(__file__).resolve().parents[1]

class Tests(unittest.TestCase):
    def test_fixed_design_and_schedule(self):
        self.assertEqual(s.a.load(REPO/'configs/ere_complete_scaling_v2.json'),s.design())
        plan=s.schedule(s.design()); self.assertEqual(len(plan),36)
        self.assertEqual(len({q['id'] for q in plan}),36)
        self.assertEqual(sum(q['horizon'] for q in plan),133200)
        for h in [100,1000,10000]: self.assertEqual(sum(q['horizon']==h for q in plan),12)

    def test_exact_source_bridge_and_no_other_drift(self):
        runtime=REPO/'output/ere_runtime50k_signal_panel_20260930'
        old=(runtime/'source_snapshots'/s.SIGNAL).read_text()
        current=(REPO/s.SIGNAL).read_text()
        self.assertEqual(s.bridge(old),current)
        self.assertNotEqual(s.bridge(old),current.replace('s.node_count() > 2000','s.node_count() > 2001'))
        with self.assertRaises(AssertionError): s.bridge(old.replace('for step in 1..=HORIZON','for step in 1..=101'))

    def test_existing_native_row_accounting_and_caps(self):
        runtime=REPO/'output/ere_runtime50k_signal_panel_20260930'
        with (runtime/'rollouts.jsonl').open() as stream: row=__import__('json').loads(next(stream))
        e,v=row['emission'],row['evaluation'];v['horizon']=100
        for step in v['steps']: step.update(nodes=100,edges=200)
        q=dict(horizon=100,emission_seed=e['emission_seed'],physics_seed=v['physics_seed'])
        r=dict(schedule=q,status='Complete',elapsed_seconds=1,evaluation=v)
        c=s.a.load(REPO/'output/ere_valid_production_grammar_20260929/catalogue.json');helper=s.a.helper(runtime)
        self.assertEqual(s.cell(r,e,c,helper),(100,100,True))
        for field,bad in [('score',.1234),('horizon',1000),('proposal_fires',999),('credited_rule_ids',[-1])]:
            tampered=copy.deepcopy(r);tampered['evaluation'][field]=bad
            with self.assertRaises(AssertionError): s.cell(tampered,e,c,helper)
        tampered=copy.deepcopy(r);tampered['evaluation']['steps'][0]['nodes']=2001
        with self.assertRaises(AssertionError): s.cell(tampered,e,c,helper)

    def test_fixed_denominator_and_null_accounting(self):
        rows=[dict(schedule=q,status='NotRun',evaluation=None) for q in s.schedule(s.design())]
        derived,triples=s.summary(rows,s.design())
        self.assertFalse(derived['measurement_complete']);self.assertFalse(derived['prefixes_equal'])
        self.assertEqual(len(triples),12)
        for counts in derived['by_horizon'].values(): self.assertEqual(counts['not_run'],12)

if __name__=='__main__': unittest.main()
