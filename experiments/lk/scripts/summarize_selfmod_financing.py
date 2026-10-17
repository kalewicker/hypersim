#!/usr/bin/env python3
"""Post-result v16 stock and retirement-phase descriptions and static figures."""
import argparse,json,pathlib,statistics
from audit_selfmod_birth import load,sha,require

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('output',type=pathlib.Path);args=p.parse_args();out=args.output.resolve();audit=load(out/'audit.json');replay=load(out/'replay_verification.json')
 require(audit['audit_passed'] and not audit['engineering_preflight'] and replay['passed'],'validated full diagnostic required')
 prior=load(out/'inputs/audit.json');records=[]
 for r in audit['records']:
  baseline=next(b for b in prior['records'] if b['seed']==r['seed'] and b['arm']==r['arm'])
  records.append(dict(seed=r['seed'],arm=r['arm'],prior_joint=baseline['reproduction']['joint_success'],prior_surplus=baseline['reproduction']['retirement_adjusted_late_surplus'],
   start_capital=r['start_capital'],final_capital=r['final_capital'],net_capital_change=r['net_capital_change'],minimum_capital=r['minimum_capital'],capital_means=[w['mean_capital'] for w in r['windows']],
   phase_capital_means=[w['retirement_phase_summary']['mean'] for w in r['windows']],copy_means=[w['mean_copies'] for w in r['windows']],
   capital_mean_slope=r['capital_mean_slope_per_1000'],phase_capital_mean_slope=r['phase_capital_mean_slope_per_1000'],
   window_surpluses=[w['adjusted_surplus'] for w in r['windows']],window_family_births=[sum(w['lineage_birth_windows']) for w in r['windows']],
   segment_joint=[s['reproduction']['joint_success'] for s in r['segments']],segment_surpluses=[s['reproduction']['retirement_adjusted_late_surplus'] for s in r['segments']],
   family_alive=r['final_lineage_alive'],cycle_alive=r['final_cycle_alive'],culture_births_every_quarter=r['all_windows_culture_births'],family_births_every_1000=r['all_windows_family_births']))
 dest=out/'post_analysis';dest.mkdir(exist_ok=False);script=pathlib.Path(__file__);(dest/script.name).write_bytes(script.read_bytes())
 target=[r for r in records if r['arm']=='prune_replace8'];families=dict(prior_primary_reinterpreted=False,independent_replication=False,
  all_families_alive=all(r['family_alive'] for r in target),all_cycles_alive=all(r['cycle_alive'] for r in target),
  every_family_births_every_1000=all(r['family_births_every_1000'] for r in target),every_culture_births_every_quarter=all(r['culture_births_every_quarter'] for r in target),
  cash_mean_slope_range=[min(r['capital_mean_slope'] for r in target),max(r['capital_mean_slope'] for r in target)],
  phase_mean_slope_range=[min((r['phase_capital_mean_slope'] for r in target if r['phase_capital_mean_slope'] is not None),default=None),max((r['phase_capital_mean_slope'] for r in target if r['phase_capital_mean_slope'] is not None),default=None)],
  seeds_with_both_positive_negative_windows=[r['seed'] for r in target if min(r['window_surpluses'])<0<max(r['window_surpluses'])],target=target)
 report=dict(description='Post-result dependent financing diagnostic. Serial windows not independent; v15 primary unchanged. V15 original allocator order unavailable; v16 starts from restored snapshots.',audit_sha256=sha(out/'audit.json'),script_sha256=sha(script),target_description=families,aggregates=audit['aggregates'],records=records)
 with (out/'interpretation.json').open('x') as f:json.dump(report,f,indent=2);f.write('\n')
 import matplotlib
 matplotlib.use('Agg')
 import matplotlib.pyplot as plt
 fig,axes=plt.subplots(2,4,figsize=(13.2,7.0),sharex=True,sharey=True)
 for ax,r in zip(axes.flat,target):
  x=list(range(2500,10000,1000));ax.plot(x,r['capital_means'],'o-',color='#1565c0',label='Window mean capital',markersize=3.5)
  ax.plot(x,r['phase_capital_means'],'s--',color='#00866a',label='Mean just after family retirement',markersize=3.5)
  ax.plot([2000]+list(range(3000,10001,1000)),[r['start_capital']]+[r['start_capital']+sum(r['window_surpluses'][:i]) for i in range(1,9)],'.:',color='#757575',label='Fixed-window end capital',alpha=.8)
  title=f"Seed {r['seed']}"+(' · v15 financing failure' if not r['prior_joint'] else '')
  ax.set_title(title,fontsize=10);ax.set_ylim(bottom=0);ax.grid(alpha=.15);ax.set_xticks([2000,6000,10000]);ax.tick_params(labelsize=9)
  ax.text(.04,.04,f"Mean copies {r['copy_means'][0]:.2f} → {r['copy_means'][-1]:.2f}",transform=ax.transAxes,fontsize=8,color='#555555')
 fig.suptitle('Pruned parent replacement: capital through the fixed extension',fontsize=15,y=.985)
 fig.supxlabel('Continuation step (v15 ends at 2,000)',fontsize=11,y=.105);fig.supylabel('Selected-family capital (tokens)',fontsize=11,x=.012)
 handles,labels=axes.flat[0].get_legend_handles_labels();fig.legend(handles,labels,loc='lower center',ncol=3,frameon=False,fontsize=9,bbox_to_anchor=(.5,.035))
 fig.text(.5,.012,'All eight seeds · dependent snapshot continuations · unchanged policy · serial windows are not independent observations',ha='center',fontsize=8,color='#555555')
 fig.tight_layout(rect=(.03,.12,1,.94));fig.savefig(dest/'prune_replacement_financing.png',dpi=180);fig.savefig(dest/'prune_replacement_financing.svg');plt.close(fig)
 print(json.dumps(families,indent=2))
if __name__=='__main__':main()
