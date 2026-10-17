#!/usr/bin/env python3
"""Tamper regressions against a persisted 100-step identity diagnostic."""
import copy
import gzip
import json
from pathlib import Path
import sys
import unittest
from verify_ere_identity_isolation import audit


class AuditorTests(unittest.TestCase):
    def test_original(self):
        self.assertTrue(audit(self.receipt)["audit_pass"])

    def rejected(self, change):
        receipt = copy.deepcopy(self.receipt)
        change(receipt)
        with self.assertRaises((ValueError, KeyError, TypeError)):
            audit(receipt)

    def test_handle_tamper(self):
        self.rejected(lambda r: self.change_world(r, lambda w: w["nodes"][0].__setitem__(0, "NodeId(2v1)")))

    def test_endpoint_tamper(self):
        self.rejected(lambda r: self.change_world(r, lambda w: w["edges"][0][1]["nodes"].reverse()))

    def change_world(self, receipt, change):
        frame = receipt["cases"][1]["child_frames"][1]
        world = copy.deepcopy(receipt["world_table"][frame["world_id"]])
        change(world)
        frame["world_id"] = len(receipt["world_table"])
        receipt["world_table"].append(world)

    def test_rng_tamper(self):
        self.rejected(lambda r: r["cases"][1]["child_frames"][1].__setitem__("rng", "0000000000000000"))

    def test_transition_tamper(self):
        self.rejected(lambda r: r["cases"][1]["child_frames"][1]["events"][0].__setitem__("rule_name", "forged"))

    def test_horizon_truncation(self):
        self.rejected(lambda r: r["cases"][1]["child_frames"].pop())

    def test_hide_legacy_failure(self):
        self.rejected(lambda r: r["cases"][0].__setitem__("child_frames", copy.deepcopy(r["cases"][0]["parent_frames"])))

    def test_promote_to_admission(self):
        self.rejected(lambda r: r.__setitem__("training_admitted", True))

    def test_binding_budget(self):
        self.rejected(lambda r: r["cases"][1]["child_frames"][1]["usage"].__setitem__("work", 400001))

    def test_erase_negative_control(self):
        self.rejected(lambda r: r["negative_control"].__setitem__("child_world", r["negative_control"]["parent_world"]))


if __name__ == "__main__":
    with gzip.open(Path(sys.argv.pop(1)) / "receipt.json.gz", "rt") as stream:
        AuditorTests.receipt = json.load(stream)
    unittest.main()
