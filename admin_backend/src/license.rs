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

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeviceBatchApprovalInput {
    schema_version: u32,
    customer_id: String,
    customer_name: String,
    entitlement_serial: u64,
    site_key_id: String,
    site_public_key: String,
    purchased_device_limit: u32,
    current_customer_entitlement: SignedCustomerEntitlementEnvelope,
    #[serde(default)]
    requested_devices: Vec<DeviceBatchApprovalDevice>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeviceBatchApprovalDevice {
    device_key_id: String,
}

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

    #[cfg(test)]
    pub fn for_test(key_id: &str, seed: [u8; 32]) -> Self {
        Self {
            key_id: key_id.to_string(),
            signing_key: SigningKey::from_bytes(&seed),
        }
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
    verify_customer_entitlement_for_site(&envelope_json, site_signing)
}

pub fn verify_customer_entitlement_for_site(
    envelope_json: &str,
    site_signing: &SigningMaterial,
) -> Result<VerifiedCustomerEntitlement> {
    let vendor_key_id = std::env::var("KEYLESSPASS_VENDOR_KEY_ID")
        .unwrap_or_else(|_| "keylesspass-vendor-root-2026".to_string());
    let vendor_public_key = std::env::var("KEYLESSPASS_VENDOR_PUBLIC_KEY_B64")
        .context("KEYLESSPASS_VENDOR_PUBLIC_KEY_B64 is required")?;
    let verified = verify_customer_entitlement(envelope_json, &vendor_key_id, &vendor_public_key)?;
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
    let batch = std::env::var("KEYLESSPASS_DEVICE_BATCH_REQUEST_FILE")
        .ok()
        .map(|path| {
            let value = std::fs::read_to_string(&path)
                .with_context(|| format!("read device batch request file {path}"))?;
            serde_json::from_str::<DeviceBatchApprovalInput>(&value)
                .context("parse device batch request file")
        })
        .transpose()?;
    let trusted_batch_entitlement = batch
        .as_ref()
        .map(|input| verify_device_batch_entitlement(input, &vendor_signing))
        .transpose()?;
    let now = Utc::now();
    let valid_until = std::env::var("KEYLESSPASS_CUSTOMER_VALID_UNTIL")
        .ok()
        .or_else(|| {
            trusted_batch_entitlement
                .as_ref()
                .map(|value| value.payload.valid_until.clone())
        })
        .unwrap_or_else(|| (now + Duration::days(365)).to_rfc3339());
    parse_time(&valid_until)?;
    let batch_key_ids = batch
        .as_ref()
        .map(|input| {
            let mut values = input
                .requested_devices
                .iter()
                .map(|device| device.device_key_id.clone())
                .chain(
                    trusted_batch_entitlement
                        .as_ref()
                        .into_iter()
                        .flat_map(|value| value.payload.authorized_device_key_ids.iter().cloned()),
                )
                .collect::<Vec<_>>();
            values.sort();
            values.dedup();
            values
        })
        .unwrap_or_default();
    let authorized_device_key_ids = std::env::var("KEYLESSPASS_AUTHORIZED_DEVICE_KEY_IDS")
        .ok()
        .map(|_| env_list("KEYLESSPASS_AUTHORIZED_DEVICE_KEY_IDS", &[]))
        .unwrap_or(batch_key_ids);
    let default_device_limit = trusted_batch_entitlement
        .as_ref()
        .map(|value| value.payload.max_registered_devices)
        .unwrap_or(25);
    let max_registered_devices = std::env::var("KEYLESSPASS_MAX_REGISTERED_DEVICES")
        .ok()
        .map(|value| {
            value
                .parse()
                .context("KEYLESSPASS_MAX_REGISTERED_DEVICES must be an integer")
        })
        .transpose()?
        .unwrap_or(default_device_limit);
    validate_authorized_device_keys(&authorized_device_key_ids, max_registered_devices)?;
    let payload = CustomerEntitlement {
        schema_version: LICENSE_SCHEMA_VERSION,
        entitlement_id: format!("ent-{}", Uuid::new_v4()),
        entitlement_serial: std::env::var("KEYLESSPASS_ENTITLEMENT_SERIAL")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or_else(|| {
                trusted_batch_entitlement
                    .as_ref()
                    .map_or(1, |value| value.payload.entitlement_serial + 1)
            }),
        customer_id: env_or_batch(
            "KEYLESSPASS_CUSTOMER_ID",
            trusted_batch_entitlement
                .as_ref()
                .map(|value| &value.payload.customer_id),
        )?,
        customer_name: env_or_batch(
            "KEYLESSPASS_CUSTOMER_NAME",
            trusted_batch_entitlement
                .as_ref()
                .map(|value| &value.payload.customer_name),
        )?,
        product: "KeyLessPass".to_string(),
        site_key_id: env_or_batch(
            "KEYLESSPASS_SITE_KEY_ID",
            trusted_batch_entitlement
                .as_ref()
                .map(|value| &value.payload.site_key_id),
        )?,
        site_public_key: env_or_batch(
            "KEYLESSPASS_SITE_PUBLIC_KEY_B64",
            trusted_batch_entitlement
                .as_ref()
                .map(|value| &value.payload.site_public_key),
        )?,
        max_registered_devices,
        max_concurrent_devices: env_u32(
            "KEYLESSPASS_MAX_CONCURRENT_DEVICES",
            trusted_batch_entitlement
                .as_ref()
                .map_or(max_registered_devices, |value| {
                    value.payload.max_concurrent_devices
                }),
        )?,
        max_offline_borrowed: env_u32(
            "KEYLESSPASS_MAX_OFFLINE_BORROWED",
            trusted_batch_entitlement
                .as_ref()
                .map_or(0, |value| value.payload.max_offline_borrowed),
        )?,
        max_offline_grace_days: env_u32(
            "KEYLESSPASS_MAX_OFFLINE_GRACE_DAYS",
            trusted_batch_entitlement
                .as_ref()
                .map_or(14, |value| value.payload.max_offline_grace_days),
        )?,
        authorized_device_key_ids,
        valid_from: now.to_rfc3339(),
        valid_until,
        features: std::env::var("KEYLESSPASS_CUSTOMER_FEATURES")
            .ok()
            .map(|_| env_list("KEYLESSPASS_CUSTOMER_FEATURES", &[]))
            .or_else(|| {
                trusted_batch_entitlement
                    .as_ref()
                    .map(|value| value.payload.features.clone())
            })
            .unwrap_or_else(|| {
                vec![
                    "desktop-client".to_string(),
                    "channel:commercial".to_string(),
                ]
            }),
        allowed_major_versions: if std::env::var("KEYLESSPASS_ALLOWED_MAJOR_VERSIONS").is_ok() {
            env_list("KEYLESSPASS_ALLOWED_MAJOR_VERSIONS", &[])
                .into_iter()
                .map(|value| {
                    value
                        .parse()
                        .context("allowed major versions must be integers")
                })
                .collect::<Result<Vec<u32>>>()?
        } else {
            trusted_batch_entitlement
                .as_ref()
                .map(|value| value.payload.allowed_major_versions.clone())
                .unwrap_or_else(|| vec![1])
        },
        issued_at: now.to_rfc3339(),
        issuer: std::env::var("KEYLESSPASS_VENDOR_ISSUER")
            .ok()
            .or_else(|| {
                trusted_batch_entitlement
                    .as_ref()
                    .map(|value| value.payload.issuer.clone())
            })
            .unwrap_or_else(|| "KeyLessPass Vendor Licensing".to_string()),
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

fn verify_device_batch_entitlement(
    input: &DeviceBatchApprovalInput,
    vendor_signing: &SigningMaterial,
) -> Result<VerifiedCustomerEntitlement> {
    if input.schema_version != 1 {
        return Err(anyhow!("unsupported device batch request schema"));
    }
    let envelope_json = serde_json::to_string(&input.current_customer_entitlement)?;
    let verified = verify_customer_entitlement(
        &envelope_json,
        &vendor_signing.key_id,
        &vendor_signing.public_key_b64(),
    )?;
    if input.customer_id != verified.payload.customer_id
        || input.customer_name != verified.payload.customer_name
        || input.entitlement_serial != verified.payload.entitlement_serial
        || input.site_key_id != verified.payload.site_key_id
        || input.site_public_key != verified.payload.site_public_key
        || input.purchased_device_limit != verified.payload.max_registered_devices
    {
        return Err(anyhow!(
            "device batch summary does not match its vendor-signed entitlement"
        ));
    }
    Ok(verified)
}

fn env_or_batch(name: &str, fallback: Option<&String>) -> Result<String> {
    std::env::var(name)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| fallback.cloned())
        .ok_or_else(|| anyhow!("{name} is required"))
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

    #[test]
    fn batch_quota_must_match_the_existing_vendor_signature() {
        let vendor = SigningMaterial::for_test("vendor-test", [5; 32]);
        let now = Utc::now();
        let payload = CustomerEntitlement {
            schema_version: LICENSE_SCHEMA_VERSION,
            entitlement_id: "ent-test".to_string(),
            entitlement_serial: 7,
            customer_id: "customer-test".to_string(),
            customer_name: "Customer Test".to_string(),
            product: "KeyLessPass".to_string(),
            site_key_id: "site-test".to_string(),
            site_public_key: STANDARD.encode([3; 32]),
            max_registered_devices: 10,
            max_concurrent_devices: 10,
            max_offline_borrowed: 0,
            max_offline_grace_days: 14,
            authorized_device_key_ids: Vec::new(),
            valid_from: (now - Duration::days(1)).to_rfc3339(),
            valid_until: (now + Duration::days(30)).to_rfc3339(),
            features: vec!["desktop-client".to_string()],
            allowed_major_versions: vec![1],
            issued_at: now.to_rfc3339(),
            issuer: "Vendor Test".to_string(),
        };
        let payload_json = serde_json::to_vec(&payload).unwrap();
        let envelope = SignedCustomerEntitlementEnvelope {
            schema_version: LICENSE_SCHEMA_VERSION,
            envelope_type: CUSTOMER_ENTITLEMENT_TYPE.to_string(),
            payload: URL_SAFE_NO_PAD.encode(&payload_json),
            signature_algorithm: LICENSE_SIGNATURE_ALGORITHM.to_string(),
            key_id: vendor.key_id.clone(),
            signature: URL_SAFE_NO_PAD.encode(vendor.signing_key.sign(&payload_json).to_bytes()),
        };
        let mut request = DeviceBatchApprovalInput {
            schema_version: 1,
            customer_id: payload.customer_id.clone(),
            customer_name: payload.customer_name.clone(),
            entitlement_serial: payload.entitlement_serial,
            site_key_id: payload.site_key_id.clone(),
            site_public_key: payload.site_public_key.clone(),
            purchased_device_limit: payload.max_registered_devices,
            current_customer_entitlement: envelope,
            requested_devices: Vec::new(),
        };
        verify_device_batch_entitlement(&request, &vendor).unwrap();
        request.purchased_device_limit = 300;
        assert!(verify_device_batch_entitlement(&request, &vendor).is_err());
    }
}
