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
        AdminSnapshot, AdminStatus, ApiMessage, BundleRecord, CreateOrganizationRequest,
        DeviceRecord, ImportDeviceRequestBody, IssueBundleRequest, OrganizationRecord,
    },
};
use anyhow::{anyhow, Context, Result};
use axum::{
    extract::{Path as AxumPath, Query, State},
    http::{header::AUTHORIZATION, HeaderMap, StatusCode},
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, path::PathBuf};
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
    admin_token: String,
    issuer: String,
    database_path: PathBuf,
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
        .route("/api/devices", get(list_devices))
        .route("/api/licenses/issue", post(issue_bundle))
        .route("/api/grants/:grant_id/revoke", post(revoke_grant))
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
        let admin_token = std::env::var("KEYLESSPASS_ADMIN_TOKEN")
            .context("KEYLESSPASS_ADMIN_TOKEN must be set for the admin API")?;
        if admin_token.trim().len() < 24 {
            return Err(anyhow!(
                "KEYLESSPASS_ADMIN_TOKEN must be at least 24 characters"
            ));
        }
        let database_path = std::env::var("KEYLESSPASS_ADMIN_DB")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("./keylesspass-admin.sqlite3"));
        let issuer = std::env::var("KEYLESSPASS_LICENSE_ISSUER")
            .unwrap_or_else(|_| "KeyLessPass Commercial Admin".to_string());
        Ok(Self {
            bind,
            admin_token,
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
    require_token(&headers, &state)?;
    Ok(Json(admin_status(&state).map_err(internal_error)?))
}

async fn api_snapshot(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<AdminSnapshot>> {
    require_token(&headers, &state)?;
    Ok(Json(snapshot(&state).map_err(internal_error)?))
}

async fn list_organizations(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<Vec<OrganizationRecord>>> {
    require_token(&headers, &state)?;
    Ok(Json(state.db.organizations().map_err(internal_error)?))
}

async fn create_organization(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CreateOrganizationRequest>,
) -> ApiResult<Json<OrganizationRecord>> {
    require_token(&headers, &state)?;
    Ok(Json(
        state
            .db
            .create_organization(request, &state.config.issuer)
            .map_err(bad_request)?,
    ))
}

async fn import_device_request(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<ImportDeviceRequestBody>,
) -> ApiResult<Json<DeviceRecord>> {
    require_token(&headers, &state)?;
    Ok(Json(
        state
            .db
            .import_device_request(request)
            .map_err(bad_request)?,
    ))
}

async fn list_devices(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<DevicesQuery>,
) -> ApiResult<Json<Vec<DeviceRecord>>> {
    require_token(&headers, &state)?;
    Ok(Json(
        state
            .db
            .devices(query.organization_id.as_deref())
            .map_err(internal_error)?,
    ))
}

async fn issue_bundle(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<IssueBundleRequest>,
) -> ApiResult<Json<IssueBundleResponse>> {
    require_token(&headers, &state)?;
    let org = state
        .db
        .organization(&request.organization_id)
        .map_err(internal_error)?
        .ok_or_else(|| bad_request(anyhow!("organization does not exist")))?;
    let devices = state
        .db
        .selected_devices(&org.id, &request.device_ids)
        .map_err(bad_request)?;
    if devices.is_empty() {
        return Err(bad_request(anyhow!("at least one device is required")));
    }
    if devices.len() as u32 > org.max_seats {
        return Err(bad_request(anyhow!(
            "selected devices exceed organization maxSeats"
        )));
    }

    let valid_until = normalize_issue_valid_until(&request, &org)?;
    let revoked_grant_ids = if request.include_revocations.unwrap_or(true) {
        state
            .db
            .revoked_grant_ids(&org.id)
            .map_err(internal_error)?
    } else {
        Vec::new()
    };
    let payload = build_payload(&org, &devices, revoked_grant_ids, valid_until);
    let envelope = sign_payload(&state.signing, &payload).map_err(internal_error)?;
    let envelope_json =
        serde_json::to_string_pretty(&envelope).map_err(|error| internal_error(anyhow!(error)))?;
    let bundle = bundle_record_from_envelope(&org, &payload, envelope_json.clone());
    state
        .db
        .store_bundle(&bundle, &payload.device_grants, &devices)
        .map_err(internal_error)?;
    Ok(Json(IssueBundleResponse {
        bundle,
        envelope_json,
    }))
}

async fn revoke_grant(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath(grant_id): AxumPath<String>,
) -> ApiResult<Json<ApiMessage>> {
    require_token(&headers, &state)?;
    state.db.revoke_grant(&grant_id).map_err(bad_request)?;
    Ok(Json(ApiMessage {
        message: "grant revoked".to_string(),
    }))
}

fn require_token(headers: &HeaderMap, state: &AppState) -> ApiResult<()> {
    let expected = format!("Bearer {}", state.config.admin_token);
    let actual = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    if actual == expected {
        Ok(())
    } else {
        Err((
            StatusCode::UNAUTHORIZED,
            Json(ApiError {
                error: "missing or invalid admin token".to_string(),
            }),
        ))
    }
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
    })
}

fn normalize_issue_valid_until(
    request: &IssueBundleRequest,
    org: &OrganizationRecord,
) -> ApiResult<String> {
    if let Some(value) = request
        .valid_until
        .as_ref()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    {
        DateTime::parse_from_rfc3339(&value).map_err(|_| {
            bad_request(anyhow!(
                "validUntil must be RFC3339, for example 2027-07-21T00:00:00Z"
            ))
        })?;
        return Ok(value);
    }
    if let Some(days) = request.valid_days {
        return Ok((Utc::now() + Duration::days(days.max(1))).to_rfc3339());
    }
    Ok(org.valid_until.clone())
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
