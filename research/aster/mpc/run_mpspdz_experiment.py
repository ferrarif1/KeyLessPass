#!/usr/bin/env python3
"""Run ASTER's fixed exact-domain circuit with official MP-SPDZ.

The raw JSON preserves complete compiler/protocol output.  The script reports
only values actually emitted by MP-SPDZ or measured around the container; it
does not estimate WAN performance or claim arbitrary-subset online
availability.
"""

from __future__ import annotations

import argparse
import json
import platform
import re
import statistics
import subprocess
import time
from pathlib import Path


IMAGE = "aster-mpspdz:mal-shamir-bmr-max5"
CONTRIBUTIONS = [
    0x00112233445566778899AABBCCDDEEFF,
    0x0F0E0D0C0B0A09080706050403020100,
    0xA5A5A5A55A5A5A5A1122334455667788,
    0x1234567890ABCDEFFEDCBA0987654321,
    0x55AA55AA55AA55AAAA55AA55AA55AA55,
]


def percentile(values: list[float], p: float) -> float:
    ordered = sorted(values)
    if len(ordered) == 1:
        return ordered[0]
    position = (len(ordered) - 1) * p
    lower = int(position)
    upper = min(lower + 1, len(ordered) - 1)
    fraction = position - lower
    return ordered[lower] * (1 - fraction) + ordered[upper] * fraction


def emitted_metrics(output: str) -> dict[str, object]:
    ranks = [int(x) for x in re.findall(r"ASTER_MPC_RANK=(\d+)", output)]
    successes = [int(x) for x in re.findall(r"ASTER_MPC_SUCCESS=(\d+)", output)]
    protocol_seconds = [float(x) for x in re.findall(r"Time\s*=\s*([0-9.]+)\s*seconds", output)]
    data_mb = [float(x) for x in re.findall(r"Data sent\s*=\s*([0-9.]+)\s*MB", output)]
    global_data_mb = [
        float(x) for x in re.findall(r"Global data sent\s*=\s*([0-9.]+)\s*MB", output)
    ]
    rounds = [int(x) for x in re.findall(r"in\s+~?(\d+)\s+rounds", output)]
    return {
        "ranks": ranks,
        "successes": successes,
        "protocolSeconds": protocol_seconds,
        "dataSentMB": data_mb,
        "globalDataSentMB": global_data_mb,
        "roundCounts": rounds,
    }


def run_configuration(mpc_dir: Path, parties: int, repetitions: int) -> dict[str, object]:
    def signed_word(word: int) -> int:
        return word if word < (1 << 63) else word - (1 << 64)

    inputs = "\n".join(
        f"echo '{signed_word(value & ((1 << 64) - 1))} "
        f"{signed_word(value >> 64)}' > Player-Data/Input-P{party}-0"
        for party, value in enumerate(CONTRIBUTIONS[:parties])
    )
    runs = "\n".join(
        f"echo ASTER_RUN_BEGIN={rep}; PLAYERS={parties} ./Scripts/mal-shamir-bmr.sh "
        f"aster_exact_domain-{parties}; echo ASTER_RUN_END={rep}"
        for rep in range(repetitions)
    )
    command = [
        "docker",
        "run",
        "--rm",
        "-v",
        f"{mpc_dir.resolve()}:/artifact:ro",
        IMAGE,
        "bash",
        "-lc",
        "\n".join(
            [
                "set -euo pipefail",
                "cp /artifact/aster_exact_domain.mpc Programs/Source/aster_exact_domain.mpc",
                f"./Scripts/setup-ssl.sh {parties} Player-Data >/tmp/aster-ssl.log 2>&1",
                inputs,
                f"./compile.py -GB 128 aster_exact_domain {parties}",
                runs,
            ]
        ),
    ]
    started = time.perf_counter()
    completed = subprocess.run(command, text=True, capture_output=True)
    wall = time.perf_counter() - started
    combined = completed.stdout + "\n" + completed.stderr
    if completed.returncode != 0:
        raise RuntimeError(
            f"MP-SPDZ {parties}-party run failed ({completed.returncode})\n{combined}"
        )
    metrics = emitted_metrics(combined)
    if len(metrics["ranks"]) != repetitions or any(x != 1 for x in metrics["successes"]):
        raise RuntimeError(f"missing or unsuccessful fixed-vector outputs\n{combined}")
    # Script markers let us retain end-to-end wall time without pretending the
    # compiler cost is part of one online evaluation. MP-SPDZ-emitted protocol
    # times remain separately available in protocolSeconds.
    blocks = re.findall(
        r"ASTER_RUN_BEGIN=\d+\n(.*?)ASTER_RUN_END=\d+", combined, re.DOTALL
    )
    run_metrics = [emitted_metrics(block) for block in blocks]
    protocol_times = [
        max(item["protocolSeconds"])
        for item in run_metrics
        if item["protocolSeconds"]
    ]
    summary = None
    if protocol_times:
        summary = {
            "samples": len(protocol_times),
            "medianSeconds": statistics.median(protocol_times),
            "p95Seconds": percentile(protocol_times, 0.95),
            "p99Seconds": percentile(protocol_times, 0.99),
        }
    return {
        "parties": parties,
        "corruptionThreshold": (parties - 1) // 2,
        "onlineAvailabilityClaim": "all configured parties participate in this experiment",
        "repetitionsRequested": repetitions,
        "containerWallSecondsIncludingCompilation": wall,
        "emitted": metrics,
        "perRunEmitted": run_metrics,
        "protocolTimeSummary": summary,
        "stdout": completed.stdout,
        "stderr": completed.stderr,
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repetitions", type=int, default=3)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--mpc-dir", type=Path, default=Path(__file__).resolve().parent)
    args = parser.parse_args()
    if args.repetitions < 1:
        raise SystemExit("--repetitions must be positive")
    reference = json.loads((args.mpc_dir.parent / "results/raw/rq6_reference_vector.json").read_text())
    rows = [run_configuration(args.mpc_dir, parties, args.repetitions) for parties in (3, 5)]
    expected = {x["parties"]: x["rank"] for x in reference["configurations"]}
    for row in rows:
        row["expectedRank"] = expected[row["parties"]]
        row["fixedVectorAgreement"] = all(
            rank == row["expectedRank"] for rank in row["emitted"]["ranks"]
        )
        if not row["fixedVectorAgreement"]:
            raise RuntimeError(f"fixed-vector disagreement: {row}")
    result = {
        "schemaVersion": 1,
        "backend": "MP-SPDZ mal-shamir-bmr-party.x",
        "image": IMAGE,
        "topology": "single-host loopback Docker container",
        "host": platform.platform(),
        "constructionBoundary": (
            "Generic malicious honest-majority MPC evaluation of an ASTER-composed "
            "AES Feistel/cycle-walk circuit; not FF1 and not a new threshold primitive."
        ),
        "sampleBoundary": (
            "Small-sample fixed-vector feasibility run. No LAN/WAN, DKG, share-refresh, "
            "or history-window MPC timing is inferred from these measurements."
        ),
        "reference": reference,
        "rows": rows,
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n")
    print(json.dumps({"output": str(args.output), "rows": rows}, indent=2))


if __name__ == "__main__":
    main()
