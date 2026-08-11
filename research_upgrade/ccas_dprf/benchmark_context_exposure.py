#!/usr/bin/env python3
"""Measure finite context-exposure analysis without external dependencies."""

from __future__ import annotations

import argparse
import json
import statistics
import time
from pathlib import Path

from context_exposure import analyze


SIZES = {
    32: [2, 2, 2, 2, 2],
    1024: [4, 4, 4, 4, 4],
    10000: [10, 10, 10, 10, 1],
    100000: [10, 10, 10, 10, 10],
}
FIELDS = ["serviceID", "accountID", "credentialLineage", "rootGeneration", "policyID"]


def model_for(cardinalities: list[int]) -> dict[str, object]:
    space = {
        field: [f"{field}-{index}" for index in range(cardinality)]
        for field, cardinality in zip(FIELDS, cardinalities)
    }
    target = {field: values[0] for field, values in space.items()}
    return {
        "context_space": space,
        "authorized_context": target,
        "cases": [
            {"name": "unscoped", "mode": "dprf_unscoped"},
            {
                "name": "full-binding",
                "mode": "dprf_scoped",
                "binding_fields": FIELDS,
            },
        ],
    }


def percentile_nearest_rank(values: list[float], percentile: float) -> float:
    ordered = sorted(values)
    index = max(0, min(len(ordered) - 1, int(percentile * len(ordered) + 0.999999) - 1))
    return ordered[index]


def run(repetitions: int) -> dict[str, object]:
    results = []
    for count, cardinalities in SIZES.items():
        model = model_for(cardinalities)
        durations = []
        for _ in range(repetitions):
            started = time.perf_counter()
            result = analyze(model)
            durations.append((time.perf_counter() - started) * 1000)
        assert result["context_count"] == count
        results.append(
            {
                "contexts": count,
                "repetitions": repetitions,
                "median_ms": statistics.median(durations),
                "p95_ms": percentile_nearest_rank(durations, 0.95),
                "min_ms": min(durations),
                "max_ms": max(durations),
                "median_contexts_per_second": count / (statistics.median(durations) / 1000),
            }
        )
    return {"benchmark": "context-exposure-analysis", "results": results}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repetitions", type=int, default=5)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    if args.repetitions < 1:
        raise SystemExit("--repetitions must be positive")
    result = run(args.repetitions)
    rendered = json.dumps(result, indent=2)
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(rendered + "\n", encoding="utf-8")
    print(rendered)


if __name__ == "__main__":
    main()
