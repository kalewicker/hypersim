"""Known-root durable prefix audit and adversarial tests; executes zero native work."""
import hashlib,json,subprocess,sys
from pathlib import Path
FLAGS=['readiness_gate_complete','scientific_acceptance','evolution_demonstrated','campaign_launch_authorized']
def recheck(package,root):
    sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
    def custody():
        if sha(package/'output_sha256.json')!=root:raise ValueError('known-root durable seal')
        seal=json.loads((package/'output_sha256.json').read_text());actual={str(p.relative_to(package)) for p in package.rglob('*') if p.is_file() and p!=package/'output_sha256.json'}
        if actual!=set(seal):raise ValueError('durable archive exact file set')
        for rel,h in seal.items():
            p=Path(rel)
            if p.is_absolute() or '..' in p.parts or (package/p).is_symlink() or sha(package/p)!=h:raise ValueError('durable custody '+rel)
        for index,directory in [('source_sha256.json','source_snapshots'),('registry_sha256.json','registry_snapshots')]:
            for rel,h in json.loads((package/index).read_text()).items():
                if sha(package/directory/rel)!=h:raise ValueError('frozen source closure '+rel)
        m=json.loads((package/'manifest.json').read_text());a=json.loads((package/'audit.json').read_text())
        if m['status']!='complete' or not m['engineering_fixture_pass'] or not m['durable_bounded_routes_engineering_complete']:raise ValueError('durability engineering status')
        if any(m[f] is not False or a[f] is not False for f in FLAGS) or m['durable_campaign_routes_complete'] is not False:raise ValueError('durable admission')
        for k in ['native_rewrites_lower_bound','native_rewrites_upper_bound','native_work_unknown']:
            if m[k]!=a[k]:raise ValueError('retained aggregate work interval')
        return seal
    seal=custody();scripts=package/'source_snapshots/experiments/cumulative/scripts'
    for script in ['audit_durability.py','test_durability_audit.py']:
        r=subprocess.run([sys.executable,'-B',str(scripts/script),str(package)],capture_output=True,text=True,timeout=60)
        if r.returncode:raise ValueError(r.stderr)
        if script=='audit_durability.py' and json.loads(r.stdout)!={k:v for k,v in json.loads((package/'audit.json').read_text()).items() if k!='routes'}:raise ValueError('frozen durable audit differs')
    custody();return {'verified_entries':len(seal),'frozen_audit_and_tests_pass':True,'native_rewrites':0,'seal_sha256':root,'partial_routes_and_work_uncertainty_preserved':True,**{f:False for f in FLAGS}}
if __name__=='__main__':
    if len(sys.argv)!=3:raise SystemExit('usage: recheck_durability.py ARCHIVE EXPECTED_SEAL_SHA256')
    print(json.dumps(recheck(Path(sys.argv[1]).resolve(),sys.argv[2]),indent=2))
