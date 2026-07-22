mod db;
mod license;
mod model;
mod ui;

use crate::{
    db::Db,
    license::{
        build_payload, bundle_record_from_envelope, generate_key_output, sign_payload,
        SigningMaterial,
    },
    model::{
        ActivateLicenseRequest, AdminSnapshot, AdminStatus, ApiMessage, BulkImportResult,
        BundleRecord, CreateOrganizationRequest, DeviceAuthorizationRequest, DeviceRecord,
        ImportDeviceRequestBody, IssueBundleRequest, OrganizationRecord,
    },
};
use anyhow::{anyhow, Context, Result};
use axum::{
    extract::{Path as AxumPath, Query, State},
    http::{header::AUTHORIZATION, HeaderMap, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, net::SocketAddr, path::PathBuf};
use tower_http::trace::TraceLayer;

#[derive(Clone)]
struct AppState {
    db: Db,
    signing: SigningMaterial,
    config: Config,
}

#[derive(Clone)]
struct Config {
    bind: SocketAddr,
    users: Vec<AdminUser>,
    issuer: String,
    database_path: PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AdminUser {
    name: String,
    role: AdminRole,
    token: String,
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DevicesQuery {
    organization_id: Option<String>,
}

type ApiResult<T> = std::result::Result<T, (StatusCode, Json<ApiError>)>;

#[tokio::main]
async fn main() -> Result<()> {
    if std::env::args().nth(1).as_deref() == Some("generate-key") {
        print!("{}", generate_key_output());
        return Ok(());
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
    let state = AppState {
        db,
        signing,
        config,
    };

    let app = Router::new()
        .route("/", get(index))
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
        Ok(Self {
            bind,
            users,
            issuer,
            database_path,
        })
    }
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}

async fn index() -> Html<&'static str> {
    Html(ui::INDEX_HTML)
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
    require_role(&headers, &state, AdminRole::Auditor)?;
    Ok(Json(admin_status(&state).map_err(internal_error)?))
}

async fn api_snapshot(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<AdminSnapshot>> {
    require_role(&headers, &state, AdminRole::Operator)?;
    Ok(Json(snapshot(&state).map_err(internal_error)?))
}

async fn list_organizations(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<Vec<OrganizationRecord>>> {
    require_role(&headers, &state, AdminRole::Operator)?;
    Ok(Json(state.db.organizations().map_err(internal_error)?))
}

async fn create_organization(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CreateOrganizationRequest>,
) -> ApiResult<Json<OrganizationRecord>> {
    let actor = require_role(&headers, &state, AdminRole::Admin)?;
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
    require_role(&headers, &state, AdminRole::Auditor)?;
    Ok(Json(
        state
            .db
            .devices(query.organization_id.as_deref())
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
        requests.push(row.map_err(|error| bad_request(anyhow!(error)))?);
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
    require_role(&headers, &state, AdminRole::Auditor)?;
    let devices = state
        .db
        .devices(query.organization_id.as_deref())
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
    require_role(&headers, &state, AdminRole::Auditor)?;
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
    let org = state
        .db
        .organization_by_activation_code(request.activation_code.trim())
        .map_err(internal_error)?
        .ok_or_else(|| api_error(StatusCode::UNAUTHORIZED, "invalid activation code"))?;
    let parsed_request: DeviceAuthorizationRequest =
        serde_json::from_str(request.request_json.trim())
            .map_err(|_| bad_request(anyhow!("device authorization request JSON is invalid")))?;
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
    let payload = build_payload(&org, &devices, revoked_grant_ids, valid_until);
    let envelope = sign_payload(&state.signing, &payload)?;
    let envelope_json = serde_json::to_string_pretty(&envelope)?;
    let bundle = bundle_record_from_envelope(&org, &payload, envelope_json.clone());
    state
        .db
        .store_bundle(&bundle, &payload.device_grants, &devices)?;
    Ok(IssueBundleResponse {
        bundle,
        envelope_json,
    })
}

async fn revoke_grant(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath(grant_id): AxumPath<String>,
) -> ApiResult<Json<ApiMessage>> {
    let actor = require_role(&headers, &state, AdminRole::Admin)?;
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
        .find(|user| user.token == actual)
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

fn admin_status(state: &AppState) -> Result<AdminStatus> {
    let (organization_count, device_count, bundle_count) = state.db.counts()?;
    Ok(AdminStatus {
        service: "keylesspass-admin".to_string(),
        key_id: state.signing.key_id.clone(),
        public_key_b64: state.signing.public_key_b64(),
        public_key_b64url: state.signing.public_key_b64url(),
        database_path: state.db.path().display().to_string(),
        organization_count,
        device_count,
        bundle_count,
    })
}

fn snapshot(state: &AppState) -> Result<AdminSnapshot> {
    Ok(AdminSnapshot {
        status: admin_status(state)?,
        organizations: state.db.organizations()?,
        devices: state.db.devices(None)?,
        grants: state.db.grants()?,
        bundles: state.db.bundles()?,
        audit_log: state.db.audit_log()?,
    })
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
