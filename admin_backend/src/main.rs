mod db;
mod license;
mod model;
mod ui;

use crate::{
    db::Db,
    license::{
        build_payload, bundle_record_from_envelope, generate_key_output,
        issue_customer_entitlement_output, load_customer_entitlement, sign_payload,
        site_public_key_output, SigningMaterial, VerifiedCustomerEntitlement,
    },
    model::{
        ActivateLicenseRequest, AdminSnapshot, AdminStatus, ApiMessage, BulkImportResult,
        BundleRecord, CreateOrganizationRequest, DeviceAuthorizationRequest, DeviceRecord,
        ImportDeviceRequestBody, IssueBundleRequest, OrganizationRecord,
    },
};
use anyhow::{anyhow, Context, Result};
use axum::{
    extract::{DefaultBodyLimit, Path as AxumPath, Query, State},
    http::{header::AUTHORIZATION, HeaderMap, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::Read;
use std::{collections::HashSet, net::SocketAddr, path::PathBuf, sync::Arc};
use tokio::sync::Mutex;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

#[derive(Clone)]
struct AppState {
    db: Db,
    signing: SigningMaterial,
    config: Config,
    issuance_lock: Arc<Mutex<()>>,
    customer_entitlement: VerifiedCustomerEntitlement,
}

#[derive(Clone)]
struct Config {
    bind: SocketAddr,
    users: Vec<AdminUser>,
    issuer: String,
    database_path: PathBuf,
    download_dir: PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AdminUser {
    name: String,
    role: AdminRole,
    token: String,
    #[serde(default)]
    organization_id: Option<String>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
enum AdminRole {
    Auditor,
    Operator,
    Admin,
}

impl AdminRole {
    fn rank(self) -> u8 {
        match self {
            Self::Auditor => 0,
            Self::Operator => 1,
            Self::Admin => 2,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Auditor => "auditor",
            Self::Operator => "operator",
            Self::Admin => "admin",
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ApiError {
    error: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct IssueBundleResponse {
    bundle: BundleRecord,
    envelope_json: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ActivateLicenseResponse {
    envelope_json: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DownloadArtifact {
    file_name: String,
    size_bytes: u64,
    download_url: String,
    sha256: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DevicesQuery {
    organization_id: Option<String>,
}

type ApiResult<T> = std::result::Result<T, (StatusCode, Json<ApiError>)>;

#[tokio::main]
async fn main() -> Result<()> {
    match std::env::args().nth(1).as_deref() {
        Some("generate-key") => {
            print!("{}", generate_key_output());
            return Ok(());
        }
        Some("issue-customer-entitlement") => {
            print!("{}", issue_customer_entitlement_output()?);
            return Ok(());
        }
        Some("site-public-key") => {
            print!("{}", site_public_key_output()?);
            return Ok(());
        }
        _ => {}
    }

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "keylesspass_admin=info,tower_http=info".into()),
        )
        .init();

    let config = Config::from_env()?;
    let db = Db::open(&config.database_path)?;
    let signing = SigningMaterial::from_env()?;
    let customer_entitlement = load_customer_entitlement(&signing)?;
    let state = AppState {
        db,
        signing,
        config,
        issuance_lock: Arc::new(Mutex::new(())),
        customer_entitlement,
    };

    let download_service =
        ServeDir::new(&state.config.download_dir).append_index_html_on_directories(false);
    let app = Router::new()
        .route("/", get(index))
        .route("/download", get(download_page))
        .route("/api/downloads", get(list_downloads))
        .route("/healthz", get(healthz))
        .route("/api/status", get(api_status))
        .route("/api/snapshot", get(api_snapshot))
        .route(
            "/api/organizations",
            get(list_organizations).post(create_organization),
        )
        .route("/api/device-requests/import", post(import_device_request))
        .route(
            "/api/device-requests/import.csv",
            post(import_device_requests_csv),
        )
        .route("/api/devices", get(list_devices))
        .route("/api/devices.csv", get(export_devices_csv))
        .route("/api/licenses/issue", post(issue_bundle))
        .route("/api/activation/activate", post(activate_license))
        .route("/api/grants/:grant_id/revoke", post(revoke_grant))
        .route("/api/audit.csv", get(export_audit_csv))
        .nest_service("/downloads", download_service)
        .layer(DefaultBodyLimit::max(1024 * 1024))
        .layer(TraceLayer::new_for_http())
        .with_state(state.clone());

    tracing::info!(
        "KeyLessPass admin backend listening on http://{}",
        state.config.bind
    );
    let listener = tokio::net::TcpListener::bind(state.config.bind)
        .await
        .context("bind admin backend listener")?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("run admin backend")?;
    Ok(())
}

impl Config {
    fn from_env() -> Result<Self> {
        let bind = std::env::var("KEYLESSPASS_ADMIN_BIND")
            .unwrap_or_else(|_| "127.0.0.1:8787".to_string())
            .parse()
            .context("KEYLESSPASS_ADMIN_BIND must be host:port")?;
        let users = if let Ok(value) = std::env::var("KEYLESSPASS_ADMIN_USERS_JSON") {
            serde_json::from_str::<Vec<AdminUser>>(&value)
                .context("KEYLESSPASS_ADMIN_USERS_JSON must be a JSON array")?
        } else {
            vec![AdminUser {
                name: "legacy-admin".to_string(),
                role: AdminRole::Admin,
                token: std::env::var("KEYLESSPASS_ADMIN_TOKEN").context(
                    "KEYLESSPASS_ADMIN_TOKEN or KEYLESSPASS_ADMIN_USERS_JSON must be set",
                )?,
                organization_id: None,
            }]
        };
        if users.is_empty()
            || users
                .iter()
                .any(|user| user.name.trim().is_empty() || user.token.trim().len() < 24)
        {
            return Err(anyhow!(
                "admin users require a name and a token of at least 24 characters"
            ));
        }
        let unique_tokens: HashSet<&str> = users.iter().map(|user| user.token.as_str()).collect();
        if unique_tokens.len() != users.len() {
            return Err(anyhow!("admin user tokens must be unique"));
        }
        let database_path = std::env::var("KEYLESSPASS_ADMIN_DB")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("./keylesspass-admin.sqlite3"));
        let issuer = std::env::var("KEYLESSPASS_LICENSE_ISSUER")
            .unwrap_or_else(|_| "KeyLessPass Commercial Admin".to_string());
        let download_dir = std::env::var("KEYLESSPASS_DOWNLOAD_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("./downloads"));
        Ok(Self {
            bind,
            users,
            issuer,
            database_path,
            download_dir,
        })
    }
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}

async fn index() -> Html<&'static str> {
    Html(ui::INDEX_HTML)
}

async fn list_downloads(State(state): State<AppState>) -> Json<Vec<DownloadArtifact>> {
    Json(download_artifacts(&state.config.download_dir))
}

async fn download_page(State(state): State<AppState>) -> Html<String> {
    let links = download_artifacts(&state.config.download_dir)
        .into_iter()
        .map(|item| {
            format!(
                r#"<li><a href="{}">{}</a> <small>({} bytes)<br>SHA-256: {}</small></li>"#,
                item.download_url, item.file_name, item.size_bytes, item.sha256
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    Html(format!(
        r#"<!doctype html><html lang="zh-CN"><meta charset="utf-8"><meta name="viewport" content="width=device-width"><title>KeyLessPass 下载</title><style>body{{font:16px system-ui;max-width:760px;margin:64px auto;padding:0 20px;background:#111;color:#eee}}a{{color:#efff3d}}li{{margin:14px 0}}small{{color:#aaa}}</style><h1>KeyLessPass 应用下载</h1><p>无需登录。请只安装由供应商签名并公布校验值的正式版本。</p><ul>{}</ul><p><a href="/">管理员登录</a></p></html>"#,
        if links.is_empty() {
            "<li>暂无可下载版本</li>".to_string()
        } else {
            links
        }
    ))
}

fn download_artifacts(directory: &std::path::Path) -> Vec<DownloadArtifact> {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return Vec::new();
    };
    let mut artifacts = entries
        .flatten()
        .filter_map(|entry| {
            let metadata = entry.metadata().ok()?;
            if !metadata.is_file() {
                return None;
            }
            let file_name = entry.file_name().to_str()?.to_string();
            if !file_name
                .bytes()
                .all(|value| value.is_ascii_alphanumeric() || b"._-".contains(&value))
                || !is_download_artifact(&file_name)
            {
                return None;
            }
            Some(DownloadArtifact {
                download_url: format!("/downloads/{file_name}"),
                file_name,
                size_bytes: metadata.len(),
                sha256: sha256_file(&entry.path())?,
            })
        })
        .collect::<Vec<_>>();
    artifacts.sort_by(|left, right| left.file_name.cmp(&right.file_name));
    artifacts
}

fn sha256_file(path: &std::path::Path) -> Option<String> {
    let mut file = std::fs::File::open(path).ok()?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer).ok()?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    Some(
        digest
            .finalize()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect(),
    )
}

fn is_download_artifact(file_name: &str) -> bool {
    let lower = file_name.to_ascii_lowercase();
    [
        ".dmg",
        ".pkg",
        ".exe",
        ".msi",
        ".zip",
        ".deb",
        ".rpm",
        ".appimage",
        ".tar.gz",
    ]
    .iter()
    .any(|suffix| lower.ends_with(suffix))
}

async fn healthz() -> impl IntoResponse {
    Json(ApiMessage {
        message: "ok".to_string(),
    })
}

async fn api_status(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<AdminStatus>> {
    let actor = require_role(&headers, &state, AdminRole::Auditor)?;
    Ok(Json(
        status_for_user(&state, &actor).map_err(internal_error)?,
    ))
}

async fn api_snapshot(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<AdminSnapshot>> {
    let actor = require_role(&headers, &state, AdminRole::Operator)?;
    Ok(Json(snapshot(&state, &actor).map_err(internal_error)?))
}

async fn list_organizations(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<Vec<OrganizationRecord>>> {
    let actor = require_role(&headers, &state, AdminRole::Operator)?;
    let mut organizations = state.db.organizations().map_err(internal_error)?;
    if let Some(scope) = actor.organization_id.as_deref() {
        organizations.retain(|organization| organization.id == scope);
    }
    Ok(Json(organizations))
}

async fn create_organization(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CreateOrganizationRequest>,
) -> ApiResult<Json<OrganizationRecord>> {
    let actor = require_role(&headers, &state, AdminRole::Admin)?;
    if actor.organization_id.is_some() {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            "tenant administrators cannot create or resize organizations",
        ));
    }
    validate_create_organization_request(&state, &request).map_err(bad_request)?;
    let record = state
        .db
        .create_organization(request, &state.config.issuer)
        .map_err(bad_request)?;
    audit(&state, &actor, "organization.create", &record.id, "{}")?;
    Ok(Json(record))
}

async fn import_device_request(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<ImportDeviceRequestBody>,
) -> ApiResult<Json<DeviceRecord>> {
    let actor = require_role(&headers, &state, AdminRole::Operator)?;
    ensure_request_scope(&actor, &request)?;
    let record = state
        .db
        .import_device_request(request)
        .map_err(bad_request)?;
    audit(&state, &actor, "device.import", &record.id, "{}")?;
    Ok(Json(record))
}

async fn list_devices(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<DevicesQuery>,
) -> ApiResult<Json<Vec<DeviceRecord>>> {
    let actor = require_role(&headers, &state, AdminRole::Auditor)?;
    let organization_id = scoped_organization(&actor, query.organization_id.as_deref())?;
    Ok(Json(
        state
            .db
            .devices(organization_id.as_deref())
            .map_err(internal_error)?,
    ))
}

async fn import_device_requests_csv(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: String,
) -> ApiResult<Json<BulkImportResult>> {
    let actor = require_role(&headers, &state, AdminRole::Operator)?;
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(body.as_bytes());
    let mut requests = Vec::new();
    for row in reader.deserialize::<ImportDeviceRequestBody>() {
        let request = row.map_err(|error| bad_request(anyhow!(error)))?;
        ensure_request_scope(&actor, &request)?;
        requests.push(request);
    }
    if requests.is_empty() {
        return Err(bad_request(anyhow!("CSV contains no device requests")));
    }
    for request in requests.iter().cloned() {
        state
            .db
            .import_device_request(request)
            .map_err(bad_request)?;
    }
    let result = BulkImportResult {
        imported: requests.len() as u32,
    };
    audit(
        &state,
        &actor,
        "device.import.csv",
        "bulk",
        &format!(r#"{{"imported":{}}}"#, result.imported),
    )?;
    Ok(Json(result))
}

async fn export_devices_csv(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<DevicesQuery>,
) -> ApiResult<Response> {
    let actor = require_role(&headers, &state, AdminRole::Auditor)?;
    let organization_id = scoped_organization(&actor, query.organization_id.as_deref())?;
    let devices = state
        .db
        .devices(organization_id.as_deref())
        .map_err(internal_error)?;
    let mut writer = csv::Writer::from_writer(Vec::new());
    for device in devices {
        writer
            .serialize(device)
            .map_err(|error| internal_error(anyhow!(error)))?;
    }
    let bytes = writer
        .into_inner()
        .map_err(|error| internal_error(anyhow!(error.into_error())))?;
    let body = String::from_utf8(bytes).map_err(|error| internal_error(anyhow!(error)))?;
    Ok(csv_response("keylesspass-devices.csv", body))
}

async fn export_audit_csv(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    let actor = require_role(&headers, &state, AdminRole::Auditor)?;
    if actor.organization_id.is_some() {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            "tenant audit export is not available from the global audit stream",
        ));
    }
    let mut writer = csv::Writer::from_writer(Vec::new());
    for record in state.db.audit_log().map_err(internal_error)? {
        writer
            .serialize(record)
            .map_err(|error| internal_error(anyhow!(error)))?;
    }
    let bytes = writer
        .into_inner()
        .map_err(|error| internal_error(anyhow!(error.into_error())))?;
    let body = String::from_utf8(bytes).map_err(|error| internal_error(anyhow!(error)))?;
    Ok(csv_response("keylesspass-audit.csv", body))
}

async fn issue_bundle(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<IssueBundleRequest>,
) -> ApiResult<Json<IssueBundleResponse>> {
    let actor = require_role(&headers, &state, AdminRole::Operator)?;
    ensure_org_access(&actor, &request.organization_id)?;
    let _issuance_guard = state.issuance_lock.lock().await;
    let response = issue_bundle_inner(&state, request).map_err(bad_request)?;
    audit(
        &state,
        &actor,
        "license.issue",
        &response.bundle.bundle_id,
        "{}",
    )?;
    Ok(Json(response))
}

async fn activate_license(
    State(state): State<AppState>,
    Json(request): Json<ActivateLicenseRequest>,
) -> ApiResult<Json<ActivateLicenseResponse>> {
    let _issuance_guard = state.issuance_lock.lock().await;
    let org = state
        .db
        .organization_by_activation_code(request.activation_code.trim())
        .map_err(internal_error)?
        .ok_or_else(|| api_error(StatusCode::UNAUTHORIZED, "invalid activation code"))?;
    let parsed_request: DeviceAuthorizationRequest =
        serde_json::from_str(request.request_json.trim())
            .map_err(|_| bad_request(anyhow!("device authorization request JSON is invalid")))?;
    if let Some(existing) = state
        .db
        .device_by_identity(
            &parsed_request.commercial_device_id,
            &parsed_request.device_fingerprint,
        )
        .map_err(internal_error)?
    {
        if state
            .db
            .device_has_revoked_grant(&existing.id)
            .map_err(internal_error)?
        {
            return Err(api_error(
                StatusCode::FORBIDDEN,
                "this device has been revoked and cannot reactivate",
            ));
        }
    }
    let active_ids: HashSet<String> = state
        .db
        .active_licensed_device_ids(&org.id)
        .map_err(internal_error)?
        .into_iter()
        .collect();
    let existing_is_active = state
        .db
        .device_by_identity(
            &parsed_request.commercial_device_id,
            &parsed_request.device_fingerprint,
        )
        .map_err(internal_error)?
        .is_some_and(|device| active_ids.contains(&device.id));
    if active_ids.len() as u32 >= org.max_seats && !existing_is_active {
        return Err(bad_request(anyhow!("organization has no available seats")));
    }
    let device = state
        .db
        .import_device_request(ImportDeviceRequestBody {
            request_json: request.request_json,
            organization_id: Some(org.id.clone()),
            seat_label: request.seat_label,
        })
        .map_err(bad_request)?;
    let response = issue_bundle_inner(
        &state,
        IssueBundleRequest {
            organization_id: org.id.clone(),
            device_ids: vec![device.id.clone()],
            valid_days: None,
            valid_until: None,
            include_revocations: Some(true),
        },
    )
    .map_err(bad_request)?;
    state
        .db
        .record_audit(
            &format!("activation:{}", org.id),
            "client",
            "license.activate",
            &device.id,
            "{}",
        )
        .map_err(internal_error)?;
    Ok(Json(ActivateLicenseResponse {
        envelope_json: response.envelope_json,
    }))
}

fn issue_bundle_inner(
    state: &AppState,
    request: IssueBundleRequest,
) -> Result<IssueBundleResponse> {
    let org = state
        .db
        .organization(&request.organization_id)?
        .ok_or_else(|| anyhow!("organization does not exist"))?;
    validate_organization_entitlement(state, &org)?;
    let org_valid_until = DateTime::parse_from_rfc3339(&org.valid_until)
        .context("organization validUntil is invalid")?
        .with_timezone(&Utc);
    if org_valid_until < Utc::now() {
        return Err(anyhow!("organization license has expired"));
    }
    let devices = state.db.selected_devices(&org.id, &request.device_ids)?;
    if devices.is_empty() {
        return Err(anyhow!("at least one device is required"));
    }
    let approved_keys: HashSet<&str> = state
        .customer_entitlement
        .payload
        .authorized_device_key_ids
        .iter()
        .map(String::as_str)
        .collect();
    if let Some(device) = devices
        .iter()
        .find(|device| !approved_keys.contains(device.device_key_id.as_str()))
    {
        return Err(anyhow!(
            "device {} has not been approved by the vendor entitlement",
            device.device_key_id
        ));
    }
    let mut licensed: HashSet<String> = state
        .db
        .active_licensed_device_ids(&org.id)?
        .into_iter()
        .collect();
    licensed.extend(devices.iter().map(|device| device.id.clone()));
    if licensed.len() as u32 > org.max_seats {
        return Err(anyhow!("issued devices exceed organization maxSeats"));
    }

    let valid_until = normalize_issue_valid_until(&request, &org)?;
    let revoked_grant_ids = if request.include_revocations.unwrap_or(true) {
        state.db.revoked_grant_ids(&org.id)?
    } else {
        Vec::new()
    };
    let payload = build_payload(
        &org,
        &devices,
        revoked_grant_ids,
        valid_until,
        state.customer_entitlement.envelope.clone(),
    );
    let envelope = sign_payload(&state.signing, &payload)?;
    let envelope_json = serde_json::to_string_pretty(&envelope)?;
    let bundle = bundle_record_from_envelope(&org, &payload, envelope_json.clone());
    state
        .db
        .store_bundle(&bundle, &payload.device_grants, &devices, org.max_seats)?;
    Ok(IssueBundleResponse {
        bundle,
        envelope_json,
    })
}

fn validate_create_organization_request(
    state: &AppState,
    request: &CreateOrganizationRequest,
) -> Result<()> {
    let entitlement = &state.customer_entitlement.payload;
    if request.max_seats.unwrap_or(25) > entitlement.max_registered_devices
        || request.max_seats.unwrap_or(25) > entitlement.max_concurrent_devices
    {
        return Err(anyhow!("requested seats exceed the vendor entitlement"));
    }
    if request.offline_grace_days.unwrap_or(14) > entitlement.max_offline_grace_days {
        return Err(anyhow!(
            "requested offline grace exceeds the vendor entitlement"
        ));
    }
    if request
        .features
        .iter()
        .any(|feature| !entitlement.features.contains(feature))
    {
        return Err(anyhow!("requested features exceed the vendor entitlement"));
    }
    if request
        .allowed_major_versions
        .iter()
        .any(|version| !entitlement.allowed_major_versions.contains(version))
    {
        return Err(anyhow!(
            "requested application versions exceed the vendor entitlement"
        ));
    }
    let requested_until = if let Some(value) = request.valid_until.as_deref() {
        DateTime::parse_from_rfc3339(value)
            .context("organization validUntil is invalid")?
            .with_timezone(&Utc)
    } else {
        Utc::now() + Duration::days(request.valid_days.unwrap_or(365).max(1))
    };
    let entitlement_until = DateTime::parse_from_rfc3339(&entitlement.valid_until)
        .context("customer entitlement validUntil is invalid")?
        .with_timezone(&Utc);
    if requested_until > entitlement_until {
        return Err(anyhow!(
            "organization validity exceeds the vendor entitlement"
        ));
    }
    Ok(())
}

fn validate_organization_entitlement(
    state: &AppState,
    organization: &OrganizationRecord,
) -> Result<()> {
    let entitlement = &state.customer_entitlement.payload;
    if organization.max_seats > entitlement.max_registered_devices
        || organization.max_seats > entitlement.max_concurrent_devices
        || organization.offline_grace_days > entitlement.max_offline_grace_days
        || organization
            .features
            .iter()
            .any(|feature| !entitlement.features.contains(feature))
        || organization
            .allowed_major_versions
            .iter()
            .any(|version| !entitlement.allowed_major_versions.contains(version))
        || DateTime::parse_from_rfc3339(&organization.valid_until)?
            > DateTime::parse_from_rfc3339(&entitlement.valid_until)?
    {
        return Err(anyhow!(
            "organization exceeds its vendor-signed customer entitlement"
        ));
    }
    Ok(())
}

async fn revoke_grant(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath(grant_id): AxumPath<String>,
) -> ApiResult<Json<ApiMessage>> {
    let actor = require_role(&headers, &state, AdminRole::Operator)?;
    let grant = state
        .db
        .grants()
        .map_err(internal_error)?
        .into_iter()
        .find(|grant| grant.grant_id == grant_id)
        .ok_or_else(|| api_error(StatusCode::NOT_FOUND, "grant does not exist"))?;
    ensure_org_access(&actor, &grant.organization_id)?;
    state.db.revoke_grant(&grant_id).map_err(bad_request)?;
    audit(&state, &actor, "grant.revoke", &grant_id, "{}")?;
    Ok(Json(ApiMessage {
        message: "grant revoked".to_string(),
    }))
}

fn require_role(
    headers: &HeaderMap,
    state: &AppState,
    minimum_role: AdminRole,
) -> ApiResult<AdminUser> {
    let actual = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .unwrap_or_default();
    let user = state
        .config
        .users
        .iter()
        .find(|user| constant_time_token_eq(&user.token, actual))
        .cloned()
        .ok_or_else(|| api_error(StatusCode::UNAUTHORIZED, "missing or invalid admin token"))?;
    if user.role.rank() < minimum_role.rank() {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            "admin role does not permit this operation",
        ));
    }
    Ok(user)
}

fn constant_time_token_eq(expected: &str, actual: &str) -> bool {
    let expected = Sha256::digest(expected.as_bytes());
    let actual = Sha256::digest(actual.as_bytes());
    expected
        .iter()
        .zip(actual.iter())
        .fold(0_u8, |difference, (left, right)| {
            difference | (left ^ right)
        })
        == 0
}

fn admin_status(state: &AppState) -> Result<AdminStatus> {
    let (organization_count, device_count, bundle_count) = state.db.counts()?;
    Ok(AdminStatus {
        service: "keylesspass-admin".to_string(),
        key_id: state.signing.key_id.clone(),
        public_key_b64: state.signing.public_key_b64(),
        public_key_b64url: state.signing.public_key_b64url(),
        database_path: state.db.path().display().to_string(),
        customer_id: state.customer_entitlement.payload.customer_id.clone(),
        entitlement_serial: state.customer_entitlement.payload.entitlement_serial,
        entitlement_valid_until: state.customer_entitlement.payload.valid_until.clone(),
        max_registered_devices: state.customer_entitlement.payload.max_registered_devices,
        approved_device_count: state
            .customer_entitlement
            .payload
            .authorized_device_key_ids
            .len() as u32,
        organization_count,
        device_count,
        bundle_count,
    })
}

fn status_for_user(state: &AppState, user: &AdminUser) -> Result<AdminStatus> {
    if user.organization_id.is_none() {
        return admin_status(state);
    }
    Ok(snapshot(state, user)?.status)
}

fn snapshot(state: &AppState, user: &AdminUser) -> Result<AdminSnapshot> {
    let scope = user.organization_id.as_deref();
    let mut organizations = state.db.organizations()?;
    let mut devices = state.db.devices(scope)?;
    let mut grants = state.db.grants()?;
    let mut bundles = state.db.bundles()?;
    if let Some(scope) = scope {
        organizations.retain(|item| item.id == scope);
        devices.retain(|item| item.organization_id == scope);
        grants.retain(|item| item.organization_id == scope);
        bundles.retain(|item| item.organization_id == scope);
    }
    let status = if scope.is_some() {
        AdminStatus {
            service: "keylesspass-admin".to_string(),
            key_id: state.signing.key_id.clone(),
            public_key_b64: state.signing.public_key_b64(),
            public_key_b64url: state.signing.public_key_b64url(),
            database_path: "tenant-scoped".to_string(),
            customer_id: state.customer_entitlement.payload.customer_id.clone(),
            entitlement_serial: state.customer_entitlement.payload.entitlement_serial,
            entitlement_valid_until: state.customer_entitlement.payload.valid_until.clone(),
            max_registered_devices: state.customer_entitlement.payload.max_registered_devices,
            approved_device_count: state
                .customer_entitlement
                .payload
                .authorized_device_key_ids
                .len() as u32,
            organization_count: organizations.len() as u32,
            device_count: devices.len() as u32,
            bundle_count: bundles.len() as u32,
        }
    } else {
        admin_status(state)?
    };
    Ok(AdminSnapshot {
        status,
        organizations,
        devices,
        grants,
        bundles,
        audit_log: if scope.is_some() {
            Vec::new()
        } else {
            state.db.audit_log()?
        },
    })
}

fn ensure_org_access(user: &AdminUser, organization_id: &str) -> ApiResult<()> {
    if user
        .organization_id
        .as_deref()
        .is_some_and(|scope| scope != organization_id)
    {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            "administrator token is scoped to another organization",
        ));
    }
    Ok(())
}

fn scoped_organization(user: &AdminUser, requested: Option<&str>) -> ApiResult<Option<String>> {
    if let Some(scope) = user.organization_id.as_deref() {
        if requested.is_some_and(|value| value != scope) {
            return Err(api_error(
                StatusCode::FORBIDDEN,
                "administrator token is scoped to another organization",
            ));
        }
        return Ok(Some(scope.to_string()));
    }
    Ok(requested.map(str::to_string))
}

fn ensure_request_scope(user: &AdminUser, body: &ImportDeviceRequestBody) -> ApiResult<()> {
    let request: DeviceAuthorizationRequest = serde_json::from_str(body.request_json.trim())
        .map_err(|_| bad_request(anyhow!("device authorization request JSON is invalid")))?;
    let organization_id = body
        .organization_id
        .as_deref()
        .or(request.organization_id.as_deref())
        .ok_or_else(|| bad_request(anyhow!("organizationId is required")))?;
    ensure_org_access(user, organization_id)
}

fn normalize_issue_valid_until(
    request: &IssueBundleRequest,
    org: &OrganizationRecord,
) -> Result<String> {
    if let Some(value) = request
        .valid_until
        .as_ref()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    {
        let requested = DateTime::parse_from_rfc3339(&value)
            .map_err(|_| anyhow!("validUntil must be RFC3339, for example 2027-07-21T00:00:00Z"))?;
        let organization = DateTime::parse_from_rfc3339(&org.valid_until)
            .context("organization validUntil is invalid")?;
        if requested > organization {
            return Err(anyhow!(
                "bundle validUntil cannot exceed organization validUntil"
            ));
        }
        return Ok(requested.to_rfc3339());
    }
    if let Some(days) = request.valid_days {
        let requested = Utc::now() + Duration::days(days.max(1));
        let organization = DateTime::parse_from_rfc3339(&org.valid_until)
            .context("organization validUntil is invalid")?
            .with_timezone(&Utc);
        return Ok(requested.min(organization).to_rfc3339());
    }
    Ok(org.valid_until.clone())
}

fn audit(
    state: &AppState,
    actor: &AdminUser,
    action: &str,
    target: &str,
    details_json: &str,
) -> ApiResult<()> {
    state
        .db
        .record_audit(
            &actor.name,
            actor.role.as_str(),
            action,
            target,
            details_json,
        )
        .map_err(internal_error)
}

fn csv_response(filename: &str, body: String) -> Response {
    let mut response = body.into_response();
    response.headers_mut().insert(
        "content-type",
        HeaderValue::from_static("text/csv; charset=utf-8"),
    );
    if let Ok(value) = HeaderValue::from_str(&format!("attachment; filename=\"{filename}\"")) {
        response.headers_mut().insert("content-disposition", value);
    }
    response
}

fn api_error(status: StatusCode, message: &str) -> (StatusCode, Json<ApiError>) {
    (
        status,
        Json(ApiError {
            error: message.to_string(),
        }),
    )
}

fn bad_request(error: anyhow::Error) -> (StatusCode, Json<ApiError>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ApiError {
            error: error.to_string(),
        }),
    )
}

fn internal_error(error: anyhow::Error) -> (StatusCode, Json<ApiError>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiError {
            error: error.to_string(),
        }),
    )
}
