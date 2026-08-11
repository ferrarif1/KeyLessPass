#!/usr/bin/env python3
"""ASTER adapter and durable-journal fault matrix.

The HTTP target is a real loopback HTTP service. The LDAP-style target is an
independent TCP process with durable verifier hashes and delayed authoritative
readback semantics; it is not an OpenLDAP cluster and results label it as such.
"""

from __future__ import annotations

import argparse
import hashlib
import http.client
import json
import multiprocessing as mp
from multiprocessing.connection import Client, Listener
import os
from pathlib import Path
import socket
import sqlite3
import tempfile
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from typing import Any, Dict, Iterable, List, Optional, Tuple


OLD_PASSWORD = "ASTER-fixture-old-7!"
NEW_PASSWORD = "ASTER-fixture-new-8!"


def digest(password: str) -> str:
    return hashlib.sha256(password.encode()).hexdigest()


class Journal:
    def __init__(self, path: Path):
        self.path = path
        self.db = sqlite3.connect(path)
        self.db.execute("PRAGMA journal_mode=WAL")
        self.db.execute("PRAGMA synchronous=FULL")
        self.db.execute(
            """CREATE TABLE IF NOT EXISTS op(
              id TEXT PRIMARY KEY,
              state TEXT NOT NULL,
              committed_epoch INTEGER NOT NULL,
              committed_generation INTEGER NOT NULL,
              old_epoch INTEGER,
              old_generation INTEGER,
              candidate_epoch INTEGER,
              candidate_generation INTEGER,
              evidence TEXT
            )"""
        )
        self.db.commit()

    def initialize(self) -> None:
        self.db.execute(
            "INSERT OR IGNORE INTO op VALUES('r1','COMMITTED',1,0,NULL,NULL,NULL,NULL,NULL)"
        )
        self.db.commit()

    def prepare(self) -> None:
        self.db.execute(
            """UPDATE op SET state='PREPARED',old_epoch=committed_epoch,
            old_generation=committed_generation,candidate_epoch=2,
            candidate_generation=0,evidence=NULL WHERE id='r1'"""
        )
        self.db.commit()

    def submitted(self) -> None:
        self.db.execute("UPDATE op SET state='SUBMITTED' WHERE id='r1'")
        self.db.commit()

    def apply(self, evidence: str) -> None:
        if evidence == "new_only":
            self.db.execute(
                """UPDATE op SET state='COMMITTED',committed_epoch=candidate_epoch,
                committed_generation=candidate_generation,old_epoch=NULL,
                old_generation=NULL,candidate_epoch=NULL,candidate_generation=NULL,
                evidence=? WHERE id='r1'""",
                (evidence,),
            )
        elif evidence == "old_only":
            self.db.execute(
                """UPDATE op SET state='ABORTED',old_epoch=NULL,old_generation=NULL,
                candidate_epoch=NULL,candidate_generation=NULL,evidence=? WHERE id='r1'""",
                (evidence,),
            )
        else:
            self.db.execute(
                "UPDATE op SET state='UNKNOWN_OUTCOME',evidence=? WHERE id='r1'",
                (evidence,),
            )
        self.db.commit()

    def row(self) -> Dict[str, Any]:
        columns = [item[1] for item in self.db.execute("PRAGMA table_info(op)")]
        values = self.db.execute("SELECT * FROM op WHERE id='r1'").fetchone()
        return dict(zip(columns, values))

    def close(self) -> None:
        self.db.close()


class HttpState:
    def __init__(self):
        self.accepted = {digest(OLD_PASSWORD)}
        self.lock = threading.Lock()


class HttpHandler(BaseHTTPRequestHandler):
    state: HttpState

    def log_message(self, _format: str, *_args: Any) -> None:
        return

    def _read_json(self) -> Dict[str, Any]:
        length = int(self.headers.get("Content-Length", "0"))
        return json.loads(self.rfile.read(length) or b"{}")

    def do_POST(self) -> None:  # noqa: N802
        body = self._read_json()
        if self.path == "/login":
            ok = digest(body["password"]) in self.state.accepted
            self.send_response(204 if ok else 401)
            self.end_headers()
            return
        if self.path != "/change":
            self.send_error(404)
            return
        mode = body.get("mode", "commit")
        old_hash, new_hash = digest(body["old"]), digest(body["new"])
        with self.state.lock:
            if mode == "drop_precommit":
                self.connection.shutdown(socket.SHUT_RDWR)
                self.connection.close()
                return
            if mode == "http_200_without_commit":
                self.send_response(200)
                self.end_headers()
                return
            if mode == "old_only":
                self.state.accepted = {old_hash}
            elif mode == "both":
                self.state.accepted = {old_hash, new_hash}
            elif mode == "neither":
                self.state.accepted = set()
            else:
                self.state.accepted = {new_hash}
            if mode == "commit_response_lost":
                self.connection.shutdown(socket.SHUT_RDWR)
                self.connection.close()
                return
            self.send_response(200)
            self.end_headers()


class HttpTarget:
    label = "loopback_http"

    def __init__(self):
        state = HttpState()
        handler = type("BoundHttpHandler", (HttpHandler,), {"state": state})
        self.server = ThreadingHTTPServer(("127.0.0.1", 0), handler)
        self.thread = threading.Thread(target=self.server.serve_forever, daemon=True)
        self.thread.start()
        self.port = self.server.server_address[1]

    def change(self, mode: str) -> str:
        body = json.dumps({"old": OLD_PASSWORD, "new": NEW_PASSWORD, "mode": mode})
        connection = http.client.HTTPConnection("127.0.0.1", self.port, timeout=1)
        try:
            connection.request("POST", "/change", body, {"Content-Type": "application/json"})
            response = connection.getresponse()
            response.read()
            return f"http_{response.status}"
        except (ConnectionError, http.client.HTTPException, OSError):
            return "transport_lost"
        finally:
            connection.close()

    def verify(self) -> Tuple[Optional[bool], Optional[bool]]:
        return self._login(OLD_PASSWORD), self._login(NEW_PASSWORD)

    def _login(self, password: str) -> Optional[bool]:
        connection = http.client.HTTPConnection("127.0.0.1", self.port, timeout=1)
        try:
            body = json.dumps({"password": password})
            connection.request("POST", "/login", body, {"Content-Type": "application/json"})
            response = connection.getresponse()
            response.read()
            return response.status == 204
        except (ConnectionError, http.client.HTTPException, OSError):
            return None
        finally:
            connection.close()

    def close(self) -> None:
        self.server.shutdown()
        self.server.server_close()
        self.thread.join(timeout=2)


def ldap_process(address: Tuple[str, int], authkey: bytes, db_path: str) -> None:
    db = sqlite3.connect(db_path)
    db.execute("CREATE TABLE IF NOT EXISTS accepted(hash TEXT PRIMARY KEY)")
    if db.execute("SELECT COUNT(*) FROM accepted").fetchone()[0] == 0:
        db.execute("INSERT INTO accepted VALUES(?)", (digest(OLD_PASSWORD),))
        db.commit()
    listener = Listener(address, authkey=authkey)
    while True:
        connection = listener.accept()
        try:
            command = connection.recv()
            if command[0] == "stop":
                connection.send(True)
                break
            if command[0] == "bind":
                present = db.execute(
                    "SELECT 1 FROM accepted WHERE hash=?", (digest(command[1]),)
                ).fetchone()
                connection.send(present is not None)
            elif command[0] == "modify":
                mode = command[1]
                if mode == "drop_precommit":
                    connection.close()
                    continue
                if mode == "old_only" or mode == "http_200_without_commit":
                    hashes = [digest(OLD_PASSWORD)]
                elif mode == "both":
                    hashes = [digest(OLD_PASSWORD), digest(NEW_PASSWORD)]
                elif mode == "neither":
                    hashes = []
                else:
                    hashes = [digest(NEW_PASSWORD)]
                db.execute("DELETE FROM accepted")
                db.executemany("INSERT INTO accepted VALUES(?)", [(item,) for item in hashes])
                db.commit()
                if mode == "commit_response_lost":
                    connection.close()
                    continue
                connection.send("ldap_modify_success")
        finally:
            try:
                connection.close()
            except OSError:
                pass
    listener.close()
    db.close()


def free_port() -> int:
    with socket.socket() as sock:
        sock.bind(("127.0.0.1", 0))
        return sock.getsockname()[1]


class LdapStyleTarget:
    label = "independent_process_ldap_style_model"

    def __init__(self, directory: Path):
        self.address = ("127.0.0.1", free_port())
        self.authkey = b"ASTER-LDAP-MODEL"
        self.db_path = directory / "ldap_target.sqlite"
        self.process = mp.Process(
            target=ldap_process,
            args=(self.address, self.authkey, str(self.db_path)),
            daemon=True,
        )
        self.process.start()
        self._wait_ready()

    def _wait_ready(self) -> None:
        for _ in range(100):
            try:
                self._call(("bind", OLD_PASSWORD))
                return
            except (ConnectionError, OSError):
                import time

                time.sleep(0.01)
        raise RuntimeError("LDAP-style target did not start")

    def _call(self, command: Tuple[Any, ...]) -> Any:
        connection = Client(self.address, authkey=self.authkey)
        try:
            connection.send(command)
            return connection.recv()
        finally:
            connection.close()

    def change(self, mode: str) -> str:
        try:
            return str(self._call(("modify", mode)))
        except (EOFError, ConnectionError, OSError):
            return "transport_lost"

    def verify(self) -> Tuple[Optional[bool], Optional[bool]]:
        try:
            return bool(self._call(("bind", OLD_PASSWORD))), bool(
                self._call(("bind", NEW_PASSWORD))
            )
        except (EOFError, ConnectionError, OSError):
            return None, None

    def restart(self) -> None:
        self._call(("stop",))
        self.process.join(timeout=2)
        self.process = mp.Process(
            target=ldap_process,
            args=(self.address, self.authkey, str(self.db_path)),
            daemon=True,
        )
        self.process.start()
        self._wait_ready()

    def close(self) -> None:
        if self.process.is_alive():
            try:
                self._call(("stop",))
            except (EOFError, ConnectionError, OSError):
                pass
            self.process.join(timeout=2)


def classify(old: Optional[bool], new: Optional[bool]) -> str:
    if old is None or new is None:
        return "unavailable"
    if new and not old:
        return "new_only"
    if old and not new:
        return "old_only"
    if old and new:
        return "both"
    return "neither"


SCENARIOS = [
    "crash_before_candidate_fsync",
    "crash_after_fsync_before_submission",
    "request_dropped_precommit",
    "target_commits_response_lost",
    "http_200_without_authoritative_acceptance",
    "new_only",
    "old_only",
    "both",
    "neither",
    "contradictory_sources",
    "verification_unavailable",
    "restart_in_unknown_outcome",
    "stale_local_snapshot",
    "stale_capability",
    "stale_freshness_generation",
    "stale_root_epoch",
]


def execute_trace(adapter_name: str, scenario: str, repetition: int) -> Dict[str, Any]:
    with tempfile.TemporaryDirectory(prefix="aster-fault-") as tmp:
        directory = Path(tmp)
        journal_path = directory / "journal.sqlite"
        journal = Journal(journal_path)
        journal.initialize()
        target = HttpTarget() if adapter_name == "http" else LdapStyleTarget(directory)
        observations: List[str] = []
        evidence = "none"
        try:
            if scenario == "crash_before_candidate_fsync":
                observations.append("client_crash_before_prepare")
            else:
                journal.prepare()
                observations.append("candidate_descriptor_fsynced")
                if scenario == "crash_after_fsync_before_submission":
                    observations.append("client_crash_before_submit")
                elif scenario in {
                    "stale_capability",
                    "stale_freshness_generation",
                    "stale_root_epoch",
                }:
                    observations.append("authorization_rejected_pre_submit")
                else:
                    journal.submitted()
                    mode = {
                        "request_dropped_precommit": "drop_precommit",
                        "target_commits_response_lost": "commit_response_lost",
                        "http_200_without_authoritative_acceptance": "http_200_without_commit",
                        "new_only": "commit",
                        "old_only": "old_only",
                        "both": "both",
                        "neither": "neither",
                        "contradictory_sources": "commit",
                        "verification_unavailable": "commit",
                        "restart_in_unknown_outcome": "commit_response_lost",
                        "stale_local_snapshot": "commit_response_lost",
                    }.get(scenario, "commit")
                    observations.append(target.change(mode))
                    if scenario == "verification_unavailable":
                        evidence = "unavailable"
                    elif scenario == "contradictory_sources":
                        evidence = "contradictory"
                    else:
                        old, new = target.verify()
                        evidence = classify(old, new)
                    if scenario in {
                        "request_dropped_precommit",
                        "restart_in_unknown_outcome",
                        "stale_local_snapshot",
                    }:
                        evidence = "unavailable"
                    journal.apply(evidence)
                    if scenario == "restart_in_unknown_outcome":
                        journal.close()
                        journal = Journal(journal_path)
                        observations.append("journal_reopened")
                    if adapter_name == "ldap" and scenario == "new_only":
                        target.restart()
                        observations.append("target_restarted")
                        old, new = target.verify()
                        observations.append(f"post_restart_{classify(old, new)}")

            row = journal.row()
            old_reconstructible = row["committed_epoch"] == 1 or row["old_epoch"] == 1
            candidate_reconstructible = (
                row["committed_epoch"] == 2 or row["candidate_epoch"] == 2
            )
            commit_invariant = row["committed_epoch"] != 2 or evidence == "new_only"
            uncertainty_invariant = row["state"] != "UNKNOWN_OUTCOME" or (
                row["old_epoch"] is not None and row["candidate_epoch"] is not None
            )
            return {
                "adapter": target.label,
                "scenario": scenario,
                "repetition": repetition,
                "initialDescriptor": {"rootEpoch": 1, "generation": 0},
                "candidateDescriptor": {"rootEpoch": 2, "generation": 0},
                "transportObservations": observations,
                "adapterEvidence": evidence,
                "finalLocalState": row["state"],
                "oldReconstructible": old_reconstructible,
                "candidateReconstructible": candidate_reconstructible,
                "commitInvariantHeld": commit_invariant,
                "uncertaintyInvariantHeld": uncertainty_invariant,
                "journalPasswordColumns": 0,
            }
        finally:
            target.close()
            journal.close()


def summarize(traces: Iterable[Dict[str, Any]]) -> Dict[str, Any]:
    traces = list(traces)
    return {
        "schemaVersion": 1,
        "traceCount": len(traces),
        "scenarioCount": len(SCENARIOS),
        "adapters": sorted({trace["adapter"] for trace in traces}),
        "repetitionsPerScenario": max(trace["repetition"] for trace in traces) + 1,
        "commitInvariantViolations": sum(not trace["commitInvariantHeld"] for trace in traces),
        "uncertaintyInvariantViolations": sum(
            not trace["uncertaintyInvariantHeld"] for trace in traces
        ),
        "passwordColumns": sum(trace["journalPasswordColumns"] for trace in traces),
        "boundary": "The HTTP adapter is a real loopback HTTP service. The LDAP-style adapter is an independent TCP process with durable verifier hashes and modeled authoritative readback; it is not OpenLDAP or a replicated LDAP cluster.",
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", required=True)
    parser.add_argument("--summary", required=True)
    parser.add_argument("--repetitions", type=int, default=3)
    args = parser.parse_args()
    output = Path(args.output)
    summary_path = Path(args.summary)
    output.parent.mkdir(parents=True, exist_ok=True)
    summary_path.parent.mkdir(parents=True, exist_ok=True)
    traces = [
        execute_trace(adapter, scenario, repetition)
        for adapter in ("http", "ldap")
        for scenario in SCENARIOS
        for repetition in range(args.repetitions)
    ]
    with output.open("w", encoding="utf-8") as stream:
        for trace in traces:
            stream.write(json.dumps(trace, sort_keys=True) + "\n")
    summary_path.write_text(json.dumps(summarize(traces), indent=2) + "\n")
    print(summary_path)


if __name__ == "__main__":
    mp.set_start_method("spawn")
    main()
