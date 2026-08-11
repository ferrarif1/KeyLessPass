#!/usr/bin/env python3

import unittest
from pathlib import Path

from ccas_analyzer import analyze, load_models


HERE = Path(__file__).resolve().parent


class CcasAnalyzerTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.results = {
            result["case"]: result
            for result in (analyze(model) for model in load_models(HERE / "cases.json"))
        }

    def test_expected_effective_thresholds(self) -> None:
        expected = {
            "case_1_naive_automatically_callable_network": 1,
            "case_2_independent_approval": 2,
            "case_3_approval_key_leaked_into_endpoint": 1,
            "case_4_removable_medium_holds_release_credential": 1,
            "case_5_three_of_five_nodes_callable_by_one_endpoint": 1,
            "case_6_shared_administrative_domain": 1,
        }
        self.assertEqual(
            expected,
            {name: result["effective_domain_threshold"] for name, result in self.results.items()},
        )

    def test_only_independent_approval_case_preserves_threshold(self) -> None:
        preserved = [
            name for name, result in self.results.items() if not result["threshold_collapse"]
        ]
        self.assertEqual(["case_2_independent_approval"], preserved)

    def test_threshold_comparison_uses_deployment_policy(self) -> None:
        for result in self.results.values():
            self.assertEqual(2, result["configured_domain_threshold"])

    def test_collapse_witnesses_identify_single_domain(self) -> None:
        expected = {
            "case_1_naive_automatically_callable_network": "D",
            "case_3_approval_key_leaked_into_endpoint": "D",
            "case_4_removable_medium_holds_release_credential": "U",
            "case_5_three_of_five_nodes_callable_by_one_endpoint": "D",
            "case_6_shared_administrative_domain": "Admin",
        }
        for case, domain in expected.items():
            minimal = self.results[case]["minimal_compromising_domain_sets"]
            self.assertIn([domain], minimal, case)

    def test_nonautomatic_rules_do_not_enter_closure(self) -> None:
        model = load_models(HERE / "cases.json")[0]
        result = analyze(model)
        d_witness = next(
            item for item in result["witnesses"] if item["compromised_domains"] == ["D"]
        )
        self.assertIn("S_N", d_witness["closure"])
        self.assertTrue(any("honest network automatically releases" in step for step in d_witness["trace"]))


if __name__ == "__main__":
    unittest.main()
