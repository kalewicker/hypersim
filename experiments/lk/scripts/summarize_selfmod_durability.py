#!/usr/bin/env python3
"""Post-result paired durability and endpoint descriptions; frozen criteria unchanged."""
import argparse,collections,json,pathlib
from audit_selfmod_birth import load,sha,require
from run_selfmod_durability import ARMS

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('output',type=pathlib.Path);args=p.parse_args();out=args.output.resolve();a=load(out/'audit.json');v=load(out/'replay_verification.json')
 require(a['audit_passed'] and not a['engineering_preflight'] and v['passed'],'audited and replayed fresh panel')
 records=[]
 for r in a['records']:
  late=r['windows'][2:]
  records.append(dict(seed=r['seed'],arm=r['arm'],durability=r['durability'],minimum_late_capital=min(w['min_capital'] for w in late),minimum_rent_headroom=min(w['minimum_rent_headroom'] for w in late),
   culture_births=sum(sum(w['birth_windows']) for w in late),family_births=sum(sum(w['lineage_birth_windows']) for w in late),window_family_births=[sum(w['lineage_birth_windows']) for w in late],
   capital_means=[w['mean_capital'] for w in late],copy_means=[w['mean_copies'] for w in late],phase_capital_means=[w['retirement_phase_summary']['mean'] for w in late],
   window_adjusted_surpluses=[w['adjusted_surplus'] for w in late],old_segment_joint=[s['reproduction']['joint_success'] for s in r['segments']],old_segment_surpluses=[s['reproduction']['retirement_adjusted_late_surplus'] for s in r['segments']],
   final_lineage_copies=r['final_lineage_copies'],final_member_copies=r['final_member_copies'],capital_mean_slope=r['capital_mean_slope_per_1000']))
 pairs={}
 for policy in ['on','off','vacancy8','replace8']:
  f={r['seed']:r for r in records if r['arm']=='faithful_'+policy};p={r['seed']:r for r in records if r['arm']=='prune_'+policy};common=sorted(set(f)&set(p))
  pairs[policy]=dict(measured_pairs=len(common),durability_gains=[s for s in common if p[s]['durability']['durable_success'] and not f[s]['durability']['durable_success']],durability_losses=[s for s in common if f[s]['durability']['durable_success'] and not p[s]['durability']['durable_success']])
 failures={arm:dict(collections.Counter(reason for r in records if r['arm']==arm for reason in r['durability']['failure_reasons'])) for arm in ARMS}
 dest=out/'post_analysis';dest.mkdir(exist_ok=False);script=pathlib.Path(__file__);(dest/script.name).write_bytes(script.read_bytes())
 report=dict(description='New prospective durability and unchanged secondary endpoint criteria, fixed fresh eight-seed panel. Historical v15 primary remains 5/8; no pooling.',audit_sha256=sha(out/'audit.json'),script_sha256=sha(script),aggregates=a['aggregates'],paired=pairs,failure_counts=failures,missing=a['missing'],records=records)
 with (out/'interpretation.json').open('x') as f:json.dump(report,f,indent=2);f.write('\n')
 import matplotlib
 matplotlib.use('Agg')
 import matplotlib.pyplot as plt
 fig,ax=plt.subplots(figsize=(11.6,5.1));positions=list(range(8));new=[a['aggregates'][arm]['durable_success'] for arm in ARMS];old=[a['aggregates'][arm]['segment_joint_counts'][-1] for arm in ARMS]
 ax.bar([x-.18 for x in positions],new,width=.36,color='#00866a',label='New durability: all eight scored windows')
 ax.bar([x+.18 for x in positions],old,width=.36,color='#8795a3',label='Old combined criterion: final 2,000-step segment')
 for x,n,o in zip(positions,new,old):
  ax.text(x-.18,n+.12,str(n),ha='center',fontsize=10);ax.text(x+.18,o+.12,str(o),ha='center',fontsize=10)
 labels=[arm.replace('faithful_','Faithful\n').replace('prune_','Prune\n').replace('replace8','Replacement 8').replace('vacancy8','Vacancy 8').replace('_',' ').replace('on','ON').replace('off','OFF') for arm in ARMS]
 ax.set_xticks(positions,labels,fontsize=9);ax.set_ylim(0,9);ax.set_yticks(range(9));ax.set_ylabel('Successful seeds / fixed panel of 8');ax.grid(axis='y',alpha=.15);ax.set_axisbelow(True);ax.set_title('Fresh v17 panel: sustained funded turnover and the endpoint criterion',pad=16,fontsize=13)
 ax.legend(loc='upper left',fontsize=9,frameon=False);fig.text(.5,.015,'Criteria fixed before fresh results · serial windows are not pooled replications · all retirement withdrawals remain audited',ha='center',fontsize=8,color='#555555')
 fig.tight_layout(rect=(0,.05,1,1));fig.savefig(dest/'durability_comparison.png',dpi=180);fig.savefig(dest/'durability_comparison.svg');plt.close(fig)
 with (dest/'plot_provenance.json').open('x') as f:json.dump(dict(matplotlib_version=matplotlib.__version__,interpretation_sha256=sha(out/'interpretation.json'),png_sha256=sha(dest/'durability_comparison.png'),svg_sha256=sha(dest/'durability_comparison.svg')),f,indent=2);f.write('\n')
 print(json.dumps(dict(aggregates=a['aggregates'],paired=pairs,failure_counts=failures),indent=2))
if __name__=='__main__':main()
