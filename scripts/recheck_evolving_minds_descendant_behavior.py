#!/usr/bin/env python3
"""Recheck an EM-16 archive using its frozen code and pinned original inputs."""
import argparse
import json
from pathlib import Path
import sys
sys.dont_write_bytecode=True


def recheck(archive, tables_only=False):
    archive=Path(archive).resolve()
    sys.path.insert(0,str(archive))
    try:
        import evolving_minds_descendant_behavior as b
        b.common.sealed_input(archive, b.common.digest((archive/'output_sha256.json').read_bytes()))
        binding=b.common.data(archive/'input_binding.json')
        native=Path(binding['native_directory']);audited=Path(binding['audited_directory'])
        b.common.sealed_input(native,b.NATIVE_SEAL);b.common.sealed_input(audited,b.AUDIT_SEAL)
        sources=b.common.data(archive/'analysis_sources.json')
        for module,key in [(b,b.SOURCE_PATHS[0]),(b.routes,b.SOURCE_PATHS[3]),(b.routes.base,b.SOURCE_PATHS[4]),(b.common,b.SOURCE_PATHS[5]),(b.routes.shared,b.SOURCE_PATHS[6])]:
            b.require(Path(module.__file__).read_text()==sources[key],'frozen recheck dependency')
        checker=b.common.load(native/'checker.py','recheck_frozen_checker')
        stored=b.common.data(archive/'independent_replay.json')
        if tables_only:
            fresh=stored;traces=b.common.data(archive/'behavior_traces.json')
        else:
            original=checker.load;collected=[]
            def instrument(path,name):
                module=original(path,name)
                if Path(path).name=='opportunity_reference.py':
                    module.LifetimeObserver=b.observer_type(module.LifetimeObserver,collected)
                return module
            checker.load=instrument
            fresh=checker.verify(native)
            for key in set(fresh)|set(stored):
                if key in ('audit_seconds','results'):continue
                b.require(fresh[key]==stored[key],'fresh recheck top-level '+key)
            b.require(set(fresh['results'])==set(stored['results']),'fresh recheck cells')
            for name,result in fresh['results'].items():
                b.require({k:v for k,v in result.items() if k!='audit_seconds'}==
                          {k:v for k,v in stored['results'][name].items() if k!='audit_seconds'},'fresh recheck full result')
            b.require(len(collected)==73,'fresh recheck observer denominator')
            traces={case['name']:obs.behavior for case,obs in zip(checker.cases(),collected)}
            b.require(traces==b.common.data(archive/'behavior_traces.json'),'fresh recheck full traces')
        a=b.common.load(native/'admission_reference.py','recheck_admission')
        d=b.common.load(native/'d0_reference.py','recheck_d0')
        cells,targets,summary=b.analyze_results(fresh,traces,checker,a,d)
        b.require(cells==b.common.data(archive/'classes_and_routes.json') and targets==b.common.data(archive/'target_comparisons.json') and summary==b.common.data(archive/'summary.json'),'exact full recheck tables')
        for directory,receipt in b.common.data(archive/'prior_archives.json').items():
            files,_=b.common.inventory(Path(directory))
            b.require(files.pop('output_sha256.json')==receipt['seal_sha256'] and files==receipt['files_sha256'],'historical preservation recheck')
        return dict(dict(profile=b.PROFILE,**b.FLAGS,recheck_pass=True,native_advances=0,mode='tables-only' if tables_only else 'fresh full physical replay',targets=144,substitutions=140,universal_equivalence=False), scientific_acceptance=False)
    finally:
        sys.path.pop(0)


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('archive');parser.add_argument('--tables-only',action='store_true',help='Validate sealed replay-derived tables without repeating the physical replay')
    args=parser.parse_args();print(json.dumps(recheck(args.archive,args.tables_only),indent=2))
