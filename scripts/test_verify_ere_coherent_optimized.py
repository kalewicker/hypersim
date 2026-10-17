#!/usr/bin/env python3
"""Differential regressions against the sealed, frozen pilot checker."""
import copy
import importlib.util
from itertools import islice
import json
import math
from pathlib import Path
import sys
import subprocess
import unittest
from unittest.mock import patch

ROOT=Path(__file__).resolve().parents[1]
ARCHIVE=ROOT/'output/ere_complete_coherent_resource_pilot_20260928'
sys.path.insert(0,str(ARCHIVE/'source_snapshots/scripts'))


def module(name,path):
    spec=importlib.util.spec_from_file_location(name,path)
    m=importlib.util.module_from_spec(spec);sys.modules[name]=m;spec.loader.exec_module(m)
    return m


b=module('frozen_baseline',ARCHIVE/'source_snapshots/scripts/verify_ere_complete_coherent_resource_pilot.py')
a=module('optimized_candidate',ROOT/'scripts/verify_ere_complete_coherent_optimized.py')
predicates=module('original_predicates',ARCHIVE/'source_snapshots/scripts/test_verify_ere_complete_coherent_resource_pilot.py')
predicates.a=a


class DifferentialTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.parents=list(b.stream(ARCHIVE/'parents.jsonl.gz'))
        cls.draws=list(b.stream(ARCHIVE/'draw_manifest.jsonl.gz'))
        cls.controls=list(b.stream(ARCHIVE/'control_manifest.jsonl.gz'))
        cls.control_rows=list(b.stream(ARCHIVE/'controls.jsonl.gz'))

    def test_all_64_selector_edit_decode_transforms_equal_baseline(self):
        for d in self.draws:
            p=self.parents[d['parent']];before=copy.deepcopy(p)
            expected=b.raw_edit(p,d);got=a.raw_edit(p,d)
            self.assertEqual(got,expected)
            self.assertEqual(a.decode(got,p['root']),b.decode(expected,p['root']))
            self.assertEqual(p,before,'edit replay mutated baseline inputs')

    def test_all_32_damage_categories_on_four_representative_parents(self):
        for index in [0,15,151,167]:
            p=self.parents[index];before=copy.deepcopy(p)
            for ix in range(index*32,(index+1)*32):
                expected=b.check_control(p,self.controls[ix],self.control_rows[ix])
                self.assertEqual(a.check_control(p,self.controls[ix],self.control_rows[ix]),expected)
            self.assertEqual(p,before,'control replay mutated parent')

    def test_material_changes_cannot_reuse_prior_decode(self):
        p=self.parents[0]
        expected=a.decode(p['inventory'],p['root'])
        changes=[]
        for key,value in [(0x200,999),(8,3),(9,{'Float':0.0})]:
            inv=copy.deepcopy(p['inventory'])
            n=next(n for k,n in inv['nodes'] if k==a.node_key(p['root']))
            for kv in n['data']['properties']:
                if kv[0]==key:kv[1]=value if isinstance(value,dict) else {'Int':value}
            changes.append(inv)
        for inv in changes:
            self.assertEqual(a.decode(inv,p['root']),b.decode(inv,p['root']))
            self.assertNotEqual(a.decode(inv,p['root']),expected)
        self.assertEqual(a.decode(p['inventory'],p['root']),expected)

    def test_hundred_step_trace_vectors_equal_baseline(self):
        targets={'parent_0','parent_125','child_19','child_32'}
        for name in ['parent_traces.jsonl.gz','child_traces.jsonl.gz']:
            selected=[]
            for r in b.stream(ARCHIVE/name):
                if r['id'] in targets:selected.append(r)
            for id in sorted({r['id'] for r in selected}):
                rows=[r for r in selected if r['id']==id]
                index=int(id.split('_')[1])
                p=self.parents[index] if id.startswith('parent') else self.parents[self.draws[index]['parent']]
                initial=p['inventory'] if id.startswith('parent') else b.raw_edit(p,self.draws[index])
                before=copy.deepcopy(initial)
                self.assertEqual(a.consume_trace(iter(rows),id,initial,p,100),b.consume_trace(iter(rows),id,initial,p,100))
                self.assertEqual(initial,before)

    def test_shared_frame_inputs_stay_unchanged(self):
        rows=list(islice(b.stream(ARCHIVE/'parent_traces.jsonl.gz'),3))
        start,step=rows[:2];initial=copy.deepcopy(start['initial'])
        nodes,edges=b.maps(initial);before=copy.deepcopy((nodes,edges,step))
        an=a.Allocation(nodes,edges,world_only=True);bn=b.Allocation(nodes,edges,world_only=True)
        got=a.replay_step(step,start['table'],nodes,edges,an)
        self.assertEqual(got,b.replay_step(step,start['table'],nodes,edges,bn))
        self.assertEqual((nodes,edges,step),before)

    def test_world_mutations_reject_even_after_warm_replay(self):
        rows=[]
        for r in b.stream(ARCHIVE/'parent_traces.jsonl.gz'):
            rows.append(r)
            if r['kind']=='end':break
        p=self.parents[0]
        a.consume_trace(iter(rows),'parent_0',p['inventory'],p,100)
        mutations=[]
        changed=copy.deepcopy(rows);changed[1]['native']['stats']['property_effects_applied']=1;mutations.append(changed)
        changed=copy.deepcopy(rows);changed[1]['ceiling']=not changed[1]['ceiling'];mutations.append(changed)
        changed=copy.deepcopy(rows);changed[-1]['terminal']['nodes'][0][1]['data']['energy']=7.0;mutations.append(changed)
        changed=copy.deepcopy(rows);changed[-1]['complete']=False;mutations.append(changed)
        for bad in mutations:
            for checker in [a,b]:
                with self.assertRaises((AssertionError,KeyError,StopIteration)):
                    checker.consume_trace(iter(bad),'parent_0',p['inventory'],p,100)

    def test_generation_handle_and_endpoint_order_remain_strict(self):
        nodes={'NodeId(1v1)':a.node_data(0),'NodeId(3v1)':a.node_data(0)}
        edges={'EdgeId(1v1)':{'label':0,'nodes':[{'idx':1,'version':1},{'idx':3,'version':1}]}}
        expected=a.world_fingerprint(nodes,edges)
        for changed in [{'EdgeId(1v3)':edges['EdgeId(1v1)']},{'EdgeId(1v1)':{**edges['EdgeId(1v1)'],'nodes':list(reversed(edges['EdgeId(1v1)']['nodes']))}}]:
            self.assertNotEqual(a.world_fingerprint(nodes,changed),expected)
            self.assertEqual(a.world_fingerprint(nodes,changed),b.world_fingerprint(nodes,changed))

    def test_property_replay_and_allocator_inputs_are_owned(self):
        id={'idx':1,'version':1};nodes={'NodeId(1v1)':a.node_data(0,[[10,{'Int':1}]])}
        inv=b.inventory(nodes,{},0);before=copy.deepcopy(inv)
        actions=[{'op':'set_property','id':id,'key':10,'value':{'Int':2}},{'op':'node_label','id':id,'label':5}]
        x=a.Builder(inv,actions);x.property(id,10,{'Int':2});x.label(id,5)
        self.assertEqual(inv,before)
        after=x.finish();self.assertEqual(after['nodes'][0][1]['data']['label'],5)
        self.assertEqual(after['nodes'][0][1]['data']['properties'],[[10,{'Int':2}]])

    def test_material_clone_retains_types_order_and_detaches_nested_values(self):
        value={'typed':[['Int',7],['Float',7.0],['Bool',True]],'signed_zero':-0.0,'unicode':'lambda λ','endpoints':[{'idx':1,'version':1},{'idx':1,'version':3}], 'properties':[[0,{'Float':0.0}],[0,{'Int':0}]]}
        clone=a.material_copy(value)
        self.assertEqual(clone,value)
        self.assertIs(type(clone['typed'][0][1]),int)
        self.assertIs(type(clone['typed'][1][1]),float)
        self.assertIs(type(clone['typed'][2][1]),bool)
        self.assertEqual(math.copysign(1,clone['signed_zero']),-1)
        clone['endpoints'][0]['version']=999
        clone['properties'][0][1]['Float']=5.0
        self.assertEqual(value['endpoints'][0]['version'],1)
        self.assertEqual(value['properties'][0][1]['Float'],0.0)

    def test_handle_cache_is_bounded_generation_sensitive_and_read_only(self):
        a.handle.cache_clear()
        self.assertEqual(a.handle('NodeId(1v1)'),(1,1))
        self.assertEqual(a.handle('NodeId(1v3)'),(1,3))
        self.assertIsInstance(a.handle('NodeId(1v1)'),tuple)
        for i in range(5000):self.assertEqual(a.handle(f'NodeId({i}v1)'),(i,1))
        self.assertLessEqual(a.handle.cache_info().currsize,4096)
        for bad in ['NodeId(1v1)junk','NodeId(-1v1)','garbage']:
            with self.assertRaises(AssertionError):a.handle(bad)
        self.assertEqual(a.handle('NodeId(1v3)'),(1,3))

    def test_candidate_cli_refuses_every_write_stage(self):
        before=(ARCHIVE/'output_sha256.json').read_bytes()
        for args in [[str(ARCHIVE),'--development'],[str(ARCHIVE),'--development','--stage','freeze','--verify-only'],[str(ARCHIVE),'--development','--stage','seal','--verify-only']]:
            with patch.object(sys,'argv',['candidate',*args]):
                with self.assertRaises(PermissionError):a.main()
        self.assertEqual((ARCHIVE/'output_sha256.json').read_bytes(),before)

    def test_disabled_assertions_refuse_before_receipt_access(self):
        r=subprocess.run([sys.executable,'-B','-O',str(ROOT/'scripts/verify_ere_complete_coherent_optimized.py'),str(ARCHIVE),'--development','--stage','full','--verify-only'],capture_output=True,text=True)
        self.assertNotEqual(r.returncode,0)
        self.assertIn('requires Python assertions enabled',r.stderr)


if __name__=='__main__':
    suite=unittest.TestSuite()
    suite.addTests(unittest.defaultTestLoader.loadTestsFromTestCase(predicates.PredicateTests))
    suite.addTests(unittest.defaultTestLoader.loadTestsFromTestCase(DifferentialTests))
    result=unittest.TextTestRunner(verbosity=2).run(suite)
    sys.exit(not result.wasSuccessful())
