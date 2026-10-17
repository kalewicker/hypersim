#!/usr/bin/env python3
"""Independent raw timeline/frame audit; Rust verifies full binary-state decoding."""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import struct
from verify_ere_complete_witness import audit_inventory, digest, write_checksums

CONTRACT_SHA = "3f2c17afb2489e200daff9433e7d1b5da0b005af366ea2a6c8c0d2d2af875b2f"
CUTS = [0, 7, 8, 9, 10, 24, 25, 33, 40, 41, 64]

def fx64(data):
    """Rust1 FxHasher, 64-bit little-endian slice Hash (length then bytes)."""
    mask = (1 << 64) - 1
    h = 0
    def add(word):
        nonlocal h
        h = ((((h << 5) | (h >> 59)) & mask) ^ word) * 0x517cc1b727220a95 & mask
    add(len(data))
    offset = 0
    for size in [8, 4, 2, 1]:
        while len(data) - offset >= size:
            add(int.from_bytes(data[offset:offset + size], "little"))
            offset += size
    return h

def frame(data):
    assert len(data) >= 24 and data[:8] == b"ERECP001"
    length, checksum = struct.unpack("<QQ", data[8:24])
    assert 0 < length <= 32 * 1024 * 1024 and length == len(data) - 24
    assert checksum == fx64(data[24:])

def audit_cell(r, index, directory=None, reference=None):
    if index < 12:
        expected = (index < 6, [0, 2, 8][(index % 6) // 2], bool(index % 2), False)
    else:
        expected = (True, 8, False, True)
    protected, fuel, ownership, random = expected
    assert (r["protected"], r["fuel"], r["ownership"], r["random"]) == expected
    assert r["pass"] is True and r["expected_outcomes"] is True and r["training_admitted"] is False
    cfg = r["effective_config"]
    assert cfg["seed"] == 984927043621 and cfg["engine"]["max_steps"] == 64
    assert cfg["engine"]["step_mode"] == ("SequentialRandom" if random else "MaxParallel")
    interval = 16 if random else 1
    assert cfg["engine"]["metrics_window_interval"] == interval and cfg["engine"]["thread_count"] == 1
    assert cfg["engine"]["max_pairings_per_step"] == 16
    ec = cfg["ere"]
    assert ec["arm"] == ("A2" if protected else "A3") and ec["epoch"] == {"Fixed": 1}
    assert ec["match_budget_policy"] == "Pooled" and ec["match_ceiling"] == 50000
    assert ec["vocabulary"] == {"protocol": "E2-witness-complete-v1", "instructions": [
        {"operation": "Seal", "fuel": 1.0}, {"operation": "RepairSealed", "fuel": fuel}]}
    assert ec["measure_family_maintenance"] is True and ec["measure_ei"] is False
    assert ec["meta_seed_energy"] == 0 and ec["meta_rules_toml"] is None
    assert cfg["evaluator"]["enabled"] is False and cfg["causal"]["enabled"] is False
    baseline = r["baseline"]
    assert len(baseline) == 64 and [s["step"] for s in baseline] == list(range(1, 65))
    audit_inventory({"inventory": r["initial_inventory"]})
    for s in baseline:
        assert s["inventory"]["step"] == s["native"]["step"] == s["observation"]["step"] == s["step"]
        audit_inventory({"inventory": s["inventory"]})
        o = s["observation"]
        assert o["rng_before_compile"] == o["rng_after_compile"]
        assert o["match_ceiling_binding"] is False and o["match_work"] <= 50000 and o["match_results"] <= 50000
        assert s["native"]["stats"]["accepted_rewrites"] > 0
        assert (s["window"] is not None) == (s["step"] % interval == 0)
    for a, b in zip(baseline, baseline[1:]):
        assert a["observation"]["rng_after_step"] == b["observation"]["rng_before_compile"]
        assert a["observation"]["world_hash_after"] == b["observation"]["world_hash_before"]
    consumed = baseline[0]["observation"]["rng_before_compile"] != baseline[-1]["observation"]["rng_after_step"]
    assert r["rng_consumed"] == consumed and consumed == random
    if not random:
        exact = 64 if protected else 7 if fuel == 0 else 21 if fuel == 2 else 35 if ownership else 60
        repairs = 0 if protected else min(fuel, 4)
        assert sum(s["native"]["exact_original"] for s in baseline) == exact
        assert sum(t["operation"] == "RepairSealed" for s in baseline
                   for t in (s["window"]["encoded_meta_operation_traces"] or [])) == repairs
        assert baseline[-1]["native"]["fuel"] == fuel - repairs
        if reference is not None:
            assert [s["native"] for s in baseline] == reference["native_steps"]
            assert [s["observation"] for s in baseline] == reference["step_observations"]
    cuts = [7, 8, 9] if random else CUTS
    forks = r["forks"]
    assert len(forks) == len(cuts) and [f["cut"] for f in forks] == cuts
    for f in forks:
        assert f["pass"] is True and f["boundary_equal"] is True and f["terminal_equal"] is True
        assert f["suffix_steps"] == 64 - f["cut"] == len(f["suffix"])
        assert f["suffix"] == baseline[f["cut"]:]
        assert f["terminal_window"] == r["terminal_window"] and f["terminal_guards"] == r["terminal_guards"]
        if directory is not None:
            data = (directory / f'checkpoint_{f["cut"]:02}.bin').read_bytes()
            assert len(data) == f["checkpoint_bytes"]
            frame(data)
    return sum(f["suffix_steps"] for f in forks)

def audit(out, reference):
    assert digest(out / "preregistration.md") == CONTRACT_SHA
    status = json.loads((out / "status.json").read_text())
    assert status == {"status": "complete", "pass": True, "training_admitted": False}
    assert sorted(p.name for p in out.glob("cell_*")) == [f"cell_{i:02}" for i in range(13)]
    total = 0
    for index in range(13):
        directory = out / f"cell_{index:02}"
        r = json.loads((directory / "receipt.json").read_text())
        old = json.loads((reference / f"attempt_{index:03}" / "receipt.json").read_text()) if index < 12 else None
        total += audit_cell(r, index, directory, old)
    summary = json.loads((out / "summary.json").read_text())
    assert summary == {"profile": "E2-complete-checkpoint-continuation-v1", "cells": 13, "forks": 135,
                       "baseline_steps": 832, "suffix_steps": 5484, "cross_process_resumes": 2,
                       "cross_process_steps": 112, "total_steps": 6428, "pass": True, "training_admitted": False}
    assert total == 5484
    processes = json.loads((out / "cross_process.json").read_text())
    assert len(processes) == 2 and [p["cell"] for p in processes] == [11, 12]
    for p in processes:
        assert p["cut"] == 8 and p["suffix_steps"] == 56 and p["exit_code"] == 0 and p["pass"] is True
        directory = out / f'cell_{p["cell"]:02}'
        r = json.loads((directory / "receipt.json").read_text())
        child = json.loads((directory / "child_resume.json").read_text())
        assert child["process_id"] != p["parent_pid"] and child["process_id"] > 0
        assert child["pass"] is True and child["training_admitted"] is False and child["cut"] == 8
        assert child["suffix_steps"] == 56 and child["suffix"] == r["baseline"][8:]
        assert child["terminal_window"] == r["terminal_window"] and child["terminal_guards"] == r["terminal_guards"]
    return {"audit_pass": True, "cells_verified": 13, "checkpoints_verified": 135,
            "baseline_steps": 832, "suffix_steps": total, "total_steps": 6428,
            "cross_process_resumes": 2, "cross_process_steps": 112,
            "prior_label_baselines_identical": 12, "rng_consuming_midwindow_control": True,
            "contract_sha256": CONTRACT_SHA, "training_admitted": False}

def rejections(out):
    original = json.loads((out / "cell_00" / "receipt.json").read_text())
    changes = {
        "short_baseline": lambda r: r["baseline"].pop(),
        "missing_fork": lambda r: r["forks"].pop(),
        "wrong_cut": lambda r: r["forks"][0].update(cut=1),
        "short_suffix": lambda r: r["forks"][1]["suffix"].pop(),
        "changed_event": lambda r: r["forks"][1]["suffix"][0]["native"]["events"].pop(),
        "changed_rng": lambda r: r["forks"][1]["suffix"][0]["observation"].update(rng_after_step="bad"),
        "changed_fuel": lambda r: r["forks"][1]["suffix"][0]["native"].update(fuel=100),
        "changed_ledger": lambda r: r["forks"][1]["terminal_window"].update(rule_activations=99),
        "wrong_policy": lambda r: r["effective_config"]["ere"].update(match_budget_policy="Split"),
        "false_boundary": lambda r: r["forks"][0].update(boundary_equal=False),
        "false_ceiling": lambda r: r["baseline"][0]["observation"].update(match_ceiling_binding=True),
    }
    rejected = []
    for name, change in changes.items():
        r = copy.deepcopy(original)
        change(r)
        try:
            audit_cell(r, 0)
        except (AssertionError, KeyError, IndexError):
            rejected.append(name)
        else:
            raise AssertionError(f"accepted tampering: {name}")
    data = (out / "cell_00" / "checkpoint_00.bin").read_bytes()
    for name, bad in [("truncated_frame", data[:-1]), ("trailing_frame", data + b"x"),
                      ("wrong_version", b"ERECP002" + data[8:]),
                      ("corrupt_payload", data[:-1] + bytes([data[-1] ^ 1]))]:
        try:
            frame(bad)
        except AssertionError:
            rejected.append(name)
        else:
            raise AssertionError(f"accepted frame tampering: {name}")
    return rejected

if __name__ == "__main__":
    if not __debug__:
        raise SystemExit("Audit assertions require Python without -O")
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path)
    parser.add_argument("--reference", type=Path, required=True)
    parser.add_argument("--test-rejections", action="store_true")
    parser.add_argument("--write-checksums", action="store_true")
    args = parser.parse_args()
    result = audit(args.output, args.reference)
    if args.test_rejections:
        result["tamper_rejections"] = rejections(args.output)
    (args.output / "receipt_verification.json").write_text(json.dumps(result, indent=2) + "\n")
    if args.write_checksums:
        result["checksummed_files"] = write_checksums(args.output)
    print(json.dumps(result, indent=2))
