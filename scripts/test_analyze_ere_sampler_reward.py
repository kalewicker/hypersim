from pathlib import Path
import unittest
import analyze_ere_sampler_reward as m

ROOT=Path(__file__).resolve().parents[1]
INP=ROOT/'output/ere_complete_direction_assay_20261001'

class MappingTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.cat=m.d.a.load(INP/'catalogue.json');cls.leaves,cls.groups,cls.counts=m.catalogue_model(cls.cat)

    def test_exact_catalogue_support_and_classes(self):
        self.assertEqual(self.counts['leaves'],18956);self.assertEqual(self.counts['active_leaves'],9478);self.assertEqual(self.counts['eligible_leaves'],9282)
        for name in ['validate-r4-c1-d8-forward_a-p0','validate-r4-c1-d8-base-p0','estimate-r0-c0-e0-p4-s1-b2']:
            row=m.d.a.load(INP/'cells'/(name+'.json'));e=row['emission'];key=(e['parent'],e['operator'],0 if e['role']=='Core' else 1,e['production_index'])
            self.assertEqual(self.leaves[key]['active'],e['classes']['active']);self.assertEqual(self.leaves[key]['eligible'],bool(row['evaluation']['credited_rule_ids']))

    def test_priority_is_not_structural_novelty_and_malformed_packets_refuse(self):
        a=b'E1RCF2'+(0).to_bytes(4,'little',signed=True)+b'body';b=b'E1RCF2'+(1).to_bytes(4,'little',signed=True)+b'body'
        def pack(xs):return (b'E2-PROGRAM\0'+len(xs).to_bytes(4,'little')+b''.join(len(x).to_bytes(4,'little')+x for x in xs)).hex()
        self.assertFalse(m.eligible(pack([a,b])));self.assertTrue(m.eligible(pack([a,b+b'x'])))
        with self.assertRaises(AssertionError):m.eligible(pack([a,b])[:-2])

    def test_softmax_normalization_mask_and_gauge(self):
        prob=m.softmax([0.,1.,100.],[0,1]);shift=m.softmax([5.,6.,105.],[0,1]);self.assertEqual(prob,shift);self.assertNotIn(2,prob);self.assertAlmostEqual(sum(prob.values()),1)
        for theta in [[0.]*183,[.25 if i%2==0 else -.25 for i in range(183)]]:
            p=m.probabilities(self.cat,theta,self.groups);self.assertAlmostEqual(sum(p['group_mass'].values()),1);self.assertAlmostEqual(sum(m.execution_mass(p,self.groups).values()),1)
            self.assertGreaterEqual(p['active_mass'],p['eligible_mass'])
        self.assertAlmostEqual(m.probabilities(self.cat,[0.]*183,self.groups)['parent_active_mass'],.5)

    def test_analytic_gradients_match_central_differences(self):
        # Central differences subtract two sums over 3,232 legal groups.
        # h=1e-4 balances O(h^2) truncation and O(epsilon/h) cancellation;
        # a step-size check gives <5e-11 maximum error on these coordinates.
        theta=[.25 if i%2==0 else -.25 for i in range(183)];p=m.probabilities(self.cat,theta,self.groups);h=1e-4
        for feature in ('active','eligible'):
            g=m.expectation_gradient(p,self.groups,feature)
            for i in (0,1,57,167,168,174,178,180,181,182):
                plus=theta[:];minus=theta[:];plus[i]+=h;minus[i]-=h
                fd=(m.probabilities(self.cat,plus,self.groups)[feature+'_mass']-m.probabilities(self.cat,minus,self.groups)[feature+'_mass'])/(2*h)
                self.assertAlmostEqual(g[i],fd,delta=2e-10)
            for _,start,end in m.GROUPS:self.assertAlmostEqual(sum(g[start:end]),0.,delta=1e-12)

    def test_cdf_churn_can_exceed_distribution_tv_without_wrong_marginals(self):
        self.assertAlmostEqual(m.parent_cdf_switch([.2,.3,.5],[.2,.3,.5]),0)
        self.assertAlmostEqual(m.distance([.5,.5,0.],[0.,.5,.5]),.5)
        self.assertAlmostEqual(m.parent_cdf_switch([.5,.5,0.],[0.,.5,.5]),1.)
        for r in range(8):
            vs=m.d.a.load(INP/'directions'/f'r{r}-c1.json')['parameters'];a=m.softmax(vs[0][:168],list(range(168)));b=m.softmax(vs[1][:168],list(range(168)))
            self.assertGreaterEqual(m.parent_cdf_switch(list(a.values()),list(b.values()))+1e-12,m.distance(list(a.values()),list(b.values())))

    def test_leaf_and_executable_tv_consistent_and_full_zero_step(self):
        theta=[0.]*183;z=m.probability_comparison(self.cat,self.groups,theta,theta)
        for name in ('parent_tv','leaf_selection_tv','executable_signature_tv','active_mass_delta','eligible_mass_delta'):self.assertEqual(z[name],0.)
        vs=m.d.a.load(INP/'directions/r4-c1.json')['parameters'];s=m.probability_comparison(self.cat,self.groups,vs[0],vs[1])
        self.assertLessEqual(s['executable_signature_tv'],s['leaf_selection_tv']+1e-12)
        self.assertAlmostEqual(sum(s['group_step_energy_fractions'].values()),1.)

    def test_no_full_native_expectation_from_activation_or_coverage(self):
        # Both examples remain credit-eligible, yet their native reward differs.
        rows=[m.d.a.load(INP/'cells'/(q['id']+'.json')) for q in m.d.schedule(m.d.design())[:64]]
        eligible=[r for r in rows if r['evaluation']['credited_rule_ids'] and r['evaluation']['score'] is not None]
        self.assertTrue(any(r['evaluation']['score']==0 for r in eligible));self.assertTrue(any(r['evaluation']['score']>0 for r in eligible))

if __name__=='__main__':unittest.main()
