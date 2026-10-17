"""Closure parity must ignore work only and retain trajectory/cap evidence."""
import copy
import unittest
from unittest.mock import patch

import verify_ere_coherent_closure as c


class ClosureTests(unittest.TestCase):
    def test_only_work_accounting_is_excluded(self):
        row = {"kind": "step", "usage": {"work": 400},
               "guards": [["rule", {"enumeration_work": 400, "enumeration_truncated": False,
                                      "property_guards": [[2, 1]]}]],
               "delta": {"typed": {"Float": 1.0}}, "rng_after": "abc", "ceiling": False,
               "native": {"events": [{"matched_edges": [3]}]}}
        before = copy.deepcopy(row)
        other = copy.deepcopy(row)
        other["usage"]["work"] = 9
        other["guards"][0][1]["enumeration_work"] = 9
        self.assertEqual(c.trajectory_material(row), c.trajectory_material(other))
        self.assertEqual(row, before)
        for change in [lambda r: r.__setitem__("ceiling", True),
                       lambda r: r.__setitem__("rng_after", "changed"),
                       lambda r: r["delta"]["typed"].__setitem__("Float", 2.0),
                       lambda r: r["guards"][0][1].__setitem__("enumeration_truncated", True),
                       lambda r: r["guards"][0][1].__setitem__("property_guards", [[2, 0]]),
                       lambda r: r["native"]["events"][0].__setitem__("matched_edges", [4])]:
            changed = copy.deepcopy(row)
            change(changed)
            self.assertNotEqual(c.trajectory_material(row), c.trajectory_material(changed))

    def test_work_model_and_prerequisites_are_required(self):
        m = {"matcher_work_model": c.WORK_MODEL, "closure_prerequisites": {
            "targeted_replay_pass": True, "corrected_preserved_audit_pass": True,
            "targeted_trace_audit_pass": True}}
        d = {"closure_campaign": {"matcher_work_model": c.WORK_MODEL}}
        with patch.object(c, "original_manifest"):
            c.check_manifest(m, d, False)
            for field in m["closure_prerequisites"]:
                changed = copy.deepcopy(m)
                changed["closure_prerequisites"][field] = False
                with self.assertRaises(AssertionError):
                    c.check_manifest(changed, d, False)
            changed = copy.deepcopy(m)
            changed["matcher_work_model"] = "legacy-dfs-v1"
            with self.assertRaises(AssertionError):
                c.check_manifest(changed, d, False)


if __name__ == "__main__":
    unittest.main()
