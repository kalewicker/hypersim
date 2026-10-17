#!/usr/bin/env python3
"""Audit frozen engineering receipts and HGSS frames, not general repair efficacy.

The Rust harness verifies full tagged-bincode store round trips. This independent
auditor checks matrix completeness, native graph deltas, finite payments,
quarantine timelines, frame headers and artifact checksums. It does not decode
bincode independently or silently treat missing checkpoints as a negative result.
"""
import argparse
import copy
from collections import Counter
import hashlib
import itertools
import json
from pathlib import Path
import struct
from unittest.mock import patch

SEEDS = range(984927043585, 984927043589)
CONDITIONS = [
    "no_attack", "binary_erase", "binary_delete", "binary_rewire",
    "witness_erase", "witness_delete", "witness_rewire", "sealed_erase",
    "sealed_delete", "sealed_rewire", "duplicate_payload", "duplicate_witness",
]
CONTRACT_SHA256 = "7b89b98b4a00712218bd08680bd81493344f1dbeee45fe6f3b4825054425579a"


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def read_json(path):
    return json.loads(path.read_text())


def edge_counts(checkpoint):
    return Counter(json.dumps(edge, sort_keys=True) for _, edge in checkpoint["inventory"]["edges"])


def audit(out):
    manifest = read_json(out / "manifest.json")
    assert manifest["protocol"] == "E2-witness-adversarial-tests-v1"
    assert manifest["repair_protocol"] == "E2-witness-repair-v2"
    assert manifest["seeds"] == list(SEEDS) and manifest["conditions"] == CONDITIONS
    assert manifest["repair_fuels"] == [0, 1] and manifest["horizon"] == 8
    assert manifest["expected_cases"] == 1248 and manifest["training_admitted"] is False
    assert manifest["match_work_limit"] == manifest["match_result_limit"] == 50000
    assert digest(out / "preregistration.md") == CONTRACT_SHA256
    assert len(manifest["roles"]) == len(set(manifest["roles"])) == 13
    expected_names = {
        f"{seed}_role{role:02}_{condition}_fuel{fuel}"
        for seed, role, condition, fuel in itertools.product(SEEDS, range(13), CONDITIONS, [0, 1])
    }
    receipts = sorted(out.glob("*_role*/receipt.json"))
    assert {path.parent.name for path in receipts} == expected_names
    counts, valid_counts, paid_counts = Counter(), Counter(), Counter()
    total_seal = 0
    max_work = max_results = 0
    checkpoints = 0
    for path in receipts:
        r = read_json(path)
        condition, initial_fuel = r["condition"], r["initial_fuel"]
        assert r["case"] == path.parent.name
        assert r["case"] == f'{r["seed"]}_role{r["role_index"]:02}_{condition}_fuel{initial_fuel}'
        assert r["role"] == manifest["roles"][r["role_index"]]
        recovered = condition == "binary_erase" and initial_fuel == 1
        valid = condition == "no_attack" or recovered
        assert r["attack_accepted"] == 1 and r["attack_valid"] == (condition == "no_attack")
        assert r["final_valid"] == r["expected_final_valid"] == r["exact_original"] == valid
        assert r["repair_paid"] == int(recovered)
        assert r["final_fuel"] == initial_fuel - int(recovered)
        assert r["seal_paid"] == int(condition.startswith("sealed"))
        assert 0 <= r["attack_work"] <= 50000 and 0 <= r["attack_results"] <= 50000
        assert len(r["observations"]) == 8
        for index, obs in enumerate(r["observations"]):
            assert obs["step"] == index + 2 and obs["valid"] == valid
            assert obs["ceiling"] is False and obs["rng_probe"] == r["repair_rng_probe"]
            assert obs["fuel"] == r["final_fuel"]
            assert obs["accepted_rewrites"] == obs["paid"] == int(recovered and index == 0)
            compile_valid = condition == "no_attack" or (recovered and index > 0)
            assert obs["world_classes"] == int(compile_valid)
            assert obs["quarantined"] == int(not compile_valid)
            assert 0 <= obs["work"] <= 50000 and 0 <= obs["results"] <= 50000
            max_work = max(max_work, obs["work"])
            max_results = max(max_results, obs["results"])
        assert [c["phase"] for c in r["checkpoints"]] == ["before_attack", "after_attack", "after_repair"]
        for c, step in zip(r["checkpoints"], [0, 1, 9]):
            assert c["roundtrip_verified"] is True and c["step"] == c["inventory"]["step"] == step
            assert c["root"] == r["checkpoints"][0]["root"]
            assert c["instruction"] == r["checkpoints"][0]["instruction"]
            snapshot_path = path.parent / c["path"]
            assert snapshot_path.resolve().is_relative_to(path.parent.resolve())
            frame = snapshot_path.read_bytes()
            magic, version, wire_step, length = struct.unpack("<4sHQQ", frame[:22])
            assert magic == b"HGSS" and version == 2 and wire_step == step
            assert len(frame) == length + 22 and length == c["bytes"]
            checkpoints += 1
        before, attacked, final = r["checkpoints"]
        assert before["fuel"] == attacked["fuel"] == initial_fuel
        assert final["fuel"] == r["final_fuel"]
        b, a, f = map(edge_counts, [before, attacked, final])
        assert [(key, node["data"]) for key, node in before["inventory"]["nodes"]] == [
            (key, node["data"]) for key, node in attacked["inventory"]["nodes"]
        ]
        removed, added = list((b - a).elements()), list((a - b).elements())
        if condition == "no_attack":
            assert b == a == f
        elif condition.startswith("duplicate"):
            assert not removed and len(added) == 1
            extra = json.loads(added[0])
            assert extra["label"] == (0xFFF00100 + r["role_index"] if condition == "duplicate_witness" else r["role"])
            assert len(extra["nodes"]) == (3 if condition == "duplicate_witness" else 2)
        else:
            assert len(removed) == 1
            assert len(added) == int(not condition.endswith("delete"))
            old = json.loads(removed[0])
            assert old["label"] == (0xFFF00100 + r["role_index"] if condition.startswith("witness") else r["role"])
            assert len(old["nodes"]) == (3 if condition.startswith(("witness", "sealed")) else 2)
            if added:
                new = json.loads(added[0])
                if condition.endswith("erase"):
                    assert new["label"] == 0 and new["nodes"] == old["nodes"]
                elif condition.endswith("rewire"):
                    endpoints = old["nodes"].copy()
                    offset = int(condition.startswith("witness"))
                    endpoints[offset], endpoints[offset + 1] = endpoints[offset + 1], endpoints[offset]
                    assert new["label"] == old["label"] and new["nodes"] == endpoints
        if recovered:
            removed, added = list((a - f).elements()), list((f - a).elements())
            assert len(removed) == len(added) == 1
            old, new = json.loads(removed[0]), json.loads(added[0])
            assert old["label"] == 0 and len(old["nodes"]) == 2
            assert new["label"] == r["role"] and new["nodes"] == old["nodes"] + [before["root"]]
        else:
            assert a == f
        final_nodes = copy.deepcopy(final["inventory"]["nodes"])
        if recovered:
            instruction = before["instruction"]
            instruction_key = f'NodeId({instruction["idx"]}v{instruction["version"]})'
            data = dict(final_nodes)[instruction_key]["data"]
            assert [1, {"Float": 0.0}] in data["properties"]
            data["properties"][data["properties"].index([1, {"Float": 0.0}])] = [1, {"Float": 1.0}]
        assert [(key, node["data"]) for key, node in final_nodes] == [
            (key, node["data"]) for key, node in attacked["inventory"]["nodes"]
        ]
        counts[condition] += 1
        valid_counts[condition] += int(valid)
        paid_counts[condition] += r["repair_paid"]
        total_seal += r["seal_paid"]
    summary = read_json(out / "summary.json")
    assert summary["completed_cases"] == len(receipts) == 1248
    assert summary["checkpoints_verified"] == checkpoints == 3744
    assert summary["engineering_pass"] is True
    assert summary["general_repair_claim"] is summary["training_admitted"] is False
    assert len(summary["cases"]) == len(receipts)
    indexed = {item["case"]: item for item in summary["cases"]}
    assert set(indexed) == expected_names
    for path in receipts:
        r = read_json(path)
        assert indexed[r["case"]] == {k: r[k] for k in ["case", "condition", "final_valid", "repair_paid"]}
    return {
        "audit_pass": True, "completed_cases": len(receipts), "checkpoint_frames_verified": checkpoints,
        "full_store_roundtrips_verified_by_rust_harness": checkpoints,
        "condition_counts": dict(counts), "valid_final_counts": dict(valid_counts),
        "repair_payments": dict(paid_counts), "seal_payments": total_seal,
        "completed_engine_steps": len(receipts) * 9, "max_repair_match_work": max_work,
        "max_repair_match_results": max_results, "contract_sha256": CONTRACT_SHA256,
        "test_source_sha256": digest(out / "test_source.rs"),
        "executed_test_binary_sha256": digest(out / "executed_test_binary"),
        "unsupported_damage_recovered": False, "training_admitted": False,
    }


def rejection_controls(out):
    """Change in-memory copies only; never damage archived receipts/checkpoints."""
    target = out / "984927043585_role00_binary_erase_fuel1" / "receipt.json"
    original = read_json(target)
    mutations = {
        "unpaid_recovery": lambda r: r.update(repair_paid=0),
        "fuel_replenishment": lambda r: r.update(final_fuel=1),
        "missing_observation": lambda r: r["observations"].pop(),
        "invisible_ceiling": lambda r: r["observations"][0].update(ceiling=True),
        "cached_world_fallback": lambda r: r["observations"][0].update(world_classes=1),
        "invented_payment": lambda r: r["observations"][1].update(paid=1, accepted_rewrites=1),
        "rng_drift": lambda r: r["observations"][0].update(rng_probe="bad"),
        "false_checkpoint_step": lambda r: r["checkpoints"][0].update(step=999),
        "nonoriginal_recovery": lambda r: r.update(exact_original=False),
        "invented_endpoint_repair": lambda r: r["checkpoints"][2]["inventory"]["edges"][0][1]["nodes"].reverse(),
    }
    real_reader = read_json
    rejected = []
    for name, change in mutations.items():
        mutant = copy.deepcopy(original)
        change(mutant)

        def injected(path):
            return mutant if path == target else real_reader(path)

        with patch.dict(globals(), read_json=injected):
            try:
                audit(out)
            except AssertionError:
                rejected.append(name)
            else:
                raise AssertionError(f"auditor accepted tampering: {name}")
    return rejected


if __name__ == "__main__":
    if not __debug__:
        raise SystemExit("Audit assertions must be enabled; run Python without -O.")
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path)
    parser.add_argument("--test-rejections", action="store_true")
    args = parser.parse_args()
    result = audit(args.output)
    if args.test_rejections:
        result["rejected_in_memory_tampering_controls"] = rejection_controls(args.output)
    (args.output / "receipt_verification.json").write_text(json.dumps(result, indent=2) + "\n")
    files = sorted(path for path in args.output.rglob("*") if path.is_file() and path.name != "checksums.sha256")
    (args.output / "checksums.sha256").write_text("".join(f"{digest(path)}  {path}\n" for path in files))
    print(json.dumps(result, indent=2))
