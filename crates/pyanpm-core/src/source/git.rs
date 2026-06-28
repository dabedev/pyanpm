use std::path::{Component, Path, PathBuf};
use std::process::Command;

use directories::ProjectDirs;
use fs_err as fs;
use sha2::{Digest, Sha256};

use crate::config::{GitRefKind, GitSourceOptions};
use crate::error::{PyanpmError, Result};

#[derive(Debug, Clone)]
pub struct GitCheckout {
    pub package_root: PathBuf,
    pub resolved_commit: String,
}

pub fn checkout_git_source(url: &str, options: &GitSourceOptions) -> Result<GitCheckout> {
    ensure_git_available()?;
    let normalized_url = normalize_git_url(url)?;
    let normalized_options = options.normalized();
    let repository_root = repository_cache_root(&normalized_url, &normalized_options)?;

    if !repository_root.join(".git").exists() {
        if repository_root.exists() {
            fs::remove_dir_all(&repository_root)?;
        }
        if let Some(parent) = repository_root.parent() {
            fs::create_dir_all(parent)?;
        }
        run_git(
            None,
            ["clone", "--quiet", &normalized_url, &repository_root.display().to_string()],
            GitCommandError::Clone {
                url: normalized_url.clone(),
            },
        )?;
    } else {
        run_git(
            Some(&repository_root),
            ["fetch", "--prune", "--tags", "origin"],
            GitCommandError::Fetch {
                url: normalized_url.clone(),
            },
        )?;
    }

    match (normalized_options.git_ref_kind, normalized_options.git_ref.as_deref()) {
        (Some(GitRefKind::Branch), Some(branch)) => {
            run_git(
                Some(&repository_root),
                ["checkout", "--force", "-B", branch, &format!("origin/{branch}")],
                GitCommandError::Checkout {
                    url: normalized_url.clone(),
                },
            )?;
        }
        (Some(GitRefKind::Tag), Some(tag)) => {
            run_git(
                Some(&repository_root),
                ["checkout", "--force", "--detach", &format!("tags/{tag}")],
                GitCommandError::Checkout {
                    url: normalized_url.clone(),
                },
            )?;
        }
        (Some(GitRefKind::Commit), Some(commit)) => {
            let fetch_result = run_git(
                Some(&repository_root),
                ["fetch", "--tags", "origin", commit],
                GitCommandError::Fetch {
                    url: normalized_url.clone(),
                },
            );
            if fetch_result.is_err() {
                // Some remotes refuse targeted commit fetches; continue with the checkout attempt.
            }
            run_git(
                Some(&repository_root),
                ["checkout", "--force", "--detach", commit],
                GitCommandError::Checkout {
                    url: normalized_url.clone(),
                },
            )?;
        }
        (None, None) => {
            let _ = run_git(
                Some(&repository_root),
                ["pull", "--ff-only", "--quiet"],
                GitCommandError::Fetch {
                    url: normalized_url.clone(),
                },
            );
        }
        _ => {
            return Err(PyanpmError::InvalidGitRef(
                "Git reference kind and value must be provided together.".to_owned(),
            ));
        }
    }

    let resolved_commit = run_git(
        Some(&repository_root),
        ["rev-parse", "HEAD"],
        GitCommandError::Checkout { url: normalized_url },
    )?
    .trim()
    .to_owned();

    let package_root = resolve_package_root(&repository_root, normalized_options.git_subdir.as_deref())?;
    Ok(GitCheckout {
        package_root,
        resolved_commit,
    })
}

pub fn normalize_git_url(url: &str) -> Result<String> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return Err(PyanpmError::InvalidGitUrl("repository URL is empty".to_owned()));
    }

    if trimmed.contains('\n') || trimmed.contains('\r') {
        return Err(PyanpmError::InvalidGitUrl(
            "repository URL must be a single line".to_owned(),
        ));
    }

    if !(trimmed.contains("://") || trimmed.starts_with("git@")) {
        return Err(PyanpmError::InvalidGitUrl(
            "repository URL must use HTTPS, SSH, or Git transport syntax".to_owned(),
        ));
    }

    Ok(trimmed.to_owned())
}

pub fn validate_git_options(options: &GitSourceOptions) -> Result<GitSourceOptions> {
    let normalized = options.normalized();
    if normalized.is_empty() {
        return Ok(normalized);
    }

    match (normalized.git_ref_kind, normalized.git_ref.as_deref()) {
        (Some(_), Some(reference)) if !reference.trim().is_empty() => {}
        (None, None) => {}
        _ => {
            return Err(PyanpmError::InvalidGitRef(
                "Git reference kind and value must both be set.".to_owned(),
            ));
        }
    }

    if let Some(subdir) = normalized.git_subdir.as_deref() {
        validate_subdir(subdir)?;
    }

    Ok(normalized)
}

pub(crate) fn ensure_git_available() -> Result<()> {
    let status = Command::new("git").arg("--version").status();
    match status {
        Ok(result) if result.success() => Ok(()),
        Ok(_) | Err(_) => Err(PyanpmError::MissingGitExecutable),
    }
}

fn repository_cache_root(url: &str, options: &GitSourceOptions) -> Result<PathBuf> {
    let project_dirs = ProjectDirs::from("dev", "bblocks", "pyanpm")
        .ok_or(PyanpmError::MissingDefaultCacheDir)?;
    let mut hasher = Sha256::new();
    hasher.update(url.as_bytes());
    hasher.update(
        options
            .git_ref_kind
            .map(|kind| match kind {
                GitRefKind::Branch => "branch",
                GitRefKind::Tag => "tag",
                GitRefKind::Commit => "commit",
            })
            .unwrap_or("default")
            .as_bytes(),
    );
    hasher.update(options.git_ref.as_deref().unwrap_or("").as_bytes());
    let key = hex::encode(hasher.finalize());
    Ok(project_dirs.cache_dir().join("git-sources").join(key))
}

fn resolve_package_root(repository_root: &Path, subdir: Option<&str>) -> Result<PathBuf> {
    let Some(subdir) = subdir else {
        return Ok(repository_root.to_path_buf());
    };

    validate_subdir(subdir)?;
    let package_root = repository_root.join(subdir);
    if !package_root.is_dir() {
        return Err(PyanpmError::InvalidGitSubdir(subdir.to_owned()));
    }
    Ok(package_root)
}

fn validate_subdir(subdir: &str) -> Result<()> {
    let path = Path::new(subdir);
    if path.is_absolute() {
        return Err(PyanpmError::InvalidGitSubdir(subdir.to_owned()));
    }

    for component in path.components() {
        match component {
            Component::Normal(_) => {}
            Component::CurDir => {}
            _ => return Err(PyanpmError::InvalidGitSubdir(subdir.to_owned())),
        }
    }

    Ok(())
}

fn run_git<const N: usize>(cwd: Option<&Path>, args: [&str; N], error_context: GitCommandError) -> Result<String> {
    let mut command = Command::new("git");
    command.args(args);
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }

    let output = command
        .output()
        .map_err(|_| error_context.clone().into_error("failed to start git".to_owned()))?;

    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned());
    }

    let message = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    Err(error_context.into_error(message))
}

#[derive(Debug, Clone)]
enum GitCommandError {
    Clone { url: String },
    Fetch { url: String },
    Checkout { url: String },
}

impl GitCommandError {
    fn into_error(self, message: String) -> PyanpmError {
        match self {
            GitCommandError::Clone { url } => PyanpmError::GitCloneFailed { url, message },
            GitCommandError::Fetch { url } => PyanpmError::GitFetchFailed { url, message },
            GitCommandError::Checkout { url } => PyanpmError::GitCheckoutFailed { url, message },
        }
    }
}
