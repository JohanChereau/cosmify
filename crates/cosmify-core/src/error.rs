use std::path::PathBuf;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, CosmifyError>;

#[derive(Debug, Error)]
pub enum CosmifyError {
    #[error("Minecraft Bedrock premium cache could not be found")]
    PremiumCacheNotFound,

    #[error("Minecraft appears to be running. Close Minecraft before modifying the premium cache")]
    MinecraftRunning,

    #[error("The selected path is outside Minecraft's skin_packs directory: {0}")]
    HostOutsideSkinPacks(PathBuf),

    #[error("Host pack changed since preview. Refresh the pack list and preview again")]
    HostChanged,

    #[error("Custom pack is invalid: {0}")]
    InvalidCustomPack(String),

    #[error("Settings are invalid: {0}")]
    InvalidSettings(String),

    #[error("Cosmetic pack is invalid: {0}")]
    InvalidCosmeticPack(String),

    #[error("Managed cosmetic pack was not found: {0}")]
    CosmeticPackNotFound(String),

    #[error("Host pack is invalid: {0}")]
    InvalidHostPack(String),

    #[error("The host pack's contents.json cannot be decrypted with the available content key")]
    ContentKeyUnavailable,

    #[error("Unsafe archive entry rejected: {0}")]
    UnsafeArchiveEntry(String),

    #[error("Backup metadata is invalid: {0}")]
    InvalidBackup(String),

    #[error("No application data directory is available")]
    AppDataUnavailable,

    #[error("Operation cancelled: {0}")]
    Cancelled(String),

    #[error("I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("Walk directory error: {0}")]
    WalkDir(#[from] walkdir::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl CosmifyError {
    pub(crate) fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }
}
