#!/usr/bin/env python3
"""Authenticate V24 inputs, observation admission and preserved prior evidence."""
import argparse,json,pathlib,hashlib,tarfile,datetime
from audit_selfmod_birth import load,sha,require

def files(root,exclude=()):return {str(f.relative_to(root)):sha(f) for f in sorted(root.rglob('*')) if f.is_file() and '__pycache__' not in f.parts and f.name not in exclude}
def main():
 p=argparse.ArgumentParser();p.add_argument('output',type=pathlib.Path);out=p.parse_args().output.resolve();base=out.parent.parent
 seal=load(out/'execution_seal.json');prov=load(out/'instrument_provenance.json');contract=load(out/'contract.json');manifest=load(out/'manifest.json');native=load(out/'native_parity.json');audit=load(out/'audit.json');fixture=load(out/'engineering_preflight.json')
 for file,digest in seal['files_sha256'].items():require(sha(out/file)==digest,'Pre-execution frozen artifact '+file)
 require(sha(out/'source.tar.gz')==prov['source_archive_sha256'],'Source archive identity')
 with tarfile.open(out/'source.tar.gz') as tar:
  require(set(tar.getnames())==set(prov['sources_sha256']),'Source inventory')
  for file,digest in prov['sources_sha256'].items():
   require(hashlib.sha256(tar.extractfile(file).read()).hexdigest()==digest==sha(out/'frozen_source'/file)==sha(base/file),'Frozen and current source '+file)
 require(sha(out/'selfmod-conflict-attribution')==prov['observer_binary_sha256']==native['observer_binary_sha256'],'Observer binary')
 require(sha(out/'reference-native')==prov['reference_binary_sha256']==native['reference_binary_sha256'],'Archived reference binary')
 require(native['runner_sha256']==sha(out/'run_selfmod_conflict_attribution.py') and audit['auditor_sha256']==sha(out/'audit_selfmod_conflict_attribution.py'),'Runner/auditor identity')
 require(native['passed'] and audit['passed'] and audit['keyed_rng_reconstructed_independently'] and audit['rows_audited']==4000,'Observation audit')
 require(native['target_advancing_steps']==8000<=contract['max_target_advancing_steps'] and fixture['fixture_advancing_steps']==256<=contract['max_engineering_fixture_advancing_steps'],'Engineering bounds')
 require(all(x['new_scientific_advancing_steps']==0 for x in [contract,manifest,native,audit]),'Zero science')
 require(load(out/'status.json')==dict(state='complete',targets_completed=4,engineering_target_advancing_steps=8000,new_scientific_advancing_steps=0),'All fixed targets complete')
 require([r['name'] for r in audit['records']]==[e['name'] for e in manifest['targets']],'Fixed order/denominator')
 require([(r['archive'],r['seed'],r['offset'],r['steps'],r['arm']) for r in manifest['targets']]==[(c['archive'],c['seed'],c['offset'],c['steps'],arm) for c in contract['targets'] for arm in contract['arms']],'Contract target order and denominator')
 corruptions=0
 for entry,record in zip(manifest['targets'],audit['records']):
  folder=out/entry['name'];require(record['passed'] and record['rows_audited']==entry['steps'],'Full target')
  archived=out.parent/entry['archive'];branch=archived/f"seed_{entry['seed']}/{entry['arm']}";offset=entry['offset'];end=offset+entry['steps']
  for copied,original_file in [('initial.json',f'checkpoint_{offset}.json'),('initial_allocator.json',f'checkpoint_{offset}_allocator.json'),('expected_final.json',f'checkpoint_{end}.json'),('expected_final_allocator.json',f'checkpoint_{end}_allocator.json')]:require(sha(folder/'input'/copied)==sha(branch/original_file),'Named archive checkpoint identity')
  original_birth=archived/f"seed_{entry['seed']}/third_prefix/{entry['arm']}_birth.json" if entry['seed']==541 else archived/f"inputs/seed_{entry['seed']}/{entry['arm']}/birth.json";require(sha(original_birth)==sha(folder/'birth.json'),'Named paid third-child identity')
  expected_hash=hashlib.sha256();row_count=0
  for line in (branch/'rows.jsonl').open('rb'):
   step=json.loads(line)['continuation_step']
   if offset<step<=end:expected_hash.update(line);row_count+=1
   if step>=end:break
  require(row_count==entry['steps'] and expected_hash.hexdigest()==sha(folder/'input/expected_rows.jsonl'),'Exact named archive row slice')
  for file,digest in record['receipts_sha256'].items():require(sha(folder/file)==digest,'Audited target receipt '+entry['name']+'/'+file)
  for file,digest in native['exact_observed_native_receipts'].items():
   if file.startswith(entry['name']+'/'):
    name=file.split('/',1)[1];require(sha(folder/'observed'/name)==digest==sha(folder/'control'/name),'Observed/reference parity '+file)
  require(sum(record['exclusive_rejection_category_sets'].values())==record['metrics']['rejected'],'Partitioned rejection counts')
  require(len(record['adversarial_corruptions_rejected'])>=14,'Meaningful adversarial audit');corruptions+=len(record['adversarial_corruptions_rejected'])
  if entry['arm']=='faithful':require(record['first_extinction']==(31496 if entry['seed']==541 else 16605) and record['final_focal_copies']==0,'Known loss preserved')
  else:require(record['first_extinction'] is None and record['final_focal_copies']>0,'Known restored survival preserved')
 prior={}
 for tag,expected in load(out/'input_provenance.json')['archives_sha256'].items():
  root=out.parent/tag;actual={str(f.relative_to(root)):sha(f) for f in sorted(root.rglob('*')) if f.is_file()};require(actual==expected,'Original archive preservation '+tag);prior[tag]=len(expected)
 summary=load(out/'attribution_summary.json');require(summary['passed'] and summary['new_native_advancing_steps']==0 and summary['summary_script_sha256']==sha(out/'summarize_selfmod_conflict_attribution.py'),'Read-only summary')
 require(summary['terminal_anchor_source_sha256']==sha(out.parent/'selfmod_v23_reached_replay_20261009/terminal_lives.json'),'Authenticated previous terminal anchors')
 findings=base/'docs/selfmod_v24_conflict_attribution_findings.md';require(findings.read_bytes()==(out/'findings.md').read_bytes(),'Findings parity')
 report=dict(passed=True,completed_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),protocol=contract['protocol'],engineering_only=True,new_scientific_advancing_steps=0,engineering_target_advancing_steps=8000,engineering_fixture_advancing_steps=256,total_engineering_advancing_steps=8256,requested_targets=4,completed_targets=4,rows_independently_audited=4000,candidate_identities_independently_enumerated=True,keyed_rng_independently_reconstructed=True,greedy_ownership_independently_reconstructed=True,production_resolver_and_runner_unchanged=True,exact_native_observation_receipts=len(native['exact_observed_native_receipts']),adversarial_corruptions_rejected=corruptions,source_files_verified=len(prov['sources_sha256']),frozen_execution_artifacts_verified=len(seal['files_sha256']),prior_archives_preserved=prior,historical_scientific_verdicts_changed=False,scientific_or_training_admission=False,findings_sha256=sha(findings),completion_script_sha256=sha(pathlib.Path(__file__)),artifacts_sha256=files(out,exclude=('completion_verification.json',)))
 with (out/'completion_verification.json').open('x') as f:json.dump(report,f,indent=2);f.write('\n')
 print(json.dumps({k:v for k,v in report.items() if k!='artifacts_sha256'},indent=2));print('Sealed artifacts',len(report['artifacts_sha256']))
if __name__=='__main__':main()
