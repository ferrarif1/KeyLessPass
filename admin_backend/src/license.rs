use crate::model::{
    BundleRecord, CustomerEntitlement, DeviceGrant, DeviceRecord, LicenseBundlePayload,
    OrganizationLicense, OrganizationRecord, SignedCustomerEntitlementEnvelope,
    SignedLicenseEnvelope, CUSTOMER_ENTITLEMENT_TYPE, LICENSE_ENVELOPE_TYPE,
    LICENSE_SCHEMA_VERSION, LICENSE_SIGNATURE_ALGORITHM,
};
use anyhow::{anyhow, Context, Result};
use base64::{
    engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD},
    Engine,
};
use chrono::{DateTime, Duration, Utc};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::{rngs::OsRng, RngCore};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Clone)]
pub struct SigningMaterial {
    pub key_id: String,
    signing_key: SigningKey,
}

#[derive(Clone)]
pub struct VerifiedCustomerEntitlement {
    pub envelope: SignedCustomerEntitlementEnvelope,
    pub payload: CustomerEntitlement,
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

    fn vendor_from_env() -> Result<Self> {
        let key_id = std::env::var("KEYLESSPASS_VENDOR_KEY_ID")
            .unwrap_or_else(|_| "keylesspass-vendor-root-2026".to_string());
        let private_key = std::env::var("KEYLESSPASS_VENDOR_SIGNING_KEY_B64").context(
            "KEYLESSPASS_VENDOR_SIGNING_KEY_B64 must contain the offline vendor Ed25519 seed",
        )?;
        Ok(Self {
            key_id,
            signing_key: signing_key_from_b64(&private_key)?,
        })
    }
}

pub fn load_customer_entitlement(
    site_signing: &SigningMaterial,
) -> Result<VerifiedCustomerEntitlement> {
    let envelope_json = if let Ok(value) = std::env::var("KEYLESSPASS_CUSTOMER_ENTITLEMENT_JSON") {
        value
    } else {
        let path = std::env::var("KEYLESSPASS_CUSTOMER_ENTITLEMENT_FILE").context(
            "KEYLESSPASS_CUSTOMER_ENTITLEMENT_FILE or KEYLESSPASS_CUSTOMER_ENTITLEMENT_JSON is required",
        )?;
        std::fs::read_to_string(&path)
            .with_context(|| format!("read customer entitlement file {path}"))?
    };
    let vendor_key_id = std::env::var("KEYLESSPASS_VENDOR_KEY_ID")
        .unwrap_or_else(|_| "keylesspass-vendor-root-2026".to_string());
    let vendor_public_key = std::env::var("KEYLESSPASS_VENDOR_PUBLIC_KEY_B64")
        .context("KEYLESSPASS_VENDOR_PUBLIC_KEY_B64 is required")?;
    let verified = verify_customer_entitlement(&envelope_json, &vendor_key_id, &vendor_public_key)?;
    if verified.payload.site_key_id != site_signing.key_id
        || decode_key(&verified.payload.site_public_key)?
            != *site_signing.signing_key.verifying_key().as_bytes()
    {
        return Err(anyhow!(
            "customer entitlement does not delegate to this site signing key"
        ));
    }
    let now = Utc::now();
    if now < parse_time(&verified.payload.valid_from)?
        || now > parse_time(&verified.payload.valid_until)?
    {
        return Err(anyhow!("customer entitlement is not currently valid"));
    }
    Ok(verified)
}

pub fn issue_customer_entitlement_output() -> Result<String> {
    let vendor_signing = SigningMaterial::vendor_from_env()?;
    let now = Utc::now();
    let valid_until = std::env::var("KEYLESSPASS_CUSTOMER_VALID_UNTIL")
        .unwrap_or_else(|_| (now + Duration::days(365)).to_rfc3339());
    parse_time(&valid_until)?;
    let authorized_device_key_ids = env_list("KEYLESSPASS_AUTHORIZED_DEVICE_KEY_IDS", &[]);
    validate_authorized_device_keys(
        &authorized_device_key_ids,
        env_u32("KEYLESSPASS_MAX_REGISTERED_DEVICES", 25)?,
    )?;
    let payload = CustomerEntitlement {
        schema_version: LICENSE_SCHEMA_VERSION,
        entitlement_id: format!("ent-{}", Uuid::new_v4()),
        entitlement_serial: std::env::var("KEYLESSPASS_ENTITLEMENT_SERIAL")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(1),
        customer_id: required_env("KEYLESSPASS_CUSTOMER_ID")?,
        customer_name: required_env("KEYLESSPASS_CUSTOMER_NAME")?,
        product: "KeyLessPass".to_string(),
        site_key_id: required_env("KEYLESSPASS_SITE_KEY_ID")?,
        site_public_key: required_env("KEYLESSPASS_SITE_PUBLIC_KEY_B64")?,
        max_registered_devices: env_u32("KEYLESSPASS_MAX_REGISTERED_DEVICES", 25)?,
        max_concurrent_devices: env_u32("KEYLESSPASS_MAX_CONCURRENT_DEVICES", 25)?,
        max_offline_borrowed: env_u32("KEYLESSPASS_MAX_OFFLINE_BORROWED", 0)?,
        max_offline_grace_days: env_u32("KEYLESSPASS_MAX_OFFLINE_GRACE_DAYS", 14)?,
        authorized_device_key_ids,
        valid_from: now.to_rfc3339(),
        valid_until,
        features: env_list(
            "KEYLESSPASS_CUSTOMER_FEATURES",
            &["desktop-client", "channel:commercial"],
        ),
        allowed_major_versions: env_list("KEYLESSPASS_ALLOWED_MAJOR_VERSIONS", &["1"])
            .into_iter()
            .map(|value| {
                value
                    .parse()
                    .context("allowed major versions must be integers")
            })
            .collect::<Result<Vec<u32>>>()?,
        issued_at: now.to_rfc3339(),
        issuer: std::env::var("KEYLESSPASS_VENDOR_ISSUER")
            .unwrap_or_else(|_| "KeyLessPass Vendor Licensing".to_string()),
    };
    if payload.max_concurrent_devices > payload.max_registered_devices
        || payload.max_offline_borrowed > payload.max_registered_devices
    {
        return Err(anyhow!("customer entitlement limits are inconsistent"));
    }
    decode_key(&payload.site_public_key)?;
    let payload_json = serde_json::to_vec(&payload)?;
    let envelope = SignedCustomerEntitlementEnvelope {
        schema_version: LICENSE_SCHEMA_VERSION,
        envelope_type: CUSTOMER_ENTITLEMENT_TYPE.to_string(),
        payload: URL_SAFE_NO_PAD.encode(&payload_json),
        signature_algorithm: LICENSE_SIGNATURE_ALGORITHM.to_string(),
        key_id: vendor_signing.key_id,
        signature: URL_SAFE_NO_PAD
            .encode(vendor_signing.signing_key.sign(&payload_json).to_bytes()),
    };
    Ok(serde_json::to_string_pretty(&envelope)?)
}

pub fn site_public_key_output() -> Result<String> {
    let signing = SigningMaterial::from_env()?;
    Ok(format!(
        "KEYLESSPASS_SITE_KEY_ID={}\nKEYLESSPASS_SITE_PUBLIC_KEY_B64={}\n",
        signing.key_id,
        signing.public_key_b64()
    ))
}

pub fn verify_customer_entitlement(
    envelope_json: &str,
    vendor_key_id: &str,
    vendor_public_key: &str,
) -> Result<VerifiedCustomerEntitlement> {
    let envelope: SignedCustomerEntitlementEnvelope = serde_json::from_str(envelope_json)?;
    if envelope.schema_version != LICENSE_SCHEMA_VERSION
        || envelope.envelope_type != CUSTOMER_ENTITLEMENT_TYPE
        || envelope.signature_algorithm != LICENSE_SIGNATURE_ALGORITHM
        || envelope.key_id != vendor_key_id
    {
        return Err(anyhow!("customer entitlement envelope is invalid"));
    }
    let payload_bytes = URL_SAFE_NO_PAD.decode(&envelope.payload)?;
    let signature: [u8; 64] = URL_SAFE_NO_PAD
        .decode(&envelope.signature)?
        .try_into()
        .map_err(|_| anyhow!("customer entitlement signature must be 64 bytes"))?;
    let public_key = decode_key(vendor_public_key)?;
    VerifyingKey::from_bytes(&public_key)?
        .verify(&payload_bytes, &Signature::from_bytes(&signature))
        .map_err(|_| anyhow!("customer entitlement signature is invalid"))?;
    let payload: CustomerEntitlement = serde_json::from_slice(&payload_bytes)?;
    if payload.schema_version != LICENSE_SCHEMA_VERSION || payload.product != "KeyLessPass" {
        return Err(anyhow!("customer entitlement payload is invalid"));
    }
    validate_authorized_device_keys(
        &payload.authorized_device_key_ids,
        payload.max_registered_devices,
    )?;
    Ok(VerifiedCustomerEntitlement { envelope, payload })
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
    customer_entitlement: SignedCustomerEntitlementEnvelope,
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
            device_key_id: device.device_key_id.clone(),
            device_public_key: device.device_public_key.clone(),
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
        customer_entitlement,
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

fn required_env(name: &str) -> Result<String> {
    std::env::var(name).with_context(|| format!("{name} is required"))
}

fn env_u32(name: &str, default: u32) -> Result<u32> {
    std::env::var(name)
        .ok()
        .map(|value| {
            value
                .parse()
                .with_context(|| format!("{name} must be an integer"))
        })
        .transpose()
        .map(|value| value.unwrap_or(default))
}

fn env_list(name: &str, default: &[&str]) -> Vec<String> {
    std::env::var(name)
        .ok()
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_else(|| default.iter().map(|value| value.to_string()).collect())
}

fn decode_key(value: &str) -> Result<[u8; 32]> {
    URL_SAFE_NO_PAD
        .decode(value)
        .or_else(|_| STANDARD.decode(value))?
        .try_into()
        .map_err(|_| anyhow!("public key must be 32 bytes"))
}

fn parse_time(value: &str) -> Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .map(|value| value.with_timezone(&Utc))
        .map_err(|_| anyhow!("invalid entitlement date"))
}

fn validate_authorized_device_keys(device_key_ids: &[String], max_devices: u32) -> Result<()> {
    if device_key_ids.len() > max_devices as usize {
        return Err(anyhow!(
            "authorized device key count exceeds maxRegisteredDevices"
        ));
    }
    let mut unique = HashSet::with_capacity(device_key_ids.len());
    for key_id in device_key_ids {
        if key_id.len() != 64
            || !key_id
                .bytes()
                .all(|value| value.is_ascii_digit() || (b'a'..=b'f').contains(&value))
        {
            return Err(anyhow!(
                "authorized device key IDs must be lowercase SHA-256 hex strings"
            ));
        }
        if !unique.insert(key_id) {
            return Err(anyhow!("authorized device key IDs must be unique"));
        }
    }
    Ok(())
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
