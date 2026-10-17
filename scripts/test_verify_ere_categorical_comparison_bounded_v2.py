import unittest
import verify_ere_categorical_comparison_bounded_v2 as v


class PackagingTests(unittest.TestCase):
    def test_preflight_has_no_control_field_and_is_not_modified(self):
        saved = dict(audit_pass=True, recorded_cells=0, training_admitted=False)
        self.assertEqual(v.accounting_only(saved, False), saved)
        self.assertNotIn('detached_tamper_rejections', saved)

    def test_development_strips_only_six_control_metadata(self):
        saved = dict(audit_pass=True, recorded_cells=192, detached_tamper_rejections=['x']*6)
        self.assertEqual(v.accounting_only(saved, True), dict(audit_pass=True, recorded_cells=192))
        self.assertEqual(len(saved['detached_tamper_rejections']), 6)

    def test_wrong_shapes_fail_closed(self):
        with self.assertRaises(KeyError):
            v.accounting_only(dict(audit_pass=True), True)
        with self.assertRaises(AssertionError):
            v.accounting_only(dict(detached_tamper_rejections=['x']*5), True)
        with self.assertRaises(AssertionError):
            v.accounting_only(dict(detached_tamper_rejections=[]), False)


if __name__ == '__main__':
    unittest.main()
