#!/usr/bin/env python3
"""Independently replay the targeted trajectories and verify historical prefix."""
import argparse
import json
from pathlib import Path
import resource
import sys
import time

import verify_ere_complete_coherent_draw_assay as a
from verify_ere_coherent_closure import trajectory_material


def main():
    if not __debug__:
        raise RuntimeError("assertions required")
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("archive", type=Path)
    p.add_argument("replay", type=Path)
    args = p.parse_args()
    started = time.monotonic()
    a.DEADLINE = started + 180
    a.MEMORY = 1024**3
    parent = next(row for row in a.stream(args.archive / "parents.jsonl.gz") if row["index"] == 139)
    draw = a.load(args.replay / "draw.json")
    after, decoded = a._checked_draw(parent, draw)
    assert decoded["program"] is not None and draw["index"] == 9730
    fresh = {}
    results = {}
    previous = {}
    for name, id, initial in [("parent_traces.jsonl.gz", "parent_139", parent["inventory"]),
                              ("child_traces.jsonl.gz", "child_9730", after)]:
        rows = list(a.stream(args.replay / name))
        current = iter(rows)
        result = a.consume_trace(current, id, initial, parent, 100)
        assert next(current, None) is None
        assert result["complete"] and result["attempted"] == result["advancing"] == 100
        fresh[id] = rows
        results[id] = {k: result[k] for k in ["complete", "attempted", "advancing", "peak_work", "peak_results"]}
        old = []
        for row in a.stream(args.archive / name):
            if row["id"] == id:
                old.append(row)
                if row["kind"] == "end":
                    break
        assert old
        if id.startswith("parent"):
            assert len(old) == len(rows) == 102
            assert [trajectory_material(r) for r in old] == [trajectory_material(r) for r in rows]
        else:
            assert old[-1]["complete"] is False and old[-1]["measured_steps"] == 91
            assert old[-2]["step"] == 92 and old[-2]["ceiling"] is True
            assert [trajectory_material(r) for r in old[:92]] == [trajectory_material(r) for r in rows[:92]]
            for key in ["full_hash_before", "world_hash_before", "rng_before"]:
                assert old[-2][key] == rows[92][key]
            assert rows[92]["usage"]["work"] == 9018 and rows[92]["usage"]["results"] == 372
            assert rows[92]["ceiling"] is False and rows[92]["store_step"] == 92
        previous[id] = len(old)
    result = {"scope": "targeted_independent_trace_audit", "audit_pass": True,
              "historical_prefix_equal": True, "historical_child_measured_prefix_steps": 91,
              "complete_parent_trajectory_equal": True, "from_initial_conditions": True,
              "traces": results, "historical_rows": previous,
              "matcher_work_model": "repeated-self-pair-feasibility-v1",
              "child_step_92_work": 9018, "child_step_92_results": 372,
              "work_accounting_exclusions": ["step.usage", "step.guards[*].enumeration_work"],
              "gate_pass": False, "training_admitted": False,
              "seconds": time.monotonic() - started,
              "peak_rss_bytes": resource.getrusage(resource.RUSAGE_SELF).ru_maxrss * (1 if sys.platform == "darwin" else 1024),
              "replay_sha256": {str(q.relative_to(args.replay)): a.digest(q) for q in a.files(args.replay)},
              "script_sha256": a.digest(Path(__file__))}
    a.guard()
    with (args.replay / "trace_audit.json").open("x") as f:
        json.dump(result, f, indent=2, sort_keys=True)
        f.write("\n")
    print(json.dumps(result, sort_keys=True))


if __name__ == "__main__":
    main()
