#!/usr/bin/env python3
"""Frozen closure adapter: revised work provenance, unchanged outcome checks."""
import copy
from pathlib import Path

import verify_ere_complete_coherent_draw_assay as a

PROFILE = "E2-complete-coherent-closure-v1"
CONTRACT_MD_SHA = "75b801d0b03e0e746a8184625f582a71bdede5031f2f78abb7dba71c1b13dc6a"
CONTRACT_JSON_SHA = "42bc764d215f3ac41783bdc3d7c787a8206f5bd2563a6b9daafaaeb515f6bffc"
WORK_MODEL = "repeated-self-pair-feasibility-v1"
original_manifest = a.check_manifest


def design(out):
    d = out / "source_snapshots/docs"
    assert a.digest(d / "ere_coherent_closure_campaign_v1.md") == CONTRACT_MD_SHA
    assert a.digest(d / "ere_coherent_closure_campaign_v1.json") == CONTRACT_JSON_SHA
    contract = a.load(d / "ere_coherent_closure_campaign_v1.json")
    original_path = d / "ere_complete_coherent_draw_assay_v1.json"
    assert a.digest(original_path) == "80c8cbc072a393d8bddd921b5c4dbf88ac60e8e4b1ebb50dfb7a0683aa967dba"
    original = a.load(original_path)
    for key in ["stages", "parents", "primary", "completeness_controls", "execution",
                "classification", "thresholds", "proposed_resources", "meta_limits",
                "acceptance", "development", "allocator", "launch", "kernel_profile"]:
        assert contract[key] == original[key], f"scientific contract changed: {key}"
    return contract


def check_manifest(m, d, development):
    original_manifest(m, d, development)
    assert m["matcher_work_model"] == d["closure_campaign"]["matcher_work_model"] == WORK_MODEL
    prerequisites = m["closure_prerequisites"]
    assert prerequisites["targeted_replay_pass"] is True
    assert prerequisites["corrected_preserved_audit_pass"] is True
    assert prerequisites["targeted_trace_audit_pass"] is True


original_integrity = a.integrity


def integrity(out, development, freezing=False):
    m, d = original_integrity(out, development, freezing)
    frozen = out / "source_snapshots/output"
    replay = a.load(frozen / "ere_closure_replay_20260929/replay_result.json")
    traces = a.load(frozen / "ere_closure_replay_20260929/trace_audit.json")
    audit = a.load(frozen / "ere_closure_preserved_audit_20260929/audit_full.json")
    assert replay["closure_prerequisite_pass"] and replay["from_initial_conditions"]
    assert replay["parent_measured_steps"] == replay["child_measured_steps"] == 100
    assert replay["attempted_steps"] == replay["advancing_steps"] == 200
    assert replay["matcher_work_model"] == WORK_MODEL
    assert traces["audit_pass"] and traces["historical_prefix_equal"]
    assert traces["historical_child_measured_prefix_steps"] == 91
    assert audit["audit_pass"] and not audit["gate_pass"] and not audit["measurement_complete"]
    assert audit["tamper_rejections"] >= 27 and audit["seconds"] < 900
    assert audit["peak_rss_bytes"] <= 1024**3
    return m, d


def trajectory_material(row):
    """Exclude only the deliberately revised work-accounting fields."""
    row = copy.deepcopy(row)
    if row["kind"] == "step":
        row.pop("usage")
        for _, guard in row["guards"]:
            guard.pop("enumeration_work")
    return row


def paired_pilot_parity(out, d):
    previous = out / "basis" / d["latest_pilot_baseline"]["archive"]
    fixed = {}
    for name in ["parents.jsonl.gz", "control_manifest.jsonl.gz", "controls.jsonl.gz"]:
        got = a.wire_digest(out / name)
        assert got == a.wire_digest(previous / name), name
        fixed[name] = got
    prefixes = {}
    for name in ["draw_manifest.jsonl.gz", "classifications.jsonl.gz"]:
        prefixes[name], tail = a.exact_prefix(out / name, previous / name)
        assert prefixes[name]["rows"] == 64
        assert tail is None or tail["index"] == 64
    traces = {}
    for name in ["parent_traces.jsonl.gz", "child_traces.jsonl.gz"]:
        current = a.stream(out / name)
        count = 0
        for row in a.stream(previous / name):
            got = next(current, None)
            assert got is not None and trajectory_material(got) == trajectory_material(row), f"pilot trajectory differs: {name} row {count}"
            count += 1
        tail = next(current, None)
        current.close()
        if name.startswith("parent"):
            assert count == 168 * 102 and tail is None
        else:
            assert count == 55 * 102
            assert tail is None or (tail["kind"] == "start" and int(tail["id"].removeprefix("child_")) >= 64)
        traces[name] = count
    rows = []
    for i, row in enumerate(a.stream(out / "classifications.jsonl.gz")):
        if i == 64:
            break
        rows.append(row)
    summary = a.tally(rows)
    summary["parent_baselines_complete"] = a.load(out / "classification_summary.json")["parent_baselines_complete"]
    assert summary == a.load(previous / "classification_summary.json")
    return {"identical_fixed_canonical_streams": fixed,
            "identical_pilot_prefixes": prefixes, "equal_trajectory_material_rows": traces,
            "work_accounting_exclusions": ["step.usage", "step.guards[*].enumeration_work"],
            "new_usage_independently_validated": True,
            "pilot_prefix_summary_equal": True, "full_summary_independently_recomputed": True,
            "timing_and_profile_envelopes_excluded": True}


def configure():
    a.PROFILE = PROFILE
    a.CONTRACT_MD_SHA = CONTRACT_MD_SHA
    a.CONTRACT_JSON_SHA = CONTRACT_JSON_SHA
    a.design = design
    a.check_manifest = check_manifest
    a.integrity = integrity
    a.paired_pilot_parity = paired_pilot_parity


if __name__ == "__main__":
    configure()
    a.main()
