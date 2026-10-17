#!/usr/bin/env python3
"""Bounded read-only coherent checker regressions; Rust emits temporary fixtures."""
import copy
import json
from pathlib import Path
import sys
import time
import unittest

import verify_ere_complete_coherent_resource_pilot as a

FIXTURES=Path(sys.argv.pop(1)) if len(sys.argv)>1 and not sys.argv[1].startswith('-') else None

class PredicateTests(unittest.TestCase):
    def test_pilot_manifest_requires_fixed_hundred_step_envelope(self):
        d=a.load(Path(__file__).resolve().parents[1]/"docs/ere_complete_coherent_resource_pilot_v1.json")
        m={"profile":a.PROFILE,"protocol":d["protocol"],"allocation_mode":"metadata-isolated-v1","family_version":10,"parents":168,"controls":5376,"training_admitted":False,"frozen_before_measurement":True,"development_only":True,"scientific_run_launched":False,"resources":d["proposed_resources"],"draws":64,"horizon":100,"development_resources":d["development"].copy(),"sources":[]}
        m["development_resources"]={k:v for k,v in m["development_resources"].items() if k not in ["draws","horizon"]}
        a.check_manifest(m,d,True)
        for key,value in [("draws",10000),("horizon",8),("scientific_run_launched",True),("training_admitted",True)]:
            with self.subTest(key=key):
                bad=copy.deepcopy(m);bad[key]=value
                with self.assertRaises(AssertionError):a.check_manifest(bad,d,True)
        with self.assertRaises(AssertionError):a.check_manifest(m,d,False)
        for key in ["wall","audit","output","steps"]:
            bad=copy.deepcopy(m);bad["development_resources"][key]+=1
            with self.assertRaises(AssertionError):a.check_manifest(bad,d,True)

    def test_frozen_child_does_not_override_exact_or_rcf_neutrality(self):
        p={"complete":True,"frames":[("same",0)]*100,"fingerprints":["typed"]*100,"rng":["seed"]*100,"exploding":False,"frozen":True}
        self.assertEqual(a.classification(True,False,p,p),"near_neutral")
        self.assertEqual(a.classification(True,True,p,p),"neutral")
        changed={**p,"fingerprints":["changed"]*100}
        self.assertEqual(a.classification(True,False,p,changed),"catastrophic")
        self.assertEqual(a.classification(True,False,p,{**changed,"complete":False}),"unmeasured")

    def test_world_edge_handles_and_endpoint_order_are_identity_fields(self):
        nodes={"NodeId(1v1)":a.node_data(0),"NodeId(3v1)":a.node_data(1)}
        e={"label":0,"nodes":[{"idx":1,"version":1},{"idx":3,"version":1}]}
        first=a.world_fingerprint(nodes,{"EdgeId(1v1)":e})
        self.assertNotEqual(first,a.world_fingerprint(nodes,{"EdgeId(1v3)":e}))
        self.assertNotEqual(first,a.world_fingerprint(nodes,{"EdgeId(1v1)":{**e,"nodes":list(reversed(e["nodes"]))}}))

    def test_allocator_recycling_is_independent_and_exact(self):
        nodes={"NodeId(1v1)":a.node_data(0),"NodeId(2v1)":a.node_data(a.BASE+5)}
        x=a.Allocation(nodes,{})
        self.assertEqual(x.add("node",a.node_data(a.BASE+5)),{"idx":4,"version":1})
        x.remove("node",{"idx":2,"version":1})
        self.assertEqual(x.add("node",a.node_data(0)),{"idx":3,"version":1})
        self.assertEqual(x.add("node",a.node_data(a.BASE+5)),{"idx":2,"version":3})
        with self.assertRaises(AssertionError):x.remove("node",{"idx":2,"version":1})

    def test_allocator_checks_ordered_repeated_incidence(self):
        nodes={"NodeId(1v1)":a.node_data(0),"NodeId(3v1)":a.node_data(0)}
        endpoints=[{"idx":1,"version":1},{"idx":1,"version":1},{"idx":3,"version":1}]
        x=a.Allocation(nodes,{"EdgeId(1v1)":{"label":0,"nodes":endpoints}})
        self.assertEqual(x.incident["NodeId(1v1)"],["EdgeId(1v1)"]*2)
        x.remove("edge",{"idx":1,"version":1})
        self.assertEqual(x.incident["NodeId(1v1)"],[])
        self.assertEqual(x.add("edge",{"label":0,"nodes":endpoints}),{"idx":1,"version":3})

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

    def test_both_allocator_tamper_categories_reject(self):
        self.assertEqual(a.allocation_tampers(self.parents[0]),['wrong_allocation_domain','wrong_allocator_sequence'])

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
