# Capability-Closed Effective Access Structures: prior-art and claim audit

Audit date: 2026-08-09  
Status: research-branch document; not yet manuscript text  
Publication rule: manuscript comparisons and experimental baselines must be
peer-reviewed publications. Specifications and preprints may constrain claims
but are identified separately.

## 1. Candidate object

Let `D = {D1,...,Dn}` be deployment compromise domains and `C` a finite set of
capabilities. A capability can denote a share, signing key, token, authenticated
session, approval authority, invocation right, or a result automatically
released by an honest component. Each domain has initial capabilities `C0(Di)`;
`Cpub` contains public facts.

An automatically exercisable rule is a monotone implication `(A -> B)`, where
`A,B subseteq C`. For compromised domain set `X`, define:

```text
T_X(S) = Cpub union (union over D in X of C0(D))
         union (union of B for every automatic rule A -> B with A subseteq S)

Cl_R(X) = lfp(T_X)
```

For a nominal cryptographic access structure `Gamma_nom` over share names,
define the deployment-domain access structure:

```text
Gamma_eff = { X subseteq D |
              exists Q in Gamma_nom : Q subseteq (Cl_R(X) intersect Shares) }
```

Two thresholds must not be conflated:

```text
tau_share = min {|Q| : Q in Gamma_nom}
tau_eff   = min {|X| : X in Gamma_eff}
```

They count different objects. The defensible collapse predicate is therefore
`tau_eff < k_deploy`, where `k_deploy` is the explicitly configured minimum
number of independent compromise domains. Comparing `tau_share` with `tau_eff`
is valid only when the deployment policy intentionally maps one independent
domain to each nominal factor. A weighted measure is:

```text
rho_eff = min { sum(cost(D)) : X in Gamma_eff and D in X }.
```

## 2. Basic results (not novelty claims)

For a finite capability universe and monotone rules:

1. `T_X` is monotone, so repeated application from the initial facts reaches a
   unique least fixed point after at most `|C|` new facts.
2. `X subseteq Y` implies `Cl_R(X) subseteq Cl_R(Y)`.
3. If `Gamma_nom` is monotone, `Gamma_eff` is monotone.
4. Enumerating every `X subseteq D`, computing `Cl_R(X)`, and testing nominal
   qualification is sound and complete for the finite input model.
5. Inclusion-minimal members of `Gamma_eff` are precisely the minimal
   deployment compromise sets for that model.

These are direct fixed-point and finite-enumeration consequences. They should
be lemmas supporting the tool, not advertised as new mathematics.

The reference analyzer uses exhaustive enumeration. With a straightforward
work-list closure, its intended small-model cost is exponential in the number
of deployment domains and polynomial in the explicit capability/rule model.
No NP-hardness, optimal-repair, SAT, SMT, or synthesis claim is made.

## 3. Closest published work

### 3.1 Cryptographic access structures

Ito, Saito, and Nishizeki defined general secret-sharing access structures and
showed how to realize any monotone qualified-set family. Benaloh and Leichter
related generalized sharing to monotone functions. This is direct prior art for
`Gamma_nom`; CCAS does not redefine or generalize cryptographic access
structures. [Ito et al., GLOBECOM 1987](https://archiv.infsec.ethz.ch/education/as09/secsem/papers/ItSaNi87.pdf),
[Benaloh and Leichter, CRYPTO 1988](https://doi.org/10.1007/0-387-34799-2_3)

### 3.2 Attack graphs and rule-based privilege derivation

Sheyner et al. generate attack graphs with symbolic model checking and analyze
paths reaching an adversarial goal. Jha, Sheyner, and Wing additionally study
minimum critical attack sets and defensive choices. MulVAL represents facts,
configuration, permissions, vulnerabilities, and interaction rules in Datalog
and automatically derives multihost, multistage consequences. These works are
strong prior art for fixed-point privilege derivation, automatic attack-path
generation, minimal witnesses, and remediation analysis. CCAS therefore cannot
claim to invent capability closure, Horn-rule security analysis, attack
witnesses, or minimum compromise-set enumeration. [Sheyner et al., IEEE S&P
2002](https://www.cs.cmu.edu/afs/cs/project/svc/www/papers/view-publications-sjwattack2002.html),
[Jha et al., CSFW 2002](https://www.cs.cmu.edu/~wing/publications/Jha-Wing02.pdf),
[MulVAL, USENIX Security 2005](https://www.usenix.org/conference/14th-usenix-security-symposium/mulval-logic-based-network-security-analyzer)

### 3.3 Capabilities and callable authority

Hardy's confused-deputy analysis establishes why authority includes what a
principal can cause another component to do, rather than only data directly
stored by the principal. CCAS applies this established insight when a token or
authenticated session makes an honest recovery service release another share.
[Hardy, ACM SIGOPS 1988](https://doi.org/10.1145/54289.871709)

### 3.4 Deployment trust domains

Flock explicitly identifies the difficulty of creating genuinely distinct
trust domains and evaluates on-demand distributed-trust deployment across cloud
providers. SafetyPin distributes recovery trust across an HSM cluster, while
SVR3 uses heterogeneous enclave types and cloud providers with rollback
protection. These systems prevent any claim that heterogeneous domains,
distributed recovery, or independent administration are new. [Flock, OSDI
2024](https://www.usenix.org/conference/osdi24/presentation/kaviani),
[SafetyPin, OSDI 2020](https://www.usenix.org/conference/osdi20/presentation/dauterman-safetypin),
[SVR3, OSDI 2024](https://www.usenix.org/conference/osdi24/presentation/connell)

## 4. Question-by-question audit

Legend: `Y` means the work substantially supplies the feature; `S` means a
related but differently scoped treatment; `N` means it is not a stated object
of the work.

| Published work | Nominal crypto access structure | Derives compromise consequences from protocol/privilege rules | Honest service can extend attacker authority | Effective threshold degradation | Automatic detection | Repair/synthesis | Routine master materialization | Context-scoped result |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| General access structures (Ito et al.; Benaloh--Leichter) | Y | N | N | N | N | N | Depends on scheme | N |
| Attack graphs (Sheyner et al.) | N | Y | Y | S | Y | S | N/A | N/A |
| Formal attack-graph analysis (Jha et al.) | N | Y | Y | S | Y | Y | N/A | N/A |
| MulVAL | N | Y | Y | S | Y | S | N/A | N/A |
| Confused deputy | N | Conceptual | Y | N | N | N | N/A | N/A |
| Flock | S | N | S | S | N | Deployment mechanism | Application-dependent | Application-dependent |
| SafetyPin | Threshold deployment | Protocol-specific | Y | Security model | N | Puncturing/recovery lifecycle | Application-specific | Recovery-scoped |
| SVR3 | Threshold deployment | Protocol-specific | Y | Security model | N | Fault/rollback handling | Enclave protocol-specific | Request-scoped |

No audited publication was found that uses the exact composition

```text
protocol-capability least fixed point
  -> shares reachable from compromised deployment domains
  -> nominal recovery access structure
  -> effective deployment access structure and collapse witness
```

as its central credential-root analysis. This is a scoped distinction, not a
priority proof. Attack-graph reviewers can reasonably view it as an application
of established rule-based analysis unless the paper demonstrates that the
mapping exposes deployment failures that existing cryptographic access
structures and conventional component-level threat models miss.

## 5. Implemented evidence

The research prototype in
`research_upgrade/ccas_dprf/ccas_analyzer.py` computes:

- `Cl_R(X)` for every deployment-domain subset;
- `Gamma_eff`;
- configured and effective deployment thresholds;
- weighted minimum compromise cost;
- inclusion-minimal compromising sets; and
- a rule-by-rule witness for each minimal set.

The six required models produce:

| Case | Configured domain threshold | Effective domain threshold | Result |
|---|---:|---:|---|
| Endpoint can automatically call network recovery | 2 | 1 | collapse through `D` |
| Endpoint request plus independent approval | 2 | 2 | preserved |
| Approval capability copied into endpoint | 2 | 1 | collapse through `D` |
| Removable medium carries network-release credential | 2 | 1 | collapse through `U` |
| One endpoint token calls all nodes in a 3-of-5 service | 2 | 1 | collapse through `D` |
| One administrative domain controls D, A, and three nodes | 2 | 1 | collapse through `Admin` |

The result demonstrates implementation value, not cryptographic novelty. In
particular, multiple machines are not multiple compromise domains when one
credential or administrator controls all of them.

## 6. Defensible claim boundary

The strongest currently supportable candidate contribution is:

> a credential-root-specific mapping from automatically exercisable deployment
> capabilities to a nominal recovery access structure, together with a small
> analyzer that reports the resulting effective domain structure and concrete
> threshold-collapse witnesses.

Do not claim a new access-structure theory, a new fixed-point method, the first
automatic security analyzer, or optimal repair. The publishability of this
contribution depends on a larger real-deployment corpus and evidence that CCAS
finds mistakes not made visible by a conventional attack-graph encoding.

