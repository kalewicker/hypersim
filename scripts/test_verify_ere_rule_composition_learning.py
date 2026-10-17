import copy
import os
from pathlib import Path
import unittest
import verify_ere_rule_composition_learning as v

ERRORS=(AssertionError,ValueError,KeyError,IndexError,TypeError)


def synthetic_zero_batches():
    transitions=[];rows=[]
    for si,seed in enumerate(v.POLICY_SEEDS):
        state=seed;theta=[0.]*12
        for iteration in range(12):
            first=si*402+48+iteration*24;drawn=[];batch=[]
            for c in v.DEVELOPMENT:
                for _ in range(2):
                    states=[];draws=[]
                    for _ in range(2):
                        state,draw=v.single.next_draw(state);states.append(state);draws.append(draw)
                    drawn.append(dict(draws=draws,states=states))
                    for name in ('live','reuse'):
                        p,actions,lp,g=v.policy(theta,c,draws);i=first+len(batch)
                        batch.append(dict(id=i,phase='training',policy_seed=seed,iteration=iteration,context=c,
                            policy=name,native_seed=v.native.SEEDS[(i//2)%2],draws=draws,draw_states=states,
                            policy_theta=theta,probabilities=p,actions=actions,log_probability=lp,score_gradient=g,reward=0.))
            rows.extend(batch)
            transitions.append(dict(policy_seed=seed,iteration=iteration,first_cell=first,last_cell=first+23,
                before_live=theta,before_reuse=theta,after_live=theta,after_reuse=theta,draws=drawn,skipped=False))
    return transitions,rows


class CompositionLearningTests(unittest.TestCase):
    def test_synthetic_zero_update_and_joint_arithmetic(self):
        transitions,rows=synthetic_zero_batches()
        policies=v.verify_transitions(transitions,rows)
        self.assertEqual(len(policies),3)
        for _,live,reuse in policies:self.assertEqual(live,reuse);self.assertEqual(live,[0.]*12)
        p,actions,lp,g=v.policy([0.]*12,[10,21],[0.3,0.7])
        self.assertEqual(actions,[1,2]);self.assertEqual(p,[[0.25]*4]*2)
        self.assertAlmostEqual(lp,__import__('math').log(1/16))
        self.assertEqual(g,[-0.25,0.75,-0.25,-0.25,-0.25,-0.25,0.75,-0.25,0.,0.,0.,0.])

    def test_twenty_detached_transition_tampers(self):
        transitions,rows=synthetic_zero_batches()
        mutations=[lambda t,r:t.pop(),lambda t,r:r.pop(),lambda t,r:t[0].update(skipped=True),
            lambda t,r:t[0].update(first_cell=49),lambda t,r:t[0].update(last_cell=72),
            lambda t,r:t[0].update(iteration=1),lambda t,r:t[0].update(policy_seed=0),
            lambda t,r:t[0]['before_live'].__setitem__(0,1.),lambda t,r:t[0]['after_reuse'].__setitem__(0,1.),
            lambda t,r:r[0].update(draws=[0.,0.]),lambda t,r:r[0].update(draw_states=[0,0]),
            lambda t,r:r[0].update(actions=[0,0]),lambda t,r:r[0].update(context=v.RESERVED[0]),
            lambda t,r:r[0].update(native_seed=v.native.SEEDS[1]),lambda t,r:r[0].update(policy='reuse'),
            lambda t,r:r[0].update(id=49),lambda t,r:r[0]['score_gradient'].__setitem__(0,1.),
            lambda t,r:r[0].update(log_probability=0.),lambda t,r:r[1].update(reward=0.375),
            lambda t,r:r[0].update(reward=float('nan'))]
        self.assertEqual(len(mutations),20)
        for i,mutate in enumerate(mutations):
            t=copy.deepcopy(transitions);r=copy.deepcopy(rows);mutate(t,r)
            with self.subTest(case=i),self.assertRaises(ERRORS):v.verify_transitions(t,r)

    def test_heldout_leakage_and_denominator_refusals(self):
        _,rows=synthetic_zero_batches();batch=rows[:24:2]
        self.assertEqual(v.update([0.]*12,batch),[0.]*12)
        for mutation in (lambda r:r.pop(),lambda r:r[0].update(context=v.RESERVED[0]),
                         lambda r:r[0].update(context=v.DEVELOPMENT[1]),lambda r:r[0].update(reward=-0.1)):
            r=copy.deepcopy(batch);mutation(r)
            with self.assertRaises(ERRORS):v.update([0.]*12,r)

    def test_reserved_endpoint_and_ten_detached_comparator_tampers(self):
        rewards=[0.]*48
        for i in (5,26,47):rewards[i]=0.375
        e=v.endpoint([0.]*12,rewards)
        self.assertEqual(e['mean_expected_reward'],9/384)
        with self.assertRaises(ERRORS):v.endpoint([0.]*12,rewards[:-1])
        observed=[];rows=[];policies=[]
        for seed in v.POLICY_SEEDS:
            policies.append((seed,[0.]*12,[0.]*12));controls=[]
            for phase in ('baseline','final'):
                rows.extend(dict(policy_seed=seed,phase=phase,reward=r) for r in rewards)
            for c in v.RESERVED:
                for arm in v.native.ARMS[1:]:
                    r=dict(context=c,arm=arm,reward=0.375 if arm=='reverse_paid' else 0.)
                    controls.append(r);rows.append(dict(r,policy_seed=seed,phase='control'))
            observed.append(dict(policy_seed=seed,baseline=e,final_live=e,final_reuse=e,final_frozen=e,
                expected_reward_gain=0.,gain_pass=False,every_reserved_joint_probability_gain=False,
                reuse_unchanged=True,frozen_unchanged=True,controls_pass=True,controls=controls))
        self.assertFalse(v.verify_comparisons(observed,policies,rows)[1])
        mutations=[lambda o:o.pop(),lambda o:o[0].update(gain_pass=True),
            lambda o:o[0].update(every_reserved_joint_probability_gain=True),lambda o:o[0].update(expected_reward_gain=0.01),
            lambda o:o[0].update(controls_pass=False),lambda o:o[0].update(frozen_unchanged=False),
            lambda o:o[0].update(reuse_unchanged=False),lambda o:o[0]['final_live']['contexts'][0].update(correct_joint_probability=1.),
            lambda o:o[0]['final_frozen'].update(mean_expected_reward=1.),lambda o:o[0]['controls'][0].update(reward=1.)]
        for i,mutate in enumerate(mutations):
            o=copy.deepcopy(observed);mutate(o)
            with self.subTest(case=i),self.assertRaises(ERRORS):v.verify_comparisons(o,policies,rows)

    def test_twelve_detached_archived_native_tampers(self):
        directory=Path('output/ere_rule_composition_development_20261004/native')
        row=v.a.load(directory/v.native.filename(v.native.SEEDS[0],v.RESERVED[0],'live_paid',[1,1]))
        self.assertEqual(v.native.audit_receipt(row)['reward'],0.375)
        mutations=[lambda r:r.update(adapter_reward=1.),lambda r:r['timeline'].pop(),
            lambda r:r.update(context=[10,21]),lambda r:r.update(actions=[1,2]),
            lambda r:r.update(completed_steps=23),lambda r:r.update(scientific_acceptance=True),
            lambda r:r['timeline'][1]['state'].update(fuel=2.),lambda r:r['timeline'][1].update(paid_edits=0),
            lambda r:r['timeline'][7]['usage'].update(truncated=True),lambda r:r.update(initial_fuel=3.),
            lambda r:r['summary'].update(novel_class_count=0),lambda r:r['timeline'][7].update(rng_after=[]) ]
        for i,mutate in enumerate(mutations):
            r=copy.deepcopy(row);mutate(r)
            with self.subTest(case=i),self.assertRaises(ERRORS):v.native.audit_receipt(r)

    @unittest.skipUnless(os.environ.get('ERE_RULE_COMPOSITION_LEARNING_AUDIT_INPUT'),'postlaunch receipts only')
    def test_full_accounting_without_requiring_a_positive_result(self):
        result=v.audit(Path(os.environ['ERE_RULE_COMPOSITION_LEARNING_AUDIT_INPUT']))
        self.assertTrue(result['accounting_pass']);self.assertEqual(result['native_steps_checked'],28944)
        self.assertEqual(result['completed_updates'],72)
        self.assertIsInstance(result['learning_criterion_pass'],bool)

    @unittest.skipUnless(os.environ.get('ERE_RULE_COMPOSITION_LEARNING_AUDIT_INPUT'),'postlaunch receipts only')
    def test_measured_transition_and_endpoint_tampers(self):
        directory=Path(os.environ['ERE_RULE_COMPOSITION_LEARNING_AUDIT_INPUT'])
        transitions=v.a.load(directory/'transitions.json');rows=[]
        for i,(_,phase,_,_,_,arm,_) in enumerate(v.schedule()):
            r=v.a.load(directory/f'cell-{i:04}.json');r['reward']=float(r.pop('native')['adapter_reward']);r['arm']=arm;rows.append(r)
        training=[r for r in rows if r['phase']=='training'];policies=v.verify_transitions(transitions,training)
        observed=v.a.load(directory/'comparisons.json');v.verify_comparisons(observed,policies,rows)
        bad=copy.deepcopy(transitions);bad[0]['after_live'][0]+=0.1
        with self.assertRaises(ERRORS):v.verify_transitions(bad,training)
        bad=copy.deepcopy(training);bad[0]['context']=v.RESERVED[0]
        with self.assertRaises(ERRORS):v.verify_transitions(transitions,bad)
        bad=copy.deepcopy(observed);bad[0]['gain_pass']=not bad[0]['gain_pass']
        with self.assertRaises(ERRORS):v.verify_comparisons(bad,policies,rows)
        bad=copy.deepcopy(observed);bad[0]['final_live']['mean_expected_reward']+=0.1
        with self.assertRaises(ERRORS):v.verify_comparisons(bad,policies,rows)


if __name__=='__main__':unittest.main()
