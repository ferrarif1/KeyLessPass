#!/usr/bin/env python3
"""Create a deterministic evidence manifest for the ASTER release bundle."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
from pathlib import Path

from pypdf import PdfReader


ROOT = Path(__file__).resolve().parents[3]
ASTER = ROOT / "research" / "aster"
OUTPUT = ASTER / "output"


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def entry(path: Path) -> dict[str, object]:
    return {
        "path": str(path.relative_to(ASTER)),
        "bytes": path.stat().st_size,
        "sha256": sha256(path),
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--version-suffix",
        default="",
        help="Suffix appended to final manuscript and manifest basenames, e.g. _v2",
    )
    args = parser.parse_args()
    if args.version_suffix and not re.fullmatch(r"_[A-Za-z0-9.-]+", args.version_suffix):
        parser.error("--version-suffix must be empty or begin with '_' and contain only safe filename characters")

    suffix = args.version_suffix
    manuscript = ASTER / "paper" / "ASTER_manuscript.md"
    docx = OUTPUT / f"ASTER_manuscript_final{suffix}.docx"
    pdf = OUTPUT / f"ASTER_manuscript_final{suffix}.pdf"
    final_md = OUTPUT / f"ASTER_manuscript_final{suffix}.md"
    tests = json.loads((ASTER / "results" / "raw" / "test_suite.json").read_text())
    blockers = json.loads((ASTER / "results" / "generated" / "BLOCKERS.json").read_text())
    scan = json.loads((ASTER / "results" / "generated" / "security_scan.json").read_text())
    paper_text = manuscript.read_text()
    references = sorted({int(value) for value in re.findall(r"^\[(\d+)\]", paper_text, re.M)})
    raw_files = sorted((ASTER / "results" / "raw").glob("*"))
    generated_files = sorted((ASTER / "results" / "generated").glob("*"))
    result = {
        "schemaVersion": 1,
        "release": f"ASTER-2026-08-11{suffix}",
        "manuscripts": [entry(path) for path in (final_md, docx, pdf)],
        "pdfPages": len(PdfReader(pdf).pages),
        "paperAudit": {
            "wordCountApprox": len(re.findall(r"\b[\w'-]+\b", paper_text)),
            "references": references,
            "unresolvedMarkers": [
                marker
                for marker in ("CODEX-RESULT", "PIVOT", "full artifact will")
                if marker in paper_text
            ],
        },
        "tests": {
            "cargoPassed": tests["cargo"]["totalPassed"],
            "cargoFailed": tests["cargo"]["totalFailed"],
            "pythonSemanticRun": tests["pythonSemantic"]["totalRun"],
            "pass": tests["acceptanceCriterion"],
        },
        "allBlockersResolved": all(item["resolved"] for item in blockers["blockers"]),
        "structuredSecurityScanPass": scan["pass"],
        "rawEvidence": [entry(path) for path in raw_files if path.is_file()],
        "generatedEvidence": [entry(path) for path in generated_files if path.is_file()],
        "boundaries": {
            "mpc": "Three repetitions per configuration on single-host loopback; no LAN/WAN, DKG, share-refresh, or history-window MPC timing claim.",
            "adapter": "Real loopback HTTP and durable LDAP-style fault targets plus a pinned single-server OpenLDAP modify/bind smoke test; no replication or production-performance claim.",
            "secretScan": scan["boundary"],
        },
    }
    target = OUTPUT / f"RELEASE_MANIFEST{suffix}.json"
    target.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n")
    print(target)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
