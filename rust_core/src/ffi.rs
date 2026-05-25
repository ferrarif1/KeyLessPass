use crate::service::{
    add_credential, cancel_rotation, confirm_rotation, derive_password, enroll, get_app_status,
    get_security_status, list_credentials, recover_local, recover_usb, reset_application_data,
    reset_mnemonic, rotate_credential, update_credential_display, AddCredentialRequest,
    CancelRotationRequest, ConfirmRotationRequest, DerivePasswordRequest, EnrollmentRequest,
    GenerateMnemonicRequest, RecoverLocalRequest, RecoverUsbRequest, ResetApplicationDataRequest,
    ResetMnemonicRequest, RotateCredentialRequest, UpdateCredentialDisplayRequest, UsbCdrRequest,
    VerifyUsbPackageRequest,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FfiRequest {
    op: String,
    #[serde(default)]
    payload: Value,
}

#[no_mangle]
pub extern "C" fn keylesspass_ffi_json(input: *const c_char) -> *mut c_char {
    let response = ffi_json_inner(input);
    CString::new(response)
        .unwrap_or_else(|_| CString::new(r#"{"ok":false,"error":"invalid ffi response"}"#).unwrap())
        .into_raw()
}

#[no_mangle]
pub extern "C" fn keylesspass_ffi_free(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let _ = CString::from_raw(ptr);
    }
}

fn ffi_json_inner(input: *const c_char) -> String {
    if input.is_null() {
        return error_response("invalid request");
    }
    let raw = unsafe { CStr::from_ptr(input) };
    let Ok(text) = raw.to_str() else {
        return error_response("request must be UTF-8 JSON");
    };
    let request: FfiRequest = match serde_json::from_str(text) {
        Ok(value) => value,
        Err(_) => return error_response("invalid request JSON"),
    };
    dispatch(request)
}

fn dispatch(request: FfiRequest) -> String {
    let op = request.op.as_str();
    let result = match op {
        "getAppStatus" => get_app_status().map(to_value),
        "getSecurityStatus" => get_security_status().map(to_value),
        "listCredentials" => list_credentials().map(to_value),
        "listUsbCandidates" => crate::service::list_usb_candidates().map(to_value),
        "verifyUsbPackage" => parse::<VerifyUsbPackageRequest>(request.payload)
            .and_then(crate::service::verify_usb_package)
            .map(to_value),
        "getUsbCdrStatus" => parse::<UsbCdrRequest>(request.payload)
            .and_then(crate::service::get_usb_cdr_status)
            .map(to_value),
        "syncCdrToUsb" => parse::<UsbCdrRequest>(request.payload)
            .and_then(crate::service::sync_cdr_to_usb)
            .map(to_value),
        "restoreCdrFromUsb" => parse::<UsbCdrRequest>(request.payload)
            .and_then(crate::service::restore_cdr_from_usb)
            .map(to_value),
        "generateMnemonic" => parse::<GenerateMnemonicRequest>(request.payload)
            .and_then(crate::service::generate_mnemonic)
            .map(to_value),
        "enroll" => parse::<EnrollmentRequest>(request.payload)
            .and_then(enroll)
            .map(to_value),
        "addCredential" => parse::<AddCredentialRequest>(request.payload)
            .and_then(add_credential)
            .map(to_value),
        "updateCredentialDisplay" => parse::<UpdateCredentialDisplayRequest>(request.payload)
            .and_then(update_credential_display)
            .map(to_value),
        "derivePassword" => parse::<DerivePasswordRequest>(request.payload)
            .and_then(derive_password)
            .map(to_value),
        "rotateCredential" => parse::<RotateCredentialRequest>(request.payload)
            .and_then(rotate_credential)
            .map(to_value),
        "confirmRotation" => parse::<ConfirmRotationRequest>(request.payload)
            .and_then(confirm_rotation)
            .map(|_| Value::Null),
        "cancelRotation" => parse::<CancelRotationRequest>(request.payload)
            .and_then(cancel_rotation)
            .map(|_| Value::Null),
        "recoverUsb" => parse::<RecoverUsbRequest>(request.payload)
            .and_then(recover_usb)
            .map(to_value),
        "recoverLocal" => parse::<RecoverLocalRequest>(request.payload)
            .and_then(recover_local)
            .map(to_value),
        "resetMnemonic" => parse::<ResetMnemonicRequest>(request.payload)
            .and_then(reset_mnemonic)
            .map(to_value),
        "resetApplicationData" => parse::<ResetApplicationDataRequest>(request.payload)
            .and_then(reset_application_data)
            .map(|_| Value::Null),
        _ => Err("unsupported operation".to_string()),
    };

    match result {
        Ok(value) => serde_json::to_string(&json!({ "ok": true, "data": value }))
            .unwrap_or_else(|_| error_response("response serialization failed")),
        Err(error) => {
            let safe = safe_error(op, &error);
            serde_json::to_string(&json!({ "ok": false, "error": safe }))
                .unwrap_or_else(|_| error_response("operation failed"))
        }
    }
}

fn parse<T: for<'de> Deserialize<'de>>(payload: Value) -> Result<T, String> {
    serde_json::from_value(payload).map_err(|_| "invalid request payload".to_string())
}

fn to_value<T: serde::Serialize>(value: T) -> Value {
    serde_json::to_value(value).unwrap_or(Value::Null)
}

fn error_response(message: &str) -> String {
    serde_json::to_string(&json!({ "ok": false, "error": message }))
        .unwrap_or_else(|_| "{\"ok\":false,\"error\":\"operation failed\"}".to_string())
}

fn safe_error(op: &str, detail: &str) -> String {
    match op {
        "derivePassword" | "recoverUsb" | "recoverLocal" | "verifyUsbPackage" | "resetMnemonic"
        | "getUsbCdrStatus" | "syncCdrToUsb" | "restoreCdrFromUsb" => {
            "无法完成操作：所需本机材料、USB 因子或输入口令未通过安全校验。".to_string()
        }
        "enroll" if detail.contains("USB") || detail.contains("usb") => {
            "无法写入 USB 因子包，请检查 U 盘路径和写入权限。".to_string()
        }
        _ => detail.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ffi_rejects_unknown_operation() {
        let input = CString::new(r#"{"op":"unknown","payload":{}}"#).unwrap();
        let ptr = keylesspass_ffi_json(input.as_ptr());
        assert!(!ptr.is_null());
        let text = unsafe { CStr::from_ptr(ptr).to_string_lossy().to_string() };
        keylesspass_ffi_free(ptr);
        assert!(text.contains(r#""ok":false"#));
        assert!(text.contains("unsupported operation"));
    }

    #[test]
    fn ffi_generates_bilingual_mnemonic() {
        let input = CString::new(
            r#"{"op":"generateMnemonic","payload":{"language":"simplifiedChinese","wordCount":20}}"#,
        )
        .unwrap();
        let ptr = keylesspass_ffi_json(input.as_ptr());
        assert!(!ptr.is_null());
        let text = unsafe { CStr::from_ptr(ptr).to_string_lossy().to_string() };
        keylesspass_ffi_free(ptr);
        let value: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(value["ok"], true);
        assert_eq!(value["data"]["language"], "simplifiedChinese");
        assert_eq!(
            value["data"]["mnemonic"]
                .as_str()
                .unwrap()
                .split_whitespace()
                .count(),
            20
        );
    }
}
