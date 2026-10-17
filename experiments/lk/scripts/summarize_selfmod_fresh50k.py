#!/usr/bin/env python3
"""Post-result paired interpretation and dependent temporal maintenance plot."""
import argparse,collections,json,pathlib
from audit_selfmod_birth import load,sha,require,rows
from selfmod_fresh50k_common import descendants

def scope_summary(s):
 w=s['windows'][2:];productive=sum(sum(x['lineage_productivity_windows']) for x in w);spend=sum(x['family_creation_spend'] for x in w)
 return dict(maintenance=s['maintenance'],first10000_durability=s['first10000_durability'],family_births=sum(sum(x['lineage_birth_windows']) for x in w),culture_births=sum(sum(x['birth_windows']) for x in w),family_productive=productive,family_creation_spend=spend,spend_per_productive=spend/productive if productive else None,minimum_capital=min(x['min_capital'] for x in w),minimum_rent_headroom=min(x['minimum_rent_headroom'] for x in w),minimum_productive_quarter=min(min(x['lineage_productivity_windows']) for x in w),minimum_family_births_window=min(sum(x['lineage_birth_windows']) for x in w),minimum_culture_births_quarter=min(min(x['birth_windows']) for x in w),minimum_member_productive_quarter=min(min(min(q) for q in x['member_productivity_windows']) for x in w),final_family_copies=s['final_family_copies'],final_member_copies=s['final_member_copies'],capital_mean_slope_per_1000=s['capital_mean_slope_per_1000'],old_segment_joint=[x['reproduction']['joint_success'] for x in s['segments']],gates={k:sum(x['gates'][k] for x in w) for k in w[0]['gates']})

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('output',type=pathlib.Path);out=p.parse_args().output.resolve();a=load(out/'audit.json');require(a['audit_passed'] and not a['engineering_preflight'] and load(out/'replay_verification.json')['passed'],'Audited replayed science');records=[]
 for record in a['records']:
  d=out/f"seed_{record['seed']}/{record['arm']}";genes={g['id']:g for g in load(d/'initial.json')['genes']};ids=descendants(genes,record['founder']);loss=culture_loss=empty=None;peak=0
  for r in rows(d/'rows.jsonl'):
   for b in r['births']:
    genes[b['child']]=b['gene']
    if b['parent'] in ids:ids.add(b['child'])
   live={s['id'] for s in r['stocks']}
   if loss is None and not live & ids:loss=r['continuation_step']
   if culture_loss is None and {genes[id_]['member'] for id_ in live}!={0,1,2}:culture_loss=r['continuation_step']
   if empty is None and not live:empty=r['continuation_step']
   peak=max(peak,r['world_nodes'])
  require(ids==descendants(genes,record['founder']),'Independent ancestry check')
  records.append(dict(seed=record['seed'],arm=record['arm'],founder=record['founder'],family_extinction_step=loss,first_culture_member_loss=culture_loss,empty_rule_population=empty,peak_world_nodes=peak,**{key:scope_summary(record[key]) for key in ['third_family','second_family','original_family']}))
 totals={};failures={}
 for arm in ['faithful','edited']:
  totals[arm]={}
  for scope in ['third_family','second_family','original_family']:
   g=[r[scope] for r in records if r['arm']==arm]
   totals[arm][scope]=dict(family_births=sum(r['family_births'] for r in g),culture_births=sum(r['culture_births'] for r in g),family_productive=sum(r['family_productive'] for r in g),family_creation_spend=sum(r['family_creation_spend'] for r in g),minimum_capital=min((r['minimum_capital'] for r in g),default=None),minimum_rent_headroom=min((r['minimum_rent_headroom'] for r in g),default=None),temporal_block_pass_counts=[sum(r['maintenance']['temporal_blocks'][i]['durable_success'] for r in g) for i in range(6)],cumulative_maintenance_counts=[sum(all(b['durable_success'] for b in r['maintenance']['temporal_blocks'][:i+1]) for r in g) for i in range(6)],old_segment_joint_counts=[sum(r['old_segment_joint'][i] for r in g) for i in range(25)])
  failures[arm]=dict(collections.Counter(x for r in records if r['arm']==arm for x in r['third_family']['maintenance']['failure_reasons']))
 faithful={r['seed']:r for r in records if r['arm']=='faithful'};edited={r['seed']:r for r in records if r['arm']=='edited'}
 paired=dict(maintenance_gains=[s for s in faithful if edited[s]['third_family']['maintenance']['maintained_success'] and not faithful[s]['third_family']['maintenance']['maintained_success']],maintenance_losses=[s for s in faithful if faithful[s]['third_family']['maintenance']['maintained_success'] and not edited[s]['third_family']['maintenance']['maintained_success']],family_birth_differences={s:edited[s]['third_family']['family_births']-faithful[s]['third_family']['family_births'] for s in faithful})
 dest=out/'post_analysis';dest.mkdir(exist_ok=False);script=pathlib.Path(__file__);(dest/script.name).write_bytes(script.read_bytes())
 report=dict(audit_sha256=sha(out/'audit.json'),script_sha256=sha(script),independent_fresh_panel=True,independent_worlds=8,renewed_warmup_steps=0,aggregates=a['aggregates'],counts=a['counts'],totals=totals,paired=paired,failure_counts=failures,records=records)
 with (out/'interpretation.json').open('x') as f:json.dump(report,f,indent=2);f.write('\n')
 import matplotlib
 matplotlib.use('Agg')
 import matplotlib.pyplot as plt
 fig,ax=plt.subplots(figsize=(9,5.3));x=[10000,18000,26000,34000,42000,50000]
 for arm,label,color in [('faithful','Var0 broad; label1 retained','#00866a'),('edited','Var0 label0 restored; label1 retained','#6a53a3')]:ax.plot(x,totals[arm]['third_family']['cumulative_maintenance_counts'],marker='o',color=color,label=label,linewidth=2)
 ax.axhline(6,color='#a85b2c',linestyle='--',label='6/8 threshold',linewidth=1);ax.set_ylim(0,8.7);ax.set_yticks(range(9));ax.set_xticks(x,[str(i//1000)+'k' for i in x]);ax.set_xlabel('Steps since the original fresh third paid birth');ax.set_ylabel('Worlds meeting every scored requirement so far / 8');ax.grid(alpha=.15);ax.legend(loc='lower left',frameon=False,fontsize=9);fig.suptitle('V23: fresh matched 50,000-step panel');fig.text(.5,.025,'Eight fresh worlds · original 2k warmup only · all 48 scored windows · temporal blocks are dependent',ha='center',fontsize=8);fig.tight_layout(rect=(0,.055,1,.94));fig.savefig(dest/'fresh50k_comparison.png',dpi=180);fig.savefig(dest/'fresh50k_comparison.svg');plt.close(fig)
 with (dest/'plot_provenance.json').open('x') as f:json.dump(dict(matplotlib_version=matplotlib.__version__,interpretation_sha256=sha(out/'interpretation.json'),png_sha256=sha(dest/'fresh50k_comparison.png'),svg_sha256=sha(dest/'fresh50k_comparison.svg')),f,indent=2);f.write('\n')
 print(json.dumps({k:report[k] for k in ['aggregates','totals','paired','failure_counts']},indent=2))
if __name__=='__main__':main()
