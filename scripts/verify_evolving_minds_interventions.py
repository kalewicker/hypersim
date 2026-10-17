#!/usr/bin/env python3
"""Read-only verifier for fixed two-phase causal-control forks."""
import argparse
import copy
import hashlib
import importlib.util
import json
import sys
from pathlib import Path
import tomllib


def verify(directory):
    directory = Path(directory)
    if not directory.is_dir() or any(p.is_symlink() for p in directory.rglob('*')):
        raise ValueError('invalid evidence directory')
    spec = importlib.util.spec_from_file_location('frozen_baseline', directory / 'baseline/checker.py')
    c = importlib.util.module_from_spec(spec)
    previous = sys.dont_write_bytecode
    try:
        sys.dont_write_bytecode = True
        spec.loader.exec_module(c)
    finally:
        sys.dont_write_bytecode = previous
    baseline = c.verify(directory / 'baseline')
    require = c.require
    require(baseline['frames_replayed'] == 18, 'fixed source horizon')
    source = json.loads((directory / 'baseline/trace.jsonl').read_text().splitlines()[-1])
    source_nodes, source_edges = c.graphs(source)
    roots = [h for h,n in source_nodes.items() if n['label'] == c.ROOT and c.value(n,c.BID,'Int') == 1]
    require(len(roots) == 1, 'founder semantic identity')
    root = roots[0]
    parts = {}
    for label in [c.SENSOR,c.CONNECTION,c.ACTUATOR]:
        ids = [h for h,n in source_nodes.items() if n['label'] == label and c.value(n,c.OWNER,'Int') == 1]
        require(len(ids) == 1, 'ambiguous semantic target')
        parts[label] = ids[0]
    require(c.value(source_nodes[parts[c.SENSOR]],c.SIGNAL,'Bool') is True,'positive sensor')
    rules = {r['name']:r for r in tomllib.loads((directory / 'baseline/rules.toml').read_text())['rules']}
    probes = json.loads((directory / 'probes.json').read_text())
    require([p['arm'] for p in probes] == ['sham','signal_clamp','foreign_owner','owner_rescue'],'arm inventory')
    results = {}
    for probe in probes:
        arm = probe['arm']
        require(probe['target_birth_id'] == 1,'target birth identity')
        for key,label in [('sensor',c.SENSOR),('connection',c.CONNECTION),('actuator',c.ACTUATOR)]:
            require(c.handle(probe[key]) == parts[label],'semantic target mismatch')
        nodes = copy.deepcopy(source_nodes); edges = copy.deepcopy(source_edges)
        edits = [[probe['actuator'],c.SIGNAL,False]]
        if arm == 'signal_clamp': edits.append([probe['sensor'],c.SIGNAL,False])
        if arm in ['foreign_owner','owner_rescue']: edits.append([probe['connection'],c.OWNER,999])
        if arm == 'owner_rescue': edits.append([probe['connection'],c.OWNER,1])
        require(probe['edits'] == edits,'intervention edit provenance')
        c.set_property(nodes[parts[c.ACTUATOR]],c.SIGNAL,{'Bool':False})
        if arm == 'signal_clamp': c.set_property(nodes[parts[c.SENSOR]],c.SIGNAL,{'Bool':False})
        if arm == 'foreign_owner': c.set_property(nodes[parts[c.CONNECTION]],c.OWNER,{'Int':999})
        require(probe['lifecycle_valid'] is (arm != 'foreign_owner'),'malformed diagnostic admission')
        frames = probe['frames']
        require(len(frames) == 3,'probe phase coverage')
        require(c.graphs(frames[0]) == (nodes,edges),'unmatched preparation edits')
        require(frames[0]['step'] == 18 and frames[0]['tick'] == 2 and frames[0]['phase'] is None and not frames[0]['events'],'probe source clock')
        initial = c.total(nodes)
        root_initial = c.value(nodes[root],c.ENERGY,'Float')
        for offset,phase in enumerate(['Signal','Action'],1):
            f = frames[offset]
            require(f['step'] == 18+offset and f['tick'] == 2 and f['phase'] == phase,'probe phase clock')
            events = f['events']
            expected_count = (0 if arm == 'foreign_owner' else 1) if phase == 'Signal' else (1 if arm in ['sham','owner_rescue'] else 0)
            require(len(events) == expected_count,'native guard coverage')
            require(f['stats']['accepted_rewrites'] == expected_count and f['stats']['property_effects_skipped'] == 0,'native effect counters')
            expected_effects = sum(len(rules[e['rule']]['rhs'].get('property_effects',[])) for e in events)
            require(f['stats']['property_effects_applied'] == expected_effects,'native applied effect counters')
            require(0 <= f['match_work'] <= 100000,'probe match work ceiling')
            for event in events:
                require(event['rule'].startswith(phase.lower()+'_'),'foreign phase event')
                c.apply(nodes,edges,event,rules[event['rule']],18+offset)
            require(c.graphs(f) == (nodes,edges),'independent probe rewrite mismatch')
            require(c.total(nodes) == initial,'probe energy conservation')
        acquired = c.value(nodes[root],c.ENERGY,'Float') - root_initial
        expected = 8 if arm in ['sham','owner_rescue'] else 0
        require(acquired == expected == probe['acquired'],'calibration outcome')
        results[arm] = {'acquired':acquired,'lifecycle_valid':probe['lifecycle_valid']}
    require(probes[0]['frames'] == probes[3]['frames'],'owner rescue must match sham exactly')
    status = json.loads((directory / 'run_status.json').read_text())
    require(status['profile'] == 'evolving-minds-causal-controls-v1' and status['engineering_execution_complete'] is True and status['development_only'] is True,'execution status')
    require(all(status[k] is False for k in ['scientific_run_launched','gate_pass','training_admitted','evolution_enabled']),'scientific boundary')
    manifest = json.loads((directory / 'manifest.json').read_text())
    require(manifest['state'] == 'Completed' and manifest['final_step'] == manifest['max_steps'] == 8,'probe manifest')
    return {'audit_pass':True,'development_only':True,'scientific_run_launched':False,'gate_pass':False,'training_admitted':False,'arms':results,
            'files_sha256':{str(p.relative_to(directory)):hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(directory.rglob('*')) if p.is_file() and '__pycache__' not in p.parts}}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('directory',type=Path)
    print(json.dumps(verify(parser.parse_args().directory),indent=2))
