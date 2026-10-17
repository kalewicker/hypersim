#!/usr/bin/env python3
import tempfile
import unittest
from pathlib import Path
import verify_evolving_minds_development as pilot


def fixture():
    rows = {}
    for case in pilot.cases():
        arm = case['arm']
        members = []
        for bid in [1, 2, 3, 4]:
            attempts = 12
            successes = 6 if arm == 'coin' else 12
            members.append(dict(id=bid, parent=None if bid <= 2 else bid - 2,
                completion_tick=5, attempts=attempts, successes=successes, failures=attempts-successes,
                payable_action_opportunities=attempts, action_funding_blocked_ticks=0,
                funded_parent_ticks=1, eligible_parent_ticks=1, requests=1))
        rows[case['name']] = dict(opportunity=dict(lifetimes=members, ticks=list(range(64)),
            primary_descendant_harvested=256 if arm == 'inherited' else 64,
            founder_harvested=192, descendants_grammar_failed=0, descendants_live_at_64=1, requests=4))
    return rows


class DevelopmentDecisionTests(unittest.TestCase):
    def test_complete_feasibility_does_not_admit_science_or_power(self):
        result = pilot.ecological_decision(fixture())
        self.assertTrue(result['ecological_feasibility_pass'])
        self.assertFalse(result['ready_for_scientific_launch'])
        self.assertFalse(result['power_adequacy_established'])
        self.assertFalse(result['scientific_sign_test_performed'])

    def test_omitted_cell_refused(self):
        rows = fixture(); rows.pop('world109_retained')
        with self.assertRaisesRegex(ValueError, 'denominator'):
            pilot.ecological_decision(rows)

    def test_failed_proposal_no_request_is_retained_as_failure(self):
        rows = fixture(); rows['world101_retained']['opportunity']['lifetimes'][0]['requests'] = 0
        result = pilot.ecological_decision(rows)
        self.assertFalse(result['ecological_feasibility_pass'])
        self.assertEqual(len(result['worlds']), 4)
        self.assertIn('world101_retained:founder_exposure', result['failed_checks'])

    def test_truncated_horizon_refused(self):
        rows = fixture(); rows['world103_faithful']['opportunity']['ticks'].pop()
        with self.assertRaisesRegex(ValueError, 'full horizon'):
            pilot.ecological_decision(rows)

    def test_missing_founder_cannot_be_replaced_by_descendant(self):
        rows = fixture(); rows['world101_coin']['opportunity']['lifetimes'][1]['parent'] = 1
        with self.assertRaisesRegex(ValueError, 'founder denominator'):
            pilot.ecological_decision(rows)

    def test_one_failed_world_not_rescued_by_large_mean(self):
        rows = fixture()
        rows['world109_inherited']['opportunity']['primary_descendant_harvested'] = 64
        rows['world101_inherited']['opportunity']['primary_descendant_harvested'] = 10000
        result = pilot.ecological_decision(rows)
        self.assertGreater(result['paired']['inherited_minus_faithful']['mean'], 64)
        self.assertFalse(result['paired']['inherited_minus_faithful']['passed'])
        self.assertFalse(result['paired']['inherited_minus_retained']['passed'])

    def test_chance_control_extreme_accuracy_not_averaged_away(self):
        rows = fixture()
        for member in rows['world107_coin']['opportunity']['lifetimes'][:2]:
            member['successes'] = 12; member['failures'] = 0
        result = pilot.ecological_decision(rows)
        self.assertIn('world107_coin:chance_control', result['failed_checks'])

    def test_paid_attempt_mismatch_refused(self):
        rows = fixture(); rows['world101_inherited']['opportunity']['lifetimes'][0]['attempts'] = 11
        with self.assertRaisesRegex(ValueError, 'paid attempt accounting'):
            pilot.ecological_decision(rows)

    def test_resource_projection_uses_worst_cell_and_fixed_margin(self):
        rows = {c['name']: dict(total_bytes=1024, advance_seconds=1, audit_seconds=1) for c in pilot.cases()}
        rows['world109_coin']['audit_seconds'] = 61
        result = pilot.resource_decision(rows)
        self.assertEqual(result['conservative_projections']['campaign_audit_seconds'], 7320)
        self.assertFalse(result['resource_projection_pass'])

    def test_extra_world_directory_refused(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            import json
            (root / 'registered_cases.json').write_text(json.dumps(pilot.cases()))
            for case in pilot.cases():
                (root / case['name']).mkdir()
            (root / 'world211_inherited').mkdir()
            with self.assertRaisesRegex(ValueError, 'directory denominator'):
                pilot.denominator(root)


if __name__ == '__main__':
    unittest.main()
