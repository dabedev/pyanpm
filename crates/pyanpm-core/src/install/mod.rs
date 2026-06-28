use std::path::{Path, PathBuf};

use fs_err as fs;
use uuid::Uuid;

use crate::cache::{copy_path, CachedArtifact};
use crate::error::{PyanpmError, Result};
use crate::plugin::{validate_plugin_name, ResolvedPlugin};

pub fn resolve_plugins_dir(override_dir: Option<&Path>) -> Result<PathBuf> {
    let plugins_dir = if let Some(path) = override_dir {
        path.to_path_buf()
    } else if cfg!(target_os = "windows") {
        std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .map(|base| base.join("Roblox").join("Plugins"))
            .ok_or(PyanpmError::MissingDefaultPluginsDir)?
    } else {
        return Err(PyanpmError::MissingDefaultPluginsDir);
    };

    fs::create_dir_all(&plugins_dir)?;
    Ok(plugins_dir.canonicalize()?)
}

pub fn install_cached_artifact(
    plugins_dir: &Path,
    resolved: &ResolvedPlugin,
    cached: &CachedArtifact,
) -> Result<PathBuf> {
    validate_plugin_name(&resolved.name)?;

    let target_path = plugins_dir.join(&resolved.install_name);
    ensure_within_root(plugins_dir, &target_path)?;

    let stage_root = plugins_dir.join(format!(".pyanpm-stage-{}", Uuid::new_v4()));
    let backup_path = plugins_dir.join(format!(".pyanpm-backup-{}", Uuid::new_v4()));
    let staged_target = stage_root.join(&resolved.install_name);

    fs::create_dir_all(&stage_root)?;
    copy_path(&cached.stored_path, &staged_target)?;

    let existing_target = target_path.exists();
    if existing_target {
        fs::rename(&target_path, &backup_path)?;
    }

    match fs::rename(&staged_target, &target_path) {
        Ok(()) => {
            remove_if_exists(&stage_root)?;
            remove_if_exists(&backup_path)?;
            Ok(target_path)
        }
        Err(error) => {
            remove_if_exists(&staged_target)?;
            remove_if_exists(&stage_root)?;
            if existing_target && backup_path.exists() {
                fs::rename(&backup_path, &target_path)?;
            }
            Err(error.into())
        }
    }
}

fn ensure_within_root(root: &Path, target: &Path) -> Result<()> {
    let canonical_root = root.canonicalize()?;
    let parent = target
        .parent()
        .ok_or_else(|| PyanpmError::InstallTargetEscapes(target.to_path_buf()))?;
    fs::create_dir_all(parent)?;
    let canonical_parent = parent.canonicalize()?;
    if canonical_parent.starts_with(&canonical_root) {
        Ok(())
    } else {
        Err(PyanpmError::InstallTargetEscapes(target.to_path_buf()))
    }
}

fn remove_if_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }

    if path.is_dir() {
        fs::remove_dir_all(path)?;
    } else {
        fs::remove_file(path)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    #[test]
    fn builds_windows_plugins_path() {
        let base = Path::new(r"C:\Users\Test\AppData\Local");
        let path = base.join("Roblox").join("Plugins");
        assert_eq!(path, Path::new(r"C:\Users\Test\AppData\Local\Roblox\Plugins"));
    }
}
