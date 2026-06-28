use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::path::Path;

use serde::Serialize;

use crate::cache::{checksum_path, CacheStore};
use crate::config::{Lockfile, Manifest};
use crate::error::{PyanpmError, Result};
use crate::install::resolve_plugins_dir;
use crate::plugin::PluginSourceKind;
use crate::source::{ensure_git_available, resolve_from_manifest};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FindingLevel {
    Ok,
    Warn,
    Error,
}

#[derive(Debug, Clone, Serialize)]
pub struct DoctorFinding {
    pub level: FindingLevel,
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DoctorReport {
    pub healthy: bool,
    pub findings: Vec<DoctorFinding>,
}

pub fn run_doctor(project_dir: &Path, plugins_dir_override: Option<&Path>, write_probe: bool) -> Result<DoctorReport> {
    let mut findings = Vec::new();

    let manifest = match Manifest::load(project_dir) {
        Ok(manifest) => {
            findings.push(ok("manifest", "Manifest parsed successfully."));

            let unique_names = manifest.plugins.keys().collect::<HashSet<_>>();
            if unique_names.len() != manifest.plugins.len() {
                findings.push(error("manifest.duplicate_names", "Duplicate plugin names were found."));
            }
            Some(manifest)
        }
        Err(load_error) => {
            findings.push(error("manifest.parse", &load_error.to_string()));
            None
        }
    };

    let lockfile = match Lockfile::load_optional(project_dir) {
        Ok(Some(lockfile)) => {
            findings.push(ok("lockfile", "Lockfile parsed successfully."));
            let mut seen = HashSet::new();
            let duplicates = lockfile
                .plugins
                .iter()
                .filter_map(|plugin| {
                    if seen.insert(plugin.name.clone()) {
                        None
                    } else {
                        Some(plugin.name.clone())
                    }
                })
                .collect::<Vec<_>>();
            for duplicate in duplicates {
                findings.push(error(
                    &format!("lockfile.duplicate.{duplicate}"),
                    &format!("Lockfile contains duplicate entries for `{duplicate}`."),
                ));
            }
            Some(lockfile)
        }
        Ok(None) => {
            findings.push(warn("lockfile.missing", "Lockfile does not exist yet."));
            None
        }
        Err(load_error) => {
            findings.push(error("lockfile.parse", &load_error.to_string()));
            None
        }
    };

    let has_git_sources = manifest
        .as_ref()
        .is_some_and(|manifest| manifest.plugins.values().any(|spec| spec.source == PluginSourceKind::Git))
        || lockfile
            .as_ref()
            .is_some_and(|lockfile| lockfile.plugins.iter().any(|plugin| plugin.source == PluginSourceKind::Git));
    let git_available = if has_git_sources {
        match ensure_git_available() {
            Ok(()) => {
                findings.push(ok("git.access", "Git is available in PATH."));
                true
            }
            Err(git_error) => {
                findings.push(error("git.access", &git_error.to_string()));
                false
            }
        }
    } else {
        true
    };

    match CacheStore::new(None) {
        Ok(cache) => findings.push(ok(
            "cache.access",
            &format!("Cache directory is accessible at {}.", cache.root().display()),
        )),
        Err(cache_error) => findings.push(error("cache.access", &cache_error.to_string())),
    }

    match resolve_plugins_dir(plugins_dir_override) {
        Ok(plugins_dir) => {
            findings.push(ok(
                "plugins_dir.access",
                &format!("Studio plugins directory is accessible at {}.", plugins_dir.display()),
            ));

            if write_probe {
                let probe_path = plugins_dir.join(".pyanpm-write-probe.tmp");
                match fs_err::File::create(&probe_path).and_then(|mut file| file.write_all(b"probe")) {
                    Ok(()) => {
                        let _ = fs_err::remove_file(&probe_path);
                        findings.push(ok(
                            "plugins_dir.write_probe",
                            "Write probe succeeded in the Studio plugins directory.",
                        ));
                    }
                    Err(write_error) => findings.push(error(
                        "plugins_dir.write_probe",
                        &format!("Write probe failed in the Studio plugins directory: {write_error}"),
                    )),
                }
            }

            if let Some(lockfile) = &lockfile {
                for plugin in &lockfile.plugins {
                    if plugin.source == PluginSourceKind::Git {
                        if plugin.source_url.is_none() {
                            findings.push(error(
                                &format!("lockfile.git.{}.source_url", plugin.name),
                                &format!("Git lockfile entry `{}` is missing its source URL.", plugin.name),
                            ));
                        }
                        if plugin.git_resolved_commit.is_none() {
                            findings.push(error(
                                &format!("lockfile.git.{}.resolved_commit", plugin.name),
                                &format!("Git lockfile entry `{}` is missing its resolved commit.", plugin.name),
                            ));
                        }
                    }

                    let target_path = Path::new(&plugin.target_path);
                    if !target_path.exists() {
                        findings.push(error(
                            &format!("plugin.{}.missing", plugin.name),
                            &format!("Managed plugin `{}` is missing from disk.", plugin.name),
                        ));
                        continue;
                    }

                    let actual_checksum = checksum_path(target_path)?;
                    if actual_checksum == plugin.checksum {
                        findings.push(ok(
                            &format!("plugin.{}.checksum", plugin.name),
                            &format!("Installed plugin `{}` matches the recorded checksum.", plugin.name),
                        ));
                    } else {
                        findings.push(error(
                            &format!("plugin.{}.checksum", plugin.name),
                            &format!("Installed plugin `{}` has checksum drift.", plugin.name),
                        ));
                    }
                }
            }
        }
        Err(path_error) => findings.push(error("plugins_dir.access", &path_error.to_string())),
    }

    if let (Some(manifest), Some(lockfile)) = (&manifest, &lockfile) {
        let lock_names = lockfile.plugins.iter().map(|plugin| plugin.name.clone()).collect::<HashSet<_>>();
        for name in manifest.plugins.keys().filter(|name| !lock_names.contains(*name)) {
            findings.push(warn(
                &format!("lockfile.missing.{name}"),
                &format!("Manifest plugin `{name}` is not present in the lockfile yet."),
            ));
        }

        for locked in &lockfile.plugins {
            if !manifest.plugins.contains_key(&locked.name) {
                findings.push(warn(
                    &format!("lockfile.stale.{}", locked.name),
                    &format!(
                        "Lockfile entry `{}` is not present in the manifest. Suggested action: remove the stale lock entry or run install.",
                        locked.name
                    ),
                ));
            }
        }
    }

    if let Some(manifest) = &manifest {
        let mut target_paths = HashMap::new();
        for (name, spec) in &manifest.plugins {
            if spec.source == PluginSourceKind::Git && !git_available {
                findings.push(error(
                    &format!("source.git.{name}.git_missing"),
                    &format!("Git source for `{name}` cannot be resolved because Git is not available in PATH."),
                ));
                continue;
            }

            match resolve_from_manifest(project_dir, name, spec) {
                Ok(resolved) => {
                    if spec.source == PluginSourceKind::Git {
                        let commit = resolved.git_resolved_commit.as_deref().unwrap_or("unknown");
                        findings.push(ok(
                            &format!("source.git.{name}"),
                            &format!("Git source for `{name}` is readable at commit {commit}."),
                        ));
                    }
                    if let Some(existing) = target_paths.insert(resolved.install_name.clone(), name.clone()) {
                        findings.push(error(
                            &format!("targets.duplicate.{}", resolved.install_name),
                            &format!(
                                "Plugins `{existing}` and `{}` resolve to the same target `{}`.",
                                name, resolved.install_name
                            ),
                        ));
                    }
                }
                Err(error_message) => {
                    if spec.source == PluginSourceKind::Git {
                        findings.push(git_source_error(name, &error_message));
                    } else {
                        findings.push(error(
                            &format!("source.unreadable.{name}"),
                            &format!("Source for `{name}` is unreadable: {error_message}"),
                        ));
                    }
                }
            }
        }
    }

    let healthy = findings
        .iter()
        .all(|finding| !matches!(finding.level, FindingLevel::Error));
    Ok(DoctorReport { healthy, findings })
}

fn ok(code: &str, message: &str) -> DoctorFinding {
    DoctorFinding {
        level: FindingLevel::Ok,
        code: code.to_owned(),
        message: message.to_owned(),
    }
}

fn warn(code: &str, message: &str) -> DoctorFinding {
    DoctorFinding {
        level: FindingLevel::Warn,
        code: code.to_owned(),
        message: message.to_owned(),
    }
}

fn error(code: &str, message: &str) -> DoctorFinding {
    DoctorFinding {
        level: FindingLevel::Error,
        code: code.to_owned(),
        message: message.to_owned(),
    }
}

fn git_source_error(name: &str, error_value: &PyanpmError) -> DoctorFinding {
    let (suffix, message) = match error_value {
        PyanpmError::MissingGitExecutable => (
            "git_missing",
            format!("Git source for `{name}` cannot be resolved because Git is not available in PATH."),
        ),
        PyanpmError::GitCloneFailed { message, .. } => (
            "clone_failed",
            format!("Git source for `{name}` could not be cloned: {message}"),
        ),
        PyanpmError::GitFetchFailed { message, .. } => (
            "fetch_failed",
            format!("Git source for `{name}` could not be refreshed: {message}"),
        ),
        PyanpmError::GitCheckoutFailed { message, .. } => (
            "checkout_failed",
            format!("Git source for `{name}` could not be checked out: {message}"),
        ),
        PyanpmError::InvalidGitUrl(details) => (
            "invalid_url",
            format!("Git source for `{name}` has an invalid URL: {details}"),
        ),
        PyanpmError::InvalidGitRef(details) => (
            "invalid_ref",
            format!("Git source for `{name}` has an invalid ref: {details}"),
        ),
        PyanpmError::InvalidGitSubdir(details) => (
            "invalid_subdir",
            format!("Git source for `{name}` has an invalid subdirectory: {details}"),
        ),
        PyanpmError::MissingMetadata(path) => (
            "missing_metadata",
            format!(
                "Git source for `{name}` is missing `pyanpm.plugin.toml` at {}.",
                path.display()
            ),
        ),
        PyanpmError::InvalidMetadata(details) => (
            "invalid_metadata",
            format!("Git source for `{name}` has invalid metadata: {details}"),
        ),
        PyanpmError::UnsupportedArtifact(path) => (
            "unsupported_artifact",
            format!(
                "Git source for `{name}` points to an unsupported artifact at {}.",
                path.display()
            ),
        ),
        other => (
            "unreadable",
            format!("Git source for `{name}` is unreadable: {other}"),
        ),
    };

    error(&format!("source.git.{name}.{suffix}"), &message)
}
