#!/usr/bin/env python3
"""Post-result failure attribution for v14; primary thresholds unchanged."""
import argparse,collections,json,pathlib,statistics
from audit_selfmod_birth import load,rows,sha,require
from audit_selfmod_bounded import ARMS

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('output',type=pathlib.Path);args=p.parse_args();out=args.output
 audit=load(out/'audit.json');require(audit['audit_passed'] and load(out/'replay_verification.json')['passed'],'validated scientific panel required');require(not audit['engineering_preflight'],'scientific panel')
 records=[]
 for r in audit['records']:
  s,j=r['score'],r['reproduction'];t=s['totals'];m=j['metrics'];reasons=[]
  if not s['bounded_establishment']:reasons.append('legacy_establishment_failed')
  if j['retirement_adjusted_late_surplus']<0:reasons.append('late_withdrawal_adjusted_deficit')
  if not all(m['late_birth_windows']):reasons.append('late_culture_birth_gap')
  if not m['late_lineage_births']:reasons.append('no_late_lineage_birth')
  gatecounts=collections.Counter();eligibility=blocked=0;lineage_birth_windows=[0]*4;late_gates=collections.Counter()
  for row in rows(out/f"seed_{r['seed']}/{r['arm']}/rows.jsonl"):
   late=row['continuation_step']>1000
   if 'bounded' in row:eligibility+=len(row['bounded']['eligible']);blocked+=len(row['bounded']['blocked'])
   if late:
    lineage_birth_windows[(row['continuation_step']-1001)//250]+=sum(b['gene']['lineage'] for b in row['births'])
   for root in row['roots']:
    if root['gate']:
     for k in ['creation_blocked','node_ceiling_blocked','edge_ceiling_blocked','enumeration_skipped']:
      gatecounts[k]+=bool(root['gate'][k])
      if late:late_gates[k]+=bool(root['gate'][k])
  records.append(dict(seed=r['seed'],arm=r['arm'],legacy_established=s['bounded_establishment'],joint_success=j['joint_success'],
   final_lineage_copies=s['final_lineage_copies'],final_member_copies=s['final_member_copies'],late_productivity_windows=t['late_lineage_windows'],
   late_gross_income_minus_rent=t['late_lineage_income']-t['late_lineage_rent'],late_retired_capital=m['late_lineage_retired'],
   late_starvation_return=m['late_lineage_starved'],late_adjusted_surplus=j['retirement_adjusted_late_surplus'],
   late_start_capital=m['late_start_capital'],final_capital=m['final_capital'],late_min_capital=m['late_min_capital'],late_max_capital=m['late_max_capital'],
   culture_birth_windows=m['late_birth_windows'],lineage_birth_windows=lineage_birth_windows,births=m['births'],lineage_births=m['lineage_births'],
   retirements=m['retirements'],retired_tokens=m['retired_tokens'],eligible_opportunities=eligibility,blocked_opportunities=blocked,
   gates=dict(gatecounts),late_gates=dict(late_gates),failure_reasons=reasons,lineage_per_copy_rate=s['lineage_per_copy_rate']))
 aggregates={}
 for arm in ARMS:
  group=[r for r in records if r['arm']==arm];gates={k:sum(r['gates'][k] for r in group) for k in ['creation_blocked','node_ceiling_blocked','edge_ceiling_blocked','enumeration_skipped']}
  aggregates[arm]=dict(legacy_established_seeds=[r['seed'] for r in group if r['legacy_established']],joint_success_seeds=[r['seed'] for r in group if r['joint_success']],
   reasons=dict(collections.Counter(reason for r in group for reason in r['failure_reasons'])),gate_totals=gates,
   eligible_opportunities=sum(r['eligible_opportunities'] for r in group),blocked_opportunities=sum(r['blocked_opportunities'] for r in group),
   late_adjusted_surplus_range=[min(r['late_adjusted_surplus'] for r in group),max(r['late_adjusted_surplus'] for r in group)],
   late_gross_surplus_range=[min(r['late_gross_income_minus_rent'] for r in group),max(r['late_gross_income_minus_rent'] for r in group)],
   late_lineage_births=sum(sum(r['lineage_birth_windows']) for r in group),late_culture_births=sum(sum(r['culture_birth_windows']) for r in group),
   late_lineage_capital_floor_range=[min(r['late_min_capital'] for r in group),max(r['late_min_capital'] for r in group)])
 dest=out/'post_analysis';dest.mkdir(exist_ok=False);script=pathlib.Path(__file__);(dest/script.name).write_bytes(script.read_bytes())
 with (out/'interpretation.json').open('x') as f:json.dump(dict(description='Post-result v14 failure/gate attribution. Primary verdicts unchanged.',audit_sha256=sha(out/'audit.json'),
  script_sha256=sha(script),aggregates=aggregates,records=records),f,indent=2);f.write('\n')
 print(json.dumps(aggregates,indent=2))
if __name__=='__main__':main()
