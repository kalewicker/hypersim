"""Zero-native corruption tests against authenticated committed journals."""
import copy,json,sys
from pathlib import Path
from audit_campaign_adapter import load,registry,recover_values,sha,read
p=Path(sys.argv[1]);c=load(p/'contract.json');reg=load(p/'registry.json');items=registry(reg,c)
checks=0

def rejected(fn):
    global checks
    try:fn()
    except (ValueError,KeyError,IndexError,TypeError,AssertionError):checks+=1;return
    raise AssertionError('corruption accepted')
for edit in [lambda x:x['assays'][-1].update(stages=[5]),lambda x:x['assays'][-1].update(id=0),lambda x:x['assays'][-1].update(role='unknown')]:
    bad=copy.deepcopy(reg);edit(bad);rejected(lambda:registry(bad,c))
name='assays_0';r=load(p/'inputs'/(name+'.json'));values,head,staging=read(p/'journals'/name);request_sha=sha((p/'inputs'/(name+'.json')).read_bytes())
mutations=[lambda v:v[2]['request'].update(id=1),lambda v:v[2]['native']['results'][0].update(score=999),lambda v:v[1].update(work_bound=19),lambda v:v[-1].update(work_total=0),lambda v:v[2]['native']['results'][0]['request'].update(role='sham'),lambda v:v.pop(2)]
for edit in mutations:
    bad=copy.deepcopy(values);edit(bad);rejected(lambda:recover_values(bad,head,staging,r,request_sha,c,items))
# Corrupt a padded alias, independently of original-role mappings.
bad=copy.deepcopy(values);next(v for v in bad if v.get('kind')=='result' and v['index']==11)['native']['results'][0]['request']['stages']=[0,1,2]
rejected(lambda:recover_values(bad,head,staging,r,request_sha,c,items))
name='stress_0';r=load(p/'inputs'/(name+'.json'));values,head,staging=read(p/'journals'/name);request_sha=sha((p/'inputs'/(name+'.json')).read_bytes());bad=copy.deepcopy(values);bad[1]['checkpoint']['state']['cells'][0]['state']['core']['transferred']=0
rejected(lambda:recover_values(bad,head,staging,r,request_sha,c,items))
print(json.dumps({'corruptions_rejected':checks,'native_rewrites':0}))
