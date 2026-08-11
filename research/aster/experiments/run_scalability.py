#!/usr/bin/env python3
"""ASTER RQ7 public-metadata and replay-ledger scalability experiment."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import sqlite3
import tempfile
import time


SCALES = [100, 1_000, 10_000, 100_000]


def percentile(values, p):
    ordered = sorted(values)
    return ordered[min(len(ordered) - 1, int((len(ordered) - 1) * p))]


def run_scale(count: int):
    with tempfile.TemporaryDirectory(prefix="aster-scale-") as tmp:
        path = Path(tmp) / "state.sqlite"
        db = sqlite3.connect(path)
        db.execute("PRAGMA journal_mode=WAL")
        db.execute("PRAGMA synchronous=FULL")
        db.executescript(
            """CREATE TABLE credential(
                 id INTEGER PRIMARY KEY,
                 service_id BLOB NOT NULL,
                 account_id BLOB NOT NULL,
                 lineage_id BLOB NOT NULL,
                 root_epoch INTEGER NOT NULL,
                 generation INTEGER NOT NULL,
                 policy_hash BLOB NOT NULL
               );
               CREATE TABLE capability_use(
                 fingerprint BLOB PRIMARY KEY,
                 nonce BLOB NOT NULL,
                 used INTEGER NOT NULL,
                 budget INTEGER NOT NULL
               );"""
        )
        start = time.perf_counter_ns()
        with db:
            db.executemany(
                "INSERT INTO credential VALUES(?,?,?,?,?,?,?)",
                (
                    (
                        i,
                        i.to_bytes(16, "big"),
                        (i + 1).to_bytes(16, "big"),
                        (i + 2).to_bytes(16, "big"),
                        1,
                        i % 100,
                        (i.to_bytes(8, "big") * 4),
                    )
                    for i in range(count)
                ),
            )
            db.executemany(
                "INSERT INTO capability_use VALUES(?,?,1,1)",
                (
                    (
                        (i.to_bytes(8, "big") * 4),
                        (i.to_bytes(8, "big") * 2),
                    )
                    for i in range(count)
                ),
            )
        insert_ms = (time.perf_counter_ns() - start) / 1_000_000
        latencies = []
        for i in range(min(2_000, count)):
            key = (i * 7919) % count
            t0 = time.perf_counter_ns()
            db.execute("SELECT used,budget FROM capability_use WHERE fingerprint=?", ((key.to_bytes(8, "big") * 4),)).fetchone()
            latencies.append((time.perf_counter_ns() - t0) / 1_000)
        db.execute("PRAGMA wal_checkpoint(TRUNCATE)")
        db.close()
        size = path.stat().st_size
        return {
            "records": count,
            "insertMillis": insert_ms,
            "databaseBytes": size,
            "bytesPerCredentialAndReplayRow": size / count,
            "lookupSamples": len(latencies),
            "lookupMedianMicros": percentile(latencies, 0.5),
            "lookupP95Micros": percentile(latencies, 0.95),
            "lookupP99Micros": percentile(latencies, 0.99),
        }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", required=True)
    args = parser.parse_args()
    output = Path(args.output)
    output.parent.mkdir(parents=True, exist_ok=True)
    result = {
        "schemaVersion": 1,
        "rows": [run_scale(scale) for scale in SCALES],
        "boundary": "SQLite public credential metadata plus durable capability-use rows; no password values and no MPC computation.",
    }
    output.write_text(json.dumps(result, indent=2) + "\n")
    print(output)


if __name__ == "__main__":
    main()
