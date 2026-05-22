use crate::domain::AppConfig;
use crate::error::{KeylessPassError, Result};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct StoragePaths {
    pub app_dir: PathBuf,
    pub config_path: PathBuf,
    pub db_path: PathBuf,
    pub local_factor_path: PathBuf,
    pub recovery_path: PathBuf,
}

impl StoragePaths {
    pub fn default() -> Result<Self> {
        let app_dir = default_app_dir()?;
        Ok(Self::from_app_dir(app_dir))
    }

    pub fn from_app_dir(app_dir: PathBuf) -> Self {
        Self {
            config_path: app_dir.join("keylesspass-config.json"),
            db_path: app_dir.join("cdr.sqlite3"),
            local_factor_path: app_dir.join("local-factor-package.json"),
            recovery_path: app_dir.join("recovery-metadata.json"),
            app_dir,
        }
    }

    pub fn ensure(&self) -> Result<()> {
        fs::create_dir_all(&self.app_dir)?;
        Ok(())
    }
}

pub fn default_app_dir() -> Result<PathBuf> {
    if let Ok(value) = env::var("KEYLESSPASS_HOME") {
        return Ok(PathBuf::from(value));
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(appdata) = env::var("APPDATA") {
            return Ok(PathBuf::from(appdata).join("KeylessPass"));
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = env::var("HOME") {
            return Ok(PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("KeylessPass"));
        }
    }

    if let Ok(xdg) = env::var("XDG_DATA_HOME") {
        return Ok(PathBuf::from(xdg).join("keylesspass"));
    }
    if let Ok(home) = env::var("HOME") {
        return Ok(PathBuf::from(home)
            .join(".local")
            .join("share")
            .join("keylesspass"));
    }
    Err(KeylessPassError::Validation(
        "cannot determine application data directory".to_string(),
    ))
}

pub fn write_json_private<T: serde::Serialize>(path: &Path, value: &T) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(value)?;
    crate::platform::fallback::write_private_file(&path.to_path_buf(), &bytes)
}

pub fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let bytes = fs::read(path)?;
    Ok(serde_json::from_slice(&bytes)?)
}

pub fn write_config(paths: &StoragePaths, config: &AppConfig) -> Result<()> {
    paths.ensure()?;
    write_json_private(&paths.config_path, config)
}

pub fn read_config(paths: &StoragePaths) -> Result<AppConfig> {
    if !paths.config_path.exists() {
        return Err(KeylessPassError::NotEnrolled);
    }
    read_json(&paths.config_path)
}
