#!/usr/bin/env python3
"""Jointly compute root reachability and context exposure from X and T."""

from __future__ import annotations

import argparse
import itertools
import json
from pathlib import Path
from typing import Callable, Iterable, Mapping, Sequence

from ccas_analyzer import Model, capability_closure, load_models
from context_exposure import Context, expand_context_space, project


def _load_json(path: Path) -> Mapping[str, object]:
    return json.loads(path.read_text(encoding="utf-8"))


def _subsets(values: Sequence[str], maximum_size: int) -> Iterable[frozenset[str]]:
    for size in range(maximum_size + 1):
        for subset in itertools.combinations(values, size):
            yield frozenset(subset)


def _context_subsets(
    contexts: Sequence[Context], maximum_size: int
) -> Iterable[tuple[Context, ...]]:
    for size in range(maximum_size + 1):
        yield from itertools.combinations(contexts, size)


def _root_reachable(model: Model, closure: frozenset[str]) -> bool:
    available = closure & model.shares
    return any(qualified.issubset(available) for qualified in model.nominal_qualified_sets)


def _ticket_exposure(
    contexts: Sequence[Context],
    authorized: Sequence[Context],
    mode: str,
    binding_fields: Sequence[str],
) -> set[int]:
    if not authorized:
        return set()
    if mode == "dprf_unscoped":
        return set(range(len(contexts)))
    if mode != "dprf_scoped":
        raise ValueError(f"unsupported joint-analysis mode: {mode}")
    projections = {project(context, binding_fields) for context in authorized}
    return {
        index
        for index, context in enumerate(contexts)
        if project(context, binding_fields) in projections
    }


def _minimal_domain_sets(domain_sets: Sequence[frozenset[str]]) -> list[list[str]]:
    ordered = sorted(domain_sets, key=lambda item: (len(item), sorted(item)))
    minimal: list[frozenset[str]] = []
    for candidate in ordered:
        if not any(existing.issubset(candidate) for existing in minimal):
            minimal.append(candidate)
    return [sorted(item) for item in minimal]


def analyze_case(
    model: Model,
    contexts: Sequence[Context],
    context_case: Mapping[str, object],
    request_capability: str,
    ticket_budget: int,
    set_cost: Callable[[frozenset[str]], float] | None = None,
) -> dict[str, object]:
    if ticket_budget < 0:
        raise ValueError("ticket_budget must be non-negative")

    protected_max = max(0, model.configured_domain_threshold - 1)
    domain_names = tuple(domain.name for domain in model.domains)
    mode = str(context_case["mode"])
    binding_fields = tuple(map(str, context_case.get("binding_fields", [])))
    factor_witnesses: list[frozenset[str]] = []
    scope_witnesses: list[dict[str, object]] = []
    maximum_spill = 0
    domain_costs = {domain.name: domain.cost for domain in model.domains}
    if set_cost is None:
        set_cost = lambda subset: sum(domain_costs[name] for name in subset)
    all_domain_sets = tuple(_subsets(domain_names, len(domain_names)))
    coalition_costs = {subset: float(set_cost(subset)) for subset in all_domain_sets}
    for smaller in all_domain_sets:
        if coalition_costs[smaller] < 0:
            raise ValueError("set_cost must be non-negative")
        for domain in domain_names:
            if domain in smaller:
                continue
            larger = smaller | {domain}
            if coalition_costs[smaller] > coalition_costs[larger]:
                raise ValueError("set_cost must be monotone under set inclusion")
    exposure_best_domains: list[dict[str, object] | None] = [None] * (
        len(contexts) + 1
    )
    exposure_best_cost: list[dict[str, object] | None] = [None] * (
        len(contexts) + 1
    )

    for compromised in _subsets(domain_names, len(domain_names)):
        closure, _proof = capability_closure(model, compromised)
        root = _root_reachable(model, closure)
        if root:
            factor_witnesses.append(compromised)

        for authorized in _context_subsets(contexts, ticket_budget):
            authorized_indexes = {
                contexts.index(context) for context in authorized
            }
            if root:
                exposed = set(range(len(contexts)))
                cause = "root_capability"
            elif request_capability in closure:
                exposed = _ticket_exposure(
                    contexts, authorized, mode, binding_fields
                )
                cause = "callable_derivation_interface"
            else:
                exposed = set()
                cause = "none"

            spill = exposed - authorized_indexes
            compromised_cost = coalition_costs[compromised]
            spectrum_witness = {
                "compromised_domains": sorted(compromised),
                "authorized_context_indexes": sorted(authorized_indexes),
                "cause": cause,
                "root_reachable": root,
            }
            for exposed_count in range(1, len(spill) + 1):
                current_domains = exposure_best_domains[exposed_count]
                domain_key = (len(compromised), compromised_cost)
                current_domain_key = (
                    (
                        current_domains["minimum_compromised_domains"],
                        current_domains["witness_cost"],
                    )
                    if current_domains is not None
                    else None
                )
                if current_domain_key is None or domain_key < current_domain_key:
                    exposure_best_domains[exposed_count] = {
                        "minimum_compromised_domains": len(compromised),
                        "witness_cost": compromised_cost,
                        "minimum_domain_witness": spectrum_witness,
                    }

                current_cost = exposure_best_cost[exposed_count]
                cost_key = (compromised_cost, len(compromised))
                current_cost_key = (
                    (
                        current_cost["minimum_cost"],
                        current_cost["witness_domain_count"],
                    )
                    if current_cost is not None
                    else None
                )
                if current_cost_key is None or cost_key < current_cost_key:
                    exposure_best_cost[exposed_count] = {
                        "minimum_cost": compromised_cost,
                        "witness_domain_count": len(compromised),
                        "minimum_cost_witness": spectrum_witness,
                    }

            if len(compromised) > protected_max:
                continue
            if len(spill) > maximum_spill:
                maximum_spill = len(spill)
                scope_witnesses = [
                    {
                        "compromised_domains": sorted(compromised),
                        "authorized_context_indexes": sorted(authorized_indexes),
                        "exposed_context_indexes": sorted(exposed),
                        "unauthorized_spill_count": len(spill),
                        "cause": cause,
                        "root_reachable": root,
                    }
                ]
            elif spill and len(spill) == maximum_spill:
                scope_witnesses.append(
                    {
                        "compromised_domains": sorted(compromised),
                        "authorized_context_indexes": sorted(authorized_indexes),
                        "exposed_context_indexes": sorted(exposed),
                        "unauthorized_spill_count": len(spill),
                        "cause": cause,
                        "root_reachable": root,
                    }
                )

    minimal_factor_sets = _minimal_domain_sets(factor_witnesses)
    master_threshold = min((len(item) for item in factor_witnesses), default=None)
    master_cost = min(
        (coalition_costs[item] for item in factor_witnesses),
        default=None,
    )
    factor_collapse = (
        master_threshold is not None
        and master_threshold < model.configured_domain_threshold
    )
    authorization_amplification = maximum_spill > 0
    return {
        "access_case": model.name,
        "context_case": str(context_case["name"]),
        "configured_domain_threshold": model.configured_domain_threshold,
        "protected_max_compromised_domains": protected_max,
        "ticket_budget": ticket_budget,
        "master_domain_threshold": master_threshold,
        "master_weighted_cost": master_cost,
        "factor_collapse": factor_collapse,
        "authorization_amplification": authorization_amplification,
        "dual_signature": [factor_collapse, authorization_amplification],
        "maximum_unauthorized_spill": maximum_spill,
        "minimal_factor_witnesses": minimal_factor_sets,
        "scope_witnesses": scope_witnesses[:4],
        "credential_exposure_threshold_spectrum": [
            {
                "unauthorized_contexts": count,
                "minimum_compromised_domains": (
                    exposure_best_domains[count]["minimum_compromised_domains"]
                    if exposure_best_domains[count] is not None
                    else None
                ),
                "minimum_domain_witness": (
                    exposure_best_domains[count]["minimum_domain_witness"]
                    if exposure_best_domains[count] is not None
                    else None
                ),
                "minimum_cost": (
                    exposure_best_cost[count]["minimum_cost"]
                    if exposure_best_cost[count] is not None
                    else None
                ),
                "minimum_cost_witness": (
                    exposure_best_cost[count]["minimum_cost_witness"]
                    if exposure_best_cost[count] is not None
                    else None
                ),
            }
            for count in range(1, len(contexts) + 1)
        ],
        "binding_fields": list(binding_fields),
    }


def analyze(spec_path: Path) -> dict[str, object]:
    spec = _load_json(spec_path)
    base = spec_path.parent
    access_models = {
        model.name: model
        for model in load_models(base / str(spec["access_models"]))
    }
    context_raw = _load_json(base / str(spec["context_models"]))
    contexts = expand_context_space(context_raw["context_space"])
    context_cases = {
        str(case["name"]): case for case in context_raw["cases"]
    }

    results = []
    for case in spec["cases"]:
        result = analyze_case(
            access_models[str(case["access_case"])],
            contexts,
            context_cases[str(case["context_case"])],
            str(case["request_capability"]),
            int(case.get("ticket_budget", 1)),
        )
        result["case"] = str(case["name"])
        results.append(result)

    return {
        "method": "dual-collapse-credential-exposure-analysis",
        "context_count": len(contexts),
        "results": results,
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("spec", type=Path)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = analyze(args.spec)
    rendered = json.dumps(result, indent=2, ensure_ascii=False)
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(rendered + "\n", encoding="utf-8")
    print(rendered)


if __name__ == "__main__":
    main()
