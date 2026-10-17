#!/usr/bin/env python3
import copy
import hashlib
import json
import os
from pathlib import Path
import shutil
import tempfile
import unittest
import verify_evolving_minds_family as audit

class FamilyCheckerTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.root=Path(os.environ['EVOLVING_MINDS_FAMILY_FIXTURE']);cls.source=cls.root/'informative_cue';cls.base=(cls.root/'base_rules.toml').read_bytes();cls.c=audit.load(cls.root/'replay.py','family_test_replay');cls.d=audit.load(cls.root/'d0_reference.py','family_test_d0');cls.s=audit.load(cls.root/'schedule_reference.py','family_test_schedule');cls.entries=[json.loads(s) for s in (cls.source/'journal.jsonl').read_text().splitlines()]
    def tamper(self,edit):
        with tempfile.TemporaryDirectory() as tmp:
            target=Path(tmp)/'run';shutil.copytree(self.source,target);edit(target)
            with self.assertRaises((ValueError,KeyError)):
                audit.verify_cell(target,self.c,self.d,self.s,self.base,'cue','Informative')
    def journal_edit(self,edit):
        def mutate(d):
            entries=copy.deepcopy(self.entries);edit(entries);(d/'journal.jsonl').write_text(''.join(json.dumps(e)+'\n' for e in entries))
        self.tamper(mutate)
    def test_clean_panel_is_read_only(self):
        hashes=lambda:{str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in self.root.rglob('*') if p.is_file()};before=hashes();r=audit.verify(self.root);self.assertTrue(r['audit_pass']);self.assertEqual(r['cells_verified'],4);self.assertEqual(before,hashes())
    def test_missing_pass(self):self.journal_edit(lambda es:es.pop(3))
    def test_missing_birth(self):self.journal_edit(lambda es:next(e for e in es if e['pass']==15 and e['events'])['events'].clear())
    def test_duplicate_birth(self):
        def edit(es):
            e=next(e for e in es if e['pass']==15 and e['events']);e['events']*=2
        self.journal_edit(edit)
    def test_missing_cleanup(self):self.journal_edit(lambda es:next(e for e in es if e['pass']>=17 and e['events'])['events'].clear())
    def test_changed_counter(self):
        def edit(d):
            p=d/'initial.json';f=json.loads(p.read_text());n=next(n for _,n in f['nodes'] if n['label']==self.c.COUNTER);n['properties'][0][1]['Int']=3;p.write_text(json.dumps(f))
        self.tamper(edit)
    def test_free_copy_law(self):self.tamper(lambda d:(d/'rules.toml').write_text((d/'rules.toml').read_text().replace('"value" = 16.5','"value" = 12.0',1)))
    def test_inherited_age(self):
        def edit(d):
            p=d/'rules.toml';raw=p.read_text();raw=raw.replace('"var" = "child"\n"key" = 240004\n"op" = "set"\n"value" = 0','"var" = "child"\n"key" = 240004\n"op" = "set"\n"value" = 9',1);p.write_text(raw)
        self.tamper(edit)
    def test_wrong_ancestry(self):
        def edit(d):
            p=d/'lineage.json';x=json.loads(p.read_text());x['2']['parent']=2;p.write_text(json.dumps(x))
        self.tamper(edit)
    def test_altered_child_genotype(self):
        def edit(d):
            p=d/'lineage.json';x=json.loads(p.read_text());x['2']['program']['instructions'][1]['source']=1;p.write_text(json.dumps(x))
        self.tamper(edit)
    def test_wrong_creation_handle(self):
        def edit(es):
            e=next(e for e in es if e['pass']==15 and e['events'])['events'][0];e['created'][0]['idx']+=100000
        self.journal_edit(edit)
    def test_changed_final_reserve(self):
        def edit(d):
            p=d/'final.json';x=json.loads(p.read_text());n=next(n for _,n in x['nodes'] if n['label']==self.c.ROOT);next(v for k,v in n['properties'] if k==self.c.ENERGY)['Float']+=1;p.write_text(json.dumps(x))
        self.tamper(edit)
    def test_bad_checkpoint(self):
        def edit(d):
            p=d/'midpoint.checkpoint';b=bytearray(p.read_bytes());b[-1]^=1;p.write_bytes(b)
        self.tamper(edit)
    def test_false_training_admission(self):
        def edit(d):
            p=d/'run_status.json';x=json.loads(p.read_text());x['training_admitted']=True;p.write_text(json.dumps(x))
        self.tamper(edit)

if __name__=='__main__':unittest.main()
