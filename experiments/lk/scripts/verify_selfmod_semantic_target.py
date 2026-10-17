#!/usr/bin/env python3
"""Zero-object-step engineering verification of the prospective targeting adapter."""
import argparse,copy,json,pathlib,shutil,subprocess,tarfile,hashlib
from audit_selfmod_birth import load,sha,require
from audit_selfmod_financing import validate_allocator
from audit_selfmod_second_edit import pairing
from selfmod_second_edit_common import SecondState
from verify_selfmod_second_edit_interruption import native_order
ROOT=pathlib.Path(__file__).resolve().parents[1]

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('output',type=pathlib.Path);p.add_argument('--binary',default='selfmod-second-edit');a=p.parse_args();out=a.output.resolve();out.mkdir(exist_ok=False)
 binary=out/a.binary;shutil.copy2(ROOT/'target/release'/a.binary,binary)
 old=ROOT/'output/selfmod_v19_second_edit_20260928';pre=ROOT/'output/selfmod_v19_second_edit_preflight_20260928'
 # Hash all preserved raw evidence before and after this engineering operation.
 originals={str(f.relative_to(ROOT)):sha(f) for d in [old,pre] for f in d.rglob('*') if f.is_file()}
 contract=load(old/'contract.json');description=load(old/'composite_description.json');cases=[]
 for seed,archive in [(7,pre),(42,pre),(311,old),(313,old)]:
  source=archive/f'seed_{seed}/second_prefix';dest=out/f'seed_{seed}'
  result=subprocess.run([str(binary),'--semantic-regression',str(source/'fork.json'),str(dest)],check=True,capture_output=True,text=True);(out/f'seed_{seed}.log').write_text(result.stdout+result.stderr)
  require(load(dest/'faithful_birth.json')==load(source/'faithful_birth.json'),'exact saved native faithful phase')
  for suffix in ['', '_allocator']:require(load(dest/f'faithful_initial{suffix}.json')==load(source/f'faithful_initial{suffix}.json'),'exact faithful state and allocator')
  fork=load(source/'fork.json');record=load(dest/'verification.json');require(record['passed'] and record['object_steps']==0 and record['excluded_from_science'],'engineering-only scope')
  state=SecondState(fork['checkpoint'],'replace8');target=record['targeting'];child=fork['faithful_selection']['child'];order=native_order(load(dest/'faithful_initial.json'),child);index=next(i for i,n in enumerate(order) if n['variable']==0);effective=list(contract['second_edit']['parameters']);effective[0]=index
  require(target==dict(operator=3,target_variable=0,child=child,native_pnode_order=order,target_index=index,original_parameters=contract['second_edit']['parameters'],effective_parameters=effective,scheduler_draws=0),'independently resolved semantic target')
  for arm in ['faithful','edited']:
   born=load(dest/f'{arm}_birth.json');pairing(born['row'],fork['faithful_birth']);branch=copy.deepcopy(state);branch.finish_selected(born['row'],born['selection'],arm,contract,description);branch.match_snapshot(born['initial']);validate_allocator(born['initial'],load(dest/f'{arm}_initial_allocator.json'))
  faithful=load(dest/'faithful_initial.json');edited=load(dest/'edited_initial.json');node=order[index]['id'];idx=node&0xffffffff
  # The native operator must remove exactly that material constraint and its
  # role edge. All other live rule material stays byte-for-byte equal.
  for i,(f,e) in enumerate(zip(faithful['rules']['nodes'],edited['rules']['nodes'])):
   if i==idx:require(f['value'] is not None and e['value'] is None,'only target constraint removed')
   else:require(f==e,'all other rule nodes unchanged')
  removed=[]
  for i,(f,e) in enumerate(zip(faithful['rules']['edges'],edited['rules']['edges'])):
   if f!=e:
    require(f['value'] is not None and e['value'] is None and f['value']['label']==2002,'only target role edge removed')
    ids=[(n['version']<<32)|n['idx'] for n in f['value']['nodes']];require(set(ids)=={child,node},'deleted exact target edge');removed.append(i)
  require(len(removed)==1,'one target role edge')
  cases.append(dict(seed=seed,native_variable_order=[n['variable'] for n in order],effective_parameter_zero=index,paid_phase_verified=True,exact_genotype_verified=True,exact_material_edit_verified=True,object_steps=0));print(f'verified semantic target {seed}: order {[n["variable"] for n in order]}',flush=True)
 require({tuple(c['native_variable_order']) for c in cases}=={(0,1),(1,0)},'both allocation orders tested')
 require(all(sha(ROOT/f)==d for f,d in originals.items()),'failed science and prior preflight unmodified')
 sources={f:str(ROOT/f) for f in ['crates/hg-selfmod/src/birth_bounded.rs','crates/hg-selfmod/src/birth_second_edit.rs','crates/hg-selfmod/src/bin/selfmod-second-edit.rs','scripts/verify_selfmod_semantic_target.py','scripts/verify_selfmod_second_edit_interruption.py']}
 hashes={}
 with tarfile.open(old/'source.tar.gz') as prior,tarfile.open(out/'source.tar.gz','w:gz') as archive:
  for item in prior.getmembers():
   if item.name in sources:continue
   data=prior.extractfile(item).read();hashes[item.name]=hashlib.sha256(data).hexdigest();archive.addfile(item,__import__('io').BytesIO(data))
  for f,path in sources.items():archive.add(path,arcname=f);hashes[f]=sha(pathlib.Path(path))
 report=dict(passed=True,engineering_only=True,excluded_from_science=True,object_steps=0,cases=cases,both_native_orders_verified=True,preserved_raw_artifacts_sha256=originals,failed_science_and_prior_preflight_unmodified=True,binary_sha256=sha(binary),source_archive_sha256=sha(out/'source.tar.gz'),sources_sha256=hashes,auditor_sha256=sha(pathlib.Path(__file__)),new_scientific_assay_started=False)
 (out/'verification.json').write_text(json.dumps(report,indent=2)+'\n')

if __name__=='__main__':main()
