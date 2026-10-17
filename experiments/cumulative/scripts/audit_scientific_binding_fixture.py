"""Zero-native audit of binding projections, refusal receipts and immutable sources."""
import json
from pathlib import Path
import sys
from compact_evidence import source_inventory,require,sha
from scientific_launch_binding import parse,canonical,validate,CONTRACT
from scientific_binding_cases import cases
from audit_scientific_controller import audit_ledger


def audit(out):
    load=lambda n:parse((out/n).read_bytes())
    source_inventory(out/'parent',CONTRACT['parent']['seal_sha256'])
    pins=load('source_sha256.json');require(all(sha((out/'source_snapshots'/n).read_bytes())==h for n,h in pins.items()),'immutable source snapshots')
    binary=sha((out/'cumulative-scientific-launch').read_bytes());a=load('authorization_preview.json')
    require(a['mode']=='preview' and a['approved'] is False and a['binary_sha256']==binary and a['source_sha256']==sha((out/'source_sha256.json').read_bytes()),'inert preview binds executable and sources')
    expected=cases(a);valid=invalid=refused=0
    require({p.name for p in (out/'bindings').iterdir()}=={c['name']+'.json' for c in expected},'complete frozen input denominator')
    for c in expected:
        raw=(out/'bindings'/(c['name']+'.json')).read_bytes();require(raw==c['raw'],'exact frozen case')
        receipt=load('commands/inspect_'+c['name']+'.json')
        try:route=validate(parse(raw),binary)
        except (ValueError,KeyError,TypeError):route=None
        require((route is not None)==c['valid'],'independent binding verdict')
        if c['valid']:
            require(receipt['exit_code']==0 and parse(receipt['stdout'].encode())==dict(binding_valid=True,execution_performed=False,native_rewrites=0,route=route),'exact Rust/Python native projection')
            rejection=load('commands/execute_'+c['name']+'.json');require(rejection['exit_code']==1,'preview execution rejected')
            error=parse(rejection['stderr'].encode());require(error['native_rewrites']==0 and 'preview cannot execute' in error['error'],'zero-native preview boundary')
            valid+=1;refused+=1
        else:
            require(receipt['exit_code']==1 and parse(receipt['stderr'].encode())['native_rewrites']==0,'malformed binding rejected before native work');invalid+=1
    require(not (out/'forbidden_journal').exists() and not (out/'forbidden_campaign').exists(),'native and whole-campaign launch remained inert')
    refusal=load('commands/driver_refusal.json');require(refusal['exit_code']!=0 and 'preview cannot execute' in refusal['stderr'],'top-level launch refuses preview before creating campaign')
    ledger=audit_ledger(out/'preview_controller')
    require(ledger['closed'] and ledger['accounted_rewrites_lower']==0 and ledger['accounted_rewrites_upper']==250 and ledger['world_statuses']==['unknown']+['not_started']*187,'synthetic pending reservation and full fixed panel retained')
    return dict(engineering_scientific_binding_pass=True,valid_inspections=valid,malformed_bindings_rejected=invalid,preview_execution_refusals=refused,world_identities=188,native_rewrites=0,scientific_worlds_started=0,source_files=len(pins),readiness_gate_complete=False,scientific_acceptance=False,evolution_demonstrated=False,campaign_launch_authorized=False)


if __name__=='__main__':print(json.dumps(audit(Path(sys.argv[1]).resolve()),indent=2,sort_keys=True))
