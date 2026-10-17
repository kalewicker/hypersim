#!/usr/bin/env python3
"""Independent archived accounting audit, not a native-physics replay claim."""
import argparse
import collections
import hashlib
import json
import math
import struct
import shutil
import tempfile
from pathlib import Path

SEED = 0xE6523A0001
PHYSICS = [941, 1051, 1153, 1259]
GRAMMAR_SEAL = 'ccc22798899e426f93cd2801020fef0e93391e8790a23a214d22b625caa58891'
CATALOGUE_SHA = 'ca02b55e5939ead818cdfce321c74b2f13c1ac7bde3c500061ca3ff3c402c8a6'
SIGNAL_SEAL = '8166cb77242f4e141f204b0c7b1d8e6c40d04f366fa9ed0143231255de72b205'
SIGNAL_SOURCE = 'crates/hg-sim-core/src/ere/signal.rs'
BUDGET = {'work':50000,'results':50000,'policy':'Pooled'}

def visibility_only(historical,current):
    return historical.count('fn evaluate_with_budget(')==1 and current==historical.replace('fn evaluate_with_budget(','pub fn evaluate_with_budget(',1)

def signal_lineage(out):
    m=load(out/'manifest.json');p=out/'prerequisites/signal'
    assert digest(p/'output_sha256.json')==SIGNAL_SEAL==m['previous_signal']['seal_sha256']
    sealed=load(p/'output_sha256.json')
    for name in ['manifest.json','summary.json','audit.json','postmeasurement_audit/correction.json',
                 'postmeasurement_audit/audit_ere_valid_optimizer_signal.py','source_snapshots/scripts/audit_ere_valid_optimizer_signal.py']:
        assert digest(p/name)==sealed[name],name
    pm=load(p/'manifest.json');ps=load(p/'summary.json');pa=load(p/'audit.json')
    assert pm['status']=='complete' and pm['training_admitted'] is False
    assert ps['precheck_pass'] and ps['valid_rollouts']==120 and ps['valid_pairs']==20
    assert pa['audit_pass'] and pa['tamper_rejections']==19
    c=load(p/'postmeasurement_audit/correction.json')
    assert c['original_checker_result']['audit_pass'] is False
    assert c['measurement_files_changed'] is False and c['native_physics_rerun'] is False
    original=p/'source_snapshots/scripts/audit_ere_valid_optimizer_signal.py'
    corrected=p/'postmeasurement_audit/audit_ere_valid_optimizer_signal.py'
    assert digest(original)==c['original_auditor_sha256']
    assert digest(corrected)==c['corrected_auditor_sha256']
    expected=original.read_text().replace('    c=grammar_basis(out, integrity)','    catalogue=grammar_basis(out, integrity)').replace('check_membership(c,e,full=True)','check_membership(catalogue,e,full=True)').replace("check_membership(c,sample['selection'])","check_membership(catalogue,sample['selection'])")
    assert corrected.read_text()==expected
    for name,key in [('manifest.json','measurement_manifest_sha256'),('summary.json','measurement_summary_sha256')]:
        assert digest(p/name)==c[key]
    assert m['previous_signal']['original_frozen_checker_pass'] is False
    assert m['previous_signal']['corrected_accounting_pass'] is True

def grammar_basis(out, integrity, runtime=True):
    m=load(out/'manifest.json')
    frozen=out/'prerequisites/grammar'
    assert digest(frozen/'output_sha256.json')==GRAMMAR_SEAL
    sealed=load(frozen/'output_sha256.json')
    for name in ['manifest.json','audit.json','catalogue.json']:
        assert digest(frozen/name)==sealed[name]
    assert digest(out/'grammar_catalogue.json')==digest(frozen/'catalogue.json')==CATALOGUE_SHA
    assert m['grammar_prerequisite']['seal_sha256']==GRAMMAR_SEAL
    assert m['grammar_prerequisite']['catalogue_sha256']==CATALOGUE_SHA
    gm=load(frozen/'manifest.json')
    ga=load(frozen/'audit.json')
    assert gm['status']=='complete' and ga['audit_pass'] and ga['legal_productions']==18956
    assert ga['engine_steps']==ga['optimizer_rollouts']==0
    if runtime:
        signal_lineage(out)
        historical=frozen/'historical_signal.rs'
        current=out/'source_snapshots'/SIGNAL_SOURCE
        assert digest(historical)==gm['sources'][SIGNAL_SOURCE]
        assert visibility_only(historical.read_text(),current.read_text())
        proof=m['grammar_prerequisite']['native_visibility']
        assert proof['historical_sha256']==digest(historical) and proof['current_sha256']==digest(current)
        assert proof['visibility_only'] and proof['evaluator_body_unchanged']
    if integrity:
        for rel,h in gm['sources'].items():
            if rel.startswith(('crates/hg-core/','crates/hg-engine/','crates/hg-meta/','crates/hg-rules/')) or rel in ['Cargo.lock','crates/hg-train/src/optim.rs','crates/hg-sim-core/src/ere/coherent.rs','crates/hg-sim-core/src/ere/signal.rs','crates/hg-sim-core/src/ere/valid_grammar.rs','crates/hg-sim-core/src/ere/semantic.rs','crates/hg-sim-core/src/ere/experiments.rs']:
                if runtime and rel==SIGNAL_SOURCE:
                    continue  # Exact whole-source visibility-only relation verified above.
                assert digest(out/'source_snapshots'/rel)==h,rel
    return load(out/'grammar_catalogue.json')

def check_membership(c,e,full=False):
    assert e['grammar_profile']=='E2-complete-valid-production-grammar-v1'
    p,op=e['parent'],e['operator']
    assert 0<=p<168 and 0<=op<13
    role=['Core','Proposal'].index(e['role'])
    parent=c['parents'][p]
    group=parent['groups'][op]
    entries=group['roles'][role]['productions']
    assert 0<=e['production_index']<len(entries)
    entry=entries[e['production_index']]
    assert e['production_count']==len(entries)
    assert e['operator_support']==parent['operator_support'] and e['role_support']==group['role_support']
    assert e['signature']==entry['execution']
    assert e['kernel_attempts']==1 and e['resampled'] is False and e['parent_fallback'] is False
    if full:
        assert e['kernel']['request']==entry['request'] and e['kernel']['role']==e['role']
        assert e['classes']['rcf']==entry['rcf'] and e['status']=='Applied' and e['admitted']
        assert e['root']==parent['root'] and e['kernel']['committed']

def load(p):
    return json.loads(p.read_text())

def digest(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()

def f32(x):
    return struct.unpack('f', struct.pack('f', x))[0]

def mean(xs):
    return sum(xs) / len(xs)

def sd(xs):
    m = mean(xs)
    return math.sqrt(mean([(x-m)**2 for x in xs]))

def ranks(fits):
    ids = sorted((i for i,x in enumerate(fits) if x is not None), key=lambda i: fits[i])
    result = [0.] * len(fits)
    if len(ids) < 2:
        return result
    start = 0
    while start < len(ids):
        end = start+1
        while end < len(ids) and fits[ids[end]] == fits[ids[start]]:
            end += 1
        w = f32(f32(f32(start+end-1) / f32(2*(len(ids)-1))) - .5)
        for i in ids[start:end]:
            result[i] = w
        start = end
    return result

def distance(a,b):
    aa = collections.Counter(json.dumps(x['signature']) for x in a)
    bb = collections.Counter(json.dumps(x['signature']) for x in b)
    return sum(abs(aa[k]-bb[k]) for k in aa.keys() | bb.keys()) / 128

def close(a,b):
    assert (a is None and b is None) or (a is not None and b is not None and abs(a-b)<1e-9), (a,b)

def check_allowances(usage,work,results):
    w,r=work,results
    for rule in usage['rules']:
        assert rule['work_allowance']==w and rule['result_allowance']==r
        assert 0<=rule['work']<=w and 0<=rule['results']<=r
        w-=rule['work'];r-=rule['results']
    assert usage['work']==work-w and usage['results']==results-r

def _audit(out, integrity=True, reference=False):
    seed,physics=(0xE552390001,[509,613,719,827]) if reference else (SEED,PHYSICS)
    catalogue=grammar_basis(out, integrity, runtime=not reference)
    m = load(out/'manifest.json')
    assert m['status']=='complete' and m['training_admitted'] is False
    assert m['frozen_before_measurement'] and m['development_only']
    assert m['profile']==('E2-complete-valid-optimizer-signal-v1' if reference else 'E2-complete-runtime50k-signal-v1')
    assert m['scheduled_rollouts']==120 and m['horizon']==100 and m['master_seed']==seed
    assert m['matcher_work_model']=='repeated-self-pair-feasibility-v1'
    if not reference:
        assert m['runtime_match_budget']==BUDGET
    if integrity:
        for name,h in m['snapshot']['sources'].items():
            assert digest(out/'source_snapshots'/name)==h, name
        assert digest(out/'executed_binary')==m['snapshot']['binary_sha256']
        gate=Path(m['prerequisite']['archive'])
        assert digest(gate/'output_sha256.json')==m['prerequisite']['seal_sha256']=='fa3725f7cf5a03b207635d4c121d47280d38b4a05ae018ec3606e00086a4b60a'
        gate_seal=load(gate/'output_sha256.json')
        for name in ['manifest.json','classification_summary.json','run_status.json','audit_full.json']:
            assert digest(gate/name)==gate_seal[name]
        assert load(gate/'run_status.json')['gate_pass']
        checked=0
        for name,h in gate_seal.items():
            rel=name.removeprefix('source_snapshots/')
            if name.startswith('source_snapshots/') and rel.startswith(('crates/hg-engine/','crates/hg-core/','crates/hg-rules/','crates/hg-meta/')) and Path(rel).suffix in ['.rs','.toml']:
                assert digest(out/'source_snapshots'/rel)==h, rel
                checked+=1
        assert checked>0
        if (out/'output_sha256.json').exists():
            sealed=load(out/'output_sha256.json')
            assert set(sealed)=={str(p.relative_to(out)) for p in out.rglob('*') if p.is_file() and p!=out/'output_sha256.json'}
            for name,h in sealed.items():
                assert digest(out/name)==h, name
    rows = [json.loads(s) for s in (out/'rollouts.jsonl').read_text().splitlines()]
    assert len(rows)==120
    by_id = {r['id']:r for r in rows}
    assert len(by_id)==120
    scores = {}
    world_hashes = set()
    attempted = advanced = valid = 0
    for r in rows:
        e,v = r['emission'],r['evaluation']
        if not reference:
            assert v['runtime_match_budget']==BUDGET
        check_membership(catalogue,e,full=True)
        world_hashes.add(e['initial_world_hash'])
        assert e['status']==e['kernel']['status']
        assert e['kernel']['training_admitted'] is False
        assert e['kernel']['advancing_engine_steps']==0
        assert not e['kernel']['codec_reencoded'] and not e['kernel']['metadata_recounted']
        assert e['admitted']==(e['status'] in ['Applied','Inapplicable'])
        steps = v['steps']
        assert v['attempted_steps']==len(steps)
        assert v['advancing_steps']==sum(s['advanced'] for s in steps)
        attempted += len(steps)
        advanced += v['advancing_steps']
        if not e['admitted']:
            assert not steps and v['score'] is None and v['reason']=='rejected_emission'
            assert e['signature'] is None and e['classes'] is None
        else:
            assert e['signature'] is not None
            c = e['classes']
            eligible = c['active'] and c['core']!=c['proposal']
            ids = [i for i,b in enumerate(v['candidate_execution']) if eligible and b==c['proposal_execution']]
            assert ids==v['credited_rule_ids']
            fires=0
            failed=False
            for i,s in enumerate(steps,1):
                assert s['step']==i and len(s['rule_ids'])<=1
                assert all(0<=n<len(v['candidate_execution']) for n in s['rule_ids'])
                assert s['usage']['work']<=(400000 if reference else 50000) and s['usage']['results']<=50000
                check_allowances(s['usage'],400000 if reference else 50000,50000)
                fires += sum(n in ids for n in s['rule_ids'])
                if s['cap'] or not s['valid_family'] or s['advanced']!=1:
                    assert i==len(steps)
                    failed=True
            assert fires==v['proposal_fires']
            full = len(steps)==100 and not failed
            assert (v['score'] is not None)==full
            assert (v['reason'] is None)==full
            close(v['score'], math.log1p(max(fires-1,0))/math.log(100) if full else None)
        scores[r['id']]=v['score']
        valid += int(v['score'] is not None)
    assert len(world_hashes)==1 and attempted<=12000
    baseline=[]
    for i in range(8):
        for p in physics:
            r=by_id[f'baseline_{i}_{p}']
            assert r['emission']['emission_seed']==seed+2000+i and r['evaluation']['physics_seed']==p
            baseline.append(scores[r['id']])
    controls=[]
    for active in [True,False]:
        for p in physics:
            r=by_id[f'control_{str(active).lower()}_{p}']
            assert r['emission']['parent']==(4 if active else 5) and r['emission']['operator']==(11 if active else 12)
            assert r['emission']['emission_seed']==seed+3000 and r['evaluation']['physics_seed']==p
            controls.append(scores[r['id']])
    panel=load(out/'initial_distribution.json')
    assert len(panel)==64 and [x['seed'] for x in panel]==list(range(seed+1000,seed+1064))
    for sample in panel:
        check_membership(catalogue,sample['selection'])
        assert sample['signature']==sample['selection']['signature'] and sample['status']=='Applied'
    theta=[0.] * 183
    valid_pairs=0
    max_tv=0.
    for g in range(5):
        rec=load(out/f'generation_{g}.json')
        assert rec['generation']==g and rec['theta_before']==theta
        assert len(rec['eps'])==4 and all(len(x)==183 for x in rec['eps'])
        fits=[]
        for pair in range(4):
            arms=[]
            for sign in ['1','-1']:
                vals=[]
                for p in physics[:2]:
                    r=by_id[f'es_{g}_{pair}_{sign}_{p}']
                    assert r['emission']['emission_seed']==seed+4000+4*g+pair
                    assert r['evaluation']['physics_seed']==p
                    vals.append(scores[r['id']])
                arms.append(vals)
            good=all(x is not None for arm in arms for x in arm)
            expected=[f32(mean(a)) for a in arms] if good else [None,None]
            assert rec['pair_scores'][pair]==expected
            valid_pairs+=good
            fits.extend(expected)
        shaped=ranks(fits)
        for pair,eps in enumerate(rec['eps']):
            d=f32(shaped[2*pair]-shaped[2*pair+1])
            c=f32(f32(f32(.05)/f32(4.))*d)
            if d:
                theta=[f32(t+f32(c*e)) for t,e in zip(theta,eps)]
        for a,b in zip(theta,rec['theta_after']):
            assert abs(a-b)<2e-8, (a,b)
        # Match the archived Rust f32 representation for exact chaining.
        theta=rec['theta_after']
        assert [x['seed'] for x in rec['distribution']]==[x['seed'] for x in panel]
        for sample in rec['distribution']:
            check_membership(catalogue,sample['selection'])
            assert sample['signature']==sample['selection']['signature'] and sample['status']=='Applied'
        tv=distance(panel,rec['distribution'])
        close(tv,rec['tv_from_initial'])
        max_tv=max(max_tv,tv)
    baseline_ok = all(x is not None for x in baseline)
    controls_ok = all(x is not None for x in controls)
    bm=mean(baseline) if baseline_ok else None
    bs=sd(baseline) if baseline_ok else None
    gap=mean(controls[:4])-mean(controls[4:]) if controls_ok else None
    positive=by_id[f'control_true_{physics[0]}']['emission']
    negative=by_id[f'control_false_{physics[0]}']['emission']
    assert positive['classes']['active'] is True and negative['classes']['active'] is False
    for key in ['core','proposal','proposal_execution']:
        assert positive['classes'][key]==negative['classes'][key]
    force=positive['signature']
    checks={'P1':all(x['signature']!=force for x in panel),
            'P2':max_tv>=.05 and valid_pairs==20,
            'P3':gap is not None and bs is not None and gap>2*bs and gap>1e-12,
            'P4':bm is not None and .02<bm<.98}
    s=load(out/'summary.json')
    assert s['rollouts']==120 and s['valid_rollouts']==valid and s['valid_pairs']==valid_pairs
    assert s['attempted_steps']==attempted and s['advancing_steps']==advanced
    assert s['checks']==checks and s['precheck_pass']==all(checks.values())
    assert s['measurement_complete'] and not s['training_admitted']
    runtime_pass=valid==120 and attempted==advanced==12000
    if not reference:
        assert s['runtime_match_budget']==BUDGET and s['runtime_envelope_pass']==runtime_pass
    assert s['elapsed_seconds']<900 and m['catalogue_rebuilt']
    assert 0<=m['grammar_compile_seconds']<=s['elapsed_seconds']
    close(s['baseline_mean'],bm);close(s['baseline_population_sd'],bs);close(s['forced_gap'],gap)
    close(s['max_distribution_tv'],max_tv)
    within=math.sqrt(mean([sd(baseline[i:i+4])**2 for i in range(0,32,4)])) if baseline_ok else None
    between=sd([mean(baseline[i:i+4]) for i in range(0,32,4)]) if baseline_ok else None
    close(s['within_genotype_physics_sd'],within);close(s['between_genotype_sd'],between)
    return {'audit_pass':True,'scope':'archive accounting, not native physics replay',
            'rollouts':120,'valid_rollouts':valid,'attempted_steps':attempted,
            'valid_pairs':valid_pairs,'checks':checks,'grammar_membership_pass':True,'training_admitted':False,
            'reference_preflight':reference, 'runtime_envelope_pass':runtime_pass if not reference else None}

def audit(out,integrity=True):
    return _audit(out,integrity,reference=False)

def audit_reference(out):
    """Read-only preflight of this accounting loop on sealed real 400k/50k data.

    Not a runtime receipt; unreachable from the production command-line path.
    """
    assert digest(out/'output_sha256.json')==SIGNAL_SEAL
    result=_audit(out,integrity=True,reference=True)
    assert result['reference_preflight'] and result['runtime_envelope_pass'] is None
    return result

def seal(out):
    paths=sorted(p for p in out.rglob('*') if p.is_file() and p!=out/'output_sha256.json')
    assert sum(p.stat().st_size for p in paths)<512*1024**2
    hashes={str(p.relative_to(out)):digest(p) for p in paths}
    (out/'output_sha256.json').write_text(json.dumps(hashes,indent=2,sort_keys=True)+'\n')

def tamper_tests(out):
    """Bounded destructive tests operate only on fresh temporary copies."""
    names=['manifest.json','rollouts.jsonl','summary.json','initial_distribution.json','grammar_catalogue.json']+[f'generation_{g}.json' for g in range(5)]
    cases=['score','fires','seed','duplicate','step','credit','cap','pair','theta','epsilon','distribution','summary','world','membership','mask','fallback','retry','catalogue','control','configured_budget','evaluation_budget','native_body','native_allowance']
    for case in cases:
        with tempfile.TemporaryDirectory(prefix='ere-signal-tamper-') as tmp:
            dst=Path(tmp)
            for name in names:
                shutil.copyfile(out/name,dst/name)
            shutil.copytree(out/'prerequisites',dst/'prerequisites')
            native=dst/'source_snapshots'/SIGNAL_SOURCE
            native.parent.mkdir(parents=True,exist_ok=True)
            shutil.copyfile(out/'source_snapshots'/SIGNAL_SOURCE,native)
            rows=[json.loads(s) for s in (dst/'rollouts.jsonl').read_text().splitlines()]
            row=next(r for r in rows if r['id']==f'control_true_{PHYSICS[0]}')
            if case=='configured_budget':
                p=dst/'manifest.json';q=load(p);q['runtime_match_budget']['work']=400000;p.write_text(json.dumps(q))
            elif case=='evaluation_budget':row['evaluation']['runtime_match_budget']['work']=400000
            elif case=='native_allowance':row['evaluation']['steps'][0]['usage']['rules'][0]['work_allowance']=400000
            elif case=='native_body':native.write_text(native.read_text().replace('let stats = engine.step(&mut s);','let stats = engine.step(&mut s); // changed body'))
            elif case=='membership':row['emission']['production_index']=1000000
            elif case=='mask':row['emission']['role_support']=[]
            elif case=='fallback':row['emission']['parent_fallback']=True
            elif case=='retry':row['emission']['kernel_attempts']=2
            elif case=='catalogue':
                p=dst/'grammar_catalogue.json';q=load(p);q['parents'].pop();p.write_text(json.dumps(q))
            elif case=='control':row['emission']['parent']=5
            elif case=='score':row['evaluation']['score']=.5
            elif case=='fires':row['evaluation']['proposal_fires']=0
            elif case=='seed':rows[0]['emission']['emission_seed']+=1
            elif case=='duplicate':rows[0]['id']=rows[1]['id']
            elif case=='step':row['evaluation']['steps'][0]['advanced']=0
            elif case=='credit':row['evaluation']['credited_rule_ids']=[]
            elif case=='cap':row['evaluation']['steps'][0]['cap']=True
            elif case=='world':rows[0]['emission']['initial_world_hash']='different'
            elif case in ['pair','theta','epsilon','distribution']:
                p=dst/'generation_0.json';r=load(p)
                if case=='pair':r['pair_scores'][0]=[0.5,0.25]
                elif case=='theta':r['theta_after'][0]+=1
                elif case=='epsilon':
                    # A nonzero gradient is not guaranteed in g0: corrupt dimensionality.
                    r['eps'][0].pop()
                else:r['distribution'][0]['seed']+=1
                p.write_text(json.dumps(r))
            elif case=='summary':
                p=dst/'summary.json';r=load(p);r['checks']['P1']=not r['checks']['P1'];p.write_text(json.dumps(r))
            (dst/'rollouts.jsonl').write_text('\n'.join(json.dumps(r) for r in rows)+'\n')
            try:
                audit(dst,integrity=False)
            except (AssertionError,IndexError,KeyError,ValueError):
                continue
            raise AssertionError(f'tamper accepted: {case}')
    return len(cases)

if __name__=='__main__':
    parser=argparse.ArgumentParser()
    parser.add_argument('output',type=Path)
    parser.add_argument('--seal',action='store_true')
    parser.add_argument('--tamper-tests',action='store_true')
    args=parser.parse_args()
    result=audit(args.output)
    if args.tamper_tests:
        result['tamper_rejections']=tamper_tests(args.output)
    if args.seal:
        (args.output/'audit.json').write_text(json.dumps(result,indent=2)+'\n')
        seal(args.output)
        audit(args.output)
    print(json.dumps(result,indent=2))
