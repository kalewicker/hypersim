"""Future launches must snapshot every local auditor dependency."""
import unittest
import run_ere_rule_maintenance as run
class PackagingTests(unittest.TestCase):
    def test_import_closure_and_future_sources(self):
        closure=run.dependency_names()
        self.assertIn('scripts/verify_ere_complete_long_gates.py',closure)
        self.assertIn('scripts/verify_ere_complete_gates.py',closure)
        self.assertIn('scripts/ere_rule_survival_metrics.py',closure)
        self.assertLessEqual(closure,set(run.names()))
        self.assertGreater(len(closure),10)
if __name__=='__main__':unittest.main()
