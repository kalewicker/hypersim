#!/usr/bin/env python3
"""Emit a read-only matcher fixture to stdout; never replay a trajectory."""
import gzip
import json
from pathlib import Path
import re
import time

import verify_ere_complete_coherent_draw_assay as a

ROOT = Path(__file__).resolve().parents[1]
ARCHIVE = ROOT / "output/ere_complete_coherent_draw_full_20260929"


def key(s):
    match = re.fullmatch(r"(?:Node|Edge)Id\((\d+)v(\d+)\)", s)
    assert match
    idx, version = map(int, match.groups())
    return {"idx": idx, "version": version}


def slots(entries, data):
    parsed = [(key(k), v) for k, v in entries]
    out = [{"value": None, "version": 0} for _ in range(max(k["idx"] for k, _ in parsed) + 1)]
    out[0]["version"] = 1  # metadata-isolated store marker
    for k, v in parsed:
        assert out[k["idx"]]["value"] is None and k["version"] % 2 == 1
        out[k["idx"]] = {"version": k["version"], "value": data(v)}
    return out


def main():
    assert __debug__, "requires Python assertions enabled"
    a.DEADLINE = time.monotonic() + 120
    inv = a.load(ROOT / "docs/ere_complete_coherent_draw_full_20260929_preservation.json")
    source = "child_traces.jsonl.gz"
    assert a.digest(ARCHIVE / source) == inv["sha256"][source]
    start, binding, terminal = None, None, None
    for row in a.stream(ARCHIVE / source):
        if row["id"] != "child_9730":
            continue
        if row["kind"] == "start":
            start = row
        elif row["kind"] == "step" and row["step"] == 92:
            binding = row
        elif row["kind"] == "end":
            terminal = row
            break
    assert terminal["complete"] is False and terminal["reason"] == "matching_cap"
    assert terminal["measured_steps"] == 91 and binding["store_step"] == 91
    assert binding["rng_before"] == binding["rng_after"]
    assert binding["full_hash_before"] == binding["full_hash"]
    graph = terminal["terminal"]
    nodes, edges = a.maps(graph)
    assert a.inventory(nodes, edges, 91) == graph
    wire = {"nodes": slots(graph["nodes"], lambda v: v["data"]),
            "edges": slots(graph["edges"], lambda v: v),
            "node_to_edges": slots(graph["nodes"], lambda v: [key(e) for e in v["incident"]]),
            "label_index": {str(label): [key(e) for e in ids] for label, ids in graph["label_indexes"]},
            "step": 91}
    wire["node_to_edges"][0]["version"] = 0  # secondary-map sentinel, not arena
    paths = ["crates/hg-engine/src/matcher.rs", "crates/hg-rules/src/compiled.rs",
             "crates/hg-core/src/graph.rs", "crates/hg-core/src/arena.rs", "crates/hg-core/src/snapshot.rs"]
    provenance = {}
    for rel in paths:
        frozen = ARCHIVE / "source_snapshots" / rel
        frozen_sha = a.digest(frozen)
        assert frozen_sha == inv["sha256"]["source_snapshots/" + rel]
        live_sha = a.digest(ROOT / rel)
        provenance[rel] = {"frozen_sha256": frozen_sha, "live_sha256": live_sha, "identical": frozen_sha == live_sha}
    for rel in paths[:2] + paths[4:]:
        assert provenance[rel]["identical"], "matcher/rule/snapshot source drift"
    print(json.dumps({"development_only": True, "gate_pass": False, "training_admitted": False,
                      "source_archive": str(ARCHIVE.relative_to(ROOT)), "source_stream_sha256": inv["sha256"][source],
                      "source_provenance": provenance,
                      "scope": "read_only_matcher_snapshot_not_allocator_or_engine_continuation",
                      "index_order": "canonical inventory order; target rules use global arena scan, not incident/label indexes",
                      "inventory": graph, "store_wire": wire, "table": start["table"],
                      "recorded_binding_step": binding, "recorded_terminal_flags": {k: v for k, v in terminal.items() if k != "terminal"}},
                     sort_keys=True, separators=(",", ":")))


if __name__ == "__main__":
    main()
