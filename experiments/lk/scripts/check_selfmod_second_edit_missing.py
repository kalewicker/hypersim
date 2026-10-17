#!/usr/bin/env python3
"""Reject mocked readiness, first-founder and denominator corruptions without editing evidence."""
import argparse,contextlib,copy,io,json,pathlib,unittest.mock
import audit_selfmod_second_edit as audit
import selfmod_second_edit_common as common
from audit_selfmod_birth import sha
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('engineering',type=pathlib.Path);p.add_argument('receipt',type=pathlib.Path);args=p.parse_args();source=args.engineering.resolve();rejected=[];original_load=audit.load;original_common_load=common.load
 def attempt(name):
  try:
   with contextlib.redirect_stdout(io.StringIO()):audit.audit(source)
  except (ValueError,FileNotFoundError):rejected.append(name)
  else:raise AssertionError('accepted corrupted '+name)
 marker=source/'seed_7/initial_prefix/untriggered.json';exists=pathlib.Path.exists
 with unittest.mock.patch.object(pathlib.Path,'exists',lambda p:True if p==marker else exists(p)):attempt('false_initial_untriggered')
 original_rows=common.rows
 def missing(p):
  if p==source/'seed_7/edited/rows.jsonl':raise FileNotFoundError('mocked missing valid branch')
  return original_rows(p)
 with unittest.mock.patch.object(common,'rows',missing):attempt('missing_edited_branch')
 for name in ['shrunken_denominator','false_readiness','wrong_second_founder']:
  def damaged(p):
   d=original_load(p)
   if name=='shrunken_denominator' and p==source/'summary.json':d=copy.deepcopy(d);d['fixed_seed_denominator']=1
   elif name=='false_readiness' and p==source/'seed_7/readiness.json':d=copy.deepcopy(d);d['durability']['durable_success']=False
   elif name=='wrong_second_founder' and p==source/'seed_7/second_prefix/edited_birth.json':d=copy.deepcopy(d);d['selection']['child']=original_load(source/'seed_7/initial_prefix/prune_birth.json')['selection']['child']
   return d
  with unittest.mock.patch.object(audit,'load',damaged):attempt(name)
 with contextlib.redirect_stdout(io.StringIO()):audit.audit(source)
 report=dict(passed=True,rejected=rejected,untampered_panel_passed=True,script_sha256=sha(pathlib.Path(__file__)),scope='Only read-only mocked views changed; native source and raw evidence preserved.')
 with args.receipt.open('x') as f:json.dump(report,f,indent=2);f.write('\n')
 args.receipt.with_suffix('.py').write_bytes(pathlib.Path(__file__).read_bytes());print(json.dumps(report,indent=2))
if __name__=='__main__':main()
