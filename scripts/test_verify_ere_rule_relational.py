import copy
import math
import os
from pathlib import Path
import unittest
import verify_ere_rule_relational as v

ERRORS=(AssertionError,ValueError,KeyError,TypeError,IndexError)


def observation(context):
    ids={label:dict(idx=i*2+1,version=1) for i,label in enumerate(v.ORDINARY_LABELS)}
    nodes=[[v.node_key(n),dict(data=dict(label=label),incident=[])] for label,n in ids.items()]
    pairs=[[ids[context[0]],ids[context[1]]],[ids[context[1]],ids[12]]]
    return dict(step=0,nodes=nodes,edges=[[f'EdgeId({i*2+1}v1)',dict(label=0,nodes=[dict(n) for n in pair])] for i,pair in enumerate(pairs)])


def detached_archived_receipt():
    # A synthetic ledger regression using saved old physics plus four isolated
    # lures. It is not a native relational outcome and is never used as one.
    root=Path('output/ere_rule_composition_development_20261004/native')
    r=v.integrity.load(root/v.filename(993748716001,[10,11],'live_paid',[1,1]))
    r['profile']=v.PROFILE;r['seed']=v.SEEDS[0]
    world=r['ordinary_nodes'];ordinary={v.node_key(n):n for n in world}
    all_ids={10:world[0],11:world[1],12:world[2]}
    for i,label in enumerate((20,30,21,31)):
        n=dict(idx=10001+i*2,version=1);all_ids[label]=n
        for name in ('initial_inventory','final_inventory'):
            inventory=r[name]
            data=copy.deepcopy(next(value['data'] for key,value in inventory['nodes'] if key==v.node_key(world[0])))
            data['label']=label
            inventory['nodes'].append([v.node_key(n),dict(data=data,incident=[])])
            inventory['nodes'].sort(key=lambda x:x[0])
    r['observation_nodes']=[all_ids[label] for label in v.ORDINARY_LABELS]
    r['encoded_observation']=v.observe(r['initial_inventory'])
    return r


def synthetic_policy_worlds():
    theta=[0.2,-0.3,0.1,-0.4,0.7,-0.1,0.2,0.8,0.4,-0.2,-0.3,0.9,0.1,0.2,-0.4,0.6]
    category=[0.2,-0.3,0.8,-1.1,1.,0.,-0.5,0.7,-0.2,0.3,0.1,0.6]
    probe=[0.]*16
    for k in range(3):probe[4+(k+1)*3+k]=4.
    blind=[v.blind_probabilities(category,10)]*2
    worlds=[];data=[]
    for seed in v.SEEDS:
        for c in v.CONTEXTS:
            o=v.observe(observation(c));p=v.policy(theta,o);g=v.policy(probe,o);u=v.policy([0.]*16,o)
            correct=[v.CHOICES[i].index(c[i]) for i in range(2)]
            rewards=[0.375 if [a,b]==correct else 0. for a in range(4) for b in range(4)]
            def expected(p):return math.fsum(p[0][a]*p[1][b]*rewards[a*4+b] for a in range(4) for b in range(4))
            actions=[]
            for a in range(4):
                for b in range(4):
                    _,lp,gradient=v.policy(theta,o,[a,b]);actions.append(dict(actions=[a,b],sample_midpoints=[a,b],log_probability=lp,score_gradient=gradient))
            worlds.append(dict(seed=seed,context=c,observation=o,probabilities=p,actions=actions,
                graph_probe_probabilities=g,uniform_probabilities=u,category_only_probabilities=blind,
                graph_probe_expected_reward=expected(g),uniform_expected_reward=expected(u),category_only_expected_reward=expected(blind)))
            data.append((seed,c,o,rewards))
    receipt=dict(theta=theta,probe_theta=probe,category_only_theta=category,category_only_cue=[10,11],worlds=worlds,
        graph_probe_mean_reward=math.fsum(r['graph_probe_expected_reward']/18 for r in worlds),
        category_only_mean_reward=math.fsum(r['category_only_expected_reward']/18 for r in worlds),
        category_only_upper_bound=1/24,training_updates=0,training_admitted=False,scientific_acceptance=False)
    return receipt,data


class RelationalTests(unittest.TestCase):
    def test_observation_invariance_and_relation_contrast(self):
        features=[]
        for c in v.CONTEXTS:
            raw=observation(c);expected=v.observe(raw);features.append(expected['features'])
            reordered=copy.deepcopy(raw);reordered['nodes'].reverse();reordered['edges'].reverse()
            for _,edge in reordered['edges']:edge['nodes'].reverse()
            self.assertEqual(v.observe(reordered),expected)
            remapped=copy.deepcopy(raw)
            for n in remapped['nodes']:
                old=n[0];idx=int(old[7:old.index('v')])+500;n[0]=f'NodeId({idx}v9)'
            for _,edge in remapped['edges']:
                for n in edge['nodes']:n['idx']+=500;n['version']=9
            self.assertEqual(v.observe(remapped),expected)
            metadata=copy.deepcopy(raw);metadata['nodes'].append(['NodeId(999v1)',dict(data=dict(label=v.META+9,answer=c))])
            self.assertEqual(v.observe(metadata),expected)
        self.assertEqual(len({str(f) for f in features}),9)

    def test_ten_malformed_observation_refusals(self):
        raw=observation([10,11])
        mutations=[lambda r:r.update(step=1),lambda r:r['nodes'].pop(),lambda r:r['edges'].pop(),
            lambda r:r['nodes'].append(copy.deepcopy(r['nodes'][0])),lambda r:r['nodes'][0][1]['data'].update(label=99),
            lambda r:r['edges'][0][1].update(label=1),lambda r:r['edges'][0][1]['nodes'].reverse() or r['edges'][0][1]['nodes'].__setitem__(1,copy.deepcopy(r['edges'][0][1]['nodes'][0])),
            lambda r:r['edges'].append(copy.deepcopy(r['edges'][0])),lambda r:r['edges'][0][1]['nodes'][0].update(idx=0),
            lambda r:r['edges'][1][1]['nodes'].__setitem__(0,copy.deepcopy(r['edges'][0][1]['nodes'][0]))]
        for i,mutate in enumerate(mutations):
            r=copy.deepcopy(raw);mutate(r)
            with self.subTest(case=i),self.assertRaises(ERRORS):v.observe(r)

    def test_all_joint_gradient_finite_differences_and_blind_bound(self):
        theta=[0.2,-0.3,0.1,-0.4,0.7,-0.1,0.2,0.8,0.4,-0.2,-0.3,0.9,0.1,0.2,-0.4,0.6]
        for c in v.CONTEXTS:
            o=v.observe(observation(c))
            for a in range(4):
                for b in range(4):
                    _,_,g=v.policy(theta,o,[a,b])
                    for i in range(16):
                        hi=theta.copy();lo=theta.copy();hi[i]+=1e-6;lo[i]-=1e-6
                        fd=(v.policy(hi,o,[a,b])[1]-v.policy(lo,o,[a,b])[1])/2e-6
                        self.assertLess(abs(fd-g[i]),3e-9)
        # Every extreme point of an arbitrary graph-blind joint distribution
        # has reward <=1/24; mixtures inherit this bound by linearity.
        for a in range(4):
            for b in range(4):
                value=math.fsum((0.375/9 if [a,b]==[v.CHOICES[s].index(c[s]) for s in range(2)] else 0.) for c in v.CONTEXTS)
                self.assertLessEqual(value,1/24)

    def test_detached_old_ledger_with_new_lure_scope_and_twenty_four_tampers(self):
        row=detached_archived_receipt();self.assertEqual(v.audit_receipt(row)['reward'],0.375)
        def lure_mutation(r):
            for key,val in r['final_inventory']['nodes']:
                if key=='NodeId(10001v1)':val['data']['energy']=1.
        mutations=[lambda r:r.update(profile='bad'),lambda r:r.update(seed=0),lambda r:r.update(context=[10,21]),
            lambda r:r.update(actions=[1,2]),lambda r:r.update(adapter_reward=1.),lambda r:r['timeline'].pop(),
            lambda r:r.update(training_admitted=True),lambda r:r.update(scientific_acceptance=True),
            lambda r:r['target_literals'].reverse(),lambda r:r['observation_nodes'].pop(),
            lambda r:r['encoded_observation']['features'][0].__setitem__(0,0.),lure_mutation,
            lambda r:r['timeline'][1]['state'].update(fuel=2.),lambda r:r['timeline'][5].update(paid_edits=0),
            lambda r:r['timeline'][7]['usage'].update(truncated=True),lambda r:r['timeline'][7].update(rng_after=-1),
            lambda r:r['timeline'][6].update(installed=copy.deepcopy(r['timeline'][2]['installed'])),
            lambda r:r['timeline'][7]['events'][0].update(rcf='bad'),lambda r:r['summary'].update(novel_class_count=0),
            lambda r:r['seeded_editor_compiled'][2]['pattern']['edges'][0].update(ordered=False),
            lambda r:r['seeded_editor_rules'][2]['lhs_node_property_guards'][1].update(value=1),
            lambda r:r['initial_inventory']['edges'][0][1].update(label=0),
            lambda r:r['timeline'][7]['events'][0]['bound_nodes'].__setitem__(0,dict(idx=10001,version=1)),
            lambda r:r['timeline'][1]['genotype'][1].update(structural_rcf='bad')]
        self.assertEqual(len(mutations),24)
        for i,mutate in enumerate(mutations):
            r=copy.deepcopy(row);mutate(r)
            with self.subTest(case=i),self.assertRaises(ERRORS):v.audit_receipt(r)

    def test_synthetic_policy_and_eight_tampers(self):
        receipt,data=synthetic_policy_worlds();result=v.audit_policy(receipt,data)
        self.assertGreater(result['graph_probe_mean_reward'],0.30)
        mutations=[lambda r:r.update(training_updates=1),lambda r:r.update(category_only_cue=[20,21]),
            lambda r:r['worlds'][0]['observation']['features'][0].__setitem__(0,0.),
            lambda r:r['worlds'][0]['actions'][5].update(log_probability=0.),
            lambda r:r['worlds'][0]['actions'][5]['score_gradient'].__setitem__(0,1.),
            lambda r:r.update(graph_probe_mean_reward=1.),lambda r:r.update(category_only_upper_bound=0.5),
            lambda r:r['worlds'][1]['category_only_probabilities'][0].__setitem__(0,1.)]
        for i,mutate in enumerate(mutations):
            r=copy.deepcopy(receipt);mutate(r)
            with self.subTest(case=i),self.assertRaises(ERRORS):v.audit_policy(r,data)

    @unittest.skipUnless(os.environ.get('ERE_RULE_RELATIONAL_AUDIT_INPUT'),'postlaunch native panel only')
    def test_full_native_matrix_and_matched_inventory(self):
        r=v.audit_panel(Path(os.environ['ERE_RULE_RELATIONAL_AUDIT_INPUT']))
        self.assertEqual((r['cells'],r['native_steps_checked'],r['novel_fires'],r['paid_edits']),(396,9504,324,522))
        self.assertTrue(r['identical_ordinary_ids_and_data']);self.assertEqual(r['training_updates'],0)

    @unittest.skipUnless(os.environ.get('ERE_RULE_RELATIONAL_AUDIT_INPUT'),'postlaunch native panel only')
    def test_measured_causal_receipts_and_relation_poisoning(self):
        root=Path(os.environ['ERE_RULE_RELATIONAL_AUDIT_INPUT'])
        for arm in v.ARMS:
            r=v.integrity.load(root/v.filename(v.SEEDS[0],[10,11],arm,[1,1]));checked=v.audit_receipt(r)
            self.assertEqual(checked['reward'],0.375 if arm in ('live_paid','reverse_paid') else 0.)
            bad=copy.deepcopy(r);bad['observation_nodes'].reverse()
            with self.assertRaises(ERRORS):v.audit_receipt(bad)
        r=v.integrity.load(root/v.filename(v.SEEDS[0],[10,11],'live_paid',[1,1]))
        bad=copy.deepcopy(r);bad['encoded_observation']=v.observe(observation([20,21]))
        with self.assertRaises(ERRORS):v.audit_receipt(bad)


if __name__=='__main__':unittest.main()
