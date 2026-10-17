"""Explicit source-stage bridge and mode-scoped conservative seed inventory."""
from pathlib import Path
import tomllib

import audit_ere_categorical_comparison as legacy
import audit_ere_categorical_comparison_bounded as bounded

a = legacy.a
p = legacy.p
ROOT = legacy.ROOT
PROFILE = 'E2-categorical-comparison-storage-launch-bridge-v1'
DEV = ROOT / 'output/ere_categorical_comparison_development_20261002'
DEV_ROOT = '3a9d48f7059dd910a8e008a566237cbf03935790629902f1b9305d496ecf3682'
OPT = ROOT / 'output/ere_categorical_comparison_bounded_auditor_v4_20261002'
OPT_ROOT = '5c5cbc9f902fa6dde7505a69f8df0fb8b32881ebbb6a3ac15a553c2fd3091671'
MODULE = 'crates/hg-train/src/ere.rs'
CARGO = 'crates/hg-train/Cargo.toml'
OLD_NATIVE = 'crates/hg-train/src/ere/categorical_comparison.rs'
NEW_NATIVE = 'crates/hg-train/src/ere/categorical_comparison_bridge.rs'
OLD_BIN = 'crates/hg-train/src/bin/ere-categorical-comparison.rs'
NEW_BIN = 'crates/hg-train/src/bin/ere-categorical-comparison-bridge.rs'
CHECKER = 'scripts/audit_ere_categorical_comparison_bridge.py'
BIN = 'ere-categorical-comparison-bridge'
BUILD = ['cargo', 'build', '--offline', '--release', '-p', 'hg-train',
         '--no-default-features', '--features', 'ere-complete-signal', '--bin', BIN]
EXTRA = {NEW_NATIVE, NEW_BIN, CHECKER,
         'scripts/ere_categorical_comparison_launch_bridge.py',
         'scripts/run_ere_categorical_comparison_bridge.py',
         'scripts/test_ere_categorical_comparison_launch_bridge.py',
         'docs/ere_categorical_comparison_launch_bridge_v1.md',
         'scripts/audit_ere_categorical_comparison_bounded.py',
         'scripts/test_audit_ere_categorical_comparison_bounded.py'}

NATIVE_SUFFIX = '\n\n/// Validate only: never constructs Runner, grammar, RNG, or Engine state.\npub fn check_prepared(out: &Path, root: &str) -> Res<Value> {\n    let _ = prepared(out, root)?;\n    let m = a::load(&out.join("manifest.json"))?;\n    a::check(\n        m == a::load(&out.join("initial_manifest.json"))?\n            && m["recorded_cells"] == 0\n            && m["grammar_compilations"] == 0\n            && m["training_updates"] == 0,\n        "check-only fresh zero-work manifest",\n    )?;\n    Ok(json!({"bridge_preflight_checked":true,"native_evaluations_run":0,\n        "native_steps_run":0,"grammar_compilations_run":0,"training_updates":0}))\n}\n'


def expected_native(original):
    path = 'source_snapshots/scripts/audit_ere_categorical_comparison.py'
    marker = '&& audit["campaign_complete"] == false,'
    assert original.count(path) == original.count(marker) == 1
    return original.replace(path, 'source_snapshots/' + CHECKER, 1).replace(
        marker, '&& audit["campaign_complete"] == false\n            && audit["launch_bridge_verified"] == true,', 1) + NATIVE_SUFFIX + '\n'


def expected_binary(original):
    result = original.replace('hg_train::ere::categorical_comparison::',
                              'hg_train::ere::categorical_comparison_bridge::')
    result = result.replace('ere-categorical-comparison ', BIN + ' ')
    result = result.replace(' | math-fixture', ' | check-prepared PREPARED_DIRECTORY PREPARED_ROOT_SHA256 (zero native work) | math-fixture')
    anchor = '        [command, catalogue, request, out]'
    assert result.count(anchor) == 1
    return result.replace(anchor, '''        [command, out, root] if command == "check-prepared" => {
            hg_train::ere::categorical_comparison_bridge::check_prepared(std::path::Path::new(out), root)
                .map(|receipt| println!("{receipt}"))
        },
        [command, catalogue, request, out]''', 1)


def registration(base, c, check_historical=False):
    base = Path(base)
    original = DEV / 'source_snapshots'
    if check_historical:
        a.verify(DEV, expected=DEV_ROOT, count=692)
        a.verify(OPT, expected=OPT_ROOT, count=29)
        legacy.registration(original, c, check_historical=True)
    old = a.load(DEV / 'snapshot.json')['sources']
    assert len(old) == 472
    for name, digest in old.items():
        if name not in (MODULE, CARGO):
            assert a.digest(base / name) == digest, f'unregistered historical source drift: {name}'
    before = (original / MODULE).read_text()
    anchor = '#[cfg(feature = "ere-complete-signal")]\npub mod categorical_comparison;\n'
    assert before.count(anchor) == 1
    assert (base / MODULE).read_text() == before.replace(anchor, anchor + '#[cfg(feature = "ere-complete-signal")]\npub mod categorical_comparison_bridge;\n', 1), 'module change beyond additive bridge export'
    before_cargo = tomllib.loads((original / CARGO).read_text())
    after_cargo = tomllib.loads((base / CARGO).read_text())
    assert after_cargo['bin'].pop() == dict(name=BIN, path='src/bin/ere-categorical-comparison-bridge.rs', **{'required-features': ['ere-complete-signal']})
    assert before_cargo == after_cargo, 'Cargo change beyond additive bridge binary'
    cargo_append = '\n[[bin]]\nname = "ere-categorical-comparison-bridge"\npath = "src/bin/ere-categorical-comparison-bridge.rs"\nrequired-features = ["ere-complete-signal"]\n'
    assert (base / CARGO).read_text() == (original / CARGO).read_text() + cargo_append, 'Cargo bytes changed beyond exact append'
    assert (base / NEW_NATIVE).read_text() == expected_native((original / OLD_NATIVE).read_text()), 'native bridge changed outside dispatch/guard/check-only function'
    assert (base / NEW_BIN).read_text() == expected_binary((original / OLD_BIN).read_text()), 'binary changed outside registered dispatch'
    opt_sources = a.load(OPT / 'registration.json')['sources']
    for name in ('scripts/audit_ere_categorical_comparison_bounded.py', 'scripts/test_audit_ere_categorical_comparison_bounded.py'):
        assert a.digest(base / name) == opt_sources[name], 'storage implementation changed'
    names = set(old) | EXTRA
    assert len(EXTRA) == 9 and len(names) == 481
    assert all((base / name).is_file() for name in names)
    train = {str(n.relative_to(base)) for n in (base / 'crates/hg-train').rglob('*') if n.is_file() and n.suffix in ('.rs', '.toml')}
    assert train == {n for n in names if n.startswith('crates/hg-train/')}, 'unregistered learning source'
    return dict(profile=PROFILE, source_names=sorted(names), baseline_sources=472,
                unchanged_historical_paths=470, new_registered_paths=sorted(EXTRA),
                additive_exports={n: dict(before_sha256=old[n], after_sha256=a.digest(base / n)) for n in (MODULE, CARGO)},
                native_body_identical_except_dispatch_guard_and_check_only=True,
                full_source_identity_claim=False, numerical_config_sha256=legacy.CONFIG_SHA,
                original_development_root=DEV_ROOT, storage_verification_root=OPT_ROOT)


def seed_inventory(c, mode, output_root=None, exclude=None):
    """Reserve whole ambiguous/authorized schedules, never outcome-selected prefixes."""
    assert mode in ('development', 'scientific')
    output_root = Path(output_root) if output_root is not None else ROOT / 'output'
    exclude = Path(exclude).resolve() if exclude is not None else None
    assert not output_root.is_symlink()
    seeds = set()
    paths = []
    for path in sorted(output_root.glob('*/schedule.json')):
        parent = path.parent
        if exclude is not None and parent.resolve() == exclude:
            continue
        assert not parent.is_symlink() and not path.is_symlink(), 'symlink seed history'
        pins = {}
        metadata = {}
        for name in ('manifest.json', 'request.json'):
            file = parent / name
            if file.exists():
                assert file.is_file() and not file.is_symlink(), 'symlink seed metadata'
                metadata[name] = a.load(file)
                pins[name] = a.digest(file)
        m = metadata.get('manifest.json', {})
        req = metadata.get('request.json', {})
        zero = (isinstance(m, dict) and isinstance(req, dict)
                and m.get('status') == 'zero_work_preflight'
                and all(type(m.get(k)) is int and m[k] == 0 for k in ('recorded_cells', 'attempted_steps', 'advancing_steps', 'unknown_partial_work_cells', 'direction_starts', 'direction_estimates', 'conditional_leaf_checks', 'grammar_compilations', 'optimizer_generations', 'training_updates'))
                and m.get('inflight_native_work_unknown') is False
                and req.get('launch_authorized') is False)
        if zero:
            continue
        schedule = a.load(path)
        if not isinstance(schedule, list):
            continue  # Same scope as the original checker: explicit list schedules.
        for q in schedule:
            if isinstance(q, dict):
                for key in ('emission_seed', 'physics_seed', 'noise_seed'):
                    value = q.get(key)
                    if value is not None:
                        assert type(value) is int and 0 <= value < 2**64, 'malformed explicit seed'
                        seeds.add(value)
        paths.append(dict(directory=parent.name, schedule_sha256=a.digest(path), metadata_sha256=pins))
    collision = seeds & p.check_schedule(c, mode)['seeds']
    assert not collision, f'{mode} historical/reserved seed collision; prospective revision required'
    return dict(profile=PROFILE, selected_mode=mode, prior_schedule_files=paths,
                explicit_seed_identities=len(seeds), collision=False,
                reservation='full listed schedule unless proven unauthorized zero-work',
                scope='explicit root schedule lists only; no claim about implicit RNG draws')


def gate(preflight=None):
    return dict(profile=PROFILE, development=dict(path=str(DEV), root_seal=DEV_ROOT),
                storage_verification=dict(path=str(OPT), root_seal=OPT_ROOT),
                launch_preflight=preflight)


def validate_preflight_reference(ref, snap, out):
    assert isinstance(ref, dict) and set(ref) == {'path', 'root_seal'}
    assert isinstance(ref['root_seal'], str) and len(ref['root_seal']) == 64 and all(c in '0123456789abcdef' for c in ref['root_seal']), 'explicit preflight root required'
    assert isinstance(ref['path'], str) and Path(ref['path']).is_absolute() and not Path(ref['path']).is_symlink()
    pre = Path(ref['path']).resolve()
    assert pre.parent == (ROOT / 'output').resolve() and pre != Path(out).resolve()
    a.verify(pre, expected=ref['root_seal'])
    request = a.load(pre / 'request.json')
    assert request['launch_authorized'] is False and request['full_training_authorized'] is False and request['mode'] == 'scientific'
    parent_snapshot = a.load(pre / 'snapshot.json')
    p.same(parent_snapshot['sources'], snap['sources'])
    assert parent_snapshot['binary_sha256'] == snap['binary_sha256'], 'bridge preflight binary identity changed'
    p.same(a.load(pre / 'development_gate.json'), gate())
    p.same(a.load(pre / 'schedule.json'), p.schedule(a.load(pre / 'config.json'), 'scientific'))
    assert a.digest(pre / 'config.json') == legacy.CONFIG_SHA
    manifest = a.load(pre / 'manifest.json')
    assert manifest == a.load(pre / 'initial_manifest.json') and manifest['status'] == 'zero_work_preflight'
    for key in ('recorded_cells', 'attempted_steps', 'advancing_steps', 'unknown_partial_work_cells', 'direction_starts', 'direction_estimates', 'conditional_leaf_checks', 'grammar_compilations', 'optimizer_generations', 'training_updates'):
        assert type(manifest[key]) is int and manifest[key] == 0
    assert manifest['training_admitted'] is False and manifest['inflight_native_work_unknown'] is False
    saved = a.load(pre / 'audit.json')
    assert saved['audit_pass'] is True and saved['launch_bridge_verified'] is True
    assert saved['classification'] == 'zero_work_preflight' and saved['frozen_sources'] == 481
    assert saved['campaign_complete'] is False and saved['scientific_campaign_complete'] is False and saved['training_admitted'] is False
    for q in a.load(pre / 'schedule.json'):
        p.same(a.load(pre / 'cells' / (q['id'] + '.json')), dict(schedule=q, status='NotRun', reason='not_started', evaluation=None))


def validate_gate(value, snap, receipt, c, mode, seeds, out):
    assert mode == 'scientific', 'bridge is science-only; development is already complete'
    assert seeds.get('selected_mode') == mode, 'seed inventory mode mismatch'
    authorized = a.load(Path(out) / 'request.json')['launch_authorized']
    assert isinstance(value, dict) and set(value) == set(gate())
    ref = value['launch_preflight']
    p.same(value, gate(ref))
    if authorized:
        assert ref is not None, 'authorized science requires source-identical zero-work bridge preflight'
        validate_preflight_reference(ref, snap, out)
    else:
        assert ref is None
    a.verify(DEV, expected=DEV_ROOT, count=692)
    a.verify(OPT, expected=OPT_ROOT, count=29)
    expected = registration(Path(out) / 'source_snapshots', c)
    p.same(receipt, expected)
    assert set(snap['sources']) == set(expected['source_names'])
    assert all(a.digest(Path(out) / 'source_snapshots' / n) == h for n, h in snap['sources'].items())
    verified = a.load(OPT / 'verification.json')
    assert verified['engineering_pass'] is True and verified['audit_results_exactly_equivalent'] is True
    assert verified['passed_regression_groups'] == 27 and verified['detached_tamper_rejections'] == 14
    assert verified['scientific_support_claim'] is False and verified['launch_bridge_implemented'] is False
    assert a.load(OPT / 'manifest.json')['status'] == 'complete'
    preserved = a.load(OPT / 'preserved_measurements.json')
    saved = a.load(DEV / 'audit.json')
    controls = saved.pop('detached_tamper_rejections')
    assert len(controls) == 6
    p.same(preserved['legacy-development']['result'], saved)
    p.same(preserved['bounded-development']['result'], saved)
    bounded.verify_storage_only()
    prior = bounded.audit(DEV)
    p.same(prior, saved)
    assert prior['campaign_complete'] and prior['classification'] == 'bounded_development' and prior['unknown_partial_work_cells'] == 0
    assert prior['training_admitted'] is False and prior['scientific_campaign_complete'] is False
    p.same(seeds, a.load(Path(out) / 'build_receipt.json')['seed_collision_check'])
    p.same(seeds, seed_inventory(c, mode, exclude=out))
