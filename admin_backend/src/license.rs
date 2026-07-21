use crate::model::{
    BundleRecord, DeviceGrant, DeviceRecord, LicenseBundlePayload, OrganizationLicense,
    OrganizationRecord, SignedLicenseEnvelope, LICENSE_ENVELOPE_TYPE, LICENSE_SCHEMA_VERSION,
    LICENSE_SIGNATURE_ALGORITHM,
};
use anyhow::{anyhow, Context, Result};
use base64::{
    engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD},
    Engine,
};
use chrono::Utc;
use ed25519_dalek::{Signer, SigningKey};
use rand::{rngs::OsRng, RngCore};
use uuid::Uuid;

#[derive(Clone)]
pub struct SigningMaterial {
    pub key_id: String,
    signing_key: SigningKey,
}

impl SigningMaterial {
    pub fn from_env() -> Result<Self> {
        let key_id = std::env::var("KEYLESSPASS_LICENSE_KEY_ID")
            .unwrap_or_else(|_| "keylesspass-license-2026-q3".to_string());
        let private_key = std::env::var("KEYLESSPASS_LICENSE_SIGNING_KEY_B64").context(
            "KEYLESSPASS_LICENSE_SIGNING_KEY_B64 must contain a base64 Ed25519 32-byte seed",
        )?;
        let signing_key = signing_key_from_b64(&private_key)?;
        Ok(Self {
            key_id,
            signing_key,
        })
    }

    pub fn public_key_b64(&self) -> String {
        STANDARD.encode(self.signing_key.verifying_key().as_bytes())
    }

    pub fn public_key_b64url(&self) -> String {
        URL_SAFE_NO_PAD.encode(self.signing_key.verifying_key().as_bytes())
    }
}

pub fn generate_key_output() -> String {
    let mut seed = [0u8; 32];
    OsRng.fill_bytes(&mut seed);
    let signing_key = SigningKey::from_bytes(&seed);
    format!(
        "KEYLESSPASS_LICENSE_SIGNING_KEY_B64={}\nKEYLESSPASS_LICENSE_PUBLIC_KEY_B64={}\nKEYLESSPASS_LICENSE_PUBLIC_KEY_B64URL={}\n",
        STANDARD.encode(seed),
        STANDARD.encode(signing_key.verifying_key().as_bytes()),
        URL_SAFE_NO_PAD.encode(signing_key.verifying_key().as_bytes())
    )
}

pub fn signing_key_from_b64(value: &str) -> Result<SigningKey> {
    let bytes = URL_SAFE_NO_PAD
        .decode(value)
        .or_else(|_| STANDARD.decode(value))
        .context("license signing key must be valid base64")?;
    match bytes.len() {
        32 => {
            let seed: [u8; 32] = bytes
                .as_slice()
                .try_into()
                .map_err(|_| anyhow!("license signing seed must be 32 bytes"))?;
            Ok(SigningKey::from_bytes(&seed))
        }
        64 => {
            let seed: [u8; 32] = bytes[..32]
                .try_into()
                .map_err(|_| anyhow!("license signing keypair seed must be 32 bytes"))?;
            Ok(SigningKey::from_bytes(&seed))
        }
        len => Err(anyhow!(
            "license signing key must decode to 32-byte seed or 64-byte keypair, got {len} bytes"
        )),
    }
}

pub fn build_payload(
    org: &OrganizationRecord,
    devices: &[DeviceRecord],
    revoked_grant_ids: Vec<String>,
    valid_until: String,
) -> LicenseBundlePayload {
    let now = Utc::now().to_rfc3339();
    let device_grants = devices
        .iter()
        .map(|device| DeviceGrant {
            schema_version: LICENSE_SCHEMA_VERSION,
            grant_id: format!("grant-{}", Uuid::new_v4()),
            license_id: org.license_id.clone(),
            organization_id: org.id.clone(),
            commercial_device_id: device.commercial_device_id.clone(),
            device_fingerprint: device.device_fingerprint.clone(),
            seat_label: device.seat_label.clone(),
            valid_from: org.valid_from.clone(),
            valid_until: valid_until.clone(),
            features: org.features.clone(),
            offline_grace_days: org.offline_grace_days,
            issued_at: now.clone(),
            issuer: org.issuer.clone(),
        })
        .collect();

    LicenseBundlePayload {
        schema_version: LICENSE_SCHEMA_VERSION,
        bundle_id: format!("bundle-{}", Uuid::new_v4()),
        organization_license: OrganizationLicense {
            schema_version: LICENSE_SCHEMA_VERSION,
            license_id: org.license_id.clone(),
            organization_id: org.id.clone(),
            organization_name: org.name.clone(),
            plan: org.plan.clone(),
            max_seats: org.max_seats,
            valid_from: org.valid_from.clone(),
            valid_until,
            features: org.features.clone(),
            offline_grace_days: org.offline_grace_days,
            allowed_major_versions: org.allowed_major_versions.clone(),
            issued_at: now.clone(),
            issuer: org.issuer.clone(),
        },
        device_grants,
        revoked_grant_ids,
        issued_at: now,
    }
}

pub fn sign_payload(
    signing: &SigningMaterial,
    payload: &LicenseBundlePayload,
) -> Result<SignedLicenseEnvelope> {
    let payload_json = serde_json::to_vec(payload)?;
    let signature = signing.signing_key.sign(&payload_json);
    Ok(SignedLicenseEnvelope {
        schema_version: LICENSE_SCHEMA_VERSION,
        envelope_type: LICENSE_ENVELOPE_TYPE.to_string(),
        payload: URL_SAFE_NO_PAD.encode(payload_json),
        signature_algorithm: LICENSE_SIGNATURE_ALGORITHM.to_string(),
        key_id: signing.key_id.clone(),
        signature: URL_SAFE_NO_PAD.encode(signature.to_bytes()),
    })
}

pub fn bundle_record_from_envelope(
    org: &OrganizationRecord,
    payload: &LicenseBundlePayload,
    envelope_json: String,
) -> BundleRecord {
    BundleRecord {
        id: format!("bundle-row-{}", Uuid::new_v4()),
        bundle_id: payload.bundle_id.clone(),
        organization_id: org.id.clone(),
        license_id: org.license_id.clone(),
        device_count: payload.device_grants.len() as u32,
        revoked_count: payload.revoked_grant_ids.len() as u32,
        valid_until: payload.organization_license.valid_until.clone(),
        issued_at: payload.issued_at.clone(),
        envelope_json,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signature, Verifier};

    #[test]
    fn generated_seed_can_sign_and_verify() {
        let mut seed = [7u8; 32];
        seed[0] = 42;
        let signing_key = signing_key_from_b64(&STANDARD.encode(seed)).unwrap();
        let signature = signing_key.sign(b"payload");
        signing_key
            .verifying_key()
            .verify(b"payload", &Signature::from_bytes(&signature.to_bytes()))
            .unwrap();
    }
}
