"""Independent verification of semantic target evidence and exact material deletion."""
from audit_selfmod_birth import require
from verify_selfmod_second_edit_interruption import native_order

def verify_target(edited,faithful,target,contract):
 child=edited['selection']['child'];original=contract['second_edit']['parameters'];before=faithful['initial'];after=edited['initial'];order=native_order(before,child)
 require(sorted(n['variable'] for n in order)==[0,1],'two unique native constraints')
 index=next(i for i,n in enumerate(order) if n['variable']==0);effective=list(original);effective[0]=index
 require(target==dict(operator=3,target_variable=0,child=child,native_pnode_order=order,target_index=index,original_parameters=original,effective_parameters=effective,scheduler_draws=0),'exact semantic target receipt')
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
