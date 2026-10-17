#!/usr/bin/env python3
"""P2 — mu_c(k) re-analysis conditional on baseline survival (v6 plan §P2).

Zero new compute: reads the archived v4 run directories only.

v4 reported an order-of-magnitude gap between mu_c(k=1) and mu_c(k=3) from
UNCONDITIONED survival. v5's C1 then established that deaths at mu = 0 are
intrinsic world instability, not fission cost -- which means a chunk of the
"mutational" deaths at every mu were seeds that were going to die anyway.
Restricting to the seeds that survive at mu = 0 removes that inherited
baseline and asks whether the k-effect survives.
"""
import re, sys
from pathlib import Path

RUNS = Path(__file__).resolve().parent / "runs"
SEEDS = [1, 2, 3, 7, 42, 99, 123, 777]
# (family prefix, population) -> the 20000-step v4 sweep families.
FAMILIES = {
    "churn3": ["E1_flux_mu{mu}_churn3", "E3g_flux_mu{mu}_churn3"],
    "churnk": ["E3_flux_mu{mu}_churnk", "E3f_flux_mu{mu}_churnk"],
}
MUS = ["0", "0.005", "0.01", "0.02", "0.03", "0.05", "0.1", "0.2", "0.4", "0.7", "1.0"]


def read_summary(d):
    """(survived, total births) or None if the run is not archived."""
    f = d / "summary.txt"
    if not f.exists():
        return None
    t = f.read_text()
    m = re.search(r"survived=(\w+)", t)
    if not m:
        return None
    b = re.search(r"births_clonal=(\d+) births_mutant=(\d+)", t)
    births = int(b.group(1)) + int(b.group(2)) if b else 0
    return (m.group(1) == "true", births)


def load(pop):
    """mu -> {seed: (survived, births)}. Skips mu with no archived family."""
    out = {}
    for mu in MUS:
        for pat in FAMILIES[pop]:
            got = {}
            for seed in SEEDS:
                d = RUNS / (pat.format(mu=mu) + f"_seed{seed}")
                v = read_summary(d)
                if v is not None:
                    got[seed] = v
            if len(got) == len(SEEDS):
                out[mu] = got
                break
    return out


def brackets(surv_by_mu, order, label):
    """Highest mu with full survival, and lowest mu with none."""
    full = [m for m in order if surv_by_mu[m][1] > 0 and surv_by_mu[m][0] == surv_by_mu[m][1]]
    zero = [m for m in order if surv_by_mu[m][0] == 0]
    lo = full[-1] if full else None
    hi = zero[0] if zero else None
    print(f"  {label}: last mu with 100% survival = {lo}, first mu with 0% survival = {hi}")
    return lo, hi


def muc_bracket(counts, order):
    """v4's definition: the grid interval in which survival crosses 0.5."""
    lo = None
    for mu in order:
        n, d = counts[mu]
        if d == 0:
            continue
        if n / d > 0.5:
            lo = mu
        elif n / d <= 0.5:
            return (lo, mu)
    return (lo, None)


def main():
    results = {}
    for pop in ("churnk", "churn3"):
        data = load(pop)
        order = [m for m in MUS if m in data]
        if not order or "0" not in data:
            print(f"{pop}: no usable archive", file=sys.stderr)
            continue
        stratum = sorted(s for s, (ok, _) in data["0"].items() if ok)
        # EXPOSED stratum: baseline-surviving AND actually reproducing at mu=0.
        # mutate-at-birth is the only variation channel, so a lineage that
        # never gives birth is untouched by mu no matter how high mu is set.
        # Counting such a seed as "survived the mutation load" is vacuous.
        exposed = sorted(s for s in stratum if data["0"][s][1] > 0)
        results[pop] = [data, order, stratum, exposed]

    print("P2 -- mu_c(k) conditional on baseline (mu=0) survival")
    print("   source: archived v4 runs, 20000 steps, flux, flat income, match_cap_per_rule=64")
    print("   NOTE: all rows are conditioned on cap 64 (ground rule 7).\n")

    for pop, k in (("churnk", 1), ("churn3", 3)):
        if pop not in results:
            continue
        data, order, stratum, exposed = results[pop]
        print(f"=== {pop} (k={k}) ===")
        print(f"  baseline-surviving stratum (mu=0):  {stratum}  (n={len(stratum)}/{len(SEEDS)})")
        print(f"  ...of which REPRODUCING at mu=0:    {exposed}  (n={len(exposed)})")
        never = [s for s in stratum if s not in exposed]
        if never:
            print(f"  ...never reproduce, so mu cannot act on them: {never}")
        print(f"\n  {'mu':>7} {'uncond':>12} {'conditional':>14} {'cond+exposed':>15}   births(exposed)")
        uncond, cond, expo = {}, {}, {}
        for mu in order:
            u = sum(1 for s in SEEDS if data[mu][s][0])
            c = sum(1 for s in stratum if data[mu][s][0])
            e = sum(1 for s in exposed if data[mu][s][0])
            uncond[mu] = (u, len(SEEDS))
            cond[mu] = (c, len(stratum))
            expo[mu] = (e, len(exposed))
            bl = ",".join(str(data[mu][s][1]) for s in exposed)
            print(f"  {mu:>7} {f'{u}/{len(SEEDS)}':>12} {f'{c}/{len(stratum)}':>14} "
                  f"{f'{e}/{len(exposed)}':>15}   {bl}")
        print()
        for tag, cs in (("unconditioned", uncond), ("conditional  ", cond), ("cond+exposed ", expo)):
            b = muc_bracket(cs, order)
            print(f"  mu_c {tag}: ({b[0]}, {b[1]}]")
        # The 0.5-crossing definition has resolution 1/n. On a stratum of 2 a
        # single seed moves it a whole grid interval, so report two endpoints
        # that do not degenerate: where the stratum first loses a member, and
        # where it goes fully extinct.
        first = next((m for m in order if expo[m][0] < expo[m][1]), None)
        prev = None
        for m in order:
            if expo[m][0] < expo[m][1]:
                break
            prev = m
        ext, extprev = None, None
        for m in order:
            if expo[m][0] == 0:
                ext = m
                break
            extprev = m
        print(f"  exposed stratum: first loss in ({prev}, {first}], full extinction in ({extprev}, {ext}]")
        results[pop] += [uncond, cond, expo, (prev, first), (extprev, ext)]
        print()

    # Pre-committed reading (plan §P2).
    if "churnk" in results and "churn3" in results:
        _, o1, st1, ex1, _, c1, e1, f1, x1 = results["churnk"]
        _, o3, st3, ex3, _, c3, e3, f3, x3 = results["churn3"]
        print("=== verdict ===")
        print(f"  v4 reported (unconditioned): mu_c(k=1) = (0.02, 0.03], mu_c(k=3) = (0, 0.005]")
        print(f"                               -> 'lower by at least an order of magnitude'\n")
        print(f"  {'':<26} {'k=1 (churnK)':>18} {'k=3 (churn3)':>18}")
        print(f"  {'baseline-surviving n':<26} {len(st1):>18} {len(st3):>18}")
        print(f"  {'...of which reproduce':<26} {len(ex1):>18} {len(ex3):>18}")
        print(f"  {'mu_c, conditional':<26} {str(muc_bracket(c1, o1)):>18} {str(muc_bracket(c3, o3)):>18}")
        print(f"  {'mu_c, cond+exposed':<26} {str(muc_bracket(e1, o1)):>18} {str(muc_bracket(e3, o3)):>18}")
        print(f"  {'first exposed loss in':<26} {str(f1):>18} {str(f3):>18}")
        print(f"  {'exposed extinction in':<26} {str(x1):>18} {str(x3):>18}")
        n = min(len(ex1), len(ex3))
        if n < 5:
            print(f"\n  The k=3 arm's EFFECTIVE sample is n={len(ex3)}, not 8: the other")
            print(f"  {len(st3) - len(ex3)} baseline-surviving churn3 seeds never reproduce, so mutate-at-birth")
            print("  -- the only variation channel -- never fires on them at any mu. v4's E3")
            print(f"  compared {len(ex3)} informative seeds against {len(ex1)}. At that resolution the 0.5-crossing")
            print("  bracket moves a full grid interval when one seed changes, and the")
            print("  order-of-magnitude claim is not supported in either direction.")


if __name__ == "__main__":
    main()
