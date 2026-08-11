#!/usr/bin/env python3
"""Clear reference for the fixed MP-SPDZ ASTER exact-domain vector."""

from __future__ import annotations

import argparse
import json
import subprocess
import tempfile
from pathlib import Path


DOMAIN_SIZE = 1_000_003
DOMAIN_BITS = 20
HALF_BITS = 10
ROUNDS = 10
MAX_CYCLE_WALKS = 4
GENERATION = 42
CONTEXT_TWEAK = 0x6F8E6FAB7BFC9321BE8FD9F529874379


def aes128_ecb(key: bytes, block: bytes) -> bytes:
    with tempfile.TemporaryDirectory(prefix="aster-aes-") as directory:
        root = Path(directory)
        source = root / "in.bin"
        target = root / "out.bin"
        source.write_bytes(block)
        subprocess.run(
            [
                "openssl",
                "enc",
                "-aes-128-ecb",
                "-K",
                key.hex(),
                "-nosalt",
                "-nopad",
                "-in",
                str(source),
                "-out",
                str(target),
            ],
            check=True,
            capture_output=True,
        )
        return target.read_bytes()


def permute(key: bytes, value: int) -> int:
    mask = (1 << HALF_BITS) - 1
    left = value & mask
    right = (value >> HALF_BITS) & mask
    for round_no in range(ROUNDS):
        separator = CONTEXT_TWEAK ^ ((round_no + 1) << 112)
        block_int = right ^ separator
        encrypted = aes128_ecb(key, block_int.to_bytes(16, "big"))
        f = int.from_bytes(encrypted, "big") & mask
        left, right = right, left ^ f
    return left | (right << HALF_BITS)


def cycle_walk(key: bytes) -> tuple[int, int]:
    candidate = GENERATION
    for walks in range(1, MAX_CYCLE_WALKS + 1):
        candidate = permute(key, candidate)
        if candidate < DOMAIN_SIZE:
            return candidate, walks
    raise RuntimeError("fixed cycle-walk cap exhausted")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    contributions = [
        0x00112233445566778899AABBCCDDEEFF,
        0x0F0E0D0C0B0A09080706050403020100,
        0xA5A5A5A55A5A5A5A1122334455667788,
        0x1234567890ABCDEFFEDCBA0987654321,
        0x55AA55AA55AA55AAAA55AA55AA55AA55,
    ]
    configurations = []
    for parties in (3, 5):
        effective_key = 0
        for contribution in contributions[:parties]:
            effective_key ^= contribution
        rank, walks = cycle_walk(effective_key.to_bytes(16, "big"))
        configurations.append(
            {
                "parties": parties,
                "corruption_threshold": (parties - 1) // 2,
                "effective_key_hex": f"{effective_key:032x}",
                "rank": rank,
                "walks": walks,
            }
        )
    result = {
        "domain_size": DOMAIN_SIZE,
        "domain_bits": DOMAIN_BITS,
        "rounds": ROUNDS,
        "max_cycle_walks": MAX_CYCLE_WALKS,
        "generation": GENERATION,
        "context_tweak_hex": f"{CONTEXT_TWEAK:032x}",
        "party_contributions_hex": [f"{x:032x}" for x in contributions],
        "configurations": configurations,
    }
    encoded = json.dumps(result, indent=2, sort_keys=True) + "\n"
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(encoded, encoding="utf-8")
    print(encoded, end="")


if __name__ == "__main__":
    main()
