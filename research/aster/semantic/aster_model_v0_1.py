"""ASTER executable semantic model.

This module validates protocol/state invariants.  It is NOT a production
threshold-cryptography implementation.

The local HMAC/Feistel service stands in for an ideal threshold/MPC exact-domain
permutation service.  A production implementation must replace it without
changing the externally visible authorization and lifecycle semantics.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from enum import Enum
import hashlib
import hmac
import json
import math
import secrets
import time
from typing import Dict, List, Optional, Tuple


def _canon(obj) -> bytes:
    return json.dumps(obj, sort_keys=True, separators=(",", ":")).encode()


def _hmac(key: bytes, *parts: bytes) -> bytes:
    m = hmac.new(key, digestmod=hashlib.sha256)
    for p in parts:
        m.update(len(p).to_bytes(4, "big"))
        m.update(p)
    return m.digest()


@dataclass(frozen=True)
class CredentialContext:
    vault_id: str
    service_id: str
    account_id: str
    lineage_id: str
    credential_salt: str
    policy_id: str
    policy_hash: str
    policy_epoch: int

    def canonical(self) -> bytes:
        return _canon(self.__dict__)


class FixedAlphabetPolicy:
    """Small exact policy used only by the reference model.

    Production ASTER should call the existing EPSCD compiler / Rank / Unrank.
    """

    def __init__(self, alphabet: str, length: int):
        if len(set(alphabet)) != len(alphabet):
            raise ValueError("alphabet must contain unique symbols")
        if len(alphabet) < 2 or length < 1:
            raise ValueError("invalid policy")
        self.alphabet = alphabet
        self.length = length
        self.base = len(alphabet)
        self.N = self.base ** self.length

    def unrank(self, rank: int) -> str:
        if not (0 <= rank < self.N):
            raise ValueError("rank outside accepted domain")
        out = [self.alphabet[0]] * self.length
        x = rank
        for i in range(self.length - 1, -1, -1):
            out[i] = self.alphabet[x % self.base]
            x //= self.base
        return "".join(out)

    def rank(self, word: str) -> int:
        if len(word) != self.length:
            raise ValueError("word outside policy")
        idx = {c: i for i, c in enumerate(self.alphabet)}
        r = 0
        for ch in word:
            if ch not in idx:
                raise ValueError("word outside policy")
            r = r * self.base + idx[ch]
        return r


@dataclass(frozen=True)
class Capability:
    request_digest: str
    root_epoch: int
    generation: int
    operation: str
    freshness_generation: int
    expires_at: int
    nonce: str
    use_budget: int
    signature: str


class ApprovalAuthority:
    """Semantic capability signer.

    A production deployment should use an independently controlled signing key,
    HSM/service identity, policy engine, and durable issuance/audit state.
    """

    def __init__(self, key: Optional[bytes] = None):
        self._key = key or secrets.token_bytes(32)

    @staticmethod
    def request_digest(
        context: CredentialContext,
        root_epoch: int,
        generation: int,
        operation: str,
        freshness_generation: int,
    ) -> str:
        return hashlib.sha256(
            _canon(
                {
                    "protocol": "ASTER-v0.1",
                    "context": json.loads(context.canonical()),
                    "root_epoch": root_epoch,
                    "generation": generation,
                    "operation": operation,
                    "freshness_generation": freshness_generation,
                }
            )
        ).hexdigest()

    def issue(
        self,
        context: CredentialContext,
        root_epoch: int,
        generation: int,
        operation: str = "derive",
        freshness_generation: int = 0,
        ttl_seconds: int = 60,
        use_budget: int = 1,
        now: Optional[int] = None,
    ) -> Capability:
        now = int(time.time()) if now is None else now
        nonce = secrets.token_hex(16)
        digest = self.request_digest(
            context, root_epoch, generation, operation, freshness_generation
        )
        payload = {
            "request_digest": digest,
            "root_epoch": root_epoch,
            "generation": generation,
            "operation": operation,
            "freshness_generation": freshness_generation,
            "expires_at": now + ttl_seconds,
            "nonce": nonce,
            "use_budget": use_budget,
        }
        sig = hmac.new(self._key, _canon(payload), hashlib.sha256).hexdigest()
        return Capability(**payload, signature=sig)

    def verify(self, cap: Capability) -> bool:
        payload = {
            "request_digest": cap.request_digest,
            "root_epoch": cap.root_epoch,
            "generation": cap.generation,
            "operation": cap.operation,
            "freshness_generation": cap.freshness_generation,
            "expires_at": cap.expires_at,
            "nonce": cap.nonce,
            "use_budget": cap.use_budget,
        }
        expected = hmac.new(self._key, _canon(payload), hashlib.sha256).hexdigest()
        return hmac.compare_digest(expected, cap.signature)


class AuthorizationError(RuntimeError):
    pass


class EpochUnavailable(RuntimeError):
    pass


class DomainExhausted(RuntimeError):
    pass


class SemanticExactPermutationService:
    """Idealized service boundary for ASTER.

    The endpoint never receives `_epoch_keys` or per-context permutation keys.
    In the paper implementation these keys are secret-shared and permutation
    evaluation is performed by threshold/MPC evaluators.
    """

    def __init__(self, authority: ApprovalAuthority):
        self.authority = authority
        self._epoch_keys: Dict[int, bytes] = {}
        self._nonce_uses: Dict[str, int] = {}

    def install_epoch(self, epoch: int, key: Optional[bytes] = None) -> None:
        if epoch in self._epoch_keys:
            raise ValueError("epoch already installed")
        self._epoch_keys[epoch] = key or secrets.token_bytes(32)

    def retire_epoch(self, epoch: int) -> None:
        if epoch not in self._epoch_keys:
            raise EpochUnavailable(epoch)
        del self._epoch_keys[epoch]

    def has_epoch(self, epoch: int) -> bool:
        return epoch in self._epoch_keys

    @staticmethod
    def _bits_for_domain(N: int) -> int:
        if N < 2:
            raise ValueError("domain too small")
        bits = max(2, math.ceil(math.log2(N)))
        if bits % 2:
            bits += 1
        return bits

    @staticmethod
    def _feistel_once(x: int, bits: int, key: bytes, tweak: bytes) -> int:
        half = bits // 2
        mask = (1 << half) - 1
        L = (x >> half) & mask
        R = x & mask
        for rnd in range(10):
            f = int.from_bytes(
                _hmac(
                    key,
                    b"ASTER/feistel/v0.1",
                    tweak,
                    rnd.to_bytes(2, "big"),
                    R.to_bytes((half + 7) // 8, "big"),
                ),
                "big",
            ) & mask
            L, R = R, L ^ f
        return (L << half) | R

    def _rank(
        self,
        context: CredentialContext,
        root_epoch: int,
        generation: int,
        N: int,
    ) -> int:
        if root_epoch not in self._epoch_keys:
            raise EpochUnavailable(root_epoch)
        if not (0 <= generation < N):
            raise DomainExhausted("generation outside exact domain")

        epoch_key = self._epoch_keys[root_epoch]
        context_key = _hmac(
            epoch_key,
            b"ASTER/context-key/v0.1",
            context.canonical(),
            root_epoch.to_bytes(8, "big"),
        )
        tweak = hashlib.sha256(
            b"ASTER/exact-domain/v0.1"
            + context.canonical()
            + root_epoch.to_bytes(8, "big")
        ).digest()

        bits = self._bits_for_domain(N)
        x = generation
        # Cycle walking over a permutation induces a permutation on the subset
        # [0,N).  The cap prevents a pathological infinite loop in the model.
        for _ in range(1_000_000):
            x = self._feistel_once(x, bits, context_key, tweak)
            if x < N:
                return x
        raise RuntimeError("cycle-walk cap reached")

    def _validate_capability(
        self,
        context: CredentialContext,
        root_epoch: int,
        generation: int,
        operation: str,
        freshness_generation: int,
        cap: Capability,
        now: int,
    ) -> None:
        if not self.authority.verify(cap):
            raise AuthorizationError("invalid signature")
        expected = self.authority.request_digest(
            context, root_epoch, generation, operation, freshness_generation
        )
        if cap.request_digest != expected:
            raise AuthorizationError("scope mismatch")
        if cap.root_epoch != root_epoch or cap.generation != generation:
            raise AuthorizationError("epoch/generation mismatch")
        if cap.operation != operation:
            raise AuthorizationError("operation mismatch")
        if cap.freshness_generation != freshness_generation:
            raise AuthorizationError("freshness mismatch")
        if now > cap.expires_at:
            raise AuthorizationError("expired")
        used = self._nonce_uses.get(cap.nonce, 0)
        if used >= cap.use_budget:
            raise AuthorizationError("replay/use budget exceeded")

    def derive(
        self,
        context: CredentialContext,
        policy: FixedAlphabetPolicy,
        root_epoch: int,
        generation: int,
        cap: Capability,
        freshness_generation: int = 0,
        now: Optional[int] = None,
    ) -> str:
        now = int(time.time()) if now is None else now
        self._validate_capability(
            context,
            root_epoch,
            generation,
            "derive",
            freshness_generation,
            cap,
            now,
        )
        rank = self._rank(context, root_epoch, generation, policy.N)
        self._nonce_uses[cap.nonce] = self._nonce_uses.get(cap.nonce, 0) + 1
        return policy.unrank(rank)

    def internal_password(
        self,
        context: CredentialContext,
        policy: FixedAlphabetPolicy,
        root_epoch: int,
        generation: int,
    ) -> str:
        """Migration-only ideal functionality.

        In production this comparison/selection stays inside the threshold/MPC
        service and is authorized as a distinct migration operation.
        """
        return policy.unrank(
            self._rank(context, root_epoch, generation, policy.N)
        )

    def select_cross_epoch_candidate(
        self,
        context: CredentialContext,
        policy: FixedAlphabetPolicy,
        candidate_epoch: int,
        history: List[Tuple[int, int]],
        history_window: int,
        max_candidates: int = 1024,
    ) -> Tuple[int, str]:
        recent = history[-history_window:] if history_window > 0 else []
        excluded = {
            self.internal_password(context, policy, e, g) for e, g in recent
        }
        for g in range(min(max_candidates, policy.N)):
            pwd = self.internal_password(context, policy, candidate_epoch, g)
            if pwd not in excluded:
                return g, pwd
        raise DomainExhausted("no candidate outside authenticated history")


class Evidence(str, Enum):
    NEW_ONLY = "NewOnly"
    OLD_ONLY = "OldOnly"
    BOTH = "Both"
    NEITHER = "Neither"
    TIMEOUT = "Timeout"
    CONTRADICTORY = "Contradictory"


class PendingStatus(str, Enum):
    PREPARED = "Prepared"
    UNKNOWN = "UnknownOutcome"


@dataclass
class PendingMigration:
    old_epoch: int
    old_generation: int
    candidate_epoch: int
    candidate_generation: int
    candidate_password: str
    status: PendingStatus = PendingStatus.PREPARED


@dataclass
class CredentialRecord:
    context: CredentialContext
    policy: FixedAlphabetPolicy
    root_epoch: int
    committed_generation: int
    history: List[Tuple[int, int]] = field(default_factory=list)
    pending: Optional[PendingMigration] = None

    def descriptor(self) -> Tuple[int, int]:
        return (self.root_epoch, self.committed_generation)


class AsterLifecycle:
    def __init__(self, service: SemanticExactPermutationService):
        self.service = service
        self.records: Dict[str, CredentialRecord] = {}

    def add_record(self, record_id: str, record: CredentialRecord) -> None:
        if record_id in self.records:
            raise ValueError("duplicate record")
        self.records[record_id] = record

    def prepare_root_migration(
        self,
        record_id: str,
        new_epoch: int,
        history_window: int = 8,
    ) -> str:
        r = self.records[record_id]
        if r.pending is not None:
            raise RuntimeError("pending migration already exists")
        # Include current committed descriptor in authenticated recent history.
        history = r.history + [r.descriptor()]
        gen, pwd = self.service.select_cross_epoch_candidate(
            r.context,
            r.policy,
            new_epoch,
            history,
            history_window,
        )
        r.pending = PendingMigration(
            old_epoch=r.root_epoch,
            old_generation=r.committed_generation,
            candidate_epoch=new_epoch,
            candidate_generation=gen,
            candidate_password=pwd,
        )
        return pwd

    def apply_evidence(self, record_id: str, evidence: Evidence) -> None:
        r = self.records[record_id]
        p = r.pending
        if p is None:
            raise RuntimeError("no pending migration")

        if evidence == Evidence.NEW_ONLY:
            r.history.append((p.old_epoch, p.old_generation))
            r.root_epoch = p.candidate_epoch
            r.committed_generation = p.candidate_generation
            r.pending = None
            return

        if evidence == Evidence.OLD_ONLY:
            r.pending = None
            return

        # BOTH is ambiguous for atomic replacement unless an adapter declares a
        # special overlap contract.  NEITHER, timeout, and contradictory
        # observations are also non-committal.
        p.status = PendingStatus.UNKNOWN

    def can_retire_epoch(self, epoch: int) -> bool:
        for r in self.records.values():
            if r.root_epoch == epoch:
                return False
            if r.pending is not None and (
                r.pending.old_epoch == epoch or r.pending.candidate_epoch == epoch
            ):
                return False
        return True

    def retire_epoch(self, epoch: int) -> None:
        if not self.can_retire_epoch(epoch):
            raise RuntimeError("epoch still referenced by committed/pending state")
        self.service.retire_epoch(epoch)
