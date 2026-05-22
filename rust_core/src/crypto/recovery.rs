use crate::crypto::mac;
use crate::domain::RecoveryMetadata;
use crate::error::Result;

pub fn build_recovery_metadata(master_key: &[u8], generation: u64) -> Result<RecoveryMetadata> {
    let fragment = mac::hmac_sha256(master_key, b"KeylessPass recovery fragment")?;
    let encrypted_fragment = crate::crypto::b64_encode(&fragment);
    let fragment_mac = mac::hmac_sha256_base64(master_key, encrypted_fragment.as_bytes())?;
    Ok(RecoveryMetadata::new(
        1,
        encrypted_fragment,
        fragment_mac,
        generation,
    ))
}
