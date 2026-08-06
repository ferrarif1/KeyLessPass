# Key Hierarchy

Crypto suite 1 uses a uniformly random 256-bit Root Key and HKDF-SHA-256 for high-entropy key separation. HKDF is not used as password hardening. User-entered legacy mnemonics are handled only by the v2 migration reader with Argon2id.

## Root and subkeys

For every subkey:

```text
salt = SHA-256("KeyLessPass/vault-subkey/salt/v1" || vaultID || rootGeneration || cryptoSuiteVersion)
K_purpose = HKDF-SHA-256(K_root, salt, "KeyLessPass/vault-subkey/v1/" || purpose, 32)
```

| Object | Length | Generation / derivation | Purpose | Persistent location | Confidentiality / integrity | Lifetime and revocation | Exposure impact | Loss impact |
|---|---:|---|---|---|---|---|---|---|
| `K_root` | 256 bit | OS CSPRNG | Root for all vault subkeys | Never stored whole in v3; reconstructed from two shares | Shamir at rest; KCV after recovery | `rootGeneration`; rotate after threshold compromise | All current vault-derived secrets exposed | Vault cannot derive passwords without two valid shares |
| Paper recovery share | 33 bytes | `vsss-rs` GF(256) Shamir | Offline custody factor | Printable package; current optional word representation | Envelope HMAC after reconstruction; package checksum | New `shareSetID` for replacement; new root after two-share compromise | One share reveals no Root Key under Shamir assumptions | Recover with computer+USB |
| Managed share | 33 bytes | Same share set | Computer factor | Generation-specific platform-protected file | OS/provider protection plus envelope HMAC | Replace computer, increment factor generation, commit new share set | One share; endpoint controls may expose it at runtime | Recover with phrase+USB |
| USB share | 33 bytes | Same share set | Removable possession factor | Generation-specific JSON on ordinary USB | Envelope HMAC; no claim of non-copyability | Replace USB and issue a new share set/phrase | One copy is one compromised factor | Recover with phrase+computer |
| `K_password_derivation` | 256 bit | purpose `password-derivation` | Service-secret hierarchy | Memory only | Root-derived | Root generation | All derived service passwords for that root generation | Password derivation unavailable |
| `K_cdr_authentication` | 256 bit | purpose `cdr-authentication` | RFC 8785 CDR MAC | Memory only | Root-derived HMAC key | Root generation | CDR forgery for affected vault | Existing CDRs cannot be authenticated |
| `K_metadata_encryption` | 256 bit | purpose `metadata-encryption` | Reserved for protected sensitive metadata | Memory only; no current encrypted metadata consumer | Root-derived | Root generation | Protected metadata disclosure | Encrypted metadata unavailable |
| `K_operation_log` | 256 bit | purpose `operation-log` | Operation digest/MAC | Memory only | Root-derived | Root generation | Local log forgery | Audit validation unavailable |
| `K_key_confirmation` | 256 bit | purpose `root-key-confirmation` | KCV | Memory only; KCV is persistent | HMAC over fixed context, vault and generation | Root generation | KCV is not a Root-Key verifier for low-entropy secrets; Root Key is random | Wrong shares cannot be confirmed |
| `K_backup_encryption` | 256 bit | purpose `backup-encryption` | Future encrypted backup payloads | Memory only | Root-derived | Root generation | Backup confidentiality loss | Backup unavailable |
| `K_share_authentication` | 256 bit | purpose `recovery-share-authentication` | Share-envelope metadata MAC | Memory only | Root-derived | Root generation/share set bindings in message | Envelope forgery after Root-Key compromise | Bad shares fail KCV/MAC |

Derivation version 1 retains the historical `derive_password_root_from_master` path for existing service-password compatibility. Derivation version 2 uses the `password-derivation` vault subkey, RFC 8785 input, and a fixed vector. A record never changes derivation version without an explicit remote credential rotation.
