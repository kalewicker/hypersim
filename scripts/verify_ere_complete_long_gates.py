#!/usr/bin/env python3
"""Streaming independent long-gate auditor; never runs the engine or admits training.

The Rust executable witnesses canonical graph hashes and RNG draws. This checker
independently replays every typed graph delta, native binding/template/effect,
complete-family graph, ecological motif, payment, boundary and paired CSV row.
Development receipts always have gate_pass=false, even when all checks succeed.
"""
import argparse
from collections import Counter
from contextlib import ExitStack, closing
import copy
import gzip
import hashlib
import json
from pathlib import Path
import re
import resource
import sys
import time

from verify_ere_complete_gates import checked_program, compare_graph
from verify_ere_complete_witness import audit_inventory, node_key, property_value

PROFILE = "E2-complete-long-horizon-gates-v1"
CONTRACT_JSON_SHA = "325652948c1c59badf60355a2456ed7a6937531c9077ae4de3acca06a8886f2f"
CONTRACT_MD_SHA = "106c4bd85f695d1fbdf093976292aee786f1e277871ec698d11ca01c9c53306c"
BASE, CLOCK, MARKER = 0xFFF00000, 0xE5560001, 0xE5560002
HEADER = "step,full_hash,world_hash,n_nodes,n_edges,attempted_matches,accepted_rewrites,nodes_created,nodes_deleted,edges_created,edges_deleted,property_effects_applied,property_effects_skipped,rng_before,rng_after,match_work,match_results,match_ceiling_binding\n"
MEMORY = 1073741824
DEADLINE = 0


def guard_resources():
    assert time.monotonic() < DEADLINE, "auditor wall cap"
    rss = resource.getrusage(resource.RUSAGE_SELF).ru_maxrss
    if sys.platform != "darwin":
        rss *= 1024
    assert rss <= MEMORY, "auditor resident memory cap"


def digest(path):
    with path.open("rb") as f:
        return hashlib.file_digest(f, "sha256").hexdigest()


def load(path):
    return json.loads(path.read_text())


def save(path, value):
    assert not path.exists(), f"refusing to overwrite {path}"
    path.write_text(json.dumps(value, sort_keys=True, indent=2) + "\n")


def safe_files(out):
    ps = list(out.rglob("*"))
    assert not any(p.is_symlink() for p in ps), "archive symlinks disallowed"
    return sorted(p for p in ps if p.is_file())


def freeze(out):
    assert not list(out.glob("*/steps.jsonl.gz")), "freeze must precede steps"
    m = load(out / "manifest.json")
    actual = sorted(str(p.relative_to(out / "source_snapshots")) for p in safe_files(out / "source_snapshots"))
    assert actual == sorted(m["sources"]) and len(actual) == len(set(actual))
    assert not any("__pycache__" in p or p.endswith(".pyc") for p in actual)
    assert digest(out / "source_snapshots/docs/ere_complete_long_gates_v1.json") == CONTRACT_JSON_SHA
    assert digest(out / "source_snapshots/docs/ere_complete_long_gates_v1.md") == CONTRACT_MD_SHA
    design = load(out / "source_snapshots/docs/ere_complete_long_gates_v1.json")
    for filename, key in [("summary.json", "summary_sha256"), ("independent_audit.json", "independent_audit_sha256"), ("run_status.json", "run_status_sha256")]:
        assert digest(out / "basis" / filename) == design["basis"][key]
    save(out / "frozen_inputs_sha256.json", {str(p.relative_to(out)): digest(p) for p in safe_files(out)})


def integrity(out):
    frozen = load(out / "frozen_inputs_sha256.json")
    for p, expected in frozen.items():
        assert digest(out / p) == expected, f"frozen input changed: {p}"
    m = load(out / "manifest.json")
    actual = sorted(str(p.relative_to(out / "source_snapshots")) for p in safe_files(out / "source_snapshots"))
    assert actual == sorted(m["sources"])
    assert sorted(k.removeprefix("source_snapshots/") for k in frozen if k.startswith("source_snapshots/")) == actual
    assert "manifest.json" in frozen and "executed_test_binary" in frozen
    assert digest(out / "source_snapshots/docs/ere_complete_long_gates_v1.json") == CONTRACT_JSON_SHA
    assert digest(out / "source_snapshots/docs/ere_complete_long_gates_v1.md") == CONTRACT_MD_SHA
    assert not any("__pycache__" in p or p.endswith(".pyc") for p in actual)
    assert sum(p.stat().st_size for p in safe_files(out)) <= 2147483648
    if (out / "output_sha256.json").exists():
        checks = load(out / "output_sha256.json")
        assert set(checks) == {str(p.relative_to(out)) for p in safe_files(out) if p.name != "output_sha256.json"}
        for p, sha in checks.items():
            assert digest(out / p) == sha, f"output checksum changed: {p}"
    return m


def stream(path):
    with gzip.open(path, "rt", encoding="utf-8", newline="") as f:
        while True:
            line = f.readline(4 * 1024 * 1024 + 1)
            if not line:
                break
            assert len(line) <= 4 * 1024 * 1024 and line.endswith("\n") and "\r" not in line
            value = json.loads(line)
            assert json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False) + "\n" == line, "noncanonical JSONL"
            guard_resources()
            yield value


def edge_key(n):
    return f'EdgeId({n["idx"]}v{n["version"]})'


def inventory(nodes, edges, step):
    inc = {n: [] for n in nodes}
    labels = {}
    for k, e in edges.items():
        labels.setdefault(e["label"], []).append(k)
        for n in e["nodes"]:
            inc[node_key(n)].append(k)
    return {"step": step, "nodes": [[n, {"data": data, "incident": sorted(inc[n])}] for n, data in sorted(nodes.items())],
            "edges": [[e, data] for e, data in sorted(edges.items())],
            "label_indexes": [[l, sorted(es)] for l, es in sorted(labels.items())]}


def prop(nodes, node, key):
    return property_value(nodes[node_key(node)] if isinstance(node, dict) else nodes[node], key)


def valid_program(inv, setup):
    try:
        compare_graph(inv, setup["root"], setup["program"])
        return True
    except (AssertionError, KeyError, IndexError):
        return False


def expected_valid(cell, step):
    if cell["arm"] != "A3":
        return True
    if cell["ownership_loss"] and step >= cell["ownership_step"]:
        return False
    if cell["root_loss"] and step >= cell["root_delete_step"]:
        return False
    lesions = [d for d in cell["payload_steps"] if d <= step]
    if not lesions:
        return True
    last = lesions[-1]
    return step > last and len(lesions) <= cell["repair_fuel"]


def schedule(cell, horizon):
    ds = cell["payload_steps"] + ([cell["ownership_step"]] if cell["ownership_loss"] else []) + ([cell["root_delete_step"]] if cell["root_loss"] else [])
    return {0, horizon} | set(range(1000, horizon + 1, 1000)) | {d + i for d in ds for i in [-1, 0, 1, 2] if 0 <= d + i <= horizon}


def check_table(table, cell, program=None):
    assert table and len(table) < 128
    for r in table:
        assert r["enabled"] and r["weight"] == 1.0 and r["schedule"] is None
        p = r["pattern"]
        assert p["match_work_limit"] == p["match_result_limit"] == 50000
        if r["name"].startswith("complete_encoded_meta_"):
            assert r["meta_visible"] is True and p["node_label_ceiling"] is None
        else:
            visible = cell["arm"] == "A3"
            assert r["meta_visible"] == visible
            assert p["node_label_ceiling"] == (None if visible else BASE)
        if r["name"].startswith("environment_"):
            assert all(e["ordered"] for e in p["edges"])
            names = r["lhs_var_names"]
            assert p["node_label_constraints"][names.index("clock")] == 777
            if r["name"] == "environment_clock":
                assert r["priority"] == 100 and not p["edges"]
            else:
                assert r["priority"] == 200
                kind, step = r["name"].split("_")[1:]
                step = int(step)
                assert (kind == "payload" and step in cell["payload_steps"] or kind == "ownership" and cell["ownership_loss"] and step == cell["ownership_step"]
                        or kind == "delete" and cell["root_loss"] and step == cell["root_delete_step"])
                expected = {("clock", CLOCK): step - 1, ("root", MARKER): 0, ("side", MARKER): 1}
                assert {(names[g["var_ix"]], g["key"]): g["value"]["Int"] for g in p["property_guards"]} == expected
                assert all(g["op"] == "Eq" for g in p["property_guards"])
                assert [names[i] for i in p["edges"][0]["var_ixs"]] == (["root", "side", "root"] if kind == "payload" else ["root", "side"])
                assert p["edges"][0]["label"] == (None if kind == "payload" else BASE + 0x120)
                if kind == "delete":
                    assert r["template"]["lhs_deleted"] == [names.index("root")]
                    assert not r["template"]["rhs_edges"]
                else:
                    assert r["template"]["rhs_edges"] == [[p["edges"][0]["var_ixs"], 0]]
        elif r["name"].startswith("e1_"):
            assert program is not None
            t = r["template"]
            candidates = [program["core"]] + ([program["proposal"]] if program["active"] else [])
            actual = {"priority": r["priority"], "labels": p["node_label_constraints"], "lhs": [e["var_ixs"] for e in p["edges"]],
                      "rhs": [ns for ns, _ in t["rhs_edges"]], "guards": []}
            assert any(actual == {k: c[k] for k in actual} for c in candidates)
            assert all(e["label"] is None and not e["ordered"] for e in p["edges"])
            assert not any(p[k] for k in ["property_guards", "property_compares", "degree_compares", "quantity_compares", "distinct_pairs"])
            assert not t["property_effects"] and not t["lhs_deleted"]
            assert t["rhs_from_lhs"] == [0, 1, 2] and t["rhs_relabels"] == [None] * 3
            assert all(label == 0 for _, label in t["rhs_edges"])
        else:
            assert r["name"].startswith("complete_encoded_meta_"), "unexpected installed rule"


def csv_row(r):
    s = r["native"]["stats"]
    vs = [r["step"], r["full_hash"], r["world_hash"], r["n_nodes"], r["n_edges"]]
    vs += [s[k] for k in ["attempted_matches", "accepted_rewrites", "nodes_created", "nodes_deleted", "edges_created", "edges_deleted", "property_effects_applied", "property_effects_skipped"]]
    vs += [r["rng_before"], r["rng_after"], r["usage"]["work"], r["usage"]["results"], str(r["ceiling"]).lower()]
    return ",".join(map(str, vs)) + "\n"


def compare_guard(actual, op, wanted):
    assert len(actual) == len(wanted) == 1 and actual.keys() == wanted.keys()
    a, b = next(iter(actual.values())), next(iter(wanted.values()))
    return {"Eq": lambda: a == b, "Ne": lambda: a != b, "Gt": lambda: a > b, "Gte": lambda: a >= b,
            "Lt": lambda: a < b, "Lte": lambda: a <= b}[op]()


def check_binding(r, b, matched, nodes, edges):
    p = r["pattern"]
    assert len(b) == p["n_vars"] and len(matched) == len(p["edges"])
    for ix, n in enumerate(b):
        data = nodes[node_key(n)]
        if p["node_label_ceiling"] is not None:
            assert data["label"] < p["node_label_ceiling"]
        if p["node_label_constraints"][ix] is not None:
            assert data["label"] == p["node_label_constraints"][ix]
    assert len(set(map(edge_key, matched))) == len(matched)
    for pattern, eid in zip(p["edges"], matched):
        e = edges[edge_key(eid)]
        want = [b[ix] for ix in pattern["var_ixs"]]
        if pattern["ordered"]:
            assert e["nodes"] == want
        else:
            assert Counter(map(node_key, e["nodes"])) == Counter(map(node_key, want))
        assert pattern["label"] is None or e["label"] == pattern["label"]
    for g in p["property_guards"]:
        assert compare_guard(prop(nodes, b[g["var_ix"]], g["key"]), g["op"], g["value"])


def replay_events(row, table, old_nodes, old_edges):
    # Copy the two small indexes, not every unchanged family/property array.
    # Only native effects/relabels need copy-on-write node data.
    nodes, edges = dict(old_nodes), dict(old_edges)
    created = dict(row["delta"]["edges"]["upsert"])
    total = Counter()
    used = set()
    for event in row["native"]["events"]:
        r = table[event["rule_id"]]
        assert r["name"] == event["rule_name"]
        b, matched = event["bound_nodes"], event["matched_edges"]
        check_binding(r, b, matched, old_nodes, old_edges)
        assert not set(map(node_key, b)) & used, "MaxParallel bindings not disjoint"
        used.update(map(node_key, b))
        t = r["template"]
        assert not event["nodes_created"] and all(ix is not None for ix in t["rhs_from_lhs"])
        rhs = [b[ix] for ix in t["rhs_from_lhs"]]
        deleted_nodes = [b[ix] for ix in t["lhs_deleted"]]
        assert event["nodes_deleted"] == deleted_nodes
        matched_keys = {edge_key(e) for e in matched}
        deleted_keys = set(map(node_key, deleted_nodes))
        cleanup_keys = {eid for eid, e in edges.items() if any(node_key(n) in deleted_keys for n in e["nodes"])} - matched_keys
        assert set(map(edge_key, event["edges_deleted"])) == matched_keys | cleanup_keys
        # Engine incidence lists retain repeated endpoints. Deleting the one
        # family root therefore reports repeated cleanup IDs for ternary
        # [root, allele, root] edges, although each physical edge disappears
        # once. Preserve this existing native accounting, not a guessed set.
        assert len(deleted_keys) <= 1
        multiplicity = Counter(matched_keys)
        multiplicity.update({eid: sum(node_key(n) in deleted_keys for n in edges[eid]["nodes"]) for eid in cleanup_keys})
        assert Counter(map(edge_key, event["edges_deleted"])) == multiplicity
        cleanup_neighbors = {node_key(n) for eid in cleanup_keys for n in edges[eid]["nodes"] if node_key(n) not in deleted_keys}
        assert set(map(node_key, event["cleanup_neighbors"])) == cleanup_neighbors
        for eid in matched_keys | cleanup_keys:
            assert eid in edges
            del edges[eid]
        for n in deleted_keys:
            assert nodes[n]["label"] >= BASE and n == node_key(row["_root"])
            del nodes[n]
        assert event["vanished_energy"] == 0.0
        assert len(event["edges_created"]) == len(t["rhs_edges"])
        for eid, (indices, label) in zip(event["edges_created"], t["rhs_edges"]):
            key = edge_key(eid)
            assert key not in edges
            assert created[key] == {"label": label, "nodes": [rhs[i] for i in indices]}
            edges[key] = created[key]
        for ix, label in enumerate(t["rhs_relabels"]):
            if label is not None:
                key = node_key(rhs[ix])
                nodes[key] = copy.deepcopy(nodes[key])
                nodes[key]["label"] = label
        for effect in t["property_effects"]:
            key = node_key(rhs[effect["target_ix"]])
            nodes[key] = copy.deepcopy(nodes[key])
            node = nodes[key]
            props = dict(node["properties"])
            operation = effect["op"]
            assert len(operation) == 1
            op, params = next(iter(operation.items()))
            assert op in ["Add", "Subtract", "Set"], "unexpected effect primitive"
            val = params["value"]
            if op == "Set":
                props[effect["key"]] = val
            else:
                old = props[effect["key"]]
                assert old.keys() == val.keys()
                typ = next(iter(old))
                props[effect["key"]] = {typ: old[typ] + (1 if op == "Add" else -1) * val[typ]}
            node["properties"] = [[k, v] for k, v in props.items()]
            total["property_effects_applied"] += 1
        total["nodes_deleted"] += len(deleted_nodes)
        total["edges_deleted"] += sum(multiplicity.values())
        total["edges_created"] += len(event["edges_created"])
    stats = row["native"]["stats"]
    assert stats["step"] == row["step"] and stats["accepted_rewrites"] == len(row["native"]["events"])
    assert stats["nodes_created"] == stats["property_effects_skipped"] == 0
    for k in ["nodes_deleted", "edges_deleted", "edges_created", "property_effects_applied"]:
        assert stats[k] == total[k], f"native stats {k} mismatch"
    # Node/edge upserts must be exactly the independently replayed differences.
    for kind, a, b in [("nodes", old_nodes, nodes), ("edges", old_edges, edges)]:
        want = {"deleted": sorted(set(a) - set(b)), "upsert": [[k, v] for k, v in sorted(b.items()) if a.get(k) != v]}
        actual = copy.deepcopy(row["delta"][kind])
        assert actual == want, f"unwitnessed {kind} delta"
    return nodes, edges


def check_row(row, setup, table, old_nodes, old_edges, previous):
    c, step = setup["cell"], row["step"]
    assert step == previous["step"] + 1 and not row["ceiling"] and not row["usage"]["truncated"]
    assert row["usage"]["work"] <= 50000 and row["usage"]["results"] <= 50000
    assert row["native"]["stats"]["attempted_matches"] == row["usage"]["results"]
    for k in ["full_hash", "world_hash", "full_hash_before", "world_hash_before", "rng_before", "rng_after_compile", "rng_after"]:
        assert re.fullmatch("[0-9a-f]{16}", row[k])
    assert row["rng_before"] == previous["rng_after"] == row["rng_after_compile"]
    assert (row["rng_before"] != row["rng_after"]) == (c["gate"] == "G-R3")
    if step > 1:
        assert row["full_hash_before"] == previous["full_hash"] and row["world_hash_before"] == previous["world_hash"]
    assert len(row["guards"]) == len({g[0] for g in row["guards"]})
    assert [g[0] for g in row["guards"]] == [r["name"] for r in table if any(r["pattern"][k] for k in ["property_guards", "property_compares", "degree_compares", "quantity_compares"])]
    usage = row["usage"]["rules"]
    assert [u["rule_id"] for u in usage] == list(range(len(table)))
    assert sum(u["work"] for u in usage) == row["usage"]["work"]
    assert sum(u["results"] for u in usage) == row["usage"]["results"]
    work, results = 50000, 50000
    for u in usage:
        assert not u["truncated"] and u["work_allowance"] == work and u["result_allowance"] == results
        assert 0 <= u["work"] <= work and 0 <= u["results"] <= results
        work -= u["work"]
        results -= u["results"]
    for name, g in row["guards"]:
        ix = setup["_table_names"][name]
        assert not g["enumeration_truncated"] and g["enumeration_work"] == usage[ix]["work"]
        assert len(g["property_guards"]) == len(table[ix]["pattern"]["property_guards"])
        for field in ["property_compares", "degree_compares", "quantity_compares"]:
            assert len(g[field]) == len(table[ix]["pattern"][field])
        for evaluated, rejected in sum([g[k] for k in ["property_guards", "property_compares", "degree_compares", "quantity_compares"]], []):
            assert 0 <= rejected <= evaluated
    report = row["compile_report"]
    if c["arm"] == "A0":
        assert report is None
    elif c["arm"] == "A1" and step > 1:
        assert report is None
    else:
        assert report["step"] == step - 1 and len(report["quarantine_by_validator"]) == 9
        assert sum(report["quarantine_by_validator"]) == report["quarantined"]
        valid_before = expected_valid(c, step - 1)
        alive_before = node_key(setup["root"]) in old_nodes
        assert len(report["classes"]) == (1 if c["identity"] else 2) * int(valid_before)
        assert report["quarantined"] == int(alive_before and not valid_before)
        assert report["validated_regions"] == len(setup["instructions"]) + int(valid_before)
        previous_valid = True if step == 1 else bool(previous["world_classes"])
        assert report["installed"] == (len(setup["instructions"]) + 1 if step == 1 else int(valid_before and not previous_valid))
        assert report["retired"] == int(previous_valid and not valid_before)
        assert report["changed"] == (step == 1 or previous_valid != valid_before)
    expected_classes = (1 if c["identity"] else 2) * int(expected_valid(c, step - 1))
    worlds = [e for e in row["native"]["events"] if e["rule_name"].startswith("e1_")]
    assert row["world_classes"] == expected_classes == sum(r["name"].startswith("e1_") for r in table)
    assert len(worlds) == (1 if c["gate"] == "G-R3" else 4) * int(expected_classes > 0)
    paid = [e for e in row["native"]["events"] if e["rule_name"].startswith("complete_encoded_meta_")]
    repair_steps = [] if c["arm"] != "A3" else [d + 1 for d in c["payload_steps"][:int(c["repair_fuel"])]]
    expect_seal = c["gate"] == "G-R5" and step == 1
    expect_repair = step in repair_steps
    assert len(paid) == int(expect_seal or expect_repair)
    for e in paid:
        assert ("RepairSealed" in e["rule_name"]) == expect_repair
        assert setup["root"] in e["bound_nodes"] and setup["core"] in e["bound_nodes"]
        created_ids = set(map(edge_key, e["edges_created"]))
        assert any(k in created_ids and edge == {"label": BASE + 0x42, "nodes": [setup["root"], setup["core"], setup["root"]]}
                   for k, edge in row["delta"]["edges"]["upsert"]), "paid target is not CORE payload"
    hazards = [e["rule_name"] for e in row["native"]["events"] if e["rule_name"].startswith("environment_") and e["rule_name"] != "environment_clock"]
    expected_hazards = []
    if c["arm"] == "A3":
        if step in c["payload_steps"]:
            expected_hazards.append(f"environment_payload_{step}")
        if c["ownership_loss"] and step == c["ownership_step"]:
            expected_hazards.append(f"environment_ownership_{step}")
        if c["root_loss"] and step == c["root_delete_step"]:
            expected_hazards.append(f"environment_delete_{step}")
    assert hazards == expected_hazards
    assert sum(e["rule_name"] == "environment_clock" for e in row["native"]["events"]) == (int(not expected_hazards) if c["gate"] == "G-R5" else 0)
    probes = row["binding_probes"]
    expected_probe_count = int(c["arm"] == "A2" and (step in c["payload_steps"] or c["ownership_loss"] and step == c["ownership_step"] or c["root_loss"] and step == c["root_delete_step"]))
    assert len(probes) == expected_probe_count
    for p in probes:
        assert p["read_only"] and p["clock"] == step - 1 and p["root"] == setup["root"] and p["core"] == setup["core"]
        r = next(r for r in table if r["name"] == p["rule"])
        assert p["protected_rule"] == r and not p["protected_bindings"] and p["visible_bindings"]
        visible = copy.deepcopy(r)
        visible["meta_visible"], visible["pattern"]["node_label_ceiling"] = True, None
        assert p["visible_rule"] == visible
        for binding in p["visible_bindings"]:
            assert setup["root"] in binding["nodes"] and setup["core"] in binding["nodes"]
            check_binding(visible, binding["nodes"], binding["edges"], old_nodes, old_edges)
        for k in ["protected_usage", "visible_usage"]:
            assert not p[k]["enumeration_truncated"] and p[k]["enumeration_work"] <= 50000
    # Internal context is added only after canonical/schema/CSV validation.
    row["_root"] = setup["root"]
    nodes, edges = replay_events(row, table, old_nodes, old_edges)
    del row["_root"]
    assert len(nodes) == row["n_nodes"] <= 128 and len(edges) == row["n_edges"] <= 512
    family = setup["_family_members"]
    family_change = any(k in family for k in row["delta"]["nodes"]["deleted"]) or any(k in family for k, _ in row["delta"]["nodes"]["upsert"])
    family_change |= any(any(node_key(n) in family for n in old_edges[k]["nodes"]) for k in row["delta"]["edges"]["deleted"])
    family_change |= any(any(node_key(n) in family for n in e["nodes"]) for _, e in row["delta"]["edges"]["upsert"])
    inv = inventory(nodes, edges, step) if family_change or step in setup["_checkpoints"] else None
    valid = valid_program(inv, setup) if family_change else previous["family_valid"]
    assert row["family_valid"] == row["original_program"] == valid == expected_valid(c, step)
    assert row["anchor_alive"] == (node_key(setup["root"]) in nodes)
    assert row["execution"] == (setup["program"]["execution"] if row["family_valid"] else None)
    for op in row["instructions"]:
        initial = next(o for o in setup["instructions"] if o["id"] == op["id"])
        assert op["valid"] and op["operation"] == initial["operation"]
        spent = int(c["gate"] == "G-R5") if op["operation"] == "Seal" else sum(d <= step for d in repair_steps)
        assert op["fuel"] == initial["fuel"] - spent
        assert prop(nodes, op["id"], 1) == {"Float": op["fuel"]}
        assert prop(nodes, op["id"], 8) == {"Int": 11}
        assert not any(op["id"] in e["nodes"] for e in edges.values())
    assert len(row["instructions"]) == len(setup["instructions"])
    if c["gate"] == "G-R5":
        assert row["clock"] == step and prop(nodes, setup["clock"], CLOCK) == {"Int": step}
    else:
        assert row["clock"] is None
    assert len(row["motifs"]) == 4
    changes = 0
    for old, motif, ids in zip(previous["motifs"], row["motifs"], setup["motifs"]):
        assert motif["nodes"] == ids and motif["phase"] in ["path", "star"]
        selected = sorted([[k, e] for k, e in edges.items() if all(n in ids for n in e["nodes"])])
        assert motif["edges"] == selected and len(selected) == 2
        patterns = sorted([sorted(map(node_key, e["nodes"])) for _, e in selected])
        want = sorted([sorted([node_key(ids[0]), node_key(ids[1])]), sorted([node_key(ids[0] if motif["phase"] == "star" else ids[1]), node_key(ids[2])])])
        assert patterns == want and all(e["label"] == 0 for _, e in selected)
        changes += old["phase"] != motif["phase"]
    assert changes == (0 if c["identity"] else len(worlds))
    initial_nodes = {k: n["data"] for k, n in setup["initial"]["nodes"]}
    for k, data in initial_nodes.items():
        if k in setup["_mutable_nodes"] or k == node_key(setup["root"]) and not row["anchor_alive"]:
            continue
        assert nodes[k] == data, "ordinary/body handle/data damaged"
    return nodes, edges, inv


def negative_controls(row, setup, table, nodes, edges, previous):
    tests = []
    for key, val in [("step", row["step"] + 1), ("ceiling", True), ("rng_after_compile", "0000000000000000"), ("family_valid", not row["family_valid"]), ("anchor_alive", not row["anchor_alive"]), ("world_classes", 99), ("n_nodes", 999)]:
        r = copy.deepcopy(row)
        r[key] = val
        tests.append((key, r, table))
    r = copy.deepcopy(row)
    r["native"]["stats"]["edges_created"] += 1
    tests.append(("stats", r, table))
    if row["native"]["events"]:
        r = copy.deepcopy(row)
        r["native"]["events"][0]["rule_id"] = len(table)
        tests.append(("event_rule", r, table))
    r = copy.deepcopy(row)
    r["instructions"][0]["fuel"] += 1
    tests.append(("payment", r, table))
    r = copy.deepcopy(row)
    r["delta"]["edges"]["deleted"] = [] if r["delta"]["edges"]["deleted"] else ["EdgeId(999999v1)"]
    tests.append(("delta", r, table))
    for name, r, t in tests:
        try:
            check_row(r, setup, t, nodes, edges, previous)
        except (AssertionError, KeyError, IndexError, StopIteration):
            continue
        raise AssertionError(f"tamper accepted: {name}")
    return len(tests)


def audit_cell(out, setup):
    with ExitStack() as stack:
        return _audit_cell(out, setup, stack)


def _audit_cell(out, setup, stack):
    c = setup["cell"]
    assert c["horizon"] > 0
    inv = setup["initial"]
    audit_inventory({"inventory": inv})
    assert inv["step"] == 0 and checked_program(setup["program"])
    compare_graph(inv, setup["root"], setup["program"])
    nodes = {k: n["data"] for k, n in inv["nodes"]}
    edges = dict(inv["edges"])
    setup["_family_members"] = {k for k, n in nodes.items() if n["label"] >= BASE} - {node_key(o["id"]) for o in setup["instructions"]}
    setup["_mutable_nodes"] = {node_key(o["id"]) for o in setup["instructions"]} | ({node_key(setup["clock"])} if setup["clock"] is not None else set())
    setup["_checkpoints"] = schedule(c, c["horizon"])
    ordinary = [n for n in nodes.values() if n["label"] in [10, 11, 12, 99]]
    assert len(ordinary) == 16 and Counter(n["label"] for n in ordinary) == {10: 4, 11: 4, 12: 4, 99: 4}
    for n in ordinary:
        assert n["generation"] == 0 and n["energy"] == 0.0 and n["properties"] == [[1, {"Float": 0.0}]]
    table = setup["initial_table"]
    check_table(table, c, setup["program"])
    setup["_table_names"] = {r["name"]: i for i, r in enumerate(table)}
    dir = out / c["id"]
    inventories = stack.enter_context(closing(iter(stream(dir / "inventories.jsonl.gz"))))
    assert next(inventories) == inv
    csv_file = stack.enter_context((dir / "trajectory.csv").open(newline="")) if c["gate"] == "G-R3" else None
    if csv_file:
        assert csv_file.readline() == HEADER
    previous = {"step": 0, "rng_after": setup["initial_rng"], "motifs": [{"phase": "path"}] * 4, "family_valid": True}
    checkpoints = schedule(c, c["horizon"])
    roles, tamper, probe_work, probe_results = Counter(), 0, 0, 0
    window_rows = stack.enter_context(closing(stream(dir / "windows.jsonl.gz")))
    window_counts = Counter()
    window_quarantine = [0] * 9
    valid_steps = 0
    previous_window_end = 0
    revision = None
    rows = stack.enter_context(closing(stream(dir / "steps.jsonl.gz")))
    for row in rows:
        assert row["step"] <= c["horizon"]
        if row["table"] is not None:
            table = row["table"]
            check_table(table, c, setup["program"])
            setup["_table_names"] = {r["name"]: i for i, r in enumerate(table)}
            assert revision is None or row["revision"] > revision
        else:
            assert revision == row["revision"]
        revision = row["revision"]
        if csv_file:
            assert csv_file.readline() == csv_row(row), "CSV/raw discrepancy"
        if row["step"] in {1, c["horizon"]}:
            tamper += negative_controls(row, setup, table, nodes, edges, previous)
        nodes, edges, inv = check_row(row, setup, table, nodes, edges, previous)
        for p in row["binding_probes"]:
            probe_work += p["protected_usage"]["enumeration_work"] + p["visible_usage"]["enumeration_work"]
            probe_results += len(p["protected_bindings"]) + len(p["visible_bindings"])
        assert probe_work <= 500000 and probe_results <= 500000
        if row["compile_report"] is not None:
            for field in ["installed", "retired", "quarantined"]:
                window_counts[field] += row["compile_report"][field]
            window_quarantine = [x + y for x, y in zip(window_quarantine, row["compile_report"]["quarantine_by_validator"])]
        window_counts["paid"] += sum(e["rule_name"].startswith("complete_encoded_meta_") for e in row["native"]["events"])
        window_counts["accepted"] += row["native"]["stats"]["accepted_rewrites"]
        window_counts["attempted"] += row["native"]["stats"]["attempted_matches"]
        valid_steps += row["family_valid"]
        for event in row["native"]["events"]:
            if event["rule_name"].startswith("e1_"):
                roles[table[event["rule_id"]]["priority"]] += 1
        if row["step"] in checkpoints:
            snap = next(inventories)
            audit_inventory({"inventory": snap})
            assert snap == inv
        if row["step"] % 128 == 0 or row["step"] == c["horizon"]:
            window = next(window_rows)
            wr = window["row"]
            assert window["end"] == row["step"] == wr["step_end"] and wr["step_start"] == previous_window_end
            assert wr["ei_unmeasured_reason"] is not None
            assert not wr["match_ceiling_binding"] and not wr["rule_region_ceiling_binding"]
            for field in ["installed", "retired", "quarantined"]:
                assert wr[f"rules_{field}"] == window_counts[field]
            assert wr["quarantine_by_validator"] == window_quarantine
            assert wr["encoded_meta_rewrite_events"] == window_counts["paid"]
            assert wr["meta_included"]["accepted_rewrites"] == window_counts["accepted"]
            assert wr["meta_included"]["attempted_matches"] == window_counts["attempted"]
            assert wr["anchor_survival_fraction"] == float(row["anchor_alive"])
            assert wr["runtime_activation_count"] == 0
            maintenance = wr["seeded_family_maintenance"]
            assert len(maintenance) == 1 and maintenance[0]["checks"] == row["step"] and maintenance[0]["valid_steps"] == valid_steps
            assert maintenance[0]["final_valid"] == row["family_valid"]
            for world in ["meta_included", "meta_excluded"]:
                for field in ["ei_micro", "ei_macro", "ce_best", "ce_best_strategy", "ce_structural", "ce_energetic", "ce_functional"]:
                    assert wr[world][field] is None
            window_counts.clear()
            window_quarantine = [0] * 9
            previous_window_end = row["step"]
        previous = row
    assert previous["step"] == c["horizon"], "short trajectory"
    assert next(inventories, None) is None
    if csv_file:
        assert csv_file.readline() == ""
        csv_file.close()
    if not c["identity"]:
        assert roles[0] and roles[1], "both oscillator roles must engage"
    assert next(window_rows, None) is None
    return {"id": c["id"], "steps": c["horizon"], "tamper_rejections": tamper, "world_core_events": roles[0], "world_proposal_events": roles[1], "probe_work": probe_work, "probe_results": probe_results}


def check_matrix(m, development):
    assert m["profile"] == PROFILE and m["protocol"] == "E2-witness-complete-v1"
    assert (m["family_version"], m["instruction_version"]) == (10, 11)
    assert m["development_only"] == development and not m["training_admitted"] and m["frozen_before_measurement"]
    expected = {f"{fixture}_{arm}" for fixture in ["identity_dormant", "oscillator_active"] for arm in ["A0", "A1"]}
    expected |= {f"{arm}_fuel{fuel}_owner{str(owner).lower()}" for arm in ["A2", "A3"] for fuel in [0, 2, 8] for owner in [False, True]}
    expected |= {f"anchor_loss_{arm}" for arm in ["A2", "A3"]}
    cs = m["cells"]
    assert len(cs) == 18 and {s["cell"]["id"] for s in cs} == expected
    for s in cs:
        c = s["cell"]
        gr3 = c["arm"] in ["A0", "A1"]
        assert c["gate"] == ("G-R3" if gr3 else "G-R5")
        assert c["seed"] == (42 if gr3 else 984927043621)
        assert c["horizon"] == (64 if development else 100000 if gr3 else 10000)
        assert c["development_only"] == development
        programs = load(Path(__file__).parents[1] / "docs/ere_complete_long_gates_v1.json")["programs"]
        for role, expected_role in [("core", "identity" if c["identity"] else "path_to_star"), ("proposal", "identity" if c["identity"] else "star_to_path")]:
            assert {k: s["program"][role][k] for k in ["priority", "labels", "lhs", "rhs", "guards"]} == {k: programs[expected_role][k] for k in ["priority", "labels", "lhs", "rhs", "guards"]}
        assert s["program"]["active"] == (not c["identity"])
        ops = s["instructions"]
        assert len(ops) == (1 if gr3 else 2) and len({node_key(o["id"]) for o in ops}) == len(ops)
        assert {o["operation"]: o["fuel"] for o in ops} == ({"Seal": 0.0} if gr3 else {"Seal": 1.0, "RepairSealed": c["repair_fuel"]})
        initial_nodes = {k: n["data"] for k, n in s["initial"]["nodes"]}
        for o in ops:
            assert prop(initial_nodes, o["id"], 8) == {"Int": 11}
            assert prop(initial_nodes, o["id"], 10) == {"Int": {"Seal": 3, "RepairSealed": 6}[o["operation"]]}
        if not gr3:
            assert prop(initial_nodes, s["clock"], CLOCK) == {"Int": 0}
            assert prop(initial_nodes, s["root"], MARKER) == {"Int": 0} and prop(initial_nodes, s["core"], MARKER) == {"Int": 1}
            assert sum(any(k == MARKER for k, _ in n["properties"]) for n in initial_nodes.values()) == 2
        assert any(e["label"] == BASE + 0x42 and e["nodes"] == [s["root"], s["core"]] for _, e in s["initial"]["edges"]), "paid preamble/CORE substitution"
        assert c["payload_steps"] == ([8, 16, 24, 56] if development else [2500, 5000, 7500, 9992])
        assert c["ownership_step"] == (40 if development else 9000)
        assert c["root_delete_step"] == (60 if development else 9996)
        assert c["step_mode"] == ("SequentialRandom" if gr3 else "MaxParallel") and c["max_pairings"] == (1 if gr3 else 16)
        if gr3:
            assert c["identity"] == c["id"].startswith("identity_") and c["id"].endswith(c["arm"])
            assert not c["ownership_loss"] and not c["root_loss"] and c["repair_fuel"] == 0
        elif c["root_loss"]:
            assert not c["identity"]
            assert c["id"] == f'anchor_loss_{c["arm"]}' and c["repair_fuel"] == 8 and not c["ownership_loss"]
        else:
            assert not c["identity"]
            assert c["id"] == f'{c["arm"]}_fuel{int(c["repair_fuel"])}_owner{str(c["ownership_loss"]).lower()}' and c["repair_fuel"] in [0, 2, 8]
        config = s["config"]
        assert config["arm"] == c["arm"] and config["measure_ei"] is False and config["measure_family_maintenance"] is True
        assert config["match_budget_policy"] == "Pooled" and config["match_ceiling"] == 50000 and config["rule_region_ceiling"] == 500
        assert config["meta_seed_energy"] == 0.0 and config["meta_rules_toml"] is None
        assert config["limits"] == load(Path(__file__).parents[1] / "docs/ere_complete_long_gates_v1.json")["meta_limits"]
        assert config["epoch"] == ("Never" if gr3 else {"Fixed": 1})
    design = load(Path(__file__).parents[1] / "docs/ere_complete_long_gates_v1.json")
    assert m["resources"] == design["proposed_resources"]


def audit(out, stage, development):
    m = integrity(out)
    check_matrix(m, development)
    cells = [s for s in m["cells"] if stage == "full" or s["cell"]["gate"] == "G-R3"]
    if stage == "full":
        stage_report = load(out / "audit_GR3.json")
        assert stage_report["audit_pass"] is True and stage_report["scope"] == "GR3" and stage_report["development_only"] == development
        barrier = load(out / "stage_barrier.json")
        assert barrier["G-R5"] == "not_started" and barrier["G-R3"] == "native and frozen stage audit passed"
        assert barrier["advancing_steps_before_G_R5"] == (256 if development else 400000)
        assert barrier["development_only"] == development and barrier["stage_report"] == "audit_GR3.json"
        assert len(barrier["native_cells"]) == 4
        assert {r["id"] for r in barrier["native_cells"]} == {s["cell"]["id"] for s in m["cells"] if s["cell"]["gate"] == "G-R3"}
        assert all(r["native_checks_pass"] and r["native_complete"] and r["completed_steps"] == (64 if development else 100000) for r in barrier["native_cells"])
    if (out / "run_status.json").exists():
        status = load(out / "run_status.json")
        assert status["status"] == "complete" and status["gate_pass"] == (not development)
        assert status["development_only"] == development and not status["training_admitted"]
        assert status["advancing_engine_steps"] == (1152 if development else 540000)
        assert status["elapsed_seconds"] < 3600 and status["auditor_seconds"] <= 900
        assert len(status["cells"]) == 18 and all(r["native_complete"] and r["native_checks_pass"] for r in status["cells"])
    results = [audit_cell(out, s) for s in cells]
    for fixture in ["identity_dormant", "oscillator_active"]:
        a, b = (out / f"{fixture}_{arm}" for arm in ["A0", "A1"])
        with (a / "trajectory.csv").open("rb") as x, (b / "trajectory.csv").open("rb") as y:
            while True:
                p, q = x.read(65536), y.read(65536)
                assert p == q, "static/frozen CSV divergence"
                if not p:
                    break
        # Arm-specific lifecycle is intentionally NOT compared here.
        for x, y in zip(stream(a / "steps.jsonl.gz"), stream(b / "steps.jsonl.gz"), strict=True):
            for k in ["native", "delta", "guards", "usage", "rng_before", "rng_after_compile", "rng_after", "motifs", "full_hash", "world_hash"]:
                assert x[k] == y[k], f"static/frozen native parity: {k}"
    return {"profile": PROFILE, "scope": stage, "audit_pass": True, "development_only": development,
            "gate_pass": not development and stage == "full", "training_admitted": False, "cells": results,
            "steps": sum(r["steps"] for r in results), "tamper_rejections": sum(r["tamper_rejections"] for r in results),
            "hash_rng_scope": "Rust witnesses graph hashes/RNG; Python replays typed native material and comparisons"}


def main():
    global DEADLINE
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("output", type=Path)
    p.add_argument("--stage", choices=["freeze", "GR3", "full", "seal"], default="full")
    p.add_argument("--development", action="store_true", help="audit only; cannot pass long gates")
    p.add_argument("--seconds", type=float, default=900)
    args = p.parse_args()
    assert 0 < args.seconds <= 900
    DEADLINE = time.monotonic() + args.seconds
    # RLIMIT_AS is not usable on macOS; streaming + an explicit resident check
    # enforce the portable bound. Linux also gets an OS allocation hard limit.
    if sys.platform != "darwin":
        resource.setrlimit(resource.RLIMIT_AS, (MEMORY, MEMORY))
    if args.stage == "freeze":
        freeze(args.output)
        print("frozen long-gate inputs; no steps launched")
    elif args.stage == "seal":
        integrity(args.output)
        save(args.output / "output_sha256.json", {str(p.relative_to(args.output)): digest(p) for p in safe_files(args.output)})
        print("sealed output inventory")
    else:
        report = audit(args.output, args.stage, args.development)
        target = args.output / f"audit_{args.stage}.json"
        if not target.exists():
            save(target, report)
        else:
            assert load(target) == report
        print(json.dumps({k: v for k, v in report.items() if k != "cells"}, sort_keys=True))


if __name__ == "__main__":
    main()
