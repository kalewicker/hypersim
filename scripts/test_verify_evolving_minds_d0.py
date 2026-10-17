#!/usr/bin/env python3
"""Adversarial read-only checks against the bounded D0 engineering receipt."""
import copy
import hashlib
import json
import os
from pathlib import Path
import shutil
import tempfile
import unittest
import verify_evolving_minds_d0 as audit

class D0CheckerTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.root=Path(os.environ['EVOLVING_MINDS_D0_FIXTURE'])
        cls.c=audit.load(cls.root/'replay.py','d0_test_replay');cls.s=audit.load(cls.root/'schedule_reference.py','d0_test_schedule')
        cls.base=(cls.root/'base_rules.toml').read_bytes();cls.source=cls.root/'informative_delayed_cue'
        cls.frames=[json.loads(s) for s in (cls.source/'trace.jsonl').read_text().splitlines()]
    def tamper(self,edit):
        with tempfile.TemporaryDirectory() as tmp:
            target=Path(tmp)/'run';shutil.copytree(self.source,target);edit(target)
            with self.assertRaises((ValueError,KeyError)):
                audit.verify_cell(target,self.c,self.base,self.s,'delayed_cue','Informative',512.0)
    def trace_edit(self,edit):
        def mutate(target):
            fs=copy.deepcopy(self.frames);edit(fs);(target/'trace.jsonl').write_text(''.join(json.dumps(f)+'\n' for f in fs))
        self.tamper(mutate)
    def test_clean_full_panel_is_read_only(self):
        hashes=lambda:{str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in self.root.rglob('*') if p.is_file()}
        before=hashes();r=audit.verify(self.root);self.assertTrue(r['audit_pass']);self.assertEqual(r['cells_verified'],20);self.assertEqual(before,hashes())
    def test_missing_pass(self):self.trace_edit(lambda fs:fs.pop(4))
    def test_omitted_instruction(self):self.trace_edit(lambda fs:next(f for f in fs if any(e['rule'].startswith('development_create') for e in f['events']))['events'].clear())
    def test_unpaid_development(self):
        def edit(fs):
            f=next(f for f in fs if any(e['rule'].startswith('development_create') for e in f['events']));n=next(n for _,n in f['nodes'] if n['label']==self.c.ROOT);next(v for k,v in n['properties'] if k==self.c.ENERGY)['Float']+=1
        self.trace_edit(edit)
    def test_hidden_initial_subsidy(self):
        def edit(fs):
            n=next(n for _,n in fs[0]['nodes'] if n['label']==self.c.ROOT);next(v for k,v in n['properties'] if k==self.c.ENERGY)['Float']+=1
        self.trace_edit(edit)
    def test_genotype_graph_drift(self):
        def edit(fs):
            n=next(n for _,n in fs[0]['nodes'] if n['label']==self.c.GENE);next(v for k,v in n['properties'] if k==audit.IDENTITY)['Int']+=1
        self.trace_edit(edit)
    def test_foreign_wire_owner(self):
        def edit(fs):
            f=next(f for f in fs if any(e['rule'].startswith('development_wire') for e in f['events']));n=next(n for _,n in f['nodes'] if n['label']==self.c.CONNECTION);next(v for k,v in n['properties'] if k==self.c.OWNER)['Int']=2
        self.trace_edit(edit)
    def test_free_maintenance(self):
        def edit(fs):
            f=next(f for f in fs if any(e['rule'].startswith('development_create') for e in f['events']));n=next(n for _,n in f['nodes'] if n['label']==self.c.ROOT);next(v for k,v in n['properties'] if k==self.c.MAINT)['Float']=0.0
        self.trace_edit(edit)
    def test_wrong_creation_clock(self):
        def edit(fs):
            f=next(f for f in fs if any(e['rule'].startswith('development_create') for e in f['events']));next(n for _,n in f['nodes'] if n['label']==audit.STATE)['generation']+=1
        self.trace_edit(edit)
    def test_false_program_halt(self):
        def edit(fs):
            n=next(n for _,n in fs[0]['nodes'] if n['label']==self.c.ROOT);next(v for k,v in n['properties'] if k==audit.STATUS)['Int']=2
        self.trace_edit(edit)
    def test_changed_program_projection(self):self.tamper(lambda d:(d/'genotype.bin').write_bytes(b'changed'))
    def test_changed_world_stream(self):
        def edit(d):
            p=d/'schedule.json';s=json.loads(p.read_text());s['cues'][0]=not s['cues'][0];p.write_text(json.dumps(s))
        self.tamper(edit)
    def test_changed_physical_price(self):self.tamper(lambda d:(d/'rules.toml').write_text((d/'rules.toml').read_text().replace('"value" = 1.0','"value" = 0.0',1)))
    def test_false_training_admission(self):
        def edit(d):
            p=d/'run_status.json';s=json.loads(p.read_text());s['training_admitted']=True;p.write_text(json.dumps(s))
        self.tamper(edit)
    def test_checkpoint_corruption(self):
        def edit(d):
            p=d/'partial.checkpoint';b=bytearray(p.read_bytes());b[-1]^=1;p.write_bytes(b)
        self.tamper(edit)

if __name__=='__main__':unittest.main()
