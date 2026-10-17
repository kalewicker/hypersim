#!/usr/bin/env python3
"""Describe audited v18 named comparisons without changing prospective criteria."""
import argparse,collections,json,pathlib,statistics
from audit_selfmod_birth import load,sha,require,rows
from run_selfmod_variants import ARMS,VARIANTS

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('output',type=pathlib.Path);args=p.parse_args();out=args.output.resolve();a=load(out/'audit.json');v=load(out/'replay_verification.json')
 require(a['audit_passed'] and not a['engineering_preflight'] and v['passed'],'audited and replayed scientific panel')
 records=[]
 for r in a['records']:
  initial=load(out/f"seed_{r['seed']}/{r['arm'].split('_')[0]}_initial.json");genes={g['id']:g for g in initial['genes']};family_extinction=None;first_member_loss=None
  for row in rows(out/f"seed_{r['seed']}/{r['arm']}/rows.jsonl"):
   for b in row['births']:genes[b['child']]=b['gene']
   living=[genes[s['id']] for s in row['stocks']]
   if family_extinction is None and not any(g['lineage'] for g in living):family_extinction=row['continuation_step']
   if first_member_loss is None and any(not any(g['member']==m for g in living) for m in range(3)):first_member_loss=row['continuation_step']
  full_productive=sum(sum(w['lineage_productivity_windows']) for w in r['windows']);full_spend=sum(w['family_creation_spend'] for w in r['windows'])
  late=r['windows'][2:];productive=sum(sum(w['lineage_productivity_windows']) for w in late);spend=sum(w['family_creation_spend'] for w in late)
  records.append(dict(seed=r['seed'],arm=r['arm'],durability=r['durability'],family_extinction_step=family_extinction,first_culture_member_loss_step=first_member_loss,full_family_productive=full_productive,full_family_creation_spend=full_spend,full_family_spend_per_productive=full_spend/full_productive if full_productive else None,minimum_late_capital=min(w['min_capital'] for w in late),minimum_rent_headroom=min(w['minimum_rent_headroom'] for w in late),
   culture_births=sum(sum(w['birth_windows']) for w in late),family_births=sum(sum(w['lineage_birth_windows']) for w in late),window_family_births=[sum(w['lineage_birth_windows']) for w in late],minimum_quarter_culture_births=min(min(w['birth_windows']) for w in late),
   capital_means=[w['mean_capital'] for w in late],copy_means=[w['mean_copies'] for w in late],phase_capital_means=[w['retirement_phase_summary']['mean'] for w in late],window_adjusted_surpluses=[w['adjusted_surplus'] for w in late],
   family_creation_spend=spend,family_productive=productive,family_creation_spend_per_productive=spend/productive if productive else None,culture_creation_spend=sum(w['culture_creation_spend'] for w in late),
   mean_world_nodes=statistics.mean(w['mean_world_nodes'] for w in late),peak_world_nodes=max(w['peak_world_nodes'] for w in late),gates={k:sum(w['gates'][k] for w in late) for k in late[0]['gates']},
   old_segment_joint=[s['reproduction']['joint_success'] for s in r['segments']],old_segment_surpluses=[s['reproduction']['retirement_adjusted_late_surplus'] for s in r['segments']],
   final_lineage_copies=r['final_lineage_copies'],final_member_copies=r['final_member_copies'],capital_mean_slope=r['capital_mean_slope_per_1000']))
 pairs={}
 for variant in VARIANTS:
  g={r['seed']:r for r in records if r['arm']==variant+'_replace8'};pairs[variant]={}
  for baseline in ['faithful','prune']:
   b={r['seed']:r for r in records if r['arm']==baseline+'_replace8'};common=sorted(set(g)&set(b))
   pairs[variant][baseline]=dict(measured_pairs=len(common),durability_gains=[s for s in common if g[s]['durability']['durable_success'] and not b[s]['durability']['durable_success']],durability_losses=[s for s in common if b[s]['durability']['durable_success'] and not g[s]['durability']['durable_success']],paired_family_birth_differences={s:g[s]['family_births']-b[s]['family_births'] for s in common})
 failures={arm:dict(collections.Counter(reason for r in records if r['arm']==arm for reason in r['durability']['failure_reasons'])) for arm in ARMS}
 descriptions={}
 for arm in ARMS:
  group=[r for r in records if r['arm']==arm];productive=sum(r['family_productive'] for r in group);spend=sum(r['family_creation_spend'] for r in group)
  full_productive=sum(r['full_family_productive'] for r in group);full_spend=sum(r['full_family_creation_spend'] for r in group)
  descriptions[arm]=dict(full_family_productive=full_productive,full_family_creation_spend=full_spend,full_family_spend_per_productive=full_spend/full_productive if full_productive else None,family_extinction_steps={r['seed']:r['family_extinction_step'] for r in group},scored_family_births=sum(r['family_births'] for r in group),scored_culture_births=sum(r['culture_births'] for r in group),scored_family_productive=productive,scored_family_creation_spend=spend,family_spend_per_productive=spend/productive if productive else None,minimum_capital=min((r['minimum_late_capital'] for r in group),default=None),minimum_rent_headroom=min((r['minimum_rent_headroom'] for r in group),default=None),peak_world_nodes=max((r['peak_world_nodes'] for r in group),default=None),gates={k:sum(r['gates'][k] for r in group) for k in ['creation_blocked','node_ceiling_blocked','edge_ceiling_blocked','enumeration_skipped']})
 dest=out/'post_analysis';dest.mkdir(exist_ok=False);script=pathlib.Path(__file__);(dest/script.name).write_bytes(script.read_bytes())
 report=dict(description='Fixed fresh v18 panel; unchanged v17 durability; paired alternatives in resident cultures, not simultaneous coexistence. No serial-window pooling or significance claim.',audit_sha256=sha(out/'audit.json'),script_sha256=sha(script),reference_platform_replicated=a['aggregates']['prune_replace8']['durability_replicated'],aggregates=a['aggregates'],descriptions=descriptions,paired=pairs,failure_counts=failures,missing=a['missing'],records=records)
 with (out/'interpretation.json').open('x') as f:json.dump(report,f,indent=2);f.write('\n')
 import matplotlib
 matplotlib.use('Agg')
 import matplotlib.pyplot as plt
 fig,ax=plt.subplots(figsize=(9,5));new=[a['aggregates'][arm]['durable_success'] for arm in ARMS];old=[a['aggregates'][arm]['segment_joint_counts'][-1] for arm in ARMS];x=list(range(4))
 ax.bar([i-.18 for i in x],new,width=.36,color='#00866a',label='Durability: all eight scored windows');ax.bar([i+.18 for i in x],old,width=.36,color='#8795a3',label='Old combined criterion: final 2,000 steps')
 for i,n,o in zip(x,new,old):ax.text(i-.18,n+.12,str(n),ha='center');ax.text(i+.18,o+.12,str(o),ha='center')
 ax.axhline(6,color='#b56224',linestyle='--',linewidth=1,label='Declared durability threshold: 6/8')
 ax.set_xticks(x,['Faithful','RHS-edge prune\nReference','LHS-edge removal','Extra RHS-created node']);ax.set_ylim(0,9.4);ax.set_yticks(range(9));ax.set_ylabel('Successful seeds / fixed panel of 8');ax.set_title('Fresh v18 panel: named R1 variants under funded parent replacement',pad=15);ax.grid(axis='y',alpha=.15);ax.set_axisbelow(True);ax.legend(loc='upper right',fontsize=8,frameon=False)
 fig.text(.5,.015,'Candidates and criteria fixed before fresh output · cap eight · serial windows are not independent replications',ha='center',fontsize=8,color='#555555');fig.tight_layout(rect=(0,.045,1,1));fig.savefig(dest/'variant_comparison.png',dpi=180);fig.savefig(dest/'variant_comparison.svg');plt.close(fig)
 with (dest/'plot_provenance.json').open('x') as f:json.dump(dict(matplotlib_version=matplotlib.__version__,interpretation_sha256=sha(out/'interpretation.json'),png_sha256=sha(dest/'variant_comparison.png'),svg_sha256=sha(dest/'variant_comparison.svg')),f,indent=2);f.write('\n')
 print(json.dumps(dict(aggregates=a['aggregates'],descriptions=descriptions,paired=pairs,failure_counts=failures),indent=2))
if __name__=='__main__':main()
