import copy
import os
from pathlib import Path
import unittest
import verify_ere_graph_encoder_exact as v
from test_verify_ere_rule_relational import observation

ERRORS=(AssertionError,ValueError,KeyError,TypeError,IndexError)
def worlds():return [dict(context=c,observation=v.e.observe(observation(c)),rewards=[float((a*7+3)%17)/17 for a in range(16)]) for c in v.prior.DEVELOPMENT]


class ExactTests(unittest.TestCase):
    def test_design_fixed_before_measured_trajectory(self):
        self.assertEqual(v.a.load(Path('configs/ere_graph_encoder_exact_v1.json')),v.DESIGN)
        self.assertEqual(v.DESIGN['checkpoints'],[12,100]);self.assertEqual(v.DESIGN['maximum_native_attempts'],0)

    def test_forward_ad_objective_finite_difference(self):
        theta=v.e.initialize();w=worlds()
        for mode in v.prior.MODES:
            value,g=v.objective(mode,theta,w)
            self.assertTrue(0<value<1)
            for j in [0,33,69,105,108,109]:
                plus=theta.copy();minus=theta.copy();plus[j]+=1e-6;minus[j]-=1e-6
                diff=(v.objective(mode,plus,w)[0]-v.objective(mode,minus,w)[0])/2e-6
                self.assertLess(abs(diff-g[j]),2e-8)

    def test_zero_reward_identity_and_dense_denominator(self):
        w=worlds();theta=v.e.initialize()
        for row in w:row['rewards']=[0.]*16
        self.assertEqual(v.objective('graph',theta,w),(0.,[0.]*110))
        for row in w:row['rewards']=[1.]*16
        value,g=v.objective('graph',theta,w);self.assertAlmostEqual(value,1.,places=12)
        self.assertTrue(all(abs(x)<2e-12 for x in g))

    def test_reserved_order_action_reward_parameter_refusals(self):
        theta=v.e.initialize()
        for mutate in (lambda w:w.pop(),lambda w:w[0].update(context=v.prior.RESERVED[0]),lambda w:w.reverse(),
                       lambda w:w[0]['rewards'].pop(),lambda w:w[0]['rewards'].append(0.),lambda w:w[0]['rewards'].__setitem__(0,float('nan')),
                       lambda w:w[0]['rewards'].__setitem__(0,-0.1),lambda w:w[0]['rewards'].__setitem__(0,1.1)):
            w=worlds();mutate(w)
            with self.assertRaises(ERRORS):v.objective('graph',theta,w)
        for bad in (theta[:-1],[float('nan')]*110):
            with self.assertRaises(ERRORS):v.objective('graph',bad,worlds())
        with self.assertRaises(ERRORS):v.objective('invalid',theta,worlds())

    def test_synthetic_transition_gradient_update_mode_and_verdict_refusals(self):
        w=worlds();theta=v.e.initialize();value,g=v.objective('graph',theta,w)
        t=dict(iteration=0,mode='graph',before=theta,objective=value,gradient=g,after=[x+4*y for x,y in zip(theta,g)])
        v.verify_transition(t,0,'graph',theta,w)
        for mutate in (lambda r:r.update(iteration=1),lambda r:r.update(mode='blind'),lambda r:r.update(objective=0.),
                       lambda r:r['gradient'].__setitem__(0,1.),lambda r:r['after'].__setitem__(0,1.),lambda r:r.update(extra=True)):
            bad=copy.deepcopy(t);mutate(bad)
            with self.assertRaises(ERRORS):v.verify_transition(bad,0,'graph',theta,w)

    @unittest.skipUnless(os.environ.get('ERE_GRAPH_ENCODER_EXACT_AUDIT_INPUT'),'measured archive only')
    def test_measured_gradient_update_input_checkpoint_and_verdict_tampers(self):
        out=Path(os.environ['ERE_GRAPH_ENCODER_EXACT_AUDIT_INPUT']);data,p=v.import_matrix(out);self.assertEqual(data,v.a.load(out/'input.json'))
        w=v.development(data);t=v.a.load(out/'transitions.json')[0];v.verify_transition(t,0,'graph',v.e.initialize(),w)
        for mutate in (lambda r:r['gradient'].__setitem__(0,r['gradient'][0]+0.1),lambda r:r['after'].__setitem__(0,r['after'][0]+0.1)):
            bad=copy.deepcopy(t);mutate(bad)
            with self.assertRaises(ERRORS):v.verify_transition(bad,0,'graph',v.e.initialize(),w)
        bad=copy.deepcopy(w);bad[0]['observation']['adjacency']=[[False]*7 for _ in range(7)]
        with self.assertRaises(ERRORS):v.verify_transition(t,0,'graph',v.e.initialize(),bad)
        records=v.a.load(out/'checkpoints.json');states={r['checkpoint']:(r['final_graph']['theta'],r['final_blind']['theta']) for r in records}
        v.verify_checkpoints(records,states,data)
        for mutate in (lambda r:r[0].update(checkpoint=13),lambda r:r[0].update(diagnostic_criterion_pass=not r[0]['diagnostic_criterion_pass']),
                       lambda r:r[0]['final_graph'].update(balanced_expected_reward=1.),lambda r:r.pop()):
            bad=copy.deepcopy(records);mutate(bad)
            with self.assertRaises(ERRORS):v.verify_checkpoints(bad,states,data)


if __name__=='__main__':unittest.main()
