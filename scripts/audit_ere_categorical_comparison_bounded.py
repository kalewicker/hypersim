#!/usr/bin/env python3
"""Storage-only supplemental auditor; no launch bridge or new scientific gate.

The v1 auditor and measured archives remain unchanged. Each full native trace is
checked in the first pass, then discarded. Later accesses reread the exact bytes
bound in that pass. No receipt projection, numerical change, or digest-only
substitution for full-emission equality is used.
"""
import argparse
import ast
from collections.abc import Mapping
import copy
import hashlib
import json
from pathlib import Path
import re
import time

import audit_ere_categorical_comparison as legacy
from audit_ere_categorical_comparison import (
    a, p, cat, ROOT, PREREG, PREREG_SEAL, CONFIG_SHA, PARENT, PARENT_SEAL,
    RUNTIME, RUNTIME_SEAL, exact, numeric, registration, estimate,
    validation, support,
)

LEGACY_SHA256 = 'f8e9222864c5b9ed0ef115282a4887d4c742b876af0d0d671ae211fe7c12914c'


class BoundedRows(Mapping):
    """Hash-bound lazy rows: O(N) tiny bindings, no retained receipt cache.

    Call read_unregistered, validate that exact object with the native checker,
    then assign it to this mapping. Assignment keeps only the SHA-256 digest.
    Every subsequent read hashes and parses the same byte buffer, rejecting
    changes since validation. Caller-local objects (at most a validation panel
    of four in the audit) are released normally; this class caches none.
    """
    def __init__(self, directory):
        self.directory = Path(directory)
        assert not self.directory.is_symlink(), 'symlink cell directory'
        self.bindings = {}
        self._pending = None
        self.reads = 0
        self.bytes_read = 0

    @staticmethod
    def _parse(payload):
        # Same duplicate-key refusal as the legacy a.load, on the hashed bytes.
        def pairs(items):
            result = {}
            for key, value in items:
                assert key not in result, 'duplicate JSON key'
                result[key] = value
            return result
        return json.loads(payload, object_pairs_hook=pairs)

    def _read(self, key):
        assert isinstance(key, str) and re.fullmatch(r'[A-Za-z0-9_-]+', key), 'unsafe cell id'
        path = self.directory / (key + '.json')
        assert path.is_file() and not path.is_symlink(), 'cell missing or symlink'
        payload = path.read_bytes()
        self.reads += 1
        self.bytes_read += len(payload)
        digest = hashlib.sha256(payload).hexdigest()
        return payload, digest

    def read_unregistered(self, key):
        assert key not in self.bindings and self._pending is None, 'duplicate or unfinished validation'
        payload, digest = self._read(key)
        row = self._parse(payload)
        assert isinstance(row, dict), 'cell must be an object'
        self._pending = (key, digest, row)
        return row

    def __setitem__(self, key, row):
        assert self._pending is not None, 'row not read for validation'
        pending_key, digest, pending_row = self._pending
        assert key == pending_key and row is pending_row, 'validated object binding changed'
        self.bindings[key] = digest
        self._pending = None

    def __getitem__(self, key):
        expected = self.bindings[key]
        payload, digest = self._read(key)
        assert digest == expected, 'cell changed after native validation'
        return self._parse(payload)

    def __iter__(self):
        return iter(self.bindings)

    def __len__(self):
        return len(self.bindings)


def verify_storage_only():
    """Fail closed if the pinned v1 or any non-storage audit statement drifts."""
    source = Path(legacy.__file__).read_bytes()
    assert hashlib.sha256(source).hexdigest() == LEGACY_SHA256, 'v1 checker changed'
    original = next(n for n in ast.parse(source).body if isinstance(n, ast.FunctionDef) and n.name == 'audit')
    current = next(n for n in ast.parse(Path(__file__).read_text()).body if isinstance(n, ast.FunctionDef) and n.name == '_audit')
    current.name = 'audit'
    allowed = {
        "BoundedRows(out / 'cells')": "{}",
        "rows.read_unregistered(q['id'])": "a.load(out / 'cells' / (q['id'] + '.json'))",
    }
    replacements = {ast.dump(ast.parse(k, mode='eval').body): ast.parse(v, mode='eval').body for k, v in allowed.items()}

    class NormalizeStorage(ast.NodeTransformer):
        def __init__(self):
            self.counts = {k: 0 for k in replacements}

        def visit_Call(self, node):
            signature = ast.dump(node)
            if signature in replacements:
                self.counts[signature] += 1
                return copy.deepcopy(replacements[signature])
            return self.generic_visit(node)

    normalizer = NormalizeStorage()
    current = normalizer.visit(current)
    assert all(n == 1 for n in normalizer.counts.values()), 'storage substitutions not unique'
    assert ast.dump(current) == ast.dump(original), 'non-storage audit semantics changed'
    return dict(legacy_sha256=LEGACY_SHA256, audit_ast_equal_except_two_storage_expressions=True)


def _audit(out, verify_seal=True):
    start=time.monotonic();out=Path(out)
    if verify_seal:a.verify(out)
    c=a.load(out/'config.json');p.validate(c);assert a.digest(out/'config.json')==CONFIG_SHA
    a.verify(ROOT/PREREG,expected=PREREG_SEAL,count=9)
    for pin in c['lineage'].values():a.verify(ROOT/pin['path'],expected=pin['root_seal'],count=pin['sealed_entries'])
    a.verify(PARENT,expected=PARENT_SEAL,count=1690);a.verify(RUNTIME,expected=RUNTIME_SEAL,count=460)
    req=a.load(out/'request.json');assert set(req)=={'profile','mode','launch_authorized','full_training_authorized','setup_elapsed_seconds'}
    assert req['profile']==p.PROFILE and req['mode'] in ('development','scientific') and type(req['launch_authorized']) is bool and req['full_training_authorized'] is False
    mode=req['mode'];l=c[mode];plan=p.schedule(c,mode);exact(a.load(out/'schedule.json'),plan);exact(a.load(out/'contexts.json'),p.contexts())
    prepared=a.load(out/'prepared_sha256.json');immutable=set(a.load(out/'snapshot.json')['sources'])
    expected={f'source_snapshots/{n}' for n in immutable}|{'snapshot.json','source_registration.json','build_receipt.json','executed_binary','config.json','schedule.json','contexts.json','request.json','initial_manifest.json','reference_catalogue.json','development_gate.json'}
    assert set(prepared)==expected
    for n,h in prepared.items():assert a.safe(n) and a.digest(out/n)==h,'prepared immutable input changed'
    snap=a.load(out/'snapshot.json');assert snap['frozen_before_measurement'] is True and snap['build_features']==['ere-complete-signal'] and snap['default_features'] is False
    assert Path(snap['repository']).resolve()==ROOT.resolve()
    receipt=registration(out/'source_snapshots',c);exact(a.load(out/'source_registration.json'),receipt)
    assert set(snap['sources'])==set(receipt['source_names']) and a.files(out/'source_snapshots')==set(snap['sources'])
    for n,h in snap['sources'].items():assert a.digest(out/'source_snapshots'/n)==h
    assert a.digest(out/'executed_binary')==snap['binary_sha256']
    gate=a.load(out/'development_gate.json')
    if mode=='scientific' and req['launch_authorized']:
        assert isinstance(gate,dict) and set(gate)=={'path','root_seal'}
        dev=Path(gate['path']);assert dev.resolve()!=out.resolve();a.verify(dev,expected=gate['root_seal'])
        prior=audit(dev)
        assert prior['classification']=='bounded_development' and prior['campaign_complete'] and prior['unknown_partial_work_cells']==0
        assert a.load(dev/'snapshot.json')['sources']==snap['sources'] and len(a.load(dev/'audit.json')['detached_tamper_rejections'])==6
    else:assert gate is None
    build=a.load(out/'build_receipt.json');assert build['command']==['cargo','build','--offline','--release','-p','hg-train','--no-default-features','--features','ere-complete-signal','--bin','ere-categorical-comparison']
    assert build['source_hashes_before']==build['source_hashes_after']==snap['sources'] and build['binary_sha256']==snap['binary_sha256'] and build['returncode']==0
    m=a.load(out/'manifest.json');initial=a.load(out/'initial_manifest.json')
    assert m['profile']==initial['profile']==p.PROFILE and m['mode']==initial['mode']==mode
    assert m['scheduled_cells']==len(plan) and m['optimizer_generations']==m['training_updates']==0 and m['training_admitted'] is False and m['full_training_authorized'] is False
    assert m['inflight_native_work_unknown'] is False,'in-flight unknown work blocks final accounting'
    assert type(m['recorded_cells']) is int and 0<=m['recorded_cells']<=len(plan)
    assert all(type(m[k]) is int and m[k]>=0 for k in ('attempted_steps','advancing_steps','unknown_partial_work_cells','grammar_compilations','direction_starts','direction_estimates','receipt_bytes','conditional_leaf_checks','optimizer_generations','training_updates'))
    assert a.files(out/'cells')=={q['id']+'.json' for q in plan}
    catalogue=a.load(out/'reference_catalogue.json');assert a.digest(out/'reference_catalogue.json')==a.CATALOGUE
    if (out/'catalogue.json').exists():assert a.digest(out/'catalogue.json')==a.CATALOGUE and m['grammar_compilations']==1
    native,h=cat.d.c.helpers(PARENT);rows=BoundedRows(out/'cells');attempts=advances=unknown=valid=null=0;prefix=False
    for q in plan:
        row=rows.read_unregistered(q['id']);cat.d.b.exact_schedule([row['schedule']],[q])
        ns,adv,score=cat.d.b.cell(row,q,catalogue,native,h);attempts+=ns;advances+=adv
        if row['status']=='NotRun':prefix=True
        else:
            assert not prefix,'fixed native sequence gap';unknown+=row['status']=='Incomplete';valid+=row['status']=='Complete';null+=row['status']=='Null'
        if q['stage']=='validation' and row['status']!='NotRun':cat.bits(row['parameters'],p.contexts()[q['context']])
        rows[q['id']]=row
        assert time.monotonic()-start<900,'auditor wall ceiling'
    assert (m['recorded_cells'],m['attempted_steps'],m['advancing_steps'],m['unknown_partial_work_cells'])==(valid+null+unknown,attempts,advances,unknown)
    assert 0<=advances<=attempts<=l['maximum_native_attempts']
    checked_emissions=sum(q['stage']=='estimation' or q.get('physics_index')==0 for q in plan[:m['recorded_cells']])
    assert type(m['conditional_leaf_checks']) is int and checked_emissions<=m['conditional_leaf_checks']<=checked_emissions+1
    n=l['blocks'];align=[[[None]*2 for _ in range(2)] for _ in range(n)];effects=[[[[None]*2 for _ in range(2)] for _ in range(2)] for _ in range(n)];vs=copy.deepcopy(effects)
    estimation=[q for q in plan if q['stage']=='estimation'];efiles=set();dfiles=set();vfiles=set();directions={};starts=updates=0;closed=False;size=l['categorical_emissions_per_estimate'];offset=0
    for r in range(n):
        for ctx in range(2):
            forwards=[];reverses=[]
            for method in ('es','categorical'):
                for e in range(2):
                    qs=estimation[offset:offset+size];offset+=size;name=f'r{r}-c{ctx}-{method}-e{e}.json';path=out/'estimates'/name
                    if path.exists():
                        assert not closed,'estimate follows unfinished estimate';efiles.add(name);starts+=1
                        rec=a.load(path);step=estimate(rec,qs,p.contexts()[ctx],rows,catalogue,native,h,l['es_emissions_per_candidate'])
                        if step is None:closed=True
                        else:updates+=1;forwards.append(step['forward']);reverses.append(step['reverse'])
                    else:assert all(rows[q['id']]['status']=='NotRun' for q in qs);closed=True
            name=f'r{r}-c{ctx}.json';path=out/'directions'/name
            if path.exists():
                assert len(forwards)==4;dfiles.add(name);d=a.load(path);vectors=[p.contexts()[ctx]]+forwards+reverses
                for x,y in zip(d['vectors'],vectors):cat.bits(x,y)
                assert len(d['vectors'])==9 and d['block']==r and d['context']==ctx
                al=[cat.d.alignment(vectors[0],vectors[a],vectors[b]) for a,b in ((1,2),(3,4))]
                numeric(d['alignment'],al);numeric(d['full_support'],support(catalogue,vectors[0],vectors))
                align[r][ctx]=[x['score'] for x in al];directions[(r,ctx)]=vectors
    assert a.files(out/'estimates')==efiles and a.files(out/'directions')==dfiles and (starts,updates)==(m['direction_starts'],m['direction_estimates'])
    for r in range(n):
        for ctx in range(2):
            qs=[q for q in plan if q['stage']=='validation' and (q['block'],q['context'])==(r,ctx)]
            if any(rows[q['id']]['status']!='NotRun' for q in qs):assert updates==l['direction_estimates'] and len(directions)==n*2,'validation before all proposals'
            name=f'r{r}-c{ctx}.json';path=out/'validation'/name
            if path.exists():
                vfiles.add(name);u,v=validation(a.load(path),r,ctx,qs,rows,catalogue,directions[(r,ctx)],l['validation_emissions_per_block_context'])
                for method in range(2):
                    for e in range(2):effects[r][ctx][method][e]=u[1+2*method+e];vs[r][ctx][method][e]=v[1+2*method+e]
    assert a.files(out/'validation')==vfiles
    complete=m['status']=='complete'
    if complete:
        assert req['launch_authorized'] and m['recorded_cells']==len(plan) and unknown==0 and updates==l['direction_estimates'] and len(vfiles)==n*2
        assert m['live_sources_rechecked'] is True and 0<=m['elapsed_seconds']<l['wall_seconds']
        assert m['conditional_leaf_checks']==l['estimation_evaluations']+l['validation_evaluations']//4
    else:assert m['status'] in ('zero_work_preflight','incomplete','blocked_before_evaluation')
    if not req['launch_authorized']:
        assert m['status']=='zero_work_preflight' and m==initial and attempts==updates==starts==m['grammar_compilations']==0
    numeric(a.load(out/'summary.json'),p.statistics(c,mode,align,effects,vs,complete))
    assert sum((out/n).stat().st_size for n in a.files(out))<c['resources']['output_cap_bytes']
    assert time.monotonic()-start<900,'auditor wall ceiling'
    return dict(audit_pass=True,classification='zero_work_preflight' if not req['launch_authorized'] else 'bounded_development' if mode=='development' else 'fixed_parameter_categorical_comparison',
        campaign_complete=complete,scientific_campaign_complete=complete and mode=='scientific',support_categorical_direction=a.load(out/'summary.json')['support_categorical_direction'],
        recorded_cells=m['recorded_cells'],valid_evaluations=valid,null_evaluations=null,unknown_partial_work_cells=unknown,
        attempted_steps=attempts,advancing_steps=advances,direction_estimates=updates,grammar_compilations=m['grammar_compilations'],
        training_updates=0,training_admitted=False,independent_physics_replay=False,independent_rng_replay=False,frozen_sources=len(snap['sources']))

def audit(out, verify_seal=True):
    verify_storage_only()
    return _audit(out, verify_seal)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('archive', type=Path)
    args = parser.parse_args()
    verify_storage_only()
    print(json.dumps(audit(args.archive), indent=2))


if __name__ == '__main__':
    main()
