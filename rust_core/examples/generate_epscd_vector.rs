use keylesspass_core::crypto::b64_encode;
use keylesspass_core::epscd::{
    derive_credential_key, derive_password, permutation_tweak, CredentialContext, SCHEME_VERSION_V1,
};
use keylesspass_core::permutation::Ff1CycleWalking;
use keylesspass_core::policy::{CharacterClassConstraint, CompiledPolicy, PolicySpec};
use serde_json::json;
use uuid::Uuid;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root_key = [0x22_u8; 32];
    let context = CredentialContext {
        scheme_version: SCHEME_VERSION_V1,
        vault_id: Uuid::parse_str("00112233-4455-6677-8899-aabbccddeeff")?,
        service_id: Uuid::parse_str("aaaaaaaa-1111-2222-3333-444444444444")?,
        account_id: Uuid::parse_str("bbbbbbbb-1111-2222-3333-444444444444")?,
        lineage_id: Uuid::nil(),
        credential_salt: [0x11; 16],
        root_generation: 7,
        policy_id: Uuid::parse_str("cccccccc-1111-2222-3333-444444444444")?,
        policy_version: 2,
        policy_epoch: 3,
    };
    let policy_spec = PolicySpec {
        policy_ir_version: 1,
        min_length: 12,
        max_length: 12,
        alphabet: "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$".to_string(),
        forbidden_characters: String::new(),
        classes: vec![
            class("lower", "abcdefghijklmnopqrstuvwxyz"),
            class("upper", "ABCDEFGHIJKLMNOPQRSTUVWXYZ"),
            class("digit", "0123456789"),
            class("symbol", "!@#$"),
        ],
        fixed_characters: Vec::new(),
        fixed_prefix: String::new(),
        fixed_suffix: String::new(),
        forbidden_first_characters: "!@#$".to_string(),
        forbidden_last_characters: "!@#$".to_string(),
        max_total_per_character: None,
        max_identical_run: None,
        max_sequential_run: None,
        forbidden_substrings: Vec::new(),
    };
    let policy = CompiledPolicy::compile(policy_spec.clone())?;
    let generation = 42_u64;
    let policy_hash = policy_spec.policy_hash()?;
    let credential_key = derive_credential_key(&root_key, &context)?;
    let tweak = permutation_tweak(&context, &policy_hash)?;
    let derived = derive_password(
        &root_key,
        &context,
        generation,
        &policy,
        &Ff1CycleWalking::default(),
    )?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "scheme": "EPSCD",
            "schemeVersion": SCHEME_VERSION_V1,
            "rootKey": b64_encode(&root_key),
            "context": {
                "vaultID": context.vault_id,
                "serviceID": context.service_id,
                "accountID": context.account_id,
                "credentialSalt": b64_encode(&context.credential_salt),
                "rootGeneration": context.root_generation,
                "policyID": context.policy_id,
                "policyVersion": context.policy_version,
                "policyEpoch": context.policy_epoch
            },
            "policy": policy_spec,
            "policyHash": b64_encode(&policy_hash),
            "credentialKey": b64_encode(&credential_key),
            "tweak": serde_json::from_slice::<serde_json::Value>(&tweak)?,
            "generation": generation,
            "domainSize": derived.domain_size.to_string(),
            "rank": derived.rank.to_string(),
            "password": derived.password
        }))?
    );
    Ok(())
}

fn class(name: &str, alphabet: &str) -> CharacterClassConstraint {
    CharacterClassConstraint {
        name: name.to_string(),
        alphabet: alphabet.to_string(),
        min_count: 1,
        max_count: None,
    }
}
