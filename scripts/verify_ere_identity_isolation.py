#!/usr/bin/env python3
"""Read-only strict receipt auditor for the bounded allocator diagnostic.

This compares native Engine evidence; it is not an independent Engine interpreter
or a G-R1 auditor. It never renames handles or sorts endpoints. --seal is only
for a fresh producer receipt; default invocation is entirely read-only.
"""
import argparse
import gzip
import hashlib
import json
from pathlib import Path
import re
import time

PROFILE = "E2-allocator-identity-diagnostic-v1"
CAP = 64 * 1024 * 1024


def require(condition, message):
    if not condition:
        raise ValueError(message)


def handle(value, kind):
    return f"{kind}({value['idx']}v{value['version']})"


def check_world(world, step, isolated):
    require(world["step"] == step, "world clock")
    ns, es = dict(world["nodes"]), dict(world["edges"])
    require(len(ns) == len(world["nodes"]) and len(es) == len(world["edges"]), "duplicate handles")
    require(world["nodes"] == sorted(world["nodes"]) and world["edges"] == sorted(world["edges"]), "inventory order")
    incidence = {n: [] for n in ns}
    labels = {}
    for n, row in ns.items():
        require(row["data"]["label"] < 0xFFF00000, "metadata leaked into world")
        require(re.fullmatch(r"NodeId\([1-9][0-9]*v[1-9][0-9]*\)", n), "node handle")
        if isolated:
            require(int(n.split("(")[1].split("v")[0]) % 2 == 1, "world node lane")
    for eid, data in es.items():
        require(data["label"] < 0xFFF00000, "metadata edge leaked into world")
        if isolated:
            require(int(eid.split("(")[1].split("v")[0]) % 2 == 1, "world edge lane")
        labels.setdefault(data["label"], []).append(eid)
        for endpoint in data["nodes"]:  # preserve endpoint sequence in the receipt
            n = handle(endpoint, "NodeId")
            require(n in ns, "dangling world endpoint")
            incidence[n].append(eid)
    require(world["label_indexes"] == [[k, sorted(v)] for k, v in sorted(labels.items())], "label index")
    for n, row in ns.items():
        require(row["incident"] == sorted(incidence[n]), "incidence index")


def check_trace(frames, horizon, isolated):
    require(len(frames) == horizon + 1, "incomplete horizon")
    births = 0
    for step, frame in enumerate(frames):
        require(frame["step"] == step, "frame clock")
        check_world(frame["world"], step, isolated)
        require(re.fullmatch(r"[0-9a-f]{16}", frame["rng"]), "RNG probe")
        if step:
            require(frame["usage"]["work"] <= 400000 and frame["usage"]["results"] <= 50000, "pooled budget")
            require(len(frame["events"]) == frame["accepted"], "accepted transition count")
            old = set(dict(frames[step - 1]["world"]["nodes"]))
            new = set(dict(frame["world"]["nodes"]))
            made = [handle(n, "NodeId") for e in frame["events"] for n in e["nodes_created"]]
            removed = [handle(n, "NodeId") for e in frame["events"] for n in e["nodes_deleted"]]
            require(set(made) == new - old and set(removed) == old - new, "native node birth/death evidence")
            olde = set(dict(frames[step - 1]["world"]["edges"]))
            newe = set(dict(frame["world"]["edges"]))
            madee = [handle(n, "EdgeId") for e in frame["events"] for n in e["edges_created"]]
            removede = [handle(n, "EdgeId") for e in frame["events"] for n in e["edges_deleted"]]
            require(set(madee) == newe - olde and set(removede) == olde - newe, "native edge birth/death evidence")
            births += len(made)
    require(births > 0, "vacuous no-birth comparison")
    return births


def audit(receipt):
    require(receipt["profile"] == PROFILE, "profile")
    require(receipt["gate_pass"] is False and receipt["training_admitted"] is False, "admission")
    horizon = receipt["horizon"]
    require(horizon == 100, "persisted diagnostic horizon")
    require(0 <= receipt["elapsed_seconds"] < 300, "producer wall cap")
    cases = receipt["cases"]
    expected = [(p, isolated, checkpoint, recycled) for p in [138, 30]
                for isolated, checkpoint, recycled in [(False, False, False), (True, False, False), (True, True, False), (True, False, True)]]
    require([(c["parent_index"], c["isolated"], c["checkpoint"], c["recycled"]) for c in cases] == expected, "case coverage/order")
    require(receipt["engine_steps"] == 2 * horizon * len(cases) <= 2000, "step accounting")
    summary = []
    table = receipt["world_table"]
    require(isinstance(table, list) and table, "world table")
    used = set()
    def expand(frames):
        result = []
        for frame in frames:
            index = frame["world_id"]
            require(type(index) is int and 0 <= index < len(table) and "world" not in frame, "world table reference")
            used.add(index)
            result.append({**{k: v for k, v in frame.items() if k != "world_id"}, "world": table[index]})
        return result
    for i, case in enumerate(cases):
        p, c = expand(case["parent_frames"]), expand(case["child_frames"])
        require((case["original_draw"], case["pattern_index"]) == ((19, 1) if case["parent_index"] == 138 else (27, 0)), "original draw mapping")
        births = check_trace(p, horizon, case["isolated"])
        check_trace(c, horizon, case["isolated"])
        # Metadata scan cost is retained and bounded separately; unlike world
        # handles/endpoints/events/RNG it is not part of typed-world identity.
        identity = lambda frames: [{k: v for k, v in f.items() if k != "usage"} for f in frames]
        equal = identity(p) == identity(c)
        require(case["identical"] == equal == case["isolated"], "strict identity conclusion")
        require(case["world_births"] == births, "birth accounting")
        first = next((j for j, (a, b) in enumerate(zip(identity(p), identity(c))) if a != b), None)
        if not case["isolated"]:
            require(first == 1 and p[1]["world_hash"] == c[1]["world_hash"], "legacy failure not reproduced")
            left = p[1]["events"][0]["nodes_created"][0]
            right = c[1]["events"][0]["nodes_created"][0]
            idx = 96 if case["parent_index"] == 138 else 89
            require(left == {"idx": idx, "version": 1} and right == {"idx": idx + 1, "version": 1}, "known legacy birth handles")
        if case["checkpoint"]:
            require(p == expand(cases[i - 1]["parent_frames"]) and c == expand(cases[i - 1]["child_frames"]), "checkpoint continuation")
        summary.append({"parent_index": case["parent_index"], "isolated": case["isolated"],
                        "checkpoint": case["checkpoint"], "strict_identity": equal,
                        "recycled": case["recycled"],
                        "first_difference": first, "world_births": births,
                        "matching_work_differs": any(a.get("usage") != b.get("usage") for a,b in zip(p,c))})
    require(used == set(range(len(table))), "unused world table rows")
    negative = receipt["negative_control"]
    a, b = negative["parent_world"], negative["child_world"]
    check_world(a, 0, True)
    check_world(b, 0, True)
    require(negative["equal_hash"] is True and a != b and a["nodes"] == b["nodes"], "negative control")
    require(a["edges"][0][1]["nodes"] == b["edges"][0][1]["nodes"][::-1], "endpoint reversal negative control")
    return {"profile": PROFILE, "audit_pass": True, "engine_steps": receipt["engine_steps"],
            "cases": summary, "endpoint_negative_control_rejected": True,
            "gate_pass": False, "training_admitted": False}


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def inventory(root):
    return {p.relative_to(root).as_posix(): digest(p) for p in sorted(root.rglob("*"))
            if p.is_file() and p != root / "output_sha256.json"}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    parser.add_argument("--seal", action="store_true")
    args = parser.parse_args()
    start = time.monotonic()
    root = args.directory.resolve()
    require(sum(p.stat().st_size for p in root.rglob("*") if p.is_file()) <= CAP, "output cap")
    if args.seal:
        require(not (root / "output_sha256.json").exists() and not (root / "audit.json").exists(), "refuse reseal")
    else:
        require(json.loads((root / "output_sha256.json").read_text()) == inventory(root), "seal membership/hash mismatch")
    # Limit decompressed evidence too; a malicious gzip must not exhaust memory.
    with gzip.open(root / "receipt.json.gz", "rb") as stream:
        payload = stream.read(CAP + 1)
    require(len(payload) <= CAP, "decompressed receipt cap")
    result = audit(json.loads(payload))
    sources = json.loads((root / "sources.json").read_text())
    require(len(sources) == len(set(sources)), "duplicate source pins")
    require(set(sources) == {p.relative_to(root / "source_snapshots").as_posix()
                            for p in (root / "source_snapshots").rglob("*") if p.is_file()}, "source inventory")
    require(time.monotonic() - start < 300, "auditor wall cap")
    result["frozen_source_count"] = len(sources)
    if args.seal:
        with (root / "audit.json").open("x") as out:
            json.dump(result, out, indent=2, sort_keys=True)
        with (root / "manifest.json").open("w") as out:
            json.dump({"profile": PROFILE, "run_status": "verified_complete_diagnostic",
                       "gate_pass": False, "training_admitted": False}, out, indent=2)
        with (root / "output_sha256.json").open("x") as out:
            json.dump(inventory(root), out, indent=2, sort_keys=True)
        require(sum(p.stat().st_size for p in root.rglob("*") if p.is_file()) <= CAP, "sealed output cap")
    else:
        require(json.loads((root / "audit.json").read_text()) == result, "stored audit mismatch")
        require(json.loads((root / "output_sha256.json").read_text()) == inventory(root), "audit modified archive")
    print(json.dumps(result, sort_keys=True))


if __name__ == "__main__":
    main()
