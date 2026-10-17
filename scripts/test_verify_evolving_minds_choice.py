#!/usr/bin/env python3
"""Negative controls against a native choice receipt, without engine execution."""
import copy
import hashlib
import json
import os
from pathlib import Path
import shutil
import tempfile
import unittest
import verify_evolving_minds_choice as audit


class ChoiceCheckerTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.root=Path(os.environ['EVOLVING_MINDS_CHOICE_FIXTURE'])
        cls.c=audit.load_replay(cls.root)
        cls.base=(cls.root/'base_rules.toml').read_bytes()
        cls.source=cls.root/'s17_informative_reactive'
        cls.frames=[json.loads(s) for s in (cls.source/'trace.jsonl').read_text().splitlines()]

    def tamper(self,edit):
        with tempfile.TemporaryDirectory() as tmp:
            target=Path(tmp)/'run';shutil.copytree(self.source,target);edit(target)
            with self.assertRaises((ValueError,KeyError)):
                audit.verify_arm(target,self.c,self.base,'Reactive',17,'Informative')

    def trace_edit(self,edit):
        def mutate(target):
            frames=copy.deepcopy(self.frames);edit(frames)
            (target/'trace.jsonl').write_text(''.join(json.dumps(f)+'\n' for f in frames))
        self.tamper(mutate)

    def test_clean_full_panel_and_read_only(self):
        def hashes():return {str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in self.root.rglob('*') if p.is_file()}
        before=hashes();r=audit.verify(self.root);self.assertTrue(r['audit_pass']);self.assertEqual(r['cells_verified'],30);self.assertEqual(before,hashes())

    def test_missing_phase(self):self.trace_edit(lambda fs:fs.pop(2))
    def test_omitted_paid_attempt(self):
        self.trace_edit(lambda fs:next(f for f in fs if any(e['rule']=='attempt_commit' for e in f['events']))['events'].clear())

    def test_unpaid_action(self):
        def edit(fs):
            f=next(f for f in fs if any(e['rule']=='attempt_commit' for e in f['events']))
            n=next(n for _,n in f['nodes'] if n['label']==self.c.ROOT)
            next(v for k,v in n['properties'] if k==self.c.ENERGY)['Float']+=1
        self.trace_edit(edit)

    def test_wrong_creation_clock(self):
        def edit(fs):
            f=next(f for f in fs if f['phase']=='Development')
            next(n for _,n in f['nodes'] if n['label']==self.c.CONNECTION)['generation']+=1
        self.trace_edit(edit)

    def test_hidden_initial_subsidy(self):
        def edit(fs):
            n=next(n for _,n in fs[0]['nodes'] if n['label']==self.c.ROOT)
            next(v for k,v in n['properties'] if k==self.c.ENERGY)['Float']+=1
        self.trace_edit(edit)

    def test_unfaithful_child(self):
        def edit(fs):
            f=next(f for f in fs if any(e['rule']=='birth_faithful' for e in f['events']))
            e=next(e for e in f['events'] if e['rule']=='birth_faithful')
            n=next(n for h,n in f['nodes'] if h in e['created'] and n['label']==self.c.GENE)
            next(v for k,v in n['properties'] if k==self.c.OP)['Int']=1
        self.trace_edit(edit)

    def test_wrong_controller(self):
        def edit(fs):
            n=next(n for _,n in fs[0]['nodes'] if n['label']==self.c.ROOT)
            next(v for k,v in n['properties'] if k==audit.POLICY)['Int']=1
        self.trace_edit(edit)

    def test_changed_world_stream(self):
        def edit(d):
            p=d/'schedule.json';s=json.loads(p.read_text());s['coins'][0][0]=not s['coins'][0][0];p.write_text(json.dumps(s))
        self.tamper(edit)

    def test_changed_physical_price(self):
        self.tamper(lambda d:(d/'rules.toml').write_text((d/'rules.toml').read_text().replace('"value" = 18.0','"value" = 0.0')))

    def test_false_training_admission(self):
        def edit(d):
            p=d/'run_status.json';s=json.loads(p.read_text());s['training_admitted']=True;p.write_text(json.dumps(s))
        self.tamper(edit)

    def test_checkpoint_corruption(self):
        def edit(d):
            p=d/'final.checkpoint';b=bytearray(p.read_bytes());b[-1]^=1;p.write_bytes(b)
        self.tamper(edit)


if __name__=='__main__':unittest.main()
