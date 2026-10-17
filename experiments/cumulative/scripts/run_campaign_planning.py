"""Freeze a zero-native campaign sizing boundary, preserving every earlier archive."""
import json,os,shutil,subprocess,sys,time
from pathlib import Path
from run_readiness import ROOT,PACKAGE,FALSE,digest,write,source_paths
from run_durability import custody as older_custody
from audit_campaign import evidence
PARENT=ROOT/'output/cumulative_durable_journal_readiness_v3_20261009'
PARENT_ROOT='8ae372d4e9f71cb34441453d2333dd524b0431b025028f20c7226c19f1709fe1'
FAILED=ROOT/'output/cumulative_campaign_planning_20261009'
FAILED_ROOT='bd30202bf798fc8f3416865a3b6288dea2c819dd955faac064cbda44b2f66b35'
def custody():
    older_custody()
    if digest(FAILED/"output_sha256.json")!=FAILED_ROOT:raise ValueError("retained planning failure root")
    failed_seal=json.loads((FAILED/"output_sha256.json").read_text());actual={str(p.relative_to(FAILED)) for p in FAILED.rglob("*") if p.is_file() and p!=FAILED/"output_sha256.json"}
    if actual!=set(failed_seal) or any(digest(FAILED/rel)!=h for rel,h in failed_seal.items()) or json.loads((FAILED/"manifest.json").read_text())["status"]!="failed":raise ValueError("retained planning failure custody/status")
    if digest(PARENT/'output_sha256.json')!=PARENT_ROOT:raise ValueError('durability parent root')
    seal=json.loads((PARENT/'output_sha256.json').read_text());actual={str(p.relative_to(PARENT)) for p in PARENT.rglob('*') if p.is_file() and p!=PARENT/'output_sha256.json'}
    if actual!=set(seal) or any(digest(PARENT/rel)!=h for rel,h in seal.items()):raise ValueError('durability parent full custody')
    m=json.loads((PARENT/'manifest.json').read_text())
    if m['status']!='complete' or not m['engineering_fixture_pass'] or any(m[f] is not False for f in FALSE):raise ValueError('parent engineering/scientific boundary')
    pins=json.loads((PARENT/'source_sha256.json').read_text())
    if any(digest(ROOT/rel)!=h for rel,h in pins.items()):raise ValueError('previous source changed; planning is additive only')
def run(out):
    start=time.monotonic();out.mkdir(parents=True,exist_ok=False);(out/'commands').mkdir();contract=json.loads((PACKAGE/'campaign_planning_contract.json').read_text());limits=contract['limits'];manifest={'profile':contract['profile'],'scope':contract['scope'],'status':'running','engineering_planning_pass':False,'native_rewrites':0,'whole_campaign_feasibility_pass':False,**FALSE};write(out/'manifest.json',manifest)
    def command(name,argv):
        remaining=limits['workflow_seconds']-(time.monotonic()-start)
        if remaining<=0:raise ValueError('planning workflow deadline')
        before=time.monotonic()
        try:r=subprocess.run(argv,cwd=ROOT,capture_output=True,text=True,timeout=remaining,env={**os.environ,'PYTHONDONTWRITEBYTECODE':'1'})
        except subprocess.TimeoutExpired as e:
            write(out/'commands'/(name+'.json'),{'argv':argv,'timed_out':True,'stdout':str(e.stdout or ''),'stderr':str(e.stderr or '')});raise
        write(out/'commands'/(name+'.json'),{'argv':argv,'exit_code':r.returncode,'seconds':time.monotonic()-before,'stdout':r.stdout,'stderr':r.stderr});print(json.dumps({'phase':name,'exit_code':r.returncode}),flush=True)
        if r.returncode:raise RuntimeError(r.stderr[-2000:])
        return r
    try:
        custody();pins={str(p.relative_to(ROOT)):digest(p) for p in source_paths()};write(out/'source_sha256.json',pins)
        for rel in pins:
            dest=out/'source_snapshots'/rel;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(ROOT/rel,dest)
        write(out/'contract.json',contract);root=out/'evidence';root.mkdir();shutil.copy2(PARENT/'output_sha256.json',root/'parent_output_sha256.json');copies={}
        fixed={'world_input.json':'inputs/world_complete.json','assay_input.json':'inputs/assay_complete.json','world_command.json':'commands/world_complete.json','assay_command.json':'commands/assay_complete.json','parent_manifest.json':'manifest.json'}
        for rel,original in fixed.items():shutil.copy2(PARENT/original,root/rel);copies[rel]=original
        for route in ['world_complete','assay_complete']:
            (root/route).mkdir()
            for file in sorted((PARENT/'journals'/route).iterdir()):
                rel=route+'/'+file.name;original='journals/'+rel;shutil.copy2(file,root/rel);copies[rel]=original
        write(root/'copy_map.json',copies);write(out/'evidence_summary.json',evidence(out));scripts=out/'source_snapshots/experiments/cumulative/scripts'
        command('plan',[sys.executable,'-B',str(scripts/'plan_campaign.py'),str(out/'contract.json'),str(out/'evidence_summary.json'),str(out/'plan.json')])
        r=command('independent_audit',[sys.executable,'-B',str(scripts/'audit_campaign.py'),str(out)]);audit=json.loads(r.stdout);write(out/'audit.json',audit)
        command('synthetic_rule_tests',[sys.executable,'-B',str(scripts/'test_campaign_planning.py'),str(out)])
        if {rel:digest(ROOT/rel) for rel in pins}!=pins:raise ValueError('planning source changed during execution')
        custody();write(out/'preservation.json',{'all_ten_prior_roots_unchanged':True,'retained_planning_failure_root':FAILED_ROOT,'all_previous_native_and_measurement_source_pins_unchanged':True,'source_files':len(pins),'no_native_command_launched':True})
        manifest.update(status='complete',engineering_planning_pass=True,selected_blocks=audit['selected_blocks'],proposed_worlds=audit['proposed_worlds'],prospective_slots=audit['prospective_slots'],uncached_assay_request_upper=audit['uncached_assay_request_upper'],sufficient_conservative_native_reservation=audit['sufficient_conservative_native_reservation'],campaign_budget_approved=False,campaign_preregistration_complete=False,statistical_scenario_cells=audit['statistical_scenario_cells'],python_test_groups=10,source_files=len(pins),elapsed_seconds=time.monotonic()-start);write(out/'manifest.json',manifest)
        size=sum(p.stat().st_size for p in out.rglob('*') if p.is_file())
        if size>limits['output_bytes']:raise ValueError('planning archive cap')
    except Exception as e:
        manifest.update(status='failed',engineering_planning_pass=False,error=str(e),elapsed_seconds=time.monotonic()-start);write(out/'manifest.json',manifest);write(out/'output_sha256.json',{str(p.relative_to(out)):digest(p) for p in sorted(out.rglob('*')) if p.is_file() and p!=out/'output_sha256.json'});raise
    seal={str(p.relative_to(out)):digest(p) for p in sorted(out.rglob('*')) if p.is_file()};write(out/'output_sha256.json',seal)
    print(json.dumps({'output':str(out),'seal_sha256':digest(out/'output_sha256.json'),'sealed_entries':len(seal),'payload_bytes':size,**audit},indent=2))
if __name__=='__main__':
    if len(sys.argv)!=2:raise SystemExit('usage: run_campaign_planning.py NEW_OUTPUT_DIRECTORY')
    run(Path(sys.argv[1]).resolve())
