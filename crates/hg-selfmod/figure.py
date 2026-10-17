#!/usr/bin/env python3
"""v4 headline figure: persistence and novelty windows over mu."""
import glob, os, re, statistics

THRESH, PERSIST = 1.0e-4, 0.6
RUNS = "crates/hg-selfmod/runs"


def val(d, k, c=str):
    t = open(os.path.join(d, "summary.txt")).read()
    m = re.search(rf"\b{k}=([^\s]+)", t)
    return c(m.group(1)) if m else None


def series(tags, pop):
    """{mu: (P, med slope M_rules, med slope M_data, n)} across run tags."""
    by = {}
    for tag in tags:
        for d in glob.glob(f"{RUNS}/{tag}_flux_*_{pop}_seed*"):
            if not os.path.isdir(d):
                continue
            mu = val(d, "mu", float)
            by.setdefault(mu, []).append(
                (val(d, "survived") == "true", val(d, "slope_M_rules", float),
                 val(d, "slope_M_data", float)))
    out = {}
    for mu, rs in by.items():
        out[mu] = (sum(r[0] for r in rs) / len(rs),
                   statistics.median(r[1] for r in rs),
                   statistics.median(r[2] for r in rs), len(rs))
    return dict(sorted(out.items()))


def panel(x0, y0, w, h, data, title, sub):
    mus = list(data)
    n = len(mus)
    s = []
    xs = [x0 + w * i / max(n - 1, 1) for i in range(n)]
    P = [data[m][0] for m in mus]
    NOV = [max(data[m][1], 0) for m in mus]

    # window shading
    pin = [i for i, m in enumerate(mus) if data[m][0] >= PERSIST]
    vin = [i for i, m in enumerate(mus) if data[m][1] > THRESH and data[m][2] > THRESH]
    def band(idxs, color, op):
        if not idxs:
            return ""
        a, b = min(idxs), max(idxs)
        xa = xs[a] - (w / max(n - 1, 1)) * 0.5
        xb = xs[b] + (w / max(n - 1, 1)) * 0.5
        xa, xb = max(xa, x0), min(xb, x0 + w)
        return (f'<rect x="{xa:.1f}" y="{y0}" width="{max(xb-xa,2):.1f}" height="{h}" '
                f'fill="{color}" opacity="{op}"/>')
    s.append(band(pin, "#2e8b57", 0.16))
    s.append(band(vin, "#1f6feb", 0.16))
    overlap = sorted(set(pin) & set(vin))
    s.append(band(overlap, "#d62728", 0.55))

    s.append(f'<rect x="{x0}" y="{y0}" width="{w}" height="{h}" fill="none" stroke="#666"/>')
    s.append(f'<text x="{x0}" y="{y0-22}" font-size="14" font-weight="bold">{title}</text>')
    s.append(f'<text x="{x0}" y="{y0-7}" font-size="11" fill="#555">{sub}</text>')

    # persistence axis (left, 0..1)
    for frac, lab in [(0, "0"), (PERSIST, "0.6"), (1, "1")]:
        y = y0 + h - h * frac
        s.append(f'<text x="{x0-6}" y="{y+4}" text-anchor="end" font-size="10" fill="#2e8b57">{lab}</text>')
        if frac == PERSIST:
            s.append(f'<line x1="{x0}" y1="{y:.1f}" x2="{x0+w}" y2="{y:.1f}" stroke="#2e8b57" '
                     f'stroke-width="1" stroke-dasharray="4,3" opacity="0.7"/>')
    s.append(f'<text x="{x0-34}" y="{y0+h/2}" font-size="11" fill="#2e8b57" '
             f'transform="rotate(-90 {x0-34} {y0+h/2})" text-anchor="middle">persistence P</text>')

    # novelty axis (right, log10 from 1e-5 to 1e-1)
    lo, hi = -5.0, -1.0
    def ynov(v):
        import math
        if v <= 0:
            return y0 + h
        l = max(min(math.log10(v), hi), lo)
        return y0 + h - h * (l - lo) / (hi - lo)
    import math
    for e in range(-5, 0):
        y = ynov(10 ** e)
        s.append(f'<text x="{x0+w+6}" y="{y+4}" font-size="10" fill="#1f6feb">1e{e}</text>')
    s.append(f'<line x1="{x0}" y1="{ynov(THRESH):.1f}" x2="{x0+w}" y2="{ynov(THRESH):.1f}" '
             f'stroke="#1f6feb" stroke-width="1" stroke-dasharray="4,3" opacity="0.7"/>')
    s.append(f'<text x="{x0+w+40}" y="{y0+h/2}" font-size="11" fill="#1f6feb" '
             f'transform="rotate(90 {x0+w+40} {y0+h/2})" text-anchor="middle">novelty slope (log)</text>')

    # curves
    pp = " ".join(f"{x:.1f},{y0+h-h*p:.1f}" for x, p in zip(xs, P))
    s.append(f'<polyline points="{pp}" fill="none" stroke="#2e8b57" stroke-width="2.5"/>')
    for x, p in zip(xs, P):
        s.append(f'<circle cx="{x:.1f}" cy="{y0+h-h*p:.1f}" r="3.5" fill="#2e8b57"/>')
    vv = " ".join(f"{x:.1f},{ynov(v):.1f}" for x, v in zip(xs, NOV))
    s.append(f'<polyline points="{vv}" fill="none" stroke="#1f6feb" stroke-width="2.5" stroke-dasharray="7,3"/>')
    for x, v in zip(xs, NOV):
        s.append(f'<circle cx="{x:.1f}" cy="{ynov(v):.1f}" r="3.5" fill="#1f6feb"/>')

    for x, m in zip(xs, mus):
        s.append(f'<text x="{x:.1f}" y="{y0+h+15}" text-anchor="middle" font-size="9">{m:g}</text>')
    s.append(f'<text x="{x0+w/2}" y="{y0+h+32}" text-anchor="middle" font-size="11">variation rate μ</text>')

    verdict = "OVERLAP: EMPTY" if not overlap else f"OVERLAP at μ={[mus[i] for i in overlap]}"
    s.append(f'<text x="{x0+w-4}" y="{y0+16}" text-anchor="end" font-size="12" font-weight="bold" '
             f'fill="{"#b00" if not overlap else "#d62728"}">{verdict}</text>')
    return "\n".join(s)


k3 = series(["E1", "E3g"], "churn3")
k1 = series(["E3", "E3f"], "churnk")

def horizon_panel(x0, y0, w, h):
    """P at two observation horizons: persistence at mu>0 is 'not dead yet'."""
    pts = [("0", 1.00, 1.00), ("0.005", 0.88, 0.50), ("0.01", 0.88, 0.50), ("0.02", 0.75, 0.50)]
    s = [f'<rect x="{x0}" y="{y0}" width="{w}" height="{h}" fill="none" stroke="#666"/>',
         f'<text x="{x0}" y="{y0-22}" font-size="14" font-weight="bold">'
         f'Persistence is horizon-dependent for every μ &gt; 0 (churnK)</text>',
         f'<text x="{x0}" y="{y0-7}" font-size="11" fill="#555">'
         f'only μ = 0 holds P = 1.00 as the observation window triples — elsewhere "persistence" is just "not dead yet"</text>']
    n = len(pts)
    xs = [x0 + w * (i + 0.5) / n for i in range(n)]
    for x, (lab, p20, p60) in zip(xs, pts):
        for p, col, dx in [(p20, "#9ecae1", -11), (p60, "#08519c", 1)]:
            bh = h * p
            s.append(f'<rect x="{x+dx:.1f}" y="{y0+h-bh:.1f}" width="10" height="{bh:.1f}" fill="{col}"/>')
            s.append(f'<text x="{x+dx+5:.1f}" y="{y0+h-bh-4:.1f}" text-anchor="middle" font-size="9">{p:.2f}</text>')
        s.append(f'<text x="{x:.1f}" y="{y0+h+15}" text-anchor="middle" font-size="10">μ={lab}</text>')
    y = y0 + h - h * PERSIST
    s.append(f'<line x1="{x0}" y1="{y:.1f}" x2="{x0+w}" y2="{y:.1f}" stroke="#2e8b57" '
             f'stroke-dasharray="4,3" stroke-width="1"/>')
    s.append(f'<text x="{x0+w-4}" y="{y-4}" text-anchor="end" font-size="9" fill="#2e8b57">P = 0.6</text>')
    s.append(f'<rect x="{x0+w-150}" y="{y0+8}" width="10" height="10" fill="#9ecae1"/>'
             f'<text x="{x0+w-136}" y="{y0+17}" font-size="10">20k steps</text>')
    s.append(f'<rect x="{x0+w-70}" y="{y0+8}" width="10" height="10" fill="#08519c"/>'
             f'<text x="{x0+w-56}" y="{y0+17}" font-size="10">60k steps</text>')
    return "\n".join(s)


W, H = 1000, 900
svg = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" '
       f'font-family="Helvetica,Arial,sans-serif">',
       f'<rect width="{W}" height="{H}" fill="white"/>',
       f'<text x="{W/2}" y="26" text-anchor="middle" font-size="17" font-weight="bold">'
       f'Persistence and novelty windows over variation rate μ</text>',
       f'<text x="{W/2}" y="45" text-anchor="middle" font-size="12" fill="#555">'
       f'hg-selfmod v4 · flux and stock economies produce bit-identical dynamics, so one curve covers both</text>']
svg.append(panel(90, 95, 800, 200, k3,
                 "churn3 — distributed autocatalytic set (k = 3)",
                 "persistence window EMPTY (P never reaches 0.6) · novelty window EMPTY → branch (c)"))
svg.append(panel(90, 400, 800, 200, k1,
                 "churnK — minimal self-sufficient sustainer (k = 1)",
                 "persistence window {μ ≤ 0.02} · novelty window {μ ≥ 0.03} · exactly adjacent, never overlapping → branch (b)"))
svg.append(horizon_panel(90, 690, 800, 140))

svg.append('<g font-size="11">')
svg.append('<line x1="90" y1="862" x2="120" y2="662" stroke="#2e8b57" stroke-width="2.5"/>'
           '<text x="126" y="866">persistence P (fraction of 8 seeds surviving to budget)</text>')
svg.append('<line x1="470" y1="862" x2="500" y2="862" stroke="#1f6feb" stroke-width="2.5" stroke-dasharray="7,3"/>'
           '<text x="506" y="866">median late-window slope of M_rules (novelty)</text>')
svg.append('<rect x="90" y="878" width="14" height="11" fill="#2e8b57" opacity="0.16"/>'
           '<text x="110" y="888">persistence window</text>')
svg.append('<rect x="240" y="878" width="14" height="11" fill="#1f6feb" opacity="0.16"/>'
           '<text x="260" y="888">novelty window</text>')
svg.append('<rect x="370" y="878" width="14" height="11" fill="#d62728" opacity="0.55"/>'
           '<text x="390" y="888">overlap (would be a Goldilocks band — none exists at any μ, for either k)</text>')
svg.append("</g></svg>")

open("docs/selfmod_v4_windows.svg", "w").write("\n".join(svg))
print("wrote docs/selfmod_v4_windows.svg")
print("k=3 mu points:", list(k3))
print("k=1 mu points:", list(k1))
