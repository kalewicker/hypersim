#!/usr/bin/env python3
"""Render the audited v16 financial traces from the preserved interpretation receipt."""
import argparse,json,pathlib
from audit_selfmod_birth import load,sha,require

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('output',type=pathlib.Path);args=p.parse_args();out=args.output.resolve()
 require(load(out/'audit.json')['audit_passed'] and load(out/'replay_verification.json')['passed'],'validated diagnostic')
 interpretation=load(out/'interpretation.json');target=interpretation['target_description']['target'];dest=out/'post_analysis'
 require(not (dest/'prune_replacement_financing.png').exists(),'exclusive plot')
 script=pathlib.Path(__file__);(dest/script.name).write_bytes(script.read_bytes())
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
 with (dest/'plot_provenance.json').open('x') as f:json.dump(dict(script_sha256=sha(script),interpretation_sha256=sha(out/'interpretation.json'),matplotlib_version=matplotlib.__version__,png_sha256=sha(dest/'prune_replacement_financing.png'),svg_sha256=sha(dest/'prune_replacement_financing.svg')),f,indent=2);f.write('\n')
 print(dest/'prune_replacement_financing.png')
if __name__=='__main__':main()
