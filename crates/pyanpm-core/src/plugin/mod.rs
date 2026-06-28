use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::config::GitRefKind;
use crate::error::{PyanpmError, Result};

const RESERVED_WINDOWS_NAMES: &[&str] = &[
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PluginSourceKind {
    File,
    Path,
    Git,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ArtifactKind {
    File,
    Directory,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResolvedPlugin {
    pub name: String,
    pub requested_version: Option<String>,
    pub resolved_version: Option<String>,
    pub source: PluginSourceKind,
    pub source_path: PathBuf,
    pub source_url: Option<String>,
    pub source_subdir: Option<String>,
    pub git_ref_kind: Option<GitRefKind>,
    pub git_requested_ref: Option<String>,
    pub git_resolved_commit: Option<String>,
    pub artifact_path: PathBuf,
    pub artifact_kind: ArtifactKind,
    pub checksum: String,
    pub install_name: String,
}

pub fn validate_plugin_name(name: &str) -> Result<()> {
    if name.is_empty()
        || name.contains(['/', '\\', ':'])
        || name.ends_with('.')
        || name.ends_with(' ')
        || RESERVED_WINDOWS_NAMES.contains(&name.to_ascii_uppercase().as_str())
    {
        return Err(PyanpmError::InvalidPluginName(name.to_owned()));
    }

    if !name
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.'))
    {
        return Err(PyanpmError::InvalidPluginName(name.to_owned()));
    }

    Ok(())
}

pub fn infer_file_plugin_name(path: &Path) -> Result<String> {
    let stem = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(|| PyanpmError::InvalidPluginName(path.display().to_string()))?;

    validate_plugin_name(stem)?;
    Ok(stem.to_owned())
}
