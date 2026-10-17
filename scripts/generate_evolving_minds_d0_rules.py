#!/usr/bin/env python3
"""Generate generic D0 interpreter laws; no program-specific compiled behavior."""
from pathlib import Path
from generate_evolving_minds_choice_rules import emit

ROOT,SITE,SENSOR,GENE,CONNECTION,ACTUATOR=range(0xE1000000,0xE1000006)
STATE,SLOT=0xE1010000,0xE1010001
OCC,SOURCE,TARGET=0xE1000064,0xE100006A,0xE100006B
OWN,MAP,NEXT=range(0xE1010064,0xE1010067)
ENERGY,OWNER,OP,AGE,SIGNAL,BID,MAINT=1,240001,240002,240004,240005,240007,240010
CLOCK,RESOURCE,CUE,COIN,_,_,ATTEMPT,CHOICE=range(241001,241009)
CURSOR,STATUS,POSITION,IDENTITY,DEST,SRC,INITIAL,USED,INPUT,BUFFER,HAS_ACT=range(242001,242012)

def guard(v,k,op,x):return dict(var=v,property=k,op=op,value=x)
def compare(a,ka,op,b,kb):return dict(var_a=a,key_a=ka,op=op,var_b=b,key_b=kb)
def effect(v,k,op,x):return dict(var=v,key=k,op=op,value=x)
def cp(v,k,s,sk):return dict(var=v,key=k,op='copy_from',source_var=s,source_key=sk)
def rule(name,labels,links,guards=(),compares=(),effects=(),new=None,newlinks=(),priority=0):
    links=list(links); nodes=list(labels); created=new or {}
    return dict(name=name,priority=priority,
        lhs=dict(nodes=nodes,edges=[list(e) for _,e in links],node_labels=labels,edge_labels={str(i):k for i,(k,_) in enumerate(links)},
                 var_guards=[dict(type='vars_distinct',vars=nodes)] if len(nodes)>1 else [],node_property_guards=list(guards),node_property_compares=list(compares)),
        rhs=dict(nodes=nodes+list(created),edges=[list(e) for _,e in links+list(newlinks)],edge_labels={str(i):k for i,(k,_) in enumerate(links+list(newlinks))},node_labels=created,property_effects=list(effects)))

def derive():
    laws=[]
    laws.append(rule('environment_influx',{'site':SITE},[],[guard('site',ENERGY,'lte',56.0)],effects=[effect('site',ENERGY,'add',8.0)]))
    for slot,key in [(0,CUE),(1,COIN)]:
        laws.append(rule(f'sense_{slot}',{'r':ROOT,'site':SITE,'s':SENSOR},[(OCC,('r','site')),(OWN,('r','s'))],
                         [guard('s',DEST,'eq',slot)],[compare('s',OWNER,'eq','r',BID)],[cp('s',SIGNAL,'site',key)]))
    # Slot passes prevent native node conflicts from dropping adjacent reads.
    # Every pass reads old SIGNAL; only the subsequent commit publishes buffers.
    for slot in range(2,8):
        for slabel in [SENSOR,STATE,ACTUATOR]:
            for dlabel in [STATE,ACTUATOR]:
                laws.append(rule(f'prepare_{slot}_{slabel}_{dlabel}',{'s':slabel,'c':CONNECTION,'d':dlabel},[(SOURCE,('s','c')),(TARGET,('c','d'))],
                                 guards=[guard('d',DEST,'eq',slot)],compares=[compare('s',OWNER,'eq','c',OWNER),compare('d',OWNER,'eq','c',OWNER)],effects=[cp('d',BUFFER,'s',SIGNAL)]))
    for label in [STATE,ACTUATOR]:
        laws.append(rule(f'commit_{label}',{'d':label},[],[guard('d',INPUT,'eq',True)],effects=[cp('d',SIGNAL,'d',BUFFER)]))
    common_g=[guard('r',STATUS,'lte',1),guard('r',ENERGY,'gte',1.0)]
    common_c=[compare('g',OWNER,'eq','r',BID),compare('g',POSITION,'eq','r',CURSOR)]
    advance=[effect('r',ENERGY,'subtract',1.0),effect('r',CURSOR,'add',1),effect('r',STATUS,'set',0)]
    for opcode,label in [(0,STATE),(1,ACTUATOR)]:
        guards=common_g+[guard('g',OP,'eq',opcode),guard('slot',USED,'eq',False)]
        if opcode==1:guards += [guard('r',HAS_ACT,'eq',False)]
        fx=advance+[effect('slot',USED,'set',True),cp('p',OWNER,'r',BID),cp('p',DEST,'g',DEST),cp('p',SIGNAL,'g',INITIAL),cp('p',BUFFER,'g',INITIAL),effect('p',INPUT,'set',False),effect('r',MAINT,'add',1.0)]
        if opcode==1:fx += [effect('r',HAS_ACT,'set',True)]
        laws.append(rule(f'development_create_{opcode}',{'r':ROOT,'g':GENE,'slot':SLOT},[(OWN,('r','g')),(OWN,('r','slot'))],guards,
                         common_c+[compare('slot',OWNER,'eq','r',BID),compare('slot',DEST,'eq','g',DEST)],fx,{'p':label},[(OWN,('r','p')),(MAP,('slot','p'))],priority=1))
    for slabel in [SENSOR,STATE,ACTUATOR]:
        for dlabel in [STATE,ACTUATOR]:
            laws.append(rule(f'development_wire_{slabel}_{dlabel}',{'r':ROOT,'g':GENE,'s':slabel,'d':dlabel},[(OWN,('r','g')),(OWN,('r','s')),(OWN,('r','d'))],
                common_g+[guard('g',OP,'eq',2),guard('d',INPUT,'eq',False)],
                common_c+[compare('s',OWNER,'eq','r',BID),compare('d',OWNER,'eq','r',BID),compare('s',DEST,'eq','g',SRC),compare('d',DEST,'eq','g',DEST)],
                advance+[cp('c',OWNER,'r',BID),effect('d',INPUT,'set',True),effect('r',MAINT,'add',1.0)],{'c':CONNECTION},[(OWN,('r','c')),(SOURCE,('s','c')),(TARGET,('c','d'))],priority=1))
    laws.append(rule('development_halt',{'r':ROOT,'g':GENE},[(OWN,('r','g'))],common_g+[guard('g',OP,'eq',3)],common_c,
                     advance+[effect('r',STATUS,'set',2)],priority=1))
    laws.append(rule('development_fail',{'r':ROOT,'g':GENE},[(OWN,('r','g'))],common_g,common_c,
                     [effect('r',ENERGY,'subtract',1.0),effect('r',STATUS,'set',3)],priority=-1))
    laws.append(rule('development_wait',{'r':ROOT},[],[guard('r',STATUS,'eq',0),guard('r',ENERGY,'lt',1.0)],effects=[effect('r',STATUS,'set',1)]))
    laws.append(rule('attempt_commit',{'r':ROOT,'a':ACTUATOR},[(OWN,('r','a'))],
                     [guard('r',STATUS,'eq',2),guard('r',ATTEMPT,'eq',False),guard('r',ENERGY,'gte',1.0)],
                     [compare('a',OWNER,'eq','r',BID)],[effect('r',ENERGY,'subtract',1.0),cp('r',CHOICE,'a',SIGNAL),effect('r',ATTEMPT,'set',True)]))
    for success in [True,False]:
        guards=[guard('r',ATTEMPT,'eq',True)]
        fx=[effect('r',ATTEMPT,'set',False)]
        if success:
            guards += [guard('site',ENERGY,'gte',8.0),guard('r',ENERGY,'lte',1000.0)]
            fx += [effect('site',ENERGY,'subtract',8.0),effect('r',ENERGY,'add',8.0)]
        laws.append(rule('settle_success' if success else 'settle_failure',{'r':ROOT,'site':SITE},[(OCC,('r','site'))],guards,
                         [compare('r',CHOICE,'eq','site',RESOURCE)] if success else [],fx,priority=1 if success else 0))
    for full in [True,False]:
        fx=[dict(var='r',key=ENERGY,op='subtract_from_property',source_var='r',source_key=MAINT)] if full else [effect('r',ENERGY,'set',0.0)]
        laws.append(rule('maintenance_full' if full else 'maintenance_exhausted',{'r':ROOT},[],compares=[compare('r',ENERGY,'gte' if full else 'lt','r',MAINT)],effects=fx+[effect('r',AGE,'add',1)]))
    return '# Generic bounded D0 development laws, v1.\n'+'\n\n'.join('[[rules]]\n'+'\n'.join(emit(r,'rules')) for r in laws)+'\n'

if __name__=='__main__':
    import hashlib
    target=Path(__file__).resolve().parents[1]/'rules/evolving_minds/d0_v1.toml'
    text=derive();target.write_text(text);print(hashlib.sha256(text.encode()).hexdigest())
