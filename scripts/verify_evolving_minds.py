#!/usr/bin/env python3
"""Independent bounded-physics replay. Reads artifacts; never launches Engine."""
import argparse
import copy
import csv
import hashlib
import io
import json
from pathlib import Path
import tomllib

PROFILE = "evolving-minds-engineering-v1"
RULE_SHA = "75d44aaeadb4977858fd5ed49cb4f3c222863f5b37218a521c28f1f4be5f607a"
ROOT, SITE, SENSOR, GENE, CONNECTION, ACTUATOR, COUNTER, DEAD = range(0xE1000000, 0xE1000008)
OCC, NEIGHBOR, OWN_S, OWN_G, OWN_C, OWN_A, SOURCE, TARGET = range(0xE1000064, 0xE100006C)
ENERGY, OWNER, OP, DEV, AGE, SIGNAL, OCCUPIED, BID, PARENT, NEXT, MAINT = (1, *range(240001, 240011))
PHASES = "Environment Sense Signal Development Action Maintenance Birth Death Cleanup".split()


def require(condition, reason):
    if not condition:
        raise ValueError(reason)


def handle(h):
    require(set(h) == {"idx", "version"} and h["idx"] > 0 and h["version"] % 2 == 1, "invalid handle")
    return h["idx"], h["version"]


def rank(h):
    return (h[1] << 32) | h[0]


def graphs(frame):
    nodes = {handle(k): copy.deepcopy(v) for k, v in frame["nodes"]}
    edges = {handle(k): copy.deepcopy(v) for k, v in frame["edges"]}
    require(len(nodes) == len(frame["nodes"]) and len(edges) == len(frame["edges"]), "duplicate handles")
    return nodes, edges


def property_value(n, key):
    vals = [v for k, v in n["properties"] if k == key]
    require(len(vals) == 1 and len(vals[0]) == 1, f"missing/duplicate property {key}")
    return vals[0]


def value(n, key, tag):
    v = property_value(n, key)
    require(set(v) == {tag}, f"wrong property variant {key}")
    return v[tag]


def typed(v):
    return {"Bool" if isinstance(v, bool) else "Float" if isinstance(v, float) else "Int": v}


def set_property(n, key, v):
    for entry in n["properties"]:
        if entry[0] == key:
            entry[1] = copy.deepcopy(v)
            return
    n["properties"].append([key, copy.deepcopy(v)])


def edge_key(e):
    return e["label"], tuple(sorted(handle(h) for h in e["nodes"]))


def total(nodes):
    return sum(value(n, ENERGY, "Float") for n in nodes.values() if n["label"] in [ROOT, SITE, DEAD])


def checkpoint_envelope(path):
    data = path.read_bytes()
    require(24 <= len(data) <= 64 * 1024 * 1024 + 24 and data[:8] == b"HGEMCP01", "checkpoint envelope")
    require(int.from_bytes(data[8:16], "little") == len(data) - 24, "checkpoint length")
    digest = 0xcbf29ce484222325
    for b in data[24:]:
        digest = ((digest ^ b) * 0x100000001b3) & ((1 << 64) - 1)
    require(int.from_bytes(data[16:24], "little") == digest, "checkpoint checksum")


def material(nodes, edges, lineage, sites, scaffold):
    roots = {value(n, BID, "Int"): h for h, n in nodes.items() if n["label"] == ROOT}
    require(len(roots) == sum(n["label"] == ROOT for n in nodes.values()), "duplicate live birth ID")
    require({h for h, n in nodes.items() if n["label"] == SITE} == sites, "site inventory changed")
    owned = {bid: {} for bid in roots}
    for h, n in nodes.items():
        require(n["energy"] == 0, "energy-field side channel")
        keys = [k for k, _ in n["properties"]]
        require(len(keys) == len(set(keys)), "duplicate property")
        require(n["label"] in range(ROOT, DEAD + 1), "unknown label")
        if n["label"] in [ROOT, SITE, DEAD]:
            require(value(n, ENERGY, "Float") >= 0, "negative reserve")
        else:
            require(ENERGY not in keys, "unaccounted internal reserve")
        if n["label"] in [SENSOR, GENE, CONNECTION, ACTUATOR]:
            bid = value(n, OWNER, "Int")
            require(bid in roots and n["label"] not in owned[bid], "foreign/orphan/duplicate owned node")
            owned[bid][n["label"]] = h
    expected = set(scaffold)
    occupancy = {}
    counters = [n for n in nodes.values() if n["label"] == COUNTER]
    require(len(counters) == 1 and value(counters[0], NEXT, "Int") == len(lineage) + 1, "counter mismatch")
    for bid, root in roots.items():
        require(bid in lineage and lineage[bid]["death_tick"] is None, "unknown/retired identity")
        r, parts = nodes[root], owned[bid]
        require(value(r, PARENT, "Int") == (lineage[bid]["parent_id"] or 0), "parent mismatch")
        require(SENSOR in parts and GENE in parts, "missing common seed")
        require(value(nodes[parts[GENE]], OP, "Int") == lineage[bid]["opcode"], "genotype inheritance")
        built = value(r, DEV, "Bool") and lineage[bid]["opcode"] == 1
        require(set(parts) == ({SENSOR, GENE, CONNECTION, ACTUATOR} if built else {SENSOR, GENE}), "phenotype shape")
        require(value(r, MAINT, "Float") == (5.0 if built else 3.0), "physical maintenance price")
        expected.update([(OWN_S, tuple(sorted([root, parts[SENSOR]]))), (OWN_G, tuple(sorted([root, parts[GENE]])))])
        if built:
            expected.update((label, tuple(sorted(pair))) for label, pair in [
                (OWN_C, [root, parts[CONNECTION]]), (OWN_A, [root, parts[ACTUATOR]]),
                (SOURCE, [parts[SENSOR], parts[CONNECTION]]), (TARGET, [parts[CONNECTION], parts[ACTUATOR]])])
    actual = set()
    for e in edges.values():
        hs = [handle(h) for h in e["nodes"]]
        require(len(hs) == 2 and len(set(hs)) == 2 and all(h in nodes for h in hs), "dangling/self/arity edge")
        key = edge_key(e)
        require(key not in actual, "duplicate relation")
        actual.add(key)
        if e["label"] == OCC:
            rs = [h for h in hs if nodes[h]["label"] == ROOT]
            ss = [h for h in hs if h in sites]
            require(len(rs) == len(ss) == 1 and ss[0] not in occupancy, "invalid occupancy")
            occupancy[ss[0]] = rs[0]
            expected.add(key)
    require(actual == expected and len(occupancy) == len(roots) and len(set(occupancy.values())) == len(roots), "ownership/topology mismatch")
    require(all(value(nodes[s], OCCUPIED, "Bool") == (s in occupancy) for s in sites), "occupancy flag mismatch")
    return roots, owned, occupancy


def comparison(a, op, b):
    require(set(a) == set(b), "guard type mismatch")
    x, y = next(iter(a.values())), next(iter(b.values()))
    return {"eq": x == y, "lt": x < y, "lte": x <= y, "gt": x > y, "gte": x >= y, "ne": x != y}[op]


def apply(nodes, edges, event, rule, step):
    lhs, rhs = rule["lhs"], rule["rhs"]
    bound = list(map(handle, event["bound"]))
    require(len(bound) == len(lhs["nodes"]) and len(set(bound)) == len(bound), "binding cardinality/distinctness")
    binding = dict(zip(lhs["nodes"], bound))
    require(all(nodes[binding[v]]["label"] == label for v, label in lhs["node_labels"].items()), "binding label")
    for g in lhs.get("node_property_guards", []):
        require(comparison(property_value(nodes[binding[g["var"]]], g["property"]), g["op"], typed(g["value"])), "failed guard accepted")
    for g in lhs.get("node_property_compares", []):
        require(comparison(property_value(nodes[binding[g["var_a"]]], g["key_a"]), g["op"], property_value(nodes[binding[g["var_b"]]], g["key_b"])), "foreign/failed compare accepted")
    matched = list(map(handle, event["matched_edges"]))
    require(len(matched) == len(lhs["edges"]) and len(set(matched)) == len(matched), "matched-edge inventory")
    for index, edge in enumerate(lhs["edges"]):
        require(matched[index] in edges and edge_key(edges[matched[index]]) ==
                (lhs["edge_labels"][str(index)], tuple(sorted(binding[v] for v in edge))), "matched relation")
    removed = {binding[v] for v in lhs["nodes"] if v not in rhs["nodes"]}
    removed_edges = set(matched) | {h for h, e in edges.items() if any(handle(n) in removed for n in e["nodes"])}
    require(set(map(handle, event["deleted"])) == removed and set(map(handle, event["edges_deleted"])) == removed_edges, "deletion/cleanup inventory")
    require(event["vanished_energy"] == 0, "deleted unsettled energy")
    for h in removed: del nodes[h]
    for h in removed_edges: del edges[h]
    new_names = [v for v in rhs["nodes"] if v not in binding]
    created = list(map(handle, event["created"]))
    require(len(created) == len(new_names) and len(set(created)) == len(created), "creation inventory")
    for name, h in zip(new_names, created):
        require(h not in nodes, "created existing node")
        binding[name] = h
        nodes[h] = {"label": rhs.get("node_labels", {}).get(name, 0), "generation": step, "properties": [], "energy": 0.0}
    for name, label in rhs.get("node_labels", {}).items(): nodes[binding[name]]["label"] = label
    for effect in rhs.get("property_effects", []):
        target, key, op = nodes[binding[effect["var"]]], effect["key"], effect["op"]
        if op == "set": result = typed(effect["value"])
        elif op == "copy_from": result = property_value(nodes[binding[effect["source_var"]]], effect["source_key"])
        else:
            left = property_value(target, key)
            right = property_value(nodes[binding[effect["source_var"]]], effect["source_key"]) if op.endswith("_from_property") else typed(effect["value"])
            require(set(left) == set(right), "effect type mismatch")
            tag = next(iter(left))
            result = {tag: left[tag] + right[tag] if op.startswith("add") else left[tag] - right[tag]}
        set_property(target, key, result)
    added_edges = list(map(handle, event["edges_created"]))
    require(len(added_edges) == len(rhs["edges"]) and len(set(added_edges)) == len(added_edges), "RHS edge inventory")
    for index, (h, endpoints) in enumerate(zip(added_edges, rhs["edges"])):
        require(h not in edges, "created existing edge")
        edges[h] = {"label": rhs["edge_labels"][str(index)], "nodes": [{"idx": binding[v][0], "version": binding[v][1]} for v in endpoints]}
    return binding


def verify(directory):
    directory = Path(directory)
    require(directory.is_dir() and not directory.is_symlink(), "run directory")
    require(all(not p.is_symlink() for p in directory.rglob("*")), "symlinked evidence")
    cfg = json.loads((directory / "config.json").read_text())
    require(3 <= cfg["sites"] <= 64 and 1 <= cfg["ticks"] <= 1024, "config envelope")
    require(sum(p.stat().st_size for p in directory.rglob("*") if p.is_file()) <= cfg["output_bytes"], "output ceiling")
    raw = (directory / "rules.toml").read_bytes()
    require(hashlib.sha256(raw).hexdigest() == RULE_SHA, "physical law identity")
    rules = {r["name"]: r for r in tomllib.loads(raw.decode())["rules"]}
    status = json.loads((directory / "run_status.json").read_text())
    manifest = json.loads((directory / "manifest.json").read_text())
    require(status["profile"] == PROFILE and status["engineering_execution_complete"] is True and status["development_only"] is True, "incomplete engineering run")
    require(all(status[k] is False for k in ["scientific_run_launched", "gate_pass", "training_admitted", "evolution_enabled"]), "scientific boundary")
    require(manifest["state"] == "Completed" and manifest["final_step"] == cfg["ticks"] * 9 and manifest["seed"] == cfg["seed"], "lifecycle/seed")
    expected_checkpoints={"final.checkpoint"}
    if cfg["ticks"]>=2:
        expected_checkpoints.add("midpoint.checkpoint")
    require({p.name for p in directory.glob("*.checkpoint")} == expected_checkpoints, "checkpoint coverage")
    for path in directory.glob("*.checkpoint"):
        checkpoint_envelope(path)
    frames = [json.loads(line) for line in (directory / "trace.jsonl").read_text().splitlines()]
    require(len(frames) == 1 + cfg["ticks"] * 9, "phase coverage")
    nodes, edges = graphs(frames[0])
    require(frames[0]["step"] == frames[0]["tick"] == 0 and frames[0]["phase"] is None and not frames[0]["events"], "initial frame")
    sites = {h for h, n in nodes.items() if n["label"] == SITE}
    require(len(nodes) == cfg["sites"] + 4 and len(edges) == cfg["sites"] + 3, "initial scaffold size")
    ordered_sites = sorted(sites)
    scaffold = {(NEIGHBOR, tuple(sorted([h, ordered_sites[(i + 1) % len(sites)]]))) for i, h in enumerate(ordered_sites)}
    require(all(value(nodes[h], ENERGY, "Float") == 32.0 for h in sites), "initial site energy")
    op = 1 if status["instruction"] == "BuildRelay" else 0
    require(status["instruction"] in ["BuildRelay", "Idle"], "program arm")
    lineage = {1: {"birth_id": 1, "parent_id": None, "born_tick": 0, "death_tick": None, "opcode": op}}
    roots, owned, occupancy = material(nodes, edges, lineage, sites, scaffold)
    require(set(roots) == {1} and value(nodes[roots[1]], ENERGY, "Float") == 24.0
            and value(nodes[roots[1]], AGE, "Int") == 0 and not value(nodes[roots[1]], DEV, "Bool"), "initial founder")
    initial = total(nodes)
    influx = dissipated = harvested = births = deaths = developed = 0
    tick_work = tick_accepted = 0
    rows = list(csv.DictReader(io.StringIO("\n".join((directory / "metrics.csv").read_text().splitlines()[1:]))))
    require(len(rows) == cfg["ticks"], "CSV coverage")
    for index, frame in enumerate(frames[1:]):
        phase = PHASES[index % 9]
        tick = index // 9 + 1
        require(frame["step"] == index + 1 and frame["tick"] == tick and frame["phase"] == phase, "clock/phase order")
        before = copy.deepcopy(nodes)
        roots, owned, occupancy = material(nodes, edges, lineage, sites, scaffold)
        events = frame["events"]
        eligible = []
        if phase == "Environment": eligible = [h for h in sites if value(nodes[h], ENERGY, "Float") <= 56]
        elif phase == "Sense": eligible = list(roots.values())
        elif phase == "Signal": eligible = [h for bid, h in roots.items() if CONNECTION in owned[bid]]
        elif phase == "Development": eligible = [h for bid, h in roots.items() if not value(nodes[h], DEV, "Bool") and (op == 0 or value(nodes[h], ENERGY, "Float") >= 2)]
        elif phase == "Action":
            eligible = [r for s, r in occupancy.items() if ACTUATOR in owned[value(nodes[r], BID, "Int")]
                        and value(nodes[owned[value(nodes[r], BID, "Int")][ACTUATOR]], SIGNAL, "Bool")
                        and value(nodes[s], ENERGY, "Float") >= 8 and value(nodes[r], ENERGY, "Float") <= 40]
        elif phase == "Maintenance": eligible = list(roots.values())
        elif phase == "Death": eligible = [h for h in roots.values() if value(nodes[h], AGE, "Int") >= 16 or value(nodes[h], ENERGY, "Float") <= 0]
        elif phase == "Cleanup": eligible = [h for h, n in nodes.items() if n["label"] == DEAD]
        else:
            counter = next(h for h, n in nodes.items() if n["label"] == COUNTER)
            candidates = []
            for site, r in occupancy.items():
                bid = value(nodes[r], BID, "Int")
                if value(nodes[r], DEV, "Bool") and value(nodes[r], AGE, "Int") >= 2 and value(nodes[r], ENERGY, "Float") >= 32:
                    for label, pair in scaffold:
                        if site in pair:
                            target = next(h for h in pair if h != site)
                            if target not in occupancy:
                                candidates.append([r, owned[bid][GENE], site, target, counter])
            if candidates:
                winner = min(candidates, key=lambda hs: sorted(map(rank, hs)))
                require(len(events) == 1 and list(map(handle, events[0]["bound"])) == winner, "birth contention/canonical winner")
            else: require(not events, "unsupported birth")
        if phase != "Birth": require(len(events) == len(eligible), "mandatory/eligible mechanism coverage")
        engaged = set()
        for e in events:
            require(e["rule"] in rules and e["rule"].startswith(phase.lower() + "_"), "wrong phase rule")
            bound = set(map(handle, e["bound"]))
            require(not (bound & engaged), "conflicting bindings applied")
            engaged |= bound
            if phase == "Maintenance":
                r = next(h for h in bound if before[h]["label"] == ROOT)
                dissipated += min(value(before[r], ENERGY, "Float"), value(before[r], MAINT, "Float"))
            if e["rule"] == "environment_influx": influx += 8
            if e["rule"] == "development_relay": dissipated += 2; developed += 1
            if e["rule"] == "action_intake": harvested += 8
            b = apply(nodes, edges, e, rules[e["rule"]], index)
            if phase == "Birth":
                dissipated += 2; births += 1
                child = nodes[b["child"]]
                bid, parent = value(child, BID, "Int"), value(child, PARENT, "Int")
                require(bid not in lineage and parent in lineage and lineage[parent]["death_tick"] is None, "birth ancestry")
                lineage[bid] = {"birth_id": bid, "parent_id": parent, "born_tick": tick, "death_tick": None, "opcode": lineage[parent]["opcode"]}
            if phase == "Death":
                bid = value(before[b["r"]], BID, "Int")
                require(lineage[bid]["death_tick"] is None, "duplicate death")
                lineage[bid]["death_tick"] = tick; deaths += 1
        if phase != "Birth": require(set(eligible) <= engaged, "skipped eligible object")
        actual_nodes, actual_edges = graphs(frame)
        require(nodes == actual_nodes and edges == actual_edges, f"independent rewrite replay mismatch at {tick}/{phase}")
        require(len(nodes) <= cfg["max_nodes"] and len(edges) <= cfg["max_edges"], "material ceiling")
        require(total(nodes) == initial + influx - dissipated, "independent ledger")
        material(nodes, edges, lineage, sites, scaffold)
        stats = frame["stats"]
        require(stats["accepted_rewrites"] == len(events) and stats["property_effects_skipped"] == 0 and stats["step"] == frame["step"], "native stats/skip")
        require(stats["property_effects_applied"] == sum(len(rules[e["rule"]]["rhs"].get("property_effects", [])) for e in events), "native effect engagement")
        require(len(events) <= stats["attempted_matches"] <= cfg["match_results"], "candidate result ceiling")
        for k, field in [("nodes_created", "created"), ("nodes_deleted", "deleted"), ("edges_created", "edges_created"), ("edges_deleted", "edges_deleted")]:
            require(stats[k] == sum(len(e[field]) for e in events), "native material counter")
        require(0 <= frame["match_work"] <= cfg["match_work"], "work ceiling")
        tick_work += frame["match_work"]; tick_accepted += len(events)
        if phase == "Cleanup":
            expected = dict(tick=tick, engine_step=frame["step"], live_organisms=sum(n["label"] == ROOT for n in nodes.values()),
                            nodes=len(nodes), edges=len(edges), births=births, deaths=deaths, developed=developed,
                            harvested=harvested, total_energy=total(nodes), influx=influx, dissipated=dissipated,
                            residual=0, match_work=tick_work, accepted=tick_accepted)
            require(set(rows[tick - 1]) == set(expected) and all(float(rows[tick - 1][k]) == v for k, v in expected.items()), "CSV independent recomputation")
            tick_work = tick_accepted = 0
    recorded = json.loads((directory / "organisms.json").read_text())
    require(recorded == {str(k): v for k, v in lineage.items()}, "lineage artifact")
    require(status["completed_ticks"] == cfg["ticks"] and status["final_row"] == expected, "status final summary")
    return {"profile": PROFILE, "audit_pass": True, "development_only": True,
            "scientific_run_launched": False, "gate_pass": False, "training_admitted": False,
            "frames_replayed": len(frames) - 1, "births": births, "deaths": deaths,
            "harvested": harvested, "live_organisms": expected["live_organisms"], "energy_residual": 0,
            "files_sha256": {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(directory.iterdir()) if p.is_file()}}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    args = parser.parse_args()
    print(json.dumps(verify(args.directory), indent=2))
