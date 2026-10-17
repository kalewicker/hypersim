"""Immutable ancestry and native receipt reconstruction for the fresh 50000-step panel."""
import copy
from audit_selfmod_birth import load,rows,sha,require,new_totals,add_totals,score,canonical_spec
from audit_selfmod_bounded import BoundedState,capital,extra_new,extra_add,reproduction
from audit_selfmod_financing import window,add_window,finish_window,validate_allocator,slope
from audit_selfmod_durability import members,family_copies,mark_floor,durability

def descendants(genes,founder):
 require(founder in genes,'founder exists')
 known={founder:True};result=set()
 for id_ in genes:
  path=[];current=id_;seen=set()
  while current is not None and current not in known:
   require(current in genes and current not in seen,'immutable ancestry coverage/cycle');seen.add(current);path.append(current);current=genes[current]['parent']
  inherited=known.get(current,False)
  for item in path:known[item]=inherited
  if known.get(id_,False):result.add(id_)
 return result

def view(state,family_ids=None):
 if family_ids is None:return state
 result=copy.copy(state);result.genes={id_:dict(g,lineage=id_ in family_ids) for id_,g in state.genes.items() if id_ in state.balances}
 return result

def row_view(row,family_ids=None):
 if family_ids is None:return row
 return dict(row,roots=[dict(r,lineage=r['id'] in family_ids) for r in row['roots']],births=[dict(b,gene=dict(b['gene'],lineage=b['child'] in family_ids)) for b in row['births']])

def reconstruct_panel(directory,selected,horizon,focal_child,second_selected):
 second_founder=second_selected["child"]
 initial=load(directory/'initial.json');state=BoundedState(initial,'replace8');validate_allocator(initial,load(directory/'initial_allocator.json'))
 family_ids=descendants(state.genes,focal_child);second_ids=descendants(state.genes,second_founder)
 groups={key:dict(windows=[],segments=[]) for key in ['focal','second','original']}
 completed=0
 for raw in rows(directory/'rows.jsonl'):
  require(raw['continuation_step']==completed+1,'continuous fixed branch offset');local=completed%2000+1
  for key,g in groups.items():
   before=view(state,family_ids if key=='focal' else second_ids if key=='second' else None)
   if completed%1000==0:g['current']=window(completed+1,before);mark_floor(g['current'],before);g['current'].update(family_creation_spend=0,culture_creation_spend=0)
   if local==1:g['totals']=new_totals();g['extra']=extra_new()
   if local==1001:g['extra'].update(late_start_capital=capital(before),late_min_capital=capital(before),late_max_capital=capital(before),late_start_recorded=True)
  state.step_row(raw);completed+=1
  if family_ids is not None:
   for b in raw['births']:
    if b['parent'] in family_ids:family_ids.add(b['child'])
    if b['parent'] in second_ids:second_ids.add(b['child'])
  for key,g in groups.items():
   chosen=second_selected if key=='second' else selected
   ids=family_ids if key=='focal' else second_ids if key=='second' else None;current=view(state,ids);r=row_view(raw,ids);w=g['current'];mark_floor(w,current);add_window(w,r,current)
   w['family_creation_spend']+=sum(x['spend'] for x in r['roots'] if x['lineage']);w['culture_creation_spend']+=sum(x['spend'] for x in r['roots'])
   # Segment bookkeeping also needs genes of retired paying parents.
   bookkeeping=copy.copy(current);bookkeeping.genes=dict(current.genes)
   for admission in raw['bounded']['admissions']:
    id_=admission['parent'];gene=state.genes[id_];bookkeeping.genes[id_]=dict(gene,lineage=id_ in ids) if ids is not None else gene
   local_row=dict(r,continuation_step=local);add_totals(g['totals'],local_row,chosen,2000);extra_add(g['extra'],bookkeeping,local_row,2000)
   if completed%1000==0:g['windows'].append(finish_window(w,current))
   if local==2000:
    readonly=copy.copy(current);readonly.cfg=dict(current.cfg,steps=2000);legacy=score(readonly,chosen,g['totals'],2000);g['segments'].append(dict(first_step=completed-1999,last_step=completed,score=legacy,reproduction=reproduction(legacy,g['extra'])))
  if completed%1000==0:
   snapshot=load(directory/f'checkpoint_{completed}.json');state.match_snapshot(snapshot);require(snapshot['cfg']==initial['cfg'],'unaltered physical config');validate_allocator(snapshot,load(directory/f'checkpoint_{completed}_allocator.json'))
 require(completed==horizon and load(directory/'status.json')==dict(state='complete',offset=0,steps_completed=horizon),'complete fixed horizon')
 final=load(directory/'final.json');state.match_snapshot(final);validate_allocator(final,load(directory/'final_allocator.json'));require(final['cfg']==initial['cfg'],'fixed final config')
 require(family_ids==descendants(state.genes,focal_child) and second_ids==descendants(state.genes,second_founder),'streaming vs full immutable ancestry')
 result={}
 for key,g in groups.items():
  chosen=second_selected if key=='second' else selected
  current=view(state,family_ids if key=='focal' else second_ids if key=='second' else None)
  result[key]=dict(maintenance=panel_maintenance(chosen,g['windows'],completed==horizon),first10000_durability=durability(chosen,g['windows'][:10],len(g['windows'])>=10),windows=g['windows'],segments=g['segments'],final_family_copies=family_copies(current),final_member_copies=members(current),capital_mean_slope_per_1000=slope([w['mean_capital'] for w in g['windows'][2:]]))
 return result

def panel_maintenance(selected,windows,complete):
 for i,w in enumerate(windows):
  require(w['first_step']==1+1000*i and w['last_step']==1000+1000*i,'all third-birth windows in fixed order')
 measured=complete and len(windows)==50 and not any(w['enumeration_censored'] for w in windows)
 warmup=windows[:2];scored=windows[2:]
 blocks=[durability(selected,warmup+scored[i:i+8],complete and len(windows)==50) for i in range(0,len(scored),8)]
 reasons=sorted({x for b in blocks for x in b['failure_reasons']})
 if not measured:reasons=sorted(set(reasons)|{'incomplete_or_enumeration_censored'})
 if len(blocks)!=6:reasons=sorted(set(reasons)|{'missing_temporal_block'})
 return dict(maintained_success=not reasons,complete=complete,uncensored_measured=measured,failure_reasons=reasons,scored_windows=len(scored),warmup_steps=2000,renewed_warmup_steps=0,temporal_blocks=blocks)
