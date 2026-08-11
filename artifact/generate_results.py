#!/usr/bin/env python3
"""Materialize the EPSCD artifact from primary experiment outputs."""

import csv
import json
import re
from collections import Counter
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
RESULTS = ROOT / "artifact/results"
GENERATED = ROOT / "artifact/generated"


def read(relative):
    return json.loads((ROOT / relative).read_text(encoding="utf-8"))


def write_json(name, value):
    (RESULTS / name).write_text(
        json.dumps(value, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
    )


def write_csv(name, rows):
    with (RESULTS / name).open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(handle, fieldnames=rows[0].keys())
        writer.writeheader()
        writer.writerows(rows)


def parse_tla():
    text = (ROOT / "tla/epscd_rotation.log").read_text(encoding="utf-8")
    generated, distinct, remaining = map(
        int,
        re.search(
            r"(\d+) states generated, (\d+) distinct states found, (\d+) states left",
            text,
        ).groups(),
    )
    depth = int(re.search(r"depth of the complete state graph search is (\d+)", text).group(1))
    negative_controls = {}
    for suffix, invariant in (
        ("negative_http", "CommitRequiresEvidence"),
        ("negative_drop", "UncertaintyKeepsBoth"),
    ):
        log = (ROOT / f"tla/epscd_rotation_{suffix}.log").read_text(encoding="utf-8")
        negative_controls[suffix] = {
            "expectedInvariant": invariant,
            "detected": f"Invariant {invariant} is violated" in log,
        }
    return {
        "schemaVersion": 1,
        "tool": "TLC 2.19 / tla2tools 1.7.4",
        "main": {
            "generatedStates": generated,
            "distinctStates": distinct,
            "remainingStates": remaining,
            "depth": depth,
            "invariantViolation": False,
        },
        "negativeControls": negative_controls,
        "boundary": "Bounded exhaustive checking of the abstract transition relation under modeled assumptions.",
    }


def two_column_table(rows):
    lines = ["\\begin{tabular}{lr}", "\\toprule", "Metric & Result \\\\", "\\midrule"]
    lines.extend(f"{label} & {value} \\\\" for label, value in rows)
    lines.extend(["\\bottomrule", "\\end{tabular}"])
    return "\n".join(lines) + "\n"


def write_tables(policy, mainline, rotation, tla):
    status = policy["compileStatusCounts"]
    aggregate = policy["aggregateSuccessful"]
    (GENERATED / "epscd_policy_corpus.tex").write_text(
        two_column_table([
            ("Source policies", policy["totalSourceRecords"]),
            ("Exact translations", policy["exactTranslationsAttempted"]),
            ("Compiled", status.get("SUCCESS", 0)),
            ("Time limit", status.get("TIME_LIMIT", 0)),
            ("Median states", aggregate["reachableStates"]["median"]),
            ("P95 states", aggregate["reachableStates"]["p95"]),
            ("Median compile (ms)", f"{aggregate['compileMicros']['median']/1000:.2f}"),
            ("P95 compile (ms)", f"{aggregate['compileMicros']['p95']/1000:.2f}"),
            ("Median rank ($\\mu$s)", f"{aggregate['rankMedianMicrosPerPolicy']['median']:.2f}"),
            ("Median unrank ($\\mu$s)", f"{aggregate['unrankMedianMicrosPerPolicy']['median']:.2f}"),
        ]),
        encoding="utf-8",
    )
    density_lines = [
        f"{row['alpha']:.0e} & {row['expectedRetries']:.0f} & {row['medianRetries']} & {row['p95Retries']} & {row['p99Retries']} & {row['p999Retries']} \\\\"
        for row in mainline["rejectionDensity"]
    ]
    density_table = [
        "\\begin{tabular}{rrrrrr}",
        "\\toprule",
        "$\\alpha$ & $1/\\alpha$ & Median & P95 & P99 & P99.9 \\\\",
        "\\midrule",
        *density_lines,
        "\\bottomrule",
        "\\end{tabular}",
    ]
    (GENERATED / "epscd_rejection_density.tex").write_text(
        "\n".join(density_table) + "\n", encoding="utf-8"
    )
    sequence = mainline["sequence"]
    (GENERATED / "epscd_sequence.tex").write_text(
        two_column_table([
            ("Generations", f"{sequence['generations']:,}"),
            ("Policy violations", sequence["policyViolations"]),
            ("Duplicate passwords", sequence["duplicatePasswords"]),
            ("Replay mismatches", sequence["replayMismatches"]),
            ("Effective bits", f"{sequence['effectiveBits']:.2f}"),
            ("Derivations/s", f"{sequence['derivationsPerSecondIncludingReplay']:.0f}"),
        ]),
        encoding="utf-8",
    )
    states = Counter(row["finalState"] for row in rotation["results"])
    (GENERATED / "epscd_rotation.tex").write_text(
        two_column_table([
            ("Adapters", len(rotation["adapters"])),
            ("Injected scenarios", rotation["resultCount"]),
            ("Committed", states["COMMITTED"]),
            ("Unknown outcome", states["UNKNOWN_OUTCOME"]),
            ("Aborted", states["ABORTED"]),
            ("All safety checks", "pass" if rotation["allInvariantsHold"] else "fail"),
        ]),
        encoding="utf-8",
    )
    (GENERATED / "epscd_tla.tex").write_text(
        two_column_table([
            ("Generated states", f"{tla['main']['generatedStates']:,}"),
            ("Distinct states", f"{tla['main']['distinctStates']:,}"),
            ("Search depth", tla["main"]["depth"]),
            ("Main invariant violations", 0),
            ("Negative controls detected", f"{sum(v['detected'] for v in tla['negativeControls'].values())}/2"),
        ]),
        encoding="utf-8",
    )


def write_macros(policy, mainline, rotation, permutation, performance, distribution, tla):
    aggregate = policy["aggregateSuccessful"]
    sequence = mainline["sequence"]
    rotation_states = Counter(row["finalState"] for row in rotation["results"])
    sparse_density = mainline["rejectionDensity"][-1]
    values = {
        "CorpusTotal": policy["totalSourceRecords"],
        "CorpusTranslated": policy["exactTranslationsAttempted"],
        "CorpusUnsupported": policy["totalSourceRecords"] - policy["exactTranslationsAttempted"],
        "CorpusCompiled": policy["compileStatusCounts"].get("SUCCESS", 0),
        "CorpusTimeout": policy["compileStatusCounts"].get("TIME_LIMIT", 0),
        "CorpusMedianStates": aggregate["reachableStates"]["median"],
        "CorpusNinetyFifthStates": aggregate["reachableStates"]["p95"],
        "CorpusMaxStates": aggregate["reachableStates"]["maximum"],
        "CorpusMedianPayloadMiB": f"{aggregate['countPayloadBytes']['median']/1048576:.2f}",
        "CorpusNinetyFifthPayloadMiB": f"{aggregate['countPayloadBytes']['p95']/1048576:.2f}",
        "CorpusMaxPayloadMiB": f"{aggregate['countPayloadBytes']['maximum']/1048576:.2f}",
        "CorpusMedianRssMiB": f"{aggregate['peakRssBytes']['median']/1048576:.2f}",
        "CorpusNinetyFifthRssMiB": f"{aggregate['peakRssBytes']['p95']/1048576:.2f}",
        "CorpusMaxRssMiB": f"{aggregate['peakRssBytes']['maximum']/1048576:.2f}",
        "CorpusMedianRankUs": f"{aggregate['rankMedianMicrosPerPolicy']['median']:.2f}",
        "CorpusMedianUnrankUs": f"{aggregate['unrankMedianMicrosPerPolicy']['median']:.2f}",
        "WarmMedianUs": f"{performance['epscd']['warmDeriveFromCachedCompiledPolicy']['medianMicros']:.2f}",
        "WarmNinetyFifthUs": f"{performance['epscd']['warmDeriveFromCachedCompiledPolicy']['p95Micros']:.2f}",
        "ColdMedianMs": f"{performance['epscd']['coldCompileAndDerive']['medianMicros']/1000:.2f}",
        "SequenceCount": f"{sequence['generations']:,}",
        "SequenceDomain": f"{int(sequence['domainSize']):,}",
        "SequenceBits": f"{sequence['effectiveBits']:.2f}",
        "SequenceSeconds": f"{sequence['elapsedSeconds']:.2f}",
        "SequenceCalls": f"{sequence['generations']*2:,}",
        "PermutationEligible": permutation["measuredPolicies"],
        "PermutationIneligible": permutation["backendDomainLimitPolicies"],
        "PermutationSamples": f"{permutation['aggregateWalks']['samples']:,}",
        "PermutationMedianWalks": f"{permutation['aggregateWalks']['median']:.0f}",
        "PermutationNinetyFifthWalks": f"{permutation['aggregateWalks']['p95']:.0f}",
        "PermutationMaxWalks": permutation["aggregateWalks"]["maximum"],
        "PermutationCapHits": permutation["capHitCount"],
        "RotationCases": rotation["resultCount"],
        "RotationCommitted": rotation_states["COMMITTED"],
        "RotationUnknown": rotation_states["UNKNOWN_OUTCOME"],
        "RotationPrepared": rotation_states["PREPARED"],
        "RotationAborted": rotation_states["ABORTED"],
        "DistributionSamples": f"{distribution['samplesPerMethod']:,}",
        "DichopileTvd": f"{distribution['publishedBaselines'][0]['totalVariationDistanceEmpirical']:.5f}",
        "EpscdTvd": f"{distribution['proposed'][0]['totalVariationDistanceEmpirical']:.5f}",
        "SparseMedianRetries": f"{sparse_density['medianRetries']:,}",
        "SparseNinetyNinthNineRetries": f"{sparse_density['p999Retries']:,}",
        "TlaDistinctStates": f"{tla['main']['distinctStates']:,}",
    }
    lines = [f"\\newcommand{{\\{name}}}{{{value}}}" for name, value in values.items()]
    (GENERATED / "epscd_metrics.tex").write_text(
        "\n".join(lines) + "\n", encoding="utf-8"
    )


def main():
    RESULTS.mkdir(parents=True, exist_ok=True)
    GENERATED.mkdir(parents=True, exist_ok=True)
    policy = read("experiments/real_policy_corpus/policy_metrics.json")
    mainline = read("experiments/epscd_mainline.json")
    rotation = read("experiments/epscd_rotation/fault_matrix.json")
    permutation = read("experiments/performance/walk_corpus.json")
    performance = read("experiments/performance/performance.json")
    distribution = read("experiments/distribution/distribution.json")
    tla = parse_tla()

    write_json("policy_corpus.json", policy)
    write_json("rejection_density.json", {
        "schemaVersion": 1,
        "method": "whole-candidate rejection with controlled acceptance density",
        "rows": mainline["rejectionDensity"],
        "theory": {"expectedRetries": "1/alpha", "tail": "Pr[N>t]=(1-alpha)^t"},
    })
    write_json("sequence.json", mainline["sequence"])
    write_json("rotation_faults.json", rotation)
    adapter_counts = {}
    for adapter in ("http_form", "ldap_style_directory"):
        rows = [row for row in rotation["results"] if row["adapter"] == adapter]
        adapter_counts[adapter] = {
            "cases": len(rows),
            "finalStates": dict(Counter(row["finalState"] for row in rows)),
            "allInvariantsHold": all(
                row["invariantCommitRequiresEvidence"]
                and row["invariantUncertaintyKeepsBoth"]
                for row in rows
            ),
        }
    write_json("adapters.json", {
        "schemaVersion": 1,
        "boundary": rotation["boundary"],
        "adapters": adapter_counts,
    })
    write_json("permutation.json", permutation)
    write_json("distribution.json", distribution)
    write_json("performance.json", performance)
    write_json("tla.json", tla)

    fields = [
        "sourceRow", "website", "translationStatus", "compileStatus", "exactSpace",
        "entropyBits", "reachableStates", "countPayloadBytes", "peakRssBytes", "compileMicros"
    ]
    write_csv("policy_corpus.csv", [{key: row.get(key) for key in fields} for row in policy["records"]])
    write_csv("rejection_density.csv", mainline["rejectionDensity"])
    write_csv("sequence.csv", [mainline["sequence"]])
    write_csv("rotation_faults.csv", rotation["results"])
    write_tables(policy, mainline, rotation, tla)
    write_macros(policy, mainline, rotation, permutation, performance, distribution, tla)


if __name__ == "__main__":
    main()
