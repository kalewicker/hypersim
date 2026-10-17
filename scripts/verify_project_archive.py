"""Verify one pinned archive, optionally restore into a new staging directory."""
import gzip
import json
import os
from pathlib import Path
import subprocess
import sys
from archive_project_evidence import ROOT, digest, verify_restore


def main():
    if len(sys.argv) not in [2,3]:
        raise SystemExit('usage: verify_project_archive.py ARCHIVE [NEW_RESTORE_DIRECTORY]')
    archive=Path(sys.argv[1]).resolve()
    saved=json.loads((ROOT/'docs/project_archive_verification_20261014.json').read_text())
    matches=[v for v in saved['archives'] if v['name']==archive.name]
    if len(matches)!=1: raise ValueError('archive name is not in pinned local receipt')
    wanted=matches[0];index=archive.parent/wanted['inventory']
    if digest(archive)!=wanted['sha256'] or digest(index)!=wanted['inventory_sha256']:
        raise ValueError('archive/inventory SHA-256 mismatch')
    subprocess.run(['zstd','-q','-t',str(archive)],check=True)
    print('Pinned archive/inventory hashes and compressed stream pass.',flush=True)
    if len(sys.argv)==3:
        destination=Path(sys.argv[2]).resolve()
        if destination.exists(): raise ValueError('restore directory must be new')
        destination.mkdir(parents=True)
        env=dict(os.environ,COPYFILE_DISABLE='1')
        z=subprocess.Popen(['zstd','-q','-d','-c',str(archive)],stdout=subprocess.PIPE)
        try:
            t=subprocess.run(['tar','--no-xattrs','-xpf','-','-C',str(destination)],stdin=z.stdout,env=env)
            z.stdout.close();zr=z.wait()
            if t.returncode or zr: raise ValueError('restore failure; staging retained')
        finally:
            if z.poll() is None: z.terminate();z.wait()
        expected=json.loads(gzip.decompress(index.read_bytes()))
        count=verify_restore(destination,expected['roots'],expected['entries'])
        print(f'Complete restore verified: {count} entries at {destination}',flush=True)


if __name__=='__main__': main()
