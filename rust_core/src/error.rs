use thiserror::Error;

pub type Result<T> = std::result::Result<T, KeylessPassError>;

#[derive(Debug, Error)]
pub enum KeylessPassError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("base64 error: {0}")]
    Base64(#[from] base64::DecodeError),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("crypto error: {0}")]
    Crypto(String),
    #[error("missing factor: {0}")]
    MissingFactor(String),
    #[error("integrity check failed: {0}")]
    Integrity(String),
    #[error("not enrolled")]
    NotEnrolled,
}

impl From<KeylessPassError> for String {
    fn from(value: KeylessPassError) -> Self {
        value.to_string()
    }
}
