#!/usr/bin/env python3
"""Post-result descriptions; never change preregistered v13 verdicts."""
import argparse,collections,json,pathlib,statistics
from audit_selfmod_birth import load,rows,sha,require
from audit_selfmod_turnover import ARMS

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('output',type=pathlib.Path);args=p.parse_args();out=args.output
    audit=load(out/'audit.json');require(audit['audit_passed'] and load(out/'replay_verification.json')['passed'],'validated panel required')
    records=[]
    for r in audit['records']:
        seed,arm,s=r['seed'],r['arm'],r['score'];t=s['totals'];counter=collections.Counter();late_counter=collections.Counter();birth_count=0
        for row in rows(out/f'seed_{seed}/{arm}/rows.jsonl'):
            birth_count+=len(row['births'])
            for root in row['roots']:
                g=root['gate']
                if g:
                    for kind in ['creation_blocked','node_ceiling_blocked','edge_ceiling_blocked','enumeration_skipped']:
                        counter[kind]+=bool(g[kind])
                        if row['continuation_step']>1000:late_counter[kind]+=bool(g[kind])
        net=t['late_lineage_income']-t['late_lineage_rent']; reasons=[]
        if s['final_lineage_copies']==0:reasons.append('lineage_lost')
        if not all(t['late_lineage_windows']):reasons.append('late_productivity_gap')
        if net<0:reasons.append('late_income_below_rent')
        if not s['whole_cycle_persistent']:reasons.append('cycle_lost')
        if t['enumeration_skips']:reasons.append('enumeration_censored')
        records.append(dict(seed=seed,arm=arm,established=s['bounded_establishment'],final_lineage_copies=s['final_lineage_copies'],
            late_net_income=net,late_income=t['late_lineage_income'],late_rent=t['late_lineage_rent'],
            late_productivity_windows=t['late_lineage_windows'],lineage_per_copy_rate=s['lineage_per_copy_rate'],
            lineage_copy_steps=t['lineage_copy_steps'],final_lineage_capital=s['final_lineage_capital'],
            final_member_copies=s['final_member_copies'],parent_final_capital=s['parent_final_capital'],
            child_death_step=t['child_death_step'],parent_death_step=t['parent_death_step'],
            total_future_births=birth_count,gates=dict(counter),late_gates=dict(late_counter),failure_reasons=reasons))
    paired={}
    for genotype in ['faithful','prune']:
        group=[x for x in audit['paired'] if x['genotype']==genotype]
        paired[genotype]=dict(gained_establishment_seeds=[x['seed'] for x in group if x['off_established'] and not x['on_established']],
            lost_establishment_seeds=[x['seed'] for x in group if x['on_established'] and not x['off_established']],
            established_both_seeds=[x['seed'] for x in group if x['on_established'] and x['off_established']],
            failed_both_seeds=[x['seed'] for x in group if not x['on_established'] and not x['off_established']])
    aggregates={}
    for arm in ARMS:
        group=[r for r in records if r['arm']==arm]
        aggregates[arm]=dict(late_net_income_range=[min(r['late_net_income'] for r in group),max(r['late_net_income'] for r in group)],
            late_net_income_mean=statistics.mean(r['late_net_income'] for r in group),
            gate_totals={kind:sum(r['gates'][kind] for r in group) for kind in ['creation_blocked','node_ceiling_blocked','edge_ceiling_blocked','enumeration_skipped']},
            future_births=sum(r['total_future_births'] for r in group),final_copy_range=[min(sum(r['final_member_copies']) for r in group),max(sum(r['final_member_copies']) for r in group)])
    dest=out/'post_analysis';dest.mkdir(exist_ok=False);script=pathlib.Path(__file__);(dest/script.name).write_bytes(script.read_bytes())
    with (out/'interpretation.json').open('x') as f:
        json.dump(dict(description='Post-result failure/gate descriptions. Frozen v13 primary verdicts unchanged.',audit_sha256=sha(out/'audit.json'),
            script_sha256=sha(script),paired=paired,aggregates=aggregates,records=records),f,indent=2);f.write('\n')
    print(json.dumps(dict(paired=paired,aggregates=aggregates),indent=2))
if __name__=='__main__':main()
