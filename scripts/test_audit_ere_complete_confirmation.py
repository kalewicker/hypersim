import copy
import unittest
from pathlib import Path
import audit_ere_complete_confirmation as c
ROOT=Path(__file__).resolve().parents[1]
PARENT=ROOT/'output/ere_complete_short_training_continuation_20260930'

class ConfirmationTests(unittest.TestCase):
    def test_schedule_config_and_wire_design(self):
        self.assertEqual(c.a.load(ROOT/'configs/ere_complete_champion_confirmation_v1.json'),c.design())
        self.assertEqual(len(c.schedule()),6144);self.assertEqual(sum(q['horizon'] for q in c.schedule()),614400)
        old=c.a.load(PARENT/'manifest.json')['design']['master_seed'];self.assertNotEqual(old,c.SEED)

    def test_null_denominators_ties_and_extreme_binomial_tail(self):
        draws=[[1.,0.,0.] for _ in range(512)];s=c.statistics(draws)
        self.assertTrue(s['improvement_supported']);self.assertEqual(s['one_sided_paired_sign_p'],2.**-512)
        draws[0][0]=None;s=c.statistics(draws);self.assertIsNone(s['champion_minus_untrained']);self.assertEqual(s['fixed_denominator'],512)
        self.assertEqual(s['complete_primary_pairs'],511);self.assertFalse(s['improvement_supported'])
        s=c.statistics([[0.,0.,0.] for _ in range(512)]);self.assertEqual(s['one_sided_paired_sign_p'],1.)

    def test_pinned_real_native_helper_and_detached_tampering(self):
        c.a.verify(PARENT,expected=c.PARENT,count=1690)
        native,h=c.helpers(PARENT);catalogue=c.a.load(PARENT/'catalogue.json');theta=c.a.load(PARENT/'champion.json')['parameters']
        row=c.a.load(PARENT/'cells/hold-0-champion-2411.json');q=row['schedule'];q={**q,'arm':0}
        row['schedule']=q
        self.assertEqual(c.cell(row,q,theta,catalogue,native,h)[:2],(100,100))
        bad=copy.deepcopy(row);bad['parameters'][0]+=1
        with self.assertRaises(AssertionError):c.cell(bad,q,theta,catalogue,native,h)
        bad=copy.deepcopy(row);bad['evaluation']['proposal_fires']+=1
        with self.assertRaises(AssertionError):c.cell(bad,q,theta,catalogue,native,h)

    def test_manifest_and_statistics_tampering(self):
        m=dict(profile=c.PROFILE,design=c.design(),scheduled_cells=6144,optimizer_generations=0,recorded_cells=0,
            inflight_native_work_unknown=False,attempted_steps=0,advancing_steps=0)
        c.check_manifest(m);m['design']['horizon']=1000
        with self.assertRaises(AssertionError):c.check_manifest(m)
        expected=c.statistics([[1.,0.,0.] for _ in range(512)]);s=copy.deepcopy(expected);c.check_statistics(s,expected,True)
        s['arm_means']['champion']+=1e-15;c.check_statistics(s,expected,True)
        s['one_sided_paired_sign_p']=0.
        with self.assertRaises(AssertionError):c.check_statistics(s,expected,True)

if __name__=='__main__':unittest.main()
