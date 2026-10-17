#!/usr/bin/env python3
"""v4 analysis: persistence and novelty windows over mu, per economy."""
import glob, os, re, statistics, sys

THRESH = 1.0e-4          # pre-registered novelty slope threshold (runner.rs)
PERSIST = 0.6            # pre-registered persistence threshold (plan §7)


def parse(d):
    p = os.path.join(d, "summary.txt")
    if not os.path.exists(p):
        return None
    txt = open(p).read()
    def g(k, cast=str, default=None):
        m = re.search(rf"\b{k}=([^\s]+)", txt)
        return cast(m.group(1)) if m else default
    def num(k):
        v = g(k)
        if v in (None, "none"):
            return None
        return int(v)
    return dict(
        seed=g("seed", int), economy=g("economy"), mu=g("mu", float),
        pop=g("seed_pop"), term=g("term"), survived=g("survived") == "true",
        m_rules=g("M_rules", int), m_data=g("M_data", int),
        t_ff=num("time_to_first_fission"), t_freeze=num("time_to_freeze"),
        s_rules=g("slope_M_rules", float), s_data=g("slope_M_data", float),
        clonal=g("births_clonal", int), mutant=g("births_mutant", int),
        noop=g("mutant_noop", int), audit=g("flow_audit_residual", int),
        dir=d,
    )


def collect(tag):
    """Only v4 run dirs: `<tag>_<economy>_mu<mu>_<pop>_seed<n>`. The bare
    `<tag>_*` glob would also sweep up v3's leftover E1/E1b/E1c directories,
    whose summaries predate the economy and flow-audit fields."""
    rows = []
    for d in sorted(glob.glob(f"crates/hg-selfmod/runs/{tag}_stock_*")
                    + glob.glob(f"crates/hg-selfmod/runs/{tag}_flux_*")):
        if d.endswith(".log") or not os.path.isdir(d):
            continue
        r = parse(d)
        if r and r["economy"] is not None:
            rows.append(r)
    return rows


def group(rows):
    g = {}
    for r in rows:
        g.setdefault((r["economy"], r["mu"]), []).append(r)
    return g


def report(tag, rows):
    bad = [r for r in rows if r["audit"] != 0]
    print(f"=== {tag}: {len(rows)} runs, flow-audit residual nonzero in {len(bad)} ===")
    g = group(rows)
    econs = sorted({e for e, _ in g})
    for econ in econs:
        mus = sorted(m for e, m in g if e == econ)
        print(f"\n{econ}")
        print(f"{'mu':>6} {'survive':>9} {'P':>5} {'med dM_rules':>13} {'med dM_data':>12} "
              f"{'births(c/m)':>13} {'repro→died':>11} {'norepro→lived':>14}")
        for mu in mus:
            rs = g[(econ, mu)]
            n = len(rs)
            surv = sum(r["survived"] for r in rs)
            P = surv / n
            sr = statistics.median(r["s_rules"] for r in rs)
            sd = statistics.median(r["s_data"] for r in rs)
            c = sum(r["clonal"] for r in rs)
            m = sum(r["mutant"] for r in rs)
            repro_died = sum(1 for r in rs if r["t_ff"] is not None and not r["survived"])
            repro_n = sum(1 for r in rs if r["t_ff"] is not None)
            norepro_lived = sum(1 for r in rs if r["t_ff"] is None and r["survived"])
            norepro_n = sum(1 for r in rs if r["t_ff"] is None)
            print(f"{mu:>6} {surv:>4}/{n:<4} {P:>5.2f} {sr:>13.3e} {sd:>12.3e} "
                  f"{str(c)+'/'+str(m):>13} {str(repro_died)+'/'+str(repro_n):>11} "
                  f"{str(norepro_lived)+'/'+str(norepro_n):>14}")

        P_win = [mu for mu in mus if sum(r["survived"] for r in g[(econ, mu)]) / len(g[(econ, mu)]) >= PERSIST]
        V_win = [mu for mu in mus
                 if statistics.median(r["s_rules"] for r in g[(econ, mu)]) > THRESH
                 and statistics.median(r["s_data"] for r in g[(econ, mu)]) > THRESH]
        overlap = sorted(set(P_win) & set(V_win))
        print(f"  persistence window (P>={PERSIST}): {P_win if P_win else 'EMPTY'}")
        print(f"  novelty window (both medians >{THRESH:g}): {V_win if V_win else 'EMPTY'}")
        print(f"  OVERLAP P∩V: {overlap if overlap else 'EMPTY'}")

    # mu_c: interpolated mu where survival crosses 0.5
    print("\nmu_c (survival crosses 0.5):")
    for econ in econs:
        mus = sorted(m for e, m in g if e == econ)
        pts = [(mu, sum(r["survived"] for r in g[(econ, mu)]) / len(g[(econ, mu)])) for mu in mus]
        mc = None
        for (m0, p0), (m1, p1) in zip(pts, pts[1:]):
            if p0 >= 0.5 > p1:
                mc = (m0, m1)
                break
        if pts and pts[0][1] < 0.5:
            print(f"  {econ}: survival already <0.5 at mu={pts[0][0]} (mu_c at or below the grid floor)")
        elif mc:
            print(f"  {econ}: mu_c brackets ({mc[0]}, {mc[1]}]")
        else:
            print(f"  {econ}: survival never drops below 0.5 on the grid")


if __name__ == "__main__":
    for tag in sys.argv[1:] or ["E1"]:
        rows = collect(tag)
        if rows:
            report(tag, rows)
            print()
        else:
            print(f"(no runs for {tag})")
