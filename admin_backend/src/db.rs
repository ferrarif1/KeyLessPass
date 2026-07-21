use crate::model::{
    BundleRecord, CreateOrganizationRequest, DeviceAuthorizationRequest, DeviceRecord, GrantRecord,
    ImportDeviceRequestBody, OrganizationRecord, LICENSE_SCHEMA_VERSION,
};
use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Duration, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use uuid::Uuid;

#[derive(Clone)]
pub struct Db {
    path: PathBuf,
    connection: Arc<Mutex<Connection>>,
}

impl Db {
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create database directory {}", parent.display()))?;
        }
        let connection = Connection::open(&path)
            .with_context(|| format!("open admin database {}", path.display()))?;
        let db = Self {
            path,
            connection: Arc::new(Mutex::new(connection)),
        };
        db.init()?;
        Ok(db)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn init(&self) -> Result<()> {
        let conn = self.lock()?;
        conn.execute_batch(
            r#"
            PRAGMA foreign_keys = ON;

            CREATE TABLE IF NOT EXISTS organizations (
                id TEXT PRIMARY KEY,
                license_id TEXT NOT NULL UNIQUE,
                name TEXT NOT NULL,
                plan TEXT NOT NULL,
                max_seats INTEGER NOT NULL,
                valid_from TEXT NOT NULL,
                valid_until TEXT NOT NULL,
                features_json TEXT NOT NULL,
                offline_grace_days INTEGER NOT NULL,
                allowed_major_versions_json TEXT NOT NULL,
                issuer TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS devices (
                id TEXT PRIMARY KEY,
                organization_id TEXT NOT NULL,
                request_id TEXT NOT NULL,
                commercial_device_id TEXT NOT NULL,
                device_fingerprint TEXT NOT NULL,
                platform TEXT NOT NULL,
                app_version TEXT NOT NULL,
                build_channel TEXT NOT NULL,
                seat_label TEXT NOT NULL,
                request_json TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                UNIQUE(commercial_device_id, device_fingerprint),
                FOREIGN KEY(organization_id) REFERENCES organizations(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS bundles (
                id TEXT PRIMARY KEY,
                bundle_id TEXT NOT NULL UNIQUE,
                organization_id TEXT NOT NULL,
                license_id TEXT NOT NULL,
                device_count INTEGER NOT NULL,
                revoked_count INTEGER NOT NULL,
                valid_until TEXT NOT NULL,
                issued_at TEXT NOT NULL,
                envelope_json TEXT NOT NULL,
                FOREIGN KEY(organization_id) REFERENCES organizations(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS grants (
                id TEXT PRIMARY KEY,
                grant_id TEXT NOT NULL UNIQUE,
                bundle_id TEXT NOT NULL,
                organization_id TEXT NOT NULL,
                device_id TEXT NOT NULL,
                commercial_device_id TEXT NOT NULL,
                seat_label TEXT NOT NULL,
                valid_until TEXT NOT NULL,
                revoked INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                revoked_at TEXT,
                FOREIGN KEY(organization_id) REFERENCES organizations(id) ON DELETE CASCADE,
                FOREIGN KEY(device_id) REFERENCES devices(id) ON DELETE CASCADE
            );
            "#,
        )?;
        Ok(())
    }

    pub fn create_organization(
        &self,
        request: CreateOrganizationRequest,
        default_issuer: &str,
    ) -> Result<OrganizationRecord> {
        let name = request.name.trim();
        if name.is_empty() {
            return Err(anyhow!("organization name is required"));
        }
        let now = Utc::now();
        let now_text = now.to_rfc3339();
        let id = request
            .organization_id
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| format!("org-{}", Uuid::new_v4()));
        let record = OrganizationRecord {
            id: id.clone(),
            license_id: format!("lic-{}", Uuid::new_v4()),
            name: name.to_string(),
            plan: request
                .plan
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| "enterprise".to_string()),
            max_seats: request.max_seats.unwrap_or(25).max(1),
            valid_from: now_text.clone(),
            valid_until: normalize_valid_until(request.valid_until, request.valid_days, now)?,
            features: default_features(request.features),
            offline_grace_days: request.offline_grace_days.unwrap_or(14),
            allowed_major_versions: if request.allowed_major_versions.is_empty() {
                vec![1]
            } else {
                request.allowed_major_versions
            },
            issuer: default_issuer.to_string(),
            created_at: now_text.clone(),
            updated_at: now_text,
        };

        let conn = self.lock()?;
        conn.execute(
            r#"
            INSERT INTO organizations (
                id, license_id, name, plan, max_seats, valid_from, valid_until,
                features_json, offline_grace_days, allowed_major_versions_json,
                issuer, created_at, updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            "#,
            params![
                record.id,
                record.license_id,
                record.name,
                record.plan,
                record.max_seats,
                record.valid_from,
                record.valid_until,
                to_json(&record.features)?,
                record.offline_grace_days,
                to_json(&record.allowed_major_versions)?,
                record.issuer,
                record.created_at,
                record.updated_at,
            ],
        )?;
        drop(conn);
        self.organization(&id)?
            .ok_or_else(|| anyhow!("created organization could not be loaded"))
    }

    pub fn organization(&self, id: &str) -> Result<Option<OrganizationRecord>> {
        let conn = self.lock()?;
        let row = conn
            .query_row(
                r#"
                SELECT id, license_id, name, plan, max_seats, valid_from, valid_until,
                       features_json, offline_grace_days, allowed_major_versions_json,
                       issuer, created_at, updated_at
                FROM organizations
                WHERE id = ?1
                "#,
                params![id],
                org_from_row,
            )
            .optional()?;
        Ok(row)
    }

    pub fn organizations(&self) -> Result<Vec<OrganizationRecord>> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            r#"
            SELECT id, license_id, name, plan, max_seats, valid_from, valid_until,
                   features_json, offline_grace_days, allowed_major_versions_json,
                   issuer, created_at, updated_at
            FROM organizations
            ORDER BY created_at DESC
            "#,
        )?;
        let rows = stmt.query_map([], org_from_row)?;
        let values = collect_rows(rows)?;
        Ok(values)
    }

    pub fn import_device_request(&self, body: ImportDeviceRequestBody) -> Result<DeviceRecord> {
        ensure_request_has_no_password_secrets(&body.request_json)?;
        let request: DeviceAuthorizationRequest = serde_json::from_str(body.request_json.trim())
            .context("device authorization request JSON is invalid")?;
        if request.schema_version != LICENSE_SCHEMA_VERSION {
            return Err(anyhow!("unsupported device authorization request schema"));
        }
        let organization_id = body
            .organization_id
            .or(request.organization_id.clone())
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .ok_or_else(|| anyhow!("organizationId is required"))?;
        if self.organization(&organization_id)?.is_none() {
            return Err(anyhow!("organization does not exist: {organization_id}"));
        }
        let seat_label = body
            .seat_label
            .or(request.seat_label.clone())
            .unwrap_or_else(|| request.commercial_device_id.chars().take(12).collect());
        let now = Utc::now().to_rfc3339();
        let row_id = self
            .device_by_identity(&request.commercial_device_id, &request.device_fingerprint)?
            .map(|device| device.id)
            .unwrap_or_else(|| format!("dev-{}", Uuid::new_v4()));

        let conn = self.lock()?;
        conn.execute(
            r#"
            INSERT INTO devices (
                id, organization_id, request_id, commercial_device_id, device_fingerprint,
                platform, app_version, build_channel, seat_label, request_json, created_at, updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
            ON CONFLICT(commercial_device_id, device_fingerprint)
            DO UPDATE SET
                organization_id = excluded.organization_id,
                request_id = excluded.request_id,
                platform = excluded.platform,
                app_version = excluded.app_version,
                build_channel = excluded.build_channel,
                seat_label = excluded.seat_label,
                request_json = excluded.request_json,
                updated_at = excluded.updated_at
            "#,
            params![
                row_id,
                organization_id,
                request.request_id,
                request.commercial_device_id,
                request.device_fingerprint,
                request.platform,
                request.app_version,
                request.build_channel,
                seat_label,
                body.request_json,
                now,
                now,
            ],
        )?;
        drop(conn);
        self.device_by_identity(&request.commercial_device_id, &request.device_fingerprint)?
            .ok_or_else(|| anyhow!("imported device could not be loaded"))
    }

    pub fn devices(&self, organization_id: Option<&str>) -> Result<Vec<DeviceRecord>> {
        let conn = self.lock()?;
        if let Some(organization_id) = organization_id {
            let mut stmt = conn.prepare(
                r#"
                SELECT id, organization_id, request_id, commercial_device_id, device_fingerprint,
                       platform, app_version, build_channel, seat_label, created_at, updated_at
                FROM devices
                WHERE organization_id = ?1
                ORDER BY updated_at DESC
                "#,
            )?;
            let rows = stmt.query_map(params![organization_id], device_from_row)?;
            let values = collect_rows(rows)?;
            Ok(values)
        } else {
            let mut stmt = conn.prepare(
                r#"
                SELECT id, organization_id, request_id, commercial_device_id, device_fingerprint,
                       platform, app_version, build_channel, seat_label, created_at, updated_at
                FROM devices
                ORDER BY updated_at DESC
                "#,
            )?;
            let rows = stmt.query_map([], device_from_row)?;
            let values = collect_rows(rows)?;
            Ok(values)
        }
    }

    pub fn selected_devices(
        &self,
        organization_id: &str,
        device_ids: &[String],
    ) -> Result<Vec<DeviceRecord>> {
        if device_ids.is_empty() {
            return self.devices(Some(organization_id));
        }
        let mut devices = Vec::with_capacity(device_ids.len());
        for id in device_ids {
            let Some(device) = self.device(id)? else {
                return Err(anyhow!("device does not exist: {id}"));
            };
            if device.organization_id != organization_id {
                return Err(anyhow!(
                    "device {id} does not belong to organization {organization_id}"
                ));
            }
            devices.push(device);
        }
        Ok(devices)
    }

    pub fn store_bundle(
        &self,
        bundle: &BundleRecord,
        grants: &[crate::model::DeviceGrant],
        devices: &[DeviceRecord],
    ) -> Result<()> {
        let conn = self.lock()?;
        conn.execute(
            r#"
            INSERT INTO bundles (
                id, bundle_id, organization_id, license_id, device_count,
                revoked_count, valid_until, issued_at, envelope_json
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
            params![
                bundle.id,
                bundle.bundle_id,
                bundle.organization_id,
                bundle.license_id,
                bundle.device_count,
                bundle.revoked_count,
                bundle.valid_until,
                bundle.issued_at,
                bundle.envelope_json,
            ],
        )?;

        for (grant, device) in grants.iter().zip(devices.iter()) {
            conn.execute(
                r#"
                INSERT INTO grants (
                    id, grant_id, bundle_id, organization_id, device_id,
                    commercial_device_id, seat_label, valid_until, revoked, created_at, revoked_at
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0, ?9, NULL)
                "#,
                params![
                    format!("grant-row-{}", Uuid::new_v4()),
                    grant.grant_id,
                    bundle.bundle_id,
                    bundle.organization_id,
                    device.id,
                    device.commercial_device_id,
                    device.seat_label,
                    grant.valid_until,
                    grant.issued_at,
                ],
            )?;
        }
        Ok(())
    }

    pub fn bundles(&self) -> Result<Vec<BundleRecord>> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            r#"
            SELECT id, bundle_id, organization_id, license_id, device_count,
                   revoked_count, valid_until, issued_at, envelope_json
            FROM bundles
            ORDER BY issued_at DESC
            "#,
        )?;
        let rows = stmt.query_map([], bundle_from_row)?;
        let values = collect_rows(rows)?;
        Ok(values)
    }

    pub fn grants(&self) -> Result<Vec<GrantRecord>> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            r#"
            SELECT g.id, g.grant_id, g.bundle_id, g.organization_id, g.device_id,
                   g.commercial_device_id, g.seat_label, g.valid_until, g.revoked,
                   g.created_at, g.revoked_at
            FROM grants g
            ORDER BY g.created_at DESC
            "#,
        )?;
        let rows = stmt.query_map([], grant_from_row)?;
        let values = collect_rows(rows)?;
        Ok(values)
    }

    pub fn revoked_grant_ids(&self, organization_id: &str) -> Result<Vec<String>> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            r#"
            SELECT grant_id
            FROM grants
            WHERE organization_id = ?1 AND revoked = 1
            ORDER BY revoked_at DESC
            "#,
        )?;
        let rows = stmt.query_map(params![organization_id], |row| row.get(0))?;
        let values = collect_rows(rows)?;
        Ok(values)
    }

    pub fn revoke_grant(&self, grant_id: &str) -> Result<()> {
        let conn = self.lock()?;
        let changed = conn.execute(
            r#"
            UPDATE grants
            SET revoked = 1, revoked_at = ?2
            WHERE grant_id = ?1
            "#,
            params![grant_id, Utc::now().to_rfc3339()],
        )?;
        if changed == 0 {
            return Err(anyhow!("grant does not exist: {grant_id}"));
        }
        Ok(())
    }

    pub fn counts(&self) -> Result<(u32, u32, u32)> {
        let conn = self.lock()?;
        let organizations = count_table(&conn, "organizations")?;
        let devices = count_table(&conn, "devices")?;
        let bundles = count_table(&conn, "bundles")?;
        Ok((organizations, devices, bundles))
    }

    fn device(&self, id: &str) -> Result<Option<DeviceRecord>> {
        let conn = self.lock()?;
        conn.query_row(
            r#"
            SELECT id, organization_id, request_id, commercial_device_id, device_fingerprint,
                   platform, app_version, build_channel, seat_label, created_at, updated_at
            FROM devices
            WHERE id = ?1
            "#,
            params![id],
            device_from_row,
        )
        .optional()
        .map_err(Into::into)
    }

    fn device_by_identity(
        &self,
        commercial_device_id: &str,
        device_fingerprint: &str,
    ) -> Result<Option<DeviceRecord>> {
        let conn = self.lock()?;
        conn.query_row(
            r#"
            SELECT id, organization_id, request_id, commercial_device_id, device_fingerprint,
                   platform, app_version, build_channel, seat_label, created_at, updated_at
            FROM devices
            WHERE commercial_device_id = ?1 AND device_fingerprint = ?2
            "#,
            params![commercial_device_id, device_fingerprint],
            device_from_row,
        )
        .optional()
        .map_err(Into::into)
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Connection>> {
        self.connection
            .lock()
            .map_err(|_| anyhow!("database lock poisoned"))
    }
}

fn normalize_valid_until(
    value: Option<String>,
    valid_days: Option<i64>,
    now: DateTime<Utc>,
) -> Result<String> {
    if let Some(value) = value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    {
        DateTime::parse_from_rfc3339(&value)
            .with_context(|| format!("validUntil must be RFC3339, got {value}"))?;
        return Ok(value);
    }
    Ok((now + Duration::days(valid_days.unwrap_or(365).max(1))).to_rfc3339())
}

fn default_features(features: Vec<String>) -> Vec<String> {
    let mut values: Vec<String> = features
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect();
    if values.is_empty() {
        values.push("desktop-client".to_string());
    }
    if !values.iter().any(|value| value == "desktop-client") {
        values.push("desktop-client".to_string());
    }
    values.sort();
    values.dedup();
    values
}

fn ensure_request_has_no_password_secrets(value: &str) -> Result<()> {
    let lowered = value.to_ascii_lowercase();
    for marker in [
        "mnemonic",
        "kmaster",
        "k_master",
        "devicesecret",
        "device_secret",
        "usbsecret",
        "usb_secret",
        "servicepassword",
        "derivedpassword",
    ] {
        if lowered.contains(marker) {
            return Err(anyhow!(
                "device authorization request must not contain password factor secrets"
            ));
        }
    }
    Ok(())
}

fn to_json<T: Serialize>(value: &T) -> Result<String> {
    Ok(serde_json::to_string(value)?)
}

fn from_json<T: DeserializeOwned + Default>(value: String) -> T {
    serde_json::from_str(&value).unwrap_or_default()
}

fn org_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<OrganizationRecord> {
    let features_json: String = row.get(7)?;
    let allowed_json: String = row.get(9)?;
    Ok(OrganizationRecord {
        id: row.get(0)?,
        license_id: row.get(1)?,
        name: row.get(2)?,
        plan: row.get(3)?,
        max_seats: row.get::<_, i64>(4)? as u32,
        valid_from: row.get(5)?,
        valid_until: row.get(6)?,
        features: from_json(features_json),
        offline_grace_days: row.get::<_, i64>(8)? as u32,
        allowed_major_versions: from_json(allowed_json),
        issuer: row.get(10)?,
        created_at: row.get(11)?,
        updated_at: row.get(12)?,
    })
}

fn device_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<DeviceRecord> {
    Ok(DeviceRecord {
        id: row.get(0)?,
        organization_id: row.get(1)?,
        request_id: row.get(2)?,
        commercial_device_id: row.get(3)?,
        device_fingerprint: row.get(4)?,
        platform: row.get(5)?,
        app_version: row.get(6)?,
        build_channel: row.get(7)?,
        seat_label: row.get(8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
    })
}

fn bundle_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<BundleRecord> {
    Ok(BundleRecord {
        id: row.get(0)?,
        bundle_id: row.get(1)?,
        organization_id: row.get(2)?,
        license_id: row.get(3)?,
        device_count: row.get::<_, i64>(4)? as u32,
        revoked_count: row.get::<_, i64>(5)? as u32,
        valid_until: row.get(6)?,
        issued_at: row.get(7)?,
        envelope_json: row.get(8)?,
    })
}

fn grant_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<GrantRecord> {
    Ok(GrantRecord {
        id: row.get(0)?,
        grant_id: row.get(1)?,
        bundle_id: row.get(2)?,
        organization_id: row.get(3)?,
        device_id: row.get(4)?,
        commercial_device_id: row.get(5)?,
        seat_label: row.get(6)?,
        valid_until: row.get(7)?,
        revoked: row.get::<_, i64>(8)? != 0,
        created_at: row.get(9)?,
        revoked_at: row.get(10)?,
    })
}

fn count_table(conn: &Connection, table: &str) -> Result<u32> {
    let sql = format!("SELECT COUNT(*) FROM {table}");
    let count: i64 = conn.query_row(&sql, [], |row| row.get(0))?;
    Ok(count as u32)
}

fn collect_rows<T>(
    rows: rusqlite::MappedRows<'_, impl FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<T>>,
) -> Result<Vec<T>> {
    let mut values = Vec::new();
    for row in rows {
        values.push(row?);
    }
    Ok(values)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn organization_and_device_request_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let db = Db::open(dir.path().join("admin.sqlite3")).unwrap();
        let org = db
            .create_organization(
                CreateOrganizationRequest {
                    organization_id: Some("org-acme".to_string()),
                    name: "Acme".to_string(),
                    plan: None,
                    max_seats: Some(2),
                    valid_days: Some(30),
                    valid_until: None,
                    features: vec![],
                    offline_grace_days: None,
                    allowed_major_versions: vec![],
                },
                "test",
            )
            .unwrap();
        let request = DeviceAuthorizationRequest {
            schema_version: LICENSE_SCHEMA_VERSION,
            request_id: "req-1".to_string(),
            organization_id: Some(org.id.clone()),
            commercial_device_id: "commercial-device".to_string(),
            device_fingerprint: "fingerprint".to_string(),
            platform: "macos".to_string(),
            app_version: "0.1.0".to_string(),
            build_channel: "desktop".to_string(),
            seat_label: Some("Finance laptop".to_string()),
            created_at: Utc::now().to_rfc3339(),
        };
        let device = db
            .import_device_request(ImportDeviceRequestBody {
                request_json: serde_json::to_string(&request).unwrap(),
                organization_id: None,
                seat_label: None,
            })
            .unwrap();
        assert_eq!(device.organization_id, "org-acme");
        assert_eq!(device.seat_label, "Finance laptop");
        assert_eq!(db.devices(Some("org-acme")).unwrap().len(), 1);
    }
}
