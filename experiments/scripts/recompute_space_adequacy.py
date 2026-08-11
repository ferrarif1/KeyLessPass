#!/usr/bin/env python3
"""Classify existing corpus counts without recompiling any policy."""

import argparse
import json
from pathlib import Path


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("input", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--floors", default="40,60,80")
    args = parser.parse_args()
    floors = [float(value) for value in args.floors.split(",")]

    source = json.loads(args.input.read_text())
    rows = []
    for record in source["records"]:
        if record.get("compileStatus") != "SUCCESS":
            continue
        bits = float(record["entropyBits"])
        exact_space = int(record["exactSpace"])
        backend_eligible = exact_space >= 1_000_000 and exact_space.bit_length() <= 512
        rows.append(
            {
                "sourceRow": record["sourceRow"],
                "website": record["website"],
                "exactSpace": record["exactSpace"],
                "credentialSpaceBits": bits,
                "backendDomainEligible": backend_eligible,
                "securityFloorEligible": {
                    f"{floor:g}Bits": bits >= floor for floor in floors
                },
            }
        )

    result = {
        "schemaVersion": 1,
        "source": str(args.input),
        "method": "reclassification of existing exact counts; no policy compilation",
        "backendRule": "N >= 1,000,000 and bit_length(N) <= 512",
        "configuredSecurityFloorsBits": floors,
        "completedPolicies": len(rows),
        "backendEligiblePolicies": sum(row["backendDomainEligible"] for row in rows),
        "securityFloorEligibleCounts": {
            f"{floor:g}Bits": sum(
                row["securityFloorEligible"][f"{floor:g}Bits"] for row in rows
            )
            for floor in floors
        },
        "records": rows,
        "boundary": "backend eligibility is not credential guessing strength",
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, indent=2) + "\n")


if __name__ == "__main__":
    main()
