use std::collections::{BTreeSet, HashSet};
use std::path::{Path, PathBuf};

use chrono::Utc;
use fs_err as fs;
use serde::Serialize;
use uuid::Uuid;

use crate::activity::ActivityStore;
use crate::audit::{run_doctor, DoctorReport};
use crate::cache::{checksum_path, CacheStore};
use crate::config::{global_state_dir, GitRefKind, GitSourceOptions, Lockfile, LockedPlugin, Manifest, ManifestPluginSpec, MANIFEST_FILE_NAME};
use crate::error::{PyanpmError, Result};
use crate::install::{install_cached_artifact, resolve_plugins_dir};
use crate::operations::{
    ActivityClearResult, ActivityListResult, ActivityRecord, ActivityShowResult, ActivityStatus,
    CacheInfoResult, CacheListResult, CacheMutationResult, CommandMeta, CompletionSummary, DiffResult,
    InitResult, InstallResult, InstalledPluginSummary, ListResult, ListedPlugin, PluginStateKind,
    PluginStateSnapshot, ProgressEvent, ProgressSeverity, RemoveResult, RemovedPluginSummary, UpdateCandidate,
    UpdateResult, ValidateSourceResult,
};
use crate::source::{
    parse_plugin_ref, resolve_from_lock, resolve_from_manifest, source_ref_from_spec, validate_source_ref,
};

pub const API_VERSION: &str = "1";

#[derive(Debug, Clone)]
pub struct PyanpmService {
    state_dir: PathBuf,
    plugins_dir_override: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonEnvelope<T>
where
    T: Serialize,
{
    pub api_version: &'static str,
    pub command: &'static str,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DoctorResult {
    pub meta: CommandMeta,
    pub report: DoctorReport,
    pub completion: CompletionSummary,
}

#[derive(Debug)]
struct OperationSession {
    command_id: String,
    state_path: String,
    command_name: String,
    command_args_summary: String,
    started_at: chrono::DateTime<Utc>,
    plugin_names: Vec<String>,
    progress: Vec<ProgressEvent>,
    persist_activity: bool,
}

impl OperationSession {
    fn new(
        state_dir: &Path,
        command_name: &str,
        command_args_summary: String,
        plugin_names: Vec<String>,
        persist_activity: bool,
    ) -> Self {
        Self {
            command_id: Uuid::new_v4().to_string(),
            state_path: canonicalize_or_display(state_dir),
            command_name: command_name.to_owned(),
            command_args_summary,
            started_at: Utc::now(),
            plugin_names,
            progress: Vec::new(),
            persist_activity,
        }
    }

    fn emit(
        &mut self,
        plugin_name: Option<&str>,
        stage: &str,
        message: impl Into<String>,
        current: Option<u32>,
        total: Option<u32>,
        severity: ProgressSeverity,
    ) {
        self.progress.push(ProgressEvent {
            command_id: self.command_id.clone(),
            state_path: self.state_path.clone(),
            plugin_name: plugin_name.map(str::to_owned),
            stage: stage.to_owned(),
            message: message.into(),
            current,
            total,
            severity,
            timestamp: Utc::now(),
        });
    }

    fn success(
        &self,
        summary: String,
        affected_plugin_count: usize,
        warning_count: usize,
        error_count: usize,
    ) -> Result<(CommandMeta, CompletionSummary)> {
        let completion = CompletionSummary {
            affected_plugin_count,
            warning_count,
            error_count,
            duration_ms: (Utc::now() - self.started_at).num_milliseconds(),
            summary: summary.clone(),
        };
        let activity_id = if self.persist_activity {
            let record = ActivityRecord {
                id: Uuid::new_v4().to_string(),
                command_id: self.command_id.clone(),
                state_path: self.state_path.clone(),
                command_name: self.command_name.clone(),
                command_args_summary: self.command_args_summary.clone(),
                started_at: self.started_at,
                finished_at: Utc::now(),
                duration_ms: completion.duration_ms,
                status: if error_count > 0 {
                    ActivityStatus::Partial
                } else {
                    ActivityStatus::Success
                },
                plugin_names: self.plugin_names.clone(),
                summary,
                error_code: None,
                error_message: None,
                error_stage: None,
            };
            ActivityStore::new()?.append(&record)?;
            Some(record.id)
        } else {
            None
        };
        Ok((
            CommandMeta {
                command_id: self.command_id.clone(),
                state_path: self.state_path.clone(),
                activity_id,
            },
            completion,
        ))
    }

    fn failure(&self, error: &PyanpmError, error_stage: Option<&str>) -> Result<()> {
        if !self.persist_activity {
            return Ok(());
        }
        let finished_at = Utc::now();
        let record = ActivityRecord {
            id: Uuid::new_v4().to_string(),
            command_id: self.command_id.clone(),
            state_path: self.state_path.clone(),
            command_name: self.command_name.clone(),
            command_args_summary: self.command_args_summary.clone(),
            started_at: self.started_at,
            finished_at,
            duration_ms: (finished_at - self.started_at).num_milliseconds(),
            status: ActivityStatus::Failed,
            plugin_names: self.plugin_names.clone(),
            summary: error.to_string(),
            error_code: Some(error_code(error)),
            error_message: Some(error.to_string()),
            error_stage: error_stage.map(str::to_owned),
        };
        ActivityStore::new()?.append(&record)?;
        Ok(())
    }

    fn meta(&self) -> CommandMeta {
        CommandMeta {
            command_id: self.command_id.clone(),
            state_path: self.state_path.clone(),
            activity_id: None,
        }
    }
}

impl PyanpmService {
    pub fn new(state_dir: impl Into<PathBuf>, plugins_dir_override: Option<PathBuf>) -> Self {
        Self {
            state_dir: state_dir.into(),
            plugins_dir_override,
        }
    }

    pub fn global(plugins_dir_override: Option<PathBuf>) -> Result<Self> {
        Ok(Self::new(global_state_dir()?, plugins_dir_override))
    }

    pub fn state_dir(&self) -> &Path {
        &self.state_dir
    }

    pub fn init(&self, force: bool) -> Result<InitResult> {
        let session = OperationSession::new(
            &self.state_dir,
            "init",
            format!("force={force}"),
            Vec::new(),
            true,
        );
        let result = (|| {
            let manifest_path = Manifest::path_in(&self.state_dir);
            if manifest_path.exists() && !force {
                return Err(PyanpmError::ManifestExists(manifest_path));
            }

            Manifest::default().save(&self.state_dir)?;
            let (meta, completion) =
                session.success("Initialized the global pyanPM manifest.".to_owned(), 0, 0, 0)?;
            Ok(InitResult {
                meta,
                manifest_path: manifest_path.display().to_string(),
                completion,
            })
        })();
        if let Err(error) = &result {
            let _ = session.failure(error, Some("manifest"));
        }
        result
    }

    pub fn add(&self, plugin_ref: &str, version: Option<String>, git_options: GitSourceOptions) -> Result<InstallResult> {
        let mut manifest = Manifest::load(&self.state_dir)?;
        let parsed = parse_plugin_ref(&self.state_dir, plugin_ref, version, &git_options)?;
        if manifest.plugins.contains_key(&parsed.name) {
            return Err(PyanpmError::PluginAlreadyExists(parsed.name));
        }
        manifest.plugins.insert(parsed.name.clone(), parsed.spec);
        manifest.save(&self.state_dir)?;
        self.install_selected(
            vec![parsed.name],
            "add",
            format_source_args(plugin_ref, &git_options),
        )
    }

    pub fn install(&self) -> Result<InstallResult> {
        let manifest = Manifest::load(&self.state_dir)?;
        self.install_manifest_specs(
            manifest.plugins.into_iter().collect(),
            "install",
            "install all managed plugins".to_owned(),
            true,
        )
    }

    pub fn list(&self) -> Result<ListResult> {
        let session = OperationSession::new(&self.state_dir, "list", String::new(), Vec::new(), false);
        let manifest = Manifest::load_optional(&self.state_dir)?.unwrap_or_default();
        let lockfile = Lockfile::load_optional(&self.state_dir)?.unwrap_or_default();

        let names = manifest
            .plugins
            .keys()
            .chain(lockfile.plugins.iter().map(|plugin| &plugin.name))
            .cloned()
            .collect::<BTreeSet<_>>();

        let plugins = names
            .into_iter()
            .map(|name| {
                let manifest_spec = manifest.plugins.get(&name);
                let locked = lockfile.plugins.iter().find(|plugin| plugin.name == name);
                ListedPlugin {
                    name: name.clone(),
                    requested_version: manifest_spec.and_then(|spec| spec.version.clone()),
                    installed_version: locked.and_then(|plugin| plugin.resolved_version.clone()),
                    source: manifest_spec
                        .map(source_ref_from_spec)
                        .or_else(|| locked.map(|plugin| format!("{:?}", plugin.source).to_ascii_lowercase()))
                        .unwrap_or_else(|| "unknown".to_owned()),
                    status: match locked {
                        Some(plugin) if Path::new(&plugin.target_path).exists() => "installed".to_owned(),
                        Some(_) => "missing".to_owned(),
                        None => "manifest-only".to_owned(),
                    },
                    target_path: locked.map(|plugin| plugin.target_path.clone()),
                }
            })
            .collect();

        Ok(ListResult {
            meta: session.meta(),
            plugins,
        })
    }

    pub fn doctor(&self) -> Result<DoctorResult> {
        self.doctor_with_options(false)
    }

    pub fn doctor_with_options(&self, write_probe: bool) -> Result<DoctorResult> {
        let session = OperationSession::new(
            &self.state_dir,
            "doctor",
            format!("write_probe={write_probe}"),
            Vec::new(),
            true,
        );
        let result = (|| {
            let report = run_doctor(&self.state_dir, self.plugins_dir_override.as_deref(), write_probe)?;
            let warning_count = report
                .findings
                .iter()
                .filter(|finding| matches!(finding.level, crate::audit::FindingLevel::Warn))
                .count();
            let error_count = report
                .findings
                .iter()
                .filter(|finding| matches!(finding.level, crate::audit::FindingLevel::Error))
                .count();
            let (meta, completion) = session.success(
                if report.healthy {
                    "Doctor completed without errors.".to_owned()
                } else {
                    "Doctor completed with findings.".to_owned()
                },
                0,
                warning_count,
                error_count,
            )?;
            Ok(DoctorResult { meta, report, completion })
        })();
        if let Err(error) = &result {
            let _ = session.failure(error, Some("doctor"));
        }
        result
    }

    pub fn validate_source(&self, source_ref: &str, git_options: GitSourceOptions) -> Result<ValidateSourceResult> {
        let session = OperationSession::new(
            &self.state_dir,
            "validate-source",
            format_source_args(source_ref, &git_options),
            Vec::new(),
            true,
        );
        let validation = validate_source_ref(&self.state_dir, source_ref, None, &git_options);
        let summary = if validation.errors.is_empty() {
            "Source validation passed."
        } else {
            "Source validation reported errors."
        };
        let (meta, _completion) = session.success(
            summary.to_owned(),
            validation.plugin_name.as_ref().map(|_| 1).unwrap_or(0),
            0,
            validation.errors.len(),
        )?;
        Ok(ValidateSourceResult {
            meta,
            valid: validation.errors.is_empty(),
            normalized_source_ref: validation.normalized_source_ref,
            plugin_name: validation.plugin_name,
            errors: validation.errors,
        })
    }

    pub fn diff(&self) -> Result<DiffResult> {
        let session = OperationSession::new(&self.state_dir, "diff", String::new(), Vec::new(), false);
        let manifest = Manifest::load_optional(&self.state_dir)?.unwrap_or_default();
        let lockfile = Lockfile::load_optional(&self.state_dir)?.unwrap_or_default();
        let activity = ActivityStore::new()?;

        let names = manifest
            .plugins
            .keys()
            .chain(lockfile.plugins.iter().map(|plugin| &plugin.name))
            .cloned()
            .collect::<BTreeSet<_>>();

        let mut snapshots = Vec::new();
        for name in names {
            let manifest_spec = manifest.plugins.get(&name);
            let locked = lockfile.plugins.iter().find(|plugin| plugin.name == name);
            let latest_activity = activity.latest_for_plugin(&name)?;

            let snapshot = match (manifest_spec, locked) {
                (Some(spec), None) => PluginStateSnapshot {
                    plugin_name: name.clone(),
                    manifest_source: Some(source_ref_from_spec(spec)),
                    manifest_version_constraint: spec.version.clone(),
                    lock_version: None,
                    lock_checksum: None,
                    installed_path: None,
                    installed_checksum: None,
                    state: PluginStateKind::ManifestOnly,
                    recommended_action: "install".to_owned(),
                    last_activity_id: latest_activity.map(|record| record.id),
                },
                (None, Some(locked)) => PluginStateSnapshot {
                    plugin_name: name.clone(),
                    manifest_source: None,
                    manifest_version_constraint: None,
                    lock_version: locked.resolved_version.clone(),
                    lock_checksum: Some(locked.checksum.clone()),
                    installed_path: Some(locked.target_path.clone()),
                    installed_checksum: installed_checksum(Path::new(&locked.target_path))?,
                    state: PluginStateKind::LockOnly,
                    recommended_action: "remove-stale-lock-entry".to_owned(),
                    last_activity_id: latest_activity.map(|record| record.id),
                },
                (Some(spec), Some(locked)) => {
                    match resolve_from_manifest(&self.state_dir, &name, spec) {
                        Ok(resolved) => {
                            let installed_checksum = installed_checksum(Path::new(&locked.target_path))?;
                            let (state, recommended_action) = if installed_checksum.is_none() {
                                (PluginStateKind::MissingInstall, "install".to_owned())
                            } else if installed_checksum.as_deref() != Some(locked.checksum.as_str()) {
                                (PluginStateKind::ChecksumDrift, "reinstall".to_owned())
                            } else if resolved.checksum != locked.checksum {
                                (PluginStateKind::StaleInstall, "update".to_owned())
                            } else {
                                (PluginStateKind::Synced, "none".to_owned())
                            };
                            PluginStateSnapshot {
                                plugin_name: name.clone(),
                                manifest_source: Some(source_ref_from_spec(spec)),
                                manifest_version_constraint: spec.version.clone(),
                                lock_version: locked.resolved_version.clone(),
                                lock_checksum: Some(locked.checksum.clone()),
                                installed_path: Some(locked.target_path.clone()),
                                installed_checksum,
                                state,
                                recommended_action,
                                last_activity_id: latest_activity.map(|record| record.id),
                            }
                        }
                        Err(_) => PluginStateSnapshot {
                            plugin_name: name.clone(),
                            manifest_source: Some(source_ref_from_spec(spec)),
                            manifest_version_constraint: spec.version.clone(),
                            lock_version: locked.resolved_version.clone(),
                            lock_checksum: Some(locked.checksum.clone()),
                            installed_path: Some(locked.target_path.clone()),
                            installed_checksum: installed_checksum(Path::new(&locked.target_path))?,
                            state: PluginStateKind::SourceUnreadable,
                            recommended_action: "inspect-source".to_owned(),
                            last_activity_id: latest_activity.map(|record| record.id),
                        },
                    }
                }
                (None, None) => continue,
            };
            snapshots.push(snapshot);
        }

        let managed_targets = lockfile
            .plugins
            .iter()
            .map(|plugin| canonicalize_or_display(Path::new(&plugin.target_path)))
            .collect::<HashSet<_>>();
        if let Ok(plugins_dir) = resolve_plugins_dir(self.plugins_dir_override.as_deref()) {
            for entry in fs::read_dir(plugins_dir)? {
                let entry = entry?;
                let path = entry.path();
                let canonical = canonicalize_or_display(&path);
                if managed_targets.contains(&canonical) {
                    continue;
                }
                let plugin_name = path
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .or_else(|| path.file_name().and_then(|name| name.to_str()))
                    .unwrap_or("unknown")
                    .to_owned();
                snapshots.push(PluginStateSnapshot {
                    plugin_name,
                    manifest_source: None,
                    manifest_version_constraint: None,
                    lock_version: None,
                    lock_checksum: None,
                    installed_path: Some(path.display().to_string()),
                    installed_checksum: installed_checksum(&path)?,
                    state: PluginStateKind::UnmanagedInstalledPlugin,
                    recommended_action: "leave-unmanaged".to_owned(),
                    last_activity_id: None,
                });
            }
        }

        snapshots.sort_by(|left, right| left.plugin_name.cmp(&right.plugin_name));
        Ok(DiffResult {
            meta: session.meta(),
            snapshots,
        })
    }

    pub fn remove(&self, plugin_name: &str, keep_manifest: bool) -> Result<RemoveResult> {
        let mut session = OperationSession::new(
            &self.state_dir,
            "remove",
            format!("plugin={plugin_name}, keep_manifest={keep_manifest}"),
            vec![plugin_name.to_owned()],
            true,
        );
        let result = (|| {
            let mut manifest = Manifest::load_optional(&self.state_dir)?.unwrap_or_default();
            let mut lockfile = Lockfile::load_optional(&self.state_dir)?.unwrap_or_default();
            let locked_index = lockfile.plugins.iter().position(|plugin| plugin.name == plugin_name);
            let locked = locked_index.map(|index| lockfile.plugins.remove(index));

            if !keep_manifest {
                manifest.plugins.remove(plugin_name);
                manifest.save(&self.state_dir)?;
            }

            let removed_target_path = if let Some(locked) = &locked {
                session.emit(Some(plugin_name), "cleanup", "Removing managed plugin target.", None, None, ProgressSeverity::Info);
                let path = PathBuf::from(&locked.target_path);
                if path.exists() {
                    remove_path(&path)?;
                    Some(path.display().to_string())
                } else {
                    None
                }
            } else {
                None
            };

            lockfile.save(&self.state_dir)?;
            let (meta, completion) = session.success(
                format!("Removed managed plugin `{plugin_name}`."),
                1,
                0,
                0,
            )?;
            Ok(RemoveResult {
                meta,
                removed: RemovedPluginSummary {
                    name: plugin_name.to_owned(),
                    keep_manifest,
                    removed_manifest_entry: !keep_manifest,
                    removed_target_path,
                },
                progress: session.progress.clone(),
                completion,
            })
        })();
        if let Err(error) = &result {
            let _ = session.failure(error, Some("cleanup"));
        }
        result
    }

    pub fn reinstall(&self, plugin_name: &str) -> Result<InstallResult> {
        let manifest = Manifest::load_optional(&self.state_dir)?.unwrap_or_default();
        if manifest.plugins.contains_key(plugin_name) {
            return self.install_selected(vec![plugin_name.to_owned()], "reinstall", format!("plugin={plugin_name}"));
        }

        let lockfile = Lockfile::load(&self.state_dir)?;
        let locked = lockfile
            .plugins
            .iter()
            .find(|plugin| plugin.name == plugin_name)
            .cloned()
            .ok_or_else(|| PyanpmError::UnknownPlugin(plugin_name.to_owned()))?;
        self.install_locked_plugins(vec![locked], "reinstall", format!("plugin={plugin_name}"))
    }

    pub fn update(&self, plugin_name: Option<&str>, all: bool, dry_run: bool) -> Result<UpdateResult> {
        let manifest = Manifest::load(&self.state_dir)?;
        let lockfile = Lockfile::load_optional(&self.state_dir)?.unwrap_or_default();
        let target_names = if all {
            manifest.plugins.keys().cloned().collect::<Vec<_>>()
        } else if let Some(plugin_name) = plugin_name {
            vec![plugin_name.to_owned()]
        } else {
            return Err(PyanpmError::InvalidPluginRef(
                "provide a plugin name or use --all".to_owned(),
            ));
        };

        let session = OperationSession::new(
            &self.state_dir,
            "update",
            format!("plugin={:?}, all={all}, dry_run={dry_run}", plugin_name),
            target_names.clone(),
            true,
        );
        let result = (|| {
            let mut candidates = Vec::new();
            for name in &target_names {
                let spec = manifest
                    .plugins
                    .get(name)
                    .ok_or_else(|| PyanpmError::UnknownPlugin(name.clone()))?;
                let resolved = resolve_from_manifest(&self.state_dir, name, spec)?;
                let current = lockfile.plugins.iter().find(|plugin| plugin.name == *name);
                let will_change = current
                    .map(|locked| locked.checksum != resolved.checksum || locked.resolved_version != resolved.resolved_version)
                    .unwrap_or(true);
                candidates.push(UpdateCandidate {
                    plugin_name: name.clone(),
                    current_version: current.and_then(|locked| locked.resolved_version.clone()),
                    candidate_version: resolved.resolved_version.clone(),
                    current_checksum: current.map(|locked| locked.checksum.clone()),
                    candidate_checksum: resolved.checksum.clone(),
                    will_change,
                    reason: if will_change {
                        "source content or resolved version changed".to_owned()
                    } else {
                        "already up to date".to_owned()
                    },
                });
            }

            let changed_names = candidates
                .iter()
                .filter(|candidate| candidate.will_change)
                .map(|candidate| candidate.plugin_name.clone())
                .collect::<Vec<_>>();

            if dry_run {
                let (meta, completion) = session.success(
                    format!("Computed {} update candidate(s).", candidates.len()),
                    changed_names.len(),
                    0,
                    0,
                )?;
                return Ok(UpdateResult {
                    meta,
                    dry_run: true,
                    candidates,
                    updated: Vec::new(),
                    progress: session.progress.clone(),
                    completion,
                });
            }

            let install_result = if changed_names.is_empty() {
                None
            } else {
                Some(self.install_selected_with_command(changed_names.clone(), "update", "update changed plugins".to_owned())?)
            };

            let (meta, completion) = session.success(
                if changed_names.is_empty() {
                    "No updates were needed.".to_owned()
                } else {
                    format!("Updated {} plugin(s).", changed_names.len())
                },
                changed_names.len(),
                0,
                0,
            )?;
            Ok(UpdateResult {
                meta,
                dry_run: false,
                candidates,
                updated: install_result.map(|result| result.installed).unwrap_or_default(),
                progress: session.progress.clone(),
                completion,
            })
        })();
        if let Err(error) = &result {
            let _ = session.failure(error, Some("update"));
        }
        result
    }

    pub fn activity_list(&self) -> Result<ActivityListResult> {
        let session = OperationSession::new(&self.state_dir, "activity-list", String::new(), Vec::new(), false);
        Ok(ActivityListResult {
            meta: session.meta(),
            records: ActivityStore::new()?.list()?,
        })
    }

    pub fn activity_show(&self, activity_id: &str) -> Result<ActivityShowResult> {
        let session = OperationSession::new(
            &self.state_dir,
            "activity-show",
            format!("id={activity_id}"),
            Vec::new(),
            false,
        );
        Ok(ActivityShowResult {
            meta: session.meta(),
            record: ActivityStore::new()?.get(activity_id)?,
        })
    }

    pub fn activity_clear(&self) -> Result<ActivityClearResult> {
        let session = OperationSession::new(&self.state_dir, "activity-clear", String::new(), Vec::new(), false);
        Ok(ActivityClearResult {
            meta: session.meta(),
            cleared: ActivityStore::new()?.clear()?,
        })
    }

    pub fn cache_list(&self) -> Result<CacheListResult> {
        let session = OperationSession::new(&self.state_dir, "cache-list", String::new(), Vec::new(), false);
        let entries = CacheStore::new(None)?.list_entries(&self.active_lock_checksums()?)?;
        let total_size_bytes = entries.iter().map(|entry| entry.size_bytes).sum();
        Ok(CacheListResult {
            meta: session.meta(),
            total_size_bytes,
            entries,
        })
    }

    pub fn cache_info(&self, cache_id: &str) -> Result<CacheInfoResult> {
        let session = OperationSession::new(
            &self.state_dir,
            "cache-info",
            format!("cache_id={cache_id}"),
            Vec::new(),
            false,
        );
        Ok(CacheInfoResult {
            meta: session.meta(),
            entry: CacheStore::new(None)?.get_entry(cache_id, &self.active_lock_checksums()?)?,
        })
    }

    pub fn cache_evict(&self, cache_id: &str) -> Result<CacheMutationResult> {
        let session = OperationSession::new(
            &self.state_dir,
            "cache-evict",
            format!("cache_id={cache_id}"),
            Vec::new(),
            true,
        );
        let result = (|| {
            let active = self.active_lock_checksums()?;
            if active.iter().any(|checksum| checksum == cache_id) {
                return Err(PyanpmError::ProtectedCacheEntry(cache_id.to_owned()));
            }
            let reclaimed_size_bytes = CacheStore::new(None)?.evict_entry(cache_id)?;
            let (meta, completion) = session.success(
                format!("Evicted cache entry `{cache_id}`."),
                0,
                0,
                0,
            )?;
            Ok(CacheMutationResult {
                meta,
                removed: vec![cache_id.to_owned()],
                protected: Vec::new(),
                reclaimed_size_bytes,
                completion,
            })
        })();
        if let Err(error) = &result {
            let _ = session.failure(error, Some("cache"));
        }
        result
    }

    pub fn cache_prune(&self) -> Result<CacheMutationResult> {
        let session = OperationSession::new(&self.state_dir, "cache-prune", String::new(), Vec::new(), true);
        let result = (|| {
            let active = self.active_lock_checksums()?;
            let (removed, reclaimed_size_bytes) = CacheStore::new(None)?.prune_unreferenced(&active)?;
            let (meta, completion) = session.success(
                format!("Pruned {} cache entrie(s).", removed.len()),
                0,
                0,
                0,
            )?;
            Ok(CacheMutationResult {
                meta,
                removed,
                protected: Vec::new(),
                reclaimed_size_bytes,
                completion,
            })
        })();
        if let Err(error) = &result {
            let _ = session.failure(error, Some("cache"));
        }
        result
    }

    fn install_selected(&self, names: Vec<String>, command_name: &str, args_summary: String) -> Result<InstallResult> {
        self.install_selected_with_command(names, command_name, args_summary)
    }

    fn install_selected_with_command(
        &self,
        names: Vec<String>,
        command_name: &str,
        args_summary: String,
    ) -> Result<InstallResult> {
        let manifest = Manifest::load(&self.state_dir)?;
        let specs = names
            .into_iter()
            .map(|name| {
                manifest
                    .plugins
                    .get(&name)
                    .cloned()
                    .map(|spec| (name.clone(), spec))
                    .ok_or_else(|| PyanpmError::UnknownPlugin(name))
            })
            .collect::<Result<Vec<_>>>()?;
        self.install_manifest_specs(specs, command_name, args_summary, true)
    }

    fn install_locked_plugins(
        &self,
        plugins: Vec<LockedPlugin>,
        command_name: &str,
        args_summary: String,
    ) -> Result<InstallResult> {
        let mut session = OperationSession::new(
            &self.state_dir,
            command_name,
            args_summary,
            plugins.iter().map(|plugin| plugin.name.clone()).collect(),
            true,
        );
        let result = (|| {
            let cache = CacheStore::new(None)?;
            let plugins_dir = resolve_plugins_dir(self.plugins_dir_override.as_deref())?;
            let mut lockfile = Lockfile::load_optional(&self.state_dir)?.unwrap_or_default();
            let mut installed = Vec::new();

            for (index, locked) in plugins.iter().enumerate() {
                session.emit(
                    Some(&locked.name),
                    "resolve",
                    "Resolving plugin from lockfile source.",
                    Some((index + 1) as u32),
                    Some(plugins.len() as u32),
                    ProgressSeverity::Info,
                );
                let resolved = resolve_from_lock(&self.state_dir, locked)?;
                session.emit(Some(&locked.name), "fetch", "Caching resolved artifact.", None, None, ProgressSeverity::Info);
                let cached = cache.cache_artifact(&resolved)?;
                session.emit(Some(&locked.name), "install", "Installing cached artifact.", None, None, ProgressSeverity::Info);
                let target_path = install_cached_artifact(&plugins_dir, &resolved, &cached)?;
                upsert_locked_plugin(&mut lockfile, build_locked_plugin(&resolved, &cached, &target_path));
                installed.push(InstalledPluginSummary {
                    name: resolved.name.clone(),
                    source: format!("{:?}", resolved.source).to_ascii_lowercase(),
                    checksum: cached.checksum.clone(),
                    target_path: target_path.display().to_string(),
                });
            }

            lockfile.save(&self.state_dir)?;
            let (meta, completion) = session.success(
                format!("Installed {} plugin(s).", installed.len()),
                installed.len(),
                0,
                0,
            )?;
            Ok(InstallResult {
                meta,
                installed,
                lockfile_path: Lockfile::path_in(&self.state_dir).display().to_string(),
                plugins_dir: plugins_dir.display().to_string(),
                progress: session.progress.clone(),
                completion,
            })
        })();
        if let Err(error) = &result {
            let _ = session.failure(error, Some("install"));
        }
        result
    }

    fn install_manifest_specs(
        &self,
        specs: Vec<(String, ManifestPluginSpec)>,
        command_name: &str,
        args_summary: String,
        persist_activity: bool,
    ) -> Result<InstallResult> {
        let mut session = OperationSession::new(
            &self.state_dir,
            command_name,
            args_summary,
            specs.iter().map(|(name, _)| name.clone()).collect(),
            persist_activity,
        );
        let result = (|| {
            let cache = CacheStore::new(None)?;
            let plugins_dir = resolve_plugins_dir(self.plugins_dir_override.as_deref())?;
            let mut lockfile = Lockfile::load_optional(&self.state_dir)?.unwrap_or_default();
            let mut installed = Vec::new();

            for (index, (name, spec)) in specs.iter().enumerate() {
                session.emit(
                    Some(name),
                    "resolve",
                    "Resolving plugin source.",
                    Some((index + 1) as u32),
                    Some(specs.len() as u32),
                    ProgressSeverity::Info,
                );
                let resolved = resolve_from_manifest(&self.state_dir, name, spec)?;
                session.emit(Some(name), "checksum", "Computing artifact checksum.", None, None, ProgressSeverity::Info);
                session.emit(Some(name), "fetch", "Caching artifact.", None, None, ProgressSeverity::Info);
                let cached = cache.cache_artifact(&resolved)?;
                session.emit(Some(name), "install", "Installing cached artifact.", None, None, ProgressSeverity::Info);
                let target_path = install_cached_artifact(&plugins_dir, &resolved, &cached)?;
                upsert_locked_plugin(&mut lockfile, build_locked_plugin(&resolved, &cached, &target_path));
                installed.push(InstalledPluginSummary {
                    name: resolved.name.clone(),
                    source: format!("{:?}", resolved.source).to_ascii_lowercase(),
                    checksum: cached.checksum.clone(),
                    target_path: target_path.display().to_string(),
                });
            }

            lockfile.save(&self.state_dir)?;
            let (meta, completion) = session.success(
                format!("Installed {} plugin(s).", installed.len()),
                installed.len(),
                0,
                0,
            )?;
            Ok(InstallResult {
                meta,
                installed,
                lockfile_path: Lockfile::path_in(&self.state_dir).display().to_string(),
                plugins_dir: plugins_dir.display().to_string(),
                progress: session.progress.clone(),
                completion,
            })
        })();
        if let Err(error) = &result {
            let _ = session.failure(error, Some("install"));
        }
        result
    }

    fn active_lock_checksums(&self) -> Result<Vec<String>> {
        Ok(Lockfile::load_optional(&self.state_dir)?
            .unwrap_or_default()
            .plugins
            .into_iter()
            .map(|plugin| plugin.checksum)
            .collect())
    }
}

impl<T> JsonEnvelope<T>
where
    T: Serialize,
{
    pub fn success(command: &'static str, data: T) -> Self {
        Self {
            api_version: API_VERSION,
            command,
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn failure(command: &'static str, error: &crate::error::PyanpmError) -> Self {
        Self {
            api_version: API_VERSION,
            command,
            success: false,
            data: None,
            error: Some(error.to_string()),
        }
    }
}

pub fn missing_manifest_message(state_dir: &Path) -> String {
    format!(
        "No global {MANIFEST_FILE_NAME} found in {}. Run `pyanpm init` first.",
        state_dir.display()
    )
}

fn build_locked_plugin(
    resolved: &crate::plugin::ResolvedPlugin,
    cached: &crate::cache::CachedArtifact,
    target_path: &Path,
) -> LockedPlugin {
    LockedPlugin {
        name: resolved.name.clone(),
        requested_version: resolved.requested_version.clone(),
        resolved_version: resolved.resolved_version.clone(),
        source: resolved.source,
        source_path: resolved.source_path.display().to_string(),
        source_url: resolved.source_url.clone(),
        source_subdir: resolved.source_subdir.clone(),
        git_ref_kind: resolved.git_ref_kind,
        git_requested_ref: resolved.git_requested_ref.clone(),
        git_resolved_commit: resolved.git_resolved_commit.clone(),
        checksum: cached.checksum.clone(),
        artifact_kind: cached.artifact_kind,
        target_path: target_path.display().to_string(),
        installed_at: Utc::now(),
    }
}

fn upsert_locked_plugin(lockfile: &mut Lockfile, plugin: LockedPlugin) {
    if let Some(existing) = lockfile.plugins.iter_mut().find(|existing| existing.name == plugin.name) {
        *existing = plugin;
    } else {
        lockfile.plugins.push(plugin);
    }
}

fn installed_checksum(path: &Path) -> Result<Option<String>> {
    if !path.exists() {
        return Ok(None);
    }
    Ok(Some(checksum_path(path)?))
}

fn remove_path(path: &Path) -> Result<()> {
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

fn canonicalize_or_display(path: &Path) -> String {
    path.canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
        .display()
        .to_string()
}

fn error_code(error: &PyanpmError) -> String {
    match error {
        PyanpmError::ManifestExists(_) => "manifest_exists",
        PyanpmError::ManifestMissing(_) => "manifest_missing",
        PyanpmError::InvalidPluginRef(_) => "invalid_plugin_ref",
        PyanpmError::UnsupportedSource(_) => "unsupported_source",
        PyanpmError::PluginAlreadyExists(_) => "plugin_exists",
        PyanpmError::UnknownPlugin(_) => "unknown_plugin",
        PyanpmError::InvalidPluginName(_) => "invalid_plugin_name",
        PyanpmError::UnsupportedArtifact(_) => "unsupported_artifact",
        PyanpmError::MissingMetadata(_) => "missing_metadata",
        PyanpmError::InvalidMetadata(_) => "invalid_metadata",
        PyanpmError::MissingGitExecutable => "missing_git_executable",
        PyanpmError::GitCloneFailed { .. } => "git_clone_failed",
        PyanpmError::GitFetchFailed { .. } => "git_fetch_failed",
        PyanpmError::GitCheckoutFailed { .. } => "git_checkout_failed",
        PyanpmError::InvalidGitUrl(_) => "invalid_git_url",
        PyanpmError::InvalidGitRef(_) => "invalid_git_ref",
        PyanpmError::InvalidGitSubdir(_) => "invalid_git_subdir",
        PyanpmError::InstallTargetEscapes(_) => "install_target_escapes",
        PyanpmError::MissingDefaultCacheDir => "missing_cache_dir",
        PyanpmError::MissingDefaultConfigDir => "missing_config_dir",
        PyanpmError::MissingDefaultPluginsDir => "missing_plugins_dir",
        PyanpmError::ActivityNotFound(_) => "activity_not_found",
        PyanpmError::CacheEntryNotFound(_) => "cache_entry_not_found",
        PyanpmError::ProtectedCacheEntry(_) => "cache_entry_protected",
        PyanpmError::ConfirmationRequired(_) => "confirmation_required",
        PyanpmError::Io(_) => "io_error",
        PyanpmError::LockfileParse(_) => "parse_error",
        PyanpmError::Serialization(_) => "serialization_error",
        PyanpmError::Json(_) => "json_error",
    }
    .to_owned()
}

fn format_source_args(source_ref: &str, git_options: &GitSourceOptions) -> String {
    let mut parts = vec![format!("source={source_ref}")];
    if let Some(kind) = git_options.git_ref_kind {
        let kind = match kind {
            GitRefKind::Branch => "branch",
            GitRefKind::Tag => "tag",
            GitRefKind::Commit => "commit",
        };
        parts.push(format!("git_ref_kind={kind}"));
    }
    if let Some(reference) = git_options.git_ref.as_deref().filter(|value| !value.trim().is_empty()) {
        parts.push(format!("git_ref={reference}"));
    }
    if let Some(subdir) = git_options.git_subdir.as_deref().filter(|value| !value.trim().is_empty()) {
        parts.push(format!("git_subdir={subdir}"));
    }
    parts.join(", ")
}
