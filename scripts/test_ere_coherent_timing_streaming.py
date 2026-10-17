#!/usr/bin/env python3
"""Engineering-only timing parity and bounded ownership regressions."""
import copy
import gzip
import importlib.util
from pathlib import Path
import unittest
import tempfile
from unittest.mock import patch
import weakref

import verify_ere_complete_coherent_draw_assay as candidate

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location(
    "frozen_timing_reference", ROOT / "output/ere_complete_coherent_draw_full_20260929/source_snapshots/scripts/verify_ere_complete_coherent_draw_assay.py")
reference = importlib.util.module_from_spec(spec)
spec.loader.exec_module(reference)


def fixture(ids=(None, "child_1", None, "child_3", None), development=False):
    controls = {"controls": 5376, "attempted_engine_steps": 0, "seconds": 1}
    timing = {"draws": 10000, "horizon": 100, "parent_phase_seconds": 200,
              "mutation_phase_seconds": 4, "child_trace_seconds": 2,
              "mutation_nontrace_seconds": 2,
              "parent_traces": [{"id": f"parent_{i}", "seconds": 1, "attempted_steps": 100} for i in range(168)],
              "child_traces": [{"id": x, "seconds": 1, "attempted_steps": 100} for x in ids if x is not None]}
    rows = [{"trace_id": x, "payload": {"typed": [{"Int": 5}, {"Float": 5.0}]}} for x in ids]
    timing["child_trace_seconds"] = len(timing["child_traces"])
    timing["mutation_phase_seconds"] = len(timing["child_traces"]) + 2
    manifest = {"draws": 10000, "horizon": 100, "development_only": development}
    totals = {"child_traces": len(timing["child_traces"]), "attempted": 16800 + 100 * len(timing["child_traces"])}
    return controls, timing, rows, manifest, totals


def accepted(module, values):
    c, t, rows, m, totals = values
    with patch.object(module, "load", side_effect=[c, t]), patch.object(module, "stream", return_value=iter(rows)):
        try:
            module.check_native_timing(Path("/synthetic"), m, totals)
        except (AssertionError, KeyError, TypeError):
            return False
        return True


class TimingTests(unittest.TestCase):
    def parity(self, values, expected):
        original = copy.deepcopy(values)
        self.assertEqual(accepted(reference, values), expected)
        self.assertEqual(accepted(candidate, values), expected)
        self.assertEqual(values, original, "read-only timing check mutated evidence")

    def test_valid_null_empty_and_duplicate_sequences(self):
        # Duplicate IDs are rejected upstream by classifier coverage, not by
        # the legacy timing function. Preserve that exact division of checks.
        for ids in [(), (None,), ("child_1",), (None, "child_1", None), ("child_1", "child_1")]:
            for dev in [True, False]:
                with self.subTest(ids=ids, development=dev):
                    self.parity(fixture(ids, dev), True)

    def test_trace_coverage_and_order_tampers(self):
        for edit in [lambda r: r.pop(1), lambda r: r.append({"trace_id": "extra"}),
                     lambda r: r[1].__setitem__("trace_id", "child_3"),
                     lambda r: r[3].__setitem__("trace_id", "child_1"),
                     lambda r: r[1].__setitem__("trace_id", None),
                     lambda r: r[-1].pop("trace_id")]:
            v = fixture(); edit(v[2]); self.parity(v, False)

    def test_missing_extra_native_child_and_parent_coverage(self):
        for edit in [lambda t: t["child_traces"].pop(),
                     lambda t: t["child_traces"].append({"id": "extra", "seconds": 0, "attempted_steps": 0}),
                     lambda t: t["parent_traces"].reverse(),
                     lambda t: t["parent_traces"].pop(),
                     lambda t: t["child_traces"].reverse()]:
            v = fixture(); edit(v[1]); self.parity(v, False)

    def test_clock_count_and_sum_tampers(self):
        for field in ["parent_phase_seconds", "mutation_phase_seconds", "child_trace_seconds", "mutation_nontrace_seconds"]:
            for value in [-1, True, float("inf"), float("nan"), 3600]:
                v = fixture(); v[1][field] = value
                # NaN does not compare equal to itself; parity function's
                # ownership assertion is tested separately on finite values.
                self.assertFalse(accepted(reference, v)); self.assertFalse(accepted(candidate, v))
        for mutate in [lambda v: v[0].__setitem__("attempted_engine_steps", 1),
                       lambda v: v[4].__setitem__("attempted", 1),
                       lambda v: v[1]["child_traces"][0].__setitem__("attempted_steps", 101),
                       lambda v: v[1]["child_traces"][0].__setitem__("seconds", -1),
                       lambda v: v[1].__setitem__("mutation_phase_seconds", 99),
                       lambda v: v[1].__setitem__("parent_phase_seconds", 1),
                       lambda v: v[1].__setitem__("draws", 9999),
                       lambda v: v[1].__setitem__("horizon", 99)]:
            v = fixture(); mutate(v); self.parity(v, False)

    def test_full_and_development_wall_boundary(self):
        for dev, value, ok in [(False, 901, True), (False, 3599.9, True), (False, 3600, False), (True, 899.9, True), (True, 900, False)]:
            v = fixture(development=dev); v[0]["seconds"] = value; self.parity(v, ok)

    def test_payloads_are_not_retained_across_ten_thousand_rows(self):
        class Row(dict):
            pass
        refs = []
        closed = []
        def records():
            try:
                for _ in range(10000):
                    # Two generator frames may briefly own adjacent rows,
                    # but neither the checker nor stream retains history.
                    self.assertLessEqual(sum(r() is not None for r in refs[-4:]), 1)
                    row = Row(trace_id=None, payload=bytearray(100000))
                    refs.append(weakref.ref(row))
                    yield row
                    del row
            finally:
                closed.append(True)
        c, t, _, m, totals = fixture(())
        with patch.object(candidate, "load", side_effect=[c, t]), patch.object(candidate, "stream", return_value=records()):
            candidate.check_native_timing(Path("/synthetic"), m, totals)
        self.assertEqual(closed, [True])
        self.assertTrue(all(r() is None for r in refs))

    def test_stream_closed_on_order_failure_and_tail_is_consumed(self):
        for wrong_id in [False, True]:
            seen, closed = [], []
            def records():
                try:
                    seen.append("first")
                    yield {"trace_id": "wrong" if wrong_id else None}
                    seen.append("tail")
                    raise AssertionError("noncanonical tail")
                finally:
                    closed.append(True)
            c, t, _, m, totals = fixture(())
            with patch.object(candidate, "load", side_effect=[c, t]), patch.object(candidate, "stream", return_value=records()):
                with self.assertRaises(AssertionError):
                    candidate.check_native_timing(Path("/synthetic"), m, totals)
            self.assertEqual(seen, ["first"] if wrong_id else ["first", "tail"])
            self.assertEqual(closed, [True])

    def test_real_parser_rejects_noncanonical_trailing_null_row(self):
        c, t, _, m, totals = fixture(())
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            with gzip.open(root / "classifications.jsonl.gz", "wt") as f:
                f.write('{"trace_id":null}\n')
                f.write('{"trace_id": null}\n')  # valid JSON, noncanonical wire
            for module in [reference, candidate]:
                with patch.object(module, "load", side_effect=[c, t]):
                    with self.assertRaisesRegex(AssertionError, "noncanonical JSONL"):
                        module.check_native_timing(root, m, totals)


if __name__ == "__main__":
    unittest.main(verbosity=2)
