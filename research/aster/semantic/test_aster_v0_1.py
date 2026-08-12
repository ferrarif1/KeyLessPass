import unittest

from aster_model_v0_1 import (
    ApprovalAuthority,
    AsterLifecycle,
    AuthorizationError,
    CredentialContext,
    CredentialRecord,
    Evidence,
    EpochUnavailable,
    FixedAlphabetPolicy,
    PendingStatus,
    SemanticExactPermutationService,
)


def ctx(service="svc-A", account="alice"):
    return CredentialContext(
        vault_id="vault-1",
        service_id=service,
        account_id=account,
        lineage_id="lineage-1",
        credential_salt="salt-1",
        policy_id="policy-1",
        policy_hash="sha256:demo",
        policy_epoch=0,
    )


class AsterTests(unittest.TestCase):
    def setUp(self):
        self.authority = ApprovalAuthority(b"A" * 32)
        self.service = SemanticExactPermutationService(self.authority)
        self.service.install_epoch(1, b"K" * 32)
        self.policy = FixedAlphabetPolicy("ABCDEFGHJKLMNPQRSTUVWXYZ23456789", 4)

    def test_exact_scope_capability_cannot_amplify(self):
        c1 = ctx()
        cap = self.authority.issue(c1, 1, 0, now=1000, ttl_seconds=60)

        p1 = self.service.derive(c1, self.policy, 1, 0, cap, now=1001)
        self.assertEqual(len(p1), 4)

        # Same capability cannot be redirected to another service.
        with self.assertRaises(AuthorizationError):
            self.service.derive(
                ctx(service="svc-B"), self.policy, 1, 0, cap, now=1001
            )

        # Nor to another generation.
        with self.assertRaises(AuthorizationError):
            self.service.derive(c1, self.policy, 1, 1, cap, now=1001)

        # Nor replayed after its single-use budget is consumed.
        with self.assertRaises(AuthorizationError):
            self.service.derive(c1, self.policy, 1, 0, cap, now=1001)

    def test_same_epoch_generation_sequence_is_injective(self):
        c = ctx()
        outputs = [
            self.service.internal_password(c, self.policy, 1, g)
            for g in range(5000)
        ]
        self.assertEqual(len(outputs), len(set(outputs)))

    def test_deterministic_reconstruction(self):
        c = ctx()
        a = self.service.internal_password(c, self.policy, 1, 123)
        b = self.service.internal_password(c, self.policy, 1, 123)
        self.assertEqual(a, b)

    def test_root_epoch_replacement_changes_credential_family(self):
        c = ctx()
        old = self.service.internal_password(c, self.policy, 1, 17)
        self.service.install_epoch(2, b"Z" * 32)
        new = self.service.internal_password(c, self.policy, 2, 17)
        self.assertNotEqual(old, new)

    def test_cross_epoch_history_exclusion(self):
        c = ctx()
        self.service.install_epoch(2, b"Z" * 32)
        history = [(1, g) for g in range(20)]
        gen, candidate = self.service.select_cross_epoch_candidate(
            c, self.policy, 2, history, history_window=20
        )
        old_pwds = {
            self.service.internal_password(c, self.policy, 1, g) for g in range(20)
        }
        self.assertNotIn(candidate, old_pwds)
        self.assertGreaterEqual(gen, 0)

    def test_unknown_outcome_preserves_both_epochs(self):
        c = ctx()
        self.service.install_epoch(2, b"Z" * 32)
        lifecycle = AsterLifecycle(self.service)
        record = CredentialRecord(
            context=c,
            policy=self.policy,
            root_epoch=1,
            committed_generation=7,
        )
        lifecycle.add_record("r1", record)
        candidate = lifecycle.prepare_root_migration("r1", 2)

        self.assertTrue(candidate)
        self.assertEqual(record.root_epoch, 1)
        self.assertIsNotNone(record.pending)

        lifecycle.apply_evidence("r1", Evidence.TIMEOUT)
        self.assertEqual(record.root_epoch, 1)
        self.assertEqual(record.pending.status, PendingStatus.UNKNOWN)
        self.assertTrue(self.service.has_epoch(1))
        self.assertTrue(self.service.has_epoch(2))
        self.assertFalse(lifecycle.can_retire_epoch(1))

    def test_commit_then_safe_old_epoch_retirement(self):
        c = ctx()
        self.service.install_epoch(2, b"Z" * 32)
        lifecycle = AsterLifecycle(self.service)
        record = CredentialRecord(
            context=c,
            policy=self.policy,
            root_epoch=1,
            committed_generation=7,
        )
        lifecycle.add_record("r1", record)
        lifecycle.prepare_root_migration("r1", 2)
        lifecycle.apply_evidence("r1", Evidence.NEW_ONLY)

        self.assertEqual(record.root_epoch, 2)
        self.assertIsNone(record.pending)
        self.assertTrue(lifecycle.can_retire_epoch(1))
        lifecycle.retire_epoch(1)
        self.assertFalse(self.service.has_epoch(1))
        with self.assertRaises(EpochUnavailable):
            self.service.internal_password(c, self.policy, 1, 7)

    def test_cannot_retire_epoch_while_unknown(self):
        c = ctx()
        self.service.install_epoch(2, b"Z" * 32)
        lifecycle = AsterLifecycle(self.service)
        lifecycle.add_record(
            "r1",
            CredentialRecord(
                context=c,
                policy=self.policy,
                root_epoch=1,
                committed_generation=7,
            ),
        )
        lifecycle.prepare_root_migration("r1", 2)
        lifecycle.apply_evidence("r1", Evidence.CONTRADICTORY)
        with self.assertRaises(RuntimeError):
            lifecycle.retire_epoch(1)

    def test_old_compromised_key_does_not_determine_new_epoch(self):
        # Model the attacker retaining the complete old epoch key.
        c = ctx()
        old_only = SemanticExactPermutationService(self.authority)
        old_only.install_epoch(1, b"K" * 32)

        self.service.install_epoch(2, b"independent-new-root-key-32-byte!"[:32])

        new_pwd = self.service.internal_password(c, self.policy, 2, 3)
        with self.assertRaises(EpochUnavailable):
            old_only.internal_password(c, self.policy, 2, 3)

        # The attacker can still reconstruct old-epoch material; ASTER does not
        # claim to make already compromised history secret again.
        old_pwd = old_only.internal_password(c, self.policy, 1, 3)
        self.assertNotEqual(old_pwd, new_pwd)


if __name__ == "__main__":
    unittest.main()
