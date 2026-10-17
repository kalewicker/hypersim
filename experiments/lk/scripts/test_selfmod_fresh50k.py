"""Boundary tests for an extension with no renewed warmup or temporal replication."""
import copy
import unittest
from test_selfmod_durability import fixture, SELECTED
from selfmod_fresh50k_common import panel_maintenance
def maintenance(selected,warmup,future,complete):
    return panel_maintenance(selected,warmup+future,complete)


def observations():
    windows = [copy.deepcopy(w) for w in fixture()*5]
    for i,w in enumerate(windows):
        w['first_step']=1+1000*i
        w['last_step']=1000+1000*i
    return windows[:2],windows[2:]


class MaintenanceTests(unittest.TestCase):
    def test_all_fortyeight_windows_and_six_blocks_pass(self):
        warmup, future = observations()
        result = maintenance(SELECTED, warmup, future, True)
        self.assertTrue(result['maintained_success'])
        self.assertEqual(result['scored_windows'], 48)
        self.assertEqual(len(result['temporal_blocks']), 6)
        self.assertEqual(result['renewed_warmup_steps'], 0)

    def test_first_two_future_windows_are_scored(self):
        for i in [0, 1]:
            warmup, future = observations()
            future[i]['minimum_rent_headroom'] = -1
            self.assertFalse(maintenance(SELECTED, warmup, future, True)['maintained_success'])

    def test_gap_in_any_temporal_block_fails_whole_world(self):
        for i in [0, 8, 16, 24, 32, 40, 47]:
            warmup, future = observations()
            future[i]['lineage_birth_windows'] = [0]*4
            self.assertFalse(maintenance(SELECTED, warmup, future, True)['maintained_success'])

    def test_any_quarter_or_culture_gap_fails(self):
        for key, bad in [('lineage_productivity_windows', [1, 0, 1, 1]),
                         ('birth_windows', [1, 1, 0, 1]),
                         ('members_alive_every_boundary', False),
                         ('family_alive_every_boundary', False)]:
            warmup, future = observations()
            future[17][key] = bad
            self.assertFalse(maintenance(SELECTED, warmup, future, True)['maintained_success'])

    def test_enumeration_and_short_observation_cannot_qualify(self):
        warmup, future = observations()
        future[0]['enumeration_censored'] = True
        result = maintenance(SELECTED, warmup, future, True)
        self.assertFalse(result['maintained_success'])
        self.assertFalse(result['uncensored_measured'])
        warmup, future = observations()
        for short in [future[:2], future[:32], future[:47]]:
            self.assertFalse(maintenance(SELECTED, warmup, short, True)['maintained_success'])
        self.assertFalse(maintenance(SELECTED, warmup, future, False)['maintained_success'])

    def test_clock_gap_or_native_stock_flow_corruption_rejected(self):
        warmup, future = observations()
        future[0]['first_step'] = 12001
        with self.assertRaises(ValueError):
            maintenance(SELECTED, warmup, future, True)
        warmup, future = observations()
        future[21]['income'] += 1
        with self.assertRaises(ValueError):
            maintenance(SELECTED, warmup, future, True)

    def test_original_warmup_only_is_reused_and_not_modified(self):
        warmup, future = observations()
        warmup[0]['minimum_rent_headroom'] = -1
        before = copy.deepcopy((warmup, future))
        self.assertTrue(maintenance(SELECTED, warmup, future, True)['maintained_success'])
        self.assertEqual((warmup, future), before)


if __name__ == '__main__':
    unittest.main()
