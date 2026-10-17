#!/usr/bin/env python3
"""Tamper controls against a native-produced development receipt."""
import copy
import json
import os
from pathlib import Path
import shutil
import tempfile
import unittest

from verify_evolving_minds import verify, ROOT, CONNECTION, GENE, ENERGY, OWNER, OP, MAINT


class CheckerTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.source = Path(os.environ["EVOLVING_MINDS_FIXTURE"])
        cls.clean = verify(cls.source)
        cls.frames = [json.loads(s) for s in (cls.source / "trace.jsonl").read_text().splitlines()]

    def tamper(self, edit):
        with tempfile.TemporaryDirectory() as tmp:
            target = Path(tmp) / "run"
            shutil.copytree(self.source, target)
            edit(target)
            with self.assertRaises((ValueError, KeyError, AssertionError)):
                verify(target)

    def frames_edit(self, edit):
        def mutation(target):
            frames = copy.deepcopy(self.frames)
            edit(frames)
            (target / "trace.jsonl").write_text("".join(json.dumps(f) + "\n" for f in frames))
        self.tamper(mutation)

    def test_clean_native_control(self):
        self.assertTrue(self.clean["audit_pass"])
        self.assertGreater(self.clean["births"], 0)

    def test_missing_phase(self):
        self.frames_edit(lambda fs: fs.pop(2))

    def test_omitted_influx_event(self):
        self.frames_edit(lambda fs: fs[1]["events"].pop())

    def test_duplicate_maintenance(self):
        def edit(fs):
            f = next(f for f in fs if f["phase"] == "Maintenance")
            f["events"].append(copy.deepcopy(f["events"][0]))
        self.frames_edit(edit)

    def test_foreign_owner(self):
        def edit(fs):
            f = next(f for f in fs if any(n["label"] == CONNECTION for _, n in f["nodes"]))
            n = next(n for _, n in f["nodes"] if n["label"] == CONNECTION)
            next(p for p in n["properties"] if p[0] == OWNER)[1]["Int"] = 999
        self.frames_edit(edit)

    def test_free_maintenance(self):
        def edit(fs):
            n = next(n for _, n in fs[5]["nodes"] if n["label"] == ROOT)
            next(p for p in n["properties"] if p[0] == MAINT)[1]["Float"] = 0.0
        self.frames_edit(edit)

    def test_wrong_initial_energy(self):
        def edit(fs):
            n = next(n for _, n in fs[0]["nodes"] if n["label"] == ROOT)
            next(p for p in n["properties"] if p[0] == ENERGY)[1]["Float"] += 1
        self.frames_edit(edit)

    def test_unfaithful_child(self):
        def edit(fs):
            f = next(f for f in fs if any(e["rule"] == "birth_faithful" for e in f["events"]))
            e = next(e for e in f["events"] if e["rule"] == "birth_faithful")
            n = next(n for h, n in f["nodes"] if h in e["created"] and n["label"] == GENE)
            next(p for p in n["properties"] if p[0] == OP)[1]["Int"] = 0
        self.frames_edit(edit)

    def test_changed_physics(self):
        self.tamper(lambda d: (d / "rules.toml").write_text((d / "rules.toml").read_text().replace("value = 2.0", "value = 0.0")))

    def test_false_training_admission(self):
        def edit(d):
            p = d / "run_status.json"; s = json.loads(p.read_text()); s["training_admitted"] = True; p.write_text(json.dumps(s))
        self.tamper(edit)

    def test_wrong_ancestry(self):
        def edit(d):
            p = d / "organisms.json"; s = json.loads(p.read_text()); s["2"]["parent_id"] = 999; p.write_text(json.dumps(s))
        self.tamper(edit)

    def test_checkpoint_corruption(self):
        def edit(d):
            p = d / "final.checkpoint"; b = bytearray(p.read_bytes()); b[-1] ^= 1; p.write_bytes(b)
        self.tamper(edit)

    def test_missing_checkpoint(self):
        self.tamper(lambda d: (d / "midpoint.checkpoint").unlink())

    def test_wrong_native_counter(self):
        self.frames_edit(lambda fs: fs[1]["stats"].update(property_effects_applied=0))

    def test_wrong_csv_ledger(self):
        def edit(d):
            p = d / "metrics.csv"; lines = p.read_text().splitlines(); columns = lines[1].split(",")
            values = lines[2].split(","); values[columns.index("dissipated")] = "0"; lines[2] = ",".join(values)
            p.write_text("\n".join(lines) + "\n")
        self.tamper(edit)


if __name__ == "__main__":
    unittest.main()
