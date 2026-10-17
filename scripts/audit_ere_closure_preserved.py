#!/usr/bin/env python3
"""Complete corrected audit of preserved evidence; writes a separate receipt."""
import argparse
import json
from pathlib import Path
import resource
import sys
import time

import verify_ere_complete_coherent_draw_assay as a


def main():
    if not __debug__:
        raise RuntimeError("assertions required")
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("archive", type=Path)
    p.add_argument("inventory", type=Path)
    p.add_argument("output", type=Path)
    args = p.parse_args()
    inv = a.load(args.inventory)
    assert inv["not_a_completion_seal"] is True
    assert args.archive.resolve() == (Path(__file__).resolve().parents[1] / inv["archive"]).resolve()
    assert not args.output.exists() and not args.output.is_symlink()

    def preserved():
        assert {str(q.relative_to(args.archive)) for q in a.files(args.archive)} == set(inv["sha256"])
        for name, sha in inv["sha256"].items():
            assert a.digest(args.archive / name) == sha, name

    preserved()
    started = time.monotonic()
    a.DEADLINE = started + 900
    a.MEMORY = 1024**3
    a.ARCHIVE_ROOT = None
    result = a.audit(args.archive, "full", False)
    a.guard()
    # This check belongs to the new external receipt, not the failed archive.
    assert result["audit_pass"] is True
    assert result["gate_pass"] is False and result["measurement_complete"] is False
    result.update({"receipt_kind": "external_corrected_full_audit",
                   "historical_archive_status_changed": False,
                   "original_frozen_auditor_replaced": False,
                   "archive": str(args.archive),
                   "checker_sha256": a.digest(Path(a.__file__)),
                   "preservation_inventory_sha256": a.digest(args.inventory),
                   "seconds": time.monotonic() - started,
                   "peak_rss_bytes": resource.getrusage(resource.RUSAGE_SELF).ru_maxrss * (1 if sys.platform == "darwin" else 1024),
                   "optimization_stats": a.optimization_stats()})
    preserved()
    args.output.mkdir(parents=True)
    with (args.output / "audit_full.json").open("x") as f:
        json.dump(result, f, indent=2, sort_keys=True)
        f.write("\n")
    print(json.dumps({k: result[k] for k in ["audit_pass", "gate_pass", "measurement_complete", "seconds", "peak_rss_bytes", "tamper_rejections"]}))


if __name__ == "__main__":
    main()
