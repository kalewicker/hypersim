import copy
from pathlib import Path
import unittest
import tempfile
import shutil
import json
import audit_ere_complete_short_training as s

ROOT=Path(__file__).resolve().parents[1]
RUNTIME=ROOT/'output/ere_runtime50k_signal_panel_20260930'

class ShortTrainingTests(unittest.TestCase):
    def test_real_native_receipt_and_score_cap_credit_tampering(self):
        import json
        manifest=s.a.load(ROOT/'output/ere_complete_scaling_trial_v2_20260930/manifest.json')
        catalogue=s.a.load(ROOT/'output/ere_complete_scaling_trial_v2_20260930/catalogue.json')
        emission=s.a.load(ROOT/'output/ere_complete_scaling_trial_v2_20260930/emissions.json')[4]
        receipt=s.a.load(ROOT/'output/ere_complete_scaling_trial_v2_20260930/cells/p4-s1361-h100.json')
        row=dict(schedule=dict(id='development',emission_seed=emission['emission_seed'],physics_seed=1361,horizon=100),
            emission=emission,parameters=[0.]*183,status='Complete',elapsed_seconds=receipt['elapsed_seconds'],evaluation=receipt['evaluation'])
        h=s.a.helper(RUNTIME)
        self.assertEqual(s.native(row,catalogue,h),(100,100,1.))
        for field,value in [('score',0.),('credited_rule_ids',[]),('proposal_fires',99)]:
            bad=copy.deepcopy(row);bad['evaluation'][field]=value
            with self.assertRaises(AssertionError):s.native(bad,catalogue,h)
        bad=copy.deepcopy(row);bad['evaluation']['steps'][-1]['cap']=True
        with self.assertRaises(AssertionError):s.native(bad,catalogue,h)

    def test_update_reconstructs_historical_optimizer(self):
        h=s.a.helper(RUNTIME)
        for g in range(5):
            r=s.a.load(RUNTIME/f'generation_{g}.json')
            fits=[x for pair in r['pair_scores'] for x in pair]
            s.vec_close(s.update(r['theta_before'],r['eps'],fits,4,h),r['theta_after'])
        self.assertEqual(s.update([0.]*183,[[1.]*183], [None,None],1,h),[0.]*183)
        self.assertEqual(s.update([0.]*183,[[1.]*183], [0.,0.],1,h),[0.]*183)

    def test_fixed_schedule_and_source_bridge(self):
        plan=s.schedule();self.assertEqual(len(plan),1208)
        self.assertEqual(sum(q['horizon'] for q in plan),120800)
        self.assertEqual(s.a.load(ROOT/'configs/ere_complete_short_training_v1.json'),s.design())
        old=(RUNTIME/'source_snapshots'/s.SIGNAL).read_text()
        self.assertEqual(s.repaired_source(old),(ROOT/s.SIGNAL).read_text())
        with self.assertRaises(AssertionError):s.repaired_source(old.replace('let before = s.step();','let before = 0;'))

    def test_fixed_denominators_and_sign_test(self):
        draws=[dict(champion=1.,untrained=0.,random=.5) for _ in range(64)]
        result=s.held_statistics(draws)
        self.assertEqual(result['champion_minus_untrained'],1.)
        self.assertTrue(result['held_out_improvement_supported'])
        draws[0]['champion']=None;result=s.held_statistics(draws)
        self.assertIsNone(result['champion_minus_untrained']);self.assertIsNone(result['one_sided_paired_sign_p'])
        self.assertEqual(result['fixed_denominator'],64);self.assertEqual(result['complete_primary_pairs'],63)
        self.assertFalse(result['held_out_improvement_supported'])
        self.assertEqual(s.sign_p(0,0),1.);self.assertEqual(s.sign_p(6,0),1/64)

    def test_real_prefix_review_and_continuation_admission_without_native_work(self):
        prefix=ROOT/'output/ere_complete_short_training_20260930'
        review=s.review_prefix(prefix)
        self.assertTrue(review['postmeasurement_accounting_pass'])
        self.assertFalse(review['original_frozen_checker_pass'])
        self.assertFalse(review['physics_replayed'])
        # Development fixture copies recorded evidence; it performs no Engine work.
        with tempfile.TemporaryDirectory(prefix='ere-short-prefix-test-') as tmp:
            out=Path(tmp)/'development';shutil.copytree(prefix,out)
            m=s.a.load(out/'manifest.json');m.update(profile=s.CONTINUATION,design=s.wire_design(s.CONTINUATION),
                status='prechecks',reason=None,precheck_prefix=str(prefix),precheck_steps_reused=12000,new_precheck_native_steps=0)
            sources=m['snapshot']['sources']
            for name in set(sources)|s.EXTRA:
                if name.startswith('crates/hg-train/') or name in s.EXTRA:
                    p=out/'source_snapshots'/name;p.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(ROOT/name,p)
                    sources[name]=s.a.digest(p)
            (out/'prefix_accounting_review.json').write_text(json.dumps(review))
            (out/'manifest.json').write_text(json.dumps(m));s.a.seal(out)
            result=s.audit(out,verify_prerequisites=False)
            self.assertTrue(result['precheck_pass']);self.assertEqual(result['counts']['training']['attempted_steps'],0)
            m['design']['alpha']=.05
            (out/'manifest.json').write_text(json.dumps(m));s.a.seal(out)
            with self.assertRaises(AssertionError):s.audit(out,verify_prerequisites=False)

if __name__=='__main__':unittest.main()
