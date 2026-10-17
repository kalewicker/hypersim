#!/usr/bin/env python3
"""Audit sealed-repair receipts/graph deltas/HGSS frames; bincode is checked in Rust."""
import argparse
from collections import Counter
import copy
import hashlib
import itertools
import json
from pathlib import Path
import struct
from unittest.mock import patch

CONTRACT_SHA = "0becb0794c3c3208f402254943eeea2ca39fe995cf64915ed5c9239a2d64da3c"
SEEDS = range(984927043585, 984927043589)
CONDITIONS = [
    "intact_paid", "erased_paid", "erased_zero", "erased_binary_only", "payload_deleted",
    "payload_reversed", "payload_erased_reversed", "payload_foreign_erased", "witness_erased",
    "witness_deleted", "both_erased", "witness_reversed_payload_erased", "duplicate_payload", "duplicate_witness",
]
ROLES = [0xFFF00042, 0xFFF00043, 0xFFF00041, *range(0xFFF00050, 0xFFF00058), 0xFFF00070, 0xFFF00071]


def read_json(path):
    return json.loads(path.read_text())


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def edges(c):
    return Counter(json.dumps(e, sort_keys=True) for _, e in c["inventory"]["edges"])


def node_data(c):
    return {key: row["data"] for key, row in c["inventory"]["nodes"]}


def audit_inventory(c):
    nodes = dict(c["inventory"]["nodes"])
    assert len(nodes) == len(c["inventory"]["nodes"])
    expected_incident = {key: Counter() for key in nodes}
    expected_labels = {}
    seen_edges = set()
    for key, edge in c["inventory"]["edges"]:
        assert key not in seen_edges
        seen_edges.add(key)
        expected_labels.setdefault(edge["label"], []).append(key)
        for endpoint in edge["nodes"]:
            node_key = f'NodeId({endpoint["idx"]}v{endpoint["version"]})'
            assert node_key in nodes
            expected_incident[node_key][key] += 1
    for key, row in nodes.items():
        assert Counter(row["incident"]) == expected_incident[key]
    indexes = c["inventory"]["label_indexes"]
    assert len(indexes) == len(expected_labels)
    assert {label: sorted(ids) for label, ids in indexes} == {
        label: sorted(ids) for label, ids in expected_labels.items()
    }


def audit(out):
    m = read_json(out / "manifest.json")
    assert m["diagnostic"] == "E2-sealed-repair-controls-v3" and m["protocol"] == "E2-witness-sealed-repair-v1"
    assert (m["opcode"], m["instruction_version"], m["family_version"]) == (6, 9, 7)
    assert m["seeds"] == list(SEEDS) and m["conditions"] == CONDITIONS and m["roles"] == ROLES
    assert m["cases"] == 728 and m["horizon"] == 8 and m["max_pairings"] == 16
    assert m["work_limit"] == m["result_limit"] == 50000 and m["training_admitted"] is False
    assert digest(out / "preregistration.md") == CONTRACT_SHA
    expected = {f"{seed}_role{ix:02}_{condition}" for seed, ix, condition in itertools.product(SEEDS, range(13), CONDITIONS)}
    paths = sorted(out.glob("*_role*/receipt.json"))
    assert {p.parent.name for p in paths} == expected
    counts, valid_counts, payments, exact_counts, changed_roles = (Counter() for _ in range(5))
    maximum_work = maximum_results = checkpoint_count = 0
    summaries = {}
    for path in paths:
        r = read_json(path)
        condition = r["condition"]
        assert r["case"] == path.parent.name == f'{r["seed"]}_role{r["role_index"]:02}_{condition}'
        assert r["role"] == ROLES[r["role_index"]] and r["seed"] in SEEDS
        recovered = condition == "erased_paid"
        composite = condition == "both_erased"
        exact = recovered or condition == "intact_paid"
        before, attacked, final = r["checkpoints"]
        # Independently compare the recorded full decoder results, not only
        # boolean validity. Rust also checks these against each restored store.
        assert before["decoded"].startswith("Ok(")
        attack_valid = attacked["decoded"].startswith("Ok(")
        valid = exact or (composite and attack_valid)
        assert final["decoded"].startswith("Ok(") == valid
        assert (final["decoded"] == before["decoded"]) == exact
        if composite:
            assert final["decoded"] == attacked["decoded"] != before["decoded"]
        elif not recovered:
            assert final["decoded"] == attacked["decoded"]
        initial = int(condition not in ["erased_zero", "erased_binary_only"])
        binary = int(condition == "erased_binary_only")
        assert r["initial_fuel"] == initial and r["binary_initial_fuel"] == binary
        assert r["seal_paid"] == 1 and r["attack"]["accepted"] == 1 and r["attack"]["rng_unchanged"] is True
        assert r["attack_valid"] == attack_valid
        if not composite:
            assert attack_valid == (condition == "intact_paid")
        assert r["final_valid"] == valid and r["exact_original"] == exact
        assert r["repair_paid"] == int(recovered) and r["final_fuel"] == initial - int(recovered)
        assert 0 <= r["attack"]["work"] <= 50000 and 0 <= r["attack"]["results"] <= 50000
        assert len(r["observations"]) == 8
        for index, obs in enumerate(r["observations"]):
            assert obs["step"] == index + 2 and obs["valid"] == valid and obs["ceiling"] is False
            assert obs["paid"] == obs["accepted"] == int(recovered and index == 0)
            assert obs["fuel"] == initial - int(recovered) and obs["binary_fuel"] == binary
            assert obs["rng"] == r["repair_rng"]
            compiled = condition == "intact_paid" or (recovered and index > 0) or (composite and attack_valid)
            assert obs["world_classes"] == int(compiled) and obs["quarantined"] == int(not compiled)
            assert 0 <= obs["work"] <= 50000 and 0 <= obs["results"] <= 50000
            maximum_work = max(maximum_work, obs["work"])
            maximum_results = max(maximum_results, obs["results"])
        assert [c["phase"] for c in r["checkpoints"]] == ["before_attack", "after_attack", "after_repair"]
        for c, step in zip(r["checkpoints"], [0, 1, 9]):
            assert c["roundtrip_verified"] is True and c["step"] == c["inventory"]["step"] == step
            audit_inventory(c)
            snapshot = path.parent / c["path"]
            assert snapshot.resolve().is_relative_to(path.parent.resolve())
            frame = snapshot.read_bytes()
            magic, version, wire_step, length = struct.unpack("<4sHQQ", frame[:22])
            assert magic == b"HGSS" and version == 2 and wire_step == step
            assert length == c["bytes"] and len(frame) == length + 22
            checkpoint_count += 1
        assert before["root"] == attacked["root"] == final["root"]
        assert before["instruction"] == attacked["instruction"] == final["instruction"]
        assert before["fuel"] == attacked["fuel"] == initial and final["fuel"] == r["final_fuel"]
        b, a, f = map(edges, [before, attacked, final])
        removed, added = list((b - a).elements()), list((a - b).elements())
        old_nodes, attacked_nodes = node_data(before), node_data(attacked)
        if condition == "payload_foreign_erased":
            assert old_nodes.keys() < attacked_nodes.keys() and len(attacked_nodes) == len(old_nodes) + 1
            assert all(attacked_nodes[key] == val for key, val in old_nodes.items())
        else:
            assert old_nodes == attacked_nodes
        if condition == "intact_paid":
            assert b == a
        elif condition.startswith("duplicate"):
            assert not removed and len(added) == 1
            extra = json.loads(added[0])
            assert extra["label"] == (r["role"] if condition == "duplicate_payload" else 0xFFF00100 + r["role_index"])
            assert len(extra["nodes"]) == 3
        else:
            multi = condition in ["both_erased", "witness_reversed_payload_erased"]
            assert len(removed) == (2 if multi else 1)
            assert len(added) == (0 if condition.endswith("deleted") else 2 if multi else 1)
            old = [json.loads(e) for e in removed]
            new = [json.loads(e) for e in added]
            expected_labels = [r["role"], 0xFFF00100 + r["role_index"]] if multi else [
                0xFFF00100 + r["role_index"] if condition.startswith("witness") else r["role"]
            ]
            assert sorted(e["label"] for e in old) == sorted(expected_labels)
            assert all(len(e["nodes"]) == 3 for e in old + new)
            for edge in old:
                witness = edge["label"] >= 0xFFF00100
                endpoints = edge["nodes"].copy()
                label = edge["label"]
                if condition in ["payload_reversed", "payload_erased_reversed"] or (witness and condition == "witness_reversed_payload_erased"):
                    offset = int(witness)
                    endpoints[offset], endpoints[offset + 1] = endpoints[offset + 1], endpoints[offset]
                erase = condition in ["erased_paid", "erased_zero", "erased_binary_only", "payload_erased_reversed", "payload_foreign_erased", "witness_erased", "both_erased"] or (not witness and condition == "witness_reversed_payload_erased")
                if erase:
                    label = 0
                if condition == "payload_foreign_erased":
                    assert len(new) == 1 and new[0]["label"] == 0
                    assert new[0]["nodes"][0] == endpoints[0] and new[0]["nodes"][2] == endpoints[2]
                    assert new[0]["nodes"][1] != endpoints[1]
                    foreign = new[0]["nodes"][1]
                    foreign_key = f'NodeId({foreign["idx"]}v{foreign["version"]})'
                    original = endpoints[1]
                    original_key = f'NodeId({original["idx"]}v{original["version"]})'
                    assert foreign_key not in old_nodes and attacked_nodes[foreign_key]["label"] == old_nodes[original_key]["label"]
                elif not condition.endswith("deleted"):
                    assert {"label": label, "nodes": endpoints} in new
        if recovered:
            lost, restored = list((a - f).elements()), list((f - a).elements())
            assert len(lost) == len(restored) == 1
            damaged, repaired = json.loads(lost[0]), json.loads(restored[0])
            assert damaged["label"] == 0 and len(damaged["nodes"]) == 3
            assert repaired["label"] == r["role"] and repaired["nodes"] == damaged["nodes"]
        else:
            assert a == f
        final_nodes = copy.deepcopy(node_data(final))
        if recovered:
            op = final["instruction"]
            props = final_nodes[f'NodeId({op["idx"]}v{op["version"]})']["properties"]
            assert [1, {"Float": 0.0}] in props
            props[props.index([1, {"Float": 0.0}])] = [1, {"Float": 1.0}]
        assert final_nodes == attacked_nodes
        counts[condition] += 1
        valid_counts[condition] += int(valid)
        exact_counts[condition] += int(exact)
        if composite and valid:
            changed_roles[f'{r["role_index"]:02}'] += 1
        payments[condition] += r["repair_paid"]
        summaries[r["case"]] = {"case": r["case"], "condition": condition, "valid": valid,
                               "exact_original": exact, "paid": r["repair_paid"]}
    summary = read_json(out / "summary.json")
    changed = sum(changed_roles.values())
    assert summary["paid_mechanism_pass"] is True and summary["training_admitted"] is False
    assert summary["valid_changed_body_cases"] == changed
    assert summary["all_negative_quarantined"] == summary["original_broader_gate_pass"] == (changed == 0)
    assert summary["completed_cases"] == len(paths) == 728 and summary["checkpoints_verified"] == checkpoint_count == 2184
    assert summary["engine_steps"] == 6552 and summary["seal_paid"] == 728
    assert len(summary["cases"]) == 728 and {r["case"]: r for r in summary["cases"]} == summaries
    return {"audit_pass": True, "completed_cases": len(paths), "checkpoint_frames_verified": checkpoint_count,
            "full_store_roundtrips_verified_by_rust": checkpoint_count, "conditions": dict(counts),
            "valid_final": dict(valid_counts), "exact_original_final": dict(exact_counts),
            "repair_payments": dict(payments), "valid_changed_body_cases": changed,
            "changed_body_role_counts": dict(changed_roles), "paid_mechanism_pass": True,
            "all_negative_quarantined": changed == 0, "original_broader_gate_pass": changed == 0,
            "max_repair_work": maximum_work, "max_repair_results": maximum_results,
            "engine_steps": 6552, "seal_payments": 728, "training_admitted": False,
            "contract_sha256": CONTRACT_SHA, "test_source_sha256": digest(out / "test_source.rs"),
            "fixture_source_sha256": digest(out / "fixture_source.rs"),
            "executed_binary_sha256": digest(out / "executed_test_binary")}


def rejection_controls(out):
    target = out / "984927043585_role00_erased_paid" / "receipt.json"
    original = read_json(target)
    mutations = {
        "unpaid_recovery": lambda r: r.update(repair_paid=0),
        "fuel_renewal": lambda r: r.update(final_fuel=1),
        "missing_step": lambda r: r["observations"].pop(),
        "cached_world_class": lambda r: r["observations"][0].update(world_classes=1),
        "hidden_ceiling": lambda r: r["observations"][0].update(ceiling=True),
        "rng_drift": lambda r: r["observations"][0].update(rng="bad"),
        "nonoriginal_recovery": lambda r: r.update(exact_original=False),
        "binary_fuel_debit": lambda r: r["observations"][0].update(binary_fuel=-1),
        "false_frame_step": lambda r: r["checkpoints"][0].update(step=999),
        "invented_endpoint": lambda r: r["checkpoints"][2]["inventory"]["edges"][0][1]["nodes"].reverse(),
    }
    real_reader = read_json
    rejected = []
    for name, change in mutations.items():
        mutant = copy.deepcopy(original)
        change(mutant)
        with patch.dict(globals(), read_json=lambda p: mutant if p == target else real_reader(p)):
            try:
                audit(out)
            except AssertionError:
                rejected.append(name)
            else:
                raise AssertionError(f"accepted tampering: {name}")
    composite_target = out / "984927043585_role03_both_erased" / "receipt.json"
    composite_original = read_json(composite_target)
    assert composite_original["final_valid"] is True and composite_original["exact_original"] is False
    summary_target = out / "summary.json"
    for name, target_path, original_value, change in [
        ("false_composite_recovery", composite_target, composite_original,
         lambda r: r.update(exact_original=True)),
        ("false_quarantine_promotion", summary_target, read_json(summary_target),
         lambda r: r.update(all_negative_quarantined=True, original_broader_gate_pass=True)),
        ("hidden_changed_body", summary_target, read_json(summary_target),
         lambda r: r.update(valid_changed_body_cases=0)),
    ]:
        mutant = copy.deepcopy(original_value)
        change(mutant)
        with patch.dict(globals(), read_json=lambda p: mutant if p == target_path else real_reader(p)):
            try:
                audit(out)
            except AssertionError:
                rejected.append(name)
            else:
                raise AssertionError(f"accepted tampering: {name}")
    return rejected


def write_checksums(out):
    """Integrity alone does not promote an aborted/incomplete matrix."""
    files = sorted(p for p in out.rglob("*") if p.is_file() and p.name != "checksums.sha256")
    (out / "checksums.sha256").write_text("".join(f"{digest(p)}  {p}\n" for p in files))
    return len(files)


if __name__ == "__main__":
    if not __debug__:
        raise SystemExit("Run Python without -O; audit assertions must be enabled.")
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path)
    parser.add_argument("--test-rejections", action="store_true")
    args = parser.parse_args()
    result = audit(args.output)
    if args.test_rejections:
        result["rejected_in_memory_tampering"] = rejection_controls(args.output)
    (args.output / "receipt_verification.json").write_text(json.dumps(result, indent=2) + "\n")
    write_checksums(args.output)
    print(json.dumps(result, indent=2))
