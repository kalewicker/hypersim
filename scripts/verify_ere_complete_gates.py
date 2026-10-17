#!/usr/bin/env python3
"""Independent bounded gate audit: graph decoding, full canonical bytes, raw EI.

No engine execution, receipt repair, optimizer or admission writes. Hash integrity
is separate from semantic checking. The Rust producer remains the witness for
native execution and canonical graph hashes; Python independently checks the
serialized material and comparisons, not the Rust bincode executable.
"""
import argparse
from collections import Counter
import copy
from functools import lru_cache
import hashlib
import itertools
import json
import math
from pathlib import Path
import struct
import tomllib

from verify_ere_complete_witness import audit_inventory, complete_material, node_key, property_value

PROFILE = "E2-complete-deterministic-gates-v1"
PROTOCOL = "E2-witness-complete-v1"
CONTRACT_SHA = "7402409998f851e28e29cd87ec661bef8ab4039763076523a1ac04dbcc28c28c"
BASE = 0xFFF00000
OPS = ["Eq", "Ne", "Lt", "Gt", "Lte", "Gte"]
VALIDATORS = ["Region", "Sides", "Members", "Slots", "Variables", "Labels", "Guards", "Bounds", "Compilability"]
CASES = ["region_cap", "reverse_sides", "missing_ownership", "arity_cap", "bad_order", "bad_literal", "bad_guard", "pattern_cap", "bad_priority",
         "duplicate_ownership", "reverse_ownership", "wrong_schema", "missing_schema", "wrong_obligation", "missing_obligation", "duplicate_obligation", "owned_orphan", "unexpected_topology"] + [f"double_erasure_role{i:02}" for i in range(13)]


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def load(path):
    return json.loads(path.read_text())


def save(path, value):
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def freeze(out):
    assert not (out / "frozen_inputs_sha256.json").exists()
    assert not any(out.glob("G-R*.json")), "freeze must precede measurements"
    paths = [out / "manifest.json", out / "executed_test_binary", *sorted((out / "source_snapshots").rglob("*"))]
    checks = {str(p.relative_to(out)): digest(p) for p in paths if p.is_file()}
    assert checks["source_snapshots/docs/ere_complete_gate_harness_v1.md"] == CONTRACT_SHA
    save(out / "frozen_inputs_sha256.json", checks)


def bare(c):
    return {k: c[k] for k in ["priority", "labels", "lhs", "rhs", "guards"]}


def bytes_map(c, permutation, canonical):
    b = bytearray(b"E1RCF2")
    b.extend(struct.pack("<iB", c["priority"], len(permutation)))
    labels = [None] * len(permutation)
    for old, new in enumerate(permutation):
        labels[new] = c["labels"][old]
    for label in labels:
        b.extend(struct.pack("<BI", label is not None, label or 0))
    for side in ["lhs", "rhs"]:
        edges = [[permutation[v] for v in e] for e in c[side]]
        if canonical:
            edges.sort()
        b.extend(struct.pack("<I", len(edges)))
        for edge in edges:
            b.append(len(edge))
            b.extend(edge)
    guards = []
    for g in c["guards"]:
        gb = bytearray(struct.pack("<BBI", permutation[g["var"]], OPS.index(g["op"]), g["key"]))
        value = g["value"]
        if "Int" in value:
            gb.extend(struct.pack("<Bq", 0, value["Int"]))
        elif "Float" in value:
            gb.extend(struct.pack("<Bd", 1, value["Float"]))
        else:
            gb.extend(struct.pack("<BB", 2, value["Bool"]))
        guards.append(bytes(gb))
    if canonical:
        guards.sort()
    b.extend(struct.pack("<I", len(guards)))
    for guard in guards:
        b.extend(struct.pack("<I", len(guard)))
        b.extend(guard)
    return bytes(b)


@lru_cache(maxsize=256)
def canonical_bytes(serialized):
    c = json.loads(serialized)
    return min(bytes_map(c, p, True) for p in itertools.permutations(range(len(c["labels"]))))


def checked_candidate(c):
    rcf = canonical_bytes(json.dumps(bare(c), sort_keys=True))
    execution = bytes_map(c, list(range(len(c["labels"]))), False)
    assert c["rcf"] == rcf.hex() and c["execution"] == execution.hex()
    return rcf, execution


def pack(parts, tag):
    return tag + struct.pack("<I", len(parts)) + b"".join(struct.pack("<I", len(p)) + p for p in parts)


def checked_program(p):
    cs = [p["core"]] + ([p["proposal"]] if p["active"] else [])
    if p["proposal"] is not None:
        checked_candidate(p["proposal"])
    pairs = sorted(set(checked_candidate(c) for c in cs))
    canonical = sorted(set(a for a, _ in pairs))
    rcf = canonical[0] if len(canonical) == 1 else pack(canonical, b"E2-PROGRAM\0")
    execution = pack([b for _, b in pairs], b"E2-EXEC\0")
    assert p["rcf"] == rcf.hex() and p["execution"] == execution.hex()
    return [b.hex() for _, b in pairs]


def body_material(inv, root):
    """Extract one family from a graph with unrelated background/instructions."""
    root_key = node_key(root)
    body = {root_key} | {node_key(e["nodes"][1]) for _, e in inv["edges"] if e["label"] == BASE + 0x120 and node_key(e["nodes"][0]) == root_key}
    selected = {"nodes": [(k, n) for k, n in inv["nodes"] if k in body],
                "edges": [(k, e) for k, e in inv["edges"] if all(node_key(n) in body for n in e["nodes"])]}
    complete_material({"inventory": selected, "root": root})
    return selected


def graph_program(inv, root):
    material = body_material(inv, root)
    nodes = {k: n["data"] for k, n in material["nodes"]}
    edges = [(e["label"], [node_key(n) for n in e["nodes"]]) for _, e in material["edges"]]
    root_key = node_key(root)
    def prop(n, k):
        return property_value(nodes[n], k)
    def relations(label, parent):
        return [ns[1] for l, ns in edges if l == label and ns[0] == parent]
    def one(xs):
        assert len(xs) == 1
        return xs[0]
    def allele(anchor):
        sides = one([ns for l, ns in edges if l == BASE + 0x40 and ns[0] == anchor])
        assert len(sides) == 3 and nodes[sides[1]]["label"] == BASE + 2 and nodes[sides[2]]["label"] == BASE + 3
        pats = [relations(BASE + 0x41, side) for side in sides[1:]]
        variables = set()
        for pat in set(sum(pats, [])):
            variables.update(ns[1] for l, ns in edges if BASE + 0x50 <= l < BASE + 0x58 and ns[0] == pat)
        variables = sorted(variables, key=lambda v: prop(v, 6)["Int"])
        ix = {v: i for i, v in enumerate(variables)}
        result = {"priority": prop(anchor, 2)["Int"], "labels": [None] * len(ix), "lhs": [], "rhs": [], "guards": []}
        for side_index, (side, patterns) in enumerate(zip(["lhs", "rhs"], pats)):
            def hint(pat):
                properties = dict(nodes[pat]["properties"])
                return properties.get(7 if side_index else 6, properties[6])["Int"]
            for pat in sorted(patterns, key=hint):
                slots = sorted((l, ix[ns[1]]) for l, ns in edges if BASE + 0x50 <= l < BASE + 0x58 and ns[0] == pat)
                result[side].append([v for _, v in slots])
        for v in variables:
            literals = relations(BASE + 0x70, v)
            assert len(literals) <= 1
            if literals:
                result["labels"][ix[v]] = prop(literals[0], 0)["Int"]
            for guard in relations(BASE + 0x71, v):
                result["guards"].append({"var": ix[v], "key": prop(guard, 3)["Int"], "op": OPS[prop(guard, 4)["Int"]], "value": prop(guard, 5)})
        # Multiple guards are not exercised by this bounded harness. No guessed
        # hash-map tie order: compare guard sets for identity separately below.
        return result
    return {"core": allele(one(relations(BASE + 0x42, root_key))), "proposal": allele(one(relations(BASE + 0x43, root_key))), "active": bool(prop(root_key, 9)["Int"])}


def compare_graph(inv, root, expected):
    audit_inventory({"inventory": inv})
    actual = graph_program(inv, root)
    assert actual["active"] == expected["active"]
    for role in ["core", "proposal"]:
        assert actual[role] == bare(expected[role])
    checked_program(expected)


def audit_r0(g):
    assert g["gate"] == "G-R0" and g["pass"] is True and len(g["rows"]) == 20
    signatures = set()
    for i, row in enumerate(g["rows"]):
        assert row["index"] == i and row["pass"] is True
        vs = row["variants"]
        assert [v["name"] for v in vs] == ["base", "renamed", "priority"]
        assert [v["prefix"] for v in vs] == [0, i + 7, i + 13]
        for v in vs:
            assert v["input"] == v["decoded"]
            compare_graph(v["inventory"], v["root"], v["decoded"])
        assert vs[0]["decoded"]["rcf"] == vs[1]["decoded"]["rcf"] != vs[2]["decoded"]["rcf"]
        signatures.add(vs[0]["decoded"]["rcf"])
        orders = row["orders"]
        assert len(orders) == 2 and [v["reverse"] for v in orders] == [False, True]
        for order in orders:
            audit_inventory({"inventory": order["inventory"]})
            assert order["read_only"] and order["rng_before"] == order["rng_after"]
            assert order["report"]["quarantined"] == 0 and order["report"]["validated_regions"] == 2
            assert len(order["table"]) == len(order["report"]["classes"]) == 4
            pairs = [checked_candidate(c) for c in order["table"]]
            assert pairs == sorted(pairs)
        assert orders[0]["table"] == orders[1]["table"] and orders[0]["report"]["classes"] == orders[1]["report"]["classes"]
    assert len(signatures) == 20
    seal = g["paid_seal"]
    assert seal["pass"] and seal["fuel_before"] == 1 and seal["fuel_after"] == 0
    assert seal["before"] != seal["after"] and seal["decoded_before"] == seal["decoded_after"]
    for phase in ["before", "after"]:
        compare_graph(seal[phase], seal["root"], seal[f"decoded_{phase}"])
        assert property_value(dict(seal[phase]["nodes"])[node_key(seal["instruction"])]["data"], 1) == {"Float": seal[f"fuel_{phase}"]}


def audit_r4(g):
    assert g["gate"] == "G-R4" and g["pass"] and g["clean_pass"]
    audit_inventory({"inventory": g["base"]})
    complete_material({"inventory": g["base"], "root": g["root"]})
    assert len(g["clean"]["table"]) == 2 and g["clean"]["report"]["quarantine_by_validator"] == [0] * 9
    assert [r["case"] for r in g["cases"]] == CASES
    for i, row in enumerate(g["cases"]):
        expected = i if i < 9 else (7 if i == 17 else 2)
        assert row["expected_validator"] == row["actual_validator"] == VALIDATORS[expected]
        assert row["pass"] and row["synthetic_input"] and row["root"] == g["root"]
        assert row["inventory"] == row["after_inventory"]
        audit_inventory({"inventory": row["inventory"]})
        assert row["primed"]["table"] == g["clean"]["table"]
        if i >= 18:
            # Every role has its own exact payload+witness double erasure.
            role_index = i - 18
            roles = [BASE+0x42, BASE+0x43, BASE+0x41, *range(BASE+0x50, BASE+0x58), BASE+0x70, BASE+0x71]
            old = Counter(json.dumps(e, sort_keys=True) for _, e in g["base"]["edges"])
            new = Counter(json.dumps(e, sort_keys=True) for _, e in row["inventory"]["edges"])
            removed = [json.loads(e) for e in (old-new).elements()]
            added = [json.loads(e) for e in (new-old).elements()]
            assert len(removed) == len(added) == 2
            payload = next(e for e in removed if e["label"] == roles[role_index])
            witness = next(e for e in removed if e["label"] == BASE+0x100+role_index)
            assert witness["nodes"] == [row["root"], *payload["nodes"]]
            assert sorted(json.dumps(e, sort_keys=True) for e in added) == sorted(json.dumps(dict(e, label=0), sort_keys=True) for e in removed)
        for phase in ["after", "repeat"]:
            report = row[phase]["report"]
            counters = [0] * 9
            counters[expected] = 1
            assert report["quarantine_by_validator"] == counters and report["classes"] == []
            assert report["quarantined"] == 1 and report["validated_regions"] == 0
            assert report["retired"] == (1 if phase == "after" else 0) and report["installed"] == 0
            assert row[phase]["table"] == [] and report["changed"] == (phase == "after")
        # Completeness failures must be independently detectable in material.
        if expected == 2 and row["case"] != "owned_orphan":
            try:
                complete_material({"inventory": row["inventory"], "root": row["root"]})
            except AssertionError:
                pass
            else:
                raise AssertionError(f"independent material accepted {row['case']}")
        elif row["case"] == "owned_orphan":
            # It has valid declared role counts but an unreachable projected var.
            complete_material({"inventory": row["inventory"], "root": row["root"]})
            orphan_nodes = set(dict(row["inventory"]["nodes"])) - set(dict(g["base"]["nodes"]))
            assert len(orphan_nodes) == 1
            orphan = orphan_nodes.pop()
            incident = [e for _, e in row["inventory"]["edges"] if orphan in [node_key(n) for n in e["nodes"]]]
            assert len(incident) == 1 and incident[0]["label"] == BASE + 0x120
        elif i in [0, 3, 7]:
            field, value = [("max_region_nodes", 2), ("max_arity", 1), ("max_pattern_edges", 0)][[0, 3, 7].index(i)]
            assert row["limits"][field] == value and row["inventory"] == g["base"]
        elif i in [1, 4, 5, 6, 8, 17]:
            # Verify exact declared mutation, not just the producer's enum.
            assert row["inventory"] != g["base"]
            nodes = dict(row["inventory"]["nodes"])
            if i == 1:
                old = dict(g["base"]["edges"])
                differences = [(e, old[k]) for k, e in row["inventory"]["edges"] if e != old[k]]
                assert len(differences) == 1
                e, prior = differences[0]
                assert e["label"] == BASE + 0x40 and e["nodes"] == [prior["nodes"][0], prior["nodes"][2], prior["nodes"][1]]
            elif i != 17:
                label, key, value = {4: (5, 6, {"Int": 99}), 5: (6, 0, {"Bool": True}), 6: (7, 4, {"Int": 99}), 8: (8, 2, {"Bool": True})}[i]
                assert any(n["data"]["label"] == BASE + label and property_value(n["data"], key) == value for n in nodes.values())
            else:
                assert any(e["label"] == BASE + 0x777 for _, e in row["inventory"]["edges"])


def micro_ei(tpm):
    hashes = tpm["hashes"]
    assert len(set(hashes)) == len(hashes) and len(hashes) <= 512
    rows = {}
    seen = set()
    for a, b, n in tpm["counts"]:
        assert 0 <= a < len(hashes) and 0 <= b < len(hashes) and n > 0 and (a, b) not in seen
        seen.add((a, b))
        rows.setdefault(a, [0] * len(hashes))[b] += n
    assert sum(map(sum, rows.values())) == tpm["total_transitions"] >= 1
    dense = [[x / sum(row) for x in row] for _, row in sorted(rows.items())]
    entropy = lambda row: -sum(p * math.log2(p) for p in row if p > 0)
    if len(dense) <= 1:
        return 0.0
    mean = [sum(row[j] for row in dense) / len(dense) for j in range(len(hashes))]
    return entropy(mean) - sum(map(entropy, dense)) / len(dense)


def audit_r6(g):
    assert g["gate"] == "G-R6" and g["pass"] and [c["condition"] for c in g["controls"]] == ["stable", "activation", "metadata_seal"]
    for c in g["controls"]:
        condition = c["condition"]
        cfg = c["config"]
        assert c["pass"] and cfg["measure_ei"] and cfg["min_tpm_transitions"] == 1 and cfg["arm"] == "A3"
        assert cfg["epoch"] == {"Fixed": 1} and cfg["match_budget_policy"] == "Pooled" and cfg["match_ceiling"] == 50000
        assert cfg["meta_seed_energy"] == 0 and cfg["meta_rules_toml"] is None and cfg["vocabulary"]["protocol"] == PROTOCOL
        assert c["initial_fuel"] == (0 if condition == "stable" else 1)
        assert cfg["vocabulary"]["instructions"] == [{"operation": "Seal" if condition == "metadata_seal" else "WitnessActivate", "fuel": c["initial_fuel"]}]
        assert len(c["steps"]) == (96 if condition == "activation" else 64)
        audit_inventory({"inventory": c["initial"]})
        revisions, tables, paid_events = [], [], []
        previous = c["initial"]
        initial_world = sorted(json.dumps(e, sort_keys=True) for _, e in previous["edges"] if e["label"] < BASE)
        for i, row in enumerate(c["steps"], 1):
            obs = row["observation"]
            assert row["step"] == obs["step"] == row["inventory"]["step"] == i
            audit_inventory({"inventory": row["inventory"]})
            assert obs["rng_before_compile"] == obs["rng_after_compile"] == obs["rng_after_step"]
            assert not obs["match_ceiling_binding"] and row["fuel"] == 0
            assert row["usage"]["work"] <= 50000 and row["usage"]["results"] <= 50000
            assert obs["world_nodes"] == 3 and obs["world_edges"] == 2
            assert len(obs["families"]) == 1 and obs["families"][0]["valid"]
            family = obs["families"][0]
            root_data = dict(row["inventory"]["nodes"])[family["anchor"]]["data"]
            # Anchor is also recoverable from the schema-labelled node table.
            root = next(n for _, e in row["inventory"]["edges"] if e["label"] == BASE + 0x120 for n in e["nodes"][:1])
            assert node_key(root) == family["anchor"] and root_data["label"] == BASE + 1
            p = graph_program(row["inventory"], root)
            seed = cfg["seed_families"][0]
            assert not seed["active"]
            assert p["core"] == candidate_from_source(tomllib.loads(seed["core_toml"])["rules"][0])
            assert p["proposal"] == candidate_from_source(tomllib.loads(seed["proposal_toml"])["rules"][0])
            assert p["active"] == family["active"] == (condition == "activation")
            material_cs = [p["core"]] + ([p["proposal"]] if p["active"] else [])
            pair = sorted(set((canonical_bytes(json.dumps(bare(x), sort_keys=True)), bytes_map(x, list(range(len(x["labels"]))), False)) for x in material_cs))
            assert family["execution_bytes"] == pack([b for _, b in pair], b"E2-EXEC\0").hex()
            assert len(row["native"]["events"]) == row["native"]["stats"]["accepted_rewrites"]
            paid_events.extend((i, e) for e in row["native"]["events"] if not e["rule_name"].startswith("e1_"))
            assert property_value(dict(row["inventory"]["nodes"])[node_key(c["instruction"])]["data"], 1) == {"Float": 0.0}
            revisions.append(obs["rule_table_revision"])
            tables.append(obs["world_rule_table"])
            installed_cs = [p["core"]] + ([p["proposal"]] if condition == "activation" and i > 1 else [])
            installed = sorted((canonical_bytes(json.dumps(bare(x), sort_keys=True)), bytes_map(x, list(range(len(x["labels"]))), False)) for x in installed_cs)
            assert obs["world_rule_table"] == [b.hex() for _, b in installed]
            if condition != "activation":
                world = sorted(json.dumps(e, sort_keys=True) for _, e in row["inventory"]["edges"] if e["label"] < BASE)
                assert world == initial_world
            if i > 1:
                assert obs["world_hash_before"] == c["steps"][i-2]["observation"]["world_hash_after"]
            previous = row["inventory"]
        assert len(paid_events) == (0 if condition == "stable" else 1)
        if paid_events:
            assert paid_events[0][0] == 1 and ("seal" if condition == "metadata_seal" else "activate") in paid_events[0][1]["rule_name"].lower()
        if condition == "activation":
            assert tables[0] != tables[1] and all(t == tables[1] for t in tables[1:])
            assert revisions[0] != revisions[1] and all(r == revisions[1] for r in revisions[1:])
            assert len(tables[0]) == 1 and len(tables[1]) == 2
        else:
            assert all(t == tables[0] for t in tables) and all(r == revisions[0] for r in revisions)
            assert len(tables[0]) == 1
        assert [w["end"] for w in c["windows"]] == ([64, 96] if condition == "activation" else [64])
        for j, w in enumerate(c["windows"]):
            stable = condition != "activation" or j == 1
            r = w["row"]
            assert r["tpm_rule_set_stable"] == stable and r["step_start"] == (0 if j == 0 else 64) and r["step_end"] == w["end"]
            assert not r["match_ceiling_binding"] and not r["rule_region_ceiling_binding"]
            for variant in ["meta_included", "meta_excluded"]:
                measured = r[variant]
                ei = micro_ei(w["tpm"][variant])
                node_count = len(c["initial"]["nodes"]) if variant == "meta_included" else 3
                assert w["tpm"][variant]["total_transitions"] == node_count * (64 if j == 0 else 32)
                if stable:
                    assert r["ei_unmeasured_reason"] is None
                    assert measured["ei_micro"] is not None and measured["ei_macro"] is not None
                    assert math.isfinite(measured["ei_micro"]) and math.isfinite(measured["ei_macro"])
                    assert math.isclose(ei, measured["ei_micro"], abs_tol=1e-10)
                    assert measured["ei_micro"] > 0
                else:
                    assert r["ei_unmeasured_reason"] == "rule table changed within the measurement window (G-R6)"
                    assert all(measured[f] is None for f in ["ei_micro", "ei_macro", "ce_best", "ce_best_strategy", "ce_structural", "ce_energetic", "ce_functional"])


def outside_reason(raw):
    if raw.get("schedule") is not None:
        return "schedule"
    if not raw["enabled"] or raw["weight"] != 1:
        return "enabled/weight"
    if raw["lhs"]["edge_labels"] or any(v != 0 for v in raw["rhs"]["edge_labels"].values()):
        return "edge-label"
    for field, reason in [("lhs_var_guards", "distinctness"), ("rhs_property_effects", "property-effects")]:
        if raw[field]:
            return reason
    if any(raw[f] for f in ["lhs_node_property_compares", "lhs_node_degree_compares", "lhs_node_quantity_compares"]):
        return "derived/property-compare"
    # Other exclusions need explicit extension of this independent auditor.
    return None


def source_reason(raw):
    """First exclusion from the shipped TOML, not the producer's parsed row."""
    if raw.get("schedule") is not None:
        return "schedule"
    if not raw.get("enabled", True) or raw.get("weight", 1) != 1:
        return "enabled/weight"
    lhs, rhs = raw["lhs"], raw["rhs"]
    if lhs.get("edge_labels") or any(v != 0 for v in rhs.get("edge_labels", {}).values()):
        return "edge-label"
    if lhs.get("var_guards") or raw.get("lhs_var_guards"):
        return "distinctness"
    if rhs.get("property_effects") or raw.get("rhs_property_effects"):
        return "property-effects"
    if any(lhs.get(f) or raw.get(f"lhs_{f}") for f in ["node_property_compares", "node_degree_compares", "node_quantity_compares"]):
        return "derived/property-compare"
    return None


def candidate_from_source(raw):
    names = list(raw["lhs"]["nodes"])
    for n in raw["rhs"]["nodes"]:
        if n not in names:
            names.append(n)
    ix = {n: i for i, n in enumerate(names)}
    labels = [None] * len(names)
    for n, label in raw["lhs"].get("node_labels", {}).items():
        labels[ix[n]] = label
    for mapping in [raw["rhs"].get("node_labels", {}), raw.get("rhs_node_labels", {})]:
        for n, label in mapping.items():
            assert n not in raw["lhs"]["nodes"], "survivor relabel outside audited envelope"
            labels[ix[n]] = label
    guards = []
    for g in raw["lhs"].get("node_property_guards", []) + raw.get("lhs_node_property_guards", []):
        v = g["value"]
        tag = "Bool" if isinstance(v, bool) else "Int" if isinstance(v, int) else "Float"
        guards.append({"var": ix[g["var"]], "key": g["property"], "op": next(op for op in OPS if op.lower() == g["op"]), "value": {tag: v}})
    return {"priority": raw.get("priority", 0), "labels": labels,
            "lhs": [[ix[n] for n in e] for e in raw["lhs"]["edges"]],
            "rhs": [[ix[n] for n in e] for e in raw["rhs"]["edges"]], "guards": guards}


def audit_r2(g, out, full=True):
    horizon = 1000 if full else 4
    assert g["gate"] == "G-R2" and g["pass"] is full and g["horizon"] == horizon
    library = g["library"]
    paths = sorted((out / "source_snapshots/rules").glob("*.toml")) if full else []
    if full:
        assert [x["file"] for x in library] == [p.name for p in paths]
    expected, sources = [], {}
    for i, entry in enumerate(library):
        if full:
            assert entry["toml"] == paths[i].read_text()
        rules = tomllib.loads(entry["toml"])["rules"]
        expected.extend((entry["file"], i, r["name"]) for i, r in enumerate(rules))
        sources.update({(entry["file"], i): r for i, r in enumerate(rules)})
    assert [(r["file"], r["index"], r["name"]) for r in g["rows"]] == expected
    supported = 0
    reasons = Counter()
    for row in g["rows"]:
        assert row["pass"] is (full or row["status"] == "outside-envelope")
        reason = outside_reason(row["raw"])
        assert reason == source_reason(sources[row["file"], row["index"]])
        if row["status"] == "outside-envelope":
            assert reason == row["reason"] and reason is not None
            reasons[reason] += 1
            continue
        assert reason is None and row["status"] == "identical"
        supported += 1
        assert bare(row["candidate"]) == candidate_from_source(sources[row["file"], row["index"]])
        checked_candidate(row["candidate"])
        compare_graph(row["initial"], row["root"], row["program"])
        assert row["program"]["core"] == row["program"]["proposal"] == row["candidate"] and not row["program"]["active"]
        assert len(row["trace"]) == horizon
        accepted = 0
        for i, pair in enumerate(row["trace"], 1):
            assert pair["direct"] == pair["decoded"]
            leg = pair["direct"]
            assert leg["step"] == i and leg["nodes"] <= 2000 and leg["edges"] <= 4000 and not leg["ceiling"]
            assert leg["usage"]["work"] <= 2000000 and leg["usage"]["results"] <= 100000
            assert leg["rng_before"] == leg["rng_after"]
            accepted += leg["stats"]["accepted_rewrites"]
        assert accepted == row["engaged"] > 0
        for phase in ["direct_final", "decoded_final"]:
            audit_inventory({"inventory": row[phase]})
            assert row[phase]["step"] == horizon
            compare_graph(row[phase], row["root"], row["program"])
    assert supported == g["supported"] > 0
    return {"supported": supported, "total": len(expected), "files": len(library), "outside_reasons": dict(reasons)}


def audit_data(data, out):
    m = data["manifest"]
    assert m["profile"] == PROFILE and m["protocol"] == PROTOCOL
    assert m["family_version"] == 10 and m["instruction_version"] == 11 and m["frozen_before_measurement"]
    assert m["gates"] == ["G-R0", "G-R4", "G-R6", "G-R2"] and not m["training_admitted"]
    assert m["pending"] == ["G-R1", "G-R3", "G-R5", "optimizer stateability", "training admission"]
    assert (m["r0_families"], m["r4_cases"], m["r6_steps"], m["r2_steps_per_supported_rule"]) == (20, 31, 224, 1000)
    audit_r0(data["G-R0"])
    audit_r4(data["G-R4"])
    audit_r6(data["G-R6"])
    envelope = audit_r2(data["G-R2"], out)
    assert data["summary"]["profile"] == PROFILE and data["summary"]["pass"] and not data["summary"]["training_admitted"]
    assert all(g["pass"] for g in data["summary"]["gates"])
    return {"audit_pass": True, "profile": PROFILE, "r0_families": 20, "r4_cases": 31, "r6_steps": 224, "G-R2": envelope,
            "limitations": ["Macro EI presence/withholding checked; macro partitions not independently reconstructed", "Native execution and graph hash production remain Rust witnesses", "No G-R1/G-R3/G-R5 or training admission"]}


def tamper_tests(data, out):
    """Semantic mutations run after integrity verification, without editing files."""
    mutations = {
        "wrong_profile": lambda x: x["manifest"].update(profile="older-profile"),
        "missing_validator": lambda x: x["G-R4"]["cases"].pop(),
        "wrong_failure": lambda x: x["G-R4"]["cases"][0].update(actual_validator="Members"),
        "canonical_bytes": lambda x: x["G-R0"]["rows"][0]["variants"][0]["decoded"].update(rcf="00"),
        "allocation_order_rng": lambda x: x["G-R0"]["rows"][0]["orders"][0].update(rng_after="00"),
        "disabled_ei": lambda x: x["G-R6"]["controls"][0]["config"].update(measure_ei=False),
        "unmeasured_stable": lambda x: x["G-R6"]["controls"][0]["windows"][0]["row"]["meta_excluded"].update(ei_micro=None),
        "unsuppressed_ei": lambda x: x["G-R6"]["controls"][1]["windows"][0]["row"]["meta_included"].update(ei_micro=1),
        "forged_micro_ei": lambda x: x["G-R6"]["controls"][2]["windows"][0]["row"]["meta_included"].update(ei_micro=99),
        "no_payment": lambda x: x["G-R6"]["controls"][1]["steps"][0].update(fuel=1),
        "no_table_change": lambda x: x["G-R6"]["controls"][1]["steps"][1]["observation"].update(world_rule_table=x["G-R6"]["controls"][1]["steps"][0]["observation"]["world_rule_table"]),
        "missing_recovery": lambda x: x["G-R6"]["controls"][1]["windows"].pop(),
        "short_horizon": lambda x: x["G-R2"].update(horizon=999),
        "omitted_library_row": lambda x: x["G-R2"]["rows"].pop(),
        "false_exclusion": lambda x: next(r for r in x["G-R2"]["rows"] if r["status"] == "outside-envelope").update(reason="invented"),
        "diverged_paired_hash": lambda x: next(r for r in x["G-R2"]["rows"] if r["status"] == "identical")["trace"][0]["decoded"].update(full_hash="00"),
    }
    rejected = []
    for name, mutate in mutations.items():
        changed = copy.deepcopy(data)
        mutate(changed)
        try:
            audit_data(changed, out)
        except (AssertionError, KeyError, IndexError, TypeError, ValueError):
            rejected.append(name)
        else:
            raise AssertionError(f"tamper accepted: {name}")
    return rejected


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("out", type=Path)
    parser.add_argument("--freeze", action="store_true")
    parser.add_argument("--tamper-tests", action="store_true")
    parser.add_argument("--write-report", action="store_true")
    parser.add_argument("--development-preflight", action="store_true")
    args = parser.parse_args()
    out = args.out.resolve()
    if args.development_preflight:
        assert load(out / "development_only.json") == {"gate_pass": False, "r2_horizon": 4}
        audit_r0(load(out / "G-R0.json"))
        audit_r4(load(out / "G-R4.json"))
        audit_r6(load(out / "G-R6.json"))
        print(json.dumps({"development_checks_pass": True, "gate_pass": False, "G-R2": audit_r2(load(out / "G-R2.json"), out, full=False)}, indent=2))
        return
    if args.freeze:
        freeze(out)
        print("Inputs frozen before measurements")
        return
    checks = load(out / "frozen_inputs_sha256.json")
    assert checks["source_snapshots/docs/ere_complete_gate_harness_v1.md"] == CONTRACT_SHA
    for path, sha in checks.items():
        assert digest(out / path) == sha, f"frozen input changed: {path}"
    m = load(out / "manifest.json")
    assert {str(p.relative_to(out / "source_snapshots")) for p in (out / "source_snapshots").rglob("*") if p.is_file()} == set(m["sources"])
    data = {name: load(out / f"{name}.json") for name in ["manifest", "summary", "G-R0", "G-R4", "G-R6", "G-R2"]}
    report = audit_data(data, out)
    if args.tamper_tests:
        report["tamper_rejected"] = tamper_tests(data, out)
    if args.write_report:
        save(out / "independent_audit.json", report)
        paths = sorted(p for p in out.rglob("*") if p.is_file() and p.name != "SHA256SUMS")
        (out / "SHA256SUMS").write_text("".join(f"{digest(p)}  {p.relative_to(out)}\n" for p in paths))
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
