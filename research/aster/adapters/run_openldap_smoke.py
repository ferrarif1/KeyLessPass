#!/usr/bin/env python3
"""Run a real OpenLDAP password-change/bind smoke test in a pinned container."""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import secrets
import socket
import subprocess
import time


IMAGE = "osixia/openldap@sha256:18742e9c449c9c1afe129d3f2f3ee15fb34cc43e5f940a20f3399728f41d7c28"
BASE_DN = "dc=aster,dc=test"
ADMIN_DN = f"cn=admin,{BASE_DN}"
USER_DN = f"uid=aster-user,ou=people,{BASE_DN}"


def run(command: list[str], *, input_text: str | None = None) -> subprocess.CompletedProcess[str]:
    return subprocess.run(command, input=input_text, text=True, capture_output=True)


def free_port() -> int:
    with socket.socket() as sock:
        sock.bind(("127.0.0.1", 0))
        return int(sock.getsockname()[1])


def bind(uri: str, dn: str, password: str) -> bool:
    return run(["ldapwhoami", "-x", "-H", uri, "-D", dn, "-w", password]).returncode == 0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()

    port = free_port()
    uri = f"ldap://127.0.0.1:{port}"
    name = f"aster-openldap-{os.getpid()}"
    admin_password = secrets.token_urlsafe(24)
    old_password = secrets.token_urlsafe(24)
    new_password = secrets.token_urlsafe(24)
    started = False
    result: dict[str, object]

    try:
        launch = run([
            "docker", "run", "-d", "--rm", "--name", name,
            "-p", f"127.0.0.1:{port}:389",
            "-e", "LDAP_ORGANISATION=ASTER Research",
            "-e", "LDAP_DOMAIN=aster.test",
            "-e", f"LDAP_ADMIN_PASSWORD={admin_password}",
            IMAGE,
        ])
        if launch.returncode != 0:
            raise RuntimeError(f"OpenLDAP container launch failed: {launch.stderr.strip()}")
        started = True

        deadline = time.monotonic() + 60
        while time.monotonic() < deadline and not bind(uri, ADMIN_DN, admin_password):
            time.sleep(0.25)
        if not bind(uri, ADMIN_DN, admin_password):
            raise RuntimeError("OpenLDAP did not become ready within 60 seconds")

        add_ldif = f"""dn: ou=people,{BASE_DN}
objectClass: organizationalUnit
ou: people

dn: {USER_DN}
objectClass: inetOrgPerson
cn: ASTER User
sn: User
uid: aster-user
userPassword: {old_password}

"""
        added = run([
            "ldapadd", "-x", "-H", uri, "-D", ADMIN_DN, "-w", admin_password,
        ], input_text=add_ldif)
        if added.returncode != 0:
            raise RuntimeError(f"LDAP fixture creation failed: {added.stderr.strip()}")

        old_before = bind(uri, USER_DN, old_password)
        new_before = bind(uri, USER_DN, new_password)
        modify_ldif = f"""dn: {USER_DN}
changetype: modify
replace: userPassword
userPassword: {new_password}

"""
        start_ns = time.perf_counter_ns()
        modified = run([
            "ldapmodify", "-x", "-H", uri, "-D", ADMIN_DN, "-w", admin_password,
        ], input_text=modify_ldif)
        modify_ms = (time.perf_counter_ns() - start_ns) / 1_000_000
        new_after = bind(uri, USER_DN, new_password)
        old_after = bind(uri, USER_DN, old_password)
        root_dse = run([
            "ldapsearch", "-LLL", "-x", "-H", uri, "-s", "base", "-b", "", "vendorName", "vendorVersion",
        ])

        passed = (
            old_before
            and not new_before
            and modified.returncode == 0
            and new_after
            and not old_after
        )
        result = {
            "schemaVersion": 1,
            "image": IMAGE,
            "topology": "single OpenLDAP server in a loopback Docker container",
            "oldCredentialAcceptedBeforeModify": old_before,
            "candidateAcceptedBeforeModify": new_before,
            "modifyReturnCode": modified.returncode,
            "modifyMillis": modify_ms,
            "candidateAcceptedAfterModify": new_after,
            "oldCredentialAcceptedAfterModify": old_after,
            "rootDse": root_dse.stdout.strip().splitlines(),
            "pass": passed,
            "boundary": "Real OpenLDAP modify and bind verification; not a replicated cluster, delayed-replication experiment, or production adapter benchmark.",
        }
    except Exception as error:
        result = {
            "schemaVersion": 1,
            "image": IMAGE,
            "pass": False,
            "errorType": type(error).__name__,
            "boundary": "OpenLDAP smoke test did not complete; no successful integration claim is permitted.",
        }
    finally:
        if started:
            run(["docker", "stop", "-t", "2", name])

    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n")
    print(args.output)
    print(json.dumps({"pass": result["pass"], "image": IMAGE}, indent=2))
    return 0 if result["pass"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
