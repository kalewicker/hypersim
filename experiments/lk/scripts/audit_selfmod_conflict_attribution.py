#!/usr/bin/env python3
"""Independent keyed order, canonical membership, greedy ownership and native flow audit."""
import argparse,collections,copy,json,pathlib,sys
from audit_selfmod_birth import load,rows,require,sha
from audit_selfmod_bounded import BoundedState
from audit_selfmod_financing import validate_allocator
from selfmod_second_edit_common import descendants
MASK32=(1<<32)-1;MASK64=(1<<64)-1

def draw_words(key,count):
 # rand_core 0.6.4 default PCG32 seed expansion; ChaCha8 with zero stream/counter.
 seed=key^0x71384C2D9B05A1F7;words=[]
 for _ in range(8):
  seed=(seed*6364136223846793005+11634580027462260723)&MASK64;x=(((seed>>18)^seed)>>27)&MASK32;rot=seed>>59
  words.append(((x>>rot)|(x<<((-rot)&31)))&MASK32)
 def rot(x,n):return ((x<<n)|(x>>(32-n)))&MASK32
 def qr(x,a,b,c,d):
  x[a]=(x[a]+x[b])&MASK32;x[d]=rot(x[d]^x[a],16);x[c]=(x[c]+x[d])&MASK32;x[b]=rot(x[b]^x[c],12)
  x[a]=(x[a]+x[b])&MASK32;x[d]=rot(x[d]^x[a],8);x[c]=(x[c]+x[d])&MASK32;x[b]=rot(x[b]^x[c],7)
 result=[];counter=0
 while len(result)<count:
  initial=[0x61707865,0x3320646e,0x79622d32,0x6b206574,*words,counter&MASK32,counter>>32,0,0];x=initial.copy()
  for _ in range(4):
   qr(x,0,4,8,12);qr(x,1,5,9,13);qr(x,2,6,10,14);qr(x,3,7,11,15)
   qr(x,0,5,10,15);qr(x,1,6,11,12);qr(x,2,7,8,13);qr(x,3,4,9,14)
  block=[(a+b)&MASK32 for a,b in zip(x,initial)]
  result.extend(block[i]|(block[i+1]<<32) for i in range(0,16,2));counter+=1
 return result[:count]

def classify(gene,id_,family):
 if id_ in family:return 'focal_family'
 if gene['lineage']:return 'older_prune_relative'
 return 'other_member_'+str(gene['member'])

def generated_from_witness(witness,rid,spec):
 require(spec['n_vars']==2 and not spec['guards'] and spec['pedges'] in [[],[[0,1]]],'supported archived primitive for independent matcher')
 labels={n['id']:n['label'] for n in witness['nodes']};expected=[]
 def eligible(nodes):return all(l is None or labels[n]==l for n,l in zip(nodes,spec['var_labels']))
 if not spec['pedges']:
  for a in labels:
   for b in labels:
    if eligible([a,b]):expected.append(dict(rid=rid,priority=spec['priority'],nodes=[a,b],edges=[]))
 else:
  for edge in witness['edges']:
   if len(edge['nodes'])!=2:continue
   a,b=edge['nodes'];perms=[[a,b]] if a==b else [[a,b],[b,a]]
   for nodes in perms:
    if eligible(nodes):expected.append(dict(rid=rid,priority=spec['priority'],nodes=nodes,edges=[edge['id']]))
 return sorted(expected,key=lambda b:(-b['priority'],rid,sorted(b['nodes'])))

def check_trace(trace,row,genes,family,cfg):
 require(trace['continuation_step']==row['continuation_step'] and trace['world_step_before']==row['absolute_step']-1,'trace/native clock')
 require(trace['tie_break']==cfg['tie_break']=='random' and trace['truncation_mode']==cfg['truncation_mode']=='canonical' and trace['cap']==cfg['match_cap_per_rule'],'unchanged operating point')
 roots={r['id']:r for r in row['roots']};table=trace['rules'];require([r['rid'] for r in table]==list(range(len(table))),'decode index order')
 require({r['root'] for r in table}=={i for i,r in roots.items() if r['active'] and r['executable']},'active decoded root coverage')
 for rule in table:
  root=roots[rule['root']];require(rule['class_hash']==genes[rule['root']]['class_hash'],'class attribution')
  g=root['gate'];require(rule['blocked']==(g['creation_blocked'] or g['node_ceiling_blocked'] or g['edge_ceiling_blocked']) and rule['enumeration_skipped']==g['enumeration_skipped'],'native gate identity')
 witness=trace['world_witness'];require(len(witness['nodes'])==row['world_nodes_before'] and len(witness['edges'])==row['world_edges_before'],'world witness counts');require(len({n['id'] for n in witness['nodes']})==len(witness['nodes']) and len({e['id'] for e in witness['edges']})==len(witness['edges']),'unique world witness IDs')
 labels=collections.Counter(n['label'] for n in witness['nodes']);require([labels[i] for i in range(cfg['n_labels'])]==row['label_counts_before'],'world witness labels')
 expected_generated=[]
 for rule in table:
  if not rule['blocked'] and not rule['enumeration_skipped']:expected_generated.extend(generated_from_witness(witness,rule['rid'],genes[rule['root']]['spec']))
 require([g['binding'] for g in trace['generated']]==expected_generated,'independently enumerated candidate identities and canonical order')
 pool=[];ranks=collections.Counter();counts={r['rid']:collections.Counter() for r in table};previous={}
 for item in trace['generated']:
  b=item['binding'];rid=b['rid'];require(type(rid) is int and 0<=rid<len(table),'candidate decode index');rule=table[rid]
  require(not rule['blocked'] and not rule['enumeration_skipped'],'no gated candidate leakage')
  require(b['priority']==genes[rule['root']]['spec']['priority'],'candidate priority')
  require(len(b['nodes'])==genes[rule['root']]['spec']['n_vars'] and len(b['edges'])==len(genes[rule['root']]['spec']['pedges']),'binding coverage')
  key=(-b['priority'],rid,sorted(b['nodes']));require(rid not in previous or previous[rid]<=key,'canonical binding order');previous[rid]=key
  rank=ranks[rid];require(item['canonical_rank']==rank,'canonical rank');ranks[rid]+=1;counts[rid]['generated']+=1
  submitted=cfg['match_cap_per_rule']==0 or rank<cfg['match_cap_per_rule']
  require((item['pool_index'] is not None)==submitted,'canonical cap membership')
  if submitted:require(item['pool_index']==len(pool),'original submitted input index');pool.append(b)
  else:counts[rid]['truncated']+=1
 resolution=trace['resolution'];key=cfg['seed']^(((row['absolute_step']-1)*0x9E3779B97F4A7C15)&MASK64);require(resolution['key']==key and resolution['native_acceptance_crosscheck'],'key/native crosscheck')
 words=draw_words(key,len(pool));order=sorted(range(len(pool)),key=lambda i:(-pool[i]['priority'],-float(words[i]),sorted(pool[i]['nodes'])))
 require(len(resolution['decisions'])==len(pool),'complete consideration denominator');owners={};metrics=collections.Counter();category_sets=collections.Counter();accepted_ids=set()
 for rank,(i,d) in enumerate(zip(order,resolution['decisions'])):
  require(d['rank']==rank and d['pool_index']==i and d['draw_word']==words[i] and d['draw']==float(words[i]),'independent keyed RNG/order')
  b=pool[i];blockers=[dict(node=n,winner_pool_index=owners[n]) for n in b['nodes'] if n in owners]
  require(d['blockers']==blockers and d['accepted']==(not blockers),'exact greedy ownership/decision')
  rid=b['rid'];root=table[rid]['root'];focal=root in family
  if not blockers:
   counts[rid]['accepted']+=1;accepted_ids.add(i)
   for n in b['nodes']:owners[n]=i
  else:
   counts[rid]['rejected']+=1
   if focal:
    categories=set()
    for blocker in blockers:
     winner=blocker['winner_pool_index'];require(winner in accepted_ids and blocker['node'] in pool[winner]['nodes'],'accepted overlapping winner')
     winnerroot=table[pool[winner]['rid']]['root'];category=classify(genes[winnerroot],winnerroot,family);categories.add(category);metrics['blocking_node_incidences/'+category]+=1
    for category in categories:metrics['rejected_with_blocker/'+category]+=1
    category_sets['+'.join(sorted(categories))]+=1
  if focal:metrics['considered']+=1;metrics['accepted' if not blockers else 'rejected']+=1
 for rule in table:
  count=counts[rule['rid']];fate=roots[rule['root']]['fate'];require(all(count[k]==fate[k] for k in ['generated','truncated','rejected']) and count['accepted']==fate['productive']+fate['sterile'],'all-root native fate closure')
  if rule['root'] in family:
   for k in ['generated','truncated','productive','sterile']:metrics[k]+=fate[k]
 require(sum(category_sets.values())==metrics['rejected'],'exclusive rejection category denominator')
 return metrics,category_sets

def audit_target(folder,entry):
 state=BoundedState(load(folder/'observed/initial.json'),'replace8');validate_allocator(load(folder/'observed/initial.json'),load(folder/'observed/initial_allocator.json'));founder=load(folder/'birth.json')['selection']['child'];family=descendants(state.genes,founder);totals=collections.Counter();sets=collections.Counter();windows={};completed=0;first_extinction=None;corruptions=[];samples={}
 with (folder/'observed/rows.jsonl').open() as native,(folder/'observed/conflicts.jsonl').open() as tracefile:
  for nline,tline in zip(native,tracefile,strict=True):
   row=json.loads(nline);trace=json.loads(tline);require(row['continuation_step']==entry['offset']+completed+1,'continuous target rows')
   m,s=check_trace(trace,row,state.genes,family,state.cfg);totals.update(m);sets.update(s)
   if m['rejected'] and 'rejection' not in samples:samples['rejection']=(copy.deepcopy(trace),copy.deepcopy(row),copy.deepcopy(state.genes),set(family),dict(state.cfg))
   if m['truncated'] and 'truncation' not in samples:samples['truncation']=(copy.deepcopy(trace),copy.deepcopy(row),copy.deepcopy(state.genes),set(family),dict(state.cfg))
   had=bool(family&state.balances.keys());state.step_row(row)
   for birth in row['births']:
    if birth['parent'] in family:family.add(birth['child'])
   if had and not(family&state.balances.keys()) and first_extinction is None:first_extinction=row['continuation_step']
   bucket=(row['continuation_step']-entry['offset']-1)//250;w=windows.setdefault(bucket,collections.Counter());w.update(m);completed+=1
 require(completed==entry['steps'] and family==descendants(state.genes,founder),'fixed horizon/ancestry');final=load(folder/'observed/final.json');state.match_snapshot(final);validate_allocator(final,load(folder/'observed/final_allocator.json'))
 # Deliberate damage to real receipts must fail, with fixed edits rather than retries.
 edits=[('candidate_node',lambda t:t['generated'][0]['binding']['nodes'].__setitem__(0,-1)),('witness_label',lambda t:t['world_witness']['nodes'][0].__setitem__('label',99)),('missing_witness_edge',lambda t:t['world_witness']['edges'].pop()),('draw_word',lambda t:t['resolution']['decisions'][0].__setitem__('draw_word',0)),('order',lambda t:t['resolution']['decisions'][0].__setitem__('pool_index',-1)),('missing_decision',lambda t:t['resolution']['decisions'].pop()),('key',lambda t:t['resolution'].__setitem__('key',0)),('root_identity',lambda t:t['rules'][0].__setitem__('root',-1)),('canonical_rank',lambda t:t['generated'][0].__setitem__('canonical_rank',999)),('missing_generated',lambda t:t['generated'].pop()),('accepted',lambda t:next(d for d in t['resolution']['decisions'] if d['blockers']).__setitem__('accepted',True)),('blocker_owner',lambda t:next(d for d in t['resolution']['decisions'] if d['blockers'])['blockers'][0].__setitem__('winner_pool_index',-1)),('missing_blocker',lambda t:next(d for d in t['resolution']['decisions'] if d['blockers'])['blockers'].pop()),('blocker_node',lambda t:next(d for d in t['resolution']['decisions'] if d['blockers'])['blockers'][0].__setitem__('node',-1))]
 for name,edit in edits:
  tr,row,genes,ids,cfg=copy.deepcopy(samples['rejection']);edit(tr)
  try:check_trace(tr,row,genes,ids,cfg)
  except (ValueError,KeyError,IndexError):corruptions.append(name)
  else:raise ValueError('Accepted corruption '+name)
 if 'truncation' in samples:
  tr,row,genes,ids,cfg=copy.deepcopy(samples['truncation']);next(g for g in tr['generated'] if g['pool_index'] is None)['pool_index']=0
  try:check_trace(tr,row,genes,ids,cfg)
  except ValueError:corruptions.append('truncated_candidate_submitted')
  else:raise ValueError('Accepted false cap membership')
 report=dict(**entry,passed=True,rows_audited=completed,founder=founder,first_extinction=first_extinction,final_focal_copies=len(family&state.balances.keys()),metrics=dict(totals),exclusive_rejection_category_sets=dict(sets),quarters=[dict(first_step=entry['offset']+i*250+1,last_step=entry['offset']+(i+1)*250,metrics=dict(w)) for i,w in sorted(windows.items())],adversarial_corruptions_rejected=corruptions,denominator_notes='rejected_with_blocker categories overlap; exclusive category sets partition rejected candidates; node incidences count bound-variable positions',receipts_sha256={str(f.relative_to(folder)):sha(f) for f in sorted(folder.rglob('*')) if f.is_file() and f.name!='attribution_audit.json'})
 (folder/'attribution_audit.json').write_text(json.dumps(report,indent=2)+'\n');return report

def main():
 p=argparse.ArgumentParser();p.add_argument('output',type=pathlib.Path);out=p.parse_args().output.resolve();manifest=load(out/'manifest.json');reports=[]
 for entry in manifest['targets']:
  report=audit_target(out/entry['name'],entry);reports.append(report);print(entry['name'],json.dumps(report['metrics']),flush=True)
 result=dict(passed=True,engineering_only=True,new_scientific_advancing_steps=0,rows_audited=sum(r['rows_audited'] for r in reports),auditor_sha256=sha(pathlib.Path(__file__)),keyed_rng_reconstructed_independently=True,records=reports)
 with (out/'audit.json').open('x') as f:json.dump(result,f,indent=2);f.write('\n')
if __name__=='__main__':main()
