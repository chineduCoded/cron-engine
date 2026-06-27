use thiserror::Error;

/// Concrete error type for storage operations.
#[derive(Error, Debug)]
pub enum StoreError {
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("sqlx error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("sled error: {0}")]
    Sled(#[from] sled::Error),

    #[error("task join error: {0}")]
    Join(#[from] tokio::task::JoinError),

    #[error("uuid parse error: {0}")]
    Uuid(#[from] uuid::Error),

    #[error("not found")]
    NotFound,

    #[error("validation error: {0}")]
    Validation(String),

    #[error("other: {0}")]
    Other(String),
}

pub type StoreResult<T> = Result<T, StoreError>;
