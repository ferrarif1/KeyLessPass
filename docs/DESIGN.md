# KeyLessPass Desktop Design

## Architecture

KeyLessPass is a desktop-only client:

- Flutter Desktop renders the native desktop UI.
- Rust Core implements cryptography, CDR storage, USB factor packages, recovery,
  and platform factor providers.
- SQLite stores only CDR metadata, never derived passwords.
- Flutter calls Rust through a small C ABI JSON FFI layer.

There is no web server, cloud sync, browser autofill, browser plugin, or WebView
main UI.

## Derivation Boundary

Mutable display fields do not affect derivation:

- `displayName`
- `serviceHint`
- `accountHint`

Stable derivation fields are:

- `recordSeq`
- `recordId`
- `version`
- `salt`
- `encodingDescriptor`

Changing `encodingDescriptor` requires a new version and is treated as password
rotation.

Normal derivation requires mnemonic + this computer. The Rust core recovers
`Kmaster` through `W_MC`; the USB package is intended to stay offline during
daily use and is only needed for enrollment, recovery, and factor replacement.

Recovery paths are 2-of-3:

- mnemonic + this computer decrypts `W_MC` and rebuilds a USB package.
- mnemonic + USB decrypts `W_MU` and rebuilds this computer's local factor.
- this computer + USB decrypts `W_CU` and resets the mnemonic without the old
  mnemonic.

Local and USB factor payloads do not persist plaintext `Kmaster`. The USB
package is ordinary copyable storage and is treated as a copyable factor
container.

## Platform Provider Trait

```rust
pub trait PlatformFactorProvider {
    fn platform_name(&self) -> String;
    fn get_or_create_device_id(&self) -> Result<String>;
    fn get_or_create_device_secret(&self) -> Result<SecretBytes>;
    fn protect_local_package(&self, plaintext: &[u8]) -> Result<Vec<u8>>;
    fn unprotect_local_package(&self, protected: &[u8]) -> Result<Vec<u8>>;
}
```

Windows uses a DPAPI extension point. macOS uses Keychain where available.
Linux/UOS/Kylin use local AEAD package protection plus file permissions in the
first release path. Fallback protection is surfaced to the UI as a reduced-security
state.

## FFI

Rust exports:

```c
char *keylesspass_ffi_json(const char *request_json);
void keylesspass_ffi_free(char *response_json);
```

Requests use:

```json
{"op":"derivePassword","payload":{...}}
```

Responses use:

```json
{"ok":true,"data":{...}}
```

or:

```json
{"ok":false,"error":"safe user-facing error"}
```

Sensitive derivation/recovery failures are intentionally generalized at the FFI
boundary.
