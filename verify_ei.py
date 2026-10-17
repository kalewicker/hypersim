#!/usr/bin/env python3
"""
verify_ei.py — independent verifier for Hypersim's hg-causal EI/CE calculator.

Recomputes EI_micro, determinism, degeneracy, and per-partition EI_macro / CE
from a raw transition-count dump, sharing no code with the Rust
implementation. Agreement to ~1e-4 across two independent implementations is
strong evidence both are correct; disagreement localizes the bug.

USAGE
  python3 verify_ei.py --self-test
      First run this. Re-derives the hand-verified 4-state fixture
      (EI_micro ~= 0.8113, EI_macro = 1.0, CE ~= 1.2325) and checks the
      mathematical invariants on random TPMs. If self-test fails, the
      verifier itself is wrong — fix that before trusting any verdicts.

  python3 verify_ei.py dump.json [--expect-ei-micro X] [--expect-ce NAME=X ...]
      Verify a real dump. Prints all quantities, runs invariant checks,
      and (optionally) compares against the values the Rust side reported.

EXPECTED DUMP FORMAT (ask Claude Code to add a debug dump in this shape):
{
  "classes": ["150dcce2706392d9", "..."],     // WL-hash names, index-aligned
  "counts": [[12, 0, ...], [...], ...],       // MxM transition counts (row=from)
  "cadence_steps": 128,                        // steps between observations
  "partitions": {                              // optional; group index per class
    "functional": [0, 0, 1, 2, 1, ...],
    "structural": [0, 1, 1, 0, ...]
  }
}

CONVENTION (must match the Rust side; a mismatch here shows up as a
systematic, explainable difference rather than a bug):
  - Rows with zero total count (classes never observed as a source) are
    EXCLUDED from M. Their columns remain as destinations.
  - Entropy is log base 2, with 0*log(0) = 0.
"""

import argparse
import json
import math
import sys

import numpy as np

TOL = 1e-9          # numerical tolerance for invariant checks
CMP_TOL = 1e-4      # tolerance when comparing against Rust-reported values


# ── Core math (deliberately minimal; mirror the spec, not the Rust code) ──────

def entropy_bits(p: np.ndarray) -> float:
    p = np.asarray(p, dtype=float)
    nz = p[p > 0.0]
    return float(-(nz * np.log2(nz)).sum())


def normalize(counts: np.ndarray):
    """Row-normalize, excluding zero rows from M (per convention above).

    Returns (tpm, kept_row_indices). tpm has shape (M, N_cols) where M is the
    number of nonzero rows; columns are NOT trimmed (row-less classes remain
    legal destinations).
    """
    counts = np.asarray(counts, dtype=float)
    row_sums = counts.sum(axis=1)
    kept = np.where(row_sums > 0)[0]
    tpm = counts[kept] / row_sums[kept, None]
    return tpm, kept


def ei_bits(tpm: np.ndarray) -> tuple[float, float, float]:
    """Returns (EI, determinism, degeneracy). M=1 degenerates to zeros."""
    m = tpm.shape[0]
    if m <= 1:
        return 0.0, 0.0, 0.0
    interventional = tpm.mean(axis=0)          # uniform @ tpm == mean of rows
    h_int = entropy_bits(interventional)
    h_noise = float(np.mean([entropy_bits(row) for row in tpm]))
    ei = h_int - h_noise
    log2m = math.log2(m)
    return ei, 1.0 - h_noise / log2m, 1.0 - h_int / log2m


def coarsen(tpm: np.ndarray, kept_rows: np.ndarray, partition: list[int]) -> np.ndarray:
    """Macro TPM: T[A][B] = mean over source rows i in A of sum over dest j in B.

    `partition[c]` is the group of ORIGINAL class index c (covers all classes,
    including row-less ones, since they appear as destination columns).
    """
    groups = sorted(set(partition))
    gidx = {g: k for k, g in enumerate(groups)}
    k = len(groups)
    n_cols = tpm.shape[1]

    # Sum destination columns into groups.
    dest = np.zeros((tpm.shape[0], k))
    for j in range(n_cols):
        dest[:, gidx[partition[j]]] += tpm[:, j]

    # Average source rows within groups (rows are indexed by kept_rows'
    # original class indices).
    macro = np.zeros((k, k))
    for a in groups:
        members = [r for r, orig in enumerate(kept_rows) if partition[orig] == a]
        if members:
            macro[gidx[a]] = dest[members].mean(axis=0)
    return macro


def ce(ei_micro: float, ei_macro: float) -> float:
    return ei_macro / ei_micro if ei_micro > 0.0 else 0.0


# ── Invariant checks (must hold for ANY correct implementation) ───────────────

def check_invariants(tpm: np.ndarray, kept: np.ndarray, verbose=True) -> bool:
    ok = True
    m = tpm.shape[0]
    ei, det, deg = ei_bits(tpm)
    log2m = math.log2(m) if m > 1 else 0.0

    def report(name, passed, detail=""):
        nonlocal ok
        ok &= passed
        if verbose or not passed:
            print(f"  [{'PASS' if passed else 'FAIL'}] {name}{': ' + detail if detail else ''}")

    # EI >= 0 (Jensen: entropy is concave, interventional = mean of rows).
    report("EI >= 0", ei >= -TOL, f"EI = {ei:.6f}")
    # EI <= log2(M).
    report("EI <= log2(M)", ei <= log2m + TOL, f"log2(M) = {log2m:.6f}")
    # Identity: EI == (det - deg) * log2(M).
    report("EI == (det - deg) * log2(M)",
           abs(ei - (det - deg) * log2m) < 1e-6 if m > 1 else True)
    # Identity partition (every class its own group) => macro == micro => CE = 1.
    if m > 1:
        n_classes = tpm.shape[1]
        ident = list(range(n_classes))
        macro = coarsen(tpm, kept, ident)
        ei_id, _, _ = ei_bits(macro)
        report("identity partition => CE = 1.0",
               abs(ce(ei, ei_id) - 1.0) < 1e-6, f"CE = {ce(ei, ei_id):.6f}")
        # Total partition (K = 1) => EI_macro = 0 => CE = 0.
        total = [0] * n_classes
        macro1 = coarsen(tpm, kept, total)
        ei_1, _, _ = ei_bits(macro1)
        report("total partition => CE = 0", abs(ce(ei, ei_1)) < 1e-9)
        # Any macro row must be stochastic.
        rs = macro.sum(axis=1)
        report("coarsened rows stochastic", bool(np.allclose(rs, 1.0, atol=1e-9)))
    return ok


# ── Self-test: the hand-verified fixture and random-TPM invariants ────────────

def self_test() -> int:
    print("Self-test: hand-verified 4-state degenerate fixture")
    counts = np.array([
        [0, 0, 0, 3],
        [0, 0, 0, 3],
        [0, 0, 0, 3],
        [1, 1, 1, 0],
    ], dtype=float)
    tpm, kept = normalize(counts)
    ei_micro, det, deg = ei_bits(tpm)
    macro = coarsen(tpm, kept, [0, 0, 0, 1])
    ei_macro, _, _ = ei_bits(macro)
    c = ce(ei_micro, ei_macro)

    checks = [
        ("EI_micro ~= 0.8113", abs(ei_micro - 0.81128) < 1e-3, f"{ei_micro:.5f}"),
        ("EI_macro == 1.0",     abs(ei_macro - 1.0)     < 1e-9, f"{ei_macro:.5f}"),
        ("CE ~= 1.2325",        abs(c - 1.23262)        < 1e-3, f"{c:.5f}"),
        ("determinism ~= 0.8019", abs(det - 0.80188)    < 1e-3, f"{det:.5f}"),
        ("degeneracy ~= 0.3962",  abs(deg - 0.39624)    < 1e-3, f"{deg:.5f}"),
    ]
    ok = True
    for name, passed, val in checks:
        ok &= passed
        print(f"  [{'PASS' if passed else 'FAIL'}] {name} (got {val})")

    print("Self-test: invariants on random row-stochastic TPMs")
    rng = np.random.default_rng(42)
    for i in range(5):
        m = int(rng.integers(2, 9))
        raw = rng.random((m, m)) ** 3          # skewed, more structured
        cnt = np.ceil(raw * 100)
        t, k = normalize(cnt)
        ok &= check_invariants(t, k, verbose=False)
    print(f"Self-test overall: {'PASS' if ok else 'FAIL'}")
    return 0 if ok else 1


# ── Main: verify a real dump ──────────────────────────────────────────────────

def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("dump", nargs="?", help="JSON dump from the Rust side")
    ap.add_argument("--self-test", action="store_true")
    ap.add_argument("--expect-ei-micro", type=float, default=None,
                    help="EI_micro value the Rust side reported")
    ap.add_argument("--expect-ce", action="append", default=[],
                    metavar="NAME=VALUE",
                    help="Rust-reported CE per partition, e.g. functional=0.42")
    args = ap.parse_args()

    if args.self_test:
        return self_test()
    if not args.dump:
        ap.error("provide a dump file or --self-test")

    with open(args.dump) as f:
        dump = json.load(f)

    counts = np.asarray(dump["counts"], dtype=float)
    classes = dump.get("classes", [str(i) for i in range(counts.shape[0])])
    cadence = dump.get("cadence_steps")
    total = int(counts.sum())

    tpm, kept = normalize(counts)
    ei_micro, det, deg = ei_bits(tpm)

    print(f"classes: {len(classes)} total, {tpm.shape[0]} with observed source rows")
    print(f"transitions: {total}" + (f"   cadence: {cadence} steps" if cadence else ""))
    print(f"EI_micro     = {ei_micro:.6f} bits")
    print(f"determinism  = {det:.6f}")
    print(f"degeneracy   = {deg:.6f}")
    if total < 1000:
        print("NOTE: below the 1000-transition gate — the Rust side should NOT "
              "have emitted a claim for this TPM.")

    expected_ce = dict(kv.split("=") for kv in args.expect_ce)
    ok = True

    for name, part in dump.get("partitions", {}).items():
        macro = coarsen(tpm, kept, part)
        ei_macro, _, _ = ei_bits(macro)
        c = ce(ei_micro, ei_macro)
        k = len(set(part))
        line = f"partition '{name}' (K={k}): EI_macro = {ei_macro:.6f}, CE = {c:.6f}"
        if name in expected_ce:
            exp = float(expected_ce[name])
            match = abs(c - exp) < CMP_TOL
            ok &= match
            line += f"   vs Rust {exp:.6f}  [{'MATCH' if match else 'MISMATCH'}]"
        print(line)

    if args.expect_ei_micro is not None:
        match = abs(ei_micro - args.expect_ei_micro) < CMP_TOL
        ok &= match
        print(f"EI_micro vs Rust {args.expect_ei_micro:.6f}: "
              f"[{'MATCH' if match else 'MISMATCH'}]")

    print("Invariant checks:")
    ok &= check_invariants(tpm, kept)
    print(f"Overall: {'PASS' if ok else 'FAIL'}")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
