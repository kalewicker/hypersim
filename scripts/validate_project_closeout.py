"""Validate closeout links/inventory and preserved campaign bytes, without execution."""
import hashlib
import json
from pathlib import Path
import re
import tomllib
from urllib.parse import unquote

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT/'docs/project_documentation_validation_20261013.json'
EXPECTED_SEAL = '7d49f529fa597678086fa23c354dbf9897d90dc285fb75158656993406f17f52'


def sha(p):
    h = hashlib.sha256()
    with p.open('rb') as f:
        for chunk in iter(lambda: f.read(1024*1024), b''): h.update(chunk)
    return h.hexdigest()


def links(p):
    fenced = False
    for number, line in enumerate(p.read_text(errors='replace').splitlines(), 1):
        if line.lstrip().startswith('```'):
            fenced = not fenced
            continue
        if fenced: continue
        for match in re.finditer(r'!?\[[^\]\n]*\]\((<[^>\n]+>|[^)\n]+)\)', line):
            target = match.group(1)
            target = target[1:-1] if target.startswith('<') else target.split(' "', 1)[0]
            if re.match(r'^[a-zA-Z][\w+.-]*:', target) or target.startswith('#'): continue
            raw = unquote(target.split('#', 1)[0].split('?', 1)[0])
            if not raw: continue
            raw = re.sub(r':\d+$', '', raw)
            resolved = Path(raw) if raw.startswith('/') else p.parent/raw
            yield dict(source=str(p.relative_to(ROOT)), line=number, target=target,
                       exists=resolved.exists() or resolved.resolve()==OUT)


def main():
    inventory_path = ROOT/'docs/project_experimental_inventory_20261013.json'
    inventory = json.loads(inventory_path.read_text())
    errors = []
    report_text = (ROOT/'docs/project_experimental_report_20261013.md').read_text()
    headings = {re.sub(r'[^\w\- ]', '', line.lstrip('# ').strip().lower()).replace(' ', '-')
                for line in report_text.splitlines() if line.startswith('#')}
    anchors = re.findall(r'\]\(#([^)]*)\)', report_text)
    errors.extend('report anchor missing: '+a for a in anchors if a not in headings)
    members = tomllib.loads((ROOT/'Cargo.toml').read_text())['workspace']['members']
    if len(members)!=21 or 'contains 21 packages' not in report_text:
        errors.append('workspace package count mismatch')
    all_links = []
    for d in inventory['documentation']:
        p = ROOT/d['path']
        if sha(p) != d['sha256']: errors.append('document changed: '+d['path'])
        all_links.extend(links(p))
    for r in inventory['metadata']:
        if 'sha256' in r and sha(ROOT/r['path']) != r['sha256']:
            errors.append('metadata changed: '+r['path'])
    catalogue = ROOT/'docs/project_experimental_inventory_20261013.md'
    all_links.extend(links(catalogue))
    current = {'README.md', 'docs/README.md', 'docs/project_experimental_report_20261013.md',
               'docs/project_experimental_inventory_20261013.md', 'experiments/cumulative/README.md'}
    missing = [v for v in all_links if not v['exists']]
    current_missing = [v for v in missing if v['source'] in current]
    errors.extend('current link missing: '+str(v) for v in current_missing)

    campaign = ROOT/'output/cumulative_scientific_campaign_20261011'
    if sha(campaign/'output_sha256.json') != EXPECTED_SEAL: errors.append('campaign seal changed')
    sealed = json.loads((campaign/'output_sha256.json').read_text())
    actual = {str(p.relative_to(campaign)) for p in campaign.rglob('*') if p.is_file()}
    if actual != set(sealed)|{'output_sha256.json'}: errors.append('campaign inventory mismatch')
    for name, expected in sealed.items():
        p = campaign/name
        if p.is_symlink() or not p.is_file() or sha(p) != expected: errors.append('campaign payload changed: '+name)
    m = json.loads((campaign/'manifest.json').read_text())
    r = json.loads((campaign/'report.json').read_text())
    recheck = json.loads((ROOT/'output/cumulative_scientific_campaign_20261013_recheck.json').read_text())
    required = dict(status='complete', scientific_worlds=188, sampling_slots=1880,
                    assay_requests=111, native_rewrites=13178, independent_campaign_audit_pass=True)
    if any(m.get(k)!=v for k,v in required.items()): errors.append('campaign reporting mismatch')
    if r['decision']['status']!='valid_negative' or any(r['decision'][k]!=0 for k in ['endogenous_successes','external_successes']):
        errors.append('campaign outcome mismatch')
    if recheck['exit_code']!=0 or recheck['seal_sha256']!=EXPECTED_SEAL:
        errors.append('saved independent recheck mismatch')
    result = dict(profile='hypersim-project-documentation-validation-v1', date='2026-10-13',
                  passed=not errors, native_execution=False, historical_experiments_rerun=False,
                  new_independent_physics_audit=False,
                  checks={'document_hashes':len(inventory['documentation']),
                          'report_toc_anchors':len(anchors), 'root_workspace_members':len(members),
                          'metadata_hashes':sum('sha256' in x for x in inventory['metadata']),
                          'metadata_not_read':sum('sha256' not in x for x in inventory['metadata']),
                          'local_markdown_links':len(all_links), 'current_document_missing_links':current_missing,
                          'final_campaign_seal':EXPECTED_SEAL, 'final_campaign_payload_hashes':len(sealed),
                          'final_campaign_manifest':required, 'final_campaign_decision':r['decision']['status'],
                          'saved_independent_recheck_exit_code':recheck['exit_code']},
                  limits='Markdown file/path links excluding fenced code and external URI schemes; report table-of-contents anchors checked, but no historical anchor or external web validation. Legacy missing targets retained below, not repaired by inventing artifacts. Oversized metadata records are inventoried by path/size only.',
                  legacy_missing_local_links=[v for v in missing if v['source'] not in current],
                  errors=errors,
                  inventory_sha256=sha(inventory_path), catalogue_sha256=sha(catalogue),
                  report_sha256=sha(ROOT/'docs/project_experimental_report_20261013.md'))
    OUT.write_text(json.dumps(result, indent=2, sort_keys=True)+'\n')
    print(json.dumps(dict(passed=result['passed'],errors=errors,checks=result['checks'],
                          legacy_missing_local_links=len(result['legacy_missing_local_links'])),indent=2))
    if errors: raise SystemExit(1)


if __name__ == '__main__':
    main()
