"""Independent verification of semantic target evidence and exact material deletion."""
from audit_selfmod_birth import require
from verify_selfmod_second_edit_interruption import native_order

def verify_target(edited,faithful,target,contract):
 child=edited['selection']['child'];original=contract['third_edit']['parameters'];before=faithful['initial'];after=edited['initial'];order=native_order(before,child)
 require(sorted(n['variable'] for n in order)==[1],'exact one remaining constraint')
 index=next(i for i,n in enumerate(order) if n['variable']==1);effective=list(original);effective[0]=index
 require(target==dict(operator=3,target_variable=1,child=child,native_pnode_order=order,target_index=index,original_parameters=original,effective_parameters=effective,scheduler_draws=0),'exact semantic target receipt')
 require(before.keys()==after.keys(),'snapshot field coverage')
 for key in before.keys()-{'rules','genes'}:require(before[key]==after[key],'unchanged paid snapshot '+key)
 require(len(before['genes'])==len(after['genes']),'same genealogy coverage')
 for f,e in zip(before['genes'],after['genes']):
  if f['id']!=child:require(f==e,'all sibling and older genes unchanged')
  else:
   require(e['id']==child and e['spec']==contract['composite_spec'] and e['class_hash']==contract['composite_class_hash'],'exact semantic composite')
   for key in f.keys()-{'spec','class_hash'}:require(f[key]==e[key],'unchanged child metadata '+key)
 node=order[index]['id'];idx=node&0xffffffff
 require(len(before['rules']['nodes'])==len(after['rules']['nodes']) and len(before['rules']['edges'])==len(after['rules']['edges']),'no hidden allocated material')
 for i,(f,e) in enumerate(zip(before['rules']['nodes'],after['rules']['nodes'])):
  if i==idx:require(f['value'] is not None and e['value'] is None,'target constraint removed')
  else:require(f==e,'all other material nodes unchanged')
 removed=[]
 for i,(f,e) in enumerate(zip(before['rules']['edges'],after['rules']['edges'])):
  if f!=e:
   require(f['value'] is not None and e['value'] is None and f['value']['label']==2002,'only target role edge removed')
   ids=[(n['version']<<32)|n['idx'] for n in f['value']['nodes']];require(set(ids)=={child,node},'exact deleted role edge');removed.append(i)
 require(len(removed)==1,'one material role edge deleted')
 return dict(native_variable_order=[n['variable'] for n in order],effective_parameter_zero=index,exact_material_edit_verified=True)

from audit_selfmod_bounded import BoundedState
from selfmod_second_edit_common import descendants
import copy
class ThirdState(BoundedState):
 def finish_selected(self,row,selected,arm,contract,description,founder):
  require(founder in self.genes,'second founder exists');anchor=self.genes[founder]
  require(anchor['member']==0 and anchor['lineage'] and anchor['executable'] and anchor['spec']==contract['second_composite_spec'] and anchor['class_hash']==contract['second_composite_class_hash'],'founder is the twice-edited genotype, not an older ancestor')
  ids=descendants(self.genes,founder);eligible=[id_ for id_ in sorted(self.balances) if id_ in ids and self.balances[id_]['capital']>=100]
  require(eligible and selected['parent']==eligible[0],'first funded SECOND-family parent; cousins excluded')
  birth=next((b for b in row['births'] if b['child']==selected['child']),None);require(birth is not None and birth['parent']==eligible[0],'actual third child')
  parent=self.genes[eligible[0]];gene=birth['gene'];require(parent['member']==0 and parent['lineage'] and parent['spec']==contract['second_composite_spec'] and parent['class_hash']==contract['second_composite_class_hash'],'exact twice-edited parent')
  expected=contract['composite_spec'] if arm=='edited' else parent['spec'];hash_=description['composite_class_hash'] if arm=='edited' else parent['class_hash'];require(gene['spec']==expected and gene['class_hash']==hash_ and gene['executable'],'exact third genotype')
  require(selected==dict(arm='prune' if arm=='edited' else 'faithful',parent=birth['parent'],child=birth['child'],executable=True,changed_class=arm=='edited',cancelled=False),'third selection')
  faithful=copy.deepcopy(row);target=next(b for b in faithful['births'] if b['child']==selected['child']);target['gene']=dict(gene,spec=parent['spec'],class_hash=parent['class_hash'],executable=parent['executable']);super().finish(faithful);self.genes[selected['child']]=copy.deepcopy(gene)
