"""Create and fully restore-verify project archives. Never deletes original evidence.

Requires a new destination directory. The evidence inventory lives with each
archive; a compact receipt stays in docs. Restores use temporary local storage.
"""
import gzip
import hashlib
import json
import os
from pathlib import Path
import shutil
import stat
import subprocess
import sys
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]
PARTS = [
    ('main-evidence', ROOT, ['output', 'runs', 'crates/hg-train/runs_stage2', 'crates/hg-selfmod/runs']),
    ('lk-evidence', ROOT, ['experiments/lk/output']),
    ('lk-historical-runs', ROOT.parent/'hypersim-lk', ['runs', 'output']),
]


def note(message):
    print(message, flush=True)


def digest(path):
    h = hashlib.sha256()
    with path.open('rb') as f:
        for b in iter(lambda: f.read(1024*1024), b''): h.update(b)
    return h.hexdigest()


def inventory(base, roots):
    result = {}; hardlinks = {}; last = time.monotonic()
    for name in roots:
        root = base/name
        if not root.is_dir() or root.is_symlink(): raise ValueError('missing/linked root '+str(root))
        for folder, dirs, files in os.walk(root, followlinks=False):
            dirs.sort(); files.sort()
            for path in [Path(folder)] + [Path(folder)/f for f in files] + [Path(folder)/d for d in dirs if (Path(folder)/d).is_symlink()]:
                key = str(path.relative_to(base)); s = path.lstat()
                item = dict(mode=stat.S_IMODE(s.st_mode))
                if stat.S_ISLNK(s.st_mode):
                    target = os.readlink(path)
                    # Reject archive links escaping the admitted source base.
                    path.resolve().relative_to(base.resolve())
                    item.update(kind='symlink', target=target)
                elif stat.S_ISDIR(s.st_mode): item['kind'] = 'directory'
                elif stat.S_ISREG(s.st_mode):
                    inode = (s.st_dev, s.st_ino)
                    first = hardlinks.get(inode)
                    if first:
                        item.update(kind='file', bytes=s.st_size, sha256=result[first]['sha256'], hardlink_to=first)
                    else:
                        item.update(kind='file', bytes=s.st_size, sha256=digest(path))
                        hardlinks[inode] = key
                    after = path.lstat()
                    if (after.st_size, after.st_mtime_ns, after.st_ino)!=(s.st_size,s.st_mtime_ns,s.st_ino):
                        raise ValueError('source changed while hashing: '+key)
                else: raise ValueError('unsupported entry '+key)
                result[key] = item
                if time.monotonic()-last>15:
                    note(f'Inventory: {len(result):,} entries verified in {base}'); last=time.monotonic()
    return result


def synced_json(path, value):
    with path.open('x') as f:
        json.dump(value,f,indent=2,sort_keys=True);f.write('\n');f.flush();os.fsync(f.fileno())


def verify_restore(base, roots, expected):
    found=inventory(base,roots)
    if set(found)!=set(expected): raise ValueError('restored entry denominator differs')
    for name, wanted in expected.items():
        got=found[name]
        for field in ['kind','mode','bytes','sha256','target']:
            if got.get(field)!=wanted.get(field): raise ValueError('restore mismatch '+name+' '+field)
        if 'hardlink_to' in wanted:
            if os.stat(base/name).st_ino!=os.stat(base/wanted['hardlink_to']).st_ino:
                raise ValueError('hardlink not restored '+name)
    return len(found)


def archive_one(destination, name, base, roots):
    note('Hashing source for '+name)
    items=inventory(base,roots)
    index=dict(profile='hypersim-full-file-archive-inventory-v1',source_base=str(base),roots=roots,entries=items)
    idx=destination/(name+'.inventory.json.gz')
    with idx.open('xb') as f:
        with gzip.GzipFile(fileobj=f,mode='wb',mtime=0,filename='') as z:
            z.write((json.dumps(index,sort_keys=True,separators=(',',':'))+'\n').encode())
        f.flush();os.fsync(f.fileno())
    partial=destination/(name+'.tar.zst.partial'); final=destination/(name+'.tar.zst')
    env=dict(os.environ,COPYFILE_DISABLE='1')
    note('Writing '+str(final))
    with partial.open('xb') as sink:
        tar=subprocess.Popen(['tar','--no-xattrs','-cf','-','-C',str(base),*roots],stdout=subprocess.PIPE,env=env)
        try:
            z=subprocess.run(['zstd','-T2','-3','-q','-c'],stdin=tar.stdout,stdout=sink)
            tar.stdout.close(); t=tar.wait()
            if t!=0 or z.returncode!=0: raise ValueError(f'archive pipeline failed tar={t} zstd={z.returncode}')
            sink.flush();os.fsync(sink.fileno())
        finally:
            if tar.poll() is None: tar.terminate();tar.wait()
    if partial.stat().st_size>=4294967296: raise ValueError('FAT32 file limit')
    if final.exists(): raise ValueError('archive path already exists')
    partial.rename(final)
    subprocess.run(['zstd','-q','-t',str(final)],check=True)
    archive_hash=digest(final)
    note('Restoring all entries from '+name)
    temporary=Path(tempfile.mkdtemp(prefix='hypersim-archive-restore-',dir='/private/tmp'))
    try:
        z=subprocess.Popen(['zstd','-q','-d','-c',str(final)],stdout=subprocess.PIPE)
        try:
            t=subprocess.run(['tar','--no-xattrs','-xpf','-','-C',str(temporary)],stdin=z.stdout,env=env)
            z.stdout.close(); zr=z.wait()
            if t.returncode or zr: raise ValueError('restore pipeline failure')
        finally:
            if z.poll() is None: z.terminate();z.wait()
        count=verify_restore(temporary,roots,items)
        if digest(final)!=archive_hash: raise ValueError('archive changed during restore')
    finally:
        shutil.rmtree(temporary)
    note('Rechecking original evidence after archive/restore: '+name)
    verify_restore(base,roots,items)
    result=dict(name=name+'.tar.zst',source_base=str(base),roots=roots,compressed_bytes=final.stat().st_size,
                sha256=archive_hash,inventory=idx.name,inventory_sha256=digest(idx),entries=count,
                file_entries=sum(v['kind']=='file' for v in items.values()),
                logical_file_bytes=sum(v.get('bytes',0) for v in items.values()),
                full_restore_verified=True,source_unchanged_after_restore=True,
                file_bytes_modes_symlinks_and_hardlinks_verified=True)
    synced_json(destination/(name+'.verification.json'),result)
    note(f'Verified {name}: {count:,} entries; {result["compressed_bytes"]:,} compressed bytes')
    return result


def main():
    if len(sys.argv)!=2: raise SystemExit('usage: archive_project_evidence.py NEW_FLASH_DRIVE_DIRECTORY')
    destination=Path(sys.argv[1]).resolve()
    if destination.exists(): raise ValueError('destination must be new')
    if shutil.disk_usage(destination.parent).free<4_500_000_000: raise ValueError('insufficient archive space')
    if shutil.disk_usage('/private/tmp').free<36*1024**3: raise ValueError('insufficient full-restore workspace')
    destination.mkdir()
    results=[];started=time.monotonic()
    for name,base,roots in PARTS: results.append(archive_one(destination,name,base,roots))
    shutil.copyfile(ROOT/'docs/project_experimental_report_20261013.md',destination/'project_experimental_report_20261013.md')
    value=dict(profile='hypersim-verified-evidence-archives-v1',date='2026-10-14',destination=str(destination),
               archives=results,total_compressed_bytes=sum(r['compressed_bytes'] for r in results),
               elapsed_seconds=time.monotonic()-started,all_full_restores_verified=True,
               original_evidence_deleted=False,native_simulation=False,
               format='tar with zstd 1.5.7 -T2 -3; macOS xattrs omitted',
               restore='zstd -d -c ARCHIVE | tar -xpf - -C ORIGINAL_SOURCE_BASE (empty destination; then verify inventory)')
    synced_json(destination/'archive_manifest.json',value)
    synced_json(ROOT/'docs/project_archive_verification_20261014.json',value)
    (destination/'README.txt').write_text('Hypersim evidence archives — 2026-10-14\n\n'
        'All three archives were fully restored and every file hash, file mode, symlink and original hardlink group verified.\n'
        'See archive_manifest.json and the per-archive inventories/verification receipts.\n'
        'main-evidence and lk-evidence restore relative to /Users/kalewicker/Documents/hypersim.\n'
        'lk-historical-runs restores relative to /Users/kalewicker/Documents/hypersim-lk.\n'
        'Restore into an empty staging directory first; do not overwrite newer sources or evidence.\n'
        'Code/configuration/contracts remain in the working repository; measured frozen sources and binaries are inside the evidence archives.\n')
    subprocess.run(['sync'],check=True)
    note(json.dumps({k:v for k,v in value.items() if k!='archives'},indent=2))


if __name__=='__main__': main()
