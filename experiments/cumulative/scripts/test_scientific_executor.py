"""Rehashed corruptions must fail semantic auditing, not only hash checks."""
import copy
import json
from pathlib import Path
import sys
import tempfile
from audit_scientific_executor import recover
from audit_durability import read, PROFILE
from scientific_admission_cases import sha, parse


def run(out):
    raw=(out/'contract.json').read_bytes();c=parse(raw);route=c['routes'][0]
    values,_,_=read(out/'journals'/route['name']);binary=sha((out/'cumulative-scientific-executor').read_bytes())
    corruptions=[]
    def changed(label,fn):
        v=copy.deepcopy(values);fn(v);corruptions.append((label,v))
    changed('wrong_seed',lambda v:v[0]['route'].__setitem__('seed',2026101002))
    changed('wrong_binary',lambda v:v[0].__setitem__('binary_sha256','0'*64))
    changed('unfunded_intent',lambda v:v[2].__setitem__('work_bound',0))
    changed('wrong_counter',lambda v:v[3].__setitem__('native_rewrites',999))
    changed('missing_intent',lambda v:v.pop(2))
    changed('wrong_checkpoint',lambda v:v[3]['checkpoint']['state'].__setitem__('reservoir',999))
    changed('early_complete',lambda v:v.__setitem__(slice(2,None),[v[-1]]))
    changed('extra_result',lambda v:v.append(v[3]))
    rejected=[]
    for label,vs in corruptions:
        with tempfile.TemporaryDirectory(prefix='cumulative-executor-audit-') as tmp:
            root=Path(tmp);head=None
            for i,v in enumerate(vs):
                data=(json.dumps(dict(profile=PROFILE,sequence=i,previous_sha256=head,value=v),sort_keys=True,separators=(',',':'))+'\n').encode()
                (root/f'{i:08}.json').write_bytes(data);head=sha(data)
            try:recover(root,route,raw,binary)
            except (ValueError,AssertionError,KeyError):rejected.append(label)
            else:raise ValueError('corruption admitted: '+label)
    pending=recover(out/'journals/pending_intent',c['routes'][-1],raw,binary)
    assert pending['status']=='unknown' and pending['lower']==0 and pending['upper']>0
    return dict(rehashed_corruptions_rejected=rejected,pending_reservation_retained=True,native_rewrites=0)


if __name__=='__main__':print(json.dumps(run(Path(sys.argv[1])),indent=2,sort_keys=True))
