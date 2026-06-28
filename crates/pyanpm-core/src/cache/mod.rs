use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use directories::ProjectDirs;
use fs_err as fs;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

use crate::error::{PyanpmError, Result};
use crate::operations::CacheEntry;
use crate::plugin::{ArtifactKind, ResolvedPlugin};

#[derive(Debug, Clone)]
pub struct CacheStore {
    root: PathBuf,
}

#[derive(Debug, Clone)]
pub struct CachedArtifact {
    pub checksum: String,
    pub artifact_kind: ArtifactKind,
    pub stored_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheMetadata {
    cache_id: String,
    plugin_name: String,
    version: Option<String>,
    source_kind: String,
    source_summary: String,
    checksum: String,
    size_bytes: u64,
    created_at: DateTime<Utc>,
    last_used_at: DateTime<Utc>,
    path: String,
}

impl CacheStore {
    pub fn new(root_override: Option<PathBuf>) -> Result<Self> {
        let root = match root_override {
            Some(path) => path,
            None => default_cache_dir()?,
        };

        fs::create_dir_all(&root)?;
        Ok(Self { root })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn cache_artifact(&self, resolved: &ResolvedPlugin) -> Result<CachedArtifact> {
        let artifact_root = self.root.join(&resolved.checksum);
        let stored_path = artifact_root.join(&resolved.install_name);
        let metadata_path = artifact_root.join("entry.json");
        if stored_path.exists() {
            self.write_metadata(
                &metadata_path,
                cache_metadata(resolved, &stored_path, metadata_path.exists())?,
            )?;
            return Ok(CachedArtifact {
                checksum: resolved.checksum.clone(),
                artifact_kind: resolved.artifact_kind,
                stored_path,
            });
        }

        fs::create_dir_all(&artifact_root)?;
        copy_path(&resolved.artifact_path, &stored_path)?;
        self.write_metadata(&metadata_path, cache_metadata(resolved, &stored_path, false)?)?;
        Ok(CachedArtifact {
            checksum: resolved.checksum.clone(),
            artifact_kind: resolved.artifact_kind,
            stored_path,
        })
    }

    pub fn list_entries(&self, active_checksums: &[String]) -> Result<Vec<CacheEntry>> {
        let mut entries = Vec::new();
        for entry in fs::read_dir(&self.root)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let metadata = self.read_metadata(&path)?;
            entries.push(CacheEntry {
                cache_id: metadata.cache_id.clone(),
                plugin_name: metadata.plugin_name,
                version: metadata.version,
                source_kind: metadata.source_kind,
                source_summary: metadata.source_summary,
                checksum: metadata.checksum.clone(),
                size_bytes: metadata.size_bytes,
                created_at: metadata.created_at,
                last_used_at: metadata.last_used_at,
                referenced_by_active_lockfile: active_checksums.iter().any(|checksum| checksum == &metadata.checksum),
                path: metadata.path,
            });
        }
        entries.sort_by(|left, right| right.last_used_at.cmp(&left.last_used_at));
        Ok(entries)
    }

    pub fn get_entry(&self, cache_id: &str, active_checksums: &[String]) -> Result<CacheEntry> {
        self.list_entries(active_checksums)?
            .into_iter()
            .find(|entry| entry.cache_id == cache_id)
            .ok_or_else(|| PyanpmError::CacheEntryNotFound(cache_id.to_owned()))
    }

    pub fn evict_entry(&self, cache_id: &str) -> Result<u64> {
        let path = self.root.join(cache_id);
        if !path.exists() {
            return Err(PyanpmError::CacheEntryNotFound(cache_id.to_owned()));
        }
        let size = directory_size(&path)?;
        fs::remove_dir_all(path)?;
        Ok(size)
    }

    pub fn prune_unreferenced(&self, active_checksums: &[String]) -> Result<(Vec<String>, u64)> {
        let entries = self.list_entries(active_checksums)?;
        let mut removed = Vec::new();
        let mut reclaimed_size_bytes = 0;
        for entry in entries.into_iter().filter(|entry| !entry.referenced_by_active_lockfile) {
            reclaimed_size_bytes += self.evict_entry(&entry.cache_id)?;
            removed.push(entry.cache_id);
        }
        Ok((removed, reclaimed_size_bytes))
    }

    fn read_metadata(&self, artifact_root: &Path) -> Result<CacheMetadata> {
        let metadata_path = artifact_root.join("entry.json");
        if metadata_path.exists() {
            let content = fs::read_to_string(metadata_path)?;
            return Ok(serde_json::from_str(&content)?);
        }

        let stored_path = first_non_metadata_entry(artifact_root)?;
        let checksum = artifact_root
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| PyanpmError::CacheEntryNotFound(artifact_root.display().to_string()))?
            .to_owned();
        let now = Utc::now();
        Ok(CacheMetadata {
            cache_id: checksum.clone(),
            plugin_name: stored_path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or("unknown")
                .to_owned(),
            version: None,
            source_kind: "unknown".to_owned(),
            source_summary: stored_path.display().to_string(),
            checksum,
            size_bytes: path_size(&stored_path)?,
            created_at: now,
            last_used_at: now,
            path: stored_path.display().to_string(),
        })
    }

    fn write_metadata(&self, metadata_path: &Path, metadata: CacheMetadata) -> Result<()> {
        fs::write(metadata_path, serde_json::to_string_pretty(&metadata)?)?;
        Ok(())
    }
}

pub fn checksum_path(path: &Path) -> Result<String> {
    if path.is_file() {
        return checksum_file(path);
    }

    if path.is_dir() {
        let mut hasher = Sha256::new();
        let mut entries = WalkDir::new(path)
            .into_iter()
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(std::io::Error::other)?;
        entries.sort_by_key(|entry| entry.path().to_path_buf());

        for entry in entries {
            let entry_path = entry.path();
            if entry_path == path {
                continue;
            }

            let relative = entry_path
                .strip_prefix(path)
                .map_err(std::io::Error::other)?
                .to_string_lossy();
            hasher.update(relative.as_bytes());

            if entry.file_type().is_file() {
                let mut file = File::open(entry_path)?;
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer)?;
                hasher.update(buffer);
            }
        }

        return Ok(hex::encode(hasher.finalize()));
    }

    Err(PyanpmError::UnsupportedArtifact(path.to_path_buf()))
}

pub fn copy_path(source: &Path, destination: &Path) -> Result<()> {
    if source.is_file() {
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::copy(source, destination)?;
        return Ok(());
    }

    if source.is_dir() {
        fs::create_dir_all(destination)?;
        for entry in WalkDir::new(source) {
            let entry = entry.map_err(std::io::Error::other)?;
            let relative = entry
                .path()
                .strip_prefix(source)
                .map_err(std::io::Error::other)?;
            if relative.as_os_str().is_empty() {
                continue;
            }

            let target = destination.join(relative);
            if entry.file_type().is_dir() {
                fs::create_dir_all(&target)?;
            } else {
                if let Some(parent) = target.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(entry.path(), &target)?;
            }
        }
        return Ok(());
    }

    Err(PyanpmError::UnsupportedArtifact(source.to_path_buf()))
}

fn checksum_file(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 8192];
    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(hex::encode(hasher.finalize()))
}

fn default_cache_dir() -> Result<PathBuf> {
    let project_dirs = ProjectDirs::from("dev", "bblocks", "pyanpm")
        .ok_or(PyanpmError::MissingDefaultCacheDir)?;
    Ok(project_dirs.cache_dir().to_path_buf())
}

fn cache_metadata(resolved: &ResolvedPlugin, stored_path: &Path, preserve_created_at: bool) -> Result<CacheMetadata> {
    let now = Utc::now();
    Ok(CacheMetadata {
        cache_id: resolved.checksum.clone(),
        plugin_name: resolved.name.clone(),
        version: resolved.resolved_version.clone().or_else(|| resolved.requested_version.clone()),
        source_kind: format!("{:?}", resolved.source).to_ascii_lowercase(),
        source_summary: match resolved.source {
            crate::plugin::PluginSourceKind::File => format!("file:{}", resolved.source_path.display()),
            crate::plugin::PluginSourceKind::Path => format!("path:{}", resolved.source_path.display()),
            crate::plugin::PluginSourceKind::Git => {
                let mut summary = format!("git:{}", resolved.source_url.as_deref().unwrap_or("unknown"));
                if let Some(kind) = resolved.git_ref_kind {
                    let kind = match kind {
                        crate::config::GitRefKind::Branch => "branch",
                        crate::config::GitRefKind::Tag => "tag",
                        crate::config::GitRefKind::Commit => "commit",
                    };
                    if let Some(reference) = resolved.git_requested_ref.as_deref() {
                        summary.push_str(&format!(" [{kind}:{reference}]"));
                    }
                }
                if let Some(subdir) = resolved.source_subdir.as_deref() {
                    summary.push_str(&format!(" [{subdir}]"));
                }
                if let Some(commit) = resolved.git_resolved_commit.as_deref() {
                    summary.push_str(&format!(" @{commit}"));
                }
                summary
            }
        },
        checksum: resolved.checksum.clone(),
        size_bytes: path_size(stored_path)?,
        created_at: if preserve_created_at { now } else { now },
        last_used_at: now,
        path: stored_path.display().to_string(),
    })
}

fn first_non_metadata_entry(root: &Path) -> Result<PathBuf> {
    let mut entries = fs::read_dir(root)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.file_name().and_then(|name| name.to_str()) != Some("entry.json"))
        .collect::<Vec<_>>();
    entries.sort();
    entries
        .into_iter()
        .next()
        .ok_or_else(|| PyanpmError::CacheEntryNotFound(root.display().to_string()))
}

fn path_size(path: &Path) -> Result<u64> {
    if path.is_file() {
        return Ok(fs::metadata(path)?.len());
    }

    let mut size = 0_u64;
    for entry in WalkDir::new(path) {
        let entry = entry.map_err(std::io::Error::other)?;
        if entry.file_type().is_file() {
            size += entry.metadata().map_err(std::io::Error::other)?.len();
        }
    }
    Ok(size)
}

fn directory_size(path: &Path) -> Result<u64> {
    let mut size = 0_u64;
    for entry in WalkDir::new(path) {
        let entry = entry.map_err(std::io::Error::other)?;
        if entry.file_type().is_file() {
            size += entry.metadata().map_err(std::io::Error::other)?.len();
        }
    }
    Ok(size)
}

#[allow(dead_code)]
fn file_name(path: &Path) -> Result<&OsStr> {
    path.file_name()
        .ok_or_else(|| PyanpmError::UnsupportedArtifact(path.to_path_buf()))
}
