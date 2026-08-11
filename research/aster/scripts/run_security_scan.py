#!/usr/bin/env python3
"""Fail-closed scan for persisted credential or root-key values in ASTER results."""

from __future__ import annotations

import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
ASTER = ROOT / "research/aster"
RESULTS = ASTER / "results"
FORBIDDEN_KEYS = {
    "password",
    "oldpassword",
    "newpassword",
    "candidatepassword",
    "rootkey",
    "rootepochkey",
    "lineagekey",
    "permutationkey",
    "secretshare",
}


def normalized(value: str) -> str:
    return "".join(character.lower() for character in value if character.isalnum())


def walk(value: object, path: str, findings: list[dict[str, str]]) -> None:
    if isinstance(value, dict):
        for key, child in value.items():
            child_path = f"{path}.{key}"
            if normalized(str(key)) in FORBIDDEN_KEYS and child not in (None, False, "", 0):
                findings.append({"path": child_path, "reason": "forbidden secret-value key"})
            walk(child, child_path, findings)
    elif isinstance(value, list):
        for index, child in enumerate(value):
            walk(child, f"{path}[{index}]", findings)


def main() -> None:
    findings: list[dict[str, str]] = []
    scanned: list[str] = []
    for file in sorted(RESULTS.rglob("*.json")):
        scanned.append(str(file.relative_to(ROOT)))
        try:
            document = json.loads(file.read_text(encoding="utf-8"))
        except json.JSONDecodeError as error:
            findings.append({"path": str(file), "reason": f"invalid JSON: {error}"})
            continue
        walk(document, str(file.relative_to(ROOT)), findings)
    result = {
        "schemaVersion": 1,
        "scannedJsonFiles": scanned,
        "forbiddenKeyFindings": findings,
        "pass": not findings,
        "boundary": (
            "Structured-result key scan plus Rust journal-schema tests; this does not "
            "prove absence from process memory, swap, operating-system crash dumps, or untracked files."
        ),
    }
    output = RESULTS / "generated/security_scan.json"
    output.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(json.dumps(result, indent=2))
    if findings:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
