#!/usr/bin/env python3
"""Bounded read-only coherent checker regressions; Rust emits temporary fixtures."""
import copy
import json
from pathlib import Path
import sys
import time
import unittest

import verify_ere_complete_coherent as a

FIXTURES=Path(sys.argv.pop(1)) if len(sys.argv)>1 and not sys.argv[1].startswith('-') else None

class PredicateTests(unittest.TestCase):
    def test_equal_hash_different_typed_world_is_not_near_neutral(self):
        p={"complete":True,"frames":[("same",1)],"fingerprints":["typed1"],"rng":["seed"],"exploding":False,"frozen":False}
        c={**p,"fingerprints":["typed2"]}
        self.assertEqual(a.classification(True,False,p,c),"divergent")
        self.assertEqual(a.classification(True,True,p,c),"neutral")
        self.assertTrue(a.neutral_failure(p,c));self.assertFalse(a.neutral_failure(p,p))

    def test_rejections_precede_even_valid_neutral_parent(self):
        p={"complete":True,"frames":[],"fingerprints":[],"rng":[],"exploding":False,"frozen":False}
        self.assertEqual(a.classification(False,True,p,None),"rejected")
        self.assertEqual(a.classification(True,True,p,None),"neutral")

    def test_missing_baseline_and_valid_unmeasured_block_completion(self):
        s={"parent_baselines_complete":False,"strata":{"overall":{"unmeasured":0}}}
        self.assertFalse(a.measurement_complete(s))
        s["parent_baselines_complete"]=True;s["strata"]["overall"]["unmeasured"]=1
        self.assertFalse(a.measurement_complete(s))

    def test_metadata_only_does_not_create_pattern_topology(self):
        inv={"step":0,"nodes":[],"edges":[],"label_indexes":[]}
        changed=copy.deepcopy(inv)
        changed["nodes"]=[["NodeId(1v1)",{"data":a.node_data(a.BASE+6,[[0,{"Int":1}]]),"incident":[]}]]
        self.assertEqual(a.pattern_topology(inv),a.pattern_topology(changed))

@unittest.skipIf(FIXTURES is None,"temporary native fixture path required")
class NativeFixtureTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        a.DEADLINE=time.monotonic()+120
        cls.f=json.loads(FIXTURES.read_text())
        cls.parents={p['index']:p for p in cls.f['parents']}
        for i,p in cls.parents.items():a.check_parent(p,i)

    def test_all_operators_both_roles_exact_replay_and_independent_decode(self):
        coverage=set()
        for d in self.f['draws']:
            p=self.parents[d['parent']]
            stage=a.raw_edit(p,d)
            self.assertEqual(a.decode(stage,p['root']),d['decoded'])
            coverage.add((d['operator'],d['role']))
        for op in a.OPERATORS[:11]:
            for role in ['Core','Proposal']:self.assertIn((op,role),coverage)
        for op in a.OPERATORS[11:]:self.assertIn((op,'Family'),coverage)

    def test_all_six_coherent_tamper_categories_reject(self):
        d=next(d for d in self.f['draws'] if d['edit']['status']=='Applied' and d['edit']['logical_topology_changed'])
        self.assertEqual(len(a.coherent_tampers(self.parents[d['parent']],d)),6)

    def test_invalid_postcondition_not_repaired_or_promoted(self):
        ds=[d for d in self.f['draws'] if d['edit']['status']=='RejectedPostcondition']
        self.assertTrue(ds)
        for d in ds:
            e=d['edit'];self.assertFalse(e['admitted']);self.assertTrue(e['caller_unchanged'])
            self.assertFalse(e['stage_decodable'])
            self.assertNotEqual(e['receipt']['staged'],e['receipt']['before'])

    def test_valid_policy_rollback_stays_rejected(self):
        ds=[d for d in self.f['draws'] if d['edit']['status']=='RejectedPolicy']
        self.assertTrue(ds)
        for d in ds:
            self.assertTrue(d['edit']['stage_decodable']);self.assertFalse(d['edit']['admitted'])
            self.assertTrue(d['edit']['caller_unchanged'])
            x=copy.deepcopy(d);x['edit']['admitted']=True
            a.reject('rollback_promotion',lambda:a.raw_edit(self.parents[d['parent']],x))

    def test_all_rejections_remain_in_attempted_strata(self):
        rows=[]
        for d in self.f['draws']:
            p=self.parents[d['parent']];e=d['edit']
            rows.append({"operator":d['operator'],"operator_index":d['operator_index'],"role":d['role'],"shape":p['shape'],"parent_active":p['program']['active'],"at_cap":a.cap_parent(p['program']),"logical_topology_changed":e['logical_topology_changed'],"raw_changed":e['raw_changed'],"metadata_changed":e['metadata_changed'],"applicable":e['applicable'],"status":e['status'],"stage_decodable":e['stage_decodable'],"parent_rcf":p['program']['rcf'],"parent":p['index'],"validator":d['decoded']['validator'],"outcome":'neutral' if e['admitted'] else 'rejected',"mode_changed":False,"trace_changed":False,"soundness_checked":False,"soundness_failure":False})
        summary=a.tally(rows)
        self.assertEqual(summary['strata']['overall']['samples'],len(rows))
        self.assertEqual(summary['strata']['overall']['rejected'],sum(not d['edit']['admitted'] for d in self.f['draws']))
        self.assertFalse(summary['numerical_pass'])
        self.assertIn('topology_operator',summary['strata']);self.assertIn('actual_pattern_change',summary['strata'])

if __name__=='__main__':unittest.main(verbosity=2)
