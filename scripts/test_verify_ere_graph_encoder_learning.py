import copy
import os
from pathlib import Path
import unittest
import verify_ere_graph_encoder_learning as v
from test_verify_ere_rule_relational import observation

ERRORS=(AssertionError,ValueError,KeyError,TypeError,IndexError)


def zero_training():
    rows=[];transitions=[]
    for si,seed in enumerate(v.POLICY_SEEDS):
        state=seed;theta=v.encoder.initialize()
        for iteration in range(12):
            first=si*594+144+iteration*24;batch=[];drawn=[]
            for c in v.DEVELOPMENT:
                for _ in range(2):
                    o=v.encoder.observe(observation(c));draws=[];states=[]
                    for _ in range(2):state,d=v.old.next_draw(state);draws.append(d);states.append(state)
                    drawn.append(dict(draws=draws,states=states))
                    for mode in v.MODES:
                        inp=v.encoder.make_input(mode,o);p,actions,lp,g=v.policy(theta,inp,draws);i=first+len(batch)
                        decision=dict(mode=mode,iteration=iteration,observation=o,input=inp,draws=draws,draw_states=states,
                            theta=theta,probabilities=p,log_probability=lp,score_gradient=g)
                        batch.append(dict(id=i,phase='training',policy_seed=seed,context=c,native_seed=v.native.SEEDS[(i//2)%2],actions=actions,
                            decision=decision,observed=o,arm='live_paid',reward=0.))
            rows.extend(batch);transitions.append(dict(policy_seed=seed,iteration=iteration,first_cell=first,last_cell=first+23,
                draws=drawn,before_graph=theta,before_blind=theta,after_graph=theta,after_blind=theta,skipped=False))
    return transitions,rows


def negative_comparisons():
    theta=v.encoder.initialize();policies=[(seed,theta,theta) for seed in v.POLICY_SEEDS];rows=[];comparisons=[]
    observations=[v.encoder.observe(observation(c)) for c in v.native.CONTEXTS];rewards=[]
    for c in v.native.CONTEXTS:
        correct=[v.native.CHOICES[s].index(c[s]) for s in range(2)];rewards.extend(0.375 if [x,y]==correct else 0. for x in range(4) for y in range(4))
    for i,seed in enumerate(v.POLICY_SEEDS):
        for phase in ('baseline','final'):rows.extend(dict(phase=phase,policy_seed=seed,reward=r,observed=observations[j//16]) for j,r in enumerate(rewards))
        controls=[]
        for c in v.RESERVED:
            for arm in v.native.ARMS[1:]:
                r=dict(context=c,arm=arm,reward=0.375 if arm=='reverse_paid' else 0.);controls.append(r);rows.append(dict(r,phase='control',policy_seed=seed))
        comparisons.append(v.comparison(seed,i,theta,theta,rewards,rewards,observations,controls))
    return comparisons,policies,rows


class LearningTests(unittest.TestCase):
    def test_explicit_incidental_metadata_pins_refuse_changes_and_other_files(self):
        for archive,entries in v.INCIDENTAL.items():
            v.verify_incidental(archive,entries)
            for bad in ({},{'.DS_Store':'0'*64},{**entries,'extra.json':'0'*64}):
                with self.assertRaises(ERRORS):v.verify_incidental(archive,bad)
        v.verify_incidental('unregistered',{})
        with self.assertRaises(ERRORS):v.verify_incidental('unregistered',{'.DS_Store':'0'*64})

    def test_zero_reward_trajectories_and_twenty_transition_tampers(self):
        transitions,rows=zero_training();policies=v.verify_transitions(transitions,rows)
        for _,graph,blind in policies:self.assertEqual(graph,v.encoder.initialize());self.assertEqual(blind,graph)
        mutations=[lambda t,r:t.pop(),lambda t,r:r.pop(),lambda t,r:t[0].update(skipped=True),
            lambda t,r:t[0].update(first_cell=145),lambda t,r:t[0].update(iteration=1),lambda t,r:t[0].update(policy_seed=0),
            lambda t,r:t[0]['after_graph'].__setitem__(0,1.),lambda t,r:r[0].update(context=v.RESERVED[0]),
            lambda t,r:r[0].update(native_seed=v.native.SEEDS[1]),lambda t,r:r[0].update(id=145),
            lambda t,r:r[0]['decision'].update(mode='blind'),lambda t,r:r[0]['decision'].update(draws=[0.,0.]),
            lambda t,r:r[0]['decision'].update(draw_states=[0,0]),lambda t,r:r[0].update(actions=[0,0]),
            lambda t,r:r[0]['decision']['score_gradient'].__setitem__(0,1.),lambda t,r:r[0]['decision'].update(log_probability=0.),
            lambda t,r:r[0]['decision']['theta'].__setitem__(0,1.),lambda t,r:r[0].update(reward=float('nan')),
            lambda t,r:r[1]['decision']['input']['observation']['adjacency'][0].__setitem__(1,True),
            lambda t,r:r[0]['decision']['probabilities'][0].__setitem__(0,1.)]
        for i,mutate in enumerate(mutations):
            t=copy.deepcopy(transitions);r=copy.deepcopy(rows);mutate(t,r)
            with self.subTest(case=i),self.assertRaises(ERRORS):v.verify_transitions(t,r)

    def test_negative_result_nonuniform_baseline_and_sixteen_criterion_tampers(self):
        comparisons,policies,rows=negative_comparisons();self.assertFalse(v.verify_comparisons(comparisons,policies,rows)[1])
        for c in comparisons:
            self.assertFalse(c['every_reserved_joint_probability_gain'])
            self.assertTrue(all(r['correct_joint_probability']>1/16 for r in c['baseline']['contexts'] if r['reserved']))
            self.assertTrue(c['positive_control_pass'])
        mutations=[lambda r:r.pop(),lambda r:r[0].update(ceiling_pass=True),lambda r:r[0].update(contrast_pass=True),
            lambda r:r[0].update(reserved_gain_pass=True),lambda r:r[0].update(every_reserved_joint_probability_gain=True),
            lambda r:r[0].update(blind_bound_pass=False),lambda r:r[0].update(frozen_unchanged=False),
            lambda r:r[0].update(uniform_unchanged=False),lambda r:r[0].update(native_matrix_unchanged=False),
            lambda r:r[0].update(positive_control_pass=False),lambda r:r[0].update(positive_control_unchanged=False),
            lambda r:r[0]['final_graph'].update(balanced_expected_reward=1.),lambda r:r[0]['baseline'].update(reserved_expected_reward=1.),
            lambda r:r[0]['final_frozen_blind']['theta'].__setitem__(0,1.),
            lambda r:r[0]['positive_control_final']['theta'].__setitem__(0,1.),lambda r:r[0]['controls'][0].update(reward=1.)]
        for i,mutate in enumerate(mutations):
            r=copy.deepcopy(comparisons);mutate(r)
            with self.subTest(case=i),self.assertRaises(ERRORS):v.verify_comparisons(r,policies,rows)
        bad=copy.deepcopy(rows);bad.pop()
        with self.assertRaises(ERRORS):v.verify_comparisons(comparisons,policies,bad)

    def test_update_mode_heldout_and_denominator_refusals(self):
        _,rows=zero_training();batch=rows[:24:2];theta=v.encoder.initialize();self.assertEqual(v.update('graph',theta,batch),theta)
        for mutate in (lambda r:r.pop(),lambda r:r[0].update(context=v.RESERVED[0]),lambda r:r[0].update(context=v.DEVELOPMENT[1]),
                       lambda r:r[0].update(reward=-0.1),lambda r:r[0]['decision'].update(mode='blind')):
            r=copy.deepcopy(batch);mutate(r)
            with self.assertRaises(ERRORS):v.update('graph',theta,r)

    def test_twelve_detached_native_and_raw_observation_tampers(self):
        root=Path('output/ere_rule_relational_development_20261004/native')
        row=v.a.load(root/v.native.filename(v.native.SEEDS[0],[10,11],'live_paid',[1,1]));row['raw_graph_observation']=v.encoder.observe(row['initial_inventory'])
        self.assertEqual(v.audit_native(row)[0]['reward'],0.375)
        mutations=[lambda r:r.update(adapter_reward=1.),lambda r:r['timeline'].pop(),lambda r:r.update(actions=[1,2]),
            lambda r:r.update(context=[10,21]),lambda r:r.update(seed=0),lambda r:r.update(scientific_acceptance=True),
            lambda r:r['raw_graph_observation']['adjacency'][0].__setitem__(0,True),
            lambda r:r['encoded_observation']['features'][0].__setitem__(0,0.),
            lambda r:r['timeline'][5].update(paid_edits=0),lambda r:r['timeline'][7]['usage'].update(truncated=True),
            lambda r:r['timeline'][1]['state'].update(fuel=2.),lambda r:r['summary'].update(novel_class_count=0)]
        for i,mutate in enumerate(mutations):
            r=copy.deepcopy(row);mutate(r)
            with self.subTest(case=i),self.assertRaises(ERRORS):v.audit_native(r)

    @unittest.skipUnless(os.environ.get('ERE_GRAPH_ENCODER_LEARNING_AUDIT_INPUT'),'postlaunch receipts only')
    def test_complete_accounting_without_positive_learning_requirement(self):
        result=v.audit(Path(os.environ['ERE_GRAPH_ENCODER_LEARNING_AUDIT_INPUT']));self.assertTrue(result['accounting_pass'])
        self.assertEqual(result['native_steps_checked'],42768);self.assertEqual(result['completed_updates'],72);self.assertIsInstance(result['learning_criterion_pass'],bool)

    @unittest.skipUnless(os.environ.get('ERE_GRAPH_ENCODER_LEARNING_AUDIT_INPUT'),'postlaunch receipts only')
    def test_four_measured_transition_mask_endpoint_corruptions(self):
        root=Path(os.environ['ERE_GRAPH_ENCODER_LEARNING_AUDIT_INPUT']);rows=[]
        for i,(_,phase,_,_,_,arm,_) in enumerate(v.schedule()):
            r=v.a.load(root/f'cell-{i:04}.json');n=r.pop('native');r.update(observed=n['raw_graph_observation'],reward=float(n['adapter_reward']),arm=arm);rows.append(r)
        training=[r for r in rows if r['phase']=='training'];t=v.a.load(root/'transitions.json');policies=v.verify_transitions(t,training)
        comparisons=v.a.load(root/'comparisons.json');v.verify_comparisons(comparisons,policies,rows)
        bad=copy.deepcopy(t);bad[0]['after_graph'][0]+=0.1
        with self.assertRaises(ERRORS):v.verify_transitions(bad,training)
        bad=copy.deepcopy(training);bad[1]['decision']['input']['observation']['adjacency'][0][1]=True
        with self.assertRaises(ERRORS):v.verify_transitions(t,bad)
        bad=copy.deepcopy(comparisons);bad[0]['ceiling_pass']=not bad[0]['ceiling_pass']
        with self.assertRaises(ERRORS):v.verify_comparisons(bad,policies,rows)
        bad=copy.deepcopy(comparisons);bad[0]['final_graph']['balanced_expected_reward']+=0.1
        with self.assertRaises(ERRORS):v.verify_comparisons(bad,policies,rows)


if __name__=='__main__':unittest.main()
