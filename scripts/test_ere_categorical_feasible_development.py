#!/usr/bin/env python3
"""Zero-physics design, source-stage, and estimation-provenance regression groups."""
import copy
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
import ere_categorical_feasible_development as d
import audit_ere_categorical_feasible_development as v


def write(path,value):path.write_text(json.dumps(value,indent=2))


class DevelopmentTests(unittest.TestCase):
    def test_fixed_allocation_and_disjoint_domains(self):
        d.same(d.a.load(d.ROOT/d.CONFIG),d.design());plan=d.schedule();d.check_schedule(plan)
        self.assertEqual(len(plan[:2112]),2112)
        self.assertTrue(all(q['stage']!='validation' for q in plan[:2112]))
        for ctx in range(2):
            for est in range(2):
                qs=[q for q in plan if q['stage']!='validation' and (q['context'],q['estimate'])==(ctx,est)]
                self.assertEqual([sum(q['stage']==stage for q in qs) for stage in ('utility_estimation','risk_estimation')],[16,512])
        for change in (lambda p:p[0].update(stage='validation'),lambda p:p[0].update(horizon=99),lambda p:p.pop()):
            bad=copy.deepcopy(plan);change(bad)
            with self.assertRaises(AssertionError):d.check_schedule(bad)

    def test_source_stage_exact_additions(self):
        rec=d.registration(d.ROOT,verify_lineage=True)
        self.assertEqual(rec['unchanged_baseline_sources']+len(rec['excluded_feature_drift']),1132)
        self.assertTrue(rec['unchanged_adapter'] and rec['unchanged_native_evaluator'])
        self.assertFalse(rec['historical_source_identity_claim'])

    def test_seed_history_whole_schedule_reserved(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);run=root/'old';run.mkdir();write(run/'schedule.json',[d.schedule()[0]])
            # Failed or ambiguous status reserves even unexecuted suffixes.
            with self.assertRaises(AssertionError):d.seed_inventory(root)
            write(run/'manifest.json',d.initial());write(run/'request.json',dict(launch_authorized=False))
            self.assertFalse(d.seed_inventory(root)['collision'])
            m=d.initial();m['grammar_compilations']=1;write(run/'manifest.json',m)
            with self.assertRaises(AssertionError):d.seed_inventory(root)

    def test_incomplete_summary_never_promoted(self):
        self.assertIsNone(d.summary(False,[dict(utility_effects=[100.]*3)])['context_points'])
        for done in (False,True):
            s=d.summary(done,[])
            self.assertFalse(s['scientific_support'] or s['training_admitted'] or s['unseen_cap_feasibility_certificate'])
            self.assertEqual(s['training_updates'],0)
        with self.assertRaises(AssertionError):d.same(0,False)

    def test_estimation_request_derived_from_rows(self):
        catalogue=d.a.load(d.BASELINE/'catalogue.json');toy=d.mathcheck.fixture(catalogue)['cases'][0]
        qs=d.schedule()[:528];rows={};panels={}
        for name,selected,labels in [('utility',qs[:16],toy['utility']['samples']),('risk',qs[16:],toy['risk']['samples'])]:
            samples=[]
            for q,old in zip(selected,labels):
                sample=dict(old,id=q['id'],emission_seed=q['emission_seed'],physics_seed=q['physics_seed'])
                samples.append(sample)
                # Only test the provenance/math subroutine here; these synthetic
                # rows deliberately do not qualify as native audited evidence.
                emission=dict(parent=sample['selection']['parent'],operator=sample['selection']['operator'],
                    role='Core' if sample['selection']['role']==0 else 'Proposal',production_index=sample['selection']['production_index'])
                rows[q['id']]=dict(status='Complete' if sample['native_score'] is not None else 'Null',
                    parameters=d.contexts()[0],emission=emission,
                    evaluation=dict(score=sample['native_score'],reason=sample['reason']),
                    constraint_utility=0. if sample['native_score'] is None else 1.+sample['native_score'])
            panels[name]=dict(stage=selected[0]['stage'],expected_draws=len(selected),samples=samples)
        request=dict(theta=d.contexts()[0],**panels)
        synthetic=dict(profile=d.mathcheck.PROFILE,purpose='engineering_synthetic_only',cases=[dict(id='provenance-toy',**request)])
        with tempfile.TemporaryDirectory() as tmp:
            path=Path(tmp);write(path/'request.json',synthetic)
            process=subprocess.run([str(d.BASELINE/'executed_binary'),str(d.BASELINE/'catalogue.json'),str(path/'request.json'),str(path/'result.json')],capture_output=True,text=True,timeout=60)
            self.assertEqual(process.returncode,0,process.stderr)
            got=d.a.load(path/'result.json')['cases'][0]['proposal']
        rec=dict(context=0,estimate=0,theta_before=d.contexts()[0],evaluation_ids=[q['id'] for q in qs],complete=True,request=request,proposal=got)
        self.assertEqual(v.estimate(rec,qs,rows,catalogue),got['theta_after'])
        for change in (lambda r:r['request']['risk'].update(stage='validation'),
                       lambda r:r['request']['risk']['samples'][0].update(id='val-c0-d0-p0'),
                       lambda r:r['request']['utility']['samples'][0].update(physics_seed=1),
                       lambda r:r['request']['risk']['samples'].pop(),
                       lambda r:r['proposal'].update(training_admitted=True)):
            bad=copy.deepcopy(rec);change(bad)
            with self.assertRaises(AssertionError):v.estimate(bad,qs,rows,catalogue)
        badrows=copy.deepcopy(rows);badrows[qs[-1]['id']]['constraint_utility']=None
        with self.assertRaises(AssertionError):v.estimate(rec,qs,badrows,catalogue)
        incomplete=copy.deepcopy(rec);incomplete.update(complete=False,request=None,proposal=None)
        self.assertIsNone(v.estimate(incomplete,qs,badrows,catalogue))

    def test_preflight_not_training_authority(self):
        m=d.initial();self.assertEqual(m['scheduled_cells'],2176)
        for k in ('recorded_cells','attempted_steps','grammar_compilations','training_updates','direction_estimates'):
            self.assertEqual(m[k],0)
        self.assertFalse(m['training_admitted'] or m['full_training_authorized'])
        inventory=d.seed_inventory();self.assertFalse(inventory['collision'])


if __name__=='__main__':unittest.main(verbosity=2)
