use std::path::PathBuf;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, PyanpmError>;

#[derive(Debug, Error)]
pub enum PyanpmError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("manifest already exists at {0}")]
    ManifestExists(PathBuf),
    #[error("manifest not found at {0}")]
    ManifestMissing(PathBuf),
    #[error("lockfile parse error: {0}")]
    LockfileParse(#[from] toml::de::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] toml::ser::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid plugin reference `{0}`")]
    InvalidPluginRef(String),
    #[error("unsupported source `{0}`")]
    UnsupportedSource(String),
    #[error("plugin `{0}` already exists in the manifest")]
    PluginAlreadyExists(String),
    #[error("unknown plugin `{0}`")]
    UnknownPlugin(String),
    #[error("invalid plugin name `{0}`")]
    InvalidPluginName(String),
    #[error("unsupported artifact at {0}")]
    UnsupportedArtifact(PathBuf),
    #[error("missing plugin metadata at {0}")]
    MissingMetadata(PathBuf),
    #[error("plugin metadata error: {0}")]
    InvalidMetadata(String),
    #[error("git executable was not found in PATH")]
    MissingGitExecutable,
    #[error("git clone failed for `{url}`: {message}")]
    GitCloneFailed { url: String, message: String },
    #[error("git fetch failed for `{url}`: {message}")]
    GitFetchFailed { url: String, message: String },
    #[error("git checkout failed for `{url}`: {message}")]
    GitCheckoutFailed { url: String, message: String },
    #[error("git source URL is invalid: {0}")]
    InvalidGitUrl(String),
    #[error("git reference is invalid: {0}")]
    InvalidGitRef(String),
    #[error("git subdirectory is invalid: {0}")]
    InvalidGitSubdir(String),
    #[error("install target escapes the Studio plugins directory: {0}")]
    InstallTargetEscapes(PathBuf),
    #[error("platform default cache directory is unavailable")]
    MissingDefaultCacheDir,
    #[error("platform default config directory is unavailable")]
    MissingDefaultConfigDir,
    #[error("platform default Studio plugins directory is unavailable; pass --plugins-dir explicitly")]
    MissingDefaultPluginsDir,
    #[error("activity record `{0}` was not found")]
    ActivityNotFound(String),
    #[error("cache entry `{0}` was not found")]
    CacheEntryNotFound(String),
    #[error("cache entry `{0}` is protected by the active lockfile")]
    ProtectedCacheEntry(String),
    #[error("confirmation required: {0}")]
    ConfirmationRequired(String),
}
