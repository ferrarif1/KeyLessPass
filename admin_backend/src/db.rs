use crate::model::{
    AuditRecord, BundleRecord, CreateOrganizationRequest, DeviceAuthorizationRequest, DeviceRecord,
    GrantRecord, ImportDeviceRequestBody, OrganizationRecord, LICENSE_SCHEMA_VERSION,
};
use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{DateTime, Duration, Utc};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde::{de::DeserializeOwned, Serialize};
use sha2::Digest;
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
                activation_code TEXT UNIQUE,
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
                device_key_id TEXT NOT NULL,
                device_public_key TEXT NOT NULL,
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

            CREATE TABLE IF NOT EXISTS seat_allocations (
                seat_id TEXT PRIMARY KEY,
                organization_id TEXT NOT NULL,
                device_id TEXT NOT NULL,
                status TEXT NOT NULL,
                allocated_at TEXT NOT NULL,
                expires_at TEXT NOT NULL,
                released_at TEXT,
                version INTEGER NOT NULL DEFAULT 1,
                UNIQUE(organization_id, device_id),
                FOREIGN KEY(organization_id) REFERENCES organizations(id) ON DELETE CASCADE,
                FOREIGN KEY(device_id) REFERENCES devices(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS audit_log (
                id TEXT PRIMARY KEY,
                actor TEXT NOT NULL,
                role TEXT NOT NULL,
                action TEXT NOT NULL,
                target TEXT NOT NULL,
                created_at TEXT NOT NULL,
                details_json TEXT NOT NULL
            );
            "#,
        )?;
        if !table_has_column(&conn, "organizations", "activation_code")? {
            conn.execute(
                "ALTER TABLE organizations ADD COLUMN activation_code TEXT",
                [],
            )?;
        }
        if !table_has_column(&conn, "devices", "device_key_id")? {
            conn.execute(
                "ALTER TABLE devices ADD COLUMN device_key_id TEXT NOT NULL DEFAULT ''",
                [],
            )?;
        }
        if !table_has_column(&conn, "devices", "device_public_key")? {
            conn.execute(
                "ALTER TABLE devices ADD COLUMN device_public_key TEXT NOT NULL DEFAULT ''",
                [],
            )?;
        }
        let mut stmt = conn.prepare(
            "SELECT id FROM organizations WHERE activation_code IS NULL OR activation_code = ''",
        )?;
        let ids = collect_rows(stmt.query_map([], |row| row.get::<_, String>(0))?)?;
        drop(stmt);
        for id in ids {
            conn.execute(
                "UPDATE organizations SET activation_code = ?2 WHERE id = ?1",
                params![id, new_activation_code()],
            )?;
        }
        conn.execute(
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_organizations_activation_code ON organizations(activation_code)",
            [],
        )?;
        conn.execute(
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_devices_device_key_id ON devices(device_key_id) WHERE device_key_id <> ''",
            [],
        )?;
        conn.execute(
            r#"
            UPDATE devices
            SET request_id = request_id || '-' || id
            WHERE rowid NOT IN (
                SELECT MIN(rowid) FROM devices GROUP BY request_id
            )
            "#,
            [],
        )?;
        conn.execute(
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_devices_request_id ON devices(request_id)",
            [],
        )?;
        let now = Utc::now().to_rfc3339();
        conn.execute(
            r#"
            INSERT OR IGNORE INTO seat_allocations (
                seat_id, organization_id, device_id, status, allocated_at,
                expires_at, released_at, version
            )
            SELECT
                'seat-migrated-' || device_id,
                organization_id,
                device_id,
                CASE
                    WHEN MAX(CASE WHEN revoked = 0 AND valid_until > ?1 THEN 1 ELSE 0 END) = 1
                        THEN 'active'
                    WHEN MAX(CASE WHEN revoked = 1 THEN 1 ELSE 0 END) = 1
                        THEN 'revoked'
                    ELSE 'expired'
                END,
                MIN(created_at),
                MAX(valid_until),
                CASE
                    WHEN MAX(CASE WHEN revoked = 0 AND valid_until > ?1 THEN 1 ELSE 0 END) = 1
                        THEN NULL
                    ELSE ?1
                END,
                1
            FROM grants
            GROUP BY organization_id, device_id
            "#,
            params![now],
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
        let activation_code = match request.activation_code {
            Some(value) if value.trim().len() < 16 => {
                return Err(anyhow!("activationCode must be at least 16 characters"));
            }
            Some(value) => value.trim().to_string(),
            None => new_activation_code(),
        };
        let record = OrganizationRecord {
            id: id.clone(),
            license_id: format!("lic-{}", Uuid::new_v4()),
            activation_code,
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
                id, license_id, activation_code, name, plan, max_seats, valid_from, valid_until,
                features_json, offline_grace_days, allowed_major_versions_json,
                issuer, created_at, updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
            "#,
            params![
                record.id,
                record.license_id,
                record.activation_code,
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
                       activation_code, features_json, offline_grace_days, allowed_major_versions_json,
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

    pub fn update_automatic_organization(
        &self,
        id: &str,
        request: &CreateOrganizationRequest,
        issuer: &str,
    ) -> Result<()> {
        let valid_until = request
            .valid_until
            .as_deref()
            .ok_or_else(|| anyhow!("automatic organization validUntil is required"))?;
        let changed = self.lock()?.execute(
            r#"
            UPDATE organizations
            SET name = ?2,
                plan = 'offline-intranet',
                max_seats = ?3,
                valid_until = ?4,
                features_json = ?5,
                offline_grace_days = ?6,
                allowed_major_versions_json = ?7,
                issuer = ?8,
                updated_at = ?9
            WHERE id = ?1
            "#,
            params![
                id,
                request.name,
                request.max_seats.unwrap_or(1).max(1),
                valid_until,
                to_json(&request.features)?,
                request.offline_grace_days.unwrap_or(0),
                to_json(&request.allowed_major_versions)?,
                issuer,
                Utc::now().to_rfc3339(),
            ],
        )?;
        if changed != 1 {
            return Err(anyhow!("automatic organization does not exist"));
        }
        Ok(())
    }

    pub fn organization_by_activation_code(
        &self,
        activation_code: &str,
    ) -> Result<Option<OrganizationRecord>> {
        let conn = self.lock()?;
        conn.query_row(
            r#"
            SELECT id, license_id, name, plan, max_seats, valid_from, valid_until,
                   activation_code, features_json, offline_grace_days, allowed_major_versions_json,
                   issuer, created_at, updated_at
            FROM organizations
            WHERE activation_code = ?1
            "#,
            params![activation_code.trim()],
            org_from_row,
        )
        .optional()
        .map_err(Into::into)
    }

    pub fn organizations(&self) -> Result<Vec<OrganizationRecord>> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            r#"
            SELECT id, license_id, name, plan, max_seats, valid_from, valid_until,
                   activation_code, features_json, offline_grace_days, allowed_major_versions_json,
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
        verify_device_request_proof(&request)?;
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
        let existing =
            self.device_by_identity(&request.commercial_device_id, &request.device_fingerprint)?;
        if existing
            .as_ref()
            .is_some_and(|device| device.organization_id != organization_id)
        {
            return Err(anyhow!(
                "device identity is already assigned to another organization"
            ));
        }
        if existing
            .as_ref()
            .is_some_and(|device| device.device_key_id != request.device_key_id)
        {
            return Err(anyhow!(
                "device identity key cannot be replaced; retire and reapprove the device"
            ));
        }
        {
            let conn = self.lock()?;
            let conflicting_identity: Option<String> = conn
                .query_row(
                    r#"
                    SELECT id FROM devices
                    WHERE device_key_id = ?1
                      AND NOT (commercial_device_id = ?2 AND device_fingerprint = ?3)
                    "#,
                    params![
                        request.device_key_id,
                        request.commercial_device_id,
                        request.device_fingerprint
                    ],
                    |row| row.get(0),
                )
                .optional()?;
            if conflicting_identity.is_some() {
                return Err(anyhow!(
                    "device identity key is already registered to another device"
                ));
            }
        }
        let row_id = existing
            .map(|device| device.id)
            .unwrap_or_else(|| format!("dev-{}", Uuid::new_v4()));

        let conn = self.lock()?;
        conn.execute(
            r#"
            INSERT INTO devices (
                id, organization_id, request_id, commercial_device_id, device_fingerprint,
                device_key_id, device_public_key, platform, app_version, build_channel,
                seat_label, request_json, created_at, updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
            ON CONFLICT(commercial_device_id, device_fingerprint)
            DO UPDATE SET
                organization_id = excluded.organization_id,
                request_id = excluded.request_id,
                device_key_id = excluded.device_key_id,
                device_public_key = excluded.device_public_key,
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
                request.device_key_id,
                request.device_public_key,
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
                       device_key_id, device_public_key, platform, app_version, build_channel,
                       seat_label, created_at, updated_at
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
                       device_key_id, device_public_key, platform, app_version, build_channel,
                       seat_label, created_at, updated_at
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
        let mut seen = std::collections::HashSet::new();
        for id in device_ids {
            if !seen.insert(id) {
                continue;
            }
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
        max_seats: u32,
    ) -> Result<()> {
        if grants.len() != devices.len() {
            return Err(anyhow!("grant and device counts do not match"));
        }
        let mut conn = self.lock()?;
        let transaction = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let now = Utc::now().to_rfc3339();
        transaction.execute(
            r#"
            UPDATE seat_allocations
            SET status = 'expired', released_at = ?1, version = version + 1
            WHERE organization_id = ?2 AND status = 'active' AND expires_at <= ?1
            "#,
            params![now, bundle.organization_id],
        )?;
        let active_count: i64 = transaction.query_row(
            "SELECT COUNT(*) FROM seat_allocations WHERE organization_id = ?1 AND status = 'active'",
            params![bundle.organization_id],
            |row| row.get(0),
        )?;
        let mut new_count = 0_i64;
        for device in devices {
            let status = transaction
                .query_row(
                    "SELECT status FROM seat_allocations WHERE organization_id = ?1 AND device_id = ?2",
                    params![bundle.organization_id, device.id],
                    |row| row.get::<_, String>(0),
                )
                .optional()?;
            if status.as_deref() != Some("active") {
                new_count += 1;
            }
        }
        if active_count + new_count > max_seats as i64 {
            return Err(anyhow!("issued devices exceed organization maxSeats"));
        }
        for device in devices {
            transaction.execute(
                r#"
                INSERT INTO seat_allocations (
                    seat_id, organization_id, device_id, status, allocated_at,
                    expires_at, released_at, version
                ) VALUES (?1, ?2, ?3, 'active', ?4, ?5, NULL, 1)
                ON CONFLICT(organization_id, device_id) DO UPDATE SET
                    status = 'active',
                    allocated_at = excluded.allocated_at,
                    expires_at = excluded.expires_at,
                    released_at = NULL,
                    version = seat_allocations.version + 1
                "#,
                params![
                    format!("seat-{}", Uuid::new_v4()),
                    bundle.organization_id,
                    device.id,
                    now,
                    bundle.valid_until,
                ],
            )?;
        }
        transaction.execute(
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
            transaction.execute(
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
        transaction.commit()?;
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
        let mut conn = self.lock()?;
        let transaction = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let device_id: Option<String> = transaction
            .query_row(
                "SELECT device_id FROM grants WHERE grant_id = ?1",
                params![grant_id],
                |row| row.get(0),
            )
            .optional()?;
        let Some(device_id) = device_id else {
            return Err(anyhow!("grant does not exist: {grant_id}"));
        };
        let now = Utc::now().to_rfc3339();
        let changed = transaction.execute(
            r#"
            UPDATE grants
            SET revoked = 1, revoked_at = ?2
            WHERE device_id = (SELECT device_id FROM grants WHERE grant_id = ?1)
              AND revoked = 0
            "#,
            params![grant_id, now],
        )?;
        if changed == 0 {
            return Err(anyhow!("grant does not exist: {grant_id}"));
        }
        transaction.execute(
            r#"
            UPDATE seat_allocations
            SET status = 'revoked', released_at = ?2, version = version + 1
            WHERE device_id = ?1 AND status = 'active'
            "#,
            params![device_id, now],
        )?;
        transaction.commit()?;
        Ok(())
    }

    pub fn device_has_revoked_grant(&self, device_id: &str) -> Result<bool> {
        let conn = self.lock()?;
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM grants WHERE device_id = ?1 AND revoked = 1",
            params![device_id],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    pub fn counts(&self) -> Result<(u32, u32, u32)> {
        let conn = self.lock()?;
        let organizations = count_table(&conn, "organizations")?;
        let devices = count_table(&conn, "devices")?;
        let bundles = count_table(&conn, "bundles")?;
        Ok((organizations, devices, bundles))
    }

    pub fn active_licensed_device_ids(&self, organization_id: &str) -> Result<Vec<String>> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            r#"
            SELECT device_id
            FROM seat_allocations
            WHERE organization_id = ?1 AND status = 'active' AND expires_at > ?2
            "#,
        )?;
        let rows = stmt.query_map(params![organization_id, Utc::now().to_rfc3339()], |row| {
            row.get(0)
        })?;
        collect_rows(rows)
    }

    pub fn latest_active_bundle_for_device(
        &self,
        organization_id: &str,
        device_id: &str,
    ) -> Result<Option<BundleRecord>> {
        let conn = self.lock()?;
        conn.query_row(
            r#"
            SELECT b.id, b.bundle_id, b.organization_id, b.license_id, b.device_count,
                   b.revoked_count, b.valid_until, b.issued_at, b.envelope_json
            FROM bundles b
            JOIN grants g ON g.bundle_id = b.bundle_id
            WHERE g.organization_id = ?1
              AND g.device_id = ?2
              AND g.revoked = 0
              AND g.valid_until > ?3
            ORDER BY b.issued_at DESC
            LIMIT 1
            "#,
            params![organization_id, device_id, Utc::now().to_rfc3339()],
            bundle_from_row,
        )
        .optional()
        .map_err(Into::into)
    }

    pub fn record_audit(
        &self,
        actor: &str,
        role: &str,
        action: &str,
        target: &str,
        details_json: &str,
    ) -> Result<()> {
        let conn = self.lock()?;
        conn.execute(
            r#"
            INSERT INTO audit_log (id, actor, role, action, target, created_at, details_json)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            params![
                format!("audit-{}", Uuid::new_v4()),
                actor,
                role,
                action,
                target,
                Utc::now().to_rfc3339(),
                details_json,
            ],
        )?;
        Ok(())
    }

    pub fn audit_log(&self) -> Result<Vec<AuditRecord>> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            r#"
            SELECT id, actor, role, action, target, created_at, details_json
            FROM audit_log
            ORDER BY created_at DESC
            LIMIT 1000
            "#,
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(AuditRecord {
                id: row.get(0)?,
                actor: row.get(1)?,
                role: row.get(2)?,
                action: row.get(3)?,
                target: row.get(4)?,
                created_at: row.get(5)?,
                details_json: row.get(6)?,
            })
        })?;
        collect_rows(rows)
    }

    fn device(&self, id: &str) -> Result<Option<DeviceRecord>> {
        let conn = self.lock()?;
        conn.query_row(
            r#"
            SELECT id, organization_id, request_id, commercial_device_id, device_fingerprint,
                   device_key_id, device_public_key, platform, app_version, build_channel,
                   seat_label, created_at, updated_at
            FROM devices
            WHERE id = ?1
            "#,
            params![id],
            device_from_row,
        )
        .optional()
        .map_err(Into::into)
    }

    pub fn device_by_identity(
        &self,
        commercial_device_id: &str,
        device_fingerprint: &str,
    ) -> Result<Option<DeviceRecord>> {
        let conn = self.lock()?;
        conn.query_row(
            r#"
            SELECT id, organization_id, request_id, commercial_device_id, device_fingerprint,
                   device_key_id, device_public_key, platform, app_version, build_channel,
                   seat_label, created_at, updated_at
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
        values.push("channel:commercial".to_string());
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

fn to_json<T: Serialize + ?Sized>(value: &T) -> Result<String> {
    Ok(serde_json::to_string(value)?)
}

fn from_json<T: DeserializeOwned + Default>(value: String) -> T {
    serde_json::from_str(&value).unwrap_or_default()
}

fn org_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<OrganizationRecord> {
    let features_json: String = row.get(8)?;
    let allowed_json: String = row.get(10)?;
    Ok(OrganizationRecord {
        id: row.get(0)?,
        license_id: row.get(1)?,
        activation_code: row.get(7)?,
        name: row.get(2)?,
        plan: row.get(3)?,
        max_seats: row.get::<_, i64>(4)? as u32,
        valid_from: row.get(5)?,
        valid_until: row.get(6)?,
        features: from_json(features_json),
        offline_grace_days: row.get::<_, i64>(9)? as u32,
        allowed_major_versions: from_json(allowed_json),
        issuer: row.get(11)?,
        created_at: row.get(12)?,
        updated_at: row.get(13)?,
    })
}

fn new_activation_code() -> String {
    format!("act-{}", Uuid::new_v4())
}

fn table_has_column(conn: &Connection, table: &str, column: &str) -> Result<bool> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
    let names = collect_rows(stmt.query_map([], |row| row.get::<_, String>(1))?)?;
    Ok(names.iter().any(|name| name == column))
}

fn device_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<DeviceRecord> {
    Ok(DeviceRecord {
        id: row.get(0)?,
        organization_id: row.get(1)?,
        request_id: row.get(2)?,
        commercial_device_id: row.get(3)?,
        device_fingerprint: row.get(4)?,
        device_key_id: row.get(5)?,
        device_public_key: row.get(6)?,
        platform: row.get(7)?,
        app_version: row.get(8)?,
        build_channel: row.get(9)?,
        seat_label: row.get(10)?,
        created_at: row.get(11)?,
        updated_at: row.get(12)?,
    })
}

fn verify_device_request_proof(request: &DeviceAuthorizationRequest) -> Result<()> {
    let public_key = URL_SAFE_NO_PAD
        .decode(&request.device_public_key)
        .context("device public key must be base64url")?;
    let public_key: [u8; 32] = public_key
        .try_into()
        .map_err(|_| anyhow!("device public key must be 32 bytes"))?;
    let expected_key_id: String = sha2::Sha256::digest(public_key)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    if request.device_key_id != expected_key_id {
        return Err(anyhow!("device key ID does not match public key"));
    }
    let signature = URL_SAFE_NO_PAD
        .decode(&request.device_proof)
        .context("device proof must be base64url")?;
    let signature: [u8; 64] = signature
        .try_into()
        .map_err(|_| anyhow!("device proof must be 64 bytes"))?;
    VerifyingKey::from_bytes(&public_key)
        .context("device public key is invalid")?
        .verify(&request.proof_message(), &Signature::from_bytes(&signature))
        .map_err(|_| anyhow!("device authorization proof is invalid"))
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
    use ed25519_dalek::{Signer, SigningKey};

    fn signed_device_request(organization_id: Option<String>) -> DeviceAuthorizationRequest {
        signed_device_request_with_seed(organization_id, 31)
    }

    fn make_org(id: &str, name: &str) -> CreateOrganizationRequest {
        CreateOrganizationRequest {
            organization_id: Some(id.to_string()),
            activation_code: None,
            name: name.to_string(),
            plan: None,
            max_seats: Some(2),
            valid_days: Some(30),
            valid_until: None,
            features: vec![],
            offline_grace_days: None,
            allowed_major_versions: vec![],
        }
    }

    fn signed_device_request_with_seed(
        organization_id: Option<String>,
        seed: u8,
    ) -> DeviceAuthorizationRequest {
        let key = SigningKey::from_bytes(&[seed; 32]);
        let public_key = key.verifying_key().to_bytes();
        let mut request = DeviceAuthorizationRequest {
            schema_version: LICENSE_SCHEMA_VERSION,
            request_id: format!("req-{}", Uuid::new_v4()),
            organization_id,
            commercial_device_id: format!("commercial-device-{seed}"),
            device_fingerprint: format!("fingerprint-{seed}"),
            device_key_id: sha2::Sha256::digest(public_key)
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect(),
            device_public_key: URL_SAFE_NO_PAD.encode(public_key),
            device_proof: String::new(),
            platform: "macos".to_string(),
            app_version: "0.1.0".to_string(),
            build_channel: "commercial".to_string(),
            seat_label: Some("Finance laptop".to_string()),
            created_at: Utc::now().to_rfc3339(),
        };
        request.device_proof =
            URL_SAFE_NO_PAD.encode(key.sign(&request.proof_message()).to_bytes());
        request
    }

    fn resign_device_request(request: &mut DeviceAuthorizationRequest, seed: u8) {
        let key = SigningKey::from_bytes(&[seed; 32]);
        request.device_proof =
            URL_SAFE_NO_PAD.encode(key.sign(&request.proof_message()).to_bytes());
    }

    fn bundle_and_grant(
        org: &OrganizationRecord,
        device: &DeviceRecord,
        suffix: &str,
        valid_until: String,
    ) -> (BundleRecord, crate::model::DeviceGrant) {
        let now = Utc::now().to_rfc3339();
        let bundle_id = format!("bundle-{suffix}");
        (
            BundleRecord {
                id: format!("bundle-row-{suffix}"),
                bundle_id: bundle_id.clone(),
                organization_id: org.id.clone(),
                license_id: org.license_id.clone(),
                device_count: 1,
                revoked_count: 0,
                valid_until: valid_until.clone(),
                issued_at: now.clone(),
                envelope_json: "{}".to_string(),
            },
            crate::model::DeviceGrant {
                schema_version: LICENSE_SCHEMA_VERSION,
                grant_id: format!("grant-{suffix}"),
                license_id: org.license_id.clone(),
                organization_id: org.id.clone(),
                commercial_device_id: device.commercial_device_id.clone(),
                device_fingerprint: device.device_fingerprint.clone(),
                device_key_id: device.device_key_id.clone(),
                device_public_key: device.device_public_key.clone(),
                seat_label: device.seat_label.clone(),
                valid_from: now.clone(),
                valid_until,
                features: vec![],
                offline_grace_days: 0,
                issued_at: now,
                issuer: "test".to_string(),
            },
        )
    }

    #[test]
    fn organization_and_device_request_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let db = Db::open(dir.path().join("admin.sqlite3")).unwrap();
        let org = db
            .create_organization(
                CreateOrganizationRequest {
                    organization_id: Some("org-acme".to_string()),
                    activation_code: None,
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
        let request = signed_device_request(Some(org.id.clone()));
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
        assert!(org.activation_code.starts_with("act-"));
        assert!(org
            .features
            .iter()
            .any(|value| value == "channel:commercial"));
        assert_eq!(
            db.organization_by_activation_code(&org.activation_code)
                .unwrap()
                .unwrap()
                .id,
            org.id
        );
    }

    #[test]
    fn device_identity_cannot_move_between_organizations() {
        let dir = tempfile::tempdir().unwrap();
        let db = Db::open(dir.path().join("admin.sqlite3")).unwrap();
        let make_org = |id: &str, name: &str| CreateOrganizationRequest {
            organization_id: Some(id.to_string()),
            activation_code: None,
            name: name.to_string(),
            plan: None,
            max_seats: Some(2),
            valid_days: Some(30),
            valid_until: None,
            features: vec![],
            offline_grace_days: None,
            allowed_major_versions: vec![],
        };
        db.create_organization(make_org("org-a", "A"), "test")
            .unwrap();
        db.create_organization(make_org("org-b", "B"), "test")
            .unwrap();
        let request = signed_device_request(None);
        let request_json = serde_json::to_string(&request).unwrap();
        db.import_device_request(ImportDeviceRequestBody {
            request_json: request_json.clone(),
            organization_id: Some("org-a".to_string()),
            seat_label: None,
        })
        .unwrap();
        let error = db
            .import_device_request(ImportDeviceRequestBody {
                request_json,
                organization_id: Some("org-b".to_string()),
                seat_label: None,
            })
            .unwrap_err();
        assert!(error
            .to_string()
            .contains("already assigned to another organization"));
    }

    #[test]
    fn invalid_device_proof_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let db = Db::open(dir.path().join("admin.sqlite3")).unwrap();
        db.create_organization(make_org("org-a", "A"), "test")
            .unwrap();
        let mut request = signed_device_request(Some("org-a".to_string()));
        request.platform = "tampered-platform".to_string();
        let error = db
            .import_device_request(ImportDeviceRequestBody {
                request_json: serde_json::to_string(&request).unwrap(),
                organization_id: None,
                seat_label: None,
            })
            .unwrap_err();
        assert!(error.to_string().contains("proof"));
    }

    #[test]
    fn one_device_key_cannot_register_multiple_device_identities() {
        let dir = tempfile::tempdir().unwrap();
        let db = Db::open(dir.path().join("admin.sqlite3")).unwrap();
        db.create_organization(make_org("org-a", "A"), "test")
            .unwrap();
        let request = signed_device_request(Some("org-a".to_string()));
        db.import_device_request(ImportDeviceRequestBody {
            request_json: serde_json::to_string(&request).unwrap(),
            organization_id: None,
            seat_label: None,
        })
        .unwrap();

        let mut duplicate = request;
        duplicate.request_id = format!("req-{}", Uuid::new_v4());
        duplicate.commercial_device_id = "another-commercial-device".to_string();
        duplicate.device_fingerprint = "another-fingerprint".to_string();
        duplicate.created_at = Utc::now().to_rfc3339();
        resign_device_request(&mut duplicate, 31);
        let error = db
            .import_device_request(ImportDeviceRequestBody {
                request_json: serde_json::to_string(&duplicate).unwrap(),
                organization_id: None,
                seat_label: None,
            })
            .unwrap_err();
        assert!(error
            .to_string()
            .contains("already registered to another device"));
    }

    #[test]
    fn one_request_id_cannot_be_replayed_for_another_identity() {
        let dir = tempfile::tempdir().unwrap();
        let db = Db::open(dir.path().join("admin.sqlite3")).unwrap();
        db.create_organization(make_org("org-a", "A"), "test")
            .unwrap();
        let first = signed_device_request_with_seed(Some("org-a".to_string()), 51);
        db.import_device_request(ImportDeviceRequestBody {
            request_json: serde_json::to_string(&first).unwrap(),
            organization_id: None,
            seat_label: None,
        })
        .unwrap();
        let mut replay = signed_device_request_with_seed(Some("org-a".to_string()), 52);
        replay.request_id = first.request_id;
        resign_device_request(&mut replay, 52);
        assert!(db
            .import_device_request(ImportDeviceRequestBody {
                request_json: serde_json::to_string(&replay).unwrap(),
                organization_id: None,
                seat_label: None,
            })
            .is_err());
    }

    #[test]
    fn seat_allocation_transaction_refuses_over_issue() {
        let dir = tempfile::tempdir().unwrap();
        let db = Db::open(dir.path().join("admin.sqlite3")).unwrap();
        let org = db
            .create_organization(
                CreateOrganizationRequest {
                    organization_id: Some("org-seat-cap".to_string()),
                    activation_code: None,
                    name: "Seat Cap".to_string(),
                    plan: None,
                    max_seats: Some(1),
                    valid_days: Some(30),
                    valid_until: None,
                    features: vec![],
                    offline_grace_days: None,
                    allowed_major_versions: vec![],
                },
                "test",
            )
            .unwrap();
        let import = |seed| {
            let request = signed_device_request_with_seed(Some(org.id.clone()), seed);
            db.import_device_request(ImportDeviceRequestBody {
                request_json: serde_json::to_string(&request).unwrap(),
                organization_id: None,
                seat_label: None,
            })
            .unwrap()
        };
        let first = import(41);
        let second = import(42);
        let valid_until = (Utc::now() + Duration::days(30)).to_rfc3339();
        let (first_bundle, first_grant) =
            bundle_and_grant(&org, &first, "first", valid_until.clone());
        db.store_bundle(&first_bundle, &[first_grant], &[first], 1)
            .unwrap();
        let (second_bundle, second_grant) = bundle_and_grant(&org, &second, "second", valid_until);
        let error = db
            .store_bundle(&second_bundle, &[second_grant], &[second], 1)
            .unwrap_err();
        assert!(error.to_string().contains("maxSeats"));
        assert_eq!(db.active_licensed_device_ids(&org.id).unwrap().len(), 1);
    }
}
