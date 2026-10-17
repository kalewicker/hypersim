#!/usr/bin/env python3
"""Read-only mocked routing, founder and result-claim rejections."""
import argparse,copy,json,pathlib,unittest.mock,contextlib,io
import audit_selfmod_fresh50k as audit
import selfmod_fresh50k_common as common
from audit_selfmod_birth import load,sha

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('engineering',type=pathlib.Path);args=p.parse_args();out=args.engineering.resolve();base=load(out/'audit.json');rejected=[]
 require=lambda c,m: audit.require(c,m)
 require(base['audit_passed'],'Qualified engineering source')
 def reject(name,func):
  try:
   with contextlib.redirect_stdout(io.StringIO()):func()
  except (ValueError,FileNotFoundError):rejected.append(name)
  else:raise AssertionError('Accepted '+name)
 old=common.rows
 with unittest.mock.patch.object(common,'rows',lambda f:(_ for _ in ()).throw(FileNotFoundError('missing branch')) if f==out/'seed_7/edited/rows.jsonl' else old(f)):
  reject('missing_edited_branch',lambda:audit.audit(out))
 original=audit.load
 for name in ['false_initial_untriggered','false_readiness','false_second_readiness','wrong_third_founder']:
  def damaged(f):
   d=original(f)
   if name=='false_initial_untriggered' and f==out/'seed_7/routing.json':d=dict(seed=7,kind='initial_untriggered')
   elif name=='false_readiness' and f==out/'seed_7/readiness.json':d=copy.deepcopy(d);d['durability']['durable_success']=False
   elif name=='false_second_readiness' and f==out/'seed_7/second_readiness.json':d=copy.deepcopy(d);d['durability']['durable_success']=False
   elif name=='wrong_third_founder' and f==out/'seed_7/third_prefix/edited_birth.json':d=copy.deepcopy(d);d['selection']['child']=original(out/'seed_7/initial_prefix/prune_birth.json')['selection']['child']
   return d
  with unittest.mock.patch.object(audit,'load',damaged):reject(name,lambda:audit.audit(out))
 for name in ['shrunken_denominator','omitted_record','false_maintenance','false_aggregate','wrong_claimed_founder','false_route']:
  d=copy.deepcopy(base)
  if name=='shrunken_denominator':d['counts']['fixed_seed_denominator']=1
  elif name=='omitted_record':d['records'].pop()
  elif name=='false_maintenance':d['records'][0]['third_family']['maintenance']['maintained_success']=True
  elif name=='false_aggregate':d['aggregates']['edited']['maintained_families']=2
  elif name=='wrong_claimed_founder':d['records'][0]['founder']=-1
  else:d['counts']['routing'][0]['kind']='initial_untriggered'
  reject(name,lambda:audit.validate_claims(out,d))
 audit.validate_claims(out,base)
 report=dict(passed=True,rejected=rejected,untampered_panel_passed=True,script_sha256=sha(pathlib.Path(__file__)),raw_evidence_unchanged=True)
 with (out/'missing_case_verification.json').open('x') as f:json.dump(report,f,indent=2);f.write('\n')
 (out/'missing_case_verification.py').write_bytes(pathlib.Path(__file__).read_bytes());print(json.dumps(report,indent=2))
if __name__=='__main__':main()
