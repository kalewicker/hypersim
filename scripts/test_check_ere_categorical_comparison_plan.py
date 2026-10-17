import copy
import math
from pathlib import Path
import tempfile
import unittest
import check_ere_categorical_comparison_plan as p
import analyze_ere_sampler_reward as mapping

class PlanTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.config=p.a.load(p.CONFIG)

    def panels(self,mode='scientific'):
        n=self.config[mode]['blocks'];align=[[[.5,1.] for c in range(2)] for r in range(n)]
        effects=[[[[.01,.01],[.02,.02]] for c in range(2)] for r in range(n)]
        validity=[[[[0.,0.],[0.,0.]] for c in range(2)] for r in range(n)]
        return align,effects,validity

    def test_fixed_budgets_unique_domains_and_layout_separation(self):
        science=p.check_schedule(self.config,'scientific');dev=p.check_schedule(self.config,'development')
        self.assertEqual(len(science['rows']),12288);self.assertEqual(len(dev['rows']),192);self.assertFalse(science['seeds']&dev['seeds'])
        self.assertEqual(science['counts'],dict(noise=32,est_emission=3072,est_physics=3072,val_emission=2048,val_physics=8192))
        self.assertEqual(dev['counts'],dict(noise=4,est_emission=96,est_physics=96,val_emission=16,val_physics=64))
        self.assertEqual(sum(r['horizon'] for r in science['rows']),1228800)
        self.assertTrue(all(r['sampled_arm']=='base' for r in science['rows'] if r['stage']=='validation'))

    def test_independent_estimates_and_equal_evaluation_not_equal_seed_count(self):
        rows=p.schedule(self.config,'scientific');sets={}
        for method in ('es','categorical'):
            for e in (0,1):
                panel=[r for r in rows if r['stage']=='estimation' and (r['block'],r['context'],r['method'],r['estimate'])==(0,0,method,e)]
                self.assertEqual(len(panel),64);sets[(method,e)]={r['emission_seed'] for r in panel}
                self.assertEqual(len(sets[(method,e)]),32 if method=='es' else 64)
        self.assertTrue(all(not sets[x]&sets[y] for x in sets for y in sets if x!=y))
        noise=[{r['noise_seed'] for r in rows if r['stage']=='estimation' and (r['block'],r['context'],r['method'],r['estimate'])==(0,0,'es',e)} for e in (0,1)]
        self.assertFalse(noise[0]&noise[1])

    def test_registered_gauge_step_f32_reverse_and_degeneracy(self):
        for t in p.contexts():
            raw=[p.f32(x+.01*math.sin(i)) for i,x in enumerate(t)];m=p.matched_step(t,raw)
            self.assertFalse(m['degenerate']);self.assertAlmostEqual(math.sqrt(sum((x-y)**2 for x,y in zip(m['forward'],t))),.1,delta=5e-8)
            self.assertEqual(m['reverse'],[p.f32(p.f32(2*b)-f) for b,f in zip(t,m['forward'])])
            for start,end in ((0,168),(168,181),(181,183)):self.assertAlmostEqual(sum(m['centered_delta'][start:end]),0.,delta=1e-12)
            raw=[p.f32(x+(1. if i<168 else 2. if i<181 else 3.)) for i,x in enumerate(t)]
            m=p.matched_step(t,raw);self.assertTrue(m['degenerate']);self.assertEqual(m['forward'],t);self.assertEqual(m['reverse'],t)

    def test_importance_control_variate_preserves_utility_and_cap_effect(self):
        for u,v in [([2.,1.],[1.,1.]),([0.,2.],[0.,1.])]:
            got=p.importance_effect([1.2,.8],u,v)
            exact=.6*u[0]+.4*u[1]-(.5*u[0]+.5*u[1])
            self.assertAlmostEqual(got['utility_effect'],exact)
            self.assertAlmostEqual(got['validity_effect'],.6*v[0]+.4*v[1]-(.5*v[0]+.5*v[1]))
        self.assertEqual(p.importance_effect([1.,1.],[0.,2.],[0.,1.]),dict(utility_effect=0.,validity_effect=0.))
        with self.assertRaises(AssertionError):p.importance_effect([2.],[1.],[1.])
        with self.assertRaises(AssertionError):p.importance_effect([1.],[math.nan],[1.])

    def test_full_catalogue_weight_normalization_and_derived_bounds(self):
        cat=p.a.load(p.ROOT/'output/ere_complete_direction_assay_20261001/catalogue.json');_,groups,counts=mapping.catalogue_model(cat)
        self.assertEqual(counts['leaves'],18956)
        for t in p.contexts():
            m=p.matched_step(t,[p.f32(x+.02*math.cos(i)) for i,x in enumerate(t)])
            base=mapping.probabilities(cat,t,groups)['group_mass']
            for vector in (m['forward'],m['reverse']):
                post=mapping.probabilities(cat,vector,groups)['group_mass'];weights={k:post[k]/base[k] for k in groups}
                self.assertGreaterEqual(min(weights.values()),math.exp(-6*.100001))
                self.assertLessEqual(max(weights.values()),math.exp(6*.100001))
                self.assertAlmostEqual(math.fsum(base[k]*weights[k] for k in groups),1.,delta=1e-12)
                self.assertTrue(all(base[k]>0 for k in groups))

    def test_three_property_conjunction_and_eight_block_sign_test(self):
        align,effects,v=self.panels();got=p.statistics(self.config,'scientific',align,effects,v,True)
        self.assertTrue(got['support_categorical_direction']);self.assertEqual(got['fixed_denominator'],8)
        self.assertEqual(got['confirmatory']['categorical_minus_es']['sign_p'],1/256)
        effects[-1]=[[[.01,.01],[-.02,-.02]] for c in range(2)]
        got=p.statistics(self.config,'scientific',align,effects,v,True)
        self.assertEqual(got['confirmatory']['categorical_minus_es']['sign_p'],9/256);self.assertTrue(got['support_categorical_direction'])
        effects[-2]=copy.deepcopy(effects[-1]);self.assertFalse(p.statistics(self.config,'scientific',align,effects,v,True)['support_categorical_direction'])
        align,effects,v=self.panels();align[0]=[[.5,-1.],[.5,-1.]];align[1]=copy.deepcopy(align[0])
        self.assertFalse(p.statistics(self.config,'scientific',align,effects,v,True)['support_categorical_direction'])
        align,effects,v=self.panels();v[0][0][1]=[-.01,-.01]
        self.assertFalse(p.statistics(self.config,'scientific',align,effects,v,True)['support_categorical_direction'])

    def test_unknowns_deadband_context_guard_and_development_do_not_promote(self):
        align,effects,v=self.panels();effects[0][0][1][0]=None
        got=p.statistics(self.config,'scientific',align,effects,v,True);self.assertFalse(got['support_categorical_direction'])
        self.assertEqual(got['block_contrasts'],[None]*8)
        self.assertTrue(all(x['mean'] is None and x['sign_p'] is None for x in got['confirmatory'].values()))
        align,effects,v=self.panels();self.assertFalse(p.statistics(self.config,'scientific',align,effects,v,False)['all_outcomes_known'])
        for r in effects:r[1]=[[.01,.01],[0.,0.]]
        self.assertFalse(p.statistics(self.config,'scientific',align,effects,v,True)['support_categorical_direction'])
        align,effects,v=self.panels()
        for r in align:
            for c in r:c[1]=5e-13
        got=p.statistics(self.config,'scientific',align,effects,v,True)
        self.assertEqual(got['confirmatory']['categorical_alignment']['ties'],8);self.assertEqual(got['confirmatory']['categorical_alignment']['sign_p'],1.)
        self.assertFalse(got['support_categorical_direction'])
        self.assertFalse(p.statistics(self.config,'development',*self.panels('development'),True)['support_categorical_direction'])

    def test_strict_plan_and_exact_registration_fail_closed(self):
        receipt=p.check_registration(self.config)
        self.assertEqual(receipt['unchanged_live_entries'],452)
        self.assertEqual(len(receipt['excluded_feature_observations']),5)
        self.assertEqual(receipt['adapter_live_entries_matching'],462)
        for path,value in [(('resources','full_training_authorized'),0),(('native','max_pairings'),True),
                           (('scientific','blocks'),7),(('development','blocks'),True),(('weight_clipping',),True),
                           (('raw_alpha',),.1),(('step_radius',),.2),(('sign_deadband',),1e-5),
                           (('excluded_feature_sources',),['crates/hg-engine/src/engine.rs'])]:
            c=copy.deepcopy(self.config);target=c
            for k in path[:-1]:target=target[k]
            target[path[-1]]=value
            with self.assertRaises(AssertionError):p.validate(c)
        c=copy.deepcopy(self.config);c['adapter_registration']['crates/hg-train/Cargo.toml']['registered_sha256']='0'*64
        with self.assertRaises(AssertionError):p.check_registration(c)
        with tempfile.TemporaryDirectory() as directory:
            with self.assertRaisesRegex(AssertionError,'fresh preregistration output required'):p.freeze(Path(directory))
            self.assertEqual(list(Path(directory).iterdir()),[])

if __name__=='__main__':unittest.main()
