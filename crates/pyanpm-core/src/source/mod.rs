use std::path::{Path, PathBuf};

use pathdiff::diff_paths;

use crate::cache::checksum_path;
use crate::config::{GitSourceOptions, LockedPlugin, ManifestPluginSpec, PackageMetadata};
use crate::error::{PyanpmError, Result};
use crate::operations::{ValidationIssue, ValidationSeverity};
use crate::plugin::{infer_file_plugin_name, validate_plugin_name, ArtifactKind, PluginSourceKind, ResolvedPlugin};

mod git;

pub(crate) use git::ensure_git_available;
use git::{checkout_git_source, normalize_git_url, validate_git_options};

#[derive(Debug, Clone)]
pub struct ParsedPluginRef {
    pub spec: ManifestPluginSpec,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct SourceValidation {
    pub normalized_source_ref: Option<String>,
    pub plugin_name: Option<String>,
    pub errors: Vec<ValidationIssue>,
}

pub fn parse_plugin_ref(
    project_dir: &Path,
    reference: &str,
    version: Option<String>,
    git_options: &GitSourceOptions,
) -> Result<ParsedPluginRef> {
    let normalized_git_options = validate_git_options(git_options)?;
    if let Some(raw_path) = reference.strip_prefix("file:") {
        reject_git_options_for_non_git(&normalized_git_options)?;
        let source_path = absolute_path(project_dir, Path::new(raw_path))?;
        let name = infer_file_plugin_name(&source_path)?;
        return Ok(ParsedPluginRef {
            name,
            spec: ManifestPluginSpec {
                source: PluginSourceKind::File,
                path: Some(relativize_or_display(&source_path, project_dir)),
                url: None,
                version,
                git_ref_kind: None,
                git_ref: None,
                subdir: None,
            },
        });
    }

    if let Some(raw_path) = reference.strip_prefix("path:") {
        reject_git_options_for_non_git(&normalized_git_options)?;
        let source_path = absolute_path(project_dir, Path::new(raw_path))?;
        let metadata = PackageMetadata::read_from(&source_path)?;
        validate_plugin_name(&metadata.name)?;
        return Ok(ParsedPluginRef {
            name: metadata.name,
            spec: ManifestPluginSpec {
                source: PluginSourceKind::Path,
                path: Some(relativize_or_display(&source_path, project_dir)),
                url: None,
                version,
                git_ref_kind: None,
                git_ref: None,
                subdir: None,
            },
        });
    }

    if let Some(raw_url) = reference.strip_prefix("git:") {
        let url = normalize_git_url(raw_url)?;
        let checkout = checkout_git_source(&url, &normalized_git_options)?;
        let metadata = PackageMetadata::read_from(&checkout.package_root)?;
        validate_plugin_name(&metadata.name)?;
        return Ok(ParsedPluginRef {
            name: metadata.name,
            spec: ManifestPluginSpec {
                source: PluginSourceKind::Git,
                path: None,
                url: Some(url),
                version,
                git_ref_kind: normalized_git_options.git_ref_kind,
                git_ref: normalized_git_options.git_ref.clone(),
                subdir: normalized_git_options.git_subdir.clone(),
            },
        });
    }

    Err(PyanpmError::InvalidPluginRef(reference.to_owned()))
}

pub fn validate_source_ref(
    project_dir: &Path,
    reference: &str,
    version: Option<String>,
    git_options: &GitSourceOptions,
) -> SourceValidation {
    let mut errors = Vec::new();
    if reference.trim().is_empty() {
        errors.push(validation_error(
            "sourceRef",
            reference,
            "source.empty",
            "Source reference is empty.",
            Some("Use a `file:`, `path:`, or `git:` source."),
        ));
        return SourceValidation {
            normalized_source_ref: None,
            plugin_name: None,
            errors,
        };
    }

    match parse_plugin_ref(project_dir, reference, version, git_options) {
        Ok(parsed) => SourceValidation {
            normalized_source_ref: Some(source_ref_from_spec(&parsed.spec)),
            plugin_name: Some(parsed.name),
            errors,
        },
        Err(error) => {
            let (code, message, suggested_fix): (&str, String, Option<&str>) = if let Some(raw_path) = reference.strip_prefix("file:") {
                if raw_path.trim().is_empty() {
                    (
                        "source.file.empty_path",
                        "File source is missing a path.".to_owned(),
                        Some("Pick an `.rbxm` or `.rbxmx` file."),
                    )
                } else {
                    (
                        "source.file.invalid",
                        error.to_string(),
                        Some("Verify that the file exists and has a supported Roblox plugin extension."),
                    )
                }
            } else if let Some(raw_path) = reference.strip_prefix("path:") {
                if raw_path.trim().is_empty() {
                    (
                        "source.path.empty_path",
                        "Path source is missing a folder path.".to_owned(),
                        Some("Pick a readable folder containing `pyanpm.plugin.toml`."),
                    )
                } else {
                    (
                        "source.path.invalid",
                        error.to_string(),
                        Some("Verify that the folder exists and contains valid plugin metadata."),
                    )
                }
            } else if let Some(raw_url) = reference.strip_prefix("git:") {
                if raw_url.trim().is_empty() {
                    (
                        "source.git.empty_url",
                        "Git source is missing a repository URL.".to_owned(),
                        Some("Enter an HTTPS or SSH Git repository URL."),
                    )
                } else {
                    (
                        "source.git.invalid",
                        error.to_string(),
                        Some("Verify the repository URL, ref fields, credentials, and plugin metadata."),
                    )
                }
            } else {
                (
                    "source.prefix.unsupported",
                    "Unsupported source reference.".to_owned(),
                    Some("Use a `file:`, `path:`, or `git:` prefix."),
                )
            };

            errors.push(validation_error("sourceRef", reference, code, &message, suggested_fix));
            SourceValidation {
                normalized_source_ref: None,
                plugin_name: None,
                errors,
            }
        }
    }
}

pub fn resolve_from_manifest(project_dir: &Path, name: &str, spec: &ManifestPluginSpec) -> Result<ResolvedPlugin> {
    validate_plugin_name(name)?;

    match spec.source {
        PluginSourceKind::File => resolve_file_source(project_dir, name, spec),
        PluginSourceKind::Path => resolve_path_source(project_dir, name, spec),
        PluginSourceKind::Git => resolve_git_source(name, spec),
    }
}

pub fn resolve_from_lock(project_dir: &Path, locked: &LockedPlugin) -> Result<ResolvedPlugin> {
    let spec = ManifestPluginSpec {
        source: locked.source,
        path: (locked.source != PluginSourceKind::Git).then(|| locked.source_path.clone()),
        url: if locked.source == PluginSourceKind::Git {
            locked.source_url.clone().or_else(|| Some(locked.source_path.clone()))
        } else {
            None
        },
        version: locked.requested_version.clone(),
        git_ref_kind: locked.git_ref_kind,
        git_ref: locked.git_requested_ref.clone(),
        subdir: locked.source_subdir.clone(),
    };
    resolve_from_manifest(project_dir, &locked.name, &spec)
}

fn resolve_file_source(project_dir: &Path, name: &str, spec: &ManifestPluginSpec) -> Result<ResolvedPlugin> {
    let artifact_path = spec_path(project_dir, spec)?;
    if !artifact_path.is_file() {
        return Err(PyanpmError::UnsupportedArtifact(artifact_path));
    }

    let extension = artifact_extension(&artifact_path)?;
    let checksum = checksum_path(&artifact_path)?;
    Ok(ResolvedPlugin {
        name: name.to_owned(),
        requested_version: spec.version.clone(),
        resolved_version: spec.version.clone(),
        source: PluginSourceKind::File,
        source_path: artifact_path.clone(),
        source_url: None,
        source_subdir: None,
        git_ref_kind: None,
        git_requested_ref: None,
        git_resolved_commit: None,
        artifact_path,
        artifact_kind: ArtifactKind::File,
        checksum,
        install_name: format!("{name}.{extension}"),
    })
}

fn resolve_path_source(project_dir: &Path, name: &str, spec: &ManifestPluginSpec) -> Result<ResolvedPlugin> {
    let source_dir = spec_path(project_dir, spec)?;
    if !source_dir.is_dir() {
        return Err(PyanpmError::UnsupportedArtifact(source_dir));
    }

    let metadata = PackageMetadata::read_from(&source_dir)?;
    if metadata.name != name {
        return Err(PyanpmError::InvalidMetadata(format!(
            "manifest plugin key `{name}` does not match metadata name `{}`",
            metadata.name
        )));
    }

    let artifact_path = source_dir.join(&metadata.entry);
    if !artifact_path.exists() {
        return Err(PyanpmError::UnsupportedArtifact(artifact_path));
    }

    let artifact_kind = if artifact_path.is_dir() {
        ArtifactKind::Directory
    } else {
        ArtifactKind::File
    };
    let checksum = checksum_path(&artifact_path)?;
    let install_name = match artifact_kind {
        ArtifactKind::Directory => name.to_owned(),
        ArtifactKind::File => format!("{name}.{}", artifact_extension(&artifact_path)?),
    };

    Ok(ResolvedPlugin {
        name: name.to_owned(),
        requested_version: spec.version.clone(),
        resolved_version: metadata.version.or_else(|| spec.version.clone()),
        source: PluginSourceKind::Path,
        source_path: source_dir,
        source_url: None,
        source_subdir: None,
        git_ref_kind: None,
        git_requested_ref: None,
        git_resolved_commit: None,
        artifact_path,
        artifact_kind,
        checksum,
        install_name,
    })
}

fn resolve_git_source(name: &str, spec: &ManifestPluginSpec) -> Result<ResolvedPlugin> {
    let url = spec
        .url
        .as_deref()
        .ok_or_else(|| PyanpmError::InvalidPluginRef("missing git URL".to_owned()))?;
    let git_options = GitSourceOptions {
        git_ref_kind: spec.git_ref_kind,
        git_ref: spec.git_ref.clone(),
        git_subdir: spec.subdir.clone(),
    };
    let checkout = checkout_git_source(url, &git_options)?;
    let metadata = PackageMetadata::read_from(&checkout.package_root)?;
    if metadata.name != name {
        return Err(PyanpmError::InvalidMetadata(format!(
            "manifest plugin key `{name}` does not match metadata name `{}`",
            metadata.name
        )));
    }

    let artifact_path = checkout.package_root.join(&metadata.entry);
    if !artifact_path.exists() {
        return Err(PyanpmError::UnsupportedArtifact(artifact_path));
    }

    let artifact_kind = if artifact_path.is_dir() {
        ArtifactKind::Directory
    } else {
        ArtifactKind::File
    };
    let checksum = checksum_path(&artifact_path)?;
    let install_name = match artifact_kind {
        ArtifactKind::Directory => name.to_owned(),
        ArtifactKind::File => format!("{name}.{}", artifact_extension(&artifact_path)?),
    };

    Ok(ResolvedPlugin {
        name: name.to_owned(),
        requested_version: spec.version.clone(),
        resolved_version: metadata.version.or_else(|| spec.version.clone()),
        source: PluginSourceKind::Git,
        source_path: checkout.package_root,
        source_url: Some(url.to_owned()),
        source_subdir: spec.subdir.clone(),
        git_ref_kind: spec.git_ref_kind,
        git_requested_ref: spec.git_ref.clone(),
        git_resolved_commit: Some(checkout.resolved_commit),
        artifact_path,
        artifact_kind,
        checksum,
        install_name,
    })
}

fn spec_path(project_dir: &Path, spec: &ManifestPluginSpec) -> Result<PathBuf> {
    let raw_path = spec
        .path
        .as_deref()
        .ok_or_else(|| PyanpmError::InvalidPluginRef("missing path field".to_owned()))?;
    absolute_path(project_dir, Path::new(raw_path))
}

pub fn source_ref_from_spec(spec: &ManifestPluginSpec) -> String {
    match spec.source {
        PluginSourceKind::File => spec
            .path
            .as_ref()
            .map(|path| format!("file:{path}"))
            .unwrap_or_else(|| "file".to_owned()),
        PluginSourceKind::Path => spec
            .path
            .as_ref()
            .map(|path| format!("path:{path}"))
            .unwrap_or_else(|| "path".to_owned()),
        PluginSourceKind::Git => spec
            .url
            .as_ref()
            .map(|url| {
                let mut source = format!("git:{url}");
                if let Some(kind) = spec.git_ref_kind {
                    let kind = match kind {
                        crate::config::GitRefKind::Branch => "branch",
                        crate::config::GitRefKind::Tag => "tag",
                        crate::config::GitRefKind::Commit => "commit",
                    };
                    if let Some(reference) = spec.git_ref.as_deref() {
                        source.push_str(&format!(" [{kind}:{reference}]"));
                    }
                }
                if let Some(subdir) = spec.subdir.as_deref() {
                    source.push_str(&format!(" [{subdir}]"));
                }
                source
            })
            .unwrap_or_else(|| "git".to_owned()),
    }
}

fn reject_git_options_for_non_git(git_options: &GitSourceOptions) -> Result<()> {
    if git_options.is_empty() {
        Ok(())
    } else {
        Err(PyanpmError::InvalidGitRef(
            "Git ref fields can only be used with `git:` sources.".to_owned(),
        ))
    }
}

fn absolute_path(project_dir: &Path, raw_path: &Path) -> Result<PathBuf> {
    let absolute = if raw_path.is_absolute() {
        raw_path.to_path_buf()
    } else {
        project_dir.join(raw_path)
    };

    Ok(absolute.canonicalize()?)
}

fn relativize_or_display(path: &Path, project_dir: &Path) -> String {
    diff_paths(path, project_dir)
        .unwrap_or_else(|| path.to_path_buf())
        .to_string_lossy()
        .replace('\\', "/")
}

fn artifact_extension(path: &Path) -> Result<String> {
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.to_ascii_lowercase())
        .ok_or_else(|| PyanpmError::UnsupportedArtifact(path.to_path_buf()))?;

    if extension == "rbxm" || extension == "rbxmx" {
        Ok(extension)
    } else {
        Err(PyanpmError::UnsupportedArtifact(path.to_path_buf()))
    }
}

fn validation_error(
    field: &str,
    source_ref: &str,
    code: &str,
    message: &str,
    suggested_fix: Option<&str>,
) -> ValidationIssue {
    ValidationIssue {
        field: field.to_owned(),
        source_ref: source_ref.to_owned(),
        code: code.to_owned(),
        severity: ValidationSeverity::Error,
        message: message.to_owned(),
        suggested_fix: suggested_fix.map(str::to_owned),
    }
}
