mod db;
mod license;
mod model;
mod ui;

use crate::{
    db::Db,
    license::{
        build_payload, bundle_record_from_envelope, generate_key_output,
        issue_customer_entitlement_output, issue_release_manifest_output,
        load_customer_entitlement, sign_payload, site_public_key_output,
        verify_customer_entitlement_for_site, verify_release_manifest_file,
        ReleaseArtifactManifest, SigningMaterial, VerifiedCustomerEntitlement,
    },
    model::{
        ActivateLicenseRequest, AdminSnapshot, AdminStatus, ApiMessage, AutomaticActivationRequest,
        AutomaticActivationResponse, BulkImportResult, BundleRecord, CreateOrganizationRequest,
        DeviceAuthorizationRequest, DeviceRecord, ImportDeviceRequestBody, IssueBundleRequest,
        OrganizationRecord,
    },
};
use anyhow::{anyhow, Context, Result};
use axum::{
    body::Body,
    extract::{ConnectInfo, DefaultBodyLimit, Path as AxumPath, Query, State},
    http::{
        header::{AUTHORIZATION, CONTENT_DISPOSITION, CONTENT_TYPE, HOST},
        HeaderMap, HeaderValue, StatusCode,
    },
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::Read;
use std::{
    collections::{HashMap, HashSet},
    net::{IpAddr, SocketAddr},
    path::PathBuf,
    sync::Arc,
    time::Instant,
};
use tokio::sync::Mutex;
use tokio_util::io::ReaderStream;
use tower_http::trace::TraceLayer;

#[derive(Clone)]
struct AppState {
    db: Db,
    signing: SigningMaterial,
    config: Config,
    issuance_lock: Arc<Mutex<()>>,
    customer_entitlement: VerifiedCustomerEntitlement,
    automatic_organization_id: String,
    release_manifest: Vec<ReleaseArtifactManifest>,
    automatic_rate_limits: Arc<Mutex<HashMap<IpAddr, (Instant, u32)>>>,
}

#[derive(Clone)]
struct Config {
    bind: SocketAddr,
    users: Vec<AdminUser>,
    issuer: String,
    database_path: PathBuf,
    download_dir: PathBuf,
    discovery_bind: SocketAddr,
    public_port: u16,
    public_base_url: Option<String>,
    customer_entitlement_file: Option<PathBuf>,
    max_automatic_registrations: Option<u32>,
    automatic_lease_hours: u32,
    automatic_grace_days: u32,
    release_manifest_file: PathBuf,
    automatic_requests_per_minute: u32,
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

const DISCOVERY_REQUEST: &[u8] = b"KEYLESSPASS_DISCOVER_V2";
const DISCOVERY_RESPONSE_PREFIX: &str = "KEYLESSPASS_SERVER_V2:";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DownloadArtifact {
    file_name: String,
    size_bytes: u64,
    download_url: String,
    sha256: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ClientConfigDocument {
    schema_version: u32,
    server_url: String,
    automatic_activation_path: &'static str,
    customer_id: String,
    entitlement_serial: u64,
    site_key_id: String,
    note: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OfflineApprovalRequestDocument {
    schema_version: u32,
    customer_id: String,
    customer_name: String,
    entitlement_serial: u64,
    site_key_id: String,
    site_public_key: String,
    purchased_device_limit: u32,
    current_customer_entitlement: crate::model::SignedCustomerEntitlementEnvelope,
    currently_approved_device_key_ids: Vec<String>,
    requested_devices: Vec<OfflineApprovalDevice>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OfflineApprovalDevice {
    device_key_id: String,
    device_public_key: String,
    commercial_device_id: String,
    device_fingerprint: String,
    platform: String,
    app_version: String,
    seat_label: String,
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
        Some("issue-release-manifest") => {
            print!("{}", issue_release_manifest_output()?);
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
    let release_manifest = load_release_manifest(&config)?;
    let automatic_organization_id = ensure_automatic_organization(
        &db,
        &customer_entitlement,
        &config.issuer,
        config.automatic_grace_days,
    )?;
    let state = AppState {
        db,
        signing,
        config,
        issuance_lock: Arc::new(Mutex::new(())),
        customer_entitlement,
        automatic_organization_id,
        release_manifest,
        automatic_rate_limits: Arc::new(Mutex::new(HashMap::new())),
    };

    let app = Router::new()
        .route("/", get(index))
        .route("/download", get(download_page))
        .route("/api/downloads", get(list_downloads))
        .route("/keylesspass-client-config.json", get(client_config))
        .route("/downloads/:file_name", get(download_file))
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
        .route("/api/automatic/activate", post(automatic_activate_license))
        .route(
            "/api/offline-approval/request",
            get(export_offline_approval_request),
        )
        .route(
            "/api/offline-approval/import",
            post(import_offline_approval),
        )
        .route("/api/grants/:grant_id/revoke", post(revoke_grant))
        .route("/api/audit.csv", get(export_audit_csv))
        .layer(DefaultBodyLimit::max(1024 * 1024))
        .layer(TraceLayer::new_for_http())
        .with_state(state.clone());

    tokio::spawn(run_discovery_responder(
        state.config.discovery_bind,
        state.config.public_port,
    ));
    tracing::info!(
        "KeyLessPass admin backend listening on http://{}",
        state.config.bind
    );
    let listener = tokio::net::TcpListener::bind(state.config.bind)
        .await
        .context("bind admin backend listener")?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .context("run admin backend")?;
    Ok(())
}

impl Config {
    fn from_env() -> Result<Self> {
        let bind: SocketAddr = std::env::var("KEYLESSPASS_ADMIN_BIND")
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
        let discovery_bind = std::env::var("KEYLESSPASS_DISCOVERY_BIND")
            .unwrap_or_else(|_| "0.0.0.0:8788".to_string())
            .parse()
            .context("KEYLESSPASS_DISCOVERY_BIND must be host:port")?;
        let public_port = std::env::var("KEYLESSPASS_PUBLIC_PORT")
            .ok()
            .map(|value| {
                value
                    .parse()
                    .context("KEYLESSPASS_PUBLIC_PORT must be a port")
            })
            .transpose()?
            .unwrap_or(bind.port());
        let public_base_url = std::env::var("KEYLESSPASS_PUBLIC_BASE_URL")
            .ok()
            .map(|value| value.trim().trim_end_matches('/').to_string())
            .filter(|value| !value.is_empty());
        if public_base_url.as_ref().is_some_and(|value| {
            (!value.starts_with("http://") && !value.starts_with("https://"))
                || value.contains('\r')
                || value.contains('\n')
        }) {
            return Err(anyhow!(
                "KEYLESSPASS_PUBLIC_BASE_URL must be an HTTP(S) URL"
            ));
        }
        let customer_entitlement_file = std::env::var("KEYLESSPASS_CUSTOMER_ENTITLEMENT_FILE")
            .ok()
            .map(PathBuf::from);
        let max_automatic_registrations = std::env::var("KEYLESSPASS_MAX_AUTOMATIC_REGISTRATIONS")
            .ok()
            .map(|value| {
                value
                    .parse()
                    .context("KEYLESSPASS_MAX_AUTOMATIC_REGISTRATIONS must be an integer")
            })
            .transpose()?;
        let automatic_lease_hours = env_u32("KEYLESSPASS_AUTOMATIC_LEASE_HOURS", 24)?;
        if !(1..=168).contains(&automatic_lease_hours) {
            return Err(anyhow!(
                "KEYLESSPASS_AUTOMATIC_LEASE_HOURS must be between 1 and 168"
            ));
        }
        let automatic_grace_days = env_u32("KEYLESSPASS_AUTOMATIC_GRACE_DAYS", 1)?;
        let release_manifest_file = std::env::var("KEYLESSPASS_RELEASE_MANIFEST_FILE")
            .map(PathBuf::from)
            .unwrap_or_else(|_| download_dir.join("release-manifest.json"));
        let automatic_requests_per_minute =
            env_u32("KEYLESSPASS_AUTOMATIC_REQUESTS_PER_MINUTE", 20)?.max(1);
        Ok(Self {
            bind,
            users,
            issuer,
            database_path,
            download_dir,
            discovery_bind,
            public_port,
            public_base_url,
            customer_entitlement_file,
            max_automatic_registrations,
            automatic_lease_hours,
            automatic_grace_days,
            release_manifest_file,
            automatic_requests_per_minute,
        })
    }
}

fn load_release_manifest(config: &Config) -> Result<Vec<ReleaseArtifactManifest>> {
    if config.release_manifest_file.is_file() {
        return verify_release_manifest_file(&config.release_manifest_file, &config.download_dir);
    }
    let contains_installer = std::fs::read_dir(&config.download_dir)
        .ok()
        .into_iter()
        .flatten()
        .flatten()
        .any(|entry| is_download_artifact(&entry.file_name().to_string_lossy()));
    if contains_installer {
        return Err(anyhow!(
            "signed release manifest is required at {}",
            config.release_manifest_file.display()
        ));
    }
    Ok(Vec::new())
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

fn ensure_automatic_organization(
    db: &Db,
    entitlement: &VerifiedCustomerEntitlement,
    issuer: &str,
    automatic_grace_days: u32,
) -> Result<String> {
    let digest = Sha256::digest(entitlement.payload.customer_id.as_bytes());
    let organization_id = format!("org-auto-{}", hex_prefix(&digest, 12));
    let max_seats = entitlement
        .payload
        .max_registered_devices
        .min(entitlement.payload.max_concurrent_devices);
    let request = CreateOrganizationRequest {
        organization_id: Some(organization_id.clone()),
        activation_code: None,
        name: entitlement.payload.customer_name.clone(),
        plan: Some("offline-intranet".to_string()),
        max_seats: Some(max_seats),
        valid_days: None,
        valid_until: Some(entitlement.payload.valid_until.clone()),
        features: entitlement.payload.features.clone(),
        offline_grace_days: Some(
            automatic_grace_days.min(entitlement.payload.max_offline_grace_days),
        ),
        allowed_major_versions: entitlement.payload.allowed_major_versions.clone(),
    };
    if db.organization(&organization_id)?.is_none() {
        db.create_organization(request.clone(), issuer)?;
    }
    db.update_automatic_organization(&organization_id, &request, issuer)?;
    Ok(organization_id)
}

fn hex_prefix(bytes: &[u8], count: usize) -> String {
    bytes
        .iter()
        .take((count + 1) / 2)
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>()
        .chars()
        .take(count)
        .collect()
}

async fn run_discovery_responder(bind: SocketAddr, public_port: u16) {
    let socket = match tokio::net::UdpSocket::bind(bind).await {
        Ok(socket) => socket,
        Err(error) => {
            tracing::error!(%error, %bind, "failed to bind intranet discovery responder");
            return;
        }
    };
    tracing::info!(%bind, "intranet discovery responder is ready");
    let response = format!("{DISCOVERY_RESPONSE_PREFIX}{public_port}");
    let mut buffer = [0_u8; 128];
    loop {
        match socket.recv_from(&mut buffer).await {
            Ok((length, peer)) if &buffer[..length] == DISCOVERY_REQUEST => {
                if let Err(error) = socket.send_to(response.as_bytes(), peer).await {
                    tracing::warn!(%error, %peer, "failed to answer discovery request");
                }
            }
            Ok(_) => {}
            Err(error) => tracing::warn!(%error, "failed to receive discovery request"),
        }
    }
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}

async fn index() -> Html<&'static str> {
    Html(ui::INDEX_HTML)
}

async fn list_downloads(State(state): State<AppState>) -> Json<Vec<DownloadArtifact>> {
    Json(download_artifacts(
        &state.config.download_dir,
        &state.release_manifest,
    ))
}

async fn client_config(State(state): State<AppState>, headers: HeaderMap) -> ApiResult<Response> {
    let server_url = match state.config.public_base_url.as_deref() {
        Some(value) => value.to_string(),
        None => {
            let host = headers
                .get(HOST)
                .and_then(|value| value.to_str().ok())
                .filter(|value| {
                    !value.is_empty()
                        && value.chars().all(|character| {
                            character.is_ascii_alphanumeric()
                                || matches!(character, '.' | '-' | ':' | '[' | ']')
                        })
                })
                .ok_or_else(|| {
                    api_error(
                        StatusCode::SERVICE_UNAVAILABLE,
                        "set KEYLESSPASS_PUBLIC_BASE_URL before downloading client config",
                    )
                })?;
            format!("http://{host}")
        }
    };
    let body = serde_json::to_string_pretty(&ClientConfigDocument {
        schema_version: 1,
        server_url,
        automatic_activation_path: "/api/automatic/activate",
        customer_id: state.customer_entitlement.payload.customer_id.clone(),
        entitlement_serial: state.customer_entitlement.payload.entitlement_serial,
        site_key_id: state.customer_entitlement.payload.site_key_id.clone(),
        note: "This file only locates the intranet server. Authorization is still verified by the vendor Ed25519 trust chain.",
    })
    .map_err(|error| internal_error(anyhow!(error)))?;
    let mut response = body.into_response();
    response.headers_mut().insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/json; charset=utf-8"),
    );
    response.headers_mut().insert(
        CONTENT_DISPOSITION,
        HeaderValue::from_static("attachment; filename=\"keylesspass-client-config.json\""),
    );
    response
        .headers_mut()
        .insert("cache-control", HeaderValue::from_static("no-store"));
    Ok(response)
}

async fn download_page(State(state): State<AppState>) -> Html<String> {
    let links = download_artifacts(&state.config.download_dir, &state.release_manifest)
        .into_iter()
        .map(|item| {
            format!(
                r#"<li><a href="{}" onclick="downloadServerConfig()">{}</a> <small>({} bytes)<br>SHA-256: {}</small></li>"#,
                item.download_url, item.file_name, item.size_bytes, item.sha256
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    Html(format!(
        r#"<!doctype html><html lang="zh-CN"><meta charset="utf-8"><meta name="viewport" content="width=device-width"><title>KeyLessPass 下载</title><style>body{{font:16px system-ui;max-width:760px;margin:64px auto;padding:0 20px;background:#111;color:#eee}}a{{color:#efff3d}}li{{margin:14px 0}}small{{color:#aaa}}</style><h1>KeyLessPass 应用下载</h1><p>无需登录。请只安装由供应商签名并公布校验值的正式版本。</p><ol><li>点击下方客户端时，会同时下载本服务器生成的 <code>keylesspass-client-config.json</code>。</li><li>安装并启动应用；客户端会直接读取下载目录中的服务器配置，不要求用户填写地址或激活码。</li><li>设备自动登记；厂商批量批准后自动完成授权。UDP 发现仅在配置文件丢失时兜底。</li></ol><p>如果浏览器阻止多个文件下载，请允许本内网站点下载，或<a href="/keylesspass-client-config.json">单独下载服务器配置</a>。</p><ul>{}</ul><p><a href="/">部署维护</a></p><script>function downloadServerConfig(){{const frame=document.createElement('iframe');frame.hidden=true;frame.src='/keylesspass-client-config.json?t='+Date.now();document.body.appendChild(frame);setTimeout(()=>frame.remove(),60000);}}</script></html>"#,
        if links.is_empty() {
            "<li>暂无可下载版本</li>".to_string()
        } else {
            links
        }
    ))
}

async fn download_file(
    State(state): State<AppState>,
    AxumPath(file_name): AxumPath<String>,
) -> ApiResult<Response> {
    let expected = state
        .release_manifest
        .iter()
        .find(|item| item.file_name == file_name)
        .ok_or_else(|| api_error(StatusCode::NOT_FOUND, "release artifact is not listed"))?;
    let path = state.config.download_dir.join(&expected.file_name);
    let metadata = path.metadata().map_err(|_| {
        api_error(
            StatusCode::NOT_FOUND,
            "release artifact is missing from the server",
        )
    })?;
    if !metadata.is_file()
        || metadata.len() != expected.size_bytes
        || sha256_file(&path).as_deref() != Some(expected.sha256.as_str())
    {
        return Err(api_error(
            StatusCode::CONFLICT,
            "release artifact no longer matches the vendor-signed manifest",
        ));
    }
    let file = tokio::fs::File::open(&path)
        .await
        .map_err(|error| internal_error(anyhow!(error)))?;
    let mut response = Body::from_stream(ReaderStream::new(file)).into_response();
    response.headers_mut().insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/octet-stream"),
    );
    response.headers_mut().insert(
        CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("attachment; filename=\"{}\"", expected.file_name))
            .map_err(|error| internal_error(anyhow!(error)))?,
    );
    Ok(response)
}

fn download_artifacts(
    directory: &std::path::Path,
    manifest: &[ReleaseArtifactManifest],
) -> Vec<DownloadArtifact> {
    manifest
        .iter()
        .filter_map(|expected| {
            let path = directory.join(&expected.file_name);
            let metadata = path.metadata().ok()?;
            let sha256 = sha256_file(&path)?;
            if !metadata.is_file()
                || metadata.len() != expected.size_bytes
                || sha256 != expected.sha256
            {
                return None;
            }
            Some(DownloadArtifact {
                download_url: format!("/downloads/{}", expected.file_name),
                file_name: expected.file_name.clone(),
                size_bytes: metadata.len(),
                sha256,
            })
        })
        .collect()
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

async fn export_offline_approval_request(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    let actor = require_role(&headers, &state, AdminRole::Auditor)?;
    let devices = state
        .db
        .devices(Some(&state.automatic_organization_id))
        .map_err(internal_error)?;
    let document = OfflineApprovalRequestDocument {
        schema_version: 1,
        customer_id: state.customer_entitlement.payload.customer_id.clone(),
        customer_name: state.customer_entitlement.payload.customer_name.clone(),
        entitlement_serial: state.customer_entitlement.payload.entitlement_serial,
        site_key_id: state.customer_entitlement.payload.site_key_id.clone(),
        site_public_key: state.customer_entitlement.payload.site_public_key.clone(),
        purchased_device_limit: state.customer_entitlement.payload.max_registered_devices,
        current_customer_entitlement: state.customer_entitlement.envelope.clone(),
        currently_approved_device_key_ids: state
            .customer_entitlement
            .payload
            .authorized_device_key_ids
            .clone(),
        requested_devices: devices
            .into_iter()
            .map(|device| OfflineApprovalDevice {
                device_key_id: device.device_key_id,
                device_public_key: device.device_public_key,
                commercial_device_id: device.commercial_device_id,
                device_fingerprint: device.device_fingerprint,
                platform: device.platform,
                app_version: device.app_version,
                seat_label: device.seat_label,
            })
            .collect(),
    };
    audit(
        &state,
        &actor,
        "offline_approval.export",
        &state.automatic_organization_id,
        "{}",
    )?;
    let body =
        serde_json::to_string_pretty(&document).map_err(|error| internal_error(anyhow!(error)))?;
    let mut response = body.into_response();
    response.headers_mut().insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/json; charset=utf-8"),
    );
    response.headers_mut().insert(
        CONTENT_DISPOSITION,
        HeaderValue::from_static(
            "attachment; filename=\"keylesspass-offline-approval-request.json\"",
        ),
    );
    Ok(response)
}

async fn import_offline_approval(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: String,
) -> ApiResult<(StatusCode, Json<ApiMessage>)> {
    let actor = require_role(&headers, &state, AdminRole::Admin)?;
    let path = state
        .config
        .customer_entitlement_file
        .as_ref()
        .ok_or_else(|| {
            api_error(
                StatusCode::SERVICE_UNAVAILABLE,
                "KEYLESSPASS_CUSTOMER_ENTITLEMENT_FILE is required for browser import",
            )
        })?;
    let verified =
        verify_customer_entitlement_for_site(body.trim(), &state.signing).map_err(bad_request)?;
    if verified.payload.customer_id != state.customer_entitlement.payload.customer_id {
        return Err(bad_request(anyhow!(
            "the imported entitlement belongs to another customer"
        )));
    }
    if verified.payload.entitlement_serial <= state.customer_entitlement.payload.entitlement_serial
    {
        return Err(bad_request(anyhow!(
            "entitlement serial must increase to prevent rollback"
        )));
    }
    let registered_keys: HashSet<String> = state
        .db
        .devices(Some(&state.automatic_organization_id))
        .map_err(internal_error)?
        .into_iter()
        .map(|device| device.device_key_id)
        .collect();
    if let Some(key_id) = verified
        .payload
        .authorized_device_key_ids
        .iter()
        .find(|key_id| !registered_keys.contains(*key_id))
    {
        return Err(bad_request(anyhow!(
            "vendor entitlement approves unknown device key {key_id}"
        )));
    }
    let temporary_path = path.with_extension(format!("json.tmp-{}", uuid::Uuid::new_v4()));
    std::fs::write(&temporary_path, body.trim())
        .with_context(|| format!("write temporary entitlement {}", temporary_path.display()))
        .map_err(internal_error)?;
    if let Err(error) = std::fs::rename(&temporary_path, path) {
        let _ = std::fs::remove_file(&temporary_path);
        return Err(internal_error(
            anyhow!(error).context(format!("replace entitlement file {}", path.display())),
        ));
    }
    audit(
        &state,
        &actor,
        "offline_approval.import",
        &verified.payload.entitlement_id,
        &format!(
            r#"{{"entitlementSerial":{},"approvedDevices":{}}}"#,
            verified.payload.entitlement_serial,
            verified.payload.authorized_device_key_ids.len()
        ),
    )?;
    tokio::spawn(async {
        tokio::time::sleep(std::time::Duration::from_millis(750)).await;
        std::process::exit(0);
    });
    Ok((
        StatusCode::ACCEPTED,
        Json(ApiMessage {
            message: "vendor approval installed; service is restarting".to_string(),
        }),
    ))
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
    validate_fresh_automatic_request(&parsed_request).map_err(bad_request)?;
    let existing_device = state
        .db
        .device_by_identity(
            &parsed_request.commercial_device_id,
            &parsed_request.device_fingerprint,
        )
        .map_err(internal_error)?;
    if let Some(existing) = existing_device.as_ref() {
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
    let existing_is_active = existing_device
        .as_ref()
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

async fn automatic_activate_license(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    Json(request): Json<AutomaticActivationRequest>,
) -> ApiResult<Response> {
    enforce_automatic_rate_limit(&state, peer.ip()).await?;
    let _issuance_guard = state.issuance_lock.lock().await;
    let parsed_request: DeviceAuthorizationRequest =
        serde_json::from_str(request.request_json.trim())
            .map_err(|_| bad_request(anyhow!("device authorization request JSON is invalid")))?;
    validate_fresh_automatic_request(&parsed_request).map_err(bad_request)?;
    let existing_device = state
        .db
        .device_by_identity(
            &parsed_request.commercial_device_id,
            &parsed_request.device_fingerprint,
        )
        .map_err(internal_error)?;
    if let Some(existing) = existing_device.as_ref() {
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
    if existing_device.is_none() {
        let registered = state
            .db
            .devices(Some(&state.automatic_organization_id))
            .map_err(internal_error)?
            .len();
        let default_limit = state
            .customer_entitlement
            .payload
            .max_registered_devices
            .saturating_mul(4)
            .max(32);
        let limit = state
            .config
            .max_automatic_registrations
            .unwrap_or(default_limit) as usize;
        if registered >= limit {
            return Err(api_error(
                StatusCode::TOO_MANY_REQUESTS,
                "automatic registration queue is full; contact customer IT",
            ));
        }
    }

    let device = state
        .db
        .import_device_request(ImportDeviceRequestBody {
            request_json: request.request_json,
            organization_id: Some(state.automatic_organization_id.clone()),
            seat_label: request.seat_label,
        })
        .map_err(bad_request)?;
    if existing_device.is_none() {
        state
            .db
            .record_audit(
                &format!("automatic:{}", state.automatic_organization_id),
                "client",
                "device.collect",
                &device.id,
                "{}",
            )
            .map_err(internal_error)?;
    }

    if !state
        .customer_entitlement
        .payload
        .authorized_device_key_ids
        .contains(&device.device_key_id)
    {
        return Ok((
            StatusCode::ACCEPTED,
            Json(AutomaticActivationResponse {
                status: "pendingVendorApproval".to_string(),
                device_key_id: device.device_key_id,
                envelope_json: None,
            }),
        )
            .into_response());
    }

    let org = state
        .db
        .organization(&state.automatic_organization_id)
        .map_err(internal_error)?
        .ok_or_else(|| {
            api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "automatic organization is missing",
            )
        })?;
    let active_ids: HashSet<String> = state
        .db
        .active_licensed_device_ids(&org.id)
        .map_err(internal_error)?
        .into_iter()
        .collect();
    if active_ids.contains(&device.id) {
        let bundle = state
            .db
            .latest_active_bundle_for_device(&org.id, &device.id)
            .map_err(internal_error)?
            .ok_or_else(|| {
                api_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "active device grant has no license bundle",
                )
            })?;
        if !automatic_bundle_needs_renewal(&bundle, state.config.automatic_lease_hours)
            .map_err(internal_error)?
        {
            return Ok(Json(AutomaticActivationResponse {
                status: "authorized".to_string(),
                device_key_id: device.device_key_id,
                envelope_json: Some(bundle.envelope_json),
            })
            .into_response());
        }
    }
    if active_ids.len() as u32 >= org.max_seats && !active_ids.contains(&device.id) {
        return Err(api_error(
            StatusCode::CONFLICT,
            "the purchased device quota has been reached",
        ));
    }
    let response = issue_bundle_inner(
        &state,
        IssueBundleRequest {
            organization_id: org.id.clone(),
            device_ids: vec![device.id.clone()],
            valid_days: None,
            valid_until: Some(
                (Utc::now() + Duration::hours(state.config.automatic_lease_hours as i64))
                    .min(
                        DateTime::parse_from_rfc3339(&org.valid_until)
                            .context("automatic organization validUntil is invalid")
                            .map_err(internal_error)?
                            .with_timezone(&Utc),
                    )
                    .to_rfc3339(),
            ),
            include_revocations: Some(true),
        },
    )
    .map_err(bad_request)?;
    state
        .db
        .record_audit(
            &format!("automatic:{}", org.id),
            "client",
            "license.activate.automatic",
            &device.id,
            "{}",
        )
        .map_err(internal_error)?;
    Ok(Json(AutomaticActivationResponse {
        status: "authorized".to_string(),
        device_key_id: device.device_key_id,
        envelope_json: Some(response.envelope_json),
    })
    .into_response())
}

async fn enforce_automatic_rate_limit(state: &AppState, address: IpAddr) -> ApiResult<()> {
    let now = Instant::now();
    let mut limits = state.automatic_rate_limits.lock().await;
    limits.retain(|_, (started, _)| now.duration_since(*started).as_secs() < 120);
    let entry = limits.entry(address).or_insert((now, 0));
    if now.duration_since(entry.0).as_secs() >= 60 {
        *entry = (now, 0);
    }
    if entry.1 >= state.config.automatic_requests_per_minute {
        return Err(api_error(
            StatusCode::TOO_MANY_REQUESTS,
            "automatic activation rate limit exceeded; retry later",
        ));
    }
    entry.1 += 1;
    Ok(())
}

fn validate_fresh_automatic_request(request: &DeviceAuthorizationRequest) -> Result<()> {
    let request_id = request
        .request_id
        .strip_prefix("req-")
        .unwrap_or(&request.request_id);
    uuid::Uuid::parse_str(request_id).context("automatic requestId must contain a UUID")?;
    let created_at = DateTime::parse_from_rfc3339(&request.created_at)
        .context("automatic request createdAt must be RFC3339")?
        .with_timezone(&Utc);
    let now = Utc::now();
    if created_at < now - Duration::minutes(15) || created_at > now + Duration::minutes(5) {
        return Err(anyhow!(
            "automatic device request is stale; check the client clock and retry"
        ));
    }
    Ok(())
}

fn automatic_bundle_needs_renewal(bundle: &BundleRecord, lease_hours: u32) -> Result<bool> {
    let issued_at = DateTime::parse_from_rfc3339(&bundle.issued_at)
        .context("stored automatic bundle issuedAt is invalid")?
        .with_timezone(&Utc);
    let valid_until = DateTime::parse_from_rfc3339(&bundle.valid_until)
        .context("stored automatic bundle validUntil is invalid")?
        .with_timezone(&Utc);
    let lease = Duration::hours(lease_hours as i64);
    let renewal_window = Duration::hours((lease_hours / 4).max(1) as i64);
    let is_bounded_lease = valid_until <= issued_at + lease + Duration::minutes(5);
    Ok(!is_bounded_lease || valid_until <= Utc::now() + renewal_window)
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

#[cfg(test)]
mod automatic_activation_tests {
    use super::*;
    use crate::model::{CustomerEntitlement, SignedCustomerEntitlementEnvelope};
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    use ed25519_dalek::{Signer, SigningKey};

    fn device_request(seed: u8) -> DeviceAuthorizationRequest {
        let key = SigningKey::from_bytes(&[seed; 32]);
        let public_key = key.verifying_key().to_bytes();
        let mut request = DeviceAuthorizationRequest {
            schema_version: 2,
            request_id: format!("req-{}", uuid::Uuid::new_v4()),
            organization_id: None,
            commercial_device_id: format!("commercial-{seed}"),
            device_fingerprint: format!("fingerprint-{seed}"),
            device_key_id: Sha256::digest(public_key)
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect(),
            device_public_key: URL_SAFE_NO_PAD.encode(public_key),
            device_proof: String::new(),
            platform: "macos".to_string(),
            app_version: "1.0.0".to_string(),
            build_channel: "commercial".to_string(),
            seat_label: None,
            created_at: Utc::now().to_rfc3339(),
        };
        request.device_proof =
            URL_SAFE_NO_PAD.encode(key.sign(&request.proof_message()).to_bytes());
        request
    }

    fn test_state(approved_device_key_ids: Vec<String>) -> (tempfile::TempDir, AppState) {
        let directory = tempfile::tempdir().unwrap();
        let db = Db::open(directory.path().join("admin.sqlite3")).unwrap();
        let signing = SigningMaterial::for_test("site-test", [9; 32]);
        let now = Utc::now();
        let customer_entitlement = VerifiedCustomerEntitlement {
            envelope: SignedCustomerEntitlementEnvelope {
                schema_version: 2,
                envelope_type: "keylesspass-customer-entitlement".to_string(),
                payload: "test".to_string(),
                signature_algorithm: "Ed25519".to_string(),
                key_id: "vendor-test".to_string(),
                signature: "test".to_string(),
            },
            payload: CustomerEntitlement {
                schema_version: 2,
                entitlement_id: "ent-test".to_string(),
                entitlement_serial: 1,
                customer_id: "customer-test".to_string(),
                customer_name: "Customer Test".to_string(),
                product: "KeyLessPass".to_string(),
                site_key_id: signing.key_id.clone(),
                site_public_key: signing.public_key_b64(),
                max_registered_devices: 2,
                max_concurrent_devices: 2,
                max_offline_borrowed: 0,
                max_offline_grace_days: 14,
                authorized_device_key_ids: approved_device_key_ids,
                valid_from: (now - Duration::days(1)).to_rfc3339(),
                valid_until: (now + Duration::days(30)).to_rfc3339(),
                features: vec![
                    "desktop-client".to_string(),
                    "channel:commercial".to_string(),
                ],
                allowed_major_versions: vec![1],
                issued_at: now.to_rfc3339(),
                issuer: "Vendor Test".to_string(),
            },
        };
        let automatic_organization_id =
            ensure_automatic_organization(&db, &customer_entitlement, "Site Test", 1).unwrap();
        let state = AppState {
            db,
            signing,
            config: Config {
                bind: "127.0.0.1:0".parse().unwrap(),
                users: vec![AdminUser {
                    name: "test".to_string(),
                    role: AdminRole::Admin,
                    token: "test-token-at-least-24-characters".to_string(),
                    organization_id: None,
                }],
                issuer: "Site Test".to_string(),
                database_path: directory.path().join("admin.sqlite3"),
                download_dir: directory.path().to_path_buf(),
                discovery_bind: "127.0.0.1:0".parse().unwrap(),
                public_port: 8787,
                public_base_url: None,
                customer_entitlement_file: None,
                max_automatic_registrations: None,
                automatic_lease_hours: 24,
                automatic_grace_days: 1,
                release_manifest_file: directory.path().join("release-manifest.json"),
                automatic_requests_per_minute: 20,
            },
            issuance_lock: Arc::new(Mutex::new(())),
            customer_entitlement,
            automatic_organization_id,
            release_manifest: Vec::new(),
            automatic_rate_limits: Arc::new(Mutex::new(HashMap::new())),
        };
        (directory, state)
    }

    #[tokio::test]
    async fn unapproved_automatic_device_is_collected_but_not_licensed() {
        let request = device_request(41);
        let (_directory, state) = test_state(Vec::new());
        let response = automatic_activate_license(
            State(state.clone()),
            ConnectInfo("127.0.0.1:41001".parse().unwrap()),
            Json(AutomaticActivationRequest {
                request_json: serde_json::to_string(&request).unwrap(),
                seat_label: None,
            }),
        )
        .await
        .unwrap();
        assert_eq!(response.status(), StatusCode::ACCEPTED);
        assert_eq!(state.db.devices(None).unwrap().len(), 1);
        assert_eq!(state.db.counts().unwrap().2, 0);
    }

    #[tokio::test]
    async fn stale_automatic_request_is_rejected_before_registration() {
        let mut request = device_request(43);
        request.created_at = (Utc::now() - Duration::minutes(16)).to_rfc3339();
        let key = SigningKey::from_bytes(&[43; 32]);
        request.device_proof =
            URL_SAFE_NO_PAD.encode(key.sign(&request.proof_message()).to_bytes());
        let (_directory, state) = test_state(Vec::new());
        let error = automatic_activate_license(
            State(state.clone()),
            ConnectInfo("127.0.0.1:41002".parse().unwrap()),
            Json(AutomaticActivationRequest {
                request_json: serde_json::to_string(&request).unwrap(),
                seat_label: None,
            }),
        )
        .await
        .unwrap_err();
        assert_eq!(error.0, StatusCode::BAD_REQUEST);
        assert!(state.db.devices(None).unwrap().is_empty());
    }

    #[test]
    fn long_lived_or_nearly_expired_automatic_bundles_are_renewed() {
        let now = Utc::now();
        let bundle = |issued_at: DateTime<Utc>, valid_until: DateTime<Utc>| BundleRecord {
            id: "row".to_string(),
            bundle_id: "bundle".to_string(),
            organization_id: "org".to_string(),
            license_id: "license".to_string(),
            device_count: 1,
            revoked_count: 0,
            valid_until: valid_until.to_rfc3339(),
            issued_at: issued_at.to_rfc3339(),
            envelope_json: "{}".to_string(),
        };
        assert!(automatic_bundle_needs_renewal(
            &bundle(now - Duration::hours(1), now + Duration::days(30)),
            24,
        )
        .unwrap());
        assert!(automatic_bundle_needs_renewal(
            &bundle(now - Duration::hours(18), now + Duration::hours(5)),
            24,
        )
        .unwrap());
        assert!(
            !automatic_bundle_needs_renewal(&bundle(now, now + Duration::hours(24)), 24,).unwrap()
        );
    }

    #[tokio::test]
    async fn automatic_activation_is_rate_limited_per_source_ip() {
        let (_directory, mut state) = test_state(Vec::new());
        state.config.automatic_requests_per_minute = 2;
        let address: IpAddr = "10.20.30.40".parse().unwrap();
        enforce_automatic_rate_limit(&state, address).await.unwrap();
        enforce_automatic_rate_limit(&state, address).await.unwrap();
        assert_eq!(
            enforce_automatic_rate_limit(&state, address)
                .await
                .unwrap_err()
                .0,
            StatusCode::TOO_MANY_REQUESTS
        );
        enforce_automatic_rate_limit(&state, "10.20.30.41".parse().unwrap())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn vendor_approved_automatic_device_receives_one_grant() {
        let request = device_request(42);
        let (_directory, state) = test_state(vec![request.device_key_id.clone()]);
        let response = automatic_activate_license(
            State(state.clone()),
            ConnectInfo("127.0.0.1:41003".parse().unwrap()),
            Json(AutomaticActivationRequest {
                request_json: serde_json::to_string(&request).unwrap(),
                seat_label: None,
            }),
        )
        .await
        .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(state.db.counts().unwrap().2, 1);
        assert_eq!(
            state
                .db
                .active_licensed_device_ids(&state.automatic_organization_id)
                .unwrap()
                .len(),
            1
        );
        let second = automatic_activate_license(
            State(state.clone()),
            ConnectInfo("127.0.0.1:41003".parse().unwrap()),
            Json(AutomaticActivationRequest {
                request_json: serde_json::to_string(&request).unwrap(),
                seat_label: None,
            }),
        )
        .await
        .unwrap();
        assert_eq!(second.status(), StatusCode::OK);
        assert_eq!(state.db.counts().unwrap().2, 1);
    }

    #[tokio::test]
    async fn udp_discovery_returns_the_configured_public_port() {
        let probe = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
        let bind = probe.local_addr().unwrap();
        drop(probe);
        let responder = tokio::spawn(run_discovery_responder(bind, 18787));
        tokio::time::sleep(std::time::Duration::from_millis(30)).await;
        let client = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
        client.send_to(DISCOVERY_REQUEST, bind).await.unwrap();
        let mut buffer = [0_u8; 64];
        let (length, _) = tokio::time::timeout(
            std::time::Duration::from_secs(1),
            client.recv_from(&mut buffer),
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(&buffer[..length], b"KEYLESSPASS_SERVER_V2:18787");
        responder.abort();
    }
}
