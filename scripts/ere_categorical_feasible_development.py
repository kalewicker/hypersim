"""Prospective fixed development design and explicit source/seed registration."""
from pathlib import Path
import audit_ere_categorical_feasible as mathcheck

a=mathcheck.a; cat=mathcheck.cat; ROOT=mathcheck.ROOT
PROFILE='E2-categorical-feasibility-development-v1'
CONFIG='configs/ere_categorical_feasible_development_v1.json'
BASELINE=ROOT/'output/ere_categorical_feasible_engineering_v2_20261003'
BASELINE_ROOT='7625a3c612264ca4ee40e769e9bcb9857af885f7ac0d52ebb1eaabaae54eb861'
BIN='ere-categorical-feasible-development'
BUILD=['cargo','build','--offline','--locked','--release','-p','hg-train','--no-default-features',
       '--features','ere-complete-signal','--bin',BIN]
NEW={CONFIG,'docs/ere_categorical_feasible_development_v1.md',
     'crates/hg-train/src/ere/categorical_feasible_development.rs',
     'crates/hg-train/src/bin/ere-categorical-feasible-development.rs',
     'scripts/ere_categorical_feasible_development.py',
     'scripts/audit_ere_categorical_feasible_development.py',
     'scripts/run_ere_categorical_feasible_development.py',
     'scripts/test_ere_categorical_feasible_development.py'}
MODULE='crates/hg-train/src/ere.rs'; CARGO='crates/hg-train/Cargo.toml'


def design():
    return dict(profile=PROFILE,mode='development',master_seed=0xE66200000000,
        blocks=1,contexts=2,estimates_per_context=2,utility_draws=16,risk_draws=512,
        validation_draws_per_context=8,physics_per_validation_draw=4,
        estimation_evaluations=2112,validation_evaluations=64,total_evaluations=2176,
        maximum_native_attempts=217600,conditional_leaf_checks=2128,parameters=183,horizon=100,
        wall_seconds=900,cell_wall_seconds=30,auditor_wall_seconds=900,output_cap_bytes=2147483648,
        receipt_reserve_bytes=134217728,final_reserve_bytes=16777216,work=50000,results=50000,
        optimizer_generations=0,training_updates=0,scientific_support=False,full_training_authorized=False)


def same(got,expected):
    if isinstance(expected,dict):
        assert type(got) is dict and got.keys()==expected.keys()
        for k,v in expected.items():same(got[k],v)
    elif isinstance(expected,list):
        assert type(got) is list and len(got)==len(expected)
        for x,y in zip(got,expected):same(x,y)
    else:assert type(got) is type(expected) and got==expected


def contexts():return mathcheck.history.s.original.p.contexts()


def schedule():
    master=design()['master_seed'];rows=[]
    for ctx in range(2):
        for est in range(2):
            for stage,n,offset in [('utility_estimation',16,0),('risk_estimation',512,1000)]:
                for draw in range(n):
                    j=ctx*100000+est*10000+offset+draw
                    rows.append(dict(id=f'est-c{ctx}-e{est}-{stage}-d{draw}',stage=stage,
                        block=0,context=ctx,estimate=est,draw=draw,
                        emission_seed=master+20000000+j,physics_seed=master+40000000+j,horizon=100))
    for ctx in range(2):
        for draw in range(8):
            for physics in range(4):
                j=ctx*100000
                rows.append(dict(id=f'val-c{ctx}-d{draw}-p{physics}',stage='validation',block=0,
                    context=ctx,draw=draw,physics_index=physics,sampled_arm='base',
                    emission_seed=master+60000000+j+draw,
                    physics_seed=master+80000000+j+4*draw+physics,horizon=100))
    return rows


def check_schedule(plan):
    same(plan,schedule());assert len(plan)==2176
    assert len({q['id'] for q in plan})==2176
    # Validation repeats one emission label for four physics seeds, nothing else.
    emissions={q['emission_seed'] for q in plan};physics={q['physics_seed'] for q in plan}
    assert len(emissions)==2128 and len(physics)==2176 and not emissions&physics
    return emissions|physics


def initial():
    return dict(profile=PROFILE,mode='development',status='zero_work_preflight',reason=None,
        scheduled_cells=2176,recorded_cells=0,attempted_steps=0,advancing_steps=0,
        unknown_partial_work_cells=0,inflight_native_work_unknown=False,grammar_compilations=0,
        optimizer_generations=0,training_updates=0,conditional_leaf_checks=0,
        direction_starts=0,direction_estimates=0,receipt_bytes=0,training_admitted=False,
        full_training_authorized=False,live_sources_rechecked=False,elapsed_seconds=0.)


def summary(complete,points=None):
    return dict(profile=PROFILE,mode='development',measurement_complete=complete,
        fixed_proposals=4,fixed_validation_leaves_per_context=8,
        context_points=points if complete else None,scientific_support=False,
        training_admitted=False,training_updates=0,unseen_cap_feasibility_certificate=False,
        independent_physics_replay=False,independent_rng_replay=False)


def registration(base,verify_lineage=False):
    base=Path(base)
    if verify_lineage:
        a.verify(BASELINE,expected=BASELINE_ROOT,count=1156)
        assert a.load(BASELINE/'manifest.json')['status']=='complete'
        assert a.load(BASELINE/'audit.json')['engineering_arithmetic_pass'] is True
    old=a.load(BASELINE/'registration.json')['sources'];assert len(old)==1134
    excluded=lambda n:n.startswith('crates/hg-sim-core/src/evolving_minds/') or n=='crates/hg-sim-core/src/bin/evolving_minds.rs' or n.startswith('crates/hg-sim-core/tests/evolving_minds_')
    changes={}
    for n,h in old.items():
        if n not in (MODULE,CARGO) and a.digest(base/n)!=h:
            assert excluded(n),'unregistered in-scope baseline drift: '+n
            changes[n]=dict(before_sha256=h,after_sha256=a.digest(base/n))
    before=(BASELINE/'source_snapshots'/MODULE).read_text();anchor='pub mod categorical_feasible;\n'
    assert before.count(anchor)==1
    assert (base/MODULE).read_text()==before.replace(anchor,anchor+'#[cfg(feature = "ere-complete-signal")]\npub mod categorical_feasible_development;\n')
    append='\n[[bin]]\nname = "'+BIN+'"\npath = "src/bin/'+BIN+'.rs"\nrequired-features = ["ere-complete-signal"]\n'
    assert (base/CARGO).read_text()==(BASELINE/'source_snapshots'/CARGO).read_text()+append
    rust={str(p.relative_to(base)) for p in (base/'crates').rglob('*')
          if p.is_file() and p.suffix in ('.rs','.toml') and 'target' not in p.parts}
    extra_rust=rust-set(old)-NEW
    assert all(excluded(n) for n in extra_rust),'unregistered in-scope workspace build source'
    names=set(old)|NEW|extra_rust
    assert rust=={n for n in names if n.startswith('crates/') and Path(n).suffix in ('.rs','.toml')}
    return dict(profile=PROFILE,baseline_root=BASELINE_ROOT,baseline_sources=1134,
        unchanged_baseline_sources=1132-len(changes),new_sources=sorted(NEW),source_names=sorted(names),
        excluded_feature='evolving-minds',excluded_feature_drift=changes,excluded_feature_additions=sorted(extra_rust),
        additive_exports={n:dict(before_sha256=old[n],after_sha256=a.digest(base/n)) for n in (MODULE,CARGO)},
        historical_source_identity_claim=False,unchanged_adapter=True,unchanged_native_evaluator=True)


def seed_inventory(output_root=None,exclude=None):
    output_root=Path(output_root or ROOT/'output');exclude=Path(exclude).resolve() if exclude else None
    assert not output_root.is_symlink();seeds=set();files=[]
    for path in sorted(output_root.glob('*/schedule.json')):
        if exclude and path.parent.resolve()==exclude:continue
        assert not path.is_symlink() and not path.parent.is_symlink()
        metadata={};pins={}
        for name in ('manifest.json','request.json'):
            file=path.parent/name
            if file.exists():
                assert file.is_file() and not file.is_symlink()
                metadata[name]=a.load(file);pins[name]=a.digest(file)
        m=metadata.get('manifest.json',{});req=metadata.get('request.json',{})
        zero=isinstance(m,dict) and isinstance(req,dict) and m.get('status')=='zero_work_preflight' and req.get('launch_authorized') is False and m.get('inflight_native_work_unknown') is False and all(type(m.get(k)) is int and m[k]==0 for k in ('recorded_cells','attempted_steps','advancing_steps','unknown_partial_work_cells','grammar_compilations','optimizer_generations','training_updates','conditional_leaf_checks','direction_starts','direction_estimates'))
        if zero:continue
        plan=a.load(path)
        if not isinstance(plan,list):continue
        for q in plan:
            if isinstance(q,dict):
                for k in ('emission_seed','physics_seed','noise_seed','random_parameter_seed'):
                    s=q.get(k)
                    if s is not None:
                        assert type(s) is int and 0<=s<2**64
                        seeds.add(s)
        files.append(dict(directory=path.parent.name,schedule_sha256=a.digest(path),metadata_sha256=pins))
    assert not seeds&check_schedule(schedule()),'historical seed collision; prospective revision required'
    return dict(profile=PROFILE,prior_schedule_files=files,explicit_seed_identities=len(seeds),
        collision=False,reservation='whole explicit root list unless proven unauthorized zero-work',
        implicit_rng_collision_claim=False)
