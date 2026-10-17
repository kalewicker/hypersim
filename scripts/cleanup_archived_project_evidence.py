"""Remove admitted local evidence only after rechecking fully verified archives.

Never changes the external original LK workspace or Git history. Receipt and
historical indexes remain in docs. Requires explicit --remove-local-copies.
"""
import gzip
import json
from pathlib import Path
import shutil
import subprocess
import sys
import time
from archive_project_evidence import ROOT, digest, note, verify_restore

EVIDENCE = {'output', 'runs', 'crates/hg-train/runs_stage2',
            'crates/hg-selfmod/runs', 'experiments/lk/output'}
CACHES = ['target', 'experiments/lk/target', 'experiments/cumulative/target',
          'bins/hypersim-cli/target']


def size():
    return int(subprocess.check_output(['du','-sk',str(ROOT)],text=True).split()[0])*1024


def save(path,value):
    path.write_text(json.dumps(value,indent=2,sort_keys=True)+'\n')


def main():
    if len(sys.argv)!=3 or sys.argv[1]!='--remove-local-copies':
        raise SystemExit('usage: --remove-local-copies VERIFIED_ARCHIVE_DIRECTORY')
    dest=Path(sys.argv[2]).resolve()
    manifest=json.loads((dest/'archive_manifest.json').read_text())
    local=json.loads((ROOT/'docs/project_archive_verification_20261014.json').read_text())
    if manifest!=local or not manifest['all_full_restores_verified']:
        raise ValueError('complete matching archive/restore receipts required')
    if dest.is_relative_to(ROOT): raise ValueError('archives must be outside cleanup checkout')
    receipt=ROOT/'docs/project_cleanup_receipt_20261014.json'
    if receipt.exists(): raise ValueError('cleanup receipt exists; no automatic repeat')
    before=size(); admitted=set(); started=time.monotonic()
    for item in manifest['archives']:
        if not item['full_restore_verified'] or not item['source_unchanged_after_restore']:
            raise ValueError('full restore confirmation missing')
        archive=dest/item['name']; index=dest/item['inventory']
        note('Checking archived bytes before deletion: '+item['name'])
        if digest(archive)!=item['sha256'] or digest(index)!=item['inventory_sha256']:
            raise ValueError('archive or index changed')
        if Path(item['source_base']).resolve()==ROOT:
            if not set(item['roots'])<=EVIDENCE: raise ValueError('unapproved local evidence root')
            expected=json.loads(gzip.decompress(index.read_bytes()))
            if expected['roots']!=item['roots'] or Path(expected['source_base']).resolve()!=ROOT:
                raise ValueError('archive input identity changed')
            note('Checking current originals before deletion: '+item['name'])
            verify_restore(ROOT,item['roots'],expected['entries'])
            admitted.update(item['roots'])
    if admitted!=EVIDENCE: raise ValueError('full local evidence closure not archived')
    for name in sorted(admitted|set(CACHES)):
        p=ROOT/name
        if p.is_symlink() or not p.is_dir() or not p.resolve().is_relative_to(ROOT):
            raise ValueError('cleanup path must be existing ordinary directory: '+name)
    value=dict(profile='hypersim-verified-evidence-cleanup-v1',date='2026-10-14',
               status='removing',archive_destination=str(dest),archive_manifest_sha256=digest(dest/'archive_manifest.json'),
               checkout_allocated_bytes_before=before,evidence_roots=sorted(admitted),cache_roots=CACHES,
               removed=[],native_simulation=False,external_original_lk_workspace_removed=False,
               archive_checks='All compressed archive/index hashes and every current local original compared to full restored inventory before any removal.')
    save(receipt,value)
    try:
        for name in sorted(admitted)+CACHES:
            note('Removing verified/generated local directory: '+name)
            shutil.rmtree(ROOT/name);value['removed'].append(name);save(receipt,value)
        after=size()
        value.update(status='complete',checkout_allocated_bytes_after=after,
                     reclaimed_allocated_bytes=before-after,elapsed_seconds=time.monotonic()-started,
                     all_admitted_roots_absent=all(not (ROOT/n).exists() for n in admitted|set(CACHES)))
        save(receipt,value)
    except Exception as ex:
        value.update(status='failed',error=str(ex));save(receipt,value);raise
    note(json.dumps(value,indent=2))


if __name__=='__main__': main()
