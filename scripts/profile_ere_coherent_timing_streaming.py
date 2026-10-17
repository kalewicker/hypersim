#!/usr/bin/env python3
"""Read-only, timing-stage-only engineering profile; never emits admission."""
import argparse
import json
from pathlib import Path
import resource
import sys
import time

import verify_ere_complete_coherent_draw_assay as a


def main():
    if not __debug__:
        raise SystemExit("requires Python assertions enabled")
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("archive", type=Path)
    p.add_argument("inventory", type=Path)
    p.add_argument("--attempted", type=int, required=True)
    p.add_argument("--children", type=int, required=True)
    args = p.parse_args()
    started = time.monotonic()
    a.DEADLINE = started + 120
    a.MEMORY = 1024**3
    inv = a.load(args.inventory)
    assert inv["not_a_completion_seal"] is True
    assert args.archive.resolve() == (Path(__file__).resolve().parents[1] / inv["archive"]).resolve()
    inputs = ["native_controls_timing.json", "native_trajectory_timing.json", "classifications.jsonl.gz", "manifest.json", "run_status.json"]
    for name in inputs:
        assert a.digest(args.archive / name) == inv["sha256"][name], name
    m = a.load(args.archive / "manifest.json")
    assert m["draws"] == 10000 and m["horizon"] == 100 and not m["development_only"]
    a.check_native_timing(args.archive, m, {"attempted": args.attempted, "child_traces": args.children})
    a.guard()
    rss = resource.getrusage(resource.RUSAGE_SELF).ru_maxrss * (1 if sys.platform == "darwin" else 1024)
    print(json.dumps({"development_only": True, "scope": "timing_stage_only", "timing_check_pass": True,
                      "complete_full_audit_pass": False, "gate_pass": False, "training_admitted": False,
                      "classification_rows_declared": m["draws"], "native_children": args.children,
                      "attempted_steps_cross_checked": args.attempted,
                      "seconds": time.monotonic() - started, "peak_rss_bytes": rss,
                      "wall_cap_seconds": 120, "rss_cap_bytes": a.MEMORY,
                      "checker_sha256": a.digest(Path(a.__file__)),
                      "input_sha256": {name: inv["sha256"][name] for name in inputs}}, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
