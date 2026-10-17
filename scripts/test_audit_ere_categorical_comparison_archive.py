import hashlib
import json
from pathlib import Path
import tempfile
import unittest

import audit_ere_categorical_comparison_archive as h

REJECTIONS = []


class ArchivePolicyTests(unittest.TestCase):
    @classmethod
    def tearDownClass(cls):
        assert len(REJECTIONS) == len(set(REJECTIONS)) == 4
        print(json.dumps(dict(detached_source_rejections=REJECTIONS)))

    def test_failed_lineage_and_original_correction_remain_pinned(self):
        self.assertEqual(h.a.digest(h.PRIOR/'output_sha256.json'), h.PRIOR_ROOT)
        self.assertEqual(h.a.digest(Path(h.s.cat.__file__)), h.s.ADAPTER_SHA)
        self.assertEqual(len(h.s.PROOF['substitutions']), 3)
        self.assertTrue(h.s.PROOF['strict_f32_bits_unchanged'])
        self.assertTrue(h.s.PROOF['scientific_criteria_unchanged'])

    def test_snapshot_content_missing_symlink_and_unsafe_name_refuse(self):
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory); file = base/'source.rs'
            payload = b'fixed measurement source'
            file.write_bytes(payload); sources = {'source.rs':hashlib.sha256(payload).hexdigest()}
            h.verify_snapshot_sources(base, sources)
            file.write_bytes(payload+b'drift')
            with self.assertRaisesRegex(AssertionError, 'snapshot source drift'):
                h.verify_snapshot_sources(base, sources)
            REJECTIONS.append('snapshot_content')
            file.unlink()
            with self.assertRaisesRegex(AssertionError, 'missing source'):
                h.verify_snapshot_sources(base, sources)
            REJECTIONS.append('snapshot_missing')
            target=base/'target'; target.write_bytes(payload); file.symlink_to(target)
            with self.assertRaisesRegex(AssertionError, 'symlink'):
                h.verify_snapshot_sources(base, sources)
            REJECTIONS.append('snapshot_symlink')
            with self.assertRaisesRegex(AssertionError, 'unsafe source'):
                h.verify_snapshot_sources(base, {'../source.rs':sources['source.rs']})
            REJECTIONS.append('snapshot_unsafe_name')

    def test_live_drift_is_descriptive_not_snapshot_substitution(self):
        with tempfile.TemporaryDirectory() as directory:
            base=Path(directory); frozen=base/'frozen'; live=base/'live'
            frozen.mkdir(); live.mkdir(); data=b'pinned'; digest=hashlib.sha256(data).hexdigest()
            (frozen/'source.rs').write_bytes(data); (live/'source.rs').write_bytes(data)
            sources={'source.rs':digest}; h.verify_snapshot_sources(frozen, sources)
            self.assertEqual(h.checkout_drift(sources, live)['matching_sources'], 1)
            (live/'source.rs').write_bytes(b'new implementation')
            report=h.checkout_drift(sources, live)
            self.assertEqual(report['changes'][0]['status'], 'changed')
            self.assertFalse(report['current_checkout_identity_claim'])
            self.assertFalse(report['current_code_admission_claim'])
            h.verify_snapshot_sources(frozen, sources)
            (live/'source.rs').unlink()
            self.assertEqual(h.checkout_drift(sources, live)['changes'][0]['status'], 'unavailable_or_unsafe')

    def test_no_live_code_or_training_claim_in_historical_policy(self):
        report=h.checkout_drift({}, h.ROOT)
        self.assertFalse(report['current_checkout_identity_claim'])
        summary=h.a.load(h.s.INPUT/'summary.json')
        self.assertFalse(summary['support_categorical_direction'])
        self.assertEqual(summary['fixed_denominator'], 8)
        self.assertLess(summary['categorical_weighted_validity_effect'], -1e-12)


if __name__ == '__main__':
    unittest.main()
