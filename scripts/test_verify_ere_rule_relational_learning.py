import copy
import math
import os
from pathlib import Path
import unittest
import verify_ere_rule_relational_learning as v
from test_verify_ere_rule_relational import observation

ERRORS=(AssertionError,ValueError,KeyError,TypeError,IndexError)


def synthetic_zero_training():
    rows=[];transitions=[]
    for si,seed in enumerate(v.POLICY_SEEDS):
        state=seed;theta=[0.]*16
        for iteration in range(12):
            first=si*594+144+iteration*24;batch=[];drawn=[]
            for c in v.DEVELOPMENT:
                for _ in range(2):
                    observed=v.native.observe(observation(c));draws=[];states=[]
                    for _ in range(2):state,draw=v.next_draw(state);draws.append(draw);states.append(state)
                    drawn.append(dict(draws=draws,states=states))
                    for mode in v.MODES:
                        inp=v.make_input(mode,observed);p,actions,lp,g=v.policy(theta,inp,draws);i=first+len(batch)
                        d=dict(mode=mode,iteration=iteration,observation=observed,input=inp,draws=draws,draw_states=states,
                            theta=theta,probabilities=p,log_probability=lp,score_gradient=g)
                        batch.append(dict(id=i,phase='training',policy_seed=seed,context=c,native_seed=v.native.SEEDS[(i//2)%2],actions=actions,
                            decision=d,observed=observed,arm='live_paid',reward=0.))
            rows.extend(batch);transitions.append(dict(policy_seed=seed,iteration=iteration,first_cell=first,last_cell=first+23,
                draws=drawn,before_graph=theta,before_blind=theta,after_graph=theta,after_blind=theta,skipped=False))
    return transitions,rows


def synthetic_zero_comparisons():
    policies=[(seed,[0.]*16,[0.]*16) for seed in v.POLICY_SEEDS];rows=[];comparisons=[]
    observations=[v.native.observe(observation(c)) for c in v.native.CONTEXTS]
    rewards=[]
    for c in v.native.CONTEXTS:
        correct=[v.native.CHOICES[s].index(c[s]) for s in range(2)]
        rewards.extend(0.375 if [a,b]==correct else 0. for a in range(4) for b in range(4))
    for seed,graph,blind in policies:
        for phase in ('baseline','final'):
            rows.extend(dict(phase=phase,policy_seed=seed,reward=r,observed=observations[i//16]) for i,r in enumerate(rewards))
        controls=[]
        for c in v.RESERVED:
            for arm in v.native.ARMS[1:]:
                row=dict(context=c,arm=arm,reward=0.375 if arm=='reverse_paid' else 0.)
                controls.append(row);rows.append(dict(row,phase='control',policy_seed=seed))
        before=v.endpoint('graph',graph,rewards,observations);blind_end=v.endpoint('blind',blind,rewards,observations)
        value=before['balanced_expected_reward'];bvalue=blind_end['balanced_expected_reward']
        comparisons.append(dict(policy_seed=seed,baseline=before,final_graph=before,final_blind=blind_end,final_frozen=before,
            ceiling_margin=value-1/24,graph_minus_blind=value-bvalue,reserved_gain=0.,ceiling_pass=False,contrast_pass=False,
            blind_bound_pass=True,reserved_gain_pass=False,every_reserved_joint_probability_gain=False,
            frozen_unchanged=True,native_matrix_unchanged=True,controls_pass=True,controls=controls))
    return comparisons,policies,rows


class RelationalLearningTests(unittest.TestCase):
    def test_capacity_matched_mask_gradients_and_finite_differences(self):
        theta=[0.2,-0.3,0.1,-0.4,0.7,-0.1,0.2,0.8,0.4,-0.2,-0.3,0.9,0.1,0.2,-0.4,0.6]
        blind=None
        for c in v.native.CONTEXTS:
            o=v.native.observe(observation(c))
            for mode in v.MODES:
                inp=v.make_input(mode,o)
                for a in range(4):
                    for b in range(4):
                        p,lp,g=v.statistics(theta,inp,[a,b])
                        if mode=='graph':
                            old=v.native.policy(theta,o,[a,b]);v.close(p,old[0]);v.close(lp,old[1]);v.close(g,old[2])
                        elif blind is None:blind=p
                        else:self.assertEqual(blind,p)
                        for i in range(16):
                            hi=theta.copy();lo=theta.copy();hi[i]+=1e-6;lo[i]-=1e-6
                            fd=(v.statistics(hi,inp,[a,b])[1]-v.statistics(lo,inp,[a,b])[1])/2e-6
                            self.assertLess(abs(fd-g[i]),3e-9)
                        draws=[math.fsum(p[s][:action])+p[s][action]/2 for s,action in enumerate([a,b])]
                        self.assertEqual(v.policy(theta,inp,draws)[1],[a,b])

    def test_zero_batches_and_twenty_four_detached_transition_tampers(self):
        transitions,rows=synthetic_zero_training();policies=v.verify_transitions(transitions,rows)
        for _,graph,blind in policies:self.assertEqual(graph,[0.]*16);self.assertEqual(blind,[0.]*16)
        mutations=[lambda t,r:t.pop(),lambda t,r:r.pop(),lambda t,r:t[0].update(skipped=True),
            lambda t,r:t[0].update(first_cell=145),lambda t,r:t[0].update(last_cell=168),
            lambda t,r:t[0].update(iteration=1),lambda t,r:t[0].update(policy_seed=0),
            lambda t,r:t[0]['before_graph'].__setitem__(0,1.),lambda t,r:t[0]['after_blind'].__setitem__(0,1.),
            lambda t,r:r[0]['decision'].update(draws=[0.,0.]),lambda t,r:r[0]['decision'].update(draw_states=[0,0]),
            lambda t,r:r[0].update(actions=[0,0]),lambda t,r:r[0].update(context=v.RESERVED[0]),
            lambda t,r:r[0].update(native_seed=v.native.SEEDS[1]),lambda t,r:r[0].update(id=145),
            lambda t,r:r[0]['decision'].update(mode='blind'),lambda t,r:r[1]['decision']['input'].update(features=[[1.,0.,0.]]*2),
            lambda t,r:r[0]['decision']['score_gradient'].__setitem__(0,1.),lambda t,r:r[0]['decision'].update(log_probability=0.),
            lambda t,r:r[0]['decision']['theta'].__setitem__(0,1.),lambda t,r:r[0].update(reward=float('nan')),
            lambda t,r:r[0]['observed']['features'][0].__setitem__(0,0.),lambda t,r:r[0]['decision'].update(iteration=1),
            lambda t,r:r[0]['decision']['probabilities'][0].__setitem__(0,1.)]
        for i,mutate in enumerate(mutations):
            t=copy.deepcopy(transitions);r=copy.deepcopy(rows);mutate(t,r)
            with self.subTest(case=i),self.assertRaises(ERRORS):v.verify_transitions(t,r)

    def test_update_leakage_mode_and_denominator_refusals(self):
        _,rows=synthetic_zero_training();batch=rows[:24:2]
        self.assertEqual(v.update('graph',[0.]*16,batch),[0.]*16)
        for mutate in (lambda r:r.pop(),lambda r:r[0].update(context=v.RESERVED[0]),
                       lambda r:r[0].update(context=v.DEVELOPMENT[1]),lambda r:r[0].update(reward=-0.1),
                       lambda r:r[0]['decision'].update(mode='blind')):
            r=copy.deepcopy(batch);mutate(r)
            with self.assertRaises(ERRORS):v.update('graph',[0.]*16,r)

    def test_full_endpoint_denominator_and_twelve_detached_criterion_tampers(self):
        comparisons,policies,rows=synthetic_zero_comparisons()
        self.assertFalse(v.verify_comparisons(comparisons,policies,rows)[1])
        self.assertEqual(comparisons[0]['baseline']['balanced_expected_reward'],9/384)
        mutations=[lambda r:r.pop(),lambda r:r[0].update(ceiling_pass=True),lambda r:r[0].update(contrast_pass=True),
            lambda r:r[0].update(reserved_gain_pass=True),lambda r:r[0].update(every_reserved_joint_probability_gain=True),
            lambda r:r[0].update(blind_bound_pass=False),lambda r:r[0].update(native_matrix_unchanged=False),
            lambda r:r[0].update(frozen_unchanged=False),lambda r:r[0].update(ceiling_margin=0.01),
            lambda r:r[0]['final_graph'].update(balanced_expected_reward=1.),lambda r:r[0]['final_blind'].update(balanced_expected_reward=1.),
            lambda r:r[0]['controls'][0].update(reward=1.)]
        for i,mutate in enumerate(mutations):
            r=copy.deepcopy(comparisons);mutate(r)
            with self.subTest(case=i),self.assertRaises(ERRORS):v.verify_comparisons(r,policies,rows)
        bad=copy.deepcopy(rows);bad.pop()
        with self.assertRaises(ERRORS):v.verify_comparisons(comparisons,policies,bad)

    def test_twelve_detached_native_tampers(self):
        p=Path('output/ere_rule_relational_development_20261004/native')
        row=v.a.load(p/v.native.filename(v.native.SEEDS[0],[10,11],'live_paid',[1,1]))
        self.assertEqual(v.native.audit_receipt(row)['reward'],0.375)
        mutations=[lambda r:r.update(adapter_reward=1.),lambda r:r['timeline'].pop(),lambda r:r.update(actions=[1,2]),
            lambda r:r.update(context=[10,21]),lambda r:r.update(seed=0),lambda r:r.update(scientific_acceptance=True),
            lambda r:r['encoded_observation']['features'][0].__setitem__(0,0.),lambda r:r['observation_nodes'].reverse(),
            lambda r:r['timeline'][5].update(paid_edits=0),lambda r:r['timeline'][7]['usage'].update(truncated=True),
            lambda r:r['timeline'][1]['state'].update(fuel=2.),lambda r:r['summary'].update(novel_class_count=0)]
        for i,mutate in enumerate(mutations):
            r=copy.deepcopy(row);mutate(r)
            with self.subTest(case=i),self.assertRaises(ERRORS):v.native.audit_receipt(r)

    @unittest.skipUnless(os.environ.get('ERE_RULE_RELATIONAL_LEARNING_AUDIT_INPUT'),'postlaunch receipts only')
    def test_full_accounting_without_positive_outcome_requirement(self):
        r=v.audit(Path(os.environ['ERE_RULE_RELATIONAL_LEARNING_AUDIT_INPUT']))
        self.assertTrue(r['accounting_pass']);self.assertEqual(r['native_steps_checked'],42768)
        self.assertEqual(r['pre_emission_decisions_checked'],864);self.assertEqual(r['completed_updates'],72)
        self.assertIsInstance(r['learning_criterion_pass'],bool)

    @unittest.skipUnless(os.environ.get('ERE_RULE_RELATIONAL_LEARNING_AUDIT_INPUT'),'postlaunch receipts only')
    def test_measured_transition_mask_and_endpoint_tampers(self):
        root=Path(os.environ['ERE_RULE_RELATIONAL_LEARNING_AUDIT_INPUT']);rows=[]
        for i,(_,phase,_,_,_,arm,_) in enumerate(v.schedule()):
            r=v.a.load(root/f'cell-{i:04}.json');n=r.pop('native');r.update(observed=n['encoded_observation'],reward=float(n['adapter_reward']),arm=arm);rows.append(r)
        training=[r for r in rows if r['phase']=='training'];transitions=v.a.load(root/'transitions.json')
        policies=v.verify_transitions(transitions,training);comparisons=v.a.load(root/'comparisons.json');v.verify_comparisons(comparisons,policies,rows)
        bad=copy.deepcopy(transitions);bad[0]['after_graph'][0]+=0.1
        with self.assertRaises(ERRORS):v.verify_transitions(bad,training)
        bad=copy.deepcopy(training);bad[1]['decision']['input']['features']=[[1.,0.,0.]]*2
        with self.assertRaises(ERRORS):v.verify_transitions(transitions,bad)
        bad=copy.deepcopy(comparisons);bad[0]['ceiling_pass']=not bad[0]['ceiling_pass']
        with self.assertRaises(ERRORS):v.verify_comparisons(bad,policies,rows)
        bad=copy.deepcopy(comparisons);bad[0]['final_graph']['balanced_expected_reward']+=0.1
        with self.assertRaises(ERRORS):v.verify_comparisons(bad,policies,rows)


if __name__=='__main__':unittest.main()
