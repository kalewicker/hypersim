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
import tempfile
import gzip
import unittest
from unittest.mock import patch

ROOT=Path(__file__).resolve().parents[1]
ARCHIVE=ROOT/'output/ere_complete_coherent_optimized_resource_pilot_20260928'
sys.path.insert(0,str(ARCHIVE/'source_snapshots/scripts'))


def module(name,path):
    spec=importlib.util.spec_from_file_location(name,path)
    m=importlib.util.module_from_spec(spec);sys.modules[name]=m;spec.loader.exec_module(m)
    return m


b=module('frozen_baseline',ARCHIVE/'source_snapshots/scripts/verify_ere_complete_coherent_optimized_resource_pilot.py')
a=module('optimized_candidate',ROOT/'scripts/verify_ere_coherent_draw_optimized.py')
predicates=module('original_predicates',ROOT/'scripts/test_verify_ere_complete_coherent_optimized_resource_pilot.py')
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

    def test_integrated_cli_refuses_historical_receipt_and_wrong_mode(self):
        before=(ARCHIVE/'output_sha256.json').read_bytes()
        for args in [[str(ARCHIVE)],*[ [str(ARCHIVE),'--development','--stage',stage] for stage in ['freeze','controls','full','seal']], [str(ARCHIVE),'--development','--stage','freeze','--verify-only'],[str(ARCHIVE),'--development','--stage','seal','--verify-only']]:
            with patch.object(sys,'argv',['candidate',*args]):
                with self.assertRaises(PermissionError):a.main()
        self.assertEqual((ARCHIVE/'output_sha256.json').read_bytes(),before)

    def test_new_root_writes_are_exclusive_bounded_and_symlink_safe(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp).resolve()
            with patch.object(a,'ARCHIVE_ROOT',root), patch.object(a,'OUTPUT_CAP',64):
                a.save(root/'one.json',{"ok":True})
                with self.assertRaises(FileExistsError):a.save(root/'one.json',{})
                with self.assertRaises(AssertionError):a.save(root/'two.json',{"large":"x"*65})
                self.assertFalse((root/'two.json').exists())
                (root/'nested').mkdir()
                with self.assertRaises(AssertionError):a.save(root/'nested'/'bad.json',{})
                (root/'symlink').symlink_to(root/'one.json')
                with self.assertRaises(AssertionError):a.save(root/'three.json',{})
                self.assertFalse((root/'three.json').exists())

    def test_paired_wire_digest_ignores_gzip_header_not_material(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp).resolve()
            for name,mtime,data in [('a',1,b'{"typed":{"Int":1}}\\n'),('b',2,b'{"typed":{"Int":1}}\\n'),('c',1,b'{"typed":{"Float":1.0}}\\n')]:
                with gzip.GzipFile(filename=str(root/name),mode='wb',mtime=mtime) as f:f.write(data)
            self.assertEqual(a.wire_digest(root/'a'),a.wire_digest(root/'b'))
            self.assertNotEqual(a.wire_digest(root/'a'),a.wire_digest(root/'c'))

    def test_unmodified_semantic_functions_remain_verbatim(self):
        import ast
        winner=ast.parse((ARCHIVE/'source_snapshots/scripts/verify_ere_complete_coherent_optimized_resource_pilot.py').read_text())
        port=ast.parse((ROOT/'scripts/verify_ere_complete_coherent_optimized_resource_pilot.py').read_text())
        wanted={'material_copy','handle','maps','world_fingerprint','replay_step','consume_trace','decode','Builder','Allocation','CoherentBuilder','classification','tally'}
        def extract(tree):
            return {n.name:ast.dump(n,include_attributes=False) for n in tree.body if isinstance(n,(ast.FunctionDef,ast.ClassDef)) and n.name in wanted}
        self.assertEqual(extract(winner),extract(port))
        self.assertEqual(set(extract(port)),wanted)


    def test_checked_draw_is_fresh_owned_and_result_matches_standalone_decode(self):
        with patch.object(a, 'DRAW_EVALUATIONS', 0):
            for d in self.draws:
                p=self.parents[d['parent']]; before=copy.deepcopy((p,d))
                after,result=a._checked_draw(p,d)
                self.assertEqual(after,b.raw_edit(p,d))
                self.assertEqual(result,b.decode(after,p['root']))
                self.assertEqual((p,d),before)
                result['validator']='mutated-result'
                after['step']=999
                fresh,derived=a._checked_draw(p,d)
                self.assertEqual(fresh,b.raw_edit(p,d))
                self.assertEqual(derived,b.decode(fresh,p['root']))
            self.assertEqual(a.DRAW_EVALUATIONS,128)

    def test_standalone_classification_rederives_and_rejects_forged_row(self):
        rows=list(b.stream(ARCHIVE/'classifications.jsonl.gz'))
        d=self.draws[0];row=rows[0];p=self.parents[d['parent']]
        id=f"parent_{p['index']}"
        selected=(r for r in b.stream(ARCHIVE/'parent_traces.jsonl.gz') if r['id']==id)
        base=b.consume_trace(selected,id,p['inventory'],p,100)
        child=b.consume_trace(b.stream(ARCHIVE/'child_traces.jsonl.gz'),'child_0',b.raw_edit(p,d),p,100)
        with patch.object(a, '_checked_draw', wraps=a._checked_draw) as replay:
            self.assertEqual(a.check_classification(p,d,row,base,child,0,0),b.check_classification(p,d,row,base,child,0,0))
            self.assertEqual(replay.call_count,1)
            bad=copy.deepcopy(row);bad['decoded']['program']['rcf']='00'
            with self.assertRaises(AssertionError):a.check_classification(p,d,bad,base,child,0,0)
            self.assertEqual(replay.call_count,2)
            bad_draw=copy.deepcopy(d);bad_draw['edit']['delta']['edges']['deleted'].append('EdgeId(999999v1)')
            bad=copy.deepcopy(row);bad['manifest']=bad_draw
            with self.assertRaises(AssertionError):a.check_classification(p,bad_draw,bad,base,child,0,0)
            self.assertEqual(replay.call_count,3)

    def test_candidate_requires_read_only_full_or_controls(self):
        for args in [[str(ARCHIVE),'--development','--stage',stage] for stage in ['freeze','controls','full','seal']]:
            with patch.object(sys,'argv',['candidate',*args]):
                with self.assertRaises(PermissionError):a.main()

    def test_refactor_preserves_all_replay_and_classification_checks(self):
        import ast
        def functions(path):
            return {n.name:n for n in ast.parse(path.read_text()).body if isinstance(n,ast.FunctionDef)}
        before=functions(ARCHIVE/'source_snapshots/scripts/verify_ere_complete_coherent_optimized_resource_pilot.py')
        after=functions(ROOT/'scripts/verify_ere_coherent_draw_optimized.py')
        original=before['raw_edit'].body[1:-1]
        port=after['_checked_draw'].body[3:-1]
        self.assertEqual([ast.dump(n,include_attributes=False) for n in original],[ast.dump(n,include_attributes=False) for n in port])
        original=before['check_classification'].body[4:]
        port=after['_check_classification_decoded'].body[2:]
        self.assertEqual([ast.dump(n,include_attributes=False) for n in original],[ast.dump(n,include_attributes=False) for n in port])
        for stage in ['freeze','seal']:
            with patch.object(sys,'argv',['candidate',str(ARCHIVE),'--development','--stage',stage,'--verify-only']):
                with self.assertRaises(PermissionError):a.main()

    def test_driver_scientific_comparison_keeps_all_nonclock_evidence(self):
        d=module('draw_profile_driver', ROOT/'scripts/profile_ere_coherent_draw_auditor.py')
        value={'phase_seconds':{},'peak_rss_bytes':1,'child_replay_timing':[],'optimization_stats':{},'paired_resource_pilot_parity':{'identical':True},'resources':{'attempted':22300},'tamper_categories':['one'],'gate_pass':False}
        self.assertEqual(d.semantic_result(value),{k:v for k,v in value.items() if k not in ['phase_seconds','peak_rss_bytes','child_replay_timing','optimization_stats']})
        for key in ['resources','paired_resource_pilot_parity','tamper_categories','gate_pass']:
            bad=copy.deepcopy(value);bad[key]='forged'
            self.assertNotEqual(d.semantic_result(bad),d.semantic_result(value))

    def test_disabled_assertions_refuse_before_receipt_access(self):
        r=subprocess.run([sys.executable,'-B','-O',str(ROOT/'scripts/verify_ere_complete_coherent_optimized_resource_pilot.py'),str(ARCHIVE),'--development','--stage','full','--verify-only'],capture_output=True,text=True)
        self.assertNotEqual(r.returncode,0)
        self.assertIn('requires Python assertions enabled',r.stderr)


if __name__=='__main__':
    suite=unittest.TestSuite()
    suite.addTests(unittest.defaultTestLoader.loadTestsFromTestCase(predicates.PredicateTests))
    suite.addTests(unittest.defaultTestLoader.loadTestsFromTestCase(DifferentialTests))
    result=unittest.TextTestRunner(verbosity=2).run(suite)
    sys.exit(not result.wasSuccessful())
