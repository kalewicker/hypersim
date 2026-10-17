"""Known-root, relocatable zero-native verification of compact storage evidence."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import time
from storage_projection import project


def digest(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()


def recheck(package, expected_root):
    started = time.monotonic()
    seal_path = package / 'output_sha256.json'
    if digest(seal_path) != expected_root:
        raise ValueError('known storage archive root')
    seal = json.loads(seal_path.read_text())
    actual = {str(p.relative_to(package)) for p in package.rglob('*') if p.is_file() and p != seal_path}
    if actual != set(seal) or any((package / rel).is_symlink() or digest(package / rel) != h for rel, h in seal.items()):
        raise ValueError('complete immutable storage archive')
    contract = json.loads((package / 'contract.json').read_text())
    manifest = json.loads((package / 'manifest.json').read_text())
    if manifest['status'] != 'complete' or manifest['new_native_rewrites'] != 0:
        raise ValueError('completed zero-native storage boundary')
    for flag in ['readiness_gate_complete', 'scientific_acceptance', 'evolution_demonstrated', 'campaign_launch_authorized']:
        if manifest[flag] is not False or contract[flag] is not False:
            raise ValueError('scientific admission changed')
    pins = json.loads((package / 'source_sha256.json').read_text())
    if any(digest(package / 'source_snapshots' / rel) != h for rel, h in pins.items()):
        raise ValueError('frozen storage source closure')
    scripts = package / 'source_snapshots/experiments/cumulative/scripts'
    packages = json.loads((package / 'compact_packages.json').read_text())
    receipts = []
    for name, info in sorted(packages.items()):
        args = [sys.executable, '-B', str(scripts / 'audit_compact_evidence.py'),
                str(package / 'stores' / name), info['compact_index_sha256'],
                info['source_seal_sha256'], info['audit_module']]
        r = subprocess.run(args, capture_output=True, text=True, timeout=120)
        if r.returncode:
            raise ValueError(r.stderr)
        receipts.append({'name': name, 'result': json.loads(r.stdout)})
    r = subprocess.run([sys.executable, '-B', str(scripts / 'test_compact_evidence.py')],
                       capture_output=True, text=True, timeout=30)
    if r.returncode:
        raise ValueError(r.stderr)
    indexes = {n: json.loads((package / 'stores' / n / 'index.json').read_text()) for n in packages}
    audits = json.loads((package / 'byte_and_semantic_audits.json').read_text())
    packs = json.loads((package / 'pack_metrics.json').read_text())
    projection = json.loads((package / 'projection.json').read_text())
    if project(contract, indexes, audits, packs) != projection:
        raise ValueError('reproducible saved projection')
    # Independently sum each capacity reservation; no imported formula for this check.
    p = contract['projection']
    if projection['world_record_slots'] != 188 * 36 or projection['assay_record_slots'] != 78960 * 2 + 617 * 2:
        raise ValueError('unchanged full campaign record denominator')
    parts = ['rounded_record_projection_bytes', 'index_reserve_bytes', 'filesystem_metadata_reserve_bytes',
             'other_evidence_reserve_bytes', 'raw_scratch_reserve_bytes', 'retained_emergency_raw_reserve_bytes']
    if sum(projection[k] for k in parts) != projection['observed_peak_storage_projection_bytes']:
        raise ValueError('independent peak capacity sum')
    if projection['campaign_cap_bytes'] != p['campaign_output_cap_bytes'] or p['campaign_output_cap_bytes'] != 8589934592:
        raise ValueError('unchanged 8 GiB comparator')
    return {'passed': True, 'archive_seal_sha256': expected_root, 'corpus_checks': receipts,
            'corruption_checks': json.loads(r.stdout), 'native_rewrites': 0,
            'full_journal_expansion_on_disk': False, 'elapsed_seconds': time.monotonic() - started}


if __name__ == '__main__':
    print(json.dumps(recheck(Path(sys.argv[1]).resolve(), sys.argv[2]), indent=2, sort_keys=True))
