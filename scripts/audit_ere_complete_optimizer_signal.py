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

SEED = 0xE452390001
PHYSICS = [101, 211, 307, 401]

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

def audit(out, integrity=True):
    m = load(out/'manifest.json')
    assert m['status']=='complete' and m['training_admitted'] is False
    assert m['frozen_before_measurement'] and m['development_only']
    assert m['profile']=='E2-complete-optimizer-signal-v1'
    assert m['scheduled_rollouts']==120 and m['horizon']==100 and m['master_seed']==SEED
    assert m['matcher_work_model']=='repeated-self-pair-feasibility-v1'
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
            for name,h in load(out/'output_sha256.json').items():
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
                assert s['usage']['work']<=400000 and s['usage']['results']<=50000
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
        for p in PHYSICS:
            r=by_id[f'baseline_{i}_{p}']
            assert r['emission']['emission_seed']==SEED+2000+i and r['evaluation']['physics_seed']==p
            baseline.append(scores[r['id']])
    controls=[]
    for active in [True,False]:
        for p in PHYSICS:
            r=by_id[f'control_{str(active).lower()}_{p}']
            assert r['emission']['parent']==4 and r['emission']['operator']==(11 if active else 12)
            assert r['emission']['emission_seed']==SEED+3000 and r['evaluation']['physics_seed']==p
            controls.append(scores[r['id']])
    panel=load(out/'initial_distribution.json')
    assert len(panel)==64 and [x['seed'] for x in panel]==list(range(SEED+1000,SEED+1064))
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
                for p in PHYSICS[:2]:
                    r=by_id[f'es_{g}_{pair}_{sign}_{p}']
                    assert r['emission']['emission_seed']==SEED+4000+4*g+pair
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
        tv=distance(panel,rec['distribution'])
        close(tv,rec['tv_from_initial'])
        max_tv=max(max_tv,tv)
    baseline_ok = all(x is not None for x in baseline)
    controls_ok = all(x is not None for x in controls)
    bm=mean(baseline) if baseline_ok else None
    bs=sd(baseline) if baseline_ok else None
    gap=mean(controls[:4])-mean(controls[4:]) if controls_ok else None
    force=by_id['control_true_101']['emission']['signature']
    checks={'P1':all(x['signature']!=force for x in panel),
            'P2':max_tv>=.05 and valid_pairs==20,
            'P3':gap is not None and bs is not None and gap>2*bs and gap>1e-12,
            'P4':bm is not None and .02<bm<.98}
    s=load(out/'summary.json')
    assert s['rollouts']==120 and s['valid_rollouts']==valid and s['valid_pairs']==valid_pairs
    assert s['attempted_steps']==attempted and s['advancing_steps']==advanced
    assert s['checks']==checks and s['precheck_pass']==all(checks.values())
    assert s['measurement_complete'] and not s['training_admitted']
    assert s['elapsed_seconds']<900
    close(s['baseline_mean'],bm);close(s['baseline_population_sd'],bs);close(s['forced_gap'],gap)
    close(s['max_distribution_tv'],max_tv)
    within=math.sqrt(mean([sd(baseline[i:i+4])**2 for i in range(0,32,4)])) if baseline_ok else None
    between=sd([mean(baseline[i:i+4]) for i in range(0,32,4)]) if baseline_ok else None
    close(s['within_genotype_physics_sd'],within);close(s['between_genotype_sd'],between)
    return {'audit_pass':True,'scope':'archive accounting, not native physics replay',
            'rollouts':120,'valid_rollouts':valid,'attempted_steps':attempted,
            'valid_pairs':valid_pairs,'checks':checks,'training_admitted':False}

def seal(out):
    paths=sorted(p for p in out.rglob('*') if p.is_file() and p.name!='output_sha256.json')
    assert sum(p.stat().st_size for p in paths)<512*1024**2
    hashes={str(p.relative_to(out)):digest(p) for p in paths}
    (out/'output_sha256.json').write_text(json.dumps(hashes,indent=2,sort_keys=True)+'\n')

def tamper_tests(out):
    """Bounded destructive tests operate only on fresh temporary copies."""
    names=['manifest.json','rollouts.jsonl','summary.json','initial_distribution.json']+[f'generation_{g}.json' for g in range(5)]
    cases=['score','fires','seed','duplicate','step','credit','cap','pair','theta','epsilon','distribution','summary','world']
    for case in cases:
        with tempfile.TemporaryDirectory(prefix='ere-signal-tamper-') as tmp:
            dst=Path(tmp)
            for name in names:
                shutil.copyfile(out/name,dst/name)
            rows=[json.loads(s) for s in (dst/'rollouts.jsonl').read_text().splitlines()]
            row=next(r for r in rows if r['id']=='control_true_101')
            if case=='score':row['evaluation']['score']=.5
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
            except (AssertionError,KeyError,ValueError):
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
