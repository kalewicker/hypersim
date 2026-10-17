#!/usr/bin/env python3
"""Independently reconstruct v13; require exact saved v12 ON controls."""
import argparse,copy,hashlib,json,pathlib,statistics,tarfile
from audit_selfmod_birth import State,load,rows,sha,require,new_totals,add_totals,score,SEEDS
ARMS=['faithful_on','faithful_off','prune_on','prune_off']
class SwitchedState(State):
    def __init__(self,snapshot,on): super().__init__(snapshot); self.on=on
    def finish(self,row,selection=None,evidence=None):
        super().finish(row,selection,evidence,fission_enabled=self.on)

def audit(out):
    manifest,summary,status,prov=(load(out/p) for p in ['manifest.json','summary.json','status.json','instrument_provenance.json'])
    pre=manifest['engineering_preflight']; seeds=[7,42] if pre else SEEDS; horizon=64 if pre else 2000
    require(manifest==dict(protocol='selfmod-v13-turnover-v1',engineering_preflight=pre,seeds=seeds,horizon=horizon,
        arms=ARMS,continuations_requested=len(seeds)*4,prefix_steps=0,input_panel='v12 precision-repaired',future_mutation=False,replication_threshold=6),'frozen protocol')
    require(sha(out/'selfmod-turnover')==prov['binary_sha256'] and sha(out/'source.tar.gz')==prov['source_archive_sha256'] and
        sha(out/'input_provenance.json')==prov['input_provenance_sha256'],'frozen hashes')
    with tarfile.open(out/'source.tar.gz') as tar:
        require(set(tar.getnames())==set(prov['sources_sha256']),'frozen source inventory')
        for file,digest in prov['sources_sha256'].items(): require(hashlib.sha256(tar.extractfile(file).read()).hexdigest()==digest,'source hash '+file)
        require(tar.extractfile('docs/selfmod_v13_turnover_preregistration.md').read()==(out/'preregistration.md').read_bytes(),'frozen contract')
    ip=load(out/'input_provenance.json'); inventory={str(p.relative_to(out)) for p in (out/'inputs').rglob('*') if p.is_file()}
    require(inventory==set(ip['inputs_sha256']),'input inventory')
    for file,digest in ip['inputs_sha256'].items(): require(sha(out/file)==digest,'input hash '+file)
    previous=load(out/'inputs/audit.json'); require(previous['audit_passed'] and load(out/'inputs/status.json')['state']=='complete','validated input')
    if not pre: require(load(out/'inputs/replay_verification.json')['passed'],'prior precision replay')
    evidence=load(out/'inputs/v11_selected_mutation.json'); records=[]; suppression={}; births={}
    for seed in seeds:
        require(sha(out/f'inputs/seed_{seed}/fork.json')==previous['evidence_sha256'][f'seed_{seed}/fork.json'],'prior audited fork')
        fork=load(out/f'inputs/seed_{seed}/fork.json')
        for arm in ARMS:
            genotype,mode=arm.split('_'); on=mode=='on'; path=out/f'seed_{seed}'/arm
            input_path=out/f'inputs/seed_{seed}'/genotype
            for file in ['birth.json','rows.jsonl','score.json','final.json']:
                require(sha(input_path/file)==previous['evidence_sha256'][f'seed_{seed}/{genotype}/{file}'],'prior audited receipt '+file)
            birth=load(path/'birth.json'); selected=birth['selection']
            require(sha(path/'birth.json')==sha(input_path/'birth.json'),'exact shared paid birth')
            require(selected['arm']==genotype and selected['executable'] and not selected['cancelled'],'selected genotype')
            base=State(fork['checkpoint']); base.finish(birth['row'],selected,evidence); base.match_snapshot(birth['initial'])
            require(birth['initial']['cfg']==fork['checkpoint']['cfg'] and birth['initial']['cfg']['steps']==horizon,'no parameter change')
            cfg=base.cfg
            require(cfg['rent']==1 and cfg['income_pool']==12 and cfg['repro_threshold']==100 and cfg['birth_cost']==50
                and cfg['creation_refill']==1 and cfg['match_cost_coeff']==0 and cfg['tie_break']=='random'
                and all(cfg[k]==0 for k in ['mu','abio','p_dup','p_del']),'declared operating point')
            branch=SwitchedState(birth['initial'],on); totals=new_totals(); completed=0; eligible_steps=0; count=0
            for row in rows(path/'rows.jsonl'):
                completed+=1; require(row['continuation_step']==completed,'fixed denominator/order')
                branch.object(row)
                eligible_steps+=any(s['capital']>=cfg['repro_threshold'] for s in branch.balances.values()) and len(branch.balances)<cfg['rule_cap']
                branch.finish(row); add_totals(totals,row,selected,horizon); count+=len(row['births'])
            require(completed==horizon,'incomplete continuation'); reconstructed=score(branch,selected,totals,completed)
            require(reconstructed==load(path/'score.json'),'independent verdict'); branch.match_snapshot(load(path/'final.json'))
            if not on: require(count==0 and len(branch.genes)==len(base.genes) and branch.seq==base.seq,'OFF births/genealogy leak')
            else:
                for file in ['rows.jsonl','score.json','final.json']: require(sha(path/file)==sha(input_path/file),'ON exact v12 control '+file)
            suppression[f'{seed}/{arm}']=eligible_steps; births[f'{seed}/{arm}']=count
            records.append(dict(seed=seed,arm=arm,score=reconstructed))
    steps=len(records)*horizon
    require(summary==dict(protocol=manifest['protocol'],engineering_preflight=pre,results=records,continuations_completed=len(records),
        scientific_advancing_steps=steps,new_off_condition_steps=steps//2,repeated_on_control_steps=steps//2,shared_prefix_steps=0,
        on_controls_byte_identical=True,duplicates_pooled=False),'summary reconstruction')
    require(status==dict(state='complete',continuations_completed=len(records),scientific_advancing_steps=steps),'completed status')
    aggregates={}
    for arm in ARMS:
        scores=[r['score'] for r in records if r['arm']==arm]
        rates=[s['lineage_per_copy_rate'] for s in scores if s['lineage_per_copy_rate'] is not None]
        aggregates[arm]=dict(continuations=len(scores),established=sum(s['bounded_establishment'] for s in scores),
            replicated_establishment=None if pre else sum(s['bounded_establishment'] for s in scores)>=6,
            lineage_alive=sum(s['final_lineage_copies']>0 for s in scores),child_alive=sum(s['totals']['child_death_step'] is None for s in scores),
            parent_alive=sum(s['parent_final_capital'] is not None for s in scores),cycle_persistent=sum(s['whole_cycle_persistent'] for s in scores),
            enumeration_censored=sum(s['totals']['enumeration_skips']>0 for s in scores),resource_gated=sum(s['totals']['resource_gates']>0 for s in scores),
            mean_per_copy_rate=statistics.mean(rates) if rates else None)
    paired=[]
    for seed in seeds:
        for genotype in ['faithful','prune']:
            a=next(r['score'] for r in records if r['seed']==seed and r['arm']==genotype+'_on')
            b=next(r['score'] for r in records if r['seed']==seed and r['arm']==genotype+'_off')
            paired.append(dict(seed=seed,genotype=genotype,on_established=a['bounded_establishment'],off_established=b['bounded_establishment'],
                on_alive=a['final_lineage_copies']>0,off_alive=b['final_lineage_copies']>0,
                on_net_income=a['totals']['late_lineage_income']-a['totals']['late_lineage_rent'],
                off_net_income=b['totals']['late_lineage_income']-b['totals']['late_lineage_rent'],
                on_per_copy_rate=a['lineage_per_copy_rate'],off_per_copy_rate=b['lineage_per_copy_rate']))
    return dict(audit_passed=True,engineering_preflight=pre,continuations_completed=len(records),scientific_advancing_steps=steps,
        new_off_condition_steps=steps//2,repeated_on_control_steps=steps//2,on_controls_byte_identical=True,
        aggregates=aggregates,paired=paired,eligible_steps=suppression,future_birth_counts=births,
        auditor_sha256=sha(pathlib.Path(__file__)),records=records,
        scope='Independent money, allowance, genealogy, gate, eligibility, RNG counts and verdict reconstruction; native matcher/effect/class hash/ChaCha checked through native tests and exact replay.',
        evidence_sha256={str(p.relative_to(out)):sha(p) for p in sorted(out.rglob('*')) if p.is_file() and p.name not in ['audit.json','replay_verification.json']})

def adversarial(out):
    seed=load(out/'manifest.json')['seeds'][0]; rejected=[]
    for arm in ['faithful_on','faithful_off']:
        birth=load(out/f'seed_{seed}/{arm}/birth.json'); row=next(rows(out/f'seed_{seed}/{arm}/rows.jsonl'))
        for label,edit in [
            ('rent',lambda r:r['roots'][0].__setitem__('cost',999)),
            ('fate',lambda r:r['roots'][0]['fate'].__setitem__('productive',999)),
            ('cash',lambda r:r['ledger'].__setitem__('free',999)),
            ('budget',lambda r:r['stocks'][0].__setitem__('budget',999)),
            ('lineage',lambda r:r['roots'][0].__setitem__('lineage',not r['roots'][0]['lineage'])),
            ('rng',lambda r:r.__setitem__('rng_word_pos',r['rng_word_pos']+2)),
            ('missing_root',lambda r:r['roots'].pop())]:
            damaged=copy.deepcopy(row);edit(damaged)
            try: SwitchedState(birth['initial'],arm.endswith('_on')).step_row(damaged)
            except (ValueError,KeyError): rejected.append(arm+'/'+label)
            else: raise ValueError('accepted damaged '+arm+'/'+label)
    # Force an eligible parent at an OFF phase. Absence is correct; any birth is a leak.
    birth=load(out/f'seed_{seed}/faithful_off/birth.json'); test=SwitchedState(birth['initial'],False)
    row=next(rows(out/f'seed_{seed}/faithful_off/rows.jsonl'));test.object(row)
    parent=min(test.balances); test.balances[parent]['capital']=120
    damaged=copy.deepcopy(row); damaged['births']=[dict(parent=parent)]
    try: test.finish(damaged)
    except ValueError: rejected.append('off_injected_birth')
    else: raise ValueError('OFF accepted a birth')
    # Native ON eligibility must remain enforced, even when the recorded births are missing.
    test=SwitchedState(birth['initial'],True);test.object(row);test.balances[parent]['capital']=120
    damaged=copy.deepcopy(row);damaged['births']=[]
    try: test.finish(damaged)
    except ValueError: rejected.append('on_missing_eligible_birth')
    else: raise ValueError('ON skipped an eligible birth')
    return rejected

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('output',type=pathlib.Path);args=p.parse_args()
    report=audit(args.output); report['adversarial_corruptions_rejected']=adversarial(args.output)
    with (args.output/'audit.json').open('x') as f:json.dump(report,f,indent=2);f.write('\n')
    print(json.dumps({k:report[k] for k in ['audit_passed','continuations_completed','scientific_advancing_steps','aggregates','paired','adversarial_corruptions_rejected']},indent=2))
if __name__=='__main__':main()
