# Context-scoped credential exposure: prior-art audit

Audit date: 2026-08-09  
Scope: authorization scope, capability attenuation, context-bound tokens,
constrained PRFs, distributed derivation, attack-graph reachability, dynamic
access structures, and password-manager compromise.

## Closest published work

| Work | What it already establishes | Why the candidate must differ |
|---|---|---|
| Boneh and Waters, constrained PRFs, ASIACRYPT 2013 | A constrained key evaluates a PRF on an allowed subset and nowhere else. | The candidate does not invent cryptographic input constraints. It analyzes whether deployment capabilities and honest protocol calls enforce the intended subset. [Author page](https://crypto.stanford.edu/~dabo/pubs/abstracts/dumbledore.html) |
| Macaroons, NDSS 2014 | Authorization credentials can be attenuated with contextual caveats that restrict how, by whom, and in what context authority is used. | Full-context ticket binding is prior art. The candidate measures cross-credential exposure when caveats/bindings are incomplete and composes this with threshold recovery. [NDSS](https://www.ndss-symposium.org/ndss2014/ndss-2014-programme/macaroons-cookies-contextual-caveats-decentralized-authorization-cloud/) |
| Stateful Least Privilege Authorization, USENIX Security 2024 | Stateful, attenuated OAuth-like bearer tokens can enforce minimal access instead of broad reusable scope. | Least-privilege token construction is not novel. The candidate's object is a set-valued credential exposure map and dual factor/scope-collapse analysis. [USENIX](https://www.usenix.org/conference/usenixsecurity24/presentation/cao-leo) |
| Pythia, USENIX Security 2015 | A PRF service can support granular rate limiting, tweaks, and key rotation for password applications. | Rate-limited password PRF service design is established. The candidate analyzes threshold/local-token invocation authority and exact EPSCD context binding. [USENIX](https://www.usenix.org/system/files/conference/usenixsecurity15/sec15-paper-everspaugh.pdf) |
| LaKey, USENIX Security 2024 | Distributed key derivation evaluates a DPRF from a secret-shared master without learning that master in the clear. | Never-reconstructed derivation is prior art. The candidate asks which identities/contexts a compromised caller can cause the service to evaluate. [USENIX](https://www.usenix.org/conference/usenixsecurity24/presentation/geihs) |
| MulVAL, USENIX Security 2005; attack graphs | Rule-based analysis derives multistage attacker consequences and witnesses. | Fixed-point reachability and attack witnesses are prior art. The candidate specializes outcomes into master reachability plus an authorization-indexed credential set and non-amplification metric. [MulVAL](https://www.usenix.org/conference/14th-usenix-security-symposium/mulval-logic-based-network-security-analyzer) |
| PolyScope, USENIX Security 2021 | Combined Android access-control policies are analyzed for permission expansion and concrete attack operations. | Multi-policy expansion analysis is close prior art. The candidate cannot claim the general idea of computing expanded authorized consequences; it must distinguish the threshold-derivation setting, the master/output independence result, and the exact approval-budget exposure profile. [USENIX](https://www.usenix.org/conference/usenixsecurity21/presentation/lee-yu-tsung) |
| Minimum-cost attack-graph analysis | Attack graphs have long been used to find minimum-cost paths and hardening choices for critical assets. | Minimum compromise cost is not new. The candidate's narrower object is the complete curve from unauthorized credential-output cardinality and approval budget to compromise-domain cost. [Computer Communications](https://doi.org/10.1016/j.comcom.2006.06.018) |
| Paralysis Proofs, AFT 2019 | Dynamic access-structure policies, migrations, and an effective access structure are formalized for custody. | “Effective access structure” and dynamic access policies cannot be claimed as new. The candidate keeps the access structure fixed and analyzes protocol-induced master and per-context derivation exposure. [ACM AFT](https://doi.org/10.1145/3318041.3355459) |
| SPHINX, ICDCS 2017 / IEEE TDSC | A device/server password-store design can limit consequences under specified compromise cases. | Password-manager compartmentalization is established motivation. The candidate contributes a finite analysis method rather than a new device/server password protocol. [IBM](https://research.ibm.com/publications/sphinx-a-password-store-that-perfectly-hides-passwords-from-itself) |

Context-sensitive RBAC, formal OAuth analysis, replay protection, capability
systems, and transaction binding further prevent broad priority claims. They
are supporting foundations, not manuscript baselines.

## Feature comparison

| Published line | Models master/threshold access | Models callable honest-service consequences | Context-scoped authorization | Computes set of exposed credential contexts | Tests approval non-amplification | Synthesizes binding fields |
|---|---:|---:|---:|---:|---:|---:|
| Constrained PRF | N/S | N | Y, cryptographic subset | N | Security game, different object | Constraint construction |
| Macaroons | N | S | Y | N | Authorization semantics | Caveat attenuation |
| Stateful least privilege | N | Y | Y | Resource permissions | Least-privilege enforcement | Policy program |
| Pythia | Server PRF key | Y | Tweak/rate scope | N | Rate-limit goal | N |
| LaKey | Y | MPC protocol | Identity input | Derived key for requested identity | Application-defined | N |
| MulVAL / attack graphs | N | Y | Generic facts/rules | Can encode separately | N | Generic hardening work exists |
| PolyScope | N | Y | Android resource permissions | Authorized attack operations | Permission expansion | Policy-specific analysis |
| Dynamic access structures | Y | Stateful migration | Access input | N | Safety/liveness | Policy migration |
| Candidate method | Y | Y | Y | Y | Y | Finite projection repair |

## Novelty verdict

The audit rejects all of the following claims:

- first context-bound token;
- first least-privilege authorization;
- first constrained or threshold PRF;
- first effective or dynamic access structure;
- first attack-graph/capability-closure analyzer; or
- first password manager with compromise compartmentalization.

It tentatively supports this narrower method claim:

> jointly analyze master-capability reachability and the set of credential
> contexts derivable from approved contexts, so that a deployment can pass a
> Root-Key threshold check yet fail an authorization non-amplification check.

The strongest distinguishing counterexample is an unscoped DPRF interface:
the master never materializes, but one compromised endpoint can derive every
credential context. Existing primitive-level DPRF security does not reject
that interface because all evaluations are legitimate oracle calls.

## Reviewer risk

Risk remains moderate to high. A reviewer can reasonably describe the method as
an application-specific product of attack-graph reachability, PolyScope-style
permission-expansion analysis, and least-privilege authorization. To support
publication, the paper must show that the dual output and exact exposure profile
find security failures missed by a Root-only access analysis, use only published
works as comparisons, and avoid claiming that the underlying mathematical tools
are new.
