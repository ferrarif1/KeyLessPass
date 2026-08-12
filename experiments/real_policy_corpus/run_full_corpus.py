#!/usr/bin/env python3
"""Run every exact PCP translation under one fixed per-policy budget."""

import json
import platform
import statistics
import subprocess
import sys
import time
from collections import Counter
from pathlib import Path


MAX_STATES = 250_000
MAX_MEMORY_BYTES = 4 * 1024**3
MAX_WALL_SECONDS = 60
ROOT = Path(__file__).resolve().parents[2]
CORPUS = ROOT / "experiments/real_policy_corpus/translated_corpus.json"
OUTPUT = ROOT / "experiments/real_policy_corpus/policy_metrics.json"
WORKER = ROOT / "rust_core/target/release/examples/compile_policy_worker"


def percentile(values, fraction):
    ordered = sorted(values)
    return ordered[round((len(ordered) - 1) * fraction)]


def summary(values):
    if not values:
        return None
    return {
        "samples": len(values),
        "mean": statistics.fmean(values),
        "median": percentile(values, 0.50),
        "p95": percentile(values, 0.95),
        "p99": percentile(values, 0.99),
        "maximum": max(values),
        "standardDeviation": statistics.pstdev(values),
    }


def resident_bytes(pid):
    """Return RSS for one worker without adding a third-party dependency."""
    result = subprocess.run(
        ["ps", "-o", "rss=", "-p", str(pid)],
        text=True,
        capture_output=True,
        check=False,
    )
    try:
        return int(result.stdout.strip()) * 1024
    except ValueError:
        return None


def run_record(record):
    base = {
        "sourceRow": record["sourceRow"],
        "website": record["website"],
        "translationStatus": record["translationStatus"],
        "minLength": record["policySpec"]["minLength"],
        "maxLength": record["policySpec"]["maxLength"],
        "alphabetSize": len(record["policySpec"]["alphabet"]),
    }
    command = [str(WORKER), str(CORPUS), str(record["sourceRow"])]
    process = subprocess.Popen(command, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    started = time.monotonic()
    peak_rss = 0
    termination = None
    while process.poll() is None:
        rss = resident_bytes(process.pid)
        if rss is not None:
            peak_rss = max(peak_rss, rss)
            if rss > MAX_MEMORY_BYTES:
                termination = "MEMORY_LIMIT"
                process.kill()
                break
        if time.monotonic() - started > MAX_WALL_SECONDS:
            termination = "TIME_LIMIT"
            process.kill()
            break
        time.sleep(0.05)
    stdout, stderr = process.communicate()
    if termination == "TIME_LIMIT":
        return {
            **base,
            "compileStatus": "TIME_LIMIT",
            "wallTimeLimitSeconds": MAX_WALL_SECONDS,
            "error": f"worker exceeded {MAX_WALL_SECONDS} seconds",
            "peakRssBytes": peak_rss or None,
            "stderr": stderr[-2000:],
        }
    if termination == "MEMORY_LIMIT":
        return {
            **base,
            "compileStatus": "MEMORY_LIMIT",
            "memoryLimitBytes": MAX_MEMORY_BYTES,
            "peakRssBytes": peak_rss,
            "stderr": stderr[-2000:],
        }
    if process.returncode != 0:
        stderr = stderr[-4000:]
        memory_markers = ("memory allocation", "cannot allocate memory", "out of memory")
        status = "MEMORY_LIMIT" if any(marker in stderr.lower() for marker in memory_markers) else "INTERNAL_ERROR"
        return {
            **base,
            "compileStatus": status,
            "returnCode": process.returncode,
            "peakRssBytes": peak_rss or None,
            "stderr": stderr,
        }
    try:
        result = json.loads(stdout)
    except json.JSONDecodeError as error:
        return {
            **base,
            "compileStatus": "INTERNAL_ERROR",
            "error": f"worker emitted invalid JSON: {error}",
            "stdout": stdout[-2000:],
            "stderr": stderr[-2000:],
        }
    result["peakRssBytes"] = peak_rss or None
    return result


def main():
    if not WORKER.exists():
        raise SystemExit(f"build the release worker first: {WORKER}")
    corpus = json.loads(CORPUS.read_text(encoding="utf-8"))
    translated = [record for record in corpus["records"] if record["translationStatus"] == "translated"]
    results = []
    for index, record in enumerate(translated, 1):
        result = run_record(record)
        results.append(result)
        print(f"[{index:3}/{len(translated)}] row={record['sourceRow']} {result['compileStatus']}", flush=True)

    successful = [row for row in results if row["compileStatus"] == "SUCCESS"]
    status_counts = Counter(row["compileStatus"] for row in results)
    rank_medians = [row["rankMicros"]["median"] for row in successful]
    unrank_medians = [row["unrankMicros"]["median"] for row in successful]
    output = {
        "schemaVersion": 2,
        "source": corpus["source"],
        "translation": corpus["translation"],
        "budgetFixedBeforeRun": {
            "maxStates": MAX_STATES,
            "maxMemoryBytes": MAX_MEMORY_BYTES,
            "maxWallTimeSecondsPerPolicy": MAX_WALL_SECONDS,
            "preCompilationLengthFilter": None,
        },
        "environment": {
            "platform": platform.platform(),
            "python": sys.version.split()[0],
            "worker": str(WORKER.relative_to(ROOT)),
        },
        "totalSourceRecords": len(corpus["records"]),
        "exactTranslationsAttempted": len(translated),
        "compileStatusCounts": dict(sorted(status_counts.items())),
        "aggregateSuccessful": {
            "reachableStates": summary([row["reachableStates"] for row in successful]),
            "countPayloadBytes": summary([row["countPayloadBytes"] for row in successful]),
            "peakRssBytes": summary([row["peakRssBytes"] for row in successful if row["peakRssBytes"] is not None]),
            "compileMicros": summary([row["compileMicros"] for row in successful]),
            "rankMedianMicrosPerPolicy": summary(rank_medians),
            "unrankMedianMicrosPerPolicy": summary(unrank_medians),
        },
        "metricBoundary": "countPayloadBytes sums serialized BigUint payloads. peakRssBytes is the maximum sampled resident set of the isolated worker and may miss short-lived peaks.",
        "records": results,
    }
    OUTPUT.write_text(json.dumps(output, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
