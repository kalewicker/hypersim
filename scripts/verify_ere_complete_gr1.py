#!/usr/bin/env python3
"""Frozen streaming complete-profile G-R1 auditor; no engine/admission writes.

Independently checks complete typed material, selector-to-edit transformations,
canonical program bytes, source retirement, native world deltas, classifications
and exact denominator/threshold accounting. Rust witnesses hash/RNG generation
and ChaCha draws; this is not a second engine or independent ChaCha implementation.
"""
import argparse
from collections import Counter, deque
import copy
import gzip
import hashlib
import json
from pathlib import Path
import re
import resource
import sys
import time

from verify_ere_complete_gates import (bare, bytes_map, canonical_bytes,
                                      checked_program, graph_program, pack, OPS as GUARD_OPS)
from verify_ere_complete_witness import audit_inventory, node_key, property_value, ROLES
from verify_ere_complete_long_gates import inventory, edge_key, check_binding

PROFILE = "E2-complete-mutation-classification-v1"
CONTRACT_MD_SHA = "1a6b302be5cfb050e92d66ba90ace25d99a27095b679d33c36d3614fda30a5be"
CONTRACT_JSON_SHA = "d1977ca3c14797c75ba5bd7cb9597c5cc2ca18b469bab632c2293c6dcf81defd"
BASE = 0xFFF00000
OPERATORS = ["AddSlot", "RemoveSlot", "RetargetSlot", "AddVar", "RemoveVar", "AddPatternEdge", "RemovePatternEdge", "AddLabel", "RemoveLabel", "ChangeLabel", "ChangePriority", "Activate", "Deactivate"]
VALIDATORS = ["Region", "Sides", "Members", "Slots", "Variables", "Labels", "Guards", "Bounds", "Compilability"]
DEADLINE = float("inf")
MEMORY = 1073741824
ARCHIVE_ROOT = None
OUTPUT_CAP = 4294967296


def guard():
    assert time.monotonic() < DEADLINE, "auditor wall cap"
    rss = resource.getrusage(resource.RUSAGE_SELF).ru_maxrss
    assert rss * (1 if sys.platform == "darwin" else 1024) <= MEMORY, "auditor RSS cap"


def digest(path):
    with path.open("rb") as f:
        return hashlib.file_digest(f, "sha256").hexdigest()


def load(path):
    return json.loads(path.read_text())


def save(path, value):
    assert not path.exists(), f"refusing overwrite: {path}"
    text = json.dumps(value, indent=2, sort_keys=True) + "\n"
    if ARCHIVE_ROOT is not None:
        assert sum(p.stat().st_size for p in files(ARCHIVE_ROOT)) + len(text.encode()) <= OUTPUT_CAP, "auditor output cap before write"
    path.write_text(text)


def files(out):
    paths = list(out.rglob("*"))
    assert not any(p.is_symlink() for p in paths), "symlink refused"
    return sorted(p for p in paths if p.is_file())


def check_seal(out):
    seal = out / "output_sha256.json"
    sealed = load(seal)
    # Only the root seal excludes itself. A nested basis receipt with the same
    # filename remains evidence and must be covered by this archive's seal.
    assert set(sealed) == {str(p.relative_to(out)) for p in files(out) if p != seal}
    for p, sha in sealed.items():
        assert digest(out / p) == sha, f"sealed output changed: {p}"


def stream(path):
    with gzip.open(path, "rt", encoding="utf-8", newline="") as f:
        while line := f.readline(8 * 1024 * 1024 + 1):
            guard()
            assert len(line) <= 8 * 1024 * 1024 and line.endswith("\n") and "\r" not in line
            row = json.loads(line)
            assert json.dumps(row, sort_keys=True, separators=(",", ":"), ensure_ascii=False) + "\n" == line, "noncanonical JSONL"
            yield row


def design(out):
    d = out / "source_snapshots/docs"
    assert digest(d / "ere_complete_gr1_v1.md") == CONTRACT_MD_SHA
    assert digest(d / "ere_complete_gr1_v1.json") == CONTRACT_JSON_SHA
    return load(d / "ere_complete_gr1_v1.json")


def check_manifest(m, d, development):
    assert m["profile"] == PROFILE and m["protocol"] == d["protocol"]
    assert m["family_version"] == 10 and m["parents"] == 168 and m["controls"] == 5376
    assert m["training_admitted"] is False and m["frozen_before_measurement"] is True
    assert m["development_only"] == development
    assert m["scientific_run_launched"] == (not development)
    assert m["resources"] == d["proposed_resources"]
    assert (m["draws"], m["horizon"]) == ((64, 8) if development else (10000, 100))
    assert m["development_resources"] == ({"wall": 300, "output": 536870912, "audit": 120, "steps": 1856} if development else None)
    assert len(m["sources"]) == len(set(m["sources"]))


def basis(out, d):
    b = d["basis"]
    pins = dict(b["design_time_source_sha256"])
    pins.update({f'{b["full_archive"]}/{name}': sha for name, sha in b["full_files_sha256"].items()})
    pins[f'{b["bounded_archive"]}/summary.json'] = b["bounded_summary_sha256"]
    pins[f'{b["bounded_archive"]}/independent_audit.json'] = b["bounded_audit_sha256"]
    pins[b["historical_classification"]] = b["historical_classification_sha256"]
    actual = {str(p.relative_to(out / "basis")) for p in files(out / "basis")}
    assert actual == set(pins)
    for rel, sha in pins.items():
        assert digest(out / "basis" / rel) == sha, f"basis changed: {rel}"
    for rel, sha in b["design_time_source_sha256"].items():
        assert digest(out / "source_snapshots" / rel) == sha, f"pinned source changed: {rel}"
    assert load(out / "basis" / b["full_archive"] / "run_status.json")["gate_pass"] is True
    assert load(out / "basis" / b["full_archive"] / "audit_full.json")["audit_pass"] is True


def integrity(out, development, freezing=False):
    d, m = design(out), load(out / "manifest.json")
    check_manifest(m, d, development)
    actual = sorted(str(p.relative_to(out / "source_snapshots")) for p in files(out / "source_snapshots"))
    assert actual == sorted(m["sources"])
    assert not any("__pycache__" in p or p.endswith(".pyc") for p in actual)
    basis(out, d)
    assert sum(p.stat().st_size for p in files(out)) <= (536870912 if development else 4294967296)
    if not freezing:
        frozen = load(out / "frozen_inputs_sha256.json")
        for name in ["manifest.json", "executed_test_binary", "parents.jsonl.gz", "draw_manifest.jsonl.gz", "control_manifest.jsonl.gz"]:
            assert name in frozen
        assert {p.removeprefix("source_snapshots/") for p in frozen if p.startswith("source_snapshots/")} == set(actual)
        for p, sha in frozen.items():
            assert (out / p).resolve().is_relative_to(out.resolve())
            assert digest(out / p) == sha, f"frozen input changed: {p}"
    if (out / "output_sha256.json").exists():
        check_seal(out)
    return m, d


def maps(inv):
    audit_inventory({"inventory": inv})
    return {k: v["data"] for k, v in inv["nodes"]}, dict(inv["edges"])


def handle(k):
    m = re.fullmatch(r"(?:Node|Edge)Id\((\d+)v(\d+)\)", k)
    assert m
    return tuple(map(int, m.groups()))


def ident(k):
    idx, version = handle(k)
    return {"idx": idx, "version": version}


def prop(nodes, n, key):
    return property_value(nodes[node_key(n) if isinstance(n, dict) else n], key)


def projected(nodes, edges):
    ns = {k: n for k, n in nodes.items() if n["label"] < BASE}
    es = {k: e for k, e in edges.items() if e["label"] < BASE and all(node_key(n) in ns for n in e["nodes"])}
    return ns, es


def world_fingerprint(nodes, edges):
    ns, es = projected(nodes, edges)
    # Ignore edge handles (allocation), but preserve typed ordinary node data,
    # endpoints/order/multiplicity. This supplements the Rust hash witness.
    value = [sorted(ns.items()), sorted(json.dumps(e, sort_keys=True) for e in es.values())]
    return hashlib.sha256(json.dumps(value, sort_keys=True).encode()).hexdigest()


class Invalid(Exception):
    def __init__(self, validator):
        self.validator = validator


def require(v, validator):
    if not v:
        raise Invalid(validator)


def walk(nodes, edges, root, component=False):
    require(root in nodes and nodes[root]["label"] in [BASE + 1, BASE + 8], "Region")
    incidence = {n: [] for n in nodes}
    for k, e in edges.items():
        for n in e["nodes"]:
            incidence[node_key(n)].append(k)
    seen, selected, q, work = {root}, {}, deque([(root, 0)]), 0
    while q:
        n, depth = q.popleft()
        for k in incidence[n]:
            work += 1
            require(work <= 4096, "Region")
            e = edges[k]
            if component and e["label"] in [BASE + 0x42, BASE + 0x43]:
                continue
            if e["label"] < BASE or k in selected:
                continue
            require(len(e["nodes"]) <= 3, "Region")
            selected[k] = e
            require(len(selected) <= 256, "Region")
            for ep in e["nodes"]:
                x = node_key(ep)
                require(x in nodes and nodes[x]["label"] >= BASE, "Region")
                if x not in seen:
                    require(depth < 5 and len(seen) < 128, "Region")
                    seen.add(x)
                    q.append((x, depth + 1))
    return seen, selected


def declared(nodes, n, key, maximum):
    values = [v for k, v in nodes[n]["properties"] if k == key]
    require(len(values) == 1 and set(values[0]) == {"Int"}, "Members")
    v = values[0]["Int"]
    require(type(v) is int and 0 <= v <= maximum, "Members")
    return v


def candidate_value(c):
    c = copy.deepcopy(c)
    c["rcf"] = canonical_bytes(json.dumps(c, sort_keys=True)).hex()
    c["execution"] = bytes_map(c, list(range(len(c["labels"]))), False).hex()
    return c


def program_value(p):
    p = {**p, "core": candidate_value(p["core"]), "proposal": candidate_value(p["proposal"])}
    candidates = [p["core"]] + ([p["proposal"]] if p["active"] else [])
    pairs = sorted(set((c["rcf"], c["execution"]) for c in candidates))
    canonical = sorted(set(bytes.fromhex(a) for a, _ in pairs))
    p["rcf"] = (canonical[0] if len(canonical) == 1 else pack(canonical, b"E2-PROGRAM\0")).hex()
    p["execution"] = pack([bytes.fromhex(b) for _, b in pairs], b"E2-EXEC\0").hex()
    checked_program(p)
    return p


def decode(inv, root):
    """Bounded independent decoder for this assay's explicit typed envelope."""
    nodes, all_edges = maps(inv)
    r = node_key(root)
    try:
        require(r in nodes and nodes[r]["label"] == BASE + 1 and len(nodes[r]["properties"]) <= 64, "Region")
        require(prop(nodes, r, 8) == {"Int": 10}, "Region")
        seen, edges = walk(nodes, all_edges, r)
        owners = set()
        for e in edges.values():
            if e["label"] == BASE + 0x120:
                ns = list(map(node_key, e["nodes"]))
                require(len(ns) == 2 and ns[0] == r and ns[1] != r and ns[1] not in owners, "Members")
                owners.add(ns[1])
        require(len(owners) == declared(nodes, r, 0x200, 128) and len(owners) + 1 == len(seen), "Members")
        counts = {n: [0] * 26 for n in seen}
        for n in seen:
            require(len(nodes[n]["properties"]) <= 64, "Region")
            require(declared(nodes, n, 0x201, 2**32 - 1) == nodes[n]["label"], "Members")
        triples, consumed, payloads, work = set(), set(), {}, 0
        incidence = {n: [] for n in nodes}
        for k, e in all_edges.items():
            for ep in e["nodes"]:
                incidence[node_key(ep)].append(k)
        for witness in edges.values():
            ix = witness["label"] - BASE - 0x100
            if not 0 <= ix < 13:
                continue
            ns = list(map(node_key, witness["nodes"]))
            require(len(ns) == 3 and ns[0] == r and ns[1] in seen and ns[2] in seen, "Members")
            _, a, b = ns
            require((ix, a, b) not in triples, "Members")
            triples.add((ix, a, b))
            matches = set()
            for k in incidence[a]:
                e = all_edges[k]
                work += 1
                require(work <= 4096, "Region")
                if e["label"] == ROLES[ix] and list(map(node_key, e["nodes"])) in [[a, b], [a, b, r]]:
                    matches.add(k)
            require(len(matches) == 1 and not matches & consumed, "Members")
            match = next(iter(matches))
            consumed.add(match)
            payloads[match] = {"label": ROLES[ix], "nodes": [ident(a), ident(b)]}
            counts[a][2 * ix] += 1
            counts[b][2 * ix + 1] += 1
        for n in seen:
            for ix, value in enumerate(counts[n]):
                require(declared(nodes, n, 0x220 + ix, 256) == value, "Members")
        projection = dict(payloads)
        for k, e in edges.items():
            if k in consumed or e["label"] == BASE + 0x120 or BASE + 0x100 <= e["label"] < BASE + 0x10D:
                continue
            require(e["label"] == BASE + 0x40 and len(e["nodes"]) == 3, "Bounds")
            projection[k] = e
        reachable, _ = walk(nodes, projection, r)
        require(reachable == seen, "Members")  # no owned orphan may disappear
        require(prop(nodes, r, 9) in [{"Int": 0}, {"Int": 1}], "Compilability")
        # All valid raw mutants retain writer ordering hints and <=1 Int guard
        # per allele. Reject unsupported material instead of guessing tie order.
        try:
            p = graph_program(inv, root)
        except (AssertionError, KeyError, IndexError, StopIteration) as exc:
            raise AssertionError("unsupported schema projection in G-R1 envelope") from exc
        for c in [p["core"], p["proposal"]]:
            require(type(c["priority"]) is int and -(2**31) <= c["priority"] < 2**31, "Compilability")
            require(1 <= len(c["labels"]) <= 8, "Variables")
            require(1 <= len(c["lhs"]) <= 16 and len(c["rhs"]) <= 16, "Bounds")
            require(all(2 <= len(e) <= 8 for e in c["lhs"] + c["rhs"]), "Slots")
            require(all(l is None or type(l) is int and 0 <= l < 2**32 for l in c["labels"]), "Labels")
            left = {v for e in c["lhs"] for v in e}
            require(all(g["var"] in left for g in c["guards"]), "Guards")
            assert len(c["guards"]) <= 1 and all(set(g["value"]) == {"Int"} for g in c["guards"]), "unsupported guard envelope"
        return {"program": program_value(p), "validator": None}
    except Invalid as exc:
        return {"program": None, "validator": exc.validator}


class Builder:
    """Execute only expected primitive edits, consuming every archived action."""
    def __init__(self, inv, actions):
        self.nodes, self.edges = map(copy.deepcopy, maps(inv))
        self.actions = iter(actions)
        self.history = {"node": {handle(k)[0]: handle(k)[1] for k in self.nodes},
                        "edge": {handle(k)[0]: handle(k)[1] for k in self.edges}}

    def take(self, op, **fields):
        a = next(self.actions)
        assert a["op"] == op
        assert {k: v for k, v in a.items() if k != "id"} == {"op": op, **fields}
        return a["id"]

    def fresh(self, id, kind, data):
        key = node_key(id) if kind == "node" else edge_key(id)
        live = self.nodes if kind == "node" else self.edges
        assert key not in live
        idx, version = handle(key)
        assert idx > 0 and version > 0 and version % 2 == 1
        assert version > self.history[kind].get(idx, 0)
        self.history[kind][idx] = version
        live[key] = copy.deepcopy(data)
        return id

    def add_node(self, data):
        return self.fresh(self.take("add_node", data=data), "node", data)

    def add_edge(self, label, ns):
        data = {"label": label, "nodes": ns}
        assert all(node_key(n) in self.nodes for n in ns)
        return self.fresh(self.take("add_edge", data=data), "edge", data)

    def delete_edge(self, id):
        actual = self.take("delete_edge")
        assert actual == id
        del self.edges[edge_key(id)]

    def delete_node(self, id):
        actual = self.take("delete_node")
        assert actual == id
        key = node_key(id)
        del self.nodes[key]
        self.edges = {k: e for k, e in self.edges.items() if all(node_key(n) != key for n in e["nodes"])}

    def property(self, id, key, value, op="set_property"):
        actual = self.take(op, key=key, **({"value": value} if op != "delete_property" else {}))
        assert actual == id
        props = self.nodes[node_key(id)]["properties"]
        if op == "delete_property":
            props[:] = [[k, v] for k, v in props if k != key]
        elif op == "append_property":
            props.append([key, value])
        else:
            for kv in props:
                if kv[0] == key:
                    kv[1] = value
                    break
            else:
                props.append([key, value])

    def label(self, id, label):
        assert self.take("node_label", label=label) == id
        self.nodes[node_key(id)]["label"] = label

    def finish(self, step=0):
        assert next(self.actions, None) is None, "hidden edit/repair action"
        return inventory(self.nodes, self.edges, step)


def node_data(label, properties=None):
    return {"label": label, "generation": 0, "energy": 0.0, "properties": properties or []}


def allele(nodes, edges, root, role):
    link = BASE + (0x42 if role == "Core" else 0x43)
    anchor = next(e["nodes"][1] for e in edges.values() if e["label"] == link and e["nodes"][0] == root and len(e["nodes"]) == 2)
    sides = next(e["nodes"][1:] for e in edges.values() if e["label"] == BASE + 0x40 and e["nodes"][0] == anchor)
    pats = [e["nodes"][1] for e in edges.values() if e["label"] == BASE + 0x41 and e["nodes"][0] in sides and len(e["nodes"]) == 2]
    vs = [e["nodes"][1] for e in edges.values() if BASE + 0x50 <= e["label"] < BASE + 0x70 and e["nodes"][0] in pats and len(e["nodes"]) == 2]
    body = {node_key(n) for n in [anchor] + sides + pats + vs}
    body.update(node_key(e["nodes"][1]) for e in edges.values() if e["label"] in [BASE + 0x70, BASE + 0x71] and e["nodes"][0] in vs and len(e["nodes"]) == 2)
    return anchor, [ident(k) for k in sorted(body, key=handle)]


def graph_delta(before, after):
    a, b = maps(before), maps(after)
    return {kind: {"deleted": sorted(set(x) - set(y)), "upsert": [[k, v] for k, v in sorted(y.items()) if x.get(k) != v]}
            for kind, x, y in zip(["nodes", "edges"], a, b)}


def topology(inv):
    ns, es = maps(inv)
    return sorted((k, n["label"]) for k, n in ns.items()), sorted(json.dumps(e, sort_keys=True) for e in es.values())


def metadata(inv):
    return [(k, n["data"]["properties"]) for k, n in inv["nodes"]]


def raw_edit(parent, d):
    op = d["operator_index"]
    assert 0 <= op < 13 and d["operator"] == OPERATORS[op]
    assert d["role"] in (["Core", "Proposal"] if op < 11 else ["Family"])
    inv, root = parent["inventory"], parent["root"]
    edit = d["edit"]
    b = Builder(inv, edit["actions"])
    choices = iter(edit["choices"])

    def choose(name, eligible):
        c = next(choices)
        assert c["name"] == name and c["eligible"] == eligible
        i = c["index"]
        assert type(i) is int and 0 <= i < max(1, len(eligible))
        value = eligible[i] if eligible else None
        assert c["selected"] == value
        return value

    def scalar(name, bound):
        c = next(choices)
        assert c["name"] == name and c["bound"] == bound and 0 <= c["index"] < bound
        return c["index"]

    applicable = False
    if op >= 11:
        applicable = True
        b.property(root, 9, {"Int": int(op == 11)})
    else:
        anchor, body = allele(b.nodes, b.edges, root, d["role"])
        vs = [n for n in body if b.nodes[node_key(n)]["label"] == BASE + 5]
        pats = [n for n in body if b.nodes[node_key(n)]["label"] == BASE + 4]
        var = choose("variable", vs)
        pat = choose("pattern", pats) if var is not None else None
        if var is not None and pat is not None:
            slots = [ident(k) for k, e in sorted(b.edges.items(), key=lambda kv: (kv[1]["label"], handle(kv[0]))) if len(e["nodes"]) == 2 and e["nodes"][0] == pat and BASE + 0x50 <= e["label"] < BASE + 0x70]
            slot = choose("slot", slots)
            free = next((k for k in range(9) if not any(b.edges[edge_key(s)]["label"] == BASE + 0x50 + k for s in slots)), None)
            if op == 0 and free is not None:
                applicable = True
                b.add_edge(BASE + 0x50 + free, [pat, var])
            elif op == 1 and slot is not None:
                applicable = True
                b.delete_edge(slot)
            elif op == 2 and slot is not None:
                applicable = True
                label = b.edges[edge_key(slot)]["label"]
                b.delete_edge(slot)
                b.add_edge(label, [pat, var])
            elif op == 3 and len(vs) < 8:
                applicable = True
                hint = next(i for i in range(8) if not any(prop(b.nodes, v, 6) == {"Int": i} for v in vs))
                n = b.add_node(node_data(BASE + 5, [[6, {"Int": hint}]]))
                if free is not None:
                    b.add_edge(BASE + 0x50 + free, [pat, n])
            elif op == 4:
                applicable = True
                b.delete_node(var)
            elif op == 5:
                sides = [n for n in body if b.nodes[node_key(n)]["label"] in [BASE + 2, BASE + 3]]
                side = choose("side", sides)
                if side is not None:
                    applicable = True
                    p = b.add_node(node_data(BASE + 4))
                    b.add_edge(BASE + 0x41, [side, p])
                    b.add_edge(BASE + 0x50, [p, var])
                    v2 = choose("second_variable", vs)
                    if v2 is not None:
                        b.add_edge(BASE + 0x51, [p, v2])
            elif op == 6:
                applicable = True
                b.delete_node(pat)
            elif op == 7 and not any(e["label"] == BASE + 0x70 and e["nodes"][0] == var and len(e["nodes"]) == 2 for e in b.edges.values()):
                applicable = True
                n = b.add_node(node_data(BASE + 6, [[0, {"Int": scalar("label_value", 3)}]]))
                b.add_edge(BASE + 0x70, [var, n])
            elif op in [8, 9]:
                lits = sorted([e["nodes"][1] for e in b.edges.values() if e["label"] == BASE + 0x70 and e["nodes"][0] == var and len(e["nodes"]) == 2], key=lambda n: (n["idx"], n["version"]))
                lit = choose("literal", lits)
                if lit is not None:
                    applicable = True
                    if op == 8:
                        b.delete_node(lit)
                    else:
                        b.property(lit, 0, {"Int": scalar("label_value", 3)})
            elif op == 10:
                applicable = True
                b.property(anchor, 2, {"Int": prop(b.nodes, anchor, 2)["Int"] + 1})
    assert next(choices, None) is None, "unaccounted selector draw"
    after = b.finish()
    assert edit["applicable"] == applicable
    assert edit["delta"] == graph_delta(inv, after)
    assert edit["raw_changed"] == (inv != after)
    assert edit["logical_topology_changed"] == (topology(inv) != topology(after))
    assert edit["metadata_changed"] == (metadata(inv) != metadata(after))
    return after


def control_edit(parent, a):
    inv, root, ix = parent["inventory"], parent["root"], parent["index"]
    b = Builder(inv, a["actions"])
    body = sorted([e["nodes"][1] for e in b.edges.values() if e["label"] == BASE + 0x120 and e["nodes"][0] == root], key=lambda n: (n["idx"], n["version"]))
    selected, key = body[ix % len(body)], 0x220 + ix % 26
    relation = None
    for offset in range(13):
        role = (ix % 13 + offset) % 13
        pairs = sorted([(k, e) for k, e in b.edges.items() if e["label"] == ROLES[role] and len(e["nodes"]) == 2 and e["nodes"][0] != e["nodes"][1]], key=lambda kv: handle(kv[0]))
        if pairs:
            payload, e = pairs[0]
            witness = next(k for k, w in b.edges.items() if w["label"] == BASE + 0x100 + role and w["nodes"] == [root] + e["nodes"])
            relation = role, ident(payload), ident(witness)
            break
    assert relation is not None
    role, payload, witness = relation
    ownership = ident(next(k for k, e in b.edges.items() if e["label"] == BASE + 0x120 and e["nodes"] == [root, selected]))
    assert a["parent"] == ix and a["body"] == selected and a["obligation_key"] == key
    assert a["relation_role_index"] == role and a["payload"] == payload and a["witness"] == witness and a["ownership"] == ownership
    assert a["obligation_zero"] == (prop(b.nodes, selected, key) == {"Int": 0})
    name = a["attack"]
    property_target = next(((prefix, node, k) for prefix, node, k in [("owned_count_", root, 0x200), ("schema_", selected, 0x201), ("obligation_", selected, key)] if name.startswith(prefix)), None)
    if property_target:
        prefix, n, k = property_target
        kind, old = name[len(prefix):], prop(b.nodes, n, k)
        if kind == "delete":
            b.property(n, k, None, "delete_property")
        elif kind == "duplicate":
            b.property(n, k, old, "append_property")
        else:
            value = {"Float": 0.0} if kind == "mistype" else {"Int": 0 if kind == "wrong_label" else old["Int"] + 1}
            b.property(n, k, value)
    elif name in ["body_delete", "root_delete"]:
        b.delete_node(selected if name == "body_delete" else root)
    elif name in ["body_relabel", "root_relabel"]:
        b.label(selected if name == "body_relabel" else root, 0)
    elif name == "encoding_wrong_version":
        b.property(root, 8, {"Int": 3})
    elif name == "activation_mistype":
        b.property(root, 9, {"Float": 0.0})
    elif name == "owned_orphan":
        n = copy.deepcopy(b.nodes[node_key(selected)])
        for kv in n["properties"]:
            if 0x220 <= kv[0] < 0x23A:
                kv[1] = {"Int": 0}
        n = b.add_node(n)
        b.add_edge(BASE + 0x120, [root, n])
        b.property(root, 0x200, {"Int": prop(b.nodes, root, 0x200)["Int"] + 1})
    elif name == "double_erasure":
        for id in [payload, witness]:
            ns = b.edges[edge_key(id)]["nodes"]
            b.delete_edge(id)
            b.add_edge(0, ns)
    else:
        prefix = next(p for p in ["ownership_", "witness_", "payload_"] if name.startswith(p))
        id = {"ownership_": ownership, "witness_": witness, "payload_": payload}[prefix]
        old = b.edges[edge_key(id)]
        kind = name[len(prefix):]
        if kind == "delete":
            b.delete_edge(id)
        elif kind == "duplicate":
            b.add_edge(old["label"], old["nodes"])
        else:
            ns, label = list(old["nodes"]), old["label"]
            if kind == "relabel":
                label = 0
            elif kind == "reverse":
                offset = int(prefix == "witness_")
                ns[offset], ns[offset + 1] = ns[offset + 1], ns[offset]
            else:
                assert kind == "retarget_foreign"
                ns[1] = b.add_node(b.nodes[node_key(selected)])
            b.delete_edge(id)
            b.add_edge(label, ns)
    return b.finish()


def expected_parent(index):
    shapes = [([[0, 1]], [[0, 1]]), ([[0, 1]], [[0, 2], [1, 2]]),
              ([[0, 1, 2]], [[0, 1], [1, 2]]), ([[0, 1], [1, 2]], [[0, 2]]),
              ([[0, 1], [1, 2]], [[0, 1, 2]]),
              ([[0, 1, 2]], [[0, 1, 3], [1, 2, 3], [0, 2, 3]]),
              ([[0, 1, 0, 1, 0, 1, 0, 1]], [[0, 1, 0, 1, 0, 1, 0, 1]])]
    base, variant, active = index // 6, (index // 2) % 3, bool(index % 2)
    lhs, rhs = shapes[base // 4]
    n = max(v for e in lhs + rhs for v in e) + 1
    labels = [None] * n
    if base % 4 & 1:
        labels[0] = 0
    guards = [{"var": 0, "key": 1, "op": "Gte", "value": {"Int": 0}}] if base % 4 & 2 else []
    core = {"priority": 0, "labels": labels, "lhs": lhs, "rhs": rhs, "guards": guards}
    proposal = copy.deepcopy(core)
    proposal["priority"] = 1
    if variant == 1:
        proposal["labels"][0] = 1
    elif variant == 2:
        proposal["rhs"].append([0, 0])
    return program_value({"core": core, "proposal": proposal, "active": active})


def check_parent(p, i):
    assert p["index"] == i and p["shape"] == i // 24
    assert p["program"] == expected_parent(i)
    assert decode(p["inventory"], p["root"]) == {"program": p["program"], "validator": None}
    nodes, edges = maps(p["inventory"])
    ns, es = projected(nodes, edges)
    world = inventory(ns, es, 0)
    assert world == p["world"]
    c = p["program"]["core"]
    n = len(c["labels"])
    assert len(ns) == n + 64 and len(es) == len(c["lhs"]) + 96
    for j in range(n + 64):
        assert ns[f"NodeId({j+1}v1)"] == node_data((c["labels"][j] or 0) if j < n else 0, [[1, {"Int": 100}]])
    expected = c["lhs"] + [[n + j % 64, n + (j + 1) % 64] for j in range(96)]
    for j, e in enumerate(expected):
        assert es[f"EdgeId({j+1}v1)"] == {"label": 0, "nodes": [{"idx": v + 1, "version": 1} for v in e]}


def check_table(table, p, engine=False):
    candidates = [] if p is None else [p["core"]] + ([p["proposal"]] if p["active"] else [])
    cs = sorted({c["execution"]: c for c in candidates}.values(), key=lambda c: (c["rcf"], c["execution"]))
    assert len(table) == len(cs)
    for r, c in zip(table, cs):
        left = sorted({v for e in c["lhs"] for v in e})
        right = sorted({v for e in c["rhs"] for v in e})
        lm, rm = {v: i for i, v in enumerate(left)}, {v: i for i, v in enumerate(right)}
        assert r["enabled"] is True and r["weight"] == 1.0 and r["schedule"] is None
        assert r["meta_visible"] is False and r["priority"] == c["priority"]
        assert re.fullmatch(r"e1_[0-9a-f]{16}", r["name"])
        assert r["lhs_var_names"] == [f"v{v}" for v in left]
        want = {"n_vars": len(left), "node_label_constraints": [c["labels"][v] for v in left],
                "node_label_ceiling": BASE, "edges": [{"var_ixs": [lm[v] for v in e], "label": None, "ordered": False} for e in c["lhs"]],
                "property_guards": [{"var_ix": lm[g["var"]], "key": g["key"], "op": g["op"], "value": g["value"]} for g in c["guards"]],
                "property_compares": [], "degree_compares": [], "quantity_compares": [], "distinct_pairs": [],
                "match_work_limit": 400000 if engine else None, "match_result_limit": 50000 if engine else None}
        assert r["pattern"] == want, "compiled pattern does not match decoded candidate"
        assert r["template"] == {"n_rhs_vars": len(right), "rhs_from_lhs": [lm.get(v) for v in right],
                                 "rhs_new_labels": [0 if v in lm else (c["labels"][v] or 0) for v in right],
                                 "rhs_relabels": [None] * len(right), "rhs_edges": [[[rm[v] for v in e], 0] for e in c["rhs"]],
                                 "lhs_deleted": [lm[v] for v in left if v not in rm], "property_effects": []}


def check_report(compiled, p, validator=None, retired=0, absent=False, engine=False, step=0):
    check_table(compiled["table"], p, engine)
    r = compiled["report"]
    assert r["step"] == step and r["installed"] == int(p is not None) and r["retired"] == retired
    assert r["validated_regions"] == int(p is not None) and r["quarantined"] == int(p is None and not absent)
    counts = [0] * 9
    if p is None and not absent:
        assert validator in VALIDATORS
        counts[VALIDATORS.index(validator)] = 1
    assert r["quarantine_by_validator"] == counts
    assert r["classes"] == [int(rule["name"][3:], 16) for rule in compiled["table"]]
    assert r["encoded_meta_operations"] == [] and r["changed"] is True


def check_control(parent, a, r):
    assert r["parent"] == parent["index"] and r["attack"] == a["attack"] and r["manifest"] == a
    after = control_edit(parent, a)
    result = decode(after, parent["root"])
    assert result["program"] is None and r["result"] == result
    assert r["clean_decoded"] == parent["program"] and r["native_pass"] is True and r["read_only"] is True
    check_report(r["clean"], parent["program"])
    absence = a["attack"] in ["root_delete", "root_relabel"]
    validator = "Compilability" if a["attack"] == "encoding_wrong_version" else result["validator"]
    check_report(r["compiled"], None, validator, retired=1, absent=absence, step=1)
    assert r["delta"] == graph_delta(parent["inventory"], after)
    nodes, _ = maps(after)
    assert r["root_alive"] == (node_key(parent["root"]) in nodes)
    assert r["root_discovered"] == (node_key(parent["root"]) in nodes and nodes[node_key(parent["root"])]["label"] == BASE + 1)
    return after


def check_usage(row, table):
    u, guards = row["usage"], dict(row["guards"])
    expected_guard_names = {table[r["rule_id"]]["name"] for r in u["rules"] if table[r["rule_id"]]["pattern"]["property_guards"]}
    assert set(guards) == expected_guard_names
    assert len(guards) == len(row["guards"]), "duplicate guard record"
    assert u["work"] <= 400000 and u["results"] <= 50000
    assert len(u["rules"]) <= len(table)
    work = results = 0
    for i, r in enumerate(u["rules"]):
        assert r["rule_id"] == i
        assert r["work_allowance"] == 400000 - work and r["result_allowance"] == 50000 - results
        assert r["work"] <= r["work_allowance"] and r["results"] <= r["result_allowance"]
        work += r["work"]
        results += r["results"]
        if table[i]["name"] in guards:
            g = guards[table[i]["name"]]
            assert g["enumeration_work"] == r["work"] and g["enumeration_truncated"] == r["truncated"]
            assert len(g["property_guards"]) == len(table[i]["pattern"]["property_guards"])
            assert not any(g[k] for k in ["property_compares", "degree_compares", "quantity_compares"])
            assert all(0 <= success <= attempts for attempts, success in g["property_guards"])
        assert not r["truncated"] or i == len(u["rules"]) - 1
    assert (work, results) == (u["work"], u["results"])
    assert u["truncated"] == row["ceiling"] == any(r["truncated"] for r in u["rules"])
    if not row["ceiling"]:
        assert len(u["rules"]) == len(table)
    assert row["native"]["stats"]["attempted_matches"] == results


def replay_step(row, table, old_nodes, old_edges):
    nodes, edges = copy.deepcopy(old_nodes), copy.deepcopy(old_edges)
    created_nodes = dict(row["delta"]["nodes"]["upsert"])
    created_edges = dict(row["delta"]["edges"]["upsert"])
    totals = Counter()
    assert len(row["native"]["events"]) <= 1  # max_pairings=1, no creation schedules
    for event in row["native"]["events"]:
        r = table[event["rule_id"]]
        assert event["rule_name"] == r["name"]
        b, matched, t = event["bound_nodes"], event["matched_edges"], r["template"]
        check_binding(r, b, matched, old_nodes, old_edges)
        erased = list(map(edge_key, matched))
        for k in erased:
            del edges[k]
        removed, neighbors = [], set()
        for ix in t["lhs_deleted"]:
            n = node_key(b[ix])
            if n not in nodes:
                continue
            assert nodes[n]["label"] < BASE
            # Store incidence repeats edge IDs for repeated endpoints.
            for k, e in list(edges.items()):
                multiplicity = sum(node_key(ep) == n for ep in e["nodes"])
                if multiplicity:
                    erased.extend([k] * multiplicity)
                    neighbors.update(node_key(ep) for ep in e["nodes"] if node_key(ep) != n)
                    del edges[k]
            del nodes[n]
            removed.append(b[ix])
        assert event["nodes_deleted"] == removed
        assert Counter(map(edge_key, event["edges_deleted"])) == Counter(erased)
        assert set(map(node_key, event["cleanup_neighbors"])) == neighbors
        assert event["vanished_energy"] == 0.0  # fixed Int substrate/new nodes
        created = iter(event["nodes_created"])
        rhs = []
        for i, ix in enumerate(t["rhs_from_lhs"]):
            if ix is not None:
                rhs.append(b[ix])
            else:
                n = next(created)
                key = node_key(n)
                assert key not in nodes
                data = node_data(t["rhs_new_labels"][i])
                data["generation"] = row["step"] - 1
                assert created_nodes[key] == data
                nodes[key] = data
                rhs.append(n)
        assert next(created, None) is None
        produced = iter(event["edges_created"])
        for indices, label in t["rhs_edges"]:
            ns = [rhs[i] for i in indices]
            if all(node_key(n) in nodes for n in ns):
                id = next(produced)
                key = edge_key(id)
                assert key not in edges
                data = {"label": label, "nodes": ns}
                assert created_edges[key] == data
                edges[key] = data
        assert next(produced, None) is None
        totals["nodes_created"] += len(event["nodes_created"])
        totals["nodes_deleted"] += len(removed)
        totals["edges_created"] += len(event["edges_created"])
        totals["edges_deleted"] += len(erased)
    stats = row["native"]["stats"]
    assert stats["accepted_rewrites"] == len(row["native"]["events"])
    assert stats["property_effects_applied"] == stats["property_effects_skipped"] == 0
    for k in ["nodes_created", "nodes_deleted", "edges_created", "edges_deleted"]:
        assert stats[k] == totals[k]
    for kind, a, b in [("nodes", old_nodes, nodes), ("edges", old_edges, edges)]:
        want = {"deleted": sorted(set(a) - set(b)), "upsert": [[k, v] for k, v in sorted(b.items()) if a.get(k) != v]}
        assert row["delta"][kind] == want, f"unwitnessed native {kind} delta"
    return nodes, edges


def consume_trace(rows, id, initial, parent, horizon):
    start = next(rows)
    assert start["kind"] == "start" and start["id"] == id and start["initial"] == initial
    assert start["root"] == parent["root"] and start["horizon"] == horizon
    result = decode(initial, parent["root"])
    assert result["program"] is not None and start["decoded"] == result
    check_report({"table": start["table"], "report": start["compile_report"]}, result["program"], engine=True)
    table = start["table"]
    nodes, edges = maps(initial)
    metadata_nodes = {k: copy.deepcopy(n) for k, n in nodes.items() if n["label"] >= BASE}
    metadata_edges = {k: copy.deepcopy(e) for k, e in edges.items() if e["label"] >= BASE}
    ns, es = projected(nodes, edges)
    initial_counts = len(ns), len(es)
    frames, rngs, fingerprints = [], [], []
    rng, before_full, before_world = start["initial_rng"], start["initial_full_hash"], start["initial_world_hash"]
    assert re.fullmatch(r"[0-9a-f]{16}", before_full) and re.fullmatch(r"[0-9a-f]{16}", before_world)
    cap, exploding = None, False
    attempted = advancing = work = results = peak_work = peak_results = 0
    while True:
        row = next(rows)
        assert row["id"] == id
        if row["kind"] == "end":
            break
        assert row["kind"] == "step" and cap is None
        assert row["step"] == attempted + 1 and attempted < horizon
        attempted += 1
        check_usage(row, table)
        for key in ["rng_before", "rng_after", "full_hash_before", "full_hash", "world_hash_before", "world_hash"]:
            assert re.fullmatch(r"[0-9a-f]{16}", row[key])
        assert row["world_hash"] != row["full_hash"] and row["world_hash_before"] != row["full_hash_before"], "full-meta hash substituted for world projection"
        assert row["rng_before"] == row["rng_after"] == rng  # MaxParallel, no random effects
        if before_full is not None:
            assert row["full_hash_before"] == before_full and row["world_hash_before"] == before_world
        nodes, edges = replay_step(row, table, nodes, edges)
        assert {k: n for k, n in nodes.items() if n["label"] >= BASE} == metadata_nodes
        assert {k: e for k, e in edges.items() if e["label"] >= BASE} == metadata_edges
        ns, es = projected(nodes, edges)
        assert (row["world_nodes"], row["world_edges"]) == (len(ns), len(es))
        assert row["graph_cap"] == (len(nodes) > 2000 or len(edges) > 4000)
        work += row["usage"]["work"]
        results += row["usage"]["results"]
        peak_work = max(peak_work, row["usage"]["work"])
        peak_results = max(peak_results, row["usage"]["results"])
        if row["ceiling"]:
            assert row["store_step"] == advancing and row["native"]["stats"]["step"] == 0
            assert not row["native"]["events"] and row["full_hash"] == row["full_hash_before"] and row["world_hash"] == row["world_hash_before"]
            cap = "matching_cap"
        else:
            advancing += 1
            assert row["store_step"] == row["native"]["stats"]["step"] == advancing
            if row["graph_cap"]:
                cap = "store_cap"
        if cap is None:
            frames.append((row["world_hash"], row["native"]["stats"]["accepted_rewrites"]))
            rngs.append(row["rng_after"])
            fingerprints.append(world_fingerprint(nodes, edges))
            exploding |= len(ns) > 4 * initial_counts[0] or len(es) > 4 * initial_counts[1]
        rng, before_full, before_world = row["rng_after"], row["full_hash"], row["world_hash"]
    assert row["terminal"] == inventory(nodes, edges, advancing)
    assert row["reason"] == cap
    complete = len(frames) == horizon
    assert row["complete"] == complete and row["measured_steps"] == len(frames)
    frozen = any(all(n == 0 for _, n in frames[i:i+50]) for i in range(max(0, len(frames)-49)))
    assert row["frozen"] == frozen and row["exploding"] == exploding
    return {"frames": frames, "rng": rngs, "fingerprints": fingerprints,
            "complete": complete, "frozen": frozen, "exploding": exploding,
            "_end": row, "attempted": attempted, "advancing": advancing,
            "work": work, "results": results, "peak_work": peak_work, "peak_results": peak_results}


def classification(valid, neutral, parent, child):
    if not valid:
        return "quarantined"
    if not parent["complete"] or child is not None and not child["complete"]:
        return "unmeasured"
    if child is not None and child["frames"] == parent["frames"]:
        assert child["fingerprints"] == parent["fingerprints"], "paired hash equality lacks typed world equality"
    if neutral:
        return "neutral"
    assert child is not None
    if child["frames"] == parent["frames"]:
        return "near_neutral"
    if child["frozen"] or child["exploding"]:
        return "catastrophic"
    return "divergent"


def cap_parent(p):
    return any(len(e) == 8 for c in [p["core"], p["proposal"]] for e in c["lhs"] + c["rhs"])


def check_classification(parent, d, row, baseline, child, checked, i):
    assert d["index"] == row["index"] == i and row["parent"] == d["parent"] == parent["index"]
    assert row["manifest"] == d
    after = raw_edit(parent, d)
    result = decode(after, parent["root"])
    assert row["decoded"] == result
    valid = result["program"] is not None
    neutral = valid and result["program"]["rcf"] == parent["program"]["rcf"]
    soundness = valid and neutral and baseline["complete"] and checked < 50
    traced = valid and (not neutral or soundness)
    assert row["trace_id"] == (f"child_{i}" if traced else None)
    assert (child is not None) == traced
    failure = soundness and (not child["complete"] or child["frames"] != baseline["frames"] or child["rng"] != baseline["rng"])
    outcome = classification(valid, neutral, baseline, child)
    changed = child is not None and child["complete"] and baseline["complete"] and child["frames"] != baseline["frames"]
    check_report(row["compiled"], result["program"], result["validator"])
    expected = {"operator": d["operator"], "operator_index": d["operator_index"], "role": d["role"],
                "valid": valid, "neutral_identity": neutral, "outcome": outcome,
                "soundness_checked": soundness, "soundness_failure": failure,
                "trace_changed": changed, "parent_active": parent["program"]["active"], "at_cap": cap_parent(parent["program"]),
                "shape": parent["shape"], "parent_rcf": parent["program"]["rcf"], "validator": result["validator"],
                "mode_changed": valid and result["program"]["active"] != parent["program"]["active"],
                "executable_changed": valid and not neutral}
    expected.update({k: d["edit"][k] for k in ["raw_changed", "logical_topology_changed", "metadata_changed", "applicable"]})
    for k, value in expected.items():
        assert row[k] == value, f"classification field {k} mismatch"
    return checked + int(soundness)


def tally(rows):
    strata = {}
    modes = changed = checks = failures = samples = 0
    for r in rows:
        samples += 1
        def literal(x):
            return json.dumps(x, separators=(",", ":"))
        keys = ["overall", f'operator:{r["operator"]}', f'shape:{r["shape"]}',
                f'active:{literal(r["parent_active"])}', f'at_cap:{literal(r["at_cap"])}',
                f'logical_change:{literal(r["logical_topology_changed"])}', f'raw_change:{literal(r["raw_changed"])}',
                f'metadata_change:{literal(r["metadata_changed"])}', f'applicable:{literal(r["applicable"])}',
                f'parent_identity:{r["parent_rcf"]}', f'genotype_parent:{r["parent"]}', f'validator:{literal(r["validator"])}']
        if r["operator_index"] < 11:
            keys.extend(["structural", f'role:{r["role"]}'])
            if r["parent_active"]:
                keys.append("active_parent_structural")
        else:
            keys.append("activation")
            modes += r["mode_changed"]
            changed += r["mode_changed"] and r["trace_changed"]
        for key in keys:
            c = strata.setdefault(key, Counter())
            c["samples"] += 1
            c[r["outcome"]] += 1
            if r["outcome"] != "quarantined":
                c["valid"] += 1
        checks += r["soundness_checked"]
        failures += r["soundness_failure"]
    for c in strata.values():
        assert c["quarantined"] + c["valid"] == c["samples"]
        assert sum(c[k] for k in ["neutral", "near_neutral", "divergent", "catastrophic", "unmeasured"]) == c["valid"]
    def thresholds(key, near, minimum):
        c = strata.get(key, Counter())
        n = c["samples"]
        return n >= minimum and n > 0 and 5*c["valid"] >= 2*n and (not near or 10*c["near_neutral"] >= n) and 4*c["catastrophic"] <= n and c["unmeasured"] == 0
    numerical = (samples == 10000 and thresholds("overall", True, 10000) and thresholds("structural", True, 1)
                 and thresholds("active_parent_structural", True, 2000) and thresholds("activation", False, 1000)
                 and modes >= 100 and changed >= 20 and checks >= 50 and failures == 0
                 and all(f"operator:{op}" in strata for op in OPERATORS)
                 and all(k in strata for k in ["role:Core", "role:Proposal", "at_cap:true", "at_cap:false", "active:true", "active:false"]))
    return {"strata": {k: dict(c) for k, c in strata.items()}, "mode_changes": modes,
            "trajectory_changing_mode_edits": changed, "neutral_soundness_checked": checks,
            "neutral_soundness_failures": failures, "numerical_pass": bool(numerical)}


def measurement_complete(summary):
    return summary["parent_baselines_complete"] and summary["strata"]["overall"].get("unmeasured", 0) == 0


def reject(name, run):
    try:
        run()
    except (AssertionError, KeyError, IndexError, StopIteration, ValueError, Invalid):
        return name
    raise AssertionError(f"tamper accepted: {name}")


def tamper_controls(parent, d, row, baseline, child, checked, control, control_manifest, trace_rows, m, design_value, control_parent, trace_parent):
    names = []
    check_classification(parent, d, row, baseline, child, checked, row["index"])
    check_control(control_parent, control_manifest, control)
    consume_trace(iter(trace_rows), f'parent_{trace_parent["index"]}', trace_parent["inventory"], trace_parent, m["horizon"])
    def classifier(name, mutate):
        r = copy.deepcopy(row)
        mutate(r)
        names.append(reject(name, lambda: check_classification(parent, d, r, baseline, child, checked, row["index"])))
    classifier("omitted_draw", lambda r: r.__setitem__("index", r["index"] + 1))
    classifier("duplicate_draw", lambda r: r.__setitem__("index", -1))
    classifier("wrong_parent_operator_role_selector", lambda r: r.__setitem__("role", "Forged"))
    for name, mutate in [
        ("hidden_graph_edit", lambda x: x["edit"]["delta"]["edges"]["deleted"].append("EdgeId(999999v1)")),
        ("automatic_metadata_repair", lambda x: x["edit"]["actions"].append({"op": "set_property", "id": parent["root"], "key": 0x200, "value": {"Int": 0}})),
        ("false_no_op", lambda x: x["edit"].__setitem__("applicable", not x["edit"]["applicable"]))]:
        x = copy.deepcopy(d)
        mutate(x)
        names.append(reject(name, lambda x=x: raw_edit(parent, x)))
    classifier("dropped_invalid_dormant_body", lambda r: r["decoded"].__setitem__("program", {"active": False}))
    c = copy.deepcopy(control)
    c["compiled"]["table"] = copy.deepcopy(c["clean"]["table"])
    names.append(reject("cached_missing_root", lambda: check_control(control_parent, control_manifest, c)))
    classifier("forged_validator_class_table", lambda r: r["compiled"]["report"].__setitem__("classes", [12345]))
    classifier("wrong_RCF", lambda r: r["decoded"].__setitem__("program", {"rcf": "00"}))
    for name, mutate in [
        ("swapped_payload_endpoint", lambda rs: rs[1]["native"]["events"][0]["bound_nodes"].reverse()),
        ("full_meta_hash_for_world", lambda rs: rs[1].__setitem__("world_hash", rs[1]["full_hash"])),
        ("altered_world_delta_hash_sequence", lambda rs: rs[1]["delta"]["edges"]["deleted"].append("EdgeId(999999v1)")),
        ("hidden_cap", lambda rs: rs[1].__setitem__("ceiling", not rs[1]["ceiling"])),
        ("premature_completion", lambda rs: rs[-1].__setitem__("measured_steps", rs[-1]["measured_steps"] + 1)),
        ("changed_catastrophe_boundary", lambda rs: rs[-1].__setitem__("frozen", not rs[-1]["frozen"]))]:
        rs = copy.deepcopy(trace_rows)
        mutate(rs)
        names.append(reject(name, lambda rs=rs: consume_trace(iter(rs), f'parent_{trace_parent["index"]}', trace_parent["inventory"], trace_parent, m["horizon"])))
    summary = tally([row])
    forged = copy.deepcopy(summary)
    forged["strata"]["overall"]["samples"] += 1
    names.append(reject("falsified_stratum_denominator", lambda: assert_equal(forged, tally([row]))))
    manifest = copy.deepcopy(m)
    manifest["training_admitted"] = True
    names.append(reject("forged_pass_training_flag", lambda: check_manifest(manifest, design_value, m["development_only"])))
    assert len(names) >= 18 and len(names) == len(set(names))
    return names


def assert_equal(a, b):
    assert a == b


def audit_controls(out, parents, d):
    rs, ms = stream(out / "controls.jsonl.gz"), stream(out / "control_manifest.jsonl.gz")
    roles, keys, zero = set(), set(), False
    saved = None
    count = 0
    for p in parents:
        for name in d["completeness_controls"]["attacks"]:
            r, a = next(rs), next(ms)
            assert a["parent"] == p["index"] and a["attack"] == name
            check_control(p, a, r)
            count += 1
            roles.add(a["relation_role_index"])
            keys.add(a["obligation_key"])
            zero |= a["obligation_zero"]
            if p["index"] == 0 and name == "root_delete":
                saved = r, a
    assert next(rs, None) is next(ms, None) is None
    assert count == 5376 and roles == set(range(13)) and keys == set(range(0x220, 0x23A)) and zero
    return {"controls": count, "roles": sorted(roles), "obligation_keys": sorted(keys), "explicit_zero_obligation": zero}, saved


def verify_parent_manifest(out):
    ps = list(stream(out / "parents.jsonl.gz"))
    assert len(ps) == 168
    for i, p in enumerate(ps):
        check_parent(p, i)
    return ps


def audit(out, stage, development):
    m, d = integrity(out, development)
    ps = verify_parent_manifest(out)
    controls, saved_control = audit_controls(out, ps, d)
    if stage == "controls":
        assert not (out / "parent_traces.jsonl.gz").exists(), "stage barrier too late"
        return {"audit_pass": True, "scope": "controls", "development_only": development, "gate_pass": False, "training_admitted": False, **controls}
    barrier = load(out / "stage_barrier.json")
    assert barrier == {"controls_native_and_frozen_audit_pass": True, "primary_mutations": "not_started", "advancing_steps": 0, "development_only": development, "training_admitted": False}
    stage_receipt = load(out / "audit_controls.json")
    assert stage_receipt["audit_pass"] is True and stage_receipt["scope"] == "controls"
    parent_rows = stream(out / "parent_traces.jsonl.gz")
    cache = []
    totals = Counter()
    for p in ps:
        t = consume_trace(parent_rows, f'parent_{p["index"]}', p["inventory"], p, m["horizon"])
        cache.append(t)
        for k in ["attempted", "advancing", "work", "results"]:
            totals[k] += t[k]
        for k in ["peak_work", "peak_results"]:
            totals[k] = max(totals[k], t[k])
    assert next(parent_rows, None) is None
    ms, rs, ts = stream(out / "draw_manifest.jsonl.gz"), stream(out / "classifications.jsonl.gz"), stream(out / "child_traces.jsonl.gz")
    checked, word_pos, context = 0, 0, None
    def classified():
        nonlocal checked, word_pos, context
        for i in range(m["draws"]):
            draw, row = next(ms), next(rs)
            assert draw["index"] == i and 0 <= draw["parent"] < 168
            assert int(draw["rng_word_pos_after"]) > word_pos
            word_pos = int(draw["rng_word_pos_after"])
            p = ps[draw["parent"]]
            after = raw_edit(p, draw)
            result = decode(after, p["root"])
            valid = result["program"] is not None
            neutral = valid and result["program"]["rcf"] == p["program"]["rcf"]
            baseline = cache[draw["parent"]]
            soundness = valid and neutral and baseline["complete"] and checked < 50
            child = consume_trace(ts, f"child_{i}", after, p, m["horizon"]) if valid and (not neutral or soundness) else None
            before_checked = checked
            checked = check_classification(p, draw, row, baseline, child, checked, i)
            if child is not None:
                for k in ["attempted", "advancing", "work", "results"]:
                    totals[k] += child[k]
                for k in ["peak_work", "peak_results"]:
                    totals[k] = max(totals[k], child[k])
            if p["index"] == 0 and context is None:
                context = p, draw, row, baseline, child, before_checked
            yield row
        assert next(ms, None) is next(rs, None) is next(ts, None) is None
    summary = tally(classified())
    summary["parent_baselines_complete"] = all(t["complete"] for t in cache)
    assert summary == load(out / "classification_summary.json")
    # An index-0 draw is not guaranteed in a short smoke. Use any saved draw
    # for classifier tests, and index-0 control/trace for independent controls.
    if context is None:
        draw = next(stream(out / "draw_manifest.jsonl.gz"))
        row = next(stream(out / "classifications.jsonl.gz"))
        p = ps[draw["parent"]]
        baseline = cache[draw["parent"]]
        child = None
        if row["trace_id"] is not None:
            child = consume_trace(stream(out / "child_traces.jsonl.gz"), row["trace_id"], raw_edit(p, draw), p, m["horizon"])
        context = p, draw, row, baseline, child, 0
    trace_sample = []
    for r in stream(out / "parent_traces.jsonl.gz"):
        trace_sample.append(r)
        if r["kind"] == "end":
            break
    # Context-specific classifier controls; root-loss and trace controls use
    # their own parent rather than borrowing the primary draw's parent.
    names = tamper_controls(*context, *saved_control, trace_sample, m, d, ps[0], ps[0])
    assert totals["attempted"] <= (168 + m["draws"]) * m["horizon"] and totals["advancing"] <= totals["attempted"]
    return {"audit_pass": True, "scope": "full", "development_only": development,
            "numerical_pass": summary["numerical_pass"], "gate_pass": not development and summary["numerical_pass"] and measurement_complete(summary),
            "training_admitted": False, "measurement_complete": measurement_complete(summary),
            **controls, "resources": dict(totals), "tamper_rejections": len(names), "tamper_categories": names,
            "unique_genotype_parents": len({json.dumps({"core":bare(p["program"]["core"]),"proposal":bare(p["program"]["proposal"]),"active":p["program"]["active"]},sort_keys=True) for p in ps}),
            "unique_executable_rcf_parents": len({p["program"]["rcf"] for p in ps}),
            "unique_executable_presentation_parents": len({p["program"]["execution"] for p in ps})}


def main():
    global DEADLINE, ARCHIVE_ROOT, OUTPUT_CAP
    p = argparse.ArgumentParser()
    p.add_argument("archive", type=Path)
    p.add_argument("--stage", choices=["freeze", "controls", "full", "seal"], default="full")
    p.add_argument("--development", action="store_true")
    p.add_argument("--seconds", type=float, default=900)
    args = p.parse_args()
    assert 0 < args.seconds <= 900
    DEADLINE = time.monotonic() + args.seconds
    # macOS RLIMIT_AS is not a resident limit; enforce measured RSS on every
    # stream record and before/after stages rather than claiming a virtual cap.
    out = args.archive.resolve()
    ARCHIVE_ROOT, OUTPUT_CAP = out, 536870912 if args.development else 4294967296
    if args.stage == "freeze":
        integrity(out, args.development, freezing=True)
        assert not (out / "controls.jsonl.gz").exists() and not (out / "parent_traces.jsonl.gz").exists()
        save(out / "frozen_inputs_sha256.json", {str(p.relative_to(out)): digest(p) for p in files(out)})
    elif args.stage == "seal":
        integrity(out, args.development)
        a, s = load(out / "audit_full.json"), load(out / "run_status.json")
        assert a["audit_pass"] is True and s["status"] == "complete"
        assert s["gate_pass"] == a["gate_pass"] and s["numerical_pass"] == a["numerical_pass"]
        assert s["training_admitted"] is False and s["development_only"] == args.development
        assert s["manifest_regenerated"] is True and s["measurement_complete"] == a["measurement_complete"]
        assert s["advancing_engine_steps"] == a["resources"]["advancing"]
        assert s["attempted_engine_steps"] == a["resources"]["attempted"]
        assert s["elapsed_seconds"] < (300 if args.development else 3600)
        assert s["auditor_seconds"] < (120 if args.development else 900)
        save(out / "output_sha256.json", {str(p.relative_to(out)): digest(p) for p in files(out)})
    else:
        result = audit(out, args.stage, args.development)
        guard()
        result["peak_rss_bytes"] = resource.getrusage(resource.RUSAGE_SELF).ru_maxrss * (1 if sys.platform == "darwin" else 1024)
        save(out / f"audit_{args.stage}.json", result)
        print(json.dumps({k: result[k] for k in ["scope", "audit_pass", "gate_pass", "training_admitted"]}))
    guard()


if __name__ == "__main__":
    main()
