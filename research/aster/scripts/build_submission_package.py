#!/usr/bin/env python3
"""Build the CEE delivery manifest and deterministic submission ZIP."""

from __future__ import annotations

import hashlib
import json
import zipfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
FINAL = ROOT / "research" / "aster" / "submission" / "cee_2026-08-11_v2" / "final"
PACKAGE = FINAL / "ASTER_CEE_Submission_Package_v2.zip"
PACKAGE_HASH = FINAL / "ASTER_CEE_Submission_Package_v2.sha256"
MANIFEST = FINAL / "SUBMISSION_MANIFEST_v2.json"
FIXED_TIMESTAMP = (2026, 8, 12, 0, 0, 0)
EXCLUDED = {PACKAGE.name, PACKAGE_HASH.name, MANIFEST.name}


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def main() -> int:
    files = sorted(
        path for path in FINAL.iterdir() if path.is_file() and path.name not in EXCLUDED
    )
    manifest = {
        "package": "ASTER_CEE CEE submission v2",
        "prepared": "2026-08-12",
        "file_count": len(files),
        "files": [
            {"file": path.name, "bytes": path.stat().st_size, "sha256": sha256(path)}
            for path in files
        ],
    }
    MANIFEST.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    archive_files = files + [MANIFEST]
    with zipfile.ZipFile(PACKAGE, "w") as archive:
        for path in archive_files:
            info = zipfile.ZipInfo(path.name, FIXED_TIMESTAMP)
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = (0o100644 << 16)
            archive.writestr(info, path.read_bytes(), compresslevel=9)
    PACKAGE_HASH.write_text(f"{sha256(PACKAGE)}  {PACKAGE.name}\n", encoding="ascii")
    print(f"{PACKAGE}: {len(archive_files)} files")
    print(PACKAGE_HASH.read_text(encoding="ascii").strip())
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
