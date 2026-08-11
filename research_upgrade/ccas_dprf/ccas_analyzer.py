#!/usr/bin/env python3
"""Capability-closed effective access-structure analyzer.

This is a deliberately small reference implementation of the formal semantics
used by the CCAS research branch.  It treats protocol rules as monotone Horn
implications and enumerates deployment-domain compromise sets.  It is intended
for small security models, not for internet-scale attack-graph analysis.
"""

from __future__ import annotations

import argparse
import itertools
import json
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable, Mapping, Sequence


@dataclass(frozen=True)
class Domain:
    name: str
    capabilities: frozenset[str]
    cost: float = 1.0


@dataclass(frozen=True)
class Rule:
    name: str
    requires: frozenset[str]
    produces: frozenset[str]
    automatic: bool = True


@dataclass(frozen=True)
class Model:
    name: str
    domains: tuple[Domain, ...]
    public_capabilities: frozenset[str]
    shares: frozenset[str]
    nominal_qualified_sets: tuple[frozenset[str], ...]
    configured_domain_threshold: int
    rules: tuple[Rule, ...]


def load_models(path: Path) -> tuple[Model, ...]:
    raw = json.loads(path.read_text(encoding="utf-8"))
    cases = raw["cases"] if isinstance(raw, dict) else raw
    return tuple(_parse_model(case) for case in cases)


def _parse_model(raw: Mapping[str, object]) -> Model:
    domains = tuple(
        Domain(
            name=str(item["name"]),
            capabilities=frozenset(map(str, item.get("capabilities", []))),
            cost=float(item.get("cost", 1.0)),
        )
        for item in raw["domains"]
    )
    rules = tuple(
        Rule(
            name=str(item["name"]),
            requires=frozenset(map(str, item.get("requires", []))),
            produces=frozenset(map(str, item.get("produces", []))),
            automatic=bool(item.get("automatic", True)),
        )
        for item in raw.get("rules", [])
    )
    nominal = tuple(
        frozenset(map(str, qualified))
        for qualified in raw["nominal_qualified_sets"]
    )
    return Model(
        name=str(raw["name"]),
        domains=domains,
        public_capabilities=frozenset(map(str, raw.get("public_capabilities", []))),
        shares=frozenset(map(str, raw["shares"])),
        nominal_qualified_sets=nominal,
        configured_domain_threshold=int(raw["configured_domain_threshold"]),
        rules=rules,
    )


def capability_closure(
    model: Model, compromised_names: Iterable[str]
) -> tuple[frozenset[str], dict[str, dict[str, object]]]:
    """Return the least fixed point Cl(X) and one derivation proof per fact."""
    compromised = frozenset(compromised_names)
    known = set(model.public_capabilities)
    proof: dict[str, dict[str, object]] = {
        cap: {"kind": "public"} for cap in sorted(model.public_capabilities)
    }

    for domain in sorted(model.domains, key=lambda item: item.name):
        if domain.name not in compromised:
            continue
        for cap in sorted(domain.capabilities):
            known.add(cap)
            proof.setdefault(cap, {"kind": "initial", "domain": domain.name})

    changed = True
    while changed:
        changed = False
        for rule in model.rules:
            if not rule.automatic or not rule.requires.issubset(known):
                continue
            for cap in sorted(rule.produces):
                if cap in known:
                    continue
                known.add(cap)
                proof[cap] = {
                    "kind": "rule",
                    "rule": rule.name,
                    "requires": sorted(rule.requires),
                }
                changed = True

    return frozenset(known), proof


def _powerset(names: Sequence[str]) -> Iterable[frozenset[str]]:
    for size in range(len(names) + 1):
        for subset in itertools.combinations(names, size):
            yield frozenset(subset)


def _matching_nominal_set(model: Model, closure: frozenset[str]) -> frozenset[str] | None:
    available_shares = closure & model.shares
    for qualified in model.nominal_qualified_sets:
        if qualified.issubset(available_shares):
            return qualified
    return None


def _minimal_sets(qualified_sets: Sequence[frozenset[str]]) -> list[frozenset[str]]:
    ordered = sorted(qualified_sets, key=lambda item: (len(item), sorted(item)))
    result: list[frozenset[str]] = []
    for candidate in ordered:
        if not any(existing.issubset(candidate) for existing in result):
            result.append(candidate)
    return result


def _proof_steps(
    capability: str,
    proof: Mapping[str, Mapping[str, object]],
    seen: set[str] | None = None,
) -> list[str]:
    seen = set() if seen is None else seen
    if capability in seen:
        return []
    seen.add(capability)
    source = proof[capability]
    kind = source["kind"]
    if kind == "public":
        return [f"public capability: {capability}"]
    if kind == "initial":
        return [f"compromise {source['domain']} -> obtain {capability}"]

    steps: list[str] = []
    requirements = list(source["requires"])
    for requirement in requirements:
        steps.extend(_proof_steps(str(requirement), proof, seen))
    joined = " + ".join(map(str, requirements)) if requirements else "true"
    steps.append(f"{joined} --[{source['rule']}]--> {capability}")
    return steps


def analyze(model: Model) -> dict[str, object]:
    names = tuple(domain.name for domain in model.domains)
    costs = {domain.name: domain.cost for domain in model.domains}
    qualified: list[frozenset[str]] = []
    records: dict[frozenset[str], tuple[frozenset[str], dict[str, dict[str, object]], frozenset[str]]] = {}

    for compromised in _powerset(names):
        closure, proof = capability_closure(model, compromised)
        matched = _matching_nominal_set(model, closure)
        if matched is not None:
            qualified.append(compromised)
            records[compromised] = (closure, proof, matched)

    minimal = _minimal_sets(qualified)
    tau_eff = min((len(item) for item in qualified), default=None)
    rho_eff = min(
        (sum(costs[name] for name in item) for item in qualified),
        default=None,
    )
    tau_nom = min(len(item) for item in model.nominal_qualified_sets)

    witnesses = []
    for compromised in minimal:
        closure, proof, matched = records[compromised]
        steps: list[str] = []
        for share in sorted(matched):
            steps.extend(_proof_steps(share, proof))
        witnesses.append(
            {
                "compromised_domains": sorted(compromised),
                "cost": sum(costs[name] for name in compromised),
                "nominal_share_set_reached": sorted(matched),
                "closure": sorted(closure),
                "trace": list(dict.fromkeys(steps)),
            }
        )

    return {
        "case": model.name,
        "nominal_share_threshold": tau_nom,
        "configured_domain_threshold": model.configured_domain_threshold,
        "effective_domain_threshold": tau_eff,
        "weighted_effective_compromise_cost": rho_eff,
        "threshold_collapse": (
            tau_eff is not None and tau_eff < model.configured_domain_threshold
        ),
        "minimal_compromising_domain_sets": [sorted(item) for item in minimal],
        "gamma_eff": [sorted(item) for item in qualified],
        "witnesses": witnesses,
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("model", type=Path, help="JSON file containing one or more cases")
    parser.add_argument("--output", type=Path, help="optional JSON output path")
    args = parser.parse_args()

    results = [analyze(model) for model in load_models(args.model)]
    rendered = json.dumps({"results": results}, indent=2, ensure_ascii=False)
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(rendered + "\n", encoding="utf-8")
    print(rendered)


if __name__ == "__main__":
    main()
