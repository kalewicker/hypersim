import copy
import json
import math
from pathlib import Path
import subprocess
import tempfile
import unittest
import audit_ere_categorical_adapter as v
import verify_ere_categorical_adapter as harness

class CategoricalTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.cat=v.a.load(v.INPUT/'catalogue.json')
        rows=[v.a.load(v.INPUT/'cells'/(id+'.json')) for id in v.source_ids()]
        cls.request=v.request_from_receipts(cls.cat,rows)
        cls.result=dict(profile=v.PROFILE,classification='engineering_arithmetic_only',native_steps_run=0,grammar_compilations=0,
            training_updates=0,training_admitted=False,catalogue_sha256=v.a.CATALOGUE,request_sha256='test-math-only',
            batches=[dict(id=b['id'],estimate=v.estimate(cls.cat,b)) for b in cls.request['batches']],score_cases=[],oracles=[])
        for case in cls.request['score_cases']:
            cls.result['score_cases'].append(dict(id=case['id'],score=v.score(cls.cat,case['theta'],v.selection(case['emission'])),native_score=None,reason='resource_cap',utility=0.))
        _,groups,_=v.mapping.catalogue_model(cls.cat)
        for t in cls.request['oracle_contexts']:
            p=v.mapping.probabilities(cls.cat,t,groups)
            cls.result['oracles'].append(dict(theta=t,**{feature:dict(value=p[feature+'_mass'],mass=1.,gradient=v.mapping.expectation_gradient(p,groups,feature)) for feature in ('active','eligible')}))

    def test_full_catalogue_sample_layout_and_no_new_work(self):
        self.assertEqual(len(self.request['oracle_leaves']),18956);self.assertEqual(len(self.request['batches']),20)
        self.assertEqual(len(self.request['score_cases']),5)
        for b in self.request['batches'][:16]:
            self.assertEqual(b['expected_draws'],16);self.assertEqual(len({s['emission_seed'] for s in b['samples']}),16)
        self.assertEqual(len(v.source_ids()),261);self.assertEqual(len(set(v.source_ids())),261)

    def test_all_coordinates_log_derivative_and_singleton_masks(self):
        t=self.request['oracle_contexts'][1];leaf=self.request['oracle_leaves'][100]['selection'];s=v.score(self.cat,t,leaf)
        for i in range(183):
            # Use the effective f32 perturbation width, not the unrounded request.
            plus=t[:];minus=t[:];plus[i]=v.f32(plus[i]+.001);minus[i]=v.f32(minus[i]-.001)
            fd=(v.score(self.cat,plus,leaf)['log_probability']-v.score(self.cat,minus,leaf)['log_probability'])/(plus[i]-minus[i])
            self.assertAlmostEqual(fd,s['gradient'][i],delta=1e-7)
        singleton=next(x['selection'] for x in self.request['oracle_leaves'] if len(self.cat['parents'][x['selection']['parent']]['groups'][x['selection']['operator']]['role_support'])==1)
        self.assertEqual(v.score(self.cat,t,singleton)['gradient'][181:],[0.,0.])

    def test_loo_not_self_centering_caps_and_fixed_denominator(self):
        e=self.result['batches'][-2]['estimate'];self.assertEqual([c['utility'] for c in e['contributions']],[0.,2.])
        self.assertEqual([c['baseline'] for c in e['contributions']],[2.,0.]);self.assertEqual(e['utility_mean'],1.)
        self.assertLess(e['gradient'][0],0);self.assertGreater(e['gradient'][1],0)
        for b in self.result['batches'][-4:-2]:self.assertEqual(b['estimate']['gradient'],[0.]*183)
        b=copy.deepcopy(self.request['batches'][-1]);b['expected_draws']=3
        with self.assertRaises(AssertionError):v.estimate(self.cat,b)
        b=copy.deepcopy(self.request['batches'][-1]);b['samples'][1]['emission_seed']=b['samples'][0]['emission_seed']
        with self.assertRaises(AssertionError):v.estimate(self.cat,b)

    def test_unsupported_nulls_and_fake_cap_scores_refuse(self):
        for score,reason in [(None,None),(None,'crash'),(0.,'resource_cap'),(math.nan,None),(1.1,None)]:
            with self.assertRaises(AssertionError):v.utility(dict(native_score=score,reason=reason))
        self.assertEqual(v.utility(dict(native_score=None,reason='resource_cap')),0.)

    def test_f32_roundtrip_and_exact_update(self):
        for b in self.result['batches']:
            e=b['estimate'];r=json.loads(json.dumps(e));v.bits(e['theta_after'],r['theta_after'])
            v.bits(e['theta_after'],[v.f32(x+v.f32(e['alpha'])*y) for x,y in zip(e['theta_before'],e['gradient'])])
        bad=copy.deepcopy(self.result['batches'][0]['estimate']);bad['theta_after'][0]+=.00001
        with self.assertRaises(AssertionError):v.check_batch(bad,self.cat,self.request['batches'][0])

    def test_independent_reconstruction_and_all_detached_tampers(self):
        check=v.check_result(self.request,self.result,self.cat);self.assertEqual(check['finite_difference_coordinates'],0)
        self.assertEqual(len(v.tamper_checks(self.request,self.result,self.cat)),18)

    def test_raw_receipt_binding_and_oracle_features_not_reward(self):
        missing=[v.a.load(v.INPUT/'cells'/(id+'.json')) for id in v.source_ids()[:-1]]
        with self.assertRaises(AssertionError):v.request_from_receipts(self.cat,missing)
        row=v.a.load(v.INPUT/'cells'/(v.source_ids()[0]+'.json'));bad=copy.deepcopy(row);bad['parameters'][0]=1.
        rows=[v.a.load(v.INPUT/'cells'/(id+'.json')) for id in v.source_ids()];rows[0]=bad
        with self.assertRaises(AssertionError):v.request_from_receipts(self.cat,rows)
        self.assertEqual(sum(x['active'] for x in self.request['oracle_leaves']),9478)
        self.assertEqual(sum(x['eligible'] for x in self.request['oracle_leaves']),9282)

    def test_cli_invalid_inputs_and_existing_output_refuse_without_write(self):
        binary=v.ROOT/'target/debug/ere-categorical-adapter'
        self.assertTrue(binary.is_file(),'build the offline binary before running this test group')
        request=copy.deepcopy(self.request);request['batches']=[request['batches'][-2]];request['score_cases']=[];request['oracle_contexts']=[];request['oracle_leaves']=[]
        mutations=[
            (lambda x:x['batches'][0].__setitem__('expected_draws',3),'complete fixed sample'),
            (lambda x:x['batches'][0]['samples'][1].__setitem__('emission_seed',0),'distinct emission seed'),
            (lambda x:x['batches'][0]['samples'][1].__setitem__('id',x['batches'][0]['samples'][0]['id']),'unique nonempty'),
            (lambda x:x['batches'][0]['samples'][0].__setitem__('reason',None),'unsupported outcome'),
            (lambda x:x['batches'][0]['samples'][0].__setitem__('native_score',0.),'unsupported outcome'),
            (lambda x:x['batches'][0]['samples'][0]['selection'].__setitem__('operator',999),'index outside'),
            (lambda x:x['batches'][0].__setitem__('theta',[0.]*3),'183 finite'),
            (lambda x:x['batches'][0].__setitem__('alpha',-.1),'nonnegative'),
            (lambda x:x['batches'][0]['theta'].__setitem__(0,True),'invalid type'),
            (lambda x:x.__setitem__('purpose','scientific_training'),'engineering-only'),
            (lambda x:x.__setitem__('catalogue_sha256','0'*64),'catalogue hash'),
        ]
        with tempfile.TemporaryDirectory(prefix='ere-categorical-tests-') as directory:
            root=Path(directory);inp=root/'request.json';out=root/'result.json'
            for change,message in mutations:
                bad=copy.deepcopy(request);change(bad);inp.write_text(json.dumps(bad))
                run=subprocess.run([str(binary),str(v.INPUT/'catalogue.json'),str(inp),str(out)],capture_output=True,text=True)
                self.assertNotEqual(run.returncode,0);self.assertIn(message,run.stderr);self.assertFalse(out.exists())
            inp.write_text(json.dumps(request));subprocess.run([str(binary),str(v.INPUT/'catalogue.json'),str(inp),str(out)],check=True,capture_output=True)
            before=out.read_bytes();run=subprocess.run([str(binary),str(v.INPUT/'catalogue.json'),str(inp),str(out)],capture_output=True,text=True)
            self.assertNotEqual(run.returncode,0);self.assertIn('fresh result file required',run.stderr);self.assertEqual(out.read_bytes(),before)

    def test_frozen_binary_keeps_execution_permission_and_bytes(self):
        binary=v.ROOT/'target/debug/ere-categorical-adapter'
        with tempfile.TemporaryDirectory(prefix='ere-categorical-mode-') as directory:
            target=Path(directory)/'executed_binary';harness.freeze_binary(binary,target)
            self.assertEqual(v.a.digest(target),v.a.digest(binary))
            self.assertEqual(target.stat().st_mode & 0o777,binary.stat().st_mode & 0o777)
            run=subprocess.run([str(target)],capture_output=True,text=True)
            self.assertNotEqual(run.returncode,0);self.assertIn('usage:',run.stderr)

if __name__=='__main__':unittest.main()
