#!/usr/bin/env python3
"""Independent timeline/graph/fuel audit; not a CandidateRule/bincode decoder."""
import argparse
from collections import Counter, defaultdict
import copy
import json
from pathlib import Path
import struct
import tomllib

from verify_ere_complete_witness import (
    audit_inventory, check_complete_material, digest, edges, node_data,
    node_key, property_value, write_checksums,
)

CONTRACT_SHA = "ef7996387796a60aceeed0ba8b4c51e2ce449308d00729a2cdc67f7899ba13ec"
SEEDS = list(range(984927043601, 984927043605))
CORE, OWNS, META = 0xFFF00042, 0xFFF00120, 0xFFF00000
CLOCK, MARKER = 0xE5560001, 0xE5560002
BOUNDARIES = [8, 9, 16, 17, 24, 25, 32, 33, 40, 41, 64]


def semantic_config(cfg):
    """AHashMap TOML key order is presentation, not experimental semantics."""
    cfg = copy.deepcopy(cfg)
    cfg["rules"]["inline_toml"] = tomllib.loads(cfg["rules"]["inline_toml"])
    for family in cfg["ere"]["seed_families"]:
        for key in ["core_toml", "proposal_toml"]:
            family[key] = tomllib.loads(family[key])
    return cfg


def read(path):
    return json.loads(path.read_text())


def meta_edges(c):
    return Counter(json.dumps(e, sort_keys=True) for _, e in c["inventory"]["edges"]
                   if any(node_data(c)[node_key(n)]["label"] >= META for n in e["nodes"]))


def expected_graph(initial, root, side, damaged, lost):
    graph = copy.deepcopy(initial)
    for edge in graph["inventory"]["edges"]:
        e = edge[1]
        if damaged and e["label"] == CORE and e["nodes"] == [root, side, root]:
            e["label"] = 0
        if lost and e["label"] == OWNS and e["nodes"] == [root, side]:
            e["label"] = 0
    return meta_edges(graph)


def audit_receipt(r, directory=None):
    protected, initial, ownership = r["protected"], r["initial_fuel"], r["late_ownership_loss"]
    assert initial in [0.0, 2.0, 8.0] and type(protected) is bool and type(ownership) is bool
    assert r["seed"] in SEEDS and r["error"] is None and r["completed_steps"] == 64
    assert r["final_hash"] is not None and r["training_admitted"] is False
    assert r["environment_ordered"] is True and r["environment_meta_visible"] == (not protected)
    assert r["environment_node_label_ceiling"] == (META if protected else None)
    cfg = r["effective_config"]
    assert cfg["seed"] == r["seed"] and cfg["engine"]["max_steps"] == 64
    assert cfg["engine"]["thread_count"] == 1 and cfg["engine"]["step_mode"] == "MaxParallel"
    assert cfg["engine"]["max_pairings_per_step"] == 16 and cfg["engine"]["metrics_window_interval"] == 1
    ec = cfg["ere"]
    assert ec["arm"] == ("A2" if protected else "A3") and ec["epoch"] == {"Fixed": 1}
    assert ec["match_budget_policy"] == "Pooled" and ec["match_ceiling"] == 50000
    assert ec["meta_seed_energy"] == 0 and ec["meta_rules_toml"] is None
    assert ec["measure_family_maintenance"] is True and ec["measure_ei"] is False
    assert ec["vocabulary"] == {"protocol": "E2-witness-complete-v1", "instructions": [
        {"operation": "Seal", "fuel": 1.0}, {"operation": "RepairSealed", "fuel": initial}]}
    assert len(ec["seed_families"]) == 1 and ec["seed_families"][0]["active"] is False
    for text, proposal in [(cfg["rules"]["inline_toml"], False),
                           (ec["seed_families"][0]["core_toml"], False),
                           (ec["seed_families"][0]["proposal_toml"], True)]:
        raw = tomllib.loads(text)["rules"]
        assert len(raw) == 1
        rule = raw[0]
        path, star = [["v0", "v1"], ["v1", "v2"]], [["v0", "v1"], ["v0", "v2"]]
        assert rule["lhs"]["edges"] == (star if proposal else path)
        assert rule["rhs"]["edges"] == (path if proposal else star)
        assert rule["lhs"]["nodes"] == rule["rhs"]["nodes"] == ["v0", "v1", "v2"]
        assert rule["lhs"]["node_labels"] == {"v0": 0, "v1": 0, "v2": 0}
        assert rule["priority"] == int(proposal) and rule["weight"] == 1.0
        assert rule["rhs_node_labels"] == rule["lhs"]["edge_labels"] == rule["rhs"]["edge_labels"] == {}
        assert rule["rhs_property_effects"] == rule["lhs_node_property_guards"] == rule["lhs_var_guards"] == []
    assert cfg["initial"]["width"] == cfg["initial"]["height"] == 4 and cfg["initial"]["periodic"] is True
    rules = r["environment_raw_rules"]
    names = ["environment_clock", *[f"environment_payload_{s}" for s in [8, 16, 24, 32]]]
    if ownership:
        names.append("environment_ownership_40")
    assert [x["name"] for x in rules] == names
    assert all(x["schedule"] is None for x in rules)
    assert all(x["priority"] == (100 if i == 0 else 200) for i, x in enumerate(rules))
    p = r["preamble"]
    assert p["seal_paid"] == 1 and p["step"] == 0 and p["rng_before"] == p["rng_after"]
    assert p["original_payload"] in p["seal_binding"]["matched_edges"]
    root, side, repair, seal, clock = [p[k] for k in ["root", "side", "repair", "seal", "clock"]]
    initial_c = {"root": root, "inventory": p["inventory"]}
    before_c = {"root": root, "inventory": p["before_inventory"]}
    for c in [initial_c, before_c]:
        audit_inventory(c)
        check_complete_material(c, True)
    old, new = edges(before_c), edges(initial_c)
    removed, added = list((old - new).elements()), list((new - old).elements())
    assert len(removed) == len(added) == 1
    assert json.loads(removed[0]) == {"label": CORE, "nodes": [root, side]}
    assert json.loads(added[0]) == {"label": CORE, "nodes": [root, side, root]}
    before_nodes, setup_nodes = node_data(before_c), node_data(initial_c)
    assert len(setup_nodes) == len(before_nodes) + 1 and set(setup_nodes) - set(before_nodes) == {node_key(clock)}
    expected_nodes = copy.deepcopy(before_nodes)
    for n, value in [(root, 0), (side, 1)]:
        expected_nodes[node_key(n)]["properties"].append([MARKER, {"Int": value}])
    props = expected_nodes[node_key(seal)]["properties"]
    props[props.index([1, {"Float": 1.0}])] = [1, {"Float": 0.0}]
    assert all(setup_nodes[key] == val for key, val in expected_nodes.items())
    assert property_value(setup_nodes[node_key(clock)], CLOCK) == {"Int": 0}
    assert setup_nodes[node_key(clock)]["label"] == 777
    assert p["checkpoint"]["roundtrip_verified"] is True
    assert p["checkpoint"]["inventory"] == p["inventory"]
    if directory is not None:
        frame = (directory / p["checkpoint"]["path"]).read_bytes()
        magic, version, step, length = struct.unpack("<4sHQQ", frame[:22])
        assert (magic, version, step) == (b"HGSS", 2, 0)
        assert len(frame) == length + 22 and length == p["checkpoint"]["bytes"]
    native, observations, rows = [r[k] for k in ["native_steps", "step_observations", "ere_windows"]]
    assert len(native) == len(observations) == len(rows) == 64
    assert [x["step"] for x in native] == [x["step"] for x in observations] == list(range(1, 65))
    assert [x["step"] for x in native if "inventory" in x] == BOUNDARIES
    damaged, lost, fuel, valid = False, False, initial, True
    valid_steps = paid_total = hazards = world_total = 0
    paid_steps, newly_damaged = [], []
    max_work = max_results = 0
    for step, (n, o, row) in enumerate(zip(native, observations, rows), 1):
        previous_valid = valid
        # Standalone PropertyValue uses its JSON scalar representation; the
        # full NodeData inventory retains the tagged Int/Float wire values.
        assert n["root"] == root and n["clock"] == step
        assert o["rng_before_compile"] == o["rng_after_compile"] == o["rng_after_step"] == p["rng_after"]
        if step > 1:
            assert o["world_hash_before"] == observations[step - 2]["world_hash_after"]
        assert o["match_ceiling_binding"] is False
        assert 0 <= o["match_work"] <= 50000 and 0 <= o["match_results"] <= 50000
        max_work, max_results = max(max_work, o["match_work"]), max(max_results, o["match_results"])
        assert o["world_rule_table"] == ([p["expected_core_bytes"]] if previous_valid else [])
        assert row["step_start"] == step - 1 and row["step_end"] == step
        assert row["match_ceiling_binding"] is False and row["rule_region_ceiling_binding"] is False
        assert row["encoded_meta_vocabulary_protocol"] == "E2-witness-complete-v1"
        events = n["events"]
        assert len(events) == n["stats"]["accepted_rewrites"] <= 16
        assert n["stats"]["property_effects_skipped"] == 0
        used_nodes = set()
        for e in events:
            keys = [node_key(v) for v in e["bound_nodes"]]
            # The production matcher is deliberately non-injective within a
            # binding. Conflict resolution requires disjoint *sets* between
            # accepted rewrites, not distinct values for all pattern variables.
            assert not used_nodes.intersection(keys)
            used_nodes.update(keys)
            assert e["nodes_created"] == e["nodes_deleted"] == e["cleanup_neighbors"] == []
            assert e["vanished_energy"] == 0
        environment = [e for e in events if e["rule_name"].startswith("environment_")]
        assert len(environment) == 1
        assert environment[0]["rule_id"] == int(previous_valid) + 28 + names.index(environment[0]["rule_name"])
        hazard = None
        if not protected and step in [8, 16, 24, 32]:
            hazard = f"environment_payload_{step}"
            if not damaged:
                newly_damaged.append(step)
            damaged = True
        if not protected and ownership and step == 40:
            hazard = "environment_ownership_40"
            lost = True
        assert environment[0]["rule_name"] == (hazard or "environment_clock")
        assert environment[0]["bound_nodes"] == ([clock, root, side] if hazard else [clock])
        assert len(environment[0]["matched_edges"]) == len(environment[0]["edges_deleted"]) == len(environment[0]["edges_created"]) == int(hazard is not None)
        hazards += hazard is not None
        traces = row["encoded_meta_operation_traces"]
        paid_events = [e for e in events if e["rule_name"].startswith("complete_encoded_meta_")]
        expected_paid = hazard is None and damaged and fuel > 0
        assert len(traces) == len(paid_events) == row["encoded_meta_rewrite_events"] == int(expected_paid)
        if expected_paid:
            t = traces[0]
            assert t["step"] == step and t["operation"] == "RepairSealed"
            assert t["instruction"] == node_key(repair) and t["target_family"] == node_key(root)
            assert t["fuel_before"] == fuel and t["fuel_after"] == fuel - 1
            assert t["family_valid_before"] is False and t["family_valid_after"] == (not lost)
            assert paid_events[0]["bound_nodes"][0] == repair
            assert int(previous_valid) + 14 <= paid_events[0]["rule_id"] < int(previous_valid) + 28
            fuel -= 1
            paid_total += 1
            paid_steps.append(step)
            damaged = False
        valid = not damaged and not lost
        valid_steps += valid
        assert n["fuel"] == fuel and n["valid"] == n["exact_original"] == valid
        assert n["execution_bytes"] == (p["expected_execution_bytes"] if valid else None)
        assert o["families"] == [{"anchor": node_key(root), "valid": valid,
                                   "active": False if valid else None,
                                   "execution_bytes": p["expected_execution_bytes"] if valid else None}]
        states = row["encoded_meta_instruction_states"]
        assert len(states) == 2
        for st, operation, value, identity in zip(states, ["Seal", "RepairSealed"], [0.0, fuel], [seal, repair]):
            assert st["instruction"] == node_key(identity) and st["operation"] == operation
            assert st["valid"] is True and st["checks"] == st["valid_steps"] == step
            assert st["first_invalid_step"] is None and st["fuel"] == value
        world = [e for e in events if e["rule_name"].startswith("e1_")]
        assert all(e["rule_id"] == 0 for e in world)
        assert bool(world) == previous_valid
        world_total += len(world)
        assert len(events) == len(world) + len(paid_events) + 1
        assert row["seeded_editor_rewrite_events"] == 1
        assert row["ordinary_meta_rewrite_events"] == int(expected_paid)
        assert row["meta_rewrite_events"] == int(expected_paid) + int(hazard is not None)
        if "inventory" in n:
            c = {"root": root, "inventory": n["inventory"]}
            assert c["inventory"]["step"] == step
            audit_inventory(c)
            check_complete_material(c, valid)
            assert meta_edges(c) == expected_graph(initial_c, root, side, damaged, lost)
            data = node_data(c)
            assert set(data) == set(setup_nodes)
            expected = copy.deepcopy(setup_nodes)
            for identity, key, value in [(repair, 1, {"Float": fuel}), (clock, CLOCK, {"Int": step})]:
                props = expected[node_key(identity)]["properties"]
                position = next(i for i, (k, _) in enumerate(props) if k == key)
                props[position] = [key, value]
            assert data == expected
    s = r["summary"]
    assert s == {"completed_steps": 64, "error": None, "valid_steps": valid_steps,
                 "exact_steps": valid_steps, "changed_valid_steps": 0, "repair_payments": paid_total,
                 "final_fuel": fuel, "final_valid": valid, "hazard_events": hazards,
                 "world_events": world_total}
    assert paid_steps == ([] if protected or initial == 0 else [9, 17] if initial == 2 else [9, 17, 25, 33])
    assert all(s - 1 in newly_damaged for s in paid_steps)
    maintenance = rows[-1]["seeded_family_maintenance"]
    assert len(maintenance) == 1 and maintenance[0]["checks"] == 64
    assert maintenance[0]["valid_steps"] == valid_steps and maintenance[0]["final_valid"] == valid
    return {**s, "paid_steps": paid_steps, "newly_damaging_steps": newly_damaged,
            "max_work": max_work, "max_results": max_results}


def rejections(r):
    mutations = {
        "missing_step": lambda x: x["native_steps"].pop(),
        "false_recovery": lambda x: x["native_steps"][7].update(valid=True, exact_original=True),
        "fuel_renewal": lambda x: x["native_steps"][8].update(fuel=8.0),
        "cached_world_admission": lambda x: x["step_observations"][8].update(world_rule_table=["bad"]),
        "hidden_ceiling": lambda x: x["step_observations"][0].update(match_ceiling_binding=True),
        "false_exposure": lambda x: x["native_steps"][7]["events"].pop(),
        "changed_program": lambda x: x["native_steps"][8].update(execution_bytes="bad"),
        "visibility_bypass": lambda x: x.update(environment_node_label_ceiling=META),
        "unpaid_trace": lambda x: x["ere_windows"][8].update(encoded_meta_operation_traces=[]),
        "missing_ownership": lambda x: x["preamble"]["inventory"]["edges"].__setitem__(
            slice(None), [e for e in x["preamble"]["inventory"]["edges"] if e[1]["label"] != OWNS]),
    }
    rejected = []
    for name, mutate in mutations.items():
        mutant = copy.deepcopy(r)
        mutate(mutant)
        try:
            audit_receipt(mutant)
        except (AssertionError, KeyError, IndexError, TypeError):
            rejected.append(name)
        else:
            raise AssertionError(f"accepted tampering: {name}")
    return rejected


def audit(out, test_rejections=False):
    m = read(out / "manifest.json")
    assert digest(out / "preregistration.md") == CONTRACT_SHA
    assert m["diagnostic"] == "E2-complete-repeated-damage-v1"
    assert m["protocol"] == "E2-witness-complete-v1" and m["seeds"] == SEEDS
    assert (m["attempts"], m["horizon"], m["step_ceiling"]) == (48, 64, 3072)
    assert m["optimizer_evaluations"] == m["held_out_evaluations"] == 0 and m["training_admitted"] is False
    paths = sorted(out.glob("attempt_*/receipt.json"))
    assert [p.parent.name for p in paths] == [f"attempt_{i:03}" for i in range(48)]
    expected = [(seed, protected, fuel, owner) for seed in SEEDS for protected in [True, False]
                for fuel in [0.0, 2.0, 8.0] for owner in [False, True]]
    results, grouped = [], defaultdict(list)
    target = None
    for index, (path, cell) in enumerate(zip(paths, expected)):
        r = read(path)
        assert (r["seed"], r["protected"], r["initial_fuel"], r["late_ownership_loss"]) == cell
        assert m["planned_configs"][index]["late_ownership_loss"] == cell[3]
        assert semantic_config(m["planned_configs"][index]["config"]) == semantic_config(r["effective_config"])
        result = audit_receipt(r, path.parent)
        results.append(result)
        grouped[f'{"A2" if cell[1] else "A3"}_fuel{cell[2]:g}_owner{int(cell[3])}'].append(result)
        if cell[1:] == (False, 8.0, True) and target is None:
            target = r
    summary = read(out / "summary.json")
    assert summary["engineering_pass"] is True and summary["training_admitted"] is False
    assert len(summary["attempts"]) == 48
    for index, (row, result, cell) in enumerate(zip(summary["attempts"], results, expected)):
        assert row == {"attempt": f"attempt_{index:03}", "seed": cell[0], "protected": cell[1],
                       "initial_fuel": cell[2], "late_ownership_loss": cell[3],
                       "summary": {k: v for k, v in result.items() if k not in ["paid_steps", "newly_damaging_steps", "max_work", "max_results"]}}
    return {"audit_pass": True, "attempts_verified": 48, "completed_steps": 3072,
            "initial_full_store_roundtrips": 48, "inventory_boundaries_verified": 576,
            "repair_payments": sum(r["repair_payments"] for r in results), "seal_payments": 48,
            "valid_changed_body_steps": 0, "repair_latency_steps": 1,
            "groups": {k: {"attempts": len(v), "valid_steps_per_attempt": sorted(set(r["valid_steps"] for r in v)),
                            "repair_payments_per_attempt": sorted(set(r["repair_payments"] for r in v)),
                            "final_fuel_per_attempt": sorted(set(r["final_fuel"] for r in v)),
                            "world_events_per_attempt": sorted(set(r["world_events"] for r in v))}
                       for k, v in grouped.items()},
            "max_match_work": max(r["max_work"] for r in results),
            "max_match_results": max(r["max_results"] for r in results),
            "tamper_rejections": rejections(target) if test_rejections else [],
            "contract_sha256": CONTRACT_SHA, "test_source_sha256": digest(out / "test_source.rs"),
            "executed_binary_sha256": digest(out / "executed_test_binary"), "training_admitted": False}


if __name__ == "__main__":
    if not __debug__:
        raise SystemExit("Run without -O; audit assertions must be enabled.")
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("out", type=Path)
    parser.add_argument("--test-rejections", action="store_true")
    parser.add_argument("--write-checksums", action="store_true")
    args = parser.parse_args()
    result = audit(args.out, args.test_rejections)
    print(json.dumps(result, indent=2))
    (args.out / "receipt_verification.json").write_text(json.dumps(result, indent=2) + "\n")
    if args.write_checksums:
        write_checksums(args.out)
