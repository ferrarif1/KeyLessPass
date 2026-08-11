use keylesspass_core::derivation::{
    derive_credential_key, derive_password_v3, permutation_tweak, Ff1CycleWalking,
    DERIVATION_VERSION_V3, ENCODER_VERSION_V3,
};
use keylesspass_core::domain::{CredentialDescriptionRecord, EncodingDescriptor};
use keylesspass_core::policy::PolicySpec;
use serde_json::json;
use uuid::Uuid;

fn main() {
    let root_key = [0x5a_u8; 32];
    let mut record = CredentialDescriptionRecord::new(
        Uuid::from_u128(0x00112233445566778899aabbccddeeff),
        7,
        42,
        "ignored display",
        "ignored service hint",
        "ignored account hint",
        "",
        EncodingDescriptor::default(),
    );
    record.service_id = Uuid::from_u128(0xaaaaaaaa111122223333444444444444);
    record.account_id = Uuid::from_u128(0xbbbbbbbb111122223333444444444444);
    record.policy_id = Uuid::from_u128(0xcccccccc111122223333444444444444);
    record.policy_version = 2;
    record.policy_epoch = Some(1);
    record.credential_generation = 0;
    record.derivation_version = DERIVATION_VERSION_V3;
    record.encoder_version = ENCODER_VERSION_V3;
    record.salt = "EREREREREREREREREREREQ==".to_string();

    let policy = PolicySpec::from_encoding_descriptor(&record.encoding_descriptor).unwrap();
    let policy_hash = policy.policy_hash().unwrap();
    let credential_key = derive_credential_key(&root_key, &record).unwrap();
    let tweak = permutation_tweak(&record, &policy_hash).unwrap();
    let derived = derive_password_v3(&root_key, &record, &Ff1CycleWalking::default()).unwrap();
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "vectorVersion": 1,
            "rootKey": keylesspass_core::crypto::b64_encode(&root_key),
            "policy": policy,
            "policyHash": keylesspass_core::crypto::b64_encode(&policy_hash),
            "credentialKey": keylesspass_core::crypto::b64_encode(&credential_key),
            "tweak": serde_json::from_slice::<serde_json::Value>(&tweak).unwrap(),
            "generation": record.credential_generation,
            "domainSize": derived.domain_size.to_string(),
            "rank": derived.rank.to_string(),
            "password": derived.password,
        }))
        .unwrap()
    );
}
