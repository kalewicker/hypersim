#!/usr/bin/env python3
"""Read-only exposure summary, terminal-gap extraction and scientific plot."""
import argparse,collections,json,pathlib
from audit_selfmod_birth import load,rows,require,sha
from selfmod_second_edit_common import descendants
from audit_selfmod_conflict_attribution import classify

def main():
 p=argparse.ArgumentParser();p.add_argument('output',type=pathlib.Path);out=p.parse_args().output.resolve();audit=load(out/'audit.json');require(audit['passed'],'Audited targets');summaries=[];gaps=[];earlier=load(out.parent/'selfmod_v23_reached_replay_20261009/terminal_lives.json')
 for record in audit['records']:
  folder=out/record['name'];initial=load(folder/'observed/initial.json');genes={g['id']:g for g in initial['genes']};family=descendants(genes,record['founder']);exposure=collections.Counter();terminal=next(c for c in earlier['cases'] if c['seed']==record['seed'])
  gap_metrics=collections.Counter();gap_sets=collections.Counter();gap_rows=0;anchor=terminal['extinction_step'];gap_start=terminal['last_income_step']+1;last_child=terminal['child']
  with (folder/'observed/rows.jsonl').open() as nf,(folder/'observed/conflicts.jsonl').open() as tf:
   for nl,tl in zip(nf,tf,strict=True):
    row=json.loads(nl);focal=[r for r in row['roots'] if r['id'] in family];exposure['active_focal_root_steps']+=sum(r['active'] for r in focal);exposure['focal_income']+=sum(r['income'] for r in focal);exposure['focal_rent']+=sum(r['cost'] for r in focal)
    if record['arm']=='faithful' and gap_start<=row['continuation_step']<anchor:
     trace=json.loads(tl);table=trace['rules'];pool={g['pool_index']:g['binding'] for g in trace['generated'] if g['pool_index'] is not None};gap_rows+=1
     for g in trace['generated']:
      if table[g['binding']['rid']]['root']==last_child:gap_metrics['generated']+=1;gap_metrics['truncated']+=g['pool_index'] is None
     for d in trace['resolution']['decisions']:
      b=pool[d['pool_index']]
      if table[b['rid']]['root']!=last_child:continue
      gap_metrics['considered']+=1;gap_metrics['accepted']+=d['accepted'];gap_metrics['rejected']+=not d['accepted'];cats=set()
      for blocker in d['blockers']:
       winnerroot=table[pool[blocker['winner_pool_index']]['rid']]['root'];cats.add(classify(genes[winnerroot],winnerroot,family))
      for cat in cats:gap_metrics['rejected_with_blocker/'+cat]+=1
      if cats:gap_sets['+'.join(sorted(cats))]+=1
    for b in row['births']:
     genes[b['child']]=b['gene']
     if b['parent'] in family:family.add(b['child'])
  metrics=record['metrics'];summary=dict(name=record['name'],arm=record['arm'],seed=record['seed'],exposure=dict(exposure),generated=metrics['generated'],truncated=metrics['truncated'],considered=metrics['considered'],accepted=metrics['accepted'],productive=metrics['productive'],rejected=metrics['rejected'],truncated_fraction=metrics['truncated']/metrics['generated'],accepted_fraction_of_considered=metrics['accepted']/metrics['considered'],productive_per_active_focal_root_step=metrics['productive']/exposure['active_focal_root_steps'],blocker_category_prevalence={k.split('/',1)[1]:v/metrics['rejected'] for k,v in metrics.items() if k.startswith('rejected_with_blocker/')},exclusive_category_sets=record['exclusive_rejection_category_sets']);summaries.append(summary)
  if record['arm']=='faithful':
   require(gap_rows==terminal['terminal_zero_income_active_steps'] and gap_metrics['considered']==gap_metrics['rejected']==terminal['terminal_considered_candidates'] and gap_metrics['accepted']==0,'Already observed terminal gap subset');require(sum(gap_sets.values())==gap_metrics['rejected'],'Terminal subset category partition')
   gaps.append(dict(name=record['name'],last_child=last_child,first_step=gap_start,last_step=anchor-1,rows=gap_rows,metrics=dict(gap_metrics),exclusive_category_sets=dict(gap_sets)))
 result=dict(passed=True,retrospective=True,new_native_advancing_steps=0,summary_script_sha256=sha(pathlib.Path(__file__)),terminal_anchor_source_sha256=sha(out.parent/'selfmod_v23_reached_replay_20261009/terminal_lives.json'),records=summaries,terminal_gaps=gaps,scope='audited observation summaries and already-known terminal gap subsets; no new causal assay')
 with (out/'attribution_summary.json').open('x') as f:json.dump(result,f,indent=2);f.write('\n')
 import matplotlib
 matplotlib.use('Agg')
 import matplotlib.pyplot as plt
 fig,axes=plt.subplots(1,2,figsize=(12,5),layout='constrained');labels=[f"V{23 if r['seed']==541 else 22} {r['seed']}\n{r['arm']}" for r in summaries];x=list(range(4))
 for category,label,color in [('other_member_2','Member 2','#267d9a'),('older_prune_relative','Older prune relatives','#dda33c'),('focal_family','Focal family','#7b67a5'),('other_member_1','Member 1','#c04b3c')]:
  axes[0].plot(x,[100*r['blocker_category_prevalence'].get(category,0) for r in summaries],marker='o',linestyle='none',markersize=8,label=label,color=color)
 axes[0].set_title('Rejected candidates with each blocker category\nCategories overlap');axes[0].set_ylabel('Percent of rejected focal candidates');axes[0].set_xticks(x,labels);axes[0].set_ylim(0,100);axes[0].legend(fontsize=9);axes[0].grid(alpha=.2)
 axes[1].bar(x,[r['productive_per_active_focal_root_step'] for r in summaries],color=['#c04b3c','#267d9a','#c04b3c','#267d9a']);axes[1].set_title('Observed productive firings per active focal root-step\nRetrospective states, no causal genotype comparison');axes[1].set_ylabel('Productive firings / active focal root-step');axes[1].set_xticks(x,labels);axes[1].grid(axis='y',alpha=.2)
 fig.savefig(out/'conflict_attribution_panel.png',dpi=160);fig.savefig(out/'conflict_attribution_panel.pdf');plt.close(fig)
 print(json.dumps(result,indent=2))
if __name__=='__main__':main()
