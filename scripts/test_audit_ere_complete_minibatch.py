import copy
from pathlib import Path
import unittest
import audit_ere_complete_minibatch as b

ROOT=Path(__file__).resolve().parents[1]
PARENT=ROOT/'output/ere_complete_short_training_continuation_20260930'

class MinibatchTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.native,cls.h=b.c.helpers(PARENT)

    def test_exact_design_pools_and_seed_independence(self):
        self.assertEqual(b.a.load(ROOT/'configs/ere_complete_minibatch_v1.json'),dict(scientific=b.design(),development=b.design(True)))
        for dev in (False,True):
            d=b.design(dev);q=b.schedule(d);self.assertEqual(len(q)*100,d['maximum_native_steps']);self.assertEqual(len({r['id'] for r in q}),len(q))
            allpools=[]
            for rep in range(d['replicates']):
                pools=[sorted((r['emission_seed'],r['physics_seed']) for r in q if r['stage']=='training' and r['replicate']==rep and r['arm']==a) for a in range(2)]
                self.assertEqual(*pools);self.assertEqual(len(pools[0]),d['single_generations']*16)
                for old in allpools:self.assertFalse(set(old)&set(pools[0]))
                allpools.append(pools[0])
            bad=copy.deepcopy(q);bad[0]['batch_draw']=False
            with self.assertRaises(AssertionError):b.exact_schedule(bad,q)

    def test_batch_one_compatibility_and_all_caps_ties(self):
        scores=[None,0.,0.,.2,.4,.7,.5,.8]*2
        old=b.f.optimizer_fits(scores,self.h);_,new=b.batch_fits([[x] for x in scores],1,self.h)
        self.assertEqual(self.h.ranks(old),self.h.ranks(new))
        means,fits=b.batch_fits([[None]*4]*16,4,self.h);self.assertEqual(means,[0.]*16);self.assertEqual(self.h.ranks(fits),[0.]*16)
        with self.assertRaises(AssertionError):b.batch_fits([[None]]*16,4,self.h)

    def test_cap_denominator_and_mean_not_strict_batch_feasibility(self):
        scores=[[0.]*4 for _ in range(16)];scores[0]=[None,0.,0.,0.]
        means,fits=b.batch_fits(scores,4,self.h);self.assertEqual(means[0],.75);self.assertLess(fits[0],fits[1])
        scores[0]=[None,1.,1.,1.];means,fits=b.batch_fits(scores,4,self.h)
        self.assertEqual(means[0],1.5);self.assertGreater(fits[0],fits[1])

    def test_eight_replicates_not_thousands_of_cells(self):
        d=b.design();u=[[[1.,2.]]*64 for _ in range(8)];n=[[[0.,1.]]*64 for _ in range(8)]
        s=b.statistics(d,u,n,[1.]*64,[[256,256]]*8,True)
        self.assertEqual(s['fixed_denominator'],8);self.assertEqual(s['utility_sign_p'],1/256);self.assertTrue(s['support_batching'])
        u[-1]=[[2.,1.]]*64;s=b.statistics(d,u,n,[1.]*64,[[256,256]]*8,True);self.assertEqual(s['utility_sign_p'],9/256)
        u[0][0]=[1.,None];s=b.statistics(d,u,n,[1.]*64,[[256,256]]*8,False)
        self.assertIsNone(s['batch_minus_single_utility']);self.assertIsNone(s['utility_sign_p']);self.assertEqual(s['fixed_denominator'],8)

    def test_development_never_supports_science_and_feasibility_guard(self):
        d=b.design(True);s=b.statistics(d,[[[1.,2.]]*2],[[[0.,1.]]*2],[1.]*2,[[8,8]],True)
        self.assertFalse(s['support_batching']);self.assertEqual(s['mode'],'development')
        d=b.design();s=b.statistics(d,[[[1.,2.]]*64]*8,[[[0.,1.]]*64]*8,[1.]*64,[[256,252]]*8,True)
        self.assertFalse(s['support_batching'])

    def test_real_native_cap_and_detached_generation_update(self):
        row=b.a.load(ROOT/'output/ere_complete_champion_confirmation_20261001/cells/d33-champion-p0.json')
        row['constraint_utility']=0.;self.assertIsNone(row['evaluation']['score'])
        b.cell(row,row['schedule'],b.a.load(PARENT/'catalogue.json'),self.native,self.h)
        old=b.a.load(PARENT/'generations/train-0.json');d=b.design(True);qs=b.schedule(d)[:16];rows={}
        for q,oldid in zip(qs,old['evaluation_ids']):rows[q['id']]=b.a.load(PARENT/'cells'/(oldid+'.json'))
        for r in rows.values():r['constraint_utility']=b.f.utility(r['evaluation']['score'],r['evaluation']['reason'])
        means,fits=b.batch_fits([[rows[q['id']]['evaluation']['score']] for q in qs],1,self.h)
        rec=dict(replicate=0,arm=0,generation=0,theta_before=[0.]*183,eps=old['eps'],evaluation_ids=[q['id'] for q in qs],
            update_applied=True,candidate_means=means,optimizer_pair_scores=[fits[i:i+2] for i in range(0,16,2)],theta_after=old['theta_after'])
        b.generation(rec,qs,[0.]*183,rows,self.native,self.h,1)
        bad=copy.deepcopy(rec);bad['theta_after'][0]+=.1
        with self.assertRaises(AssertionError):b.generation(bad,qs,[0.]*183,rows,self.native,self.h,1)

    def test_partial_generation_start_is_auditable_not_an_update(self):
        d=b.design(True);qs=[q for q in b.schedule(d) if q['stage']=='training' and q['arm']==1]
        rows={q['id']:dict(status='NotRun') for q in qs}
        rec=dict(replicate=0,arm=1,generation=0,theta_before=[0.]*183,eps=[[0.]*183]*8,evaluation_ids=[q['id'] for q in qs],
            update_applied=False,candidate_means=None,optimizer_pair_scores=None,theta_after=None)
        self.assertEqual(b.generation(rec,qs,[0.]*183,rows,self.native,self.h,2),([0.]*183,False))
        rec['update_applied']=True
        with self.assertRaises(AssertionError):b.generation(rec,qs,[0.]*183,rows,self.native,self.h,2)

if __name__=='__main__':unittest.main()
