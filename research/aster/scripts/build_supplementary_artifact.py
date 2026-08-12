#!/usr/bin/env python3
"""Build a deterministic, ASTER-only supplementary artifact.

The archive is assembled from an explicit allow-list. Manuscript trees,
submission files, temporary output, caches, logs, and unrelated research are
never traversed. The build also fails if a retired internal research name is
present in either an archived path or text file.
"""

from __future__ import annotations

import argparse
import re
import stat
import zipfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
ASTER = ROOT / "research" / "aster"
ARCHIVE_ROOT = "ASTER_CEE_Supplementary_Artifact"
FIXED_TIMESTAMP = (2026, 8, 12, 0, 0, 0)

# Split retired names so the packaging guard does not trigger on its own source.
FORBIDDEN = re.compile(
    "|".join(
        (
            "EP" + "SCD",
            "PS" + "PPD",
            "CE" + "TS",
            r"View[ _-]?" + "Key",
            r"Data[ _-]?" + "Key",
            r"factor[ _-]?" + "preserving",
            r"opaque[ _-]?" + "peer",
            r"peer[ _-]?" + "recovery",
            "Local" + "Root",
            "Local" + "Lineage",
            "Remote" + "BroadPRF",
            "ASTER" + "Exact",
            r"Encoder[ _-]?" + "v2",
            "Key" + "LessPass",
        )
    ),
    re.IGNORECASE,
)

ASTER_TOP_LEVEL = {
    "README.md",
    "LIMITATIONS.md",
    "PROTOCOL_V0_1.md",
    "SECURITY_REVIEW.md",
    "SPEC.md",
    "THREAT_MODEL.md",
}
ASTER_SUBTREES = {
    "adapters",
    "experiments",
    "mpc",
    "results",
    "scripts",
    "semantic",
    "tla",
}
RUST_EXAMPLES = {"aster_experiments.rs", "aster_rq1.rs"}

ARTIFACT_LIB_RS = """pub mod aster_exact_domain;
pub mod error;
pub mod permutation;
pub mod policy;
#[cfg(feature = "research")]
pub mod research;
"""

ARTIFACT_TOOLCHAIN = """[toolchain]
channel = "1.87.0"
profile = "minimal"
"""


def is_generated(path: Path) -> bool:
    return any(part in {"__pycache__", "target"} for part in path.parts) or path.name in {
        ".DS_Store"
    } or path.suffix in {".pyc", ".pyo", ".log"}


def tree_files(root: Path) -> list[Path]:
    return [path for path in root.rglob("*") if path.is_file() and not is_generated(path)]


def selected_files() -> list[Path]:
    files: list[Path] = []
    files.extend(ASTER / name for name in ASTER_TOP_LEVEL)
    for subtree in ASTER_SUBTREES:
        files.extend(tree_files(ASTER / subtree))

    corpus = ROOT / "experiments" / "real_policy_corpus"
    files.extend(tree_files(corpus))

    rust = ROOT / "rust_core"
    files.append(rust / "src" / "error.rs")
    for subtree in ("aster_exact_domain", "permutation", "policy", "research"):
        files.extend(tree_files(rust / "src" / subtree))
    files.append(rust / "test-vectors" / "aster-exact-domain-scheme-v1.json")
    files.extend(rust / "examples" / name for name in RUST_EXAMPLES)
    artifact_crate = ASTER / "artifact-crate"
    files.extend((artifact_crate / "Cargo.toml", artifact_crate / "Cargo.lock"))

    missing = [path for path in files if not path.is_file()]
    if missing:
        raise SystemExit("missing required artifact files: " + ", ".join(map(str, missing)))
    return sorted(set(files), key=lambda path: path.relative_to(ROOT).as_posix())


def audit(path: Path) -> None:
    relative = path.relative_to(ROOT).as_posix()
    if FORBIDDEN.search(relative):
        raise SystemExit(f"retired research name in artifact path: {relative}")
    try:
        text = artifactize_text(path.read_text(encoding="utf-8"))
    except (UnicodeDecodeError, OSError):
        return
    match = FORBIDDEN.search(text)
    if match:
        raise SystemExit(
            f"retired research name in artifact text: {relative}: {match.group(0)}"
        )


def artifactize_text(text: str) -> str:
    """Remove product-era identifiers from the standalone ASTER artifact."""
    return (
        text.replace("keylesspass_core", "aster_core")
        .replace("KeylessPassError", "AsterCoreError")
        .replace("keylesspass error", "ASTER core error")
    )


def add_bytes(archive: zipfile.ZipFile, destination: str, content: bytes, mode: int) -> None:
    info = zipfile.ZipInfo(destination, FIXED_TIMESTAMP)
    info.compress_type = zipfile.ZIP_DEFLATED
    info.external_attr = (stat.S_IFREG | mode) << 16
    archive.writestr(info, content, compresslevel=9)


def add_file(archive: zipfile.ZipFile, source: Path) -> None:
    relative = source.relative_to(ROOT).as_posix()
    content = source.read_bytes()
    try:
        content = artifactize_text(content.decode("utf-8")).encode("utf-8")
    except UnicodeDecodeError:
        pass
    if relative.startswith("research/aster/artifact-crate/"):
        relative = "rust_core/" + relative.removeprefix(
            "research/aster/artifact-crate/"
        )
        if relative == "rust_core/Cargo.toml":
            content = (
                content.decode("utf-8")
                .replace(
                    'path = "../../../rust_core/examples/aster_experiments.rs"',
                    'path = "examples/aster_experiments.rs"',
                )
                .replace(
                    'path = "../../../rust_core/examples/aster_rq1.rs"',
                    'path = "examples/aster_rq1.rs"',
                )
                .encode("utf-8")
            )
    destination = f"{ARCHIVE_ROOT}/{relative}"
    mode = 0o755 if source.stat().st_mode & stat.S_IXUSR else 0o644
    add_bytes(archive, destination, content, mode)


def audit_virtual(destination: str, content: str) -> None:
    match = FORBIDDEN.search(destination) or FORBIDDEN.search(content)
    if match:
        raise SystemExit(
            f"retired research name in generated artifact file {destination}: {match.group(0)}"
        )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    files = selected_files()
    for path in files:
        audit(path)
    virtual_files = {
        "rust-toolchain.toml": ARTIFACT_TOOLCHAIN,
        "rust_core/src/lib.rs": ARTIFACT_LIB_RS,
    }
    for destination, content in virtual_files.items():
        audit_virtual(destination, content)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(args.output, "w") as archive:
        for path in files:
            add_file(archive, path)
        for destination, content in sorted(virtual_files.items()):
            add_bytes(
                archive,
                f"{ARCHIVE_ROOT}/{destination}",
                content.encode("utf-8"),
                0o644,
            )
    print(f"{args.output}: {len(files) + len(virtual_files)} files")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
