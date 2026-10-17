#!/usr/bin/env python3
import ast
import unittest
from pathlib import Path
import evolving_minds_handoff_audit_continuation as repair
import verify_evolving_minds_handoff as checker
def frozen_source():
    return Path(checker.__file__).read_text().replace("dict(observed_total_bytes=actual,**resource)","dict(total_bytes=actual,**resource)")

class PackagingRepairTests(unittest.TestCase):
    def test_ast_restores_all_original_statements(self):
        _,r=repair.repaired_function(checker,frozen_source());self.assertEqual(r['original_ast_sha256'],r['restored_ast_sha256']);self.assertEqual(r['physical_predicates_changed'],0);self.assertEqual(r['metadata_keyword_renames'],1)
    def test_collision_reproduced_and_both_measurements_retained(self):
        source=frozen_source();tree=ast.parse(source);node=next(n for n in ast.walk(tree) if isinstance(n,ast.Assign) and len(n.targets)==1 and ast.unparse(n.targets[0])=='resources[directory.name]');namespace=dict(actual=100,resource=dict(total_bytes=90,advance_seconds=1))
        with self.assertRaises(TypeError):eval(compile(ast.Expression(node.value),'original-metadata','eval'),namespace)
        node.value.keywords[0].arg='observed_total_bytes';result=eval(compile(ast.Expression(node.value),'repaired-metadata','eval'),namespace);self.assertEqual(result,dict(total_bytes=90,observed_total_bytes=100,advance_seconds=1))
    def test_changed_or_duplicated_anchor_refused(self):
        source=frozen_source()
        with self.assertRaisesRegex(ValueError,'unique frozen'):repair.repaired_function(checker,source.replace('dict(total_bytes=actual,**resource)','dict(measured=actual,**resource)'))
        with self.assertRaisesRegex(ValueError,'unique frozen'):repair.repaired_function(checker,source.replace('resources[directory.name]=dict(total_bytes=actual,**resource)','resources[directory.name]=dict(total_bytes=actual,**resource)\n            resources[directory.name]=dict(total_bytes=actual,**resource)'))
if __name__=='__main__':unittest.main()
