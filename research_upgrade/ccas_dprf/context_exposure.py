#!/usr/bin/env python3
"""Analyze credential exposure caused by under-scoped derivation approval.

The cryptographic DPRF is abstracted as a context-indexed oracle.  This tool
does not test DPRF security; it checks whether an authorization projection lets
an endpoint obtain outputs for contexts other than the one a user approved.
"""

from __future__ import annotations

import argparse
import collections
import hashlib
import itertools
import json
from pathlib import Path
from typing import Iterable, Mapping, Sequence


Context = dict[str, str]


def expand_context_space(field_values: Mapping[str, Sequence[str]]) -> list[Context]:
    fields = tuple(field_values)
    return [
        dict(zip(fields, values))
        for values in itertools.product(*(field_values[field] for field in fields))
    ]


def canonical_context(context: Mapping[str, str], fields: Sequence[str]) -> str:
    return json.dumps(
        {field: context[field] for field in fields},
        ensure_ascii=False,
        separators=(",", ":"),
        sort_keys=True,
    )


def context_id(context: Mapping[str, str], fields: Sequence[str]) -> str:
    digest = hashlib.sha256(canonical_context(context, fields).encode()).hexdigest()[:12]
    readable = "/".join(context[field] for field in fields)
    return f"{readable}#{digest}"


def project(context: Mapping[str, str], binding_fields: Sequence[str]) -> tuple[str, ...]:
    return tuple(context[field] for field in binding_fields)


def exposed_contexts(
    contexts: Sequence[Context],
    target: Context,
    mode: str,
    binding_fields: Sequence[str],
) -> list[Context]:
    if mode in {"reconstructing", "dprf_unscoped"}:
        return list(contexts)
    if mode != "dprf_scoped":
        raise ValueError(f"unknown mode: {mode}")
    target_projection = project(target, binding_fields)
    return [
        context
        for context in contexts
        if project(context, binding_fields) == target_projection
    ]


def minimal_injective_bindings(
    contexts: Sequence[Context], fields: Sequence[str]
) -> list[tuple[str, ...]]:
    """Return inclusion-minimal field sets whose projections are collision-free."""
    candidates: list[tuple[str, ...]] = []
    for size in range(len(fields) + 1):
        for subset in itertools.combinations(fields, size):
            if any(set(existing).issubset(subset) for existing in candidates):
                continue
            projections = [project(context, subset) for context in contexts]
            if len(set(projections)) == len(contexts):
                candidates.append(subset)
    return candidates


def projection_class_sizes(
    contexts: Sequence[Context], binding_fields: Sequence[str]
) -> list[int]:
    counts = collections.Counter(project(context, binding_fields) for context in contexts)
    return sorted(counts.values(), reverse=True)


def maximum_spill_profile(class_sizes: Sequence[int], max_ticket_budget: int) -> list[int]:
    """Exact maximum spill for at most q projection-bound tickets, q=1..limit."""
    gains = [size - 1 for size in class_sizes]
    profile = []
    running = 0
    for budget in range(1, max_ticket_budget + 1):
        if budget <= len(gains):
            running += gains[budget - 1]
        profile.append(running)
    return profile


def analyze(raw: Mapping[str, object]) -> dict[str, object]:
    field_values = raw["context_space"]
    fields = tuple(field_values)
    contexts = expand_context_space(field_values)
    target = {field: str(raw["authorized_context"][field]) for field in fields}
    if target not in contexts:
        raise ValueError("authorized_context is outside context_space")

    target_id = context_id(target, fields)
    max_ticket_budget = int(raw.get("max_ticket_budget", 1))
    if max_ticket_budget < 1:
        raise ValueError("max_ticket_budget must be positive")
    results = []
    for case in raw["cases"]:
        binding_fields = tuple(map(str, case.get("binding_fields", [])))
        unknown = set(binding_fields) - set(fields)
        if unknown:
            raise ValueError(f"unknown binding fields: {sorted(unknown)}")
        exposed = exposed_contexts(contexts, target, str(case["mode"]), binding_fields)
        exposed_ids = [context_id(context, fields) for context in exposed]
        spill = [identifier for identifier in exposed_ids if identifier != target_id]
        if case["mode"] == "reconstructing":
            failure_class = "routine_master_exposure"
        elif spill:
            failure_class = "authorization_amplification"
        else:
            failure_class = "none"
        class_sizes = projection_class_sizes(contexts, binding_fields)
        spill_profile = (
            None
            if case["mode"] == "reconstructing"
            else maximum_spill_profile(class_sizes, max_ticket_budget)
        )
        results.append(
            {
                "case": str(case["name"]),
                "mode": str(case["mode"]),
                "binding_fields": list(binding_fields),
                "master_materialized_at_endpoint": case["mode"] == "reconstructing",
                "authorized_context_count": 1,
                "exposed_context_count": len(exposed_ids),
                "unauthorized_spill_count": len(spill),
                "exposure_fraction": len(exposed_ids) / len(contexts),
                "authorization_non_amplifying": not spill,
                "failure_class": failure_class,
                "projection_class_sizes": (
                    None if case["mode"] == "reconstructing" else class_sizes
                ),
                "maximum_unauthorized_spill_by_ticket_budget": spill_profile,
                "exposed_contexts": exposed_ids,
            }
        )

    return {
        "context_fields": list(fields),
        "context_count": len(contexts),
        "authorized_context": target_id,
        "ticket_budgets": list(range(1, max_ticket_budget + 1)),
        "empirical_minimal_injective_bindings": [
            list(binding) for binding in minimal_injective_bindings(contexts, fields)
        ],
        "results": results,
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("model", type=Path)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = analyze(json.loads(args.model.read_text(encoding="utf-8")))
    rendered = json.dumps(result, indent=2, ensure_ascii=False)
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(rendered + "\n", encoding="utf-8")
    print(rendered)


if __name__ == "__main__":
    main()
