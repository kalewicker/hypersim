#!/usr/bin/env python3
"""Validate the prospective CC-1 contract and source namespace, without simulating."""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import re
import sys
import tomllib
sys.dont_write_bytecode=True


def require(value, reason):
    if not value:raise ValueError(reason)


def validate(workspace):
    workspace=Path(workspace).resolve()
    relative='experiments/constructive_chemistry/contracts/cc1_atomic_fixture_v1.json'
    contract_path=workspace/relative
    c=json.loads(contract_path.read_text())
    require(c['status']=='prospectively_frozen_not_executed','prospective scope')
    require(c['seeds']==[] and c['independent_scientific_worlds']==0,'no scientific seed/world campaign')
    for flag in ['scientific_acceptance','scientific_run_launched','training_admitted','evolution_enabled','sustained_claim','reproductive_fairness_accepted','power_adequacy_established','ready_for_scientific_launch','gate_pass']:
        require(c.get(flag) is False,'explicit false boundary '+flag)
    cases=c['cases'];ids=[x['id'] for x in cases]
    require(len(cases)==41 and len(set(ids))==41,'fixed41 unique cases')
    require(c['denominator']==dict(cases=41,worker_counts=[1,4],worker_variants_per_case=2,total_case_variants=82,worker_variants_are_replicates=False,all_cases_retained=True,no_outcome_dependent_replacement=True),'fixed82 worker variants')
    schema=c['schema'];energy=c['energy'];caps=c['caps']
    require(schema['adjacency']==[[0,1],[1,2]] and schema['site_indices']==[0,1,2] and schema['site_semantic_ids']==[1,2,3],'fixed nonperiodic substrate')
    require(schema['site_capacity']==8 and schema['material_valence']==dict(A=1,B=1,C=1,D=1,F=0),'physical capacities')
    require(energy['activated_fuel_quanta']==energy['bond_quanta']+energy['bond_formation_heat']==4,'bond/fuel/heat balance')
    require(energy['physical_time'] is None and c['atomicity']['rng'] is None,'no implied reaction timing')
    require(c['atomicity']['rejection_preserves_all_fingerprint_fields'] and c['acceptance']['rejected_attempts_advance_physics'] is False,'failure atomicity')
    properties=list(schema['properties'].values());labels=[int(v,16) for v in schema['labels'].values()]
    require(len(set(properties))==7 and set(properties)==set(range(243001,243008)),'property reservation')
    require(len(set(labels))==7 and all(0xE2000000<=v<0xE2010000 for v in labels),'label reservation')
    fixtures=c['fixtures'];resolved={}
    def integer(value):
        require(type(value) is int or (type(value) is str and re.fullmatch(r'0|[1-9][0-9]*',value)), 'exact nonnegative integer encoding')
        return int(value)
    def resolve(name, trail=()):
        require(name in fixtures and name not in trail,'fixture existence/acyclic inheritance')
        if name in resolved:return resolved[name]
        raw=fixtures[name]
        base=json.loads(json.dumps(resolve(raw['base'],trail+(name,)))) if 'base' in raw else dict(beads=[],initial_bonds=[],initial_bond_ids=[])
        for key,value in raw.items():
            if key!='base':base[key]=json.loads(json.dumps(value))
        omitted=set(base.pop('omit_beads',[]));base['beads']=[x for x in base['beads'] if x['id'] not in omitted]
        for bead in base['beads']:
            if str(bead['id']) in base.get('set_activation',{}):bead['activated']=base['set_activation'][str(bead['id'])]
            if str(bead['id']) in base.get('set_site',{}):bead['site']=base['set_site'][str(bead['id'])]
        base.pop('set_activation',None);base.pop('set_site',None)
        resolved[name]=base;return base
    summaries={}
    for name in fixtures:
        f=resolve(name);beads=f['beads'];by_id={x['id']:x for x in beads}
        require(len(by_id)==len(beads) and all(type(i) is int and 0<i<2**63 for i in by_id) and set(by_id).isdisjoint({1,2,3}),'fixture semantic identity')
        require(all(type(x['activated']) is bool and x['species'] in schema['species'] and (not x['activated'] or x['species']=='F') and x['site'] in [0,1,2] for x in beads),'fixture bead schema')
        require(all(n<=8 for n in Counter(x['site'] for x in beads).values()),'base fixture occupancy')
        bonds=f.get('initial_bonds',[]);bond_ids=f.get('initial_bond_ids',[])
        require(len(bonds)==len(bond_ids) and len(set(bond_ids))==len(bond_ids) and set(bond_ids).isdisjoint(set(by_id)|{1,2,3}),'initial bond identity')
        endpoints=[]
        for a,b in bonds:
            require(a in by_id and b in by_id and by_id[a]['species']=='A' and by_id[b]['species']=='B' and by_id[a]['site']==by_id[b]['site'],'bond role/placement')
            endpoints.extend([a,b])
        require(len(set(endpoints))==len(endpoints),'initial valence')
        heat=integer(f.get('initial_heat',0));active=integer(f.get('reservoir_active_fuel',0));inactive=integer(f.get('reservoir_inactive_fuel',0))
        require(all(0<=x<2**63 for x in [heat,active,inactive]),'valid diagnostic ledger fields')
        revision=integer(f.get('initial_revision',0));step=integer(f.get('initial_graph_step',0))
        require(revision==step and 0<=revision<2**64,'revision/graph step')
        frontier=integer(f.get('semantic_id_frontier',max([1,2,3]+list(by_id)+bond_ids)+1))
        require(max([1,2,3]+list(by_id)+bond_ids)<frontier<2**63,'semantic frontier')
        world_energy=4*sum(x['activated'] for x in beads)+2*len(bonds)
        total=world_energy+4*active+heat
        require(total<2**127,'derived energy baseline fits checked i128')
        nodes=3+len(beads)+len(bonds);edges=2+len(beads)+2*len(bonds)
        require(len(beads)<=caps['physical_beads'] and nodes<=caps['graph_nodes'] and edges<=caps['graph_edges'],'base fixture engineering limits')
        summaries[name]=dict(material_vector=[sum(x['species']==sp for x in beads) for sp in schema['species']],world_energy=str(world_energy),reservoir_energy=str(4*active),heat=str(heat),total_energy=str(total),nodes=nodes,edges=edges)
    layers=Counter()
    for case in cases:
        require(case['fixture'] in fixtures and bool(case['expected']),'case initial state and declared outcome')
        layers[case['layer']]+=1
        require(case['layer'] in ['commit','prepare','generic_stage','commit_fault','invariant_check','initial_admission'],'case layer')
        require(case['operation'] and (case.get('order') is None or len(case['order'])<=caps['action_offers_per_case']),'case proposal budget')
    require(layers['commit_fault']==5 and layers['invariant_check']==5 and layers['initial_admission']==7,'negative/fault coverage')
    production=[p for folder in ['crates','rules','configs'] for p in (workspace/folder).rglob('*') if p.is_file() and p.suffix in ['.rs','.toml','.json'] and 'target' not in p.parts]
    token=re.compile(r'\b(?:0x[0-9a-fA-F_]+|[0-9][0-9_]*)\b')
    collisions=[]
    for path in production:
        for t in token.findall(path.read_text()):
            try:v=int(t.replace('_',''),16 if t.lower().startswith('0x') else 10)
            except ValueError:continue
            if v in properties or v in labels:collisions.append(dict(file=str(path.relative_to(workspace)),token=t))
    require(not collisions,'production namespace collision '+str(collisions[:4]))
    manifests={p:tomllib.loads((workspace/p).read_text()) for p in ['crates/hg-engine/Cargo.toml','crates/hg-sim-core/Cargo.toml']}
    feature_status=dict(checked_transitions_present='checked-transitions' in manifests['crates/hg-engine/Cargo.toml'].get('features',{}),constructive_chemistry_present='constructive-chemistry' in manifests['crates/hg-sim-core/Cargo.toml'].get('features',{}))
    return dict(profile='constructive-chemistry-cc0-validation-v1',validation_pass=True,cc0_specification_complete=True,cc1_execution_verified=False,runtime_executed_by_validator=False,contract_executed=False,native_advances=0,scientific_acceptance=False,scientific_run_launched=False,training_admitted=False,evolution_enabled=False,sustained_claim=False,reproductive_fairness_accepted=False,power_adequacy_established=False,ready_for_scientific_launch=False,gate_pass=False,cases=41,worker_variants=82,fixture_count=len(fixtures),case_layers=dict(layers),fixture_initial_accounting=summaries,production_files_scanned=len(production),namespace_collisions=collisions,feature_status=feature_status,contract_sha256=hashlib.sha256(contract_path.read_bytes()).hexdigest())


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--workspace',default=str(Path(__file__).resolve().parents[1]))
    args=parser.parse_args();print(json.dumps(validate(args.workspace),indent=2,sort_keys=True))
