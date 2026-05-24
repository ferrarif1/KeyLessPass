use keylesspass_core::platform::current_platform_provider;
use keylesspass_core::service::credentials::{add_credential_with_provider, AddCredentialRequest};
use keylesspass_core::service::enrollment::{enroll_with_provider, EnrollmentRequest};
use keylesspass_core::storage::StoragePaths;
use std::env;
use std::path::PathBuf;

const DEMO_MNEMONIC: &str =
    "anchor bridge cedar delta ember forest galaxy harbor ivory jasmine kernel lantern";

fn main() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let home = args
        .next()
        .ok_or_else(|| "usage: seed_ui_state <keylesspass-home> <usb-root>".to_string())?;
    let usb = args
        .next()
        .ok_or_else(|| "usage: seed_ui_state <keylesspass-home> <usb-root>".to_string())?;

    let paths = StoragePaths::from_app_dir(PathBuf::from(home));
    let provider = current_platform_provider(&paths.app_dir);
    enroll_with_provider(
        &paths,
        provider.as_ref(),
        EnrollmentRequest {
            mnemonic: DEMO_MNEMONIC.to_string(),
            usb_path: usb,
        },
    )
    .map_err(String::from)?;

    for (display_name, service_hint, account_hint) in [
        ("Operations Console", "ops.internal.local", "operator"),
        ("Vendor Portal", "vendor.internal.local", "reviewer"),
        (
            "Database Gateway",
            "database.internal.local",
            "admin",
        ),
    ] {
        add_credential_with_provider(
            &paths,
            provider.as_ref(),
            AddCredentialRequest {
                display_name: display_name.to_string(),
                service_hint: service_hint.to_string(),
                account_hint: account_hint.to_string(),
                notes: String::new(),
                encoding_descriptor: None,
            },
        )
        .map_err(String::from)?;
    }

    Ok(())
}
