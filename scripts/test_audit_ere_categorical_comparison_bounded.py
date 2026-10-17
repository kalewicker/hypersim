import copy
import json
from pathlib import Path
import tempfile
import unittest

import audit_ere_categorical_comparison_bounded as b

DEV = b.ROOT / 'output/ere_categorical_comparison_development_20261002'


class BoundedAuditTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.plan = b.a.load(DEV / 'schedule.json')
        cls.catalogue = b.a.load(DEV / 'reference_catalogue.json')
        cls.native, cls.h = b.cat.d.c.helpers(b.PARENT)

    def register(self, store, q):
        row = store.read_unregistered(q['id'])
        b.cat.d.b.exact_schedule([row['schedule']], [q])
        b.cat.d.b.cell(row, q, self.catalogue, self.native, self.h)
        store[q['id']] = row
        return row

    def test_exact_storage_only_ast_and_pinned_legacy(self):
        result = b.verify_storage_only()
        self.assertTrue(result['audit_ast_equal_except_two_storage_expressions'])
        self.assertEqual(result['legacy_sha256'], b.a.load(DEV / 'snapshot.json')['sources']['scripts/audit_ere_categorical_comparison.py'])

    def test_complete_rows_are_discarded_and_exactly_reloaded(self):
        store = b.BoundedRows(DEV / 'cells')
        for q in self.plan[:16]:
            original = self.register(store, q)
            reloaded = store[q['id']]
            b.exact(reloaded, original)
            self.assertIsNot(reloaded, original)
            self.assertIsNone(store._pending)
        self.assertEqual(len(store), 16)
        self.assertTrue(all(isinstance(h, str) and len(h) == 64 for h in store.bindings.values()))
        self.assertEqual(store.reads, 32)
        with self.assertRaises(KeyError):
            store['missing']

    def test_duplicate_json_and_unsafe_id_refusal(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)
            (path / 'x.json').write_text('{"status":"Complete","status":"NotRun"}')
            store = b.BoundedRows(path)
            with self.assertRaisesRegex(AssertionError, 'duplicate JSON key'):
                store.read_unregistered('x')
            for key in ('../x', '/x', 'x/y', 'x\\y', ''):
                with self.assertRaisesRegex(AssertionError, 'unsafe cell id'):
                    store.read_unregistered(key)
            (path / 'link.json').symlink_to(DEV / 'cells' / (self.plan[0]['id'] + '.json'))
            with self.assertRaisesRegex(AssertionError, 'symlink'):
                store.read_unregistered('link')

    def test_validated_binding_and_later_mutation_refusal(self):
        q = self.plan[0]
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / (q['id'] + '.json')
            path.write_bytes((DEV / 'cells' / path.name).read_bytes())
            store = b.BoundedRows(path.parent)
            row = store.read_unregistered(q['id'])
            with self.assertRaises(AssertionError):
                store[q['id']] = copy.deepcopy(row)
            with self.assertRaises(AssertionError):
                store.read_unregistered(self.plan[1]['id'])
            b.cat.d.b.cell(row, q, self.catalogue, self.native, self.h)
            store[q['id']] = row
            with self.assertRaises(AssertionError):
                store.read_unregistered(q['id'])
            changed = copy.deepcopy(row)
            changed['evaluation']['steps'][0]['advanced'] = 0
            path.write_text(json.dumps(changed))
            with self.assertRaisesRegex(AssertionError, 'changed after native validation'):
                store[q['id']]

    def test_cap_incomplete_and_notrun_native_semantics(self):
        complete = b.a.load(DEV / 'cells' / (self.plan[0]['id'] + '.json'))
        cap = b.a.load(b.ROOT / 'output/ere_complete_champion_confirmation_20261001/cells/d33-champion-p0.json')
        cap['constraint_utility'] = 0.
        incomplete = copy.deepcopy(complete)
        incomplete.update(status='Incomplete', evaluation=None, reason='engineering fixture interrupted', partial_steps_unknown=True, constraint_utility=None)
        notrun = dict(schedule=self.plan[0], status='NotRun', reason='not_started', evaluation=None)
        with tempfile.TemporaryDirectory() as directory:
            for i, source in enumerate((cap, incomplete, notrun)):
                key = 'fixture-' + str(i)
                (Path(directory) / (key + '.json')).write_text(json.dumps(source))
                store = b.BoundedRows(directory)
                row = store.read_unregistered(key)
                result = b.cat.d.b.cell(row, row['schedule'], self.catalogue, self.native, self.h)
                store[key] = row
                b.exact(store[key], source)
                self.assertIsNone(result[2])

    def test_native_trace_credit_and_utility_tampers_rejected(self):
        q = self.plan[0]
        original = b.a.load(DEV / 'cells' / (q['id'] + '.json'))
        mutations = (
            lambda x: x['evaluation']['steps'][0].update(advanced=True),
            lambda x: x['evaluation']['steps'][0].update(world_hash='bad'),
            lambda x: x['evaluation'].update(credited_rule_ids=[999]),
            lambda x: x.update(constraint_utility=99.),
        )
        with tempfile.TemporaryDirectory() as directory:
            for mutate in mutations:
                row = copy.deepcopy(original)
                mutate(row)
                (Path(directory) / (q['id'] + '.json')).write_text(json.dumps(row))
                store = b.BoundedRows(directory)
                bad = store.read_unregistered(q['id'])
                with self.assertRaises(AssertionError):
                    b.cat.d.b.cell(bad, q, self.catalogue, self.native, self.h)
                self.assertEqual(len(store), 0)

    def test_four_physics_full_emission_equality_not_digest_projection(self):
        qs = [q for q in self.plan if q['stage'] == 'validation' and q['context'] == 0]
        vectors = b.a.load(DEV / 'directions/r0-c0.json')['vectors']
        rec = b.a.load(DEV / 'validation/r0-c0.json')
        store = b.BoundedRows(DEV / 'cells')
        for q in qs:
            self.register(store, q)
        expected = b.validation(rec, 0, 0, qs, store, self.catalogue, vectors, 8)
        eager = {q['id']: b.a.load(DEV / 'cells' / (q['id'] + '.json')) for q in qs}
        self.assertEqual(expected, b.validation(rec, 0, 0, qs, eager, self.catalogue, vectors, 8))
        eager[qs[1]['id']]['emission']['extra_engineering_marker'] = True
        # Marker is outside estimator selection; full receipt equality must still reject it.
        with self.assertRaisesRegex(AssertionError, 'physics repeats changed emission'):
            b.validation(rec, 0, 0, qs, eager, self.catalogue, vectors, 8)


if __name__ == '__main__':
    unittest.main()
