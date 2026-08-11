# Distributed-PRF and never-reconstructed-root prior-art audit

Audit date: 2026-08-09  
Status: research-branch document; not yet manuscript text  
Publication rule: only peer-reviewed papers are eligible manuscript comparison
targets. Standards and preprints are listed separately and cannot be used as
published experimental baselines.

## 1. Candidate operational property

The proposed routine path is:

```text
partial_D(c) + partial_U(c)
    -> F_K(c)
    -> K_cred,c
    -> existing EPSCD
```

The master PRF key `K` is shared and is never materialized during routine
credential derivation. A hardware token, smart card, secure element, or
separate trusted device must keep the raw `U` share inside the `U` domain. An
ordinary USB file does not meet this property: once the raw share enters a
compromised endpoint that already has the other share, the endpoint can recover
the master.

This profile must use a published, reviewed threshold/distributed PRF or a
general malicious-secure MPC framework. It must not use a locally invented
exponentiation protocol.

## 2. Closest published cryptographic work

### Distributed and threshold PRFs

Naor, Pinkas, and Reingold introduced distributed pseudorandom functions in
EUROCRYPT 1999. Nielsen subsequently gave a practical threshold PRF construction
and a multiparty evaluation protocol at CRYPTO 2002. These papers directly
preclude any claim that threshold parties jointly evaluate a PRF without one
party learning the key is new. [Naor--Pinkas--Reingold, EUROCRYPT
1999](https://dblp.org/rec/conf/eurocrypt/NaorPR99.html),
[Nielsen, CRYPTO 2002](https://www.iacr.org/archive/crypto2002/24420403/24420403.pdf)

### Distributed key derivation

LaKey is the closest published work. It defines distributed key derivation from
a constant-size secret-shared master key and an identity, evaluates a DPRF
without learning the master key in the clear, keeps the derived output
secret-shared for subsequent MPC, supports master-share refresh requirements,
provides a UC treatment, and implements the construction in MP-SPDZ. This
substantially covers the proposed “never-reconstructed master plus
context-derived key” cryptographic architecture. The remaining difference is
the application to a local endpoint/token credential workflow and EPSCD—not a
new DPRF or distributed-key-derivation primitive. [LaKey, USENIX Security
2024](https://www.usenix.org/conference/usenixsecurity24/presentation/geihs)

### Password and recovery systems using PRF services

Pythia builds a remote partially oblivious PRF service with rate limiting and
key rotation for password-hardening applications. TOPPSS combines
password-protected secret sharing with a threshold OPRF. SPHINX uses a
device/server password-store architecture with compromise-specific guarantees.
These works make password-related remote or threshold PRF integration prior
art. [Pythia, USENIX Security
2015](https://www.usenix.org/system/files/conference/usenixsecurity15/sec15-paper-everspaugh.pdf),
[TOPPSS, ACNS 2017](https://www.research.ed.ac.uk/en/publications/toppss-cost-minimal-password-protected-secret-sharing-based-on-th/),
[SPHINX, IEEE TDSC 2019](https://research.ibm.com/publications/building-and-studying-a-password-store-that-perfectly-hides-passwords-from-itself)

MFKDF provides threshold multi-factor key derivation and factor-loss recovery;
SafetyPin and SVR3 provide distributed recovery with strong deployment and
lifecycle mechanisms. Flock demonstrates that genuinely distinct trust domains
are a deployment problem of their own. These systems constrain any claim about
factor distribution, recovery, or hardware-backed trust, even though they do
not implement the exact EPSCD path. [MFKDF, USENIX Security
2023](https://www.usenix.org/conference/usenixsecurity23/presentation/nair-mfkdf),
[SafetyPin, OSDI 2020](https://www.usenix.org/conference/osdi20/presentation/dauterman-safetypin),
[SVR3, OSDI 2024](https://www.usenix.org/conference/osdi24/presentation/connell),
[Flock, OSDI 2024](https://www.usenix.org/conference/osdi24/presentation/kaviani)

### Refresh and repair

Proactive sharing and verifiable share repair are established research areas.
Basu et al. provide verifiable secret sharing with share recovery for BFT
protocols. Recovery/re-provisioning without exposing the master could be useful
engineering, but it cannot be claimed as the invention of share refresh or
secure repair. [Basu et al., ACM CCS
2019](https://doi.org/10.1145/3319535.3354207)

## 3. Claim matrix

| Published work | Nominal access structure | Protocol-call capability closure | Effective threshold metric/analyzer | Master clear in routine use | Context-scoped derived result | Automatic context authorization | Repair/refresh |
|---|---:|---:|---:|---:|---:|---:|---:|
| Naor--Pinkas--Reingold | Y | N | N | No by construction | Y | Application-defined | N/S |
| Nielsen threshold PRF | Y | N | N | No by construction | Y | Application-defined | N |
| LaKey | Y | N | N | No | Y, secret-shared | Application-defined | Share refresh requirement |
| Pythia | Server-held PRF key | N | N | Server retains key | Y | Rate-limited service policy | Key rotation |
| TOPPSS | Threshold servers | N | N | Threshold protocol | Password-scoped | Password protocol | N/S |
| MFKDF | Threshold factor policy | N | N | Client obtains derived key | Policy/context-derived | Factor policy | Factor replacement |
| SPHINX | Device/server split | Protocol-specific | N | Architecture-specific | Service password | Protocol-specific | Service state |
| SafetyPin | Threshold HSM cluster | Protocol-specific | N | Recovery output exposed as designed | Recovery-scoped | PIN/location-hiding protocol | Puncturing |
| SVR3 | Heterogeneous enclave threshold | Protocol-specific | N | Recovery output exposed as designed | Account/recovery-scoped | Enclave protocol | Fault/rollback lifecycle |
| Flock | Application-dependent | Deployment orchestration | N | Application-dependent | Application-dependent | Application-defined | Deployment lifecycle |

The table shows that the DPRF half of the candidate is established. Its only
plausible paper role is a system-security profile whose authorization behavior
is analyzed by CCAS and whose output is consumed by EPSCD.

## 4. Critical authorization limitation

PRF security alone does not give credential compromise locality against a
persistently compromised endpoint. If token `U` automatically evaluates every
context supplied by `D`, malware can request:

```text
c_1, c_2, ..., c_N
```

and collect every corresponding derived output without ever recovering the
master key. The system has prevented master extraction but not whole-vault
online enumeration.

The desired locality property therefore needs both:

1. a DPRF theorem: observed outputs do not reveal the master or enable local
   evaluation on a fresh input; and
2. an authorization theorem: the compromised endpoint cannot cause the token
   or another honest party to evaluate an unauthorized context.

The second property requires a capability outside `D`, such as token-local user
confirmation over an authenticated human-readable context, a pre-provisioned
allow-list, an independent approval domain, or enforceable rate/usage policy.
Merely moving a raw share into token firmware is insufficient. CCAS must include
`CanInvokeEval(c)` and every approval/session capability that can produce it.

## 5. Security statement that would be supportable

For a selected published DPRF with its stated adversary and corruption model,
and for contexts authorized by an independent token-domain policy:

> exposure of the client-visible result for authorized context `c` does not by
> itself reveal the shared master PRF key or enable offline evaluation at a
> fresh context `c*`; the attacker may still learn the derived password and key
> for `c`, and may obtain further outputs to the extent allowed by the token's
> invocation policy.

This is conditional, computational, and construction-specific. It is not an
unconditional new theorem. The manuscript would need to instantiate the exact
LaKey/Nielsen/other published security model, corruption threshold, malicious
behavior handling, verification, replay binding, and share-refresh semantics.

## 6. Standards and non-published constraints

RFC 9497 standardizes two-party OPRF, VOPRF, and POPRF, not a general threshold
DPRF. It is useful implementation context but not a peer-reviewed experimental
baseline. The 2018 threshold partially oblivious PRF report is an IACR ePrint
preprint and must remain labelled as such unless a peer-reviewed version is
verified. [RFC 9497](https://www.rfc-editor.org/rfc/rfc9497), [Jarecki et al.
ePrint 2018/733](https://eprint.iacr.org/2018/733)

## 7. Prototype decision

No toy DPRF has been added. LaKey's published artifact uses MP-SPDZ and an
honest-majority, malicious-secure MPC profile; reimplementing only its visible
algebra would discard the proof-carrying protocol context and create exactly the
kind of unreviewed cryptography this project forbids. A credible implementation
would need to reuse and pin that artifact or another reviewed implementation,
then add a genuinely isolated token/approver interface.

Until that integration and hardware/software-isolation experiment exists,
“never-reconstructed root” is not ready to replace the current recovery
contribution. The correct present classification is a security-motivated system
extension with high implementation cost and heavy prior-art overlap.

