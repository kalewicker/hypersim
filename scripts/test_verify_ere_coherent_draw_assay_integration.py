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
a=module('optimized_candidate',ROOT/'scripts/verify_ere_complete_coherent_draw_assay.py')
predicates=module('original_predicates',ROOT/'scripts/test_verify_ere_complete_coherent_draw_assay.py')
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
        port=ast.parse((ROOT/'scripts/verify_ere_complete_coherent_draw_assay.py').read_text())
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

    def test_new_profile_refuses_historical_write_stages(self):
        for args in [[str(ARCHIVE),'--development','--stage',stage] for stage in ['freeze','controls','full','seal']]:
            with patch.object(sys,'argv',['candidate',*args]):
                with self.assertRaises(PermissionError):a.main()

    def test_refactor_preserves_all_replay_and_classification_checks(self):
        import ast
        def functions(path):
            return {n.name:n for n in ast.parse(path.read_text()).body if isinstance(n,ast.FunctionDef)}
        before=functions(ARCHIVE/'source_snapshots/scripts/verify_ere_complete_coherent_optimized_resource_pilot.py')
        after=functions(ROOT/'scripts/verify_ere_complete_coherent_draw_assay.py')
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


    def test_selected_draw_semantics_are_verbatim(self):
        import ast
        wanted={'material_copy','handle','maps','world_fingerprint','replay_step','consume_trace','decode','Builder','Allocation','CoherentBuilder','classification','tally','_checked_draw','raw_edit','check_classification','_check_classification_decoded'}
        def extract(path):
            return {n.name:ast.dump(n,include_attributes=False) for n in ast.parse(path.read_text()).body if isinstance(n,(ast.FunctionDef,ast.ClassDef)) and n.name in wanted}
        expected=extract(ROOT/'scripts/verify_ere_coherent_draw_optimized.py')
        got=extract(ROOT/'scripts/verify_ere_complete_coherent_draw_assay.py')
        self.assertEqual(got,expected)
        self.assertEqual(set(got),wanted)

    def test_new_profile_requires_explicit_mode_before_receipt_access(self):
        with patch.object(sys,'argv',['new_profile','missing-receipt','--stage','freeze']):
            with self.assertRaises(PermissionError):a.main()

    def test_disabled_assertions_refuse_before_receipt_access(self):
        r=subprocess.run([sys.executable,'-B','-O',str(ROOT/'scripts/verify_ere_complete_coherent_draw_assay.py'),str(ARCHIVE),'--development','--stage','full','--verify-only'],capture_output=True,text=True)
        self.assertNotEqual(r.returncode,0)
        self.assertIn('requires Python assertions enabled',r.stderr)


class FullAssayTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.design=a.load(ROOT/'docs/ere_complete_coherent_draw_assay_v1.json')
        cls.pilot=ROOT/'output/ere_complete_coherent_draw_resource_pilot_20260928'

    def test_contract_and_all_new_basis_pins_match_current_material(self):
        self.assertEqual(a.digest(ROOT/'docs/ere_complete_coherent_draw_assay_v1.md'),a.CONTRACT_MD_SHA)
        self.assertEqual(a.digest(ROOT/'docs/ere_complete_coherent_draw_assay_v1.json'),a.CONTRACT_JSON_SHA)
        for path,sha in self.design['basis']['design_time_source_sha256'].items():
            self.assertEqual(a.digest(ROOT/path),sha,path)
        for key in ['raw_baseline','coherent_baseline','identity_baseline','isolated_baseline','resource_baseline','optimized_baseline','integrated_baseline','draw_baseline','latest_pilot_baseline']:
            for name,sha in self.design[key]['sha256'].items():
                self.assertEqual(a.digest(ROOT/self.design[key]['archive']/name),sha,f'{key}/{name}')
        self.assertFalse(self.design['scientific_run_authorized'])
        self.assertFalse(self.design['training_admitted'])

    def test_full_resources_and_pending_launch_are_exact(self):
        r=self.design['proposed_resources']
        expected={'advancing_engine_steps_max':1016800,'thread_count':1,'assay_wall_time_cap_seconds':3600,'assay_output_cap_bytes':4*1024**3,'auditor_wall_time_cap_seconds_shared':900,'auditor_resident_memory_cap_bytes':1024**3,'classification_work_limit_per_step':400000,'classification_result_limit_per_step':50000,'runtime_work_limit_unchanged':50000,'runtime_result_limit_unchanged':50000,'full_store_node_cap':2000,'full_store_edge_cap':4000,'oracle_or_cap_replay_budget':0}
        for key,value in expected.items():self.assertEqual(r[key],value,key)
        self.assertTrue(self.design['launch']['explicit_user_approval_required'])
        self.assertFalse((ROOT/self.design['proposed_artifact_directory']).exists())

    def test_frozen_full_authorization_rejects_missing_or_changed_tokens(self):
        expected={'profile':a.PROFILE,'approved_steps':'1016800','approved_caps':'3600s,4GiB,900s,1GiB,400k/50k','contract_md_sha256':a.CONTRACT_MD_SHA,'contract_json_sha256':a.CONTRACT_JSON_SHA,'authorization_source':'explicit user launch approval represented by launcher environment','training_admitted':False}
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);file=root/'launch_authorization.json';m={'development_only':False}
            with self.assertRaises(FileNotFoundError):a.check_authorization(root,m)
            file.write_text(json.dumps(expected));a.check_authorization(root,m)
            for key in expected:
                bad=copy.deepcopy(expected);bad[key]='forged'
                file.write_text(json.dumps(bad))
                with self.subTest(key=key):
                    with self.assertRaises(AssertionError):a.check_authorization(root,m)
            with self.assertRaises(AssertionError):a.check_authorization(root,{'development_only':True})
            file.unlink();a.check_authorization(root,{'development_only':True})

    def population(self,n=10000):
        return {'parents.jsonl.gz':[{'index':i} for i in range(168)],'draw_manifest.jsonl.gz':[{'index':i,'parent':i%168,'operator_index':i%13,'operator':a.OPERATORS[i%13],'role':'Core' if i%13<11 else 'Family','rng_word_pos_after':str(i+1)} for i in range(n)],'control_manifest.jsonl.gz':[{'parent':i//32,'attack':self.design['completeness_controls']['attacks'][i%32]} for i in range(5376)]}

    def check_population(self,rows):
        with patch.object(a,'stream',side_effect=lambda path:iter(rows[path.name])),patch.object(a,'check_parent') as checked,patch.object(a,'design',return_value=self.design):
            a.check_frozen_population(Path('/synthetic-unit-fixture'),{'draws':10000})
            self.assertEqual(checked.call_count,168)

    def test_zero_engine_full_frozen_population_requires_exact_counts(self):
        self.check_population(self.population())
        for name in ['parents.jsonl.gz','draw_manifest.jsonl.gz','control_manifest.jsonl.gz']:
            rows=self.population();rows[name].pop()
            with self.subTest(name=name):
                with self.assertRaises(AssertionError):self.check_population(rows)
        rows=self.population(10001)
        with self.assertRaises(AssertionError):self.check_population(rows)

    def test_full_population_rejects_duplicate_draws_roles_rng_and_controls(self):
        for stream_name,key,value in [('draw_manifest.jsonl.gz','index',0),('draw_manifest.jsonl.gz','operator','Forged'),('draw_manifest.jsonl.gz','role','Family'),('draw_manifest.jsonl.gz','rng_word_pos_after','1'),('control_manifest.jsonl.gz','parent',9),('control_manifest.jsonl.gz','attack','forged')]:
            rows=self.population();rows[stream_name][1][key]=value
            with self.subTest(key=key):
                with self.assertRaises(AssertionError):self.check_population(rows)

    def test_prefix_uses_canonical_bytes_and_rejects_omission(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);old=root/'old.gz';new=root/'new.gz'
            def write(path,rows):
                with gzip.open(path,'wt') as f:
                    for row in rows:f.write(json.dumps(row,sort_keys=True,separators=(',',':'))+'\n')
            write(old,[{'x':1}]);write(new,[{'x':1},{'x':2}])
            receipt,tail=a.exact_prefix(new,old)
            self.assertEqual(receipt['rows'],1);self.assertEqual(tail,{'x':2})
            for rows in [[],[{'x':1.0}],[{'x':2}]]:
                write(new,rows)
                with self.assertRaises(AssertionError):a.exact_prefix(new,old)

    def parity_fixture(self,root):
        import shutil
        basis=root/'basis'/self.design['latest_pilot_baseline']['archive'];basis.mkdir(parents=True)
        for name in a.PARITY_STREAMS+['classification_summary.json']:
            shutil.copyfile(self.pilot/name,root/name);shutil.copyfile(self.pilot/name,basis/name)
        return basis

    def append_stream(self,path,rows):
        with gzip.open(path,'at') as f:
            for row in rows:f.write(json.dumps(row,sort_keys=True,separators=(',',':'),ensure_ascii=False)+'\n')

    def test_parity_accepts_full_extension_but_not_changed_prefix(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);self.parity_fixture(root)
            for name in ['draw_manifest.jsonl.gz','classifications.jsonl.gz']:
                row=next(a.stream(root/name));row['index']=64;self.append_stream(root/name,[row])
            child=[]
            for row in a.stream(root/'child_traces.jsonl.gz'):
                if row['id']!='child_0':break
                row['id']='child_64';child.append(row)
            self.append_stream(root/'child_traces.jsonl.gz',child)
            full=a.load(root/'classification_summary.json');full['numerical_pass']=True;full['strata']['overall']['samples']=10000
            (root/'classification_summary.json').write_text(json.dumps(full))
            result=a.paired_pilot_parity(root,self.design)
            self.assertTrue(result['pilot_prefix_summary_equal'])
            self.assertEqual(result['identical_pilot_prefixes']['child_traces.jsonl.gz']['rows'],5610)
            rows=list(a.stream(root/'draw_manifest.jsonl.gz'));rows[0]['parent']=167
            with gzip.open(root/'draw_manifest.jsonl.gz','wt') as f:
                for row in rows:f.write(json.dumps(row,sort_keys=True,separators=(',',':'),ensure_ascii=False)+'\n')
            with self.assertRaises(AssertionError):a.paired_pilot_parity(root,self.design)

    def test_native_timing_uses_full_wall_ceiling(self):
        c={'controls':5376,'attempted_engine_steps':0,'seconds':901}
        t={'draws':10000,'horizon':100,'parent_phase_seconds':901,'mutation_phase_seconds':1801,'child_trace_seconds':0,'mutation_nontrace_seconds':1801,'parent_traces':[{'id':f'parent_{i}','seconds':0,'attempted_steps':0} for i in range(168)],'child_traces':[]}
        with patch.object(a,'load',side_effect=[c,t]),patch.object(a,'stream',return_value=iter([])):
            a.check_native_timing(Path('/synthetic'),{'draws':10000,'horizon':100,'development_only':False},{'child_traces':0,'attempted':0})
        for field in ['seconds']:
            c[field]=3600
            with patch.object(a,'load',side_effect=[c,t]),patch.object(a,'stream',return_value=iter([])):
                with self.assertRaises(AssertionError):a.check_native_timing(Path('/synthetic'),{'draws':10000,'horizon':100,'development_only':False},{'child_traces':0,'attempted':0})

    def test_seal_keeps_nested_basis_seals_and_rejects_changed_basis(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);(root/'basis').mkdir();(root/'basis/output_sha256.json').write_text('{}')
            (root/'output_sha256.json').write_text(json.dumps({'basis/output_sha256.json':a.digest(root/'basis/output_sha256.json')}))
            a.check_seal(root)
            (root/'basis/output_sha256.json').write_text('{"changed":true}')
            with self.assertRaises(AssertionError):a.check_seal(root)

    def test_full_numerical_predicate_can_pass_and_missing_requirements_fail(self):
        seed=next(a.stream(self.pilot/'classifications.jsonl.gz'))
        rows=[]
        for i in range(10000):
            r=copy.copy(seed);op=i%13
            r.update(operator_index=op,operator=a.OPERATORS[op],role='Core' if (i//13)%2 else 'Proposal',parent_active=bool((i//13)%2),at_cap=bool(i%2),logical_topology_changed=True,outcome='neutral' if i<50 else 'near_neutral',mode_changed=op>=11,trace_changed=op>=11,soundness_checked=i<50,soundness_failure=False)
            rows.append(r)
        self.assertTrue(a.tally(rows)['numerical_pass'])
        self.assertFalse(a.tally(rows[:-1])['numerical_pass'])
        for key,value in [('soundness_checked',False),('parent_active',False),('logical_topology_changed',False),('mode_changed',False),('trace_changed',False),('at_cap',False),('role','Proposal'),('outcome','catastrophic')]:
            bad=[{**r,key:value} for r in rows]
            with self.subTest(key=key):self.assertFalse(a.tally(bad)['numerical_pass'])
        for key,value in [('soundness_failure',True),('outcome','unmeasured')]:
            bad=[r.copy() for r in rows];bad[0][key]=value
            self.assertFalse(a.tally(bad)['numerical_pass'])

    def test_native_scientific_functions_and_thresholds_are_text_identical(self):
        import re
        def functions(path):
            text=path.read_text();matches=list(re.finditer(r'^fn (\w+)(?:<[^\n]+>)?\(',text,re.M))
            return {m.group(1):text[m.start():matches[i+1].start() if i+1<len(matches) else len(text)].split('\n#[test]')[0] for i,m in enumerate(matches)}
        before=functions(ROOT/'crates/hg-sim-core/tests/ere_complete_coherent_draw_resource_pilot.rs')
        after=functions(ROOT/'crates/hg-sim-core/tests/ere_complete_coherent_draw_assay.rs')
        for name in ['world','parents','source','compile','decoded','allele','choose','scalar','legacy_raw_edit','pattern_topology','raw_edit','draw','attack','controls','typed_world','neutral_failure','engine','trace','frozen_window','outcome','measurement_complete','at_cap','tally','primary']:
            self.assertEqual(before[name],after[name],name)
        # The tally implementation carries the full sample/coverage predicates.
        def tally_impl(path):return path.read_text().split('impl Tally {',1)[1].split('\nfn tally(',1)[0]
        self.assertEqual(tally_impl(ROOT/'crates/hg-sim-core/tests/ere_complete_coherent_draw_resource_pilot.rs'),tally_impl(ROOT/'crates/hg-sim-core/tests/ere_complete_coherent_draw_assay.rs'))

if __name__=='__main__':
    suite=unittest.TestSuite()
    suite.addTests(unittest.defaultTestLoader.loadTestsFromTestCase(predicates.PredicateTests))
    suite.addTests(unittest.defaultTestLoader.loadTestsFromTestCase(DifferentialTests))
    suite.addTests(unittest.defaultTestLoader.loadTestsFromTestCase(FullAssayTests))
    result=unittest.TextTestRunner(verbosity=2).run(suite)
    sys.exit(not result.wasSuccessful())
