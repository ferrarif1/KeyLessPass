#!/usr/bin/env python3

import json
import unittest
from pathlib import Path

from context_exposure import analyze


HERE = Path(__file__).resolve().parent


class ContextExposureTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        raw = json.loads((HERE / "context_cases.json").read_text(encoding="utf-8"))
        cls.result = analyze(raw)
        cls.cases = {case["case"]: case for case in cls.result["results"]}

    def test_expected_exposure_curve(self) -> None:
        self.assertEqual(
            [32, 32, 16, 8, 4, 2, 1],
            [case["exposed_context_count"] for case in self.result["results"]],
        )

    def test_hiding_master_without_scope_does_not_reduce_worst_case_exposure(self) -> None:
        old = self.cases["old_root_reconstruction"]
        unscoped = self.cases["dprf_token_accepts_any_context"]
        self.assertTrue(old["master_materialized_at_endpoint"])
        self.assertFalse(unscoped["master_materialized_at_endpoint"])
        self.assertEqual(old["exposed_context_count"], unscoped["exposed_context_count"])

    def test_full_context_binding_is_non_amplifying(self) -> None:
        exact = self.cases["ticket_binds_full_canonical_context"]
        self.assertTrue(exact["authorization_non_amplifying"])
        self.assertEqual(0, exact["unauthorized_spill_count"])
        self.assertEqual("none", exact["failure_class"])

    def test_unscoped_dprf_is_classified_as_authorization_amplification(self) -> None:
        unscoped = self.cases["dprf_token_accepts_any_context"]
        self.assertEqual("authorization_amplification", unscoped["failure_class"])

    def test_full_schema_is_the_only_key_for_cartesian_corpus(self) -> None:
        self.assertEqual(
            [self.result["context_fields"]],
            self.result["empirical_minimal_injective_bindings"],
        )

    def test_projection_partition_gives_exact_ticket_budget_profile(self) -> None:
        service = self.cases["ticket_binds_service_only"]
        self.assertEqual([16, 16], service["projection_class_sizes"])
        self.assertEqual(
            [15, 30, 30, 30],
            service["maximum_unauthorized_spill_by_ticket_budget"],
        )

    def test_exact_binding_has_zero_spill_for_every_ticket_budget(self) -> None:
        exact = self.cases["ticket_binds_full_canonical_context"]
        self.assertEqual(
            [0, 0, 0, 0],
            exact["maximum_unauthorized_spill_by_ticket_budget"],
        )


if __name__ == "__main__":
    unittest.main()
