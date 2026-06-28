use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandMeta {
    pub command_id: String,
    pub state_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activity_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletionSummary {
    pub affected_plugin_count: usize,
    pub warning_count: usize,
    pub error_count: usize,
    pub duration_ms: i64,
    pub summary: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProgressSeverity {
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressEvent {
    pub command_id: String,
    pub state_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugin_name: Option<String>,
    pub stage: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<u32>,
    pub severity: ProgressSeverity,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ValidationSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationIssue {
    pub field: String,
    pub source_ref: String,
    pub code: String,
    pub severity: ValidationSeverity,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggested_fix: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidateSourceResult {
    pub meta: CommandMeta,
    pub valid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub normalized_source_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugin_name: Option<String>,
    #[serde(default)]
    pub errors: Vec<ValidationIssue>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityStatus {
    Success,
    Failed,
    Partial,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityRecord {
    pub id: String,
    pub command_id: String,
    pub state_path: String,
    pub command_name: String,
    pub command_args_summary: String,
    pub started_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
    pub duration_ms: i64,
    pub status: ActivityStatus,
    #[serde(default)]
    pub plugin_names: Vec<String>,
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_stage: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityListResult {
    pub meta: CommandMeta,
    pub records: Vec<ActivityRecord>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityShowResult {
    pub meta: CommandMeta,
    pub record: ActivityRecord,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityClearResult {
    pub meta: CommandMeta,
    pub cleared: usize,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginStateKind {
    Synced,
    ManifestOnly,
    LockOnly,
    MissingInstall,
    StaleInstall,
    ChecksumDrift,
    SourceUnreadable,
    UnmanagedInstalledPlugin,
    TargetWriteRisk,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginStateSnapshot {
    pub plugin_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manifest_source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manifest_version_constraint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lock_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lock_checksum: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub installed_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub installed_checksum: Option<String>,
    pub state: PluginStateKind,
    pub recommended_action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_activity_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffResult {
    pub meta: CommandMeta,
    pub snapshots: Vec<PluginStateSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheEntry {
    pub cache_id: String,
    pub plugin_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    pub source_kind: String,
    pub source_summary: String,
    pub checksum: String,
    pub size_bytes: u64,
    pub created_at: DateTime<Utc>,
    pub last_used_at: DateTime<Utc>,
    pub referenced_by_active_lockfile: bool,
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheListResult {
    pub meta: CommandMeta,
    pub total_size_bytes: u64,
    pub entries: Vec<CacheEntry>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheInfoResult {
    pub meta: CommandMeta,
    pub entry: CacheEntry,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheMutationResult {
    pub meta: CommandMeta,
    pub removed: Vec<String>,
    pub protected: Vec<String>,
    pub reclaimed_size_bytes: u64,
    pub completion: CompletionSummary,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemovedPluginSummary {
    pub name: String,
    pub keep_manifest: bool,
    pub removed_manifest_entry: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub removed_target_path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveResult {
    pub meta: CommandMeta,
    pub removed: RemovedPluginSummary,
    #[serde(default)]
    pub progress: Vec<ProgressEvent>,
    pub completion: CompletionSummary,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledPluginSummary {
    pub name: String,
    pub source: String,
    pub checksum: String,
    pub target_path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallResult {
    pub meta: CommandMeta,
    pub installed: Vec<InstalledPluginSummary>,
    pub lockfile_path: String,
    pub plugins_dir: String,
    #[serde(default)]
    pub progress: Vec<ProgressEvent>,
    pub completion: CompletionSummary,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InitResult {
    pub meta: CommandMeta,
    pub manifest_path: String,
    pub completion: CompletionSummary,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListedPlugin {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requested_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub installed_version: Option<String>,
    pub source: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListResult {
    pub meta: CommandMeta,
    pub plugins: Vec<ListedPlugin>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCandidate {
    pub plugin_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub candidate_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_checksum: Option<String>,
    pub candidate_checksum: String,
    pub will_change: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateResult {
    pub meta: CommandMeta,
    pub dry_run: bool,
    pub candidates: Vec<UpdateCandidate>,
    pub updated: Vec<InstalledPluginSummary>,
    #[serde(default)]
    pub progress: Vec<ProgressEvent>,
    pub completion: CompletionSummary,
}
