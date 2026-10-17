"""Adversarial wiring checks; synthetic pedigrees/scores never count as evolution."""
import copy
import json
from pathlib import Path
import sys
import unittest
from campaign_measurement import plan, extend, evaluate
from audit_campaign_measurement import check, worlds, add_roles, independent_report
from plan_versions import descent
from audit_versions import reconstruction
import test_versions_audit as fixtures


class MeasurementTests(unittest.TestCase):
    def test_saved_complete_census(self):
        m,r=plan(CASES,C); self.assertEqual(check(CASES,C,m,r),dict(samples=40,requests=9,retained_pairs=0))
        self.assertTrue(any(s['status']=='dead' for s in m['samples']))
        self.assertEqual(len({(s['source'],s['census'],s['slot']) for s in m['samples']}),40)

    def test_census_and_registry_corruptions(self):
        m,r=plan(CASES,C)
        for mutate in [lambda m,r:m['samples'].pop(),lambda m,r:m['samples'][0]['ranking'].reverse(),
                       lambda m,r:r['assays'][0].update(role='restored'),lambda m,r:r['assays'][0]['stages'].append(0),
                       lambda m,r:m['samples'][0]['lineage']['tokens'][0].update(acquisition='invented')]:
            a,b=copy.deepcopy((m,r));mutate(a,b)
            with self.assertRaises(ValueError):check(CASES,C,a,b)

    def test_partial_world_preserves_unknown_slots(self):
        cases=copy.deepcopy(CASES); cases[0]['rows']=cases[0]['rows'][:4];cases[0]['final']=cases[0]['rows'][-1]['after']
        m,r=plan(cases,C);check(cases,C,m,r)
        self.assertEqual(sum(s['status']=='unmeasured' for s in m['samples']),6)
        report=evaluate(m,r,[],cases,C)
        self.assertTrue(all(w['qualified'] is None for w in report['worlds']))
        self.assertEqual(report,independent_report(m,r,[],cases,C))

    def test_synthetic_full_pair_and_redundant_padding(self):
        states,hs=fixtures.synthetic();lineage=extend(descent(states,hs,2)); other=reconstruction(states,hs,2)
        for pair in other['pairs']:add_roles(other,pair)
        self.assertEqual(lineage,other);pair=lineage['pairs'][0]
        self.assertEqual(set(pair['programs']),set(C['candidate']['roles']))
        self.assertEqual(pair['programs']['padded_remove_later'],[0,1,-1,1])
        self.assertEqual(pair['programs']['padded_remove_both'],[0,-1,-1,-1])
        self.assertEqual(pair['programs']['remove_earlier'],[0,2])
        self.assertEqual(pair['programs']['single_remove_earlier'],[0,2,1])
        self.assertEqual(pair['origins'],['authored','authored'])
        self.assertEqual(pair['active_descendant_generations'],[[2],[1,2]])

    def test_reacquisition_not_novel_and_bad_event_rejected(self):
        states,hs=fixtures.synthetic(True);self.assertEqual(extend(descent(states,hs,2))['pairs'],[])
        states,hs=fixtures.synthetic();lineage=descent(states,hs,2)
        lineage['pairs'][0]['later']=lineage['pairs'][0]['earlier']
        with self.assertRaises(ValueError):extend(lineage)

    def test_synthetic_decisions_all_arms_missing_and_unfunded(self):
        roles=C['candidate']['roles'];scores=[0,7,7,22,22,-5,7,-10,-5,22,22,2,-10]
        registry=dict(assays=[dict(id=i,role=r,stages=[0,1,2]) for i,r in enumerate(roles)])
        results=[dict(request=r,native=dict(results=[dict(score=scores[r['id']],status='complete')])) for r in registry['assays']]
        pair=dict(earlier='synthetic-a',later='synthetic-b',origins=['endogenous']*2,transmissions=[2,2],active_descendant_generations=[[1,2],[1,2]],assays=dict(zip(roles,range(13))),inadmissible_empty_ablation=False)
        for arm in C['design']['arms']:
            for mode in ['positive','authored','missing','unfunded','empty','no_dependency','short_activity']:
                p=copy.deepcopy(pair); p['origins']=[arm]*2
                m=dict(profile='cumulative-campaign-measurement-readiness-v1',samples=[dict(source=arm,census=t,slot=s,status='living',lineage=dict(pairs=[p] if t==0 and s==0 else [])) for t in C['design']['census_ticks'] for s in range(2)])
                rs=copy.deepcopy(results)
                if mode=='authored':p['origins']=['authored']*2
                if mode=='missing':rs.pop()
                if mode=='unfunded':rs[0]['native']['results'][0]['status']='unfunded_copy'
                if mode=='empty':p['assays']['remove_earlier']=None;p['inadmissible_empty_ablation']=True
                if mode=='no_dependency':rs[-1]['native']['results'][0]['score']=-20
                if mode=='short_activity':p['active_descendant_generations']=[[2],[1,2]]
                cases=[dict(name=arm,initial=dict(arm=arm),final=dict(tick=16))]
                actual=evaluate(m,registry,rs,cases,C)
                self.assertEqual(actual,independent_report(m,registry,rs,cases,C))
                decision=actual['worlds'][0]
                if mode=='missing':self.assertEqual(decision['status'],'unknown');self.assertIsNone(decision['qualified'])
                elif mode in ['positive','authored'] and arm in ['nonheritable','frozen']:self.assertEqual(decision['status'],'mechanical_control_violation')
                elif mode=='positive' and arm in ['endogenous','external']:self.assertTrue(decision['qualified'])
                else:self.assertFalse(decision['qualified'])

    def test_result_identity_duplicate_and_boolean_score_rejected(self):
        m,r=plan(CASES,C); record=dict(request=r['assays'][0],native=dict(results=[dict(score=0,status='complete')]))
        variants=[[record,record],[dict(request=dict(id=0,role='sham',stages=[0]),native=record['native'])],
                  [dict(request=r['assays'][0],native=dict(results=[dict(score=True,status='complete')]))],
                  [dict(request=r['assays'][0],native=dict(results=[]))]]
        for records in variants:
            with self.assertRaises(ValueError):evaluate(m,r,records,CASES,C)


if __name__=='__main__':
    root=Path(sys.argv.pop(1));C=json.loads((root/'experiments/cumulative/campaign_planning_contract.json').read_text())
    CASES=worlds(root/'output/cumulative_campaign_adapter_readiness_20261010')
    fixtures.DATA=[json.loads((root/'output/cumulative_regulated_world_lineage_readiness_20261009/fixture.json').read_text())]
    unittest.main(verbosity=2)
