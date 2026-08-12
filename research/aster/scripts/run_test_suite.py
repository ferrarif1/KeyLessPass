#!/usr/bin/env python3
"""Run ASTER's Rust and Python tests and persist machine-readable evidence."""

from __future__ import annotations

import json
import re
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
OUT = ROOT / "research" / "aster" / "results" / "raw" / "test_suite.json"


def run(command: list[str], cwd: Path) -> dict[str, object]:
    completed = subprocess.run(command, cwd=cwd, text=True, capture_output=True)
    return {
        "command": command,
        "returnCode": completed.returncode,
        "stdout": completed.stdout.replace(str(ROOT), "<repo>"),
        "stderr": completed.stderr.replace(str(ROOT), "<repo>"),
    }


def main() -> int:
    artifact_crate = ROOT / "research" / "aster" / "artifact-crate"
    if not artifact_crate.is_dir():
        artifact_crate = ROOT / "rust_core"
    cargo = run(
        ["cargo", "test", "--release", "--all-features", "--", "--quiet"],
        artifact_crate,
    )
    python = run(
        ["python3", "research/aster/semantic/test_aster_v0_1.py"],
        ROOT,
    )

    cargo_text = str(cargo["stdout"]) + str(cargo["stderr"])
    python_text = str(python["stdout"]) + str(python["stderr"])
    cargo_counts = [
        {"passed": int(passed), "failed": int(failed)}
        for passed, failed in re.findall(r"test result: ok\. (\d+) passed; (\d+) failed", cargo_text)
    ]
    python_match = re.search(r"Ran (\d+) tests?", python_text)
    evidence = {
        "schemaVersion": 1,
        "cargo": {
            **cargo,
            "testBinaries": cargo_counts,
            "totalPassed": sum(item["passed"] for item in cargo_counts),
            "totalFailed": sum(item["failed"] for item in cargo_counts),
        },
        "pythonSemantic": {
            **python,
            "totalRun": int(python_match.group(1)) if python_match else None,
            "passed": python["returnCode"] == 0,
        },
        "acceptanceCriterion": cargo["returnCode"] == 0 and python["returnCode"] == 0,
        "boundary": "Unit and protocol tests; not a cryptographic proof or production deployment assessment.",
    }
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(evidence, indent=2, sort_keys=True) + "\n")
    print(OUT)
    print(json.dumps({
        "cargoPassed": evidence["cargo"]["totalPassed"],
        "pythonRun": evidence["pythonSemantic"]["totalRun"],
        "pass": evidence["acceptanceCriterion"],
    }, indent=2))
    return 0 if evidence["acceptanceCriterion"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
