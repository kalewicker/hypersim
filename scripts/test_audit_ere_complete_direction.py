import copy
from pathlib import Path
import unittest
import audit_ere_complete_direction as d

ROOT=Path(__file__).resolve().parents[1]
PARENT=ROOT/'output/ere_complete_short_training_continuation_20260930'

class DirectionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.native,cls.h=d.c.helpers(PARENT)

    def test_exact_design_schedule_and_disjoint_panels(self):
        self.assertEqual(d.a.load(ROOT/'configs/ere_complete_direction_v1.json'),dict(scientific=d.design(),development=d.design(True)))
        for dev in (False,True):
            design=d.design(dev);q=d.schedule(design);self.assertEqual(len(q)*100,design['maximum_native_steps']);self.assertEqual(len({v['id'] for v in q}),len(q))
            all_seeds=set()
            for r in range(design['blocks']):
                for ctx in range(2):
                    for e in range(2):
                        rows=[v for v in q if v['stage']=='estimation' and v['block']==r and v['context']==ctx and v['estimate']==e]
                        seeds={v['emission_seed'] for v in rows};self.assertEqual(len(seeds),8*design['batch_size']);self.assertFalse(all_seeds&seeds);all_seeds|=seeds
                        for p in range(8):
                            plus=[(v['emission_seed'],v['physics_seed']) for v in rows if v['pair']==p and v['sign']==1]
                            minus=[(v['emission_seed'],v['physics_seed']) for v in rows if v['pair']==p and v['sign']==-1];self.assertEqual(plus,minus)
            validation={v['emission_seed'] for v in q if v['stage']=='validation'};self.assertFalse(all_seeds&validation)
            self.assertEqual(len(validation),design['blocks']*2*design['validation_draws'])
            bad=copy.deepcopy(q);bad[0]['batch_draw']=False
            with self.assertRaises(AssertionError):d.b.exact_schedule(bad,q)

    def test_fixed_context_zero_alignment_and_exact_reverse(self):
        t=d.contexts();self.assertEqual(t[1][:4],[.25,-.25,.25,-.25]);self.assertEqual(len(t[1]),183)
        z=d.alignment(t[0],t[0],t[0]);self.assertTrue(z['degenerate']);self.assertEqual(z['score'],0.)
        x=[1.]*183;y=[-1.]*183;self.assertEqual(d.alignment(t[0],x,y)['score'],-1.);self.assertEqual(d.alignment(t[0],x,x)['score'],1.)
        arms=d.arm_parameters(t[1],[.1]*183,[-.3]*183,self.h)
        self.assertEqual(arms[2][0],self.h.f32(.5-self.h.f32(.1)));self.assertEqual(arms[2][1],self.h.f32(-.5-self.h.f32(.1)))

    def test_eight_blocks_not_contexts_or_cells_and_conjunction(self):
        design=d.design();align=[[dict(score=1.,degenerate=False,norm_a=1.,norm_b=1.)]*2 for _ in range(8)]
        u=[[[1.,1.2,.8,1.2,.8] for _ in range(2)] for _ in range(8)];counts=[[[64]*5 for _ in range(2)] for _ in range(8)]
        s=d.statistics(design,align,u,u,counts,True);self.assertEqual(s['fixed_denominator'],8);self.assertEqual(s['reliability']['sign_p'],1/256);self.assertEqual(s['usefulness']['sign_p'],1/256);self.assertTrue(s['support_direction'])
        align[-1]=[dict(score=-1.)]*2;s=d.statistics(design,align,u,u,counts,True);self.assertEqual(s['reliability']['sign_p'],9/256)
        align[0]=[dict(score=-1.)]*2;s=d.statistics(design,align,u,u,counts,True);self.assertFalse(s['support_direction'])
        align=[[dict(score=1.)]*2 for _ in range(8)];u[0][0][1]=None;s=d.statistics(design,align,u,u,counts,False)
        self.assertIsNone(s['usefulness']['mean']);self.assertIsNone(s['reliability']['sign_p']);self.assertEqual(s['fixed_denominator'],8)

    def test_development_caps_and_context_guard(self):
        design=d.design(True);align=[[dict(score=1.)]*2];u=[[[1.,1.2,.8,1.2,.8]]*2];counts=[[[8]*5]*2]
        s=d.statistics(design,align,u,u,counts,True);self.assertFalse(s['support_direction'])
        design=d.design();align=[[dict(score=1.)]*2 for _ in range(8)];u=[[[1.,1.2,.8,1.2,.8] for _ in range(2)] for _ in range(8)];counts=[[[64]*5 for _ in range(2)] for _ in range(8)]
        counts[0][0][1]-=1;s=d.statistics(design,align,u,u,counts,True);self.assertFalse(s['support_direction'])
        counts[0][0][1]+=1
        for r in u:r[1]=[1.,.99,.8,.99,.8]
        s=d.statistics(design,align,u,u,counts,True);self.assertGreater(s['usefulness']['mean'],0);self.assertFalse(s['support_direction'])
        native=copy.deepcopy(u);native[0][0][2]=None;s=d.statistics(design,align,u,native,counts,True);self.assertFalse(s['all_validation_native_valid']);self.assertEqual(s['native_arm_means'],[None]*5)

    def test_real_native_cap_and_partial_estimate(self):
        row=d.a.load(ROOT/'output/ere_complete_champion_confirmation_20261001/cells/d33-champion-p0.json');row['constraint_utility']=0.
        d.b.cell(row,row['schedule'],d.a.load(PARENT/'catalogue.json'),self.native,self.h)
        qs=d.schedule(d.design(True))[:32];rows={q['id']:dict(status='NotRun') for q in qs}
        rec=dict(block=0,context=0,estimate=0,theta_before=[0.]*183,eps=[[0.]*183]*8,evaluation_ids=[q['id'] for q in qs],update_applied=False,candidate_means=None,optimizer_pair_scores=None,theta_after=None)
        self.assertIsNone(d.estimate(rec,qs,d.contexts()[0],rows,self.native,self.h,2));rec['update_applied']=True
        with self.assertRaises(AssertionError):d.estimate(rec,qs,d.contexts()[0],rows,self.native,self.h,2)
        means,_=d.b.batch_fits([[None,0.]]*16,2,self.h);self.assertEqual(means,[.5]*16)

    def test_saved_native_noise_and_rank_update_fixture(self):
        old=d.a.load(PARENT/'generations/train-0.json');qs=d.schedule(d.design(True))[:32];rows={}
        for i,q in enumerate(qs):
            row=copy.deepcopy(d.a.load(PARENT/'cells'/(old['evaluation_ids'][i//2]+'.json')))
            row['constraint_utility']=d.f.utility(row['evaluation']['score'],row['evaluation']['reason']);rows[q['id']]=row
        means,fits=d.b.batch_fits([[rows[qs[2*i]['id']]['evaluation']['score']]*2 for i in range(16)],2,self.h)
        rec=dict(block=0,context=0,estimate=0,theta_before=[0.]*183,eps=old['eps'],evaluation_ids=[q['id'] for q in qs],update_applied=True,candidate_means=means,optimizer_pair_scores=[fits[i:i+2] for i in range(0,16,2)],theta_after=old['theta_after'])
        self.assertEqual(d.estimate(rec,qs,d.contexts()[0],rows,self.native,self.h,2),old['theta_after'])
        second=copy.deepcopy(rec);second['estimate']=1;d.shared_basis(rec,second);second['eps'][0][0]+=.1
        with self.assertRaises(AssertionError):d.shared_basis(rec,second)
        first=old['theta_after'];second=old['theta_after'];params=d.arm_parameters(d.contexts()[0],first,second,self.h)
        rec=dict(block=0,context=0,selection='both_fixed_estimates_no_validation_selection',alignment=d.alignment(d.contexts()[0],first,second),parameters=params)
        d.direction(rec,0,0,d.contexts()[0],first,second,self.h);rec['parameters'][2][0]+=.01
        with self.assertRaises(AssertionError):d.direction(rec,0,0,d.contexts()[0],first,second,self.h)

    def test_all_zero_directions_retained_and_no_unsupported_imputation(self):
        design=d.design();align=[[dict(score=0.,degenerate=True,norm_a=0.,norm_b=0.)]*2 for _ in range(8)];u=[[[1.]*5]*2 for _ in range(8)];counts=[[[64]*5]*2 for _ in range(8)]
        s=d.statistics(design,align,u,u,counts,True);self.assertEqual(s['reliability']['ties'],8);self.assertEqual(s['reliability']['mean'],0.);self.assertEqual(s['reliability']['sign_p'],1.);self.assertEqual(s['fixed_denominator'],8);self.assertFalse(s['support_direction'])
        with self.assertRaises(AssertionError):d.f.utility(None,'unsupported')

if __name__=='__main__':unittest.main()
