import unittest
from pathlib import Path

from ccas_analyzer import Domain, Model, Rule
from dual_collapse_analyzer import analyze, analyze_case


class DualCollapseAnalyzerTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.report = analyze(Path(__file__).with_name("dual_cases.json"))
        cls.results = {item["case"]: item for item in cls.report["results"]}

    def test_only_three_states_are_reachable_in_root_derived_profile(self):
        signatures = {
            tuple(item["dual_signature"]) for item in self.report["results"]
        }
        self.assertEqual(signatures, {(False, False), (False, True), (True, True)})

    def test_safe_root_threshold_does_not_imply_context_confinement(self):
        result = self.results["scope_amplification_without_root"]
        self.assertFalse(result["factor_collapse"])
        self.assertTrue(result["authorization_amplification"])
        self.assertEqual(result["maximum_unauthorized_spill"], 31)
        self.assertEqual(
            result["scope_witnesses"][0]["cause"],
            "callable_derivation_interface",
        )

    def test_root_reachability_dominates_exact_ticket_binding(self):
        result = self.results["root_collapse_dominates_exact_scope"]
        self.assertTrue(result["factor_collapse"])
        self.assertTrue(result["authorization_amplification"])
        self.assertEqual(result["maximum_unauthorized_spill"], 32)
        self.assertIn(["D"], result["minimal_factor_witnesses"])
        self.assertTrue(result["scope_witnesses"][0]["root_reachable"])

    def test_exact_scope_is_non_amplifying_below_root_threshold(self):
        result = self.results["safe_threshold_and_exact_scope"]
        self.assertFalse(result["factor_collapse"])
        self.assertFalse(result["authorization_amplification"])
        self.assertEqual(result["maximum_unauthorized_spill"], 0)

    def test_exposure_threshold_spectrum_distinguishes_scope_from_root(self):
        safe = self.results["safe_threshold_and_exact_scope"]
        broad = self.results["scope_amplification_without_root"]
        collapsed = self.results["root_collapse_dominates_exact_scope"]

        safe_thresholds = [
            item["minimum_compromised_domains"]
            for item in safe["credential_exposure_threshold_spectrum"]
        ]
        broad_thresholds = [
            item["minimum_compromised_domains"]
            for item in broad["credential_exposure_threshold_spectrum"]
        ]
        collapsed_thresholds = [
            item["minimum_compromised_domains"]
            for item in collapsed["credential_exposure_threshold_spectrum"]
        ]

        self.assertEqual(safe_thresholds, [2] * 32)
        self.assertEqual(broad_thresholds, [1] * 31 + [2])
        self.assertEqual(collapsed_thresholds, [1] * 32)

    def test_weighted_spectrum_is_optimized_independently(self):
        model = Model(
            name="weighted",
            domains=(
                Domain("expensive", frozenset({"ClientToken"}), 100),
                Domain("cheap-a", frozenset({"A"}), 1),
                Domain("cheap-b", frozenset({"B"}), 1),
            ),
            public_capabilities=frozenset(),
            shares=frozenset(),
            nominal_qualified_sets=(),
            configured_domain_threshold=3,
            rules=(Rule("cheap pair", frozenset({"A", "B"}), frozenset({"ClientToken"})),),
        )
        result = analyze_case(
            model,
            [{"credential": "a"}, {"credential": "b"}],
            {"name": "broad", "mode": "dprf_unscoped"},
            "ClientToken",
            1,
        )
        first = result["credential_exposure_threshold_spectrum"][0]
        self.assertEqual(first["minimum_compromised_domains"], 1)
        self.assertEqual(first["minimum_cost"], 2)
        self.assertEqual(
            first["minimum_domain_witness"]["compromised_domains"],
            ["expensive"],
        )
        self.assertEqual(
            first["minimum_cost_witness"]["compromised_domains"],
            ["cheap-a", "cheap-b"],
        )

    def test_general_monotone_set_cost_supports_nonadditive_coalitions(self):
        model = Model(
            name="nonadditive",
            domains=(
                Domain("direct", frozenset({"ClientToken"}), 99),
                Domain("a", frozenset({"A"}), 99),
                Domain("b", frozenset({"B"}), 99),
            ),
            public_capabilities=frozenset(),
            shares=frozenset(),
            nominal_qualified_sets=(),
            configured_domain_threshold=3,
            rules=(Rule("pair", frozenset({"A", "B"}), frozenset({"ClientToken"})),),
        )

        table = {
            frozenset(): 0.0,
            frozenset({"direct"}): 10.0,
            frozenset({"a"}): 1.0,
            frozenset({"b"}): 1.0,
            frozenset({"a", "b"}): 1.5,
            frozenset({"direct", "a"}): 11.0,
            frozenset({"direct", "b"}): 11.0,
            frozenset({"direct", "a", "b"}): 12.0,
        }
        result = analyze_case(
            model,
            [{"credential": "a"}, {"credential": "b"}],
            {"name": "broad", "mode": "dprf_unscoped"},
            "ClientToken",
            1,
            set_cost=table.__getitem__,
        )
        first = result["credential_exposure_threshold_spectrum"][0]
        self.assertEqual(first["minimum_compromised_domains"], 1)
        self.assertEqual(first["minimum_cost"], 1.5)
        self.assertEqual(
            first["minimum_cost_witness"]["compromised_domains"],
            ["a", "b"],
        )

    def test_nonmonotone_set_cost_is_rejected(self):
        model = Model(
            name="bad-cost",
            domains=(Domain("a", frozenset({"ClientToken"}), 1),),
            public_capabilities=frozenset(),
            shares=frozenset(),
            nominal_qualified_sets=(),
            configured_domain_threshold=1,
            rules=(),
        )
        with self.assertRaisesRegex(ValueError, "monotone"):
            analyze_case(
                model,
                [{"credential": "a"}],
                {"name": "broad", "mode": "dprf_unscoped"},
                "ClientToken",
                0,
                set_cost=lambda subset: 1.0 if not subset else 0.0,
            )


if __name__ == "__main__":
    unittest.main()
