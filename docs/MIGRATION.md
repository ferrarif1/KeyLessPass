# Pairwise-Wrapper to Shamir v3 Migration

## Safety properties

Migration does not change the Root Key or any service password. It verifies that recovery+computer, recovery+USB, and computer+USB wrappers all produce the same 256-bit key before writing v3.

## Procedure

1. Run dry-run validation with the legacy mnemonic and USB path.
2. Recover and compare all three legacy paths.
3. Generate a new `shareSetID` and three Shamir shares.
4. Write the platform-protected managed share to a generation-specific file.
5. Write the USB share and USB manifest to generation-specific storage.
6. Read back and test all three new recovery combinations.
7. Write the local v3 manifest last; this is the commit point.
8. Write a migration audit record without the recovery phrase.
9. Optionally move old wrapper files into `deprecated-pairwise-wrappers/<shareSetID>/`.

Generation-specific files ensure that a crash before manifest commit leaves the previous manifest selectable. A retry may create unused staged files; they are inert because no committed manifest references them.

## FFI request

```json
{
  "op": "migratePairwiseRecovery",
  "payload": {
    "mnemonic": "legacy phrase",
    "usbPath": "/Volumes/RECOVERY",
    "dryRun": true,
    "archiveLegacyWrappers": false
  }
}
```

Repeat with `dryRun: false`. The successful response contains the new recovery share phrase exactly once; it is deliberately excluded from the audit JSON. Save it offline before archiving v2. `archiveLegacyWrappers` is recoverable archival, not guaranteed secure deletion on flash or copy-on-write filesystems.

After v3 manifest commit, password derivation prefers v3 and interprets the input phrase as a recovery-share phrase. Mixing v2 and v3 factors is rejected.
