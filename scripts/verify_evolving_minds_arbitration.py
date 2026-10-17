#!/usr/bin/env python3
"""Independent native birth-candidate, conflict and allocation audit."""
import copy,json,hashlib,tomllib,argparse,importlib.util,sys
from pathlib import Path
PROFILE='evolving-minds-d0-birth-arbitration-v1'
SHA='52a8442dd42eae4c06d371f28c8f03f3cf3a3ef5a8d0a1e4f136622ea7e5fc8b'
OFFER,LOCAL=242017,242018
LENGTH,PARTS=242012,242013
def load(path,name):
 spec=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(spec);old=sys.dont_write_bytecode
 try:sys.dont_write_bytecode=True;spec.loader.exec_module(m)
 finally:sys.dont_write_bytecode=old
 return m
def boundary(status,req):
 req(status['profile']==PROFILE and status['development_only'] is True and status['engineering_execution_complete'] is True,'arbitration completion')
 req(all(status[k] is False for k in ['scientific_run_launched','gate_pass','training_admitted','evolution_enabled','reproductive_fairness_accepted']),'arbitration research boundary')
def verify_cell(directory,c,d,s,base,recipe):
 req=c.require;record=json.loads((directory/'result.json').read_text());req(record['profile']==PROFILE and record['recipe']==recipe,'arbitration recipe')
 rules={r['name']:r for r in tomllib.loads(base.decode())['rules']};frames=record['frames'];req(len(frames)==5,'arbitration complete native frames');nodes,edges=c.graphs(frames[0]);initial=copy.deepcopy((nodes,edges));rot=recipe['rotation'];home=[rot,(rot+(4 if recipe['topology']=='Disjoint' else 2))%8];targets=[(rot+1)%8]+([(rot+5)%8] if recipe['topology']=='Disjoint' else []);local=recipe['policy']=='Local';sites={c.value(n,241009,'Int'):h for h,n in nodes.items() if n['label']==c.SITE};counters=[h for h,n in nodes.items() if n['label']==c.COUNTER];req(len(counters)==(0 if local else 1),'arbitration policy counter');counter=counters[0] if counters else None
 req(set(sites)==set(range(8)) and len(nodes)==32+(0 if local else 1) and c.total(nodes)==384.0,'arbitration initial scaffold')
 class Ref(d.Reference):
  def material(self):return 2.125
 p={'instructions':[dict(id=100,op='Halt',slot=0,source=0,initial=False)]};table=s.schedule(17,'Informative');refs={1:Ref(p,table,64.0,c),2:Ref(p,table,64.0,c)};root_handles={c.value(n,c.BID,'Int'):h for h,n in nodes.items() if n['label']==c.ROOT};ancestry={1:dict(parent=0,site=home[0]),2:dict(parent=0,site=home[1])};frontiers={i:3+i for i in range(8)};offers=[];accepted=[];child_ids=[]
 def material():
  expected=set();supported=set(sites.values())|({counter} if counter else set());occupied={a['site'] for a in ancestry.values()};key=lambda l,a,b:(l,tuple(sorted([a,b])))
  for i,h in sites.items():
   props={c.ENERGY:32.0,d.CLOCK:0,241009:i,d.RESOURCE:False,d.CUE:False,d.COIN:False,c.OCCUPIED:i in occupied,OFFER:i in targets}
   if local:props[LOCAL]=frontiers[i]
   req(nodes[h]['label']==c.SITE and nodes[h]['energy']==0.0 and dict(nodes[h]['properties'])=={k:c.typed(v) for k,v in props.items()},'arbitration physical site/offer/frontier')
   expected.add(key(c.NEIGHBOR,h,sites[(i+1)%8]))
  if counter:req(dict(nodes[counter]['properties'])=={c.NEXT:c.typed(len(ancestry)+1)} and nodes[counter]['energy']==0.0,'arbitration global ID frontier')
  for bid,r in refs.items():
   h=root_handles[bid];site=sites[ancestry[bid]['site']];members={x for x,n in nodes.items() if n['label'] not in [c.ROOT,c.SITE,c.COUNTER] and c.value(n,c.OWNER,'Int')==bid};subset=members|{h,site};supported|=subset;small={k:copy.deepcopy(nodes[k]) for k in subset};es={k:copy.deepcopy(e) for k,e in edges.items() if all(c.handle(n) in subset for n in e['nodes'])}
   req(c.value(nodes[h],c.PARENT,'Int')==ancestry[bid]['parent'] and c.value(nodes[h],LENGTH,'Int')==1 and c.value(nodes[h],PARTS,'Int')==11,'arbitration common seed and ancestry')
   small[h]['properties']=[v for v in small[h]['properties'] if v[0] not in [c.PARENT,LENGTH,PARTS]];c.set_property(small[h],c.BID,c.typed(1));small[site]['properties']=[v for v in small[site]['properties'] if v[0] not in [c.OCCUPIED,OFFER,LOCAL]];c.set_property(small[site],241009,c.typed(0));r.site=32.0
   for m in members:c.set_property(small[m],c.OWNER,c.typed(1))
   d.material(small,es,r,c);expected|={c.edge_key(e) for e in es.values()}
  req(set(nodes)==supported and len(expected)==len(edges) and {c.edge_key(e) for e in edges.values()}==expected,'arbitration full material/relations')
  req(len(nodes)<=128 and len(edges)<=384,'arbitration material bounds')
 req(frames[0]['step']==frames[0]['tick']==0 and frames[0]['phase'] is None and not frames[0]['events'] and all(n['generation']==0 for n in nodes.values()),'arbitration initial clock')
 material()
 for index,name in enumerate(['development_halt','maintenance_full','maintenance_full','birth_local' if local else 'birth_global']):
  frame=frames[index+1];req(frame['step']==index+1 and frame['tick']==0 and frame['phase'] is None,'arbitration pass clock');events=frame['events']
  if index<3:
   expected=[name,name];phase=11 if index==0 else 14
   for bid in [1,2]:req(refs[bid].step(phase,0)==[name],'arbitration native paid warmup')
   req(len(events)==2 and all(e['rule']==name for e in events),'arbitration mandatory paid warmup coverage')
  else:
   for h,n in nodes.items():
    if n['label']!=c.ROOT:continue
    bid=c.value(n,c.BID,'Int');home_handle=sites[ancestry[bid]['site']];gene=next(g for g,gn in nodes.items() if gn['label']==c.GENE and c.value(gn,c.OWNER,'Int')==bid)
    req(refs[bid].status==2 and refs[bid].age==2 and refs[bid].reserve==58.75,'arbitration equal eligibility')
    for target in targets:
     if (target-ancestry[bid]['site'])%8 not in [1,7]:continue
     binding={'r':h,'site':home_handle,'target':sites[target],'g0':gene}
     if counter:binding['counter']=counter
     bound=[binding[v] for v in rules[name]['lhs']['nodes']];offers.append(dict(parent=bid,target=target,bound=bound,canonical_key=sorted(c.rank(x) for x in bound),physical_conflicts=[],counter_only_conflicts=[],accepted=False))
   for i,o in enumerate(offers):
    for j,v in enumerate(offers):
     if i==j:continue
     common=set(o['bound'])&set(v['bound'])
     if common-({counter} if counter else set()):o['physical_conflicts'].append(v['parent'])
     elif common:o['counter_only_conflicts'].append(v['parent'])
   engaged=set();winners=[]
   for o in sorted(offers,key=lambda x:x['canonical_key']):
    if not set(o['bound'])&engaged:o['accepted']=True;engaged|=set(o['bound']);winners.append(o)
   req(len(offers)==2 and len(events)==len(winners) and all(e['rule']==name and list(map(c.handle,e['bound']))==o['bound'] for e,o in zip(events,winners)),'arbitration exact candidate conflict winners')
  engaged=set()
  for event in events:
   bound=set(map(c.handle,event['bound']));req(not engaged&bound,'arbitration conflicting rewrites');engaged|=bound;binding=c.apply(nodes,edges,event,rules[event['rule']],index)
   if index==3:
    offer=next(o for o in offers if o['bound']==list(map(c.handle,event['bound'])));parent=offer['parent'];target=offer['target'];bid=frontiers[target] if local else len(ancestry)+1;refs[parent].reserve-=16.0;refs[bid]=Ref(p,table,12.0,c);root_handles[bid]=binding['child'];ancestry[bid]=dict(parent=parent,site=target);accepted.append(parent);child_ids.append(bid)
    if local:frontiers[target]+=8
  material();req(c.graphs(frame)==(nodes,edges),'arbitration exact full frame replay');req(c.total(nodes)==384.0-sum(r.dissipated for r in refs.values())-4.0*len(accepted),'arbitration paid conservation')
  stats=frame['stats'];req(stats['accepted_rewrites']==len(events) and stats['property_effects_skipped']==0 and stats['property_effects_applied']==sum(len(rules[e['rule']]['rhs']['property_effects']) for e in events),'arbitration event/stat coverage')
  for field,key in [('nodes_created','created'),('nodes_deleted','deleted'),('edges_created','edges_created'),('edges_deleted','edges_deleted')]:req(stats[field]==sum(len(e[key]) for e in events),'arbitration material event stats')
  req(0<=frame['match_work']<=1000000,'arbitration matcher budget')
 expected_offers=copy.deepcopy(offers)
 for o in expected_offers:o['bound']=[{'idx':h[0],'version':h[1]} for h in o['bound']]
 req(record['offers']==expected_offers and record['accepted_parents']==accepted and record['child_ids']==child_ids,'arbitration offer inventory/IDs/conflict classification')
 req(record['initial_energy']==384.0 and record['warmup_dissipation']==10.5 and record['copying_dissipation']==4.0*len(accepted) and record['final_energy']==c.total(nodes) and record['residual']==0.0,'arbitration costs and final total')
 req(len(accepted)==(2 if local and recipe['topology']=='Disjoint' else 1),'arbitration diagnostic concurrency criterion')
 status=json.loads((directory/'run_status.json').read_text());boundary(status,req);req(status['recipe']==recipe and status['accepted_parents']==accepted,'arbitration cell status');m=json.loads((directory/'manifest.json').read_text());req(m['state']=='Completed' and m['max_steps']==m['final_step']==4 and m['seed']==17,'arbitration cell completion')
 return dict(offers_verified=2,accepted_parents=accepted,child_ids=child_ids,counter_only_conflict_pairs=sum(len(o['counter_only_conflicts']) for o in offers)//2,physical_conflict_pairs=sum(len(o['physical_conflicts']) for o in offers)//2,final_energy=c.total(nodes))
def verify(root):
 root=Path(root);c=load(root/'replay.py','arbitration_replay');d=load(root/'d0_reference.py','arbitration_d0');s=load(root/'schedule_reference.py','arbitration_schedule');req=c.require;req(not root.is_symlink() and not any(p.is_symlink() for p in root.rglob('*')),'arbitration evidence path');req(sum(p.stat().st_size for p in root.rglob('*') if p.is_file())<=16*1024*1024,'arbitration output budget');base=(root/'base_rules.toml').read_bytes();req(hashlib.sha256(base).hexdigest()==SHA,'arbitration law identity');results={};expected_names=set();expected_order=[]
 for policy in ['Global','Local']:
  for topology in ['Disjoint','Competing']:
   for rotation in range(8):
    for reversed in [False,True]:
     recipe=dict(policy=policy,topology=topology,rotation=rotation,reversed=reversed);name=f'{policy}_{topology}_r{rotation}_{"reverse" if reversed else "forward"}'.lower();expected_names.add(name);expected_order.append(name);results[name]=verify_cell(root/name,c,d,s,base,recipe)
 req(json.loads((root/'summary.json').read_text())==[json.loads((root/name/'run_status.json').read_text()) for name in expected_order],'arbitration complete summary')
 req({p.name for p in root.iterdir() if p.is_dir()}==expected_names,'arbitration fixed 64-cell denominator');boundary(json.loads((root/'run_status.json').read_text()),req);m=json.loads((root/'manifest.json').read_text());req(m['state']=='Completed' and m['max_steps']==m['final_step']==256,'arbitration panel completion')
 sources=json.loads((root/'sources.json').read_text())
 for f,p in [('checker.py','scripts/verify_evolving_minds_arbitration.py'),('base_rules.toml','rules/evolving_minds/arbitration_v1.toml'),('contract.md','experiments/evolving_minds/contracts/d0_lifecycle_fairness_v1.md'),('replay.py','scripts/verify_evolving_minds.py'),('d0_reference.py','scripts/verify_evolving_minds_d0.py'),('schedule_reference.py','scripts/verify_evolving_minds_choice.py')]:req((root/f).read_text()==sources[p],'arbitration frozen source binding')
 return dict(profile=PROFILE,audit_pass=True,development_only=True,scientific_run_launched=False,gate_pass=False,training_admitted=False,evolution_enabled=False,reproductive_fairness_accepted=False,cells_verified=64,native_advances_verified=256,source_sha256={p:hashlib.sha256(v.encode()).hexdigest() for p,v in sources.items()},cells=results)
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('directory',type=Path);print(json.dumps(verify(p.parse_args().directory),indent=2))
