#!/usr/bin/env python3
"""Small checker regression suite. Uses an existing development archive read-only.

Run python3 -B scripts/test_verify_ere_complete_long_gates.py ARCHIVE.
No experiment is launched and no existing receipt/source is rewritten.
"""
import copy
import gzip
import json
import shutil
from pathlib import Path
import sys
import tempfile
import time
import unittest

import verify_ere_complete_long_gates as a

ARCHIVE = Path(sys.argv.pop(1)).resolve()


class CheckerControls(unittest.TestCase):
    def setUp(self):
        a.DEADLINE = time.monotonic() + 60
        self.manifest = a.load(ARCHIVE / "manifest.json")

    def test_development_cannot_pass_scientific_horizon(self):
        with self.assertRaises(AssertionError):
            a.check_matrix(self.manifest, False)
        for horizon in [8, 16, 64, 9999, 99999]:
            m = copy.deepcopy(self.manifest)
            m["development_only"] = False
            for s in m["cells"]:
                s["cell"]["development_only"] = False
                s["cell"]["horizon"] = horizon
            with self.assertRaises(AssertionError):
                a.check_matrix(m, False)

    def test_missing_duplicate_seed_fixture_budget_and_schedule(self):
        mutations = [lambda m: m["cells"].pop(),
                     lambda m: m["cells"].__setitem__(1, copy.deepcopy(m["cells"][0])),
                     lambda m: m["cells"][0]["cell"].__setitem__("seed", 43),
                     lambda m: m["cells"][0]["cell"].__setitem__("identity", False),
                     lambda m: m["cells"][4]["cell"].__setitem__("payload_steps", [9, 16, 24, 56]),
                     lambda m: m["cells"][4]["config"].__setitem__("match_ceiling", 50001),
                     lambda m: m["resources"].__setitem__("assay_output_cap_bytes", 2147483649),
                     lambda m: m.__setitem__("training_admitted", True)]
        for mutate in mutations:
            m = copy.deepcopy(self.manifest)
            mutate(m)
            with self.assertRaises(AssertionError):
                a.check_matrix(m, True)

    def row_context(self, target_id, step):
        setup = copy.deepcopy(next(s for s in self.manifest["cells"] if s["cell"]["id"] == target_id))
        c, initial = setup["cell"], setup["initial"]
        nodes = {k: n["data"] for k, n in initial["nodes"]}
        edges = dict(initial["edges"])
        setup["_family_members"] = {k for k, n in nodes.items() if n["label"] >= a.BASE} - {a.node_key(o["id"]) for o in setup["instructions"]}
        setup["_mutable_nodes"] = {a.node_key(o["id"]) for o in setup["instructions"]} | {a.node_key(setup["clock"])}
        setup["_checkpoints"] = a.schedule(c, c["horizon"])
        table = setup["initial_table"]
        previous = {"step": 0, "rng_after": setup["initial_rng"], "motifs": [{"phase": "path"}] * 4, "family_valid": True}
        for row in a.stream(ARCHIVE / target_id / "steps.jsonl.gz"):
            if row["table"] is not None:
                table = row["table"]
            setup["_table_names"] = {r["name"]: i for i, r in enumerate(table)}
            if row["step"] == step:
                return row, setup, table, nodes, edges, previous
            nodes, edges, _ = a.check_row(row, setup, table, nodes, edges, previous)
            previous = row
        self.fail("missing target boundary")

    def rejects(self, context, mutate):
        row, setup, table, nodes, edges, previous = copy.deepcopy(context)
        mutate(row, table)
        with self.assertRaises((AssertionError, KeyError, IndexError, StopIteration)):
            a.check_row(row, setup, table, nodes, edges, previous)

    def test_paid_target_and_ownership_not_whole_family_recovery(self):
        context = self.row_context("A3_fuel8_ownertrue", 57)
        self.assertFalse(context[0]["family_valid"])
        self.assertEqual(context[0]["world_classes"], 0)
        self.assertTrue(any("RepairSealed" in e["rule_name"] for e in context[0]["native"]["events"]))
        self.rejects(context, lambda r, _: r.__setitem__("family_valid", True))
        self.rejects(context, lambda r, _: r.__setitem__("world_classes", 2))
        self.rejects(context, lambda r, _: r["native"]["events"].__setitem__(0, {}))
        self.rejects(context, lambda r, _: next(o for o in r["instructions"] if o["operation"] == "RepairSealed").__setitem__("fuel", 8.0))
        seal = self.row_context("A2_fuel8_ownerfalse", 1)
        self.rejects(seal, lambda r, _: r["native"]["events"].pop(0))

    def test_protection_visibility_and_binding_probe(self):
        context = self.row_context("A2_fuel8_ownerfalse", 8)
        self.assertEqual(len(context[0]["binding_probes"]), 1)
        self.rejects(context, lambda r, _: r.__setitem__("binding_probes", []))
        self.rejects(context, lambda r, _: r["binding_probes"][0].__setitem__("visible_bindings", []))
        self.rejects(context, lambda r, _: r["binding_probes"][0].__setitem__("read_only", False))
        table = copy.deepcopy(context[2])
        next(r for r in table if r["name"].startswith("environment_payload"))["meta_visible"] = True
        with self.assertRaises(AssertionError):
            a.check_table(table, context[1]["cell"], context[1]["program"])

    def test_shifted_hazard_and_root_cleanup_counter(self):
        context = self.row_context("anchor_loss_A3", 60)
        self.assertFalse(context[0]["anchor_alive"])
        self.rejects(context, lambda r, _: r.__setitem__("anchor_alive", True))
        self.rejects(context, lambda r, _: r["delta"]["nodes"].__setitem__("deleted", []))
        self.rejects(context, lambda r, _: r["native"]["events"][0].__setitem__("cleanup_neighbors", []))
        next_step = self.row_context("anchor_loss_A3", 61)
        self.assertEqual(next_step[0]["compile_report"]["quarantined"], 0)
        self.rejects(next_step, lambda r, _: r["compile_report"].__setitem__("quarantined", 1))
        hazard = self.row_context("A3_fuel8_ownerfalse", 8)
        self.rejects(hazard, lambda r, _: next(e for e in r["native"]["events"] if e["rule_name"].startswith("environment_payload")).__setitem__("rule_name", "environment_payload_9"))

    def test_canonical_stream_and_truncation(self):
        with tempfile.TemporaryDirectory() as tmp:
            p = Path(tmp) / "rows.gz"
            for content in ['{"b":1,"a":2}\n', '{"a":2}\r\n', '{"a":2}', '{bad}\n']:
                with gzip.open(p, "wt") as f:
                    f.write(content)
                with self.assertRaises((AssertionError, ValueError)):
                    list(a.stream(p))
        row = self.row_context("A2_fuel8_ownerfalse", 8)[0]
        self.assertEqual(len(a.csv_row(row).strip().split(",")), 18)

    def test_exact_inventory_bytecode_and_checksum_failure(self):
        with tempfile.TemporaryDirectory() as tmp:
            out = Path(tmp)
            for name in ["manifest.json", "executed_test_binary"]:
                (out / name).write_text("{}")
            (out / "frozen_inputs_sha256.json").write_text(json.dumps({"manifest.json": "0" * 64}))
            with self.assertRaises(AssertionError):
                a.integrity(out)
        self.assertFalse(any(ARCHIVE.rglob("__pycache__")))

    def test_short_trajectory_and_csv_corruption(self):
        setup = next(s for s in self.manifest["cells"] if s["cell"]["id"] == "identity_dormant_A0")
        with tempfile.TemporaryDirectory() as tmp:
            out = Path(tmp)
            cell = out / setup["cell"]["id"]
            shutil.copytree(ARCHIVE / cell.name, cell)
            raw = cell / "steps.jsonl.gz"
            with gzip.open(raw, "rt") as f:
                lines = f.readlines()
            with gzip.open(raw, "wt", newline="\n") as f:
                f.writelines(lines[:-1])
            with self.assertRaises(AssertionError):
                a.audit_cell(out, copy.deepcopy(setup))
            shutil.copyfile(ARCHIVE / cell.name / "steps.jsonl.gz", raw)
            csv = cell / "trajectory.csv"
            contents = csv.read_text()
            csv.write_text(contents.replace(a.HEADER, a.HEADER.replace("rng_before", "wrong_rng_header"), 1))
            with self.assertRaises(AssertionError):
                a.audit_cell(out, copy.deepcopy(setup))
            csv.write_text(contents.replace("false", "true", 1))
            with self.assertRaises(AssertionError):
                a.audit_cell(out, copy.deepcopy(setup))


if __name__ == "__main__":
    unittest.main()
