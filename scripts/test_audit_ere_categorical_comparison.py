import copy
import json
import math
from pathlib import Path
import subprocess
import tempfile
import unittest
import audit_ere_categorical_comparison as v
import run_ere_categorical_comparison as driver

class ComparisonTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.c=v.config();cls.catalogue=v.a.load(v.cat.INPUT/'catalogue.json');cls.native,cls.h=v.cat.d.c.helpers(v.PARENT)

    def test_exact_implementation_registration_and_frozen_plan(self):
        r=v.registration(v.ROOT,self.c);self.assertEqual(len(r['source_names']),472)
        self.assertEqual(r['unchanged_in_scope_adapter_paths'],460)
        for name,h in v.a.load(v.ROOT/v.PREREG/'manifest.json')['sources'].items():self.assertEqual(v.a.digest(v.ROOT/name),h)

    def test_saved_es_update_and_partial_estimate_caps(self):
        old=v.a.load(v.PARENT/'generations/train-0.json');qs=v.p.schedule(self.c,'development')[:16];rows={}
        for q,old_id in zip(qs,old['evaluation_ids']):
            row=copy.deepcopy(v.a.load(v.PARENT/'cells'/(old_id+'.json')));rows[q['id']]=row
        means,fits=v.cat.d.b.batch_fits([[rows[q['id']]['evaluation']['score']] for q in qs],1,self.h)
        rec=dict(block=0,context=0,method='es',estimate=0,theta_before=[0.]*183,evaluation_ids=[q['id'] for q in qs],noise_seed=qs[0]['noise_seed'],eps=old['eps'],complete=True,
            raw=dict(candidate_means=means,optimizer_pair_scores=[fits[i:i+2] for i in range(0,16,2)],theta_after=old['theta_after']),matched=v.p.matched_step([0.]*183,old['theta_after']))
        v.estimate(rec,qs,[0.]*183,rows,self.catalogue,self.native,self.h,1)
        for path in ('noise_seed','complete'):
            bad=copy.deepcopy(rec);bad[path]=None
            with self.assertRaises(AssertionError):v.estimate(bad,qs,[0.]*183,rows,self.catalogue,self.native,self.h,1)
        bad=copy.deepcopy(rec);bad['raw']['theta_after'][0]+=.001
        with self.assertRaises(AssertionError):v.estimate(bad,qs,[0.]*183,rows,self.catalogue,self.native,self.h,1)
        partial=copy.deepcopy(rec);partial.update(complete=False,raw=None,matched=None)
        self.assertIsNone(v.estimate(partial,qs,[0.]*183,rows,self.catalogue,self.native,self.h,1))

    def test_categorical_archived_native_inputs_and_fixed_loo(self):
        qs=[q for q in v.p.schedule(self.c,'development') if q['stage']=='estimation' and (q['block'],q['context'],q['method'],q['estimate'])==(0,0,'categorical',0)]
        ids=v.cat.source_ids()[:16];rows={};samples=[]
        for q,id in zip(qs,ids):
            row=copy.deepcopy(v.a.load(v.cat.INPUT/'cells'/(id+'.json')));rows[q['id']]=row
            samples.append(dict(id=q['id'],emission_seed=q['emission_seed'],physics_seed=q['physics_seed'],selection=v.cat.selection(row['emission']),native_score=row['evaluation']['score'],reason=row['evaluation']['reason']))
        batch=dict(theta=[0.]*183,samples=samples,expected_draws=16,alpha=v.p.f32(.05));raw=v.cat.estimate(self.catalogue,batch)
        rec=dict(block=0,context=0,method='categorical',estimate=0,theta_before=[0.]*183,evaluation_ids=[q['id'] for q in qs],noise_seed=None,eps=[],complete=True,raw=raw,matched=v.p.matched_step([0.]*183,raw['theta_after']))
        v.estimate(rec,qs,[0.]*183,rows,self.catalogue,self.native,self.h,1)
        bad=copy.deepcopy(rec);bad['raw']['contributions'][0]['baseline']+=.1
        with self.assertRaises(AssertionError):v.estimate(bad,qs,[0.]*183,rows,self.catalogue,self.native,self.h,1)
        bad=copy.deepcopy(rec);bad['matched']['reverse'][0]+=.0001
        with self.assertRaises(AssertionError):v.estimate(bad,qs,[0.]*183,rows,self.catalogue,self.native,self.h,1)

    def test_native_cap_and_unknown_accounting_retained(self):
        row=v.a.load(v.ROOT/'output/ere_complete_champion_confirmation_20261001/cells/d33-champion-p0.json');row['constraint_utility']=0.
        n,adv,score=v.cat.d.b.cell(row,row['schedule'],self.catalogue,self.native,self.h)
        self.assertIsNone(score);self.assertLessEqual(adv,n);self.assertEqual(row['evaluation']['reason'],'resource_cap')
        bad=copy.deepcopy(row);bad['constraint_utility']=1.
        with self.assertRaises(AssertionError):v.cat.d.b.cell(bad,bad['schedule'],self.catalogue,self.native,self.h)
        self.assertAlmostEqual(v.p.importance_effect([1.2],[0.],[0.])['utility_effect'],-.2)

    def test_offline_rust_math_matches_independent_python_full_support(self):
        binary=v.ROOT/'target/release/ere-categorical-comparison';self.assertTrue(binary.exists(),'build release binary before this group')
        raw=[[v.p.f32(.02*math.sin(i+e)) for i in range(183)] for e in range(4)]
        with tempfile.TemporaryDirectory() as directory:
            req=Path(directory)/'request.json';result=Path(directory)/'result.json';req.write_text(json.dumps(dict(purpose='engineering_math_only',raw_vectors=raw)))
            done=subprocess.run([str(binary),'math-fixture',str(v.cat.INPUT/'catalogue.json'),str(req),str(result)],capture_output=True,text=True)
            self.assertEqual(done.returncode,0,done.stderr);got=v.a.load(result)
            self.assertEqual(got['native_steps_run'],0);self.assertEqual(got['grammar_compilations'],0);self.assertFalse(got['training_admitted'])
            for ctx in got['contexts']:
                for saved,r in zip(ctx['matched'],raw):v.matched(saved,ctx['base'],r)
                v.numeric(ctx['full_support'],v.support(self.catalogue,ctx['base'],ctx['vectors']))
            align=[[[.5,1.] for _ in range(2)] for _ in range(8)];effects=[[[[.01,.01],[.02,.02]] for _ in range(2)] for _ in range(8)];validity=[[[[0.,0.],[0.,0.]] for _ in range(2)] for _ in range(8)]
            v.numeric(got['toy_statistics_not_evidence'],v.p.statistics(self.c,'scientific',align,effects,validity,True))
            again=subprocess.run([str(binary),'math-fixture',str(v.cat.INPUT/'catalogue.json'),str(req),str(result)],capture_output=True)
            self.assertNotEqual(again.returncode,0)
            bad=copy.deepcopy(got['contexts'][0]['full_support']);bad['groups'][0]['ratios'][1]['weight']+=.01
            with self.assertRaises(AssertionError):v.numeric(bad,v.support(self.catalogue,got['contexts'][0]['base'],got['contexts'][0]['vectors']))

    def test_default_and_bad_preparation_zero_work(self):
        self.assertFalse(driver.initial('development',192)['training_admitted'])
        binary=v.ROOT/'target/release/ere-categorical-comparison'
        with tempfile.TemporaryDirectory() as directory:
            done=subprocess.run([str(binary),'run-prepared',directory,'0'*64],capture_output=True)
            self.assertNotEqual(done.returncode,0);self.assertEqual(list(Path(directory).iterdir()),[])
            with self.assertRaisesRegex(AssertionError,'fresh comparison output'):driver.prepare(Path(directory),'development')

    def test_bounded_weight_effect_fixture_and_tampers(self):
        # Synthetic arithmetic only. No new rollout and never a launchable archive.
        qs=[q for q in v.p.schedule(self.c,'development') if q['stage']=='validation' and q['context']==0];rows={}
        source=v.a.load(v.cat.INPUT/'cells'/(v.cat.source_ids()[0]+'.json'))
        for q in qs:
            row=copy.deepcopy(source);row.update(status='Complete',constraint_utility=1.,parameters=[0.]*183);row['evaluation']['score']=0.;rows[q['id']]=row
        vectors=[[0.]*183]*9;s=v.cat.selection(source['emission']);log=v.cat.score(self.catalogue,vectors[0],s)['log_probability']
        terms=[dict(draw=d,selection=s,evaluation_ids=[q['id'] for q in qs[4*d:4*d+4]],utility=1.,validity=1.,weights=[dict(base_log_probability=log,proposal_log_probability=log,weight=1.)]*9) for d in range(8)]
        rec=dict(block=0,context=0,terms=terms,utility_effects=[0.]*9,validity_effects=[0.]*9,observed_base_mean_utility=1.,base_native_mean=0.)
        self.assertEqual(v.validation(rec,0,0,qs,rows,self.catalogue,vectors,8),([0.]*9,[0.]*9))
        for mutate in (lambda x:x['terms'][0]['weights'][1].update(weight=1.1),lambda x:x['terms'][0].update(utility=2.),lambda x:x['utility_effects'].__setitem__(1,.1)):
            bad=copy.deepcopy(rec);mutate(bad)
            with self.assertRaises(AssertionError):v.validation(bad,0,0,qs,rows,self.catalogue,vectors,8)

if __name__=='__main__':unittest.main()
