#!/usr/bin/env python3
import unittest
import evolving_minds_descendant_behavior as b

class AdmissionMetadataTests(unittest.TestCase):
    def test_scientific_and_admission_boundaries_are_explicit(self):
        for key in ['scientific_acceptance','scientific_run_launched','sustained_claim','training_admitted','evolution_enabled','reproductive_fairness_accepted','power_adequacy_established','ready_for_scientific_launch','gate_pass']:
            self.assertIn(key,b.FLAGS)
            self.assertIs(b.FLAGS[key],False)
if __name__=='__main__':unittest.main()
