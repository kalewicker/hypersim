#!/usr/bin/env python3
"""Corrupt read-only engineering views to check missing-seed/branch/denominator handling."""
import argparse,contextlib,copy,io,json,pathlib,unittest.mock
import audit_selfmod_durability as audit
from audit_selfmod_birth import sha

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('engineering',type=pathlib.Path);p.add_argument('receipt',type=pathlib.Path);args=p.parse_args();source=args.engineering.resolve();rejected=[]
 marker=source/'seed_7/untriggered.json';exists=pathlib.Path.exists
 with unittest.mock.patch.object(pathlib.Path,'exists',lambda p:True if p==marker else exists(p)):
  try:
   with contextlib.redirect_stdout(io.StringIO()):audit.audit(source)
  except ValueError:rejected.append('false_untriggered_marker')
  else:raise AssertionError('accepted false untriggered seed')
 target=source/'seed_7/prune_replace8/rows.jsonl';original_rows=audit.rows
 def missing_rows(p):
  if p==target:raise FileNotFoundError('corrupted view: removed admissible branch')
  return original_rows(p)
 with unittest.mock.patch.object(audit,'rows',missing_rows):
  try:
   with contextlib.redirect_stdout(io.StringIO()):audit.audit(source)
  except FileNotFoundError:rejected.append('missing_admissible_branch')
  else:raise AssertionError('accepted missing valid branch')
 original_load=audit.load;summary=source/'summary.json'
 def damaged_load(p):
  value=original_load(p)
  if p==summary:value=copy.deepcopy(value);value['fixed_seed_denominator']=1
  return value
 with unittest.mock.patch.object(audit,'load',damaged_load):
  try:
   with contextlib.redirect_stdout(io.StringIO()):audit.audit(source)
  except ValueError:rejected.append('shrunken_denominator')
  else:raise AssertionError('accepted shrunken denominator')
 with contextlib.redirect_stdout(io.StringIO()):audit.audit(source)
 report=dict(passed=True,rejected=rejected,untampered_panel_passed=True,engineering=str(source),script_sha256=sha(pathlib.Path(__file__)),scope='Only mocked read-only views changed; no source or raw evidence modified.')
 with args.receipt.open('x') as f:json.dump(report,f,indent=2);f.write('\n')
 args.receipt.with_suffix('.py').write_bytes(pathlib.Path(__file__).read_bytes());print(json.dumps(report,indent=2))
if __name__=='__main__':main()
