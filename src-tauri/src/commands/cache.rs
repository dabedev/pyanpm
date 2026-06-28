use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub fn list_cache(state: State<'_, AppState>) -> Result<pyanpm_core::CacheListResult, String> {
    state.service().cache_list().map_err(|error| error.to_string())
}

#[tauri::command]
pub fn evict_cache(state: State<'_, AppState>, cache_id: String) -> Result<pyanpm_core::CacheMutationResult, String> {
    state
        .service()
        .cache_evict(&cache_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn prune_cache(state: State<'_, AppState>) -> Result<pyanpm_core::CacheMutationResult, String> {
    state.service().cache_prune().map_err(|error| error.to_string())
}
