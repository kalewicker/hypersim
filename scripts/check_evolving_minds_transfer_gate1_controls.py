#!/usr/bin/env python3
"""Read-only complete causal-history comparison for the pinned TA-1 development result."""
import argparse
import hashlib
import json
from pathlib import Path

NATIVE='f23ef0ab8142399006e280f52332104a776b2842948602847f3e563650741661'
AUDIT='9b1e8e020b3a1b69138d8fe1f7e0d702c73084549c21dacfeaacf3182e7c86cb'

def require(ok,reason):
    if not ok:raise ValueError(reason)

def sealed(root,pin):
    require(root.is_dir() and not any(p.is_symlink() for p in root.rglob('*')),'input path')
    require(hashlib.sha256((root/'output_sha256.json').read_bytes()).hexdigest()==pin,'pinned input seal')
    seal=json.loads((root/'output_sha256.json').read_text())
    files={str(p.relative_to(root)):hashlib.sha256(p.read_bytes()).hexdigest()
           for p in root.rglob('*') if p.is_file() and p!=root/'output_sha256.json'}
    require(files==seal['files_sha256'],'complete payload inventory')

def check(native,audit):
    native,audit=Path(native),Path(audit);sealed(native,NATIVE);sealed(audit,AUDIT)
    status=json.loads((audit/'run_status.json').read_text())
    require(status['audit_pass'] and status['native_advances']==0,'full independent audit passed')
    comparisons=[]
    for pattern in ['Alternating','Blocks4']:
        for target,reference in [('sham','useful'),('restoration','useful'),('reversion','ancestor')]:
            a=native/f'world17_{pattern}_Informative_{target}'
            b=native/f'world17_{pattern}_Informative_{reference}'
            for name in ['schedule.json','rules.toml','initial.json','initial_lineage.json','ticks.jsonl']:
                require((a/name).read_bytes()==(b/name).read_bytes(),'complete causal native history '+pattern+':'+target+':'+name)
            for tick in range(0,65,8):
                name=f'state_{tick:04}.json'
                x=json.loads((a/name).read_text());y=json.loads((b/name).read_text())
                # Each already-validated archive has its own commit-object identity.
                # Keep every clock, log hash, graph, allocator and engine state field.
                x['native']['cursor'].pop('previous_commit_sha256')
                y['native']['cursor'].pop('previous_commit_sha256')
                require(x==y,'complete causal physical state '+pattern+':'+target+':'+str(tick))
            comparisons.append(dict(pattern=pattern,target=target,reference=reference,
                complete_native_history_byte_equal=True,physical_state_attestations_equal=9))
    return dict(native_seal_sha256=NATIVE,audit_seal_sha256=AUDIT,comparisons=comparisons,
        causal_controls_pass=True,excluded_archive_identity_field='native.cursor.previous_commit_sha256',
        independent_hash_chains_verified=True,native_advances=0,
        checker_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest())

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('native');p.add_argument('audit')
    a=p.parse_args();print(json.dumps(check(a.native,a.audit),indent=2))
