#!/usr/bin/env python3
"""Tamper controls for persisted native intervention fixtures."""
import json
import os
from pathlib import Path
import shutil
import tempfile
import unittest
from verify_evolving_minds_interventions import verify


class InterventionCheckerTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.source=Path(os.environ['EVOLVING_MINDS_INTERVENTIONS_FIXTURE'])

    def tamper(self,edit):
        with tempfile.TemporaryDirectory() as tmp:
            target=Path(tmp)/'run'
            shutil.copytree(self.source,target)
            path=target/'probes.json'
            probes=json.loads(path.read_text())
            edit(probes)
            path.write_text(json.dumps(probes))
            with self.assertRaises((ValueError,KeyError)):
                verify(target)

    def test_clean_and_read_only(self):
        before={str(p.relative_to(self.source)):p.read_bytes() for p in self.source.rglob('*') if p.is_file()}
        result=verify(self.source)
        self.assertTrue(result['audit_pass'])
        self.assertEqual({k:v['acquired'] for k,v in result['arms'].items()},dict(sham=8,signal_clamp=0,foreign_owner=0,owner_rescue=8))
        after={str(p.relative_to(self.source)):p.read_bytes() for p in self.source.rglob('*') if p.is_file()}
        self.assertEqual(before,after)

    def test_wrong_semantic_target(self):
        self.tamper(lambda p:p[0].update(target_birth_id=999))

    def test_unmatched_energy_preparation(self):
        def edit(p):
            n=next(n for _,n in p[0]['frames'][0]['nodes'] if n['label']==0xE1000000)
            next(v for k,v in n['properties'] if k==1)['Float']+=1
        self.tamper(edit)

    def test_omitted_native_signal(self):
        self.tamper(lambda p:p[0]['frames'][1]['events'].clear())

    def test_false_owner_admission(self):
        self.tamper(lambda p:p[2].update(lifecycle_valid=True))

    def test_wrong_outcome(self):
        self.tamper(lambda p:p[1].update(acquired=8))

    def test_missing_phase(self):
        self.tamper(lambda p:p[0]['frames'].pop())

    def test_missing_rescue_edit(self):
        self.tamper(lambda p:p[3]['edits'].pop())

    def test_wrong_effect_counter(self):
        self.tamper(lambda p:p[0]['frames'][1]['stats'].update(property_effects_applied=0))


if __name__=='__main__':
    unittest.main()
