import copy
import unittest
from pathlib import Path
import audit_ere_complete_feasibility_training as f

ROOT=Path(__file__).resolve().parents[1]
PARENT=ROOT/'output/ere_complete_short_training_continuation_20260930'

class FeasibilityTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.native,cls.h=f.c.helpers(PARENT)

    def test_design_and_fresh_fixed_schedule(self):
        self.assertEqual(f.a.load(ROOT/'configs/ere_complete_feasibility_training_v1.json'),f.design())
        q=f.schedule();self.assertEqual(len(q),6464);self.assertEqual(sum(x['horizon'] for x in q),646400)
        self.assertEqual(len({x['id'] for x in q}),6464)
        for i in range(0,320,2):self.assertEqual(q[i]['emission_seed'],q[i+1]['emission_seed']);self.assertEqual(q[i]['physics_seed'],q[i+1]['physics_seed'])
        old=f.a.load(PARENT/'schedule.json') if (PARENT/'schedule.json').exists() else f.a.load(PARENT/'manifest.json')['schedule']
        self.assertFalse({x['physics_seed'] for x in q}&{x['physics_seed'] for x in old})

    def test_caps_ties_and_all_feasible_ranks(self):
        scores=[0.]*16;scores[0]=None;fits=f.optimizer_fits(scores,self.h)
        self.assertLess(fits[0],fits[1]);scores[1]=None;self.assertEqual(*f.optimizer_fits(scores,self.h)[:2])
        self.assertEqual(f.optimizer_fits([None]*16,self.h),[0.]*16)
        scores=[0.,0.,0.1,0.1,0.2,0.4,0.5,0.7]*2
        self.assertEqual(self.h.ranks(scores),self.h.ranks(f.optimizer_fits(scores,self.h)))
        self.assertEqual(f.utility(None,'resource_cap'),0.);self.assertEqual(f.utility(0.,None),1.)
        with self.assertRaises(AssertionError):f.utility(None,'invalid_family')

    def test_null_native_and_known_utility_statistics(self):
        s=f.statistics([[None,0.,0.] for _ in range(512)],[[0.,1.,1.] for _ in range(512)],[0,2048,2048],True)
        self.assertIsNone(s['champion_minus_untrained']);self.assertIsNone(s['one_sided_paired_sign_p'])
        self.assertEqual(s['constraint_utility']['complete_pairs'],512);self.assertEqual(s['constraint_utility']['losses'],512)
        self.assertFalse(s['constraint_utility']['improvement_supported'])
        s=f.statistics([[None,None,None]]*512,[[None,None,None]]*512,[0,0,0],False)
        self.assertIsNone(s['constraint_utility']['one_sided_paired_sign_p'])
        s=f.statistics([[1.,0.,0.]]*512,[[2.,1.,1.]]*512,[2048]*3,True)
        self.assertTrue(s['constraint_utility']['improvement_supported']);self.assertEqual(s['constraint_utility']['one_sided_paired_sign_p'],2.**-512)
        s['constraint_utility']['one_sided_paired_sign_p']=0.
        with self.assertRaises(AssertionError):f.check_statistics(s,f.statistics([[1.,0.,0.]]*512,[[2.,1.,1.]]*512,[2048]*3,True))

    def test_prior_real_es_update_and_native_receipt_tampering(self):
        rec=f.a.load(PARENT/'generations/train-0.json');fits=[x for p in rec['pair_scores'] for x in p]
        self.native.vec_close(rec['theta_after'],self.native.update(rec['theta_before'],rec['eps'],fits,8,self.h))
        mapped=f.optimizer_fits(fits,self.h)
        self.native.vec_close(rec['theta_after'],self.native.update(rec['theta_before'],rec['eps'],mapped,8,self.h))
        row=f.a.load(PARENT/'cells/hold-0-champion-2411.json');row['constraint_utility']=f.utility(row['evaluation']['score'],row['evaluation']['reason'])
        catalogue=f.a.load(PARENT/'catalogue.json');q=copy.deepcopy(row['schedule']);f.cell(row,q,catalogue,self.native,self.h)
        row['constraint_utility']+=.1
        with self.assertRaises(AssertionError):f.cell(row,q,catalogue,self.native,self.h)

    def test_real_delayed_cap_is_null_and_not_fabricated_fitness(self):
        row=f.a.load(ROOT/'output/ere_complete_champion_confirmation_20261001/cells/d33-champion-p0.json')
        self.assertEqual(row['status'],'Null');self.assertIsNone(row['evaluation']['score'])
        row['constraint_utility']=f.utility(row['evaluation']['score'],row['evaluation']['reason'])
        self.assertEqual(row['constraint_utility'],0.)
        q=copy.deepcopy(row['schedule']);catalogue=f.a.load(PARENT/'catalogue.json')
        self.assertEqual(f.cell(row,q,catalogue,self.native,self.h)[:2],(81,80))
        row['evaluation']['steps'][-1]['rule_ids']=[0]
        with self.assertRaises(AssertionError):f.cell(row,q,catalogue,self.native,self.h)

    def test_feasibility_count_guard_not_only_utility_gain(self):
        s=f.statistics([[None,0.,0.]]*512,[[1.9,1.,1.]]*512,[2044,2048,2048],True)
        self.assertGreater(s['constraint_utility']['champion_minus_untrained'],0.)
        self.assertLess(s['constraint_utility']['one_sided_paired_sign_p'],.05)
        self.assertFalse(s['constraint_utility']['improvement_supported'])

    def test_new_generation_reconstruction_and_tampering_without_physics(self):
        # Detached development fixture: old real noise/scores, new bookkeeping.
        rec=f.a.load(PARENT/'generations/train-0.json');rows={}
        for q,oldid in zip(f.schedule()[:16],rec['evaluation_ids']):
            rows[q['id']]=f.a.load(PARENT/'cells'/(oldid+'.json'))
        rec['evaluation_ids']=[q['id'] for q in f.schedule()[:16]]
        rec['native_scores']=[rows[q['id']]['evaluation']['score'] for q in f.schedule()[:16]]
        fits=f.optimizer_fits(rec['native_scores'],self.h);rec['optimizer_pair_scores']=[fits[i:i+2] for i in range(0,16,2)]
        initial=f.a.load(PARENT/'initial_distribution.json')
        for panel in (initial,rec['distribution']):
            for i,item in enumerate(panel):
                item['seed']=f.SEED+1000+i;item['selection']['emission_seed']=item['seed']
        catalogue=f.a.load(PARENT/'catalogue.json')
        f.generation(rec,0,[0.]*183,rows,self.native,self.h,catalogue,initial)
        for key in ('theta_after','optimizer_pair_scores'):
            bad=copy.deepcopy(rec)
            if key=='theta_after':bad[key][0]+=.1
            else:bad[key][0][0]+=1
            with self.assertRaises(AssertionError):f.generation(bad,0,[0.]*183,rows,self.native,self.h,catalogue,initial)

if __name__=='__main__':unittest.main()
