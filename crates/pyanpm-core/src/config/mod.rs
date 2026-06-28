use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use directories::ProjectDirs;
use fs_err as fs;
use serde::{Deserialize, Serialize};

use crate::error::{PyanpmError, Result};
use crate::plugin::{ArtifactKind, PluginSourceKind};

pub const MANIFEST_FILE_NAME: &str = "pyanpm.toml";
pub const LOCKFILE_FILE_NAME: &str = "pyanpm.lock";
pub const PACKAGE_METADATA_FILE_NAME: &str = "pyanpm.plugin.toml";
pub const GLOBAL_STATE_DIR_NAME: &str = "state";

pub fn global_state_dir() -> Result<PathBuf> {
    let project_dirs = ProjectDirs::from("dev", "bblocks", "pyanpm")
        .ok_or(PyanpmError::MissingDefaultConfigDir)?;
    let state_dir = project_dirs.config_dir().join(GLOBAL_STATE_DIR_NAME);
    fs::create_dir_all(&state_dir)?;
    Ok(state_dir)
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Manifest {
    #[serde(default)]
    pub plugins: BTreeMap<String, ManifestPluginSpec>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum GitRefKind {
    Branch,
    Tag,
    Commit,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GitSourceOptions {
    #[serde(default)]
    pub git_ref_kind: Option<GitRefKind>,
    #[serde(default)]
    pub git_ref: Option<String>,
    #[serde(default)]
    pub git_subdir: Option<String>,
}

impl GitSourceOptions {
    pub fn is_empty(&self) -> bool {
        self.git_ref_kind.is_none()
            && self
                .git_ref
                .as_ref()
                .is_none_or(|git_ref| git_ref.trim().is_empty())
            && self
                .git_subdir
                .as_ref()
                .is_none_or(|git_subdir| git_subdir.trim().is_empty())
    }

    pub fn normalized(&self) -> Self {
        Self {
            git_ref_kind: self.git_ref_kind,
            git_ref: self.git_ref.as_ref().map(|value| value.trim().to_owned()).filter(|value| !value.is_empty()),
            git_subdir: self
                .git_subdir
                .as_ref()
                .map(|value| value.trim().replace('\\', "/"))
                .filter(|value| !value.is_empty()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestPluginSpec {
    pub source: PluginSourceKind,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub git_ref_kind: Option<GitRefKind>,
    #[serde(default)]
    pub git_ref: Option<String>,
    #[serde(default)]
    pub subdir: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Lockfile {
    #[serde(default = "default_lock_version")]
    pub version: u32,
    #[serde(default)]
    pub generated_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub plugins: Vec<LockedPlugin>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedPlugin {
    pub name: String,
    #[serde(default)]
    pub requested_version: Option<String>,
    #[serde(default)]
    pub resolved_version: Option<String>,
    pub source: PluginSourceKind,
    pub source_path: String,
    #[serde(default)]
    pub source_url: Option<String>,
    #[serde(default)]
    pub source_subdir: Option<String>,
    #[serde(default)]
    pub git_ref_kind: Option<GitRefKind>,
    #[serde(default)]
    pub git_requested_ref: Option<String>,
    #[serde(default)]
    pub git_resolved_commit: Option<String>,
    pub checksum: String,
    pub artifact_kind: ArtifactKind,
    pub target_path: String,
    pub installed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMetadata {
    pub name: String,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
    pub entry: String,
}

fn default_lock_version() -> u32 {
    1
}

impl Manifest {
    pub fn path_in(project_dir: &Path) -> PathBuf {
        project_dir.join(MANIFEST_FILE_NAME)
    }

    pub fn load(project_dir: &Path) -> Result<Self> {
        let path = Self::path_in(project_dir);
        if !path.exists() {
            return Err(PyanpmError::ManifestMissing(path));
        }

        let content = fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }

    pub fn load_optional(project_dir: &Path) -> Result<Option<Self>> {
        let path = Self::path_in(project_dir);
        if !path.exists() {
            return Ok(None);
        }

        Self::load(project_dir).map(Some)
    }

    pub fn save(&self, project_dir: &Path) -> Result<()> {
        let path = Self::path_in(project_dir);
        fs::create_dir_all(project_dir)?;
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}

impl Lockfile {
    pub fn path_in(project_dir: &Path) -> PathBuf {
        project_dir.join(LOCKFILE_FILE_NAME)
    }

    pub fn load(project_dir: &Path) -> Result<Self> {
        let path = Self::path_in(project_dir);
        let content = fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }

    pub fn load_optional(project_dir: &Path) -> Result<Option<Self>> {
        let path = Self::path_in(project_dir);
        if !path.exists() {
            return Ok(None);
        }

        Self::load(project_dir).map(Some)
    }

    pub fn save(&mut self, project_dir: &Path) -> Result<()> {
        self.plugins.sort_by(|left, right| left.name.cmp(&right.name));
        self.generated_at = Some(Utc::now());
        fs::create_dir_all(project_dir)?;
        let content = toml::to_string_pretty(self)?;
        fs::write(Self::path_in(project_dir), content)?;
        Ok(())
    }
}

impl PackageMetadata {
    pub fn read_from(source_dir: &Path) -> Result<Self> {
        let metadata_path = source_dir.join(PACKAGE_METADATA_FILE_NAME);
        if !metadata_path.exists() {
            return Err(PyanpmError::MissingMetadata(metadata_path));
        }

        let content = fs::read_to_string(metadata_path)?;
        Ok(toml::from_str(&content)?)
    }
}
