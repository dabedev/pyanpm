use serde::Deserialize;
use tauri::State;

use crate::state::AppState;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddPluginInput {
    pub plugin_ref: String,
    pub version: Option<String>,
    #[serde(default)]
    pub git_ref_kind: Option<pyanpm_core::GitRefKind>,
    #[serde(default)]
    pub git_ref: Option<String>,
    #[serde(default)]
    pub git_subdir: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidateSourceInput {
    pub source_ref: String,
    #[serde(default)]
    pub git_ref_kind: Option<pyanpm_core::GitRefKind>,
    #[serde(default)]
    pub git_ref: Option<String>,
    #[serde(default)]
    pub git_subdir: Option<String>,
}

impl AddPluginInput {
    fn git_options(&self) -> pyanpm_core::GitSourceOptions {
        pyanpm_core::GitSourceOptions {
            git_ref_kind: self.git_ref_kind,
            git_ref: self.git_ref.clone(),
            git_subdir: self.git_subdir.clone(),
        }
    }
}

impl ValidateSourceInput {
    fn git_options(&self) -> pyanpm_core::GitSourceOptions {
        pyanpm_core::GitSourceOptions {
            git_ref_kind: self.git_ref_kind,
            git_ref: self.git_ref.clone(),
            git_subdir: self.git_subdir.clone(),
        }
    }
}

#[tauri::command]
pub fn init_manifest(state: State<'_, AppState>, force: bool) -> Result<pyanpm_core::InitResult, String> {
    state.service().init(force).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn list_plugins(state: State<'_, AppState>) -> Result<pyanpm_core::ListResult, String> {
    state.service().list().map_err(|error| error.to_string())
}

#[tauri::command]
pub fn add_plugin(
    state: State<'_, AppState>,
    input: AddPluginInput,
) -> Result<pyanpm_core::InstallResult, String> {
    state
        .service()
        .add(&input.plugin_ref, input.version.clone(), input.git_options())
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn install_plugins(state: State<'_, AppState>) -> Result<pyanpm_core::InstallResult, String> {
    state.service().install().map_err(|error| error.to_string())
}

#[tauri::command]
pub fn validate_source(
    state: State<'_, AppState>,
    input: ValidateSourceInput,
) -> Result<pyanpm_core::ValidateSourceResult, String> {
    state
        .service()
        .validate_source(&input.source_ref, input.git_options())
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn diff_plugins(state: State<'_, AppState>) -> Result<pyanpm_core::DiffResult, String> {
    state.service().diff().map_err(|error| error.to_string())
}

#[tauri::command]
pub fn remove_plugin(
    state: State<'_, AppState>,
    plugin_name: String,
    keep_manifest: bool,
) -> Result<pyanpm_core::RemoveResult, String> {
    state
        .service()
        .remove(&plugin_name, keep_manifest)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn reinstall_plugin(
    state: State<'_, AppState>,
    plugin_name: String,
) -> Result<pyanpm_core::InstallResult, String> {
    state
        .service()
        .reinstall(&plugin_name)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn update_plugins(
    state: State<'_, AppState>,
    plugin_name: Option<String>,
    all: bool,
    dry_run: bool,
) -> Result<pyanpm_core::UpdateResult, String> {
    state
        .service()
        .update(plugin_name.as_deref(), all, dry_run)
        .map_err(|error| error.to_string())
}
