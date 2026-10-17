#!/usr/bin/env python3
"""Read-only checker regressions. Optional argument: existing development archive."""
import copy
import gzip
import json
from pathlib import Path
import sys
import tempfile
import time
import unittest

import verify_ere_complete_gr1 as a

ARCHIVE = Path(sys.argv.pop(1)).resolve() if len(sys.argv) > 1 and not sys.argv[1].startswith("-") else None


class PredicateTests(unittest.TestCase):
    def setUp(self):
        a.DEADLINE = time.monotonic() + 120

    def test_quarantine_and_caps_precede_neutral_or_catastrophe(self):
        complete = {"complete": True, "frames": [("x", 1)], "fingerprints": ["same"], "exploding": False, "frozen": False}
        capped = {**complete, "complete": False, "exploding": True, "frozen": True}
        self.assertEqual(a.classification(False, False, complete, None), "quarantined")
        self.assertEqual(a.classification(True, True, capped, None), "unmeasured")
        self.assertEqual(a.classification(True, True, complete, capped), "unmeasured")
        self.assertEqual(a.classification(True, True, complete, None), "neutral")
        self.assertEqual(a.classification(True, False, complete, complete), "near_neutral")
        exploding = {**complete, "frames": [("y", 1)], "exploding": True}
        self.assertEqual(a.classification(True, False, complete, exploding), "catastrophic")
        self.assertEqual(a.classification(True, False, complete, {**exploding, "exploding": False}), "divergent")

    def test_paired_hash_equality_requires_typed_world_agreement(self):
        p = {"complete": True, "frames": [("x", 1)], "fingerprints": ["one"]}
        for neutral in [False, True]:
            with self.assertRaises(AssertionError):
                a.classification(True, neutral, p, {**p, "fingerprints": ["other"]})
        self.assertEqual(a.classification(True, True, p, p), "neutral")
        self.assertEqual(a.classification(True, True, p, None), "neutral")

    def test_unsampled_or_quarantined_parents_cannot_hide_failed_baseline(self):
        summary = {"parent_baselines_complete": True, "strata": {"overall": {"quarantined": 10000}}}
        self.assertTrue(a.measurement_complete(summary))
        summary["parent_baselines_complete"] = False
        self.assertFalse(a.measurement_complete(summary))
        summary["parent_baselines_complete"] = True
        summary["strata"]["overall"]["unmeasured"] = 1
        self.assertFalse(a.measurement_complete(summary))

    def test_short_or_missing_scientific_envelope_and_training_flag(self):
        d = a.load(Path(__file__).resolve().parents[1] / "docs/ere_complete_gr1_v1.json")
        m = {"profile": a.PROFILE, "protocol": d["protocol"], "family_version": 10, "parents": 168,
             "controls": 5376, "training_admitted": False, "frozen_before_measurement": True,
             "development_only": True, "scientific_run_launched": False, "resources": d["proposed_resources"],
             "draws": 64, "horizon": 8, "sources": ["a"],
             "development_resources": {"wall": 300, "output": 536870912, "audit": 120, "steps": 1856}}
        a.check_manifest(m, d, True)
        for change in [lambda r: r.__setitem__("draws", 63), lambda r: r.__setitem__("horizon", 100),
                       lambda r: r.__setitem__("training_admitted", True), lambda r: r.__setitem__("controls", 5375),
                       lambda r: r["resources"].__setitem__("classification_work_limit_per_step", 400001)]:
            r = copy.deepcopy(m)
            change(r)
            with self.assertRaises(AssertionError):
                a.check_manifest(r, d, True)
        with self.assertRaises(AssertionError):
            a.check_manifest(m, d, False)

    def test_49_50_window_and_exact_fourfold_boundary(self):
        frozen = lambda frames: any(all(n == 0 for n in frames[i:i+50]) for i in range(max(0, len(frames)-49)))
        self.assertFalse(frozen([0] * 49))
        self.assertTrue(frozen([0] * 50))
        self.assertFalse(frozen([0] * 49 + [1]))
        self.assertTrue(frozen([1] + [0] * 50))
        self.assertFalse(400 > 4 * 100)
        self.assertTrue(401 > 4 * 100)
        self.assertFalse(a.tally([])["numerical_pass"])

    def test_unguarded_counters_are_omitted_not_fabricated(self):
        table = [{"name": "e1_test", "pattern": {"property_guards": []}}]
        row = {"guards": [], "ceiling": False, "native": {"stats": {"attempted_matches": 2}},
               "usage": {"work": 3, "results": 2, "truncated": False,
                         "rules": [{"rule_id": 0, "work": 3, "results": 2, "truncated": False,
                                    "work_allowance": 400000, "result_allowance": 50000}]}}
        a.check_usage(row, table)
        forged = copy.deepcopy(row)
        forged["guards"] = [["e1_test", {}]]
        with self.assertRaises(AssertionError):
            a.check_usage(forged, table)
        # A cap stops enumeration before later guarded rules are visited.
        capped = copy.deepcopy(row)
        capped["ceiling"] = capped["usage"]["truncated"] = capped["usage"]["rules"][0]["truncated"] = True
        a.check_usage(capped, table + [{"name": "unvisited", "pattern": {"property_guards": [{}]}}])

    def test_canonical_stream_truncation_and_deadline(self):
        with tempfile.TemporaryDirectory(prefix="ere-gr1-checker-") as tmp:
            p = Path(tmp) / "data.gz"
            for bad in ['{"a": 1}\n', '{"a":1}', '{"a":1,"a":1}\n']:
                with gzip.open(p, "wt") as f:
                    f.write(bad)
                with self.assertRaises(AssertionError):
                    list(a.stream(p))
            with gzip.open(p, "wt") as f:
                f.write('{"a":1}\n')
            self.assertEqual(list(a.stream(p)), [{"a": 1}])
            a.DEADLINE = time.monotonic() - 1
            with self.assertRaises(AssertionError):
                list(a.stream(p))

    def test_root_seal_covers_nested_basis_seals(self):
        with tempfile.TemporaryDirectory(prefix="ere-gr1-seal-") as tmp:
            root = Path(tmp)
            nested = root / "basis" / "output_sha256.json"
            nested.parent.mkdir()
            nested.write_text("{}\n")
            seal = root / "output_sha256.json"
            seal.write_text(json.dumps({"basis/output_sha256.json": a.digest(nested)}))
            a.check_seal(root)
            seal.write_text("{}\n")
            with self.assertRaises(AssertionError):
                a.check_seal(root)
            seal.write_text(json.dumps({"basis/output_sha256.json": a.digest(nested)}))
            nested.write_text('{"tampered":true}\n')
            with self.assertRaises(AssertionError):
                a.check_seal(root)


@unittest.skipIf(ARCHIVE is None, "existing development archive required")
class ArchiveTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        a.DEADLINE = time.monotonic() + 120
        cls.m, cls.d = a.integrity(ARCHIVE, True)
        cls.parents = a.verify_parent_manifest(ARCHIVE)

    def setUp(self):
        a.DEADLINE = time.monotonic() + 120

    def test_all_mutation_edits_and_independent_decoding(self):
        rows = list(a.stream(ARCHIVE / "classifications.jsonl.gz"))
        draws = list(a.stream(ARCHIVE / "draw_manifest.jsonl.gz"))
        self.assertEqual(len(rows), 64)
        for i, (row, d) in enumerate(zip(rows, draws)):
            self.assertEqual(d["index"], i)
            p = self.parents[d["parent"]]
            after = a.raw_edit(p, d)
            self.assertEqual(a.decode(after, p["root"]), row["decoded"])
        expected = a.tally(rows)
        expected["parent_baselines_complete"] = True
        self.assertEqual(expected, a.load(ARCHIVE / "classification_summary.json"))

    def test_noop_dormant_and_metadata_edit_accounting(self):
        examples = list(a.stream(ARCHIVE / "draw_manifest.jsonl.gz"))
        self.assertTrue(any(not d["edit"]["applicable"] for d in examples))
        self.assertTrue(any(d["role"] == "Proposal" and not self.parents[d["parent"]]["program"]["active"] for d in examples))
        for d in examples:
            p = self.parents[d["parent"]]
            if d["operator"] in ["AddSlot", "RemoveSlot", "RetargetSlot"]:
                after = a.raw_edit(p, d)
                self.assertEqual(a.metadata(after), a.metadata(p["inventory"]))

    def test_root_absence_not_quarantine_or_cached_install(self):
        rs = a.stream(ARCHIVE / "controls.jsonl.gz")
        ms = a.stream(ARCHIVE / "control_manifest.jsonl.gz")
        for r, m in zip(rs, ms):
            if m["attack"] not in ["root_delete", "root_relabel"]:
                continue
            a.check_control(self.parents[m["parent"]], m, r)
            self.assertEqual(r["compiled"]["report"]["quarantined"], 0)
            self.assertEqual(r["compiled"]["report"]["retired"], 1)
            forged = copy.deepcopy(r)
            forged["compiled"]["report"]["quarantined"] = 1
            with self.assertRaises(AssertionError):
                a.check_control(self.parents[m["parent"]], m, forged)
            break

    def test_seal_and_all_eighteen_recorded_tamper_categories(self):
        audit = a.load(ARCHIVE / "audit_full.json")
        self.assertTrue(audit["audit_pass"])
        self.assertFalse(audit["gate_pass"])
        self.assertEqual(audit["tamper_rejections"], 18)
        self.assertEqual(len(set(audit["tamper_categories"])), 18)
        status = a.load(ARCHIVE / "run_status.json")
        self.assertFalse(status["scientific_run_launched"])
        self.assertFalse(status["training_admitted"])
        self.assertEqual(status["advancing_engine_steps"], audit["resources"]["advancing"])


if __name__ == "__main__":
    unittest.main(verbosity=2)
