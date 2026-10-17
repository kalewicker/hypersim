#!/usr/bin/env python3
"""Post-result descriptions of third-founder and original-family outcomes."""
import argparse,collections,json,pathlib
from audit_selfmod_birth import load,sha,require,rows
from selfmod_second_edit_common import descendants
from run_selfmod_reversible_edit import ARMS
def summarize_scope(s):
 late=s['windows'][2:];productive=sum(sum(w['lineage_productivity_windows']) for w in late);spend=sum(w['family_creation_spend'] for w in late)
 return dict(durability=s['durability'],minimum_capital=min(w['min_capital'] for w in late),minimum_rent_headroom=min(w['minimum_rent_headroom'] for w in late),family_births=sum(sum(w['lineage_birth_windows']) for w in late),culture_births=sum(sum(w['birth_windows']) for w in late),window_family_births=[sum(w['lineage_birth_windows']) for w in late],family_productive=productive,family_creation_spend=spend,spend_per_productive=spend/productive if productive else None,window_surpluses=[w['adjusted_surplus'] for w in late],capital_means=[w['mean_capital'] for w in late],copy_means=[w['mean_copies'] for w in late],phase_capital_means=[w['retirement_phase_summary']['mean'] for w in late],final_family_copies=s['final_family_copies'],final_member_copies=s['final_member_copies'],old_segment_joint=[g['reproduction']['joint_success'] for g in s['segments']],gates={k:sum(w['gates'][k] for w in late) for k in late[0]['gates']})
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('output',type=pathlib.Path);args=p.parse_args();out=args.output.resolve();a=load(out/'audit.json');v=load(out/'replay_verification.json');require(a['audit_passed'] and not a['engineering_preflight'] and v['passed'],'audited and replayed science');records=[]
 for r in a['records']:
  directory=out/f"seed_{r['seed']}"/r['arm'];initial=load(directory/'initial.json');genes={g['id']:g for g in initial['genes']};ids=descendants(genes,r['founder']);second_loss=None;original_loss=None
  for row in rows(directory/'rows.jsonl'):
   for b in row['births']:
    genes[b['child']]=b['gene']
    if b['parent'] in ids:ids.add(b['child'])
   alive={s['id'] for s in row['stocks']}
   if second_loss is None and not alive&ids:second_loss=row['continuation_step']
   if original_loss is None and not any(genes[id_]['lineage'] for id_ in alive):original_loss=row['continuation_step']
  records.append(dict(seed=r['seed'],arm=r['arm'],founder=r['founder'],third_extinction_step=second_loss,original_extinction_step=original_loss,third_family=summarize_scope(r['third_family']),second_family=summarize_scope(r['second_family']),original_family=summarize_scope(r['original_family'])))
 faithful={r['seed']:r for r in records if r['arm']=='faithful'};edited={r['seed']:r for r in records if r['arm']=='edited'};common=sorted(set(faithful)&set(edited))
 paired=dict(measured_pairs=len(common),durability_gains=[s for s in common if edited[s]['third_family']['durability']['durable_success'] and not faithful[s]['third_family']['durability']['durable_success']],durability_losses=[s for s in common if faithful[s]['third_family']['durability']['durable_success'] and not edited[s]['third_family']['durability']['durable_success']],family_birth_differences={s:edited[s]['third_family']['family_births']-faithful[s]['third_family']['family_births'] for s in common})
 failures={arm:dict(collections.Counter(reason for r in records if r['arm']==arm for reason in r['third_family']['durability']['failure_reasons'])) for arm in ARMS};totals={}
 for arm in ARMS:
  group=[r for r in records if r['arm']==arm];totals[arm]={}
  for scope in ['third_family','second_family','original_family']:
   items=[r[scope] for r in group];productive=sum(s['family_productive'] for s in items);spend=sum(s['family_creation_spend'] for s in items)
   totals[arm][scope]=dict(scored_family_births=sum(s['family_births'] for s in items),scored_culture_births=sum(s['culture_births'] for s in items),family_productive=productive,family_creation_spend=spend,spend_per_productive=spend/productive if productive else None,minimum_capital=min((s['minimum_capital'] for s in items),default=None),minimum_rent_headroom=min((s['minimum_rent_headroom'] for s in items),default=None),old_segment_joint_counts=[sum(s['old_segment_joint'][i] for s in items) for i in range(5)])
 dest=out/'post_analysis';dest.mkdir(exist_ok=False);script=pathlib.Path(__file__);(dest/script.name).write_bytes(script.read_bytes())
 report=dict(description='Prospectively fixed third-child ancestry criterion, two preregistered readiness phases; older-family survival cannot rescue loss of the third family. No panel pooling or significance claim.',audit_sha256=sha(out/'audit.json'),script_sha256=sha(script),aggregates=a['aggregates'],counts=a['counts'],first_readiness_passes=a['first_readiness_passes'],second_readiness_passes=a['second_readiness_passes'],faithful_founder_benchmark_passed=a['aggregates']['faithful']['replicated'],paired=paired,failure_counts=failures,totals=totals,records=records)
 with (out/'interpretation.json').open('x') as f:json.dump(report,f,indent=2);f.write('\n')
 import matplotlib
 matplotlib.use('Agg')
 import matplotlib.pyplot as plt
 fig,ax=plt.subplots(figsize=(9,5.4));x=[0,1];second=[a['aggregates'][arm]['durable_third_families'] for arm in ARMS];original=[a['aggregates'][arm]['original_family_durable'] for arm in ARMS]
 ax.bar([i-.18 for i in x],second,width=.36,color='#00866a',label='Third-child family: primary criterion');ax.bar([i+.18 for i in x],original,width=.36,color='#8795a3',label='Original prune family: secondary criterion')
 for i,n,o in zip(x,second,original):ax.text(i-.18,n+.12,str(n),ha='center');ax.text(i+.18,o+.12,str(o),ha='center')
 ax.axhline(6,color='#b56224',linestyle='--',linewidth=1);ax.text(1.45,6.1,'6/8 threshold',ha='right',fontsize=9,color='#b56224');ax.set_xticks(x,['Faithful third birth','Third edit: restore var0 label0']);ax.set_ylim(0,9);ax.set_yticks(range(9));ax.set_ylabel('Durable worlds / fixed panel of 8');ax.grid(axis='y',alpha=.15);ax.set_axisbelow(True)
 fig.suptitle('Fresh v21 panel: third edit after durable twice-edited establishment',y=.97,fontsize=13);fig.legend(loc='upper center',bbox_to_anchor=(.5,.91),ncol=2,fontsize=8,frameon=False);fig.text(.5,.02,'Same paid child and resident culture · immutable ancestry · 10,000 post-birth steps · no serial-window pooling',ha='center',fontsize=8,color='#555555');fig.tight_layout(rect=(0,.055,1,.83));fig.savefig(dest/'reversible_edit_comparison.png',dpi=180);fig.savefig(dest/'reversible_edit_comparison.svg');plt.close(fig)
 with (dest/'plot_provenance.json').open('x') as f:json.dump(dict(matplotlib_version=matplotlib.__version__,interpretation_sha256=sha(out/'interpretation.json'),png_sha256=sha(dest/'reversible_edit_comparison.png'),svg_sha256=sha(dest/'reversible_edit_comparison.svg')),f,indent=2);f.write('\n')
 print(json.dumps(dict(aggregates=a['aggregates'],paired=paired,failure_counts=failures,totals=totals),indent=2))
if __name__=='__main__':main()
