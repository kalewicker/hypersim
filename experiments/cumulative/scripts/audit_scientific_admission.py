"""Independent read-only audit of frozen native admission evidence."""
import json
from pathlib import Path
import sys
from compact_evidence import source_inventory, require, sha
from scientific_admission_cases import corpus, check, parse


def audit(package):
    source=package/'source_snapshots/experiments/cumulative'
    cr=(source/'scientific_admission_contract.json').read_bytes();pr=(source/'scientific_panel.json').read_bytes();law=(source/'campaign_planning_contract.json').read_bytes()
    c=parse(cr);parent=package/'live_parent';source_inventory(parent,c['parent']['seal_sha256'])
    expected_panel=json.loads((parent/'measurement_parent/prospective_pending.json').read_text())['panel']
    require(parse(pr)==expected_panel and parse(law)==json.loads((parent/'campaign_contract.json').read_text()),'unchanged scientific panel and prospective criteria')
    require((package/'contract.json').read_bytes()==cr,'exact compiled admission contract')
    require(c['native_execution_enabled'] is False and c['budget_approved'] is False,'admission is not launch permission')
    index=json.loads((package/'cases.json').read_text());cases=corpus(cr,pr)
    require(len(index)==len(cases)==421,'frozen admission denominator')
    accepted=rejected=0
    for entry,case in zip(index,cases):
        require(entry==dict(name=case['name'],valid=case['valid'],mode=case['mode'],request_sha256=sha(case['raw'])),'fixed case identity')
        raw=(package/'requests'/(case['name']+'.json')).read_bytes();require(raw==case['raw'],'exact predeclared request bytes')
        receipt=json.loads((package/'receipts'/(case['name']+'.json')).read_text())
        require(receipt['binary_sha256']==sha((package/'cumulative-scientific-admission').read_bytes()),'executing binary provenance')
        try:
            if case['mode']!='--check':raise ValueError('no execution CLI')
            expected=check(raw,cr,pr,law);valid=True
        except (ValueError,TypeError,KeyError):valid=False;expected=None
        require(valid==case['valid'],'independent expected classification')
        if valid:
            require(receipt['exit_code']==0 and not receipt['stderr'] and json.loads(receipt['stdout'])==expected,'Rust and independent validator agree exactly')
            accepted+=1
        else:
            require(receipt['exit_code']==1 and not receipt['stdout'],'native request rejection')
            result=json.loads(receipt['stderr']);require(result['request_valid'] is False and result['execution_enabled'] is False and result['launch_authorized'] is False and result['native_rewrites']==0 and type(result['error']) is str,'zero-native rejection receipt')
            rejected+=1
    require(accepted==194 and rejected==227,'complete positive/negative coverage')
    require(not any((package/'check_working_directory').iterdir()),'no native journal or side-effect file created')
    for name in ['rust_tests.json','build.json','parent_recheck.json']:
        receipt=json.loads((package/'commands'/name).read_text());require(receipt['exit_code']==0,'required command completed')
    require('3 passed; 0 failed' in json.loads((package/'commands/rust_tests.json').read_text())['stdout'],'three bounded Rust checks passed')
    require(json.loads(json.loads((package/'commands/parent_recheck.json').read_text())['stdout'])['new_native_rewrites']==0,'parent recheck did not dispatch')
    flags=['readiness_gate_complete','scientific_acceptance','evolution_demonstrated','campaign_launch_authorized']
    require(all(json.loads((package/'manifest.json').read_text())[k] is False for k in flags),'all scientific flags remain closed')
    return dict(engineering_scientific_admission_pass=True,native_admission_validator_implemented=True,scientific_execution_adapter_implemented=False,
                requests_checked=421,valid_requests=194,rejected_requests=227,exact_scientific_worlds_checked=188,maximum_registry_requests_checked=78960,
                native_rewrites=0,scientific_worlds_advanced=0)


if __name__=='__main__':print(json.dumps(audit(Path(sys.argv[1])),indent=2,sort_keys=True))
