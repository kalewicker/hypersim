#!/usr/bin/env python3
"""Bounded targeted primary-draw profiling and full read-only benchmarking.

Outputs go only to a fresh diagnostic directory, not the measured receipt.
Use the archive's frozen helper modules for both baseline and candidate.
"""
import argparse
import contextlib
import cProfile
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import pstats
import resource
import shutil
import sys
import time
import traceback


def digest(p):
    with p.open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


def seal_check(archive):
    seal = json.loads((archive/'output_sha256.json').read_text())
    actual = {str(p.relative_to(archive)) for p in archive.rglob('*') if p.is_file() and p != archive/'output_sha256.json'}
    assert actual == set(seal)
    assert not any(p.is_symlink() for p in archive.rglob('*'))
    for rel, sha in seal.items():
        assert digest(archive/rel) == sha, rel
    return digest(archive/'output_sha256.json')


def semantic_result(result):
    return {k:v for k,v in result.items() if k not in ['phase_seconds','peak_rss_bytes','child_replay_timing','optimization_stats']}


def run(args):
    if not __debug__:
        raise RuntimeError('profiling requires Python assertions enabled; -O/-OO are refused')
    archive, checker, out = args.archive.resolve(), args.checker.resolve(), args.output.resolve()
    assert not out.is_relative_to(archive) and not archive.is_relative_to(out), 'diagnostics must be outside receipt'
    assert 0 < args.seconds <= (300 if args.profile else 120)
    out.mkdir(parents=False, exist_ok=False)
    source = out/'source_snapshots'
    source.mkdir()
    helpers = archive/'source_snapshots/scripts'
    protocol=Path(__file__).resolve().parents[1]/'docs/ere_coherent_draw_optimization_v1.md'
    paths = [checker,Path(__file__).resolve(),protocol,*[helpers/name for name in ['verify_ere_complete_gates.py','verify_ere_complete_witness.py','verify_ere_complete_long_gates.py']]]
    pins = {}
    for path in paths:
        assert path.is_file() and not path.is_symlink()
        shutil.copyfile(path,source/path.name)
        pins[path.name] = digest(path)
    initial_seal = seal_check(archive)
    prereg = {'scope':'read-only diagnostic full replay; no Engine or admission writes','archive':str(archive),'archive_seal_sha256':initial_seal,'checker':str(checker),'sources':pins,'seconds_cap':args.seconds,'cprofile':args.profile,'profile_scope':'first full-audit primary-draw tally/generator including child replay','protocol_sha256':digest(protocol),'training_admitted':False,'gate_pass':False}
    (out/'manifest.json').write_text(json.dumps(prereg,indent=2,sort_keys=True)+'\n')
    # Both candidates use identical frozen helper implementations, not whichever
    # files happen to be on the live workspace's import path.
    sys.path.insert(0,str(source))
    spec = importlib.util.spec_from_file_location('measured_auditor',source/checker.name)
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    captured = {}
    original = module.audit
    def audit(*a,**kw):
        result = original(*a,**kw)
        captured.update(result)
        return result
    module.audit = audit
    sys.argv = [str(checker),str(archive),'--development','--stage','full','--verify-only','--seconds',str(args.seconds)]
    profiler = cProfile.Profile() if args.profile else None
    profile_calls = 0
    if profiler:
        original_tally = module.tally
        def tally(rows):
            nonlocal profile_calls
            if profile_calls:
                return original_tally(rows)
            profile_calls += 1
            profiler.enable()
            try:
                return original_tally(rows)
            finally:
                profiler.disable()
        module.tally = tally
    started, stdout, error = time.monotonic(), io.StringIO(), None
    try:
        with contextlib.redirect_stdout(stdout):
            module.main()
            assert semantic_result(captured)==semantic_result(json.loads((archive/'audit_full.json').read_text())), 'frozen scientific audit result differs'
            assert not profiler or profile_calls==1, 'targeted profile not executed exactly once'
    except Exception:
        error = traceback.format_exc()
    finally:
        if profiler:profiler.disable()
    elapsed = time.monotonic()-started
    if profiler:
        profiler.dump_stats(str(out/'profile.pstats'))
        stats = pstats.Stats(profiler)
        selected = []
        for (file,line,name),(primitive,total,self_time,cumulative,callers) in sorted(stats.stats.items(),key=lambda x:x[1][3],reverse=True)[:60]:
            selected.append({'file':file,'line':line,'function':name,'primitive_calls':primitive,'total_calls':total,'self_seconds':self_time,'cumulative_seconds':cumulative})
        (out/'hotspots.json').write_text(json.dumps(selected,indent=2,sort_keys=True)+'\n')
    # Recheck receipt and checker inputs after the entire diagnostic invocation.
    assert seal_check(archive) == initial_seal, 'receipt changed during replay'
    for path in paths:assert digest(path)==pins[path.name], 'source changed during replay'
    result = {'status':'failed' if error else 'complete','error':error,'wall_seconds':elapsed,'peak_rss_bytes':resource.getrusage(resource.RUSAGE_SELF).ru_maxrss*(1 if sys.platform=='darwin' else 1024),'profiled':args.profile,'profile_calls':profile_calls,'scientific_result_equal_frozen':error is None,'semantic_result':semantic_result(captured),'phase_seconds':captured.get('phase_seconds',{}),'stdout':stdout.getvalue(),'archive_unchanged':True,'training_admitted':False,'gate_pass':False}
    if hasattr(module,'optimization_stats'):result['optimization_stats']=module.optimization_stats()
    result['child_replay_timing']=captured.get('child_replay_timing',[])
    (out/'result.json').write_text(json.dumps(result,indent=2,sort_keys=True)+'\n')
    (out/'output_sha256.json').write_text(json.dumps({str(p.relative_to(out)):digest(p) for p in sorted(out.rglob('*')) if p.is_file()},indent=2,sort_keys=True)+'\n')
    print(json.dumps({k:result[k] for k in ['status','wall_seconds','peak_rss_bytes','archive_unchanged','profiled']}),flush=True)
    if error:raise RuntimeError(error)


if __name__ == '__main__':
    p=argparse.ArgumentParser()
    p.add_argument('archive',type=Path)
    p.add_argument('--checker',type=Path,required=True)
    p.add_argument('--output',type=Path,required=True)
    p.add_argument('--seconds',type=float,default=120)
    p.add_argument('--profile',action='store_true')
    run(p.parse_args())
