#!/usr/bin/env python3
"""Read-only segment integrity and independent admission-physics reconstruction.
Derived audit views live in temporary directories; they are not execution archives.
SHA-256 binds evidence, but does not authenticate a producer.
"""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import re
import tempfile
import time
import importlib.util
import sys

PROFILE = 'evolving-minds-d0-streaming-v1'
CASES = ['faithful_disjoint', 'mixed_competing', 'mixed_settlement']
FAULTS = ['clean_stop', 'torn_payload', 'torn_checkpoint', 'before_commit', 'after_commit', 'quota']
CAP = 2 * 1024 * 1024


def require(condition, reason):
    if not condition:
        raise ValueError(reason)


def digest(b):
    return hashlib.sha256(b).hexdigest()


def read(p, cap=CAP):
    p = Path(p)
    require(p.is_file() and not p.is_symlink() and p.stat().st_size <= cap, 'file type/ceiling')
    return p.read_bytes()


def data(p, cap=CAP):
    return json.loads(read(p, cap))


def load(path, name):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    old=sys.dont_write_bytecode
    try:sys.dont_write_bytecode=True;spec.loader.exec_module(module)
    finally:sys.dont_write_bytecode=old
    return module


def envelope(b,magic=b'HGEMS001'):
    require(24 <= len(b) <= CAP + 24 and b[:8] == magic and int.from_bytes(b[8:16], 'little') == len(b)-24, 'live checkpoint envelope')
    h = 0xcbf29ce484222325
    for x in b[24:]:
        h = ((h ^ x) * 0x100000001b3) & ((1 << 64)-1)
    require(h == int.from_bytes(b[16:24], 'little'), 'live checkpoint checksum')


def boundary(status,profile=PROFILE):
    require(status['profile'] == profile and status['development_only'] is True, 'stream profile/scope')
    require(all(status[k] is False for k in ['scientific_run_launched', 'gate_pass', 'training_admitted', 'evolution_enabled', 'reproductive_fairness_accepted']), 'stream research boundary')


def inspect_segment(directory, cut=None, seen=(), funded_config=None, retained_policy=None, handoff_config=None, descendant_config=None, transfer_environment=None, mutation_stress_policy=None):
    """Return an exact committed chain; pending files and tails remain untouched."""
    p = Path(directory)
    require(p.is_dir() and not p.is_symlink() and len(seen) < 8 and p.resolve() not in seen, 'segment path/cycle/depth')
    require(not any(f.is_symlink() or not f.is_file() for f in p.iterdir()), 'segment evidence file inventory')
    require(sum(f.stat().st_size for f in p.iterdir()) <= 32*1024*1024, 'segment output ceiling')
    header_bytes = read(p/'segment.json', 65536)
    header = json.loads(header_bytes)
    recipe = header['recipe']
    profile=PROFILE if funded_config is None else 'evolving-minds-d0-funded-streaming-v1'
    if retained_policy is not None:
        require(retained_policy=='founder-first-proposal-v1' and funded_config is not None,'retained storage policy')
        profile='evolving-minds-d0-retained-streaming-v1'
    if handoff_config is not None:
        require(funded_config is not None and retained_policy is None,'handoff storage policy conflict')
        profile='evolving-minds-d0-handoff-streaming-v1'
    if descendant_config is not None:
        require(funded_config is not None and retained_policy is None and handoff_config is None,'descendant storage policy conflict')
        profile='evolving-minds-d0-paid-descendant-streaming-v1'
    if transfer_environment is not None:
        require(funded_config is not None and retained_policy is None and handoff_config is None and descendant_config is None,'transfer storage policy conflict')
        profile='evolving-minds-ta1-temporal-streaming-v1'
    if mutation_stress_policy is not None:
        require(mutation_stress_policy=='grammar-source-stress-v1' and transfer_environment is not None,'stress storage policy')
        profile='evolving-minds-ta1-mutation-stress-streaming-v1'
    require(header.get('mutation_stress_policy')==mutation_stress_policy,'stress header policy')
    require(header.get('transfer_environment')==transfer_environment,'temporal header binding')
    require(header.get('descendant_config')==descendant_config,'descendant header configuration')
    require(header.get('handoff_config')==handoff_config,'handoff header config binding')
    require(header.get('retained_policy')==retained_policy,'retained header policy binding')
    if funded_config is None:
        require(header.get('funded_config') is None and header['profile']==profile and header['dynamics_profile']=='evolving-minds-d0-admission-v1' and recipe['case'] in CASES and recipe['rotation']==0 and recipe['reversed'] is False,'pilot header recipe/profile')
    else:
        require(header.get('funded_config')==funded_config and header['profile']==profile and header['dynamics_profile']==('evolving-minds-ta1-mutation-stress-v1' if mutation_stress_policy is not None else 'evolving-minds-ta1-temporal-v1' if transfer_environment is not None else 'evolving-minds-d0-paid-descendant-neighborhood-v1' if descendant_config is not None else 'evolving-minds-d0-paid-handoff-v1' if handoff_config is not None else 'evolving-minds-d0-funded-admission-v1' if retained_policy is None else 'evolving-minds-d0-retained-proposal-v1') and recipe==dict(case='faithful_disjoint',rotation=funded_config['rotation'],reversed=funded_config['reversed']),'funded header parameter/profile binding')

    start = header['start_tick']
    require(isinstance(start, int) and 0 <= start <= 64 and start % 8 == 0, 'segment start')
    require(header['payload_limit'] == (262144 if header['fault'] == 'Quota' else 16*1024*1024), 'declared payload ceiling')
    require(digest(read(p/'rules.toml', 4*CAP)) == header['law_sha256'], 'segment laws binding')
    parent = header['parent']
    chain = []
    if parent is not None:
        require(re.fullmatch(r'commit_\d{4}\.json', parent['commit']) is not None, 'parent commit filename')
        chain = inspect_segment(parent['directory'], parent['commit'], seen+(p.resolve(),), funded_config=funded_config,retained_policy=retained_policy,handoff_config=handoff_config,descendant_config=descendant_config,transfer_environment=transfer_environment,mutation_stress_policy=mutation_stress_policy)
        last = chain[-1]
        require(last['marker_sha256'] == parent['sha256'] and last['marker']['cursor']['tick'] == start and last['header']['recipe'] == recipe, 'parent linkage')
        require(data(p/'initial.json') == last['state']['frame'] and data(p/'initial_lineage.json') == last['state']['native']['lineage'], 'child initial state linkage')
    else:
        require(start == 0, 'missing parent')
    names = sorted(f.name for f in p.iterdir() if f.name.startswith('commit_') and f.name.endswith('.json'))
    require(names and all(re.fullmatch(r'commit_\d{4}\.json', n) for n in names), 'published commit inventory')
    target = cut or names[-1]
    require(target in names, 'selected commit absent')
    names = [n for n in names if n <= target]
    log = read(p/'ticks.jsonl', 16*1024*1024)
    require(len(log) <= header['payload_limit'], 'payload budget')
    previous = None
    commits = []
    for index, name in enumerate(names):
        raw = read(p/name, 65536)
        marker = json.loads(raw)
        c = marker['cursor']
        tick = start+8*index
        require(marker['profile'] == profile and name == f'commit_{tick:04}.json' and c['tick'] == tick and tick <= 64 and c['step'] == 46*tick and c['local_ticks'] == tick-start and c['segment_sha256'] == digest(header_bytes) and c['previous_commit_sha256'] == previous, 'commit cursor/chain')
        offset = c['offset']
        require(isinstance(offset, int) and 0 <= offset <= len(log) and digest(log[:offset]) == c['log_sha256'], 'committed prefix offset/hash')
        prefix = log[:offset]
        require(not prefix or prefix.endswith(b'\n'), 'torn committed record')
        lines = prefix.splitlines()
        require(len(lines) == tick-start and all(lines), 'gap/duplicate/empty bundle')
        bundles = []
        for i, line in enumerate(lines):
            require(len(line) <= CAP, 'tick payload ceiling')
            bundle = json.loads(line)
            t = start+i+1
            require(bundle['profile'] == profile and bundle['tick'] == t and bundle['row']['tick'] == t and bundle['row']['engine_step'] == 46*t and bundle['round']['tick'] == t and len(bundle['entries']) == 46, 'bundle coverage/clocks')
            require(all(e['tick'] == t and e['pass'] == n and e['step'] == (t-1)*46+n+1 for n,e in enumerate(bundle['entries'])), 'native entry clocks')
            bundles.append(bundle)
        require(marker['checkpoint'] == f'checkpoint_{tick:04}.bin' and marker['attestation'] == f'state_{tick:04}.json', 'commit object filenames')
        cp = read(p/marker['checkpoint'], CAP+24)
        state_bytes = read(p/marker['attestation'])
        require(digest(cp) == marker['checkpoint_sha256'] and digest(state_bytes) == marker['attestation_sha256'], 'published state hash')
        envelope(cp, b'HGEMMS01' if mutation_stress_policy is not None else b'HGEMTA01' if transfer_environment is not None else b'HGEMDN01' if descendant_config is not None else b'HGEMHO01' if handoff_config is not None else b'HGEMRP01' if retained_policy is not None else b'HGEMS001' if funded_config is None else b'HGEMFS01')
        state = json.loads(state_bytes)
        native = state['native']
        require(native['profile'] == profile and native['cursor'] == c and native['tick'] == tick and native['admission'] == dict(recipe=recipe, rounds=[], records=[]), 'native live state/cursor/history')
        require(native.get('mutation_stress_policy')==mutation_stress_policy,'stress checkpoint policy')
        require(native.get('transfer_environment')==transfer_environment,'temporal checkpoint binding')
        require(native.get('funded_config')==funded_config,'live checkpoint parameter binding')
        require((native.get('descendant') or {}).get('config')==descendant_config,'descendant checkpoint configuration')
        require(native.get('handoff_config')==handoff_config,'handoff state config binding')
        require(native.get('retained',{}).get('policy')==retained_policy,'retained state policy binding')
        require(not any(k in native for k in ['journal','rows','mutation','lifecycle']) and state['frame']['tick'] == tick and state['frame']['step'] == tick*46 and state['frame']['phase'] is None and not state['frame']['events'], 'live state boundary')
        previous = digest(raw)
        commits.append(dict(marker=marker, state=state, bundles=bundles, marker_sha256=previous, checkpoint_bytes=len(cp)))
    current = dict(directory=p, header=header, commits=commits, **commits[-1])
    if parent is not None:
        a = copy.deepcopy(chain[-1]['state']['native'])
        b = copy.deepcopy(commits[0]['state']['native'])
        a.pop('cursor'); b.pop('cursor')
        require(a == b and commits[0]['state']['frame'] == chain[-1]['state']['frame'], 'child checkpoint continuity')
    chain.append(current)
    return chain


def flatten(chain, last_commit=None):
    bundles = []
    lineage = data(chain[0]['directory']/'initial_lineage.json')
    for index, segment in enumerate(chain):
        selected = last_commit if index == len(chain)-1 and last_commit is not None else segment
        for b in selected['bundles']:
            require(b['tick'] == len(bundles)+1, 'merged tick coverage')
            ids = set()
            for birth in b['lineage_delta']:
                key = str(birth['id'])
                require(key not in ids, 'duplicate lineage delta'); ids.add(key)
                if key in lineage:
                    old = copy.deepcopy(lineage[key]); old['death_tick'] = b['tick']
                    require(birth == old and lineage[key]['death_tick'] is None, 'death delta identity')
                else:
                    require(birth['born_tick'] == b['tick'] and birth['parent'] is not None and str(birth['parent']) in lineage and birth['death_tick'] in [None,b['tick']], 'birth delta ancestry')
                lineage[key] = birth
            bundles.append(b)
    return dict(journal=[e for b in bundles for e in b['entries']], rows=[b['row'] for b in bundles], lineage=lineage, admission=dict(recipe=chain[-1]['header']['recipe'], rounds=[b['round'] for b in bundles], records=[r for b in bundles for r in b['records']]))


def audit_state(chain, commit, evidence):
    n = commit['state']['native']; tick = commit['marker']['cursor']['tick']
    live = {k:v for k,v in evidence['lineage'].items() if v['death_tick'] is None}
    require(n['lineage'] == live and len(live) <= 8, 'live lineage pruning/identity')
    metadata = {}
    for key, birth in sorted(evidence['lineage'].items(), key=lambda kv:int(kv[0])):
        metadata[key] = dict(founder=int(key),depth=0) if birth['parent'] is None else dict(founder=metadata[str(birth['parent'])]['founder'],depth=metadata[str(birth['parent'])]['depth']+1)
    require(n['live'] == dict(members={k:metadata[k] for k in live},maximum_depth=max(v['depth'] for v in metadata.values())), 'live founder/depth')
    ledger = n['ledger']
    require(ledger['initial'] == 384.0 and ledger['births']+2 == ledger['deaths']+len(live), 'live cumulative ledger')
    if tick:
        row = evidence['rows'][-1]
        require(all(ledger[k] == row[k] for k in ['influx','dissipated','harvested','births','deaths','development_paid','development_failures','attempts','successes','failures']), 'live ledger row binding')
    else:
        require(all(ledger[k] == 0 for k in ledger if k != 'initial'), 'initial ledger')


def physical_audit(chain, commit, modules, base, funded_config=None, retained_policy=None, handoff_config=None, descendant_config=None, transfer_environment=None, mutation_stress_policy=None):
    a,c,d,s = modules
    evidence = flatten(chain, commit)
    audit_state(chain, commit, evidence)
    tick = commit['marker']['cursor']['tick']
    with tempfile.TemporaryDirectory(prefix='evolving_minds_derived_audit_') as tmp:
        p = Path(tmp)
        objects = dict(recipe=chain[-1]['header']['recipe'], schedule=data(chain[0]['directory']/'schedule.json'), initial=data(chain[0]['directory']/'initial.json'), final=commit['state']['frame'], rows=evidence['rows'], lineage=evidence['lineage'], admission_state=evidence['admission'])
        objects['manifest'] = dict(state='Completed',seed=17 if funded_config is None else funded_config['seed'],max_steps=tick*46,final_step=tick*46)
        objects['run_status'] = dict(profile=a.PROFILE if funded_config is None else a.FUNDED_PROFILE,development_only=True,engineering_execution_complete=True,scientific_run_launched=False,gate_pass=False,training_admitted=False,evolution_enabled=False,reproductive_fairness_accepted=False,recipe=objects['recipe'],requests=len(evidence['admission']['records']),final_row=evidence['rows'][-1] if tick else None)
        if funded_config is not None:objects['funded_config']=funded_config
        for name,x in objects.items(): (p/(name+'.json')).write_text(json.dumps(x))
        (p/'journal.jsonl').write_text(''.join(json.dumps(e,separators=(',',':'))+'\n' for e in evidence['journal']))
        (p/'rules.toml').write_bytes(read(chain[0]['directory']/'rules.toml',4*CAP))
        kwargs=dict(checkpoints=False,final_tick=tick)
        if funded_config is not None:kwargs['funded_config']=funded_config
        if retained_policy is not None:kwargs['retained_policy']=retained_policy
        if handoff_config is not None:kwargs['handoff_config']=handoff_config
        if descendant_config is not None:kwargs['descendant_config']=descendant_config
        if transfer_environment is not None:kwargs['transfer_environment']=transfer_environment
        if mutation_stress_policy is not None:kwargs['mutation_stress_policy']=mutation_stress_policy
        _, result = a.verify_cell(p,c,d,s,base,objects['recipe'],**kwargs)
    if retained_policy is not None:require(commit['state']['native'].get('retained')==result['retained'],'independently regenerated retained checkpoint samples')
    if handoff_config is not None:require(commit['state']['native'].get('handoff_config')==result['handoff_config'],'independently regenerated handoff config')
    if descendant_config is not None:require(commit['state']['native'].get('descendant')==result['descendant'],'independently regenerated first descendant targets')
    return evidence,result


def verify(root, full_history=None, extra_segment=None):
    started = time.monotonic(); root = Path(root)
    require(root.is_dir() and not root.is_symlink() and not any(p.is_symlink() for p in root.rglob('*')), 'pilot path')
    require(sum(p.stat().st_size for p in root.rglob('*') if p.is_file()) <= 256*1024*1024, 'pilot output ceiling')
    sources = data(root/'sources.json',16*CAP)
    bindings = [('checker.py','scripts/verify_evolving_minds_streaming.py'),('admission_reference.py','scripts/verify_evolving_minds_admission.py'),('replay.py','scripts/verify_evolving_minds.py'),('d0_reference.py','scripts/verify_evolving_minds_d0.py'),('schedule_reference.py','scripts/verify_evolving_minds_choice.py'),('base_rules.toml','rules/evolving_minds/admission_v1.toml'),('contract.md','experiments/evolving_minds/contracts/d0_streaming_v1.md')]
    for file,source in bindings:
        require(read(root/file,4*CAP).decode() == sources[source], 'frozen source binding')
    modules = tuple(load(root/f,n) for f,n in [('admission_reference.py','stream_admission'),('replay.py','stream_replay'),('d0_reference.py','stream_d0'),('schedule_reference.py','stream_schedule')])
    base = read(root/'base_rules.toml',4*CAP)
    require(digest(base) == modules[0].LAW_SHA, 'fixed native law')
    expected = [f'baseline_{c}' for c in CASES]+[f'{p}_{f}' for f in FAULTS for p in ['fault','recovered']]
    require(sorted(p.name for p in root.iterdir() if p.is_dir()) == sorted(expected), 'fixed pilot denominator')
    summary = data(root/'summary.json'); reports=[]; results=[]; commit_count=0
    for name in expected+([str(Path(extra_segment).resolve())] if extra_segment else []):
        directory = Path(name) if Path(name).is_absolute() else root/name
        chain = inspect_segment(directory)
        segment = chain[-1]
        report = data(directory/'segment_report.json'); status = data(directory/'run_status.json'); m = data(directory/'manifest.json')
        boundary(status)
        tick = segment['marker']['cursor']['tick']; faulted = name.startswith('fault_')
        require(tick == (16 if name == 'fault_after_commit' else 8 if faulted else 64), 'fixed pilot recovery cut')
        require(report['profile'] == PROFILE and report['recipe'] == segment['header']['recipe'] and report['fault'] == segment['header']['fault'] and report['start_tick'] == segment['header']['start_tick'] and report['committed_final_tick'] == tick and report['native_final_tick'] >= tick and m['seed'] == 17 and m['max_steps'] == 2944 and m['final_step'] == tick*46 and m['state'] == report['state'] == ('Interrupted' if faulted else 'Completed') and status['engineering_execution_complete'] is (not faulted), 'segment manifest/report scope')
        resources = report['resources']
        require(resources['drained_history_empty'] is True and resources['retained_entries_peak'] == 46 and resources['retained_rows_peak'] == resources['retained_rounds_peak'] == 1 and resources['retained_requests_peak'] <= 8 and resources['retained_lineage_peak'] <= 16 and resources['payload_bytes'] == (directory/'ticks.jsonl').stat().st_size and resources['advance_seconds'] <= 120 and all(resources[k] >= 0 for k in ['io_seconds','checkpoint_seconds','resume_seconds']), 'measured retention/resource bounds')
        require(resources['checkpoints'] == [[c['marker']['cursor']['tick'],c['checkpoint_bytes']] for c in segment['commits']], 'checkpoint resource inventory')
        if faulted:
            expected_fault = dict(clean_stop='CleanStop',torn_payload='TornPayload',torn_checkpoint='TornCheckpoint',before_commit='BeforeCommit',after_commit='AfterCommit',quota='Quota')[name[6:]]
            require(segment['header']['fault'] == expected_fault and segment['header']['parent'] is None and report['stop_reason'] is not None, 'fixed fault mechanism')
            native_tick = report['native_final_tick']
            require(native_tick == dict(CleanStop=8,TornPayload=9,TornCheckpoint=16,BeforeCommit=16,AfterCommit=16).get(expected_fault,native_tick), 'injected write-stage clock')
            if expected_fault == 'TornPayload':
                require(not (directory/'ticks.jsonl').read_bytes().endswith(b'\n') and len((directory/'ticks.jsonl').read_bytes()) > segment['marker']['cursor']['offset'], 'preserved torn payload')
            if expected_fault == 'TornCheckpoint':
                require((directory/'checkpoint_0016.bin.pending').is_file() and not (directory/'commit_0016.json').exists(), 'preserved torn checkpoint')
            if expected_fault == 'BeforeCommit':
                require((directory/'checkpoint_0016.bin').is_file() and (directory/'state_0016.json').is_file() and (directory/'commit_0016.json.pending').is_file() and not (directory/'commit_0016.json').exists(), 'preserved orphan state')
            if expected_fault == 'Quota':
                require(native_tick > tick and report['stop_reason'] == 'declared segment payload quota' and resources['payload_bytes'] <= 262144, 'quota censoring scope')
        else:
            require(segment['header']['fault'] == 'Normal' and report['stop_reason'] is None, 'normal segment scope')
        if name == 'fault_after_commit': require(data(directory/'manifest_at_crash.json')['final_step'] == 8*46, 'after commit stale manifest')
        if name.startswith('recovered_'):
            require(segment['header']['parent']['directory'] == str((root/name.replace('recovered_','fault_')).resolve()), 'fixed recovery source')
        # Reconstruct each published native state independently, including zero/start snapshots.
        for commit in segment['commits']:
            evidence,result = physical_audit(chain,commit,modules,base); commit_count += 1
        if full_history is not None:
            old = Path(full_history)/(segment['header']['recipe']['case']+'_r0_forward')
            for key,file in [('journal','journal.jsonl'),('rows','rows.json'),('lineage','lineage.json'),('admission','admission_state.json')]:
                original = [json.loads(l) for l in (old/file).read_text().splitlines()] if key == 'journal' else data(old/file,16*CAP)
                if tick == 64: require(evidence[key] == original, 'exact preserved full-history '+key)
            if tick == 64: require(segment['state']['frame'] == data(old/'final.json'), 'exact preserved final graph')
        results.append(dict(segment=str(directory),committed_tick=tick,**result)); reports.append(report)
    require(summary == reports[:15], 'fixed summary inventory')
    boundary(data(root/'run_status.json'))
    m=data(root/'manifest.json'); require(m['state']=='Completed' and m['max_steps']==m['final_step']==9*2944,'pilot completion manifest')
    return dict(profile=PROFILE,development_only=True,scientific_run_launched=False,gate_pass=False,training_admitted=False,evolution_enabled=False,reproductive_fairness_accepted=False,segments_verified=15,complete_chains_verified=9,interrupted_prefixes_verified=6,published_states_reconstructed=commit_count,full_history_compared=full_history is not None,extra_segment_verified=extra_segment is not None,audit_seconds=time.monotonic()-started,results=results)

if __name__ == '__main__':
    parser=argparse.ArgumentParser(); parser.add_argument('root'); parser.add_argument('--full-history'); parser.add_argument('--extra-segment'); args=parser.parse_args()
    print(json.dumps(verify(args.root,args.full_history,args.extra_segment),sort_keys=True))
