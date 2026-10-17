"""Known-root frozen sizing/audit/synthetic-rule recheck; never executes a native binary."""
import hashlib,json,subprocess,sys
from pathlib import Path
FLAGS=['readiness_gate_complete','scientific_acceptance','evolution_demonstrated','campaign_launch_authorized']
def recheck(package,root):
    sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
    def custody():
        if sha(package/'output_sha256.json')!=root:raise ValueError('known campaign-planning seal')
        seal=json.loads((package/'output_sha256.json').read_text());actual={str(p.relative_to(package)) for p in package.rglob('*') if p.is_file() and p!=package/'output_sha256.json'}
        if actual!=set(seal):raise ValueError('exact planning archive file set')
        for rel,h in seal.items():
            p=Path(rel)
            if p.is_absolute() or '..' in p.parts or (package/p).is_symlink() or sha(package/p)!=h:raise ValueError('planning custody '+rel)
        for rel,h in json.loads((package/'source_sha256.json').read_text()).items():
            if sha(package/'source_snapshots'/rel)!=h:raise ValueError('frozen planning source '+rel)
        m=json.loads((package/'manifest.json').read_text())
        if m['status']!='complete' or not m['engineering_planning_pass'] or m['native_rewrites']!=0 or m['whole_campaign_feasibility_pass'] is not False:raise ValueError('planning status')
        if any(m[f] is not False for f in FLAGS):raise ValueError('planning admission')
        return seal
    seal=custody();scripts=package/'source_snapshots/experiments/cumulative/scripts'
    for script in ['audit_campaign.py','test_campaign_planning.py']:
        r=subprocess.run([sys.executable,'-B',str(scripts/script),str(package)],capture_output=True,text=True,timeout=60)
        if r.returncode:raise ValueError(r.stderr)
        if script=='audit_campaign.py' and json.loads(r.stdout)!=json.loads((package/'audit.json').read_text()):raise ValueError('frozen planning audit differs')
    custody();return {'verified_entries':len(seal),'frozen_audit_and_tests_pass':True,'native_rewrites':0,'seal_sha256':root,'whole_campaign_feasibility_pass':False,'proposed_panel_stays_unadmitted':True,**{f:False for f in FLAGS}}
if __name__=='__main__':
    if len(sys.argv)!=3:raise SystemExit('usage: recheck_campaign.py ARCHIVE EXPECTED_ROOT')
    print(json.dumps(recheck(Path(sys.argv[1]).resolve(),sys.argv[2]),indent=2))
