#!/usr/bin/env python3
"""Post-crash accounting, not a full frozen scaling-audit pass."""
import argparse
import json
from pathlib import Path
import sys
sys.dont_write_bytecode=True

def audit(out,integrity=True):
    sys.path.insert(0,str(out/'source_snapshots/scripts'))
    import audit_ere_complete_scaling as s
    a=s.a
    if integrity:a.verify(out)
    m=a.load(out/'manifest.json');original=a.load(out/'manifest_at_crash.json')
    assert original['status']=='evaluating' and original['inflight_native_work_unknown'] is True
    assert m['status']=='incomplete' and m['recovered_after_native_panic'] is True
    assert m['training_admitted'] is False and m['full_training_authorized'] is False
    assert m['design']==s.design() and m['schedule']==s.schedule(s.design())
    assert m['counts_cover_receipted_cells_only'] is True and m['unknown_partial_work_cells']==1
    for k in ('snapshot','admission','design','schedule','attempted_steps','advancing_steps','terminal_native_cells','recorded_cells','grammar_compilations'):
        assert m[k]==original[k]
    assert all(m[k]==0 for k in ('optimizer_evaluation_steps','precheck_evaluation_steps','held_out_evaluation_steps'))
    snap=m['snapshot'];assert len(snap['sources'])==244 and snap['frozen_before_measurement'] is True
    for n,h in snap['sources'].items():assert a.digest(out/'source_snapshots'/n)==h
    assert a.digest(out/'executed_binary')==snap['binary_sha256']
    for n,(seal,sha,count) in a.SPECS.items():
        p=m['admission']['historical_evidence']['receipts'][n];a.verify(Path(p['archive']),seal,sha,count)
    p=m['admission']['engineering_prerequisite'];a.verify(Path(p['archive']),expected=s.ENGINEERING,count=247)
    runtime=Path(m['admission']['historical_evidence']['receipts']['runtime']['archive'])
    old=(runtime/'source_snapshots'/s.SIGNAL).read_text();assert (out/'source_snapshots'/s.SIGNAL).read_text()==s.bridge(old)
    assert a.digest(out/'catalogue.json')==a.CATALOGUE
    c=a.load(out/'catalogue.json');e=a.load(out/'emissions.json');helper=a.helper(runtime)
    rows=[a.load(out/'cells'/(q['id']+'.json')) for q in m['schedule']]
    assert {p.name for p in (out/'cells').iterdir()}=={q['id']+'.json' for q in m['schedule']}
    attempts=advances=complete=unknown=notrun=0
    by_h={str(h):dict(scheduled=12,complete=0,incomplete=0,not_run=0) for h in m['design']['horizons']}
    for q,r in zip(m['schedule'],rows):
        assert r['schedule']==q
        if r['status']=='Complete':
            x=s.cell(r,e[q['production']],c,helper);assert x[2]
            attempts+=x[0];advances+=x[1];complete+=1;by_h[str(q['horizon'])]['complete']+=1
        elif r['status']=='Incomplete':
            assert r['evaluation'] is None and r['partial_steps_unknown'] is True and r['recovered_after_process_exit'] is True
            assert q==original['current_cell'];unknown+=1;by_h[str(q['horizon'])]['incomplete']+=1
        else:
            assert r['status']=='NotRun' and r['evaluation'] is None and r['reason']=='prior_native_panic'
            notrun+=1;by_h[str(q['horizon'])]['not_run']+=1
    assert (attempts,advances,complete,unknown,notrun)==(89900,89900,26,1,9)
    assert attempts==m['attempted_steps'] and advances==m['advancing_steps'] and complete==m['terminal_native_cells']
    pairs=triples=0
    for p in range(6):
        for seed in m['design']['physics_seeds']:
            trio=[x for x in rows if x['schedule']['production']==p and x['schedule']['physics_seed']==seed]
            for x,y in zip(trio,trio[1:]):
                if x['status']==y['status']=='Complete':
                    short=x['evaluation']['steps'];assert short==y['evaluation']['steps'][:len(short)];pairs+=1
            triples+=all(x['status']=='Complete' for x in trio)
    summary=a.load(out/'summary.json')
    assert summary['by_horizon']==by_h and summary['measurement_complete'] is False and summary['scaling_panel_pass'] is False
    assert summary['total_native_attempts'] is None and summary['total_native_advances'] is None
    assert summary['complete_cells']==26 and summary['incomplete_cells']==1 and summary['not_run_cells']==9
    return dict(recovery_audit_pass=True,classification='incomplete_crash_recovery_accounting',
        full_frozen_trial_audit_pass=False,measurement_complete=False,scaling_panel_pass=False,
        training_admitted=False,full_training_authorized=False,recorded_native_attempts=attempts,
        recorded_native_advances=advances,total_native_attempts=None,total_native_advances=None,
        complete_cells=complete,incomplete_cells=unknown,not_run_cells=notrun,
        verified_prefix_pairs=pairs,verified_complete_triples=triples,by_horizon=by_h,
        frozen_sources=244,independent_physics_replay=False)

def main():
    assert __debug__
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('output',type=Path);p.add_argument('--seal',action='store_true');args=p.parse_args()
    if args.seal:
        result=audit(args.output,integrity=False)
        (args.output/'recovery_audit.json').write_text(json.dumps(result,indent=2))
        sys.path.insert(0,str(args.output/'source_snapshots/scripts'))
        import audit_ere_complete_admission as a;a.seal(args.output)
    print(json.dumps(audit(args.output)))

if __name__=='__main__':main()
