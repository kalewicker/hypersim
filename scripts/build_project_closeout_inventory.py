"""Read-only bounded evidence inventory; writes documentation, never runs experiments.

Run from any directory. The Markdown catalogue is navigation, not scientific
adjudication. JSON retains file identities and raw top-level status fields.
"""
import hashlib
import json
import os
from collections import Counter, defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
STEM = 'project_experimental_inventory_20261013'
EXCLUDE = {'target', 'output', 'runs', 'vendor', '.git', 'node_modules',
           'source_snapshots', 'frozen_source', '__pycache__'}
METADATA = {'manifest.json', 'campaign_manifest.json', 'status.json',
            'run_status.json', 'audit.json', 'audit_full.json',
            'interpretation.json', 'completion_verification.json'}
FIELDS = {'status', 'state', 'profile', 'outcome', 'verdict', 'gate_pass',
          'scientific_acceptance', 'training_admitted', 'evolution_demonstrated',
          'independent_campaign_audit_pass', 'scientific_worlds', 'sampling_slots',
          'assay_requests', 'native_rewrites', 'measurement_complete',
          'candidate_qualifies', 'last_completed_generation', 'final_step',
          'max_steps', 'error'}


def digest(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()


def group(p):
    name = p.name
    if p.parent == ROOT: return 'Project specifications'
    for prefix, title in [('cumulative', 'Final cumulative evolution'),
                          ('final_cumulative', 'Final cumulative evolution'),
                          ('evolving_minds', 'Evolving Minds'),
                          ('constructive_chemistry', 'Constructive chemistry'),
                          ('selfmod', 'Self-modification'), ('ere_', 'ERE and grammar'),
                          ('gge', 'ERE and grammar'), ('exp_r09', 'ERE and grammar'),
                          ('cognition', 'Cognition'), ('experiment5', 'ALife'),
                          ('experiment6', 'Causal emergence'), ('validation', 'Validation')]:
        if name.startswith(prefix): return title
    if '/contracts/' in str(p): return 'Machine-profile companion contracts'
    return 'Architecture, substrate, locality and supporting contracts'


def main():
    documents = []
    for folder, dirs, files in os.walk(ROOT):
        dirs[:] = sorted(d for d in dirs if d not in EXCLUDE and not d.startswith('.')
                         and not (Path(folder)/d).is_symlink())
        for name in sorted(files):
            p = Path(folder)/name
            if p.suffix != '.md' or p.is_symlink() or p.stem == STEM: continue
            raw = p.read_text(errors='replace')
            title = next((x.lstrip('# ').strip() for x in raw.splitlines() if x.startswith('# ')), name)
            documents.append(dict(path=str(p.relative_to(ROOT)), title=title,
                                  sha256=digest(p), group=group(p)))
    roots = ['output', 'runs', 'experiments/lk/output', 'experiments/lk/runs',
             'crates/hg-train/runs_stage2', 'experiments/lk/crates/hg-train/runs_stage2']
    records, packages = [], []
    for name in roots:
        base = ROOT/name
        if not base.exists(): continue
        packages.extend(str(p.relative_to(ROOT)) for p in sorted(base.iterdir())
                        if p.is_dir() and not p.is_symlink())
        for folder, dirs, files in os.walk(base):
            depth = len(Path(folder).relative_to(base).parts)
            dirs[:] = sorted(d for d in dirs if depth < 3 and d not in EXCLUDE
                             and d not in {'snapshots', 'controller', 'native_spool', 'verification'}
                             and not (Path(folder)/d).is_symlink())
            for f in sorted(set(files) & METADATA):
                p = Path(folder)/f
                if p.is_symlink(): continue
                rec = dict(path=str(p.relative_to(ROOT)), bytes=p.stat().st_size)
                if rec['bytes'] > 4*1024**2:
                    rec['read_status'] = 'not_read_over_4MiB'
                else:
                    rec['sha256'] = digest(p)
                    try:
                        v = json.loads(p.read_text())
                        rec['read_status'] = 'parsed'
                        rec['fields'] = {k: v[k] for k in sorted(FIELDS & v.keys())
                                         if isinstance(v[k], (str, bool, int, float, type(None)))} if isinstance(v, dict) else {}
                    except (ValueError, UnicodeError) as e:
                        rec['read_status'] = 'parse_error'
                        rec['error'] = str(e)
                records.append(rec)
    payload = dict(profile='hypersim-project-documentation-inventory-v1', date='2026-10-13',
                   scope='First-party Markdown excluding build/vendor/evidence trees; named metadata to depth 3 below six explicit evidence roots. No payload replay or full historical seal verification.',
                   evidence_roots=roots, documentation=documents, packages=packages,
                   metadata=records, native_execution=False)
    (ROOT/'docs'/f'{STEM}.json').write_text(json.dumps(payload, indent=2, sort_keys=True)+'\n')
    lines = ['# Project documentation and evidence catalogue — 2026-10-13', '',
             '[Read the experimental report](project_experimental_report_20261013.md).', '',
             f'This catalogue indexes **{len(documents)} first-party Markdown files**, '
             f'**{len(packages)} immediate evidence directories**, and **{len(records)} metadata files**.', '',
             'Scope: Markdown outside build, vendor and evidence trees; named lifecycle/audit metadata '
             'at directory depths 0–3 below `output`, `runs`, isolated LK equivalents and the two '
             '`crates/hg-train/runs_stage2` roots. Nested evidence payloads and external historical '
             'LK archives are not recursively inventoried. No simulation, replay, or historical '
             'audit was launched. Counts describe files, not independent experiments.', '',
             f'[Machine inventory and SHA-256 identities]({STEM}.json). '
             '[Generator](../scripts/build_project_closeout_inventory.py). '
             'Raw statuses are preserved: a complete lifecycle is not scientific acceptance. '
             'A missing status means no recognized direct metadata field, not a negative result.', '']
    grouped = defaultdict(list)
    for d in documents: grouped[d['group']].append(d)
    for label, ds in sorted(grouped.items()):
        lines += [f'## {label}', '']
        for d in sorted(ds, key=lambda d:d['path']):
            title = d['title'].replace('[', '(').replace(']', ')')
            lines.append(f"- [{title}](../{d['path']}) — `{d['path']}`")
        lines.append('')
    lines += ['## Immediate evidence packages', '',
              'Direct lifecycle fields are listed when present; all inspected metadata, '
              'including nested records and audit fields, is in the JSON inventory.', '',
              '| Package | Direct lifecycle records | Inspected metadata files in subtree |',
              '|---|---|---:|']
    for pkg in packages:
        recs = [r for r in records if r['path'].startswith(pkg+'/')]
        direct = [r for r in recs if str(Path(r['path']).parent)==pkg
                  and Path(r['path']).name in {'manifest.json','campaign_manifest.json','status.json','run_status.json'}]
        statuses = '; '.join(f"{Path(r['path']).name}: " + str(r.get('fields',{}).get('status', r.get('fields',{}).get('state','no top-level status/state'))).replace('|','/') for r in direct)
        lines.append(f'| [{pkg}](../{pkg}) | {statuses or "No direct lifecycle record found"} | {len(recs)} |')
    lines += ['', '## Documentation snapshot limits', '',
              'Historical documents retain superseded recommendations and date-specific counts. '
              'Use the experimental report for current programme closure. Duplicate documents across '
              'root and isolated LK are provenance copies, not independent findings. Historical LK '
              'runs were not all copied with its source; see the [integration record](lk_workspace_integration.md).', '']
    (ROOT/'docs'/f'{STEM}.md').write_text('\n'.join(lines))
    print(json.dumps(dict(documents=len(documents), packages=len(packages), metadata=len(records),
                          read_status=dict(Counter(r['read_status'] for r in records))), indent=2))


if __name__ == '__main__':
    main()
